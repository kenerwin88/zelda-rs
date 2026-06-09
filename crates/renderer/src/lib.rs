//! wgpu-based renderer for zelda3-rs.
//!
//! Two renderers share the same blit pipeline:
//! - [`FrameRenderer`]: renders to a winit window surface (live display).
//! - [`OffscreenRenderer`]: renders to an offscreen texture and reads pixels
//!   back to CPU (headless replay, `--render-hash-log`, `--dump-frame`).
//!
//! Phase 1a — GPU tile atlas infrastructure:
//! - [`GpuFrame`]: zero-copy data bundle from `PpuState` to the renderer.
//! - [`TileAtlas`]: GPU texture of decoded 4bpp tile palette indices (512×256).
//! - [`CgramPalette`]: GPU texture of decoded CGRAM colours (256×1 RGBA).
//!
//! Phase 1b — BG layer rendering:
//! - [`BgLayerRenderer`]: single-layer BG pipeline (tilemap → atlas → CGRAM).

pub mod bg_layer;
pub mod gpu_frame;
pub mod gpu_renderer;
pub mod mode7_renderer;
pub mod post_process;
pub mod sprite_renderer;
pub mod tile_atlas;

pub use bg_layer::BgLayerRenderer;
pub use gpu_frame::{BgLayerRegs, GpuFrame, Mode7Regs, ObjRegs, ScanlineRegs};
pub use gpu_renderer::GpuFrameRenderer;
pub use mode7_renderer::Mode7Renderer;
pub use post_process::scanlines_from_raw;
pub use tile_atlas::{CgramPalette, TileAtlas, ATLAS_HEIGHT, ATLAS_TILE_COUNT, ATLAS_WIDTH};

use std::{env, sync::Arc};

use winit::dpi::PhysicalSize;
use winit::window::Window;

// ── Viewport ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct Viewport {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewportScaleMode {
    Integer,
    Fit,
    Stretch,
}

impl ViewportScaleMode {
    fn from_env() -> Self {
        let value = env::var("ZELDA3_VIEWPORT_SCALE")
            .ok()
            .or_else(|| env::var("ZELDA3_SCALE_MODE").ok());

        match value.map(|s| s.to_ascii_lowercase()) {
            Some(value) if matches!(value.as_str(), "fit" | "aspect-fit" | "aspect_fit") => {
                Self::Fit
            }
            Some(value) if matches!(value.as_str(), "stretch" | "fullscreen") => Self::Stretch,
            Some(value) if matches!(value.as_str(), "integer" | "pixel" | "pixel-perfect") => {
                Self::Integer
            }
            Some(_) => Self::Integer,
            None if env::var_os("ZELDA3_STEAMDECK").is_some() => Self::Fit,
            None => Self::Integer,
        }
    }
}

/// Compute the centered game rect that fits in `surface`.
fn compute_viewport(
    surface_w: u32,
    surface_h: u32,
    game_w: u32,
    game_h: u32,
    mode: ViewportScaleMode,
) -> Viewport {
    match mode {
        ViewportScaleMode::Integer => {
            compute_integer_viewport(surface_w, surface_h, game_w, game_h)
        }
        ViewportScaleMode::Fit => compute_fit_viewport(surface_w, surface_h, game_w, game_h),
        ViewportScaleMode::Stretch => Viewport {
            x: 0.0,
            y: 0.0,
            w: surface_w as f32,
            h: surface_h as f32,
        },
    }
}

/// Compute the largest integer-scaled, centered game rect that fits in `surface`.
fn compute_integer_viewport(surface_w: u32, surface_h: u32, game_w: u32, game_h: u32) -> Viewport {
    let scale = (surface_w / game_w).min(surface_h / game_h).max(1);
    let scaled_w = game_w * scale;
    let scaled_h = game_h * scale;
    // saturating_sub: if scale=1 and the window is smaller than the game,
    // offset is 0 (no centering room) and the viewport is clamped to the surface.
    let x = surface_w.saturating_sub(scaled_w) / 2;
    let y = surface_h.saturating_sub(scaled_h) / 2;
    let w = scaled_w.min(surface_w);
    let h = scaled_h.min(surface_h);
    Viewport {
        x: x as f32,
        y: y as f32,
        w: w as f32,
        h: h as f32,
    }
}

