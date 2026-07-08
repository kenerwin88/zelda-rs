//! SNES planar CHR tile codec: 2/3/4bpp packed planes <-> 8x8 index tiles.
//!
//! Port of `scripts/chr_editable_sheets.decode_planar_tile_indices` and its
//! inverse. A tile is 64 palette indices in row-major order.

/// Bytes per planar tile for a given bit depth.
fn stride(bpp: u8) -> Result<usize, String> {
    match bpp {
        2 => Ok(16),
        3 => Ok(24),
        4 => Ok(32),
        other => Err(format!("unsupported SNES tile bit depth: {other}")),
    }
}

/// Decode packed planar CHR bytes into 8x8 palette-index tiles.
pub fn decode_planar_tile_indices(data: &[u8], bpp: u8) -> Result<Vec<[u8; 64]>, String> {
    let stride = stride(bpp)?;
    if data.len() % stride != 0 {
        return Err(format!(
            "{} bytes is not a multiple of {stride} for {bpp}bpp",
            data.len()
        ));
    }

    let mut tiles = Vec::with_capacity(data.len() / stride);
    for tile in data.chunks_exact(stride) {
        let mut pixels = [0u8; 64];
        for y in 0..8 {
            let plane0 = tile[y * 2];
            let plane1 = tile[y * 2 + 1];
            let plane2 = if bpp == 3 { tile[16 + y] } else { 0 };
            let plane2_4 = if bpp == 4 { tile[16 + y * 2] } else { 0 };
            let plane3 = if bpp == 4 { tile[16 + y * 2 + 1] } else { 0 };
            for x in 0..8 {
                let bit = 7 - x;
                let mut value = ((plane0 >> bit) & 1) | (((plane1 >> bit) & 1) << 1);
                if bpp == 3 {
                    value |= ((plane2 >> bit) & 1) << 2;
                } else if bpp == 4 {
                    value |= ((plane2_4 >> bit) & 1) << 2;
                    value |= ((plane3 >> bit) & 1) << 3;
                }
                pixels[y * 8 + x] = value;
            }
        }
        tiles.push(pixels);
    }
    Ok(tiles)
}

/// Encode 8x8 palette-index tiles back into packed planar CHR bytes. Exact
/// inverse of [`decode_planar_tile_indices`]; index bits above the bit depth are
/// dropped (they cannot be represented and never occur for validated sheets).
pub fn encode_planar_tiles(tiles: &[[u8; 64]], bpp: u8) -> Result<Vec<u8>, String> {
    let stride = stride(bpp)?;
    let mut out = vec![0u8; tiles.len() * stride];
    for (tile_index, tile) in tiles.iter().enumerate() {
        let base = tile_index * stride;
        for y in 0..8 {
            let (mut plane0, mut plane1, mut plane2, mut plane2_4, mut plane3) = (0u8, 0, 0, 0, 0);
            for x in 0..8 {
                let bit = 7 - x;
                let value = tile[y * 8 + x];
                plane0 |= (value & 1) << bit;
                plane1 |= ((value >> 1) & 1) << bit;
                if bpp == 3 {
                    plane2 |= ((value >> 2) & 1) << bit;
                } else if bpp == 4 {
                    plane2_4 |= ((value >> 2) & 1) << bit;
                    plane3 |= ((value >> 3) & 1) << bit;
                }
            }
            out[base + y * 2] = plane0;
            out[base + y * 2 + 1] = plane1;
            if bpp == 3 {
                out[base + 16 + y] = plane2;
            } else if bpp == 4 {
                out[base + 16 + y * 2] = plane2_4;
                out[base + 16 + y * 2 + 1] = plane3;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(bpp: u8, tile_count: usize) {
        let stride = stride(bpp).unwrap();
        let max = (1u16 << bpp) as u8;
        let data: Vec<u8> = (0..tile_count * stride)
            .map(|i| ((i * 37 + 11) % 251) as u8)
            .collect();
        let tiles = decode_planar_tile_indices(&data, bpp).unwrap();
        assert_eq!(tiles.len(), tile_count);
        for tile in &tiles {
            assert!(tile.iter().all(|&v| v < max), "index exceeds {bpp}bpp range");
        }
        let re = encode_planar_tiles(&tiles, bpp).unwrap();
        assert_eq!(re, data, "{bpp}bpp planar round-trip must be byte-identical");
    }

    #[test]
    fn round_trip_all_depths() {
        round_trip(2, 5);
        round_trip(3, 7);
        round_trip(4, 3);
    }

    #[test]
    fn decode_single_2bpp_tile_top_left_bit() {
        // Mirrors the Python test: plane0 = 0x80 -> pixel (0,0) == index 1.
        let mut data = vec![0u8; 16];
        data[0] = 0x80;
        let tiles = decode_planar_tile_indices(&data, 2).unwrap();
        assert_eq!(tiles[0][0], 1);
        assert!(tiles[0][1..].iter().all(|&v| v == 0));
    }

    #[test]
    fn rejects_bad_bpp_and_length() {
        assert!(decode_planar_tile_indices(&[0; 16], 5).is_err());
        assert!(decode_planar_tile_indices(&[0; 15], 2).is_err());
    }
}
