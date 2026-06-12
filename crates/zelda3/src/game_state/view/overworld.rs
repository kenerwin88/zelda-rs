use super::*;

pub(crate) struct OverworldEventInfoView<'a> {
    ram: &'a [u8],
}

impl<'a> OverworldEventInfoView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn event_info(&self, screen: usize) -> u8 {
        byte(self.ram, OVERWORLD_EVENT_INFO + screen)
    }

    pub(crate) fn has_event_bits(&self, screen: usize, mask: u8) -> bool {
        self.event_info(screen) & mask != 0
    }
}

pub(crate) struct OverworldEventInfoViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OverworldEventInfoViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_event_info(&mut self, screen: usize, value: u8) {
        self.ram[OVERWORLD_EVENT_INFO + screen] = value;
    }

    pub(crate) fn set_event_bits(&mut self, screen: usize, mask: u8) {
        self.ram[OVERWORLD_EVENT_INFO + screen] |= mask;
    }

    pub(crate) fn clear_event_bits(&mut self, screen: usize, mask: u8) {
        self.ram[OVERWORLD_EVENT_INFO + screen] &= !mask;
    }
}

pub(crate) struct OverworldConfigTableView<'a> {
    ram: &'a [u8],
}

impl<'a> OverworldConfigTableView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn music(&self, screen: usize) -> u8 {
        byte(self.ram, OVERWORLD_MUSIC_TABLE + screen)
    }

    pub(crate) fn current_music(&self) -> u8 {
        self.music(usize::from(byte(self.ram, OVERWORLD_SCREEN_INDEX)))
    }

    pub(crate) fn sprite_palette(&self, screen: usize) -> u8 {
        byte(self.ram, OVERWORLD_SPRITE_PALETTE_TABLE + screen)
    }

    pub(crate) fn sprite_graphics(&self, screen: usize) -> u8 {
        byte(self.ram, OVERWORLD_SPRITE_GFX_TABLE + screen)
    }
}

pub(crate) struct DungeonMapViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DungeonMapViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_current_floor_high(&mut self) {
        self.ram[DUNGEON_MAP_CURRENT_FLOOR + 1] = 0;
    }
}

pub(crate) struct OverworldConfigTableViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OverworldConfigTableViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn copy_music_primary(&mut self, data: &[u8]) {
        self.ram[OVERWORLD_MUSIC_TABLE..OVERWORLD_MUSIC_TABLE + 64].copy_from_slice(&data[..64]);
    }

    pub(crate) fn copy_music_secondary(&mut self, data: &[u8]) {
        self.ram[OVERWORLD_MUSIC_TABLE + 64..OVERWORLD_MUSIC_TABLE + 160]
            .copy_from_slice(&data[..96]);
    }

    pub(crate) fn set_music(&mut self, screen: usize, value: u8) {
        self.ram[OVERWORLD_MUSIC_TABLE + screen] = value;
    }
}

pub(crate) struct OverworldScreenSizeView<'a> {
    ram: &'a [u8],
}

impl<'a> OverworldScreenSizeView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn is_big_area_word(&self) -> u16 {
        word(self.ram, OVERWORLD_AREA_IS_BIG)
    }

    pub(crate) fn right_bottom_bound_word(&self) -> u16 {
        word(self.ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND)
    }
}

pub(crate) struct OverworldScreenSizeViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OverworldScreenSizeViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_big_area_high(&mut self) {
        self.ram[OVERWORLD_AREA_IS_BIG + 1] = 0;
    }

    pub(crate) fn set_big_area_low(&mut self, value: u8) {
        self.ram[OVERWORLD_AREA_IS_BIG] = value;
    }

    pub(crate) fn backup_big_area_low(&mut self) {
        self.ram[OVERWORLD_AREA_IS_BIG_BACKUP] = self.ram[OVERWORLD_AREA_IS_BIG];
    }

    pub(crate) fn set_right_bottom_bound_low(&mut self, value: u8) {
        self.ram[OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND] = value;
    }

    pub(crate) fn set_right_bottom_bound_high(&mut self, value: u8) {
        self.ram[OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND + 1] = value;
    }
}

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

pub(crate) struct RoomBoundsView<'a> {
    ram: &'a [u8],
}

