//! Packed-array container codec (the `kSprGfx`/`kBgGfx` outer format).
//!
//! Layout: `[offset; count-1][item bytes...][marker: u16 LE]`. Offsets are
//! absolute starts into the concatenated payload; the trailing marker encodes
//! the item count and the offset width (2 bytes normally, 4 bytes when the
//! summed length of all-but-last item reaches 64 KiB). Port of
//! `extract_assets.unpack_packed_arrays` and `test_chr_editable_sheets.pack_arrays`.

const WIDE_MARKER_BASE: usize = 8192;

/// Split a packed container into its constituent item byte slices.
pub fn unpack_packed_arrays(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    if data.len() < 2 {
        return Err("packed container is shorter than its 2-byte marker".to_string());
    }
    let marker = u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]) as usize;
    let (count, offset_size) = if marker >= WIDE_MARKER_BASE {
        (marker - WIDE_MARKER_BASE + 1, 4usize)
    } else {
        (marker + 1, 2usize)
    };

    let header = (count - 1) * offset_size;
    if header + 2 > data.len() {
        return Err(format!(
            "packed container header ({header} bytes for {count} items) exceeds {} bytes",
            data.len()
        ));
    }
    let mut offsets = Vec::with_capacity(count - 1);
    for i in 0..count - 1 {
        let pos = i * offset_size;
        let value = if offset_size == 4 {
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize
        } else {
            u16::from_le_bytes([data[pos], data[pos + 1]]) as usize
        };
        offsets.push(value);
    }

    let payload = &data[header..data.len() - 2];
    let mut items = Vec::with_capacity(count);
    let mut start = 0usize;
    for &end in &offsets {
        if end < start || end > payload.len() {
            return Err(format!("packed container offset {end} out of range"));
        }
        items.push(payload[start..end].to_vec());
        start = end;
    }
    items.push(payload[start..].to_vec());
    Ok(items)
}

/// Encode item byte slices into the packed container format. The offset width is
/// chosen deterministically so that `pack_arrays(unpack_packed_arrays(x)) == x`
/// for any container this codec produced.
pub fn pack_arrays(items: &[Vec<u8>]) -> Vec<u8> {
    let all_but_last: usize = items
        .split_last()
        .map(|(_, rest)| rest.iter().map(Vec::len).sum())
        .unwrap_or(0);
    let wide = all_but_last >= 65536;

    let mut out = Vec::new();
    let mut offset = 0usize;
    for item in items.iter().take(items.len().saturating_sub(1)) {
        offset += item.len();
        if wide {
            out.extend_from_slice(&(offset as u32).to_le_bytes());
        } else {
            out.extend_from_slice(&(offset as u16).to_le_bytes());
        }
    }
    for item in items {
        out.extend_from_slice(item);
    }
    let marker = if wide {
        WIDE_MARKER_BASE + items.len() - 1
    } else {
        items.len() - 1
    };
    out.extend_from_slice(&(marker as u16).to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_narrow() {
        let items = vec![vec![1u8, 2, 3], vec![], vec![9, 8, 7, 6], vec![42]];
        let packed = pack_arrays(&items);
        assert_eq!(unpack_packed_arrays(&packed).unwrap(), items);
    }

    #[test]
    fn round_trip_single() {
        let items = vec![vec![5u8; 10]];
        let packed = pack_arrays(&items);
        assert_eq!(unpack_packed_arrays(&packed).unwrap(), items);
    }

    #[test]
    fn round_trip_wide() {
        // Force the 4-byte offset path (all-but-last sum >= 64 KiB).
        let items = vec![vec![7u8; 70_000], vec![1, 2, 3], vec![4, 5]];
        let packed = pack_arrays(&items);
        // Marker high bit set -> wide.
        let marker = u16::from_le_bytes([packed[packed.len() - 2], packed[packed.len() - 1]]);
        assert!(marker as usize >= WIDE_MARKER_BASE);
        assert_eq!(unpack_packed_arrays(&packed).unwrap(), items);
    }
}
