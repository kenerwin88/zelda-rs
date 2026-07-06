use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;
use std::time::Instant;

use crate::gpu_readback::GpuRgbaReadbackFrame;
use platform::NativeFrontend;
use renderer::{GpuFrame, RawScanlineFrame};
use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

const PLAYER_IS_INDOORS: usize = 0x001b;
const MAIN_MODULE_INDEX: usize = 0x10;

pub struct LiveGpuFrameCapture {
    ppu: snes::ppu::PpuState,
    cgram: Vec<u16>,
    raw_scanlines: Box<RawScanlineFrame>,
    source_entries: Vec<zelda3::LogicalChrSrc>,
    mode7_source_chars: Option<Vec<u8>>,
    main_module: u8,
    player_indoors: u8,
}

struct ModernAssetGpuReadbackFrame {
    frame: GpuRgbaReadbackFrame,
    #[cfg(test)]
    via: &'static str,
}

pub(crate) struct ModernAssetGpuReadbackRenderer {
    resources: renderer::ModernIndexCompareResources,
    validation_cache: HashMap<u64, Result<(), String>>,
    validation_cache_hits: u64,
    validation_cache_misses: u64,
    validation_key_time: Duration,
    validation_miss_time: Duration,
    validation_bg_extract_nanos: u128,
    validation_sprite_extract_nanos: u128,
    validation_stats_nanos: u128,
}

impl LiveGpuFrameCapture {
    pub fn from_game(game: &mut ZeldaState) -> Self {
        let cgram = game.cgram_after_first_hdma_line();
        let raw_scanlines = game.ppu_scanline_windows();
        let ppu = game.ppu.clone();
        let source_entries = game.vram_chr_source().as_slice().to_vec();
        let mode7_source_chars = game.mode7_character_source().map(<[u8]>::to_vec);
        let main_module = game.ram[MAIN_MODULE_INDEX];
        let player_indoors = game.ram[PLAYER_IS_INDOORS];
        Self {
            ppu,
            cgram,
            raw_scanlines,
            source_entries,
            mode7_source_chars,
            main_module,
            player_indoors,
        }
    }

    pub fn capture_input(&self) -> renderer::GpuFrameCaptureInput<'_> {
        gpu_frame_capture_from_ppu(&self.ppu, &self.cgram, self.raw_scanlines.as_ref())
    }

    pub fn gpu_frame(&self) -> GpuFrame<'_> {
        GpuFrame::from_capture_input(self.capture_input())
    }

    pub fn ppu(&self) -> &snes::ppu::PpuState {
        &self.ppu
    }

    pub fn cgram(&self) -> &[u16] {
        &self.cgram
    }

    pub fn raw_scanlines(&self) -> &RawScanlineFrame {
        self.raw_scanlines.as_ref()
    }

    pub fn source_entries(&self) -> &[zelda3::LogicalChrSrc] {
        &self.source_entries
    }

    pub fn mode7_source_chars(&self) -> Option<&[u8]> {
        self.mode7_source_chars.as_deref()
    }

    pub fn main_module(&self) -> u8 {
        self.main_module
    }

    pub fn player_indoors(&self) -> u8 {
        self.player_indoors
    }

    pub fn modern_asset_present_input<'a>(
        &'a self,
        resources: &'a renderer::ModernAssetFrameResources,
        stats: &'a mut renderer::ModernAssetLiveStats,
    ) -> renderer::ModernAssetFrameLivePresentInput<'a, 'a, zelda3::LogicalChrSrc> {
        renderer::ModernAssetFrameLivePresentInput {
            frame: self.capture_input(),
            source_entries: &self.source_entries,
            resources,
            stats,
            player_indoors: self.player_indoors,
        }
    }
}

/// Modern asset resources + HD override store for the live present path, loaded
/// once. The renderer crate owns which atlases each renderer mode requires and
/// how they route through GPU/software presentation.
struct GpuPlayRenderer {
    live_mode: renderer::EffectiveRendererMode<'static>,
    modern_assets: renderer::ModernAssetFrameResources,
    variant_live_stats: renderer::ModernAssetLiveStats,
}

