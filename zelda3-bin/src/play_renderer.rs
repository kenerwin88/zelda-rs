use std::env;
use std::process;

use platform::{Frontend, HostMenuInput, HostMenuState, NativeFrontend, NativeFrontendOptions};
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

    pub(crate) fn quit_requested(&self) -> bool {
        self.frontend.quit_requested()
    }

    pub(crate) fn poll_input(&mut self) -> u16 {
        self.frontend.poll_input()
    }

    pub(crate) fn poll_input_with_menu(&mut self, menu_open: bool) -> u16 {
        self.frontend.poll_input_with_menu(menu_open)
    }

    pub(crate) fn drain_host_menu_inputs(&mut self) -> Vec<HostMenuInput> {
        self.frontend.drain_host_menu_inputs()
    }

    pub(crate) fn apply_runtime_settings(&mut self, settings: platform::RuntimeSettings) {
        self.frontend.apply_runtime_settings(settings);
    }

    pub(crate) fn present_menu_overlay(&mut self, menu: &HostMenuState) {
        self.frontend.present_menu_overlay(menu);
    }

    pub(crate) fn audio_samples_per_frame(&self) -> usize {
        self.frontend.audio_samples_per_frame()
    }

    pub(crate) fn audio_channels(&self) -> usize {
        self.frontend.audio_channels()
    }

    pub(crate) fn present_frame(&mut self, game: &mut ZeldaState) {
        self.backend
            .present_frame(game, &mut self.frontend, &mut self.frame, self.render_flags);
    }

    pub(crate) fn push_audio(&mut self, audio: &[i16]) {
        self.frontend.push_audio(audio);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayRendererBackendChoice {
    Gpu,
}

impl PlayRendererBackendChoice {
    fn from_env_value(value: Option<&str>) -> Result<Self, String> {
        match value {
            Some(value) if value.eq_ignore_ascii_case("cpu") => Err(
                "ZELDA3_RENDER_BACKEND=cpu is diagnostic-only; live play requires gpu".to_string(),
            ),
            Some(value) if value.eq_ignore_ascii_case("gpu") => Ok(Self::Gpu),
            Some(value) => Err(format!(
                "unknown ZELDA3_RENDER_BACKEND={value:?}; expected gpu"
            )),
            None => Ok(Self::Gpu),
        }
    }
}

fn from_env() -> Box<dyn PlayRendererBackend> {
    let value = env::var("ZELDA3_RENDER_BACKEND").ok();
    match PlayRendererBackendChoice::from_env_value(value.as_deref()) {
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

#[cfg(test)]
mod tests {
    use super::{PlayRendererBackendChoice, PlayRendererBackendChoice::Gpu};

    #[test]
    fn unset_backend_defaults_to_gpu() {
        assert_eq!(PlayRendererBackendChoice::from_env_value(None), Ok(Gpu));
    }

    #[test]
    fn explicit_backend_accepts_gpu_case_insensitively() {
        assert_eq!(
            PlayRendererBackendChoice::from_env_value(Some("GPU")),
            Ok(Gpu)
        );
    }

    #[test]
    fn live_backend_rejects_cpu_renderer() {
        let error = PlayRendererBackendChoice::from_env_value(Some("cpu")).unwrap_err();

        assert_eq!(
            error,
            "ZELDA3_RENDER_BACKEND=cpu is diagnostic-only; live play requires gpu"
        );
    }

    #[test]
    fn invalid_backend_reports_expected_values() {
        let error = PlayRendererBackendChoice::from_env_value(Some("software")).unwrap_err();

        assert_eq!(
            error,
            "unknown ZELDA3_RENDER_BACKEND=\"software\"; expected gpu"
        );
    }
}
