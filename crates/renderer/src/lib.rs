//! wgpu-based renderer for zelda3-rs.
//!
//! Two renderers share the same blit pipeline:
//! - [`FrameRenderer`]: renders to a winit window surface (live display).
//! - [`OffscreenRenderer`]: renders to an offscreen texture and reads pixels
//!   back to CPU (headless replay, `--render-hash-log`, `--dump-frame`).
//!
//! Phase 1a — GPU tile atlas infrastructure:
//! - [`GpuFrame`]: zero-copy data bundle from `PpuState` to the renderer.
//! - [`TileAtlas`]: GPU texture of decoded 4bpp tile palette indices (512×256).
//! - [`CgramPalette`]: GPU texture of decoded CGRAM colours (256×1 RGBA).
//!
//! Phase 1b — BG layer rendering:
//! - [`BgLayerRenderer`]: single-layer BG pipeline (tilemap → atlas → CGRAM).

mod frame_compare;
pub mod gpu_frame;
mod gpu_work_item;
pub mod hd_authoring;
pub mod modern_assets;
mod modern_bg_renderer;
pub mod modern_dungeon_atlas;
pub mod modern_extract;
mod modern_finalizer;
pub mod modern_frame;
pub mod modern_gpu;
mod modern_gpu_work_command;
pub mod modern_hd_overrides;
pub mod modern_index_atlas;
mod modern_index_compare_stats;
mod modern_index_renderer;
mod modern_live_stats;
mod modern_mode1_effect_plan;
pub mod modern_palette;
mod modern_screen_builder;
pub mod modern_software;
pub mod modern_source_atlas;
pub mod modern_sprite_atlas;
#[cfg(test)]
mod modern_sprite_renderer;
pub mod modern_variant_atlas;
pub mod modern_variant_draw;
mod modern_variant_render_plan;
pub mod renderer_mode;

pub use frame_compare::{
    compare_gpu_render_frame_bgra_to_rgba, gpu_render_hash_frame_rgba,
    render_fingerprint_leaf_bgra, render_hash_frame_bgra, render_hash_frame_rgba,
    render_hash_pair_bgra_rgba, GpuRenderComparison, GpuRenderDiff, GpuRenderFrameComparison,
    RenderFrameHashReport, RenderHashPair,
};
pub use gpu_frame::{
    BgLayerRegs, GpuBg3SourceTile, GpuBg3VwfGlyphRun, GpuFrame, GpuFrameCaptureInput,
    GpuFrameRegisterSnapshot, GpuFrameSource, Mode7Regs, ObjRegs, RawScanlineFrame,
    RawScanlineRegs, ScanlineRegs,
};
pub use modern_gpu::{
    ModernGpuCompositor, ModernGpuHeadless, ModernGpuVariantHeadless, ModernGpuVariantRenderer,
};
pub use modern_index_compare_stats::{
    ModernIndexCompareFrameOutputInput, ModernIndexCompareFrameReport,
    ModernIndexCompareOutputLine, ModernIndexCompareOutputLines, ModernIndexCompareOutputStream,
    ModernIndexCompareRunConfig, ModernIndexCompareRunConfigError, ModernIndexCompareStats,
};
pub use modern_live_stats::{ModernAssetLiveFrameReport, ModernAssetLiveStats};
pub use renderer_mode::{
    default_renderer_env_for_variant_setting, renderer_env_or_default, source_atlas_renderer_mode,
    variant_atlas_renderer_mode, EffectiveRendererMode, RendererMode,
};

pub fn source_table_from_entries<'a, T>(
    entries: &'a [T],
) -> impl modern_extract::SourceTableView + 'a
where
    T: Copy + Into<(u8, u16, u16)> + 'a,
{
    modern_extract::MappedSourceTableView::from_entries(entries)
}
use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
    path::Path,
    sync::Arc,
};

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// ── Viewport ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct Viewport {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewportScaleMode {
    Integer,
    Fit,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererViewportChoice {
    Integer,
    Fit,
    Stretch,
}

impl ViewportScaleMode {
    fn from_env() -> Self {
        let value = env::var("ZELDA3_VIEWPORT_SCALE")
            .ok()
            .or_else(|| env::var("ZELDA3_SCALE_MODE").ok());

        match value.map(|s| s.to_ascii_lowercase()) {
            Some(value) if matches!(value.as_str(), "fit" | "aspect-fit" | "aspect_fit") => {
                Self::Fit
            }
            Some(value) if matches!(value.as_str(), "stretch" | "fullscreen") => Self::Stretch,
            Some(value) if matches!(value.as_str(), "integer" | "pixel" | "pixel-perfect") => {
                Self::Integer
            }
            Some(_) => Self::Integer,
            None if env::var_os("ZELDA3_STEAMDECK").is_some() => Self::Fit,
            None => Self::Integer,
        }
    }

