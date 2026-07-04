use crate::gpu_frame::{GpuFrame, ScanlineRegs};
use crate::tile_atlas::CgramPalette;

const VRAM_WORDS: usize = 0x8000;
const VRAM_BYTES: usize = VRAM_WORDS * 4;
const SCANLINE_BYTES: usize = 56 * 16;
const HEADER_BYTES: usize = 64;
const SCANLINE_TM_OFFSET: usize = HEADER_BYTES;
const SCANLINE_WINDOW_OFFSET: usize = SCANLINE_TM_OFFSET + SCANLINE_BYTES;
const SCANLINE_MODE7_OFFSET: usize = SCANLINE_WINDOW_OFFSET + SCANLINE_BYTES;
const SCANLINE_MODE7_BYTES: usize = 224 * 8 * 4;
const UNIFORM_BYTES: usize = SCANLINE_MODE7_OFFSET + SCANLINE_MODE7_BYTES;

pub struct Mode7Renderer {
    pipeline: wgpu::RenderPipeline,
    vram_buf: wgpu::Buffer,
    uniform_buf: [wgpu::Buffer; 2],
    bind_group: [wgpu::BindGroup; 2],
    last_vram_hash: u32,
    vram_bytes: Vec<u8>,
}

impl Mode7Renderer {
    pub fn new(
        device: &wgpu::Device,
        cgram_palette: &CgramPalette,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        let vram_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mode7_vram"),
            size: VRAM_BYTES as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_buf = std::array::from_fn(|screen| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(if screen == 0 {
                    "mode7_uniform_main"
                } else {
                    "mode7_uniform_sub"
                }),
                size: UNIFORM_BYTES as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mode7_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = std::array::from_fn(|screen| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(if screen == 0 {
                    "mode7_bg_main"
                } else {
                    "mode7_bg_sub"
                }),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&cgram_palette.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: vram_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform_buf[screen].as_entire_binding(),
                    },
                ],
            })
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("mode7.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mode7_pipeline_layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mode7_pipeline"),
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

        Self {
            pipeline,
            vram_buf,
            uniform_buf,
            bind_group,
            last_vram_hash: 0,
            vram_bytes: vec![0; VRAM_BYTES],
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
        math_bit_pos: u32,
        layer_bit: u32,
        screen_idx: usize,
        window_layer_bit: u8,
        window_flags_shift: u32,
    ) {
        self.prepare_vram(queue, frame.vram);
        self.render_prepared(
            encoder,
            queue,
            frame,
            output_view,
            math_bit_pos,
            layer_bit,
            screen_idx,
            window_layer_bit,
            window_flags_shift,
        );
    }

    /// Upload Mode 7 VRAM once before one or more prepared render passes.
    pub fn prepare_vram(&mut self, queue: &wgpu::Queue, vram: &[u16]) {
        let hash = fnv32_u16(vram);
        if hash == self.last_vram_hash {
            return;
        }
        self.last_vram_hash = hash;
        for (i, &word) in vram.iter().take(VRAM_WORDS).enumerate() {
            self.vram_bytes[i * 4..i * 4 + 4].copy_from_slice(&u32::from(word).to_le_bytes());
        }
        queue.write_buffer(&self.vram_buf, 0, &self.vram_bytes);
    }

    /// Render using the Mode 7 VRAM buffer last supplied to `prepare_vram`.
    pub fn render_prepared(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        frame: &GpuFrame<'_>,
        output_view: &wgpu::TextureView,
        math_bit_pos: u32,
        layer_bit: u32,
        screen_idx: usize,
        window_layer_bit: u8,
        window_flags_shift: u32,
    ) {
        queue.write_buffer(
            &self.uniform_buf[screen_idx],
            0,
            &build_uniform_bytes(
                frame,
                math_bit_pos,
                layer_bit,
                screen_idx,
                window_layer_bit,
                window_flags_shift,
                &frame.scanlines,
            ),
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("mode7"),
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
        pass.set_bind_group(0, &self.bind_group[screen_idx], &[]);
        pass.draw(0..3, 0..1);
    }
}

fn build_uniform_bytes(
    frame: &GpuFrame<'_>,
    math_bit_pos: u32,
    layer_bit: u32,
    screen_idx: usize,
    window_layer_bit: u8,
    window_flags_shift: u32,
    scanlines: &[ScanlineRegs; 224],
) -> Vec<u8> {
    let mut bytes = vec![0u8; UNIFORM_BYTES];
    for (i, &value) in frame.mode7.matrix.iter().enumerate() {
        bytes[i * 4..i * 4 + 4].copy_from_slice(&i32::from(value).to_le_bytes());
    }
    let flags = u32::from(frame.mode7.large_field)
        | (u32::from(frame.mode7.char_fill) << 1)
        | (u32::from(frame.mode7.x_flip) << 2)
        | (u32::from(frame.mode7.y_flip) << 3)
        | (u32::from(frame.mode7.ext_bg_always_zero) << 4);
    bytes[32..36].copy_from_slice(&flags.to_le_bytes());
    bytes[36..40].copy_from_slice(&layer_bit.to_le_bytes());
    bytes[40..44].copy_from_slice(&math_bit_pos.to_le_bytes());
    let window_flags = (frame.windowsel >> window_flags_shift) & 0x0f;
    let windowed = u32::from(frame.screen_windowed[screen_idx] & window_layer_bit != 0);
    bytes[44..48].copy_from_slice(&window_flags.to_le_bytes());
    bytes[48..52].copy_from_slice(&windowed.to_le_bytes());

    for (i, sl) in scanlines.iter().enumerate().take(224) {
        let off = SCANLINE_TM_OFFSET + i * 4;
        bytes[off] = sl.screen_enabled_main;
    }
    for (i, sl) in scanlines.iter().enumerate().take(224) {
        let packed = u32::from(sl.window1_left)
            | (u32::from(sl.window1_right) << 8)
            | (u32::from(sl.window2_left) << 16)
            | (u32::from(sl.window2_right) << 24);
        let off = SCANLINE_WINDOW_OFFSET + i * 4;
        bytes[off..off + 4].copy_from_slice(&packed.to_le_bytes());
    }
    for (line, sl) in scanlines.iter().enumerate().take(224) {
        for (i, &value) in sl.mode7_matrix.iter().enumerate() {
            let off = SCANLINE_MODE7_OFFSET + (line * 8 + i) * 4;
            bytes[off..off + 4].copy_from_slice(&i32::from(value).to_le_bytes());
        }
    }
    bytes
}

fn fnv32_u16(words: &[u16]) -> u32 {
    let mut hash = 2166136261u32;
    for &word in words {
        let [lo, hi] = word.to_le_bytes();
        hash = (hash ^ u32::from(lo)).wrapping_mul(16777619);
        hash = (hash ^ u32::from(hi)).wrapping_mul(16777619);
    }
    hash
}
