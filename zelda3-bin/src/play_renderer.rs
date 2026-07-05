use std::env;
use std::process;

use platform::{Frontend, NativeFrontend};
use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

pub(crate) trait PlayRendererBackend {
    fn name(&self) -> &'static str;

    fn configure_frontend(&self, frontend: &mut NativeFrontend);

    fn present_frame(
        &mut self,
        game: &mut ZeldaState,
        frontend: &mut NativeFrontend,
        frame: &mut [u8],
        render_flags: PpuRenderFlags,
    );
}

struct CpuPlayRenderer;

impl PlayRendererBackend for CpuPlayRenderer {
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
        crate::draw_play_ppu_frame(game, frame, 256 * 4, render_flags);
        let pixels =
            unsafe { std::slice::from_raw_parts(frame.as_ptr().cast::<u32>(), frame.len() / 4) };
        frontend.present_frame(pixels, 256, 224);
    }
}

pub(crate) fn from_env() -> Box<dyn PlayRendererBackend> {
    match env::var("ZELDA3_RENDER_BACKEND") {
        Ok(value) if value.eq_ignore_ascii_case("cpu") => Box::new(CpuPlayRenderer),
        Ok(value) if value.eq_ignore_ascii_case("gpu") => {
            crate::gpu_capture::new_gpu_play_renderer()
        }
        Ok(value) => {
            eprintln!("unknown ZELDA3_RENDER_BACKEND={value:?}; expected cpu or gpu");
            process::exit(2);
        }
        Err(_) => crate::gpu_capture::new_gpu_play_renderer(),
    }
}
