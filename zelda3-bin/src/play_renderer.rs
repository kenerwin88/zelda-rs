use std::env;
use std::process;

use platform::{Frontend, NativeFrontend, NativeFrontendOptions};
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

pub(crate) struct ConfiguredPlayRenderer {
    backend: Box<dyn PlayRendererBackend>,
    frontend: NativeFrontend,
    frame: Vec<u8>,
    render_flags: PpuRenderFlags,
}

impl ConfiguredPlayRenderer {
    pub(crate) fn name(&self) -> &'static str {
        self.backend.name()
    }

    pub(crate) fn frontend(&self) -> &NativeFrontend {
        &self.frontend
    }

    pub(crate) fn frontend_mut(&mut self) -> &mut NativeFrontend {
        &mut self.frontend
    }

    pub(crate) fn present_frame(&mut self, game: &mut ZeldaState) {
        self.backend
            .present_frame(game, &mut self.frontend, &mut self.frame, self.render_flags);
    }
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
        draw_play_ppu_frame(game, frame, 256 * 4, render_flags);
        let pixels =
            unsafe { std::slice::from_raw_parts(frame.as_ptr().cast::<u32>(), frame.len() / 4) };
        frontend.present_frame(pixels, 256, 224);
    }
}

fn from_env() -> Box<dyn PlayRendererBackend> {
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

pub(crate) fn configured_from_env(
    width: u32,
    height: u32,
    options: NativeFrontendOptions,
) -> Result<ConfiguredPlayRenderer, String> {
    let backend = from_env();
    let mut frontend = NativeFrontend::new_with_options(width, height, options)?;
    backend.configure_frontend(&mut frontend);
    Ok(ConfiguredPlayRenderer {
        backend,
        frontend,
        frame: vec![0u8; width as usize * height as usize * 4],
        render_flags: PpuRenderFlags::empty(),
    })
}

pub(crate) fn draw_play_ppu_frame(
    game: &mut ZeldaState,
    frame: &mut [u8],
    pitch: usize,
    render_flags: PpuRenderFlags,
) {
    game.zelda_draw_display_frame(frame, pitch, render_flags);
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