    fn from_runtime_choice(choice: RendererViewportChoice) -> Self {
        match choice {
            RendererViewportChoice::Integer => Self::Integer,
            RendererViewportChoice::Fit => Self::Fit,
            RendererViewportChoice::Stretch => Self::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum PresentationMode {
    Off,
    Sharp,
    Crt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererPresentationChoice {
    Off,
    Sharp,
    Crt,
}

impl PresentationMode {
    fn from_env() -> Self {
        Self::from_value(env::var("ZELDA3_PRESENTATION").ok().as_deref())
    }

    fn from_value(value: Option<&str>) -> Self {
        match value.map(|s| s.to_ascii_lowercase()) {
            Some(value) if matches!(value.as_str(), "sharp" | "sharp-bilinear") => Self::Sharp,
            Some(value) if matches!(value.as_str(), "crt" | "scanline" | "scanlines") => Self::Crt,
            Some(value) if matches!(value.as_str(), "off" | "none" | "nearest") => Self::Off,
            Some(_) | None => Self::Off,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Off => Self::Sharp,
            Self::Sharp => Self::Crt,
            Self::Crt => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum LightingMode {
    Off,
    Ambient,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererLightingChoice {
    Off,
    Ambient,
    Dynamic,
}

impl LightingMode {
    fn from_env() -> Self {
        Self::from_value(env::var("ZELDA3_LIGHTING").ok().as_deref())
    }

    fn from_value(value: Option<&str>) -> Self {
        match value.map(|s| s.to_ascii_lowercase()) {
            Some(value) if value == "ambient" => Self::Ambient,
            Some(value) if value == "dynamic" => Self::Dynamic,
            Some(value) if matches!(value.as_str(), "off" | "none") => Self::Off,
            Some(_) | None => Self::Off,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Off => Self::Ambient,
            Self::Ambient => Self::Dynamic,
            Self::Dynamic => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum ShadowMode {
    Off = 0,
    Raycast = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererShadowChoice {
    Off,
    Raycast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererRuntimeSettings {
    pub presentation: RendererPresentationChoice,
    pub lighting: RendererLightingChoice,
    pub shadows: RendererShadowChoice,
    pub viewport: RendererViewportChoice,
}

impl ShadowMode {
    fn from_env() -> Self {
        Self::from_value(env::var("ZELDA3_SHADOWS").ok().as_deref())
    }

    fn from_value(value: Option<&str>) -> Self {
        match value.map(|s| s.to_ascii_lowercase()) {
            Some(value) if value == "raycast" => Self::Raycast,
            Some(value) if matches!(value.as_str(), "off" | "none") => Self::Off,
            Some(_) | None => Self::Off,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Off => Self::Raycast,
            Self::Raycast => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PresentationParams {
    presentation: PresentationMode,
    lighting: LightingMode,
    shadows: ShadowMode,
}

impl PresentationParams {
    fn from_env() -> Self {
        Self::new(
            PresentationMode::from_env(),
            LightingMode::from_env(),
            ShadowMode::from_env(),
        )
    }

    fn new(presentation: PresentationMode, lighting: LightingMode, shadows: ShadowMode) -> Self {
        Self {
            presentation,
            lighting,
            shadows,
        }
    }

    fn from_runtime_settings(settings: RendererRuntimeSettings) -> Self {
        Self::new(
            match settings.presentation {
                RendererPresentationChoice::Off => PresentationMode::Off,
                RendererPresentationChoice::Sharp => PresentationMode::Sharp,
                RendererPresentationChoice::Crt => PresentationMode::Crt,
            },
            match settings.lighting {
                RendererLightingChoice::Off => LightingMode::Off,
                RendererLightingChoice::Ambient => LightingMode::Ambient,
                RendererLightingChoice::Dynamic => LightingMode::Dynamic,
            },
            match settings.shadows {
                RendererShadowChoice::Off => ShadowMode::Off,
                RendererShadowChoice::Raycast => ShadowMode::Raycast,
            },
        )
    }

    fn cycle_presentation(&mut self) {
        self.presentation = self.presentation.next();
    }

    fn cycle_lighting(&mut self) {
        self.lighting = self.lighting.next();
    }

    fn cycle_shadows(&mut self) {
        self.shadows = self.shadows.next();
    }

    #[cfg(test)]
    fn as_words(&self) -> [u32; 4] {
        [
            self.presentation as u32,
            self.lighting as u32,
            self.shadows as u32,
            0,
        ]
    }
}

const PRESENTATION_NOTICE_FRAMES: u32 = 90;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct PresentationNotice {
    code: u32,
    frames_remaining: u32,
}

impl PresentationNotice {
    fn show_presentation(&mut self, mode: PresentationMode) {
        self.show(match mode {
            PresentationMode::Off => 1,
            PresentationMode::Sharp => 2,
            PresentationMode::Crt => 3,
        });
    }

    fn show_lighting(&mut self, mode: LightingMode) {
        self.show(match mode {
            LightingMode::Off => 10,
            LightingMode::Ambient => 11,
            LightingMode::Dynamic => 12,
        });
    }

    fn show_shadows(&mut self, mode: ShadowMode) {
        self.show(match mode {
            ShadowMode::Off => 20,
            ShadowMode::Raycast => 22,
        });
    }

    fn show_viewport(&mut self, mode: ViewportScaleMode) {
        self.show(match mode {
            ViewportScaleMode::Integer => 30,
            ViewportScaleMode::Fit => 31,
            ViewportScaleMode::Stretch => 32,
        });
    }

    fn show(&mut self, code: u32) {
        self.code = code;
        self.frames_remaining = PRESENTATION_NOTICE_FRAMES;
    }

    fn tick(&mut self) {
        self.frames_remaining = self.frames_remaining.saturating_sub(1);
        if self.frames_remaining == 0 {
            self.code = 0;
        }
    }

    fn code(&self) -> u32 {
        if self.frames_remaining == 0 {
            0
        } else {
            self.code
        }
    }

    fn frames_remaining(&self) -> u32 {
        self.frames_remaining
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuOverlayTab {
    Play,
    Video,
    Controls,
    DeveloperMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuOverlayThumbnail {
    RouteStart,
    FileSelect,
    Sanctuary,
    LateDungeon,
    DevRoom,
    LockedOverworld,
    LockedDungeon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuOverlayModel {
    pub tab: MenuOverlayTab,
    pub selected_index: usize,
    pub lines: Vec<&'static str>,
    pub detail_lines: Vec<String>,
    pub thumbnail: Option<MenuOverlayThumbnail>,
}

impl MenuOverlayModel {
    fn resume_first_play_tab() -> Self {
        Self {
            tab: MenuOverlayTab::Play,
            selected_index: 0,
            lines: vec![
                "PLAY  VIDEO  CONTROLS  DEV MAP",
                "> RESUME QUEST",
                "  VIDEO & EFFECTS",
                "  CONTROLS",
                "DEVELOPER MAP",
                "  SAVE & QUIT",
            ],
            detail_lines: Vec::new(),
            thumbnail: None,
        }
    }
}

fn menu_overlay_lines(menu: &MenuOverlayModel) -> Vec<&'static str> {
    if !menu.lines.is_empty() {
        return menu.lines.clone();
    }
    match menu.tab {
        MenuOverlayTab::Play => MenuOverlayModel::resume_first_play_tab().lines,
        MenuOverlayTab::Video => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> PRESENTATION",
            "  LIGHTING",
            "  SHADOWS",
            "  VIEWPORT",
        ],
        MenuOverlayTab::Controls => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> KEYBOARD",
            "  GAMEPAD",
            "  RESET DEFAULTS",
        ],
        MenuOverlayTab::DeveloperMap => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> CURATED PRESETS",
            "  ROUTE BOOKMARKS",
            "  LOCKED BROWSER",
        ],
    }
}

const MENU_COLOR_BACKDROP: u32 = 0xff050713;
const MENU_COLOR_PANEL: u32 = 0xff101629;
const MENU_COLOR_PANEL_DARK: u32 = 0xff080b18;
const MENU_COLOR_BORDER: u32 = 0xffd6b45a;
const MENU_COLOR_BORDER_SHADOW: u32 = 0xff5a3d1a;
const MENU_COLOR_TEXT: u32 = 0xfffff0b8;
const MENU_COLOR_TEXT_DIM: u32 = 0xffb9a56c;
const MENU_COLOR_THUMB_STONE: u32 = 0xff81746a;
const MENU_COLOR_THUMB_LIGHT: u32 = 0xffffd66d;
const MENU_COLOR_THUMB_GRASS: u32 = 0xff2f7d3b;
const MENU_COLOR_THUMB_LOCKED: u32 = 0xff383f4f;

fn build_menu_overlay_pixels(menu: &MenuOverlayModel, width: u32, height: u32) -> Vec<u32> {
    let mut pixels = vec![MENU_COLOR_BACKDROP; width.saturating_mul(height) as usize];
    if width == 0 || height == 0 {
        return pixels;
    }

    draw_rect(
        &mut pixels,
        width,
        height,
        18,
        22,
        220,
        178,
        MENU_COLOR_PANEL,
    );
    draw_rect(
        &mut pixels,
        width,
        height,
        22,
        26,
        212,
        170,
        MENU_COLOR_PANEL_DARK,
    );
    draw_border(&mut pixels, width, height, 18, 22, 220, 178);
    draw_border(&mut pixels, width, height, 22, 26, 212, 170);
    draw_rect(&mut pixels, width, height, 30, 38, 196, 18, 0xff1b2338);

    let lines = menu_overlay_lines(menu);
    if let Some(header) = lines.first() {
        draw_text(
            &mut pixels,
            width,
            height,
            38,
            44,
            header,
            1,
            MENU_COLOR_TEXT_DIM,
        );
    }
    for (line_index, line) in lines.iter().skip(1).enumerate() {
        let y = 74 + line_index as i32 * 22;
        let color = if line.starts_with('>') {
            let highlight_width = if menu.thumbnail.is_some() || !menu.detail_lines.is_empty() {
                90
            } else {
                188
            };
            draw_rect(
                &mut pixels,
                width,
                height,
                34,
                y - 4,
                highlight_width,
                17,
                0xff202a43,
            );
            MENU_COLOR_TEXT
        } else {
            MENU_COLOR_TEXT_DIM
        };
        draw_text(&mut pixels, width, height, 42, y, line, 1, color);
    }

    if menu.thumbnail.is_some() || !menu.detail_lines.is_empty() {
        draw_developer_detail_panel(&mut pixels, width, height, menu);
    }

    pixels
}

fn draw_developer_detail_panel(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    menu: &MenuOverlayModel,
) {
    draw_rect(pixels, width, height, 136, 68, 86, 106, 0xff141c31);
    draw_border(pixels, width, height, 136, 68, 86, 106);
    if let Some(thumbnail) = menu.thumbnail {
        draw_thumbnail(pixels, width, height, 144, 78, thumbnail);
    }
    for (index, line) in menu.detail_lines.iter().take(5).enumerate() {
        let y = 120 + index as i32 * 10;
        let color = if index == 0 {
            MENU_COLOR_TEXT
        } else {
            MENU_COLOR_TEXT_DIM
        };
        draw_text(pixels, width, height, 144, y, line, 1, color);
    }
}

fn draw_thumbnail(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    thumbnail: MenuOverlayThumbnail,
) {
    draw_rect(pixels, width, height, x, y, 56, 34, 0xff03050b);
    draw_border(pixels, width, height, x, y, 56, 34);
    match thumbnail {
        MenuOverlayThumbnail::RouteStart => {
            draw_rect(pixels, width, height, x + 3, y + 3, 50, 28, 0xff191f3a);
            draw_rect(
                pixels,
                width,
                height,
                x + 10,
                y + 18,
                36,
                6,
                MENU_COLOR_THUMB_GRASS,
            );
            draw_rect(
                pixels,
                width,
                height,
                x + 24,
                y + 8,
                8,
                18,
                MENU_COLOR_THUMB_LIGHT,
            );
        }
        MenuOverlayThumbnail::FileSelect => {
            draw_rect(pixels, width, height, x + 3, y + 3, 50, 28, 0xff10172a);
            draw_rect(
                pixels,
                width,
                height,
                x + 12,
                y + 8,
                32,
                4,
                MENU_COLOR_THUMB_LIGHT,
            );
            draw_rect(
                pixels,
                width,
                height,
                x + 12,
                y + 16,
                32,
                4,
                MENU_COLOR_THUMB_STONE,
            );
            draw_rect(
                pixels,
                width,
                height,
                x + 12,
                y + 24,
                32,
                4,
                MENU_COLOR_THUMB_STONE,
            );
        }
        MenuOverlayThumbnail::Sanctuary => {
            draw_rect(pixels, width, height, x + 3, y + 3, 50, 28, 0xff202437);
            draw_rect(
                pixels,
                width,
                height,
                x + 8,
                y + 18,
                40,
                10,
                MENU_COLOR_THUMB_STONE,
            );
            draw_rect(pixels, width, height, x + 16, y + 10, 24, 10, 0xff6a5a4c);
            draw_rect(
                pixels,
                width,
                height,
                x + 25,
                y + 7,
                6,
                21,
                MENU_COLOR_THUMB_LIGHT,
            );
        }
        MenuOverlayThumbnail::LateDungeon => {
            draw_rect(pixels, width, height, x + 3, y + 3, 50, 28, 0xff16101f);
            draw_rect(
                pixels,
                width,
                height,
                x + 8,
                y + 9,
                40,
                18,
                MENU_COLOR_THUMB_STONE,
            );
            draw_rect(pixels, width, height, x + 12, y + 13, 12, 10, 0xff5b2231);
            draw_rect(
                pixels,
                width,
                height,
                x + 32,
                y + 13,
                12,
                10,
                MENU_COLOR_THUMB_LIGHT,
            );
        }
        MenuOverlayThumbnail::DevRoom => {
            draw_rect(pixels, width, height, x + 3, y + 3, 50, 28, 0xff17232f);
            draw_rect(pixels, width, height, x + 6, y + 6, 44, 22, 0xff243240);
            draw_rect(pixels, width, height, x + 10, y + 10, 12, 8, 0xff4b8f68);
            draw_rect(pixels, width, height, x + 28, y + 10, 12, 8, 0xff9b4d5b);
            draw_rect(
                pixels,
                width,
                height,
                x + 18,
                y + 21,
                16,
                5,
                MENU_COLOR_THUMB_LIGHT,
            );
            draw_rect(pixels, width, height, x + 25, y + 8, 4, 20, 0xffd6b45a);
        }
        MenuOverlayThumbnail::LockedOverworld => {
            draw_rect(
                pixels,
                width,
                height,
                x + 3,
                y + 3,
                50,
                28,
                MENU_COLOR_THUMB_LOCKED,
            );
            draw_rect(
                pixels,
                width,
                height,
                x + 8,
                y + 19,
                40,
                7,
                MENU_COLOR_THUMB_GRASS,
            );
            draw_rect(pixels, width, height, x + 18, y + 8, 20, 16, 0xff222831);
        }
        MenuOverlayThumbnail::LockedDungeon => {
            draw_rect(
                pixels,
                width,
                height,
                x + 3,
                y + 3,
                50,
                28,
                MENU_COLOR_THUMB_LOCKED,
            );
            draw_rect(pixels, width, height, x + 9, y + 9, 38, 18, 0xff202737);
            draw_rect(
                pixels,
                width,
                height,
                x + 22,
                y + 12,
                12,
                12,
                MENU_COLOR_THUMB_STONE,
            );
        }
    }
}

fn draw_border(pixels: &mut [u32], width: u32, height: u32, x: i32, y: i32, w: i32, h: i32) {
    draw_rect(pixels, width, height, x, y, w, 1, MENU_COLOR_BORDER);
    draw_rect(pixels, width, height, x, y, 1, h, MENU_COLOR_BORDER);
    draw_rect(
        pixels,
        width,
        height,
        x,
        y + h - 1,
        w,
        1,
        MENU_COLOR_BORDER_SHADOW,
    );
    draw_rect(
        pixels,
        width,
        height,
        x + w - 1,
        y,
        1,
        h,
        MENU_COLOR_BORDER_SHADOW,
    );
}

fn draw_rect(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u32,
) {
    for py in y.max(0)..(y + h).min(height as i32) {
        for px in x.max(0)..(x + w).min(width as i32) {
            pixels[(py as u32 * width + px as u32) as usize] = color;
        }
    }
}

fn draw_text(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    text: &str,
    scale: i32,
    color: u32,
) {
    let mut cursor_x = x;
    for ch in text.chars() {
        if ch == ' ' {
            cursor_x += 4 * scale;
            continue;
        }
        draw_glyph(pixels, width, height, cursor_x, y, ch, scale, color);
        cursor_x += 6 * scale;
    }
}

fn draw_glyph(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    ch: char,
    scale: i32,
    color: u32,
) {
    let rows = glyph_rows(ch);
    for (row, bits) in rows.iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) == 0 {
                continue;
            }
            draw_rect(
                pixels,
                width,
                height,
                x + col * scale,
                y + row as i32 * scale,
                scale,
                scale,
                color,
            );
        }
    }
}

fn glyph_rows(ch: char) -> [i32; 7] {
    match ch {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        '>' => [
            0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000,
        ],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        _ => [
            0b11111, 0b10001, 0b10101, 0b10101, 0b10101, 0b10001, 0b11111,
        ],
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PresentationContext {
    pub in_dungeon: bool,
}

impl PresentationContext {
    fn scene_flags(&self) -> u32 {
        self.in_dungeon as u32
    }
}

const MAX_PRESENTATION_LIGHTS: usize = 8;
const PRESENTATION_LIGHT_MASK_GRID_W: usize = 16;
const PRESENTATION_LIGHT_MASK_GRID_H: usize = 14;
const PRESENTATION_LIGHT_MASK_CELLS: usize =
    PRESENTATION_LIGHT_MASK_GRID_W * PRESENTATION_LIGHT_MASK_GRID_H;
const PRESENTATION_LIGHT_MASK_VECS: usize = PRESENTATION_LIGHT_MASK_CELLS / 4;
const PRESENTATION_OCCLUDER_WORDS: usize = 8;
const PRESENTATION_UNIFORM_BYTES: usize = 32
    + MAX_PRESENTATION_LIGHTS * 16
    + PRESENTATION_OCCLUDER_WORDS * 4
    + PRESENTATION_LIGHT_MASK_VECS * 16;
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct PresentationLight {
    x: f32,
    y: f32,
    radius: f32,
    intensity: f32,
}

fn build_low_res_light_mask(lights: &[PresentationLight]) -> [f32; PRESENTATION_LIGHT_MASK_CELLS] {
    let mut mask = [0.0f32; PRESENTATION_LIGHT_MASK_CELLS];
    for cell_y in 0..PRESENTATION_LIGHT_MASK_GRID_H {
        for cell_x in 0..PRESENTATION_LIGHT_MASK_GRID_W {
            let uv = (
                (cell_x as f32 + 0.5) / PRESENTATION_LIGHT_MASK_GRID_W as f32,
                (cell_y as f32 + 0.5) / PRESENTATION_LIGHT_MASK_GRID_H as f32,
            );
            let mut lift = 0.0f32;
            for light in lights.iter().take(MAX_PRESENTATION_LIGHTS) {
                if light.radius <= 0.0 || light.intensity <= 0.0 {
                    continue;
                }
                let dx = uv.0 - light.x;
                let dy = uv.1 - light.y;
                let distance = (dx * dx + dy * dy).sqrt();
                let t = (1.0 - distance / light.radius).clamp(0.0, 1.0);
                let falloff = t * t * (3.0 - 2.0 * t);
                lift += falloff * light.intensity;
            }
            mask[cell_y * PRESENTATION_LIGHT_MASK_GRID_W + cell_x] = lift.min(1.0);
        }
    }
    mask
}

fn build_presentation_uniform_bytes(
    params: PresentationParams,
    context: PresentationContext,
    lights: &[PresentationLight],
    occluders: &[u32; PRESENTATION_OCCLUDER_WORDS],
) -> Vec<u8> {
    build_presentation_uniform_bytes_with_notice(
        params,
        context,
        lights,
        occluders,
        PresentationNotice::default(),
    )
}

fn build_presentation_uniform_bytes_with_notice(
    params: PresentationParams,
    context: PresentationContext,
    lights: &[PresentationLight],
    occluders: &[u32; PRESENTATION_OCCLUDER_WORDS],
    notice: PresentationNotice,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(PRESENTATION_UNIFORM_BYTES);
    let light_count = lights.len().min(MAX_PRESENTATION_LIGHTS) as u32;
    for word in [
        params.presentation as u32,
        params.lighting as u32,
        params.shadows as u32,
        light_count,
        context.scene_flags(),
        notice.code(),
        notice.frames_remaining(),
        0,
    ] {
        bytes.extend_from_slice(&word.to_ne_bytes());
    }
    for light in lights.iter().take(MAX_PRESENTATION_LIGHTS) {
        bytes.extend_from_slice(&light.x.to_ne_bytes());
        bytes.extend_from_slice(&light.y.to_ne_bytes());
        bytes.extend_from_slice(&light.radius.to_ne_bytes());
        bytes.extend_from_slice(&light.intensity.to_ne_bytes());
    }
    for _ in lights.len().min(MAX_PRESENTATION_LIGHTS)..MAX_PRESENTATION_LIGHTS {
        bytes.extend_from_slice(&[0u8; 16]);
    }
    for word in occluders {
        bytes.extend_from_slice(&word.to_ne_bytes());
    }
    for lift in build_low_res_light_mask(lights) {
        bytes.extend_from_slice(&lift.to_ne_bytes());
    }
    debug_assert_eq!(bytes.len(), PRESENTATION_UNIFORM_BYTES);
    bytes
}

/// Compute the centered game rect that fits in `surface`.
fn compute_viewport(
    surface_w: u32,
    surface_h: u32,
    game_w: u32,
    game_h: u32,
    mode: ViewportScaleMode,
) -> Viewport {
    match mode {
        ViewportScaleMode::Integer => {
            compute_integer_viewport(surface_w, surface_h, game_w, game_h)
        }
        ViewportScaleMode::Fit => compute_fit_viewport(surface_w, surface_h, game_w, game_h),
        ViewportScaleMode::Stretch => Viewport {
            x: 0.0,
            y: 0.0,
            w: surface_w as f32,
            h: surface_h as f32,
        },
    }
}

/// Compute the largest integer-scaled, centered game rect that fits in `surface`.
fn compute_integer_viewport(surface_w: u32, surface_h: u32, game_w: u32, game_h: u32) -> Viewport {
    let scale = (surface_w / game_w).min(surface_h / game_h).max(1);
    let scaled_w = game_w * scale;
    let scaled_h = game_h * scale;
    // saturating_sub: if scale=1 and the window is smaller than the game,
    // offset is 0 (no centering room) and the viewport is clamped to the surface.
    let x = surface_w.saturating_sub(scaled_w) / 2;
    let y = surface_h.saturating_sub(scaled_h) / 2;
    let w = scaled_w.min(surface_w);
    let h = scaled_h.min(surface_h);
    Viewport {
        x: x as f32,
        y: y as f32,
        w: w as f32,
        h: h as f32,
    }
}

/// Compute the largest aspect-preserving centered game rect that fits in `surface`.
fn compute_fit_viewport(surface_w: u32, surface_h: u32, game_w: u32, game_h: u32) -> Viewport {
    if surface_w == 0 || surface_h == 0 || game_w == 0 || game_h == 0 {
        return Viewport {
            x: 0.0,
            y: 0.0,
            w: surface_w as f32,
            h: surface_h as f32,
        };
    }

    let scale = (surface_w as f32 / game_w as f32).min(surface_h as f32 / game_h as f32);
    let w = game_w as f32 * scale;
    let h = game_h as f32 * scale;
    Viewport {
        x: (surface_w as f32 - w) * 0.5,
        y: (surface_h as f32 - h) * 0.5,
        w,
        h,
    }
}

// ── RenderError ───────────────────────────────────────────────────────────────

/// Errors returned by [`FrameRenderer::render`].
#[derive(Debug)]
pub enum RenderError {
    /// Surface was lost or became outdated; caller should call [`FrameRenderer::resize`].
    SurfaceReconfigureNeeded,
    /// Surface was temporarily unavailable; caller can skip this frame.
    SurfaceSkipped,
    /// Unrecoverable render error.
    Fatal(String),
}

// ── Shared GPU helpers ────────────────────────────────────────────────────────

fn create_wgpu_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: None,
    })
}

async fn create_device_queue(
    instance: &wgpu::Instance,
    compatible_surface: Option<&wgpu::Surface<'_>>,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface,
            force_fallback_adapter: false,
        })
        .await
        .expect("no suitable GPU adapter");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            experimental_features: Default::default(),
            memory_hints: Default::default(),
            trace: Default::default(),
        })
        .await
        .expect("failed to create wgpu device");
    // Make GPU resource-load / validation / out-of-memory failures LOUD on the
    // offscreen devices (compare, validation, capture). wgpu's default
    // uncaptured-error behavior only logs and then hands back an "error resource"
    // that renders defined-but-garbage output; under GPU memory pressure (e.g. a
    // concurrent GPU process exhausting the shared pool) that silently corrupts a
    // single compare frame and surfaces as a nondeterministic parity mismatch with
    // no panic. Panicking converts any such failure into a hard, attributable
    // abort. The live windowed device (the only caller passing a surface) keeps
    // the default glitch-don't-crash behavior.
    if compatible_surface.is_none() {
        device.on_uncaptured_error(std::sync::Arc::new(|error: wgpu::Error| {
            panic!("wgpu uncaptured error on offscreen device: {error}");
        }));
    }
    (adapter, device, queue)
}

/// Creates the `Rgba8Unorm` (TEXTURE_BINDING | COPY_DST) game-frame input
/// texture at `(width, height)`. Shared by the initial construction
/// ([`create_game_texture_resources`]) and HD resize
/// ([`GameTexture::ensure_size`]), which recreate it at a new size.
fn create_game_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("game_frame"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// The game-frame texture + its bind group, size-tracked so callers only
/// recreate it when the uploaded frame's dimensions actually change (the
/// classic path always uploads native 256×224; the modern HD path uploads
/// `scale*256 × scale*224` and may change `scale` at runtime).
struct GameTexture {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl GameTexture {
    /// Recreate the texture + bind group at `(width, height)` if they differ
    /// from the current size (the existing `bind_group_layout` and
    /// `presentation_buf` are reused — only the sized texture and its bind
    /// group change); returns whether it recreated. A same-size call is a
    /// cheap no-op, so this can run unconditionally every present.
    fn ensure_size(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        presentation_buf: &wgpu::Buffer,
        presentation: PresentationMode,
        width: u32,
        height: u32,
    ) -> bool {
        if width == self.width && height == self.height {
            return false;
        }
        let texture = create_game_texture(device, width, height);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_presentation_sampler(device, presentation, "blit");
        let bind_group = create_blit_bind_group(
            device,
            bind_group_layout,
            &view,
            &sampler,
            presentation_buf,
            "blit",
        );
        self.texture = texture;
        self.bind_group = bind_group;
        self.width = width;
        self.height = height;
        true
    }
}

struct ModernGpuTarget {
    texture: wgpu::Texture,
    scanout_history_texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    has_scanout_history: bool,
}

struct ModernSourceExtractionCache {
    fingerprint: u64,
    assets: modern_extract::AssetResolvedModernFrame,
}

const MODERN_SOURCE_FINGERPRINT_SLOTS: usize = 4096;

fn modern_source_extraction_fingerprint<S: modern_extract::SourceTableView + ?Sized>(
    frame: &GpuFrame<'_>,
    src_table: &S,
) -> Option<u64> {
    if frame.dialogue_message_id.is_some()
        || frame.dialogue_layout_origin_tile_number.is_some()
        || !frame.bg3_vwf_glyph_runs.is_empty()
        || !frame.source_dialogue_ir.is_empty()
        || !frame.dialogue_ir.is_empty()
        || !frame.dialogue_layout.is_empty()
    {
        return None;
    }

    let mut hasher = DefaultHasher::new();
    frame.vram.hash(&mut hasher);
    // Sprite extraction can intentionally decode a renderer-only OBJ cache
    // generation while BG extraction continues to use the published raw VRAM.
    // Hash the effective OBJ source so latch-only publication cannot reuse
    // sprite cells extracted from the preceding frame.
    frame.obj_vram().hash(&mut hasher);
    // BG pattern extraction has the same split-generation contract: tilemap
    // words come from raw VRAM, while pattern fetches can come from the
    // explicitly published decoded BG cache. Include that effective source
    // in the cache identity so a BG-latch-only boundary cannot reuse cells
    // extracted from a different hardware generation.
    frame.bg_vram.unwrap_or(frame.vram).hash(&mut hasher);
    frame.cgram.hash(&mut hasher);
    frame.oam.hash(&mut hasher);
    frame.mode.hash(&mut hasher);
    frame.mode1_bg3_priority.hash(&mut hasher);
    for bg in frame.bg {
        bg.h_scroll.hash(&mut hasher);
        bg.v_scroll.hash(&mut hasher);
        bg.tilemap_wider.hash(&mut hasher);
        bg.tilemap_higher.hash(&mut hasher);
        bg.tilemap_adr.hash(&mut hasher);
        bg.tile_adr.hash(&mut hasher);
    }
    frame.obj.tile_adr1.hash(&mut hasher);
    frame.obj.tile_adr2.hash(&mut hasher);
    frame.obj.obj_size.hash(&mut hasher);
    frame.mosaic_enabled.hash(&mut hasher);
    frame.mosaic_size.hash(&mut hasher);
    frame.extra_left_right.hash(&mut hasher);
    frame.mode7.matrix.hash(&mut hasher);
    frame.mode7.large_field.hash(&mut hasher);
    frame.mode7.char_fill.hash(&mut hasher);
    frame.mode7.x_flip.hash(&mut hasher);
    frame.mode7.y_flip.hash(&mut hasher);
    frame.mode7.ext_bg_always_zero.hash(&mut hasher);
    frame.screen_enabled.hash(&mut hasher);
    frame.screen_windowed.hash(&mut hasher);
    frame.brightness.hash(&mut hasher);
    frame.scanout_top_crop.hash(&mut hasher);
    frame.forced_blank.hash(&mut hasher);
    frame.math_enabled.hash(&mut hasher);
    frame.subtract_color.hash(&mut hasher);
    frame.half_color.hash(&mut hasher);
    frame.fixed_color_r.hash(&mut hasher);
    frame.fixed_color_g.hash(&mut hasher);
    frame.fixed_color_b.hash(&mut hasher);
    frame.add_subscreen.hash(&mut hasher);
    frame.clip_mode.hash(&mut hasher);
    frame.prevent_math_mode.hash(&mut hasher);
    frame.windowsel_cm.hash(&mut hasher);
    frame.windowsel.hash(&mut hasher);
    for scanline in frame.scanlines.iter() {
        scanline.window1_left.hash(&mut hasher);
        scanline.window1_right.hash(&mut hasher);
        scanline.window2_left.hash(&mut hasher);
        scanline.window2_right.hash(&mut hasher);
        scanline.screen_enabled_main.hash(&mut hasher);
        scanline.bg_h_scroll.hash(&mut hasher);
        scanline.bg_v_scroll.hash(&mut hasher);
        scanline.mode7_matrix.hash(&mut hasher);
    }
    for tile in frame.bg3_source_tiles {
        tile.chr_base.hash(&mut hasher);
        tile.tile_number.hash(&mut hasher);
        tile.source_key.hash(&mut hasher);
    }
    if let Some(snapshot) = frame.cgram_provenance {
        snapshot.words.hash(&mut hasher);
        snapshot.known.hash(&mut hasher);
    } else {
        0u8.hash(&mut hasher);
    }
    for slot in 0..MODERN_SOURCE_FINGERPRINT_SLOTS {
        src_table.get(slot).hash(&mut hasher);
    }
    Some(hasher.finish())
}

fn create_modern_gpu_target(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    presentation_buf: &wgpu::Buffer,
    presentation: PresentationMode,
    format: wgpu::TextureFormat,
) -> ModernGpuTarget {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("modern_gpu_live_target"),
        size: wgpu::Extent3d {
            width: 256,
            height: 224,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let scanout_history_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("modern_gpu_scanout_history"),
        size: wgpu::Extent3d {
            width: 256,
            height: 224,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = create_presentation_sampler(device, presentation, "modern_gpu_blit");
    let bind_group = create_blit_bind_group(
        device,
        bind_group_layout,
        &view,
        &sampler,
        presentation_buf,
        "modern_gpu_blit",
    );
    ModernGpuTarget {
        texture,
        scanout_history_texture,
        view,
        bind_group,
        has_scanout_history: false,
    }
}

/// Rows that scanned out before an active-display forced-blank transition.
///
/// The NMI may replace VRAM/source state after these rows are already visible,
/// so they must retain the preceding physical surface instead of being
/// reconstructed from the post-NMI asset state.
fn active_display_history_rows(frame: &modern_frame::ModernFrame) -> Option<(u32, u32)> {
    if !frame.retain_active_display_history {
        return None;
    }
    // Pinned Snes9x leaves its host screen buffer untouched when the whole
    // completed scanout is force-blank. Preserve the entire preceding host
    // surface in that case; partial active-display transitions retain only
    // the interval which scanned out before the blanking edge.
    if frame.forced_blank || frame.forced_blank_scanlines >= 224 {
        return Some((0, 224));
    }
    let start = u32::from(frame.forced_blank_scanlines).min(224);
    let end = u32::from(frame.forced_blank_from_scanline?).min(224);
    (start < end).then_some((start, end - start))
}

fn copy_modern_gpu_target_rows(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &wgpu::Texture,
    destination: &wgpu::Texture,
    start_row: u32,
    row_count: u32,
    label: &'static str,
) {
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: source,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: 0,
                y: start_row,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: destination,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: 0,
                y: start_row,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: 256,
            height: row_count,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
}

/// Creates the game-frame input texture, its bind group layout, and bind group.
///
/// The texture is `Rgba8Unorm` (TEXTURE_BINDING | COPY_DST). Callers upload
/// pixels via [`upload_ppu_pixels`]; the bind group wires it to the blit shader.
fn create_game_texture_resources(
    device: &wgpu::Device,
    game_width: u32,
    game_height: u32,
    params: PresentationParams,
) -> (
    wgpu::Texture,
    wgpu::BindGroupLayout,
    wgpu::BindGroup,
    wgpu::Buffer,
) {
    let game_texture = create_game_texture(device, game_width, game_height);

    let game_texture_view = game_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("blit"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(PRESENTATION_UNIFORM_BYTES as u64),
                },
                count: None,
            },
        ],
    });

    let sampler = create_presentation_sampler(device, params.presentation, "nearest");
    let params_buf = create_presentation_buffer(device, params, "blit_presentation_params");
    let bind_group = create_blit_bind_group(
        device,
        &bind_group_layout,
        &game_texture_view,
        &sampler,
        &params_buf,
        "blit",
    );

    (game_texture, bind_group_layout, bind_group, params_buf)
}

fn create_presentation_sampler(
    device: &wgpu::Device,
    mode: PresentationMode,
    label: &str,
) -> wgpu::Sampler {
    let filter = match mode {
        PresentationMode::Sharp => wgpu::FilterMode::Linear,
        PresentationMode::Off | PresentationMode::Crt => wgpu::FilterMode::Nearest,
    };
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

fn create_presentation_buffer(
    device: &wgpu::Device,
    params: PresentationParams,
    label: &str,
) -> wgpu::Buffer {
    let bytes = build_presentation_uniform_bytes(
        params,
        PresentationContext::default(),
        &[],
        &[0; PRESENTATION_OCCLUDER_WORDS],
    );
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: &bytes,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

fn create_blit_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    params_buf: &wgpu::Buffer,
    label: &str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buf.as_entire_binding(),
            },
        ],
    })
}

fn create_blit_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("blit"),
        source: wgpu::ShaderSource::Wgsl(include_str!("blit.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("blit"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blit"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: output_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// Swap PPU pixels (0xFF_RR_GG_BB) to RGBA order and upload to the game texture.
///
/// Uses `staging` as a pre-allocated scratch buffer to avoid per-frame allocation.
fn upload_ppu_pixels(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    pixels: &[u32],
    staging: &mut Vec<u8>,
    width: u32,
    height: u32,
) {
    debug_assert_eq!(pixels.len(), (width * height) as usize);
    for (dst, &src) in staging.chunks_exact_mut(4).zip(pixels.iter()) {
        // PPU: 0xFF_RR_GG_BB  →  to_le_bytes() = [BB, GG, RR, FF]
        let [b, g, r, a] = src.to_le_bytes();
        dst[0] = r;
        dst[1] = g;
        dst[2] = b;
        dst[3] = a;
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        staging,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

// ── FrameRenderer ─────────────────────────────────────────────────────────────

/// Presents Zelda frames to a winit window surface.
///
/// The default live path is PNG-backed GPU asset rendering
/// (`assets-variant-gpu`). Classic and diagnostic modes may still upload
/// CPU-produced framebuffers explicitly.
pub struct FrameRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    game_texture: GameTexture,
    bind_group_layout: wgpu::BindGroupLayout,
    presentation_buf: wgpu::Buffer,
    upload_buf: Vec<u8>,
    game_width: u32,
    game_height: u32,
    scale_mode: ViewportScaleMode,
    presentation_params: PresentationParams,
    presentation_notice: PresentationNotice,
    viewport: Viewport,
    log_viewport: bool,
    /// Integer HD scale for source-backed modern asset presentation
    /// (`ZELDA3_HD_SCALE`, default 2), read once at construction.
    hd_scale: modern_hd_overrides::HdScale,
    /// Lazily built on first diagnostic `present_modern_gpu` call
    /// (`assets-anim-gpu` mode).
    modern_gpu: Option<ModernGpuCompositor>,
    /// Lazily built on first `present_modern_variant_gpu` call
    /// (`assets-variant-gpu`, the default modern asset path).
    modern_variant_gpu: Option<ModernGpuVariantRenderer>,
    /// Offscreen Rgba8Unorm 256x224 target the compositor renders into before
    /// it is sampled directly by the presentation blit.
    modern_gpu_target: Option<ModernGpuTarget>,
    modern_source_extraction_cache: Option<ModernSourceExtractionCache>,
}

#[derive(Debug, Default)]
pub enum ModernAssetFramePresentResult {
    Presented {
        via: &'static str,
        variant_stats: Option<modern_software::VariantAtlasRenderStats>,
    },
    Unsupported {
        via: &'static str,
        variant_stats: Option<modern_software::VariantAtlasRenderStats>,
    },
    #[default]
    Unhandled,
}

impl ModernAssetFramePresentResult {
    pub fn is_presented(&self) -> bool {
        matches!(self, Self::Presented { .. })
    }
}

pub struct ModernAssetFramePresentInput<'a, 'frame, T>
where
    T: Copy + Into<(u8, u16, u16)>,
{
    pub frame: &'a GpuFrame<'frame>,
    pub source_entries: &'a [T],
    pub resources: &'a ModernAssetFrameResources,
    pub player_indoors: u8,
}

pub struct ModernAssetFramePresentOutput {
    pub result: ModernAssetFramePresentResult,
    pub in_dungeon: bool,
}

pub struct ModernAssetFrameLivePresentInput<'a, 'frame, T>
where
    T: Copy + Into<(u8, u16, u16)>,
{
    pub frame: GpuFrameCaptureInput<'frame>,
    pub source_entries: &'a [T],
    pub resources: &'a ModernAssetFrameResources,
    pub stats: &'a mut ModernAssetLiveStats,
    pub player_indoors: u8,
}

/// Renderer-owned resource bundle for live modern-asset presentation.
///
/// `zelda3-bin` owns game state and runtime inputs; the renderer owns which
/// modern asset stores a mode requires and how those stores are routed.
pub struct ModernAssetFrameResources {
    source_atlas: Option<modern_source_atlas::ModernSourceAtlas>,
    variant_atlas: Option<modern_variant_atlas::ModernVariantAtlas>,
    variant_headless: Option<ModernGpuVariantHeadless>,
    gpu_asset_mode: bool,
    variant_gpu_mode: bool,
}

impl ModernAssetFrameResources {
    pub fn load_from_env(root: &Path) -> Result<Self, String> {
        Self::load_for_mode(EffectiveRendererMode::from_env(), root)
    }

    pub fn load_live_gpu_from_env(
        root: &Path,
    ) -> Result<(Self, EffectiveRendererMode<'static>), String> {
        let mode = EffectiveRendererMode::live_gpu_asset_from_env()?;
        let resources = Self::load_for_mode(mode, root)?;
        Ok((resources, mode))
    }

    pub fn load_for_mode(mode: EffectiveRendererMode<'_>, root: &Path) -> Result<Self, String> {
        let variant_atlas = if mode.uses_variant_atlas() {
            Some(modern_variant_atlas::load_modern_canonical_art_atlas(root)?)
        } else {
            None
        };
        let source_atlas = if mode.uses_variant_atlas() {
            Some(
                modern_source_atlas::load_modern_source_atlas(root)
                    .map_err(|e| format!("assets-by-source atlas missing: {e}"))?,
            )
        } else {
            None
        };

        let variant_headless = variant_atlas.as_ref().map(ModernGpuVariantHeadless::new);

        Ok(Self {
            source_atlas,
            variant_atlas,
            variant_headless,
            gpu_asset_mode: mode.uses_gpu_assets(),
            variant_gpu_mode: mode.uses_variant_atlas(),
        })
    }

    pub fn source_atlas(&self) -> Option<&modern_source_atlas::ModernSourceAtlas> {
        self.source_atlas.as_ref()
    }

    pub fn variant_atlas(&self) -> Option<&modern_variant_atlas::ModernVariantAtlas> {
        self.variant_atlas.as_ref()
    }

    pub fn variant_headless(&self) -> Option<&ModernGpuVariantHeadless> {
        self.variant_headless.as_ref()
    }

    pub fn mode7_source_chars(&self) -> Option<&[u8]> {
        self.variant_atlas_mode7_source_chars()
    }

    fn variant_atlas_mode7_source_chars(&self) -> Option<&[u8]> {
        self.variant_atlas
            .as_ref()
            .and_then(modern_variant_atlas::ModernVariantAtlas::mode7_source_chars)
    }

    fn has_mode7_source_art(&self) -> bool {
        self.variant_atlas
            .as_ref()
            .is_some_and(modern_variant_atlas::ModernVariantAtlas::has_mode7_source_art)
    }

    fn gpu_asset_mode(&self) -> bool {
        self.gpu_asset_mode
    }

    fn variant_gpu_mode(&self) -> bool {
        self.variant_gpu_mode
    }

    fn unhandled_gpu_asset_frame_line(&self) -> Option<&'static str> {
        self.gpu_asset_mode
            .then_some("modern asset renderer did not handle a GPU asset frame")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ModernIndexCompareResourcePlan {
    load_source_atlas: bool,
    load_variant_atlas: bool,
    load_gpu_headless: bool,
}

fn modern_index_compare_resource_plan(
    enabled: bool,
    mode: EffectiveRendererMode<'_>,
    allow_source_cpu_fallback: bool,
) -> ModernIndexCompareResourcePlan {
    if !enabled {
        return ModernIndexCompareResourcePlan {
            load_source_atlas: false,
            load_variant_atlas: false,
            load_gpu_headless: false,
        };
    }

    let source_gpu = mode.name() == "assets-anim-gpu";
    let source_cpu = allow_source_cpu_fallback && mode.name() == "assets-anim";
    let variant_gpu = mode.uses_variant_atlas();
    ModernIndexCompareResourcePlan {
        load_source_atlas: source_gpu || source_cpu || variant_gpu,
        load_variant_atlas: variant_gpu,
        load_gpu_headless: source_gpu || variant_gpu,
    }
}

/// Renderer-owned resource bundle for modern-index compare runs.
///
/// The binary decides whether a compare is requested and supplies the effective
/// renderer mode. The renderer owns which atlases and headless GPU helpers that
/// mode needs for modern-index rendering.
pub struct ModernIndexCompareResources {
    source_atlas: Option<modern_source_atlas::ModernSourceAtlas>,
    gpu_headless: Option<ModernGpuHeadless>,
    variant_headless: Option<ModernGpuVariantHeadless>,
}

pub struct ModernAssetReadbackFrame {
    pub rgba: Vec<u8>,
    pub via: &'static str,
    pub variant_stats: Option<modern_software::VariantAtlasRenderStats>,
}

fn source_backed_missing_art_is_resolvable(reason: &str, missing_source_count: usize) -> bool {
    reason == "missing-art" && missing_source_count == 0
}

pub struct ModernAssetValidationFrame {
    pub via: &'static str,
    pub timings: modern_gpu::ModernIndexCompareValidationTimings,
}

impl ModernIndexCompareResources {
    pub fn load_from_env(
        enabled: bool,
        root: &Path,
        allow_source_cpu_fallback: bool,
    ) -> Result<Self, String> {
        Self::load_for_mode(
            enabled,
            EffectiveRendererMode::from_env(),
            root,
            allow_source_cpu_fallback,
        )
    }

    pub fn load_live_gpu_from_env(enabled: bool, root: &Path) -> Result<Self, String> {
        Self::load_for_mode(
            enabled,
            EffectiveRendererMode::live_gpu_asset_from_env()?,
            root,
            false,
        )
    }

    pub fn load_for_mode(
        enabled: bool,
        mode: EffectiveRendererMode<'_>,
        root: &Path,
        allow_source_cpu_fallback: bool,
    ) -> Result<Self, String> {
        let plan = modern_index_compare_resource_plan(enabled, mode, allow_source_cpu_fallback);
        let source_atlas = if plan.load_source_atlas {
            Some(
                modern_source_atlas::load_modern_source_atlas(root)
                    .map_err(|e| format!("assets-by-source atlas load failed: {e}"))?,
            )
        } else {
            None
        };
        let variant_atlas = if plan.load_variant_atlas {
            Some(
                modern_variant_atlas::load_modern_canonical_art_atlas(root)
                    .map_err(|e| format!("canonical art atlas load failed: {e}"))?,
            )
        } else {
            None
        };
        let gpu_headless = plan.load_gpu_headless.then(ModernGpuHeadless::new);
        let variant_headless = variant_atlas.as_ref().map(ModernGpuVariantHeadless::new);

        Ok(Self {
            source_atlas,
            gpu_headless,
            variant_headless,
        })
    }

    /// CPU-only comparison resources: the source atlas without any GPU
    /// device. The compare-frame router renders via the software source
    /// compositor (and the CPU Mode-7 path); useful for GPU-free, parallel
    /// comparison runs.
    pub fn load_cpu_compare(root: &Path) -> Result<Self, String> {
        let source_atlas = Some(
            modern_source_atlas::load_modern_source_atlas(root)
                .map_err(|e| format!("assets-by-source atlas load failed: {e}"))?,
        );
        Ok(Self {
            source_atlas,
            gpu_headless: None,
            variant_headless: None,
        })
    }

    pub fn source_atlas(&self) -> Option<&modern_source_atlas::ModernSourceAtlas> {
        self.source_atlas.as_ref()
    }

    pub fn gpu_headless(&self) -> Option<&ModernGpuHeadless> {
        self.gpu_headless.as_ref()
    }

    pub fn variant_headless(&self) -> Option<&ModernGpuVariantHeadless> {
        self.variant_headless.as_ref()
    }

    pub fn variant_atlas(&self) -> Option<&modern_variant_atlas::ModernVariantAtlas> {
        self.variant_headless
            .as_ref()
            .map(ModernGpuVariantHeadless::atlas)
    }

    pub fn render_full_gpu_asset_rgba_from_entries<T>(
        &self,
        frame: &GpuFrame<'_>,
        source_entries: &[T],
        scene: ModernAssetFrameScene,
    ) -> Result<ModernAssetReadbackFrame, String>
    where
        T: Copy + Into<(u8, u16, u16)>,
    {
        if self.variant_headless.is_none() {
            return Err(
                "modern asset GPU readback requires canonical RGBA variant atlas".to_string(),
            );
        }
        let src_table = source_table_from_entries(source_entries);
        if frame.mode != 7 {
            let (rgba, variant_stats) = self
                .variant_headless()
                .expect("checked above")
                .render_live_gpu_rgba_from_sources(
                    frame,
                    &src_table,
                    self.source_atlas()
                        .expect("variant GPU readback requires source atlas"),
                    scene.bg_palette_name(),
                    scene.sprite_palette_name(),
                )?;
            return Ok(ModernAssetReadbackFrame {
                rgba,
                via: "variant-gpu",
                variant_stats: Some(variant_stats),
            });
        }
        let render = modern_gpu::render_modern_index_compare_frame(
            frame,
            Some(&src_table),
            self.source_atlas(),
            self.gpu_headless(),
            self.variant_headless(),
            None,
            scene,
            None,
            false,
        );
        if let Some(fallback) =
            modern_gpu::modern_gpu_path_fallback_reason(render.via, render.variant_stats.as_ref())
        {
            // A source-keyed tile without a pre-baked final-pixel variant is
            // still canonical modern art: the source atlas plus live palette
            // metadata resolves it exactly. This is the same source-backed
            // path accepted by the live presenter below. Only genuinely
            // unresolvable sources (or another fallback class) are unsupported.
            if source_backed_missing_art_is_resolvable(
                fallback.reason,
                render.missing_sources.len(),
            ) {
                return Ok(ModernAssetReadbackFrame {
                    rgba: render.rgba,
                    via: render.via,
                    variant_stats: render.variant_stats,
                });
            }
            let missing_report =
                modern_extract::format_missing_asset_source_report(&render.missing_sources, 4);
            let detail = if missing_report.is_empty() {
                String::new()
            } else {
                format!(" {missing_report}")
            };
            return Err(format!(
                "modern asset GPU readback unsupported via={} reason={} count={}",
                render.via, fallback.reason, fallback.count
            ) + &detail);
        }
        Ok(ModernAssetReadbackFrame {
            rgba: render.rgba,
            via: render.via,
            variant_stats: render.variant_stats,
        })
    }

    /// Render the same modern GPU route accepted by the production presenter.
    ///
    /// This intentionally differs from [`Self::render_full_gpu_asset_rgba_from_entries`]:
    /// the latter is a strict asset-coverage audit that rejects source-backed
    /// dynamic materials when they do not have a pre-baked final-pixel variant.
    /// Production renders those materials on the GPU from canonical source art
    /// plus live material inputs, so parity readback must accept them too. Truly
    /// missing/unkeyable sources and non-production routes remain errors.
    pub fn render_production_gpu_asset_rgba_from_entries<T>(
        &self,
        frame: &GpuFrame<'_>,
        source_entries: &[T],
        scene: ModernAssetFrameScene,
    ) -> Result<ModernAssetReadbackFrame, String>
    where
        T: Copy + Into<(u8, u16, u16)>,
    {
        if self.variant_headless.is_none() {
            return Err(
                "modern asset GPU readback requires canonical RGBA variant atlas".to_string(),
            );
        }
        let src_table = source_table_from_entries(source_entries);
        if frame.mode != 7 {
            let source_atlas = self.source_atlas().ok_or_else(|| {
                "modern production GPU readback requires a source atlas".to_string()
            })?;
            // Native play presents this exact source-keyed GPU compositor. Do
            // not send oracle readback through the separate variant planner:
            // its dynamic-material accounting describes pre-baked variant
            // availability, not the PNG/source GPU path that is actually on
            // screen.
            let resolved = modern_extract::extract_asset_resolved_modern_frame_from_sources(
                frame,
                &src_table,
                source_atlas,
            );
            if resolved.has_unresolved_sources() {
                let report = modern_extract::format_missing_asset_source_report(
                    &resolved.missing_sources,
                    4,
                );
                return Err(format!(
                    "modern production GPU readback has unresolved asset source(s): {report}"
                ));
            }
            let rgba = self
                .gpu_headless()
                .ok_or_else(|| {
                    "modern production GPU readback requires a source GPU compositor".to_string()
                })?
                .render_asset_resolved_gpu_rgba(
                    &resolved.frame,
                    &resolved.bg_cells,
                    &resolved.sprite_cells,
                )?;
            return Ok(ModernAssetReadbackFrame {
                rgba,
                via: "source-gpu",
                variant_stats: None,
            });
        }
        let render = modern_gpu::render_modern_index_compare_frame(
            frame,
            Some(&src_table),
            self.source_atlas(),
            self.gpu_headless(),
            self.variant_headless(),
            None,
            scene,
            None,
            false,
        );
        if let Some(fallback) =
            modern_gpu::modern_gpu_path_fallback_reason(render.via, render.variant_stats.as_ref())
        {
            let missing_report =
                modern_extract::format_missing_asset_source_report(&render.missing_sources, 4);
            let detail = if missing_report.is_empty() {
                String::new()
            } else {
                format!(" {missing_report}")
            };
            return Err(format!(
                "modern production GPU readback unsupported via={} reason={} count={}",
                render.via, fallback.reason, fallback.count,
            ) + &detail);
        }
        Ok(ModernAssetReadbackFrame {
            rgba: render.rgba,
            via: render.via,
            variant_stats: render.variant_stats,
        })
    }

    pub fn validate_full_gpu_asset_from_entries<T>(
        &self,
        frame: &GpuFrame<'_>,
        source_entries: &[T],
        scene: ModernAssetFrameScene,
    ) -> Result<ModernAssetValidationFrame, String>
    where
        T: Copy + Into<(u8, u16, u16)>,
    {
        if self.variant_headless.is_none() {
            return Err(
                "modern asset GPU validation requires canonical RGBA variant atlas".to_string(),
            );
        }
        let src_table = source_table_from_entries(source_entries);
        let render = modern_gpu::validate_modern_index_compare_frame(
            frame,
            Some(&src_table),
            self.source_atlas(),
            self.gpu_headless(),
            self.variant_headless(),
            None,
            scene,
            false,
        );
        if let Some(fallback) =
            modern_gpu::modern_gpu_path_fallback_reason(render.via, render.variant_stats.as_ref())
        {
            if source_backed_missing_art_is_resolvable(
                fallback.reason,
                render.missing_sources.len(),
            ) {
                return Ok(ModernAssetValidationFrame {
                    via: render.via,
                    timings: render.timings,
                });
            }
            let missing_report =
                modern_extract::format_missing_asset_source_report(&render.missing_sources, 4);
            let detail = if missing_report.is_empty() {
                String::new()
            } else {
                format!(" {missing_report}")
            };
            return Err(format!(
                "modern asset GPU validation unsupported via={} reason={} count={}",
                render.via, fallback.reason, fallback.count
            ) + &detail);
        }
        Ok(ModernAssetValidationFrame {
            via: render.via,
            timings: render.timings,
        })
    }

    pub fn validate_full_gpu_asset_from_resolved_frame(
        &self,
        modern_assets: &modern_extract::AssetResolvedModernFrame,
        scene: ModernAssetFrameScene,
    ) -> Result<ModernAssetValidationFrame, String> {
        let Some(variant_headless) = self.variant_headless() else {
            return Err(
                "modern asset GPU validation requires canonical RGBA variant atlas".to_string(),
            );
        };
        let validation = variant_headless.validate_asset_resolved_frame(
            modern_assets,
            scene.bg_palette_name(),
            scene.sprite_palette_name(),
        );
        let render = modern_gpu::ModernIndexCompareValidation {
            via: "variant-gpu",
            variant_stats: Some(validation.stats),
            missing_sources: validation.missing_sources,
            timings: validation.timings,
        };
        if let Some(fallback) =
            modern_gpu::modern_gpu_path_fallback_reason(render.via, render.variant_stats.as_ref())
        {
            if source_backed_missing_art_is_resolvable(
                fallback.reason,
                render.missing_sources.len(),
            ) {
                return Ok(ModernAssetValidationFrame {
                    via: render.via,
                    timings: render.timings,
                });
            }
            let missing_report =
                modern_extract::format_missing_asset_source_report(&render.missing_sources, 4);
            let detail = if missing_report.is_empty() {
                String::new()
            } else {
                format!(" {missing_report}")
            };
            return Err(format!(
                "modern asset GPU validation unsupported via={} reason={} count={}",
                render.via, fallback.reason, fallback.count
            ) + &detail);
        }
        Ok(ModernAssetValidationFrame {
            via: render.via,
            timings: render.timings,
        })
    }
}

/// Renderer-owned resource bundle for the legacy modern-atlas compare path.
///
/// The binary supplies only whether the diagnostic compare is requested. The
/// renderer owns which atlas that compare path needs.
pub struct ModernAtlasCompareResources {
    atlas: Option<modern_assets::ModernTileAtlasAsset>,
}

struct ModernAtlasCompareFrameInput<'a, 'frame> {
    pub frame: u32,
    pub gpu_frame: &'a GpuFrame<'frame>,
    pub classic_rgba: &'a [u8],
}

pub struct ModernAtlasCompareFrameReport {
    pub line: String,
    pub result: modern_gpu::ModernAtlasCompareResult,
}

impl ModernAtlasCompareResources {
    pub fn load(enabled: bool, root: &Path) -> Result<Self, String> {
        let atlas = if enabled {
            Some(
                modern_assets::load_modern_overworld_tile_atlas(root)
                    .map_err(|e| format!("modern atlas load failed: {e}"))?,
            )
        } else {
            None
        };
        Ok(Self { atlas })
    }

    pub fn atlas(&self) -> Option<&modern_assets::ModernTileAtlasAsset> {
        self.atlas.as_ref()
    }

    fn compare_frame(
        &self,
        input: ModernAtlasCompareFrameInput<'_, '_>,
    ) -> Option<ModernAtlasCompareFrameReport> {
        let atlas = self.atlas.as_ref()?;
        let result =
            modern_gpu::compare_modern_atlas_to_rgba(input.classic_rgba, input.gpu_frame, atlas);
        let line = modern_atlas_compare_frame_line(input.frame, &result);
        Some(ModernAtlasCompareFrameReport { line, result })
    }

    pub fn compare_frame_rgba(
        &self,
        frame: u32,
        gpu_frame: &GpuFrame<'_>,
        classic_rgba: &[u8],
    ) -> Option<ModernAtlasCompareFrameReport> {
        self.compare_frame(ModernAtlasCompareFrameInput {
            frame,
            gpu_frame,
            classic_rgba,
        })
    }
}

fn modern_atlas_compare_frame_line(
    frame: u32,
    result: &modern_gpu::ModernAtlasCompareResult,
) -> String {
    format!(
        "modern_render_compare frame={frame} old=0x{:08x} modern=0x{:08x} match={}",
        result.classic_hash, result.modern_hash, result.matches
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernAssetFrameScene {
    in_dungeon: bool,
}

impl ModernAssetFrameScene {
    pub const DUNGEON_BG_PALETTE_NAME: &'static str = "palette_dung_bg_main";
    pub const OVERWORLD_BG_PALETTE_NAME: &'static str = "palette_overworld_bg_main";
    pub const SPRITE_PALETTE_NAME: &'static str = "palette_main_spr";

    pub const fn from_in_dungeon(in_dungeon: bool) -> Self {
        Self { in_dungeon }
    }

    pub const fn from_player_indoors_flag(player_indoors: u8) -> Self {
        Self {
            in_dungeon: player_indoors != 0,
        }
    }

    pub const fn in_dungeon(self) -> bool {
        self.in_dungeon
    }

    pub const fn bg_palette_name(self) -> &'static str {
        if self.in_dungeon {
            Self::DUNGEON_BG_PALETTE_NAME
        } else {
            Self::OVERWORLD_BG_PALETTE_NAME
        }
    }

    pub const fn sprite_palette_name(self) -> &'static str {
        Self::SPRITE_PALETTE_NAME
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ModernIndexCompareScene {
    mode_label: String,
    asset_scene: ModernAssetFrameScene,
}

impl ModernIndexCompareScene {
    pub fn from_main_module_and_player_indoors_flag(main_module: u8, player_indoors: u8) -> Self {
        let mode_label = match main_module {
            9 | 11 => "ow".to_string(),
            7 | 16 => "dungeon".to_string(),
            module => format!("mod{module}"),
        };
        Self {
            mode_label,
            asset_scene: ModernAssetFrameScene::from_player_indoors_flag(player_indoors),
        }
    }

    pub fn mode_label(&self) -> &str {
        &self.mode_label
    }

    pub const fn asset_scene(&self) -> ModernAssetFrameScene {
        self.asset_scene
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModernAssetFramePresentRoute {
    Mode7SourceGpu,
    SourceVariantGpu,
    Unhandled,
}

fn modern_asset_frame_present_route(
    frame_mode: u8,
    has_src_table: bool,
    has_source_atlas: bool,
    has_variant_atlas: bool,
    has_mode7_source_chars: bool,
    has_mode7_source_art: bool,
    gpu_asset_mode: bool,
    variant_gpu_mode: bool,
) -> ModernAssetFramePresentRoute {
    if frame_mode == 7 {
        return if variant_gpu_mode {
            if gpu_asset_mode && has_mode7_source_chars && has_mode7_source_art {
                ModernAssetFramePresentRoute::Mode7SourceGpu
            } else {
                ModernAssetFramePresentRoute::Unhandled
            }
        } else if gpu_asset_mode && has_mode7_source_chars {
            ModernAssetFramePresentRoute::Mode7SourceGpu
        } else {
            ModernAssetFramePresentRoute::Unhandled
        };
    }

    if variant_gpu_mode {
        if gpu_asset_mode && has_src_table && has_source_atlas && has_variant_atlas {
            return ModernAssetFramePresentRoute::SourceVariantGpu;
        }
        return ModernAssetFramePresentRoute::Unhandled;
    }

    ModernAssetFramePresentRoute::Unhandled
}

impl FrameRenderer {
    pub async fn new(window: Arc<Window>, game_width: u32, game_height: u32) -> Self {
        let instance = create_wgpu_instance();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let (adapter, device, queue) = create_device_queue(&instance, Some(&surface)).await;

        let caps = surface.get_capabilities(&adapter);

        // Prefer a non-sRGB format so SNES palette values aren't double-gamma'd.
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                !f.is_srgb()
                    && matches!(
                        f,
                        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                    )
            })
            .unwrap_or(caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let presentation_params = PresentationParams::from_env();
        let (texture, bind_group_layout, bind_group, presentation_buf) =
            create_game_texture_resources(&device, game_width, game_height, presentation_params);
        let game_texture = GameTexture {
            texture,
            bind_group,
            width: game_width,
            height: game_height,
        };
        let pipeline = create_blit_pipeline(&device, &bind_group_layout, surface_format);
        let scale_mode = ViewportScaleMode::from_env();
        let viewport = compute_viewport(
            config.width,
            config.height,
            game_width,
            game_height,
            scale_mode,
        );
        let upload_buf = vec![0u8; (game_width * game_height * 4) as usize];
        let hd_scale = modern_hd_overrides::HdScale::from_env();

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            game_texture,
            bind_group_layout,
            presentation_buf,
            upload_buf,
            game_width,
            game_height,
            scale_mode,
            presentation_params,
            presentation_notice: PresentationNotice::default(),
            viewport,
            log_viewport: env::var_os("ZELDA3_RENDER_VIEWPORT_LOG").is_some(),
            hd_scale,
            modern_gpu: None,
            modern_variant_gpu: None,
            modern_gpu_target: None,
            modern_source_extraction_cache: None,
        }
    }

    /// Current live HD scale (`ZELDA3_HD_SCALE`, default 2), cached at
    /// construction. Callers that render the modern sources+overrides path
    /// themselves (see module docs on `hd_scale` field) use this so their
    /// finished RGBA uses the same configured scale as the live asset route.
    pub fn hd_scale(&self) -> u32 {
        self.hd_scale.get()
    }

    pub fn cycle_presentation_mode(&mut self) {
        self.presentation_params.cycle_presentation();
        self.presentation_notice
            .show_presentation(self.presentation_params.presentation);
        self.rebuild_presentation_bind_groups();
        self.write_cpu_presentation_params();
    }

    pub fn cycle_lighting_mode(&mut self) {
        self.presentation_params.cycle_lighting();
        self.presentation_notice
            .show_lighting(self.presentation_params.lighting);
        self.write_cpu_presentation_params();
    }

    pub fn cycle_shadow_mode(&mut self) {
        self.presentation_params.cycle_shadows();
        self.presentation_notice
            .show_shadows(self.presentation_params.shadows);
        self.write_cpu_presentation_params();
    }

    pub fn apply_runtime_settings(&mut self, settings: RendererRuntimeSettings) {
        let next_scale_mode = ViewportScaleMode::from_runtime_choice(settings.viewport);
        let next = PresentationParams::from_runtime_settings(settings);
        let presentation_changed = self.presentation_params.presentation != next.presentation;
        let lighting_changed = self.presentation_params.lighting != next.lighting;
        let shadows_changed = self.presentation_params.shadows != next.shadows;
        let viewport_changed = self.scale_mode != next_scale_mode;
        self.presentation_params = next;
        if presentation_changed {
            self.presentation_notice
                .show_presentation(self.presentation_params.presentation);
        } else if lighting_changed {
            self.presentation_notice
                .show_lighting(self.presentation_params.lighting);
        } else if shadows_changed {
            self.presentation_notice
                .show_shadows(self.presentation_params.shadows);
        } else if viewport_changed {
            self.presentation_notice.show_viewport(next_scale_mode);
        }
        if viewport_changed {
            self.scale_mode = next_scale_mode;
            self.viewport = compute_viewport(
                self.config.width,
                self.config.height,
                self.game_width,
                self.game_height,
                self.scale_mode,
            );
        }
        if presentation_changed {
            self.rebuild_presentation_bind_groups();
        }
        self.write_cpu_presentation_params();
    }

    fn write_cpu_presentation_params(&self) {
        let bytes = build_presentation_uniform_bytes_with_notice(
            self.presentation_params,
            PresentationContext::default(),
            &[],
            &[0; PRESENTATION_OCCLUDER_WORDS],
            self.presentation_notice,
        );
        self.queue.write_buffer(&self.presentation_buf, 0, &bytes);
    }

    fn tick_presentation_notice(&mut self) {
        self.presentation_notice.tick();
    }

    fn rebuild_presentation_bind_groups(&mut self) {
        let game_texture_view = self
            .game_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_presentation_sampler(
            &self.device,
            self.presentation_params.presentation,
            "blit",
        );
        self.game_texture.bind_group = create_blit_bind_group(
            &self.device,
            &self.bind_group_layout,
            &game_texture_view,
            &sampler,
            &self.presentation_buf,
            "blit",
        );

        if let Some(target) = &mut self.modern_gpu_target {
            let modern_sampler = create_presentation_sampler(
                &self.device,
                self.presentation_params.presentation,
                "modern_gpu_blit",
            );
            target.bind_group = create_blit_bind_group(
                &self.device,
                &self.bind_group_layout,
                &target.view,
                &modern_sampler,
                &self.presentation_buf,
                "modern_gpu_blit",
            );
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.viewport = compute_viewport(
            new_size.width,
            new_size.height,
            self.game_width,
            self.game_height,
            self.scale_mode,
        );
    }

    pub fn wait_idle(&self) {
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
    }

    pub fn read_modern_gpu_target_rgba(&self) -> Option<Vec<u8>> {
        let target = self.modern_gpu_target.as_ref()?;
        let width = 256u32;
        let height = 224u32;
        let row_bytes = width * 4;
        let readback_bytes_per_row = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_gpu_live_target_readback"),
            size: (readback_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("modern_gpu_live_target_readback"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(readback_bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during live target readback");
        let mapped = slice.get_mapped_range();
        let mut out = Vec::with_capacity((row_bytes * height) as usize);
        let stride = readback_bytes_per_row as usize;
        let row_bytes = row_bytes as usize;
        for row in 0..height as usize {
            out.extend_from_slice(&mapped[row * stride..row * stride + row_bytes]);
        }
        drop(mapped);
        readback.unmap();
        Some(out)
    }

    pub fn read_modern_gpu_screen_pixel(&self, x: u32, y: u32) -> Option<(u32, u32)> {
        self.modern_gpu
            .as_ref()?
            .read_screen_pixel(&self.device, &self.queue, x, y)
    }

    fn maybe_log_viewport(&mut self) {
        if !self.log_viewport {
            return;
        }
        eprintln!(
            "renderer viewport: mode={:?} surface={}x{} game={}x{} viewport={:.1},{:.1} {:.1}x{:.1}",
            self.scale_mode,
            self.config.width,
            self.config.height,
            self.game_width,
            self.game_height,
            self.viewport.x,
            self.viewport.y,
            self.viewport.w,
            self.viewport.h
        );
        self.log_viewport = false;
    }

    /// Upload one frame of pixels. `pixels` must be `game_width * game_height`
    /// packed `u32` values in PPU format `0xFF_RR_GG_BB`. Always native
    /// 256×224 (the classic path never runs at HD scale), but still routed
    /// through [`GameTexture::ensure_size`] so the texture shrinks back down
    /// if a prior modern HD frame grew it.
    pub fn upload_frame(&mut self, pixels: &[u32]) {
        self.game_texture.ensure_size(
            &self.device,
            &self.bind_group_layout,
            &self.presentation_buf,
            self.presentation_params.presentation,
            self.game_width,
            self.game_height,
        );
        upload_ppu_pixels(
            &self.queue,
            &self.game_texture.texture,
            pixels,
            &mut self.upload_buf,
            self.game_width,
            self.game_height,
        );
    }

    pub fn render_menu_overlay(&mut self, menu: &MenuOverlayModel) -> Result<(), RenderError> {
        let pixels = build_menu_overlay_pixels(menu, self.game_width, self.game_height);
        self.upload_frame(&pixels);
        self.render()
    }

    /// Upload an already-RGBA (R,G,B,A byte order) framebuffer straight into the
    /// `Rgba8Unorm` game texture — no BGR→RGB swap (unlike `upload_frame`, whose
    /// input is packed PPU `0xAARRGGBB` u32s). `rgba` must be `width * height * 4`
    /// bytes; the game texture is recreated first if `(width, height)` changed
    /// (e.g. the modern renderer's HD scale), via [`GameTexture::ensure_size`].
    pub fn upload_rgba8(&mut self, rgba: &[u8], width: u32, height: u32) {
        self.game_texture.ensure_size(
            &self.device,
            &self.bind_group_layout,
            &self.presentation_buf,
            self.presentation_params.presentation,
            width,
            height,
        );
        debug_assert_eq!(rgba.len(), (width * height * 4) as usize);
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.game_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Present an already-composited modern frame built by the caller (the
    /// sources+overrides path in `zelda3-bin`'s live present loop, which holds
    /// the CHR-source table `FrameRenderer` can't reach. `width`/`height`
    /// should be `scale*256 × scale*224` for [`FrameRenderer::hd_scale`]; the
    /// game texture is recreated on size change via [`GameTexture::ensure_size`].
    pub fn present_modern_rgba(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.upload_rgba8(rgba, width, height);
        self.render()
    }

    /// Modern source-atlas present path for callers that hold the live CHR
    /// source table. The caller supplies the source table and HD override
    /// context, while the renderer owns composition, scale selection, upload,
    /// and final presentation.
    pub fn present_modern_frame_from_sources<S: modern_extract::SourceTableView + ?Sized>(
        &mut self,
        frame: &GpuFrame<'_>,
        src_table: &S,
        atlas: &modern_source_atlas::ModernSourceAtlas,
        ctx: &modern_hd_overrides::HdOverrideCtx,
    ) -> Result<(), RenderError> {
        let scale = self.hd_scale.get();
        let rgba = modern_extract::render_modern_frame_full_scaled_from_sources(
            frame, src_table, atlas, ctx, scale,
        );
        self.present_modern_rgba(&rgba, 256 * scale, 224 * scale)
    }

    /// Present one live modern-asset frame using the route required by the
    /// active asset mode. The caller supplies game-owned inputs (source table
    /// and semantic scene state); this method owns the route and asset-palette
    /// choices across source-backed variant GPU and source-backed Mode 7.
    /// Indexed GPU diagnostics use lower-level renderer entry points instead.
    /// Frames without source/variant inputs are unhandled instead of falling
    /// back to indexed or live-VRAM rendering.
    pub fn present_modern_asset_frame<S: modern_extract::SourceTableView + ?Sized>(
        &mut self,
        frame: &GpuFrame<'_>,
        src_table: Option<&S>,
        resources: &ModernAssetFrameResources,
        _scene: ModernAssetFrameScene,
    ) -> Result<ModernAssetFramePresentResult, RenderError> {
        // Reset/startup material is an explicit modern-frame input, not Mode-7
        // game art.  The dedicated Mode-7 route uploads its own CPU result and
        // therefore cannot carry this material through its finalizer.  Route
        // it through the normal source compositor instead, which is the same
        // GPU finalizer used by every other source-backed frame.
        if frame.hardware_startup_transient.is_some() {
            self.present_modern_source_gpu_from_sources(
                frame,
                src_table.expect("startup material requires source table"),
                resources
                    .source_atlas()
                    .expect("startup material requires source atlas"),
                resources.variant_atlas(),
            )?;
            return Ok(ModernAssetFramePresentResult::Presented {
                via: "source-gpu-startup-material",
                variant_stats: None,
            });
        }
        let mode7_source_chars = resources.mode7_source_chars();
        match modern_asset_frame_present_route(
            frame.mode,
            src_table.is_some(),
            resources.source_atlas().is_some(),
            resources.variant_atlas().is_some(),
            mode7_source_chars.is_some(),
            resources.has_mode7_source_art(),
            resources.gpu_asset_mode(),
            resources.variant_gpu_mode(),
        ) {
            ModernAssetFramePresentRoute::Mode7SourceGpu => {
                self.present_modern_mode7_source_gpu(
                    frame,
                    mode7_source_chars.expect("route requires Mode 7 source chars"),
                )?;
                Ok(ModernAssetFramePresentResult::Presented {
                    via: "mode7-source-gpu",
                    variant_stats: None,
                })
            }
            ModernAssetFramePresentRoute::SourceVariantGpu => {
                self.present_modern_source_gpu_from_sources(
                    frame,
                    src_table.expect("route requires source table"),
                    resources
                        .source_atlas()
                        .expect("route requires source atlas"),
                    resources.variant_atlas(),
                )?;
                Ok(ModernAssetFramePresentResult::Presented {
                    // Native window and oracle readback deliberately share
                    // this source-resolved compositor. Variant planning remains
                    // authoring/diagnostic-only until it can prove identical
                    // output for every live frame.
                    via: "source-gpu",
                    variant_stats: None,
                })
            }
            ModernAssetFramePresentRoute::Unhandled => Ok(ModernAssetFramePresentResult::Unhandled),
        }
    }

    pub fn present_modern_asset_frame_from_entries<T>(
        &mut self,
        input: ModernAssetFramePresentInput<'_, '_, T>,
    ) -> Result<ModernAssetFramePresentOutput, RenderError>
    where
        T: Copy + Into<(u8, u16, u16)>,
    {
        let src_table = source_table_from_entries(input.source_entries);
        let scene = ModernAssetFrameScene::from_player_indoors_flag(input.player_indoors);
        let result =
            self.present_modern_asset_frame(input.frame, Some(&src_table), input.resources, scene)?;
        Ok(ModernAssetFramePresentOutput {
            result,
            in_dungeon: scene.in_dungeon(),
        })
    }

    /// Present the semantic source atlas through the production GPU compositor.
    /// This is the single live route used both by the native window and by
    /// oracle readback; it does not decode a classic PPU frame or use a
    /// variant/fallback renderer.
    fn present_modern_source_gpu_from_sources<S: modern_extract::SourceTableView + ?Sized>(
        &mut self,
        frame: &GpuFrame<'_>,
        src_table: &S,
        atlas: &modern_source_atlas::ModernSourceAtlas,
        variant_atlas: Option<&modern_variant_atlas::ModernVariantAtlas>,
    ) -> Result<(), RenderError> {
        let (mut modern, mut bg_cells) =
            modern_extract::extract_modern_frame_from_sources(frame, src_table, atlas);
        if let Some(glyph_atlas) =
            variant_atlas.and_then(|atlas| atlas.dialogue_vwf_glyph_atlas.as_ref())
        {
            let visible_runs = modern.vwf_glyph_runs_for_draw().to_vec();
            modern_extract::append_source_vwf_glyph_cells(
                &mut modern,
                &mut bg_cells,
                glyph_atlas,
                &visible_runs,
            );
        }
        let (sprite_cells, sprites) =
            modern_extract::extract_modern_sprites_from_sources(frame, src_table, atlas);
        modern.index_sprites = sprites;
        self.present_modern_gpu(&modern, &bg_cells, &sprite_cells)
    }

    /// Diagnostic GPU present of the indexed source-atlas path
    /// (`ZELDA3_RENDERER=assets-anim-gpu`).
    /// Renders the compositor into an offscreen 256x224 target, then samples
    /// that target directly in the presentation blit. No CPU readback.
    pub fn present_modern_gpu(
        &mut self,
        frame: &modern_frame::ModernFrame,
        bg_cells: &[modern_index_atlas::ModernIndexTile],
        sprite_cells: &[modern_index_atlas::ModernIndexTile],
    ) -> Result<(), RenderError> {
        let format = wgpu::TextureFormat::Rgba8Unorm;
        if self.modern_gpu.is_none() {
            self.modern_gpu = Some(ModernGpuCompositor::new(&self.device, &self.queue, format));
        }
        self.ensure_modern_gpu_target(format);

        let history_rows = active_display_history_rows(frame).filter(|_| {
            self.modern_gpu_target
                .as_ref()
                .is_some_and(|target| target.has_scanout_history)
        });
        if let Some((start_row, row_count)) = history_rows {
            let target = self.modern_gpu_target.as_ref().expect("target built above");
            copy_modern_gpu_target_rows(
                &self.device,
                &self.queue,
                &target.texture,
                &target.scanout_history_texture,
                start_row,
                row_count,
                "modern_gpu_save_active_scanout",
            );
        }

        let compositor = self.modern_gpu.as_ref().expect("compositor built above");
        let target = self.modern_gpu_target.as_ref().expect("target built above");
        compositor.render(
            &self.device,
            &self.queue,
            frame,
            bg_cells,
            sprite_cells,
            &target.texture,
        );
        if let Some((start_row, row_count)) = history_rows {
            copy_modern_gpu_target_rows(
                &self.device,
                &self.queue,
                &target.scanout_history_texture,
                &target.texture,
                start_row,
                row_count,
                "modern_gpu_restore_active_scanout",
            );
        }
        self.modern_gpu_target
            .as_mut()
            .expect("target rendered above")
            .has_scanout_history = true;

        self.present_modern_gpu_target_to_surface()
    }

    /// Diagnostic GPU present of a VRAM-decoded modern frame. The default
    /// modern-asset path uses source-backed PNG/variant art instead.
    pub fn present_modern_gpu_from_vram(
        &mut self,
        frame: &GpuFrame<'_>,
    ) -> Result<(), RenderError> {
        debug_assert_ne!(frame.mode, 7);
        let (mut modern, bg_cells) = modern_extract::extract_modern_frame_from_vram(frame);
        let (sprite_cells, sprites) = modern_extract::extract_modern_sprites_from_vram(frame);
        modern.index_sprites = sprites;
        self.present_modern_gpu(&modern, &bg_cells, &sprite_cells)
    }

    /// Live GPU present of the PNG-atlas path from the source table boundary.
    /// The caller supplies the game-owned CHR source table; the renderer owns
    /// the Mode-1 scene build and GPU present, which keeps the binary out of
    /// the draw pipeline and gives this crate the replacement point for a
    /// future GPU scene builder.
    pub fn present_modern_gpu_from_sources<S: modern_extract::SourceTableView + ?Sized>(
        &mut self,
        frame: &GpuFrame<'_>,
        src_table: &S,
        atlas: &modern_source_atlas::ModernSourceAtlas,
    ) -> Result<(), RenderError> {
        debug_assert_ne!(frame.mode, 7);
        let (mut modern, bg_cells) =
            modern_extract::extract_modern_frame_from_sources(frame, src_table, atlas);
        let (sprite_cells, sprites) =
            modern_extract::extract_modern_sprites_from_sources(frame, src_table, atlas);
        modern.index_sprites = sprites;
        self.present_modern_gpu(&modern, &bg_cells, &sprite_cells)
    }

    /// Live GPU present of the compact RGBA canonical-art/effect atlas path
    /// (`ZELDA3_RENDERER=assets-variant-gpu`, also the default). This keeps the
    /// variant render and final presentation on the live renderer's GPU device:
    /// no headless readback and no CPU RGBA upload.
    pub fn present_modern_variant_gpu(
        &mut self,
        frame: &modern_frame::ModernFrame,
        bg_cells: &[modern_index_atlas::ModernIndexTile],
        sprite_cells: &[modern_index_atlas::ModernIndexTile],
        atlas: &modern_variant_atlas::ModernVariantAtlas,
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> Result<modern_gpu::ModernGpuVariantLiveRender, RenderError> {
        let format = wgpu::TextureFormat::Rgba8Unorm;
        if self.modern_variant_gpu.is_none() {
            self.modern_variant_gpu = Some(ModernGpuVariantRenderer::new(
                &self.device,
                &self.queue,
                atlas,
                format,
            ));
        }
        self.ensure_modern_gpu_target(format);

        let variant = self
            .modern_variant_gpu
            .as_ref()
            .expect("variant renderer built above");
        let target = self.modern_gpu_target.as_ref().expect("target built above");
        let render = variant.render(
            &self.device,
            &self.queue,
            frame,
            bg_cells,
            sprite_cells,
            bg_palette_name,
            sprite_palette_name,
            &target.texture,
        );
        if !render.rendered {
            return Ok(render);
        }

        self.modern_gpu_target
            .as_mut()
            .expect("target rendered above")
            .has_scanout_history = true;
        self.present_modern_gpu_target_to_surface()?;
        Ok(render)
    }

    fn ensure_modern_gpu_target(&mut self, format: wgpu::TextureFormat) {
        if self.modern_gpu_target.is_some() {
            return;
        }
        self.modern_gpu_target = Some(create_modern_gpu_target(
            &self.device,
            &self.bind_group_layout,
            &self.presentation_buf,
            self.presentation_params.presentation,
            format,
        ));
    }

    fn present_modern_gpu_target_to_surface(&mut self) -> Result<(), RenderError> {
        self.maybe_log_viewport();
        if self.presentation_notice.frames_remaining() > 0 {
            self.write_cpu_presentation_params();
        }
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(RenderError::SurfaceReconfigureNeeded);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RenderError::Fatal(
                    "wgpu validation error in get_current_texture".to_string(),
                ));
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("modern_gpu_target_blit"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("modern_gpu_target_blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let target = self.modern_gpu_target.as_ref().expect("target built above");
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &target.bind_group, &[]);
            pass.set_viewport(
                self.viewport.x,
                self.viewport.y,
                self.viewport.w,
                self.viewport.h,
                0.0,
                1.0,
            );
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        surface_texture.present();
        self.tick_presentation_notice();
        Ok(())
    }

    fn modern_live_timings_enabled() -> bool {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var_os("ZELDA3_RENDER_TIMINGS").is_some())
    }

    fn modern_live_timing_mark(last: &mut Option<std::time::Instant>) -> u128 {
        let Some(previous) = last else {
            return 0;
        };
        let elapsed = previous.elapsed().as_micros();
        *previous = std::time::Instant::now();
        elapsed
    }

    fn extract_asset_resolved_modern_frame_from_sources_cached<
        S: modern_extract::SourceTableView + ?Sized,
    >(
        &mut self,
        frame: &GpuFrame<'_>,
        src_table: &S,
        source_atlas: &modern_source_atlas::ModernSourceAtlas,
    ) -> modern_extract::AssetResolvedModernFrame {
        let Some(fingerprint) = modern_source_extraction_fingerprint(frame, src_table) else {
            self.modern_source_extraction_cache = None;
            return modern_extract::extract_asset_resolved_modern_frame_from_sources(
                frame,
                src_table,
                source_atlas,
            );
        };
        if let Some(cache) = &self.modern_source_extraction_cache {
            if cache.fingerprint == fingerprint {
                return cache.assets.clone();
            }
        }
        let assets = modern_extract::extract_asset_resolved_modern_frame_from_sources(
            frame,
            src_table,
            source_atlas,
        );
        self.modern_source_extraction_cache = Some(ModernSourceExtractionCache {
            fingerprint,
            assets: assets.clone(),
        });
        assets
    }

    /// Live GPU present of the compact RGBA canonical-art/effect atlas path
    /// from the source table boundary. This is the default renderer path's
    /// highest-level Mode-1 entry point: zelda3-bin provides current PPU state
    /// plus the CHR source table, and this crate owns extraction, variant
    /// planning, GPU rendering, and final presentation.
    pub fn present_modern_variant_gpu_from_sources<S: modern_extract::SourceTableView + ?Sized>(
        &mut self,
        frame: &GpuFrame<'_>,
        src_table: &S,
        source_atlas: &modern_source_atlas::ModernSourceAtlas,
        variant_atlas: &modern_variant_atlas::ModernVariantAtlas,
        _bg_palette_name: &str,
        _sprite_palette_name: &str,
    ) -> Result<modern_gpu::ModernGpuVariantLiveRender, RenderError> {
        debug_assert_ne!(frame.mode, 7);
        let timings_enabled = Self::modern_live_timings_enabled();
        let mut timing_last = timings_enabled.then(std::time::Instant::now);
        let mut modern_assets = self.extract_asset_resolved_modern_frame_from_sources_cached(
            frame,
            src_table,
            source_atlas,
        );
        if let Some(glyph_atlas) = &variant_atlas.dialogue_vwf_glyph_atlas {
            let visible_runs = modern_assets.frame.vwf_glyph_runs_for_draw().to_vec();
            modern_extract::append_source_vwf_glyph_cells(
                &mut modern_assets.frame,
                &mut modern_assets.bg_cells,
                glyph_atlas,
                &visible_runs,
            );
        }
        let extract_us = Self::modern_live_timing_mark(&mut timing_last);
        if modern_assets.has_unresolved_sources() {
            if timings_enabled {
                eprintln!(
                    "modern_live_timing rendered=false extract_us={extract_us} reason=unresolved_sources"
                );
            }
            return Ok(modern_gpu::ModernGpuVariantLiveRender {
                stats: modern_assets.unresolved_stats,
                rendered: false,
            });
        }
        let format = wgpu::TextureFormat::Rgba8Unorm;
        if self.modern_gpu.is_none() {
            self.modern_gpu = Some(ModernGpuCompositor::new(&self.device, &self.queue, format));
        }
        self.ensure_modern_gpu_target(format);
        let target = self.modern_gpu_target.as_ref().expect("target built above");
        let rendered = self
            .modern_gpu
            .as_ref()
            .expect("modern compositor built above")
            .render(
                &self.device,
                &self.queue,
                &modern_assets.frame,
                &modern_assets.bg_cells,
                &modern_assets.sprite_cells,
                &target.texture,
            );
        let render_us = Self::modern_live_timing_mark(&mut timing_last);
        // The live source-table route above renders the source-index frame
        // through ModernGpuCompositor. Variant-atlas planning is a separate
        // renderer and its missing/dynamic counters do not describe these
        // pixels. Reporting those counters here used to reject fully rendered
        // title/file-entry frames as fallback content.
        let mut stats = modern_software::VariantAtlasRenderStats::default();
        let stats_us = Self::modern_live_timing_mark(&mut timing_last);
        if rendered {
            stats.gpu_prefinal_base_frames += 1;
            stats.gpu_screen_builder_frames += 1;
            self.modern_gpu_target
                .as_mut()
                .expect("target rendered above")
                .has_scanout_history = true;
            self.present_modern_gpu_target_to_surface()?;
        }
        let present_us = Self::modern_live_timing_mark(&mut timing_last);
        if timings_enabled {
            eprintln!(
                "modern_live_timing rendered={rendered} bg_cells={} sprite_cells={} bg_tiles={} sprites={} extract_us={extract_us} render_us={render_us} stats_us={stats_us} present_us={present_us}",
                modern_assets.bg_cells.len(),
                modern_assets.sprite_cells.len(),
                modern_assets
                    .frame
                    .bg_layers
                    .iter()
                    .map(|layer| layer.index_tiles.len())
                    .sum::<usize>(),
                modern_assets.frame.index_sprites.len(),
            );
        }
        Ok(modern_gpu::ModernGpuVariantLiveRender { stats, rendered })
    }

    /// Present a source-backed Mode-7 frame through the live GPU PPU path, then
    /// GPU-copy the native 256x224 result into the standard presentation texture.
    pub fn present_modern_mode7_source_gpu(
        &mut self,
        frame: &GpuFrame<'_>,
        mode7_source_chars: &[u8],
    ) -> Result<(), RenderError> {
        self.present_modern_mode7_gpu_inner(
            frame,
            Some(mode7_source_chars),
            "modern_gpu_mode7_source_live",
        )
    }

    /// Present a Mode-7 frame through the live GPU PPU path, then GPU-copy the
    /// native 256x224 result into the standard presentation texture. This is
    /// used by GPU atlas modes because Mode 7 is not a Mode-1 source-atlas
    /// tilemap, but it still has a real GPU renderer.
    pub fn present_modern_mode7_gpu(&mut self, frame: &GpuFrame<'_>) -> Result<(), RenderError> {
        self.present_modern_mode7_gpu_inner(frame, None, "modern_gpu_mode7_live")
    }

    fn present_modern_mode7_gpu_inner(
        &mut self,
        frame: &GpuFrame<'_>,
        mode7_source_chars: Option<&[u8]>,
        _encoder_label: &'static str,
    ) -> Result<(), RenderError> {
        debug_assert_eq!(frame.mode, 7);
        // Mode 7 is finalized before presentation, so publish its exact RGBA
        // result into the same sampled target used by the source compositor.
        // Native-window readback must observe this target: reading an older
        // compositor target here made the oracle compare a stale black frame
        // while the window sampled the newly uploaded game texture.
        let rgba = modern_mode7_cpu_rgba(frame, mode7_source_chars);
        let format = wgpu::TextureFormat::Rgba8Unorm;
        self.ensure_modern_gpu_target(format);
        let target = self
            .modern_gpu_target
            .as_ref()
            .expect("Mode 7 target allocated above");
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
        );
        self.modern_gpu_target
            .as_mut()
            .expect("target rendered above")
            .has_scanout_history = true;
        self.present_modern_gpu_target_to_surface()
    }

    pub fn render(&mut self) -> Result<(), RenderError> {
        self.maybe_log_viewport();
        if self.presentation_notice.frames_remaining() > 0 {
            self.write_cpu_presentation_params();
        }
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(RenderError::SurfaceReconfigureNeeded);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RenderError::Fatal(
                    "wgpu validation error in get_current_texture".to_string(),
                ));
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("blit"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.game_texture.bind_group, &[]);
            pass.set_viewport(
                self.viewport.x,
                self.viewport.y,
                self.viewport.w,
                self.viewport.h,
                0.0,
                1.0,
            );
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        surface_texture.present();
        self.tick_presentation_notice();
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::{fs, process};

    #[test]
    fn modern_source_fingerprint_distinguishes_obj_latch_only_publication() {
        let vram = vec![0u16; 0x8000];
        let mut decoded_obj = vram.clone();
        decoded_obj[0x4020] = 1;
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let raw_scanlines = [(0, 0, 0, 0, 0, [0; 4], [0; 4], [0; 8], false); 224];
        let registers = GpuFrameRegisterSnapshot {
            vram: &vram,
            oam: &oam,
            mode: 1,
            mode1_bg3_priority: false,
            bg: [BgLayerRegs::default(); 4],
            obj: ObjRegs::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Mode7Regs::default(),
            screen_enabled: [0; 2],
            screen_windowed: [0; 2],
            brightness: 15,
            scanout_brightness_override: None,
            scanout_top_crop: 0,
            forced_blank: false,
            retain_active_display_history: false,
            math_enabled: 0,
            subtract_color: false,
            half_color: false,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            add_subscreen: false,
            clip_mode: 0,
            prevent_math_mode: 0,
            windowsel: 0,
        };
        let capture = |obj_vram| {
            GpuFrame::from_capture_input(GpuFrameCaptureInput {
                hardware_startup_transient: None,
                registers,
                obj_vram,
                bg_vram: None,
                cgram: &cgram,
                raw_scanlines: &raw_scanlines,
                bg3_source_tiles: &[],
                bg3_vwf_glyph_runs: &[],
                dialogue_message_id: None,
                source_dialogue_ir: &[],
                dialogue_ir: &[],
                dialogue_layout: &[],
                dialogue_layout_origin_tile_number: None,
                cgram_provenance: None,
            })
        };
        let source_table = |_: usize| (0, 0, 0);

        let raw = modern_source_extraction_fingerprint(&capture(None), &source_table).unwrap();
        let latched = modern_source_extraction_fingerprint(
            &capture(Some(decoded_obj.as_slice())),
            &source_table,
        )
        .unwrap();

        assert_ne!(raw, latched);
    }

    #[test]
    fn modern_source_fingerprint_distinguishes_bg_latch_only_publication() {
        let vram = vec![0u16; 0x8000];
        let mut decoded_bg = vram.clone();
        decoded_bg[0x3b20] = 1;
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let raw_scanlines = [(0, 0, 0, 0, 0, [0; 4], [0; 4], [0; 8], false); 224];
        let registers = GpuFrameRegisterSnapshot {
            vram: &vram,
            oam: &oam,
            mode: 1,
            mode1_bg3_priority: false,
            bg: [BgLayerRegs::default(); 4],
            obj: ObjRegs::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Mode7Regs::default(),
            screen_enabled: [0; 2],
            screen_windowed: [0; 2],
            brightness: 15,
            scanout_brightness_override: None,
            scanout_top_crop: 0,
            forced_blank: false,
            retain_active_display_history: false,
            math_enabled: 0,
            subtract_color: false,
            half_color: false,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            add_subscreen: false,
            clip_mode: 0,
            prevent_math_mode: 0,
            windowsel: 0,
        };
        let capture = |bg_vram| {
            GpuFrame::from_capture_input(GpuFrameCaptureInput {
                hardware_startup_transient: None,
                registers,
                obj_vram: None,
                bg_vram,
                cgram: &cgram,
                raw_scanlines: &raw_scanlines,
                bg3_source_tiles: &[],
                bg3_vwf_glyph_runs: &[],
                dialogue_message_id: None,
                source_dialogue_ir: &[],
                dialogue_ir: &[],
                dialogue_layout: &[],
                dialogue_layout_origin_tile_number: None,
                cgram_provenance: None,
            })
        };
        let source_table = |_: usize| (0, 0, 0);

        let raw = modern_source_extraction_fingerprint(&capture(None), &source_table).unwrap();
        let latched = modern_source_extraction_fingerprint(
            &capture(Some(decoded_bg.as_slice())),
            &source_table,
        )
        .unwrap();

        assert_ne!(raw, latched);
    }

    #[test]
    fn active_display_history_rows_preserve_only_the_visible_gap() {
        let mut frame = modern_frame::ModernFrame::empty();
        frame.forced_blank_scanlines = 2;
        frame.forced_blank_from_scanline = Some(7);
        frame.retain_active_display_history = true;

        assert_eq!(active_display_history_rows(&frame), Some((2, 5)));
    }

    #[test]
    fn active_display_history_rows_preserve_the_whole_snes9x_force_blank_surface() {
        let mut frame = modern_frame::ModernFrame::empty();
        frame.forced_blank = true;
        frame.forced_blank_scanlines = 224;
        frame.retain_active_display_history = true;

        assert_eq!(active_display_history_rows(&frame), Some((0, 224)));
    }

    #[test]
    fn active_display_history_rows_require_nmi_memory_publication() {
        let mut frame = modern_frame::ModernFrame::empty();
        assert_eq!(active_display_history_rows(&frame), None);

        frame.forced_blank_scanlines = 2;
        frame.forced_blank_from_scanline = Some(7);
        assert_eq!(active_display_history_rows(&frame), None);

        frame.retain_active_display_history = true;
        frame.forced_blank_scanlines = 7;
        frame.forced_blank_from_scanline = Some(7);
        assert_eq!(active_display_history_rows(&frame), None);
    }

    fn temp_modern_asset_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("z3rs-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create temp root");
        root
    }

    #[test]
    fn source_backed_tiles_do_not_become_missing_art_without_a_baked_variant() {
        assert!(source_backed_missing_art_is_resolvable("missing-art", 0));
        assert!(!source_backed_missing_art_is_resolvable("missing-art", 1));
        assert!(!source_backed_missing_art_is_resolvable("cpu-fallback", 0));
    }

    fn test_variant_atlas_with_mode7_chars(
        chars: Option<Vec<u8>>,
    ) -> modern_variant_atlas::ModernVariantAtlas {
        modern_variant_atlas::ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
            mode7_source_chars: chars,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
        }
    }

    #[test]
    fn modern_asset_resources_skip_classic_atlases() {
        let root = temp_modern_asset_root("modern-asset-classic");

        let resources = ModernAssetFrameResources::load_for_mode(
            EffectiveRendererMode::from_name("classic"),
            &root,
        )
        .expect("classic loads no modern asset atlases");

        assert!(resources.source_atlas().is_none());
        assert!(resources.variant_atlas().is_none());
        assert!(resources.unhandled_gpu_asset_frame_line().is_none());

        fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn modern_asset_resources_own_unhandled_gpu_asset_failure_line() {
        let resources = ModernAssetFrameResources {
            source_atlas: None,
            variant_atlas: None,
            variant_headless: None,
            gpu_asset_mode: true,
            variant_gpu_mode: true,
        };

        assert_eq!(
            resources.unhandled_gpu_asset_frame_line(),
            Some("modern asset renderer did not handle a GPU asset frame")
        );
    }

    #[test]
    fn unsupported_asset_present_result_is_not_presented() {
        let result = ModernAssetFramePresentResult::Unsupported {
            via: "variant-gpu",
            variant_stats: Some(modern_software::VariantAtlasRenderStats {
                live_index_draws: 1,
                ..Default::default()
            }),
        };

        assert!(!result.is_presented());
    }

    #[test]
    fn variant_gpu_mode7_chars_come_from_canonical_atlas() {
        let atlas_chars = vec![7u8; 0x4000];
        let resources = ModernAssetFrameResources {
            source_atlas: None,
            variant_atlas: Some(test_variant_atlas_with_mode7_chars(Some(atlas_chars))),
            variant_headless: None,
            gpu_asset_mode: true,
            variant_gpu_mode: true,
        };

        let resolved = resources
            .mode7_source_chars()
            .expect("variant atlas chars should resolve");

        assert_eq!(resolved[0], 7);
    }

    #[test]
    fn variant_gpu_mode7_chars_do_not_fall_back_to_capture_bytes() {
        let resources = ModernAssetFrameResources {
            source_atlas: None,
            variant_atlas: Some(test_variant_atlas_with_mode7_chars(None)),
            variant_headless: None,
            gpu_asset_mode: true,
            variant_gpu_mode: true,
        };

        assert!(resources.mode7_source_chars().is_none());
    }

    #[test]
    fn explicit_indexed_gpu_mode7_does_not_use_asset_resource_chars() {
        let resources = ModernAssetFrameResources {
            source_atlas: None,
            variant_atlas: None,
            variant_headless: None,
            gpu_asset_mode: true,
            variant_gpu_mode: false,
        };

        assert!(resources.mode7_source_chars().is_none());
    }

    #[test]
    fn modern_asset_resources_require_canonical_art_atlas_for_variant_gpu() {
        let root = temp_modern_asset_root("modern-asset-variant-missing");

        let err = match ModernAssetFrameResources::load_for_mode(
            EffectiveRendererMode::from_name("assets-variant-gpu"),
            &root,
        ) {
            Ok(_) => panic!("variant GPU requires canonical art atlas"),
            Err(err) => err,
        };

        assert!(err.contains("canonical art atlas missing"), "{err}");

        fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn modern_asset_resources_skip_indexed_gpu_atlases() {
        let root = temp_modern_asset_root("modern-asset-indexed-gpu");

        let resources = ModernAssetFrameResources::load_for_mode(
            EffectiveRendererMode::from_name("assets-anim-gpu"),
            &root,
        )
        .expect("indexed GPU is diagnostic-only for modern asset resources");

        assert!(resources.source_atlas().is_none());
        assert!(resources.variant_atlas().is_none());

        fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn modern_index_compare_resources_skip_when_compare_disabled() {
        let root = temp_modern_asset_root("modern-index-disabled");

        let resources = ModernIndexCompareResources::load_for_mode(
            false,
            EffectiveRendererMode::from_name("assets-variant-gpu"),
            &root,
            false,
        )
        .expect("disabled compare loads no resources");

        assert!(resources.source_atlas().is_none());
        assert!(resources.gpu_headless().is_none());
        assert!(resources.variant_headless().is_none());

        fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn full_gpu_asset_readback_requires_variant_atlas_resources() {
        let resources = ModernIndexCompareResources {
            source_atlas: None,
            gpu_headless: None,
            variant_headless: None,
        };
        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let frame = GpuFrame {
            hardware_startup_transient: None,
            vram: &vram,
            obj_vram: None,
            bg_vram: None,
            cgram: &cgram,
            oam: &oam,
            mode: 1,
            mode1_bg3_priority: false,
            bg: Default::default(),
            obj: Default::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Default::default(),
            screen_enabled: [0, 0],
            screen_windowed: [0, 0],
            brightness: 15,
            scanout_brightness_override: None,
            scanout_top_crop: 0,
            forced_blank: false,
            retain_active_display_history: false,
            math_enabled: 0,
            subtract_color: false,
            half_color: false,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            add_subscreen: false,
            clip_mode: 0,
            prevent_math_mode: 0,
            windowsel_cm: 0,
            windowsel: 0,
            scanlines: Box::new([gpu_frame::ScanlineRegs::default(); 224]),
            bg3_source_tiles: &[],
            bg3_vwf_glyph_runs: &[],
            dialogue_message_id: None,
            source_dialogue_ir: &[],
            dialogue_ir: &[],
            dialogue_layout: &[],
            dialogue_layout_origin_tile_number: None,
            cgram_provenance: None,
        };
        let entries: [(u8, u16, u16); 0] = [];

        let err = match resources.render_full_gpu_asset_rgba_from_entries(
            &frame,
            &entries,
            ModernAssetFrameScene::from_in_dungeon(false),
        ) {
            Ok(_) => panic!("full asset readback must not use indexed GPU resources"),
            Err(err) => err,
        };

        assert!(
            err.contains("requires canonical RGBA variant atlas"),
            "{err}"
        );
    }

    #[test]
    fn full_gpu_asset_validation_requires_variant_atlas_resources() {
        let resources = ModernIndexCompareResources {
            source_atlas: None,
            gpu_headless: None,
            variant_headless: None,
        };
        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let frame = GpuFrame {
            hardware_startup_transient: None,
            vram: &vram,
            obj_vram: None,
            bg_vram: None,
            cgram: &cgram,
            oam: &oam,
            mode: 1,
            mode1_bg3_priority: false,
            bg: Default::default(),
            obj: Default::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Default::default(),
            screen_enabled: [0, 0],
            screen_windowed: [0, 0],
            brightness: 15,
            scanout_brightness_override: None,
            scanout_top_crop: 0,
            forced_blank: false,
            retain_active_display_history: false,
            math_enabled: 0,
            subtract_color: false,
            half_color: false,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            add_subscreen: false,
            clip_mode: 0,
            prevent_math_mode: 0,
            windowsel_cm: 0,
            windowsel: 0,
            scanlines: Box::new([gpu_frame::ScanlineRegs::default(); 224]),
            bg3_source_tiles: &[],
            bg3_vwf_glyph_runs: &[],
            dialogue_message_id: None,
            source_dialogue_ir: &[],
            dialogue_ir: &[],
            dialogue_layout: &[],
            dialogue_layout_origin_tile_number: None,
            cgram_provenance: None,
        };
        let entries: [(u8, u16, u16); 0] = [];

        let err = match resources.validate_full_gpu_asset_from_entries(
            &frame,
            &entries,
            ModernAssetFrameScene::from_in_dungeon(false),
        ) {
            Ok(_) => panic!("full asset validation must not use indexed GPU resources"),
            Err(err) => err,
        };

        assert!(
            err.contains("requires canonical RGBA variant atlas"),
            "{err}"
        );
    }

    #[test]
    fn modern_atlas_compare_resources_skip_when_compare_disabled() {
        let root = temp_modern_asset_root("modern-atlas-disabled");

        let resources = ModernAtlasCompareResources::load(false, &root)
            .expect("disabled atlas compare loads no resources");

        assert!(resources.atlas().is_none());

        fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn modern_atlas_compare_frame_line_matches_legacy_output() {
        let result = modern_gpu::ModernAtlasCompareResult {
            classic_hash: 0x1234_abcd,
            modern_hash: 0x5678_ef90,
            matches: false,
            render: modern_gpu::ModernAtlasCompareRender {
                rgba: Vec::new(),
                hash: 0x5678_ef90,
                via: "atlas-software",
            },
        };

        assert_eq!(
            modern_atlas_compare_frame_line(42, &result),
            "modern_render_compare frame=42 old=0x1234abcd modern=0x5678ef90 match=false"
        );
    }

    #[test]
    fn modern_index_compare_resource_plan_covers_gpu_and_cpu_routes() {
        assert_eq!(
            modern_index_compare_resource_plan(
                true,
                EffectiveRendererMode::from_name("assets-anim-gpu"),
                false,
            ),
            ModernIndexCompareResourcePlan {
                load_source_atlas: true,
                load_variant_atlas: false,
                load_gpu_headless: true,
            }
        );
        assert_eq!(
            modern_index_compare_resource_plan(
                true,
                EffectiveRendererMode::from_name("assets-variant-gpu"),
                false,
            ),
            ModernIndexCompareResourcePlan {
                load_source_atlas: true,
                load_variant_atlas: true,
                load_gpu_headless: true,
            }
        );
        assert_eq!(
            modern_index_compare_resource_plan(
                true,
                EffectiveRendererMode::from_name("assets-anim"),
                true,
            ),
            ModernIndexCompareResourcePlan {
                load_source_atlas: true,
                load_variant_atlas: false,
                load_gpu_headless: false,
            }
        );
        assert_eq!(
            modern_index_compare_resource_plan(
                true,
                EffectiveRendererMode::from_name("assets-anim"),
                false,
            ),
            ModernIndexCompareResourcePlan {
                load_source_atlas: false,
                load_variant_atlas: false,
                load_gpu_headless: false,
            }
        );
    }

    #[test]
    fn modern_asset_frame_scene_owns_palette_names() {
        let overworld = ModernAssetFrameScene::from_in_dungeon(false);
        assert!(!overworld.in_dungeon());
        assert_eq!(overworld.bg_palette_name(), "palette_overworld_bg_main");
        assert_eq!(overworld.sprite_palette_name(), "palette_main_spr");

        let dungeon = ModernAssetFrameScene::from_in_dungeon(true);
        assert!(dungeon.in_dungeon());
        assert_eq!(dungeon.bg_palette_name(), "palette_dung_bg_main");
        assert_eq!(dungeon.sprite_palette_name(), "palette_main_spr");

        assert!(!ModernAssetFrameScene::from_player_indoors_flag(0).in_dungeon());
        assert!(ModernAssetFrameScene::from_player_indoors_flag(2).in_dungeon());
    }

    #[test]
    fn modern_index_compare_scene_owns_route_labels_and_asset_scene() {
        let overworld = ModernIndexCompareScene::from_main_module_and_player_indoors_flag(9, 0);
        assert_eq!(overworld.mode_label(), "ow");
        assert!(!overworld.asset_scene().in_dungeon());

        let overworld_alt =
            ModernIndexCompareScene::from_main_module_and_player_indoors_flag(11, 0);
        assert_eq!(overworld_alt.mode_label(), "ow");

        let dungeon = ModernIndexCompareScene::from_main_module_and_player_indoors_flag(7, 1);
        assert_eq!(dungeon.mode_label(), "dungeon");
        assert!(dungeon.asset_scene().in_dungeon());

        let dungeon_alt = ModernIndexCompareScene::from_main_module_and_player_indoors_flag(16, 1);
        assert_eq!(dungeon_alt.mode_label(), "dungeon");

        let fallback = ModernIndexCompareScene::from_main_module_and_player_indoors_flag(3, 0);
        assert_eq!(fallback.mode_label(), "mod3");
    }

    #[test]
    fn modern_asset_frame_route_keeps_default_paths_on_gpu() {
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, true, true, true, true, true),
            ModernAssetFramePresentRoute::Mode7SourceGpu
        );
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, true, true, false, true, true),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, true, false, true, true, true),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, true, false, false, true, true),
            ModernAssetFramePresentRoute::SourceVariantGpu
        );
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, true, false, false, false, true),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, false, false, false, true, true),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(1, false, false, false, false, false, true, true),
            ModernAssetFramePresentRoute::Unhandled
        );
    }

    #[test]
    fn modern_asset_frame_route_preserves_source_backed_mode7_path() {
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, false, false, false, true, false),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, false, true, false, true, false),
            ModernAssetFramePresentRoute::Mode7SourceGpu
        );
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, false, false, false, true, false),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, true, false, false, true, false),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(1, false, false, false, false, false, true, false),
            ModernAssetFramePresentRoute::Unhandled
        );
    }

    #[test]
    fn modern_asset_frame_route_rejects_non_gpu_asset_fallbacks() {
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, false, false, false, false, false),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, false, true, false, false, false),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, true, true, true, false, true),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(1, false, false, false, false, false, false, false),
            ModernAssetFramePresentRoute::Unhandled
        );
    }

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.05,
            "expected {actual} to be near {expected}"
        );
    }

    #[test]
    fn viewport_exact_fit() {
        let vp = compute_viewport(768, 672, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.w, 768.0);
        assert_eq!(vp.h, 672.0);
    }

    #[test]
    fn viewport_letterbox_wide() {
        // scale = min(1000/256, 672/224) = min(3, 3) = 3; x-bar = (1000-768)/2 = 116
        let vp = compute_viewport(1000, 672, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.x, 116.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.w, 768.0);
        assert_eq!(vp.h, 672.0);
    }

    #[test]
    fn viewport_letterbox_tall() {
        // scale = 3; y-bar = (800-672)/2 = 64
        let vp = compute_viewport(768, 800, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 64.0);
        assert_eq!(vp.w, 768.0);
        assert_eq!(vp.h, 672.0);
    }

    #[test]
    fn viewport_scale_one_when_surface_smaller_than_game() {
        // scale clamps to 1; viewport is clamped to surface size, offset is 0
        let vp = compute_viewport(100, 100, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.w, 100.0);
        assert_eq!(vp.h, 100.0);
    }

    #[test]
    fn viewport_scale_limited_by_height() {
        // scale = min(1280/256, 720/224) = min(5, 3) = 3
        let vp = compute_viewport(1280, 720, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.w, 768.0);
        assert_eq!(vp.h, 672.0);
    }

    #[test]
    fn viewport_fit_fills_steam_deck_height_without_cropping() {
        let vp = compute_viewport(1280, 800, 256, 224, ViewportScaleMode::Fit);
        assert_near(vp.x, 182.86);
        assert_eq!(vp.y, 0.0);
        assert_near(vp.w, 914.29);
        assert_eq!(vp.h, 800.0);
    }

    #[test]
    fn viewport_stretch_uses_full_surface() {
        let vp = compute_viewport(1280, 800, 256, 224, ViewportScaleMode::Stretch);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.w, 1280.0);
        assert_eq!(vp.h, 800.0);
    }

    #[test]
    fn presentation_mode_defaults_to_off() {
        assert_eq!(PresentationMode::from_value(None), PresentationMode::Off);
        assert_eq!(
            PresentationMode::from_value(Some("bogus")),
            PresentationMode::Off
        );
    }

    #[test]
    fn presentation_mode_accepts_named_shader_presets() {
        assert_eq!(
            PresentationMode::from_value(Some("sharp")),
            PresentationMode::Sharp
        );
        assert_eq!(
            PresentationMode::from_value(Some("crt")),
            PresentationMode::Crt
        );
        assert_eq!(
            PresentationMode::from_value(Some("soft")),
            PresentationMode::Off
        );
    }

    #[test]
    fn presentation_mode_cycles_through_runtime_hotkey_order() {
        assert_eq!(PresentationMode::Off.next(), PresentationMode::Sharp);
        assert_eq!(PresentationMode::Sharp.next(), PresentationMode::Crt);
        assert_eq!(PresentationMode::Crt.next(), PresentationMode::Off);
    }

    #[test]
    fn presentation_enhancement_modes_default_to_off() {
        assert_eq!(LightingMode::from_value(None), LightingMode::Off);
        assert_eq!(
            LightingMode::from_value(Some("ambient")),
            LightingMode::Ambient
        );
        assert_eq!(
            LightingMode::from_value(Some("dynamic")),
            LightingMode::Dynamic
        );
        assert_eq!(LightingMode::from_value(Some("bogus")), LightingMode::Off);

        assert_eq!(ShadowMode::from_value(None), ShadowMode::Off);
        assert_eq!(ShadowMode::from_value(Some("soft")), ShadowMode::Off);
        assert_eq!(ShadowMode::from_value(Some("raycast")), ShadowMode::Raycast);
        assert_eq!(ShadowMode::from_value(Some("bogus")), ShadowMode::Off);
    }

    #[test]
    fn lighting_and_shadow_modes_cycle_through_runtime_hotkey_order() {
        assert_eq!(LightingMode::Off.next(), LightingMode::Ambient);
        assert_eq!(LightingMode::Ambient.next(), LightingMode::Dynamic);
        assert_eq!(LightingMode::Dynamic.next(), LightingMode::Off);

        assert_eq!(ShadowMode::Off.next(), ShadowMode::Raycast);
        assert_eq!(ShadowMode::Raycast.next(), ShadowMode::Off);
    }

    #[test]
    fn presentation_params_pack_shader_mode_words() {
        assert_eq!(
            PresentationParams::new(PresentationMode::Off, LightingMode::Off, ShadowMode::Off)
                .as_words(),
            [0, 0, 0, 0]
        );
        assert_eq!(
            PresentationParams::new(
                PresentationMode::Crt,
                LightingMode::Dynamic,
                ShadowMode::Raycast,
            )
            .as_words(),
            [2, 2, 2, 0]
        );
    }

    #[test]
    fn runtime_settings_map_to_renderer_presentation_params() {
        let settings = RendererRuntimeSettings {
            presentation: RendererPresentationChoice::Crt,
            lighting: RendererLightingChoice::Dynamic,
            shadows: RendererShadowChoice::Raycast,
            viewport: RendererViewportChoice::Integer,
        };
        let params = PresentationParams::from_runtime_settings(settings);
        assert_eq!(params.presentation, PresentationMode::Crt);
        assert_eq!(params.lighting, LightingMode::Dynamic);
        assert_eq!(params.shadows, ShadowMode::Raycast);
    }

    #[test]
    fn menu_overlay_lines_use_resume_first_play_tab() {
        let menu = MenuOverlayModel::resume_first_play_tab();
        let lines = menu_overlay_lines(&menu);
        assert_eq!(lines[0], "PLAY  VIDEO  CONTROLS  DEV MAP");
        assert_eq!(lines[1], "> RESUME QUEST");
        assert!(lines.iter().any(|line| *line == "DEVELOPER MAP"));
    }

    #[test]
    fn menu_overlay_pixels_include_panel_border_and_text() {
        let menu = MenuOverlayModel::resume_first_play_tab();
        let pixels = build_menu_overlay_pixels(&menu, 256, 224);
        assert_eq!(pixels.len(), 256 * 224);
        assert!(pixels.contains(&MENU_COLOR_PANEL));
        assert!(pixels.contains(&MENU_COLOR_BORDER));
        assert!(pixels.contains(&MENU_COLOR_TEXT));
    }

    #[test]
    fn developer_menu_overlay_paints_thumbnail_and_detail_panel() {
        let menu = MenuOverlayModel {
            tab: MenuOverlayTab::DeveloperMap,
            selected_index: 0,
            lines: vec![
                "PLAY  VIDEO  CONTROLS  DEV MAP",
                "> SANCTUARY",
                "  FILE SELECT",
            ],
            detail_lines: vec![
                "SANCTUARY".to_string(),
                "ROOM 0050".to_string(),
                "VERIFIED".to_string(),
            ],
            thumbnail: Some(MenuOverlayThumbnail::Sanctuary),
        };

        let pixels = build_menu_overlay_pixels(&menu, 256, 224);
        assert!(pixels.contains(&MENU_COLOR_THUMB_STONE));
        assert!(pixels.contains(&MENU_COLOR_THUMB_LIGHT));
        assert!(pixels.contains(&MENU_COLOR_TEXT));
    }

    #[test]
    fn every_developer_thumbnail_variant_paints_preview_pixels() {
        let thumbnails = [
            MenuOverlayThumbnail::RouteStart,
            MenuOverlayThumbnail::FileSelect,
            MenuOverlayThumbnail::Sanctuary,
            MenuOverlayThumbnail::LateDungeon,
            MenuOverlayThumbnail::DevRoom,
            MenuOverlayThumbnail::LockedOverworld,
            MenuOverlayThumbnail::LockedDungeon,
        ];

        for thumbnail in thumbnails {
            let mut pixels = vec![0u32; 80 * 50];
            draw_thumbnail(&mut pixels, 80, 50, 10, 8, thumbnail);
            assert!(pixels.contains(&MENU_COLOR_BORDER));
            assert!(
                pixels
                    .iter()
                    .any(|pixel| *pixel != 0 && *pixel != MENU_COLOR_BORDER),
                "{thumbnail:?} did not draw distinct thumbnail pixels"
            );
        }
    }

    #[test]
    fn presentation_params_cycle_one_runtime_dimension_at_a_time() {
        let mut params =
            PresentationParams::new(PresentationMode::Off, LightingMode::Off, ShadowMode::Off);

        params.cycle_presentation();
        assert_eq!(
            params,
            PresentationParams::new(PresentationMode::Sharp, LightingMode::Off, ShadowMode::Off)
        );

        params.cycle_lighting();
        assert_eq!(
            params,
            PresentationParams::new(
                PresentationMode::Sharp,
                LightingMode::Ambient,
                ShadowMode::Off,
            )
        );

        params.cycle_shadows();
        assert_eq!(
            params,
            PresentationParams::new(
                PresentationMode::Sharp,
                LightingMode::Ambient,
                ShadowMode::Raycast,
            )
        );
    }

    #[test]
    fn presentation_notice_codes_track_hotkey_result_and_expire() {
        let mut notice = PresentationNotice::default();

        notice.show_presentation(PresentationMode::Crt);
        assert_eq!(notice.code(), 3);
        assert_eq!(notice.frames_remaining(), PRESENTATION_NOTICE_FRAMES);

        for _ in 0..PRESENTATION_NOTICE_FRAMES {
            notice.tick();
        }
        assert_eq!(notice.code(), 0);
        assert_eq!(notice.frames_remaining(), 0);

        notice.show_lighting(LightingMode::Dynamic);
        assert_eq!(notice.code(), 12);
        notice.show_shadows(ShadowMode::Raycast);
        assert_eq!(notice.code(), 22);
        notice.show_viewport(ViewportScaleMode::Fit);
        assert_eq!(notice.code(), 31);
    }

    #[test]
    fn presentation_shader_applies_bloom_color_grade_only_after_enhanced_sampling() {
        let shader = include_str!("blit.wgsl");

        assert!(shader.contains("fn apply_bloom_color_grade("));
        assert!(shader.contains("color = apply_bloom_color_grade(color, in.uv);"));
        assert!(shader.contains("if params.presentation == 1u"));
        assert!(shader.contains("} else {\n        color = textureSample(t_game, s_nearest, in.uv).rgb;\n    }\n    if params.presentation != 0u"));
    }

    #[test]
    fn presentation_shader_renders_hotkey_notice_text() {
        let shader = include_str!("blit.wgsl");

        assert!(shader.contains("notice_code: u32"));
        assert!(shader.contains("fn notice_char_code("));
        assert!(shader.contains("fn apply_notice_overlay("));
        assert!(shader.contains("color = apply_notice_overlay(color, in.pos.xy);"));
    }

    #[test]
    fn raycast_shadow_shader_softens_occlusion_with_multiple_ray_taps() {
        let shader = include_str!("blit.wgsl");

        assert!(shader.contains("fn soft_ray_shadow("));
        assert!(shader.contains("ray_shadow(light, uv + normal * 0.004)"));
        assert!(shader.contains("ray_shadow(light, uv - normal * 0.004)"));
        assert!(shader.contains(
            "ray_shadow_amount = max(ray_shadow_amount, soft_ray_shadow(params.lights[i], uv));"
        ));
        assert!(shader.contains("if params.shadows == 2u && params.lighting == 2u"));
    }

    #[test]
    fn dynamic_lighting_shader_samples_low_res_light_mask() {
        let shader = include_str!("blit.wgsl");

        assert!(shader.contains("light_mask: array<vec4<f32>, 56>"));
        assert!(shader.contains("fn sample_light_mask("));
        assert!(shader.contains("fn light_mask_cell("));
        assert!(shader.contains("let lift = sample_light_mask(uv);"));
        assert!(shader.contains("let warm_lift = vec3<f32>(1.0, 0.86, 0.58) * lift * 0.18;"));
        assert!(shader.contains("return color * (ambient + lift * 0.20) + warm_lift;"));
    }

    #[test]
    fn presentation_uniform_bytes_include_light_count_and_vectors() {
        let params = PresentationParams::new(
            PresentationMode::Crt,
            LightingMode::Dynamic,
            ShadowMode::Raycast,
        );
        let light = PresentationLight {
            x: 0.25,
            y: 0.5,
            radius: 0.2,
            intensity: 0.4,
        };

        let bytes = build_presentation_uniform_bytes(
            params,
            PresentationContext::default(),
            &[light],
            &[0; PRESENTATION_OCCLUDER_WORDS],
        );

        assert_eq!(bytes.len(), PRESENTATION_UNIFORM_BYTES);
        assert_eq!(u32::from_ne_bytes(bytes[0..4].try_into().unwrap()), 2);
        assert_eq!(u32::from_ne_bytes(bytes[4..8].try_into().unwrap()), 2);
        assert_eq!(u32::from_ne_bytes(bytes[8..12].try_into().unwrap()), 2);
        assert_eq!(u32::from_ne_bytes(bytes[12..16].try_into().unwrap()), 1);
        assert_eq!(u32::from_ne_bytes(bytes[16..20].try_into().unwrap()), 0);
        assert_eq!(f32::from_ne_bytes(bytes[32..36].try_into().unwrap()), 0.25);
        assert_eq!(f32::from_ne_bytes(bytes[36..40].try_into().unwrap()), 0.5);
        assert_eq!(f32::from_ne_bytes(bytes[40..44].try_into().unwrap()), 0.2);
        assert_eq!(f32::from_ne_bytes(bytes[44..48].try_into().unwrap()), 0.4);
    }

    #[test]
    fn low_res_light_mask_uses_soft_radial_falloff() {
        let light = PresentationLight {
            x: 0.5,
            y: 0.5,
            radius: 0.25,
            intensity: 0.6,
        };

        let mask = build_low_res_light_mask(&[light]);
        let center = mask[(PRESENTATION_LIGHT_MASK_GRID_H / 2) * PRESENTATION_LIGHT_MASK_GRID_W
            + PRESENTATION_LIGHT_MASK_GRID_W / 2];
        let corner = mask[0];

        assert_eq!(mask.len(), PRESENTATION_LIGHT_MASK_CELLS);
        assert!(center > 0.45, "center lift was {center}");
        assert_eq!(corner, 0.0);
    }

    #[test]
    fn presentation_uniform_packs_low_res_light_mask_after_occluders() {
        let params = PresentationParams::new(
            PresentationMode::Crt,
            LightingMode::Dynamic,
            ShadowMode::Off,
        );
        let light = PresentationLight {
            x: 0.5,
            y: 0.5,
            radius: 0.25,
            intensity: 0.6,
        };

        let bytes = build_presentation_uniform_bytes(
            params,
            PresentationContext::default(),
            &[light],
            &[0; PRESENTATION_OCCLUDER_WORDS],
        );
        let mask_offset = 32 + MAX_PRESENTATION_LIGHTS * 16 + PRESENTATION_OCCLUDER_WORDS * 4;
        let mask = &bytes[mask_offset..];
        let center_index = (PRESENTATION_LIGHT_MASK_GRID_H / 2) * PRESENTATION_LIGHT_MASK_GRID_W
            + PRESENTATION_LIGHT_MASK_GRID_W / 2;
        let center_offset = center_index * 4;
        let center = f32::from_ne_bytes(mask[center_offset..center_offset + 4].try_into().unwrap());

        assert_eq!(bytes.len(), PRESENTATION_UNIFORM_BYTES);
        assert_eq!(mask.len(), PRESENTATION_LIGHT_MASK_VECS * 16);
        assert!(center > 0.45, "center mask lift was {center}");
    }

    #[test]
    fn presentation_uniform_marks_dungeon_context() {
        let params = PresentationParams::new(
            PresentationMode::Off,
            LightingMode::Ambient,
            ShadowMode::Off,
        );
        let context = PresentationContext { in_dungeon: true };

        let bytes = build_presentation_uniform_bytes(
            params,
            context,
            &[],
            &[0; PRESENTATION_OCCLUDER_WORDS],
        );

        assert_eq!(u32::from_ne_bytes(bytes[16..20].try_into().unwrap()), 1);
    }

    #[test]
    fn upload_frame_swaps_bgr_to_rgb() {
        // PPU pixel 0xFF_10_20_30 (R=0x10, G=0x20, B=0x30)
        // to_le_bytes → [0x30, 0x20, 0x10, 0xFF]
        // after swap  → [0x10, 0x20, 0x30, 0xFF] (RGBA for Rgba8Unorm)
        let pixels = [0xFF_10_20_30u32];
        let mut buf = vec![0u8; 4];
        for (dst, &src) in buf.chunks_exact_mut(4).zip(pixels.iter()) {
            let [b, g, r, a] = src.to_le_bytes();
            dst[0] = r;
            dst[1] = g;
            dst[2] = b;
            dst[3] = a;
        }
        assert_eq!(buf, [0x10, 0x20, 0x30, 0xFF]);
    }

    #[test]
    fn offscreen_readback_row_alignment() {
        // 256px wide: 256*4=1024, already a multiple of 256
        assert_eq!(
            (256u32 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            1024
        );
        // 100px wide: 100*4=400, next multiple of 256 is 512
        assert_eq!(
            (100u32 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            512
        );
        // 64px wide: 64*4=256, already aligned
        assert_eq!(
            (64u32 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            256
        );
    }

    // `FrameRenderer` itself needs a live winit window (real `wgpu::Surface`), so
    // it can't be constructed in a headless test; exercise the size-aware
    // texture-recreation logic it delegates to (`GameTexture::ensure_size`)
    // directly against a device/queue, the same headless pattern
    // `render_test_bg_with_rgba_override` uses for `GpuFrameRenderer`.
    #[test]
    fn game_texture_recreates_only_on_size_change() {
        let instance = create_wgpu_instance();
        let (_adapter, device, queue) = pollster::block_on(create_device_queue(&instance, None));
        let presentation_params = PresentationParams::from_env();
        let (texture, bind_group_layout, bind_group, presentation_buf) =
            create_game_texture_resources(&device, 256, 224, presentation_params);
        let mut game_texture = GameTexture {
            texture,
            bind_group,
            width: 256,
            height: 224,
        };
        assert_eq!(game_texture.texture.size().width, 256);
        assert_eq!(game_texture.texture.size().height, 224);

        // A live 2× HD frame (`render_modern_frame_full_scaled(…, 2)`) is
        // 512×448 — the size the texture must grow to.
        let hd_rgba = crate::modern_software::render_modern_frame_full_scaled(
            &crate::modern_frame::ModernFrame::empty(),
            &[],
            &[],
            &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
            2,
        );
        assert_eq!(hd_rgba.len(), 512 * 448 * 4);

        let recreated = game_texture.ensure_size(
            &device,
            &bind_group_layout,
            &presentation_buf,
            presentation_params.presentation,
            512,
            448,
        );
        assert!(recreated, "texture must recreate when the frame size grows");
        assert_eq!(game_texture.texture.size().width, 512);
        assert_eq!(game_texture.texture.size().height, 448);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &game_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &hd_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(512 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 512,
                height: 448,
                depth_or_array_layers: 1,
            },
        );

        // Guard: re-requesting the SAME size must not recreate (no per-frame churn).
        let recreated_again = game_texture.ensure_size(
            &device,
            &bind_group_layout,
            &presentation_buf,
            presentation_params.presentation,
            512,
            448,
        );
        assert!(
            !recreated_again,
            "unchanged size must not recreate the texture"
        );
    }
}

/// Render a Mode-7 `GpuFrame` with the modern CPU compositor. When
/// `mode7_source_chars` is provided (the source-art route), the chars replace
/// the CHR bytes (high byte of each of the first 0x4000 VRAM words) exactly as
/// the retired GPU source-chars texture did; the tilemap low bytes stay live.
pub fn modern_mode7_cpu_rgba(frame: &GpuFrame<'_>, mode7_source_chars: Option<&[u8]>) -> Vec<u8> {
    match mode7_source_chars {
        None => crate::modern_software::render_modern_mode7_frame(frame),
        Some(chars) => {
            if std::env::var_os("ZELDA3_DEBUG_MODE7_SOURCE_AUDIT").is_some() {
                let mismatches = frame
                    .vram
                    .iter()
                    .take(0x4000)
                    .zip(chars.iter())
                    .filter(|(word, source)| ((**word >> 8) as u8) != **source)
                    .count();
                if mismatches != 0 {
                    use std::io::Write;
                    if let Ok(mut trace) = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("/tmp/zelda3-mode7-source-audit.trace")
                    {
                        let first = frame
                            .vram
                            .iter()
                            .take(0x4000)
                            .zip(chars.iter())
                            .position(|(word, source)| ((word >> 8) as u8) != *source)
                            .unwrap_or(0);
                        let _ = writeln!(
                            trace,
                            "mismatches={mismatches} first={first:04x} live={:02x} source={:02x}",
                            (frame.vram[first] >> 8) as u8,
                            chars[first],
                        );
                    }
                }
            }
            let mut vram = frame.vram.to_vec();
            for (word, &ch) in vram.iter_mut().take(0x4000).zip(chars.iter()) {
                *word = (*word & 0x00ff) | (u16::from(ch) << 8);
            }
            let mut patched: GpuFrame<'_> = frame.clone();
            patched.vram = &vram;
            crate::modern_software::render_modern_mode7_frame(&patched)
        }
    }
}