/// Compute the largest aspect-preserving centered game rect that fits in `surface`.
fn compute_fit_viewport(surface_w: u32, surface_h: u32, game_w: u32, game_h: u32) -> Viewport {
    if surface_w == 0 || surface_h == 0 || game_w == 0 || game_h == 0 {
        return Viewport {
            x: 0.0,
            y: 0.0,
            w: surface_w as f32,
            h: surface_h as f32,
        };
    }

    let scale = (surface_w as f32 / game_w as f32).min(surface_h as f32 / game_h as f32);
    let w = game_w as f32 * scale;
    let h = game_h as f32 * scale;
    Viewport {
        x: (surface_w as f32 - w) * 0.5,
        y: (surface_h as f32 - h) * 0.5,
        w,
        h,
    }
}

// ── RenderError ───────────────────────────────────────────────────────────────

/// Errors returned by [`FrameRenderer::render`].
#[derive(Debug)]
pub enum RenderError {
    /// Surface was lost or became outdated; caller should call [`FrameRenderer::resize`].
    SurfaceReconfigureNeeded,
    /// Surface was temporarily unavailable; caller can skip this frame.
    SurfaceSkipped,
    /// Unrecoverable render error.
    Fatal(String),
}

// ── Shared GPU helpers ────────────────────────────────────────────────────────

fn create_wgpu_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: None,
    })
}

async fn create_device_queue(
    instance: &wgpu::Instance,
    compatible_surface: Option<&wgpu::Surface<'_>>,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface,
            force_fallback_adapter: false,
        })
        .await
        .expect("no suitable GPU adapter");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            experimental_features: Default::default(),
            memory_hints: Default::default(),
            trace: Default::default(),
        })
        .await
        .expect("failed to create wgpu device");
    (adapter, device, queue)
}

