//! Platform abstraction: window, audio, input.
//!
//! `NativeFrontend` owns the winit event loop, wgpu renderer (via `crates/renderer`),
//! and cpal audio output. It exposes the same `Frontend` trait that `zelda3-bin`
//! uses, so the game loop structure in the binary is unchanged.
//!
//! Replaces the old winit 0.28 + `pixels` implementation. Key differences:
//! - winit 0.30 `ApplicationHandler` + `pump_app_events` instead of `run_return`
//! - `FrameRenderer` (wgpu 29) instead of `pixels` crate
//! - `Arc<Window>` held by the handler; no `Box::leak`

#![allow(dead_code)]

use std::collections::VecDeque;
use std::env;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use renderer::{FrameRenderer, GpuFrame, PresentationContext, RenderError};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

// ── Frontend trait ────────────────────────────────────────────────────────────

pub trait Frontend {
    fn poll_input(&mut self) -> u16;
    fn present_frame(&mut self, pixels: &[u32], width: u32, height: u32);
    fn push_audio(&mut self, samples: &[i16]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeFrontendOptions {
    pub scale: u32,
    pub enable_audio: bool,
    pub fullscreen: bool,
}

impl NativeFrontendOptions {
    pub fn from_env(scale: u32, enable_audio: bool) -> Self {
        Self::from_values(
            scale,
            enable_audio,
            env::var_os("ZELDA3_STEAMDECK"),
            env::var_os("ZELDA3_FULLSCREEN"),
        )
    }

    fn from_values(
        scale: u32,
        enable_audio: bool,
        steamdeck: Option<OsString>,
        fullscreen: Option<OsString>,
    ) -> Self {
        let deck_default = env_flag_option(steamdeck).unwrap_or(false);
        let fullscreen = env_flag_option(fullscreen).unwrap_or(deck_default);
        Self {
            scale,
            enable_audio,
            fullscreen,
        }
    }
}

// ── NativeFrontend ────────────────────────────────────────────────────────────

pub struct NativeFrontend {
    event_loop: EventLoop<()>,
    handler: NativeHandler,
    gamepad: Option<GamepadInput>,
    audio: Option<AudioOutput>,
    audio_samples_per_frame: usize,
    audio_channels: usize,
    audio_queue_target_bytes: u32,
    audio_queue_limit_bytes: u32,
    next_frame_tick: Instant,
    presented_frames: u32,
}

impl NativeFrontend {
    pub fn new(width: u32, height: u32, scale: u32, enable_audio: bool) -> Result<Self, String> {
        Self::new_with_options(
            width,
            height,
            NativeFrontendOptions::from_values(scale, enable_audio, None, None),
        )
    }

    pub fn new_with_options(
        width: u32,
        height: u32,
        options: NativeFrontendOptions,
    ) -> Result<Self, String> {
        let event_loop = EventLoop::new().map_err(|e| e.to_string())?;

        let mut window_attrs = WindowAttributes::default()
            .with_title("The Legend of Zelda: A Link to the Past")
            .with_inner_size(LogicalSize::new(
                width.saturating_mul(options.scale.max(1)),
                height.saturating_mul(options.scale.max(1)),
            ))
            .with_min_inner_size(LogicalSize::new(width, height))
            .with_resizable(true);
        if options.fullscreen {
            window_attrs = window_attrs.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let handler = NativeHandler {
            window_attrs,
            game_width: width,
            game_height: height,
            window: None,
            renderer: None,
            input_state: 0,
            quit: false,
        };

        let mut frontend = Self {
            event_loop,
            handler,
            gamepad: GamepadInput::new(),
            audio: None,
            audio_samples_per_frame: 735,
            audio_channels: 2,
            audio_queue_target_bytes: (735 * 2 * std::mem::size_of::<i16>() * 4) as u32,
            audio_queue_limit_bytes: (735 * 2 * std::mem::size_of::<i16>() * 10) as u32,
            next_frame_tick: Instant::now(),
            presented_frames: 0,
        };

        // Pump the event loop until Resumed fires and creates the window + renderer.
        // Resumed is guaranteed to fire during the first active pump on all platforms.
        let deadline = Instant::now() + Duration::from_secs(10);
        while !frontend.handler.window.is_some() && !frontend.handler.quit {
            if Instant::now() > deadline {
                return Err("timed out waiting for window creation".to_string());
            }
            match frontend
                .event_loop
                .pump_app_events(Some(Duration::from_millis(5)), &mut frontend.handler)
            {
                PumpStatus::Exit(_) => break,
                PumpStatus::Continue => {}
            }
        }

        if frontend.handler.window.is_none() {
            return Err("window creation failed".to_string());
        }

        if options.enable_audio {
            if let Ok(audio) = AudioOutput::new() {
                let samples = ((534 * audio.sample_rate as usize) / 32_000).max(1);
                let channels = audio.channels.max(1) as usize;
                let block_bytes = (samples * channels * std::mem::size_of::<i16>()) as u32;
                frontend.audio_samples_per_frame = samples;
                frontend.audio_channels = channels;
                frontend.audio_queue_target_bytes = block_bytes.saturating_mul(4);
                frontend.audio_queue_limit_bytes = block_bytes.saturating_mul(10);
                frontend.audio = Some(audio);
            }
        }

        Ok(frontend)
    }

    pub fn quit_requested(&self) -> bool {
        self.handler.quit
    }

    pub fn audio_samples_per_frame(&self) -> usize {
        self.audio_samples_per_frame
    }

    pub fn audio_channels(&self) -> usize {
        self.audio_channels
    }

    pub fn audio_queued_bytes(&self) -> u32 {
        self.audio
            .as_ref()
            .map(AudioOutput::queued_bytes)
            .unwrap_or(0)
    }

    pub fn audio_target_bytes(&self) -> u32 {
        self.audio_queue_target_bytes
    }

    pub fn present_gpu_frame(&mut self, frame: &GpuFrame<'_>) {
        self.present_gpu_frame_with_context(frame, PresentationContext::default());
    }

    pub fn present_gpu_frame_with_context(
        &mut self,
        frame: &GpuFrame<'_>,
        context: PresentationContext,
    ) {
        if let Some(renderer) = &mut self.handler.renderer {
            match renderer.render_gpu_frame_with_context(frame, context) {
                Ok(()) => {}
                Err(RenderError::SurfaceReconfigureNeeded) => {
                    if let Some(window) = &self.handler.window {
                        renderer.resize(window.inner_size());
                    }
                }
                Err(RenderError::SurfaceSkipped) => {}
                Err(RenderError::Fatal(e)) => eprintln!("render error: {e}"),
            }
        }
        self.sleep_after_present();
    }

    fn sleep_after_present(&mut self) {
        // Frame pacing — function name and increment text must match exactly;
        // scripts/test_standard_replay_parity.py greps for both literals.
        self.next_frame_tick += frame_delay(self.presented_frames);
        self.presented_frames = self.presented_frames.wrapping_add(1);
        let now = Instant::now();
        if self.next_frame_tick > now {
            std::thread::sleep(self.next_frame_tick - now);
        } else if now.duration_since(self.next_frame_tick) > Duration::from_millis(500) {
            self.next_frame_tick = now;
        }
    }
}

impl Frontend for NativeFrontend {
    fn poll_input(&mut self) -> u16 {
        // pump_app_events replaces winit 0.28's run_return: non-blocking event drain.
        match self
            .event_loop
            .pump_app_events(Some(Duration::ZERO), &mut self.handler)
        {
            PumpStatus::Exit(_) => self.handler.quit = true,
            PumpStatus::Continue => {}
        }
        let gamepad_state = self.gamepad.as_mut().map(GamepadInput::poll).unwrap_or(0);
        self.handler.input_state | gamepad_state
    }

    fn present_frame(&mut self, pixels: &[u32], _width: u32, _height: u32) {
        if let Some(renderer) = &mut self.handler.renderer {
            renderer.upload_frame(pixels);
            match renderer.render() {
                Ok(()) => {}
                Err(RenderError::SurfaceReconfigureNeeded) => {
                    if let Some(window) = &self.handler.window {
                        renderer.resize(window.inner_size());
                    }
                }
                Err(RenderError::SurfaceSkipped) => {}
                Err(RenderError::Fatal(e)) => eprintln!("render error: {e}"),
            }
        }
        self.sleep_after_present();
    }

    fn push_audio(&mut self, samples: &[i16]) {
        if let Some(audio) = &mut self.audio {
            while audio.queued_bytes() >= self.audio_queue_limit_bytes {
                std::thread::sleep(Duration::from_millis(1));
            }
            audio.push(samples, self.audio_queue_target_bytes);
        }
    }
}

fn frame_delay(frame: u32) -> Duration {
    const DELAYS_MS: [u64; 3] = [17, 17, 16];
    Duration::from_millis(DELAYS_MS[(frame % 3) as usize])
}

// ── NativeHandler (ApplicationHandler) ───────────────────────────────────────

struct NativeHandler {
    window_attrs: WindowAttributes,
    game_width: u32,
    game_height: u32,
    // Populated in resumed():
    window: Option<Arc<Window>>,
    renderer: Option<FrameRenderer>,
    // Updated by input events:
    input_state: u16,
    quit: bool,
}

impl ApplicationHandler for NativeHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = match event_loop.create_window(self.window_attrs.clone()) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("failed to create window: {e}");
                self.quit = true;
                return;
            }
        };
        let renderer = pollster::block_on(FrameRenderer::new(
            Arc::clone(&window),
            self.game_width,
            self.game_height,
        ));
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.quit = true;
                event_loop.exit();
            }
            WindowEvent::Focused(focused) => {
                handle_focus_input_state(&mut self.input_state, focused)
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    renderer.resize(window.inner_size());
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    if key == KeyCode::Escape && event.state == ElementState::Pressed {
                        self.quit = true;
                        event_loop.exit();
                    }
                    if let Some(action) = presentation_hotkey_action(key, event.state) {
                        if let Some(renderer) = &mut self.renderer {
                            apply_presentation_hotkey(renderer, action);
                        }
                    }
                    handle_key_input_state(&mut self.input_state, key, event.state);
                }
            }
            _ => {}
        }
    }
}

