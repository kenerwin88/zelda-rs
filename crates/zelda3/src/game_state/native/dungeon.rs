use crate::game_state::constants::{
    AUX_TILE_THEME_INDEX, DUNGEON_HEADER_HOLE_TELEPORTER_PLANE, DUNGEON_HEADER_STAIRCASE_PLANE,
    DUNGEON_HEADER_TRAVEL_DESTINATIONS, DUNGEON_TORCH_ATTR, DUNGEON_TORCH_DATA, DUNGEON_WORK_R16,
    DUNGEON_WORK_R18, DUNG_INDEX_OF_TORCHES_START, DUNG_OBJECT_POS_IN_OBJDATA, GANON_TORCH_COUNT,
    MAIN_TILE_THEME_INDEX, MOVABLE_BLOCK_DATAS, OVERLAY_INDEX, OVERWORLD_EXIT_TILE_THEME_INDEX,
    OVERWORLD_SCREEN_INDEX, OVERWORLD_TILE_THEME_INDEX, SPRITE_GRAPHICS_INDEX, TORCH_TIMERS,
};
use crate::types::{read_le_u16, write_le_u16};

const DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT: usize = 5;
const DUNGEON_HEADER_PLANE_SCRATCH_COUNT: usize = 5;
const DUNGEON_TORCH_TIMER_COUNT: usize = 16;
const DUNGEON_TORCH_OBJECT_POS_COUNT: usize = 16;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonState {
    pub(crate) header: DungeonHeaderState,
    pub(crate) scratch_word: DungeonScratchWordState,
    pub(crate) entrance_backup: DungeonEntranceBackupState,
    pub(crate) torch: DungeonTorchState,
}

impl DungeonState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            header: DungeonHeaderState::load_from_ram(ram),
            scratch_word: DungeonScratchWordState::load_from_ram(ram),
            entrance_backup: DungeonEntranceBackupState::load_from_ram(ram),
            torch: DungeonTorchState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.header.write_to_ram(ram);
        self.scratch_word.write_to_ram(ram);
        self.entrance_backup.write_to_ram(ram);
        self.torch.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonTorchState {
    timers: [u8; DUNGEON_TORCH_TIMER_COUNT],
    attr: u8,
    ganon_torch_count: u8,
    torches_start_index: u16,
    object_data_positions: [u16; DUNGEON_TORCH_OBJECT_POS_COUNT],
}

