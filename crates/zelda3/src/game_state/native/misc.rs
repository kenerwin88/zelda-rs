use super::ram_byte;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

// C memorized_tile_value is 0x40 bytes wide (0xfa00..0xfa40) = 0x20 u16 slots, and
// num_memorized_tiles is bounded by it. Modeling 0x80 slots made write_to_ram project
// 0x00 over 0xfa40..0xfb00 (word_7EFA40, dung_torch_data 0xfb40, ...), clobbering real
// data on every frame. Match the C value-table size so those bytes keep their owners.
const MEMORIZED_TILE_ENTRY_SLOTS: usize = 0x20;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ScratchCounterState {
    value: u8,
}

impl ScratchCounterState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            value: ram_byte(ram, TEMP_COUNTER),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TEMP_COUNTER] = self.value;
    }

    pub(crate) fn value(&self) -> u8 {
        self.value
    }

    pub(crate) fn as_usize(&self) -> usize {
        usize::from(self.value())
    }

    pub(crate) fn is_negative(&self) -> bool {
        (self.value() as i8).is_negative()
    }

    pub(crate) fn set(&mut self, value: u8) {
        self.value = value;
    }

    pub(crate) fn decrement(&mut self) -> u8 {
        self.value = self.value.wrapping_sub(1);
        self.value
    }
}

pub(crate) struct NativeScratchCounterBridgeMut<'a> {
    scratch_counter: &'a mut ScratchCounterState,
    ram: &'a mut [u8],
}

impl<'a> NativeScratchCounterBridgeMut<'a> {
    pub(crate) fn new(scratch_counter: &'a mut ScratchCounterState, ram: &'a mut [u8]) -> Self {
        Self {
            scratch_counter,
            ram,
        }
    }

    fn sync(&mut self) {
        self.scratch_counter.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.scratch_counter,
            ScratchCounterState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set(&mut self, value: u8) {
        self.scratch_counter.set(value);
        self.sync();
    }

    pub(crate) fn decrement(&mut self) -> u8 {
        let value = self.scratch_counter.decrement();
        self.sync();
        value
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MemorizedTileState {
    count: u16,
    addresses: Vec<u16>,
    values: Vec<u16>,
}

impl MemorizedTileState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        // memorized_tile_value (0xfa00..0xfa3f) is an OVERWORLD-only transient; while
        // INDOORS those same bytes are movable_block_datas records 48..63 (0xf940 +
        // index*4, a SNES byte-reuse). We must NOT *project* the value table indoors
        // (it would clobber the dungeon movable-block records — see write_to_ram), but
        // we still LOAD it every frame so the movable-block leftover carries through a
        // dungeon→overworld transition until the overworld redraw repopulates it
        // (matches the C oracle, which leaves those bytes untouched on exit).
        // (memorized_tile_addr 0xf800..0xf83f and count 0x4ac sit below
        // movable_block_datas@0xf940 and never overlap, so they are always owned here.)
        let mut addresses = vec![0; MEMORIZED_TILE_ENTRY_SLOTS];
        let mut values = vec![0; MEMORIZED_TILE_ENTRY_SLOTS];
        for slot in 0..MEMORIZED_TILE_ENTRY_SLOTS {
            addresses[slot] = read_le_u16(ram, MEMORIZED_TILE_ADDR + slot * 2);
            values[slot] = read_le_u16(ram, MEMORIZED_TILE_VALUE + slot * 2);
        }
        Self {
            count: read_le_u16(ram, NUM_MEMORIZED_TILES),
            addresses,
            values,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, NUM_MEMORIZED_TILES, self.count);
        let outdoors = ram.get(PLAYER_IS_INDOORS).copied().unwrap_or(0) == 0;
        for slot in 0..MEMORIZED_TILE_ENTRY_SLOTS {
            write_le_u16(ram, MEMORIZED_TILE_ADDR + slot * 2, self.addresses[slot]);
            if outdoors {
                write_le_u16(ram, MEMORIZED_TILE_VALUE + slot * 2, self.values[slot]);
            }
        }
    }

    pub(crate) fn count(&self) -> u16 {
        self.count
    }

    pub(crate) fn entry_addr(&self, index: usize) -> u16 {
        self.addresses.get(index).copied().unwrap_or_default()
    }

    pub(crate) fn entry_value(&self, index: usize) -> u16 {
        self.values.get(index).copied().unwrap_or_default()
    }

    pub(crate) fn set_count(&mut self, value: u16) {
        self.count = value;
    }

    pub(crate) fn clear_count(&mut self) {
        self.count = 0;
    }

    pub(crate) fn set_entry_addr(&mut self, byte_offset: usize, pos: u16) {
        self.addresses[byte_offset / 2] = pos;
    }

    pub(crate) fn set_entry_value(&mut self, byte_offset: usize, tile: u16) {
        self.values[byte_offset / 2] = tile;
    }

    pub(crate) fn append_entry(&mut self, pos: u16, tile: u16) {
        let byte_offset = usize::from(self.count);
        self.set_entry_value(byte_offset, tile);
        self.set_entry_addr(byte_offset, pos);
        self.set_count(byte_offset as u16 + 2);
    }

    pub(crate) fn clear_entry_addresses(&mut self) {
        self.addresses.fill(0);
    }
}

impl Default for MemorizedTileState {
    fn default() -> Self {
        Self {
            count: 0,
            addresses: vec![0; MEMORIZED_TILE_ENTRY_SLOTS],
            values: vec![0; MEMORIZED_TILE_ENTRY_SLOTS],
        }
    }
}

pub(crate) struct NativeMemorizedTileBridgeMut<'a> {
    memorized_tiles: &'a mut MemorizedTileState,
    ram: &'a mut [u8],
}

