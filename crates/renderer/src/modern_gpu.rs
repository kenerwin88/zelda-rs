use crate::modern_assets::ModernTileAtlasAsset;
use crate::modern_frame::ModernFrame;

/// Packed per-instance data uploaded as an instance-step vertex buffer.
///
/// Layout (40 bytes, little-endian) — kept in sync with the vertex attributes in
/// `pipeline` and the `@location` inputs in `modern_bg.wgsl`:
///   offset  0: atlas_x, atlas_y, atlas_w, atlas_h  (4 x u32, Uint32x4) SOURCE rect
///   offset 16: screen_x, screen_y                  (2 x i32, Sint32x2)
///   offset 24: screen_w, screen_h                  (2 x u32, Uint32x2) on-screen footprint
///   offset 32: flags                               (u32, Uint32) bit0=hflip bit1=vflip bit2=transparent
///   offset 36: padding                             (u32)
const INSTANCE_STRIDE: u64 = 40;

pub struct ModernGpuRenderer {
    pipeline: wgpu::RenderPipeline,
    // Held to keep the GPU resources backing `bind_group` alive for the
    // renderer's lifetime; not read directly after construction.
    #[allow(dead_code)]
    atlas_texture: wgpu::Texture,
    #[allow(dead_code)]
    atlas_view: wgpu::TextureView,
    #[allow(dead_code)]
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

impl ModernGpuRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &ModernTileAtlasAsset,
        format: wgpu::TextureFormat,
    ) -> Self {
        // ── Atlas texture ─────────────────────────────────────────────────────
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_bg_atlas"),
            size: wgpu::Extent3d {
                width: atlas.width_px,
                height: atlas.height_px,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas.width_px * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: atlas.width_px,
                height: atlas.height_px,
                depth_or_array_layers: 1,
            },
        );

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ── Sampler ───────────────────────────────────────────────────────────
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("modern_bg_nearest"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // ── Atlas bind group (for Task 9; not wired into the placeholder pipeline) ──
        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("modern_bg_atlas"),
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
            label: Some("modern_bg_atlas"),
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // ── Instanced tile pipeline (atlas bind group + per-instance vertex buffer) ──
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("modern_bg"),
            source: wgpu::ShaderSource::Wgsl(include_str!("modern_bg.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("modern_bg"),
            bind_group_layouts: &[Some(&atlas_bind_group_layout)],
            immediate_size: 0,
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: INSTANCE_STRIDE,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32x4, // atlas_xywh
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32x2, // screen_xy
                    offset: 16,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32x2, // screen_wh
                    offset: 24,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32, // flags
                    offset: 32,
                    shader_location: 3,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("modern_bg"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
        });

        Self {
            pipeline,
            atlas_texture,
            atlas_view,
            sampler,
            bind_group,
        }
    }

    /// Render `frame`'s enabled main BG layers into `output_view` (a 256x224
    /// `Rgba8Unorm` render target with `RENDER_ATTACHMENT` usage).
    ///
    /// Semantics match [`crate::modern_software::render_modern_frame_software`]
    /// byte-for-byte: clear to the backdrop (or opaque black if `forced_blank`),
    /// then paint tiles in `bg_layers` order, each layer's tiles in push order
    /// (painter's algorithm — REPLACE, no blending). Out-of-bounds atlas texels
    /// and (when `transparent_color_zero`) alpha-zero texels are discarded.
    ///
    /// `device` is taken so a per-call instance buffer can be created; the work
    /// is submitted to `queue` before returning (readback is the caller's job).
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        output_view: &wgpu::TextureView,
    ) {
        // Build the packed instance buffer in draw order (painter's algorithm).
        let mut instance_bytes: Vec<u8> = Vec::new();
        let mut instance_count: u32 = 0;
        if !frame.forced_blank {
            for layer in &frame.bg_layers {
                if !layer.enabled_main {
                    continue;
                }
                for tile in &layer.tiles {
                    if tile.atlas_width_px == 0
                        || tile.atlas_height_px == 0
                        || tile.screen_width_px == 0
                        || tile.screen_height_px == 0
                    {
                        continue; // degenerate quad — software loops produce nothing
                    }
                    instance_bytes.extend_from_slice(&(u32::from(tile.atlas_x_px)).to_le_bytes());
                    instance_bytes.extend_from_slice(&(u32::from(tile.atlas_y_px)).to_le_bytes());
                    instance_bytes
                        .extend_from_slice(&(u32::from(tile.atlas_width_px)).to_le_bytes());
                    instance_bytes
                        .extend_from_slice(&(u32::from(tile.atlas_height_px)).to_le_bytes());
                    instance_bytes.extend_from_slice(&(i32::from(tile.screen_x)).to_le_bytes());
                    instance_bytes.extend_from_slice(&(i32::from(tile.screen_y)).to_le_bytes());
                    instance_bytes
                        .extend_from_slice(&(u32::from(tile.screen_width_px)).to_le_bytes());
                    instance_bytes
                        .extend_from_slice(&(u32::from(tile.screen_height_px)).to_le_bytes());
                    let mut flags = 0u32;
                    if tile.hflip {
                        flags |= 0b001;
                    }
                    if tile.vflip {
                        flags |= 0b010;
                    }
                    if tile.transparent_color_zero {
                        flags |= 0b100;
                    }
                    instance_bytes.extend_from_slice(&flags.to_le_bytes());
                    instance_bytes.extend_from_slice(&0u32.to_le_bytes()); // padding
                    instance_count += 1;
                }
            }
        }
        debug_assert_eq!(
            instance_bytes.len() as u64,
            u64::from(instance_count) * INSTANCE_STRIDE
        );

        // Clear color = backdrop (opaque black when forced blank). Rgba8Unorm is
        // not sRGB, so `b/255` round-trips back to byte `b` exactly.
        let backdrop = if frame.forced_blank {
            [0u8, 0, 0, 0xff]
        } else {
            frame.backdrop_color_rgba
        };
        let clear = wgpu::Color {
            r: f64::from(backdrop[0]) / 255.0,
            g: f64::from(backdrop[1]) / 255.0,
            b: f64::from(backdrop[2]) / 255.0,
            a: f64::from(backdrop[3]) / 255.0,
        };

        let instance_buffer = if instance_count > 0 {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_bg_instances"),
                size: instance_bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&buffer, 0, &instance_bytes);
            Some(buffer)
        } else {
            None
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("modern_bg"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("modern_bg"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some(buffer) = &instance_buffer {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..6, 0..instance_count);
            }
        }
        queue.submit([encoder.finish()]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::{ModernBgLayer, ModernFrame, ModernTileInstance};
    use crate::modern_software::render_modern_frame_software;

    #[test]
    fn modern_gpu_one_tile_matches_software() {
        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            // 8x8 atlas: one solid red opaque tile.
            let atlas_rgba: Vec<u8> = {
                let mut v = vec![0u8; 8 * 8 * 4];
                for px in v.chunks_exact_mut(4) {
                    px.copy_from_slice(&[255, 0, 0, 0xff]);
                }
                v
            };
            let atlas_asset = ModernTileAtlasAsset {
                tile_width_px: 8,
                tile_height_px: 8,
                atlas_scale: 1,
                width_px: 8,
                height_px: 8,
                rgba: atlas_rgba.clone(),
                entries: Vec::new(),
            };

            // One tile at (0,0) on enabled main layer 0, opaque (no transparency).
            let mut frame = ModernFrame::empty();
            let mut layer = ModernBgLayer::new(0);
            layer.enabled_main = true;
            layer.tiles.push(ModernTileInstance {
                atlas_id: 0,
                atlas_x_px: 0,
                atlas_y_px: 0,
                atlas_width_px: 8,
                atlas_height_px: 8,
                screen_width_px: 8,
                screen_height_px: 8,
                screen_x: 0,
                screen_y: 0,
                palette: 0,
                priority: 0,
                hflip: false,
                vflip: false,
                transparent_color_zero: false,
            });
            frame.bg_layers[0] = layer;

            let renderer = ModernGpuRenderer::new(
                &device,
                &queue,
                &atlas_asset,
                wgpu::TextureFormat::Rgba8Unorm,
            );

            // Render target: exact game resolution, readable back.
            let width = 256u32;
            let height = 224u32;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_test_target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());

            renderer.render(&device, &queue, &frame, &view);

            // Read back the rendered pixels. 256*4 = 1024 is already a multiple of
            // COPY_BYTES_PER_ROW_ALIGNMENT (256), so there is no row padding.
            let bytes_per_row = width * 4;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_gpu_test_readback"),
                size: (bytes_per_row * height) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &target,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);

            let slice = readback.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("GPU poll failed during readback");
            let mapped = slice.get_mapped_range();
            let gpu_rgba = mapped.to_vec();
            drop(mapped);
            readback.unmap();

            let software_rgba = render_modern_frame_software(&frame, &atlas_rgba, 8, 8);

            assert_eq!(gpu_rgba, software_rgba);
        });
    }

    #[test]
    fn modern_gpu_downsampled_tile_matches_software() {
        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            // 32x32 atlas = 4x nearest upscale of a pixel-distinct 8x8 pattern.
            const SCALE: usize = 4;
            const SRC: usize = 8 * SCALE;
            let pattern = |x: usize, y: usize| -> [u8; 4] {
                [(x as u8) * 30 + 5, (y as u8) * 30 + 7, 100, 0xff]
            };
            let mut atlas_rgba = vec![0u8; SRC * SRC * 4];
            for ay in 0..SRC {
                for ax in 0..SRC {
                    let px = pattern(ax / SCALE, ay / SCALE);
                    let o = (ay * SRC + ax) * 4;
                    atlas_rgba[o..o + 4].copy_from_slice(&px);
                }
            }
            let atlas_asset = ModernTileAtlasAsset {
                tile_width_px: 8,
                tile_height_px: 8,
                atlas_scale: SCALE as u16,
                width_px: SRC as u32,
                height_px: SRC as u32,
                rgba: atlas_rgba.clone(),
                entries: Vec::new(),
            };

            // One tile: 32x32 source rect downsampled into an 8x8 footprint.
            let mut frame = ModernFrame::empty();
            let mut layer = ModernBgLayer::new(0);
            layer.enabled_main = true;
            layer.tiles.push(ModernTileInstance {
                atlas_id: 0,
                atlas_x_px: 0,
                atlas_y_px: 0,
                atlas_width_px: SRC as u16,
                atlas_height_px: SRC as u16,
                screen_width_px: 8,
                screen_height_px: 8,
                screen_x: 0,
                screen_y: 0,
                palette: 0,
                priority: 0,
                hflip: false,
                vflip: false,
                transparent_color_zero: false,
            });
            frame.bg_layers[0] = layer;

            let renderer = ModernGpuRenderer::new(
                &device,
                &queue,
                &atlas_asset,
                wgpu::TextureFormat::Rgba8Unorm,
            );

            let width = 256u32;
            let height = 224u32;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_downsample_test_target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());

            renderer.render(&device, &queue, &frame, &view);

            let bytes_per_row = width * 4;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_gpu_downsample_test_readback"),
                size: (bytes_per_row * height) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &target,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);

            let slice = readback.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("GPU poll failed during readback");
            let mapped = slice.get_mapped_range();
            let gpu_rgba = mapped.to_vec();
            drop(mapped);
            readback.unmap();

            let software_rgba =
                render_modern_frame_software(&frame, &atlas_rgba, SRC as u16, SRC as u16);

            assert_eq!(gpu_rgba, software_rgba);
        });
    }

    #[test]
    fn modern_gpu_renderer_constructs() {
        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let atlas = crate::modern_assets::load_modern_overworld_tile_atlas(&root)
                .expect("atlas should load");

            let _renderer =
                ModernGpuRenderer::new(&device, &queue, &atlas, wgpu::TextureFormat::Rgba8Unorm);
        });
    }
}