impl DungeonTorchState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut timers = [0; DUNGEON_TORCH_TIMER_COUNT];
        for (index, timer) in timers.iter_mut().enumerate() {
            *timer = ram.get(TORCH_TIMERS + index).copied().unwrap_or(0);
        }

        let mut object_data_positions = [0; DUNGEON_TORCH_OBJECT_POS_COUNT];
        for (index, position) in object_data_positions.iter_mut().enumerate() {
            *position = read_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2);
        }

        Self {
            timers,
            attr: ram.get(DUNGEON_TORCH_ATTR).copied().unwrap_or(0),
            ganon_torch_count: ram.get(GANON_TORCH_COUNT).copied().unwrap_or(0),
            torches_start_index: read_le_u16(ram, DUNG_INDEX_OF_TORCHES_START),
            object_data_positions,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TORCH_TIMERS..TORCH_TIMERS + DUNGEON_TORCH_TIMER_COUNT].copy_from_slice(&self.timers);
        ram[DUNGEON_TORCH_ATTR] = self.attr;
        ram[GANON_TORCH_COUNT] = self.ganon_torch_count;
        write_le_u16(ram, DUNG_INDEX_OF_TORCHES_START, self.torches_start_index);
        for (index, position) in self.object_data_positions.iter().enumerate() {
            write_le_u16(ram, DUNG_OBJECT_POS_IN_OBJDATA + index * 2, *position);
        }
    }

    pub(crate) fn timer(&self, index: usize) -> u8 {
        self.timers.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn attr_index(&self) -> usize {
        usize::from(self.attr & 0x0f)
    }

    pub(crate) fn torch_attr(&self) -> u8 {
        self.attr
    }

    pub(crate) fn ganon_torch_count(&self) -> u8 {
        self.ganon_torch_count
    }

    pub(crate) fn torches_start_index(&self) -> u16 {
        self.torches_start_index
    }

    pub(crate) fn torch_object_data_pos(&self, index: usize) -> u16 {
        self.object_data_positions.get(index).copied().unwrap_or(0)
    }

    fn clear_timer(&mut self, index: usize) {
        if let Some(timer) = self.timers.get_mut(index) {
            *timer = 0;
        }
    }

    fn set_timer(&mut self, index: usize, value: u8) {
        if let Some(timer) = self.timers.get_mut(index) {
            *timer = value;
        }
    }

    fn set_attr(&mut self, value: u8) {
        self.attr = value;
    }

    fn clear_attr(&mut self) {
        self.attr = 0;
    }

    fn set_ganon_torch_count(&mut self, value: u8) {
        self.ganon_torch_count = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonEntranceBackupState {
    exit_tile_themes: [u8; 4],
    overworld_screen_high: u8,
    overlay_high: u8,
}

impl DungeonEntranceBackupState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut exit_tile_themes = [0; 4];
        for (index, theme) in exit_tile_themes.iter_mut().enumerate() {
            *theme = ram
                .get(OVERWORLD_EXIT_TILE_THEME_INDEX + index)
                .copied()
                .unwrap_or(0);
        }
        Self {
            exit_tile_themes,
            overworld_screen_high: ram.get(OVERWORLD_SCREEN_INDEX + 1).copied().unwrap_or(0),
            overlay_high: ram.get(OVERLAY_INDEX + 1).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[OVERWORLD_EXIT_TILE_THEME_INDEX..OVERWORLD_EXIT_TILE_THEME_INDEX + 4]
            .copy_from_slice(&self.exit_tile_themes);
        ram[OVERWORLD_SCREEN_INDEX + 1] = self.overworld_screen_high;
        ram[OVERLAY_INDEX + 1] = self.overlay_high;
    }

    pub(crate) fn exit_tile_theme(&self, index: usize) -> u8 {
        self.exit_tile_themes.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn overworld_screen_high(&self) -> u8 {
        self.overworld_screen_high
    }

    pub(crate) fn overlay_high(&self) -> u8 {
        self.overlay_high
    }

    pub(crate) fn cache_exit_tile_themes(&mut self, overworld: u8, main: u8, aux: u8, sprite: u8) {
        self.exit_tile_themes = [overworld, main, aux, sprite];
    }

    pub(crate) fn clear_overworld_screen_high(&mut self) {
        self.overworld_screen_high = 0;
    }

    pub(crate) fn clear_overlay_high(&mut self) {
        self.overlay_high = 0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonHeaderState {
    travel_destinations: [u8; DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT],
    plane_scratch: [u8; DUNGEON_HEADER_PLANE_SCRATCH_COUNT],
}

impl DungeonHeaderState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut travel_destinations = [0; DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT];
        for (index, destination) in travel_destinations.iter_mut().enumerate() {
            *destination = ram
                .get(DUNGEON_HEADER_TRAVEL_DESTINATIONS + index)
                .copied()
                .unwrap_or(0);
        }

        let mut plane_scratch = [0; DUNGEON_HEADER_PLANE_SCRATCH_COUNT];
        for (index, plane) in plane_scratch.iter_mut().enumerate() {
            *plane = ram
                .get(DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + index)
                .copied()
                .unwrap_or(0);
        }

        Self {
            travel_destinations,
            plane_scratch,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS
            ..DUNGEON_HEADER_TRAVEL_DESTINATIONS + DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT]
            .copy_from_slice(&self.travel_destinations);
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE
            ..DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + DUNGEON_HEADER_PLANE_SCRATCH_COUNT]
            .copy_from_slice(&self.plane_scratch);
    }

    pub(crate) fn travel_destination(&self, index: usize) -> u8 {
        self.travel_destinations.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn hole_teleporter_plane(&self, index: usize) -> u8 {
        self.plane_scratch.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn staircase_plane(&self, index: usize) -> u8 {
        self.plane_scratch
            .get(DUNGEON_HEADER_STAIRCASE_PLANE - DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + index)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn set_hole_teleporter_planes(&mut self, packed: u8, extra: u8) {
        self.plane_scratch[0] = packed & 3;
        self.plane_scratch[1] = (packed >> 2) & 3;
        self.plane_scratch[2] = (packed >> 4) & 3;
        self.plane_scratch[3] = (packed >> 6) & 3;
        self.plane_scratch[4] = extra & 3;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonScratchWordState {
    r16: u16,
    r18: u16,
}

impl DungeonScratchWordState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            r16: read_le_u16(ram, DUNGEON_WORK_R16),
            r18: read_le_u16(ram, DUNGEON_WORK_R18),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, DUNGEON_WORK_R16, self.r16);
        write_le_u16(ram, DUNGEON_WORK_R18, self.r18);
    }

    pub(crate) fn high(&self) -> u8 {
        (self.r16 >> 8) as u8
    }

    pub(crate) fn word(&self) -> u16 {
        self.r16
    }

    pub(crate) fn minigame_previous_chest_choice(&self) -> u8 {
        self.r16 as u8
    }

    pub(crate) fn primary_word(&self) -> u16 {
        self.r16
    }

    pub(crate) fn secondary_word(&self) -> u16 {
        self.r18
    }

    pub(crate) fn primary_low(&self) -> u8 {
        self.r16 as u8
    }

    pub(crate) fn secondary_low(&self) -> u8 {
        self.r18 as u8
    }

    pub(crate) fn decrement_high(&mut self) -> u8 {
        let next = self.high().wrapping_sub(1);
        self.r16 = (self.r16 & 0x00ff) | (u16::from(next) << 8);
        next
    }

    pub(crate) fn set_word(&mut self, value: u16) {
        self.r16 = value;
    }

    pub(crate) fn clear_word(&mut self) {
        self.set_word(0);
    }

    pub(crate) fn set_liftable_tile_probe_position(&mut self, y: u16, x: u16) {
        self.r16 = y;
        self.r18 = x;
    }

    pub(crate) fn set_ganon_door_bounce_countdown(&mut self, value: u16) {
        self.set_word(value);
    }

    pub(crate) fn decrement_ganon_door_bounce_low(&mut self) -> u8 {
        let next = (self.r16 as u8).wrapping_sub(1);
        self.r16 = (self.r16 & 0xff00) | u16::from(next);
        next
    }

    pub(crate) fn clear_module_transition_counter(&mut self) {
        self.r16 = (self.r16 & 0xff00) | 0;
    }

    pub(crate) fn set_minigame_previous_chest_choice(&mut self, value: u8) {
        self.r16 = (self.r16 & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_primary_word(&mut self, value: u16) {
        self.r16 = value;
    }

    pub(crate) fn set_secondary_word(&mut self, value: u16) {
        self.r18 = value;
    }

    pub(crate) fn clear_primary_word(&mut self) {
        self.set_primary_word(0);
    }

    pub(crate) fn set_primary_low(&mut self, value: u8) {
        self.r16 = (self.r16 & 0xff00) | u16::from(value);
    }

    pub(crate) fn decrement_primary_low(&mut self) -> u8 {
        let next = self.primary_low().wrapping_sub(1);
        self.set_primary_low(next);
        next
    }

    pub(crate) fn increment_secondary_low(&mut self) -> u8 {
        let next = self.secondary_low().wrapping_add(1);
        self.r18 = (self.r18 & 0xff00) | u16::from(next);
        next
    }
}

pub(crate) struct NativeDungeonEntranceBackupBridgeMut<'a> {
    state: &'a mut DungeonEntranceBackupState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonEntranceBackupBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DungeonEntranceBackupState, ram: &'a mut [u8]) -> Self {
        *state = DungeonEntranceBackupState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DungeonEntranceBackupState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn cache_exit_tile_themes(&mut self) {
        self.state.cache_exit_tile_themes(
            self.ram[OVERWORLD_TILE_THEME_INDEX],
            self.ram[MAIN_TILE_THEME_INDEX],
            self.ram[AUX_TILE_THEME_INDEX],
            self.ram[SPRITE_GRAPHICS_INDEX],
        );
        self.sync();
    }

    pub(crate) fn clear_overworld_screen_high(&mut self) {
        self.state.clear_overworld_screen_high();
        self.sync();
    }

    pub(crate) fn clear_overlay_high(&mut self) {
        self.state.clear_overlay_high();
        self.sync();
    }
}

pub(crate) struct NativeDungeonScratchWordBridgeMut<'a> {
    scratch: &'a mut DungeonScratchWordState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonScratchWordBridgeMut<'a> {
    pub(crate) fn new(scratch: &'a mut DungeonScratchWordState, ram: &'a mut [u8]) -> Self {
        *scratch = DungeonScratchWordState::load_from_ram(ram);
        Self { scratch, ram }
    }

    fn sync(&mut self) {
        self.scratch.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.scratch,
            DungeonScratchWordState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn decrement_high(&mut self) -> u8 {
        let next = self.scratch.decrement_high();
        self.sync();
        next
    }

    pub(crate) fn set_word(&mut self, value: u16) {
        self.scratch.set_word(value);
        self.sync();
    }

    pub(crate) fn clear_word(&mut self) {
        self.scratch.clear_word();
        self.sync();
    }

    pub(crate) fn set_liftable_tile_probe_position(&mut self, y: u16, x: u16) {
        self.scratch.set_liftable_tile_probe_position(y, x);
        self.sync();
    }

    pub(crate) fn set_ganon_door_bounce_countdown(&mut self, value: u16) {
        self.scratch.set_ganon_door_bounce_countdown(value);
        self.sync();
    }

    pub(crate) fn decrement_ganon_door_bounce_low(&mut self) -> u8 {
        let next = self.scratch.decrement_ganon_door_bounce_low();
        self.sync();
        next
    }

    pub(crate) fn clear_module_transition_counter(&mut self) {
        self.scratch.clear_module_transition_counter();
        self.sync();
    }

    pub(crate) fn set_minigame_previous_chest_choice(&mut self, value: u8) {
        self.scratch.set_minigame_previous_chest_choice(value);
        self.sync();
    }

    pub(crate) fn set_primary_word(&mut self, value: u16) {
        self.scratch.set_primary_word(value);
        self.sync();
    }

    pub(crate) fn set_secondary_word(&mut self, value: u16) {
        self.scratch.set_secondary_word(value);
        self.sync();
    }

    pub(crate) fn clear_primary_word(&mut self) {
        self.scratch.clear_primary_word();
        self.sync();
    }

    pub(crate) fn set_primary_low(&mut self, value: u8) {
        self.scratch.set_primary_low(value);
        self.sync();
    }

    pub(crate) fn decrement_primary_low(&mut self) -> u8 {
        let next = self.scratch.decrement_primary_low();
        self.sync();
        next
    }

    pub(crate) fn increment_secondary_low(&mut self) -> u8 {
        let next = self.scratch.increment_secondary_low();
        self.sync();
        next
    }
}

pub(crate) struct NativeDungeonTorchBridgeMut<'a> {
    torch: &'a mut DungeonTorchState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonTorchBridgeMut<'a> {
    pub(crate) fn new(torch: &'a mut DungeonTorchState, ram: &'a mut [u8]) -> Self {
        *torch = DungeonTorchState::load_from_ram(ram);
        Self { torch, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.torch, DungeonTorchState::load_from_ram(self.ram));
    }

    pub(crate) fn copy_torch_init_to_movable_blocks(&mut self, torch_init: &[u8]) {
        self.ram[MOVABLE_BLOCK_DATAS + 99 * 4..MOVABLE_BLOCK_DATAS + 99 * 4 + 116]
            .copy_from_slice(&torch_init[..116]);
    }

    pub(crate) fn copy_torch_junk(&mut self, torch_junk: &[u8]) {
        self.ram[DUNGEON_TORCH_DATA + 144 * 2..DUNGEON_TORCH_DATA + 144 * 2 + torch_junk.len()]
            .copy_from_slice(torch_junk);
    }

    pub(crate) fn clear_timer(&mut self, index: usize) {
        self.torch.clear_timer(index);
        if index < DUNGEON_TORCH_TIMER_COUNT {
            self.ram[TORCH_TIMERS + index] = 0;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_timer(&mut self, index: usize, value: u8) {
        self.torch.set_timer(index, value);
        if index < DUNGEON_TORCH_TIMER_COUNT {
            self.ram[TORCH_TIMERS + index] = value;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_torch_data_word(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNGEON_TORCH_DATA + index * 2, value);
    }

    pub(crate) fn set_attr(&mut self, value: u8) {
        self.torch.set_attr(value);
        self.ram[DUNGEON_TORCH_ATTR] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_ganon_torch_count(&mut self, value: u8) {
        self.torch.set_ganon_torch_count(value);
        self.ram[GANON_TORCH_COUNT] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_attr(&mut self) {
        self.torch.clear_attr();
        self.ram[DUNGEON_TORCH_ATTR] = 0;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn refresh_object_data_positions(&mut self) {
        *self.torch = DungeonTorchState::load_from_ram(self.ram);
    }
}

pub(crate) struct NativeDungeonHeaderBridgeMut<'a> {
    header: &'a mut DungeonHeaderState,
    ram: &'a mut [u8],
}

impl<'a> NativeDungeonHeaderBridgeMut<'a> {
    pub(crate) fn new(header: &'a mut DungeonHeaderState, ram: &'a mut [u8]) -> Self {
        *header = DungeonHeaderState::load_from_ram(ram);
        Self { header, ram }
    }

    fn sync(&mut self) {
        self.header.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.header, DungeonHeaderState::load_from_ram(self.ram));
    }

    pub(crate) fn set_hole_teleporter_planes(&mut self, packed: u8, extra: u8) {
        self.header.set_hole_teleporter_planes(packed, extra);
        self.sync();
    }
}
