use crate::modern_assets::ModernTileAtlasAsset;
use crate::modern_frame::ModernFrame;
use crate::modern_index_atlas::ModernIndexTile;
use std::cell::RefCell;

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
        self.render_with_load_op(
            device,
            queue,
            frame,
            output_view,
            modern_frame_clear_op(frame),
        );
    }

    fn render_overlay(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        output_view: &wgpu::TextureView,
    ) {
        self.render_with_load_op(device, queue, frame, output_view, wgpu::LoadOp::Load);
    }

    fn render_with_load_op(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        output_view: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
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
                        load,
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

fn modern_frame_clear_op(frame: &ModernFrame) -> wgpu::LoadOp<wgpu::Color> {
    // Clear color = backdrop (opaque black when forced blank). Rgba8Unorm is not
    // sRGB, so `b/255` round-trips back to byte `b` exactly.
    let backdrop = if frame.forced_blank {
        [0u8, 0, 0, 0xff]
    } else {
        frame.backdrop_color_rgba
    };
    wgpu::LoadOp::Clear(wgpu::Color {
        r: f64::from(backdrop[0]) / 255.0,
        g: f64::from(backdrop[1]) / 255.0,
        b: f64::from(backdrop[2]) / 255.0,
        a: f64::from(backdrop[3]) / 255.0,
    })
}

pub struct ModernGpuVariantRenderer {
    atlas: crate::modern_variant_atlas::ModernVariantAtlas,
    renderer: ModernGpuRenderer,
    effect_renderer: ModernGpuVariantEffectRenderer,
}

fn debug_variant_missing_key(key: &crate::modern_variant_atlas::VariantAtlasKey) {
    static PRINTED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let Ok(limit) = std::env::var("ZELDA3_VARIANT_DEBUG_MISSING") else {
        return;
    };
    let limit = limit.parse::<usize>().unwrap_or(16);
    let printed = PRINTED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if printed < limit {
        eprintln!(
            "variant_missing source_kind={} asset={} pack={} tile={} bpp={} palette={} row={}",
            key.source_kind, key.asset, key.pack, key.tile, key.bpp, key.palette, key.palette_row
        );
    }
}

impl ModernGpuVariantRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        format: wgpu::TextureFormat,
    ) -> Self {
        let atlas_asset = ModernTileAtlasAsset {
            tile_width_px: 8,
            tile_height_px: 8,
            atlas_scale: 1,
            width_px: atlas.width,
            height_px: atlas.height,
            rgba: atlas.rgba.clone(),
            entries: Vec::new(),
        };
        Self {
            atlas: atlas.clone(),
            renderer: ModernGpuRenderer::new(device, queue, &atlas_asset, format),
            effect_renderer: ModernGpuVariantEffectRenderer::new(device, queue, atlas, format),
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
        output_view: &wgpu::TextureView,
    ) -> crate::modern_software::VariantAtlasRenderStats {
        let (variant_frame, stats) = self.build_variant_frame(
            frame,
            bg_cells,
            sprite_cells,
            bg_palette_name,
            sprite_palette_name,
        );
        if stats.fallback_draws == 0 && stats.effect_draws == stats.stable_draws {
            self.effect_renderer.render_bg(
                device,
                queue,
                frame,
                bg_cells,
                &self.atlas,
                bg_palette_name,
                output_view,
                modern_frame_clear_op(frame),
            );
            self.effect_renderer.render_sprites(
                device,
                queue,
                frame,
                sprite_cells,
                &self.atlas,
                sprite_palette_name,
                output_view,
            );
            return stats;
        }
        if stats.fallback_draws != 0 {
            let bg = ModernGpuIndexRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
            let spr = ModernGpuSpriteRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
            bg.render(device, queue, bg_cells, frame, output_view);
            spr.render(device, queue, sprite_cells, frame, output_view);
            if stats.effect_draws != 0 {
                self.effect_renderer.render_bg(
                    device,
                    queue,
                    frame,
                    bg_cells,
                    &self.atlas,
                    bg_palette_name,
                    output_view,
                    wgpu::LoadOp::Load,
                );
                self.effect_renderer.render_sprites(
                    device,
                    queue,
                    frame,
                    sprite_cells,
                    &self.atlas,
                    sprite_palette_name,
                    output_view,
                );
            }
            if stats.stable_draws != stats.effect_draws {
                self.renderer
                    .render_overlay(device, queue, &variant_frame, output_view);
            }
        } else if stats.effect_draws != 0 {
            self.effect_renderer.render_bg(
                device,
                queue,
                frame,
                bg_cells,
                &self.atlas,
                bg_palette_name,
                output_view,
                modern_frame_clear_op(frame),
            );
            self.effect_renderer.render_sprites(
                device,
                queue,
                frame,
                sprite_cells,
                &self.atlas,
                sprite_palette_name,
                output_view,
            );
            if stats.stable_draws != stats.effect_draws {
                self.renderer
                    .render_overlay(device, queue, &variant_frame, output_view);
            }
        } else {
            self.renderer
                .render(device, queue, &variant_frame, output_view);
        }
        stats
    }

    fn build_variant_frame(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> (ModernFrame, crate::modern_software::VariantAtlasRenderStats) {
        let mut out = ModernFrame::empty();
        out.backdrop_color_rgba = frame.backdrop_color_rgba;
        out.forced_blank = frame.forced_blank;
        let mut stats = crate::modern_software::VariantAtlasRenderStats::default();

        if frame.forced_blank {
            return (out, stats);
        }

        out.bg_layers[0].enabled_main = true;
        for layer in &frame.bg_layers {
            if !layer.enabled_main {
                continue;
            }
            for inst in &layer.index_tiles {
                let Some(cell) = bg_cells.get(inst.cell_id as usize) else {
                    continue;
                };
                let key = crate::modern_variant_atlas::variant_key_for_index_tile(
                    cell,
                    bg_palette_name,
                    inst.palette,
                );
                let draw = self.atlas.resolve_draw(key.as_ref());
                stats.record_draw(&draw);
                match draw {
                    crate::modern_variant_atlas::VariantAtlasDraw::Stable { entry, effect } => {
                        if effect.is_none() {
                            out.bg_layers[0].tiles.push(variant_tile_instance(
                                entry,
                                inst.screen_x,
                                inst.screen_y,
                                cell.hflip ^ entry.source_hflip,
                                cell.vflip ^ entry.source_vflip,
                            ));
                        }
                    }
                    crate::modern_variant_atlas::VariantAtlasDraw::DynamicPalette { .. } => {}
                    crate::modern_variant_atlas::VariantAtlasDraw::MissingArt => {
                        if let Some(key) = key.as_ref() {
                            debug_variant_missing_key(key);
                        }
                    }
                    crate::modern_variant_atlas::VariantAtlasDraw::Unkeyed => {}
                }
            }
        }

        out.bg_layers[1].enabled_main = true;
        for inst in frame.index_sprites.iter().rev() {
            let Some(cell) = sprite_cells.get(inst.cell_id as usize) else {
                continue;
            };
            let key = crate::modern_variant_atlas::variant_key_for_index_tile(
                cell,
                sprite_palette_name,
                inst.palette,
            );
            let draw = self.atlas.resolve_draw(key.as_ref());
            stats.record_draw(&draw);
            match draw {
                crate::modern_variant_atlas::VariantAtlasDraw::Stable { entry, effect } => {
                    if effect.is_none() {
                        out.bg_layers[1].tiles.push(variant_tile_instance(
                            entry,
                            inst.screen_x,
                            inst.screen_y,
                            inst.hflip ^ entry.source_hflip,
                            inst.vflip ^ entry.source_vflip,
                        ));
                    }
                }
                crate::modern_variant_atlas::VariantAtlasDraw::DynamicPalette { .. } => {}
                crate::modern_variant_atlas::VariantAtlasDraw::MissingArt => {
                    if let Some(key) = key.as_ref() {
                        debug_variant_missing_key(key);
                    }
                }
                crate::modern_variant_atlas::VariantAtlasDraw::Unkeyed => {}
            }
        }

        (out, stats)
    }
}

struct ModernGpuVariantEffectRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    effect_lut_texture: wgpu::Texture,
    effect_lut_view: wgpu::TextureView,
}

