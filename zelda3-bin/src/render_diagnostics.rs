use snes::ppu::PpuRenderFlags;
use zelda3::{LockstepOracle, ZeldaState};

pub(crate) fn run_diagnostic_play_frame_bgra(
    game: &mut ZeldaState,
    input: u16,
    frame: &mut [u8],
    render_flags: PpuRenderFlags,
) {
    crate::classic_frame_renderer::run_play_frame_bgra(game, input, frame, render_flags);
}

pub(crate) fn render_diagnostic_lockstep_artifact_frame_bgra(
    game: &mut ZeldaState,
    frame: &mut [u8],
) {
    crate::classic_frame_renderer::render_standard_play_frame_bgra(game, frame);
}

pub(crate) fn replay_projection_bgra(game: &mut ZeldaState, frame: &mut [u8]) {
    crate::classic_frame_renderer::render_standard_play_frame_bgra(game, frame);
}

pub(crate) fn replay_fingerprint_leaf_bgra(game: &mut ZeldaState, frame: &mut [u8]) -> u32 {
    replay_projection_bgra(game, frame);
    renderer::render_fingerprint_leaf_bgra(frame)
}

pub(crate) struct DiagnosticOracleRenderFramePair {
    pub(crate) game_state: ZeldaState,
    pub(crate) snes_state: ZeldaState,
}

pub(crate) fn render_diagnostic_oracle_compare_frames_bgra(
    oracle: &LockstepOracle,
    game_frame: &mut [u8],
    snes_frame: &mut [u8],
    pitch: usize,
) -> DiagnosticOracleRenderFramePair {
    let mut game_state = oracle.game.clone();
    let mut snes_state = oracle_compare_render_state(oracle);

    crate::classic_frame_renderer::render_play_frame_bgra(
        &mut game_state,
        game_frame,
        pitch,
        PpuRenderFlags::empty(),
    );
    crate::classic_frame_renderer::render_play_frame_bgra(
        &mut snes_state,
        snes_frame,
        pitch,
        PpuRenderFlags::empty(),
    );

    DiagnosticOracleRenderFramePair {
        game_state,
        snes_state,
    }
}

pub(crate) struct RenderDiff {
    pub(crate) mismatched_pixels: usize,
    pub(crate) first_pixel: usize,
    pub(crate) mine_pixel: [u8; 4],
    pub(crate) theirs_pixel: [u8; 4],
    pub(crate) mine_ppu: String,
    pub(crate) theirs_ppu: String,
}

pub(crate) fn compare_diagnostic_oracle_render_frame(
    oracle: &LockstepOracle,
    game_frame: &mut [u8],
    snes_frame: &mut [u8],
    pitch: usize,
    width: usize,
) -> Option<RenderDiff> {
    let rendered =
        render_diagnostic_oracle_compare_frames_bgra(oracle, game_frame, snes_frame, pitch);

    let mut mismatched_pixels = 0usize;
    let mut first_pixel = usize::MAX;
    let mut mine_pixel = [0; 4];
    let mut theirs_pixel = [0; 4];
    for (idx, (game_pixel, snes_pixel)) in game_frame
        .chunks_exact(4)
        .zip(snes_frame.chunks_exact(4))
        .take(width * 224)
        .enumerate()
    {
        if game_pixel != snes_pixel {
            mismatched_pixels += 1;
            if first_pixel == usize::MAX {
                first_pixel = idx;
                mine_pixel.copy_from_slice(game_pixel);
                theirs_pixel.copy_from_slice(snes_pixel);
            }
        }
    }

    (mismatched_pixels != 0).then_some(RenderDiff {
        mismatched_pixels,
        first_pixel,
        mine_pixel,
        theirs_pixel,
        mine_ppu: format_render_ppu_summary(&rendered.game_state),
        theirs_ppu: format_render_ppu_summary(&rendered.snes_state),
    })
}

