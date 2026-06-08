//! Internal RAM semantics grouped by subsystem.

use crate::types::{read_le_u16, write_le_u16};

pub(crate) mod semantic {
    use super::{read_le_u16, write_le_u16};

    // Source addresses for semantic parity snapshots. Keep these constants close
    // to the typed views so every semantic field has an auditable WRAM source.
    const MAIN_MODULE: usize = 0x0010;
    const SUBMODULE: usize = 0x0011;
    const SUBSUBMODULE: usize = 0x00b0;
    const NMI_THREAD_ACTIVE: usize = 0x012a;
    const POLY_THREAD_STACK: usize = 0x1f0a;
    const RUN_MAIN_THREAD: u8 = 1;
    const RUN_POLY_THREAD: u8 = 2;

    const LINK_Y_COORD: usize = 0x0020;
    const LINK_X_COORD: usize = 0x0022;
    const LINK_Z_COORD: usize = 0x0024;
    const LINK_LAST_DIRECTION: usize = 0x0026;
    const LINK_Z_VELOCITY: usize = 0x0029;
    const LINK_FACING: usize = 0x002f;
    const LINK_Y_VELOCITY: usize = 0x0030;
    const LINK_X_VELOCITY: usize = 0x0031;
    const LINK_AUXILIARY_STATE: usize = 0x004d;
    const LINK_HANDLER_STATE: usize = 0x005d;
    const LINK_DIRECTION: usize = 0x0067;
    const LINK_ITEM_IN_HAND: usize = 0x0301;
    const LINK_CURRENT_ITEM_Y: usize = 0x0303;
    const LINK_CURRENT_ITEM_ACTIVE: usize = 0x0304;
    const LINK_CURRENT_HEALTH: usize = 0x0f36d;
    const LINK_MAGIC_POWER: usize = 0x0f36e;
    const LINK_EQUIPPED_ITEM: usize = 0x0f340;

    const DUNGEON_ROOM: usize = 0x00a0;
    const OVERWORLD_SCREEN_INDEX: usize = 0x008a;
    const OVERWORLD_AREA_INDEX: usize = 0x040a;
    const OVERWORLD_TRANSITION_DIR: usize = 0x069c;
    const OVERLAY_INDEX: usize = 0x008c;
    const MAP16_LOAD_SRC_OFF: usize = 0x0084;
    const MAP16_LOAD_DST_OFF: usize = 0x0086;
    const MAP16_LOAD_Y_UNIT: usize = 0x0088;
    const BG1_X_SCROLL: usize = 0x00e0;
    const BG2_X_SCROLL: usize = 0x00e2;
    const BG1_Y_SCROLL: usize = 0x00e6;
    const BG2_Y_SCROLL: usize = 0x00e8;
    const CAMERA_Y: usize = 0x0618;
    const CAMERA_X: usize = 0x061c;
    const RNG_SEED: usize = 0x0fa1;

    const SPRITE_STATE: usize = 0x0dd0;
    const SPRITE_TYPE: usize = 0x0e20;
    const SPRITE_Y_LO: usize = 0x0d00;
    const SPRITE_X_LO: usize = 0x0d10;
    const SPRITE_Y_HI: usize = 0x0d20;
    const SPRITE_X_HI: usize = 0x0d30;
    const SPRITE_Y_VELOCITY: usize = 0x0d40;
    const SPRITE_X_VELOCITY: usize = 0x0d50;
    const SPRITE_Y_SUBPIXEL: usize = 0x0d60;
    const SPRITE_X_SUBPIXEL: usize = 0x0d70;
    const SPRITE_AI_STATE: usize = 0x0d80;
    const SPRITE_DELAY_MAIN: usize = 0x0df0;
    const SPRITE_HEALTH: usize = 0x0e50;
    const SPRITE_HIT_TIMER: usize = 0x0ef0;
    const SPRITE_Z: usize = 0x0f70;
    const SPRITE_Z_VELOCITY: usize = 0x0f80;
    const SPRITE_Z_SUBPIXEL: usize = 0x0f90;