impl<'a> NativeMemorizedTileBridgeMut<'a> {
    pub(crate) fn new(memorized_tiles: &'a mut MemorizedTileState, ram: &'a mut [u8]) -> Self {
        Self {
            memorized_tiles,
            ram,
        }
    }

    fn sync(&mut self) {
        self.memorized_tiles.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        let mut fresh = MemorizedTileState::load_from_ram(self.ram);
        if self.ram.get(PLAYER_IS_INDOORS).copied().unwrap_or(0) != 0 {
            fresh.values.clone_from(&self.memorized_tiles.values);
        }
        debug_assert_eq!(*self.memorized_tiles, fresh);
    }

    pub(crate) fn set_count(&mut self, value: u16) {
        self.memorized_tiles.set_count(value);
        self.sync();
    }

    pub(crate) fn clear_count(&mut self) {
        self.memorized_tiles.clear_count();
        self.sync();
    }

    pub(crate) fn set_entry_addr(&mut self, byte_offset: usize, pos: u16) {
        self.memorized_tiles.set_entry_addr(byte_offset, pos);
        self.sync();
    }

    pub(crate) fn set_entry_value(&mut self, byte_offset: usize, tile: u16) {
        self.memorized_tiles.set_entry_value(byte_offset, tile);
        self.sync();
    }

    pub(crate) fn append_entry(&mut self, pos: u16, tile: u16) {
        self.memorized_tiles.append_entry(pos, tile);
        self.sync();
    }

    pub(crate) fn clear_entry_addresses(&mut self) {
        self.memorized_tiles.clear_entry_addresses();
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonSecretState {
    pending_kind: u8,
    pending_kind_high: u8,
    overworld_substitution_counter: u8,
}

impl DungeonSecretState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            pending_kind: ram_byte(ram, DUNGEON_SECRET_PENDING_KIND),
            pending_kind_high: ram_byte(ram, DUNGEON_SECRET_PENDING_KIND + 1),
            overworld_substitution_counter: ram_byte(ram, OVERWORLD_SECRET_SUBST_CTR),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DUNGEON_SECRET_PENDING_KIND] = self.pending_kind;
        ram[DUNGEON_SECRET_PENDING_KIND + 1] = self.pending_kind_high;
        ram[OVERWORLD_SECRET_SUBST_CTR] = self.overworld_substitution_counter;
    }

    pub(crate) fn pending_kind(&self) -> u8 {
        self.pending_kind
    }

    pub(crate) fn overworld_subst_counter(&self) -> u8 {
        self.overworld_substitution_counter
    }

    pub(crate) fn has_pending_kind(&self) -> bool {
        self.pending_kind() != 0
    }

    pub(crate) fn is_available(&self) -> bool {
        self.pending_kind() != 0xff
    }

    pub(crate) fn graphics_kind(&self) -> Option<u8> {
        if self.pending_kind & 0x80 != 0 {
            Some(self.pending_kind & 0x7f)
        } else {
            None
        }
    }

    pub(crate) fn clear_pending_kind(&mut self) {
        self.pending_kind = 0;
    }

    pub(crate) fn set_pending_kind(&mut self, value: u8) {
        self.pending_kind = value;
    }

    pub(crate) fn increment_overworld_subst_counter(&mut self) {
        self.overworld_substitution_counter = self.overworld_substitution_counter.wrapping_add(1);
    }

    pub(crate) fn set_powder_pending_kind(&mut self) {
        self.pending_kind = 4;
        self.pending_kind_high = 0;
    }

    pub(crate) fn or_pending_kind(&mut self, value: u8) {
        self.pending_kind |= value;
    }

    pub(crate) fn mark_graphics_kind(&mut self) {
        self.pending_kind |= 0x80;
    }
}

pub(crate) struct NativeDungeonSecretBridgeMut<'a> {
    dungeon_secret: &'a mut DungeonSecretState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonSecretBridgeMut<'a> {
    pub(crate) fn new(dungeon_secret: &'a mut DungeonSecretState, ram: &'a mut [u8]) -> Self {
        Self {
            dungeon_secret,
            ram,
        }
    }

    fn sync(&mut self) {
        self.dungeon_secret.write_to_ram(self.ram);
        debug_assert_eq!(
            *self.dungeon_secret,
            DungeonSecretState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_pending_kind(&mut self) {
        self.dungeon_secret.clear_pending_kind();
        self.sync();
    }

    pub(crate) fn set_pending_kind(&mut self, value: u8) {
        self.dungeon_secret.set_pending_kind(value);
        self.sync();
    }

    pub(crate) fn increment_overworld_subst_counter(&mut self) {
        self.dungeon_secret.increment_overworld_subst_counter();
        self.sync();
    }

    pub(crate) fn set_powder_pending_kind(&mut self) {
        self.dungeon_secret.set_powder_pending_kind();
        self.sync();
    }

    pub(crate) fn or_pending_kind(&mut self, value: u8) {
        self.dungeon_secret.or_pending_kind(value);
        self.sync();
    }

    pub(crate) fn mark_graphics_kind(&mut self) {
        self.dungeon_secret.mark_graphics_kind();
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SaveLoadTransferState {
    source_offset: u16,
}

impl SaveLoadTransferState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            source_offset: read_le_u16(ram, SAVE_LOAD_SOURCE_OFFSET),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, SAVE_LOAD_SOURCE_OFFSET, self.source_offset);
    }

    pub(crate) fn source_offset(&self) -> u16 {
        self.source_offset
    }

    pub(crate) fn source_offset_usize(&self) -> usize {
        usize::from(self.source_offset)
    }

    pub(crate) fn set_source_offset(&mut self, value: u16) {
        self.source_offset = value;
    }
}

pub(crate) struct NativeSaveLoadTransferBridgeMut<'a> {
    transfer: &'a mut SaveLoadTransferState,
    ram: &'a mut [u8],
}

