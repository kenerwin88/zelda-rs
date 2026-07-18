use std::env;
use std::process;

use platform::{
    Frontend, HostMenuInput, HostMenuState, NativeFrontend, NativeFrontendOptions, RecorderControl,
};
use renderer::RendererMode;
use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

const PLAY_WIDTH: u32 = 256;
const PLAY_HEIGHT: u32 = 224;

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
    raw_pixels: Vec<u32>,
    render_flags: PpuRenderFlags,
}

impl ConfiguredPlayRenderer {
    pub(crate) fn name(&self) -> &'static str {
        self.backend.name()
    }

    pub(crate) fn quit_requested(&self) -> bool {
        self.frontend.quit_requested()
    }

    pub(crate) fn set_window_title(&self, title: &str) {
        self.frontend.set_window_title(title);
    }

    pub(crate) fn request_window_attention(&self) {
        self.frontend.request_window_attention();
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

    pub(crate) fn drain_recorder_controls(&mut self) -> Vec<RecorderControl> {
        self.frontend.drain_recorder_controls()
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

    pub(crate) fn audio_underflow_samples(&self) -> u64 {
        self.frontend.audio_underflow_samples()
    }

    pub(crate) fn audio_dropped_samples(&self) -> u64 {
        self.frontend.audio_dropped_samples()
    }

    pub(crate) fn present_frame(&mut self, game: &mut ZeldaState) {
        self.backend
            .present_frame(game, &mut self.frontend, &mut self.frame, self.render_flags);
    }

    pub(crate) fn present_rgba_frame(&mut self, rgba: &[u8], width: u32, height: u32) {
        let pixel_count = width as usize * height as usize;
        self.raw_pixels.resize(pixel_count, 0);
        for (dst, src) in self.raw_pixels.iter_mut().zip(rgba.chunks_exact(4)) {
            *dst = u32::from_le_bytes([src[2], src[1], src[0], src[3]]);
        }
        self.frontend.present_frame(&self.raw_pixels, width, height);
    }

    pub(crate) fn wait_idle(&self) {
        self.frontend.wait_idle();
    }

    pub(crate) fn push_audio(&mut self, audio: &[i16]) {
        self.frontend.push_audio(audio);
    }
}

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

struct CpuPlayRenderer {
    pixels: Vec<u32>,
}

impl CpuPlayRenderer {
    fn new() -> Self {
        Self {
            pixels: vec![0u32; (PLAY_WIDTH * PLAY_HEIGHT) as usize],
        }
    }
}

impl PlayRendererBackend for CpuPlayRenderer {
    fn name(&self) -> &'static str {
        "cpu_render"
    }

    fn configure_frontend(&self, frontend: &mut NativeFrontend) {
        frontend.set_renderer_mode(RendererMode::Classic);
    }

    fn present_frame(
        &mut self,
        game: &mut ZeldaState,
        frontend: &mut NativeFrontend,
        frame: &mut [u8],
        render_flags: PpuRenderFlags,
    ) {
        crate::classic_frame_renderer::render_play_frame_bgra(
            game,
            frame,
            PLAY_WIDTH as usize * 4,
            render_flags,
        );
        for (dst, src) in self.pixels.iter_mut().zip(frame.chunks_exact(4)) {
            *dst = u32::from_le_bytes([src[0], src[1], src[2], src[3]]);
        }
        frontend.present_frame(&self.pixels, PLAY_WIDTH, PLAY_HEIGHT);
    }
}

fn from_env() -> Box<dyn PlayRendererBackend> {
    let value = env::var("ZELDA3_RENDER_BACKEND").ok();
    match PlayRendererBackendChoice::from_env_value(value.as_deref()) {
        Ok(PlayRendererBackendChoice::Cpu) => Box::new(CpuPlayRenderer::new()),
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
        raw_pixels: vec![0u32; width as usize * height as usize],
        render_flags: PpuRenderFlags::empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        PlayRendererBackendChoice, PlayRendererBackendChoice::Cpu, PlayRendererBackendChoice::Gpu,
    };

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
    fn explicit_backend_accepts_cpu_case_insensitively() {
        assert_eq!(
            PlayRendererBackendChoice::from_env_value(Some("CPU")),
            Ok(Cpu)
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
