//! The per-item LZ-ish asset compression used inside the CHR containers.
//!
//! [`decompress_asset`] is a port of `extract_assets.decomp_asset` (full command
//! set, needed to decode donor items). [`compress_literal`] emits the minimal
//! all-literal stream (0xE0 long-copy chunks + 0xFF terminator) that the C engine
//! decodes back to the exact input — used to re-encode edited packs. Round-trip
//! (`decompress_asset(compress_literal(x)) == x`) is guaranteed by construction.

/// Decompress one packed asset item.
pub fn decompress_asset(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut result: Vec<u8> = Vec::new();
    let mut offset = 0usize;
    let get = |i: usize| -> Result<u8, String> {
        data.get(i)
            .copied()
            .ok_or_else(|| "asset decompression ran past end of input".to_string())
    };

    loop {
        let control = get(offset)?;
        offset += 1;
        if control == 0xFF {
            return Ok(result);
        }
        let (cmd, mut length);
        if control & 0xE0 != 0xE0 {
            cmd = control & 0xE0;
            length = (control & 0x1F) as usize;
        } else {
            cmd = (control << 3) & 0xE0;
            length = (((control & 3) as usize) << 8) | get(offset)? as usize;
            offset += 1;
        }
        length += 1;

        if cmd == 0x00 {
            for _ in 0..length {
                let byte = get(offset)?;
                offset += 1;
                result.push(byte);
            }
        } else if cmd & 0x80 != 0 {
            let mut src = get(offset)? as usize | ((get(offset + 1)? as usize) << 8);
            offset += 2;
            for _ in 0..length {
                let byte = *result
                    .get(src)
                    .ok_or_else(|| "asset back-reference out of range".to_string())?;
                result.push(byte);
                src += 1;
            }
        } else if cmd & 0x40 == 0 {
            let value = get(offset)?;
            offset += 1;
            result.extend(std::iter::repeat(value).take(length));
        } else if cmd & 0x20 == 0 {
            let first = get(offset)?;
            let second = get(offset + 1)?;
            offset += 2;
            while length > 0 {
                result.push(first);
                if length == 1 {
                    break;
                }
                result.push(second);
                length -= 2;
            }
        } else {
            let mut value = get(offset)?;
            offset += 1;
            for _ in 0..length {
                result.push(value);
                value = value.wrapping_add(1);
            }
        }
    }
}

/// Encode `payload` as an all-literal compressed stream. Emits 0xE0 long-copy
/// literal chunks (max 1024 bytes each) terminated by 0xFF.
pub fn compress_literal(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + payload.len() / 1024 * 2 + 3);
    for chunk in payload.chunks(1024) {
        let encoded = chunk.len() - 1;
        out.push(0xE0 | (encoded >> 8) as u8);
        out.push((encoded & 0xFF) as u8);
        out.extend_from_slice(chunk);
    }
    out.push(0xFF);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_round_trip() {
        for len in [0usize, 1, 1023, 1024, 1025, 4096, 5000] {
            let payload: Vec<u8> = (0..len).map(|i| ((i * 31 + 7) % 256) as u8).collect();
            let compressed = compress_literal(&payload);
            assert_eq!(*compressed.last().unwrap(), 0xFF);
            assert_eq!(decompress_asset(&compressed).unwrap(), payload, "len {len}");
        }
    }
}
