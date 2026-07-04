use crate::modern_frame::ModernFrame;
use crate::modern_index_atlas::ModernIndexTile;

/// Number of 8x8 cells per row in the index-atlas grid texture.
pub(crate) const INDEX_GRID_COLS: u32 = 64;
/// Per-instance stride for the index path (little-endian):
///   offset  0: cell_origin_x, cell_origin_y (2 x u32, Uint32x2) atlas grid origin
///   offset  8: screen_x, screen_y           (2 x i32, Sint32x2)
///   offset 16: palette                       (u32, Uint32)
///   offset 20: padding                       (u32)
pub(crate) const INDEX_INSTANCE_STRIDE: u64 = 24;

pub(crate) fn build_index_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cells: &[ModernIndexTile],
    label: &'static str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let cell_count = cells.len() as u32;
    let grid_rows = cell_count.div_ceil(INDEX_GRID_COLS).max(1);
    let tex_width = INDEX_GRID_COLS * 8;
    let tex_height = grid_rows * 8;

    let mut data = vec![0u8; (tex_width * tex_height) as usize];
    for cell in cells {
        let col = cell.id % INDEX_GRID_COLS;
        let row = cell.id / INDEX_GRID_COLS;
        let ox = col * 8;
        let oy = row * 8;
        for ly in 0..8u32 {
            for lx in 0..8u32 {
                let px = (oy + ly) * tex_width + (ox + lx);
                data[px as usize] = cell.indices[(ly * 8 + lx) as usize];
            }
        }
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: tex_width,
            height: tex_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(tex_width),
            rows_per_image: Some(tex_height),
        },
        wgpu::Extent3d {
            width: tex_width,
            height: tex_height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// GPU renderer for the palette-index path: an `R8Uint` atlas of 8x8 index
/// cells + the live CGRAM as a 256x1 `Rgba8Unorm` texture. Produces output
/// byte-for-byte identical to [`crate::modern_software::render_modern_frame_software_indexed`].
pub(crate) struct ModernGpuIndexRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ModernGpuIndexRenderer {
    /// Build the index renderer's persistent pipeline. Per-frame cells are
    /// uploaded into an `R8Uint` grid texture by [`Self::render`].
    pub(crate) fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        // Bind group: index atlas (Uint) at binding 2, CGRAM (Float) at binding 3
        // — matching the `@binding` slots in `modern_bg.wgsl`'s index path.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("modern_index"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Uint,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("modern_bg_index"),
            source: wgpu::ShaderSource::Wgsl(include_str!("modern_bg.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("modern_index"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: INDEX_INSTANCE_STRIDE,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32x2, // cell_origin
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32x2, // screen_xy
                    offset: 8,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32, // palette
                    offset: 16,
                    shader_location: 2,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("modern_index"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_index"),
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_index"),
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
            bind_group_layout,
        }
    }

    /// Render `frame`'s enabled main BG layers' index tiles into `output_view`
    /// (a 256x224 `Rgba8Unorm` render target). Semantics match
    /// [`crate::modern_software::render_modern_frame_software_indexed`]: clear to
    /// the backdrop (opaque black if `forced_blank`), then paint index tiles in
    /// `bg_layers` order, each layer's tiles in push order (painter's algorithm —
    /// REPLACE, no blending); index 0 is transparent (discarded); the final color
    /// is `cgram_rgba[palette*16 + index]`.
    pub(crate) fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cells: &[ModernIndexTile],
        frame: &ModernFrame,
        output_view: &wgpu::TextureView,
    ) {
        let (_index_atlas_texture, index_atlas_view) =
            build_index_atlas(device, queue, cells, "modern_index_atlas");

        // Build the per-tile instance buffer in draw order.
        let mut instance_bytes: Vec<u8> = Vec::new();
        let mut instance_count: u32 = 0;
        if !frame.forced_blank {
            for layer in &frame.bg_layers {
                if !layer.enabled_main {
                    continue;
                }
                for inst in &layer.index_tiles {
                    if inst.cell_id as usize >= cells.len() {
                        continue; // software's `atlas.cells.get(..)` returns None → skip
                    }
                    let col = inst.cell_id % INDEX_GRID_COLS;
                    let row = inst.cell_id / INDEX_GRID_COLS;
                    instance_bytes.extend_from_slice(&(col * 8).to_le_bytes());
                    instance_bytes.extend_from_slice(&(row * 8).to_le_bytes());
                    instance_bytes.extend_from_slice(&(i32::from(inst.screen_x)).to_le_bytes());
                    instance_bytes.extend_from_slice(&(i32::from(inst.screen_y)).to_le_bytes());
                    instance_bytes.extend_from_slice(&(u32::from(inst.palette)).to_le_bytes());
                    instance_bytes.extend_from_slice(&0u32.to_le_bytes()); // padding
                    instance_count += 1;
                }
            }
        }
        debug_assert_eq!(
            instance_bytes.len() as u64,
            u64::from(instance_count) * INDEX_INSTANCE_STRIDE
        );

        // CGRAM texture (256x1 Rgba8Unorm). Rgba8Unorm round-trips bytes exactly,
        // so `textureLoad` -> output is byte-identical to the software lookup.
        let mut cgram_bytes = Vec::with_capacity(256 * 4);
        for px in &frame.cgram_rgba {
            cgram_bytes.extend_from_slice(px);
        }
        let cgram_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_index_cgram"),
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
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
                texture: &cgram_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &cgram_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let cgram_view = cgram_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("modern_index"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&index_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&cgram_view),
                },
            ],
        });

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
                label: Some("modern_index_instances"),
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
            label: Some("modern_index"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("modern_index"),
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
                pass.set_bind_group(0, &bind_group, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..6, 0..instance_count);
            }
        }
        queue.submit([encoder.finish()]);
    }
}
