use super::*;

pub(crate) struct RamPlayerStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> RamPlayerStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    #[cfg(test)]
    pub(crate) fn set_direction_lock(&mut self, value: u8) {
        self.ram[LINK_CANT_CHANGE_DIRECTION] = value;
    }

    pub(crate) fn link_dma_source_offset(&self) -> u16 {
        read_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET)
    }

    pub(crate) fn set_link_dma_source_offset(&mut self, value: u16) {
        write_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET, value);
    }

    pub(crate) fn link_dma_tile_offset(&self) -> u16 {
        read_le_u16(self.ram, LINK_DMA_TILE_OFFSET)
    }

    pub(crate) fn set_link_dma_tile_offset(&mut self, value: u16) {
        write_le_u16(self.ram, LINK_DMA_TILE_OFFSET, value);
    }

    pub(crate) fn set_link_dma_countdown(&mut self, value: u16) {
        write_le_u16(self.ram, LINK_DMA_COUNTDOWN, value);
    }

    pub(crate) fn link_dma_countdown(&self) -> u16 {
        read_le_u16(self.ram, LINK_DMA_COUNTDOWN)
    }
}
