use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use crate::image_output::decode_rgba_png;
use platform::{NativeFrontend, NativeFrontendOptions};
use renderer::{GpuFrame, RawScanlineFrame};
use serde::Deserialize;
use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

/// RGBA readback container for modern asset GPU captures.
pub(crate) struct GpuRgbaReadbackFrame {
    rgba: Vec<u8>,
}

impl GpuRgbaReadbackFrame {
    pub(crate) fn from_rgba(rgba: Vec<u8>) -> Self {
        Self { rgba }
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.rgba
    }
}

impl std::ops::Deref for GpuRgbaReadbackFrame {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.rgba
    }
}

const PLAYER_IS_INDOORS: usize = 0x001b;
const MAIN_MODULE_INDEX: usize = 0x10;

pub struct LiveGpuFrameCapture {
    hardware_startup_transient: Option<renderer::gpu_frame::HardwareStartupTransient>,
    ppu: snes::ppu::PpuState,
    cgram: Vec<u16>,
    raw_scanlines: Box<RawScanlineFrame>,
    source_entries: Vec<zelda3::LogicalChrSrc>,
    bg3_source_tiles: Vec<renderer::GpuBg3SourceTile>,
    bg3_vwf_glyph_runs: Vec<renderer::GpuBg3VwfGlyphRun>,
    dialogue_message_id: Option<u16>,
    source_dialogue_ir: Vec<zelda3::dialogue_ir::DialogueIrOp>,
    dialogue_ir: Vec<zelda3::dialogue_ir::DialogueIrOp>,
    dialogue_layout: Vec<zelda3::dialogue_ir::DialogueGlyphPlacement>,
    dialogue_layout_origin_tile_number: Option<u16>,
    #[allow(dead_code)]
    mode7_source_chars: Option<Vec<u8>>,
    #[allow(dead_code)]
    main_module: u8,
    player_indoors: u8,
    cgram_provenance: zelda3_palette::CgramProvenanceSnapshot,
}

struct ModernAssetGpuReadbackFrame {
    frame: GpuRgbaReadbackFrame,
    #[cfg(test)]
    variant_stats: Option<renderer::modern_software::VariantAtlasRenderStats>,
    #[cfg(test)]
    via: &'static str,
}

pub(crate) struct ModernAssetGpuReadbackRenderer {
    resources: renderer::ModernIndexCompareResources,
    /// `ZELDA3_COMPARE_VIDEO_CPU=1`: render comparison video with the software
    /// source compositor (no GPU device at all). GPU-free comparisons are
    /// deterministic under parallelism; the GPU render remains the default and
    /// the final single-process gate.
    cpu_video: bool,
    /// Memoized (input fingerprint -> rendered RGBA) for the previous frame.
    /// The production render is a pure function of the capture, so identical
    /// inputs (static menus, text pauses) reuse the exact previous pixels.
    render_cache: std::sync::Mutex<Option<(u64, Vec<u8>)>>,
    validation_cache: HashMap<u64, Result<(), String>>,
    validation_cache_hits: u64,
    validation_cache_misses: u64,
    validation_key_time: Duration,
    validation_miss_time: Duration,
    validation_bg_extract_nanos: u128,
    validation_sprite_extract_nanos: u128,
    validation_stats_nanos: u128,
}

/// Snes9x oracle video renderer which exercises the same native window route
/// as `cargo run`.  The readback comes from that window renderer's production
/// compositor target after it has been presented to the surface; it is not an
/// offscreen/headless substitute.
pub(crate) struct NativeWindowOracleRenderer {
    frontend: NativeFrontend,
    resources: renderer::ModernAssetFrameResources,
    live_stats: renderer::ModernAssetLiveStats,
}

impl LiveGpuFrameCapture {
    pub fn from_game(game: &mut ZeldaState) -> Self {
        game.with_display_snapshot(Self::from_current_game)
    }

    fn from_current_game(game: &mut ZeldaState) -> Self {
        let cgram = game.cgram_after_first_hdma_line();
        // Capture-point audit (see also the commit-point audit in `commit_palette_provenance_cgram`):
        // the modern renderer substitutes the mirror's committed CGRAM image for the live PPU CGRAM,
        // so audit them here — every captured frame — against the base (pre-HDMA) PPU CGRAM. This is
        // the check that catches a stale committed image between upload commits (e.g. a snapshot
        // restore during a fade). `=panic` aborts on the first divergence.
        if let Ok(audit_mode) = std::env::var("ZELDA3_PALETTE_CGRAM_AUDIT") {
            let audit = game.audit_cgram_mirror(&game.ppu.cgram);
            if !audit.is_clean() {
                let first = audit
                    .mismatches
                    .first()
                    .map(|w| {
                        format!(
                            " first=idx{}:mirror={:04x}:ppu={:04x}",
                            w.index,
                            w.mirror.unwrap_or(0),
                            w.actual
                        )
                    })
                    .unwrap_or_default();
                eprintln!(
                    "palette_cgram_capture_audit mirror_vs_ppu_cgram: mismatches={} unknown={}{first}",
                    audit.mismatches.len(),
                    audit.unknown.len(),
                );
                if audit_mode == "panic" {
                    panic!("mirror CGRAM image diverged from live PPU CGRAM at a render capture");
                }
            }
        }
        let raw_scanlines = game.ppu_scanline_windows();
        let ppu = game.ppu.clone();
        if std::env::var_os("ZELDA3_DEBUG_BG3_PUBLICATION").is_some()
            && game.ram[MAIN_MODULE_INDEX] == 14
            && game.ram[0x11] == 1
        {
            let authored = u16::from_le_bytes([game.ram[0x00ea], game.ram[0x00eb]]) & 0x03ff;
            eprintln!(
                "bg3_publication authored={authored:04x} visible={:04x} main={:02x} sub={:02x}",
                ppu.bg_layer[2].v_scroll, game.ram[MAIN_MODULE_INDEX], game.ram[0x11],
            );
        }
        let source_entries = game.vram_chr_source().as_slice().to_vec();
        let dialogue_active = game.is_dialogue_display_active();
        let bg3_source_tiles = if dialogue_active {
            dialogue_glyph_source_tiles_from_ppu(&ppu)
        } else {
            Vec::new()
        };
        let bg3_vwf_glyph_run_offsets = if dialogue_active {
            game.bg3_vwf_glyph_run_dialogue_offsets()
        } else {
            &[]
        };
        let bg3_vwf_glyph_run_ir_kinds: Vec<_> = if dialogue_active {
            (0..game.bg3_vwf_glyph_runs().len())
                .map(|index| game.bg3_vwf_glyph_run_dialogue_ir(index).map(|op| op.kind))
                .collect()
        } else {
            Vec::new()
        };
        let dialogue_message_id = dialogue_active.then(|| game.current_dialogue_message_id());
        let source_dialogue_ir = if dialogue_active {
            game.current_source_dialogue_ir()
        } else {
            Vec::new()
        };
        let dialogue_ir = game.current_displayed_source_render_dialogue_ir();
        let dialogue_vwf_widths = game.dialogue_vwf_widths().unwrap_or_default();
        let dialogue_layout =
            zelda3::dialogue_ir::layout_dialogue_ir(&dialogue_ir, &dialogue_vwf_widths);
        let dialogue_layout_origin_tile_number =
            (!dialogue_layout.is_empty()).then(|| game.dialogue_vwf_origin_tile_number());
        let bg3_vwf_glyph_runs = if dialogue_active {
            game.bg3_vwf_glyph_runs()
                .iter()
                .enumerate()
                .map(|(index, run)| renderer::GpuBg3VwfGlyphRun {
                    glyph_code: run.glyph_code,
                    origin_tile_number: run.origin_tile_number,
                    x: run.x,
                    y: run.y,
                    width: run.width,
                    dialogue_offset: bg3_vwf_glyph_run_offsets
                        .get(index)
                        .copied()
                        .filter(|offset| *offset != zelda3_compat::UNKNOWN_DIALOGUE_OFFSET),
                    dialogue_ir_kind: bg3_vwf_glyph_run_ir_kinds.get(index).cloned().flatten(),
                })
                .collect()
        } else {
            Vec::new()
        };
        let mode7_source_chars = game.mode7_character_source().map(<[u8]>::to_vec);
        let main_module = game.ram[MAIN_MODULE_INDEX];
        let player_indoors = game.ram[PLAYER_IS_INDOORS];
        let cgram_provenance = game.cgram_provenance_snapshot();
        Self {
            hardware_startup_transient: snes9x_boot_material(
                game.frame_ctr_dbg,
                game.ram[MAIN_MODULE_INDEX],
                game.ram[MAIN_MODULE_INDEX + 1],
                game.ram[MAIN_MODULE_INDEX + 2],
                game.ppu.brightness,
            ),
            ppu,
            cgram,
            raw_scanlines,
            source_entries,
            bg3_source_tiles,
            bg3_vwf_glyph_runs,
            dialogue_message_id,
            source_dialogue_ir,
            dialogue_ir,
            dialogue_layout,
            dialogue_layout_origin_tile_number,
            mode7_source_chars,
            main_module,
            player_indoors,
            cgram_provenance,
        }
    }