impl ModernGpuVariantEffectRenderer {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        format: wgpu::TextureFormat,
    ) -> Self {
        let effect_rows = atlas.effects.len().max(1) as u32;
        let effect_lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_variant_effect_lut"),
            size: wgpu::Extent3d {
                width: EFFECT_LUT_WIDTH,
                height: effect_rows,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut lut_bytes = vec![0u8; (EFFECT_LUT_WIDTH * effect_rows * 4) as usize];
        for (row, effect) in atlas.effects.iter().enumerate() {
            for (index, color) in effect
                .index_to_rgba
                .iter()
                .enumerate()
                .take(EFFECT_LUT_WIDTH as usize)
            {
                let offset = (row * EFFECT_LUT_WIDTH as usize + index) * 4;
                lut_bytes[offset..offset + 4].copy_from_slice(color);
            }
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &effect_lut_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &lut_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(EFFECT_LUT_WIDTH * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: EFFECT_LUT_WIDTH,
                height: effect_rows,
                depth_or_array_layers: 1,
            },
        );
        let effect_lut_view =
            effect_lut_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("modern_variant_effect"),
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
                    binding: 4,
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
            label: Some("modern_variant_effect"),
            source: wgpu::ShaderSource::Wgsl(include_str!("modern_bg.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("modern_variant_effect"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: INDEX_INSTANCE_STRIDE,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32x2,
                    offset: 8,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 16,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 20,
                    shader_location: 3,
                },
            ],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("modern_variant_effect"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_effect"),
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_effect"),
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
            effect_lut_texture,
            effect_lut_view,
        }
    }

    fn render_bg(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        bg_palette_name: &str,
        output_view: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
    ) {
        let (_index_atlas_texture, index_atlas_view) =
            build_index_atlas(device, queue, bg_cells, "modern_variant_effect_index_atlas");
        let mut instance_bytes = Vec::new();
        let mut instance_count = 0u32;
        if !frame.forced_blank {
            for layer in &frame.bg_layers {
                if !layer.enabled_main {
                    continue;
                }
                for inst in &layer.index_tiles {
                    let Some(cell) = bg_cells.get(inst.cell_id as usize) else {
                        continue;
                    };
                    let Some(key) = crate::modern_variant_atlas::variant_key_for_index_tile(
                        cell,
                        bg_palette_name,
                        inst.palette,
                    ) else {
                        continue;
                    };
                    let (entry, effect) = match atlas.resolve_draw(Some(&key)) {
                        crate::modern_variant_atlas::VariantAtlasDraw::Stable {
                            entry,
                            effect: Some(effect),
                        } => (entry, effect),
                        _ => continue,
                    };
                    let Some(effect_row) = atlas.effect_row_for_effect(effect) else {
                        continue;
                    };
                    let col = inst.cell_id % INDEX_GRID_COLS;
                    let row = inst.cell_id / INDEX_GRID_COLS;
                    instance_bytes.extend_from_slice(&(col * 8).to_le_bytes());
                    instance_bytes.extend_from_slice(&(row * 8).to_le_bytes());
                    instance_bytes.extend_from_slice(&(i32::from(inst.screen_x)).to_le_bytes());
                    instance_bytes.extend_from_slice(&(i32::from(inst.screen_y)).to_le_bytes());
                    let mut flags = 0xffu32 << 8;
                    if cell.hflip ^ entry.source_hflip {
                        flags |= 0b001;
                    }
                    if cell.vflip ^ entry.source_vflip {
                        flags |= 0b010;
                    }
                    instance_bytes.extend_from_slice(&flags.to_le_bytes());
                    instance_bytes.extend_from_slice(&effect_row.to_le_bytes());
                    instance_count += 1;
                }
            }
        }
        debug_assert_eq!(
            instance_bytes.len() as u64,
            u64::from(instance_count) * INDEX_INSTANCE_STRIDE
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("modern_variant_effect"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&index_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.effect_lut_view),
                },
            ],
        });
        let instance_buffer = if instance_count > 0 {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_variant_effect_instances"),
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
            label: Some("modern_variant_effect"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("modern_variant_effect"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load,
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

    fn render_sprites(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        sprite_cells: &[ModernIndexTile],
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        sprite_palette_name: &str,
        output_view: &wgpu::TextureView,
    ) {
        let mut instance_bytes = Vec::new();
        let mut instance_count = 0u32;
        if !frame.forced_blank {
            for inst in frame.index_sprites.iter().rev() {
                let Some(cell) = sprite_cells.get(inst.cell_id as usize) else {
                    continue;
                };
                let Some(key) = crate::modern_variant_atlas::variant_key_for_index_tile(
                    cell,
                    sprite_palette_name,
                    inst.palette,
                ) else {
                    continue;
                };
                let (entry, effect) = match atlas.resolve_draw(Some(&key)) {
                    crate::modern_variant_atlas::VariantAtlasDraw::Stable {
                        entry,
                        effect: Some(effect),
                    } => (entry, effect),
                    _ => continue,
                };
                let Some(effect_row) = atlas.effect_row_for_effect(effect) else {
                    continue;
                };
                let col = inst.cell_id % INDEX_GRID_COLS;
                let row = inst.cell_id / INDEX_GRID_COLS;
                instance_bytes.extend_from_slice(&(col * 8).to_le_bytes());
                instance_bytes.extend_from_slice(&(row * 8).to_le_bytes());
                instance_bytes.extend_from_slice(&(i32::from(inst.screen_x)).to_le_bytes());
                instance_bytes.extend_from_slice(&(i32::from(inst.screen_y)).to_le_bytes());
                let mut flags = u32::from(inst.row_mask) << 8;
                if inst.hflip ^ entry.source_hflip {
                    flags |= 0b001;
                }
                if inst.vflip ^ entry.source_vflip {
                    flags |= 0b010;
                }
                instance_bytes.extend_from_slice(&flags.to_le_bytes());
                instance_bytes.extend_from_slice(&effect_row.to_le_bytes());
                instance_count += 1;
            }
        }
        debug_assert_eq!(
            instance_bytes.len() as u64,
            u64::from(instance_count) * INDEX_INSTANCE_STRIDE
        );
        if instance_count == 0 {
            return;
        }

        let (_index_atlas_texture, index_atlas_view) = build_index_atlas(
            device,
            queue,
            sprite_cells,
            "modern_variant_effect_sprite_index_atlas",
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("modern_variant_effect_sprite"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&index_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.effect_lut_view),
                },
            ],
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_variant_effect_sprite_instances"),
            size: instance_bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&instance_buffer, 0, &instance_bytes);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("modern_variant_effect_sprite"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("modern_variant_effect_sprite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
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

fn variant_tile_instance(
    entry: &crate::modern_variant_atlas::VariantAtlasEntry,
    screen_x: i16,
    screen_y: i16,
    hflip: bool,
    vflip: bool,
) -> crate::modern_frame::ModernTileInstance {
    crate::modern_frame::ModernTileInstance {
        atlas_id: 0,
        atlas_x_px: entry.rect[0] as u16,
        atlas_y_px: entry.rect[1] as u16,
        atlas_width_px: entry.rect[2] as u16,
        atlas_height_px: entry.rect[3] as u16,
        screen_width_px: 8,
        screen_height_px: 8,
        screen_x,
        screen_y,
        palette: 0,
        priority: 0,
        hflip,
        vflip,
        transparent_color_zero: true,
    }
}

/// Number of 8x8 cells per row in the index-atlas grid texture.
const INDEX_GRID_COLS: u32 = 64;
/// Per-instance stride for the index path (little-endian):
///   offset  0: cell_origin_x, cell_origin_y (2 x u32, Uint32x2) atlas grid origin
///   offset  8: screen_x, screen_y           (2 x i32, Sint32x2)
///   offset 16: palette                       (u32, Uint32)
///   offset 20: padding                       (u32)
const INDEX_INSTANCE_STRIDE: u64 = 24;
const EFFECT_LUT_WIDTH: u32 = 16;

fn build_index_atlas(
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
pub struct ModernGpuIndexRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ModernGpuIndexRenderer {
    /// Build the index renderer's persistent pipeline. Per-frame cells are
    /// uploaded into an `R8Uint` grid texture by [`Self::render`].
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
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
    pub fn render(
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
pub struct ModernGpuSpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ModernGpuSpriteRenderer {
    /// Build the sprite renderer's persistent pipeline. Per-frame cells are
    /// uploaded into an `R8Uint` grid texture by [`Self::render`].
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
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
    pub fn render(
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
            // Reverse OAM order → earliest OAM sprite drawn last (on top).
            for inst in frame.index_sprites.iter().rev() {
                if inst.cell_id as usize >= cells.len() {
                    continue; // software's `cells.get(..)` returns None → skip
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
                        // Load the BG render — sprites composite OVER it.
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

fn u32s_to_le_bytes(words: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn frame_windows_to_words(frame: &ModernFrame) -> Vec<u32> {
    (0..usize::from(crate::modern_frame::MODERN_FRAME_HEIGHT))
        .map(|win| {
            let win = frame.window_scanlines.get(win).copied().unwrap_or([0u8; 4]);
            u32::from(win[0])
                | (u32::from(win[1]) << 8)
                | (u32::from(win[2]) << 16)
                | (u32::from(win[3]) << 24)
        })
        .collect()
}

pub(crate) struct ModernGpuFinalizer {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    main_buffer: wgpu::Buffer,
    sub_buffer: wgpu::Buffer,
    window_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    out_buffer: wgpu::Buffer,
}

impl ModernGpuFinalizer {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("modern_finalize"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pixel_count = u64::from(crate::modern_frame::MODERN_FRAME_WIDTH)
            * u64::from(crate::modern_frame::MODERN_FRAME_HEIGHT);
        let screen_bytes = pixel_count * 4;
        let main_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_main"),
            size: screen_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sub_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_sub"),
            size: screen_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let window_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_windows"),
            size: u64::from(crate::modern_frame::MODERN_FRAME_HEIGHT) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_params"),
            size: 12 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let out_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_out"),
            size: screen_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("modern_finalize"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: main_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sub_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: window_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: out_buffer.as_entire_binding(),
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("modern_finalize"),
            source: wgpu::ShaderSource::Wgsl(include_str!("modern_finalize.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("modern_finalize"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("modern_finalize"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            pipeline,
            bind_group,
            main_buffer,
            sub_buffer,
            window_buffer,
            params_buffer,
            out_buffer,
        }
    }

    pub(crate) fn render_to_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        screens: &crate::modern_software::ModernCompositedScreens,
        output_texture: &wgpu::Texture,
    ) {
        let len = screens.main.len() as u32;
        debug_assert_eq!(screens.sub.len(), screens.main.len());
        debug_assert!(
            len <= u32::from(crate::modern_frame::MODERN_FRAME_WIDTH)
                * u32::from(crate::modern_frame::MODERN_FRAME_HEIGHT)
        );
        let windows = frame_windows_to_words(frame);
        let rendered_subscreen = (frame.screen_enabled_sub & 0x1f) != 0;
        let no_effect_math = frame.fixed_color_r == 0
            && frame.fixed_color_g == 0
            && frame.fixed_color_b == 0
            && !frame.half_color
            && !rendered_subscreen;
        let mut flags = 0u32;
        if frame.subtract_color {
            flags |= 0x1;
        }
        if frame.half_color {
            flags |= 0x2;
        }
        if frame.add_subscreen {
            flags |= 0x4;
        }
        if no_effect_math {
            flags |= 0x8;
        }
        if frame.forced_blank {
            flags |= 0x10;
        }
        let fixed = u32::from(frame.fixed_color_r)
            | (u32::from(frame.fixed_color_g) << 8)
            | (u32::from(frame.fixed_color_b) << 16);
        let params = [
            len,
            screens.width as u32,
            screens.scale as u32,
            u32::from(frame.brightness),
            u32::from(frame.math_enabled),
            flags,
            fixed,
            u32::from(frame.clip_mode),
            u32::from(frame.prevent_math_mode),
            u32::from(frame.windowsel_cm),
            0,
            0,
        ];

        queue.write_buffer(&self.main_buffer, 0, &u32s_to_le_bytes(&screens.main));
        queue.write_buffer(&self.sub_buffer, 0, &u32s_to_le_bytes(&screens.sub));
        queue.write_buffer(&self.window_buffer, 0, &u32s_to_le_bytes(&windows));
        queue.write_buffer(&self.params_buffer, 0, &u32s_to_le_bytes(&params));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("modern_finalize"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("modern_finalize"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(len.div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &self.out_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some((screens.width * 4) as u32),
                    rows_per_image: None,
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: screens.width as u32,
                height: (len / screens.width as u32),
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
    }
}

/// GPU finalizer compositor for the PNG index-atlas path. The Mode-1 priority
/// MAIN/SUB screens are built through the same packed intermediate as the
/// byte-exact CPU renderer; the final color-math, windows, and master brightness
/// resolve runs as a compute pass into the caller's `Rgba8Unorm` texture.
pub struct ModernGpuCompositor {
    finalizer: ModernGpuFinalizer,
}

impl ModernGpuCompositor {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, _format: wgpu::TextureFormat) -> Self {
        Self {
            finalizer: ModernGpuFinalizer::new(device),
        }
    }

    /// Build the packed MAIN/SUB screens and resolve the final RGBA through the
    /// GPU finalizer into `output_texture`.
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        output_texture: &wgpu::Texture,
    ) {
        let screens =
            crate::modern_software::build_modern_composited_screens(frame, bg_cells, sprite_cells);
        self.finalizer
            .render_to_texture(device, queue, frame, &screens, output_texture);
    }
}

/// Owns a headless wgpu device + a 256x224 offscreen `Rgba8Unorm` target + the
/// compositor. Construct once and reuse; device creation is expensive.
pub struct ModernGpuHeadless {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compositor: ModernGpuCompositor,
    gpu_frame_renderer: RefCell<crate::gpu_renderer::GpuFrameRenderer>,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
}

impl ModernGpuHeadless {
    pub fn new() -> Self {
        let instance = crate::create_wgpu_instance();
        let (_adapter, device, queue) =
            pollster::block_on(crate::create_device_queue(&instance, None));
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let compositor = ModernGpuCompositor::new(&device, &queue, format);
        let gpu_frame_renderer = RefCell::new(crate::gpu_renderer::GpuFrameRenderer::new(
            &device, &queue, None,
        ));
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_gpu_headless_target"),
            size: wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            device,
            queue,
            compositor,
            gpu_frame_renderer,
            target,
            target_view,
        }
    }

    pub fn render_rgba(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
    ) -> Vec<u8> {
        self.compositor.render(
            &self.device,
            &self.queue,
            frame,
            bg_cells,
            sprite_cells,
            &self.target,
        );

        self.read_target_rgba()
    }

    pub fn render_mode7_rgba(&self, frame: &crate::gpu_frame::GpuFrame<'_>) -> Vec<u8> {
        debug_assert_eq!(frame.mode, 7);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("modern_gpu_mode7"),
            });
        self.gpu_frame_renderer.borrow_mut().render_frame(
            &mut encoder,
            &self.queue,
            frame,
            &self.target_view,
        );
        self.queue.submit([encoder.finish()]);
        self.read_target_rgba()
    }

    fn read_target_rgba(&self) -> Vec<u8> {
        let (width, height) = (256u32, 224u32);
        let bytes_per_row = width * 4;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_gpu_headless_readback"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.target,
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
        self.queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during readback");
        let mapped = slice.get_mapped_range();
        let out = mapped.to_vec();
        drop(mapped);
        readback.unmap();
        out
    }
}

pub struct ModernGpuVariantHeadless {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compositor: ModernGpuCompositor,
    renderer: ModernGpuVariantRenderer,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
}

impl ModernGpuVariantHeadless {
    pub fn new(atlas: &crate::modern_variant_atlas::ModernVariantAtlas) -> Self {
        let instance = crate::create_wgpu_instance();
        let (_adapter, device, queue) =
            pollster::block_on(crate::create_device_queue(&instance, None));
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let compositor = ModernGpuCompositor::new(&device, &queue, format);
        let renderer = ModernGpuVariantRenderer::new(&device, &queue, atlas, format);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_gpu_variant_headless_target"),
            size: wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            device,
            queue,
            compositor,
            renderer,
            target,
            target_view,
        }
    }

    pub fn render_rgba(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> (Vec<u8>, crate::modern_software::VariantAtlasRenderStats) {
        self.render_rgba_with_fallback(
            frame,
            bg_cells,
            sprite_cells,
            frame,
            bg_cells,
            sprite_cells,
            bg_palette_name,
            sprite_palette_name,
        )
    }

    pub fn render_rgba_with_fallback(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        fallback_frame: &ModernFrame,
        fallback_bg_cells: &[ModernIndexTile],
        fallback_sprite_cells: &[ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> (Vec<u8>, crate::modern_software::VariantAtlasRenderStats) {
        let (variant_frame, stats) = self.renderer.build_variant_frame(
            frame,
            bg_cells,
            sprite_cells,
            bg_palette_name,
            sprite_palette_name,
        );
        if stats.fallback_draws != 0 {
            self.compositor.render(
                &self.device,
                &self.queue,
                fallback_frame,
                fallback_bg_cells,
                fallback_sprite_cells,
                &self.target,
            );
            if stats.effect_draws != 0 {
                self.renderer.effect_renderer.render_bg(
                    &self.device,
                    &self.queue,
                    frame,
                    bg_cells,
                    &self.renderer.atlas,
                    bg_palette_name,
                    &self.target_view,
                    wgpu::LoadOp::Load,
                );
                self.renderer.effect_renderer.render_sprites(
                    &self.device,
                    &self.queue,
                    frame,
                    sprite_cells,
                    &self.renderer.atlas,
                    sprite_palette_name,
                    &self.target_view,
                );
            }
            if stats.stable_draws != stats.effect_draws {
                self.renderer.renderer.render_overlay(
                    &self.device,
                    &self.queue,
                    &variant_frame,
                    &self.target_view,
                );
            }
        } else if stats.effect_draws != 0 {
            self.renderer.effect_renderer.render_bg(
                &self.device,
                &self.queue,
                frame,
                bg_cells,
                &self.renderer.atlas,
                bg_palette_name,
                &self.target_view,
                modern_frame_clear_op(frame),
            );
            self.renderer.effect_renderer.render_sprites(
                &self.device,
                &self.queue,
                frame,
                sprite_cells,
                &self.renderer.atlas,
                sprite_palette_name,
                &self.target_view,
            );
            if stats.stable_draws != stats.effect_draws {
                self.renderer.renderer.render_overlay(
                    &self.device,
                    &self.queue,
                    &variant_frame,
                    &self.target_view,
                );
            }
        } else {
            self.renderer.renderer.render(
                &self.device,
                &self.queue,
                &variant_frame,
                &self.target_view,
            );
        }
        (self.read_target_rgba(), stats)
    }

    fn read_target_rgba(&self) -> Vec<u8> {
        let (width, height) = (256u32, 224u32);
        let bytes_per_row = width * 4;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_gpu_variant_headless_readback"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.target,
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
        self.queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during variant readback");
        let mapped = slice.get_mapped_range();
        let out = mapped.to_vec();
        drop(mapped);
        readback.unmap();
        out
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

    #[test]
    fn modern_gpu_indexed_matches_software() {
        use crate::modern_frame::ModernIndexTileInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_software_indexed;

        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            // Same synthetic cells + frame as the Task 5 software test.
            let mut indices = [0u8; 64];
            indices[0] = 1; // pixel (0,0)
            indices[1] = 2; // pixel (1,0)
            let cells = vec![ModernIndexTile {
                id: 0,
                indices,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }];

            let mut frame = ModernFrame::empty();
            frame.backdrop_color_rgba = [0, 0, 0, 0xff];
            frame.cgram_rgba[3 * 16 + 1] = [10, 20, 30, 0xff];
            frame.cgram_rgba[3 * 16 + 2] = [40, 50, 60, 0xff];

            let mut layer = ModernBgLayer::new(0);
            layer.enabled_main = true;
            layer.index_tiles.push(ModernIndexTileInstance {
                cell_id: 0,
                screen_x: 0,
                screen_y: 0,
                palette: 3,
                hflip: false,
                vflip: false,
                priority: false,
            });
            frame.bg_layers[0] = layer;

            let renderer =
                ModernGpuIndexRenderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm);

            let width = 256u32;
            let height = 224u32;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_index_test_target"),
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

            renderer.render(&device, &queue, &cells, &frame, &view);

            let bytes_per_row = width * 4;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_gpu_index_test_readback"),
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

            let software_rgba = render_modern_frame_software_indexed(&frame, &cells);

            assert_eq!(gpu_rgba.len(), software_rgba.len());
            assert_eq!(gpu_rgba, software_rgba);
        });
    }

    #[test]
    fn modern_gpu_compositor_matches_full_software_basic_bg_obj() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_full;

        let mut a = [0u8; 64];
        a[0] = 1;
        a[9] = 2;
        let mut b = [0u8; 64];
        b[63] = 3;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: a,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: b,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
        ];

        let mut s0 = [0u8; 64];
        s0[0] = 4;
        let mut s1 = [0u8; 64];
        s1[7] = 5;
        let sprite_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: s0,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: s1,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
        ];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[3 * 16 + 1] = [10, 20, 30, 0xff];
        frame.cgram_rgba[3 * 16 + 2] = [40, 50, 60, 0xff];
        frame.cgram_rgba[3 * 16 + 3] = [70, 80, 90, 0xff];
        frame.cgram_rgba[0x80 + 16 + 4] = [100, 110, 120, 0xff];
        frame.cgram_rgba[0x80 + 16 + 5] = [130, 140, 150, 0xff];

        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 3,
            hflip: false,
            vflip: false,
            priority: false,
        });
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            screen_x: 16,
            screen_y: 8,
            palette: 3,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 40,
            screen_y: 40,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 1,
            screen_x: 48,
            screen_y: 40,
            palette: 1,
            priority: 0,
            hflip: true,
            vflip: true,
            row_mask: 0xff,
        });
        frame.screen_enabled_main = 0x11; // BG1 + OBJ.
        frame.brightness = 15;

        let gpu = ModernGpuHeadless::new().render_rgba(&frame, &bg_cells, &sprite_cells);
        let software = render_modern_frame_full(&frame, &bg_cells, &sprite_cells);

        assert_eq!(gpu.len(), software.len());
        assert_eq!(
            gpu, software,
            "GPU compositor must match full CPU reference"
        );
    }

    #[test]
    fn modern_gpu_compositor_matches_full_software_color_math() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_full;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[1] = [20 << 3, 18 << 3, 16 << 3, 0xff];
        frame.cgram_rgba[16 + 1] = [10 << 3, 8 << 3, 6 << 3, 0xff];

        let mut main = ModernBgLayer::new(0);
        main.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main;

        let mut sub = ModernBgLayer::new(1);
        sub.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub;

        frame.screen_enabled_main = 0x01; // BG1.
        frame.screen_enabled_sub = 0x02; // BG2.
        frame.math_enabled = 0x01; // Math on winning BG1.
        frame.add_subscreen = true;
        frame.half_color = true;
        frame.brightness = 11;

        let gpu = ModernGpuHeadless::new().render_rgba(&frame, &cells, &[]);
        let software = render_modern_frame_full(&frame, &cells, &[]);

        assert_eq!(gpu.len(), software.len());
        assert_eq!(
            &gpu[0..4],
            &software[0..4],
            "first pixel exercises sub-screen half-add plus brightness"
        );
        assert_eq!(gpu, software);
    }

    #[test]
    fn modern_gpu_variant_headless_missing_tiles_fallback_matches_full_compositor() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[1] = [20 << 3, 18 << 3, 16 << 3, 0xff];
        frame.cgram_rgba[16 + 1] = [10 << 3, 8 << 3, 6 << 3, 0xff];

        let mut main = ModernBgLayer::new(0);
        main.enabled_main = true;
        main.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main;

        let mut sub = ModernBgLayer::new(1);
        sub.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub;

        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.half_color = true;
        frame.brightness = 11;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let full = ModernGpuHeadless::new().render_rgba(&frame, &cells, &[]);

        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.fallback_draws, 1);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.missing_variant_draws, 1);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 0);
        assert_eq!(stats.missing_art_draws, 1);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
        assert_eq!(variant, full);
    }

    #[test]
    fn modern_gpu_variant_headless_missing_tiles_use_live_fallback_cells() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut source_indices = [0u8; 64];
        source_indices[0] = 1;
        let source_cells = vec![ModernIndexTile {
            id: 0,
            indices: source_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut fallback_indices = [0u8; 64];
        fallback_indices[0] = 1;
        let fallback_cells = vec![ModernIndexTile {
            id: 0,
            indices: fallback_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut source_frame = ModernFrame::empty();
        source_frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        source_frame.cgram_rgba[1] = [200, 0, 0, 0xff];
        let mut source_layer = ModernBgLayer::new(0);
        source_layer.enabled_main = true;
        source_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_frame.bg_layers[0] = source_layer;

        let mut fallback_frame = ModernFrame::empty();
        fallback_frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        fallback_frame.cgram_rgba[1] = [0, 160, 80, 0xff];
        let mut fallback_layer = ModernBgLayer::new(0);
        fallback_layer.enabled_main = true;
        fallback_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        fallback_frame.bg_layers[0] = fallback_layer;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_fallback(
            &source_frame,
            &source_cells,
            &[],
            &fallback_frame,
            &fallback_cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let fallback = ModernGpuHeadless::new().render_rgba(&fallback_frame, &fallback_cells, &[]);

        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.fallback_draws, 1);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.missing_variant_draws, 1);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 0);
        assert_eq!(stats.missing_art_draws, 1);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
        assert_eq!(variant, fallback);
    }

    #[test]
    fn modern_gpu_variant_headless_mixed_fallback_uses_effect_overlay() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let mut missing_indices = [0u8; 64];
        missing_indices[0] = 1;
        let source_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: stable_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: missing_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let fallback_cells = source_cells.clone();

        let mut source_frame = ModernFrame::empty();
        source_frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        let mut source_layer = ModernBgLayer::new(0);
        source_layer.enabled_main = true;
        source_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            screen_x: 8,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_frame.bg_layers[0] = source_layer;

        let mut fallback_frame = source_frame.clone();
        fallback_frame.cgram_rgba[1] = [0, 160, 80, 0xff];

        let mut atlas_rgba = vec![0u8; 8 * 8 * 4];
        for px in atlas_rgba.chunks_exact_mut(4) {
            px.copy_from_slice(&[180, 20, 40, 0xff]);
        }
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: atlas_rgba,
            entries: vec![VariantAtlasEntry {
                id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                key: VariantAtlasKey {
                    source_kind: "bg".to_string(),
                    asset: "kBgGfx".to_string(),
                    pack: 0,
                    tile: 0,
                    bpp: 3,
                    palette: "palette_dung_bg_main".to_string(),
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [90, 100, 110, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_fallback(
            &source_frame,
            &source_cells,
            &[],
            &fallback_frame,
            &fallback_cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let fallback = ModernGpuHeadless::new().render_rgba(&fallback_frame, &fallback_cells, &[]);

        assert_eq!(stats.stable_draws, 1);
        assert_eq!(stats.effect_draws, 1);
        assert_eq!(stats.fallback_draws, 1);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.missing_variant_draws, 1);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 1);
        assert_eq!(stats.dynamic_material_draws, 0);
        assert_eq!(stats.missing_art_draws, 1);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
        assert_eq!(&variant[0..4], &[90, 100, 110, 0xff]);
        let missing_offset = 8 * 4;
        assert_eq!(
            &variant[missing_offset..missing_offset + 4],
            &fallback[missing_offset..missing_offset + 4]
        );
    }

    #[test]
    fn modern_gpu_variant_headless_unkeyed_tiles_are_fallback_not_missing() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[1] = [40, 80, 120, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (_variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.fallback_draws, 1);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.missing_variant_draws, 0);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 0);
        assert_eq!(stats.missing_art_draws, 0);
        assert_eq!(stats.unkeyed_fallback_draws, 1);
    }

    #[test]
    fn modern_gpu_mode7_matches_modern_software_oracle() {
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x0002; // tilemap entry (0,0): low byte = tile number 2
        vram[2 * 64 + 8] = 5u16 << 8; // tile 2, texel row1 col0: high byte = index 5
        let mut cgram = vec![0u16; 0x100];
        cgram[5] = 0x7c1f; // BGR555 magenta
        let oam = vec![0u16; 0x110];
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 7;
        frame.screen_enabled = [0x01, 0];
        for sl in frame.scanlines.iter_mut() {
            sl.mode7_matrix = [256, 0, 0, 256, 0, 0, 0, 0];
            sl.screen_enabled_main = 0x01;
        }

        let gpu = ModernGpuHeadless::new().render_mode7_rgba(&frame);
        let software = crate::modern_software::render_modern_mode7_frame(&frame);

        assert_eq!(gpu.len(), software.len());
        assert_eq!(gpu, software);
    }

    fn test_gpu_frame<'a>(
        vram: &'a [u16],
        cgram: &'a [u16],
        oam: &'a [u16],
        brightness: u8,
        forced_blank: bool,
    ) -> crate::gpu_frame::GpuFrame<'a> {
        crate::gpu_frame::GpuFrame {
            vram,
            cgram,
            oam,
            mode: 1,
            bg: Default::default(),
            obj: Default::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Default::default(),
            screen_enabled: [0, 0],
            screen_windowed: [0, 0],
            brightness,
            forced_blank,
            math_enabled: 0,
            subtract_color: false,
            half_color: false,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            add_subscreen: false,
            clip_mode: 0,
            prevent_math_mode: 0,
            windowsel_cm: 0,
            windowsel: 0,
            scanlines: Box::new([crate::gpu_frame::ScanlineRegs::default(); 224]),
        }
    }

    /// Render the BG index pass then the sprite pass on the GPU, reading back the
    /// composited 256x224 RGBA target.
    async fn gpu_bg_then_sprites(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
    ) -> Vec<u8> {
        let bg = ModernGpuIndexRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
        let spr = ModernGpuSpriteRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);

        let width = 256u32;
        let height = 224u32;
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_gpu_sprite_test_target"),
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

        bg.render(device, queue, bg_cells, frame, &view);
        spr.render(device, queue, sprite_cells, frame, &view);

        let bytes_per_row = width * 4;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_gpu_sprite_test_readback"),
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
        gpu_rgba
    }

    #[test]
    fn modern_gpu_variant_atlas_bg_tile_matches_software_variant() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_software_variant_atlas;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            let mut atlas_rgba = vec![0u8; 8 * 8 * 4];
            for px in atlas_rgba.chunks_exact_mut(4) {
                px.copy_from_slice(&[180, 20, 40, 0xff]);
            }
            let atlas = ModernVariantAtlas {
                width: 8,
                height: 8,
                rgba: atlas_rgba,
                entries: vec![
                    VariantAtlasEntry {
                        id: "bg:kBgGfx:pack0:tile0:3bpp:palette_dung_bg_main:row2".to_string(),
                        key: VariantAtlasKey {
                            source_kind: "bg".to_string(),
                            asset: "kBgGfx".to_string(),
                            pack: 0,
                            tile: 0,
                            bpp: 3,
                            palette: "palette_dung_bg_main".to_string(),
                            palette_row: 2,
                        },
                        rect: [0, 0, 8, 8],
                        sha1: "test".to_string(),
                        duplicate_of: None,
                        dynamic_policy: "stable".to_string(),
                        source_hflip: false,
                        source_vflip: false,
                    },
                    VariantAtlasEntry {
                        id: "sprite:kSprGfx:pack0:tile0:3bpp:palette_main_spr:row4".to_string(),
                        key: VariantAtlasKey {
                            source_kind: "sprite".to_string(),
                            asset: "kSprGfx".to_string(),
                            pack: 0,
                            tile: 0,
                            bpp: 3,
                            palette: "palette_main_spr".to_string(),
                            palette_row: 4,
                        },
                        rect: [0, 0, 8, 8],
                        sha1: "test-sprite".to_string(),
                        duplicate_of: None,
                        dynamic_policy: "stable".to_string(),
                        source_hflip: false,
                        source_vflip: false,
                    },
                ],
                effects: vec![
                    TileEffect {
                        id: "palette_dung_bg_main:8color:row2".to_string(),
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                        colors_per_row: 8,
                        index_to_rgba: vec![
                            [0, 0, 0, 0xff],
                            [90, 100, 110, 0xff],
                            [2, 2, 2, 0xff],
                            [3, 3, 3, 0xff],
                            [4, 4, 4, 0xff],
                            [5, 5, 5, 0xff],
                            [6, 6, 6, 0xff],
                            [7, 7, 7, 0xff],
                        ],
                        dynamic_policy: "stable".to_string(),
                    },
                    TileEffect {
                        id: "palette_main_spr:8color:row4".to_string(),
                        palette: "palette_main_spr".to_string(),
                        palette_row: 4,
                        colors_per_row: 8,
                        index_to_rgba: vec![
                            [0, 0, 0, 0xff],
                            [1, 1, 1, 0xff],
                            [2, 2, 2, 0xff],
                            [120, 130, 140, 0xff],
                            [4, 4, 4, 0xff],
                            [5, 5, 5, 0xff],
                            [6, 6, 6, 0xff],
                            [7, 7, 7, 0xff],
                        ],
                        dynamic_policy: "stable".to_string(),
                    },
                ],
            };

            let cells = vec![ModernIndexTile {
                id: 0,
                indices: [1u8; 64],
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            }];
            let mut sprite_indices = [0u8; 64];
            sprite_indices[0] = 3;
            let sprite_cells = vec![ModernIndexTile {
                id: 0,
                indices: sprite_indices,
                source_key: modern_source_key(2, 0, 0),
                hflip: false,
                vflip: false,
            }];
            let mut frame = ModernFrame::empty();
            frame.backdrop_color_rgba = [0, 0, 0, 0xff];
            let mut layer = ModernBgLayer::new(0);
            layer.enabled_main = true;
            layer.index_tiles.push(ModernIndexTileInstance {
                cell_id: 0,
                screen_x: 3,
                screen_y: 5,
                palette: 2,
                hflip: false,
                vflip: false,
                priority: false,
            });
            frame.bg_layers[0] = layer;
            frame
                .index_sprites
                .push(crate::modern_frame::ModernIndexSpriteInstance {
                    cell_id: 0,
                    screen_x: 0,
                    screen_y: 0,
                    palette: 4,
                    hflip: false,
                    vflip: false,
                    priority: 0,
                    row_mask: 0xff,
                });

            let renderer = ModernGpuVariantRenderer::new(
                &device,
                &queue,
                &atlas,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            let width = 256u32;
            let height = 224u32;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_variant_test_target"),
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

            let stats = renderer.render(
                &device,
                &queue,
                &frame,
                &cells,
                &sprite_cells,
                "palette_dung_bg_main",
                "palette_main_spr",
                &view,
            );

            let bytes_per_row = width * 4;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_gpu_variant_test_readback"),
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

            let (software_rgba, software_stats) = render_modern_frame_software_variant_atlas(
                &frame,
                &cells,
                &sprite_cells,
                &atlas,
                "palette_dung_bg_main",
                "palette_main_spr",
            );

            assert_eq!(stats.stable_draws, 2);
            assert_eq!(stats.effect_draws, 2);
            assert_eq!(stats.fallback_draws, 0);
            assert_eq!(stats, software_stats);
            if gpu_rgba != software_rgba {
                let mismatch = gpu_rgba
                    .chunks_exact(4)
                    .zip(software_rgba.chunks_exact(4))
                    .enumerate()
                    .find(|(_, (gpu, software))| gpu != software)
                    .expect("framebuffers differ but no pixel mismatch was found");
                let pixel = mismatch.0;
                panic!(
                    "first GPU/software variant mismatch at ({}, {}): gpu={:?} software={:?}",
                    pixel % width as usize,
                    pixel / width as usize,
                    mismatch.1 .0,
                    mismatch.1 .1
                );
            }
        });
    }

    #[test]
    fn modern_gpu_sprite_matches_software() {
        use crate::modern_frame::ModernIndexSpriteInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::{
            draw_modern_sprites_indexed, render_modern_frame_software_indexed,
        };

        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            // ── Case 1: no flip ────────────────────────────────────────────────
            let mut indices = [0u8; 64];
            indices[0] = 1; // pixel (0,0) → index 1
            let sprite_cells = vec![ModernIndexTile {
                id: 0,
                indices,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }];

            let mut frame = ModernFrame::empty();
            frame.backdrop_color_rgba = [0, 0, 0, 0xff];
            frame.cgram_rgba[0x80 + 3 * 16 + 1] = [200, 10, 20, 0xff];
            frame.index_sprites.push(ModernIndexSpriteInstance {
                cell_id: 0,
                screen_x: 5,
                screen_y: 7,
                palette: 3,
                priority: 0,
                hflip: false,
                vflip: false,
                row_mask: 0xff,
            });

            let gpu_rgba = gpu_bg_then_sprites(&device, &queue, &frame, &[], &sprite_cells).await;
            let mut software_rgba = render_modern_frame_software_indexed(&frame, &[]);
            draw_modern_sprites_indexed(&mut software_rgba, &frame, &sprite_cells);
            assert_eq!(gpu_rgba.len(), software_rgba.len());
            assert_eq!(gpu_rgba, software_rgba, "no-flip sprite gpu==software");

            // ── Case 2: hflip ──────────────────────────────────────────────────
            let mut indices2 = [0u8; 64];
            indices2[7] = 2; // pixel (7,0) → index 2; hflip lands it at screen x=0
            let sprite_cells2 = vec![ModernIndexTile {
                id: 0,
                indices: indices2,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }];

            let mut frame2 = ModernFrame::empty();
            frame2.backdrop_color_rgba = [0, 0, 0, 0xff];
            frame2.cgram_rgba[0x80 + 3 * 16 + 2] = [9, 99, 199, 0xff];
            frame2.index_sprites.push(ModernIndexSpriteInstance {
                cell_id: 0,
                screen_x: 5,
                screen_y: 7,
                palette: 3,
                priority: 0,
                hflip: true,
                vflip: false,
                row_mask: 0xff,
            });

            let gpu_rgba2 =
                gpu_bg_then_sprites(&device, &queue, &frame2, &[], &sprite_cells2).await;
            let mut software_rgba2 = render_modern_frame_software_indexed(&frame2, &[]);
            draw_modern_sprites_indexed(&mut software_rgba2, &frame2, &sprite_cells2);
            // The flipped pixel must land at screen_x (5,7).
            let px = (7usize * 256 + 5) * 4;
            assert_eq!(
                &software_rgba2[px..px + 4],
                &[9, 99, 199, 0xff],
                "hflip software pixel at x=5"
            );
            assert_eq!(gpu_rgba2, software_rgba2, "hflip sprite gpu==software");
        });
    }
}
