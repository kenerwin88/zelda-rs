use crate::game_state::constants::{
    DUNGEON_HEADER_HOLE_TELEPORTER_PLANE, DUNGEON_HEADER_STAIRCASE_PLANE,
    DUNGEON_HEADER_TRAVEL_DESTINATIONS,
};

const DUNGEON_HEADER_TRAVEL_DESTINATION_COUNT: usize = 5;
const DUNGEON_HEADER_PLANE_SCRATCH_COUNT: usize = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonState {
    pub(crate) header: DungeonHeaderState,
}

impl DungeonState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            header: DungeonHeaderState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.header.write_to_ram(ram);
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