impl GpuPlayRenderer {
    fn new() -> Self {
        let repo_root = repo_root();
        let (modern_assets, live_mode) =
            renderer::ModernAssetFrameResources::load_live_gpu_from_env(&repo_root).unwrap_or_else(
                |e| {
                    eprintln!("modern asset load failed: {e}");
                    process::exit(2);
                },
            );
        Self {
            live_mode,
            modern_assets,
            variant_live_stats: renderer::ModernAssetLiveStats::from_env(),
        }
    }
}

impl ModernAssetGpuReadbackRenderer {
    pub(crate) fn load_from_env() -> Result<Self, String> {
        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)?;
        Ok(Self {
            resources,
            validation_cache: HashMap::new(),
            validation_cache_hits: 0,
            validation_cache_misses: 0,
            validation_key_time: Duration::ZERO,
            validation_miss_time: Duration::ZERO,
            validation_bg_extract_nanos: 0,
            validation_sprite_extract_nanos: 0,
            validation_stats_nanos: 0,
        })
    }

    pub(crate) fn render_game_rgba(
        &self,
        game: &mut ZeldaState,
    ) -> Result<GpuRgbaReadbackFrame, String> {
        let capture = capture_gpu_frame_from_game(game);
        render_modern_asset_capture_rgba(&capture, &self.resources).map(|render| render.frame)
    }

    pub(crate) fn validate_game_full_gpu_path(
        &mut self,
        game: &mut ZeldaState,
    ) -> Result<(), String> {
        let capture = capture_gpu_frame_from_game(game);
        let key_start = Instant::now();
        let key = validation_cache_key(&capture);
        self.validation_key_time += key_start.elapsed();
        if let Some(result) = self.validation_cache.get(&key) {
            self.validation_cache_hits += 1;
            return result.clone();
        }
        self.validation_cache_misses += 1;
        let miss_start = Instant::now();
        let result = match validate_modern_asset_capture(&capture, &self.resources) {
            Ok(validation) => {
                self.validation_bg_extract_nanos += validation.timings.bg_extract_nanos;
                self.validation_sprite_extract_nanos += validation.timings.sprite_extract_nanos;
                self.validation_stats_nanos += validation.timings.stats_nanos;
                Ok(())
            }
            Err(e) => Err(e),
        };
        self.validation_miss_time += miss_start.elapsed();
        self.validation_cache.insert(key, result.clone());
        result
    }

    pub(crate) fn validation_cache_stats(&self) -> (u64, u64, usize, u128, u128, u128, u128, u128) {
        (
            self.validation_cache_hits,
            self.validation_cache_misses,
            self.validation_cache.len(),
            self.validation_key_time.as_millis(),
            self.validation_miss_time.as_millis(),
            self.validation_bg_extract_nanos / 1_000_000,
            self.validation_sprite_extract_nanos / 1_000_000,
            self.validation_stats_nanos / 1_000_000,
        )
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zelda3-bin lives under the workspace root")
        .to_path_buf()
}

impl crate::play_renderer::PlayRendererBackend for GpuPlayRenderer {
    fn name(&self) -> &'static str {
        "gpu_render"
    }

    fn configure_frontend(&self, frontend: &mut NativeFrontend) {
        // Live play is always PNG-backed GPU rendering. `classic`, `modern`,
        // `modern-compare`, and CPU atlas modes remain available only through
        // explicit diagnostic commands, not the default playable frontend.
        frontend.set_renderer_mode(renderer::RendererMode::from_effective_mode(self.live_mode));
    }

    fn present_frame(
        &mut self,
        game: &mut ZeldaState,
        frontend: &mut NativeFrontend,
        _frame: &mut [u8],
        _render_flags: PpuRenderFlags,
    ) {
        let capture = LiveGpuFrameCapture::from_game(game);
        let report = frontend.present_modern_asset_live_frame_from_entries(
            capture.modern_asset_present_input(&self.modern_assets, &mut self.variant_live_stats),
        );
        if let Some(line) = report.failure_line() {
            eprintln!("{line}");
            process::exit(2);
        }
    }
}

pub(crate) fn new_gpu_play_renderer() -> Box<dyn crate::play_renderer::PlayRendererBackend> {
    Box::new(GpuPlayRenderer::new())
}

