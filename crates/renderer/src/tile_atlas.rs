/// GPU tile atlas and CGRAM palette textures, decoded from VRAM each frame.
///
/// ## Atlas layout
///
/// VRAM is decoded as 4bpp tiles (the dominant format for ALttP Mode 1 BGs and
/// sprites). All 2048 possible tile slots are decoded regardless of which are
/// actually referenced by tilemaps, giving a complete 512×256 atlas texture in
/// `R8Uint` format. Each texel stores the raw palette index (0–15); palette
/// colour lookup happens in the BG layer shader via the CGRAM palette texture.
///
/// Tile slot N occupies atlas pixels:
/// - x: `(N % 64) * 8 .. (N % 64) * 8 + 8`
/// - y: `(N / 64) * 8 .. (N / 64) * 8 + 8`
///
/// The BG shader computes the slot from the tilemap entry:
/// `slot = bg.tile_adr / 16 + tile_num` (for 4bpp; tile_adr is in VRAM words).
///
/// ## CGRAM palette
///
/// A 256×1 `Rgba8Unorm` texture decoded from CGRAM. Each texel is the full
/// RGBA colour for that palette entry. Index 0 within any sub-palette is
/// conventionally transparent in the BG layer shader.

// ── Constants ─────────────────────────────────────────────────────────────────

pub const ATLAS_TILES_WIDE: u32 = 64;
pub const ATLAS_TILES_TALL: u32 = 32;
/// Width of the tile atlas in pixels.
pub const ATLAS_WIDTH: u32 = ATLAS_TILES_WIDE * 8;
/// Height of the tile atlas in pixels.
pub const ATLAS_HEIGHT: u32 = ATLAS_TILES_TALL * 8;
/// Total number of 4bpp tile slots encoded in the atlas (= all of 64 KB VRAM).
pub const ATLAS_TILE_COUNT: u32 = ATLAS_TILES_WIDE * ATLAS_TILES_TALL;
pub const RGBA_TILE_OVERRIDE_LOOKUP_COUNT: usize = 1024;

#[derive(Clone, Copy)]
pub struct RgbaTileOverrideData<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
    pub lookup: &'a [[u32; 4]],
}

pub struct RgbaTileOverrideTextures {
    _atlas: wgpu::Texture,
    pub atlas_view: wgpu::TextureView,
    _lookup: wgpu::Texture,
    pub lookup_view: wgpu::TextureView,
}

impl RgbaTileOverrideTextures {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: Option<RgbaTileOverrideData<'_>>,
    ) -> Self {
        let fallback_lookup = [[0u32; 4]; RGBA_TILE_OVERRIDE_LOOKUP_COUNT];
        let fallback_rgba = [0u8; 4];
        let (width, height, rgba, lookup) = match data {
            Some(data) => (data.width, data.height, data.rgba, data.lookup),
            None => (1, 1, fallback_rgba.as_slice(), fallback_lookup.as_slice()),
        };
        debug_assert_eq!(rgba.len(), (width * height * 4) as usize);
        debug_assert_eq!(lookup.len(), RGBA_TILE_OVERRIDE_LOOKUP_COUNT);

        let atlas = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rgba_tile_override_atlas"),
            size: wgpu::Extent3d {
                width,
                height,
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
                texture: &atlas,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor::default());

        let mut lookup_bytes = Vec::with_capacity(RGBA_TILE_OVERRIDE_LOOKUP_COUNT * 16);
        for entry in lookup {
            for word in entry {
                lookup_bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        let lookup_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rgba_tile_override_lookup"),
            size: wgpu::Extent3d {
                width: RGBA_TILE_OVERRIDE_LOOKUP_COUNT as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &lookup_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &lookup_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(RGBA_TILE_OVERRIDE_LOOKUP_COUNT as u32 * 16),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: RGBA_TILE_OVERRIDE_LOOKUP_COUNT as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let lookup_view = lookup_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            _atlas: atlas,
            atlas_view,
            _lookup: lookup_texture,
            lookup_view,
        }
    }
}

// ── TileAtlas ─────────────────────────────────────────────────────────────────

/// GPU texture holding decoded tile palette indices for the entire VRAM.
pub struct TileAtlas {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    last_hash: u32,
    pixel_buf: Vec<u8>,
}

impl TileAtlas {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tile_atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let pixel_buf = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize];
        Self {
            texture,
            texture_view,
            last_hash: 0,
            pixel_buf,
        }
    }

    /// Decode all 4bpp tiles from `vram` into the atlas texture, skipping the
    /// upload if VRAM is unchanged since the last call (FNV-1a hash check).
    pub fn update(&mut self, queue: &wgpu::Queue, vram: &[u16]) {
        let hash = fnv32_u16(vram);
        if hash == self.last_hash {
            return;
        }
        self.last_hash = hash;

        decode_4bpp_atlas(vram, &mut self.pixel_buf);

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixel_buf,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ATLAS_WIDTH),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
        );
    }
}