impl<'a> NativeSaveLoadTransferBridgeMut<'a> {
    pub(crate) fn new(transfer: &'a mut SaveLoadTransferState, ram: &'a mut [u8]) -> Self {
        Self { transfer, ram }
    }

    fn sync(&mut self) {
        self.transfer.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.transfer,
            SaveLoadTransferState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_source_offset(&mut self, value: u16) {
        self.transfer.set_source_offset(value);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonMapDisplayState {
    scroll_draw_offset: u16,
    scroll_input: u16,
    marker_x_offset: u16,
    marker_y_offset: u16,
    location_marker_base_y: u16,
    init_state: u8,
    current_floor: u16,
    floor_scroll_step: u8,
    idx: u16,
    scroll_target_y: u16,
    player_marker_x: u16,
    player_marker_y: u16,
}

impl DungeonMapDisplayState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            scroll_draw_offset: read_le_u16(ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET),
            scroll_input: read_le_u16(ram, DUNGEON_MAP_SCROLL_INPUT),
            marker_x_offset: read_le_u16(ram, DUNGEON_MAP_MARKER_X_OFFSET),
            marker_y_offset: read_le_u16(ram, DUNGEON_MAP_MARKER_Y_OFFSET),
            location_marker_base_y: read_le_u16(ram, DUNGEON_MAP_LOCATION_MARKER_BASE_Y),
            init_state: ram_byte(ram, DUNGMAP_INIT_STATE),
            current_floor: read_le_u16(ram, DUNGMAP_CUR_FLOOR),
            floor_scroll_step: ram_byte(ram, DUNGMAP_FLOOR_SCROLL_STEP),
            idx: read_le_u16(ram, DUNGMAP_IDX),
            scroll_target_y: read_le_u16(ram, DUNGMAP_SCROLL_TARGET_Y),
            player_marker_x: read_le_u16(ram, DUNGMAP_PLAYER_MARKER_X),
            player_marker_y: read_le_u16(ram, DUNGMAP_PLAYER_MARKER_Y),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET, self.scroll_draw_offset);
        write_le_u16(ram, DUNGEON_MAP_SCROLL_INPUT, self.scroll_input);
        write_le_u16(ram, DUNGEON_MAP_MARKER_X_OFFSET, self.marker_x_offset);
        write_le_u16(ram, DUNGEON_MAP_MARKER_Y_OFFSET, self.marker_y_offset);
        write_le_u16(
            ram,
            DUNGEON_MAP_LOCATION_MARKER_BASE_Y,
            self.location_marker_base_y,
        );
        ram[DUNGMAP_INIT_STATE] = self.init_state;
        write_le_u16(ram, DUNGMAP_CUR_FLOOR, self.current_floor);
        ram[DUNGMAP_FLOOR_SCROLL_STEP] = self.floor_scroll_step;
        write_le_u16(ram, DUNGMAP_IDX, self.idx);
        write_le_u16(ram, DUNGMAP_SCROLL_TARGET_Y, self.scroll_target_y);
        write_le_u16(ram, DUNGMAP_PLAYER_MARKER_X, self.player_marker_x);
        write_le_u16(ram, DUNGMAP_PLAYER_MARKER_Y, self.player_marker_y);
    }

    pub(crate) fn scroll_draw_offset(&self) -> u16 {
        self.scroll_draw_offset
    }

    pub(crate) fn scroll_input(&self) -> u16 {
        self.scroll_input
    }

    pub(crate) fn scroll_input_direction_index(&self) -> usize {
        usize::from((self.scroll_input() >> 3) & 1)
    }

    pub(crate) fn marker_x_offset(&self) -> u16 {
        self.marker_x_offset
    }

    pub(crate) fn marker_y_offset(&self) -> u16 {
        self.marker_y_offset
    }

    pub(crate) fn location_marker_base_y(&self) -> u8 {
        self.location_marker_base_y as u8
    }

    pub(crate) fn dungmap_init_state(&self) -> u8 {
        self.init_state
    }

    pub(crate) fn dungmap_cur_floor(&self) -> u16 {
        self.current_floor
    }

    pub(crate) fn dungmap_cur_floor_byte(&self) -> u8 {
        self.current_floor as u8
    }

    pub(crate) fn dungmap_floor_scroll_step(&self) -> u8 {
        self.floor_scroll_step
    }

    pub(crate) fn dungmap_idx(&self) -> u16 {
        self.idx
    }

    pub(crate) fn dungmap_scroll_target_y(&self) -> u16 {
        self.scroll_target_y
    }

    pub(crate) fn dungmap_player_marker_x(&self) -> u16 {
        self.player_marker_x
    }

    pub(crate) fn dungmap_player_marker_x_byte(&self) -> u8 {
        self.player_marker_x as u8
    }

    pub(crate) fn dungmap_player_marker_y(&self) -> u16 {
        self.player_marker_y
    }

    pub(crate) fn current_floor(&self) -> u16 {
        self.current_floor
    }

    pub(crate) fn clear_scroll_state(&mut self) {
        self.scroll_draw_offset = 0;
        self.scroll_input = 0;
    }

    pub(crate) fn set_scroll_draw_offset(&mut self, value: u16) {
        self.scroll_draw_offset = value;
    }

    pub(crate) fn set_scroll_input(&mut self, value: u16) {
        self.scroll_input = value;
    }

    pub(crate) fn reset_marker_offsets(&mut self) {
        self.marker_x_offset = 0x0040;
        self.marker_y_offset = 0x0040;
    }

    pub(crate) fn set_marker_x_offset(&mut self, value: u16) {
        self.marker_x_offset = value;
    }

    pub(crate) fn set_marker_y_offset(&mut self, value: u16) {
        self.marker_y_offset = value;
    }

