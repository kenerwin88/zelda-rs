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

pub mod bg_layer;
mod frame_compare;
pub mod gpu_frame;
mod gpu_frame_render_plan;
mod gpu_frame_renderer_backend;
mod gpu_frame_renderer_resources;
mod gpu_frame_work_command;
pub mod gpu_renderer;
mod gpu_work_item;
pub mod hd_authoring;
pub mod mode7_renderer;
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
mod modern_sprite_renderer;
pub mod modern_variant_atlas;
pub mod modern_variant_draw;
mod modern_variant_render_plan;
pub mod post_process;
pub mod renderer_mode;
pub mod sprite_renderer;
pub mod tile_atlas;

pub use bg_layer::BgLayerRenderer;
pub use frame_compare::{
    compare_bgra_to_rgba, compare_rgba_to_rgba, render_frame_rgb_hash_bgra,
    render_frame_rgb_hash_rgba, GpuRenderDiff,
};
pub use gpu_frame::{
    BgLayerRegs, GpuFrame, GpuFrameSource, Mode7Regs, ObjRegs, RawScanlineFrame, RawScanlineRegs,
    ScanlineRegs,
};
pub use gpu_renderer::GpuFrameRenderer;
pub use mode7_renderer::Mode7Renderer;
pub use modern_extract::MappedSourceTableView;
pub use modern_gpu::{
    ModernGpuCompositor, ModernGpuHeadless, ModernGpuVariantHeadless, ModernGpuVariantRenderer,
};
pub use modern_index_compare_stats::{
    compare_modern_index_rgba, ModernIndexCompareFrameDiff, ModernIndexCompareFrameLine,
    ModernIndexComparePixelDiff, ModernIndexCompareStats,
};
pub use modern_live_stats::ModernAssetLiveStats;
pub use post_process::scanlines_from_raw;
pub use renderer_mode::{
    default_renderer_env_for_variant_setting, renderer_env_or_default, source_atlas_renderer_mode,
    variant_atlas_renderer_mode, EffectiveRendererMode, RendererMode,
};
pub use tile_atlas::{
    CgramPalette, RgbaTileOverrideData, TileAtlas, ATLAS_HEIGHT, ATLAS_TILE_COUNT, ATLAS_WIDTH,
    RGBA_TILE_OVERRIDE_LOOKUP_COUNT,
};

