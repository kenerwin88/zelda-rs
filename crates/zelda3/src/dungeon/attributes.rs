//! Dungeon collision attributes and crystal-peg attribute updates.

use super::*;
use crate::tile_definition::{DungeonRole, NativeTile, TilePair};

impl ZeldaState {

    /// ROM: `Dungeon_LoadAttributeTable`
    pub(in crate::zelda_rtl) fn dungeon_load_attribute_table(&mut self) {
        self.dungeon_room_load_mut()
            .set_draw_width_indicator_word(0);
        self.dungeon_room_load_mut()
            .set_draw_height_indicator_word(0);
        self.dungeon_load_basic_attribute_full(0x1000);
        self.dungeon_load_object_attribute();
        self.dungeon_load_door_attribute();
        if self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            != 0
        {
            self.dungeon_flip_crystal_peg_attribute();
        }
        self.set_overworld_map_state(0);
    }

    /// Transfer the long selectable-loader peg flip to the live source
    /// cursor, retaining the exact translated caller which follows the shared
    /// helper's return.
    pub(super) fn suspend_dungeon_selectable_peg_attribute_flip(
        &mut self,
        caller: DungeonPegAttributeFlipCaller,
    ) -> bool {
        if self.overworld_map_state() != 4
            || self
                .game_state
                .dungeon
                .environment
                .orange_blue_barrier_state()
                == 0
        {
            return false;
        }
        let Some(progress) = self.take_original_timing_dungeon_peg_attribute_flip_progress() else {
            return false;
        };
        // Dungeon_LoadAttribute_Selectable advances the selector before it
        // enters Dungeon_FlipCrystalPegAttribute.
        self.set_overworld_map_state(5);
        self.dungeon_flip_crystal_peg_attribute_through_progress(progress);
        assert!(
            self.dungeon_peg_attribute_flip_pending
                .replace(DungeonPegAttributeFlipContinuation { caller, progress })
                .is_none(),
            "peg-attribute flip continuation was already pending",
        );
        match progress.boundary {
            OriginalTimingBoundary::NmiAccepted => self
                .game_execution_scheduler
                .schedule_after_current_trailing_nmi(
                    GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
                ),
            OriginalTimingBoundary::HostReturn => self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
                1,
            ),
        }
        true
    }

    /// ROM: `Dungeon_LoadAttribute_Selectable`
    pub(super) fn dungeon_load_attribute_selectable(&mut self) {
        // Cycle ledger: Dungeon_LoadAttribute_Selectable $01:B8B4, entered
        // with M=8 X=8. $01B8B4-$01B8BE: LDA $0200, ASL, TAX, JSR ($B8A8,X)
        // (62, priced in the block), SEP #$30, RTL: 188 for every state.
        let _scope = crate::cycle_ledger::routine(0x01_b8b4);
        crate::cycle_ledger::charge(188);
        match self.overworld_map_state() {
            0 => {
                // Table entry 0 is Dungeon_LoadBasicAttribute $01:B8E3:
                // $01B8E3-$01B8EA REP #$20, INC $0200, STZ $B2, STZ $B4 (148),
                // then it falls through into Dungeon_LoadBasicAttr_partial at
                // $01B8EC (188) and the shared body, all under this frame.
                let _entry = crate::cycle_ledger::routine(0x01_b8e3);
                crate::cycle_ledger::charge(148 + 188);
                self.set_overworld_map_state(1);
                self.dungeon_room_load_mut()
                    .set_draw_width_indicator_word(0);
                self.dungeon_room_load_mut()
                    .set_draw_height_indicator_word(0);
                self.dungeon_load_basic_attribute_body(0x40);
            }
            // Table entry 1 is Dungeon_LoadBasicAttr_partial $01:B8EC, which
            // scopes itself.
            1 => self.dungeon_load_basic_attribute_full(0x40),
            2 => self.dungeon_load_object_attribute(),
            3 => self.dungeon_load_door_attribute(),
            4 => {
                // Table entry 4 is Dungeon_InitializePegAttribute $01:C21C:
                // INC $0200, LDA $7EC172, BEQ (102); JSL the flip (62, the
                // callee charges itself) or BEQ taken (+6); RTS (42).
                let _entry = crate::cycle_ledger::routine(0x01_c21c);
                crate::cycle_ledger::charge(102);
                self.set_overworld_map_state(5);
                if self
                    .game_state
                    .dungeon
                    .environment
                    .orange_blue_barrier_state()
                    != 0
                {
                    crate::cycle_ledger::charge(62);
                    self.dungeon_flip_crystal_peg_attribute();
                } else {
                    crate::cycle_ledger::charge(6);
                }
                crate::cycle_ledger::charge(42);
            }
            // Table entry 5 is the bare RTS at $01:B966.
            5 => crate::cycle_ledger::charge_routine(0x01_b966, 42),
            // C Dungeon_LoadAttribute_Selectable asserts outside states 0..=5.
            _ => panic!(
                "Dungeon_LoadAttribute_Selectable overworld_map_state {}",
                self.overworld_map_state()
            ),
        }
    }

    /// ROM: `Dungeon_LoadBasicAttribute_full`
    fn dungeon_load_basic_attribute_full(&mut self, loops: usize) {
        // Cycle ledger: the 0x40-loop form is Dungeon_LoadBasicAttr_partial
        // $01:B8EC (M=8 X=8 at entry): REP #$20, LDA #$0040, STA $00 (78) and
        // then the shared entry at $01:B8F3 (Dungeon_LoadBasicAttribute_full,
        // reached by JSR from Dungeon_LoadAttributeTable with $00 already set):
        // PHB, LDX #$7E, PHX, PLB, REP #$10 (110).
        let (address, entry) = if loops == 0x40 {
            (0x01_b8ec, 188)
        } else {
            (0x01_b8f3, 110)
        };
        let _scope = crate::cycle_ledger::routine(address);
        crate::cycle_ledger::charge(entry);
        self.dungeon_load_basic_attribute_body(loops);
    }

    /// The loop of Dungeon_LoadBasicAttribute_full from $01:B8FA, unscoped so
    /// that the three entries ($01B8E3, $01B8EC, $01B8F3) can each own it.
    fn dungeon_load_basic_attribute_body(&mut self, loops: usize) {
        for loop_index in 0..loops {
            // $01B8FA-$01B918: both tile words, the two attribute lookups,
            // SEP #$20, CMP #$10 with BCC priced not taken.
            crate::cycle_ledger::charge(374);
            let i = self
                .game_state
                .dungeon
                .room_load
                .draw_width_indicator_word() as usize
                / 2;
            let tile0 = self.game_state.dungeon.room_tilemaps.attr_source_tile(i);
            let tile1 = self
                .game_state
                .dungeon
                .room_tilemaps
                .attr_source_tile(i + 1);
            // The flip-merge branches on the raw attribute bytes: the ROM tests
            // tile0's attribute at $01B916/$01B91A and tile1's at $01B92E/$01B932.
            let attr0 = self.dungeon_tile_definition(tile0 as usize).cartridge_attribute();
            let attr1 = self.dungeon_tile_definition(tile1 as usize).cartridge_attribute();
            if attr0 < 0x10 {
                // BCC $01B92A taken.
                crate::cycle_ledger::charge(6);
            } else {
                // $01B91A-$01B91C: CMP #$1C, BCS priced not taken.
                crate::cycle_ledger::charge(32);
                if attr0 >= 0x1c {
                    // BCS $01B92A taken.
                    crate::cycle_ledger::charge(6);
                } else {
                    // $01B91E-$01B927: merge tile0's flip bits into the attribute.
                    crate::cycle_ledger::charge(136);
                }
            }
            // $01B92A-$01B930: STA $02, LDA $04, CMP #$10, BCC priced not taken.
            crate::cycle_ledger::charge(80);
            if attr1 < 0x10 {
                // BCC $01B941 taken.
                crate::cycle_ledger::charge(6);
            } else {
                // $01B932-$01B934: CMP #$1C, BCS priced not taken.
                crate::cycle_ledger::charge(32);
                if attr1 >= 0x1c {
                    // BCS $01B941 taken.
                    crate::cycle_ledger::charge(6);
                } else {
                    // $01B936-$01B93F: merge tile1's flip bits into the attribute.
                    crate::cycle_ledger::charge(122);
                }
            }
            // $01B941-$01B959: pack both attributes, store the word at
            // $7F2000+$B4, $B4 += 2, $B2 += 4, DEC $00, BNE (priced not taken).
            crate::cycle_ledger::charge(364);
            if loop_index + 1 < loops {
                // BNE $01B8FA taken for every loop but the last.
                crate::cycle_ledger::charge(6);
            }
            let a0 = self.attribute_for_bg_tile(tile0);
            let a1 = self.attribute_for_bg_tile(tile1);
            let j = self
                .game_state
                .dungeon
                .room_load
                .draw_height_indicator_word() as usize;
            self.dungeon_bg2_attributes_mut().set_bg2_tile(j, a0);
            self.dungeon_bg2_attributes_mut().set_bg2_tile(j + 1, a1);
            self.dungeon_room_load_mut()
                .set_draw_height_indicator_word((j as u16).wrapping_add(2));
            let width = self
                .game_state
                .dungeon
                .room_load
                .draw_width_indicator_word()
                .wrapping_add(4);
            self.dungeon_room_load_mut()
                .set_draw_width_indicator_word(width);
        }
        // $01B95B-$01B960: LDA $B4, CMP #$2000, BNE priced not taken.
        crate::cycle_ledger::charge(72);
        if self
            .game_state
            .dungeon
            .room_load
            .draw_height_indicator_word()
            == 0x2000
        {
            // $01B962: INC $0200.
            crate::cycle_ledger::charge(62);
            self.increment_overworld_map_state();
        } else {
            // BNE $01B965 taken.
            crate::cycle_ledger::charge(6);
        }
        // $01B965-$01B966: PLB / RTS.
        crate::cycle_ledger::charge(70);
    }

    fn attribute_for_bg_tile(&self, tile: u16) -> NativeTile {
        self.dungeon_tile_definition(tile as usize)
            .with_cartridge_orientation((tile >> 14) as u8)
    }

    /// ROM: `Dungeon_LoadObjectAttribute`
    fn dungeon_load_object_attribute(&mut self) {
        let mut i = 0usize;
        while i != self.game_state.dungeon.room_parser.star_switch_count_x2() as usize {
            let j = self
                .game_state
                .dungeon
                .room_parser
                .star_switch_tilemap_pos(i) as usize;
            self.write_attr2(j + xy(0, 0), TilePair::repeated(NativeTile::STAR_SWITCH));
            self.write_attr2(j + xy(0, 1), TilePair::repeated(NativeTile::STAR_SWITCH));
            i += 2;
        }

        i = 0;
        let mut attr = TilePair::stair_landing(0);
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpNorth) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(j + xy(1, 2), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(
                j + xy(1, 0),
                TilePair::repeated(NativeTile::SPIRAL_STAIR_HEAD),
            );
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallUpNorthSpiral) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                j + xy(1, 0),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD),
            );
            self.write_attr2(
                j + xy(1, 2),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD),
            );
            self.write_attr2(
                j + xy(1, 3),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD),
            );
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallUpNorthSpiralBg1) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                j + xy(1, 0),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD_2),
            );
            self.write_attr2(
                j + xy(1, 2),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD_2),
            );
            self.write_attr2(
                j + xy(1, 3),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD_2),
            );
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpNorthStraight)
                as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                j + xy(1, 0),
                TilePair::repeated(NativeTile::STRAIGHT_STAIR_UP_HEAD),
            );
            self.write_attr2(j + xy(1, 2), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(j + xy(1, 3), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(j + xy(1, 1), attr);
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomUpSouthStraight)
                as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(j + xy(1, 0), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(j + xy(1, 1), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(
                j + xy(1, 3),
                TilePair::repeated(NativeTile::STRAIGHT_STAIR_DOWN_HEAD),
            );
            i += 2;
            attr = attr.next_identity();
        }
        attr = attr.descending_stair_sequence();
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomSouthDown) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(
                j + xy(1, 3),
                TilePair::repeated(NativeTile::SPIRAL_STAIR_HEAD),
            );
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallDownNorthSpiral) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                j + xy(1, 0),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD),
            );
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(
                j + xy(1, 2),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD),
            );
            self.write_attr2(
                j + xy(1, 3),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD),
            );
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::WallDownNorthSpiralBg1) as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                j + xy(1, 0),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD_2),
            );
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(
                j + xy(1, 2),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD_2),
            );
            self.write_attr2(
                j + xy(1, 3),
                TilePair::repeated(NativeTile::WALL_SPIRAL_STAIR_HEAD_2),
            );
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomDownNorthStraight)
                as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(
                j + xy(1, 0),
                TilePair::repeated(NativeTile::STRAIGHT_STAIR_UP_HEAD),
            );
            self.write_attr2(j + xy(1, 1), attr);
            self.write_attr2(j + xy(1, 2), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(j + xy(1, 3), TilePair::repeated(NativeTile::GROUND));
            i += 2;
            attr = attr.next_identity();
        }
        while i
            != self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InterRoomDownSouthStraight)
                as usize
        {
            let j = self
                .game_state
                .dungeon
                .stair_lists
                .inter_staircase_pos(i >> 1) as usize;
            self.write_attr2(j + xy(1, 0), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(j + xy(1, 1), TilePair::repeated(NativeTile::GROUND));
            self.write_attr2(j + xy(1, 2), attr);
            self.write_attr2(
                j + xy(1, 3),
                TilePair::repeated(NativeTile::STRAIGHT_STAIR_DOWN_HEAD),
            );
            i += 2;
            attr = attr.next_identity();
        }

        i = 0;
        let mut stair_type = 0u16;
        let mut iend = self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::InRoomUpNorth) as usize;
        attr = TilePair::from_cartridge(0x1f1f);
        if iend == 0 {
            stair_type = 1;
            attr = TilePair::from_cartridge(0x1e1e);
            iend = self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::InRoomSouthDown) as usize;
            if iend == 0 {
                stair_type = 2;
                attr = TilePair::repeated(NativeTile::IN_ROOM_STAIR_PSEUDO_UP_NORTH);
                iend = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::InterPseudoUpNorth)
                    as usize;
            }
        }
        if iend != 0 {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(stair_type);
            while i != iend {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, i as u16)
                    as usize;
                self.write_attr2(
                    j + xy(0, 0),
                    TilePair([NativeTile::SOLID_WALL, NativeTile::GROUND]),
                );
                self.write_attr1(
                    j + xy(0, 3),
                    TilePair([NativeTile::SOLID_WALL, NativeTile::GROUND]),
                );
                self.write_attr2(
                    j + xy(2, 0),
                    TilePair([NativeTile::GROUND, NativeTile::SOLID_WALL]),
                );
                self.write_attr1(
                    j + xy(2, 3),
                    TilePair([NativeTile::GROUND, NativeTile::SOLID_WALL]),
                );
                self.write_attr2(j + xy(0, 1), TilePair::from_cartridge(0x0001));
                self.write_attr1(j + xy(0, 2), TilePair::from_cartridge(0x0001));
                self.write_attr2(j + xy(2, 1), TilePair::from_cartridge(0x0100));
                self.write_attr1(j + xy(2, 2), TilePair::from_cartridge(0x0100));
                self.write_attr2(j + xy(1, 1), attr);
                self.write_attr1(j + xy(1, 1), attr);
                self.write_attr2(j + xy(1, 2), attr);
                self.write_attr1(j + xy(1, 2), attr);
                i += 2;
            }
        }
        if i != self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::WaterSideStepSwitch) as usize
        {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(2);
            while i
                != self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::WaterSideStepSwitch)
                    as usize
            {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, i as u16)
                    as usize;
                self.write_attr2(
                    j + xy(0, 0),
                    TilePair([NativeTile::LAYER_WALL, NativeTile::LAYER_LANDING]),
                );
                self.write_attr1(
                    j + xy(0, 0),
                    TilePair([NativeTile::LAYER_WALL, NativeTile::LAYER_LANDING]),
                );
                self.write_attr2(
                    j + xy(2, 0),
                    TilePair([NativeTile::LAYER_LANDING, NativeTile::LAYER_WALL]),
                );
                self.write_attr1(
                    j + xy(2, 0),
                    TilePair([NativeTile::LAYER_LANDING, NativeTile::LAYER_WALL]),
                );
                self.write_attr2(j + xy(0, 1), TilePair::from_cartridge(0x0803));
                self.write_attr2(j + xy(2, 1), TilePair::from_cartridge(0x0308));
                i += 2;
            }
        }
        i = 0;
        if i != self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::InRoomUpNorthWater) as usize
        {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(2);
            while i
                != self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::InRoomUpNorthWater)
                    as usize
            {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, i as u16)
                    as usize;
                self.write_attr2(
                    j + xy(0, 0),
                    TilePair([NativeTile::LAYER_WALL, NativeTile::GROUND]),
                );
                self.write_attr2(
                    j + xy(2, 0),
                    TilePair([NativeTile::GROUND, NativeTile::LAYER_WALL]),
                );
                self.write_attr1(
                    j + xy(0, 0),
                    TilePair([NativeTile::LAYER_WALL, NativeTile::LAYER_LANDING]),
                );
                self.write_attr1(
                    j + xy(2, 0),
                    TilePair([NativeTile::LAYER_LANDING, NativeTile::LAYER_WALL]),
                );
                self.write_attr2(j + xy(0, 1), TilePair::repeated(NativeTile::DEEP_WATER));
                self.write_attr2(j + xy(2, 1), TilePair::repeated(NativeTile::DEEP_WATER));
                i += 2;
            }
        }
        if i != self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::ActivatedWaterLadders) as usize
        {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(2);
            while i
                != self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::ActivatedWaterLadders)
                    as usize
            {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::InRoomUpNorth, i as u16)
                    as usize;
                self.write_attr2(
                    j + xy(0, 0),
                    TilePair([NativeTile::LAYER_WALL, NativeTile::GROUND]),
                );
                self.write_attr2(
                    j + xy(2, 0),
                    TilePair([NativeTile::GROUND, NativeTile::LAYER_WALL]),
                );
                self.write_attr1(
                    j + xy(0, 0),
                    TilePair([NativeTile::LAYER_WALL, NativeTile::LAYER_LANDING]),
                );
                self.write_attr1(
                    j + xy(2, 0),
                    TilePair([NativeTile::LAYER_LANDING, NativeTile::LAYER_WALL]),
                );
                i += 2;
            }
        }

        let mut i = 0usize;
        let mut attr = TilePair::from_cartridge(0x7070);
        let misc_end = self.game_state.dungeon.object_tracking.misc_object_index() as usize;
        while i != misc_end {
            let record = self
                .game_state
                .dungeon
                .object_tracking
                .object_record(i >> 1);
            if !record.skips_room_attribute() {
                let j = (self
                    .game_state
                    .dungeon
                    .object_tracking
                    .object_tilemap_pos(i >> 1)
                    & 0x3fff)
                    >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
            }
            i += 2;
            attr = attr.next_identity();
        }

        if i != self.game_state.dungeon.torch.torch_index() as usize {
            attr = TilePair::from_cartridge(0xc0c0);
            while i != self.game_state.dungeon.torch.torch_index() as usize {
                let j = (self
                    .game_state
                    .dungeon
                    .object_tracking
                    .object_tilemap_pos(i >> 1)
                    & 0x3fff)
                    >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
                i += 2;
                attr = attr.next_torch();
            }
            self.dungeon_torch_mut().set_torch_index(0);
        }

        let mut attr = TilePair::from_cartridge(0x5858);
        let mut i = 0usize;
        let skip_big_key_locks = self.game_state.dungeon.room_items.num_chests_x2() != 0
            && self.hud_tags_suppress_big_key_locks();
        if self.game_state.dungeon.room_items.num_chests_x2() != 0 && !skip_big_key_locks {
            while i != self.game_state.dungeon.room_items.num_chests_x2() as usize {
                let k = self
                    .game_state
                    .dungeon
                    .room_items
                    .chest_location_for_offset_x2(i);
                if k != 0 {
                    let j = (k & 0x7fff) >> 1;
                    self.write_attr2(j as usize + xy(0, 0), attr);
                    self.write_attr2(j as usize + xy(0, 1), attr);
                    if k & 0x8000 != 0 {
                        self.dungeon_room_items_mut()
                            .set_chest_location_for_offset_x2(i, k & 0x7fff);
                        self.write_attr2(j as usize + xy(2, 1), attr);
                        self.write_attr2(j as usize + xy(0, 2), attr);
                        self.write_attr2(j as usize + xy(2, 2), attr);
                    }
                }
                i += 2;
                attr = attr.next_identity();
            }
        }

        if !skip_big_key_locks {
            while i != self.game_state.dungeon.room_items.num_big_key_locks_x2() as usize {
                let k = self
                    .game_state
                    .dungeon
                    .room_items
                    .chest_location_for_offset_x2(i);
                self.dungeon_room_items_mut()
                    .set_chest_location_for_offset_x2(i, k | 0x8000);
                let j = (k & 0x7fff) >> 1;
                self.write_attr2(j as usize + xy(0, 0), attr);
                self.write_attr2(j as usize + xy(0, 1), attr);
                i += 2;
                attr = attr.next_identity();
            }
        }

        i = 0;
        let mut stair_type = 0u16;
        let mut iend = self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::Stairs1) as usize;
        attr = TilePair::from_cartridge(0x3f3f);
        if iend == 0 {
            stair_type = 1;
            attr = TilePair::from_cartridge(0x3e3e);
            iend = self
                .game_state
                .dungeon
                .stair_lists
                .stair_list_count(DungeonStairList::Stairs2) as usize;
            if iend == 0 {
                stair_type = 2;
                attr = TilePair::from_cartridge(0x3d3d);
                iend = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::WetStairs) as usize;
            }
        }
        if iend != 0 {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(stair_type);
            while i != iend {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::Stairs1, i as u16)
                    as usize;
                self.write_attr1(
                    j + xy(0, 0),
                    TilePair([NativeTile::SOLID_WALL, NativeTile::GROUND]),
                );
                self.write_attr2(
                    j + xy(0, 3),
                    TilePair([NativeTile::SOLID_WALL, NativeTile::GROUND]),
                );
                self.write_attr1(j + xy(0, 1), TilePair::from_cartridge(0x0001));
                self.write_attr2(j + xy(0, 2), TilePair::from_cartridge(0x0001));
                self.write_attr1(
                    j + xy(2, 0),
                    TilePair([NativeTile::GROUND, NativeTile::SOLID_WALL]),
                );
                self.write_attr2(
                    j + xy(2, 3),
                    TilePair([NativeTile::GROUND, NativeTile::SOLID_WALL]),
                );
                self.write_attr1(j + xy(2, 1), TilePair::from_cartridge(0x0100));
                self.write_attr2(j + xy(2, 2), TilePair::from_cartridge(0x0100));
                self.write_attr1(j + xy(1, 1), attr);
                self.write_attr2(j + xy(1, 1), attr);
                self.write_attr1(j + xy(1, 2), attr);
                self.write_attr2(j + xy(1, 2), attr);
                i += 2;
            }
        }

        if self
            .game_state
            .dungeon
            .stair_lists
            .stair_list_count(DungeonStairList::InRoomUpSouthWater)
            != 0
        {
            self.dungeon_stair_movement_mut()
                .set_kind_of_in_room_staircase_word(2);
            i = 0;
            while i
                != self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_count(DungeonStairList::InRoomUpSouthWater)
                    as usize
            {
                let j = self
                    .game_state
                    .dungeon
                    .stair_lists
                    .stair_list_tilemap_pos(DungeonStairList::Stairs1, i as u16)
                    as usize;
                self.write_attr1(
                    j + xy(0, 3),
                    TilePair([NativeTile::LAYER_WALL, NativeTile::LAYER_LANDING]),
                );
                self.write_attr1(
                    j + xy(2, 3),
                    TilePair([NativeTile::LAYER_LANDING, NativeTile::LAYER_WALL]),
                );
                self.write_attr2(
                    j + xy(0, 3),
                    TilePair([NativeTile::LAYER_WALL, NativeTile::GROUND]),
                );
                self.write_attr2(
                    j + xy(2, 3),
                    TilePair([NativeTile::GROUND, NativeTile::LAYER_WALL]),
                );
                self.write_attr2(j + xy(0, 2), TilePair::repeated(NativeTile::DEEP_WATER));
                self.write_attr2(j + xy(2, 2), TilePair::repeated(NativeTile::DEEP_WATER));
                i += 2;
            }
        }
        self.increment_overworld_map_state();
    }

    fn hud_tags_suppress_big_key_locks(&self) -> bool {
        (0..2).any(|i| {
            let tag = self.game_state.dungeon.header.header_tag(i);
            tag == 0x27 || tag == 0x3c || tag == 0x3e || (0x29..0x33).contains(&tag)
        })
    }

    /// ROM: `Dungeon_LoadDoorAttribute`
    fn dungeon_load_door_attribute(&mut self) {
        for k in 0..16 {
            if self.game_state.dungeon.doors.door_tilemap_address(k) != 0 {
                self.dungeon_load_single_door_attribute(k);
            }
        }
        self.dungeon_load_single_door_tile_attribute();
        self.ChangeDoorToSwitch();
        self.increment_overworld_map_state();
    }

    /// ROM: `Dungeon_LoadToggleDoorAttr_OtherEntry`
    pub(super) fn dungeon_load_toggle_door_attr_other_entry(&mut self, door: i32) {
        self.dungeon_load_single_door_attribute(door as usize);
        self.dungeon_load_single_door_tile_attribute();
    }

    /// ROM: `Dungeon_LoadSingleDoorAttribute`
    pub(super) fn dungeon_load_single_door_attribute(&mut self, k: usize) {
        let t = self.game_state.dungeon.doors.door_type_and_slot(k) & 0xfe;
        if !matches!(
            t,
            DOOR_TYPE_REGULAR
                | DOOR_TYPE_ENTRANCE_DOOR
                | DOOR_TYPE_EXIT_TO_OW
                | DOOR_TYPE_ENTRANCE_LARGE
                | DOOR_TYPE_ENTRANCE_CAVE
                | DOOR_TYPE_ENTRANCE_LARGE2
                | DOOR_TYPE_ENTRANCE_CAVE2
                | DOOR_TYPE_4
                | DOOR_TYPE_REGULAR2
                | DOOR_TYPE_WATERFALL_TUNNEL
        ) {
            if t == DOOR_TYPE_LG_EXPLOSION {
                return;
            }
            if t >= DOOR_TYPE_REGULAR_DOOR33 {
                if t != DOOR_TYPE_REGULAR_DOOR33
                    && t != DOOR_TYPE_WARP_ROOM_DOOR
                    && self
                        .game_state
                        .dungeon
                        .doors
                        .opened_doors_including_adjacent()
                        & upper_bitmask(k)
                        == 0
                {
                    let j = self.game_state.dungeon.doors.door_tilemap_address(k) >> 1;
                    let attr = TilePair::closed_door(k);
                    self.write_attr2(j as usize + xy(1, 1), attr);
                    self.write_attr2(j as usize + xy(1, 2), attr);
                    return;
                }
            } else {
                let i = if t == DOOR_TYPE_SHUTTERS_TWO_WAY || t == DOOR_TYPE_SHUTTER {
                    k
                } else {
                    k & 7
                };
                if self
                    .game_state
                    .dungeon
                    .doors
                    .opened_doors_including_adjacent()
                    & upper_bitmask(i)
                    == 0
                {
                    let j = self.game_state.dungeon.doors.door_tilemap_address(k) >> 1;
                    let attr = TilePair::closed_door(k);
                    self.write_attr2(j as usize + xy(1, 1), attr);
                    self.write_attr2(j as usize + xy(1, 2), attr);
                    return;
                }
            }
        }

        if (DOOR_TYPE_STAIR_MASK_LOCKED0..=DOOR_TYPE_STAIR_MASK_LOCKED3).contains(&t) {
            return;
        }
        let mut attr = DUNGEON_LOAD_SINGLE_DOOR_ATTRIBUTE_TILE_ATTRS_BY_DOOR
            .get(t as usize >> 1)
            .copied()
            .unwrap_or(TilePair::from_cartridge(0x8080));
        let dir = self.game_state.dungeon.doors.door_direction(k) & 3;
        let address = self.game_state.dungeon.doors.door_tilemap_address(k);
        let beta = matches!(
            t,
            DOOR_TYPE_ENTRANCE_LARGE2
                | DOOR_TYPE_ENTRANCE_CAVE2
                | DOOR_TYPE_4
                | DOOR_TYPE_REGULAR2
                | DOOR_TYPE_WATERFALL_TUNNEL
                | DOOR_TYPE_REGULAR_DOOR33
                | DOOR_TYPE_WARP_ROOM_DOOR
        ) || (t >= DOOR_TYPE_REGULAR_DOOR33
            && self
                .game_state
                .dungeon
                .doors
                .opened_doors_including_adjacent()
                & upper_bitmask(k)
                != 0);

        if !beta {
            if dir == 0 {
                if self.door_address_is_exit(address) {
                    attr = TilePair::from_cartridge(0x8e8e);
                }
                let j = ((address >> 1) & !0x07c0) as usize;
                for y in 0..=6 {
                    self.write_attr2(j + xy(1, y), attr);
                }
                self.write_attr2(j + xy(1, 7), TilePair::repeated(NativeTile::GROUND));
            } else if dir == 1 {
                if t == DOOR_TYPE_ENTRANCE_LARGE
                    || t == DOOR_TYPE_ENTRANCE_CAVE
                    || self.door_address_is_exit(address)
                {
                    attr = TilePair::from_cartridge(0x8e8e);
                }
                let j = (address >> 1) as usize;
                for y in 1..=5 {
                    self.write_attr2(j + xy(1, y), attr);
                }
            } else if dir == 2 {
                let j = ((address >> 1) & !0x001f) as usize;
                self.write_attr2(j + xy(0, 1), attr.next_identity());
                self.write_attr2(j + xy(2, 1), attr.next_identity());
                self.write_attr2(j + xy(0, 2), attr.next_identity());
                self.write_attr2(j + xy(2, 2), attr.next_identity());
                self.write_attr2(j + xy(4, 1), attr.next_identity().with_ground_on_right());
                self.write_attr2(j + xy(4, 2), attr.next_identity().with_ground_on_right());
            } else {
                let j = (address >> 1) as usize;
                self.write_attr2(j + xy(2, 1), attr.next_identity());
                self.write_attr2(j + xy(4, 1), attr.next_identity());
                self.write_attr2(j + xy(2, 2), attr.next_identity());
                self.write_attr2(j + xy(4, 2), attr.next_identity());
                self.write_attr2(j + xy(0, 1), attr.next_identity().with_ground_on_left());
                self.write_attr2(j + xy(0, 2), attr.next_identity().with_ground_on_left());
            }
            return;
        }

        if dir == 0 {
            let j = ((address >> 1) & !0x07c0) as usize;
            for y in 0..=9 {
                self.write_attr2(j + xy(1, y), attr);
            }
        } else if dir == 1 {
            if t == DOOR_TYPE_ENTRANCE_LARGE2
                || t == DOOR_TYPE_ENTRANCE_CAVE2
                || t == DOOR_TYPE_4
                || self.door_address_is_exit(address & 0x1fff)
            {
                attr = TilePair::from_cartridge(0x8e8e);
            }
            let j = (address >> 1) as usize;
            for y in 1..=8 {
                self.write_attr2(j + xy(1, y), attr);
            }
        } else if dir == 2 {
            let j = ((address >> 1) & !0x001f) as usize;
            for y in 1..=2 {
                for x in [0, 2, 4, 6] {
                    self.write_attr2(j + xy(x, y), attr.next_identity());
                }
            }
        } else {
            let j = (address >> 1).wrapping_add(1) as usize;
            for y in 1..=2 {
                for x in [0, 2, 4, 6] {
                    self.write_attr2(j + xy(x, y), attr.next_identity());
                }
            }
        }
    }

    fn door_address_is_exit(&self, address: u16) -> bool {
        self.game_state
            .dungeon
            .door_setup
            .has_exit_door_address(address)
    }

    /// ROM: `Door_LoadBlastWallAttr`
    pub(super) fn door_load_blast_wall_attr(&mut self, k: usize) {
        let mut j = (self.game_state.dungeon.doors.door_tilemap_address(k) >> 1) as usize;
        if self.game_state.dungeon.doors.door_direction(k) & 2 == 0 {
            for _ in 0..12 {
                self.write_attr2(j + xy(0, 0), TilePair::from_cartridge(0x0102));
                for i in (2..20).step_by(2) {
                    self.write_attr2(j + xy(i, 0), TilePair::repeated(NativeTile::GROUND));
                }
                self.write_attr2(j + xy(20, 0), TilePair::from_cartridge(0x0201));
                j += xy(0, 1);
            }
        } else {
            for _ in 0..5 {
                self.write_attr2(j + xy(0, 0), TilePair::from_cartridge(0x0101));
                self.write_attr2(j + xy(0, 21), TilePair::from_cartridge(0x0101));
                self.write_attr2(j + xy(0, 1), TilePair::repeated(NativeTile::SOLID_WALL));
                self.write_attr2(j + xy(0, 20), TilePair::repeated(NativeTile::SOLID_WALL));
                for i in 2..20 {
                    self.write_attr2(j + xy(0, i), TilePair::repeated(NativeTile::GROUND));
                }
                j += xy(2, 0);
            }
        }
    }

    fn ChangeDoorToSwitch(&self) {
        assert_eq!(self.game_state.dungeon.door_setup.width_road_address(), 0);
    }

    fn flip_crystal_peg_attribute_bank(&mut self, index: usize, bank: u8) {
        let (bg1, offset) = match bank {
            0 => (false, index),
            1 => (false, 0x0800 + index),
            2 => (true, index),
            3 => (true, 0x0800 + index),
            _ => unreachable!("crystal-peg attribute bank exceeded the source loop"),
        };
        let value = if bg1 {
            self.game_state.dungeon.bg2_attributes.bg1_tile(offset)
        } else {
            self.game_state.dungeon.bg2_attributes.bg2_tile(offset)
        };
        if let Some(tile) = value.toggled_crystal_peg() {
            if bg1 {
                self.dungeon_bg2_attributes_mut()
                    .set_bg2_tile(0x1000 + offset, tile);
            } else {
                self.dungeon_bg2_attributes_mut().set_bg2_tile(offset, tile);
            }
        }
    }

    fn flip_crystal_peg_attribute_index(&mut self, index: usize) {
        for bank in 0..4 {
            self.flip_crystal_peg_attribute_bank(index, bank);
        }
    }

    /// ROM: `Dungeon_FlipCrystalPegAttribute`
    pub(super) fn dungeon_flip_crystal_peg_attribute(&mut self) {
        for index in (0..=0x07ff).rev() {
            self.flip_crystal_peg_attribute_index(index);
        }
    }

    fn dungeon_peg_attribute_flip_progress_ordinal(
        progress: crate::DungeonPegAttributeFlipProgressReceipt,
    ) -> usize {
        if progress.index == 0xffff {
            assert_eq!(progress.completed_banks, 0);
            return 0x0800 * 4;
        }
        assert!(progress.index <= 0x07ff);
        assert!(progress.completed_banks <= 4);
        (0x07ff - usize::from(progress.index)) * 4 + usize::from(progress.completed_banks)
    }

    fn apply_dungeon_peg_attribute_flip_ordinal_range(&mut self, start: usize, end: usize) {
        assert!(start <= end);
        assert!(end <= 0x0800 * 4);
        for ordinal in start..end {
            let index = 0x07ff - ordinal / 4;
            let bank = (ordinal & 3) as u8;
            self.flip_crystal_peg_attribute_bank(index, bank);
        }
    }

    pub(super) fn dungeon_flip_crystal_peg_attribute_through_progress(
        &mut self,
        progress: crate::DungeonPegAttributeFlipProgressReceipt,
    ) {
        let end = Self::dungeon_peg_attribute_flip_progress_ordinal(progress);
        self.apply_dungeon_peg_attribute_flip_ordinal_range(0, end);
    }

    pub(in crate::zelda_rtl) fn advance_dungeon_peg_attribute_flip_between_progress(
        &mut self,
        previous: crate::DungeonPegAttributeFlipProgressReceipt,
        next: crate::DungeonPegAttributeFlipProgressReceipt,
    ) {
        let start = Self::dungeon_peg_attribute_flip_progress_ordinal(previous);
        let end = Self::dungeon_peg_attribute_flip_progress_ordinal(next);
        assert!(
            start <= end,
            "the source peg-attribute cursor moved backward: {previous:?} -> {next:?}",
        );
        self.apply_dungeon_peg_attribute_flip_ordinal_range(start, end);
    }

    pub(in crate::zelda_rtl) fn complete_dungeon_peg_attribute_flip_body_after_progress(
        &mut self,
        progress: crate::DungeonPegAttributeFlipProgressReceipt,
    ) {
        let start = Self::dungeon_peg_attribute_flip_progress_ordinal(progress);
        self.apply_dungeon_peg_attribute_flip_ordinal_range(start, 0x0800 * 4);
    }

    pub(in crate::zelda_rtl) fn complete_dungeon_peg_attribute_flip_after_progress(
        &mut self,
        progress: crate::DungeonPegAttributeFlipProgressReceipt,
    ) {
        self.complete_dungeon_peg_attribute_flip_body_after_progress(progress);
        self.set_subsubmodule(0);
        self.set_submodule(0);
    }

    pub(super) fn write_attr2(&mut self, j: usize, attr: TilePair) {
        let attr_view = &self.game_state.dungeon.bg2_attributes;
        if attr_view.bg2_attr_pair(j).is_none() {
            return;
        }
        self.dungeon_bg2_attributes_mut().set_bg2_tiles(j, attr.0);
    }

    pub(super) fn write_attr1(&mut self, j: usize, attr: TilePair) {
        if j + 1 >= 0x1000 {
            return;
        }
        self.dungeon_bg2_attributes_mut().set_bg1_tiles(j, attr.0);
    }

    /// ROM: `Dungeon_LoadSingleDoorTileAttribute`
    fn dungeon_load_single_door_tile_attribute(&mut self) {
        let mut i = 0usize;
        while i != self.game_state.dungeon.room_parser.toggle_floor_count_x2() as usize {
            let j = self.game_state.dungeon.room_parser.toggle_floor_pos(i >> 1) as usize;
            if self
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_tile(j)
                .dungeon_role()
                == DungeonRole::OpenDoor
            {
                let attr = self.game_state.dungeon.bg2_attributes.bg2_tiles(j);
                self.write_attr2(j + xy(0, 0), attr.with_floor_transition());
                self.write_attr2(j + xy(0, 1), attr.with_floor_transition());
            } else {
                let attr = self.game_state.dungeon.bg2_attributes.bg1_tiles(j);
                self.write_attr1(j + xy(0, 0), attr.with_floor_transition());
                self.write_attr1(j + xy(0, 1), attr.with_floor_transition());
            }
            i += 2;
        }

        i = 0;
        while i != self.game_state.dungeon.room_parser.toggle_palace_count_x2() as usize {
            let j = self
                .game_state
                .dungeon
                .room_parser
                .toggle_palace_pos(i >> 1) as usize;
            if self
                .game_state
                .dungeon
                .bg2_attributes
                .bg2_tile(j)
                .dungeon_role()
                == DungeonRole::OpenDoor
            {
                let attr = self.game_state.dungeon.bg2_attributes.bg2_tiles(j);
                self.write_attr2(j + xy(0, 0), attr.with_palace_transition());
                self.write_attr2(j + xy(0, 1), attr.with_palace_transition());
            } else {
                let attr = self.game_state.dungeon.bg2_attributes.bg1_tiles(j);
                self.write_attr1(j + xy(0, 0), attr.with_palace_transition());
                self.write_attr1(j + xy(0, 1), attr.with_palace_transition());
            }
            i += 2;
        }
    }
}
