/// SNES CGRAM BGR15 -> RGBA8, byte-exact with classic `cgram_to_wgpu_color`
/// (each 5-bit channel left-shifted by 3). Index/entry alpha always 0xff.
pub fn snes_cgram_to_rgba(entry: u16) -> [u8; 4] {
    let r = ((entry & 0x1f) as u8) << 3;
    let g = (((entry >> 5) & 0x1f) as u8) << 3;
    let b = (((entry >> 10) & 0x1f) as u8) << 3;
    [r, g, b, 0xff]
}

pub fn cgram_words_to_rgba256(cgram: &[u16]) -> [[u8; 4]; 256] {
    let mut out = [[0, 0, 0, 0xff]; 256];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = snes_cgram_to_rgba(cgram.get(i).copied().unwrap_or(0));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cgram_conversion_matches_classic_formula() {
        assert_eq!(snes_cgram_to_rgba(0x0000), [0, 0, 0, 0xff]);
        assert_eq!(snes_cgram_to_rgba(0x7fff), [248, 248, 248, 0xff]); // 31<<3 = 248
        assert_eq!(snes_cgram_to_rgba(0x001f), [248, 0, 0, 0xff]); // R=31
        assert_eq!(snes_cgram_to_rgba(0x7c00), [0, 0, 248, 0xff]); // B=31
        assert_eq!(snes_cgram_to_rgba(0x03e0), [0, 248, 0, 0xff]); // G=31
        let pal = cgram_words_to_rgba256(&[0x001f, 0x7c00]);
        assert_eq!(pal[0], [248, 0, 0, 0xff]);
        assert_eq!(pal[1], [0, 0, 248, 0xff]);
        assert_eq!(pal[255], [0, 0, 0, 0xff]); // missing entries default to opaque black
    }
}

fn decode_cgram_entry(entry: u16, dst: &mut [u8]) {
    // 5-bit channels shifted left 3 to fill the top 5 bits of an 8-bit value.
    // This matches the SNES hardware output range (0, 8, 16, ... 248).
    dst[0] = ((entry & 0x1F) as u8) << 3; // R
    dst[1] = (((entry >> 5) & 0x1F) as u8) << 3; // G
    dst[2] = (((entry >> 10) & 0x1F) as u8) << 3; // B
    dst[3] = 0xFF;
}

/// Expand a CGRAM (up to 256 `u16` BGR555 entries) into a flat 256×RGBA8 buffer
/// (`1024` bytes), using the exact same channel expansion as the live `cgram_palette`
/// texture. This is the canonical way to snapshot a CGRAM as an HD "reference palette"
/// PNG: art authored by rendering under `expand_cgram_to_rgba8(cgram)` recolors to the
/// live palette exactly (detail == 1) under the shader's detail-modulate path. Entries
/// past `cgram.len()` (or beyond 256) are left transparent-black.
pub fn expand_cgram_to_rgba8(cgram: &[u16]) -> Vec<u8> {
    let mut out = vec![0u8; 256 * 4];
    for (i, &entry) in cgram.iter().take(256).enumerate() {
        decode_cgram_entry(entry, &mut out[i * 4..i * 4 + 4]);
    }
    out
}
