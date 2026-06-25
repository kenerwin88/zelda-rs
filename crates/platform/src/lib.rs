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

pub mod host_menu;
pub use host_menu::{
    ControlsPanel, DeveloperDestination, DeveloperDestinationStatus, HostMenuAction,
    HostMenuInput, HostMenuMode, HostMenuState, HostMenuTab, LightingChoice, PresentationChoice,
    RuntimeSettings, ShadowChoice, ViewportChoice,
};

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
            host_menu_inputs: VecDeque::new(),
            menu_open: false,
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

    pub fn set_menu_open(&mut self, open: bool) {
        self.handler.menu_open = open;
        if open {
            self.handler.input_state = 0;
        }
        if let Some(audio) = &mut self.audio {
            audio.set_volume_scale(if open { 0.35 } else { 1.0 });
        }
    }

    pub fn drain_host_menu_inputs(&mut self) -> Vec<HostMenuInput> {
        self.handler.host_menu_inputs.drain(..).collect()
    }

    pub fn poll_input_with_menu(&mut self, menu_open: bool) -> u16 {
        self.set_menu_open(menu_open);
        if menu_open {
            let _ = self.poll_input();
            0
        } else {
            self.poll_input()
        }
    }

    pub fn apply_runtime_settings(&mut self, settings: RuntimeSettings) {
        let renderer_settings = renderer::RendererRuntimeSettings {
            presentation: match settings.presentation {
                PresentationChoice::Off => renderer::RendererPresentationChoice::Off,
                PresentationChoice::Sharp => renderer::RendererPresentationChoice::Sharp,
                PresentationChoice::Crt => renderer::RendererPresentationChoice::Crt,
            },
            lighting: match settings.lighting {
                LightingChoice::Off => renderer::RendererLightingChoice::Off,
                LightingChoice::Ambient => renderer::RendererLightingChoice::Ambient,
                LightingChoice::Dynamic => renderer::RendererLightingChoice::Dynamic,
            },
            shadows: match settings.shadows {
                ShadowChoice::Off => renderer::RendererShadowChoice::Off,
                ShadowChoice::Raycast => renderer::RendererShadowChoice::Raycast,
            },
            viewport: match settings.viewport {
                ViewportChoice::Integer => renderer::RendererViewportChoice::Integer,
                ViewportChoice::Fit => renderer::RendererViewportChoice::Fit,
                ViewportChoice::Stretch => renderer::RendererViewportChoice::Stretch,
            },
        };
        if let Some(renderer) = &mut self.handler.renderer {
            renderer.apply_runtime_settings(renderer_settings);
        }
    }

    pub fn present_menu_overlay(&mut self, menu: &HostMenuState) {
        let overlay = renderer::MenuOverlayModel {
            tab: match menu.active_tab() {
                HostMenuTab::Play => renderer::MenuOverlayTab::Play,
                HostMenuTab::Video => renderer::MenuOverlayTab::Video,
                HostMenuTab::Controls => renderer::MenuOverlayTab::Controls,
                HostMenuTab::DeveloperMap => renderer::MenuOverlayTab::DeveloperMap,
            },
            selected_index: 0,
            lines: menu_overlay_lines(menu),
        };
        if let Some(renderer) = &mut self.handler.renderer {
            match renderer.render_menu_overlay(&overlay) {
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
    host_menu_inputs: VecDeque<HostMenuInput>,
    menu_open: bool,
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
                    if let Some(input) = key_to_host_menu_input(key, event.state) {
                        self.host_menu_inputs.push_back(input);
                    }
                    handle_key_input_state_with_menu(
                        &mut self.input_state,
                        key,
                        event.state,
                        self.menu_open,
                    );
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

fn key_to_host_menu_input(key: KeyCode, state: ElementState) -> Option<HostMenuInput> {
    if state != ElementState::Pressed {
        return None;
    }
    match key {
        KeyCode::Escape => Some(HostMenuInput::Cancel),
        KeyCode::F6 => Some(HostMenuInput::CyclePresentation),
        KeyCode::F7 => Some(HostMenuInput::CycleLighting),
        KeyCode::F8 => Some(HostMenuInput::CycleShadows),
        KeyCode::ArrowUp => Some(HostMenuInput::Up),
        KeyCode::ArrowDown => Some(HostMenuInput::Down),
        KeyCode::ArrowLeft => Some(HostMenuInput::Left),
        KeyCode::ArrowRight => Some(HostMenuInput::Right),
        KeyCode::Enter | KeyCode::KeyZ | KeyCode::KeyX => Some(HostMenuInput::Confirm),
        KeyCode::Tab | KeyCode::KeyE | KeyCode::KeyV | KeyCode::KeyW => {
            Some(HostMenuInput::NextTab)
        }
        KeyCode::KeyQ | KeyCode::KeyC => Some(HostMenuInput::PreviousTab),
        _ => None,
    }
}

fn handle_key_input_state_with_menu(
    input_state: &mut u16,
    key: KeyCode,
    state: ElementState,
    menu_open: bool,
) {
    if menu_open {
        return;
    }
    handle_key_input_state(input_state, key, state);
}

fn menu_overlay_lines(menu: &HostMenuState) -> Vec<&'static str> {
    match menu.active_tab() {
        HostMenuTab::Play => play_menu_overlay_lines(menu.mode(), menu.selected_label()),
        HostMenuTab::Video => video_menu_overlay_lines(menu.selected_label()),
        HostMenuTab::Controls => controls_menu_overlay_lines(menu.selected_label(), menu.controls_panel()),
        HostMenuTab::DeveloperMap => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> CURATED PRESETS",
            "  ROUTE BOOKMARKS",
            "  LOCKED BROWSER",
        ],
    }
}

fn play_menu_overlay_lines(mode: HostMenuMode, selected: &'static str) -> Vec<&'static str> {
    let primary = match mode {
        HostMenuMode::PreGame => {
            selected_line(selected, "Start Quest", "> START QUEST", "  START QUEST")
        }
        HostMenuMode::InGame => {
            selected_line(selected, "Resume Quest", "> RESUME QUEST", "  RESUME QUEST")
        }
    };
    let exit = match mode {
        HostMenuMode::PreGame => selected_line(selected, "Quit", "> QUIT", "  QUIT"),
        HostMenuMode::InGame => {
            selected_line(selected, "Save & Quit", "> SAVE & QUIT", "  SAVE & QUIT")
        }
    };
    vec![
        "PLAY  VIDEO  CONTROLS  DEV MAP",
        primary,
        selected_line(
            selected,
            "Video & Effects",
            "> VIDEO & EFFECTS",
            "  VIDEO & EFFECTS",
        ),
        selected_line(selected, "Controls", "> CONTROLS", "  CONTROLS"),
        selected_line(
            selected,
            "Developer Map",
            "> DEVELOPER MAP",
            "DEVELOPER MAP",
        ),
        exit,
    ]
}

fn video_menu_overlay_lines(selected: &'static str) -> Vec<&'static str> {
    vec![
        "PLAY  VIDEO  CONTROLS  DEV MAP",
        selected_line(selected, "Presentation", "> PRESENTATION", "  PRESENTATION"),
        selected_line(selected, "Lighting", "> LIGHTING", "  LIGHTING"),
        selected_line(selected, "Shadows", "> SHADOWS", "  SHADOWS"),
        selected_line(selected, "Viewport", "> VIEWPORT", "  VIEWPORT"),
    ]
}

fn controls_menu_overlay_lines(
    selected: &'static str,
    panel: Option<ControlsPanel>,
) -> Vec<&'static str> {
    match panel {
        Some(ControlsPanel::Keyboard) => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> KEYBOARD",
            "  Z/X CONFIRM    ESC BACK",
            "  ARROWS MOVE    F6-F8 FX",
            selected_line(
                selected,
                "Reset Defaults",
                "> RESET DEFAULTS",
                "  RESET DEFAULTS",
            ),
        ],
        Some(ControlsPanel::Gamepad) => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> GAMEPAD",
            "  DPAD MOVE      A CONFIRM",
            "  B BACK         L/R TABS",
            selected_line(
                selected,
                "Reset Defaults",
                "> RESET DEFAULTS",
                "  RESET DEFAULTS",
            ),
        ],
        None => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            selected_line(selected, "Keyboard", "> KEYBOARD", "  KEYBOARD"),
            selected_line(selected, "Gamepad", "> GAMEPAD", "  GAMEPAD"),
            selected_line(
                selected,
                "Reset Defaults",
                "> RESET DEFAULTS",
                "  RESET DEFAULTS",
            ),
        ],
    }
}