    pub(crate) fn set_location_marker_base_y(&mut self, value: u8) {
        self.location_marker_base_y = u16::from(value);
    }

    pub(crate) fn shift_marker_x_left(&mut self) -> u16 {
        self.marker_x_offset = self.marker_x_offset.wrapping_sub(0x10);
        self.marker_x_offset
    }

    pub(crate) fn reset_marker_x_offset(&mut self) {
        self.marker_x_offset = 0x0040;
    }

    pub(crate) fn shift_marker_y_low_up(&mut self) {
        self.marker_y_offset = (self.marker_y_offset & 0xff00)
            | u16::from((self.marker_y_offset as u8).wrapping_sub(0x10));
    }

    pub(crate) fn add_marker_y_offset_signed(&mut self, value: i16) -> u16 {
        self.marker_y_offset = self.marker_y_offset.wrapping_add_signed(value);
        self.marker_y_offset
    }

    pub(crate) fn increment_dungmap_init_state(&mut self) {
        self.init_state = self.init_state.wrapping_add(1);
    }

    pub(crate) fn clear_dungmap_init_state(&mut self) {
        self.init_state = 0;
    }

    pub(crate) fn set_dungmap_cur_floor(&mut self, value: u16) {
        self.current_floor = value;
    }

    pub(crate) fn decrement_dungmap_cur_floor_byte(&mut self) {
        self.current_floor =
            (self.current_floor & 0xff00) | u16::from((self.current_floor as u8).wrapping_sub(1));
    }

    pub(crate) fn increment_dungmap_cur_floor(&mut self) -> u16 {
        self.current_floor = self.current_floor.wrapping_add(1);
        self.current_floor
    }

    pub(crate) fn increment_dungmap_cur_floor_byte(&mut self) {
        self.current_floor =
            (self.current_floor & 0xff00) | u16::from((self.current_floor as u8).wrapping_add(1));
    }

    pub(crate) fn set_dungmap_floor_scroll_step(&mut self, value: u8) {
        self.floor_scroll_step = value;
    }

    pub(crate) fn clear_dungmap_floor_scroll_step(&mut self) {
        self.floor_scroll_step = 0;
    }

    pub(crate) fn increment_dungmap_floor_scroll_step(&mut self) {
        self.floor_scroll_step = self.floor_scroll_step.wrapping_add(1);
    }

    pub(crate) fn set_dungmap_idx(&mut self, value: u16) {
        self.idx = value;
    }

    pub(crate) fn clear_dungmap_idx(&mut self) {
        self.idx = 0;
    }

    pub(crate) fn set_dungmap_scroll_target_y(&mut self, value: u16) {
        self.scroll_target_y = value;
    }

    pub(crate) fn set_dungmap_player_marker_x(&mut self, value: u16) {
        self.player_marker_x = value;
    }

    pub(crate) fn set_dungmap_player_marker_y(&mut self, value: u16) {
        self.player_marker_y = value;
    }

    pub(crate) fn clear_current_floor_high(&mut self) {
        self.current_floor &= 0x00ff;
    }
}

