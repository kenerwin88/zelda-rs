use platform::{Frontend, NativeFrontend};
use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

struct CpuPlayRenderer;

impl crate::play_renderer::PlayRendererBackend for CpuPlayRenderer {
    fn name(&self) -> &'static str {
        "cpu_render"
    }

    fn configure_frontend(&self, frontend: &mut NativeFrontend) {
        frontend.set_renderer_mode(renderer::RendererMode::Classic);
    }

    fn present_frame(
        &mut self,
        game: &mut ZeldaState,
        frontend: &mut NativeFrontend,
        frame: &mut [u8],
        render_flags: PpuRenderFlags,
    ) {
        render_play_frame_bgra(game, frame, 256 * 4, render_flags);
        let pixels =
            unsafe { std::slice::from_raw_parts(frame.as_ptr().cast::<u32>(), frame.len() / 4) };
        frontend.present_frame(pixels, 256, 224);
    }
}

pub(crate) fn new_cpu_play_renderer() -> Box<dyn crate::play_renderer::PlayRendererBackend> {
    Box::new(CpuPlayRenderer)
}

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