pub(crate) fn format_render_ppu_summary(state: &ZeldaState) -> String {
    let ppu = &state.ppu;
    format!(
        "mode={} forced_blank={} brightness={} screen={:02x}/{:02x} window={:02x}/{:02x} math={:02x} cg={:02x}/{:02x} fixed=({:02x},{:02x},{:02x}) m7={:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x} bg1=({:04x},{:04x},tm={:04x},chr={:04x}) bg2=({:04x},{:04x},tm={:04x},chr={:04x}) hdma={:02x} dma6={:02x}:{:04x}->{:02x} dma7={:02x}:{:04x}->{:02x} cgram0={:04x} cgram1={:04x} vram0000={:04x} vram1000={:04x}",
        ppu.mode,
        ppu.forced_blank,
        ppu.brightness,
        ppu.screen_enabled[0],
        ppu.screen_enabled[1],
        ppu.screen_windowed[0],
        ppu.screen_windowed[1],
        ppu.math_enabled,
        ppu.clip_mode,
        ppu.prevent_math_mode,
        ppu.fixed_color_r,
        ppu.fixed_color_g,
        ppu.fixed_color_b,
        ppu.m7_matrix[0] as u16,
        ppu.m7_matrix[1] as u16,
        ppu.m7_matrix[2] as u16,
        ppu.m7_matrix[3] as u16,
        ppu.m7_matrix[4] as u16,
        ppu.m7_matrix[5] as u16,
        ppu.m7_matrix[6] as u16,
        ppu.m7_matrix[7] as u16,
        ppu.bg_layer[0].h_scroll,
        ppu.bg_layer[0].v_scroll,
        ppu.bg_layer[0].tilemap_adr,
        ppu.bg_layer[0].tile_adr,
        ppu.bg_layer[1].h_scroll,
        ppu.bg_layer[1].v_scroll,
        ppu.bg_layer[1].tilemap_adr,
        ppu.bg_layer[1].tile_adr,
        state.ram[0x9b],
        state.dma.channel[6].a_bank,
        state.dma.channel[6].a_adr,
        state.dma.channel[6].b_adr,
        state.dma.channel[7].a_bank,
        state.dma.channel[7].a_adr,
        state.dma.channel[7].b_adr,
        ppu.cgram[0],
        ppu.cgram[1],
        ppu.vram[0],
        ppu.vram[0x1000],
    )
}

fn oracle_compare_render_state(oracle: &LockstepOracle) -> ZeldaState {
    let mut snes_state = oracle.game.clone();
    snes_state.ppu = oracle.snes.ppu.clone();
    snes_state.dma = oracle.snes.dma.clone();
    snes_state.ram.copy_from_slice(&oracle.snes.ram);
    snes_state
        .ram
        .copy_within(0x1b00..0x1b00 + 224 * 2, 0x1dba0);
    snes_state.sram.copy_from_slice(&oracle.snes.cart.ram);
    snes_state
}

pub(crate) fn render_diagnostic_lockstep_oracle_frames_in_place(
    oracle: &mut LockstepOracle,
    game_frame: &mut [u8],
    snes_frame: &mut [u8],
    pitch: usize,
) {
    crate::classic_frame_renderer::render_play_frame_bgra(
        &mut oracle.game,
        game_frame,
        pitch,
        PpuRenderFlags::empty(),
    );

    let mut snes_state = oracle.game.clone();
    snes_state.ppu = oracle.snes.ppu.clone();
    snes_state.dma = oracle.snes.dma.clone();
    snes_state.ram.copy_from_slice(&oracle.snes.ram);
    snes_state.sram.copy_from_slice(&oracle.snes.cart.ram);
    crate::classic_frame_renderer::render_play_frame_bgra(
        &mut snes_state,
        snes_frame,
        pitch,
        PpuRenderFlags::empty(),
    );
    oracle.snes.ppu = snes_state.ppu;
    oracle.snes.dma = snes_state.dma;
    oracle.snes.ram.copy_from_slice(&snes_state.ram);
    oracle.snes.cart.ram.copy_from_slice(&snes_state.sram);
}
