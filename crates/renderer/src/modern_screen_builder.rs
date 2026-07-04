use crate::modern_frame::{ModernFrame, MODERN_FRAME_HEIGHT, MODERN_FRAME_WIDTH};
use crate::modern_index_atlas::ModernIndexTile;

fn u32s_to_le_bytes(words: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_buffer_with_words(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    words: &[u32],
) -> wgpu::Buffer {
    let bytes = u32s_to_le_bytes(if words.is_empty() { &[0] } else { words });
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buffer, 0, &bytes);
    buffer
}

fn uniform_buffer_with_words(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    words: &[u32],
) -> wgpu::Buffer {
    let bytes = u32s_to_le_bytes(words);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buffer, 0, &bytes);
    buffer
}

pub(crate) struct ModernGpuScreenBuilder {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ModernGpuScreenBuilder {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("modern_screen_builder"),
            entries: &[
                storage_entry(0, false),
                storage_entry(1, false),
                storage_entry(2, true),
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
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("modern_screen_builder"),
            source: wgpu::ShaderSource::Wgsl(include_str!("modern_screen_builder.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("modern_screen_builder"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("modern_screen_builder"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub(crate) fn render_into(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        main_buffer: &wgpu::Buffer,
        sub_buffer: &wgpu::Buffer,
    ) {
        let cell_words = modern_screen_builder_cell_words(bg_cells, sprite_cells);
        let bg_instance_words = modern_screen_builder_bg_instance_words(frame, bg_cells.len());
        let sprite_instance_words =
            modern_screen_builder_sprite_instance_words(frame, sprite_cells.len());
        let cgram_words = modern_screen_builder_cgram_words(frame);
        let scroll_words = modern_screen_builder_scroll_words(frame);
        let main_tm_words = modern_screen_builder_main_tm_words(frame);
        let window_words = modern_screen_builder_window_words(frame);
        let (data_words, offsets) = modern_screen_builder_data_words(
            &cell_words,
            &bg_instance_words,
            &sprite_instance_words,
            &cgram_words,
            &scroll_words,
            &main_tm_words,
            &window_words,
        );
        let params = modern_screen_builder_params(
            frame,
            bg_cells,
            &bg_instance_words,
            &sprite_instance_words,
            offsets,
        );

        let data_buffer =
            storage_buffer_with_words(device, queue, "modern_screen_data", &data_words);
        let params_buffer =
            uniform_buffer_with_words(device, queue, "modern_screen_params", &params);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("modern_screen_builder"),
            layout: &self.bind_group_layout,
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
                    resource: data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let pixel_count = u32::from(MODERN_FRAME_WIDTH) * u32::from(MODERN_FRAME_HEIGHT);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("modern_screen_builder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("modern_screen_builder"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(pixel_count.div_ceil(64), 1, 1);
        }
        queue.submit([encoder.finish()]);
    }
}

fn modern_screen_builder_cell_words(
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
) -> Vec<u32> {
    let mut words = Vec::with_capacity((bg_cells.len() + sprite_cells.len()).max(1) * 64);
    for cell in bg_cells.iter().chain(sprite_cells.iter()) {
        words.extend(cell.indices.iter().map(|&index| u32::from(index)));
    }
    if words.is_empty() {
        words.push(0);
    }
    words
}

fn modern_screen_builder_bg_instance_words(frame: &ModernFrame, bg_cell_count: usize) -> Vec<u32> {
    let mut words = Vec::new();
    for layer in frame.bg_layers.iter().take(3) {
        for inst in &layer.index_tiles {
            if inst.cell_id as usize >= bg_cell_count {
                continue;
            }
            words.extend_from_slice(&[
                inst.cell_id,
                i32::from(inst.screen_x) as u32,
                i32::from(inst.screen_y) as u32,
                u32::from(inst.palette),
                u32::from(inst.priority),
                u32::from(layer.index),
                0,
                0,
            ]);
        }
    }
    words
}

fn modern_screen_builder_sprite_instance_words(
    frame: &ModernFrame,
    sprite_cell_count: usize,
) -> Vec<u32> {
    let mut words = Vec::new();
    for inst in &frame.index_sprites {
        if inst.cell_id as usize >= sprite_cell_count {
            continue;
        }
        let mut flags = 0u32;
        if inst.hflip {
            flags |= 0x1;
        }
        if inst.vflip {
            flags |= 0x2;
        }
        words.extend_from_slice(&[
            inst.cell_id,
            i32::from(inst.screen_x) as u32,
            i32::from(inst.screen_y) as u32,
            u32::from(inst.palette),
            u32::from(inst.priority),
            flags,
            u32::from(inst.row_mask),
            0,
        ]);
    }
    words
}

fn modern_screen_builder_cgram_words(frame: &ModernFrame) -> Vec<u32> {
    frame
        .cgram_rgba
        .iter()
        .map(|px| u32::from(px[0]) | (u32::from(px[1]) << 8) | (u32::from(px[2]) << 16))
        .collect()
}

fn modern_screen_builder_scroll_words(frame: &ModernFrame) -> Vec<u32> {
    let mut words = Vec::with_capacity(usize::from(MODERN_FRAME_HEIGHT) * 8);
    for row in 0..usize::from(MODERN_FRAME_HEIGHT) {
        let scanline = frame.bg_scroll_scanlines.get(row);
        for layer in 0..4usize {
            let base = [
                frame.bg_layers.get(layer).map_or(0, |bg| bg.scroll_x),
                frame.bg_layers.get(layer).map_or(0, |bg| bg.scroll_y),
            ];
            let scroll = scanline.map_or(base, |sl| sl[layer]);
            words.push(u32::from(scroll[0]));
            words.push(u32::from(scroll[1]));
        }
    }
    words
}

fn modern_screen_builder_main_tm_words(frame: &ModernFrame) -> Vec<u32> {
    (0..usize::from(MODERN_FRAME_HEIGHT))
        .map(|row| u32::from(frame.main_tm_scanlines.get(row).copied().unwrap_or(0xff)))
        .collect()
}

fn modern_screen_builder_window_words(frame: &ModernFrame) -> Vec<u32> {
    let mut words = Vec::with_capacity(usize::from(MODERN_FRAME_HEIGHT) * 4);
    for row in 0..usize::from(MODERN_FRAME_HEIGHT) {
        words.extend(
            frame
                .window_scanlines
                .get(row)
                .copied()
                .unwrap_or([0u8; 4])
                .map(u32::from),
        );
    }
    words
}

#[derive(Clone, Copy)]
struct ModernScreenBuilderOffsets {
    cells: u32,
    bg_instances: u32,
    sprite_instances: u32,
    cgram: u32,
    scroll: u32,
    main_tm: u32,
    window: u32,
}

fn modern_screen_builder_data_words(
    cell_words: &[u32],
    bg_instance_words: &[u32],
    sprite_instance_words: &[u32],
    cgram_words: &[u32],
    scroll_words: &[u32],
    main_tm_words: &[u32],
    window_words: &[u32],
) -> (Vec<u32>, ModernScreenBuilderOffsets) {
    let mut data = Vec::new();
    let cells = data.len() as u32;
    data.extend_from_slice(cell_words);
    let bg_instances = data.len() as u32;
    data.extend_from_slice(if bg_instance_words.is_empty() {
        &[0]
    } else {
        bg_instance_words
    });
    let sprite_instances = data.len() as u32;
    data.extend_from_slice(if sprite_instance_words.is_empty() {
        &[0]
    } else {
        sprite_instance_words
    });
    let cgram = data.len() as u32;
    data.extend_from_slice(cgram_words);
    let scroll = data.len() as u32;
    data.extend_from_slice(scroll_words);
    let main_tm = data.len() as u32;
    data.extend_from_slice(main_tm_words);
    let window = data.len() as u32;
    data.extend_from_slice(window_words);
    (
        data,
        ModernScreenBuilderOffsets {
            cells,
            bg_instances,
            sprite_instances,
            cgram,
            scroll,
            main_tm,
            window,
        },
    )
}

fn modern_screen_builder_params(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    bg_instance_words: &[u32],
    sprite_instance_words: &[u32],
    offsets: ModernScreenBuilderOffsets,
) -> [u32; 32] {
    let backdrop = frame.backdrop_color_rgba;
    let backdrop_c5 = [
        u32::from(backdrop[0] >> 3),
        u32::from(backdrop[1] >> 3),
        u32::from(backdrop[2] >> 3),
    ];
    let backdrop_word = backdrop_c5[0] | (backdrop_c5[1] << 5) | (backdrop_c5[2] << 10) | (5 << 15);
    let scroll_mask = (0..3usize).fold(0u32, |mask, layer| {
        if modern_screen_builder_layer_needs_scroll(frame, layer) {
            mask | (1u32 << layer)
        } else {
            mask
        }
    });
    let layer_params = |layer: usize| -> [u32; 4] {
        let Some(bg) = frame.bg_layers.get(layer) else {
            return [0, 0, 256, 224];
        };
        [
            u32::from(bg.scroll_x),
            u32::from(bg.scroll_y),
            u32::from(bg.wrap_w).max(256),
            u32::from(bg.wrap_h).max(224),
        ]
    };
    let p2 = layer_params(0);
    let p3 = layer_params(1);
    let p4 = layer_params(2);
    [
        u32::from(MODERN_FRAME_WIDTH) * u32::from(MODERN_FRAME_HEIGHT),
        bg_cells.len() as u32,
        (bg_instance_words.len() / 8) as u32,
        (sprite_instance_words.len() / 8) as u32,
        backdrop_word,
        u32::from(frame.screen_enabled_main),
        u32::from(frame.screen_enabled_sub),
        scroll_mask,
        p2[0],
        p2[1],
        p2[2],
        p2[3],
        p3[0],
        p3[1],
        p3[2],
        p3[3],
        p4[0],
        p4[1],
        p4[2],
        p4[3],
        offsets.cells,
        offsets.bg_instances,
        offsets.sprite_instances,
        offsets.cgram,
        offsets.scroll,
        offsets.main_tm,
        offsets.window,
        frame.windowsel,
        u32::from(frame.screen_windowed_main),
        u32::from(frame.screen_windowed_sub),
        u32::from(frame.mosaic_enabled),
        u32::from(frame.mosaic_size),
    ]
}

fn modern_screen_builder_layer_needs_scroll(frame: &ModernFrame, layer: usize) -> bool {
    let Some(bg) = frame.bg_layers.get(layer) else {
        return false;
    };
    let varies = frame
        .bg_scroll_scanlines
        .iter()
        .any(|sl| sl[layer][0] != bg.scroll_x || sl[layer][1] != bg.scroll_y);
    varies || bg.scroll_x != 0 || bg.scroll_y != 0
}
