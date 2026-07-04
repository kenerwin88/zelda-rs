/// GPU sprite (OBJ) renderer for SNES Mode 1.
///
/// Resolves OAM on the CPU into the same per-pixel OBJ buffer used by the CPU
/// renderer, uploads that buffer as a small integer texture, and renders it
/// through priority-filtered GPU passes.
///
/// The prepass is deliberately CPU-side: SNES OBJ selection is not a pure
/// z-order problem. The first nontransparent pixel in OAM order wins before
/// BG/OBJ priority is considered.
///
/// SNES OBJ palettes 0-3 (`CGRAM[0x80..0xbf]`) use the CPU renderer's layer-6
/// sentinel and do not participate in color math. That bit is carried per tile
/// in the uploaded flags so the post-process pass can make the same decision.
///
use crate::gpu_frame::{ObjRegs, ScanlineRegs};
use crate::tile_atlas::{CgramPalette, TileAtlas};

// Per SNES PPU OBSEL: maps obj_size (3-bit index) to [small_px, large_px].
const SPRITE_SIZES: [[u8; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

// Uniform buffer layout:
//   bytes 0–3:    has_pixels
//   bytes 4–7:    math_bit_pos (4=OBJ for main screen, 255=sub-screen)
//   bytes 8–11:   priority_filter (0..3, or 255=all)
//   bytes 12–15:  window_flags (OBJ W1inv, W1en, W2inv, W2en)
//   bytes 16–19:  windowed (whether OBJ uses window masking)
//   bytes 20–31:  padding
//   bytes 32–927: scanline_tm (56 vec4<u32>, 896 bytes; low byte = screen_enabled[0])
//   bytes 928–1823: scanline_window (56 vec4<u32>, packed window bounds)
const SCANLINE_TM_OFFSET: usize = 32;
const SCANLINE_TM_BYTES: usize = 56 * 16; // 896
const SCANLINE_WINDOW_OFFSET: usize = SCANLINE_TM_OFFSET + SCANLINE_TM_BYTES;
const SCANLINE_WINDOW_BYTES: usize = 56 * 16; // 896
const UNIFORM_BYTES: usize = SCANLINE_WINDOW_OFFSET + SCANLINE_WINDOW_BYTES; // 1824
const SCREEN_PASS_COUNT: usize = 2;
const PRIORITY_PASS_COUNT: usize = 5; // priority 0..3, plus 255=all
const MAX_SPRITES_PER_LINE_PLUS_ONE: i32 = 33;
const MAX_SPRITE_TILES_PER_LINE_PLUS_ONE: i32 = 35;
const PPU_BUFFER_EXTRA_LEFT_RIGHT: i32 = 96;
const OBJ_WIDTH: u32 = 256;
const OBJ_HEIGHT: u32 = 224;
const OBJ_PIXEL_BYTES: usize = (OBJ_WIDTH as usize) * (OBJ_HEIGHT as usize) * 4;

pub struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    pixel_tex: wgpu::Texture,
    pixel_buf: Vec<u8>,
    /// [screen][priority-pass], where screen 0 = main and 1 = sub. Separate
    /// buffers prevent deferred `write_buffer` calls from clobbering uniforms
    /// for earlier recorded passes in the same frame.
    uniform_buf: [[wgpu::Buffer; PRIORITY_PASS_COUNT]; SCREEN_PASS_COUNT],
    bind_group: [[wgpu::BindGroup; PRIORITY_PASS_COUNT]; SCREEN_PASS_COUNT],
    has_pixels: bool,
}

impl SpriteRenderer {
    pub fn new(
        device: &wgpu::Device,
        _atlas: &TileAtlas,
        palette: &CgramPalette,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        let pixel_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sprite_pixels"),
            size: wgpu::Extent3d {
                width: OBJ_WIDTH,
                height: OBJ_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let pixel_view = pixel_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let pixel_buf = vec![0u8; OBJ_PIXEL_BYTES];

        let make_buf = |label: &str| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: UNIFORM_BYTES as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let uniform_buf = std::array::from_fn(|screen| {
            std::array::from_fn(|pass| {
                let label = match screen {
                    0 => format!("sprite_uniforms_main_{pass}"),
                    _ => format!("sprite_uniforms_sub_{pass}"),
                };
                make_buf(&label)
            })
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite_bgl"),
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
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let make_bg = |label: &str, buf: &wgpu::Buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&pixel_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&palette.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buf.as_entire_binding(),
                    },
                ],
            })
        };
        let bind_group = std::array::from_fn(|screen| {
            std::array::from_fn(|pass| {
                let label = match screen {
                    0 => format!("sprite_bg_main_{pass}"),
                    _ => format!("sprite_bg_sub_{pass}"),
                };
                make_bg(&label, &uniform_buf[screen][pass])
            })
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("sprite_pixels.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite_pipeline"),
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
            pixel_tex,
            pixel_buf,
            uniform_buf,
            bind_group,
            has_pixels: false,
        }
    }

    /// Resolve the SNES OBJ buffer from OAM. Must be called before `render()`.
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        vram: &[u16],
        oam: &[u16],
        obj: &ObjRegs,
        extra_left_right: u8,
    ) {
        let sizes = SPRITE_SIZES[obj.obj_size as usize & 7];
        self.pixel_buf.fill(0);
        self.has_pixels = resolve_obj_pixels(
            vram,
            oam,
            obj,
            sizes,
            i32::from(extra_left_right),
            &mut self.pixel_buf,
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.pixel_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixel_buf,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(OBJ_WIDTH * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: OBJ_WIDTH,
                height: OBJ_HEIGHT,
                depth_or_array_layers: 1,
            },
        );

        let mut bytes = [0u8; UNIFORM_BYTES];
        bytes[0..4].copy_from_slice(&(self.has_pixels as u32).to_le_bytes());
        for screen_bufs in &self.uniform_buf {
            for buf in screen_bufs {
                queue.write_buffer(buf, 0, &bytes);
            }
        }
    }

    /// Record a render pass drawing all sprites prepared this frame.
    ///
    /// `math_bit_pos`: 4 for main-screen OBJ, 255 for sub-screen (no TM check,
    ///   output alpha=1.0 marks real pixels for sub-screen backdrop detection).
    /// `screen_idx`: command-selected target screen buffer (0=main, 1=sub).
    /// `priority_filter`: sprite priority band to draw (0..3), or 255 for all.
    /// `scanlines`: per-scanline HDMA snapshot used for the per-row TM check.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
        math_bit_pos: u32,
        screen_idx: usize,
        priority_filter: u32,
        window_flags: u32,
        windowed: bool,
        scanlines: &[ScanlineRegs; 224],
    ) {
        if !self.has_pixels {
            return;
        }

        let pass_idx = priority_pass_index(priority_filter);

        // Write math_bit_pos and scanline TM into the command-selected buffer.
        // Tile data was already written to both buffers in prepare().
        let mut hdr = [0u8; UNIFORM_BYTES];
        hdr[4..8].copy_from_slice(&math_bit_pos.to_le_bytes());
        hdr[8..12].copy_from_slice(&priority_filter.to_le_bytes());
        hdr[12..16].copy_from_slice(&window_flags.to_le_bytes());
        hdr[16..20].copy_from_slice(&(windowed as u32).to_le_bytes());
        for (i, sl) in scanlines.iter().enumerate().take(224) {
            hdr[SCANLINE_TM_OFFSET + i * 4] = sl.screen_enabled_main;
        }
        for (i, sl) in scanlines.iter().enumerate().take(224) {
            let packed = u32::from(sl.window1_left)
                | (u32::from(sl.window1_right) << 8)
                | (u32::from(sl.window2_left) << 16)
                | (u32::from(sl.window2_right) << 24);
            let off = SCANLINE_WINDOW_OFFSET + i * 4;
            hdr[off..off + 4].copy_from_slice(&packed.to_le_bytes());
        }
        queue.write_buffer(&self.uniform_buf[screen_idx][pass_idx], 4, &hdr[4..]);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sprite"),
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
        pass.set_bind_group(0, &self.bind_group[screen_idx][pass_idx], &[]);
        pass.draw(0..3, 0..1);
    }
}

