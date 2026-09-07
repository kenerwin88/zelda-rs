//! Frame fixtures shared by the renderer unit tests.

use crate::{GpuFrame, ScanlineRegs};

/// A minimal mode-1 `GpuFrame` over the given memories.
pub(crate) fn test_gpu_frame<'a>(
    vram: &'a [u16],
    cgram: &'a [u16],
    oam: &'a [u16],
    brightness: u8,
    forced_blank: bool,
) -> GpuFrame<'a> {
    GpuFrame {
        hardware_startup_transient: None,
        vram,
        obj_vram: None,
        bg_vram: None,
        cgram,
        oam,
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
        brightness,
        scanout_brightness_override: None,
        scanout_top_crop: 0,
        forced_blank,
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
        // A physical forced-blank frame has the same INIDISP state in
        // every captured scanline receipt; the frame-level latch alone is
        // not a substitute for those authoritative rows.
        scanlines: Box::new(
            [ScanlineRegs {
                forced_blank,
                ..ScanlineRegs::default()
            }; 224],
        ),
        bg3_source_tiles: &[],
        bg3_vwf_glyph_runs: &[],
        dialogue_message_id: None,
        source_dialogue_ir: &[],
        dialogue_ir: &[],
        dialogue_layout: &[],
        dialogue_layout_origin_tile_number: None,
        cgram_provenance: None,
    }
}
