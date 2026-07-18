//! Fixed-size per-frame parity fingerprint, shared by the streaming hook and the
//! checker. The C oracle (../zelda3/src/main.c) mirrors this layout exactly.

pub const PAGE: usize = 0x400;
pub const WRAM_PAGES: usize = 128;
pub const VRAM_PAGES: usize = 64;
/// 4(frame) + 128*4(wram) + 64*4(vram) + 4(sram) + 4(render) + 4(audio) + 4(rollup)
pub const RECORD_LEN: usize = 4 + WRAM_PAGES * 4 + VRAM_PAGES * 4 + 4 + 4 + 4 + 4;

/// Individual WRAM byte offsets zeroed before hashing their page.
///
/// These are display/HDMA scratch bytes whose stable observable output is already
/// covered by the render fingerprint leaf.
pub const FINGERPRINT_MASK: &[usize] = &[0x654, 0x68a];

/// WRAM byte ranges zeroed before hashing their page.
///
/// `0x1b00..0x1cc0` is the SAVELOAD spotlight/window HDMA projection table. The
/// table is volatile projection scratch and is also excluded by the replay RAM
/// checksum used for human-readable diagnostics.
///
/// `0x171c0..0x1766a` is `TEXT_DIALOGUE_POINTERS`, the 398-entry × 3-byte message
/// pointer table `Text_GenerateMessagePointers` stages into WRAM. Since the
/// modern build authors `kDialogue` from editable source text as *uncompressed*
/// bytecode (empty dictionary), these pointers are byte offsets into a differently
/// sized message stream than the ROM-compressed C oracle, so they legitimately
/// diverge. The divergence is transient scratch and provably non-cascading: the
/// decoded message content is verified byte-identical (`expanded_sha1`), and
/// from-scratch C-vs-Rust WRAM is zero-diff once the game runs (incl. active
/// on-screen messages and the ending credits). See
/// `docs/assets/readable-sources.md`.
pub const FINGERPRINT_MASK_RANGES: &[(usize, usize)] =
    &[(0x1b00, 0x1b00 + 224 * 2), (0x171c0, 0x171c0 + 398 * 3)];

#[inline]
pub fn fingerprint_mask_contains(offset: usize) -> bool {
    FINGERPRINT_MASK.contains(&offset)
        || FINGERPRINT_MASK_RANGES
            .iter()
            .any(|&(start, end)| (start..end).contains(&offset))
}

pub fn fingerprint_mask_offsets() -> Vec<usize> {
    let mut offsets = FINGERPRINT_MASK.to_vec();
    for &(start, end) in FINGERPRINT_MASK_RANGES {
        offsets.extend(start..end);
    }
    offsets
}

#[inline]
pub fn fnv1a(bytes: &[u8]) -> u32 {
    let mut h: u32 = 2166136261;
    for &b in bytes {
        h ^= u32::from(b);
        h = h.wrapping_mul(16777619);
    }
    h
}

