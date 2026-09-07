use crate::modern_frame::{ModernFrame, MODERN_FRAME_HEIGHT, MODERN_FRAME_WIDTH};
use crate::modern_index_atlas::ModernIndexTile;
use std::cell::RefCell;

const BG_INSTANCE_STRIDE_WORDS: usize = 8;
/// BG instance classes: three layers times two priorities. The first
/// `BG_CLASS_COUNT * 2` reserved words hold (start instance, count) per
/// class so the screen shader walks only the instances a paint pass can use.
const BG_CLASS_COUNT: usize = 6;
/// BG row buckets: per class, one (offset, count) header per 8-pixel row in the
/// coordinate space the shader samples that layer in (wrapped BG rows for a
/// per-scanline-scroll layer, screen rows otherwise). Offsets are relative to
/// the reserved block. The table starts at `BG_ROW_TABLE_WORDS`.
const BG_ROW_TABLE_WORDS: usize = 16;
const BG_ROW_BUCKETS_PER_CLASS: usize = 64;
const BG_ROW_HEADER_WORDS: usize = BG_CLASS_COUNT * BG_ROW_BUCKETS_PER_CLASS * 2;
/// Sprite candidate buckets: one (offset, count) header per 8x8 screen cell,
/// followed by the candidate instance indices in OAM (traversal) order. The
/// table starts after the BG row headers inside the reserved block; candidates
/// of both tables share the pool that follows the sprite header.
const SPRITE_BUCKET_GRID_W: usize = MODERN_FRAME_WIDTH as usize / 8;
const SPRITE_BUCKET_GRID_H: usize = MODERN_FRAME_HEIGHT as usize / 8;
const SPRITE_BUCKET_TABLE_WORDS: usize = BG_ROW_TABLE_WORDS + BG_ROW_HEADER_WORDS;
const SPRITE_BUCKET_HEADER_WORDS: usize = SPRITE_BUCKET_GRID_W * SPRITE_BUCKET_GRID_H * 2;
const RESERVED_POOL_START_WORDS: usize = SPRITE_BUCKET_TABLE_WORDS + SPRITE_BUCKET_HEADER_WORDS;

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
    reserved_words: Vec<u32>,
    bg_row_buckets_enabled: bool,
    sprite_buckets_enabled: bool,
    sprite_instance_words: Vec<u32>,
    cgram_words: Vec<u32>,
    scroll_words: Vec<u32>,
    main_tm_words: Vec<u32>,
    window_words: Vec<u32>,
    data_words: Vec<u32>,
    params_words: Vec<u32>,
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
        self.build_bg_instances(frame, bg_cells.len());
        modern_screen_builder_sprite_instance_words(
            frame,
            sprite_cells.len(),
            &mut self.sprite_instance_words,
        );
        self.bg_row_buckets_enabled = modern_screen_builder_bg_row_buckets(
            frame,
            &self.bg_instance_words,
            &mut self.reserved_words,
        );
        self.sprite_buckets_enabled = modern_screen_builder_sprite_buckets(
            &self.sprite_instance_words,
            &mut self.reserved_words,
        );
        modern_screen_builder_cgram_words(frame, &mut self.cgram_words);
        modern_screen_builder_scroll_words(frame, &mut self.scroll_words);
        modern_screen_builder_main_tm_words(frame, &mut self.main_tm_words);
        modern_screen_builder_window_words(frame, &mut self.window_words);
        let offsets = modern_screen_builder_data_words(
            &self.cell_words,
            &self.bg_instance_words,
            &self.reserved_words,
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
            self.bg_row_buckets_enabled,
            self.sprite_buckets_enabled,
            self.reserved_words.len(),
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

    fn build_bg_instances(&mut self, frame: &ModernFrame, bg_cell_count: usize) {
        self.bg_instance_words.clear();
        self.reserved_words.clear();
        self.reserved_words.resize(RESERVED_POOL_START_WORDS, 0);
        // The shader paints one (layer, priority) class per pass and keeps the
        // last covering instance in traversal order. Grouping the instances by
        // class, preserving their relative order inside each class, lets every
        // pass walk only its own segment with an identical result; the segment
        // table (start instance, count) per class lives at the head of the
        // reserved block (see `BG_CLASS_SEGMENT_WORDS`).
        let mut classes: [Vec<u32>; BG_CLASS_COUNT] = Default::default();
        for layer in frame.bg_layers.iter().take(3) {
            let layer_index = usize::from(layer.index).min(2);
            for inst in &layer.index_tiles {
                if inst.cell_id as usize >= bg_cell_count {
                    continue;
                }
                let class = layer_index * 2 + usize::from(inst.priority);
                classes[class].extend_from_slice(&[
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
        for (class, words) in classes.iter().enumerate() {
            let start = (self.bg_instance_words.len() / BG_INSTANCE_STRIDE_WORDS) as u32;
            let count = (words.len() / BG_INSTANCE_STRIDE_WORDS) as u32;
            self.reserved_words[class * 2] = start;
            self.reserved_words[class * 2 + 1] = count;
            self.bg_instance_words.extend_from_slice(words);
        }
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
    reserved: u32,
    sprite_instances: u32,
    cgram: u32,
    scroll: u32,
    main_tm: u32,
    window: u32,
}

fn modern_screen_builder_data_words(
    cell_words: &[u32],
    bg_instance_words: &[u32],
    reserved_words: &[u32],
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
    let reserved = data.len() as u32;
    data.extend_from_slice(reserved_words);
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
        reserved,
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
    bg_row_buckets_enabled: bool,
    sprite_buckets_enabled: bool,
    reserved_len: usize,
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
        offsets.reserved,
        0,
        reserved_len as u32,
        // p8.w bit 0: BG instances are grouped by (layer, priority) with the
        // segment table at the head of the reserved block; bit 1: sprite
        // candidate buckets at `SPRITE_BUCKET_TABLE_WORDS`; bit 2: BG row
        // buckets at `BG_ROW_TABLE_WORDS`.
        1 | if sprite_buckets_enabled { 2 } else { 0 } | if bg_row_buckets_enabled { 4 } else { 0 },
    ]);
}

/// Fill the sprite candidate buckets: every instance is appended, in
/// traversal order, to each 8x8 screen cell its 8x8 footprint overlaps, so a
/// per-pixel walk of one bucket meets the same first covering sprite as the
/// full walk. Returns false (table untouched, full walk) when it cannot fit.
fn modern_screen_builder_sprite_buckets(
    sprite_instance_words: &[u32],
    reserved: &mut Vec<u32>,
) -> bool {
    let instance_count = sprite_instance_words.len() / BG_INSTANCE_STRIDE_WORDS;
    let cell_count = SPRITE_BUCKET_GRID_W * SPRITE_BUCKET_GRID_H;
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); cell_count];
    for index in 0..instance_count {
        let base = index * BG_INSTANCE_STRIDE_WORDS;
        let x = sprite_instance_words[base + 1] as i32;
        let y = sprite_instance_words[base + 2] as i32;
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + 7).min(i32::from(MODERN_FRAME_WIDTH) - 1);
        let y1 = (y + 7).min(i32::from(MODERN_FRAME_HEIGHT) - 1);
        if x0 > x1 || y0 > y1 {
            continue;
        }
        for cy in (y0 >> 3)..=(y1 >> 3) {
            for cx in (x0 >> 3)..=(x1 >> 3) {
                buckets[cy as usize * SPRITE_BUCKET_GRID_W + cx as usize].push(index as u32);
            }
        }
    }
    if reserved.len() < RESERVED_POOL_START_WORDS {
        return false;
    }
    for (cell, candidates) in buckets.iter().enumerate() {
        reserved[SPRITE_BUCKET_TABLE_WORDS + cell * 2] = reserved.len() as u32;
        reserved[SPRITE_BUCKET_TABLE_WORDS + cell * 2 + 1] = candidates.len() as u32;
        reserved.extend_from_slice(candidates);
    }
    true
}