/// Creates the game-frame input texture, its bind group layout, and bind group.
///
/// The texture is `Rgba8Unorm` (TEXTURE_BINDING | COPY_DST). Callers upload
/// pixels via [`upload_ppu_pixels`]; the bind group wires it to the blit shader.
fn create_game_texture_resources(
    device: &wgpu::Device,
    game_width: u32,
    game_height: u32,
) -> (wgpu::Texture, wgpu::BindGroupLayout, wgpu::BindGroup) {
    let game_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("game_frame"),
        size: wgpu::Extent3d {
            width: game_width,
            height: game_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let game_texture_view = game_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("nearest"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("blit"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("blit"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&game_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    (game_texture, bind_group_layout, bind_group)
}

fn create_blit_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    output_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("blit"),
        source: wgpu::ShaderSource::Wgsl(include_str!("blit.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("blit"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blit"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: output_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// Swap PPU pixels (0xFF_RR_GG_BB) to RGBA order and upload to the game texture.
///
/// Uses `staging` as a pre-allocated scratch buffer to avoid per-frame allocation.
fn upload_ppu_pixels(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    pixels: &[u32],
    staging: &mut Vec<u8>,
    width: u32,
    height: u32,
) {
    debug_assert_eq!(pixels.len(), (width * height) as usize);
    for (dst, &src) in staging.chunks_exact_mut(4).zip(pixels.iter()) {
        // PPU: 0xFF_RR_GG_BB  →  to_le_bytes() = [BB, GG, RR, FF]
        let [b, g, r, a] = src.to_le_bytes();
        dst[0] = r;
        dst[1] = g;
        dst[2] = b;
        dst[3] = a;
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        staging,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

// ── FrameRenderer ─────────────────────────────────────────────────────────────

/// Blits a CPU BGRA framebuffer to a winit window surface each frame.
pub struct FrameRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    game_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    gpu_renderer: GpuFrameRenderer,
    _gpu_texture: wgpu::Texture,
    gpu_view: wgpu::TextureView,
    gpu_bind_group: wgpu::BindGroup,
    upload_buf: Vec<u8>,
    game_width: u32,
    game_height: u32,
    scale_mode: ViewportScaleMode,
    viewport: Viewport,
    log_viewport: bool,
}

impl FrameRenderer {
    pub async fn new(window: Arc<Window>, game_width: u32, game_height: u32) -> Self {
        let instance = create_wgpu_instance();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let (adapter, device, queue) = create_device_queue(&instance, Some(&surface)).await;

        let caps = surface.get_capabilities(&adapter);

        // Prefer a non-sRGB format so SNES palette values aren't double-gamma'd.
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                !f.is_srgb()
                    && matches!(
                        f,
                        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                    )
            })
            .unwrap_or(caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (game_texture, bind_group_layout, bind_group) =
            create_game_texture_resources(&device, game_width, game_height);
        let pipeline = create_blit_pipeline(&device, &bind_group_layout, surface_format);
        let gpu_renderer = GpuFrameRenderer::new(&device);
        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gpu_game_frame"),
            size: wgpu::Extent3d {
                width: game_width,
                height: game_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let gpu_view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let gpu_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("gpu_nearest"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let gpu_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu_blit"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gpu_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&gpu_sampler),
                },
            ],
        });
        let scale_mode = ViewportScaleMode::from_env();
        let viewport = compute_viewport(
            config.width,
            config.height,
            game_width,
            game_height,
            scale_mode,
        );
        let upload_buf = vec![0u8; (game_width * game_height * 4) as usize];

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            game_texture,
            bind_group,
            gpu_renderer,
            _gpu_texture: gpu_texture,
            gpu_view,
            gpu_bind_group,
            upload_buf,
            game_width,
            game_height,
            scale_mode,
            viewport,
            log_viewport: env::var_os("ZELDA3_RENDER_VIEWPORT_LOG").is_some(),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.viewport = compute_viewport(
            new_size.width,
            new_size.height,
            self.game_width,
            self.game_height,
            self.scale_mode,
        );
    }

    fn maybe_log_viewport(&mut self) {
        if !self.log_viewport {
            return;
        }
        eprintln!(
            "renderer viewport: mode={:?} surface={}x{} game={}x{} viewport={:.1},{:.1} {:.1}x{:.1}",
            self.scale_mode,
            self.config.width,
            self.config.height,
            self.game_width,
            self.game_height,
            self.viewport.x,
            self.viewport.y,
            self.viewport.w,
            self.viewport.h
        );
        self.log_viewport = false;
    }

    /// Upload one frame of pixels. `pixels` must be `game_width * game_height`
    /// packed `u32` values in PPU format `0xFF_RR_GG_BB`.
    pub fn upload_frame(&mut self, pixels: &[u32]) {
        upload_ppu_pixels(
            &self.queue,
            &self.game_texture,
            pixels,
            &mut self.upload_buf,
            self.game_width,
            self.game_height,
        );
    }

    pub fn render(&mut self) -> Result<(), RenderError> {
        self.maybe_log_viewport();
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(RenderError::SurfaceReconfigureNeeded);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RenderError::Fatal(
                    "wgpu validation error in get_current_texture".to_string(),
                ));
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("blit"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_viewport(
                self.viewport.x,
                self.viewport.y,
                self.viewport.w,
                self.viewport.h,
                0.0,
                1.0,
            );
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        surface_texture.present();
        Ok(())
    }

    pub fn render_gpu_frame(&mut self, frame: &GpuFrame<'_>) -> Result<(), RenderError> {
        self.maybe_log_viewport();
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(RenderError::SurfaceReconfigureNeeded);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RenderError::Fatal(
                    "wgpu validation error in get_current_texture".to_string(),
                ));
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gpu_frame_surface"),
            });
        self.gpu_renderer
            .render_frame(&mut encoder, &self.queue, frame, &self.gpu_view);
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gpu_frame_blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.gpu_bind_group, &[]);
            pass.set_viewport(
                self.viewport.x,
                self.viewport.y,
                self.viewport.w,
                self.viewport.h,
                0.0,
                1.0,
            );
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        surface_texture.present();
        Ok(())
    }
}

// ── OffscreenRenderer ─────────────────────────────────────────────────────────

/// Headless renderer for pixel readback.
///
/// Renders the same blit pipeline as [`FrameRenderer`] but targets an offscreen
/// `Rgba8Unorm` texture instead of a window surface, then copies pixels back to
/// CPU memory. Used by the binary's `--render-hash-log` and `--dump-frame` paths
/// so those can eventually run through the GPU tile renderer without needing a
/// display.
pub struct OffscreenRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    game_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    /// GPU tile renderer — used by [`Self::render_gpu_frame`].
    gpu_renderer: GpuFrameRenderer,
    /// Output target: RENDER_ATTACHMENT | COPY_SRC, exact game resolution.
    render_texture: wgpu::Texture,
    /// Cached view of `render_texture` — shared by both render paths.
    render_view: wgpu::TextureView,
    /// MAP_READ buffer that receives the copy of `render_texture` each frame.
    readback_buf: wgpu::Buffer,
    /// Aligned row pitch (multiple of COPY_BYTES_PER_ROW_ALIGNMENT = 256).
    readback_bytes_per_row: u32,
    upload_buf: Vec<u8>,
    game_width: u32,
    game_height: u32,
}