fn handle_focus_input_state(input_state: &mut u16, focused: bool) {
    if !focused {
        *input_state = 0;
    }
}

fn handle_key_input_state(input_state: &mut u16, key: KeyCode, state: ElementState) {
    if let Some(bit) = key_to_input_bit(key) {
        match state {
            ElementState::Pressed => *input_state |= bit,
            ElementState::Released => *input_state &= !bit,
        }
    }
}

fn key_to_input_bit(key: KeyCode) -> Option<u16> {
    match key {
        KeyCode::KeyZ => Some(1 << 0),       // B
        KeyCode::KeyA => Some(1 << 1),       // Y
        KeyCode::ShiftRight => Some(1 << 2), // Select
        KeyCode::Enter => Some(1 << 3),      // Start
        KeyCode::ArrowUp => Some(1 << 4),
        KeyCode::ArrowDown => Some(1 << 5),
        KeyCode::ArrowLeft => Some(1 << 6),
        KeyCode::ArrowRight => Some(1 << 7),
        KeyCode::KeyX => Some(1 << 8),                  // A
        KeyCode::KeyS => Some(1 << 9),                  // X
        KeyCode::KeyC | KeyCode::KeyQ => Some(1 << 10), // L
        KeyCode::KeyV | KeyCode::KeyW => Some(1 << 11), // R
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresentationHotkeyAction {
    CyclePresentation,
    CycleLighting,
    CycleShadows,
}

fn presentation_hotkey_action(
    key: KeyCode,
    state: ElementState,
) -> Option<PresentationHotkeyAction> {
    if state != ElementState::Pressed {
        return None;
    }
    match key {
        KeyCode::F6 => Some(PresentationHotkeyAction::CyclePresentation),
        KeyCode::F7 => Some(PresentationHotkeyAction::CycleLighting),
        KeyCode::F8 => Some(PresentationHotkeyAction::CycleShadows),
        _ => None,
    }
}

fn apply_presentation_hotkey(renderer: &mut FrameRenderer, action: PresentationHotkeyAction) {
    match action {
        PresentationHotkeyAction::CyclePresentation => renderer.cycle_presentation_mode(),
        PresentationHotkeyAction::CycleLighting => renderer.cycle_lighting_mode(),
        PresentationHotkeyAction::CycleShadows => renderer.cycle_shadow_mode(),
    }
}

const GAMEPAD_DEADZONE: f32 = 0.35;

struct GamepadInput {
    gilrs: Gilrs,
    button_state: u16,
    axis_state: u16,
}

impl GamepadInput {
    fn new() -> Option<Self> {
        if env_flag("ZELDA3_DISABLE_GAMEPAD") {
            return None;
        }
        let gilrs = Gilrs::new().ok()?;
        Some(Self {
            gilrs,
            button_state: 0,
            axis_state: 0,
        })
    }

    fn poll(&mut self) -> u16 {
        while let Some(event) = self.gilrs.next_event() {
            self.apply_event(event);
        }
        self.button_state | self.axis_state
    }

    fn apply_event(&mut self, event: Event) {
        match event.event {
            EventType::ButtonPressed(button, _) => {
                if let Some(bit) = gamepad_button_bit(button) {
                    self.button_state |= bit;
                }
            }
            EventType::ButtonReleased(button, _) => {
                if let Some(bit) = gamepad_button_bit(button) {
                    self.button_state &= !bit;
                }
            }
            EventType::AxisChanged(axis, value, _) => {
                update_axis_state(&mut self.axis_state, axis, value);
            }
            _ => {}
        }
    }
}

fn gamepad_button_bit(button: Button) -> Option<u16> {
    match button {
        Button::South => Some(1 << 0),
        Button::West => Some(1 << 1),
        Button::Select => Some(1 << 2),
        Button::Start => Some(1 << 3),
        Button::DPadUp => Some(1 << 4),
        Button::DPadDown => Some(1 << 5),
        Button::DPadLeft => Some(1 << 6),
        Button::DPadRight => Some(1 << 7),
        Button::East => Some(1 << 8),
        Button::North => Some(1 << 9),
        Button::LeftTrigger | Button::LeftTrigger2 => Some(1 << 10),
        Button::RightTrigger | Button::RightTrigger2 => Some(1 << 11),
        _ => None,
    }
}

fn update_axis_state(axis_state: &mut u16, axis: Axis, value: f32) {
    match axis {
        Axis::LeftStickX => update_axis_pair(axis_state, value, 1 << 6, 1 << 7),
        Axis::LeftStickY => update_axis_pair(axis_state, value, 1 << 5, 1 << 4),
        Axis::LeftZ => update_axis_trigger(axis_state, value, 1 << 10),
        Axis::RightZ => update_axis_trigger(axis_state, value, 1 << 11),
        _ => {}
    }
}

fn update_axis_pair(axis_state: &mut u16, value: f32, negative_bit: u16, positive_bit: u16) {
    *axis_state &= !(negative_bit | positive_bit);
    if value <= -GAMEPAD_DEADZONE {
        *axis_state |= negative_bit;
    } else if value >= GAMEPAD_DEADZONE {
        *axis_state |= positive_bit;
    }
}

fn update_axis_trigger(axis_state: &mut u16, value: f32, bit: u16) {
    if value >= GAMEPAD_DEADZONE {
        *axis_state |= bit;
    } else {
        *axis_state &= !bit;
    }
}

fn env_flag(name: &str) -> bool {
    env_flag_option(env::var_os(name)).unwrap_or(false)
}

fn env_flag_option(value: Option<OsString>) -> Option<bool> {
    let value = value?;
    let value = value.to_string_lossy();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Some(false);
    }
    Some(!matches!(
        trimmed.to_ascii_lowercase().as_str(),
        "0" | "false" | "no" | "off"
    ))
}

// ── AudioOutput ───────────────────────────────────────────────────────────────

struct AudioOutput {
    queue: Arc<Mutex<VecDeque<i16>>>,
    stream: cpal::Stream,
    sample_rate: u32,
    channels: u16,
    buffer_frames: u32,
    started: bool,
}

impl AudioOutput {
    fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "no default audio output device".to_string())?;
        let supported = device.default_output_config().map_err(|e| e.to_string())?;
        let sample_rate = supported.sample_rate().0;
        let channels = supported.channels();
        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let stream = match supported.sample_format() {
            cpal::SampleFormat::I16 => {
                build_output_stream::<i16>(&device, &config, Arc::clone(&queue))?
            }
            cpal::SampleFormat::U16 => {
                build_output_stream::<u16>(&device, &config, Arc::clone(&queue))?
            }
            cpal::SampleFormat::F32 => {
                build_output_stream::<f32>(&device, &config, Arc::clone(&queue))?
            }
            other => return Err(format!("unsupported audio sample format {other:?}")),
        };
        Ok(Self {
            queue,
            stream,
            sample_rate,
            channels,
            buffer_frames: 2048,
            started: false,
        })
    }

    fn push(&mut self, samples: &[i16], start_threshold_bytes: u32) {
        let mut should_start = false;
        if let Ok(mut queue) = self.queue.lock() {
            queue.extend(samples.iter().copied());
            should_start = queue.len().saturating_mul(std::mem::size_of::<i16>())
                >= start_threshold_bytes as usize;
        }
        if !self.started && should_start && self.stream.play().is_ok() {
            self.started = true;
        }
    }

    fn queued_bytes(&self) -> u32 {
        self.queue
            .lock()
            .map(|q| q.len().saturating_mul(std::mem::size_of::<i16>()) as u32)
            .unwrap_or(0)
    }
}

