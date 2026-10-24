//! Shared byte hashing for native diagnostics and logical replay.
use sha2::{Digest, Sha256};
use std::{fs::File, io::{BufReader, Read}, path::Path};

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let file =
        File::open(path).map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone)]
pub struct Sha256Digest(Sha256);

impl Sha256Digest {
    pub fn new() -> Self {
        Self(Sha256::new())
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    /// Feed canonical RGB bytes from one or more tightly packed RGBA chunks.
    /// Packing a bounded block before updating SHA-256 preserves the exact byte
    /// stream while avoiding one digest call per pixel. The scratch storage is
    /// stack-owned and independent of frame size.
    pub fn update_rgb_from_rgba<'a, I>(&mut self, chunks: I)
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        const PACKED_BYTES: usize = 12 * 1024;
        let mut packed = [0u8; PACKED_BYTES];
        let mut used = 0usize;
        for chunk in chunks {
            for pixel in chunk.chunks_exact(4) {
                if used + 3 > packed.len() {
                    self.update(&packed[..used]);
                    used = 0;
                }
                packed[used..used + 3].copy_from_slice(&pixel[..3]);
                used += 3;
            }
        }
        if used != 0 {
            self.update(&packed[..used]);
        }
    }

    /// Feed signed samples in the canonical little-endian interleaved format
    /// without issuing one digest update per sample.
    pub fn update_i16_le(&mut self, samples: &[i16]) {
        const PACKED_BYTES: usize = 8 * 1024;
        let mut packed = [0u8; PACKED_BYTES];
        let mut used = 0usize;
        for sample in samples {
            if used + 2 > packed.len() {
                self.update(&packed[..used]);
                used = 0;
            }
            packed[used..used + 2].copy_from_slice(&sample.to_le_bytes());
            used += 2;
        }
        if used != 0 {
            self.update(&packed[..used]);
        }
    }

    pub fn finish(self) -> String {
        format!("{:x}", self.0.finalize())
    }
}

impl Default for Sha256Digest {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packed_rgba_rgb_updates_match_the_canonical_per_pixel_stream() {
        let rows = [
            [1u8, 2, 3, 4, 5, 6, 7, 8],
            [9u8, 10, 11, 12, 13, 14, 15, 16],
        ];
        let mut canonical = Sha256Digest::new();
        for row in &rows {
            for pixel in row.chunks_exact(4) {
                canonical.update(&pixel[..3]);
            }
        }
        let mut packed = Sha256Digest::new();
        packed.update_rgb_from_rgba(rows.iter().map(<[u8; 8]>::as_slice));
        assert_eq!(packed.finish(), canonical.finish());
    }
    #[test]
    fn packed_i16_updates_match_the_canonical_little_endian_stream() {
        let samples = [i16::MIN, -1, 0, 1, i16::MAX];
        let mut canonical = Sha256Digest::new();
        for sample in samples {
            canonical.update(&sample.to_le_bytes());
        }
        let mut packed = Sha256Digest::new();
        packed.update_i16_le(&samples);
        assert_eq!(packed.finish(), canonical.finish());
    }
}
