use std::fs;
use std::path::{Path, PathBuf};

use memmap2::Mmap;
use serde::{Deserialize, Serialize};

use crate::RECORD_LEN;

pub const SCHEMA: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub schema: u32,
    pub frames: u32,
    pub rom_sha256: String,
    pub save_sha256: String,
    pub c_oracle_rev: String,
    pub timing_hacks: Vec<String>,
    pub mask: Vec<usize>,
    pub block_size: u32,
    pub page_kb: u32,
}

impl Manifest {
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        fs::write(path, serde_json::to_vec_pretty(self).unwrap())
    }
    pub fn load(path: &Path) -> std::io::Result<Self> {
        Ok(serde_json::from_slice(&fs::read(path)?).expect("manifest json"))
    }
}

pub fn write_rollup(path: &Path, rollups: &[u32]) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(rollups.len() * 4);
    for &r in rollups {
        bytes.extend_from_slice(&r.to_le_bytes());
    }
    fs::write(path, bytes)
}

/// mmap'd read-only view of rollup.bin as a u32 column.
pub struct RollupMap {
    _mmap: Mmap,
    len: usize,
    ptr: *const u8,
}

// SAFETY: ptr points into _mmap which lives as long as self; read-only.
unsafe impl Send for RollupMap {}
unsafe impl Sync for RollupMap {}

impl RollupMap {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let f = fs::File::open(path)?;
        let mmap = unsafe { Mmap::map(&f)? };
        let len = mmap.len() / 4;
        let ptr = mmap.as_ptr();
        Ok(RollupMap { _mmap: mmap, len, ptr })
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn get(&self, i: usize) -> u32 {
        assert!(i < self.len);
        let mut b = [0u8; 4];
        // SAFETY: i < len, mmap covers len*4 bytes.
        unsafe { std::ptr::copy_nonoverlapping(self.ptr.add(i * 4), b.as_mut_ptr(), 4) };
        u32::from_le_bytes(b)
    }
    pub fn to_vec(&self) -> Vec<u32> {
        (0..self.len).map(|i| self.get(i)).collect()
    }
}

fn detail_path(dir: &Path, block_idx: usize) -> PathBuf {
    dir.join(format!("detail/{block_idx:05}.zst"))
}

pub fn write_detail_block(dir: &Path, block_idx: usize, raw: &[u8]) -> std::io::Result<()> {
    let p = detail_path(dir, block_idx);
    fs::create_dir_all(p.parent().unwrap())?;
    let compressed = zstd::encode_all(raw, 10).expect("zstd encode");
    fs::write(p, compressed)
}

pub fn read_detail_block(dir: &Path, block_idx: usize) -> std::io::Result<Vec<u8>> {
    let p = detail_path(dir, block_idx);
    Ok(zstd::decode_all(fs::read(p)?.as_slice()).expect("zstd decode"))
}

/// Number of whole + partial records in a detail block buffer.
pub fn records_in(raw: &[u8]) -> usize {
    raw.len() / RECORD_LEN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollup_roundtrip() {
        let dir = tempdir();
        let p = dir.join("rollup.bin");
        let data: Vec<u32> = (0..1000).map(|i| i * 7).collect();
        write_rollup(&p, &data).unwrap();
        let m = RollupMap::open(&p).unwrap();
        assert_eq!(m.len(), 1000);
        assert_eq!(m.get(13), 91);
        assert_eq!(m.to_vec(), data);
    }

    #[test]
    fn manifest_roundtrip() {
        let dir = tempdir();
        let p = dir.join("manifest.json");
        let man = Manifest {
            schema: SCHEMA, frames: 100, rom_sha256: "a".into(), save_sha256: "b".into(),
            c_oracle_rev: "c".into(), timing_hacks: vec!["X".into()], mask: vec![0x654],
            block_size: 8192, page_kb: 1,
        };
        man.save(&p).unwrap();
        assert_eq!(Manifest::load(&p).unwrap(), man);
    }

    #[test]
    fn detail_roundtrip() {
        let dir = tempdir();
        let raw = vec![0x5au8; RECORD_LEN * 3];
        write_detail_block(&dir, 2, &raw).unwrap();
        assert_eq!(read_detail_block(&dir, 2).unwrap(), raw);
        assert_eq!(records_in(&raw), 3);
    }

    fn tempdir() -> PathBuf {
        let p = std::env::temp_dir().join(format!("parity-test-{}", std::process::id()))
            .join(format!("{:?}", std::time::SystemTime::now()));
        fs::create_dir_all(&p).unwrap();
        p
    }
}