impl OffscreenRenderer {
    pub async fn new(game_width: u32, game_height: u32) -> Self {
        let instance = create_wgpu_instance();
        // No surface compatibility needed — any adapter works for offscreen rendering.
        let (_adapter, device, queue) = create_device_queue(&instance, None).await;

        let (game_texture, bind_group_layout, bind_group) =
            create_game_texture_resources(&device, game_width, game_height);

        // Output format matches game_texture (Rgba8Unorm) — no conversion in the shader.
        let pipeline =
            create_blit_pipeline(&device, &bind_group_layout, wgpu::TextureFormat::Rgba8Unorm);

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen"),
            size: wgpu::Extent3d {
                width: game_width,
                height: game_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let gpu_renderer = GpuFrameRenderer::new(&device);

        // copy_texture_to_buffer requires bytes_per_row to be a multiple of
        // COPY_BYTES_PER_ROW_ALIGNMENT (256). For game_width=256: 256*4=1024, already aligned.
        let readback_bytes_per_row =
            (game_width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

        let readback_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (readback_bytes_per_row * game_height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let upload_buf = vec![0u8; (game_width * game_height * 4) as usize];

        Self {
            device,
            queue,
            pipeline,
            game_texture,
            bind_group,
            gpu_renderer,
            render_texture,
            render_view,
            readback_buf,
            readback_bytes_per_row,
            upload_buf,
            game_width,
            game_height,
        }
    }

    /// Upload one frame of pixels. Same format as [`FrameRenderer::upload_frame`].
    pub fn upload_frame(&mut self, pixels: &[u32]) {
        upload_ppu_pixels(
            &self.queue,
            &self.game_texture,
            pixels,
            &mut self.upload_buf,
            self.game_width,
            self.game_height,
        );
    }

    /// Upload one frame from a BGRA byte slice (the native output of `zelda_draw_display_frame`).
    ///
    /// BGRA layout: `[B, G, R, A]` per pixel. Swaps to RGBA for the `Rgba8Unorm` texture.
    /// No allocation — uses the pre-allocated staging buffer.
    pub fn upload_bgra_frame(&mut self, bgra: &[u8]) {
        debug_assert_eq!(
            bgra.len(),
            (self.game_width * self.game_height * 4) as usize,
        );
        for (dst, src) in self
            .upload_buf
            .chunks_exact_mut(4)
            .zip(bgra.chunks_exact(4))
        {
            dst[0] = src[2]; // BGRA[2] = R → RGBA[0]
            dst[1] = src[1]; // G unchanged
            dst[2] = src[0]; // BGRA[0] = B → RGBA[2]
            dst[3] = src[3]; // A unchanged
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.game_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.upload_buf,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.game_width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: self.game_width,
                height: self.game_height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Blit the current CPU PPU frame offscreen and read back RGBA bytes.
    ///
    /// Returns exactly `game_width * game_height * 4` bytes, row-major
    /// top-to-bottom. Blocks until the GPU completes the readback.
    pub fn render_to_rgba(&mut self) -> Vec<u8> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("offscreen"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("offscreen"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            // No letterboxing — headless output is exact game resolution.
            pass.set_viewport(
                0.0,
                0.0,
                self.game_width as f32,
                self.game_height as f32,
                0.0,
                1.0,
            );
            pass.draw(0..3, 0..1);
        }

        self.finish_and_readback(encoder)
    }

    /// Render one frame using the GPU tile pipeline and read back RGBA bytes.
    ///
    /// Returns exactly `game_width * game_height * 4` bytes, row-major
    /// top-to-bottom. Blocks until the GPU completes the readback.
    pub fn render_gpu_frame(&mut self, frame: &GpuFrame<'_>) -> Vec<u8> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gpu_frame"),
            });
        self.gpu_renderer
            .render_frame(&mut encoder, &self.queue, frame, &self.render_view);
        self.finish_and_readback(encoder)
    }

    /// Append a texture→buffer copy to `encoder`, submit, and block on readback.
    fn finish_and_readback(&mut self, mut encoder: wgpu::CommandEncoder) -> Vec<u8> {
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback_buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.readback_bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: self.game_width,
                height: self.game_height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit([encoder.finish()]);

        // map_async registers intent; poll(Wait) blocks until the GPU is idle and
        // the callback fires, after which get_mapped_range is valid.
        let slice = self.readback_buf.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during offscreen readback");

        let row_bytes = (self.game_width * 4) as usize;
        let stride = self.readback_bytes_per_row as usize;
        let mapped = slice.get_mapped_range();
        let mut out = Vec::with_capacity(row_bytes * self.game_height as usize);
        for row in 0..self.game_height as usize {
            out.extend_from_slice(&mapped[row * stride..row * stride + row_bytes]);
        }
        drop(mapped);
        self.readback_buf.unmap();
        out
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.05,
            "expected {actual} to be near {expected}"
        );
    }

    #[test]
    fn viewport_exact_fit() {
        let vp = compute_viewport(768, 672, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.w, 768.0);
        assert_eq!(vp.h, 672.0);
    }

    #[test]
    fn viewport_letterbox_wide() {
        // scale = min(1000/256, 672/224) = min(3, 3) = 3; x-bar = (1000-768)/2 = 116
        let vp = compute_viewport(1000, 672, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.x, 116.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.w, 768.0);
        assert_eq!(vp.h, 672.0);
    }

    #[test]
    fn viewport_letterbox_tall() {
        // scale = 3; y-bar = (800-672)/2 = 64
        let vp = compute_viewport(768, 800, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 64.0);
        assert_eq!(vp.w, 768.0);
        assert_eq!(vp.h, 672.0);
    }

    #[test]
    fn viewport_scale_one_when_surface_smaller_than_game() {
        // scale clamps to 1; viewport is clamped to surface size, offset is 0
        let vp = compute_viewport(100, 100, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.w, 100.0);
        assert_eq!(vp.h, 100.0);
    }

    #[test]
    fn viewport_scale_limited_by_height() {
        // scale = min(1280/256, 720/224) = min(5, 3) = 3
        let vp = compute_viewport(1280, 720, 256, 224, ViewportScaleMode::Integer);
        assert_eq!(vp.w, 768.0);
        assert_eq!(vp.h, 672.0);
    }

    #[test]
    fn viewport_fit_fills_steam_deck_height_without_cropping() {
        let vp = compute_viewport(1280, 800, 256, 224, ViewportScaleMode::Fit);
        assert_near(vp.x, 182.86);
        assert_eq!(vp.y, 0.0);
        assert_near(vp.w, 914.29);
        assert_eq!(vp.h, 800.0);
    }

    #[test]
    fn viewport_stretch_uses_full_surface() {
        let vp = compute_viewport(1280, 800, 256, 224, ViewportScaleMode::Stretch);
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.w, 1280.0);
        assert_eq!(vp.h, 800.0);
    }

    #[test]
    fn upload_frame_swaps_bgr_to_rgb() {
        // PPU pixel 0xFF_10_20_30 (R=0x10, G=0x20, B=0x30)
        // to_le_bytes → [0x30, 0x20, 0x10, 0xFF]
        // after swap  → [0x10, 0x20, 0x30, 0xFF] (RGBA for Rgba8Unorm)
        let pixels = [0xFF_10_20_30u32];
        let mut buf = vec![0u8; 4];
        for (dst, &src) in buf.chunks_exact_mut(4).zip(pixels.iter()) {
            let [b, g, r, a] = src.to_le_bytes();
            dst[0] = r;
            dst[1] = g;
            dst[2] = b;
            dst[3] = a;
        }
        assert_eq!(buf, [0x10, 0x20, 0x30, 0xFF]);
    }

    #[test]
    fn offscreen_readback_row_alignment() {
        // 256px wide: 256*4=1024, already a multiple of 256
        assert_eq!(
            (256u32 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            1024
        );
        // 100px wide: 100*4=400, next multiple of 256 is 512
        assert_eq!(
            (100u32 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            512
        );
        // 64px wide: 64*4=256, already aligned
        assert_eq!(
            (64u32 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            256
        );
    }
}
