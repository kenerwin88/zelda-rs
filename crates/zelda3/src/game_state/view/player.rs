use super::*;

pub(crate) struct RamPlayerStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> RamPlayerStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_direction_lock(&mut self, value: u8) {
        self.ram[LINK_CANT_CHANGE_DIRECTION] = value;
    }

    pub(crate) fn advance_link_dma_source_offset(&mut self) -> u16 {
        let mut source_offset = read_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET).wrapping_add(0x400);
        if source_offset == 0x0c00 {
            source_offset = 0;
        }
        write_le_u16(self.ram, LINK_DMA_SOURCE_OFFSET, source_offset);
        source_offset
    }

    pub(crate) fn advance_link_dma_tile_offset(&mut self) -> u16 {
        let mut tile_offset = read_le_u16(self.ram, LINK_DMA_TILE_OFFSET).wrapping_add(2);
        if tile_offset == 12 {
            tile_offset = 0;
        }
        write_le_u16(self.ram, LINK_DMA_TILE_OFFSET, tile_offset);
        tile_offset
    }

    pub(crate) fn set_link_dma_countdown(&mut self, value: u16) {
        write_le_u16(self.ram, LINK_DMA_COUNTDOWN, value);
    }

    pub(crate) fn decrement_link_dma_countdown(&mut self) -> u16 {
        let countdown = read_le_u16(self.ram, LINK_DMA_COUNTDOWN).wrapping_sub(1);
        write_le_u16(self.ram, LINK_DMA_COUNTDOWN, countdown);
        countdown
    }
}