fn selected_line(
    selected: &'static str,
    label: &'static str,
    active: &'static str,
    inactive: &'static str,
) -> &'static str {
    if selected == label {
        active
    } else {
        inactive
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
    volume_scale: Arc<Mutex<f32>>,
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
        let volume_scale = Arc::new(Mutex::new(1.0));
        let stream = match supported.sample_format() {
            cpal::SampleFormat::I16 => build_output_stream::<i16>(
                &device,
                &config,
                Arc::clone(&queue),
                Arc::clone(&volume_scale),
            )?,
            cpal::SampleFormat::U16 => build_output_stream::<u16>(
                &device,
                &config,
                Arc::clone(&queue),
                Arc::clone(&volume_scale),
            )?,
            cpal::SampleFormat::F32 => build_output_stream::<f32>(
                &device,
                &config,
                Arc::clone(&queue),
                Arc::clone(&volume_scale),
            )?,
            other => return Err(format!("unsupported audio sample format {other:?}")),
        };
        Ok(Self {
            queue,
            volume_scale,
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

    fn set_volume_scale(&mut self, scale: f32) {
        if let Ok(mut value) = self.volume_scale.lock() {
            *value = scale.clamp(0.0, 1.0);
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
    volume_scale: Arc<Mutex<f32>>,
) -> Result<cpal::Stream, String>
where
    T: cpal::Sample + cpal::SizedSample + FromI16,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                if let Ok(mut queue) = queue.lock() {
                    let scale = volume_scale.lock().map(|value| *value).unwrap_or(1.0);
                    for sample in data {
                        let value = scale_i16_sample(queue.pop_front().unwrap_or(0), scale);
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

fn scale_i16_sample(sample: i16, scale: f32) -> i16 {
    let scaled = (sample as f32 * scale).round();
    scaled.clamp(i16::MIN as f32, i16::MAX as f32) as i16
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
    fn escape_maps_to_host_menu_input() {
        assert_eq!(
            key_to_host_menu_input(KeyCode::Escape, ElementState::Pressed),
            Some(HostMenuInput::Cancel)
        );
        assert_eq!(
            key_to_host_menu_input(KeyCode::Escape, ElementState::Released),
            None
        );
    }

    #[test]
    fn menu_open_consumes_snes_direction_keys() {
        let mut input_state = 0;
        handle_key_input_state_with_menu(
            &mut input_state,
            KeyCode::ArrowDown,
            ElementState::Pressed,
            true,
        );
        assert_eq!(input_state, 0);
        handle_key_input_state_with_menu(
            &mut input_state,
            KeyCode::ArrowDown,
            ElementState::Pressed,
            false,
        );
        assert_eq!(input_state, 1 << 5);
    }

    #[test]
    fn audio_ducking_scales_i16_samples() {
        assert_eq!(scale_i16_sample(10_000, 0.25), 2_500);
        assert_eq!(scale_i16_sample(-10_000, 0.25), -2_500);
        assert_eq!(scale_i16_sample(i16::MAX, 2.0), i16::MAX);
    }

    #[test]
    fn function_keys_map_to_host_menu_shortcuts_only_on_press() {
        assert_eq!(
            key_to_host_menu_input(KeyCode::F6, ElementState::Pressed),
            Some(HostMenuInput::CyclePresentation)
        );
        assert_eq!(
            key_to_host_menu_input(KeyCode::F7, ElementState::Pressed),
            Some(HostMenuInput::CycleLighting)
        );
        assert_eq!(
            key_to_host_menu_input(KeyCode::F8, ElementState::Pressed),
            Some(HostMenuInput::CycleShadows)
        );
        assert_eq!(
            key_to_host_menu_input(KeyCode::F6, ElementState::Released),
            None
        );
        assert_eq!(
            key_to_host_menu_input(KeyCode::F5, ElementState::Pressed),
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