pub(crate) struct NativeDungeonMapDisplayBridgeMut<'a> {
    display: &'a mut DungeonMapDisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonMapDisplayBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DungeonMapDisplayState, ram: &'a mut [u8]) -> Self {
        Self { display, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.display,
            DungeonMapDisplayState::load_from_ram(self.ram)
        );
    }

    fn sync(&mut self) {
        self.display.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_scroll_state(&mut self) {
        self.display.clear_scroll_state();
        self.sync();
    }

    pub(crate) fn set_scroll_draw_offset(&mut self, value: u16) {
        self.display.set_scroll_draw_offset(value);
        self.sync();
    }

    pub(crate) fn set_scroll_input(&mut self, value: u16) {
        self.display.set_scroll_input(value);
        self.sync();
    }

    pub(crate) fn reset_marker_offsets(&mut self) {
        self.display.reset_marker_offsets();
        self.sync();
    }

    pub(crate) fn set_marker_x_offset(&mut self, value: u16) {
        self.display.set_marker_x_offset(value);
        self.sync();
    }

    pub(crate) fn set_marker_y_offset(&mut self, value: u16) {
        self.display.set_marker_y_offset(value);
        self.sync();
    }

    pub(crate) fn set_location_marker_base_y(&mut self, value: u8) {
        self.display.set_location_marker_base_y(value);
        self.sync();
    }

    pub(crate) fn shift_marker_x_left(&mut self) -> u16 {
        let value = self.display.shift_marker_x_left();
        self.sync();
        value
    }

    pub(crate) fn reset_marker_x_offset(&mut self) {
        self.display.reset_marker_x_offset();
        self.sync();
    }

    pub(crate) fn shift_marker_y_low_up(&mut self) {
        self.display.shift_marker_y_low_up();
        self.sync();
    }

    pub(crate) fn reset_marker_x_and_shift_marker_y_low_up(&mut self) {
        self.display.reset_marker_x_offset();
        self.display.shift_marker_y_low_up();
        self.sync();
    }

    pub(crate) fn add_marker_y_offset_signed(&mut self, value: i16) -> u16 {
        let value = self.display.add_marker_y_offset_signed(value);
        self.sync();
        value
    }

    pub(crate) fn increment_dungmap_init_state(&mut self) {
        self.display.increment_dungmap_init_state();
        self.sync();
    }

    pub(crate) fn clear_dungmap_init_state(&mut self) {
        self.display.clear_dungmap_init_state();
        self.sync();
    }

    pub(crate) fn set_dungmap_cur_floor(&mut self, value: u16) {
        self.display.set_dungmap_cur_floor(value);
        self.sync();
    }

    pub(crate) fn decrement_dungmap_cur_floor_byte(&mut self) {
        self.display.decrement_dungmap_cur_floor_byte();
        self.sync();
    }

    pub(crate) fn increment_dungmap_cur_floor(&mut self) -> u16 {
        let value = self.display.increment_dungmap_cur_floor();
        self.sync();
        value
    }

    pub(crate) fn increment_dungmap_cur_floor_byte(&mut self) {
        self.display.increment_dungmap_cur_floor_byte();
        self.sync();
    }

    pub(crate) fn set_dungmap_floor_scroll_step(&mut self, value: u8) {
        self.display.set_dungmap_floor_scroll_step(value);
        self.sync();
    }

    pub(crate) fn clear_dungmap_floor_scroll_step(&mut self) {
        self.display.clear_dungmap_floor_scroll_step();
        self.sync();
    }

    pub(crate) fn increment_dungmap_floor_scroll_step(&mut self) {
        self.display.increment_dungmap_floor_scroll_step();
        self.sync();
    }

    pub(crate) fn set_dungmap_idx(&mut self, value: u16) {
        self.display.set_dungmap_idx(value);
        self.sync();
    }

    pub(crate) fn clear_dungmap_idx(&mut self) {
        self.display.clear_dungmap_idx();
        self.sync();
    }

    pub(crate) fn set_dungmap_scroll_target_y(&mut self, value: u16) {
        self.display.set_dungmap_scroll_target_y(value);
        self.sync();
    }

    pub(crate) fn set_dungmap_player_marker_x(&mut self, value: u16) {
        self.display.set_dungmap_player_marker_x(value);
        self.sync();
    }

    pub(crate) fn set_dungmap_player_marker_y(&mut self, value: u16) {
        self.display.set_dungmap_player_marker_y(value);
        self.sync();
    }

    pub(crate) fn clear_current_floor_high(&mut self) {
        self.display.clear_current_floor_high();
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MinigameState {
    archery_or_digging_game_mode: u8,
    credits: u8,
    boomerang_in_place_flag: u8,
    boomerang_temp_x: u16,
    boomerang_temp_y: u16,
}

impl MinigameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            archery_or_digging_game_mode: ram_byte(ram, IS_ARCHER_OR_SHOVEL_GAME),
            credits: ram_byte(ram, MINIGAME_CREDITS),
            boomerang_in_place_flag: ram_byte(ram, FLAG_FOR_BOOMERANG_IN_PLACE),
            boomerang_temp_x: read_le_u16(ram, BOOMERANG_TEMP_X),
            boomerang_temp_y: read_le_u16(ram, BOOMERANG_TEMP_Y),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[IS_ARCHER_OR_SHOVEL_GAME] = self.archery_or_digging_game_mode;
        ram[MINIGAME_CREDITS] = self.credits;
        ram[FLAG_FOR_BOOMERANG_IN_PLACE] = self.boomerang_in_place_flag;
        write_le_u16(ram, BOOMERANG_TEMP_X, self.boomerang_temp_x);
        write_le_u16(ram, BOOMERANG_TEMP_Y, self.boomerang_temp_y);
    }

    pub(crate) fn is_archer_or_shovel_game(&self) -> u8 {
        self.archery_or_digging_game_mode
    }

    pub(crate) fn credits(&self) -> u8 {
        self.credits
    }

    pub(crate) fn flag_boomerang_in_place(&self) -> u8 {
        self.boomerang_in_place_flag
    }

    pub(crate) fn boomerang_temp_x(&self) -> u16 {
        self.boomerang_temp_x
    }

    pub(crate) fn boomerang_temp_y(&self) -> u16 {
        self.boomerang_temp_y
    }

    pub(crate) fn set_is_archer_or_shovel_game(&mut self, value: u8) {
        self.archery_or_digging_game_mode = value;
    }

    pub(crate) fn clear_is_archer_or_shovel_game(&mut self) {
        self.archery_or_digging_game_mode = 0;
    }

    pub(crate) fn set_credits(&mut self, value: u8) {
        self.credits = value;
    }

    pub(crate) fn decrement_credits(&mut self) -> u8 {
        self.credits = self.credits.wrapping_sub(1);
        self.credits
    }

    pub(crate) fn clear_flag_boomerang_in_place(&mut self) {
        self.boomerang_in_place_flag = 0;
    }

    pub(crate) fn set_flag_boomerang_in_place(&mut self, value: u8) {
        self.boomerang_in_place_flag = value;
    }

    pub(crate) fn set_boomerang_temp_x(&mut self, value: u16) {
        self.boomerang_temp_x = value;
    }

    pub(crate) fn set_boomerang_temp_y(&mut self, value: u16) {
        self.boomerang_temp_y = value;
    }
}

pub(crate) struct NativeMinigameBridgeMut<'a> {
    minigame: &'a mut MinigameState,
    ram: &'a mut [u8],
}

impl<'a> NativeMinigameBridgeMut<'a> {
    pub(crate) fn new(minigame: &'a mut MinigameState, ram: &'a mut [u8]) -> Self {
        Self { minigame, ram }
    }