// ── CgramPalette ──────────────────────────────────────────────────────────────

/// GPU texture holding the decoded CGRAM colour palette.
///
/// A 256×1 `Rgba8Unorm` texture. Texel `i` is the RGBA colour for CGRAM entry
/// `i`. The BG shader computes `cgram_index = palette_base * 16 + atlas_index`
/// and samples this texture for the final pixel colour.
pub struct CgramPalette {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    last_hash: u32,
    pixel_buf: Vec<u8>,
}

impl CgramPalette {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cgram_palette"),
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
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let pixel_buf = vec![0u8; 256 * 4];
        Self {
            texture,
            texture_view,
            last_hash: 0,
            pixel_buf,
        }
    }

    /// Decode CGRAM 15-bit BGR entries into the RGBA palette texture, skipping
    /// the upload if CGRAM is unchanged since the last call.
    pub fn update(&mut self, queue: &wgpu::Queue, cgram: &[u16]) {
        let hash = fnv32_u16(cgram);
        if hash == self.last_hash {
            return;
        }
        self.last_hash = hash;

        for (i, &entry) in cgram.iter().enumerate().take(256) {
            decode_cgram_entry(entry, &mut self.pixel_buf[i * 4..i * 4 + 4]);
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixel_buf,
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
    }
}

// ── Decode helpers ────────────────────────────────────────────────────────────

/// Decode all 2048 4bpp tile slots from `vram` into `out` (one byte per pixel,
/// value = palette index 0–15). `out` must be `ATLAS_WIDTH * ATLAS_HEIGHT` bytes.
///
/// SNES 4bpp tile layout in VRAM (16 words per tile):
/// - Words 0–7 : pairs (bp0_row[y], bp1_row[y]) packed as lo/hi bytes per word.
/// - Words 8–15: pairs (bp2_row[y], bp3_row[y]) packed the same way.
pub(crate) fn decode_4bpp_atlas(vram: &[u16], out: &mut [u8]) {
    debug_assert_eq!(out.len(), (ATLAS_WIDTH * ATLAS_HEIGHT) as usize);

    for tile_n in 0..ATLAS_TILE_COUNT as usize {
        let base = tile_n * 16; // VRAM word base for this tile slot
        let atlas_ox = (tile_n % ATLAS_TILES_WIDE as usize) * 8;
        let atlas_oy = (tile_n / ATLAS_TILES_WIDE as usize) * 8;

        for py in 0..8usize {
            // Two words per row: one for bitplanes 0+1, one for 2+3.
            let w01 = vram.get(base + py).copied().unwrap_or(0);
            let w23 = vram.get(base + 8 + py).copied().unwrap_or(0);

            let bp0 = (w01 & 0xFF) as u8;
            let bp1 = (w01 >> 8) as u8;
            let bp2 = (w23 & 0xFF) as u8;
            let bp3 = (w23 >> 8) as u8;

            let row_start = (atlas_oy + py) * ATLAS_WIDTH as usize + atlas_ox;
            for px in 0..8usize {
                let bit = 7 - px; // MSB = pixel 0
                let idx = ((bp0 >> bit) & 1)
                    | (((bp1 >> bit) & 1) << 1)
                    | (((bp2 >> bit) & 1) << 2)
                    | (((bp3 >> bit) & 1) << 3);
                out[row_start + px] = idx;
            }
        }
    }
}