    pub fn capture_input(&self) -> renderer::GpuFrameCaptureInput<'_> {
        gpu_frame_capture_from_ppu(
            self.hardware_startup_transient.clone(),
            &self.ppu,
            &self.cgram,
            self.raw_scanlines.as_ref(),
            &self.bg3_source_tiles,
            &self.bg3_vwf_glyph_runs,
            self.dialogue_message_id,
            &self.source_dialogue_ir,
            &self.dialogue_ir,
            &self.dialogue_layout,
            self.dialogue_layout_origin_tile_number,
            Some(&self.cgram_provenance),
        )
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



    pub fn source_entries(&self) -> &[zelda3::LogicalChrSrc] {
        &self.source_entries
    }

    pub fn bg3_source_tiles(&self) -> &[renderer::GpuBg3SourceTile] {
        &self.bg3_source_tiles
    }

    pub fn dialogue_message_id(&self) -> Option<u16> {
        self.dialogue_message_id
    }

    pub fn source_dialogue_ir(&self) -> &[zelda3::dialogue_ir::DialogueIrOp] {
        &self.source_dialogue_ir
    }

    pub fn dialogue_ir(&self) -> &[zelda3::dialogue_ir::DialogueIrOp] {
        &self.dialogue_ir
    }

    pub fn dialogue_layout(&self) -> &[zelda3::dialogue_ir::DialogueGlyphPlacement] {
        &self.dialogue_layout
    }