    fn sync(&mut self) {
        self.minigame.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.minigame, MinigameState::load_from_ram(self.ram));
    }

    pub(crate) fn set_is_archer_or_shovel_game(&mut self, value: u8) {
        self.minigame.set_is_archer_or_shovel_game(value);
        self.sync();
    }

    pub(crate) fn clear_is_archer_or_shovel_game(&mut self) {
        self.minigame.clear_is_archer_or_shovel_game();
        self.sync();
    }

    pub(crate) fn set_credits(&mut self, value: u8) {
        self.minigame.set_credits(value);
        self.sync();
    }

    pub(crate) fn decrement_credits(&mut self) -> u8 {
        let value = self.minigame.decrement_credits();
        self.sync();
        value
    }

    pub(crate) fn clear_flag_boomerang_in_place(&mut self) {
        self.minigame.clear_flag_boomerang_in_place();
        self.sync();
    }

    pub(crate) fn set_flag_boomerang_in_place(&mut self, value: u8) {
        self.minigame.set_flag_boomerang_in_place(value);
        self.sync();
    }

    pub(crate) fn set_boomerang_temp_x(&mut self, value: u16) {
        self.minigame.set_boomerang_temp_x(value);
        self.sync();
    }

    pub(crate) fn set_boomerang_temp_y(&mut self, value: u16) {
        self.minigame.set_boomerang_temp_y(value);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct IntroSwordState {
    y_position: u16,
    sparkle_timer: u8,
    sparkle_step: u8,
    animation_step: u8,
    sparkle_y_offset: u8,
    flash_rgb_channel: u8,
    flash_rgb_channel_high: u8,
}

impl IntroSwordState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            y_position: read_le_u16(ram, INTRO_SWORD_YPOS),
            sparkle_timer: ram_byte(ram, INTRO_SWORD_SPARKLE_TIMER),
            sparkle_step: ram_byte(ram, INTRO_SWORD_SPARKLE_STEP),
            animation_step: ram_byte(ram, INTRO_SWORD_ANIM_STEP),
            sparkle_y_offset: ram_byte(ram, INTRO_SWORD_SPARKLE_Y_OFFSET),
            flash_rgb_channel: ram_byte(ram, INTRO_SWORD_FLASH_RGB_CHANNEL),
            flash_rgb_channel_high: ram_byte(ram, INTRO_SWORD_FLASH_RGB_CHANNEL + 1),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, INTRO_SWORD_YPOS, self.y_position);
        ram[INTRO_SWORD_SPARKLE_TIMER] = self.sparkle_timer;
        ram[INTRO_SWORD_SPARKLE_STEP] = self.sparkle_step;
        ram[INTRO_SWORD_ANIM_STEP] = self.animation_step;
        ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = self.sparkle_y_offset;
        ram[INTRO_SWORD_FLASH_RGB_CHANNEL] = self.flash_rgb_channel;
        ram[INTRO_SWORD_FLASH_RGB_CHANNEL + 1] = self.flash_rgb_channel_high;
    }

    pub(crate) fn ypos(&self) -> u16 {
        self.y_position
    }

    pub(crate) fn sparkle_timer(&self) -> u8 {
        self.sparkle_timer
    }

    pub(crate) fn sparkle_step(&self) -> u8 {
        self.sparkle_step
    }

    pub(crate) fn anim_phase(&self) -> u8 {
        self.animation_step >> 1
    }

    pub(crate) fn anim_step_raw(&self) -> u8 {
        self.animation_step
    }

    pub(crate) fn sparkle_y_offset(&self) -> u8 {
        self.sparkle_y_offset
    }

    pub(crate) fn flash_rgb_channel(&self) -> usize {
        usize::from(self.flash_rgb_channel)
    }

    pub(crate) fn reset_sword_state(&mut self) {
        self.sparkle_step = 7;
        self.animation_step = 0;
        self.sparkle_y_offset = 0;
        self.y_position = (-130i16) as u16;
    }

    pub(crate) fn set_ypos(&mut self, value: u16) {
        self.y_position = value;
    }

    pub(crate) fn advance_ypos(&mut self) {
        self.y_position = self.y_position.wrapping_add(16);
    }

    pub(crate) fn decrement_sparkle_timer(&mut self) {
        self.sparkle_timer = self.sparkle_timer.wrapping_sub(1);
    }

    pub(crate) fn set_sparkle_timer(&mut self, value: u8) {
        self.sparkle_timer = value;
    }

    pub(crate) fn set_sparkle_step(&mut self, value: u8) {
        self.sparkle_step = value;
    }

    pub(crate) fn decrement_sparkle_step_check_negative(&mut self) -> bool {
        self.sparkle_step = self.sparkle_step.wrapping_sub(1);
        (self.sparkle_step as i8) < 0
    }

    pub(crate) fn advance_anim_step(&mut self) {
        self.animation_step = self.animation_step.wrapping_add(2);
    }

    pub(crate) fn set_sparkle_y_offset(&mut self, value: u8) {
        self.sparkle_y_offset = value;
    }

    pub(crate) fn advance_sparkle_y_offset(&mut self) {
        self.sparkle_y_offset = self.sparkle_y_offset.wrapping_add(4);
    }

    pub(crate) fn set_flash_rgb_channel_word(&mut self, value: u16) {
        self.flash_rgb_channel = value as u8;
        self.flash_rgb_channel_high = (value >> 8) as u8;
    }

    pub(crate) fn cycle_flash_rgb_channel(&mut self) {
        self.flash_rgb_channel = if self.flash_rgb_channel == 2 {
            0
        } else {
            self.flash_rgb_channel.wrapping_add(1)
        };
    }
}

pub(crate) struct NativeIntroSwordBridgeMut<'a> {
    intro_sword: &'a mut IntroSwordState,
    ram: &'a mut [u8],
}

impl<'a> NativeIntroSwordBridgeMut<'a> {
    pub(crate) fn new(intro_sword: &'a mut IntroSwordState, ram: &'a mut [u8]) -> Self {
        Self { intro_sword, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.intro_sword, IntroSwordState::load_from_ram(self.ram));
    }

