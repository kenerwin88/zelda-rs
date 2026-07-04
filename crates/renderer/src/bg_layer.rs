/// GPU pipeline for rendering one SNES 4bpp or 2bpp BG layer (Phase 1b).
///
/// Each `BgLayerRenderer` draws a single BG layer to an `Rgba8Unorm`
/// intermediate texture. Transparent pixels (palette index 0) are discarded.
/// The caller composites multiple layers back-to-front using `load_op`.
///
/// Priority rendering: call `render()` twice — once with `hi_priority_only=false`
/// (lo-priority tiles) and once with `hi_priority_only=true` (hi-priority tiles),
/// sandwiching higher-numbered layers in between to match SNES priority order.
use crate::gpu_frame::{BgLayerRegs, ScanlineRegs};
use crate::tile_atlas::{CgramPalette, RgbaTileOverrideTextures, TileAtlas};

// Maximum tilemap dimensions (64×64 tiles when both tilemap_wider and
// tilemap_higher are set). Standard is 32×32.
const TILEMAP_MAX_TILES: usize = 64 * 64; // 4096
const TILEMAP_MAX_BYTES: usize = TILEMAP_MAX_TILES * 2; // 8192

// Scanline data: 224 scanlines packed as 56 vec4<u32>, 4 bytes per scanline.
const SCANLINE_COUNT: usize = 224;
const SCANLINE_TM_BYTES: usize = 56 * 16; // 56 vec4<u32> = 896 bytes
const SCANLINE_SCROLL_BYTES: usize = 56 * 16; // h_scroll | (v_scroll << 16)
const SCANLINE_WINDOW_BYTES: usize = 56 * 16; // w1l | w1r<<8 | w2l<<16 | w2r<<24

// Uniform buffer layout:
//   bytes 0–63:   scalar params (16 u32s: h_scroll, v_scroll, atlas_slot_base,
//                 tilemap_width, tilemap_height, is_2bpp, hi_priority_only,
//                 layer_bit, math_bit_pos, window_flags, windowed,
//                 mosaic_enabled, mosaic_size, padding x3)
//   bytes 64–959: scanline_tm (56 vec4<u32>, 896 bytes)
//   bytes 960–1855: scanline_scroll (56 vec4<u32>, h | v<<16)
//   bytes 1856–2751: scanline_window (56 vec4<u32>, packed window bounds)
//   bytes 2752–10943: tilemap_data (512 vec4<u32>, 8192 bytes)
const HEADER_BYTES: usize = 64;
const SCANLINE_SCROLL_OFFSET: usize = HEADER_BYTES + SCANLINE_TM_BYTES; // 960
const SCANLINE_WINDOW_OFFSET: usize = SCANLINE_SCROLL_OFFSET + SCANLINE_SCROLL_BYTES; // 1856
const TILEMAP_OFFSET: usize = SCANLINE_WINDOW_OFFSET + SCANLINE_WINDOW_BYTES; // 2752
const UNIFORM_BYTES: usize = TILEMAP_OFFSET + TILEMAP_MAX_BYTES; // 10944

pub struct BgLayerRenderer {
    pipeline: wgpu::RenderPipeline,
    /// Separate uniform buffers for [screen][priority] passes. `screen` is
    /// 0=main, 1=sub; `priority` is 0=lo, 1=hi.
    uniform_buf: [[wgpu::Buffer; 2]; 2],
    bind_group: [[wgpu::BindGroup; 2]; 2],
}