pub(crate) fn capture_gpu_frame_from_game(game: &mut ZeldaState) -> LiveGpuFrameCapture {
    LiveGpuFrameCapture::from_game(game)
}

pub(crate) fn render_live_game_gpu_frame_rgba(
    game: &mut ZeldaState,
    width: u32,
    height: u32,
) -> Result<GpuRgbaReadbackFrame, String> {
    if (width, height) != (256, 224) {
        return Err(format!(
            "modern asset GPU readback is fixed at 256x224, got {width}x{height}"
        ));
    }
    let renderer = ModernAssetGpuReadbackRenderer::load_from_env()?;
    renderer.render_game_rgba(game)
}

#[cfg(test)]
fn render_live_game_modern_asset_frame_rgba(
    game: &mut ZeldaState,
) -> Result<ModernAssetGpuReadbackFrame, String> {
    let capture = capture_gpu_frame_from_game(game);
    let repo_root = repo_root();
    let resources =
        renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)?;
    render_modern_asset_capture_rgba(&capture, &resources)
}

fn render_modern_asset_capture_rgba(
    capture: &LiveGpuFrameCapture,
    resources: &renderer::ModernIndexCompareResources,
) -> Result<ModernAssetGpuReadbackFrame, String> {
    let gpu_frame = capture.gpu_frame();
    let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(capture.player_indoors());
    let render = resources.render_full_gpu_asset_rgba_from_entries(
        &gpu_frame,
        capture.source_entries(),
        capture.mode7_source_chars(),
        scene,
    )?;
    Ok(ModernAssetGpuReadbackFrame {
        frame: GpuRgbaReadbackFrame::from_rgba(render.rgba),
        #[cfg(test)]
        via: render.via,
    })
}

fn validate_modern_asset_capture(
    capture: &LiveGpuFrameCapture,
    resources: &renderer::ModernIndexCompareResources,
) -> Result<renderer::ModernAssetValidationFrame, String> {
    let gpu_frame = capture.gpu_frame();
    let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(capture.player_indoors());
    resources.validate_full_gpu_asset_from_entries(
        &gpu_frame,
        capture.source_entries(),
        capture.mode7_source_chars(),
        scene,
    )
}

fn validation_cache_key(capture: &LiveGpuFrameCapture) -> u64 {
    let mut hasher = DefaultHasher::new();
    let input = capture.capture_input();
    let registers = input.registers;

    registers.vram.hash(&mut hasher);
    input.cgram.hash(&mut hasher);
    registers.oam.hash(&mut hasher);
    registers.mode.hash(&mut hasher);
    for bg in registers.bg {
        bg.h_scroll.hash(&mut hasher);
        bg.v_scroll.hash(&mut hasher);
        bg.tilemap_wider.hash(&mut hasher);
        bg.tilemap_higher.hash(&mut hasher);
        bg.tilemap_adr.hash(&mut hasher);
        bg.tile_adr.hash(&mut hasher);
    }
    registers.obj.tile_adr1.hash(&mut hasher);
    registers.obj.tile_adr2.hash(&mut hasher);
    registers.obj.obj_size.hash(&mut hasher);
    registers.mosaic_enabled.hash(&mut hasher);
    registers.mosaic_size.hash(&mut hasher);
    registers.extra_left_right.hash(&mut hasher);
    registers.mode7.matrix.hash(&mut hasher);
    registers.mode7.large_field.hash(&mut hasher);
    registers.mode7.char_fill.hash(&mut hasher);
    registers.mode7.x_flip.hash(&mut hasher);
    registers.mode7.y_flip.hash(&mut hasher);
    registers.mode7.ext_bg_always_zero.hash(&mut hasher);
    registers.screen_enabled.hash(&mut hasher);
    registers.screen_windowed.hash(&mut hasher);
    registers.brightness.hash(&mut hasher);
    registers.forced_blank.hash(&mut hasher);
    registers.math_enabled.hash(&mut hasher);
    registers.subtract_color.hash(&mut hasher);
    registers.half_color.hash(&mut hasher);
    registers.fixed_color_r.hash(&mut hasher);
    registers.fixed_color_g.hash(&mut hasher);
    registers.fixed_color_b.hash(&mut hasher);
    registers.add_subscreen.hash(&mut hasher);
    registers.clip_mode.hash(&mut hasher);
    registers.prevent_math_mode.hash(&mut hasher);
    registers.windowsel.hash(&mut hasher);
    input.raw_scanlines.hash(&mut hasher);
    for src in capture.source_entries() {
        src.kind.hash(&mut hasher);
        src.pack.hash(&mut hasher);
        src.tile_off.hash(&mut hasher);
    }
    capture.mode7_source_chars().hash(&mut hasher);
    capture.player_indoors().hash(&mut hasher);

    hasher.finish()
}

