//! Projection targets for the native states.
//!
//! Every native state publishes its bytes through `write_to_ram`, which is generic over
//! a `RamTarget` so the same projection can address live WRAM directly or record the
//! bytes it would write. Reads inside a projection (mode gates such as the indoors flag)
//! always come from live WRAM.

use std::ops::Range;

pub(crate) trait RamTarget {
    fn write_byte(&mut self, addr: usize, value: u8);
    fn write_bytes(&mut self, addr: usize, bytes: &[u8]);
    /// Write a slice into an explicit range; the lengths must agree, as `copy_from_slice`
    /// required. The range stays visible to the ownership scanner.
    fn write_range(&mut self, range: Range<usize>, bytes: &[u8]) {
        assert_eq!(
            range.len(),
            bytes.len(),
            "projection range and slice length differ"
        );
        self.write_bytes(range.start, bytes);
    }
    fn read_byte(&self, addr: usize) -> u8;
    fn get_byte(&self, addr: usize) -> Option<u8>;
    fn len(&self) -> usize;

    fn write_word(&mut self, addr: usize, value: u16) {
        self.write_byte(addr, value as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }

    fn read_word(&self, addr: usize) -> u16 {
        u16::from(self.read_byte(addr)) | (u16::from(self.read_byte(addr + 1)) << 8)
    }

    fn fill_bytes(&mut self, range: Range<usize>, value: u8) {
        for addr in range {
            self.write_byte(addr, value);
        }
    }
}

impl RamTarget for [u8] {
    fn write_byte(&mut self, addr: usize, value: u8) {
        self[addr] = value;
    }

    fn write_bytes(&mut self, addr: usize, bytes: &[u8]) {
        self[addr..addr + bytes.len()].copy_from_slice(bytes);
    }

    fn read_byte(&self, addr: usize) -> u8 {
        self[addr]
    }

    fn get_byte(&self, addr: usize) -> Option<u8> {
        self.get(addr).copied()
    }

    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    fn write_word(&mut self, addr: usize, value: u16) {
        crate::types::write_le_u16(self, addr, value);
    }

    fn fill_bytes(&mut self, range: Range<usize>, value: u8) {
        self[range].fill(value);
    }
}

impl RamTarget for Vec<u8> {
    fn write_byte(&mut self, addr: usize, value: u8) {
        self.as_mut_slice().write_byte(addr, value);
    }

    fn write_bytes(&mut self, addr: usize, bytes: &[u8]) {
        self.as_mut_slice().write_bytes(addr, bytes);
    }

    fn read_byte(&self, addr: usize) -> u8 {
        self.as_slice().read_byte(addr)
    }

    fn get_byte(&self, addr: usize) -> Option<u8> {
        self.as_slice().get_byte(addr)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn write_word(&mut self, addr: usize, value: u16) {
        self.as_mut_slice().write_word(addr, value);
    }

    fn fill_bytes(&mut self, range: Range<usize>, value: u8) {
        self.as_mut_slice().fill_bytes(range, value);
    }
}

/// Records the bytes a projection would write, reading gates from live WRAM.
pub(crate) struct ProjectionLog<'a> {
    live: &'a [u8],
    writes: Vec<(usize, u8)>,
}

impl<'a> ProjectionLog<'a> {
    pub(crate) fn new(live: &'a [u8]) -> Self {
        Self {
            live,
            writes: Vec::new(),
        }
    }

    pub(crate) fn into_writes(self) -> Vec<(usize, u8)> {
        self.writes
    }
}

impl RamTarget for ProjectionLog<'_> {
    fn write_byte(&mut self, addr: usize, value: u8) {
        self.writes.push((addr, value));
    }

    fn write_bytes(&mut self, addr: usize, bytes: &[u8]) {
        self.writes
            .extend(bytes.iter().enumerate().map(|(i, &b)| (addr + i, b)));
    }

    fn read_byte(&self, addr: usize) -> u8 {
        self.live[addr]
    }

    fn get_byte(&self, addr: usize) -> Option<u8> {
        self.live.get(addr).copied()
    }

    fn len(&self) -> usize {
        self.live.len()
    }
}

/// Live WRAM that accepts a projection but stores only the bytes whose value differs
/// from what WRAM already holds. A bridge whose state was adopted from WRAM at
/// construction therefore publishes exactly the bytes its mutation changed: a field
/// the mutation left alone still equals WRAM and is never re-stamped.
pub(crate) struct DiffTarget<'a> {
    live: &'a mut [u8],
}

impl<'a> DiffTarget<'a> {
    pub(crate) fn new(live: &'a mut [u8]) -> Self {
        Self { live }
    }
}

impl RamTarget for DiffTarget<'_> {
    fn write_byte(&mut self, addr: usize, value: u8) {
        if self.live[addr] != value {
            self.live[addr] = value;
        }
    }

    fn write_bytes(&mut self, addr: usize, bytes: &[u8]) {
        for (i, &b) in bytes.iter().enumerate() {
            self.write_byte(addr + i, b);
        }
    }

    fn read_byte(&self, addr: usize) -> u8 {
        self.live[addr]
    }

    fn get_byte(&self, addr: usize) -> Option<u8> {
        self.live.get(addr).copied()
    }

    fn len(&self) -> usize {
        self.live.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::native::world::WorldTransientState;
    use snes::WRAM_SIZE;

    #[test]
    fn a_logged_projection_replays_to_the_slice_projection() {
        let mut ram = vec![0u8; WRAM_SIZE];
        for (i, b) in ram.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(31);
        }
        let state = WorldTransientState::load_from_ram(&ram);

        let mut direct = ram.clone();
        state.write_to_ram(&mut direct);

        let mut log = ProjectionLog::new(&ram);
        state.write_to_ram(&mut log);
        let mut replayed = ram.clone();
        for (addr, value) in log.into_writes() {
            replayed[addr] = value;
        }
        assert_eq!(direct, replayed);
    }
}

#[cfg(test)]
mod bridge_contract_tests {
    use crate::game_state::constants::{BG2_X_SCROLL, MAPBAK_CGWSEL};
    use crate::game_state::native::display::{NativePpuScrollCopyBridgeMut, PpuScrollCopyState};
    use crate::types::{read_le_u16, write_le_u16};
    use snes::WRAM_SIZE;

    /// A bridge adopts live WRAM for its state when constructed and publishes only the
    /// bytes its mutation changed; a byte another owner wrote stays as written.
    #[test]
    fn a_bridge_adopts_live_ram_and_publishes_only_changed_bytes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, BG2_X_SCROLL, 0x0060);
        write_le_u16(&mut ram, MAPBAK_CGWSEL, 0x1234);
        let mut scroll = PpuScrollCopyState::default();
        scroll.set_bg2_h_copy2(0x2200);
        scroll.set_mapbak_cgwsel_word(0x5678);

        {
            let mut bridge = NativePpuScrollCopyBridgeMut::new(&mut scroll, &mut ram);
            bridge.add_bg2_h_copy2(0x10);
        }

        assert_eq!(scroll.bg2_h_copy2(), 0x0070);
        assert_eq!(read_le_u16(&ram, BG2_X_SCROLL), 0x0070);
        assert_eq!(scroll.mapbak_cgwsel_word(), 0x1234);
        assert_eq!(read_le_u16(&ram, MAPBAK_CGWSEL), 0x1234);
        assert_eq!(scroll, PpuScrollCopyState::load_from_ram(&ram));
    }
}
