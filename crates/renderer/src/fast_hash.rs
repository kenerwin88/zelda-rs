//! Small non-cryptographic hasher for the renderer's hot per-tile lookups.
//!
//! The extraction path performs several thousand `HashMap` probes per frame
//! (CHR slot content hashes, decoded tiles, source-key and pattern atlas
//! lookups). The default SipHash-1-3 dominated those probes in profiles; this
//! FxHash-style multiply-rotate mixer is a fraction of the cost and every key
//! type here is a fixed-width integer, tuple, or 64-byte pattern controlled
//! by this crate, so hash-flooding resistance is not needed.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[derive(Clone, Copy, Default)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while let Some((chunk, tail)) = rest.split_first_chunk::<8>() {
            self.add(u64::from_le_bytes(*chunk));
            rest = tail;
        }
        if let Some((chunk, tail)) = rest.split_first_chunk::<4>() {
            self.add(u64::from(u32::from_le_bytes(*chunk)));
            rest = tail;
        }
        if let Some((chunk, tail)) = rest.split_first_chunk::<2>() {
            self.add(u64::from(u16::from_le_bytes(*chunk)));
            rest = tail;
        }
        if let Some(&byte) = rest.first() {
            self.add(u64::from(byte));
        }
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.add(u64::from(value));
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.add(value);
    }

    #[inline]
    fn write_usize(&mut self, value: usize) {
        self.add(value as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        // Fold the high half into the low bits: hashbrown indexes buckets by
        // the LOW bits and tags by the top 7, while `add` alone leaves the low
        // bits of `key * SEED` depending only on the key's low bits (so keys
        // that differ only in their upper fields, e.g. `kind << 32 | pack <<
        // 16 | tile_off` for the same tile_off, would share a bucket chain).
        let h = self.hash;
        let h = (h ^ (h >> 32)).wrapping_mul(0xd6e8_feb8_6659_fd93);
        h ^ (h >> 32)
    }
}

pub type FxBuildHasher = BuildHasherDefault<FxHasher>;
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_keys_hash_equal_and_maps_round_trip() {
        let mut map: FxHashMap<[u8; 64], usize> = FxHashMap::default();
        let mut pattern = [0u8; 64];
        pattern[3] = 7;
        map.insert(pattern, 1);
        map.insert([0u8; 64], 2);
        assert_eq!(map.get(&pattern), Some(&1));
        assert_eq!(map.get(&[0u8; 64]), Some(&2));
        let mut tuples: FxHashMap<(usize, u16), u8> = FxHashMap::default();
        tuples.insert((0x2000, 0x8123), 9);
        assert_eq!(tuples.get(&(0x2000, 0x8123)), Some(&9));
        assert_eq!(tuples.get(&(0x2000, 0x8124)), None);
    }

    #[test]
    fn keys_differing_only_in_upper_fields_spread_across_low_bits() {
        use std::hash::BuildHasher;
        let build = FxBuildHasher::default();
        // 4096 keys into 65536 low-bit buckets: a well-mixed hash leaves
        // nearly all distinct (expected ~3970); a plain multiply leaves 1.
        let low_bits: std::collections::HashSet<u64> = (0u64..4096)
            .map(|pack| build.hash_one((3u64 << 32) | (pack << 16) | 0x12) & 0xffff)
            .collect();
        assert!(low_bits.len() > 3800, "{} distinct low 16-bit values of 4096", low_bits.len());
    }
}
