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