/// Decode a single CGRAM entry (15-bit `0bBBBBBGGGGGRRRRR`) into `dst[0..4]`
/// as `[R, G, B, 0xFF]` (RGBA).
fn decode_cgram_entry(entry: u16, dst: &mut [u8]) {
    // 5-bit channels shifted left 3 to fill the top 5 bits of an 8-bit value.
    // This matches the SNES hardware output range (0, 8, 16, … 248).
    dst[0] = ((entry & 0x1F) as u8) << 3; // R
    dst[1] = (((entry >> 5) & 0x1F) as u8) << 3; // G
    dst[2] = (((entry >> 10) & 0x1F) as u8) << 3; // B
    dst[3] = 0xFF;
}

/// FNV-1a hash over a `u16` slice (little-endian byte order).
pub(crate) fn fnv32_u16(data: &[u16]) -> u32 {
    let mut hash = 2166136261u32;
    for &word in data {
        let [lo, hi] = word.to_le_bytes();
        hash = hash.wrapping_mul(16777619) ^ u32::from(lo);
        hash = hash.wrapping_mul(16777619) ^ u32::from(hi);
    }
    hash
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_4bpp_all_zero_vram() {
        let vram = vec![0u16; 0x8000];
        let mut buf = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize];
        decode_4bpp_atlas(&vram, &mut buf);
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn decode_4bpp_bp0_row0_all_ones() {
        // Tile 0, bitplane-0 row 0 = 0xFF → all 8 pixels have index bit0=1 → index=1.
        // All other planes/rows = 0.
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x00FF; // lo byte = bp0_row0 = 0xFF, hi byte = bp1_row0 = 0
        let mut buf = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize];
        decode_4bpp_atlas(&vram, &mut buf);
        for px in 0..8 {
            assert_eq!(buf[px], 1, "tile0 row0 px={px}");
        }
        // Row 1 should be 0.
        for px in 0..8 {
            assert_eq!(buf[ATLAS_WIDTH as usize + px], 0, "tile0 row1 px={px}");
        }
    }

    #[test]
    fn decode_4bpp_all_planes_set() {
        // Tile 1 (word base=16): all four bitplanes fully set → all pixels index=15.
        let mut vram = vec![0u16; 0x8000];
        for row in 0..8 {
            vram[16 + row] = 0xFFFF; // bp0+bp1 both 0xFF
            vram[16 + 8 + row] = 0xFFFF; // bp2+bp3 both 0xFF
        }
        let mut buf = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize];
        decode_4bpp_atlas(&vram, &mut buf);
        let tile1_x = 8; // tile 1 is at atlas column 1
        for py in 0..8 {
            for px in 0..8 {
                let pos = py * ATLAS_WIDTH as usize + tile1_x + px;
                assert_eq!(buf[pos], 15, "tile1 py={py} px={px}");
            }
        }
    }

    #[test]
    fn decode_4bpp_pixel_order_msb_first() {
        // Tile 0, bp0 row 0 = 0x80 → only pixel 0 (leftmost) has index 1.
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x0080;
        let mut buf = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize];
        decode_4bpp_atlas(&vram, &mut buf);
        assert_eq!(buf[0], 1, "pixel 0 should be 1");
        for px in 1..8 {
            assert_eq!(buf[px], 0, "pixel {px} should be 0");
        }
    }

    #[test]
    fn cgram_red_entry() {
        // 15-bit BGR: R=31, G=0, B=0 → entry = 0x001F
        let mut dst = [0u8; 4];
        decode_cgram_entry(0x001F, &mut dst);
        assert_eq!(dst, [248, 0, 0, 255]); // 31 << 3 = 248
    }

    #[test]
    fn cgram_white_entry() {
        // 15-bit BGR: R=31, G=31, B=31 → entry = 0x7FFF
        let mut dst = [0u8; 4];
        decode_cgram_entry(0x7FFF, &mut dst);
        assert_eq!(dst, [248, 248, 248, 255]);
    }

    #[test]
    fn cgram_blue_entry() {
        // 15-bit BGR: R=0, G=0, B=31 → entry = 0x7C00
        let mut dst = [0u8; 4];
        decode_cgram_entry(0x7C00, &mut dst);
        assert_eq!(dst, [0, 0, 248, 255]);
    }

    #[test]
    fn atlas_dimensions() {
        assert_eq!(ATLAS_WIDTH, 512);
        assert_eq!(ATLAS_HEIGHT, 256);
        assert_eq!(ATLAS_TILE_COUNT, 2048);
    }
}