    pub fn dialogue_layout_origin_tile_number(&self) -> Option<u16> {
        self.dialogue_layout_origin_tile_number
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

/// Model the reset-only material surface observed from Snes9x. It deliberately
/// has no PPU input: the residue is reset-stack state, while the Nintendo card
/// is a promoted PNG asset with no authored SNES source state in Rust yet.
fn snes9x_boot_material(
    game_frame: u32,
    main_module: u8,
    submodule: u8,
    subsubmodule: u8,
    brightness: u8,
) -> Option<renderer::gpu_frame::HardwareStartupTransient> {
    // The ROM's boot card is its own intro state, not a host-frame schedule:
    // Snes9x shows it from intro substep 2 through 0x19, and substep 0x1a
    // enters forced blank. This is the semantic publication boundary for the
    // PNG material promoted from the ROM-owned DMA surface.
    let nintendo_presents_visible = (84..=180).contains(&game_frame)
        && submodule == 1
        // The port's live substep is sampled on a different phase from the
        // displayed ROM substep, so it is intentionally not used to shorten
        // this Snes9x-observed publication interval yet.
        && subsubmodule <= 0x19;
    if main_module != 0 || (game_frame != 83 && !nintendo_presents_visible) {
        return None;
    }
    const POWER_ON_RGBA: [u8; 4] = [0xad, 0x52, 0xad, 0xff];
    const STACK_CORNER_RESIDUE: [usize; 4] = [0, 8, 15, 23];
    let mut rgba = [[0u8; 4]; 64];
    if game_frame == 83 {
        for pixel in STACK_CORNER_RESIDUE {
            rgba[pixel] = POWER_ON_RGBA;
        }
    }
    Some(renderer::gpu_frame::HardwareStartupTransient {
        rgba,
        origins: [(0, 0), (85, 85)],
        direct_pixels: if nintendo_presents_visible {
            nintendo_presents_asset_pixels()
                .iter()
                .copied()
                .map(|mut pixel| {
                    pixel.rgba = scale_boot_asset_brightness(pixel.rgba, brightness);
                    pixel
                })
                .collect()
        } else {
            Vec::new()
        },
    })
}

fn scale_boot_asset_brightness(rgba: [u8; 4], brightness: u8) -> [u8; 4] {
    let scale = |channel: u8| {
        let c5 = u32::from(channel) >> 3;
        let scaled5 = (c5 * u32::from(brightness.min(15)) + 7) / 15;
        ((scaled5 << 3) | (scaled5 >> 2)) as u8
    };
    [scale(rgba[0]), scale(rgba[1]), scale(rgba[2]), rgba[3]]
}

fn nintendo_presents_asset_pixels() -> &'static [renderer::gpu_frame::HardwareStartupDirectPixel]
{
    static PIXELS: OnceLock<Vec<renderer::gpu_frame::HardwareStartupDirectPixel>> = OnceLock::new();
    PIXELS.get_or_init(|| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../assets/boot/nintendo_presents.png");
        let (rgba, width, height) = decode_rgba_png(&path)
            .unwrap_or_else(|| panic!("required boot PNG asset failed to decode: {}", path.display()));
        assert_eq!((width, height), (56, 16), "boot PNG asset has unexpected dimensions");
        rgba.chunks_exact(4)
            .enumerate()
            .filter_map(|(index, rgba)| {
                (rgba[3] != 0).then_some(renderer::gpu_frame::HardwareStartupDirectPixel {
                    screen_x: 96 + (index % width as usize) as i16,
                    screen_y: 104 + (index / width as usize) as i16,
                    rgba: [rgba[0], rgba[1], rgba[2], rgba[3]],
                })
            })
            .collect()
    })
}

/// Native play has one authoritative visual path: the PNG/asset GPU compositor.
/// There is no VRAM-compositor presentation path.
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
        let cpu_video = std::env::var_os("ZELDA3_COMPARE_VIDEO_CPU").is_some();
        let resources = if cpu_video {
            renderer::ModernIndexCompareResources::load_cpu_compare(&repo_root)?
        } else {
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)?
        };
        Ok(Self {
            resources,
            cpu_video,
            render_cache: std::sync::Mutex::new(None),
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
        let key = validation_cache_key(&capture);
        if let Some((cached_key, rgba)) = self.render_cache.lock().unwrap().as_ref() {
            if *cached_key == key {
                return Ok(GpuRgbaReadbackFrame::from_rgba(rgba.clone()));
            }
        }
        let frame = if self.cpu_video {
            let gpu_frame = capture.gpu_frame();
            let src_table = renderer::source_table_from_entries(capture.source_entries());
            let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(
                capture.player_indoors(),
            );
            let render = renderer::modern_gpu::render_modern_index_compare_frame(
                &gpu_frame,
                Some(&src_table),
                self.resources.source_atlas(),
                None,
                None,
                None,
                scene,
                None,
                true,
            );
            GpuRgbaReadbackFrame::from_rgba(render.rgba)
        } else {
            render_modern_asset_capture_rgba(&capture, &self.resources)
                .map(|render| render.frame)?
        };
        *self.render_cache.lock().unwrap() = Some((key, frame.as_slice().to_vec()));
        Ok(frame)
    }

    pub(crate) fn trace_game_pixel(
        &self,
        game: &mut ZeldaState,
        x: i16,
        y: i16,
    ) -> Result<Vec<String>, String> {
        let capture = capture_gpu_frame_from_game(game);
        let gpu_frame = capture.gpu_frame();
        let scene =
            renderer::ModernAssetFrameScene::from_player_indoors_flag(capture.player_indoors());
        let production = self.resources.render_production_gpu_asset_rgba_from_entries(
            &gpu_frame,
            capture.source_entries(),
            scene,
        )?;
        let pixel_offset = (usize::try_from(y).unwrap_or(usize::MAX) * 256
            + usize::try_from(x).unwrap_or(usize::MAX))
        .saturating_mul(4);
        let production_pixel = production.rgba
            .get(pixel_offset..pixel_offset.saturating_add(4))
            .unwrap_or(&[]);
        Ok(vec![format!(
            "production pixel={production_pixel:02x?} via={}",
            production.via,
        )])
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

impl NativeWindowOracleRenderer {
    pub(crate) fn load_from_env() -> Result<Self, String> {
        if std::env::var_os("ZELDA3_COMPARE_VIDEO_CPU").is_some() {
            return Err(
                "ZELDA3_COMPARE_VIDEO_CPU is no longer supported: Snes9x video comparison requires the native window renderer"
                    .to_string(),
            );
        }
        let repo_root = repo_root();
        let (resources, _live_mode) =
            renderer::ModernAssetFrameResources::load_live_gpu_from_env(&repo_root)?;
        let frontend = NativeFrontend::new_with_options(
            256,
            224,
            NativeFrontendOptions {
                scale: 1,
                enable_audio: false,
                fullscreen: false,
                frame_pacing: false,
            },
        )?;
        Ok(Self {
            frontend,
            resources,
            live_stats: renderer::ModernAssetLiveStats::from_env(),
        })
    }

    pub(crate) fn render_game_rgba(
        &mut self,
        game: &mut ZeldaState,
    ) -> Result<GpuRgbaReadbackFrame, String> {
        let capture = capture_gpu_frame_from_game(game);
        let report = self.frontend.present_modern_asset_live_frame_from_entries(
            capture.modern_asset_present_input(&self.resources, &mut self.live_stats),
        );
        if let Some(reason) = report.failure_line() {
            return Err(format!("native window renderer rejected frame: {reason}"));
        }
        self.frontend
            .read_modern_gpu_target_rgba()
            .map(GpuRgbaReadbackFrame::from_rgba)
            .ok_or_else(|| {
                "native window renderer did not produce a modern GPU target".to_string()
            })
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zelda3-bin lives under the workspace root")
        .to_path_buf()
}

#[derive(Deserialize)]
struct DialogueGlyphAtlasManifest {
    tiles: Vec<DialogueGlyphAtlasTile>,
}

#[derive(Deserialize)]
struct DialogueGlyphAtlasTile {
    rect: [u32; 4],
    source_pack: u16,
    source_tile: u16,
}

#[derive(Deserialize)]
struct DialogueFontTileAtlasManifest {
    tiles: Vec<DialogueFontTileAtlasTile>,
}

#[derive(Deserialize)]
struct DialogueFontTileAtlasTile {
    rect: [u32; 4],
    source_pack: u16,
    source_tile: u16,
}

#[derive(Deserialize)]
struct ChrSourceManifest {
    palette: ChrSourcePalette,
    // v2 sidecars carry per-row palettes and per-block tile->row assignments; v1
    // sidecars have neither (the flat `palette.index_to_rgb` is authoritative).
    #[serde(default)]
    palette_rows: Vec<ChrSourcePaletteRow>,
    #[serde(default)]
    blocks: Vec<ChrSourceBlock>,
}

#[derive(Deserialize)]
struct ChrSourcePalette {
    index_to_rgb: Vec<[u8; 3]>,
}

#[derive(Deserialize)]
struct ChrSourcePaletteRow {
    id: u32,
    #[serde(default)]
    index_to_rgb: Vec<[u8; 3]>,
}

#[derive(Deserialize)]
struct ChrSourceBlock {
    tile_start: u32,
    tile_count: u32,
    #[serde(default)]
    tile_palette_rows: Vec<u32>,
}

impl ChrSourceManifest {
    /// Colors used to decode one glyph atlas tile back to CHR indices. In v2 the
    /// tile's palette row (resolved via its sidecar block's `tile_palette_rows`)
    /// supplies exactly its 4 colors; in v1 the flat combined palette is used.
    fn glyph_tile_palette(&self, source_tile: u16) -> Result<&[[u8; 3]], String> {
        if self.palette_rows.is_empty() {
            return Ok(&self.palette.index_to_rgb);
        }
        let source_tile = source_tile as u32;
        let block = self
            .blocks
            .iter()
            .find(|b| source_tile >= b.tile_start && source_tile < b.tile_start + b.tile_count)
            .ok_or_else(|| {
                format!("glyph source_tile {source_tile} not covered by any sidecar block")
            })?;
        let offset = (source_tile - block.tile_start) as usize;
        let row_id = *block.tile_palette_rows.get(offset).ok_or_else(|| {
            format!("glyph source_tile {source_tile} beyond block tile_palette_rows")
        })?;
        let row = self
            .palette_rows
            .iter()
            .find(|r| r.id == row_id)
            .ok_or_else(|| format!("glyph palette row {row_id} undefined in sidecar"))?;
        Ok(&row.index_to_rgb)
    }
}

#[derive(Clone, Copy)]
#[cfg(test)]
struct DialogueGlyphSourceTile {
    source_key: u64,
    indices: [u8; 64],
}

struct DialogueGlyphSourceMatcher {
    #[cfg(test)]
    tiles: Vec<DialogueGlyphSourceTile>,
    source_key_by_indices: HashMap<[u8; 64], u64>,
}

impl DialogueGlyphSourceMatcher {
    fn load(repo_root: &Path) -> Result<Self, String> {
        let atlas_manifest_path =
            repo_root.join("generated/zelda3_assets/atlas/dialogue_glyph_tiles.json");
        let atlas_png_path =
            repo_root.join("generated/zelda3_assets/atlas/dialogue_glyph_tiles.png");
        let source_manifest_path = repo_root.join("assets/chr/1w-2d.json");

        let atlas_manifest: DialogueGlyphAtlasManifest = serde_json::from_slice(
            &fs::read(&atlas_manifest_path)
                .map_err(|err| format!("{}: {err}", atlas_manifest_path.display()))?,
        )
        .map_err(|err| format!("{}: {err}", atlas_manifest_path.display()))?;
        let source_manifest: ChrSourceManifest = serde_json::from_slice(
            &fs::read(&source_manifest_path)
                .map_err(|err| format!("{}: {err}", source_manifest_path.display()))?,
        )
        .map_err(|err| format!("{}: {err}", source_manifest_path.display()))?;
        let (rgba, width, height) = decode_rgba_png(&atlas_png_path)
            .ok_or_else(|| format!("{}: failed to decode RGBA PNG", atlas_png_path.display()))?;

        #[cfg(test)]
        let mut tiles = Vec::with_capacity(atlas_manifest.tiles.len());
        let mut source_key_by_indices = HashMap::new();
        for tile in atlas_manifest.tiles {
            let palette = source_manifest
                .glyph_tile_palette(tile.source_tile)
                .map_err(|err| format!("{}: {err}", source_manifest_path.display()))?;
            let indices = decode_dialogue_glyph_tile(&rgba, width, height, palette, tile.rect)
                .map_err(|err| format!("{}: {err}", atlas_png_path.display()))?;
            let source_key = renderer::modern_source_atlas::modern_source_key(
                9,
                tile.source_pack,
                tile.source_tile,
            );
            #[cfg(test)]
            tiles.push(DialogueGlyphSourceTile {
                source_key,
                indices,
            });
            source_key_by_indices.entry(indices).or_insert(source_key);
        }

        let font_manifest_path =
            repo_root.join("generated/zelda3_assets/atlas/dialogue_font_tiles.json");
        let font_png_path = repo_root.join("generated/zelda3_assets/atlas/dialogue_font_tiles.png");
        if font_manifest_path.is_file() || font_png_path.is_file() {
            let font_manifest: DialogueFontTileAtlasManifest = serde_json::from_slice(
                &fs::read(&font_manifest_path)
                    .map_err(|err| format!("{}: {err}", font_manifest_path.display()))?,
            )
            .map_err(|err| format!("{}: {err}", font_manifest_path.display()))?;
            let (font_rgba, font_width, font_height) = decode_rgba_png(&font_png_path)
                .ok_or_else(|| format!("{}: failed to decode RGBA PNG", font_png_path.display()))?;
            #[cfg(test)]
            tiles.reserve(font_manifest.tiles.len());
            for tile in font_manifest.tiles {
                let indices =
                    decode_dialogue_font_tile(&font_rgba, font_width, font_height, tile.rect)
                        .map_err(|err| format!("{}: {err}", font_png_path.display()))?;
                let source_key = renderer::modern_source_atlas::modern_source_key(
                    11,
                    tile.source_pack,
                    tile.source_tile,
                );
                #[cfg(test)]
                tiles.push(DialogueGlyphSourceTile {
                    source_key,
                    indices,
                });
                source_key_by_indices.entry(indices).or_insert(source_key);
            }
        }

        Ok(Self {
            #[cfg(test)]
            tiles,
            source_key_by_indices,
        })
    }

    fn source_key_for_indices(&self, indices: &[u8; 64]) -> Option<u64> {
        self.source_key_by_indices.get(indices).copied()
    }

    #[cfg(test)]
    fn tile_for_source_key(&self, source_key: u64) -> Option<DialogueGlyphSourceTile> {
        self.tiles
            .iter()
            .copied()
            .find(|tile| tile.source_key == source_key)
    }

    #[cfg(test)]
    fn unique_pattern_count(&self) -> usize {
        self.source_key_by_indices.len()
    }
}

fn decode_dialogue_glyph_tile(
    rgba: &[u8],
    width: u32,
    height: u32,
    palette: &[[u8; 3]],
    rect: [u32; 4],
) -> Result<[u8; 64], String> {
    let [x, y, w, h] = rect;
    if w != 8 || h != 8 {
        return Err(format!("expected 8x8 glyph tile rect, got {w}x{h}"));
    }
    if x + w > width || y + h > height {
        return Err(format!(
            "glyph tile rect [{x}, {y}, {w}, {h}] exceeds {width}x{height} atlas"
        ));
    }

    let mut indices = [0u8; 64];
    for py in 0..8u32 {
        for px in 0..8u32 {
            let pixel = (((y + py) * width + (x + px)) * 4) as usize;
            let rgb = [rgba[pixel], rgba[pixel + 1], rgba[pixel + 2]];
            let Some(index) = palette.iter().position(|candidate| *candidate == rgb) else {
                return Err(format!(
                    "glyph tile pixel {},{} uses color rgb({},{},{}) not found in 1w-2d palette",
                    x + px,
                    y + py,
                    rgb[0],
                    rgb[1],
                    rgb[2]
                ));
            };
            if index > 3 {
                return Err(format!(
                    "glyph tile pixel {},{} decoded to palette index {index}, expected 0..3",
                    x + px,
                    y + py
                ));
            }
            indices[(py * 8 + px) as usize] = index as u8;
        }
    }
    Ok(indices)
}

fn decode_dialogue_font_tile(
    rgba: &[u8],
    width: u32,
    height: u32,
    rect: [u32; 4],
) -> Result<[u8; 64], String> {
    const INDEX_COLORS: [[u8; 4]; 4] = [
        [0, 0, 0, 0],
        [248, 248, 248, 255],
        [88, 88, 88, 255],
        [184, 184, 184, 255],
    ];

    let [x, y, w, h] = rect;
    if w != 8 || h != 8 {
        return Err(format!("expected 8x8 font tile rect, got {w}x{h}"));
    }
    if x + w > width || y + h > height {
        return Err(format!(
            "font tile rect [{x}, {y}, {w}, {h}] exceeds {width}x{height} atlas"
        ));
    }

    let mut indices = [0u8; 64];
    for py in 0..8u32 {
        for px in 0..8u32 {
            let pixel = (((y + py) * width + (x + px)) * 4) as usize;
            let rgba_pixel = [
                rgba[pixel],
                rgba[pixel + 1],
                rgba[pixel + 2],
                rgba[pixel + 3],
            ];
            let Some(index) = INDEX_COLORS
                .iter()
                .position(|candidate| *candidate == rgba_pixel)
            else {
                return Err(format!(
                    "font tile pixel {},{} uses rgba({},{},{},{}) not found in dialogue font palette",
                    x + px,
                    y + py,
                    rgba_pixel[0],
                    rgba_pixel[1],
                    rgba_pixel[2],
                    rgba_pixel[3]
                ));
            };
            indices[(py * 8 + px) as usize] = index as u8;
        }
    }
    Ok(indices)
}

fn dialogue_glyph_source_matcher() -> Option<&'static DialogueGlyphSourceMatcher> {
    static MATCHER: OnceLock<Option<DialogueGlyphSourceMatcher>> = OnceLock::new();
    MATCHER
        .get_or_init(|| match DialogueGlyphSourceMatcher::load(&repo_root()) {
            Ok(matcher) => Some(matcher),
            Err(err) => {
                eprintln!("dialogue glyph source PNG matcher disabled: {err}");
                None
            }
        })
        .as_ref()
}

fn dialogue_glyph_source_tiles_from_ppu(
    _ppu: &snes::ppu::PpuState,
) -> Vec<renderer::GpuBg3SourceTile> {
    // A BG3 VWF tile is a transient composition, not a source-art tile.  The
    // old pattern-only matcher could mistake one of those compositions for an
    // unrelated sprite-sheet PNG with the same 2bpp pattern, then replace the
    // correct live glyph with that PNG.  VWF text has an explicit semantic
    // glyph-run channel below; do not infer a source identity from pixels.
    Vec::new()
}

fn dialogue_glyph_source_tiles_from_ppu_with_matcher(
    ppu: &snes::ppu::PpuState,
    matcher: &DialogueGlyphSourceMatcher,
) -> Vec<renderer::GpuBg3SourceTile> {
    let bg3 = &ppu.bg_layer[2];
    let tilemap_base = bg3.tilemap_adr as usize;
    let chr_base = bg3.tile_adr as usize;
    let cols = if bg3.tilemap_wider { 64usize } else { 32 };
    let rows = if bg3.tilemap_higher { 64usize } else { 32 };
    let mut by_chr_tile = HashMap::<(u16, u16), u64>::new();

    for ty in 0..rows {
        for tx in 0..cols {
            let q = (if bg3.tilemap_wider && tx >= 32 { 1 } else { 0 })
                + (if bg3.tilemap_higher && ty >= 32 {
                    if bg3.tilemap_wider {
                        2
                    } else {
                        1
                    }
                } else {
                    0
                });
            let within = (ty % 32) * 32 + (tx % 32);
            let addr = tilemap_base + q * 0x400 + within;
            let entry_word = ppu.vram.get(addr).copied().unwrap_or(0);
            if entry_word == 0 {
                continue;
            }
            let tile_number = entry_word & 0x03ff;
            let indices = renderer::modern_extract::decode_snes_2bpp_tile_indices(
                &ppu.vram,
                chr_base,
                tile_number,
            );
            if let Some(source_key) = matcher.source_key_for_indices(&indices) {
                by_chr_tile
                    .entry((bg3.tile_adr, tile_number))
                    .or_insert(source_key);
            }
        }
    }

    let mut source_tiles = by_chr_tile
        .into_iter()
        .map(
            |((chr_base, tile_number), source_key)| renderer::GpuBg3SourceTile {
                chr_base,
                tile_number,
                source_key,
            },
        )
        .collect::<Vec<_>>();
    source_tiles.sort_by_key(|tile| (tile.chr_base, tile.tile_number, tile.source_key));
    source_tiles
}

impl crate::play_renderer::PlayRendererBackend for GpuPlayRenderer {
    fn name(&self) -> &'static str {
        "gpu_render"
    }

    fn configure_frontend(&self, frontend: &mut NativeFrontend) {
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
            if std::env::var_os("ZELDA3_DEBUG_ASSET_COVERAGE").is_some() {
                let gpu_frame = capture.gpu_frame();
                let source_table = renderer::source_table_from_entries(capture.source_entries());
                let resolved = renderer::modern_extract::extract_asset_resolved_modern_frame_from_sources(
                    &gpu_frame,
                    &source_table,
                    self.modern_assets.source_atlas().expect("live renderer has source atlas"),
                );
                let unkeyed = resolved
                    .sprite_cells
                    .iter()
                    .filter(|cell| cell.source_key == renderer::modern_hd_overrides::NO_SOURCE_KEY)
                    .map(|cell| {
                        let pattern = cell
                            .indices
                            .iter()
                            .map(|index| format!("{index:x}"))
                            .collect::<String>();
                        format!("cell={} pattern={pattern}", cell.id)
                    })
                    .collect::<Vec<_>>();
                eprintln!(
                    "asset_coverage_missing main={:02x} sub={:02x} mode={} obj=({:04x},{:04x}) {} patterns=[{}]",
                    capture.main_module,
                    game.ram[0x11],
                    gpu_frame.mode,
                    gpu_frame.obj.tile_adr1,
                    gpu_frame.obj.tile_adr2,
                    resolved.missing_source_report(64),
                    unkeyed.join(", ")
                );
            }
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
    use std::sync::atomic::{AtomicU64, Ordering};
    static RENDER_NS: AtomicU64 = AtomicU64::new(0);
    static CALLS: AtomicU64 = AtomicU64::new(0);
    let timing = std::env::var_os("ZELDA3_SNES9X_TIMING").is_some();
    let start = timing.then(std::time::Instant::now);
    let gpu_frame = capture.gpu_frame();
    let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(capture.player_indoors());
    let render = resources.render_production_gpu_asset_rgba_from_entries(
        &gpu_frame,
        capture.source_entries(),
        scene,
    )?;
    if let Some(start) = start {
        let ns = RENDER_NS.fetch_add(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
        let calls = CALLS.fetch_add(1, Ordering::Relaxed) + 1;
        if calls % 2000 == 0 {
            eprintln!(
                "gpu_render_timing calls={calls} production_render_ms={}",
                (ns + start.elapsed().as_nanos() as u64) / 1_000_000
            );
        }
    }
    Ok(ModernAssetGpuReadbackFrame {
        frame: GpuRgbaReadbackFrame::from_rgba(render.rgba),
        #[cfg(test)]
        variant_stats: render.variant_stats,
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
    let source_atlas = resources
        .source_atlas()
        .ok_or_else(|| "modern asset GPU validation requires source atlas".to_string())?;
    let source_table = renderer::source_table_from_entries(capture.source_entries());
    let extract_start = Instant::now();
    let modern_assets = renderer::modern_extract::extract_asset_resolved_modern_frame_from_sources(
        &gpu_frame,
        &source_table,
        source_atlas,
    );
    let bg_extract_nanos = extract_start.elapsed().as_nanos();
    let mut validation =
        resources.validate_full_gpu_asset_from_resolved_frame(&modern_assets, scene)?;
    validation.timings.bg_extract_nanos = bg_extract_nanos;
    validate_no_dynamic_bg3_text_chunks_from_assets(capture, &modern_assets)?;
    Ok(validation)
}

#[cfg(test)]
fn validate_no_dynamic_bg3_text_chunks(
    capture: &LiveGpuFrameCapture,
    resources: &renderer::ModernIndexCompareResources,
) -> Result<(), String> {
    let source_atlas = resources
        .source_atlas()
        .ok_or_else(|| "modern asset GPU validation requires source atlas".to_string())?;
    let gpu_frame = capture.gpu_frame();
    let source_table = renderer::source_table_from_entries(capture.source_entries());
    let modern_assets = renderer::modern_extract::extract_asset_resolved_modern_frame_from_sources(
        &gpu_frame,
        &source_table,
        source_atlas,
    );
    validate_no_dynamic_bg3_text_chunks_from_assets(capture, &modern_assets)
}

fn validate_no_dynamic_bg3_text_chunks_from_assets(
    capture: &LiveGpuFrameCapture,
    modern_assets: &renderer::modern_extract::AssetResolvedModernFrame,
) -> Result<(), String> {
    const BULK_BG3_FILL_INSTANCE_THRESHOLD: usize = 256;

    let gpu_frame = capture.gpu_frame();
    if gpu_frame.forced_blank || gpu_frame.brightness == 0 {
        return Ok(());
    }
    let mut dynamic_key_counts = HashMap::<u64, usize>::new();
    for inst in &modern_assets.frame.bg_layers[2].index_tiles {
        if !bg3_tile_overlaps_viewport(inst) {
            continue;
        }
        if let Some(key) = dynamic_bg3_text_key_for_instance(inst, &modern_assets.bg_cells) {
            *dynamic_key_counts.entry(key).or_default() += 1;
        }
    }
    let mut dynamic_count = 0usize;
    let mut skipped_bulk_fill_count = 0usize;
    let mut samples = Vec::new();
    for inst in &modern_assets.frame.bg_layers[2].index_tiles {
        if !bg3_tile_overlaps_viewport(inst) {
            continue;
        }
        let Some(key) = dynamic_bg3_text_key_for_instance(inst, &modern_assets.bg_cells) else {
            continue;
        };
        let cell_source_key = modern_assets
            .bg_cells
            .get(inst.cell_id as usize)
            .map(|cell| cell.source_key)
            .unwrap_or(renderer::modern_hd_overrides::NO_SOURCE_KEY);
        if dynamic_key_counts.get(&key).copied().unwrap_or(0) >= BULK_BG3_FILL_INSTANCE_THRESHOLD {
            skipped_bulk_fill_count += 1;
            continue;
        }
        dynamic_count += 1;
        if samples.len() < 4 {
            let ppu_tile = bg3_ppu_tile_debug_for_screen(capture, inst.screen_x, inst.screen_y)
                .unwrap_or_else(|| "ppu_tile=<none>".to_string());
            samples.push(format!(
                "cell={} xy=({}, {}) inst_key=0x{:016x} cell_key=0x{:016x} {}",
                inst.cell_id,
                inst.screen_x,
                inst.screen_y,
                inst.source_key,
                cell_source_key,
                ppu_tile
            ));
        }
    }
    if dynamic_count == 0 {
        return Ok(());
    }
    Err(format!(
        "modern asset GPU validation still has BG3 dynamic text chunks count={} skipped_bulk_fill={} samples=[{}]",
        dynamic_count,
        skipped_bulk_fill_count,
        samples.join(", ")
    ))
}

fn bg3_ppu_tile_debug_for_screen(
    capture: &LiveGpuFrameCapture,
    screen_x: i16,
    screen_y: i16,
) -> Option<String> {
    let ppu = capture.ppu();
    let bg3 = &ppu.bg_layer[2];
    let cols = if bg3.tilemap_wider { 64usize } else { 32 };
    let rows = if bg3.tilemap_higher { 64usize } else { 32 };
    let map_w = (cols * 8) as i32;
    let map_h = (rows * 8) as i32;
    if map_w == 0 || map_h == 0 {
        return None;
    }
    let sx = (i32::from(screen_x) + i32::from(bg3.h_scroll)).rem_euclid(map_w) as usize;
    let sy = (i32::from(screen_y) + i32::from(bg3.v_scroll) + 1).rem_euclid(map_h) as usize;
    let tx = sx / 8;
    let ty = sy / 8;
    let q = (if bg3.tilemap_wider && tx >= 32 { 1 } else { 0 })
        + (if bg3.tilemap_higher && ty >= 32 {
            if bg3.tilemap_wider {
                2
            } else {
                1
            }
        } else {
            0
        });
    let within = (ty % 32) * 32 + (tx % 32);
    let tilemap_addr = bg3.tilemap_adr as usize + q * 0x400 + within;
    let entry_word = ppu.vram.get(tilemap_addr).copied()?;
    let tile_number = entry_word & 0x03ff;
    let pal = (entry_word >> 10) & 7;
    let raw = renderer::modern_extract::decode_snes_2bpp_tile_indices(
        &ppu.vram,
        bg3.tile_adr as usize,
        tile_number,
    );
    let matcher_key = dialogue_glyph_source_matcher()
        .and_then(|matcher| matcher.source_key_for_indices(&raw))
        .map(|key| format!("0x{key:016x}"))
        .unwrap_or_else(|| "none".to_string());
    Some(format!(
        "ppu_tile=map=0x{tilemap_addr:04x} entry=0x{entry_word:04x} chr=0x{:04x} tile=0x{tile_number:03x} pal={} matcher_key={matcher_key} raw={}",
        bg3.tile_adr,
        pal,
        raw.iter()
            .map(|value| char::from_digit(u32::from(*value), 16).unwrap_or('?'))
            .collect::<String>()
    ))
}

fn dynamic_bg3_text_key_for_instance(
    inst: &renderer::modern_frame::ModernIndexTileInstance,
    bg_cells: &[renderer::modern_index_atlas::ModernIndexTile],
) -> Option<u64> {
    if is_dynamic_or_unkeyed_bg3_text_key(inst.source_key) {
        return Some(inst.source_key);
    }
    let cell_source_key = bg_cells
        .get(inst.cell_id as usize)
        .map(|cell| cell.source_key)
        .unwrap_or(renderer::modern_hd_overrides::NO_SOURCE_KEY);
    is_dynamic_or_unkeyed_bg3_text_key(cell_source_key).then_some(cell_source_key)
}

fn bg3_tile_overlaps_viewport(inst: &renderer::modern_frame::ModernIndexTileInstance) -> bool {
    let x = i32::from(inst.screen_x);
    let y = i32::from(inst.screen_y);
    x < 256 && x + 8 > 0 && y < 224 && y + 8 > 0
}

fn is_dynamic_or_unkeyed_bg3_text_key(source_key: u64) -> bool {
    source_key == renderer::modern_hd_overrides::NO_SOURCE_KEY || ((source_key >> 32) as u8) == 7
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
    for tile in capture.bg3_source_tiles() {
        tile.chr_base.hash(&mut hasher);
        tile.tile_number.hash(&mut hasher);
        tile.source_key.hash(&mut hasher);
    }
    for run in &capture.bg3_vwf_glyph_runs {
        run.glyph_code.hash(&mut hasher);
        run.origin_tile_number.hash(&mut hasher);
        run.x.hash(&mut hasher);
        run.y.hash(&mut hasher);
        run.width.hash(&mut hasher);
        run.dialogue_ir_kind.hash(&mut hasher);
    }
    capture.dialogue_message_id().hash(&mut hasher);
    capture.source_dialogue_ir().hash(&mut hasher);
    capture.dialogue_ir().hash(&mut hasher);
    capture.dialogue_layout().hash(&mut hasher);
    capture
        .dialogue_layout_origin_tile_number()
        .hash(&mut hasher);
    capture.player_indoors().hash(&mut hasher);
    capture.cgram_provenance.words.hash(&mut hasher);
    capture.cgram_provenance.known.hash(&mut hasher);

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
    hardware_startup_transient: Option<renderer::gpu_frame::HardwareStartupTransient>,
    ppu: &'a snes::ppu::PpuState,
    cgram: &'a [u16],
    raw_scanlines: &'a RawScanlineFrame,
    bg3_source_tiles: &'a [renderer::GpuBg3SourceTile],
    bg3_vwf_glyph_runs: &'a [renderer::GpuBg3VwfGlyphRun],
    dialogue_message_id: Option<u16>,
    source_dialogue_ir: &'a [zelda3::dialogue_ir::DialogueIrOp],
    dialogue_ir: &'a [zelda3::dialogue_ir::DialogueIrOp],
    dialogue_layout: &'a [zelda3::dialogue_ir::DialogueGlyphPlacement],
    dialogue_layout_origin_tile_number: Option<u16>,
    cgram_provenance: Option<&'a zelda3_palette::CgramProvenanceSnapshot>,
) -> renderer::GpuFrameCaptureInput<'a> {
    renderer::GpuFrameCaptureInput {
        hardware_startup_transient,
        registers: gpu_frame_register_snapshot_from_ppu(ppu),
        cgram,
        raw_scanlines,
        bg3_source_tiles,
        bg3_vwf_glyph_runs,
        dialogue_message_id,
        source_dialogue_ir,
        dialogue_ir,
        dialogue_layout,
        dialogue_layout_origin_tile_number,
        cgram_provenance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUBMODULE_INDEX: usize = 0x11;
    const SAVED_MODULE_FOR_MENU: usize = 0x010c;
    const MESSAGING_MODULE: usize = 0x1cd8;
    #[test]
    fn dialogue_glyph_source_matcher_loads_generated_png_tiles() {
        let matcher = DialogueGlyphSourceMatcher::load(&repo_root())
            .expect("dialogue glyph source PNG matcher should load");
        let glyph_source_key = renderer::modern_source_atlas::modern_source_key(9, 103, 128);
        let font_source_key = renderer::modern_source_atlas::modern_source_key(11, 0, 0xc7);

        assert_eq!(matcher.tiles.len(), 896);
        assert!(matcher.unique_pattern_count() > 100);
        assert!(matcher.tile_for_source_key(glyph_source_key).is_some());
        assert!(matcher.tile_for_source_key(font_source_key).is_some());
    }

    #[test]
    fn bg3_dialogue_glyph_sidecar_renders_from_png_in_default_gpu_path() {
        let matcher = DialogueGlyphSourceMatcher::load(&repo_root())
            .expect("dialogue glyph source PNG matcher should load");
        let glyph = matcher
            .tiles
            .iter()
            .copied()
            .find(|tile| {
                tile.indices.iter().any(|&index| index != 0)
                    && matcher.source_key_for_indices(&tile.indices) == Some(tile.source_key)
            })
            .expect("nonblank canonical dialogue glyph source tile should exist");
        let source_key = glyph.source_key;
        let mut ppu = snes::ppu::PpuState::default();
        ppu.mode = 1;
        ppu.brightness = 15;
        ppu.screen_enabled = [0x04, 0x00];
        ppu.bg_layer[2].tilemap_adr = 0x0000;
        ppu.bg_layer[2].tile_adr = 0x1000;
        ppu.vram[0] = 7 | (3 << 10);
        encode_2bpp_tile(&mut ppu.vram, 0x1000, 7, &glyph.indices);

        let bg3_source_tiles = dialogue_glyph_source_tiles_from_ppu_with_matcher(&ppu, &matcher);
        assert_eq!(
            bg3_source_tiles,
            vec![renderer::GpuBg3SourceTile {
                chr_base: 0x1000,
                tile_number: 7,
                source_key,
            }]
        );

        let mut raw_scanlines: Box<RawScanlineFrame> =
            Box::new([(0, 0, 0, 0, 0x04, [0; 4], [0; 4], [0; 8], false); 224]);
        for scanline in raw_scanlines.iter_mut() {
            scanline.5[2] = 0;
            scanline.6[2] = 0;
        }
        let capture = LiveGpuFrameCapture {
            hardware_startup_transient: None,
            ppu,
            cgram: vec![0u16; 256],
            raw_scanlines,
            source_entries: Vec::new(),
            bg3_source_tiles,
            bg3_vwf_glyph_runs: Vec::new(),
            dialogue_message_id: None,
            source_dialogue_ir: Vec::new(),
            dialogue_ir: Vec::new(),
            dialogue_layout: Vec::new(),
            dialogue_layout_origin_tile_number: None,
            mode7_source_chars: None,
            main_module: 0,
            player_indoors: 1,
            cgram_provenance: Default::default(),
        };
        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)
                .expect("modern asset resources should load");

        let render = render_modern_asset_capture_rgba(&capture, &resources)
            .expect("GPU readback should render BG3 dialogue glyph from PNG source");
        assert_eq!(render.via, "variant-gpu");
        validate_no_dynamic_bg3_text_chunks(&capture, &resources)
            .expect("dialogue glyph sidecar should remove BG3 dynamic text chunks");
        let stats = render
            .variant_stats
            .expect("variant renderer should report draw stats");
        assert_eq!(stats.missing_art_draws, 0);
        assert!(
            stats.stable_preview_draws > 0,
            "expected dialogue glyph sidecar key to resolve to a stable PNG preview draw, got {stats:?}"
        );
    }

    #[test]
    fn bg3_dialogue_font_sidecar_renders_ending_font_tile_from_png() {
        let matcher = DialogueGlyphSourceMatcher::load(&repo_root())
            .expect("dialogue glyph source PNG matcher should load");
        let source_key = renderer::modern_source_atlas::modern_source_key(11, 0, 0xc7);
        let font_tile = matcher
            .tile_for_source_key(source_key)
            .expect("ending font tile should exist in dialogue font source atlas");
        let mut ppu = snes::ppu::PpuState::default();
        ppu.mode = 1;
        ppu.brightness = 15;
        ppu.screen_enabled = [0x04, 0x00];
        ppu.bg_layer[2].tilemap_adr = 0x0000;
        ppu.bg_layer[2].tile_adr = 0x7000;
        ppu.vram[0] = 0xc7 | (3 << 10);
        encode_2bpp_tile(&mut ppu.vram, 0x7000, 0xc7, &font_tile.indices);

        let bg3_source_tiles = dialogue_glyph_source_tiles_from_ppu_with_matcher(&ppu, &matcher);
        assert_eq!(
            bg3_source_tiles,
            vec![renderer::GpuBg3SourceTile {
                chr_base: 0x7000,
                tile_number: 0xc7,
                source_key,
            }]
        );

        let mut raw_scanlines: Box<RawScanlineFrame> =
            Box::new([(0, 0, 0, 0, 0x04, [0; 4], [0; 4], [0; 8], false); 224]);
        for scanline in raw_scanlines.iter_mut() {
            scanline.5[2] = 0;
            scanline.6[2] = 0;
        }
        let capture = LiveGpuFrameCapture {
            hardware_startup_transient: None,
            ppu,
            cgram: vec![0u16; 256],
            raw_scanlines,
            source_entries: Vec::new(),
            bg3_source_tiles,
            bg3_vwf_glyph_runs: Vec::new(),
            dialogue_message_id: None,
            source_dialogue_ir: Vec::new(),
            dialogue_ir: Vec::new(),
            dialogue_layout: Vec::new(),
            dialogue_layout_origin_tile_number: None,
            mode7_source_chars: None,
            main_module: 0,
            player_indoors: 1,
            cgram_provenance: Default::default(),
        };
        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)
                .expect("modern asset resources should load");

        let render = render_modern_asset_capture_rgba(&capture, &resources)
            .expect("GPU readback should render BG3 font tile from PNG source");
        assert_eq!(render.via, "variant-gpu");
        validate_no_dynamic_bg3_text_chunks(&capture, &resources)
            .expect("dialogue font sidecar should remove BG3 dynamic text chunks");
        let stats = render
            .variant_stats
            .expect("variant renderer should report draw stats");
        assert_eq!(stats.missing_art_draws, 0);
        assert!(
            stats.stable_preview_draws > 0,
            "expected dialogue font sidecar key to resolve to a stable PNG preview draw, got {stats:?}"
        );
    }

    #[test]
    fn bg3_dynamic_text_chunk_validator_rejects_unkeyed_bg3_tiles() {
        let mut ppu = snes::ppu::PpuState::default();
        ppu.mode = 1;
        ppu.brightness = 15;
        ppu.screen_enabled = [0x04, 0x00];
        ppu.bg_layer[2].tilemap_adr = 0x0000;
        ppu.bg_layer[2].tile_adr = 0x1000;
        ppu.vram[0] = 999 | (7 << 10);
        let mut indices = [0u8; 64];
        indices[0] = 1;
        indices[1] = 2;
        indices[8] = 3;
        encode_2bpp_tile(&mut ppu.vram, 0x1000, 999, &indices);
        let capture = LiveGpuFrameCapture {
            hardware_startup_transient: None,
            ppu,
            cgram: vec![0u16; 256],
            raw_scanlines: Box::new([(0, 0, 0, 0, 0x04, [0; 4], [0; 4], [0; 8], false); 224]),
            source_entries: Vec::new(),
            bg3_source_tiles: Vec::new(),
            bg3_vwf_glyph_runs: Vec::new(),
            dialogue_message_id: None,
            source_dialogue_ir: Vec::new(),
            dialogue_ir: Vec::new(),
            dialogue_layout: Vec::new(),
            dialogue_layout_origin_tile_number: None,
            mode7_source_chars: None,
            main_module: 0,
            player_indoors: 1,
            cgram_provenance: Default::default(),
        };
        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)
                .expect("modern asset resources should load");

        let err = validate_no_dynamic_bg3_text_chunks(&capture, &resources)
            .expect_err("unkeyed BG3 tile should be rejected");

        assert!(err.contains("BG3 dynamic text chunks count=1"));
    }

    #[test]
    fn bg3_dynamic_text_chunk_validator_ignores_offscreen_wrap_tiles() {
        let mut ppu = snes::ppu::PpuState::default();
        ppu.mode = 1;
        ppu.brightness = 15;
        ppu.screen_enabled = [0x04, 0x00];
        ppu.bg_layer[2].tilemap_adr = 0x0000;
        ppu.bg_layer[2].tile_adr = 0x1000;
        ppu.bg_layer[2].tilemap_wider = true;
        ppu.bg_layer[2].h_scroll = 96;
        ppu.vram[0] = 999 | (7 << 10);
        let mut indices = [0u8; 64];
        indices[0] = 1;
        encode_2bpp_tile(&mut ppu.vram, 0x1000, 999, &indices);
        let capture = test_bg3_capture(ppu);
        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)
                .expect("modern asset resources should load");

        validate_no_dynamic_bg3_text_chunks(&capture, &resources)
            .expect("fully offscreen BG3 wrap tiles should not be glyph failures");
    }

    #[test]
    fn bg3_dynamic_text_chunk_validator_ignores_bulk_fill_tiles() {
        let mut ppu = snes::ppu::PpuState::default();
        ppu.mode = 1;
        ppu.brightness = 15;
        ppu.screen_enabled = [0x04, 0x00];
        ppu.bg_layer[2].tilemap_adr = 0x0000;
        ppu.bg_layer[2].tile_adr = 0x1000;
        for entry in ppu.vram.iter_mut().take(32 * 32) {
            *entry = 999 | (7 << 10);
        }
        let mut indices = [0u8; 64];
        indices[0] = 1;
        indices[1] = 2;
        indices[8] = 3;
        encode_2bpp_tile(&mut ppu.vram, 0x1000, 999, &indices);
        let capture = test_bg3_capture(ppu);
        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)
                .expect("modern asset resources should load");

        validate_no_dynamic_bg3_text_chunks(&capture, &resources)
            .expect("bulk repeated BG3 fill should not be treated as glyph text");
    }

    #[test]
    fn bg3_dynamic_text_chunk_validator_ignores_invisible_blank_frames() {
        let mut ppu = snes::ppu::PpuState::default();
        ppu.mode = 1;
        ppu.brightness = 0;
        ppu.screen_enabled = [0x04, 0x00];
        ppu.bg_layer[2].tilemap_adr = 0x0000;
        ppu.bg_layer[2].tile_adr = 0x1000;
        ppu.vram[0] = 999 | (7 << 10);
        let mut indices = [0u8; 64];
        indices[0] = 1;
        indices[1] = 2;
        indices[8] = 3;
        encode_2bpp_tile(&mut ppu.vram, 0x1000, 999, &indices);
        let capture = test_bg3_capture(ppu);
        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)
                .expect("modern asset resources should load");

        validate_no_dynamic_bg3_text_chunks(&capture, &resources)
            .expect("brightness-zero BG3 tiles are not visible glyph failures");
    }

    fn test_bg3_capture(ppu: snes::ppu::PpuState) -> LiveGpuFrameCapture {
        LiveGpuFrameCapture {
            hardware_startup_transient: None,
            ppu,
            cgram: vec![0u16; 256],
            raw_scanlines: Box::new([(0, 0, 0, 0, 0x04, [0; 4], [0; 4], [0; 8], false); 224]),
            source_entries: Vec::new(),
            bg3_source_tiles: Vec::new(),
            bg3_vwf_glyph_runs: Vec::new(),
            dialogue_message_id: None,
            source_dialogue_ir: Vec::new(),
            dialogue_ir: Vec::new(),
            dialogue_layout: Vec::new(),
            dialogue_layout_origin_tile_number: None,
            mode7_source_chars: None,
            main_module: 0,
            player_indoors: 1,
            cgram_provenance: Default::default(),
        }
    }

    #[test]
    fn closed_dialogue_does_not_claim_bg3_semantic_rendering() {
        let mut game = crate::load_embedded_play_state();
        game.set_current_dialogue_message_id(0);
        assert!(!game.is_dialogue_display_active());

        let capture = capture_gpu_frame_from_game(&mut game);

        assert_eq!(capture.dialogue_message_id(), None);
        assert!(capture.source_dialogue_ir().is_empty());
        assert!(capture.dialogue_ir().is_empty());
        assert!(capture.dialogue_layout().is_empty());
        assert!(capture.bg3_source_tiles().is_empty());
        assert!(capture.bg3_vwf_glyph_runs.is_empty());
    }

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
        let capture = capture_gpu_frame_from_game(&mut game);
        assert_no_dynamic_bg3_text_chunks(&capture);
    }

    

    

    #[test]
    fn live_game_dialogue_vwf_runs_render_from_png_glyph_atlas() {
        let (mut game, _) =
            crate::developer_room_commands::load_developer_destination("preset-dev-sandbox")
                .expect("developer sandbox should load");
        game.set_current_dialogue_message_id(0xc8);
        game.ram[MESSAGING_MODULE] = 0;
        game.ram[SUBMODULE_INDEX] = 2;
        game.ram[SAVED_MODULE_FOR_MENU] = game.ram[MAIN_MODULE_INDEX];
        game.ram[MAIN_MODULE_INDEX] = 14;

        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)
                .expect("modern asset resources should load");
        let variant_atlas =
            renderer::modern_variant_atlas::load_modern_canonical_art_atlas(&repo_root)
                .expect("canonical variant atlas should load");
        let mut saw_source_glyph_runs = None;
        for _ in 0..160 {
            game.zelda_run_frame(0);
            let frame_capture = capture_gpu_frame_from_game(&mut game);
            let gpu_frame = frame_capture.gpu_frame();
            let source_table = renderer::source_table_from_entries(frame_capture.source_entries());
            let modern_assets =
                renderer::modern_extract::extract_asset_resolved_modern_frame_from_sources(
                    &gpu_frame,
                    &source_table,
                    resources.source_atlas().expect("source atlas"),
                );
            let source_glyph_run_count = modern_assets.frame.vwf_glyph_runs_for_draw().len();
            if source_glyph_run_count == 0 {
                continue;
            }
            saw_source_glyph_runs = Some(modern_assets.frame.vwf_glyph_runs_for_draw().to_vec());
            let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(
                frame_capture.player_indoors(),
            );
            let stats = renderer::modern_variant_draw::compile_variant_draw_stats(
                &modern_assets.frame,
                &modern_assets.bg_cells,
                &modern_assets.sprite_cells,
                &variant_atlas,
                scene.bg_palette_name(),
                scene.sprite_palette_name(),
            );
            if stats.stable_preview_draws == source_glyph_run_count as u32 * 4 {
                assert_eq!(stats.unkeyed_bg3_fallback_draws, 0);
                assert_no_dynamic_bg3_text_chunks(&frame_capture);
                return;
            }
        }
        panic!(
            "source dialogue layout emitted VWF glyph runs but the default GPU path did not consume them from PNG: {:?}",
            saw_source_glyph_runs
        );
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
            hardware_startup_transient: None,
            ppu,
            cgram,
            raw_scanlines: Box::new([(0, 0, 0, 0, 0, [0; 4], [0; 4], [0; 8], false); 224]),
            source_entries: Vec::new(),
            bg3_source_tiles: Vec::new(),
            bg3_vwf_glyph_runs: Vec::new(),
            dialogue_message_id: None,
            source_dialogue_ir: Vec::new(),
            dialogue_ir: Vec::new(),
            dialogue_layout: Vec::new(),
            dialogue_layout_origin_tile_number: None,
            mode7_source_chars: None,
            main_module: 0,
            player_indoors: 0,
            cgram_provenance: Default::default(),
        };
        let gpu_render = ModernAssetGpuReadbackFrame {
            frame: GpuRgbaReadbackFrame::from_rgba(vec![0x7f; 256 * 224 * 4]),
            #[cfg(test)]
            variant_stats: None,
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

    fn encode_2bpp_tile(
        vram: &mut [u16],
        chr_base_words: usize,
        tile_number: u16,
        indices: &[u8; 64],
    ) {
        let tile_base = chr_base_words + usize::from(tile_number) * 8;
        for y in 0..8usize {
            let mut bp0 = 0u8;
            let mut bp1 = 0u8;
            for x in 0..8usize {
                let index = indices[y * 8 + x];
                let bit = 1u8 << (7 - x);
                if index & 1 != 0 {
                    bp0 |= bit;
                }
                if index & 2 != 0 {
                    bp1 |= bit;
                }
            }
            vram[tile_base + y] = u16::from(bp0) | (u16::from(bp1) << 8);
        }
    }

    fn assert_no_dynamic_bg3_text_chunks(capture: &LiveGpuFrameCapture) {
        let repo_root = repo_root();
        let resources =
            renderer::ModernIndexCompareResources::load_live_gpu_from_env(true, &repo_root)
                .expect("modern asset resources should load");
        validate_no_dynamic_bg3_text_chunks(capture, &resources)
            .expect("live frame should not retain BG3 dynamic text chunks");
        let gpu_frame = capture.gpu_frame();
        let source_table = renderer::source_table_from_entries(capture.source_entries());
        let modern_assets =
            renderer::modern_extract::extract_asset_resolved_modern_frame_from_sources(
                &gpu_frame,
                &source_table,
                resources.source_atlas().expect("source atlas"),
            );
        let overlap_count = modern_assets.frame.bg_layers[2]
            .index_tiles
            .iter()
            .filter(|inst| {
                let cell_source_key = modern_assets
                    .bg_cells
                    .get(inst.cell_id as usize)
                    .map(|cell| cell.source_key)
                    .unwrap_or(renderer::modern_hd_overrides::NO_SOURCE_KEY);
                (is_dynamic_or_unkeyed_bg3_text_key(inst.source_key)
                    || is_dynamic_or_unkeyed_bg3_text_key(cell_source_key))
                    && modern_assets
                        .frame
                        .vwf_glyph_runs_for_draw()
                        .iter()
                        .any(|run| {
                            rects_overlap(
                                inst.screen_x,
                                inst.screen_y,
                                8,
                                8,
                                run.screen_x,
                                run.screen_y,
                                16,
                                16,
                            )
                        })
            })
            .count();
        let dynamic_bg3_count = modern_assets.frame.bg_layers[2]
            .index_tiles
            .iter()
            .filter(|inst| {
                let cell_source_key = modern_assets
                    .bg_cells
                    .get(inst.cell_id as usize)
                    .map(|cell| cell.source_key)
                    .unwrap_or(renderer::modern_hd_overrides::NO_SOURCE_KEY);
                is_dynamic_or_unkeyed_bg3_text_key(inst.source_key)
                    || is_dynamic_or_unkeyed_bg3_text_key(cell_source_key)
            })
            .count();

        assert_eq!(
            overlap_count, 0,
            "semantic VWF source glyph runs should own covered BG3 dynamic text chunks"
        );
        assert_eq!(
            dynamic_bg3_count, 0,
            "live frame still has BG3 dynamic text chunks outside source glyph ownership"
        );
    }

    fn rects_overlap(
        ax: i16,
        ay: i16,
        aw: i16,
        ah: i16,
        bx: i16,
        by: i16,
        bw: i16,
        bh: i16,
    ) -> bool {
        let ax0 = i32::from(ax);
        let ay0 = i32::from(ay);
        let bx0 = i32::from(bx);
        let by0 = i32::from(by);
        ax0 < bx0 + i32::from(bw)
            && bx0 < ax0 + i32::from(aw)
            && ay0 < by0 + i32::from(bh)
            && by0 < ay0 + i32::from(ah)
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