pub(crate) fn render_hd_capture_from_game(
    game: &mut ZeldaState,
    atlas: &renderer::modern_source_atlas::ModernSourceAtlas,
) -> Result<renderer::hd_authoring::HdCaptureFrame, String> {
    let capture = capture_gpu_frame_from_game(game);
    let repo_root = repo_root();
    let resources =
        renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)?;
    let gpu_render = render_modern_asset_capture_rgba(&capture, &resources)?;
    Ok(render_hd_capture_from_gpu_readback(
        &capture, atlas, gpu_render,
    ))
}

fn render_hd_capture_from_gpu_readback(
    capture: &LiveGpuFrameCapture,
    atlas: &renderer::modern_source_atlas::ModernSourceAtlas,
    gpu_render: ModernAssetGpuReadbackFrame,
) -> renderer::hd_authoring::HdCaptureFrame {
    if capture.gpu_frame().mode == 7 {
        return renderer::hd_authoring::HdCaptureFrame {
            rgba: gpu_render.frame.as_slice().to_vec(),
            placements: Vec::new(),
            cgram_rgba: cgram_rgba_from_capture(capture),
        };
    }
    let mut hd_capture = render_hd_capture_from_gpu_capture(capture, atlas);
    hd_capture.rgba = gpu_render.frame.as_slice().to_vec();
    hd_capture
}

fn cgram_rgba_from_capture(capture: &LiveGpuFrameCapture) -> [[u8; 4]; 256] {
    std::array::from_fn(|i| {
        renderer::modern_palette::snes_cgram_to_rgba(capture.cgram().get(i).copied().unwrap_or(0))
    })
}

fn render_hd_capture_from_gpu_capture(
    capture: &LiveGpuFrameCapture,
    atlas: &renderer::modern_source_atlas::ModernSourceAtlas,
) -> renderer::hd_authoring::HdCaptureFrame {
    let gpu_frame = capture.gpu_frame();
    let source_table = renderer::source_table_from_entries(capture.source_entries());
    renderer::hd_authoring::render_hd_capture_from_sources(&gpu_frame, &source_table, atlas)
}

