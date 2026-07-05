use snes::ppu::PpuRenderFlags;
use zelda3::{LockstepOracle, ZeldaState};

pub(crate) fn render_diagnostic_lockstep_artifact_frame_bgra(
    game: &mut ZeldaState,
    frame: &mut [u8],
) {
    crate::classic_frame_renderer::render_standard_play_frame_bgra(game, frame);
}

pub(crate) fn render_diagnostic_overworld_screen_bgra(game: &mut ZeldaState, frame: &mut [u8]) {
    crate::classic_frame_renderer::render_standard_play_frame_bgra(game, frame);
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