fn priority_pass_index(priority_filter: u32) -> usize {
    match priority_filter {
        0..=3 => priority_filter as usize,
        255 => 4,
        other => panic!("invalid sprite priority filter {other}"),
    }
}

#[derive(Clone, Copy)]
struct VisibleSpriteLine {
    sprite_num: usize,
    row: i32,
    x: i32,
    sprite_size: i32,
}

fn resolve_obj_pixels(
    vram: &[u16],
    oam: &[u16],
    obj: &ObjRegs,
    sizes: [u8; 2],
    extra_left_right: i32,
    out: &mut [u8],
) -> bool {
    debug_assert_eq!(out.len(), OBJ_PIXEL_BYTES);
    let mut any = false;

    for line in 1..=OBJ_HEIGHT as i32 {
        let mut sprites_left = MAX_SPRITES_PER_LINE_PLUS_ONE;
        let mut tiles_left = MAX_SPRITE_TILES_PER_LINE_PLUS_ONE;
        let mut sprites = Vec::with_capacity(32);

        for sprite_num in 0..128usize {
            let idx = sprite_num * 2;
            let oam0 = oam.get(idx).copied().unwrap_or(0);
            let yy = (((oam0 >> 8) as i32) + 1) & 0xff;
            if yy == 0xf0 {
                continue;
            }

            let row = (line - yy) & 0xff;
            let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
            let hi_bits = (hi_word >> (idx % 16)) as i32;
            let sprite_size = sizes[((hi_bits >> 1) & 1) as usize] as i32;
            if row >= sprite_size {
                continue;
            }

            let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
            if object_x > 256 && object_x + sprite_size - 1 < 512 {
                continue;
            }

            let mut x = object_x;
            if x >= 256 + extra_left_right {
                x -= 512;
            }
            if x <= -(sprite_size + extra_left_right) {
                continue;
            }

            sprites_left -= 1;
            if sprites_left == 0 {
                break;
            }
            sprites.push(VisibleSpriteLine {
                sprite_num,
                row,
                x,
                sprite_size,
            });
        }

        let out_y = (line - 1) as usize;
        'tiles: for sprite in sprites {
            let idx = sprite.sprite_num * 2;
            let oam1 = oam.get(idx + 1).copied().unwrap_or(0);
            let mut row = sprite.row;
            if oam1 & 0x8000 != 0 {
                row = sprite.sprite_size - 1 - row;
            }
            let obj_addr = if oam1 & 0x0100 != 0 {
                obj.tile_adr2
            } else {
                obj.tile_adr1
            };
            let palette_sub = ((oam1 & 0x0e00) >> 9) as u8;
            let cgram_base = 0x80u16 + 16 * u16::from(palette_sub);
            let priority = ((oam1 & 0x3000) >> 12) as u8;
            let layer_bit_pos = if palette_sub < 4 { 6 } else { 4 };
            let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
            let tile_col_base = (oam1 & 0x0f) as i32;

            let mut col = 0;
            while col < sprite.sprite_size {
                if col + sprite.x <= -8 - extra_left_right
                    || col + sprite.x >= 256 + extra_left_right
                {
                    col += 8;
                    continue;
                }

                tiles_left -= 1;
                if tiles_left == 0 {
                    break 'tiles;
                }

                let used_col = if oam1 & 0x4000 != 0 {
                    sprite.sprite_size - 1 - col
                } else {
                    col
                };
                let used_tile = ((tile_row_base + (row >> 3)) << 4)
                    | ((tile_col_base + (used_col >> 3)) & 0x0f);
                let addr = obj_addr
                    .wrapping_add((used_tile as u16).wrapping_mul(16))
                    .wrapping_add((row & 7) as u16)
                    & 0x7fff;
                let plane = vram_word(vram, addr) as u32
                    | ((vram_word(vram, addr.wrapping_add(8) & 0x7fff) as u32) << 16);
                let px_left = (-(col + sprite.x + PPU_BUFFER_EXTRA_LEFT_RIGHT)).max(0);
                let px_right = (256 + PPU_BUFFER_EXTRA_LEFT_RIGHT - (col + sprite.x)).min(8);
                for px in px_left..px_right {
                    let shift = if oam1 & 0x4000 != 0 { px } else { 7 - px };
                    let bits = plane >> shift;
                    let pixel =
                        (bits & 1) | ((bits >> 7) & 2) | ((bits >> 14) & 4) | ((bits >> 21) & 8);
                    if pixel == 0 {
                        continue;
                    }

                    let out_x = col + sprite.x + px;
                    if !(0..OBJ_WIDTH as i32).contains(&out_x) {
                        continue;
                    }
                    let off = (out_y * OBJ_WIDTH as usize + out_x as usize) * 4;
                    if out[off + 3] != 0 {
                        continue;
                    }

                    out[off] = (cgram_base + pixel as u16) as u8;
                    out[off + 1] = priority;
                    out[off + 2] = layer_bit_pos;
                    out[off + 3] = 1;
                    any = true;
                }
                col += 8;
            }
        }
    }

    any
}