fn gpu_frame_capture_from_ppu<'a>(
    ppu: &'a snes::ppu::PpuState,
    cgram: &'a [u16],
    raw_scanlines: &'a RawScanlineFrame,
) -> renderer::GpuFrameCaptureInput<'a> {
    renderer::GpuFrameCaptureInput {
        registers: gpu_frame_register_snapshot_from_ppu(ppu),
        cgram,
        raw_scanlines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_game_gpu_frame_readback_uses_variant_asset_route() {
        let (mut game, _) =
            crate::developer_room_commands::load_developer_destination("preset-dev-sandbox")
                .expect("developer sandbox should load");
        game.zelda_run_frame(0);

        let render = render_live_game_modern_asset_frame_rgba(&mut game)
            .expect("GPU readback should render");

        assert_eq!(render.via, "variant-gpu");
        assert_eq!(render.frame.as_slice().len(), 256 * 224 * 4);
    }

    #[test]
    fn hd_capture_visible_frame_uses_asset_gpu_readback() {
        let repo_root = repo_root();
        let atlas =
            renderer::modern_source_atlas::load_modern_source_atlas(&repo_root).expect("atlas");
        let (mut game, _) =
            crate::developer_room_commands::load_developer_destination("preset-dev-sandbox")
                .expect("developer sandbox should load");
        game.zelda_run_frame(0);

        let hd_capture =
            render_hd_capture_from_game(&mut game, &atlas).expect("HD capture should render");
        let gpu_render = render_live_game_modern_asset_frame_rgba(&mut game)
            .expect("GPU readback should render");

        assert_eq!(gpu_render.via, "variant-gpu");
        assert_eq!(hd_capture.rgba, gpu_render.frame.as_slice());
        assert!(!hd_capture.placements.is_empty());
    }

    #[test]
    fn hd_capture_mode7_keeps_gpu_rgba_with_empty_affine_placement_map() {
        let mut ppu = snes::ppu::PpuState::default();
        ppu.mode = 7;
        let mut cgram = vec![0u16; 256];
        cgram[1] = 0x7fff;
        let capture = LiveGpuFrameCapture {
            ppu,
            cgram,
            raw_scanlines: Box::new([(0, 0, 0, 0, 0, [0; 4], [0; 4], [0; 8]); 224]),
            source_entries: Vec::new(),
            mode7_source_chars: None,
            main_module: 0,
            player_indoors: 0,
        };
        let gpu_render = ModernAssetGpuReadbackFrame {
            frame: GpuRgbaReadbackFrame::from_rgba(vec![0x7f; 256 * 224 * 4]),
            #[cfg(test)]
            via: "mode7-source-gpu",
        };
        let repo_root = repo_root();
        let atlas =
            renderer::modern_source_atlas::load_modern_source_atlas(&repo_root).expect("atlas");

        let hd_capture = render_hd_capture_from_gpu_readback(&capture, &atlas, gpu_render);

        assert_eq!(hd_capture.rgba, vec![0x7f; 256 * 224 * 4]);
        assert!(hd_capture.placements.is_empty());
        assert_eq!(
            hd_capture.cgram_rgba[1],
            renderer::modern_palette::snes_cgram_to_rgba(0x7fff)
        );
    }
}

fn gpu_frame_register_snapshot_from_ppu<'a>(
    ppu: &'a snes::ppu::PpuState,
) -> renderer::GpuFrameRegisterSnapshot<'a> {
    renderer::GpuFrameRegisterSnapshot {
        vram: &ppu.vram,
        oam: &ppu.oam,
        mode: ppu.mode,
        bg: std::array::from_fn(|layer| renderer::BgLayerRegs {
            h_scroll: ppu.bg_layer[layer].h_scroll,
            v_scroll: ppu.bg_layer[layer].v_scroll,
            tilemap_wider: ppu.bg_layer[layer].tilemap_wider,
            tilemap_higher: ppu.bg_layer[layer].tilemap_higher,
            tilemap_adr: ppu.bg_layer[layer].tilemap_adr,
            tile_adr: ppu.bg_layer[layer].tile_adr,
        }),
        obj: renderer::ObjRegs {
            tile_adr1: ppu.obj_tile_adr1,
            tile_adr2: ppu.obj_tile_adr2,
            obj_size: ppu.obj_size,
        },
        mosaic_enabled: ppu.mosaic_enabled,
        mosaic_size: ppu.mosaic_size,
        extra_left_right: ppu.extra_left_right,
        mode7: renderer::Mode7Regs {
            matrix: ppu.m7_matrix,
            large_field: ppu.m7_large_field,
            char_fill: ppu.m7_char_fill,
            x_flip: ppu.m7_x_flip,
            y_flip: ppu.m7_y_flip,
            ext_bg_always_zero: ppu.m7_ext_bg_always_zero,
        },
        screen_enabled: ppu.screen_enabled,
        screen_windowed: ppu.screen_windowed,
        brightness: ppu.brightness,
        forced_blank: ppu.forced_blank,
        math_enabled: ppu.math_enabled,
        subtract_color: ppu.subtract_color,
        half_color: ppu.half_color,
        fixed_color_r: ppu.fixed_color_r,
        fixed_color_g: ppu.fixed_color_g,
        fixed_color_b: ppu.fixed_color_b,
        add_subscreen: ppu.add_subscreen,
        clip_mode: ppu.clip_mode,
        prevent_math_mode: ppu.prevent_math_mode,
        windowsel: ppu.windowsel,
    }
}
