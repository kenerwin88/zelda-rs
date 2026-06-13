use super::*;

pub(crate) struct RawRamView<'a> {
    ram: &'a [u8],
}

impl<'a> RawRamView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn byte_at(&self, offset: usize) -> u8 {
        byte(self.ram, offset)
    }

    pub(crate) fn word_at(&self, offset: usize) -> u16 {
        word(self.ram, offset)
    }

    pub(crate) fn long_at(&self, offset: usize) -> u32 {
        u32::from(byte(self.ram, offset))
            | (u32::from(byte(self.ram, offset + 1)) << 8)
            | (u32::from(byte(self.ram, offset + 2)) << 16)
            | (u32::from(byte(self.ram, offset + 3)) << 24)
    }

    pub(crate) fn range(&self, offset: usize, len: usize) -> &[u8] {
        &self.ram[offset..offset + len]
    }

    pub(crate) fn watched_byte(&self, offset: u32) -> Option<u8> {
        self.ram.get(offset as usize).copied()
    }
}

pub(crate) struct RawRamViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> RawRamViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_byte_at(&mut self, offset: usize, value: u8) {
        self.ram[offset] = value;
    }

    pub(crate) fn set_word_at(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, offset, value);
    }

    pub(crate) fn set_long_at(&mut self, offset: usize, value: u32) {
        self.ram[offset] = value as u8;
        self.ram[offset + 1] = (value >> 8) as u8;
        self.ram[offset + 2] = (value >> 16) as u8;
        self.ram[offset + 3] = (value >> 24) as u8;
    }

    pub(crate) fn range_mut(&mut self, offset: usize, len: usize) -> &mut [u8] {
        &mut self.ram[offset..offset + len]
    }

    pub(crate) fn copy_to(&mut self, offset: usize, data: &[u8]) {
        self.range_mut(offset, data.len()).copy_from_slice(data);
    }

    pub(crate) fn fill(&mut self, offset: usize, len: usize, value: u8) {
        self.range_mut(offset, len).fill(value);
    }

    pub(crate) fn move_link_axis_by_velocity(
        &mut self,
        subpixel_offset: usize,
        coord_offset: usize,
        velocity: u8,
    ) -> u16 {
        move_link_axis_by_velocity(self.ram, subpixel_offset, coord_offset, velocity)
    }

    pub(crate) fn move_link_axis_by_subpixel_delta(
        &mut self,
        subpixel_offset: usize,
        coord_offset: usize,
        delta: u16,
    ) -> u16 {
        move_link_axis_by_subpixel_delta(self.ram, subpixel_offset, coord_offset, delta)
    }
}