    fn sync(&mut self) {
        self.intro_sword.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn reset_sword_state(&mut self) {
        self.intro_sword.reset_sword_state();
        self.sync();
    }

    pub(crate) fn set_ypos(&mut self, value: u16) {
        self.intro_sword.set_ypos(value);
        self.sync();
    }

    pub(crate) fn advance_ypos(&mut self) {
        self.intro_sword.advance_ypos();
        self.sync();
    }

    pub(crate) fn decrement_sparkle_timer(&mut self) {
        self.intro_sword.decrement_sparkle_timer();
        self.sync();
    }

    pub(crate) fn set_sparkle_timer(&mut self, value: u8) {
        self.intro_sword.set_sparkle_timer(value);
        self.sync();
    }

    pub(crate) fn set_sparkle_step(&mut self, value: u8) {
        self.intro_sword.set_sparkle_step(value);
        self.sync();
    }

    pub(crate) fn decrement_sparkle_step_check_negative(&mut self) -> bool {
        let is_negative = self.intro_sword.decrement_sparkle_step_check_negative();
        self.sync();
        is_negative
    }

    pub(crate) fn advance_anim_step(&mut self) {
        self.intro_sword.advance_anim_step();
        self.sync();
    }

    pub(crate) fn set_sparkle_y_offset(&mut self, value: u8) {
        self.intro_sword.set_sparkle_y_offset(value);
        self.sync();
    }

    pub(crate) fn advance_sparkle_y_offset(&mut self) {
        self.intro_sword.advance_sparkle_y_offset();
        self.sync();
    }

    pub(crate) fn set_flash_rgb_channel_word(&mut self, value: u16) {
        self.intro_sword.set_flash_rgb_channel_word(value);
        self.sync();
    }

    pub(crate) fn cycle_flash_rgb_channel(&mut self) {
        self.intro_sword.cycle_flash_rgb_channel();
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ArcheryGameState {
    hit_counter: u8,
    arrows_left: u8,
    out_of_arrows: u8,
}

impl ArcheryGameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            hit_counter: ram_byte(ram, ARCHERY_GAME_HIT_COUNTER),
            arrows_left: ram_byte(ram, ARCHERY_GAME_ARROWS_LEFT),
            out_of_arrows: ram_byte(ram, ARCHERY_GAME_OUT_OF_ARROWS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[ARCHERY_GAME_HIT_COUNTER] = self.hit_counter;
        ram[ARCHERY_GAME_ARROWS_LEFT] = self.arrows_left;
        ram[ARCHERY_GAME_OUT_OF_ARROWS] = self.out_of_arrows;
    }

    pub(crate) fn hit_counter(&self) -> u8 {
        self.hit_counter
    }

    pub(crate) fn arrows_left(&self) -> u8 {
        self.arrows_left
    }

    pub(crate) fn out_of_arrows(&self) -> u8 {
        self.out_of_arrows
    }

    pub(crate) fn clear_hit_counter(&mut self) {
        self.hit_counter = 0;
    }

    pub(crate) fn increment_hit_counter(&mut self) {
        self.hit_counter = self.hit_counter.wrapping_add(1);
    }

    pub(crate) fn set_arrows_left(&mut self, value: u8) {
        self.arrows_left = value;
    }

    pub(crate) fn decrement_arrows_left(&mut self) {
        self.arrows_left = self.arrows_left.wrapping_sub(1);
    }

    pub(crate) fn increment_out_of_arrows(&mut self) {
        self.out_of_arrows = self.out_of_arrows.wrapping_add(1);
    }

    pub(crate) fn clear_out_of_arrows(&mut self) {
        self.out_of_arrows = 0;
    }
}

pub(crate) struct NativeArcheryGameBridgeMut<'a> {
    archery_game: &'a mut ArcheryGameState,
    ram: &'a mut [u8],
}

impl<'a> NativeArcheryGameBridgeMut<'a> {
    pub(crate) fn new(archery_game: &'a mut ArcheryGameState, ram: &'a mut [u8]) -> Self {
        Self { archery_game, ram }
    }

