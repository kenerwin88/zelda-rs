use crate::modern_frame::{ModernFrame, MODERN_FRAME_HEIGHT, MODERN_FRAME_WIDTH};

fn u32s_to_le_bytes(words: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn frame_windows_to_words(frame: &ModernFrame) -> Vec<u32> {
    (0..usize::from(MODERN_FRAME_HEIGHT))
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
    pub(crate) main_buffer: wgpu::Buffer,
    pub(crate) sub_buffer: wgpu::Buffer,
    window_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    out_buffer: wgpu::Buffer,
    startup_overlay_buffer: wgpu::Buffer,
    startup_direct_pixels_buffer: wgpu::Buffer,
    startup_material_pipeline: wgpu::ComputePipeline,
    startup_material_bind_group: wgpu::BindGroup,
}

impl ModernGpuFinalizer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
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
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pixel_count = u64::from(MODERN_FRAME_WIDTH) * u64::from(MODERN_FRAME_HEIGHT);
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
            size: u64::from(MODERN_FRAME_HEIGHT) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_params"),
            size: 16 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let out_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_out"),
            size: screen_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let startup_overlay_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_startup_overlay"),
            size: 128 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let startup_direct_pixels_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_finalize_startup_direct_pixels"),
            size: 512 * 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: startup_overlay_buffer.as_entire_binding(),
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
        let startup_material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("modern_startup_material"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
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
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let startup_material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("modern_startup_material"),
            layout: &startup_material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: out_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: startup_direct_pixels_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });
        let startup_material_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("modern_startup_material"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("modern_startup_material.wgsl").into(),
            ),
        });
        let startup_material_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("modern_startup_material"),
                bind_group_layouts: &[Some(&startup_material_bind_group_layout)],
                immediate_size: 0,
            });
        let startup_material_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("modern_startup_material"),
                layout: Some(&startup_material_pipeline_layout),
                module: &startup_material_shader,
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
            startup_overlay_buffer,
            startup_direct_pixels_buffer,
            startup_material_pipeline,
            startup_material_bind_group,
        }
    }

    pub(crate) fn render_current_buffers_to_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        len: u32,
        width: u32,
        scale: u32,
        output_texture: &wgpu::Texture,
    ) {
        debug_assert!(len <= u32::from(MODERN_FRAME_WIDTH) * u32::from(MODERN_FRAME_HEIGHT));
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
        let (startup_origin0, startup_origin1, startup_overlay, direct_pixels, direct_pixel_count) = match frame.hardware_startup_transient.as_ref() {
            Some(transient) => {
                let origin = |(x, y): (i16, i16)| {
                    0x8000_0000 | u32::from(x.max(0) as u16) | (u32::from(y.max(0) as u16) << 8)
                };
                let pixels = transient.rgba.map(u32::from_le_bytes);
                let mut overlays = [0; 128];
                overlays[..64].copy_from_slice(&pixels);
                overlays[64..].copy_from_slice(&pixels);
                let mut direct_pixels = [[0; 4]; 512];
                for (out, pixel) in direct_pixels.iter_mut().zip(&transient.direct_pixels) {
                    *out = [
                        pixel.screen_x.max(0) as u32,
                        pixel.screen_y.max(0) as u32,
                        u32::from_le_bytes(pixel.rgba),
                        0,
                    ];
                }
                (
                    origin(transient.origins[0]),
                    origin(transient.origins[1]),
                    overlays,
                    direct_pixels,
                    transient.direct_pixels.len().min(512) as u32,
                )
            }
            None => (0, 0, [0; 128], [[0; 4]; 512], 0),
        };
        let params = [
            len,
            width,
            scale,
            u32::from(frame.brightness),
            u32::from(frame.math_enabled),
            flags,
            fixed,
            u32::from(frame.clip_mode),
            u32::from(frame.prevent_math_mode),
            u32::from(frame.windowsel_cm),
            u32::from(frame.forced_blank_scanlines),
            startup_origin0,
            startup_origin1,
            direct_pixel_count,
            0,
            0,
        ];

        queue.write_buffer(&self.window_buffer, 0, &u32s_to_le_bytes(&windows));
        queue.write_buffer(&self.params_buffer, 0, &u32s_to_le_bytes(&params));
        queue.write_buffer(
            &self.startup_overlay_buffer,
            0,
            &u32s_to_le_bytes(&startup_overlay),
        );
        queue.write_buffer(
            &self.startup_direct_pixels_buffer,
            0,
            &u32s_to_le_bytes(&direct_pixels.concat()),
        );

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
        if direct_pixel_count != 0 {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("modern_startup_material"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.startup_material_pipeline);
            pass.set_bind_group(0, &self.startup_material_bind_group, &[]);
            pass.dispatch_workgroups(direct_pixel_count.div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &self.out_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
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
                width,
                height: len / width,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
    }
}
