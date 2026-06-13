use super::*;

pub(crate) struct OverworldMap16DecodeView<'a> {
    ram: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverworldMap16SourcePage {
    Main,
    Overlay,
}

impl OverworldMap16SourcePage {
    fn base_address(self) -> usize {
        match self {
            Self::Main => 0x2000,
            Self::Overlay => 0x4000,
        }
    }
}

impl<'a> OverworldMap16DecodeView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn source_byte(&self, index: usize) -> u8 {
        byte(self.ram, OVERWORLD_MAP16_DECODE_SRC + index)
    }

    pub(crate) fn source_word(&self, index: usize) -> u16 {
        word(self.ram, OVERWORLD_MAP16_DECODE_SRC + index)
    }

    pub(crate) fn source_page_word(&self, page: OverworldMap16SourcePage, offset: usize) -> u16 {
        word(self.ram, page.base_address() + offset)
    }

    pub(crate) fn decode_last(&self) -> u16 {
        word(self.ram, MAP16_DECODE_LAST)
    }

    pub(crate) fn decode_quad(&self, idx: usize) -> (u16, u16, u16, u16) {
        (
            word(self.ram, MAP16_DECODE_0 + idx),
            word(self.ram, MAP16_DECODE_1 + idx),
            word(self.ram, MAP16_DECODE_2 + idx),
            word(self.ram, MAP16_DECODE_3 + idx),
        )
    }

    pub(crate) fn decode_block_byte(&self, base: usize, index: usize) -> u8 {
        byte(self.ram, base + index)
    }
}

pub(crate) struct OverworldMap16DecodeViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OverworldMap16DecodeViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn copy_source_from(&mut self, data: &[u8]) {
        self.ram[OVERWORLD_MAP16_DECODE_SRC..OVERWORLD_MAP16_DECODE_SRC + data.len()]
            .copy_from_slice(data);
    }

    pub(crate) fn copy_scratch_to_source_words_high(&mut self, len: usize) {
        for i in 0..len {
            self.ram[OVERWORLD_MAP16_DECODE_SRC + 1 + i * 2] =
                self.ram[OVERWORLD_DECOMP_BUFFER + i];
        }
    }

    pub(crate) fn copy_scratch_to_source_words_low(&mut self, len: usize) {
        for i in 0..len {
            self.ram[OVERWORLD_MAP16_DECODE_SRC + i * 2] = self.ram[OVERWORLD_DECOMP_BUFFER + i];
        }
    }

    pub(crate) fn write_decompressed_byte(&mut self, dst: usize, value: u8) {
        self.ram[dst] = value;
    }

    pub(crate) fn copy_decompressed_byte(&mut self, dst_org: usize, dst: usize, offset: usize) {
        self.ram[dst] = self.ram[dst_org + offset];
    }

    pub(crate) fn decomp_scratch_byte_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.ram[OVERWORLD_DECOMP_BUFFER + index]
    }

    pub(crate) fn decomp_scratch_slice_mut(&mut self) -> &mut [u8] {
        &mut self.ram[OVERWORLD_DECOMP_BUFFER..]
    }

    pub(crate) fn decode_block_fill(&mut self, dst: usize, table: &[u8], x: usize) {
        self.ram[dst] = table[x];
        self.ram[dst + 2] = table[x + 1];
        self.ram[dst + 4] = table[x + 2];
        self.ram[dst + 6] = table[x + 3];
        let packed0 = table[x + 4];
        let packed1 = table[x + 5];
        self.ram[dst + 1] = packed0 >> 4;
        self.ram[dst + 3] = packed0 & 0x0f;
        self.ram[dst + 5] = packed1 >> 4;
        self.ram[dst + 7] = packed1 & 0x0f;
    }

    pub(crate) fn set_decode_last(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_DECODE_LAST, value);
    }

    pub(crate) fn set_decode_tmp(&mut self, value: u16) {
        write_le_u16(self.ram, MAP16_DECODE_WORK_WORD, value);
    }

    pub(crate) fn write_decoded_map32_to_bg2_tilemap(&mut self, dst: usize, idx: usize) {
        let v0 = word(self.ram, MAP16_DECODE_0 + idx);
        let v1 = word(self.ram, MAP16_DECODE_1 + idx);
        let v2 = word(self.ram, MAP16_DECODE_2 + idx);
        let v3 = word(self.ram, MAP16_DECODE_3 + idx);
        write_le_u16(self.ram, dst, v0);
        write_le_u16(self.ram, dst + 128, v2);
        write_le_u16(self.ram, dst + 2, v1);
        write_le_u16(self.ram, dst + 130, v3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overworld_map16_source_pages_read_named_wram_pages() {
        let mut ram = vec![0; 0x8000];
        write_le_u16(&mut ram, 0x2000 + 0x010, 0x1234);
        write_le_u16(&mut ram, 0x4000 + 0x010, 0xabcd);

        let view = OverworldMap16DecodeView::new(&ram);
        assert_eq!(
            view.source_page_word(OverworldMap16SourcePage::Main, 0x010),
            0x1234
        );
        assert_eq!(
            view.source_page_word(OverworldMap16SourcePage::Overlay, 0x010),
            0xabcd
        );
    }
}