    fn sync(&mut self) {
        self.archery_game.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.archery_game,
            ArcheryGameState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_hit_counter(&mut self) {
        self.archery_game.clear_hit_counter();
        self.sync();
    }

    pub(crate) fn increment_hit_counter(&mut self) {
        self.archery_game.increment_hit_counter();
        self.sync();
    }

    pub(crate) fn set_arrows_left(&mut self, value: u8) {
        self.archery_game.set_arrows_left(value);
        self.sync();
    }

    pub(crate) fn decrement_arrows_left(&mut self) {
        self.archery_game.decrement_arrows_left();
        self.sync();
    }

    pub(crate) fn increment_out_of_arrows(&mut self) {
        self.archery_game.increment_out_of_arrows();
        self.sync();
    }

    pub(crate) fn clear_out_of_arrows(&mut self) {
        self.archery_game.clear_out_of_arrows();
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteBattleState {
    sprites_killed: u8,
    times_hurt_by_sprites: u8,
    item_drop_luck: u8,
    luck_kill_counter: u8,
    item_drop_counter: u8,
    damage_type_determiner: u8,
    damaging_enemies_timer: u8,
}

impl SpriteBattleState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            sprites_killed: ram_byte(ram, NUM_SPRITES_KILLED),
            times_hurt_by_sprites: ram_byte(ram, TIMES_HURT_BY_SPRITES),
            item_drop_luck: ram_byte(ram, ITEM_DROP_LUCK),
            luck_kill_counter: ram_byte(ram, LUCK_KILL_COUNTER),
            item_drop_counter: ram_byte(ram, ITEM_DROP_COUNTER),
            damage_type_determiner: ram_byte(ram, DAMAGE_TYPE_DETERMINER),
            damaging_enemies_timer: ram_byte(ram, SET_WHEN_DAMAGING_ENEMIES),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[NUM_SPRITES_KILLED] = self.sprites_killed;
        ram[TIMES_HURT_BY_SPRITES] = self.times_hurt_by_sprites;
        ram[ITEM_DROP_LUCK] = self.item_drop_luck;
        ram[LUCK_KILL_COUNTER] = self.luck_kill_counter;
        ram[ITEM_DROP_COUNTER] = self.item_drop_counter;
        ram[DAMAGE_TYPE_DETERMINER] = self.damage_type_determiner;
        ram[SET_WHEN_DAMAGING_ENEMIES] = self.damaging_enemies_timer;
    }

    pub(crate) fn sprites_killed(&self) -> u8 {
        self.sprites_killed
    }

    pub(crate) fn times_hurt_by_sprites(&self) -> u8 {
        self.times_hurt_by_sprites
    }

    pub(crate) fn item_drop_luck(&self) -> u8 {
        self.item_drop_luck
    }

    pub(crate) fn luck_kill_counter(&self) -> u8 {
        self.luck_kill_counter
    }

    pub(crate) fn item_drop_counter(&self) -> u8 {
        self.item_drop_counter
    }

    pub(crate) fn damage_type_determiner(&self) -> u8 {
        self.damage_type_determiner
    }

    pub(crate) fn damaging_enemies_timer(&self) -> u8 {
        self.damaging_enemies_timer
    }
}

pub(crate) struct NativeSpriteBattleBridgeMut<'a> {
    sprite_battle: &'a mut SpriteBattleState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteBattleBridgeMut<'a> {
    pub(crate) fn new(sprite_battle: &'a mut SpriteBattleState, ram: &'a mut [u8]) -> Self {
        Self { sprite_battle, ram }
    }

    fn sync(&mut self) {
        self.sprite_battle.write_to_ram(self.ram);
        debug_assert_eq!(
            *self.sprite_battle,
            SpriteBattleState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_sprites_killed(&mut self) {
        self.sprite_battle.sprites_killed = 0;
        self.sync();
    }

    pub(crate) fn set_sprites_killed(&mut self, value: u8) {
        self.sprite_battle.sprites_killed = value;
        self.sync();
    }

    pub(crate) fn clear_times_hurt_by_sprites(&mut self) {
        self.sprite_battle.times_hurt_by_sprites = 0;
        self.sync();
    }

    pub(crate) fn set_times_hurt_by_sprites(&mut self, value: u8) {
        self.sprite_battle.times_hurt_by_sprites = value;
        self.sync();
    }

    pub(crate) fn increment_times_hurt_by_sprites(&mut self) {
        self.sprite_battle.times_hurt_by_sprites =
            self.sprite_battle.times_hurt_by_sprites.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn set_item_drop_luck(&mut self, value: u8) {
        self.sprite_battle.item_drop_luck = value;
        self.sync();
    }

    pub(crate) fn clear_luck_kill_counter(&mut self) {
        self.sprite_battle.luck_kill_counter = 0;
        self.sync();
    }

    pub(crate) fn clear_item_drop_counter(&mut self) {
        self.sprite_battle.item_drop_counter = 0;
        self.sync();
    }

    pub(crate) fn set_item_drop_counter(&mut self, value: u8) {
        self.sprite_battle.item_drop_counter = value;
        self.sync();
    }

    pub(crate) fn increment_sprites_killed(&mut self) {
        self.sprite_battle.sprites_killed = self.sprite_battle.sprites_killed.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn increment_luck_kill_counter(&mut self) {
        self.sprite_battle.luck_kill_counter = self.sprite_battle.luck_kill_counter.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn set_damage_type_determiner(&mut self, value: u8) {
        self.sprite_battle.damage_type_determiner = value;
        self.sync();
    }

    pub(crate) fn set_damaging_enemies_timer(&mut self, value: u8) {
        self.sprite_battle.damaging_enemies_timer = value;
        self.sync();
    }

    pub(crate) fn clear_damaging_enemies_timer(&mut self) {
        self.sprite_battle.damaging_enemies_timer = 0;
        self.sync();
    }

    pub(crate) fn tick_damaging_enemies_timer(&mut self) {
        self.sprite_battle.damaging_enemies_timer =
            if self.sprite_battle.damaging_enemies_timer & 0x7f != 0 {
                self.sprite_battle.damaging_enemies_timer.wrapping_sub(1)
            } else {
                0
            };
        self.sync();
    }

    pub(crate) fn increment_item_drop_counter(&mut self) -> u8 {
        self.sprite_battle.item_drop_counter = self.sprite_battle.item_drop_counter.wrapping_add(1);
        let value = self.sprite_battle.item_drop_counter;
        self.sync();
        value
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EnhancedFeaturesState {
    bits: u32,
}

impl EnhancedFeaturesState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            bits: u32::from(ram_byte(ram, ENHANCED_FEATURE_FLAGS))
                | (u32::from(ram_byte(ram, ENHANCED_FEATURE_FLAGS + 1)) << 8)
                | (u32::from(ram_byte(ram, ENHANCED_FEATURE_FLAGS + 2)) << 16)
                | (u32::from(ram_byte(ram, ENHANCED_FEATURE_FLAGS + 3)) << 24),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        let bytes = self.bits.to_le_bytes();
        ram[ENHANCED_FEATURE_FLAGS..ENHANCED_FEATURE_FLAGS + 4].copy_from_slice(&bytes);
    }

    pub(crate) fn bits(&self) -> u32 {
        self.bits
    }

    pub(crate) fn has(&self, mask: u32) -> bool {
        self.bits & mask != 0
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

pub(crate) struct NativeEnhancedFeaturesBridgeMut<'a> {
    enhanced_features: &'a mut EnhancedFeaturesState,
    ram: &'a mut [u8],
}

impl<'a> NativeEnhancedFeaturesBridgeMut<'a> {
    pub(crate) fn new(enhanced_features: &'a mut EnhancedFeaturesState, ram: &'a mut [u8]) -> Self {
        Self {
            enhanced_features,
            ram,
        }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.enhanced_features,
            EnhancedFeaturesState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_bits(&mut self, value: u32) {
        self.enhanced_features.bits = value;
        self.enhanced_features.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }
}
