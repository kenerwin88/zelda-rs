use std::env;
use std::process;

use platform::{Frontend, HostMenuInput, HostMenuState, NativeFrontend, NativeFrontendOptions};
use snes::ppu::PpuRenderFlags;
use zelda3::{LockstepOracle, ZeldaState};

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
        render_play_frame_bgra(game, frame, 256 * 4, render_flags);
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

pub(crate) fn render_replay_projection_bgra(game: &mut ZeldaState, frame: &mut [u8]) {
    render_standard_play_frame_bgra(game, frame);
}

pub(crate) fn render_lockstep_artifact_frame_bgra(game: &mut ZeldaState, frame: &mut [u8]) {
    render_standard_play_frame_bgra(game, frame);
}

pub(crate) fn render_overworld_screen_dump_bgra(game: &mut ZeldaState, frame: &mut [u8]) {
    render_standard_play_frame_bgra(game, frame);
}

pub(crate) struct OracleRenderFramePair {
    pub(crate) game_state: ZeldaState,
    pub(crate) snes_state: ZeldaState,
}

pub(crate) fn render_oracle_compare_frames_bgra(
    oracle: &LockstepOracle,
    game_frame: &mut [u8],
    snes_frame: &mut [u8],
    pitch: usize,
) -> OracleRenderFramePair {
    let mut game_state = oracle.game.clone();
    let mut snes_state = oracle_compare_render_state(oracle);

    render_play_frame_bgra(&mut game_state, game_frame, pitch, PpuRenderFlags::empty());
    render_play_frame_bgra(&mut snes_state, snes_frame, pitch, PpuRenderFlags::empty());

    OracleRenderFramePair {
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

pub(crate) fn render_lockstep_oracle_frames_in_place(
    oracle: &mut LockstepOracle,
    game_frame: &mut [u8],
    snes_frame: &mut [u8],
    pitch: usize,
) {
    render_play_frame_bgra(&mut oracle.game, game_frame, pitch, PpuRenderFlags::empty());

    let mut snes_state = oracle.game.clone();
    snes_state.ppu = oracle.snes.ppu.clone();
    snes_state.dma = oracle.snes.dma.clone();
    snes_state.ram.copy_from_slice(&oracle.snes.ram);
    snes_state.sram.copy_from_slice(&oracle.snes.cart.ram);
    render_play_frame_bgra(&mut snes_state, snes_frame, pitch, PpuRenderFlags::empty());
    oracle.snes.ppu = snes_state.ppu;
    oracle.snes.dma = snes_state.dma;
    oracle.snes.ram.copy_from_slice(&snes_state.ram);
    oracle.snes.cart.ram.copy_from_slice(&snes_state.sram);
}

pub(crate) fn render_fingerprint_leaf_bgra(frame: &[u8]) -> u32 {
    renderer::render_fingerprint_leaf_bgra(frame)
}

pub(crate) fn render_replay_fingerprint_leaf_bgra(game: &mut ZeldaState, frame: &mut [u8]) -> u32 {
    render_replay_projection_bgra(game, frame);
    render_fingerprint_leaf_bgra(frame)
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
