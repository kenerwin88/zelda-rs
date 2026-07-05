use crate::gpu_frame::{GpuFrame, RawScanlineFrame, ScanlineRegs};

const PARAMS_SIZE: u64 = 64;
const SCANLINE_SIZE: u64 = 56 * 16; // 56 vec4<u32>
const UNIFORM_SIZE: u64 = PARAMS_SIZE + SCANLINE_SIZE; // 960 bytes

pub struct PostProcessRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl PostProcessRenderer {
    /// Build the renderer.
    ///
    /// `comp_view` is the main BG+sprite composite; `sub_comp_view` is the sub-screen
    /// composite used as the second operand when `add_subscreen=true`. Both are baked
    /// into the bind group at construction.
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        comp_view: &wgpu::TextureView,
        sub_comp_view: &wgpu::TextureView,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post_process"),
            source: wgpu::ShaderSource::Wgsl(include_str!("post_process.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post_process"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(UNIFORM_SIZE),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post_process"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post_process"),
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
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post_process_nearest"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("post_process_uniforms"),
            size: UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post_process"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(comp_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(sub_comp_view),
                },
            ],
        });

        Self {
            pipeline,
            uniform_buf,
            bind_group,
        }
    }

    /// Upload color math uniforms and render the post-process pass.
    ///
    /// `output_view` is the final render target.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
    ) {
        queue.write_buffer(&self.uniform_buf, 0, &build_uniform_bytes(frame));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("post_process"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                depth_slice: None,
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
        pass.draw(0..3, 0..1);
    }
}

/// Serialize GpuFrame color math fields into the 960-byte uniform buffer layout.
fn build_uniform_bytes(frame: &GpuFrame<'_>) -> Vec<u8> {
    let mut buf = Vec::with_capacity(UNIFORM_SIZE as usize);

    macro_rules! pu32 {
        ($v:expr) => {
            buf.extend_from_slice(&($v as u32).to_ne_bytes())
        };
    }

    pu32!(frame.brightness);
    pu32!(frame.forced_blank);
    pu32!(frame.math_enabled);
    pu32!(frame.subtract_color);
    pu32!(frame.half_color);
    pu32!(frame.fixed_color_r);
    pu32!(frame.fixed_color_g);
    pu32!(frame.fixed_color_b);
    pu32!(frame.add_subscreen);
    pu32!(frame.clip_mode);
    pu32!(frame.prevent_math_mode);
    pu32!(frame.windowsel_cm);
    let rendered_subscreen = frame.prevent_math_mode != 3
        && frame.add_subscreen
        && frame.math_enabled != 0
        && frame.screen_enabled[1] != 0;
    pu32!(rendered_subscreen);
    // 3 padding u32s → 64 bytes total for the scalar section
    buf.extend_from_slice(&[0u8; 12]);

    debug_assert_eq!(buf.len(), PARAMS_SIZE as usize);

    // Scanline window data: 224 scanlines packed into 56 vec4<u32>.
    for chunk in frame.scanlines.chunks(4) {
        for sl in chunk {
            let packed = sl.window1_left as u32
                | ((sl.window1_right as u32) << 8)
                | ((sl.window2_left as u32) << 16)
                | ((sl.window2_right as u32) << 24);
            buf.extend_from_slice(&packed.to_ne_bytes());
        }
        for _ in chunk.len()..4 {
            buf.extend_from_slice(&[0u8; 4]);
        }
    }

    debug_assert_eq!(buf.len(), UNIFORM_SIZE as usize);
    buf
}

/// Convert raw scanline tuples from `zelda_rtl::ppu_scanline_windows()` into
/// the `ScanlineRegs` array stored in `GpuFrame`.
pub fn scanlines_from_raw(raw: &RawScanlineFrame) -> Box<[ScanlineRegs; 224]> {
    GpuFrame::scanlines_from_raw(raw)
}
