//! Native dungeon and story progress. Cartridge layout is confined to codecs
//! and the eager compatibility writes in the bridge below.

use super::ram_byte;
use crate::game_state::constants::*;
use crate::game_state::native::ram_target::RamTarget;
use crate::game_state::save_format::SAVE_CHECKSUM_OFFSET;
use crate::types::{read_le_u16, write_le_u16};

pub(crate) const DUNGEON_ROOM_COUNT: usize = 320;
const DEATH_COUNT_PALACE_SLOTS: usize = 14;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DungeonRoomProgress {
    flags: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SaveProgressState {
    palace_index_x2: u8,
    hud_current_items: [u8; 4],
    #[serde(with = "serde_big_array::BigArray")]
    rooms: [DungeonRoomProgress; DUNGEON_ROOM_COUNT],
    progress_indicator: u8,
    progress_flags: u8,
    map_icons_indicator: u8,
    which_starting_point: u8,
    progress_indicator_3: u8,
    dark_world_state: u8,
    deaths_per_palace: [u16; DEATH_COUNT_PALACE_SLOTS],
    pending_death_save_counter: u16,
    total_death_save_counter: u16,
    checksum: u16,
    post_message_refresh_flag: u8,
}

impl Default for SaveProgressState {
    fn default() -> Self {
        Self {
            palace_index_x2: 0,
            hud_current_items: [0; 4],
            rooms: [DungeonRoomProgress::default(); DUNGEON_ROOM_COUNT],
            progress_indicator: 0,
            progress_flags: 0,
            map_icons_indicator: 0,
            which_starting_point: 0,
            progress_indicator_3: 0,
            dark_world_state: 0,
            deaths_per_palace: [0; DEATH_COUNT_PALACE_SLOTS],
            pending_death_save_counter: 0,
            total_death_save_counter: 0,
            checksum: 0,
            post_message_refresh_flag: 0,
        }
    }
}

impl SaveProgressState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            palace_index_x2: ram_byte(ram, CUR_PALACE_INDEX_X2),
            hud_current_items: [HUD_CUR_ITEM, HUD_CUR_ITEM_X, HUD_CUR_ITEM_L, HUD_CUR_ITEM_R]
                .map(|address| ram_byte(ram, address)),
            post_message_refresh_flag: ram_byte(ram, HUD_POST_MESSAGE_REFRESH_FLAG),
            rooms: std::array::from_fn(|room| DungeonRoomProgress {
                flags: read_le_u16(ram, SAVE_DUNG_INFO + room * 2),
            }),
            progress_indicator: ram_byte(ram, SRAM_PROGRESS_INDICATOR),
            progress_flags: ram_byte(ram, SRAM_PROGRESS_FLAGS),
            map_icons_indicator: ram_byte(ram, SAVEGAME_MAP_ICONS_INDICATOR),
            which_starting_point: ram_byte(ram, WHICH_STARTING_POINT),
            progress_indicator_3: ram_byte(ram, SRAM_PROGRESS_INDICATOR_3),
            dark_world_state: ram_byte(ram, SAVEGAME_IS_DARKWORLD),
            deaths_per_palace: std::array::from_fn(|palace| {
                read_le_u16(ram, DEATHS_PER_PALACE + palace * 2)
            }),
            pending_death_save_counter: read_le_u16(ram, PENDING_DEATH_SAVE_COUNTER),
            total_death_save_counter: read_le_u16(ram, TOTAL_DEATH_SAVE_COUNTER),
            checksum: read_le_u16(ram, SAVE_DUNG_INFO + SAVE_CHECKSUM_OFFSET),
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        // Save fields publish at their mutation points. Keep the existing frame
        // projection limited to runtime/HUD fields while legacy consumers remain.
        ram.write_byte(CUR_PALACE_INDEX_X2, self.palace_index_x2);
        ram.write_byte(HUD_CUR_ITEM, self.hud_current_items[0]);
        ram.write_byte(HUD_CUR_ITEM_X, self.hud_current_items[1]);
        ram.write_byte(HUD_CUR_ITEM_L, self.hud_current_items[2]);
        ram.write_byte(HUD_CUR_ITEM_R, self.hud_current_items[3]);
        ram.write_byte(
            HUD_POST_MESSAGE_REFRESH_FLAG,
            self.post_message_refresh_flag,
        );
    }

    /// Encode only progress-owned fields; equipment, resources, overworld events,
    /// followers, and reserved save bytes retain their current values.
    #[cfg(test)]
    pub(crate) fn encode_progress_to_ram(&self, ram: &mut [u8]) {
        for (room, progress) in self.rooms.iter().enumerate() {
            write_le_u16(ram, SAVE_DUNG_INFO + room * 2, progress.flags);
        }
        ram[SRAM_PROGRESS_INDICATOR] = self.progress_indicator;
        ram[SRAM_PROGRESS_FLAGS] = self.progress_flags;
        ram[SAVEGAME_MAP_ICONS_INDICATOR] = self.map_icons_indicator;
        ram[WHICH_STARTING_POINT] = self.which_starting_point;
        ram[SRAM_PROGRESS_INDICATOR_3] = self.progress_indicator_3;
        ram[SAVEGAME_IS_DARKWORLD] = self.dark_world_state;
        for (palace, deaths) in self.deaths_per_palace.iter().copied().enumerate() {
            write_le_u16(ram, DEATHS_PER_PALACE + palace * 2, deaths);
        }
        write_le_u16(
            ram,
            PENDING_DEATH_SAVE_COUNTER,
            self.pending_death_save_counter,
        );
        write_le_u16(ram, TOTAL_DEATH_SAVE_COUNTER, self.total_death_save_counter);
        write_le_u16(ram, SAVE_DUNG_INFO + SAVE_CHECKSUM_OFFSET, self.checksum);
    }

    pub(crate) fn palace_index_x2(&self) -> u8 {
        self.palace_index_x2
    }

    pub(crate) fn palace_index_x2_word(&self) -> u16 {
        u16::from(self.palace_index_x2)
    }

    #[cfg(test)]
    pub(crate) fn palace_index(&self) -> usize {
        usize::from(self.palace_index_x2() >> 1)
    }

    pub(crate) fn progress_indicator(&self) -> u8 {
        self.progress_indicator
    }

    pub(crate) fn progress_indicator_word(&self) -> u16 {
        u16::from_le_bytes([self.progress_indicator, self.progress_flags])
    }

    pub(crate) fn progress_flags(&self) -> u8 {
        self.progress_flags
    }

    #[cfg(test)]
    pub(crate) fn progress_flags_has(&self, mask: u8) -> bool {
        self.progress_flags() & mask != 0
    }

    pub(crate) fn map_icons_indicator(&self) -> u8 {
        self.map_icons_indicator
    }

    pub(crate) fn dark_world_state(&self) -> u8 {
        self.dark_world_state
    }

    #[cfg(test)]
    pub(crate) fn dark_world_bit6(&self) -> u8 {
        (self.dark_world_state() >> 6) & 1
    }

    pub(crate) fn hud_current_item(&self) -> u8 {
        self.hud_current_item_slot(0)
    }

    pub(crate) fn hud_current_item_slot(&self, slot: usize) -> u8 {
        self.hud_current_items
            .get(slot)
            .copied()
            .unwrap_or(self.hud_current_items[0])
    }

    pub(crate) fn dungeon_info_word(&self, room: usize) -> u16 {
        self.rooms.get(room).map_or(0, |progress| progress.flags)
    }

    pub(crate) fn death_count_for_palace(&self, palace: usize) -> u16 {
        if palace < DEATH_COUNT_PALACE_SLOTS {
            self.deaths_per_palace[palace]
        } else {
            0
        }
    }

    pub(crate) fn pending_death_save_counter(&self) -> u16 {
        self.pending_death_save_counter
    }

    pub(crate) fn total_death_save_counter(&self) -> u16 {
        self.total_death_save_counter
    }

    pub(crate) fn total_death_save_counter_is_uninitialized(&self) -> bool {
        self.total_death_save_counter() == 0xffff
    }

    pub(crate) fn which_starting_point(&self) -> u8 {
        self.which_starting_point
    }

    pub(crate) fn progress_indicator_3(&self) -> u8 {
        self.progress_indicator_3
    }

    pub(crate) fn set_palace_index_x2(&mut self, value: u8) {
        self.palace_index_x2 = value;
    }

    pub(crate) fn set_which_starting_point(&mut self, value: u8) {
        self.which_starting_point = value;
    }

    pub(crate) fn xor_palace_index_x2(&mut self, value: u8) {
        self.palace_index_x2 ^= value;
    }

    pub(crate) fn set_progress_indicator(&mut self, value: u8) {
        self.progress_indicator = value;
    }

    pub(crate) fn or_progress_flags(&mut self, value: u8) {
        self.progress_flags = self.progress_flags() | value;
    }

    pub(crate) fn or_progress_indicator_3(&mut self, bits: u8) {
        self.progress_indicator_3 = self.progress_indicator_3() | bits;
    }

    pub(crate) fn set_progress_indicator_3(&mut self, value: u8) {
        self.progress_indicator_3 = value;
    }

    pub(crate) fn clear_progress_indicator_3_bits(&mut self, bits: u8) {
        self.progress_indicator_3 = self.progress_indicator_3() & !bits;
    }

    pub(crate) fn xor_progress_flags(&mut self, value: u8) {
        self.progress_flags = self.progress_flags() ^ value;
    }

    pub(crate) fn set_progress_flags(&mut self, value: u8) {
        self.progress_flags = value;
    }

    pub(crate) fn set_map_icons_indicator(&mut self, value: u8) {
        self.map_icons_indicator = value;
    }

    pub(crate) fn set_dark_world_state(&mut self, value: u8) {
        self.dark_world_state = value;
    }

    pub(crate) fn xor_dark_world_state(&mut self, value: u8) {
        self.dark_world_state = self.dark_world_state() ^ value;
    }

    pub(crate) fn set_hud_current_item(&mut self, value: u8) {
        self.set_hud_current_item_slot(0, value);
    }

    pub(crate) fn set_hud_current_item_slot(&mut self, slot: usize, value: u8) {
        let slot = slot.min(self.hud_current_items.len() - 1);
        self.hud_current_items[slot] = value;
    }

    pub(crate) fn set_death_count_for_palace(&mut self, palace: usize, value: u16) {
        if palace < DEATH_COUNT_PALACE_SLOTS {
            self.deaths_per_palace[palace] = value;
        }
    }

    pub(crate) fn increment_pending_death_save_counter(&mut self) -> u16 {
        let deaths = self.pending_death_save_counter().wrapping_add(1);
        self.pending_death_save_counter = deaths;
        deaths
    }

    pub(crate) fn clear_pending_death_save_counter(&mut self) {
        self.pending_death_save_counter = 0;
    }

    pub(crate) fn set_total_death_save_counter(&mut self, value: u16) {
        self.total_death_save_counter = value;
    }

    pub(crate) fn clear_post_message_refresh_flag(&mut self) {
        self.post_message_refresh_flag = 0;
    }

    pub(crate) fn request_post_message_refresh(&mut self) {
        self.post_message_refresh_flag = 0x80;
    }

    pub(crate) fn set_dungeon_info_word(&mut self, room: usize, value: u16) {
        if let Some(progress) = self.rooms.get_mut(room) {
            progress.flags = value;
        }
    }

    pub(crate) fn or_dungeon_info_word(&mut self, room: usize, value: u16) -> u16 {
        let word = self.dungeon_info_word(room) | value;
        self.set_dungeon_info_word(room, word);
        word
    }

    pub(crate) fn set_dungeon_info_checksum(&mut self, value: u16) {
        self.checksum = value;
    }
}

