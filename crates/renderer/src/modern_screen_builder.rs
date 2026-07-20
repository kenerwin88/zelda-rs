use crate::modern_frame::ModernIndexTileInstance;
use crate::modern_frame::{ModernFrame, MODERN_FRAME_HEIGHT, MODERN_FRAME_WIDTH};
use crate::modern_index_atlas::ModernIndexTile;
use std::cell::RefCell;

const BG_INSTANCE_STRIDE_WORDS: usize = 8;
const BG_BUCKET_COLS: usize = MODERN_FRAME_WIDTH as usize / 8;
const BG_BUCKET_ROWS: usize = MODERN_FRAME_HEIGHT as usize / 8;
const BG_BUCKET_LAYERS: usize = 3;
const BG_BUCKET_PRIORITIES: usize = 2;
const BG_BUCKET_COUNT: usize =
    BG_BUCKET_LAYERS * BG_BUCKET_PRIORITIES * BG_BUCKET_COLS * BG_BUCKET_ROWS;
const BG_BUCKET_HEADER_WORDS: usize = BG_BUCKET_COUNT * 2;

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

fn fill_u32_bytes(words: &[u32], bytes: &mut Vec<u8>) {
    bytes.clear();
    bytes.reserve(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
}

fn ensure_buffer(
    device: &wgpu::Device,
    buffer: &mut Option<wgpu::Buffer>,
    capacity_bytes: &mut u64,
    label: &str,
    usage: wgpu::BufferUsages,
    needed: u64,
) -> wgpu::Buffer {
    if buffer.is_none() || *capacity_bytes < needed {
        let capacity = needed.next_power_of_two().max(4);
        *buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: capacity,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        *capacity_bytes = capacity;
    }
    buffer.as_ref().expect("buffer created above").clone()
}

pub(crate) struct ModernGpuScreenBuilder {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    scratch: RefCell<ModernScreenBuilderScratch>,
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
            scratch: RefCell::new(ModernScreenBuilderScratch::default()),
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
        let (data_buffer, params_buffer) = {
            let mut scratch = self.scratch.borrow_mut();
            scratch.build(frame, bg_cells, sprite_cells);
            scratch.upload(device, queue)
        };

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

#[derive(Default)]
struct ModernScreenBuilderScratch {
    cell_words: Vec<u32>,
    bg_instance_words: Vec<u32>,
    bg_bucket_words: Vec<u32>,
    sprite_instance_words: Vec<u32>,
    cgram_words: Vec<u32>,
    scroll_words: Vec<u32>,
    main_tm_words: Vec<u32>,
    window_words: Vec<u32>,
    data_words: Vec<u32>,
    params_words: Vec<u32>,
    bucket_lists: Vec<Vec<u32>>,
    data_bytes: Vec<u8>,
    params_bytes: Vec<u8>,
    data_buffer: Option<wgpu::Buffer>,
    data_capacity_bytes: u64,
    params_buffer: Option<wgpu::Buffer>,
    params_capacity_bytes: u64,
}

impl ModernScreenBuilderScratch {
    fn build(
        &mut self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
    ) {
        modern_screen_builder_cell_words(bg_cells, sprite_cells, &mut self.cell_words);
        self.build_bg_instances_and_buckets(frame, bg_cells.len());
        modern_screen_builder_sprite_instance_words(
            frame,
            sprite_cells.len(),
            &mut self.sprite_instance_words,
        );
        modern_screen_builder_cgram_words(frame, &mut self.cgram_words);
        modern_screen_builder_scroll_words(frame, &mut self.scroll_words);
        modern_screen_builder_main_tm_words(frame, &mut self.main_tm_words);
        modern_screen_builder_window_words(frame, &mut self.window_words);
        let offsets = modern_screen_builder_data_words(
            &self.cell_words,
            &self.bg_instance_words,
            &self.bg_bucket_words,
            &self.sprite_instance_words,
            &self.cgram_words,
            &self.scroll_words,
            &self.main_tm_words,
            &self.window_words,
            &mut self.data_words,
        );
        modern_screen_builder_params(
            frame,
            bg_cells,
            &self.bg_instance_words,
            &self.sprite_instance_words,
            offsets,
            &mut self.params_words,
        );
    }

    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::Buffer, wgpu::Buffer) {
        let data_words: &[u32] = if self.data_words.is_empty() {
            &[0]
        } else {
            &self.data_words
        };
        fill_u32_bytes(data_words, &mut self.data_bytes);
        let data_buffer = ensure_buffer(
            device,
            &mut self.data_buffer,
            &mut self.data_capacity_bytes,
            "modern_screen_data",
            wgpu::BufferUsages::STORAGE,
            self.data_bytes.len() as u64,
        );
        queue.write_buffer(&data_buffer, 0, &self.data_bytes);

        let params_words: &[u32] = if self.params_words.is_empty() {
            &[0]
        } else {
            &self.params_words
        };
        fill_u32_bytes(params_words, &mut self.params_bytes);
        let params_buffer = ensure_buffer(
            device,
            &mut self.params_buffer,
            &mut self.params_capacity_bytes,
            "modern_screen_params",
            wgpu::BufferUsages::UNIFORM,
            self.params_bytes.len() as u64,
        );
        queue.write_buffer(&params_buffer, 0, &self.params_bytes);

        (data_buffer, params_buffer)
    }

    fn build_bg_instances_and_buckets(&mut self, frame: &ModernFrame, bg_cell_count: usize) {
        self.bg_instance_words.clear();
        if self.bucket_lists.len() != BG_BUCKET_COUNT {
            self.bucket_lists = vec![Vec::new(); BG_BUCKET_COUNT];
        } else {
            for bucket in &mut self.bucket_lists {
                bucket.clear();
            }
        }

        let mut bucket_layer_mask = 0u32;
        for layer in frame.bg_layers.iter().take(3) {
            let layer_index = usize::from(layer.index);
            let bucket_layer = modern_screen_builder_layer_is_bucketable(frame, layer_index);
            if bucket_layer {
                bucket_layer_mask |= 1u32 << layer_index;
            }
            for inst in &layer.index_tiles {
                if inst.cell_id as usize >= bg_cell_count {
                    continue;
                }
                let instance_index =
                    (self.bg_instance_words.len() / BG_INSTANCE_STRIDE_WORDS) as u32;
                self.bg_instance_words.extend_from_slice(&[
                    inst.cell_id,
                    i32::from(inst.screen_x) as u32,
                    i32::from(inst.screen_y) as u32,
                    u32::from(inst.palette),
                    u32::from(inst.priority),
                    u32::from(layer.index),
                    0,
                    0,
                ]);
                if bucket_layer {
                    push_bg_instance_buckets(
                        &mut self.bucket_lists,
                        layer_index,
                        inst,
                        layer.wrap_w,
                        layer.wrap_h,
                        instance_index,
                    );
                }
            }
        }
        self.bg_bucket_words.clear();
        self.bg_bucket_words.resize(BG_BUCKET_HEADER_WORDS, 0);
        let mut candidates = Vec::new();
        for (bucket_index, bucket) in self.bucket_lists.iter().enumerate() {
            self.bg_bucket_words[bucket_index * 2] = candidates.len() as u32;
            self.bg_bucket_words[bucket_index * 2 + 1] = bucket.len() as u32;
            candidates.extend(bucket.iter().copied());
        }
        self.bg_bucket_words.extend(candidates);
        if self.bg_bucket_words.is_empty() {
            self.bg_bucket_words.push(0);
        }
        let _ = bucket_layer_mask;
    }
}

fn modern_screen_builder_cell_words(
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    words: &mut Vec<u32>,
) {
    words.clear();
    words.reserve((bg_cells.len() + sprite_cells.len()).max(1) * 64);
    for cell in bg_cells.iter().chain(sprite_cells.iter()) {
        words.extend(cell.indices.iter().map(|&index| u32::from(index)));
    }
    if words.is_empty() {
        words.push(0);
    }
}

fn modern_screen_builder_sprite_instance_words(
    frame: &ModernFrame,
    sprite_cell_count: usize,
    words: &mut Vec<u32>,
) {
    words.clear();
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
}

fn modern_screen_builder_cgram_words(frame: &ModernFrame, words: &mut Vec<u32>) {
    words.clear();
    words.extend(
        frame
            .cgram_rgba
            .iter()
            .map(|px| u32::from(px[0]) | (u32::from(px[1]) << 8) | (u32::from(px[2]) << 16)),
    );
}

fn modern_screen_builder_scroll_words(frame: &ModernFrame, words: &mut Vec<u32>) {
    words.clear();
    words.reserve(usize::from(MODERN_FRAME_HEIGHT) * 8);
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
}

fn modern_screen_builder_main_tm_words(frame: &ModernFrame, words: &mut Vec<u32>) {
    words.clear();
    words.extend(
        (0..usize::from(MODERN_FRAME_HEIGHT))
            .map(|row| u32::from(frame.main_tm_scanlines.get(row).copied().unwrap_or(0xff))),
    );
}

fn modern_screen_builder_window_words(frame: &ModernFrame, words: &mut Vec<u32>) {
    words.clear();
    words.reserve(usize::from(MODERN_FRAME_HEIGHT) * 4);
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
}

#[derive(Clone, Copy)]
struct ModernScreenBuilderOffsets {
    cells: u32,
    bg_instances: u32,
    bg_buckets: u32,
    sprite_instances: u32,
    cgram: u32,
    scroll: u32,
    main_tm: u32,
    window: u32,
}

fn modern_screen_builder_data_words(
    cell_words: &[u32],
    bg_instance_words: &[u32],
    bg_bucket_words: &[u32],
    sprite_instance_words: &[u32],
    cgram_words: &[u32],
    scroll_words: &[u32],
    main_tm_words: &[u32],
    window_words: &[u32],
    data: &mut Vec<u32>,
) -> ModernScreenBuilderOffsets {
    data.clear();
    let cells = data.len() as u32;
    data.extend_from_slice(cell_words);
    let bg_instances = data.len() as u32;
    data.extend_from_slice(if bg_instance_words.is_empty() {
        &[0]
    } else {
        bg_instance_words
    });
    let bg_buckets = data.len() as u32;
    data.extend_from_slice(bg_bucket_words);
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
    ModernScreenBuilderOffsets {
        cells,
        bg_instances,
        bg_buckets,
        sprite_instances,
        cgram,
        scroll,
        main_tm,
        window,
    }
}

fn modern_screen_builder_params(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    bg_instance_words: &[u32],
    sprite_instance_words: &[u32],
    offsets: ModernScreenBuilderOffsets,
    params: &mut Vec<u32>,
) {
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
    let bucket_layer_mask = (0..3usize).fold(0u32, |mask, layer| {
        if modern_screen_builder_layer_is_bucketable(frame, layer) {
            mask | (1u32 << layer)
        } else {
            mask
        }
    });
    params.clear();
    params.extend_from_slice(&[
        u32::from(MODERN_FRAME_WIDTH) * u32::from(MODERN_FRAME_HEIGHT),
        bg_cells.len() as u32,
        (bg_instance_words.len() / BG_INSTANCE_STRIDE_WORDS) as u32,
        (sprite_instance_words.len() / BG_INSTANCE_STRIDE_WORDS) as u32,
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
        offsets.bg_buckets,
        bucket_layer_mask,
        BG_BUCKET_HEADER_WORDS as u32,
        0,
    ]);
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

fn modern_screen_builder_layer_varies_by_scanline(frame: &ModernFrame, layer: usize) -> bool {
    let Some(bg) = frame.bg_layers.get(layer) else {
        return false;
    };
    frame
        .bg_scroll_scanlines
        .iter()
        .any(|sl| sl[layer][0] != bg.scroll_x || sl[layer][1] != bg.scroll_y)
}

fn modern_screen_builder_layer_mosaic_enabled(frame: &ModernFrame, layer: usize) -> bool {
    frame.mosaic_size > 1 && (frame.mosaic_enabled & (1u8 << layer)) != 0
}

fn modern_screen_builder_layer_is_bucketable(frame: &ModernFrame, layer: usize) -> bool {
    layer < BG_BUCKET_LAYERS
        && !modern_screen_builder_layer_varies_by_scanline(frame, layer)
        && !modern_screen_builder_layer_mosaic_enabled(frame, layer)
}

fn bg_bucket_index(layer: usize, priority: bool, x: usize, y: usize) -> usize {
    (((layer * BG_BUCKET_PRIORITIES + usize::from(priority)) * BG_BUCKET_ROWS + y) * BG_BUCKET_COLS)
        + x
}

fn push_bg_instance_buckets(
    buckets: &mut [Vec<u32>],
    layer: usize,
    inst: &ModernIndexTileInstance,
    wrap_w: u16,
    wrap_h: u16,
    instance_index: u32,
) {
    let wrap_w = i32::from(wrap_w.max(256));
    let wrap_h = i32::from(wrap_h.max(224));
    for wrap_y in [-wrap_h, 0, wrap_h] {
        for wrap_x in [-wrap_w, 0, wrap_w] {
            push_bg_instance_bucket_position(
                buckets,
                layer,
                inst,
                i32::from(inst.screen_x) + wrap_x,
                i32::from(inst.screen_y) + wrap_y,
                instance_index,
            );
        }
    }
}

fn push_bg_instance_bucket_position(
    buckets: &mut [Vec<u32>],
    layer: usize,
    inst: &ModernIndexTileInstance,
    screen_x: i32,
    screen_y: i32,
    instance_index: u32,
) {
    let x0 = screen_x.max(0);
    let y0 = screen_y.max(0);
    let x1 = (screen_x + 7).min(i32::from(MODERN_FRAME_WIDTH) - 1);
    let y1 = (screen_y + 7).min(i32::from(MODERN_FRAME_HEIGHT) - 1);
    if x0 > x1 || y0 > y1 {
        return;
    }
    let bx0 = usize::try_from(x0 / 8).unwrap_or(0);
    let by0 = usize::try_from(y0 / 8).unwrap_or(0);
    let bx1 = usize::try_from(x1 / 8).unwrap_or(0).min(BG_BUCKET_COLS - 1);
    let by1 = usize::try_from(y1 / 8).unwrap_or(0).min(BG_BUCKET_ROWS - 1);
    for by in by0..=by1 {
        for bx in bx0..=bx1 {
            let bucket = bg_bucket_index(layer, inst.priority, bx, by);
            buckets[bucket].push(instance_index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_hd_overrides::NO_SOURCE_KEY;

    fn test_inst(
        cell_id: u32,
        screen_x: i16,
        screen_y: i16,
        priority: bool,
    ) -> ModernIndexTileInstance {
        ModernIndexTileInstance {
            cell_id,
            source_key: NO_SOURCE_KEY,
            screen_x,
            screen_y,
            palette: 0,
            hflip: false,
            vflip: false,
            priority,
        }
    }

    #[test]
    fn bg_bucket_pack_preserves_candidate_order_for_overlapping_instances() {
        let mut frame = ModernFrame::empty();
        frame.bg_layers[0]
            .index_tiles
            .push(test_inst(0, 0, 0, false));
        frame.bg_layers[0]
            .index_tiles
            .push(test_inst(1, 4, 4, false));
        let mut scratch = ModernScreenBuilderScratch::default();

        scratch.build_bg_instances_and_buckets(&frame, 2);

        let bucket = bg_bucket_index(0, false, 0, 0);
        let offset = scratch.bg_bucket_words[bucket * 2] as usize;
        let count = scratch.bg_bucket_words[bucket * 2 + 1] as usize;
        let candidates = &scratch.bg_bucket_words
            [BG_BUCKET_HEADER_WORDS + offset..BG_BUCKET_HEADER_WORDS + offset + count];
        assert_eq!(candidates, &[0, 1]);
    }

    #[test]
    fn bg_bucket_pack_skips_layers_with_scanline_scroll_variance() {
        let mut frame = ModernFrame::empty();
        frame.bg_layers[0]
            .index_tiles
            .push(test_inst(0, 0, 0, false));
        frame.bg_scroll_scanlines = vec![[[0u16; 2]; 4]; usize::from(MODERN_FRAME_HEIGHT)];
        frame.bg_scroll_scanlines[12][0][0] = 1;
        let mut scratch = ModernScreenBuilderScratch::default();

        scratch.build_bg_instances_and_buckets(&frame, 1);

        let bucket = bg_bucket_index(0, false, 0, 0);
        assert_eq!(scratch.bg_bucket_words[bucket * 2 + 1], 0);
    }

    #[test]
    fn bg_bucket_pack_covers_wrapped_tiles_at_a_scroll_origin() {
        let mut frame = ModernFrame::empty();
        frame.bg_layers[0].scroll_x = 1;
        frame.bg_layers[0]
            .index_tiles
            .push(test_inst(0, -255, 0, false));
        let mut scratch = ModernScreenBuilderScratch::default();

        scratch.build_bg_instances_and_buckets(&frame, 1);

        let bucket = bg_bucket_index(0, false, 0, 0);
        assert_eq!(scratch.bg_bucket_words[bucket * 2 + 1], 1);
        assert!(modern_screen_builder_layer_is_bucketable(&frame, 0));
    }
}
