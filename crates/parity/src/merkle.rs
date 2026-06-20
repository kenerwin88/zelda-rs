use crate::fnv1a_u32s;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleIndex {
    pub block_size: u32,
    pub block_hashes: Vec<u32>,
    pub root: u32,
}

impl MerkleIndex {
    pub fn build(rollups: &[u32], block_size: u32) -> Self {
        let bs = block_size as usize;
        let block_hashes: Vec<u32> = rollups.chunks(bs).map(fnv1a_u32s).collect();
        let root = fnv1a_u32s(&block_hashes);
        MerkleIndex { block_size, block_hashes, root }
    }

    /// First block index whose hash differs (and thus contains the first
    /// diverging frame). None if roots match.
    pub fn first_diff_block(&self, other: &MerkleIndex) -> Option<usize> {
        if self.root == other.root {
            return None;
        }
        let n = self.block_hashes.len().max(other.block_hashes.len());
        for i in 0..n {
            if self.block_hashes.get(i) != other.block_hashes.get(i) {
                return Some(i);
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(8 + self.block_hashes.len() * 4 + 4);
        b.extend_from_slice(&self.block_size.to_le_bytes());
        b.extend_from_slice(&(self.block_hashes.len() as u32).to_le_bytes());
        for &h in &self.block_hashes {
            b.extend_from_slice(&h.to_le_bytes());
        }
        b.extend_from_slice(&self.root.to_le_bytes());
        b
    }

    pub fn from_bytes(b: &[u8]) -> Self {
        let block_size = u32::from_le_bytes(b[0..4].try_into().unwrap());
        let n = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
        let mut block_hashes = Vec::with_capacity(n);
        let mut o = 8;
        for _ in 0..n {
            block_hashes.push(u32::from_le_bytes(b[o..o + 4].try_into().unwrap()));
            o += 4;
        }
        let root = u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        MerkleIndex { block_size, block_hashes, root }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_stable_and_roundtrip() {
        let rollups: Vec<u32> = (0..20000).collect();
        let m = MerkleIndex::build(&rollups, 8192);
        assert_eq!(m.block_hashes.len(), 3); // 20000/8192 -> 3 blocks
        assert_eq!(MerkleIndex::from_bytes(&m.to_bytes()), m);
    }

    #[test]
    fn detects_first_diff_block() {
        let a: Vec<u32> = (0..20000).collect();
        let mut b = a.clone();
        b[9000] ^= 1; // block 1
        let ma = MerkleIndex::build(&a, 8192);
        let mb = MerkleIndex::build(&b, 8192);
        assert_eq!(ma.first_diff_block(&mb), Some(1));
        assert_eq!(ma.first_diff_block(&ma), None);
    }
}