impl<'a> RoomBoundsView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn y_bound(&self, index: usize) -> u16 {
        word(self.ram, ROOM_BOUNDS + index * 2)
    }

    pub(crate) fn x_bound(&self, index: usize) -> u16 {
        word(self.ram, ROOM_BOUNDS + 8 + index * 2)
    }

    pub(crate) fn packed_bound(&self, index: usize) -> u16 {
        word(self.ram, ROOM_BOUNDS + index * 2)
    }

    pub(crate) fn packed_top(&self) -> u16 {
        self.packed_bound(0)
    }

    pub(crate) fn packed_bottom(&self) -> u16 {
        self.packed_bound(1)
    }

    pub(crate) fn packed_left(&self) -> u16 {
        self.packed_bound(2)
    }

    pub(crate) fn packed_right(&self) -> u16 {
        self.packed_bound(3)
    }
}

pub(crate) struct RoomBoundsViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> RoomBoundsViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_y_bound(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, ROOM_BOUNDS + index * 2, value);
    }

    pub(crate) fn set_x_bound(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, ROOM_BOUNDS + 8 + index * 2, value);
    }

    pub(crate) fn set_packed_bound(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, ROOM_BOUNDS + index * 2, value);
    }

    pub(crate) fn set_packed_bounds(&mut self, top: u16, bottom: u16, left: u16, right: u16) {
        self.set_packed_bound(0, top);
        self.set_packed_bound(1, bottom);
        self.set_packed_bound(2, left);
        self.set_packed_bound(3, right);
    }

    pub(crate) fn copy_y_bound_from(&mut self, index: usize, src: usize) {
        let value = word(self.ram, src);
        self.set_y_bound(index, value);
    }

    pub(crate) fn copy_x_bound_from(&mut self, index: usize, src: usize) {
        let value = word(self.ram, src);
        self.set_x_bound(index, value);
    }

    pub(crate) fn copy_packed_bound_from(&mut self, index: usize, src: usize) {
        let value = word(self.ram, src);
        self.set_packed_bound(index, value);
    }

    pub(crate) fn add_y_bounds_a(&mut self, value: u16) {
        for index in [0, 2] {
            let next = word(self.ram, ROOM_BOUNDS + index * 2).wrapping_add(value);
            self.set_y_bound(index, next);
        }
    }

    pub(crate) fn add_y_bounds_b(&mut self, value: u16) {
        for index in [1, 3] {
            let next = word(self.ram, ROOM_BOUNDS + index * 2).wrapping_add(value);
            self.set_y_bound(index, next);
        }
    }

    pub(crate) fn add_x_bounds_a(&mut self, value: u16) {
        for index in [0, 2] {
            let next = word(self.ram, ROOM_BOUNDS + 8 + index * 2).wrapping_add(value);
            self.set_x_bound(index, next);
        }
    }

    pub(crate) fn add_x_bounds_b(&mut self, value: u16) {
        for index in [1, 3] {
            let next = word(self.ram, ROOM_BOUNDS + 8 + index * 2).wrapping_add(value);
            self.set_x_bound(index, next);
        }
    }

    pub(crate) fn copy_y_bounds_from(&mut self, src: usize, count: usize) {
        for i in 0..count {
            self.ram[ROOM_BOUNDS + i] = self.ram[src + i];
        }
    }
}

pub(crate) struct VramUploadDataView<'a> {
    ram: &'a [u8],
}

impl<'a> VramUploadDataView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn offset(&self) -> u16 {
        word(self.ram, VRAM_UPLOAD_OFFSET)
    }

    pub(crate) fn offset_usize(&self) -> usize {
        usize::from(self.offset())
    }

    pub(crate) fn data_base(&self) -> usize {
        VRAM_UPLOAD_DATA
    }

    pub(crate) fn data_address(&self, offset: usize) -> usize {
        VRAM_UPLOAD_DATA + offset
    }

    pub(crate) fn current_data_address(&self) -> usize {
        self.data_address(self.offset_usize())
    }

    pub(crate) fn word(&self, offset: usize) -> u16 {
        word(self.ram, VRAM_UPLOAD_DATA + offset)
    }

    pub(crate) fn tilemap_word(&self, offset: usize) -> u16 {
        word(self.ram, VRAM_UPLOAD_OFFSET + offset)
    }

    pub(crate) fn byte(&self, offset: usize) -> u8 {
        byte(self.ram, VRAM_UPLOAD_DATA + offset)
    }

    pub(crate) fn remaining_data(&self) -> &[u8] {
        &self.ram[VRAM_UPLOAD_DATA..]
    }
}

