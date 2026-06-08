//! Asset helpers for generated Zelda 3 runtime asset files.
//!
//! Asset packs stay outside git. Builders provide an original ROM,
//! `scripts/extract_assets.py` creates `generated/zelda3_assets/`, and
//! `zelda3-bin` embeds the split files into the standalone executable.

#![allow(dead_code)]

pub const GENERATED_ASSET_DIR: &str = "generated/zelda3_assets";
pub const ASSET_FILES_DIR: &str = "assets";
pub const MANIFEST_FILE: &str = "manifest.json";

const ASSET_SIGNATURE_PREFIX: &[u8; 16] = b"Zelda3_v0     \n\0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetPackInfo {
    pub count: usize,
    pub key_signature_len: usize,
    pub sizes: Vec<usize>,
}

pub fn parse_asset_pack_info(data: &[u8]) -> Result<AssetPackInfo, String> {
    if data.len() < 88 || &data[..16] != ASSET_SIGNATURE_PREFIX {
        return Err("invalid zelda3_assets.dat signature".to_string());
    }

    let count = read_le_u32(data, 80)? as usize;
    let key_signature_len = read_le_u32(data, 84)? as usize;
    let sizes_start = 88usize;
    let key_sig_start = sizes_start
        .checked_add(count.checked_mul(4).ok_or("asset count overflow")?)
        .ok_or("asset header overflow")?;
    let mut offset = key_sig_start
        .checked_add(key_signature_len)
        .ok_or("asset key signature overflow")?;
    if key_sig_start > data.len() || offset > data.len() {
        return Err("asset header extends past file".to_string());
    }

    let mut sizes = Vec::with_capacity(count);
    for i in 0..count {
        let size = read_le_u32(data, sizes_start + i * 4)? as usize;
        offset = (offset + 3) & !3;
        let end = offset.checked_add(size).ok_or("asset range overflow")?;
        if end > data.len() {
            return Err("asset range extends past file".to_string());
        }
        sizes.push(size);
        offset = end;
    }

    Ok(AssetPackInfo {
        count,
        key_signature_len,
        sizes,
    })
}

fn read_le_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| "asset header truncated".to_string())?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_pack(sizes: &[usize]) -> Vec<u8> {
        let mut data = vec![0u8; 88 + sizes.len() * 4];
        data[..16].copy_from_slice(ASSET_SIGNATURE_PREFIX);
        data[80..84].copy_from_slice(&(sizes.len() as u32).to_le_bytes());
        data[84..88].copy_from_slice(&0u32.to_le_bytes());
        for (i, size) in sizes.iter().enumerate() {
            data[88 + i * 4..92 + i * 4].copy_from_slice(&(*size as u32).to_le_bytes());
        }
        for size in sizes {
            while data.len() % 4 != 0 {
                data.push(0);
            }
            data.extend(std::iter::repeat(0xaa).take(*size));
        }
        data
    }

    #[test]
    fn parses_asset_count_and_sizes() {
        let pack = synthetic_pack(&[3, 0, 5]);
        let info = parse_asset_pack_info(&pack).unwrap();

        assert_eq!(info.count, 3);
        assert_eq!(info.sizes, vec![3, 0, 5]);
    }

    #[test]
    fn rejects_truncated_asset_data() {
        let mut pack = synthetic_pack(&[8]);
        pack.truncate(pack.len() - 1);

        assert!(parse_asset_pack_info(&pack).is_err());
    }
}