impl BgLayerRenderer {
    pub fn new(
        device: &wgpu::Device,
        atlas: &TileAtlas,
        palette: &CgramPalette,
        rgba_overrides: &RgbaTileOverrideTextures,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        let make_uniform_buf = |label| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: UNIFORM_BYTES as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let uniform_buf = std::array::from_fn(|screen| {
            std::array::from_fn(|priority| {
                let label = match (screen, priority) {
                    (0, 0) => "bg_uniforms_main_lo",
                    (0, _) => "bg_uniforms_main_hi",
                    (_, 0) => "bg_uniforms_sub_lo",
                    (_, _) => "bg_uniforms_sub_hi",
                };
                make_uniform_buf(label)
            })
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bg_layer_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Binding 5: reference CGRAM the HD override art was authored against
                // (for palette-responsive "detail-modulated" recolor). See
                // `bg_layer.wgsl::sample_tile_override` / RgbaTileOverrideData.
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let make_bind_group = |label, buf: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&atlas.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&palette.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&rgba_overrides.atlas_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&rgba_overrides.lookup_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(
                            &rgba_overrides.reference_cgram_view,
                        ),
                    },
                ],
            })
        };
        let bind_group = std::array::from_fn(|screen| {
            std::array::from_fn(|priority| {
                let label = match (screen, priority) {
                    (0, 0) => "bg_layer_bg_main_lo",
                    (0, _) => "bg_layer_bg_main_hi",
                    (_, 0) => "bg_layer_bg_sub_lo",
                    (_, _) => "bg_layer_bg_sub_hi",
                };
                make_bind_group(label, &uniform_buf[screen][priority])
            })
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("bg_layer.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bg_layer_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform_buf,
            bind_group,
        }
    }

    /// Upload this layer's uniforms+tilemap and record a render pass.
    ///
    /// `layer_bit`: the TM bit for this layer (e.g. 1 for BG1). Set to 0 to
    ///   skip the per-scanline TM check (used for sub-screen renders).
    /// `math_bit_pos`: SNES math_enabled bit position (0=BG1, 2=BG3, etc.).
    ///   Pass 255 for sub-screen renders where alpha=1.0 marks real pixels.
    /// `screen_idx`: command-selected target screen buffer (0=main, 1=sub).
    /// `scanlines`: per-scanline HDMA register snapshot from `ppu_scanline_windows`.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        vram: &[u16],
        layer_idx: usize,
        layer: &BgLayerRegs,
        output_view: &wgpu::TextureView,
        load_op: wgpu::LoadOp<wgpu::Color>,
        is_2bpp: bool,
        hi_priority_only: bool,
        layer_bit: u32,
        math_bit_pos: u32,
        screen_idx: usize,
        window_flags: u32,
        windowed: bool,
        mosaic_enabled: bool,
        mosaic_size: u8,
        scanlines: &[ScanlineRegs; 224],
    ) {
        let buf_idx = usize::from(hi_priority_only);
        self.write_uniforms(
            queue,
            &self.uniform_buf[screen_idx][buf_idx],
            vram,
            u32::from(layer.h_scroll),
            u32::from(layer.v_scroll),
            u32::from(layer.tile_adr) / 16,
            layer.tilemap_adr,
            layer.tilemap_wider,
            layer.tilemap_higher,
            is_2bpp,
            hi_priority_only,
            layer_bit,
            math_bit_pos,
            window_flags,
            windowed,
            mosaic_enabled,
            mosaic_size,
            scanlines,
            layer_idx,
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("bg_layer"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group[screen_idx][buf_idx], &[]);
        pass.draw(0..3, 0..1);
    }

    #[allow(clippy::too_many_arguments)]
    fn write_uniforms(
        &self,
        queue: &wgpu::Queue,
        buf: &wgpu::Buffer,
        vram: &[u16],
        h_scroll: u32,
        v_scroll: u32,
        atlas_slot_base: u32,
        tilemap_adr: u16,
        tilemap_wider: bool,
        tilemap_higher: bool,
        is_2bpp: bool,
        hi_priority_only: bool,
        layer_bit: u32,
        math_bit_pos: u32,
        window_flags: u32,
        windowed: bool,
        mosaic_enabled: bool,
        mosaic_size: u8,
        scanlines: &[ScanlineRegs; 224],
        scroll_layer_idx: usize,
    ) {
        let tilemap_w = if tilemap_wider { 64u32 } else { 32u32 };
        let tilemap_h = if tilemap_higher { 64u32 } else { 32u32 };

        let mut bytes = [0u8; UNIFORM_BYTES];

        // Header (48 bytes = 12 u32s).
        bytes[0..4].copy_from_slice(&h_scroll.to_le_bytes());
        bytes[4..8].copy_from_slice(&v_scroll.to_le_bytes());
        bytes[8..12].copy_from_slice(&atlas_slot_base.to_le_bytes());
        bytes[12..16].copy_from_slice(&tilemap_w.to_le_bytes());
        bytes[16..20].copy_from_slice(&tilemap_h.to_le_bytes());
        bytes[20..24].copy_from_slice(&(is_2bpp as u32).to_le_bytes());
        bytes[24..28].copy_from_slice(&(hi_priority_only as u32).to_le_bytes());
        bytes[28..32].copy_from_slice(&layer_bit.to_le_bytes());
        bytes[32..36].copy_from_slice(&math_bit_pos.to_le_bytes());
        bytes[36..40].copy_from_slice(&window_flags.to_le_bytes());
        bytes[40..44].copy_from_slice(&(windowed as u32).to_le_bytes());
        bytes[44..48].copy_from_slice(&(mosaic_enabled as u32).to_le_bytes());
        bytes[48..52].copy_from_slice(&u32::from(mosaic_size).to_le_bytes());
        // bytes[52..64] = zero padding

        // Scanline TM data: one u32 per scanline, low byte = TM.
        for (i, sl) in scanlines.iter().enumerate().take(SCANLINE_COUNT) {
            let off = HEADER_BYTES + i * 4;
            bytes[off] = sl.screen_enabled_main;
            // bytes[off+1..off+4] = 0 (already zero)
        }

        // Per-scanline scroll for this layer.
        for (i, sl) in scanlines.iter().enumerate().take(SCANLINE_COUNT) {
            let packed = u32::from(sl.bg_h_scroll[scroll_layer_idx])
                | (u32::from(sl.bg_v_scroll[scroll_layer_idx]) << 16);
            let off = SCANLINE_SCROLL_OFFSET + i * 4;
            bytes[off..off + 4].copy_from_slice(&packed.to_le_bytes());
        }

        // Per-scanline window bounds.
        for (i, sl) in scanlines.iter().enumerate().take(SCANLINE_COUNT) {
            let packed = u32::from(sl.window1_left)
                | (u32::from(sl.window1_right) << 8)
                | (u32::from(sl.window2_left) << 16)
                | (u32::from(sl.window2_right) << 24);
            let off = SCANLINE_WINDOW_OFFSET + i * 4;
            bytes[off..off + 4].copy_from_slice(&packed.to_le_bytes());
        }

        // Tilemap data.
        pack_tilemap(
            vram,
            usize::from(tilemap_adr),
            tilemap_wider,
            tilemap_higher,
            &mut bytes[TILEMAP_OFFSET..],
        );

        queue.write_buffer(buf, 0, &bytes);
    }
}