fn vram_word(vram: &[u16], addr: u16) -> u16 {
    vram.get(addr as usize & 0x7fff).copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_sizes_index_0_is_8_16() {
        assert_eq!(SPRITE_SIZES[0], [8, 16]);
    }

    #[test]
    fn oam_parse_basic() {
        // Construct a single sprite at x=10, y=20, tile=0x00, attrs=palette2/no_flip.
        let mut oam = vec![0u16; 0x110];
        oam[0] = (20u16 << 8) | 10u16; // x_low=10, y=20
        oam[1] = (2u16 << 9) << 8; // palette_sub=2, no flip, tile=0
                                   // High byte: sprite 0 → hi_word = oam[0x100], bits 1:0
                                   // size_bit=0 (small=8×8), x_high=0.

        let sizes = SPRITE_SIZES[0];

        // Simulate the first iteration: sprite 0 (processed last = highest priority)
        let idx = 0usize;
        let oam0 = oam[idx];
        let oam1 = oam[idx + 1];
        let hi_word = oam[0x100 + idx / 16];
        let hi_bits = (hi_word >> (idx % 16)) as u32;
        let x_high = hi_bits & 1;
        let size_bit = (hi_bits >> 1) & 1;

        assert_eq!(x_high, 0);
        assert_eq!(size_bit, 0);

        let x_low = (oam0 & 0xFF) as i32;
        let y_pos = ((oam0 >> 8) & 0xFF) as i32;
        assert_eq!(x_low, 10);
        assert_eq!(y_pos, 20);

        let y_screen = (y_pos + 1) & 0xFF;
        assert_ne!(y_screen, 0xF0); // not off-screen

        let sprite_size = sizes[size_bit as usize] as i32;
        assert_eq!(sprite_size, 8); // small sprite

        let tile_byte = (oam1 & 0xFF) as i32;
        assert_eq!(tile_byte, 0);

        assert_eq!(sprite_size, 8); // only 1 sub-tile for 8×8
    }

    #[test]
    fn atlas_slot_base_for_tile_adr1() {
        let tile_adr1: u16 = 0x4000;
        let atlas_slot_base = u32::from(tile_adr1) / 16;
        assert_eq!(atlas_slot_base, 0x400);
    }

    #[test]
    fn uniform_byte_count() {
        assert_eq!(SCANLINE_TM_OFFSET, 32);
        assert_eq!(SCANLINE_WINDOW_OFFSET, 928);
        assert_eq!(UNIFORM_BYTES, 1824);
    }

    #[test]
    fn priority_filter_maps_to_stable_pass_buffers() {
        assert_eq!(priority_pass_index(0), 0);
        assert_eq!(priority_pass_index(3), 3);
        assert_eq!(priority_pass_index(255), 4);
    }

    #[test]
    fn resolves_first_nontransparent_obj_pixel() {
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x0080;

        let mut oam = vec![0u16; 0x110];
        oam[0] = 0;
        oam[1] = 0;

        let obj = ObjRegs {
            tile_adr1: 0,
            tile_adr2: 0,
            obj_size: 0,
        };
        let mut out = vec![0u8; OBJ_PIXEL_BYTES];

        assert!(resolve_obj_pixels(
            &vram,
            &oam,
            &obj,
            SPRITE_SIZES[0],
            0,
            &mut out
        ));

        assert_eq!(out[0], 0x81);
        assert_eq!(out[1], 0);
        assert_eq!(out[2], 6);
        assert_eq!(out[3], 1);
    }
}