pub(crate) struct NativeSaveProgressBridgeMut<'a> {
    state: &'a mut SaveProgressState,
    ram: &'a mut [u8],
}

impl<'a> NativeSaveProgressBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SaveProgressState, ram: &'a mut [u8]) -> Self {
        // These six runtime/HUD bytes still have legacy writers. In particular,
        // an out-of-range HUD slot clamps the native write to R but publishes
        // the first-slot fallback. Import only this runtime surface; dungeon
        // records and saved progress are never reloaded before mutations.
        state.palace_index_x2 = ram_byte(ram, CUR_PALACE_INDEX_X2);
        state.hud_current_items = [HUD_CUR_ITEM, HUD_CUR_ITEM_X, HUD_CUR_ITEM_L, HUD_CUR_ITEM_R]
            .map(|address| ram_byte(ram, address));
        state.post_message_refresh_flag = ram_byte(ram, HUD_POST_MESSAGE_REFRESH_FLAG);
        Self { state, ram }
    }

    fn sync_byte(&mut self, address: usize, value: u8) {
        self.ram[address] = value;
    }

    fn sync_word(&mut self, address: usize, value: u16) {
        write_le_u16(self.ram, address, value);
    }

    fn sync_hud_current_item_slot(&mut self, slot: usize) {
        let address = match slot {
            0 => HUD_CUR_ITEM,
            1 => HUD_CUR_ITEM_X,
            2 => HUD_CUR_ITEM_L,
            _ => HUD_CUR_ITEM_R,
        };
        self.sync_byte(address, self.state.hud_current_item_slot(slot));
    }

    fn sync_dungeon_info_word(&mut self, room: usize) {
        if room < DUNGEON_ROOM_COUNT {
            write_le_u16(
                self.ram,
                SAVE_DUNG_INFO + room * 2,
                self.state.dungeon_info_word(room),
            );
        }
    }

    pub(crate) fn set_palace_index_x2(&mut self, value: u8) {
        self.state.set_palace_index_x2(value);
        self.sync_byte(CUR_PALACE_INDEX_X2, self.state.palace_index_x2());
    }

    pub(crate) fn set_which_starting_point(&mut self, value: u8) {
        self.state.set_which_starting_point(value);
        self.sync_byte(WHICH_STARTING_POINT, self.state.which_starting_point());
    }

    pub(crate) fn xor_palace_index_x2(&mut self, value: u8) {
        self.state.xor_palace_index_x2(value);
        self.sync_byte(CUR_PALACE_INDEX_X2, self.state.palace_index_x2());
    }

    pub(crate) fn set_progress_indicator(&mut self, value: u8) {
        self.state.set_progress_indicator(value);
        self.sync_byte(SRAM_PROGRESS_INDICATOR, self.state.progress_indicator());
    }

    pub(crate) fn or_progress_flags(&mut self, value: u8) {
        self.state.or_progress_flags(value);
        self.sync_byte(SRAM_PROGRESS_FLAGS, self.state.progress_flags());
    }

    pub(crate) fn or_progress_indicator_3(&mut self, bits: u8) {
        self.state.or_progress_indicator_3(bits);
        self.sync_byte(SRAM_PROGRESS_INDICATOR_3, self.state.progress_indicator_3());
    }

    pub(crate) fn set_progress_indicator_3(&mut self, value: u8) {
        self.state.set_progress_indicator_3(value);
        self.sync_byte(SRAM_PROGRESS_INDICATOR_3, self.state.progress_indicator_3());
    }

    pub(crate) fn clear_progress_indicator_3_bits(&mut self, bits: u8) {
        self.state.clear_progress_indicator_3_bits(bits);
        self.sync_byte(SRAM_PROGRESS_INDICATOR_3, self.state.progress_indicator_3());
    }

    pub(crate) fn xor_progress_flags(&mut self, value: u8) {
        self.state.xor_progress_flags(value);
        self.sync_byte(SRAM_PROGRESS_FLAGS, self.state.progress_flags());
    }

    pub(crate) fn set_progress_flags(&mut self, value: u8) {
        self.state.set_progress_flags(value);
        self.sync_byte(SRAM_PROGRESS_FLAGS, self.state.progress_flags());
    }

    pub(crate) fn set_map_icons_indicator(&mut self, value: u8) {
        self.state.set_map_icons_indicator(value);
        self.sync_byte(
            SAVEGAME_MAP_ICONS_INDICATOR,
            self.state.map_icons_indicator(),
        );
    }

    pub(crate) fn set_dark_world_state(&mut self, value: u8) {
        self.state.set_dark_world_state(value);
        self.sync_byte(SAVEGAME_IS_DARKWORLD, self.state.dark_world_state());
    }

    pub(crate) fn xor_dark_world_state(&mut self, value: u8) {
        self.state.xor_dark_world_state(value);
        self.sync_byte(SAVEGAME_IS_DARKWORLD, self.state.dark_world_state());
    }

    pub(crate) fn set_hud_current_item(&mut self, value: u8) {
        self.state.set_hud_current_item(value);
        self.sync_hud_current_item_slot(0);
    }

    pub(crate) fn set_hud_current_item_slot(&mut self, slot: usize, value: u8) {
        self.state.set_hud_current_item_slot(slot, value);
        self.sync_hud_current_item_slot(slot);
    }

    pub(crate) fn set_death_count_for_palace(&mut self, palace: usize, value: u16) {
        self.state.set_death_count_for_palace(palace, value);
        if palace < DEATH_COUNT_PALACE_SLOTS {
            self.sync_word(
                DEATHS_PER_PALACE + palace * 2,
                self.state.death_count_for_palace(palace),
            );
        }
    }

    pub(crate) fn increment_pending_death_save_counter(&mut self) -> u16 {
        let deaths = self.state.increment_pending_death_save_counter();
        self.sync_word(PENDING_DEATH_SAVE_COUNTER, deaths);
        deaths
    }

    pub(crate) fn clear_pending_death_save_counter(&mut self) {
        self.state.clear_pending_death_save_counter();
        self.sync_word(
            PENDING_DEATH_SAVE_COUNTER,
            self.state.pending_death_save_counter(),
        );
    }

    pub(crate) fn set_total_death_save_counter(&mut self, value: u16) {
        self.state.set_total_death_save_counter(value);
        self.sync_word(
            TOTAL_DEATH_SAVE_COUNTER,
            self.state.total_death_save_counter(),
        );
    }

    pub(crate) fn clear_post_message_refresh_flag(&mut self) {
        self.state.clear_post_message_refresh_flag();
        self.sync_byte(HUD_POST_MESSAGE_REFRESH_FLAG, 0);
    }

    pub(crate) fn request_post_message_refresh(&mut self) {
        self.state.request_post_message_refresh();
        self.sync_byte(HUD_POST_MESSAGE_REFRESH_FLAG, 0x80);
    }

    pub(crate) fn set_dungeon_info_word(&mut self, room: usize, value: u16) {
        self.state.set_dungeon_info_word(room, value);
        self.sync_dungeon_info_word(room);
    }

    pub(crate) fn or_dungeon_info_word(&mut self, room: usize, value: u16) -> u16 {
        let word = self.state.or_dungeon_info_word(room, value);
        self.sync_dungeon_info_word(room);
        word
    }

    pub(crate) fn set_dungeon_info_checksum(&mut self, value: u16) {
        self.state.set_dungeon_info_checksum(value);
        write_le_u16(self.ram, SAVE_DUNG_INFO + SAVE_CHECKSUM_OFFSET, value);
    }
}