/// Mirror of the shader's `mosaic_active()`: any BG mosaic with a size above 1
/// makes every layer sample in screen space.
fn modern_screen_builder_mosaic_active(frame: &ModernFrame) -> bool {
    frame.mosaic_size > 1 && (frame.mosaic_enabled & 0x07) != 0
}

/// Fill the BG row buckets: every instance is appended, in class traversal
/// order, to each 8-pixel row bucket its 8-pixel-tall footprint can cover in
/// the coordinate space the shader samples its layer in. For a per-scanline
/// scroll layer that is the wrapped BG row of `inst_y + off_y` (the shader's
/// `by0`), otherwise the screen row. A per-row walk then meets exactly the
/// instances the full class walk can match, in the same order. Returns false
/// (table untouched, full segment walk) when a layer's torus needs more rows
/// than the table has.
fn modern_screen_builder_bg_row_buckets(
    frame: &ModernFrame,
    bg_instance_words: &[u32],
    reserved: &mut Vec<u32>,
) -> bool {
    if reserved.len() < RESERVED_POOL_START_WORDS {
        return false;
    }
    let mosaic_active = modern_screen_builder_mosaic_active(frame);
    let mut layer_rows = [(false, 224i32); 3];
    for (layer, entry) in layer_rows.iter_mut().enumerate() {
        let bg_h = frame
            .bg_layers
            .get(layer)
            .map_or(224, |bg| i32::from(bg.wrap_h).max(224));
        let scroll_space = !mosaic_active && modern_screen_builder_layer_needs_scroll(frame, layer);
        if scroll_space && (bg_h % 8 != 0 || bg_h / 8 > BG_ROW_BUCKETS_PER_CLASS as i32) {
            return false;
        }
        *entry = (scroll_space, bg_h);
    }
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); BG_CLASS_COUNT * BG_ROW_BUCKETS_PER_CLASS];
    let instance_count = bg_instance_words.len() / BG_INSTANCE_STRIDE_WORDS;
    for index in 0..instance_count {
        let base = index * BG_INSTANCE_STRIDE_WORDS;
        let inst_y = bg_instance_words[base + 2] as i32;
        let priority = usize::from(bg_instance_words[base + 4] != 0);
        let layer = (bg_instance_words[base + 5] as usize).min(2);
        let class = layer * 2 + priority;
        let (scroll_space, bg_h) = layer_rows[layer];
        let class_base = class * BG_ROW_BUCKETS_PER_CLASS;
        let mut last_row: Option<usize> = None;
        for k in 0..8 {
            let row = if scroll_space {
                let off_y = bg_h - 224;
                ((inst_y + off_y + k).rem_euclid(bg_h) / 8) as usize
            } else {
                let sy = inst_y + k;
                if sy < 0 || sy >= i32::from(MODERN_FRAME_HEIGHT) {
                    continue;
                }
                (sy / 8) as usize
            };
            if last_row != Some(row) {
                buckets[class_base + row].push(index as u32);
                last_row = Some(row);
            }
        }
    }
    for (bucket, candidates) in buckets.iter().enumerate() {
        reserved[BG_ROW_TABLE_WORDS + bucket * 2] = reserved.len() as u32;
        reserved[BG_ROW_TABLE_WORDS + bucket * 2 + 1] = candidates.len() as u32;
        reserved.extend_from_slice(candidates);
    }
    true
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_compositor_disables_candidate_selection() {
        let frame = ModernFrame::empty();
        let offsets = ModernScreenBuilderOffsets {
            cells: 0,
            bg_instances: 0,
            reserved: 0,
            sprite_instances: 0,
            cgram: 0,
            scroll: 0,
            main_tm: 0,
            window: 0,
        };
        let mut params = Vec::new();

        modern_screen_builder_params(
            &frame,
            &[],
            &[],
            &[],
            offsets,
            false,
            false,
            RESERVED_POOL_START_WORDS,
            &mut params,
        );

        assert_eq!(params.len(), 36);
        assert_eq!(params[33], 0, "candidate selection must remain disabled");
        assert_eq!(
            params[35], 1,
            "class-segmented BG instance lists are enabled"
        );
        assert_eq!(params[34], RESERVED_POOL_START_WORDS as u32);
    }

    /// CPU mirror of the shader's per-instance coverage test (`bg_instance_pixel`
    /// without the cell/index lookup): does instance `base` cover sample (sx, sy)?
    fn covers(
        frame: &ModernFrame,
        words: &[u32],
        base: usize,
        sx: i32,
        sy: i32,
        scroll_mask: u32,
    ) -> bool {
        let inst_x = words[base + 1] as i32;
        let inst_y = words[base + 2] as i32;
        let layer = words[base + 5] as usize;
        let bg = &frame.bg_layers[layer];
        let (local_x, local_y) =
            if !modern_screen_builder_mosaic_active(frame) && (scroll_mask >> layer) & 1 != 0 {
                let bg_w = i32::from(bg.wrap_w).max(256);
                let bg_h = i32::from(bg.wrap_h).max(224);
                let off_x = bg_w - 256;
                let off_y = bg_h - 224;
                let sl = frame.bg_scroll_scanlines[sy as usize][layer];
                let dh = i32::from(sl[0]) - i32::from(bg.scroll_x);
                let dv = i32::from(sl[1]) - i32::from(bg.scroll_y);
                let bx = (sx + dh + off_x).rem_euclid(bg_w);
                let by = (sy + dv + off_y).rem_euclid(bg_h);
                (
                    (bx - (inst_x + off_x)).rem_euclid(bg_w),
                    (by - (inst_y + off_y)).rem_euclid(bg_h),
                )
            } else {
                (sx - inst_x, sy - inst_y)
            };
        (0..8).contains(&local_x) && (0..8).contains(&local_y)
    }

    fn row_bucket_frame(scroll: bool, wrap_h: u16) -> ModernFrame {
        use crate::modern_frame::ModernIndexTileInstance;
        let mut frame = ModernFrame::empty();
        for (layer_index, bg) in frame.bg_layers.iter_mut().enumerate().take(3) {
            bg.index = layer_index as u8;
            bg.wrap_w = 512;
            bg.wrap_h = wrap_h;
            bg.scroll_x = if scroll { 37 } else { 0 };
            bg.scroll_y = if scroll {
                500 + layer_index as u16 * 9
            } else {
                0
            };
            let mut seed = 0x1234_5678u32 ^ (layer_index as u32 * 0x9e37);
            let mut next = || {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed
            };
            for n in 0..600u32 {
                let (x, y) = if scroll {
                    (
                        (next() % 80) as i16 * 8 - 37,
                        (next() % 80) as i16 * 8 - 500,
                    )
                } else {
                    (
                        (next() % 40) as i16 * 8 - 24,
                        (next() % 36) as i16 * 8 - 20 + (n % 3) as i16,
                    )
                };
                bg.index_tiles.push(ModernIndexTileInstance {
                    cell_id: n % 7,
                    source_key: 0,
                    screen_x: x,
                    screen_y: y,
                    palette: 1,
                    hflip: false,
                    vflip: false,
                    priority: n % 2 == 1,
                });
            }
        }
        frame.bg_scroll_scanlines = (0..usize::from(MODERN_FRAME_HEIGHT))
            .map(|_| {
                let mut sl = [[0u16; 2]; 4];
                for (layer, bg) in frame.bg_layers.iter().enumerate().take(3) {
                    sl[layer] = [bg.scroll_x, bg.scroll_y];
                }
                sl
            })
            .collect();
        if scroll {
            for (row, sl) in frame.bg_scroll_scanlines.iter_mut().enumerate() {
                for (layer, entry) in sl.iter_mut().enumerate().take(3) {
                    entry[0] = 37 + (row as u16 / 40) * 3;
                    entry[1] = 500 + layer as u16 * 9 + if row % 50 == 0 { 7 } else { 0 };
                }
            }
        }
        frame
    }

    fn assert_row_buckets_exact(frame: &ModernFrame) {
        let mut scratch = ModernScreenBuilderScratch::default();
        scratch.build_bg_instances(frame, 8);
        assert!(modern_screen_builder_bg_row_buckets(
            frame,
            &scratch.bg_instance_words,
            &mut scratch.reserved_words
        ));
        let words = &scratch.bg_instance_words;
        let reserved = &scratch.reserved_words;
        let scroll_mask = (0..3).fold(0u32, |m, l| {
            if modern_screen_builder_layer_needs_scroll(frame, l) {
                m | 1 << l
            } else {
                m
            }
        });
        let mosaic_active = modern_screen_builder_mosaic_active(frame);
        let mut checked = 0usize;
        for sy in (0..i32::from(MODERN_FRAME_HEIGHT)).step_by(3) {
            for sx in (0..i32::from(MODERN_FRAME_WIDTH)).step_by(5) {
                for class in 0..BG_CLASS_COUNT {
                    let layer = class / 2;
                    let bg = &frame.bg_layers[layer];
                    let row = if !mosaic_active && (scroll_mask >> layer) & 1 != 0 {
                        let bg_h = i32::from(bg.wrap_h).max(224);
                        let sl = frame.bg_scroll_scanlines[sy as usize][layer];
                        let dv = i32::from(sl[1]) - i32::from(bg.scroll_y);
                        ((sy + dv + bg_h - 224).rem_euclid(bg_h) / 8) as usize
                    } else {
                        (sy / 8) as usize
                    };
                    let header = BG_ROW_TABLE_WORDS + (class * BG_ROW_BUCKETS_PER_CLASS + row) * 2;
                    let (offset, count) =
                        (reserved[header] as usize, reserved[header + 1] as usize);
                    let bucket: Vec<u32> = reserved[offset..offset + count].to_vec();
                    // Full class walk, last match wins.
                    let start = reserved[class * 2] as usize;
                    let n = reserved[class * 2 + 1] as usize;
                    let full: Vec<u32> = (start..start + n)
                        .filter(|&i| {
                            covers(
                                frame,
                                words,
                                i * BG_INSTANCE_STRIDE_WORDS,
                                sx,
                                sy,
                                scroll_mask,
                            )
                        })
                        .map(|i| i as u32)
                        .collect();
                    let via_bucket: Vec<u32> = bucket
                        .iter()
                        .copied()
                        .filter(|&i| {
                            covers(
                                frame,
                                words,
                                i as usize * BG_INSTANCE_STRIDE_WORDS,
                                sx,
                                sy,
                                scroll_mask,
                            )
                        })
                        .collect();
                    assert_eq!(via_bucket, full, "class {class} at ({sx},{sy})");
                    checked += full.len();
                }
            }
        }
        assert!(
            checked > 1000,
            "the fixture must exercise real coverage ({checked})"
        );
    }

    #[test]
    fn bg_row_buckets_match_the_full_class_walk_in_screen_space() {
        assert_row_buckets_exact(&row_bucket_frame(false, 256));
    }

    #[test]
    fn bg_row_buckets_match_the_full_class_walk_with_per_scanline_scroll() {
        assert_row_buckets_exact(&row_bucket_frame(true, 512));
        assert_row_buckets_exact(&row_bucket_frame(true, 256));
    }

    #[test]
    fn bg_row_buckets_use_screen_rows_when_mosaic_is_active() {
        // Screen-space instance placement with a scrolling layer: without the
        // mosaic the layer would be sampled in BG space.
        let mut frame = row_bucket_frame(false, 512);
        for bg in frame.bg_layers.iter_mut().take(3) {
            bg.scroll_x = 37;
        }
        assert!(modern_screen_builder_layer_needs_scroll(&frame, 0));
        frame.mosaic_enabled = 0x01;
        frame.mosaic_size = 4;
        assert_row_buckets_exact(&frame);
    }

    #[test]
    fn sprite_buckets_append_to_the_shared_pool_after_bg_rows() {
        let frame = row_bucket_frame(false, 256);
        let mut scratch = ModernScreenBuilderScratch::default();
        scratch.build_bg_instances(&frame, 8);
        assert!(modern_screen_builder_bg_row_buckets(
            &frame,
            &scratch.bg_instance_words,
            &mut scratch.reserved_words
        ));
        let pool_after_bg = scratch.reserved_words.len();
        let sprites: Vec<u32> =
            [[0u32, 4, 4, 0, 0, 0, 0, 0], [0, 250, 220, 0, 0, 0, 0, 0]].concat();
        assert!(modern_screen_builder_sprite_buckets(
            &sprites,
            &mut scratch.reserved_words
        ));
        let r = &scratch.reserved_words;
        let cell = |cx: usize, cy: usize| {
            let h = SPRITE_BUCKET_TABLE_WORDS + (cy * SPRITE_BUCKET_GRID_W + cx) * 2;
            r[h] as usize..(r[h] + r[h + 1]) as usize
        };
        assert!(cell(0, 0).start >= pool_after_bg);
        assert_eq!(&r[cell(0, 0)], &[0]);
        assert_eq!(&r[cell(1, 1)], &[0]);
        assert_eq!(&r[cell(31, 27)], &[1]);
        assert!(r[cell(2, 2)].is_empty());
    }
}