fn build_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    queue: Arc<Mutex<VecDeque<i16>>>,
) -> Result<cpal::Stream, String>
where
    T: cpal::Sample + cpal::SizedSample + FromI16,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                if let Ok(mut queue) = queue.lock() {
                    for sample in data {
                        let value = queue.pop_front().unwrap_or(0);
                        *sample = T::from_i16(value);
                    }
                } else {
                    for sample in data {
                        *sample = T::from_i16(0);
                    }
                }
            },
            move |err| eprintln!("audio stream error: {err}"),
            None,
        )
        .map_err(|e| e.to_string())
}

trait FromI16 {
    fn from_i16(sample: i16) -> Self;
}

impl FromI16 for i16 {
    fn from_i16(sample: i16) -> Self {
        sample
    }
}

impl FromI16 for u16 {
    fn from_i16(sample: i16) -> Self {
        (sample as i32 + 32768) as u16
    }
}

impl FromI16 for f32 {
    fn from_i16(sample: i16) -> Self {
        sample as f32 / i16::MAX as f32
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_delay_matches_c_pacing_pattern() {
        let delays: Vec<_> = (1..=6).map(frame_delay).collect();
        assert_eq!(
            delays,
            vec![
                Duration::from_millis(17),
                Duration::from_millis(16),
                Duration::from_millis(17),
                Duration::from_millis(17),
                Duration::from_millis(16),
                Duration::from_millis(17),
            ],
        );
    }

    #[test]
    fn key_release_clears_live_input_latch() {
        let mut input_state = 0;
        handle_key_input_state(&mut input_state, KeyCode::ArrowUp, ElementState::Pressed);
        assert_eq!(input_state, 1 << 4);
        handle_key_input_state(&mut input_state, KeyCode::ArrowUp, ElementState::Released);
        assert_eq!(input_state, 0);
    }

    #[test]
    fn focus_loss_clears_live_input_latch() {
        let mut input_state = 0;
        handle_key_input_state(&mut input_state, KeyCode::ArrowDown, ElementState::Pressed);
        handle_key_input_state(&mut input_state, KeyCode::ArrowRight, ElementState::Pressed);
        assert_ne!(input_state, 0);
        handle_focus_input_state(&mut input_state, false);
        assert_eq!(input_state, 0);
    }

    #[test]
    fn function_keys_map_to_presentation_hotkeys_only_on_press() {
        assert_eq!(
            presentation_hotkey_action(KeyCode::F6, ElementState::Pressed),
            Some(PresentationHotkeyAction::CyclePresentation)
        );
        assert_eq!(
            presentation_hotkey_action(KeyCode::F7, ElementState::Pressed),
            Some(PresentationHotkeyAction::CycleLighting)
        );
        assert_eq!(
            presentation_hotkey_action(KeyCode::F8, ElementState::Pressed),
            Some(PresentationHotkeyAction::CycleShadows)
        );
        assert_eq!(
            presentation_hotkey_action(KeyCode::F6, ElementState::Released),
            None
        );
        assert_eq!(
            presentation_hotkey_action(KeyCode::F5, ElementState::Pressed),
            None
        );
    }

    #[test]
    fn steamdeck_env_enables_fullscreen_by_default() {
        let options = NativeFrontendOptions::from_values(3, true, Some("1".into()), None);
        assert_eq!(
            options,
            NativeFrontendOptions {
                scale: 3,
                enable_audio: true,
                fullscreen: true,
            }
        );
    }

    #[test]
    fn fullscreen_env_overrides_steamdeck_default() {
        let options =
            NativeFrontendOptions::from_values(3, true, Some("1".into()), Some("0".into()));
        assert!(!options.fullscreen);
    }

    #[test]
    fn deck_face_buttons_map_to_snes_buttons() {
        assert_eq!(gamepad_button_bit(Button::South), Some(1 << 0));
        assert_eq!(gamepad_button_bit(Button::West), Some(1 << 1));
        assert_eq!(gamepad_button_bit(Button::East), Some(1 << 8));
        assert_eq!(gamepad_button_bit(Button::North), Some(1 << 9));
    }

    #[test]
    fn deck_left_stick_updates_direction_bits_with_deadzone() {
        let mut input_state = 0;
        update_axis_state(&mut input_state, Axis::LeftStickX, -0.5);
        assert_eq!(input_state, 1 << 6);
        update_axis_state(&mut input_state, Axis::LeftStickX, 0.1);
        assert_eq!(input_state, 0);
        update_axis_state(&mut input_state, Axis::LeftStickY, 0.5);
        assert_eq!(input_state, 1 << 4);
    }

    #[test]
    fn deck_analog_triggers_update_shoulder_bits_with_deadzone() {
        let mut input_state = 0;
        update_axis_state(&mut input_state, Axis::LeftZ, 0.5);
        update_axis_state(&mut input_state, Axis::RightZ, 0.5);
        assert_eq!(input_state, (1 << 10) | (1 << 11));
        update_axis_state(&mut input_state, Axis::LeftZ, 0.1);
        assert_eq!(input_state, 1 << 11);
    }
}
