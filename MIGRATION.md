---
  Platform migration plan: winit 0.28 + pixels → winit 0.30 + wgpu

  Mental model first

  The fundamental change is control-flow inversion. Today the game drives the window:

  loop {
      frontend.poll_input()     // pumps winit events internally
      game.step()
      frontend.present_frame()  // blits pixels + sleeps
  }

  In winit 0.30, the window drives the game. event_loop.run_app() never returns (on macOS it literally can't — the OS owns the main thread). Your code lives inside callbacks:

  RedrawRequested → step game + upload pixels + render + sleep + request_redraw()
  KeyboardInput   → update input_state bitmask
  CloseRequested  → event_loop.exit()
  Resumed         → create window + wgpu surface (first time only)

  Every piece of the plan flows from this.

  ---
  Crate map after migration

  crates/renderer/   ← NEW: wgpu blit layer, later grows into modern renderer
  crates/platform/   ← REWRITTEN: winit 0.30 ApplicationHandler, drops pixels
  zelda3-bin/        ← MODIFIED: game loop moves into GameApp impl
  crates/snes/       ← untouched
  crates/zelda3/     ← untouched

  ---
  Step 1 — New crate: crates/renderer

  Purpose: own the wgpu device/queue/surface and expose one operation: "here are raw BGRA pixels at game resolution, put them on screen scaled correctly." Everything else (the modern tile renderer, HD sprites,
  etc.) will also live here eventually — starting it as a separate crate draws the boundary clearly.

  crates/renderer/Cargo.toml:
  [package]
  name = "renderer"
  edition.workspace = true
  rust-version.workspace = true
  license.workspace = true

  [dependencies]
  wgpu = "29"
  winit = "0.30"
  pollster = "0.3"

  [lints]
  workspace = true

  crates/renderer/src/blit.wgsl:

  Fullscreen triangle trick — no vertex buffer needed:
  struct VertexOut {
      @builtin(position) pos: vec4<f32>,
      @location(0) uv: vec2<f32>,
  }

  @vertex
  fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOut {
      let x = f32(vi & 1u) * 4.0 - 1.0;
      let y = 1.0 - f32(vi >> 1u) * 4.0;
      var out: VertexOut;
      out.pos = vec4<f32>(x, y, 0.0, 1.0);
      out.uv  = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
      return out;
  }

  @group(0) @binding(0) var t_frame: texture_2d<f32>;
  @group(0) @binding(1) var s_nearest: sampler;

  @fragment
  fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
      return textureSample(t_frame, s_nearest, in.uv);
  }

  crates/renderer/src/lib.rs — public API:

  use std::sync::Arc;
  use winit::dpi::PhysicalSize;
  use winit::window::Window;

  pub struct FrameRenderer {
      surface: wgpu::Surface<'static>,
      device: wgpu::Device,
      queue: wgpu::Queue,
      config: wgpu::SurfaceConfiguration,
      pipeline: wgpu::RenderPipeline,
      game_texture: wgpu::Texture,        // game_width × game_height, Bgra8Unorm
      bind_group: wgpu::BindGroup,
      game_width: u32,
      game_height: u32,
      viewport: Viewport,                 // letterbox rect, recomputed on resize
  }

  struct Viewport { x: f32, y: f32, w: f32, h: f32 }

  impl FrameRenderer {
      pub async fn new(window: Arc<Window>, game_width: u32, game_height: u32) -> Self;
      pub fn resize(&mut self, new_size: PhysicalSize<u32>);
      /// `pixels` is BGRA u32s, length == game_width * game_height.
      pub fn upload_frame(&mut self, pixels: &[u32]);
      pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
  }

  Key implementation choices:

  - wgpu::TextureFormat::Bgra8Unorm for the game texture. The PPU writes BGRA bytes (color_map_rgb is 0xff000000 | B<<16 | G<<8 | R) so we can write_texture directly without any channel swap. This replaces the
  current swap in present_frame().
  - wgpu::FilterMode::Nearest for the sampler — pixel art, always.
  - wgpu::PresentMode::Fifo (vsync) for the surface — we'll use our own frame_delay sleep on top; Fifo prevents screen tearing while still letting us control pacing.
  - Letterbox viewport: on resize, compute scale = min(surface_w/game_w, surface_h/game_h) (integer-only for sharp pixels), center the scale*game_w × scale*game_h rect, pass to render_pass.set_viewport().
  - Texture upload: queue.write_texture() per frame. At 256×224×4 = ~230 KB, this is fine — no staging buffer complexity needed at this scale.
  - wgpu::Surface<'static> accepts Arc<Window> directly via instance.create_surface(Arc::clone(&window)). No Box::leak, no lifetime gymnastics.

  Workspace change (Cargo.toml):
  members = [
      "crates/snes",
      "crates/zelda3",
      "crates/platform",
      "crates/assets",
      "crates/renderer",   # add
      "zelda3-bin",
  ]

  ---
  Step 2 — Rewrite crates/platform

  crates/platform/Cargo.toml:
  [dependencies]
  winit = "0.30"
  renderer = { path = "../renderer" }
  cpal = "0.15"
  pollster = "0.3"
  # removed: pixels = "0.13"

  New module layout:
  crates/platform/src/
    lib.rs     — public re-exports + GameApp trait
    app.rs     — NativeApp: the ApplicationHandler impl
    audio.rs   — AudioOutput (copy-paste unchanged from current lib.rs)
    input.rs   — key_to_input_bit (VirtualKeyCode → KeyCode)

  The GameApp trait

  This replaces the current Frontend trait. The binary implements this; platform calls it.

  pub trait GameApp {
      /// Called once to get the game's pixel dimensions.
      fn game_size(&self) -> (u32, u32);

      /// Called once after audio device is negotiated.
      fn configure_audio(&mut self, samples_per_frame: usize, channels: usize);

      /// The frame tick. Fill `pixels` (BGRA u32) and `audio` (i16 interleaved).
      /// Return false to request quit.
      fn step(&mut self, input: u16, pixels: &mut [u32], audio: &mut [i16]) -> bool;

      /// Called when the window closes (save SRAM, flush logs, etc.).
      fn on_quit(&mut self);
  }

  pub fn run_native<G: GameApp + 'static>(
      mut game: G,
      title: &str,
      scale: u32,
      enable_audio: bool,
  );

  The pixels and audio slices are pre-allocated by NativeApp from the dimensions game.game_size() returns, so neither the trait nor the game implementation allocates per-frame.

  NativeApp: ApplicationHandler

  struct NativeApp<G: GameApp> {
      game: G,
      title: String,
      scale: u32,
      enable_audio: bool,
      // filled in Resumed:
      window: Option<Arc<Window>>,
      renderer: Option<FrameRenderer>,
      audio: Option<AudioOutput>,
      // frame state:
      input_state: u16,
      pixels: Vec<u32>,         // game_w * game_h
      audio_buf: Vec<i16>,      // samples_per_frame * channels
      next_frame_tick: Instant,
      presented_frames: u32,
  }

  fn resumed(&mut self, event_loop: &ActiveEventLoop):
  let attrs = Window::default_attributes()
      .with_title(&self.title)
      .with_inner_size(LogicalSize::new(game_w * self.scale, game_h * self.scale))
      .with_min_inner_size(LogicalSize::new(game_w, game_h))
      .with_resizable(true);
  let window = Arc::new(event_loop.create_window(attrs).unwrap());
  let renderer = pollster::block_on(FrameRenderer::new(Arc::clone(&window), game_w, game_h));
  let audio = if self.enable_audio { AudioOutput::new().ok() } else { None };
  if let Some(a) = &audio {
      self.game.configure_audio(samples_per_frame(a), a.channels as usize);
      self.audio_buf.resize(samples_per_frame(a) * a.channels as usize, 0);
  }
  self.window = Some(window);
  self.renderer = Some(renderer);
  self.audio = audio;
  self.window.as_ref().unwrap().request_redraw();

  fn window_event(..., WindowEvent::RedrawRequested):
  let should_continue = self.game.step(self.input_state, &mut self.pixels, &mut self.audio_buf);
  if !should_continue {
      event_loop.exit();
      return;
  }
  if let Some(r) = &mut self.renderer {
      r.upload_frame(&self.pixels);
      match r.render() {
          Ok(()) => {}
          Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => r.resize(...),
          Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
          Err(e) => eprintln!("render error: {e:?}"),
      }
  }
  if let Some(a) = &mut self.audio {
      a.push(&self.audio_buf, self.audio_queue_target_bytes);
  }
  // Frame pacing — identical to current to keep parity gate happy
  self.next_frame_tick += frame_delay(self.presented_frames);
  self.presented_frames = self.presented_frames.wrapping_add(1);
  let now = Instant::now();
  if self.next_frame_tick > now {
      std::thread::sleep(self.next_frame_tick - now);
  } else if now.duration_since(self.next_frame_tick) > Duration::from_millis(500) {
      self.next_frame_tick = now;
  }
  self.window.as_ref().unwrap().request_redraw();

  fn window_event(..., WindowEvent::KeyboardInput { event, .. }):
  // winit 0.30: event.physical_key is PhysicalKey::Code(KeyCode)
  if let PhysicalKey::Code(key) = event.physical_key {
      if key == KeyCode::Escape && event.state.is_pressed() {
          event_loop.exit();
      }
      if let Some(bit) = key_to_input_bit(key) {
          match event.state {
              ElementState::Pressed  => self.input_state |= bit,
              ElementState::Released => self.input_state &= !bit,
          }
      }
  }

  fn window_event(..., WindowEvent::Resized(size)):
  if let Some(r) = &mut self.renderer { r.resize(size); }

  fn window_event(..., WindowEvent::CloseRequested):
  self.game.on_quit();
  event_loop.exit();

  Input mapping update (input.rs)

  winit 0.30 removed VirtualKeyCode, replaced with winit::keyboard::KeyCode (physical position, layout-independent):

  ┌────────────────────────────────────┬──────────────────────────────────┐
  │                Old                 │               New                │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::Z                  │ KeyCode::KeyZ                    │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::A                  │ KeyCode::KeyA                    │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::RShift             │ KeyCode::ShiftRight              │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::Return             │ KeyCode::Enter                   │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::Up/Down/Left/Right │ KeyCode::ArrowUp/Down/Left/Right │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::X                  │ KeyCode::KeyX                    │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::S                  │ KeyCode::KeyS                    │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::C, Q               │ KeyCode::KeyC, KeyCode::KeyQ     │
  ├────────────────────────────────────┼──────────────────────────────────┤
  │ VirtualKeyCode::V, W               │ KeyCode::KeyV, KeyCode::KeyW     │
  └────────────────────────────────────┴──────────────────────────────────┘

  AudioOutput (audio.rs)

  Zero changes. Copy the 70 lines verbatim. cpal is winit-independent.

  frame_delay — preserve exactly

  The parity gate (scripts/test_standard_replay_parity.py:157) literally grep-checks for the string self.next_frame_tick += frame_delay(self.presented_frames) in Rust source and fails the test if it's absent. Keep
  the function name and the call site text identical:

  fn frame_delay(frame: u32) -> Duration {
      const DELAYS_MS: [u64; 3] = [17, 17, 16];
      Duration::from_millis(DELAYS_MS[(frame % 3) as usize])
  }

  The parity test also checks DELAYS_MS: [u64; 3] = [17, 17, 16] by regex. Keep the constant spelled out exactly like that.

  ---
  Step 3 — Update zelda3-bin/src/main.rs

  Two game loops need to implement GameApp: play mode and play-lockstep mode. The headless/replay-crash/replay-save modes never use a window, so they stay as-is.

  Pixel format cleanup

  Currently (current present_frame):
  // pixels: &[u32] packed 0x00RRGGBB from ppu
  // swap B↔R before upload to pixels crate's Rgba8 surface
  let rgb = src.to_le_bytes();  // bytes are [B, G, R, 0]
  dst[0] = rgb[2];  // R
  dst[1] = rgb[1];  // G
  dst[2] = rgb[0];  // B

  With Bgra8Unorm texture in the new renderer, the PPU's native BGRA layout is correct as-is. The &[u32] from frame can go directly to upload_frame with no swap. The unsafe from_raw_parts reinterpret cast in the
  binary stays, but it's now correct without the RGB shuffle.

  PlayApp: GameApp

  struct PlayApp {
      game: ZeldaState,
      frame: Vec<u8>,                // 256 * 224 * 4 bytes, BGRA from PPU
      render_flags: PpuRenderFlags,
      audio_samples: usize,
      audio_channels: usize,
      last_panic: Arc<Mutex<Option<String>>>,
      previous_input: u16,
      host_frame: u32,
  }

  impl GameApp for PlayApp {
      fn game_size(&self) -> (u32, u32) { (256, 224) }

      fn configure_audio(&mut self, samples: usize, channels: usize) {
          self.audio_samples = samples;
          self.audio_channels = channels;
      }

      fn step(&mut self, input: u16, pixels: &mut [u32], audio: &mut [i16]) -> bool {
          let input = pulse_live_name_entry_directions(input, self.previous_input, &self.game.ram);
          self.previous_input = input;
          let run_what = select_run_what(&self.game.ram);
          let pre_frame = self.game.clone();
          let result = panic::catch_unwind(AssertUnwindSafe(|| {
              run_play_frame(&mut self.game, input, &mut self.frame, self.render_flags);
              self.game.zelda_render_audio(audio, self.audio_samples as i32, self.audio_channels as i32);
              self.game.zelda_discard_unused_audio_frames();
          }));
          if let Err(payload) = result {
              // crash report path — identical to current
              write_play_crash_report(&pre_frame, self.host_frame, input, run_what, ...);
              return false;
          }
          // Reinterpret BGRA u8 → u32 directly (no swap needed with Bgra8Unorm texture)
          let src = bytemuck::cast_slice::<u8, u32>(&self.frame);
          pixels.copy_from_slice(src);
          self.host_frame = self.host_frame.wrapping_add(1);
          true
      }

      fn on_quit(&mut self) {
          // SRAM save — same as current cleanup code
      }
  }

  (bytemuck for the cast avoids the unsafe from_raw_parts; add it to bin's deps. Alternatively keep the unsafe cast — it's correct.)

  run_play becomes:
  fn run_play(rom_path: &str) {
      let game = load_play_state(rom_path);
      let app = PlayApp { game, frame: vec![0u8; 256 * 224 * 4], ... };
      platform::run_native(app, "The Legend of Zelda: A Link to the Past", 3, true);
  }

  LockstepApp: GameApp

  Same shape. step calls oracle.run_frame_with_compare(input, run_what), handles the Err path (print diff, write artifacts, return false). The frame_limit field: step returns false when local_frame >= frame_limit.

  One subtlety: lockstep mode currently renders two frames (game + SNES oracle) side-by-side in some display modes. That logic stays inside step — it writes into pixels using the same 256-wide buffer (or a wider
  one if the side-by-side display is active).

  ---
  Step 4 — Dependency cleanup

  Remove from crates/platform/Cargo.toml:
  # delete:
  pixels = "0.13"
  winit = "0.28"

  After this, the transitive wgpu that pixels was pulling in disappears from the lockfile and is replaced by the direct wgpu = "29" dep in crates/renderer. Cargo.lock will regenerate cleanly.

  ---
  Parity gate preservation checklist

  ┌───────────────────────────────────────────────────────────────────────────┬────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
  │                                   Check                                   │                                                 How we preserve it                                                 │
  ├───────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ DELAYS_MS: [u64; 3] = [17, 17, 16] (regex)                                │ Constant kept verbatim in platform/src/lib.rs                                                                      │
  ├───────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ self.next_frame_tick += frame_delay(self.presented_frames) (literal grep) │ Code string kept identical in NativeApp::window_event                                                              │
  ├───────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ (534 * sample_rate) / 32000 audio calc                                    │ AudioOutput unchanged, formula preserved                                                                           │
  ├───────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ WRAM/SRAM frame-state comparison                                          │ Untouched — in crates/zelda3, not platform                                                                         │
  ├───────────────────────────────────────────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ Render hash (pixel comparison)                                            │ PPU output path unchanged; pixel format now Bgra8Unorm (same byte layout as PPU native output, no channel reorder) │
  └───────────────────────────────────────────────────────────────────────────┴────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

  ---
  What this unlocks afterward

  Once this lands, crates/renderer is the natural home for:
  - Scale filtering (integer scale + configurable letterbox vs. stretch)
  - Modern tile renderer — a second wgpu pipeline that reads DisplaySnapshot (VRAM/CGRAM/OAM) and renders natively in the GPU rather than through the scanline emulator
  - HD sprite substitution — keyed on (charnum, flags, palette_hash) from OAM entries, resolved at render time, game state never touched
  - Frame interpolation — lerp between two consecutive DisplaySnapshots for sub-frame motion smoothing at 120Hz

  None of those touch parity. They're all additive pipelines in crates/renderer gated by a flag in PpuRenderFlags or a new render mode enum.

  ---
  Execution order summary

  1. crates/renderer — new crate, wgpu blit, independent of everything. Build and test standalone (unit test: upload a solid-color frame, verify rendered output matches via readback).
  2. crates/platform rewrite — swap winit + drop pixels. Gate: cargo build compiles, existing frame_delay unit test passes.
  3. zelda3-bin GameApp impls — implement for play and lockstep. Gate: manual smoke test (ROM loads, renders, input works).
  4. Parity gate — run python3 scripts/test_standard_replay_parity.py. This is the full correctness check.
  5. Delete pixels — verify it's no longer in the lockfile.