/// Pack a tilemap from VRAM into the flat byte array for the shader uniform.
///
/// SNES tilemaps consist of one to four 32×32 pages in VRAM:
/// - base+0     : rows 0–31, cols 0–31
/// - base+1024  : rows 0–31, cols 32–63 (tilemap_wider=true)
/// - base+2048  : rows 32–63, cols 0–31 (tilemap_higher=true)
/// - base+3072  : rows 32–63, cols 32–63 (both)
fn pack_tilemap(vram: &[u16], base: usize, wider: bool, higher: bool, out: &mut [u8]) {
    debug_assert_eq!(out.len(), TILEMAP_MAX_BYTES);
    out.fill(0);

    let width = if wider { 64usize } else { 32usize };
    let height = if higher { 64usize } else { 32usize };
    let pages_wide = if wider { 2usize } else { 1usize };

    for ty in 0..height {
        for tx in 0..width {
            let page_offset = (ty / 32 * pages_wide + tx / 32) * 1024;
            let vram_idx = base + page_offset + (ty % 32) * 32 + tx % 32;
            let entry = vram.get(vram_idx).copied().unwrap_or(0);
            let flat = ty * width + tx;
            let [lo, hi] = entry.to_le_bytes();
            out[flat * 2] = lo;
            out[flat * 2 + 1] = hi;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_tilemap_little_endian() {
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x1234;
        vram[1] = 0x5678;
        let mut out = vec![0u8; TILEMAP_MAX_BYTES];
        pack_tilemap(&vram, 0, false, false, &mut out);
        assert_eq!(out[0..4], [0x34, 0x12, 0x78, 0x56]);
    }

    #[test]
    fn pack_tilemap_at_offset() {
        let mut vram = vec![0u16; 0x8000];
        vram[0x200] = 0xABCD;
        let mut out = vec![0u8; TILEMAP_MAX_BYTES];
        pack_tilemap(&vram, 0x200, false, false, &mut out);
        assert_eq!(out[0..2], [0xCD, 0xAB]);
        assert_eq!(out[2..4], [0x00, 0x00]);
    }

    #[test]
    fn pack_tilemap_oob_zeroed() {
        let vram = vec![0xFFFFu16; 0x7FFF];
        let mut out = vec![0u8; TILEMAP_MAX_BYTES];
        pack_tilemap(&vram, 0x7F00, false, false, &mut out);
        assert_eq!(u16::from_le_bytes([out[0], out[1]]), 0xFFFF);
        let oob_offset = (0x8000 - 0x7F00) * 2;
        if oob_offset < TILEMAP_MAX_BYTES {
            assert_eq!(
                u16::from_le_bytes([out[oob_offset], out[oob_offset + 1]]),
                0
            );
        }
    }

    #[test]
    fn pack_tilemap_higher_second_page() {
        let mut vram = vec![0u16; 0x8000];
        vram[1024] = 0xBEEF;
        let mut out = vec![0u8; TILEMAP_MAX_BYTES];
        pack_tilemap(&vram, 0, false, true, &mut out);
        assert_eq!(
            u16::from_le_bytes([out[1024 * 2], out[1024 * 2 + 1]]),
            0xBEEF
        );
    }

    #[test]
    fn pack_tilemap_wider_interleaved() {
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x1111;
        vram[1024] = 0x2222;
        let mut out = vec![0u8; TILEMAP_MAX_BYTES];
        pack_tilemap(&vram, 0, true, false, &mut out);
        assert_eq!(u16::from_le_bytes([out[0], out[1]]), 0x1111);
        assert_eq!(u16::from_le_bytes([out[32 * 2], out[32 * 2 + 1]]), 0x2222);
    }

    #[test]
    fn uniform_size() {
        assert_eq!(UNIFORM_BYTES, 10944);
        assert_eq!(SCANLINE_SCROLL_OFFSET, 960);
        assert_eq!(SCANLINE_WINDOW_OFFSET, 1856);
        assert_eq!(TILEMAP_OFFSET, 2752);
    }
}