#[inline]
pub fn fnv1a_u32s(words: &[u32]) -> u32 {
    let mut h: u32 = 2166136261;
    for &w in words {
        for b in w.to_le_bytes() {
            h ^= u32::from(b);
            h = h.wrapping_mul(16777619);
        }
    }
    h
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameFingerprint {
    pub frame: u32,
    pub wram: [u32; WRAM_PAGES],
    pub vram: [u32; VRAM_PAGES],
    pub sram: u32,
    pub render: u32,
    pub audio: u32,
    pub rollup: u32,
}

impl FrameFingerprint {
    /// Hash a 1KB page with FINGERPRINT_MASK bytes (that fall inside it) zeroed.
    fn page_hash_masked(wram: &[u8], page: usize) -> u32 {
        let start = page * PAGE;
        let end = (start + PAGE).min(wram.len());
        let mut h: u32 = 2166136261;
        for off in start..end {
            let mut b = wram[off];
            if fingerprint_mask_contains(off) {
                b = 0;
            }
            h ^= u32::from(b);
            h = h.wrapping_mul(16777619);
        }
        h
    }

    pub fn compute(
        frame: u32,
        wram: &[u8],
        vram_bytes: &[u8],
        sram: &[u8],
        render: u32,
        audio: u32,
    ) -> Self {
        let mut w = [0u32; WRAM_PAGES];
        for (p, slot) in w.iter_mut().enumerate() {
            *slot = Self::page_hash_masked(wram, p);
        }
        let mut v = [0u32; VRAM_PAGES];
        for (p, slot) in v.iter_mut().enumerate() {
            let start = p * PAGE;
            if start >= vram_bytes.len() {
                *slot = fnv1a(&[]);
            } else {
                let end = (start + PAGE).min(vram_bytes.len());
                *slot = fnv1a(&vram_bytes[start..end]);
            }
        }
        let sram_h = fnv1a(sram);
        // rollup folds all leaves in a fixed order.
        let mut leaves = Vec::with_capacity(WRAM_PAGES + VRAM_PAGES + 3);
        leaves.extend_from_slice(&w);
        leaves.extend_from_slice(&v);
        leaves.push(sram_h);
        leaves.push(render);
        leaves.push(audio);
        let rollup = fnv1a_u32s(&leaves);
        FrameFingerprint {
            frame,
            wram: w,
            vram: v,
            sram: sram_h,
            render,
            audio,
            rollup,
        }
    }

    /// Rollup with the audio leaf forced to 0.
    ///
    /// The legacy SPC/DSP audio oracle was removed from the Rust port, so the
    /// Rust side always emits a zero audio leaf. C oracle streams still carry a
    /// live audio hash in the slot; normalizing both sides here keeps the
    /// rollup comparison meaningful for the remaining layers.
    pub fn normalized_rollup(&self) -> u32 {
        let mut leaves = Vec::with_capacity(WRAM_PAGES + VRAM_PAGES + 3);
        leaves.extend_from_slice(&self.wram);
        leaves.extend_from_slice(&self.vram);
        leaves.push(self.sram);
        leaves.push(self.render);
        leaves.push(0);
        fnv1a_u32s(&leaves)
    }

    pub fn to_bytes(&self) -> [u8; RECORD_LEN] {
        let mut b = [0u8; RECORD_LEN];
        let mut o = 0usize;
        let put = |b: &mut [u8; RECORD_LEN], o: &mut usize, v: u32| {
            b[*o..*o + 4].copy_from_slice(&v.to_le_bytes());
            *o += 4;
        };
        put(&mut b, &mut o, self.frame);
        for &x in &self.wram {
            put(&mut b, &mut o, x);
        }
        for &x in &self.vram {
            put(&mut b, &mut o, x);
        }
        put(&mut b, &mut o, self.sram);
        put(&mut b, &mut o, self.render);
        put(&mut b, &mut o, self.audio);
        put(&mut b, &mut o, self.rollup);
        debug_assert_eq!(o, RECORD_LEN);
        b
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= RECORD_LEN, "record too short");
        let mut o = 0usize;
        let get = |o: &mut usize| {
            let v = u32::from_le_bytes(bytes[*o..*o + 4].try_into().unwrap());
            *o += 4;
            v
        };
        let frame = get(&mut o);
        let mut wram = [0u32; WRAM_PAGES];
        for x in &mut wram {
            *x = get(&mut o);
        }
        let mut vram = [0u32; VRAM_PAGES];
        for x in &mut vram {
            *x = get(&mut o);
        }
        let sram = get(&mut o);
        let render = get(&mut o);
        let audio = get(&mut o);
        let rollup = get(&mut o);
        FrameFingerprint {
            frame,
            wram,
            vram,
            sram,
            render,
            audio,
            rollup,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_len_is_788() {
        assert_eq!(RECORD_LEN, 788);
    }

    #[test]
    fn mask_audit() {
        // Pinned set: documented display/HDMA scratch bytes and ranges, plus the
        // uncompressed source-authored dialogue pointer table (0x171c0..0x1766a).
        assert_eq!(FINGERPRINT_MASK, &[0x654, 0x68a]);
        assert_eq!(
            FINGERPRINT_MASK_RANGES,
            &[(0x1b00, 0x1b00 + 224 * 2), (0x171c0, 0x171c0 + 398 * 3)]
        );
        assert_eq!(fingerprint_mask_offsets().len(), 2 + 224 * 2 + 398 * 3);
    }

    #[test]
    fn mask_zeroes_dialogue_pointer_table() {
        // Boot stages the dialogue pointer table at 0x171c0 (398 * 3-byte entries).
        // Under uncompressed source-authored dialogue these byte offsets differ from
        // the ROM-compressed C oracle, so the whole table span must be masked.
        let a = vec![0u8; 0x20000];
        let mut b = a.clone();
        b[0x171c0] = 0xff; // first entry, low byte
        b[0x1766a - 1] = 0xff; // last entry, high byte
        let fa = FrameFingerprint::compute(0, &a, &[], &[], 0, 0);
        let fb = FrameFingerprint::compute(0, &b, &[], &[], 0, 0);
        assert_eq!(fa.wram[0x171c0 / PAGE], fb.wram[0x171c0 / PAGE]);
        assert_eq!(fa.wram[(0x1766a - 1) / PAGE], fb.wram[(0x1766a - 1) / PAGE]);
        // The byte immediately after the table (0x1766a) must NOT be masked.
        let mut c = a.clone();
        c[0x1766a] = 0xff;
        let fc = FrameFingerprint::compute(0, &c, &[], &[], 0, 0);
        assert_ne!(fa.wram[0x1766a / PAGE], fc.wram[0x1766a / PAGE]);
    }

    #[test]
    fn roundtrip() {
        let wram = vec![0xabu8; 0x20000];
        let vram = vec![0xcdu8; 0x10000];
        let sram = vec![0x11u8; 0x500];
        let fp = FrameFingerprint::compute(42, &wram, &vram, &sram, 0xdead, 0xbeef);
        let back = FrameFingerprint::from_bytes(&fp.to_bytes());
        assert_eq!(fp, back);
        assert_eq!(back.frame, 42);
        assert_eq!(back.render, 0xdead);
        assert_eq!(back.audio, 0xbeef);
    }

    #[test]
    fn mask_zeroes_byte() {
        let mut a = vec![0u8; 0x800];
        let mut b = a.clone();
        b[0x654] = 0xff; // inside page 1
        let fa = FrameFingerprint::compute(0, &a, &[], &[], 0, 0);
        let fb = FrameFingerprint::compute(0, &b, &[], &[], 0, 0);
        assert_eq!(
            fa.wram[1], fb.wram[1],
            "masked byte must not change page hash"
        );
        // a non-masked byte change DOES change the page hash:
        a[0x655] = 0xff;
        let fa2 = FrameFingerprint::compute(0, &a, &[], &[], 0, 0);
        assert_ne!(fa.wram[1], fa2.wram[1]);
    }

    #[test]
    fn mask_zeroes_hdma_window_scratch() {
        let a = vec![0u8; 0x2000];
        let mut b = a.clone();
        b[0x68a] = 0x64;
        b[0x1b00] = 0xff;
        b[0x1cc0 - 1] = 0xc8;
        let fa = FrameFingerprint::compute(0, &a, &[], &[], 0, 0);
        let fb = FrameFingerprint::compute(0, &b, &[], &[], 0, 0);
        assert_eq!(fa.wram[1], fb.wram[1]);
        assert_eq!(fa.wram[6], fb.wram[6]);
        assert_eq!(fa.wram[7], fb.wram[7]);
    }
}
