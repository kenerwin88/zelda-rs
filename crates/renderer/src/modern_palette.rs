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