use std::{
    env,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtSidecarConfig {
    manifest_path: Option<PathBuf>,
}

impl ArtSidecarConfig {
    fn from_env() -> Self {
        Self::from_value(env::var_os("ZELDA3_ART_SIDECARS").as_deref())
    }

    fn from_value(value: Option<&std::ffi::OsStr>) -> Self {
        Self {
            manifest_path: value.map(PathBuf::from),
        }
    }

    #[cfg(test)]
    fn enabled(&self) -> bool {
        self.manifest_path.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct ArtSidecarManifest {
    tiles: Vec<ArtSidecarTile>,
    /// Path (relative to the manifest) to the 256×1 RGBA PNG holding the CGRAM the HD
    /// override art was authored against. The shader recolors overrides as
    /// `live_cgram[idx] * (override / reference_cgram[idx])`, so the reference lets HD
    /// art track the runtime palette. Absent → detail-modulate falls back to a 1×1
    /// placeholder (only matters once real overrides ship).
    #[serde(default)]
    reference_palette: Option<String>,
}

impl ArtSidecarManifest {
    fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct ArtSidecarTile {
    tile: u16,
    normal: Option<String>,
    depth: Option<String>,
    rgba: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtSidecarImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtSidecarTileAssets {
    tile: u16,
    normal: Option<ArtSidecarImage>,
    depth: Option<ArtSidecarImage>,
    rgba: Option<ArtSidecarImage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArtSidecarAtlasEntry {
    tile: u16,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtSidecarRgbaAtlas {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    lookup: Vec<ArtSidecarAtlasEntry>,
    /// Reference CGRAM (256 × RGBA8 = 1024 bytes) the overrides were authored against,
    /// carried through to `RgbaTileOverrideData` for detail-modulated recolor. Empty
    /// when the manifest omits a reference palette.
    reference_cgram: Vec<u8>,
}

impl ArtSidecarRgbaAtlas {
    #[cfg(test)]
    fn texture_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    #[cfg(test)]
    fn bytes_per_row(&self) -> u32 {
        self.width * 4
    }

    #[cfg(test)]
    fn upload_byte_len(&self) -> usize {
        (self.width * self.height * 4) as usize
    }

    fn lookup_texture_pixels(&self) -> Vec<[u32; 4]> {
        let mut lookup = vec![[0u32; 4]; RGBA_TILE_OVERRIDE_LOOKUP_COUNT];
        for entry in &self.lookup {
            let tile = usize::from(entry.tile);
            if tile < lookup.len() {
                lookup[tile] = [entry.x, entry.y, entry.width, entry.height];
            }
        }
        lookup
    }

    fn as_tile_override_data<'a>(&'a self, lookup: &'a [[u32; 4]]) -> RgbaTileOverrideData<'a> {
        RgbaTileOverrideData {
            width: self.width,
            height: self.height,
            rgba: &self.rgba,
            lookup,
            // Sourced from the sidecar manifest's `reference_palette` (a 256×1 RGBA
            // PNG). Empty when the manifest omits it → 1×1 placeholder (the shader's
            // detail-modulate path only fires where an override tile exists, and
            // today's parity runs load no sidecar at all).
            reference_cgram: &self.reference_cgram,
        }
    }

    #[cfg(test)]
    fn lookup_for_tile(&self, tile: u16) -> Option<ArtSidecarAtlasEntry> {
        self.lookup.iter().copied().find(|entry| entry.tile == tile)
    }
}

#[derive(Debug, Default)]
struct ArtSidecarAssets {
    _manifest: Option<ArtSidecarManifest>,
    tiles: Vec<ArtSidecarTileAssets>,
    /// The reference CGRAM (256 entries × RGBA8 = 1024 bytes) the HD overrides were
    /// authored against, decoded from `manifest.reference_palette`. Empty when the
    /// manifest omits it (or it fails to load) → the override recolor uses a 1×1
    /// placeholder reference.
    reference_cgram: Vec<u8>,
}

impl ArtSidecarAssets {
    fn load(config: &ArtSidecarConfig) -> Self {
        let Some(path) = &config.manifest_path else {
            return Self::default();
        };
        match std::fs::read_to_string(path) {
            Ok(json) => match ArtSidecarManifest::from_json(&json) {
                Ok(manifest) => {
                    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
                    let tiles = manifest
                        .tiles
                        .iter()
                        .map(|tile| ArtSidecarTileAssets {
                            tile: tile.tile,
                            normal: load_optional_sidecar_image(base_dir, tile.normal.as_deref()),
                            depth: load_optional_sidecar_image(base_dir, tile.depth.as_deref()),
                            rgba: load_optional_sidecar_image(base_dir, tile.rgba.as_deref()),
                        })
                        .collect();
                    let reference_cgram =
                        load_reference_cgram(base_dir, manifest.reference_palette.as_deref());
                    Self {
                        _manifest: Some(manifest),
                        tiles,
                        reference_cgram,
                    }
                }
                Err(err) => {
                    eprintln!(
                        "failed to parse ZELDA3_ART_SIDECARS manifest {}: {err}",
                        path.display()
                    );
                    Self::default()
                }
            },
            Err(err) => {
                eprintln!(
                    "failed to read ZELDA3_ART_SIDECARS manifest {}: {err}",
                    path.display()
                );
                Self::default()
            }
        }
    }

    #[cfg(test)]
    fn enabled(&self) -> bool {
        self._manifest.is_some()
    }

    fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    fn build_rgba_override_atlas(&self) -> Option<ArtSidecarRgbaAtlas> {
        let overrides: Vec<_> = self
            .tiles
            .iter()
            .filter_map(|tile| tile.rgba.as_ref().map(|image| (tile.tile, image)))
            .collect();
        if overrides.is_empty() {
            return None;
        }

        let width = overrides.iter().map(|(_, image)| image.width).sum();
        let height = overrides
            .iter()
            .map(|(_, image)| image.height)
            .max()
            .unwrap_or(0);
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        let mut lookup = Vec::with_capacity(overrides.len());
        let mut x = 0u32;
        for (tile, image) in overrides {
            for row in 0..image.height {
                let dst = (((row * width) + x) * 4) as usize;
                let src = (row * image.width * 4) as usize;
                let len = (image.width * 4) as usize;
                rgba[dst..dst + len].copy_from_slice(&image.rgba[src..src + len]);
            }
            lookup.push(ArtSidecarAtlasEntry {
                tile,
                x,
                y: 0,
                width: image.width,
                height: image.height,
            });
            x += image.width;
        }
        Some(ArtSidecarRgbaAtlas {
            width,
            height,
            rgba,
            lookup,
            reference_cgram: self.reference_cgram.clone(),
        })
    }
}

/// Decode the reference-palette PNG (expected 256×1 RGBA, the authoring CGRAM) into a
/// flat 1024-byte RGBA8 buffer. Returns empty on any problem (absent path, load error,
/// or wrong pixel count) so the detail-modulate path degrades to its 1×1 placeholder
/// rather than mis-recoloring.
fn load_reference_cgram(base_dir: &Path, path: Option<&str>) -> Vec<u8> {
    let Some(path) = path else {
        return Vec::new();
    };
    let resolved = base_dir.join(path);
    match load_art_sidecar_image(&resolved) {
        Ok(image) if (image.width * image.height) as usize == 256 && image.rgba.len() == 1024 => {
            image.rgba
        }
        Ok(image) => {
            eprintln!(
                "ZELDA3_ART_SIDECARS reference_palette {} must be 256 RGBA pixels \
                 (got {}×{}); ignoring",
                resolved.display(),
                image.width,
                image.height
            );
            Vec::new()
        }
        Err(err) => {
            eprintln!(
                "failed to load ZELDA3_ART_SIDECARS reference_palette {}: {err}",
                resolved.display()
            );
            Vec::new()
        }
    }
}

fn load_optional_sidecar_image(base_dir: &Path, path: Option<&str>) -> Option<ArtSidecarImage> {
    let path = path?;
    let resolved = base_dir.join(path);
    match load_art_sidecar_image(&resolved) {
        Ok(image) => Some(image),
        Err(err) => {
            eprintln!(
                "failed to load ZELDA3_ART_SIDECARS image {}: {err}",
                resolved.display()
            );
            None
        }
    }
}

fn load_art_sidecar_image(path: &Path) -> Result<ArtSidecarImage, String> {
    let file = File::open(path).map_err(|err| err.to_string())?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().map_err(|err| err.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|err| err.to_string())?;
    if info.width == 0 || info.height == 0 {
        return Err("image dimensions must be nonzero".to_string());
    }
    let bytes = &buffer[..info.buffer_size()];
    let rgba = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => bytes.to_vec(),
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for rgb in bytes.chunks_exact(3) {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 0xff]);
            }
            rgba
        }
        (png::ColorType::Grayscale, png::BitDepth::Eight) => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for &luma in bytes {
                rgba.extend_from_slice(&[luma, luma, luma, 0xff]);
            }
            rgba
        }
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for ga in bytes.chunks_exact(2) {
                rgba.extend_from_slice(&[ga[0], ga[0], ga[0], ga[1]]);
            }
            rgba
        }
        (color, depth) => {
            return Err(format!("unsupported PNG format {color:?}/{depth:?}"));
        }
    };
    let expected_len = (info.width * info.height * 4) as usize;
    if rgba.len() != expected_len {
        return Err(format!(
            "decoded RGBA length {} did not match expected {expected_len}",
            rgba.len()
        ));
    }
    Ok(ArtSidecarImage {
        width: info.width,
        height: info.height,
        rgba,
    })
}

const MAX_PRESENTATION_LIGHTS: usize = 8;
const PRESENTATION_LIGHT_MASK_GRID_W: usize = 16;
const PRESENTATION_LIGHT_MASK_GRID_H: usize = 14;
const PRESENTATION_LIGHT_MASK_CELLS: usize =
    PRESENTATION_LIGHT_MASK_GRID_W * PRESENTATION_LIGHT_MASK_GRID_H;
const PRESENTATION_LIGHT_MASK_VECS: usize = PRESENTATION_LIGHT_MASK_CELLS / 4;
const PRESENTATION_OCCLUDER_GRID_W: usize = 16;
const PRESENTATION_OCCLUDER_GRID_H: usize = 14;
const PRESENTATION_OCCLUDER_BITS: usize =
    PRESENTATION_OCCLUDER_GRID_W * PRESENTATION_OCCLUDER_GRID_H;
const PRESENTATION_OCCLUDER_WORDS: usize = 8;
const PRESENTATION_UNIFORM_BYTES: usize = 32
    + MAX_PRESENTATION_LIGHTS * 16
    + PRESENTATION_OCCLUDER_WORDS * 4
    + PRESENTATION_LIGHT_MASK_VECS * 16;
const PRESENTATION_SPRITE_SIZES: [[u8; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct PresentationLight {
    x: f32,
    y: f32,
    radius: f32,
    intensity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PresentationLightProfile {
    radius: f32,
    intensity: f32,
}

fn sprite_tile_light_profile(tile: u8, palette_sub: u8) -> Option<PresentationLightProfile> {
    let profile = match tile {
        // Small flame/spark/fireball tiles.
        0x58..=0x5f => PresentationLightProfile {
            radius: 0.16,
            intensity: 0.40,
        },
        // Larger sword-beam, magic, lamp/torch-flame-like effect tiles.
        0x78..=0x7f => PresentationLightProfile {
            radius: 0.24,
            intensity: 0.55,
        },
        _ if palette_sub >= 4 => PresentationLightProfile {
            radius: 0.18,
            intensity: 0.38,
        },
        _ => return None,
    };
    Some(profile)
}

fn extract_presentation_lights(
    oam: &[u16],
    obj: &ObjRegs,
    lighting: LightingMode,
) -> Vec<PresentationLight> {
    if lighting != LightingMode::Dynamic {
        return Vec::new();
    }

    let sizes = PRESENTATION_SPRITE_SIZES[obj.obj_size as usize & 7];
    let mut lights = Vec::new();
    for sprite_num in 0..128usize {
        if lights.len() == MAX_PRESENTATION_LIGHTS {
            break;
        }

        let idx = sprite_num * 2;
        let oam0 = oam.get(idx).copied().unwrap_or(0);
        let oam1 = oam.get(idx + 1).copied().unwrap_or(0);
        let y = (((oam0 >> 8) as i32) + 1) & 0xff;
        if y == 0xf0 {
            continue;
        }

        let palette_sub = ((oam1 & 0x0e00) >> 9) as u8;
        let tile = (oam1 & 0xff) as u8;
        let Some(profile) = sprite_tile_light_profile(tile, palette_sub) else {
            continue;
        };

        let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
        let hi_bits = (hi_word >> (idx % 16)) as i32;
        let sprite_size = sizes[((hi_bits >> 1) & 1) as usize] as i32;
        let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
        if object_x > 256 && object_x + sprite_size - 1 < 512 {
            continue;
        }

        let mut x = object_x;
        if x >= 256 {
            x -= 512;
        }
        if x <= -sprite_size || x >= 256 {
            continue;
        }

        let top = y - 1;
        if top <= -sprite_size || top >= 224 {
            continue;
        }

        let center_x = (x + sprite_size / 2).clamp(0, 255) as f32 / 256.0;
        let center_y = (top + sprite_size / 2).clamp(0, 223) as f32 / 224.0;
        let size_scale = (sprite_size as f32 / 16.0).clamp(0.75, 2.0);
        lights.push(PresentationLight {
            x: center_x,
            y: center_y,
            radius: profile.radius * size_scale,
            intensity: profile.intensity * size_scale.min(1.5),
        });
    }
    lights
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

fn extract_shadow_occluder_words(
    vram: &[u16],
    bg: &BgLayerRegs,
    shadows: ShadowMode,
) -> [u32; PRESENTATION_OCCLUDER_WORDS] {
    if shadows == ShadowMode::Off {
        return [0; PRESENTATION_OCCLUDER_WORDS];
    }

    let mut words = [0u32; PRESENTATION_OCCLUDER_WORDS];
    for cell_y in 0..PRESENTATION_OCCLUDER_GRID_H {
        for cell_x in 0..PRESENTATION_OCCLUDER_GRID_W {
            let mut occluded = false;
            for sub_y in 0..2 {
                for sub_x in 0..2 {
                    let screen_x = (cell_x * 16 + sub_x * 8 + 4) as u32;
                    let screen_y = (cell_y * 16 + sub_y * 8 + 4) as u32;
                    let map_px_w = if bg.tilemap_wider { 512 } else { 256 };
                    let map_px_h = if bg.tilemap_higher { 512 } else { 256 };
                    let world_x = (screen_x + u32::from(bg.h_scroll)) % map_px_w;
                    let world_y = (screen_y + u32::from(bg.v_scroll)) % map_px_h;
                    let tile_x = world_x / 8;
                    let tile_y = world_y / 8;
                    let page_offset = if tile_x >= 32 && bg.tilemap_wider {
                        0x400
                    } else {
                        0
                    } + if tile_y >= 32 && bg.tilemap_higher {
                        if bg.tilemap_wider {
                            0x800
                        } else {
                            0x400
                        }
                    } else {
                        0
                    };
                    let local_x = tile_x % 32;
                    let local_y = tile_y % 32;
                    let vram_idx = bg.tilemap_adr as usize
                        + page_offset as usize
                        + (local_y * 32 + local_x) as usize;
                    let entry = vram.get(vram_idx & 0x7fff).copied().unwrap_or(0);
                    occluded |= entry & 0x2000 != 0;
                }
            }
            if !occluded {
                continue;
            }
            let bit = cell_y * PRESENTATION_OCCLUDER_GRID_W + cell_x;
            debug_assert!(bit < PRESENTATION_OCCLUDER_BITS);
            words[bit / 32] |= 1u32 << (bit % 32);
        }
    }
    words
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

/// Blits a CPU BGRA framebuffer to a winit window surface each frame.
pub struct FrameRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    game_texture: GameTexture,
    bind_group_layout: wgpu::BindGroupLayout,
    presentation_buf: wgpu::Buffer,
    gpu_renderer: GpuFrameRenderer,
    _gpu_texture: wgpu::Texture,
    gpu_view: wgpu::TextureView,
    gpu_bind_group: wgpu::BindGroup,
    gpu_presentation_buf: wgpu::Buffer,
    _art_sidecars: ArtSidecarAssets,
    _art_sidecar_rgba_atlas: Option<ArtSidecarRgbaAtlas>,
    upload_buf: Vec<u8>,
    game_width: u32,
    game_height: u32,
    scale_mode: ViewportScaleMode,
    presentation_params: PresentationParams,
    presentation_notice: PresentationNotice,
    viewport: Viewport,
    log_viewport: bool,
    /// Integer HD scale for the modern (off-VRAM) live render path
    /// (`ZELDA3_HD_SCALE`, default 2); read once at construction so every
    /// `render_modern_frame` and `present_modern_frame_from_sources` call uses
    /// one consistent size.
    hd_scale: modern_hd_overrides::HdScale,
    /// Lazily built on first `present_modern_gpu` call (assets-anim-gpu mode).
    modern_gpu: Option<ModernGpuCompositor>,
    /// Lazily built on first `present_modern_variant_gpu` call
    /// (`assets-variant-gpu`, the default modern asset path).
    modern_variant_gpu: Option<ModernGpuVariantRenderer>,
    /// Offscreen Rgba8Unorm 256x224 target the compositor renders into before
    /// it is GPU-copied into `game_texture` and blit by `render()`.
    modern_gpu_target: Option<(wgpu::Texture, wgpu::TextureView)>,
}

#[derive(Debug, Default)]
pub enum ModernAssetFramePresentResult {
    Presented {
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

/// Renderer-owned resource bundle for live modern-asset presentation.
///
/// `zelda3-bin` owns game state and runtime inputs; the renderer owns which
/// modern asset stores a mode requires and how those stores are routed.
pub struct ModernAssetFrameResources {
    source_atlas: Option<modern_source_atlas::ModernSourceAtlas>,
    variant_atlas: Option<modern_variant_atlas::ModernVariantAtlas>,
    hd_overrides: Option<modern_hd_overrides::ModernHdOverrides>,
    gpu_asset_mode: bool,
}

impl ModernAssetFrameResources {
    pub fn load_for_mode(mode: EffectiveRendererMode<'_>, root: &Path) -> Result<Self, String> {
        let variant_atlas = if mode.uses_variant_atlas() {
            Some(modern_variant_atlas::load_modern_canonical_art_atlas(root)?)
        } else {
            None
        };
        let source_atlas = if mode.uses_source_atlas() {
            Some(
                modern_source_atlas::load_modern_source_atlas(root)
                    .map_err(|e| format!("assets-by-source atlas missing: {e}"))?,
            )
        } else {
            None
        };

        Ok(Self {
            source_atlas,
            variant_atlas,
            hd_overrides: modern_hd_overrides::ModernHdOverrides::from_env(),
            gpu_asset_mode: mode.uses_gpu_assets(),
        })
    }

    pub fn source_atlas(&self) -> Option<&modern_source_atlas::ModernSourceAtlas> {
        self.source_atlas.as_ref()
    }

    pub fn variant_atlas(&self) -> Option<&modern_variant_atlas::ModernVariantAtlas> {
        self.variant_atlas.as_ref()
    }

    pub fn gpu_asset_mode(&self) -> bool {
        self.gpu_asset_mode
    }

    pub fn hd_override_ctx(&self) -> modern_hd_overrides::HdOverrideCtx<'_> {
        match &self.hd_overrides {
            Some(store) => modern_hd_overrides::HdOverrideCtx::new(store),
            None => modern_hd_overrides::HdOverrideCtx::disabled(),
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModernAssetFramePresentRoute {
    Mode7Gpu,
    SourceVariantGpu,
    SourceGpu,
    SourceSoftware,
    VramGpu,
    Unhandled,
}

fn modern_asset_frame_present_route(
    frame_mode: u8,
    has_src_table: bool,
    has_source_atlas: bool,
    has_variant_atlas: bool,
    gpu_asset_mode: bool,
) -> ModernAssetFramePresentRoute {
    if frame_mode == 7 {
        return if gpu_asset_mode {
            ModernAssetFramePresentRoute::Mode7Gpu
        } else {
            ModernAssetFramePresentRoute::Unhandled
        };
    }

    if has_src_table && has_source_atlas {
        if has_variant_atlas {
            return ModernAssetFramePresentRoute::SourceVariantGpu;
        }
        if gpu_asset_mode {
            return ModernAssetFramePresentRoute::SourceGpu;
        }
        return ModernAssetFramePresentRoute::SourceSoftware;
    }

    if gpu_asset_mode {
        ModernAssetFramePresentRoute::VramGpu
    } else {
        ModernAssetFramePresentRoute::Unhandled
    }
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
        let art_sidecars = ArtSidecarAssets::load(&ArtSidecarConfig::from_env());
        let art_sidecar_rgba_atlas = art_sidecars.build_rgba_override_atlas();
        let art_sidecar_rgba_lookup = art_sidecar_rgba_atlas
            .as_ref()
            .map(ArtSidecarRgbaAtlas::lookup_texture_pixels);
        let art_sidecar_rgba_override = art_sidecar_rgba_atlas
            .as_ref()
            .zip(art_sidecar_rgba_lookup.as_deref())
            .map(|(atlas, lookup)| atlas.as_tile_override_data(lookup));
        if env::var_os("ZELDA3_ART_SIDECAR_LOG").is_some() {
            eprintln!("renderer art sidecars: tiles={}", art_sidecars.tile_count());
            if let Some(atlas) = &art_sidecar_rgba_atlas {
                eprintln!(
                    "renderer art sidecar rgba atlas: overrides={} size={}x{}",
                    atlas.lookup.len(),
                    atlas.width,
                    atlas.height
                );
            }
        }
        let (texture, bind_group_layout, bind_group, presentation_buf) =
            create_game_texture_resources(&device, game_width, game_height, presentation_params);
        let game_texture = GameTexture {
            texture,
            bind_group,
            width: game_width,
            height: game_height,
        };
        let pipeline = create_blit_pipeline(&device, &bind_group_layout, surface_format);
        let gpu_renderer =
            GpuFrameRenderer::new(&device, &queue, art_sidecar_rgba_override.as_ref().copied());
        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gpu_game_frame"),
            size: wgpu::Extent3d {
                width: game_width,
                height: game_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let gpu_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let gpu_sampler =
            create_presentation_sampler(&device, presentation_params.presentation, "gpu_blit");
        let gpu_presentation_buf =
            create_presentation_buffer(&device, presentation_params, "gpu_presentation_params");
        let gpu_bind_group = create_blit_bind_group(
            &device,
            &bind_group_layout,
            &gpu_view,
            &gpu_sampler,
            &gpu_presentation_buf,
            "gpu_blit",
        );
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
            gpu_renderer,
            _gpu_texture: gpu_texture,
            gpu_view,
            gpu_bind_group,
            gpu_presentation_buf,
            _art_sidecars: art_sidecars,
            _art_sidecar_rgba_atlas: art_sidecar_rgba_atlas,
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
        }
    }

    /// Current live HD scale (`ZELDA3_HD_SCALE`, default 2), cached at
    /// construction. Callers that render the modern sources+overrides path
    /// themselves (see module docs on `hd_scale` field) use this so their
    /// finished RGBA is sized consistently with `render_modern_frame`'s own
    /// VRAM-only fallback.
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

        let gpu_sampler = create_presentation_sampler(
            &self.device,
            self.presentation_params.presentation,
            "gpu_blit",
        );
        self.gpu_bind_group = create_blit_bind_group(
            &self.device,
            &self.bind_group_layout,
            &self.gpu_view,
            &gpu_sampler,
            &self.gpu_presentation_buf,
            "gpu_blit",
        );
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

    /// Modern (software) live-VRAM render path: the fallback used when the
    /// caller can't supply the sources+overrides render (that path needs the
    /// CHR-source table, which lives on the zelda3 `GameState` this crate
    /// can't depend on — see [`FrameRenderer::present_modern_frame_from_sources`]
    /// for the source-table boundary). Decodes BG + sprites from the live
    /// `GpuFrame` VRAM, composites at [`FrameRenderer::hd_scale`] (N× nearest,
    /// no HD overrides — those are source-keyed and VRAM-decoded cells never
    /// carry a source key), then uploads the resulting `scale*256 × scale*224`
    /// RGBA and blits it with the standard presentation pipeline (`render()`).
    /// Mode 7 (not a Mode-1 tilemap) routes through the dedicated CPU
    /// compositor, nearest-upscaled to match. Default/Classic callers are
    /// unaffected.
    pub fn render_modern_frame(&mut self, frame: &GpuFrame<'_>) -> Result<(), RenderError> {
        let scale = self.hd_scale.get();
        let rgba = if frame.mode == 7 {
            let native = crate::modern_software::render_modern_mode7_frame(frame);
            crate::modern_software::upscale_rgba_nearest(&native, 256, 224, scale as usize)
        } else {
            let (mut modern, bg_cells) =
                crate::modern_extract::extract_modern_frame_from_vram(frame);
            let (sprite_cells, sprites) =
                crate::modern_extract::extract_modern_sprites_from_vram(frame);
            modern.index_sprites = sprites;
            crate::modern_software::render_modern_frame_full_scaled(
                &modern,
                &bg_cells,
                &sprite_cells,
                &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
                scale,
            )
        };
        self.upload_rgba8(&rgba, 256 * scale, 224 * scale);
        self.render()
    }

    /// Present an already-composited modern frame built by the caller (the
    /// sources+overrides path in `zelda3-bin`'s live present loop, which holds
    /// the CHR-source table `FrameRenderer` can't reach — see
    /// [`FrameRenderer::render_modern_frame`]'s docs). `width`/`height` should
    /// be `scale*256 × scale*224` for [`FrameRenderer::hd_scale`] so present
    /// stays consistent with the VRAM-only fallback; the game texture is
    /// recreated on size change via [`GameTexture::ensure_size`].
    pub fn present_modern_rgba(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.upload_rgba8(rgba, width, height);
        self.render()
    }

    /// Modern (software) source-atlas present path for callers that hold the
    /// live CHR source table. This is the source-backed sibling of
    /// [`FrameRenderer::render_modern_frame`]: the caller supplies the source
    /// table and HD override context, while the renderer owns composition,
    /// scale selection, upload, and final presentation.
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

    /// Present one live modern-asset frame using the highest available renderer
    /// path. The caller supplies game-owned inputs (source table and semantic
    /// scene state); this method owns the route and asset-palette choices across
    /// Mode 7 GPU, source variant GPU, source GPU, source software, and VRAM GPU
    /// fallback.
    pub fn present_modern_asset_frame<S: modern_extract::SourceTableView + ?Sized>(
        &mut self,
        frame: &GpuFrame<'_>,
        src_table: Option<&S>,
        resources: &ModernAssetFrameResources,
        scene: ModernAssetFrameScene,
    ) -> Result<ModernAssetFramePresentResult, RenderError> {
        match modern_asset_frame_present_route(
            frame.mode,
            src_table.is_some(),
            resources.source_atlas().is_some(),
            resources.variant_atlas().is_some(),
            resources.gpu_asset_mode(),
        ) {
            ModernAssetFramePresentRoute::Mode7Gpu => {
                self.present_modern_mode7_gpu(frame)?;
                Ok(ModernAssetFramePresentResult::Presented {
                    variant_stats: None,
                })
            }
            ModernAssetFramePresentRoute::SourceVariantGpu => {
                let stats = self.present_modern_variant_gpu_from_sources(
                    frame,
                    src_table.expect("route requires source table"),
                    resources
                        .source_atlas()
                        .expect("route requires source atlas"),
                    resources
                        .variant_atlas()
                        .expect("route requires variant atlas"),
                    scene.bg_palette_name(),
                    scene.sprite_palette_name(),
                )?;
                Ok(ModernAssetFramePresentResult::Presented {
                    variant_stats: Some(stats),
                })
            }
            ModernAssetFramePresentRoute::SourceGpu => {
                self.present_modern_gpu_from_sources(
                    frame,
                    src_table.expect("route requires source table"),
                    resources
                        .source_atlas()
                        .expect("route requires source atlas"),
                )?;
                Ok(ModernAssetFramePresentResult::Presented {
                    variant_stats: None,
                })
            }
            ModernAssetFramePresentRoute::SourceSoftware => {
                let ctx = resources.hd_override_ctx();
                self.present_modern_frame_from_sources(
                    frame,
                    src_table.expect("route requires source table"),
                    resources
                        .source_atlas()
                        .expect("route requires source atlas"),
                    &ctx,
                )?;
                Ok(ModernAssetFramePresentResult::Presented {
                    variant_stats: None,
                })
            }
            ModernAssetFramePresentRoute::VramGpu => {
                self.present_modern_gpu_from_vram(frame)?;
                Ok(ModernAssetFramePresentResult::Presented {
                    variant_stats: None,
                })
            }
            ModernAssetFramePresentRoute::Unhandled => Ok(ModernAssetFramePresentResult::Unhandled),
        }
    }

    /// Live GPU present of the PNG-atlas path (`ZELDA3_RENDERER=assets-anim-gpu`).
    /// Renders the compositor into an offscreen 256x224 target, GPU-copies it
    /// into `game_texture`, then blits via the standard presentation path. No
    /// CPU readback.
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
        if self.modern_gpu_target.is_none() {
            let target = self.device.create_texture(&wgpu::TextureDescriptor {
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
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());
            self.modern_gpu_target = Some((target, view));
        }

        self.game_texture.ensure_size(
            &self.device,
            &self.bind_group_layout,
            &self.presentation_buf,
            self.presentation_params.presentation,
            256,
            224,
        );

        let compositor = self.modern_gpu.as_ref().expect("compositor built above");
        let (target_texture, _target_view) =
            self.modern_gpu_target.as_ref().expect("target built above");
        compositor.render(
            &self.device,
            &self.queue,
            frame,
            bg_cells,
            sprite_cells,
            target_texture,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("modern_gpu_copy_to_game_texture"),
            });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: target_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.game_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        self.render()
    }

    /// Live GPU present of a VRAM-decoded modern frame. This is the fallback
    /// GPU asset path when the caller has no source atlas loaded: the renderer
    /// owns VRAM extraction plus GPU presentation, keeping the binary out of
    /// ModernFrame/cell assembly.
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
    ) -> Result<modern_software::VariantAtlasRenderStats, RenderError> {
        let format = wgpu::TextureFormat::Rgba8Unorm;
        if self.modern_variant_gpu.is_none() {
            self.modern_variant_gpu = Some(ModernGpuVariantRenderer::new(
                &self.device,
                &self.queue,
                atlas,
                format,
            ));
        }
        if self.modern_gpu_target.is_none() {
            let target = self.device.create_texture(&wgpu::TextureDescriptor {
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
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());
            self.modern_gpu_target = Some((target, view));
        }

        self.game_texture.ensure_size(
            &self.device,
            &self.bind_group_layout,
            &self.presentation_buf,
            self.presentation_params.presentation,
            256,
            224,
        );

        let variant = self
            .modern_variant_gpu
            .as_ref()
            .expect("variant renderer built above");
        let (target_texture, target_view) =
            self.modern_gpu_target.as_ref().expect("target built above");
        let stats = variant.render(
            &self.device,
            &self.queue,
            frame,
            bg_cells,
            sprite_cells,
            bg_palette_name,
            sprite_palette_name,
            target_view,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("modern_variant_gpu_copy_to_game_texture"),
            });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: target_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.game_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        self.render()?;
        Ok(stats)
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
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> Result<modern_software::VariantAtlasRenderStats, RenderError> {
        debug_assert_ne!(frame.mode, 7);
        let (mut modern, bg_cells) =
            modern_extract::extract_modern_frame_from_sources(frame, src_table, source_atlas);
        let (sprite_cells, sprites) =
            modern_extract::extract_modern_sprites_from_sources(frame, src_table, source_atlas);
        modern.index_sprites = sprites;
        self.present_modern_variant_gpu(
            &modern,
            &bg_cells,
            &sprite_cells,
            variant_atlas,
            bg_palette_name,
            sprite_palette_name,
        )
    }

    /// Present a Mode-7 frame through the live GPU PPU path, then GPU-copy the
    /// native 256x224 result into the standard presentation texture. This is
    /// used by GPU atlas modes because Mode 7 is not a Mode-1 source-atlas
    /// tilemap, but it still has a real GPU renderer.
    pub fn present_modern_mode7_gpu(&mut self, frame: &GpuFrame<'_>) -> Result<(), RenderError> {
        debug_assert_eq!(frame.mode, 7);
        let format = wgpu::TextureFormat::Rgba8Unorm;
        if self.modern_gpu_target.is_none() {
            let target = self.device.create_texture(&wgpu::TextureDescriptor {
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
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());
            self.modern_gpu_target = Some((target, view));
        }

        self.game_texture.ensure_size(
            &self.device,
            &self.bind_group_layout,
            &self.presentation_buf,
            self.presentation_params.presentation,
            256,
            224,
        );

        let (target_texture, target_view) =
            self.modern_gpu_target.as_ref().expect("target built above");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("modern_gpu_mode7_live"),
            });
        self.gpu_renderer
            .render_frame(&mut encoder, &self.queue, frame, target_view);
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: target_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.game_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        self.render()
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

    pub fn render_gpu_frame(&mut self, frame: &GpuFrame<'_>) -> Result<(), RenderError> {
        self.render_gpu_frame_with_context(frame, PresentationContext::default())
    }

    pub fn render_gpu_frame_with_context(
        &mut self,
        frame: &GpuFrame<'_>,
        context: PresentationContext,
    ) -> Result<(), RenderError> {
        self.maybe_log_viewport();
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
        let lights =
            extract_presentation_lights(frame.oam, &frame.obj, self.presentation_params.lighting);
        let mut occluders = extract_shadow_occluder_words(
            frame.vram,
            &frame.bg[0],
            self.presentation_params.shadows,
        );
        let bg2_occluders = extract_shadow_occluder_words(
            frame.vram,
            &frame.bg[1],
            self.presentation_params.shadows,
        );
        for (dst, src) in occluders.iter_mut().zip(bg2_occluders) {
            *dst |= src;
        }
        let notice = self.presentation_notice;
        let presentation_bytes = build_presentation_uniform_bytes_with_notice(
            self.presentation_params,
            context,
            &lights,
            &occluders,
            notice,
        );
        self.queue
            .write_buffer(&self.gpu_presentation_buf, 0, &presentation_bytes);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gpu_frame_surface"),
            });
        self.gpu_renderer
            .render_frame(&mut encoder, &self.queue, frame, &self.gpu_view);
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gpu_frame_blit"),
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
            pass.set_bind_group(0, &self.gpu_bind_group, &[]);
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

// ── OffscreenRenderer ─────────────────────────────────────────────────────────

/// Headless renderer for pixel readback.
///
/// Renders the same blit pipeline as [`FrameRenderer`] but targets an offscreen
/// `Rgba8Unorm` texture instead of a window surface, then copies pixels back to
/// CPU memory. Used by the binary's `--render-hash-log` and `--dump-frame` paths
/// so those can eventually run through the GPU tile renderer without needing a
/// display.
pub struct OffscreenRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    game_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    _presentation_buf: wgpu::Buffer,
    /// GPU tile renderer — used by [`Self::render_gpu_frame`].
    gpu_renderer: GpuFrameRenderer,
    /// Output target: RENDER_ATTACHMENT | COPY_SRC, exact game resolution.
    render_texture: wgpu::Texture,
    /// Cached view of `render_texture` — shared by both render paths.
    render_view: wgpu::TextureView,
    /// MAP_READ buffer that receives the copy of `render_texture` each frame.
    readback_buf: wgpu::Buffer,
    /// Aligned row pitch (multiple of COPY_BYTES_PER_ROW_ALIGNMENT = 256).
    readback_bytes_per_row: u32,
    upload_buf: Vec<u8>,
    game_width: u32,
    game_height: u32,
}

impl OffscreenRenderer {
    pub async fn new(game_width: u32, game_height: u32) -> Self {
        let instance = create_wgpu_instance();
        // No surface compatibility needed — any adapter works for offscreen rendering.
        let (_adapter, device, queue) = create_device_queue(&instance, None).await;

        let presentation_params =
            PresentationParams::new(PresentationMode::Off, LightingMode::Off, ShadowMode::Off);
        let (game_texture, bind_group_layout, bind_group, presentation_buf) =
            create_game_texture_resources(&device, game_width, game_height, presentation_params);

        // Output format matches game_texture (Rgba8Unorm) — no conversion in the shader.
        let pipeline =
            create_blit_pipeline(&device, &bind_group_layout, wgpu::TextureFormat::Rgba8Unorm);

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen"),
            size: wgpu::Extent3d {
                width: game_width,
                height: game_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let gpu_renderer = GpuFrameRenderer::new(&device, &queue, None);

        // copy_texture_to_buffer requires bytes_per_row to be a multiple of
        // COPY_BYTES_PER_ROW_ALIGNMENT (256). For game_width=256: 256*4=1024, already aligned.
        let readback_bytes_per_row =
            (game_width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

        let readback_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (readback_bytes_per_row * game_height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let upload_buf = vec![0u8; (game_width * game_height * 4) as usize];

        Self {
            device,
            queue,
            pipeline,
            game_texture,
            bind_group,
            _presentation_buf: presentation_buf,
            gpu_renderer,
            render_texture,
            render_view,
            readback_buf,
            readback_bytes_per_row,
            upload_buf,
            game_width,
            game_height,
        }
    }

    /// Upload one frame of pixels. Same format as [`FrameRenderer::upload_frame`].
    pub fn upload_frame(&mut self, pixels: &[u32]) {
        upload_ppu_pixels(
            &self.queue,
            &self.game_texture,
            pixels,
            &mut self.upload_buf,
            self.game_width,
            self.game_height,
        );
    }

    /// Upload one frame from a BGRA byte slice (the native output of `zelda_draw_display_frame`).
    ///
    /// BGRA layout: `[B, G, R, A]` per pixel. Swaps to RGBA for the `Rgba8Unorm` texture.
    /// No allocation — uses the pre-allocated staging buffer.
    pub fn upload_bgra_frame(&mut self, bgra: &[u8]) {
        debug_assert_eq!(
            bgra.len(),
            (self.game_width * self.game_height * 4) as usize,
        );
        for (dst, src) in self
            .upload_buf
            .chunks_exact_mut(4)
            .zip(bgra.chunks_exact(4))
        {
            dst[0] = src[2]; // BGRA[2] = R → RGBA[0]
            dst[1] = src[1]; // G unchanged
            dst[2] = src[0]; // BGRA[0] = B → RGBA[2]
            dst[3] = src[3]; // A unchanged
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.game_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.upload_buf,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.game_width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: self.game_width,
                height: self.game_height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Blit the current CPU PPU frame offscreen and read back RGBA bytes.
    ///
    /// Returns exactly `game_width * game_height * 4` bytes, row-major
    /// top-to-bottom. Blocks until the GPU completes the readback.
    pub fn render_to_rgba(&mut self) -> Vec<u8> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("offscreen"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("offscreen"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_view,
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
            pass.set_bind_group(0, &self.bind_group, &[]);
            // No letterboxing — headless output is exact game resolution.
            pass.set_viewport(
                0.0,
                0.0,
                self.game_width as f32,
                self.game_height as f32,
                0.0,
                1.0,
            );
            pass.draw(0..3, 0..1);
        }

        self.finish_and_readback(encoder)
    }

    /// Render one frame using the GPU tile pipeline and read back RGBA bytes.
    ///
    /// Returns exactly `game_width * game_height * 4` bytes, row-major
    /// top-to-bottom. Blocks until the GPU completes the readback.
    pub fn render_gpu_frame(&mut self, frame: &GpuFrame<'_>) -> Vec<u8> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gpu_frame"),
            });
        self.gpu_renderer
            .render_frame(&mut encoder, &self.queue, frame, &self.render_view);
        self.finish_and_readback(encoder)
    }

    /// Append a texture→buffer copy to `encoder`, submit, and block on readback.
    fn finish_and_readback(&mut self, mut encoder: wgpu::CommandEncoder) -> Vec<u8> {
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback_buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.readback_bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: self.game_width,
                height: self.game_height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit([encoder.finish()]);

        // map_async registers intent; poll(Wait) blocks until the GPU is idle and
        // the callback fires, after which get_mapped_range is valid.
        let slice = self.readback_buf.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during offscreen readback");

        let row_bytes = (self.game_width * 4) as usize;
        let stride = self.readback_bytes_per_row as usize;
        let mapped = slice.get_mapped_range();
        let mut out = Vec::with_capacity(row_bytes * self.game_height as usize);
        for row in 0..self.game_height as usize {
            out.extend_from_slice(&mapped[row * stride..row * stride + row_bytes]);
        }
        drop(mapped);
        self.readback_buf.unmap();
        out
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process};

    fn temp_modern_asset_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("z3rs-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create temp root");
        root
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
        assert!(!resources.gpu_asset_mode());

        fs::remove_dir_all(root).expect("remove temp root");
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
    fn modern_asset_resources_require_source_atlas_for_source_gpu() {
        let root = temp_modern_asset_root("modern-asset-source-missing");

        let err = match ModernAssetFrameResources::load_for_mode(
            EffectiveRendererMode::from_name("assets-anim-gpu"),
            &root,
        ) {
            Ok(_) => panic!("source GPU requires source atlas"),
            Err(err) => err,
        };

        assert!(err.contains("assets-by-source atlas missing"), "{err}");

        fs::remove_dir_all(root).expect("remove temp root");
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
    }

    #[test]
    fn modern_asset_frame_route_keeps_default_paths_on_gpu() {
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, true, true),
            ModernAssetFramePresentRoute::Mode7Gpu
        );
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, true, true),
            ModernAssetFramePresentRoute::SourceVariantGpu
        );
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, false, true),
            ModernAssetFramePresentRoute::SourceGpu
        );
        assert_eq!(
            modern_asset_frame_present_route(1, false, false, false, true),
            ModernAssetFramePresentRoute::VramGpu
        );
    }

    #[test]
    fn modern_asset_frame_route_preserves_explicit_non_gpu_fallbacks() {
        assert_eq!(
            modern_asset_frame_present_route(1, true, true, false, false),
            ModernAssetFramePresentRoute::SourceSoftware
        );
        assert_eq!(
            modern_asset_frame_present_route(7, true, true, false, false),
            ModernAssetFramePresentRoute::Unhandled
        );
        assert_eq!(
            modern_asset_frame_present_route(1, false, false, false, false),
            ModernAssetFramePresentRoute::Unhandled
        );
    }

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.05,
            "expected {actual} to be near {expected}"
        );
    }

    async fn render_test_bg_with_rgba_override() -> Vec<u8> {
        let instance = create_wgpu_instance();
        let (_adapter, device, queue) = create_device_queue(&instance, None).await;
        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test_rgba_override_output"),
            size: wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let readback_bytes_per_row =
            (256u32 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("test_rgba_override_readback"),
            size: (readback_bytes_per_row * 224) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Detail-modulated HD recolor: final = live_cgram * (override / reference).
        // Author-intent: the HD art (`rgba`) is authored against `reference_cgram`;
        // dividing by the reference recovers a palette-agnostic "detail" ratio that is
        // then re-lit by the LIVE CGRAM every frame (keeps day/night, flashes, area
        // swaps). Here: override 0x40 / reference 0x80 = detail 0.5, live 0xf8 → 0x7c.
        let rgba = [0x40, 0x40, 0x40, 0xff];
        let mut reference_cgram = vec![0u8; 1024];
        reference_cgram[4..8].copy_from_slice(&[0x80, 0x80, 0x80, 0xff]); // CGRAM slot 1
        let mut lookup = vec![[0u32; 4]; RGBA_TILE_OVERRIDE_LOOKUP_COUNT];
        lookup[1] = [0, 0, 1, 1];
        let override_data = RgbaTileOverrideData {
            width: 1,
            height: 1,
            rgba: &rgba,
            lookup: &lookup,
            reference_cgram: &reference_cgram,
        };
        let mut renderer = GpuFrameRenderer::new(&device, &queue, Some(override_data));

        let mut vram = vec![0u16; 0x8000];
        vram[0] = 1;
        for row in 0..8 {
            vram[16 + row] = 0x00ff;
        }
        let mut cgram = vec![0u16; 0x100];
        cgram[0] = 0;
        cgram[1] = 0x7fff; // live white; each channel expands 31<<3 = 0xf8
        let oam = vec![0u16; 0x110];
        let mut scanlines = Box::new([ScanlineRegs::default(); 224]);
        for scanline in scanlines.iter_mut() {
            scanline.screen_enabled_main = 1;
        }
        let frame = GpuFrame {
            vram: &vram,
            cgram: &cgram,
            oam: &oam,
            mode: 1,
            bg: [
                BgLayerRegs::default(),
                BgLayerRegs::default(),
                BgLayerRegs::default(),
                BgLayerRegs::default(),
            ],
            obj: ObjRegs::default(),
            mosaic_enabled: 0,
            mosaic_size: 1,
            extra_left_right: 0,
            mode7: Mode7Regs::default(),
            screen_enabled: [1, 0],
            screen_windowed: [0, 0],
            brightness: 15,
            forced_blank: false,
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
            scanlines,
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("test_rgba_override_encoder"),
        });
        renderer.render_frame(&mut encoder, &queue, &frame, &render_view);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(readback_bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);

        let slice = readback_buf.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during rgba override test readback");
        let mapped = slice.get_mapped_range();
        let mut out = vec![0u8; 256 * 224 * 4];
        for row in 0..224usize {
            let src = row * readback_bytes_per_row as usize;
            let dst = row * 256 * 4;
            out[dst..dst + 256 * 4].copy_from_slice(&mapped[src..src + 256 * 4]);
        }
        drop(mapped);
        readback_buf.unmap();
        out
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
    fn high_priority_bg_tile_sets_coarse_shadow_occluder() {
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x2001;
        let bg = BgLayerRegs::default();

        let words = extract_shadow_occluder_words(&vram, &bg, ShadowMode::Raycast);

        assert_eq!(words[0] & 1, 1);
    }

    #[test]
    fn shadow_occluders_are_empty_when_shadows_are_off() {
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x2001;
        let bg = BgLayerRegs::default();

        let words = extract_shadow_occluder_words(&vram, &bg, ShadowMode::Off);

        assert_eq!(words, [0; PRESENTATION_OCCLUDER_WORDS]);
    }

    #[test]
    fn art_sidecars_are_disabled_without_manifest_path() {
        let config = ArtSidecarConfig::from_value(None);

        assert!(!config.enabled());
    }

    #[test]
    fn art_sidecar_manifest_parses_tile_maps_and_overrides() {
        let manifest = ArtSidecarManifest::from_json(
            r#"{
                "tiles": [
                    {
                        "tile": 42,
                        "normal": "normals/002a.png",
                        "depth": "depth/002a.png",
                        "rgba": "rgba/002a.png"
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(manifest.tiles.len(), 1);
        assert_eq!(manifest.tiles[0].tile, 42);
        assert_eq!(
            manifest.tiles[0].normal.as_deref(),
            Some("normals/002a.png")
        );
        assert_eq!(manifest.tiles[0].depth.as_deref(), Some("depth/002a.png"));
        assert_eq!(manifest.tiles[0].rgba.as_deref(), Some("rgba/002a.png"));
    }

    #[test]
    fn art_sidecar_assets_load_manifest_from_config_path() {
        let path =
            std::env::temp_dir().join(format!("zelda3-sidecar-test-{}.json", std::process::id()));
        std::fs::write(&path, r#"{ "tiles": [{ "tile": 7 }] }"#).unwrap();
        let config = ArtSidecarConfig {
            manifest_path: Some(path.clone()),
        };

        let assets = ArtSidecarAssets::load(&config);

        std::fs::remove_file(path).ok();
        assert!(assets.enabled());
        assert_eq!(assets.tiles[0].tile, 7);
    }

    fn write_test_png(path: &std::path::Path, width: u32, height: u32, rgba: &[u8]) {
        let file = std::fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(rgba).unwrap();
    }

    #[test]
    fn art_sidecar_assets_decode_relative_rgba_pngs() {
        let root =
            std::env::temp_dir().join(format!("zelda3-sidecar-image-test-{}", std::process::id()));
        let rgba_dir = root.join("rgba");
        std::fs::create_dir_all(&rgba_dir).unwrap();
        let png_path = rgba_dir.join("0007.png");
        let pixels = [
            0x10, 0x20, 0x30, 0xff, 0x40, 0x50, 0x60, 0xff, 0x70, 0x80, 0x90, 0xff, 0xa0, 0xb0,
            0xc0, 0xff,
        ];
        write_test_png(&png_path, 2, 2, &pixels);

        let manifest_path = root.join("manifest.json");
        std::fs::write(
            &manifest_path,
            r#"{ "tiles": [{ "tile": 7, "rgba": "rgba/0007.png" }] }"#,
        )
        .unwrap();
        let config = ArtSidecarConfig {
            manifest_path: Some(manifest_path),
        };

        let assets = ArtSidecarAssets::load(&config);

        std::fs::remove_dir_all(root).ok();
        assert_eq!(assets.tiles.len(), 1);
        assert_eq!(assets.tiles[0].tile, 7);
        let rgba = assets.tiles[0].rgba.as_ref().unwrap();
        assert_eq!(rgba.width, 2);
        assert_eq!(rgba.height, 2);
        assert_eq!(rgba.rgba, pixels);
    }

    #[test]
    fn art_sidecar_assets_pack_rgba_overrides_into_atlas() {
        let first = ArtSidecarImage {
            width: 2,
            height: 1,
            rgba: vec![1, 2, 3, 4, 5, 6, 7, 8],
        };
        let second = ArtSidecarImage {
            width: 1,
            height: 2,
            rgba: vec![9, 10, 11, 12, 13, 14, 15, 16],
        };
        let assets = ArtSidecarAssets {
            _manifest: None,
            tiles: vec![
                ArtSidecarTileAssets {
                    tile: 0x20,
                    normal: None,
                    depth: None,
                    rgba: Some(first),
                },
                ArtSidecarTileAssets {
                    tile: 0x21,
                    normal: None,
                    depth: None,
                    rgba: Some(second),
                },
            ],
            reference_cgram: Vec::new(),
        };

        let atlas = assets.build_rgba_override_atlas().unwrap();

        assert_eq!(atlas.width, 3);
        assert_eq!(atlas.height, 2);
        assert_eq!(atlas.lookup.len(), 2);
        assert_eq!(
            atlas.lookup_for_tile(0x20).unwrap(),
            ArtSidecarAtlasEntry {
                tile: 0x20,
                x: 0,
                y: 0,
                width: 2,
                height: 1,
            }
        );
        assert_eq!(
            atlas.lookup_for_tile(0x21).unwrap(),
            ArtSidecarAtlasEntry {
                tile: 0x21,
                x: 2,
                y: 0,
                width: 1,
                height: 2,
            }
        );
        assert_eq!(&atlas.rgba[0..8], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(&atlas.rgba[8..12], &[9, 10, 11, 12]);
        let second_row_offset = (atlas.width * 4) as usize;
        assert_eq!(
            &atlas.rgba[second_row_offset + 8..second_row_offset + 12],
            &[13, 14, 15, 16]
        );
    }

    #[test]
    fn art_sidecar_reference_cgram_flows_through_to_override_data() {
        let mut reference_cgram = vec![0u8; 1024];
        reference_cgram[4..8].copy_from_slice(&[0x80, 0x80, 0x80, 0xff]); // slot 1
        let assets = ArtSidecarAssets {
            _manifest: None,
            tiles: vec![ArtSidecarTileAssets {
                tile: 0x20,
                normal: None,
                depth: None,
                rgba: Some(ArtSidecarImage {
                    width: 1,
                    height: 1,
                    rgba: vec![0x40, 0x40, 0x40, 0xff],
                }),
            }],
            reference_cgram: reference_cgram.clone(),
        };

        let atlas = assets.build_rgba_override_atlas().unwrap();
        // The reference palette is carried onto the atlas...
        assert_eq!(atlas.reference_cgram, reference_cgram);
        // ...and exposed to the GPU override data for detail-modulated recolor.
        let lookup = atlas.lookup_texture_pixels();
        let data = atlas.as_tile_override_data(&lookup);
        assert_eq!(data.reference_cgram, reference_cgram.as_slice());
    }

    #[test]
    fn art_sidecar_rgba_atlas_reports_gpu_upload_layout() {
        let atlas = ArtSidecarRgbaAtlas {
            width: 3,
            height: 2,
            rgba: vec![0; 3 * 2 * 4],
            lookup: Vec::new(),
            reference_cgram: Vec::new(),
        };

        assert_eq!(atlas.texture_extent().width, 3);
        assert_eq!(atlas.texture_extent().height, 2);
        assert_eq!(atlas.bytes_per_row(), 12);
        assert_eq!(atlas.upload_byte_len(), 24);
    }

    #[test]
    fn art_sidecar_rgba_atlas_builds_tile_lookup_table_for_shader() {
        let atlas = ArtSidecarRgbaAtlas {
            width: 3,
            height: 2,
            rgba: vec![0; 3 * 2 * 4],
            lookup: vec![
                ArtSidecarAtlasEntry {
                    tile: 0x20,
                    x: 0,
                    y: 0,
                    width: 2,
                    height: 1,
                },
                ArtSidecarAtlasEntry {
                    tile: 0x21,
                    x: 2,
                    y: 0,
                    width: 1,
                    height: 2,
                },
            ],
            reference_cgram: Vec::new(),
        };

        let lookup = atlas.lookup_texture_pixels();

        assert_eq!(lookup.len(), 1024);
        assert_eq!(lookup[0x1f], [0, 0, 0, 0]);
        assert_eq!(lookup[0x20], [0, 0, 2, 1]);
        assert_eq!(lookup[0x21], [2, 0, 1, 2]);
    }

    #[test]
    fn bg_shader_samples_rgba_sidecar_overrides_before_cgram_color() {
        let shader = include_str!("bg_layer.wgsl");

        assert!(shader.contains("@binding(3) var tile_override_atlas: texture_2d<f32>"));
        assert!(shader.contains("@binding(4) var tile_override_lookup: texture_2d<u32>"));
        assert!(shader.contains("fn sample_tile_override("));
        assert!(shader.contains("let override_color = sample_tile_override(tile_num, px, py);"));
        assert!(shader.contains("if override_color.a > 0.0"));
        // Detail-modulated HD recolor: the override is divided by the reference CGRAM
        // and re-lit by the live CGRAM, not returned verbatim.
        assert!(shader.contains("@binding(5) var reference_cgram: texture_2d<f32>"));
        assert!(shader.contains("let detail = override_color.rgb / max(reference.rgb"));
    }

    #[test]
    fn rgba_sidecar_override_changes_bg_tile_output_color() {
        let pixels = pollster::block_on(render_test_bg_with_rgba_override());

        // Detail-modulated: live 0xf8 * (override 0x40 / reference 0x80) = 0xf8 * 0.5 ≈ 0x7c.
        // Allow ±1 for GPU float-division rounding (backend-dependent last-bit).
        for &c in &pixels[0..3] {
            assert!(
                (c as i32 - 0x7c).abs() <= 1,
                "expected ~0x7c (detail-modulated), got {c:#x}"
            );
        }
        assert_eq!(pixels[3], 0xff);
        // Not the raw live color (override applied) and not raw override passthrough
        // (it was re-lit through the live palette, not copied verbatim).
        assert_ne!(&pixels[0..4], &[0xf8, 0xf8, 0xf8, 0xff]);
        assert_ne!(&pixels[0..4], &[0x40, 0x40, 0x40, 0xff]);
    }

    #[test]
    fn art_sidecar_rgba_override_atlas_is_absent_without_rgba_images() {
        let assets = ArtSidecarAssets {
            _manifest: None,
            tiles: vec![ArtSidecarTileAssets {
                tile: 0x20,
                normal: Some(ArtSidecarImage {
                    width: 1,
                    height: 1,
                    rgba: vec![1, 2, 3, 4],
                }),
                depth: None,
                rgba: None,
            }],
            reference_cgram: Vec::new(),
        };

        assert!(assets.build_rgba_override_atlas().is_none());
    }

    #[test]
    fn dynamic_lighting_extracts_visible_sprite_light() {
        let mut oam = vec![0u16; 0x110];
        oam[0] = (48u16 << 8) | 32u16;
        oam[1] = 4u16 << 9;
        oam[0x100] = 0b10;

        let lights = extract_presentation_lights(&oam, &ObjRegs::default(), LightingMode::Dynamic);

        assert_eq!(lights.len(), 1);
        assert_near(lights[0].x, 40.0 / 256.0);
        assert_near(lights[0].y, 56.0 / 224.0);
        assert_near(lights[0].radius, 0.18);
    }

    #[test]
    fn dynamic_lighting_uses_sprite_tile_light_profiles() {
        let mut oam = vec![0u16; 0x110];
        oam[0] = (48u16 << 8) | 32u16;
        oam[1] = (4u16 << 9) | 0x5c;
        oam[2] = (80u16 << 8) | 64u16;
        oam[3] = (4u16 << 9) | 0x7c;
        oam[0x100] = 0b1000;

        let lights = extract_presentation_lights(&oam, &ObjRegs::default(), LightingMode::Dynamic);

        assert_eq!(lights.len(), 2);
        assert_near(lights[0].radius, 0.12);
        assert_near(lights[0].intensity, 0.30);
        assert_near(lights[1].radius, 0.24);
        assert_near(lights[1].intensity, 0.55);
    }

    #[test]
    fn dynamic_lighting_keeps_bright_palette_fallback_for_unclassified_sprites() {
        let mut oam = vec![0u16; 0x110];
        oam[0] = (48u16 << 8) | 32u16;
        oam[1] = (4u16 << 9) | 0x11;
        oam[0x100] = 0b10;

        let lights = extract_presentation_lights(&oam, &ObjRegs::default(), LightingMode::Dynamic);

        assert_eq!(lights.len(), 1);
        assert_near(lights[0].radius, 0.18);
        assert_near(lights[0].intensity, 0.38);
    }

    #[test]
    fn dynamic_lighting_ignores_unclassified_dim_palette_sprites() {
        let mut oam = vec![0u16; 0x110];
        oam[0] = (48u16 << 8) | 32u16;
        oam[1] = (2u16 << 9) | 0x11;

        let lights = extract_presentation_lights(&oam, &ObjRegs::default(), LightingMode::Dynamic);

        assert!(lights.is_empty());
    }

    #[test]
    fn presentation_lights_are_disabled_unless_dynamic() {
        let mut oam = vec![0u16; 0x110];
        oam[0] = (48u16 << 8) | 32u16;
        oam[1] = 4u16 << 9;
        oam[0x100] = 0b10;

        assert!(
            extract_presentation_lights(&oam, &ObjRegs::default(), LightingMode::Off).is_empty()
        );
        assert!(
            extract_presentation_lights(&oam, &ObjRegs::default(), LightingMode::Ambient)
                .is_empty()
        );
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