    const ANCILLA_Z_VELOCITY: usize = 0x0294;
    const ANCILLA_Z: usize = 0x029e;
    const ANCILLA_Z_SUBPIXEL_PLAYER: usize = 0x02a8;
    const ANCILLA_Y_LO: usize = 0x0bfa;
    const ANCILLA_X_LO: usize = 0x0c04;
    const ANCILLA_Y_HI: usize = 0x0c0e;
    const ANCILLA_X_HI: usize = 0x0c18;
    const ANCILLA_Y_VELOCITY: usize = 0x0c22;
    const ANCILLA_X_VELOCITY: usize = 0x0c2c;
    const ANCILLA_Y_SUBPIXEL: usize = 0x0c36;
    const ANCILLA_X_SUBPIXEL: usize = 0x0c40;
    const ANCILLA_TYPE: usize = 0x0c4a;
    const ANCILLA_ITEM_TO_LINK: usize = 0x0c5e;
    const ANCILLA_TIMER: usize = 0x0c68;
    const ANCILLA_DIRECTION: usize = 0x0c72;

    pub(crate) struct FrameControlView<'a> {
        ram: &'a [u8],
    }

    impl<'a> FrameControlView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn main_module(&self) -> u8 {
            byte(self.ram, MAIN_MODULE)
        }

        pub(crate) fn submodule(&self) -> u8 {
            byte(self.ram, SUBMODULE)
        }

        pub(crate) fn subsubmodule(&self) -> u8 {
            byte(self.ram, SUBSUBMODULE)
        }

        pub(crate) fn nmi_thread_active(&self) -> bool {
            byte(self.ram, NMI_THREAD_ACTIVE) != 0
        }

        pub(crate) fn selected_run_thread(&self) -> u8 {
            if self.nmi_thread_active() && word(self.ram, POLY_THREAD_STACK) != 0x1f31 {
                RUN_POLY_THREAD
            } else {
                RUN_MAIN_THREAD
            }
        }
    }

    pub(crate) struct FrameControlViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> FrameControlViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_main_module(&mut self, value: u8) {
            self.ram[MAIN_MODULE] = value;
        }

        pub(crate) fn set_submodule(&mut self, value: u8) {
            self.ram[SUBMODULE] = value;
        }

        pub(crate) fn set_subsubmodule(&mut self, value: u8) {
            self.ram[SUBSUBMODULE] = value;
        }

        pub(crate) fn increment_submodule(&mut self) {
            self.ram[SUBMODULE] = self.ram[SUBMODULE].wrapping_add(1);
        }

        pub(crate) fn decrement_submodule(&mut self) {
            self.ram[SUBMODULE] = self.ram[SUBMODULE].wrapping_sub(1);
        }

        pub(crate) fn increment_subsubmodule(&mut self) {
            self.ram[SUBSUBMODULE] = self.ram[SUBSUBMODULE].wrapping_add(1);
        }

        pub(crate) fn decrement_subsubmodule(&mut self) {
            self.ram[SUBSUBMODULE] = self.ram[SUBSUBMODULE].wrapping_sub(1);
        }
    }

    pub(crate) struct PlayerStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> PlayerStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn x(&self) -> u16 {
            word(self.ram, LINK_X_COORD)
        }

        pub(crate) fn y(&self) -> u16 {
            word(self.ram, LINK_Y_COORD)
        }

        pub(crate) fn z(&self) -> u16 {
            word(self.ram, LINK_Z_COORD)
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            byte(self.ram, LINK_X_VELOCITY)
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            byte(self.ram, LINK_Y_VELOCITY)
        }

        pub(crate) fn z_velocity(&self) -> u8 {
            byte(self.ram, LINK_Z_VELOCITY)
        }

        pub(crate) fn direction(&self) -> u8 {
            byte(self.ram, LINK_DIRECTION)
        }

        pub(crate) fn last_direction(&self) -> u8 {
            byte(self.ram, LINK_LAST_DIRECTION)
        }

        pub(crate) fn facing(&self) -> u8 {
            byte(self.ram, LINK_FACING)
        }

        pub(crate) fn handler_state(&self) -> u8 {
            byte(self.ram, LINK_HANDLER_STATE)
        }

        pub(crate) fn auxiliary_state(&self) -> u8 {
            byte(self.ram, LINK_AUXILIARY_STATE)
        }

        pub(crate) fn current_health(&self) -> u8 {
            byte(self.ram, LINK_CURRENT_HEALTH)
        }

        pub(crate) fn magic_power(&self) -> u8 {
            byte(self.ram, LINK_MAGIC_POWER)
        }

        pub(crate) fn item_in_hand(&self) -> u8 {
            byte(self.ram, LINK_ITEM_IN_HAND)
        }

        pub(crate) fn current_item_y(&self) -> u8 {
            byte(self.ram, LINK_CURRENT_ITEM_Y)
        }

        pub(crate) fn current_item_active(&self) -> u8 {
            byte(self.ram, LINK_CURRENT_ITEM_ACTIVE)
        }

        pub(crate) fn equipped_item(&self) -> u8 {
            byte(self.ram, LINK_EQUIPPED_ITEM)
        }
    }

    pub(crate) struct PlayerStateViewMut<'a> {
        ram: &'a mut [u8],
    }

    impl<'a> PlayerStateViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_X_COORD, value);
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_Y_COORD, value);
        }

        pub(crate) fn set_z(&mut self, value: u16) {
            write_le_u16(self.ram, LINK_Z_COORD, value);
        }
    }

    pub(crate) struct WorldStateView<'a> {
        ram: &'a [u8],
    }

    impl<'a> WorldStateView<'a> {
        pub(crate) fn new(ram: &'a [u8]) -> Self {
            Self { ram }
        }

        pub(crate) fn dungeon_room(&self) -> u16 {
            word(self.ram, DUNGEON_ROOM)
        }

        pub(crate) fn overworld_screen(&self) -> u8 {
            byte(self.ram, OVERWORLD_SCREEN_INDEX)
        }

        pub(crate) fn overworld_area(&self) -> u16 {
            word(self.ram, OVERWORLD_AREA_INDEX)
        }

        pub(crate) fn transition_direction(&self) -> u8 {
            byte(self.ram, OVERWORLD_TRANSITION_DIR)
        }

        pub(crate) fn overlay_index(&self) -> u8 {
            byte(self.ram, OVERLAY_INDEX)
        }

        pub(crate) fn map16_load_src(&self) -> u16 {
            word(self.ram, MAP16_LOAD_SRC_OFF)
        }

        pub(crate) fn map16_load_dst(&self) -> u16 {
            word(self.ram, MAP16_LOAD_DST_OFF)
        }

        pub(crate) fn map16_load_y_unit(&self) -> u16 {
            word(self.ram, MAP16_LOAD_Y_UNIT)
        }

        pub(crate) fn bg1_x(&self) -> u16 {
            word(self.ram, BG1_X_SCROLL)
        }

        pub(crate) fn bg1_y(&self) -> u16 {
            word(self.ram, BG1_Y_SCROLL)
        }

        pub(crate) fn bg2_x(&self) -> u16 {
            word(self.ram, BG2_X_SCROLL)
        }

        pub(crate) fn bg2_y(&self) -> u16 {
            word(self.ram, BG2_Y_SCROLL)
        }

        pub(crate) fn camera_x(&self) -> u16 {
            word(self.ram, CAMERA_X)
        }

        pub(crate) fn camera_y(&self) -> u16 {
            word(self.ram, CAMERA_Y)
        }

        pub(crate) fn rng_seed(&self) -> u8 {
            byte(self.ram, RNG_SEED)
        }
    }

    pub(crate) struct SpriteSlotView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> SpriteSlotView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn slot(&self) -> u8 {
            self.slot as u8
        }

        pub(crate) fn sprite_type(&self) -> u8 {
            byte(self.ram, SPRITE_TYPE + self.slot)
        }

        pub(crate) fn state(&self) -> u8 {
            byte(self.ram, SPRITE_STATE + self.slot)
        }

        pub(crate) fn is_active(&self) -> bool {
            self.sprite_type() != 0 || self.state() != 0
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(self.ram, SPRITE_X_LO + self.slot, SPRITE_X_HI + self.slot)
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(self.ram, SPRITE_Y_LO + self.slot, SPRITE_Y_HI + self.slot)
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            byte(self.ram, SPRITE_X_VELOCITY + self.slot)
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            byte(self.ram, SPRITE_Y_VELOCITY + self.slot)
        }

        pub(crate) fn z_velocity(&self) -> u8 {
            byte(self.ram, SPRITE_Z_VELOCITY + self.slot)
        }

        pub(crate) fn x_subpixel(&self) -> u8 {
            byte(self.ram, SPRITE_X_SUBPIXEL + self.slot)
        }

        pub(crate) fn y_subpixel(&self) -> u8 {
            byte(self.ram, SPRITE_Y_SUBPIXEL + self.slot)
        }

        pub(crate) fn z(&self) -> u8 {
            byte(self.ram, SPRITE_Z + self.slot)
        }

        pub(crate) fn z_subpixel(&self) -> u8 {
            byte(self.ram, SPRITE_Z_SUBPIXEL + self.slot)
        }

        pub(crate) fn ai_state(&self) -> u8 {
            byte(self.ram, SPRITE_AI_STATE + self.slot)
        }

        pub(crate) fn delay_main(&self) -> u8 {
            byte(self.ram, SPRITE_DELAY_MAIN + self.slot)
        }

        pub(crate) fn health(&self) -> u8 {
            byte(self.ram, SPRITE_HEALTH + self.slot)
        }

        pub(crate) fn hit_timer(&self) -> u8 {
            byte(self.ram, SPRITE_HIT_TIMER + self.slot)
        }
    }

    pub(crate) struct SpriteSlotViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> SpriteSlotViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_sprite_type(&mut self, value: u8) {
            self.ram[SPRITE_TYPE + self.slot] = value;
        }

        pub(crate) fn set_state(&mut self, value: u8) {
            self.ram[SPRITE_STATE + self.slot] = value;
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_position(
                self.ram,
                SPRITE_X_LO + self.slot,
                SPRITE_X_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_position(
                self.ram,
                SPRITE_Y_LO + self.slot,
                SPRITE_Y_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_x_velocity(&mut self, value: u8) {
            self.ram[SPRITE_X_VELOCITY + self.slot] = value;
        }

        pub(crate) fn set_y_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] = value;
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            self.ram[SPRITE_X_VELOCITY + self.slot]
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            self.ram[SPRITE_Y_VELOCITY + self.slot]
        }

        pub(crate) fn add_x_velocity(&mut self, value: u8) {
            self.ram[SPRITE_X_VELOCITY + self.slot] =
                self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_add(value);
        }

        pub(crate) fn add_y_velocity(&mut self, value: u8) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] =
                self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_add(value);
        }

        pub(crate) fn negate_x_velocity(&mut self) {
            self.ram[SPRITE_X_VELOCITY + self.slot] =
                self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_neg();
        }

        pub(crate) fn negate_y_velocity(&mut self) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] =
                self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_neg();
        }

        pub(crate) fn halve_x_velocity(&mut self) {
            self.ram[SPRITE_X_VELOCITY + self.slot] =
                ((self.ram[SPRITE_X_VELOCITY + self.slot] as i8) >> 1) as u8;
        }

        pub(crate) fn halve_y_velocity(&mut self) {
            self.ram[SPRITE_Y_VELOCITY + self.slot] =
                ((self.ram[SPRITE_Y_VELOCITY + self.slot] as i8) >> 1) as u8;
        }

        pub(crate) fn set_z(&mut self, value: u8) {
            self.ram[SPRITE_Z + self.slot] = value;
        }

        pub(crate) fn move_x(&mut self) {
            if self.ram[SPRITE_X_VELOCITY + self.slot] == 0 {
                return;
            }
            move_axis24(
                self.ram,
                SPRITE_X_SUBPIXEL + self.slot,
                SPRITE_X_LO + self.slot,
                SPRITE_X_HI + self.slot,
                SPRITE_X_VELOCITY + self.slot,
            );
        }

        pub(crate) fn move_y(&mut self) {
            if self.ram[SPRITE_Y_VELOCITY + self.slot] == 0 {
                return;
            }
            move_axis24(
                self.ram,
                SPRITE_Y_SUBPIXEL + self.slot,
                SPRITE_Y_LO + self.slot,
                SPRITE_Y_HI + self.slot,
                SPRITE_Y_VELOCITY + self.slot,
            );
        }

        pub(crate) fn move_z(&mut self) {
            move_axis16(
                self.ram,
                SPRITE_Z_SUBPIXEL + self.slot,
                SPRITE_Z + self.slot,
                SPRITE_Z_VELOCITY + self.slot,
            );
        }
    }

    pub(crate) struct AncillaSlotView<'a> {
        ram: &'a [u8],
        slot: usize,
    }

    impl<'a> AncillaSlotView<'a> {
        pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn slot(&self) -> u8 {
            self.slot as u8
        }

        pub(crate) fn ancilla_type(&self) -> u8 {
            byte(self.ram, ANCILLA_TYPE + self.slot)
        }

        pub(crate) fn is_active(&self) -> bool {
            self.ancilla_type() != 0
        }

        pub(crate) fn x(&self) -> u16 {
            packed_position(self.ram, ANCILLA_X_LO + self.slot, ANCILLA_X_HI + self.slot)
        }

        pub(crate) fn y(&self) -> u16 {
            packed_position(self.ram, ANCILLA_Y_LO + self.slot, ANCILLA_Y_HI + self.slot)
        }

        pub(crate) fn x_velocity(&self) -> u8 {
            byte(self.ram, ANCILLA_X_VELOCITY + self.slot)
        }

        pub(crate) fn y_velocity(&self) -> u8 {
            byte(self.ram, ANCILLA_Y_VELOCITY + self.slot)
        }

        pub(crate) fn z_velocity(&self) -> u8 {
            byte(self.ram, ANCILLA_Z_VELOCITY + self.slot)
        }

        pub(crate) fn x_subpixel(&self) -> u8 {
            byte(self.ram, ANCILLA_X_SUBPIXEL + self.slot)
        }

        pub(crate) fn y_subpixel(&self) -> u8 {
            byte(self.ram, ANCILLA_Y_SUBPIXEL + self.slot)
        }

        pub(crate) fn z(&self) -> u8 {
            byte(self.ram, ANCILLA_Z + self.slot)
        }

        pub(crate) fn z_subpixel(&self) -> u8 {
            byte(self.ram, ANCILLA_Z_SUBPIXEL_PLAYER + self.slot)
        }

        pub(crate) fn item_to_link(&self) -> u8 {
            byte(self.ram, ANCILLA_ITEM_TO_LINK + self.slot)
        }

        pub(crate) fn timer(&self) -> u8 {
            byte(self.ram, ANCILLA_TIMER + self.slot)
        }

        pub(crate) fn direction(&self) -> u8 {
            byte(self.ram, ANCILLA_DIRECTION + self.slot)
        }
    }

    pub(crate) struct AncillaSlotViewMut<'a> {
        ram: &'a mut [u8],
        slot: usize,
    }

    impl<'a> AncillaSlotViewMut<'a> {
        pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
            Self { ram, slot }
        }

        pub(crate) fn set_ancilla_type(&mut self, value: u8) {
            self.ram[ANCILLA_TYPE + self.slot] = value;
        }

        pub(crate) fn set_x(&mut self, value: u16) {
            write_position(
                self.ram,
                ANCILLA_X_LO + self.slot,
                ANCILLA_X_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_y(&mut self, value: u16) {
            write_position(
                self.ram,
                ANCILLA_Y_LO + self.slot,
                ANCILLA_Y_HI + self.slot,
                value,
            );
        }

        pub(crate) fn set_x_velocity(&mut self, value: u8) {
            self.ram[ANCILLA_X_VELOCITY + self.slot] = value;
        }

        pub(crate) fn set_y_velocity(&mut self, value: u8) {
            self.ram[ANCILLA_Y_VELOCITY + self.slot] = value;
        }

        pub(crate) fn set_z_velocity(&mut self, value: u8) {
            self.ram[ANCILLA_Z_VELOCITY + self.slot] = value;
        }

        pub(crate) fn set_z(&mut self, value: u8) {
            self.ram[ANCILLA_Z + self.slot] = value;
        }

        pub(crate) fn move_x(&mut self) {
            move_axis24(
                self.ram,
                ANCILLA_X_SUBPIXEL + self.slot,
                ANCILLA_X_LO + self.slot,
                ANCILLA_X_HI + self.slot,
                ANCILLA_X_VELOCITY + self.slot,
            );
        }

        pub(crate) fn move_y(&mut self) {
            move_axis24(
                self.ram,
                ANCILLA_Y_SUBPIXEL + self.slot,
                ANCILLA_Y_LO + self.slot,
                ANCILLA_Y_HI + self.slot,
                ANCILLA_Y_VELOCITY + self.slot,
            );
        }

        pub(crate) fn move_z(&mut self) {
            move_axis16(
                self.ram,
                ANCILLA_Z_SUBPIXEL_PLAYER + self.slot,
                ANCILLA_Z + self.slot,
                ANCILLA_Z_VELOCITY + self.slot,
            );
        }

        pub(crate) fn set_item_to_link(&mut self, value: u8) {
            self.ram[ANCILLA_ITEM_TO_LINK + self.slot] = value;
        }

        pub(crate) fn set_timer(&mut self, value: u8) {
            self.ram[ANCILLA_TIMER + self.slot] = value;
        }

        pub(crate) fn set_direction(&mut self, value: u8) {
            self.ram[ANCILLA_DIRECTION + self.slot] = value;
        }
    }

    fn byte(ram: &[u8], offset: usize) -> u8 {
        ram.get(offset).copied().unwrap_or(0)
    }

    fn word(ram: &[u8], offset: usize) -> u16 {
        if offset + 1 < ram.len() {
            read_le_u16(ram, offset)
        } else {
            0
        }
    }

    fn packed_position(ram: &[u8], low_offset: usize, high_offset: usize) -> u16 {
        u16::from(byte(ram, low_offset)) | (u16::from(byte(ram, high_offset)) << 8)
    }

    fn write_position(ram: &mut [u8], low_offset: usize, high_offset: usize, value: u16) {
        ram[low_offset] = value as u8;
        ram[high_offset] = (value >> 8) as u8;
    }

    fn move_axis24(
        ram: &mut [u8],
        subpixel_offset: usize,
        low_offset: usize,
        high_offset: usize,
        velocity_offset: usize,
    ) {
        let pos = u32::from(ram[subpixel_offset])
            | (u32::from(ram[low_offset]) << 8)
            | (u32::from(ram[high_offset]) << 16);
        let delta = ((ram[velocity_offset] as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        ram[subpixel_offset] = moved as u8;
        ram[low_offset] = (moved >> 8) as u8;
        ram[high_offset] = (moved >> 16) as u8;
    }

    fn move_axis16(ram: &mut [u8], subpixel_offset: usize, offset: usize, velocity_offset: usize) {
        let pos = (u16::from(ram[offset]) << 8) | u16::from(ram[subpixel_offset]);
        let delta = ((ram[velocity_offset] as i8 as i32) << 4) as u16;
        let moved = pos.wrapping_add(delta);
        ram[subpixel_offset] = moved as u8;
        ram[offset] = (moved >> 8) as u8;
    }
}

pub(crate) mod player {
    pub(crate) const LAYER_COLLISION_FLAGS: usize = 0x322;
    pub(crate) const FAINT_ANIMATION_ACTIVE: usize = 0x36b;

    pub(crate) const LAYER_COLLISION_BG1: u8 = 0x01;
    pub(crate) const LAYER_COLLISION_BG2: u8 = 0x02;
    pub(crate) const LAYER_COLLISION_BOTH: u8 = LAYER_COLLISION_BG1 | LAYER_COLLISION_BG2;
}

pub(crate) mod nmi {
    pub(crate) const TILEMAP_UPLOAD_BUFFER: usize = 0x1000;
    pub(crate) const VRAM_UPLOAD_DATA: usize = 0x1002;
    pub(crate) const VRAM_UPLOAD_OFFSET: usize = 0x1000;
    pub(crate) const VRAM_UPLOAD_TILE_BUF: usize = 0x1100;
    pub(crate) const BG_CHAR_BUFFER: usize = 0x10000;
    pub(crate) const BG_CHAR_BUFFER_1: usize = 0x10800;
    pub(crate) const BG_CHAR_HALF_BUFFER: usize = 0x11000;
    pub(crate) const BG1_WALL_TOP_BUFFER: usize = 0x0c880;
    pub(crate) const BG1_WALL_BOTTOM_BUFFER: usize = 0x0c8c0;
    pub(crate) const GAME_OVER_TEXT_BUFFER: usize = 0x2000;
    pub(crate) const GAME_OVER_TEXT_TAIL_BUFFER: usize = 0x3400;
    pub(crate) const STRIPE_BUFFER_021B: usize = 0x021b;
    pub(crate) const ARBITRARY_TILEMAP_DST_BUFFER: usize = 0x14000;
}

pub(crate) mod messaging {
    pub(crate) const MODULE: usize = 0x1cd8;
    pub(crate) const DIALOGUE_MESSAGE_INDEX: usize = 0x1cf0;
    pub(crate) const CHOICE_IN_MULTISELECT_BOX: usize = 0x1ce8;
    pub(crate) const CHOICE_IN_MULTISELECT_BOX_BAK: usize = 0x1cf4;
    pub(crate) const TEXT_MSGBOX_TOPLEFT_COPY: usize = 0x1cd0;
    pub(crate) const TEXT_MSGBOX_TOPLEFT: usize = 0x1cd2;
    pub(crate) const TEXT_RENDER_STATE: usize = 0x1cd4;
    pub(crate) const VWF_LINE_SPEED_CUR: usize = 0x1cd5;
    pub(crate) const VWF_LINE_SPEED: usize = 0x1cd6;
    pub(crate) const TEXT_INCREMENTAL_STATE: usize = 0x1cd7;
    pub(crate) const DIALOGUE_MSG_READ_POS: usize = 0x1cd9;
    pub(crate) const DIALOGUE_TEXT_COLOR: usize = 0x1cdc;
    pub(crate) const DIALOGUE_MSG_SRC_OFFS: usize = 0x1cdd;
    pub(crate) const TEXT_WAIT_COUNTDOWN: usize = 0x1ce0;
    pub(crate) const TEXT_TILEMAP_CUR: usize = 0x1ce2;
    pub(crate) const TEXT_WAIT_COUNTDOWN2: usize = 0x1ce9;
    pub(crate) const DIALOGUE_SCROLL_SPEED: usize = 0x1cea;
    pub(crate) const MESSAGE_DMA_DST_ADDR: usize = 0x219;
    pub(crate) const MESSAGE_DMA_TILE_BASE: usize = 0x21d;
    pub(crate) const MESSAGE_DMA_TILE_LIMIT: usize = 0x21f;
    pub(crate) const MESSAGE_DMA_TILE_SENTINEL: usize = 0x221;
    pub(crate) const VWF_FLAG_NEXT_LINE: usize = 0x720;
    pub(crate) const VWF_CURLINE: usize = 0x722;
    pub(crate) const VWF_VAR1: usize = 0x724;
    pub(crate) const VWF_LINE_PTR: usize = 0x726;
    pub(crate) const VWF_ARR: usize = 0x0c230;
    pub(crate) const TEXT_BUFFER: usize = 0x11200;
}
