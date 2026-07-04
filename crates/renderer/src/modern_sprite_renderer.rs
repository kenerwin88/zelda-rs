use crate::modern_frame::ModernFrame;
use crate::modern_index_atlas::ModernIndexTile;
use crate::modern_index_renderer::{build_index_atlas, INDEX_GRID_COLS};

/// Per-instance stride for the sprite (OBJ) path (little-endian):
///   offset  0: cell_origin_x, cell_origin_y (2 x u32, Uint32x2) atlas grid origin
///   offset  8: screen_x, screen_y           (2 x i32, Sint32x2)
///   offset 16: palette                       (u32, Uint32)
///   offset 20: flags                         (u32, Uint32) bit0=hflip bit1=vflip
const SPRITE_INSTANCE_STRIDE: u64 = 24;

/// GPU renderer for the palette-index OBJ (sprite) path. Uploads the SPRITE
/// index atlas as an `R8Uint` grid (cells stored UNFLIPPED) and draws sprites
/// OVER an existing BG render (`LoadOp::Load`, no clear). Output is byte-for-byte
/// identical to [`crate::modern_software::draw_modern_sprites_indexed`].
pub(crate) struct ModernGpuSpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ModernGpuSpriteRenderer {
    /// Build the sprite renderer's persistent pipeline. Per-frame cells are
    /// uploaded into an `R8Uint` grid texture by [`Self::render`].
    pub(crate) fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        // Same bind group layout as the BG index path: sprite atlas (Uint) at
        // binding 2, CGRAM (Float) at binding 3.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("modern_sprite"),
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
            label: Some("modern_sprite"),
            source: wgpu::ShaderSource::Wgsl(include_str!("modern_bg.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("modern_sprite"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: SPRITE_INSTANCE_STRIDE,
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
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32, // flags
                    offset: 20,
                    shader_location: 3,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("modern_sprite"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_sprite"),
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_sprite"),
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

    /// Draw `frame.index_sprites` OVER the existing contents of `output_view`
    /// (a 256x224 `Rgba8Unorm` target that already holds the BG render). The
    /// target is LOADED, not cleared. Sprites are emitted in REVERSE OAM order so
    /// the earliest OAM sprite is drawn last and wins (REPLACE), matching
    /// [`crate::modern_software::draw_modern_sprites_indexed`]. Index 0 is
    /// transparent; color = `cgram_rgba[0x80 + palette*16 + index]`.
    pub(crate) fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cells: &[ModernIndexTile],
        frame: &ModernFrame,
        output_view: &wgpu::TextureView,
    ) {
        let (_sprite_atlas_texture, sprite_atlas_view) =
            build_index_atlas(device, queue, cells, "modern_sprite_atlas");

        let mut instance_bytes: Vec<u8> = Vec::new();
        let mut instance_count: u32 = 0;
        if !frame.forced_blank {
            // Reverse OAM order -> earliest OAM sprite drawn last (on top).
            for inst in frame.index_sprites.iter().rev() {
                if inst.cell_id as usize >= cells.len() {
                    continue; // software's `cells.get(..)` returns None -> skip
                }
                let col = inst.cell_id % INDEX_GRID_COLS;
                let row = inst.cell_id / INDEX_GRID_COLS;
                instance_bytes.extend_from_slice(&(col * 8).to_le_bytes());
                instance_bytes.extend_from_slice(&(row * 8).to_le_bytes());
                instance_bytes.extend_from_slice(&(i32::from(inst.screen_x)).to_le_bytes());
                instance_bytes.extend_from_slice(&(i32::from(inst.screen_y)).to_le_bytes());
                instance_bytes.extend_from_slice(&(u32::from(inst.palette)).to_le_bytes());
                let mut flags = 0u32;
                if inst.hflip {
                    flags |= 0b001;
                }
                if inst.vflip {
                    flags |= 0b010;
                }
                instance_bytes.extend_from_slice(&flags.to_le_bytes());
                instance_count += 1;
            }
        }
        debug_assert_eq!(
            instance_bytes.len() as u64,
            u64::from(instance_count) * SPRITE_INSTANCE_STRIDE
        );

        if instance_count == 0 {
            return; // nothing to composite; leave the BG render untouched
        }

        // CGRAM texture (256x1 Rgba8Unorm) — full palette incl. OBJ half 128..255.
        let mut cgram_bytes = Vec::with_capacity(256 * 4);
        for px in &frame.cgram_rgba {
            cgram_bytes.extend_from_slice(px);
        }
        let cgram_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_sprite_cgram"),
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
            label: Some("modern_sprite"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&sprite_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&cgram_view),
                },
            ],
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_sprite_instances"),
            size: instance_bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&instance_buffer, 0, &instance_bytes);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("modern_sprite"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("modern_sprite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Load the BG render; sprites composite over it.
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, instance_buffer.slice(..));
            pass.draw(0..6, 0..instance_count);
        }
        queue.submit([encoder.finish()]);
    }
}
