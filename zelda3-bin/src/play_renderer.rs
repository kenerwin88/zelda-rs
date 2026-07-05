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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayRendererBackendChoice {
    Cpu,
    Gpu,
}

impl PlayRendererBackendChoice {
    fn from_env_value(value: Option<&str>) -> Result<Self, String> {
        match value {
            Some(value) if value.eq_ignore_ascii_case("cpu") => Ok(Self::Cpu),
            Some(value) if value.eq_ignore_ascii_case("gpu") => Ok(Self::Gpu),
            Some(value) => Err(format!(
                "unknown ZELDA3_RENDER_BACKEND={value:?}; expected cpu or gpu"
            )),
            None => Ok(Self::Gpu),
        }
    }
}

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
    let value = env::var("ZELDA3_RENDER_BACKEND").ok();
    match PlayRendererBackendChoice::from_env_value(value.as_deref()) {
        Ok(PlayRendererBackendChoice::Cpu) => Box::new(CpuPlayRenderer),
        Ok(PlayRendererBackendChoice::Gpu) => crate::gpu_capture::new_gpu_play_renderer(),
        Err(message) => {
            eprintln!("{message}");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PlayRendererBackendChoice, PlayRendererBackendChoice::*};

    #[test]
    fn unset_backend_defaults_to_gpu() {
        assert_eq!(PlayRendererBackendChoice::from_env_value(None), Ok(Gpu));
    }

    #[test]
    fn explicit_backend_accepts_cpu_or_gpu_case_insensitively() {
        assert_eq!(
            PlayRendererBackendChoice::from_env_value(Some("cpu")),
            Ok(Cpu)
        );
        assert_eq!(
            PlayRendererBackendChoice::from_env_value(Some("GPU")),
            Ok(Gpu)
        );
    }

    #[test]
    fn invalid_backend_reports_expected_values() {
        let error = PlayRendererBackendChoice::from_env_value(Some("software")).unwrap_err();

        assert_eq!(
            error,
            "unknown ZELDA3_RENDER_BACKEND=\"software\"; expected cpu or gpu"
        );
    }
}
