use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

pub(crate) fn render_play_frame_bgra(
    game: &mut ZeldaState,
    frame: &mut [u8],
    pitch: usize,
    render_flags: PpuRenderFlags,
) {
    game.zelda_draw_display_frame(frame, pitch, render_flags);
}

pub(crate) fn render_standard_play_frame_bgra(game: &mut ZeldaState, frame: &mut [u8]) {
    render_play_frame_bgra(game, frame, 256 * 4, PpuRenderFlags::empty());
}

pub(crate) fn run_play_frame_bgra(
    game: &mut ZeldaState,
    input: u16,
    frame: &mut [u8],
    render_flags: PpuRenderFlags,
) {
    game.zelda_run_frame(input as i32);
    render_play_frame_bgra(game, frame, 256 * 4, render_flags);
}

pub(crate) fn run_play_frame_with_run_what_bgra(
    game: &mut ZeldaState,
    input: u16,
    run_what: u8,
    frame: &mut [u8],
    render_flags: PpuRenderFlags,
) {
    game.run_frame_internal(input, run_what);
    game.zelda_push_apu_state();
    render_play_frame_bgra(game, frame, 256 * 4, render_flags);
}