pub(crate) struct VramUploadDataViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> VramUploadDataViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn offset(&self) -> u16 {
        read_le_u16(self.ram, VRAM_UPLOAD_OFFSET)
    }

    pub(crate) fn set_offset(&mut self, value: u16) {
        write_le_u16(self.ram, VRAM_UPLOAD_OFFSET, value);
    }

    pub(crate) fn clear_offset(&mut self) {
        self.set_offset(0);
    }

    pub(crate) fn advance_offset_by(&mut self, value: u16) -> u16 {
        let next = read_le_u16(self.ram, VRAM_UPLOAD_OFFSET).wrapping_add(value);
        write_le_u16(self.ram, VRAM_UPLOAD_OFFSET, next);
        next
    }

    pub(crate) fn set_byte(&mut self, offset: usize, value: u8) {
        self.ram[VRAM_UPLOAD_DATA + offset] = value;
    }

    pub(crate) fn set_word(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, VRAM_UPLOAD_DATA + offset, value);
    }

    pub(crate) fn write_le_u16_at(&mut self, abs_addr: usize, value: u16) {
        write_le_u16(self.ram, abs_addr, value);
    }

    pub(crate) fn write_byte_at(&mut self, abs_addr: usize, value: u8) {
        self.ram[abs_addr] = value;
    }

    pub(crate) fn set_tilemap_word(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, VRAM_UPLOAD_OFFSET + offset, value);
    }

    pub(crate) fn write_overworld_vram_word(&mut self, word_index: usize, value: u16) {
        write_le_u16(self.ram, UVRAM_DATA + word_index * 2, value);
    }

    pub(crate) fn set_level_label_tiles(&mut self, left: &[u8; 14], right: &[u8; 14]) {
        self.ram[VRAM_UPLOAD_DATA + 32] = 0xff;
        for i in (0..14).rev() {
            self.ram[VRAM_UPLOAD_DATA + i] = left[i];
            self.ram[VRAM_UPLOAD_DATA + i + 16] = right[i];
        }
    }

    pub(crate) fn terminate_at(&mut self, offset: usize) {
        self.ram[VRAM_UPLOAD_DATA + offset] = 0xff;
    }

    pub(crate) fn copy_bytes(&mut self, offset: usize, data: &[u8]) {
        self.ram[VRAM_UPLOAD_DATA + offset..VRAM_UPLOAD_DATA + offset + data.len()]
            .copy_from_slice(data);
    }

    pub(crate) fn write_map16_update_packet(
        &mut self,
        abs_addr: usize,
        vram_pos: u16,
        tiles: [u16; 4],
    ) {
        write_le_u16(self.ram, abs_addr, vram_pos.swap_bytes());
        write_le_u16(self.ram, abs_addr + 2, 0x0300);
        write_le_u16(self.ram, abs_addr + 4, tiles[0]);
        write_le_u16(self.ram, abs_addr + 6, tiles[1]);
        write_le_u16(
            self.ram,
            abs_addr + 8,
            vram_pos.wrapping_add(0x20).swap_bytes(),
        );
        write_le_u16(self.ram, abs_addr + 10, 0x0300);
        write_le_u16(self.ram, abs_addr + 12, tiles[2]);
        write_le_u16(self.ram, abs_addr + 14, tiles[3]);
        write_le_u16(self.ram, abs_addr + 16, 0xffff);
    }

    pub(crate) fn write_single_tile_stripe_packet(
        &mut self,
        abs_addr: usize,
        stripe: u16,
        tile: u16,
    ) {
        write_le_u16(self.ram, abs_addr, stripe);
        write_le_u16(self.ram, abs_addr + 2, 0x0100);
        write_le_u16(self.ram, abs_addr + 4, tile);
    }

    pub(crate) fn write_tile_stripe_sentinel(&mut self, abs_addr: usize) {
        write_le_u16(self.ram, abs_addr, 0xffff);
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
