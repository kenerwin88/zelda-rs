// Methods ported from zelda3/src/overlord.c and included inside ZeldaState.

use super::*;
use crate::tile_definition::NativeTile;

mod overlord_shared;
use crate::types::sign8;
use overlord_shared::*;

impl ZeldaState {
    pub(super) fn overlord_get_x(&self, k: usize) -> u16 {
        self.overlord_slot_view(k).x_low() as u16
            | ((self.overlord_slot_view(k).x_high() as u16) << 8)
    }

    pub(super) fn overlord_get_y(&self, k: usize) -> u16 {
        self.overlord_slot_view(k).y_low() as u16
            | ((self.overlord_slot_view(k).y_high() as u16) << 8)
    }

    pub(super) fn overlord_stalfos_factory(&self, _k: usize) {
        // Overlord_StalfosFactory is an unused assert-only slot in C.
        panic!("overlord_stalfos_factory assert");
    }

    pub(super) fn overlord_set_x(&mut self, k: usize, v: u16) {
        let value = v as u8;
        self.overlord_slot_view_mut(k).set_x_low(value);
        let value = (v >> 8) as u8;
        self.overlord_slot_view_mut(k).set_x_high(value);
    }

    pub(super) fn overlord_set_y(&mut self, k: usize, v: u16) {
        let value = v as u8;
        self.overlord_slot_view_mut(k).set_y_low(value);
        let value = (v >> 8) as u8;
        self.overlord_slot_view_mut(k).set_y_high(value);
    }

    pub(super) fn overlord_spawn_boulder(&mut self) {
        // Cycle ledger: Overlord_SpawnBoulder $09:B714 (m8 x8, JSR target).
        // The ROM tests the exits in order: $09:B714-B716 LDA $1b : BNE (40),
        // $09:B718-B71B LDA $0ffd : BEQ (48), $09:B71D-B722 LDA $11 ORA
        // $0fc1 BNE (72); a taken branch (+6) lands on $09:B772 RTS (42).
        let _scope = crate::cycle_ledger::routine(0x09_b714);
        crate::cycle_ledger::charge(if self.game_state.world.location.is_indoors() {
            40 + 6 + 42
        } else if self.game_state.sprites.garnish_runtime.boulder_trap_count() == 0 {
            40 + 48 + 6 + 42
        } else if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) != 0
        {
            40 + 48 + 72 + 6 + 42
        } else {
            40 + 48 + 72
        });
        if self.game_state.world.location.is_indoors()
            || self.game_state.sprites.garnish_runtime.boulder_trap_count() == 0
            || (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) != 0
        {
            return;
        }
        self.overlord_spawn_boulder_after_guard();
    }

    pub(super) fn overlord_spawn_boulder_after_guard(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x09_b714);
        let timer = self.garnish_state_mut().increment_boulder_trap_timer();
        // $09:B724-B72C INC $0ffe LDA AND #$3f BNE (110), taken (+6) to the
        // RTS (42) unless the counter wrapped.
        if timer & 63 != 0 {
            crate::cycle_ledger::charge(110 + 6 + 42);
            return;
        }
        crate::cycle_ledger::charge(110);
        let camera_y_hi = (self.game_state.display.ppu_scroll_copy.bg2_v_copy2() >> 8) as u8;
        let coll_y_hi = self.game_state.sprites.garnish_runtime.sprcoll_y_hi();
        // $09:B72E-B736 LDA $e9 SEC SBC $0fbf CMP #$02 BMI (102), taken (+6)
        // to the RTS (42) when the camera is above the collision base.
        if sign8(camera_y_hi.wrapping_sub(coll_y_hi).wrapping_sub(2)) {
            crate::cycle_ledger::charge(102 + 6 + 42);
            return;
        }
        // $09:B738-B740 LDA #$c2 LDY #$0d JSL Sprite_SpawnDynamically BMI
        // (110): no slot takes the BMI (+6) to the RTS; a slot runs
        // $09:B742-B76F (562, the JSL GetRandomNumber included) then the RTS.
        crate::cycle_ledger::charge(102 + 110);
        if let Some((j, _info)) = self.try_spawn_sprite_with_info(0, 0xc2) {
            crate::cycle_ledger::charge(562 + 42);
            let x = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add((self.get_random_number() & 127) as u16)
                .wrapping_add(64);
            let y = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_sub(0x30);
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_floor(0);
            sprite.set_direction(0);
            sprite.set_z(0);
        } else {
            // BMI taken (+6) to $09:B772 RTS (42).
            crate::cycle_ledger::charge(6 + 42);
        }
    }

    pub(super) fn overlord_main(&mut self) {
        // Cycle ledger: Overlord_Main $09:B773 (m8 x8, JSL target): PHB PHK
        // PLB JSR Overlord_ExecuteAll JSR Overlord_SpawnBoulder PLB RTL (236;
        // the callees charge their own bodies).
        let _scope = crate::cycle_ledger::routine(0x09_b773);
        crate::cycle_ledger::charge(236);
        self.overlord_execute_all();
        self.overlord_spawn_boulder();
    }

    pub(super) fn overlord_execute_all(&mut self) {
        // Cycle ledger: Overlord_ExecuteAll $09:B77E (m8 x8, JSR target).
        // $09:B77E-B783 LDA $11 ORA $0fc1 BNE (72): taken (+6) to $09:B792
        // RTS (42) while a submodule or the pause flag is active.
        let _scope = crate::cycle_ledger::routine(0x09_b77e);
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) != 0 {
            crate::cycle_ledger::charge(72 + 6 + 42);
            return;
        }
        // $09:B785 LDX #$07 (16); per slot $09:B787-B78A LDA $b00,x : BEQ
        // (48) then $09:B78C JSR Overlord_ExecuteSingle (46) or the taken BEQ
        // (+6), and $09:B78F-B790 DEX : BPL (30, +6 taken for slots 7..1).
        crate::cycle_ledger::charge(72 + 16);
        self.overlord_execute_slots_below(8);
    }

    fn overlord_execute_slots_below(&mut self, exclusive: usize) {
        assert!(exclusive<=8);
        for i in (0..exclusive).rev() {
            if self.overlord_slot_view(i).overlord_type() != 0 {
                crate::cycle_ledger::charge(48 + 46);
                self.overlord_execute_single(i);
            } else {
                crate::cycle_ledger::charge(48 + 6);
            }
            crate::cycle_ledger::charge(if i > 0 { 30 + 6 } else { 30 });
        }
        // $09:B792 RTS (42).
        crate::cycle_ledger::charge(42);
    }

    pub(super) fn overlord_execute_single(&mut self, k: usize) {
        // Cycle ledger: Overlord_ExecuteSingle $09:B793 (JSR target): PHA,
        // JSR Overlord_CheckIfActive, PLA DEC REP #$30 AND ASL TAY LDA
        // $b7a8,y DEC PHA SEP #$30 RTS (338, an RTS dispatch, so the handler
        // runs inside this scope; Overlord_CheckIfActive charges itself when
        // annotated).
        let _scope = crate::cycle_ledger::routine(0x09_b793);
        crate::cycle_ledger::charge(338);
        let j = self.overlord_slot_view(k).overlord_type();
        self.overlord_check_if_active(k);
        match j {
            1 => self.overlord01_position_target(k),
            2 => self.overlord02_full_room_cannons(k),
            3 => self.overlord03_vertical_cannon(k),
            4 => self.overlord_stalfos_factory(k),
            5 => self.overlord05_falling_stalfos(k),
            6 => self.overlord06_bad_switch_snake(k),
            7 => self.overlord07_moving_floor(k),
            8 => self.overlord08_blob_spawner(k),
            9 => self.overlord09_wallmaster_spawner(k),
            10..=15 => self.overlord0_a_falling_square(k),
            16..=19 => self.overlord10_pirogusu_spawner_left(k),
            20 => self.overlord14_tile_room(k),
            21 => self.overlord15_wizzrobe_spawner(k),
            22 => self.overlord16_zoro_spawner(k),
            23 => self.overlord17_pot_trap(k),
            24 => self.overlord18_invisible_stalfos(k),
            25 => self.overlord19_armos_coordinator_bounce(k),
            26 => self.overlord06_bad_switch_snake(k),
            _ => panic!("overlord_execute_single invalid type {j}"),
        }
    }

    pub(super) fn overlord19_armos_coordinator_bounce(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x1d_ebf3);
        let lag_free = self.lag_free;
        let charge = |clocks| { if lag_free { crate::cycle_ledger::charge(clocks); } };
        // $1D:EBF3..EC01, M8/X8; JumpTableLocal consumes its JSL frame.
        charge(48 + if self.overlord_slot_view(k).gen2() == 0 { 6 } else { 52 } + 32 + 62 + 414);
        if self.overlord_slot_view(k).gen2() != 0 {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
        }
        match self.overlord_slot_view(k).gen1() {
            0 => {
                charge(48);
                if self.sprite_slot_view(0).a() != 0 {
                    charge(298 + 42); // $1D:EC17..EC33, child body excluded.
                    let value = 120;
                    self.overlord_slot_view_mut(k).set_x_low(value);
                    let value = 255;
                    self.overlord_slot_view_mut(k).set_floor(value);
                    let value = 64;
                    self.overlord_slot_view_mut(2).set_x_low(value);
                    self.overlord_slot_view_mut(0).set_x_low(192);
                    let value = 1;
                    self.overlord_slot_view_mut(1).set_x_low(value);
                    let _child = crate::cycle_ledger::routine(0x1d_eccc);
                    self.armos_coordinator_rotate_knights(k);
                } else {
                    charge(6 + 42);
                }
            }
            1 => {
                charge(62 + 42); // $1D:EC34..EC41, excluding CheckKnights.
                if self.armos_coordinator_check_knights() {
                    charge(106);
                    let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                    self.overlord_slot_view_mut(k).set_gen1(value);
                    let value = 0xff;
                    self.overlord_slot_view_mut(k).set_gen2(value);
                } else {
                    charge(6);
                }
            }
            2 | 4 => self.armos_coordinator_rotate_knights(k),
            3 => {
                charge(110 + 22); // $1D:EC96..ECA9, tail Rotate below.
                let value = self.overlord_slot_view(2).x_low().wrapping_sub(1);
                self.overlord_slot_view_mut(2).set_x_low(value);
                if self.overlord_slot_view(2).x_low() == 32 {
                    charge(106);
                    let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                    self.overlord_slot_view_mut(k).set_gen1(value);
                    let value = 64;
                    self.overlord_slot_view_mut(k).set_gen2(value);
                } else {
                    charge(6);
                }
                self.armos_coordinator_rotate(k);
            }
            5 => {
                charge(110 + 22); // $1D:ECAB..ECBE, tail Rotate below.
                let value = self.overlord_slot_view(2).x_low().wrapping_add(1);
                self.overlord_slot_view_mut(2).set_x_low(value);
                if self.overlord_slot_view(2).x_low() == 64 {
                    charge(106);
                    let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                    self.overlord_slot_view_mut(k).set_gen1(value);
                    let value = 64;
                    self.overlord_slot_view_mut(k).set_gen2(value);
                } else {
                    charge(6);
                }
                self.armos_coordinator_rotate(k);
            }
            6 => {
                charge(48 + 42); // $1D:EC48..EC68.
                if self.overlord_slot_view(k).gen2() != 0 {
                    charge(6);
                    return;
                }
                charge(62 + 6 * 154 + 5 * 6 + 106);
                self.armos_coordinator_disable_coercion(k);
                for j in (0..=5).rev() {
                    let value = ARMOS_COORDINATOR_BACK_WALL_X_OFFSETS[j];
                    self.overlord_slot_view_mut(j).set_x_high(value);
                    let value = 48;
                    self.overlord_slot_view_mut(j).set_gen2(value);
                }
                let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
                let value = 255;
                self.overlord_slot_view_mut(k).set_gen2(value);
            }
            7 => {
                charge(48); // $1D:EC69..EC6C.
                if self.overlord_slot_view(k).gen2() != 0 {
                    charge(6 + 42);
                    return;
                }
                charge(16); // $1D:EC6E.
                for j in (0..=5).rev() {
                    charge(132); // $1D:EC70..EC7B.
                    let value = self.overlord_slot_view(j).gen2().wrapping_add(1);
                    self.overlord_slot_view_mut(j).set_gen2(value);
                    if self.overlord_slot_view(j).gen2() == 192 {
                        charge(288); // $1D:EC7D..EC91, children excluded.
                        let value = 1;
                        self.overlord_slot_view_mut(k).set_gen1(value);
                        let value = 0u8.wrapping_sub(self.overlord_slot_view(k).floor());
                        self.overlord_slot_view_mut(k).set_floor(value);
                        self.armos_coordinator_disable_coercion(k);
                        let _child = crate::cycle_ledger::routine(0x1d_ecd4);
                        self.armos_coordinator_rotate(k);
                        return;
                    }
                    charge(6 + 30 + if j != 0 { 6 } else { 42 });
                }
            }
            _ => {}
        }
    }

    pub(super) fn overlord18_invisible_stalfos(&mut self, k: usize) {
        let x = self.overlord_get_x(k);
        let y = self.overlord_get_y(k);
        if x.wrapping_sub(self.game_state.player.follower_link.x())
            .wrapping_add(24)
            >= 48
            || y.wrapping_sub(self.game_state.player.follower_link.y())
                .wrapping_add(24)
                >= 48
        {
            return;
        }
        let value = 0;
        self.overlord_slot_view_mut(k).set_overlord_type(value);
        self.temp_counter_mut().set(3);
        loop {
            let i = self.game_state.scratch_counter.value() as usize;
            let Some((j, _info)) = self.try_spawn_sprite_with_info_up_to_slot(k, 0xa7, 12) else {
                return;
            };
            self.sprite_set_x(
                j,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(INVISIBLE_STALFOS_TRAP_X_OFFSETS[i] as i16 as u16),
            );
            self.sprite_set_y(
                j,
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(INVISIBLE_STALFOS_TRAP_Y_OFFSETS[i] as i16 as u16),
            );
            let floor = self.overlord_slot_view(k).floor();
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_delay_main(INVISIBLE_STALFOS_TRAP_DELAYS[i]);
            sprite.set_floor(floor);
            sprite.set_e(1);
            sprite.set_flags2(3);
            sprite.set_direction(2);
            self.temp_counter_mut().decrement();
            if sign8(self.game_state.scratch_counter.value()) {
                break;
            }
        }
    }

    pub(super) fn overlord17_pot_trap(&mut self, k: usize) {
        let x = self.overlord_get_x(k);
        let y = self.overlord_get_y(k);
        if x.wrapping_sub(self.game_state.player.follower_link.x())
            .wrapping_add(32)
            < 64
            && y.wrapping_sub(self.game_state.player.follower_link.y())
                .wrapping_add(32)
                < 64
        {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            self.dungeon_room_effects_mut()
                .increment_trap_trigger_latch();
        }
    }

    pub(super) fn overlord16_zoro_spawner(&mut self, k: usize) {
        let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
        self.overlord_slot_view_mut(k).set_gen2(value);
        let x = self.overlord_get_x(k).wrapping_add(8);
        let y = self.overlord_get_y(k).wrapping_add(8);
        // The spawner only works from its nest, the original door identity 0x82.
        let floor = self.overlord_slot_view(k).floor();
        if self.probe_entity_tile(floor, x, y).0 != NativeTile::from_cartridge(0x82) {
            return;
        }
        if self.overlord_slot_view(k).gen2() >= 0x18 || self.overlord_slot_view(k).gen2() & 3 != 0 {
            return;
        }
        if let Some((j, info)) = self.try_spawn_sprite_with_info_up_to_slot(k, 0x9c, 12) {
            let random_x = ZORO_SPAWNER_X_OFFSETS[(self.get_random_number() & 7) as usize];
            self.sprite_set_x(
                j,
                info.r5_overlord_x
                    .wrapping_add(random_x as i16 as u16)
                    .wrapping_add(8),
            );
            let floor = self.overlord_slot_view(k).floor();
            let subtype2 = self.get_random_number();
            let y = (info.r7_overlord_y & 0xff00) | info.r7_overlord_y.wrapping_add(8) & 0x00ff;
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_y(y);
            sprite.set_floor(floor);
            sprite.set_flags4(1);
            sprite.set_e(1);
            sprite.set_ignore_projectile(1);
            sprite.set_y_velocity(16);
            sprite.set_flags2(32);
            sprite.set_oam_flags(13);
            sprite.set_subtype2(subtype2);
            sprite.set_delay_main(48);
            sprite.set_bump_damage(3);
        }
    }

    pub(super) fn overlord15_wizzrobe_spawner(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 128 {
            if self.game_state.frame.frame_counter & 1 != 0 {
                let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
                self.overlord_slot_view_mut(k).set_gen2(value);
            }
            return;
        }
        let value = 127;
        self.overlord_slot_view_mut(k).set_gen2(value);
        for i in (0..=3).rev() {
            if let Some((j, _info)) = self.try_spawn_sprite_with_info_up_to_slot(k, 0x9b, 12) {
                self.sprite_set_x(
                    j,
                    self.game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_add(WIZZROBE_SPAWNER_X_OFFSETS[i] as i16 as u16),
                );
                self.sprite_set_y(
                    j,
                    self.game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(WIZZROBE_SPAWNER_Y_OFFSETS[i] as i16 as u16),
                );
                let floor = self.overlord_slot_view(k).floor();
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_delay_main(WIZZROBE_SPAWNER_DELAYS[i]);
                sprite.set_floor(floor);
                sprite.set_b(1);
            }
        }
        self.temp_counter_mut().set(0xff);
    }

    pub(super) fn overlord14_tile_room(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x & 0xff00 != 0 || y & 0xff00 != 0 {
            return;
        }
        let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
        self.overlord_slot_view_mut(k).set_gen2(value);
        if self.overlord_slot_view(k).gen2() != 0x80 {
            return;
        }
        if self.tile_room_spawn_tile(k) < 0 {
            let value = 0x81;
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if self.overlord_slot_view(k).gen1() != 22 {
            let value = 0xe0;
            self.overlord_slot_view_mut(k).set_gen2(value);
        } else {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
        }
    }

    pub(super) fn tile_room_spawn_tile(&mut self, k: usize) -> i32 {
        let Some((j, _info)) = self.try_spawn_sprite_with_info(k, 0x94) else {
            return -1;
        };
        let i = self.overlord_slot_view(k).gen1() as usize;
        let x = TILE_ROOM_FLYING_TILE_X_POSITIONS[i] as u16
            | ((self.overlord_slot_view(k).x_high() as u16) << 8);
        let y = TILE_ROOM_FLYING_TILE_Y_POSITIONS[i].wrapping_sub(8) as u16
            | ((self.overlord_slot_view(k).y_high() as u16) << 8);
        let floor = self.overlord_slot_view(k).floor();
        let mut sprite = self.sprite_slot_view_mut(j);
        sprite.set_e(1);
        sprite.set_x(x);
        sprite.set_y(y);
        sprite.set_floor(floor);
        sprite.set_health(4);
        sprite.set_flags5(0);
        sprite.set_health(0);
        sprite.set_deflection_bits(8);
        sprite.set_flags2(4);
        sprite.set_oam_flags(1);
        sprite.set_bump_damage(4);
        j as i32
    }

    pub(super) fn overlord10_pirogusu_spawner_left(&mut self, k: usize) {
        let counter = self.overlord_slot_view(k).overlord_type().wrapping_sub(16);
        self.temp_counter_mut().set(counter);
        if self.overlord_slot_view(k).gen2() != 128 {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        let value = (self.get_random_number() & 31).wrapping_add(96);
        self.overlord_slot_view_mut(k).set_gen2(value);
        let mut n = 0;
        for i in 0..16 {
            let sprite = self.sprite_slot_view(i);
            if sprite.state() != 0 && sprite.sprite_type() == 0x10 {
                n += 1;
            }
        }
        if n >= 5 {
            return;
        }
        if let Some((j, info)) = self.try_spawn_sprite_with_info_up_to_slot(k, 0x94, 12) {
            self.sprite_set_x(j, info.r5_overlord_x);
            self.sprite_set_y(j, info.r7_overlord_y);
            let floor = self.overlord_slot_view(k).floor();
            let direction = self.game_state.scratch_counter.value();
            let a = PIROGUSU_SPAWNER_DIRECTIONS[direction as usize];
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_floor(floor);
            sprite.set_delay_main(32);
            sprite.set_direction(direction);
            sprite.set_a(a);
        }
    }

    pub(super) fn overlord0_a_falling_square(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 0 {
            if self.overlord_slot_view(k).gen3() != 0 {
                let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
                self.overlord_slot_view_mut(k).set_gen2(value);
                return;
            }
            let x = self
                .overlord_get_x(k)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y = self
                .overlord_get_y(k)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            if !(x & 0xff00 != 0 || y & 0xff00 != 0) {
                let value = self.overlord_slot_view(k).gen3().wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen3(value);
            }
            return;
        }
        let value = 16;
        self.overlord_slot_view_mut(k).set_gen2(value);
        self.spawn_falling_tile(k);
        let j = self.overlord_slot_view(k).overlord_type().wrapping_sub(10) as usize;
        let i = self.overlord_slot_view(k).gen1() as usize;
        let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if i == (FALLING_SQUARE_CRUMBLE_PATH_OFFSETS[j + 1]
            - FALLING_SQUARE_CRUMBLE_PATH_OFFSETS[j]) as usize
        {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
        }
        let t = FALLING_SQUARE_CRUMBLE_PATH_DIRECTIONS
            [FALLING_SQUARE_CRUMBLE_PATH_OFFSETS[j] as usize + i];
        if t == 0xff {
            self.overlord_set_x(k, self.overlord_get_x(k).wrapping_add(0x0c1a));
            self.overlord_set_y(k, self.overlord_get_y(k).wrapping_add(0xbb66));
        } else {
            self.overlord_set_x(
                k,
                self.overlord_get_x(k)
                    .wrapping_add(FALLING_SQUARE_CRUMBLE_X_DELTAS[t as usize] as i16 as u16),
            );
            self.overlord_set_y(
                k,
                self.overlord_get_y(k)
                    .wrapping_add(FALLING_SQUARE_CRUMBLE_Y_DELTAS[t as usize] as i16 as u16),
            );
        }
    }

    pub(super) fn spawn_falling_tile(&mut self, k: usize) {
        if let Some(j) = self.GarnishAlloc() {
            let x_high = self.overlord_slot_view(k).x_high();
            let x_low = self.overlord_slot_view(k).x_low();
            {
                let mut garnish = self.garnish_slot_view_mut(j);
                garnish.set_garnish_type(3);
                garnish.set_x_high(x_high);
                garnish.set_x_low(x_low);
            }
            let sfx = self.calculate_sfx_pan_arbitrary(x_low) | 0x1f;
            self.set_sound_effect_1(sfx);
            let y = self.overlord_get_y(k).wrapping_add(16);
            let mut garnish = self.garnish_slot_view_mut(j);
            garnish.set_y_low(y as u8);
            garnish.set_y_high((y >> 8) as u8);
            garnish.set_countdown(31);
            self.garnish_state_mut().set_active_type(31);
        }
    }

    pub(super) fn overlord09_wallmaster_spawner(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 128 {
            if self.game_state.frame.frame_counter & 1 == 0 {
                let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
                self.overlord_slot_view_mut(k).set_gen2(value);
            }
            return;
        }
        let value = 127;
        self.overlord_slot_view_mut(k).set_gen2(value);
        let Some((j, _info)) = self.try_spawn_sprite_with_info_up_to_slot(k, 0x90, 12) else {
            return;
        };
        self.sprite_set_x(j, self.game_state.player.follower_link.x());
        self.sprite_set_y(j, self.game_state.player.follower_link.y());
        self.sprite_slot_view_mut(j).set_z(208);
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.sprite_slot_view_mut(j).set_floor(floor);
    }

    pub(super) fn overlord08_blob_spawner(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 0 {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        let value = 0xa0;
        self.overlord_slot_view_mut(k).set_gen2(value);
        let mut n = 0;
        for i in 0..16 {
            let sprite = self.sprite_slot_view(i);
            if sprite.state() != 0 && sprite.sprite_type() == 0x8f {
                n += 1;
            }
        }
        if n >= 5 {
            return;
        }
        if let Some((j, _info)) = self.try_spawn_sprite_with_info_up_to_slot(k, 0x8f, 12) {
            let i = self.game_state.player.follower_link.facing_index();
            self.sprite_set_x(
                j,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(BLOB_SPAWNER_ZOL_X_OFFSETS[i] as i16 as u16),
            );
            self.sprite_set_y(
                j,
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(BLOB_SPAWNER_ZOL_Y_OFFSETS[i] as i16 as u16),
            );
            let floor = self.game_state.player.follower_link.lower_level_state();
            let head_direction = self.get_random_number() & 31 | 16;
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_z(192);
            sprite.set_floor(floor);
            sprite.set_ai_state(2);
            sprite.set_e(2);
            sprite.set_c(2);
            sprite.set_head_direction(head_direction);
        }
    }

    pub(super) fn overlord07_moving_floor(&mut self, k: usize) {
        if self.sprite_slot_view(0).state() == 4 {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            self.dungeon_moving_floor_mut().set_floor_move_flags(1);
            return;
        }

        if self.overlord_slot_view(k).gen1() == 0 {
            let value = self.overlord_slot_view(k).gen2().wrapping_add(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
            if self.overlord_slot_view(k).gen2() == 32 {
                let value = 0;
                self.overlord_slot_view_mut(k).set_gen2(value);
                let mask = if self.overlord_slot_view(k).x_low() != 0 {
                    3
                } else {
                    1
                };
                let flags = (self.get_random_number() & mask) * 2;
                self.dungeon_moving_floor_mut()
                    .set_floor_move_flags(u16::from(flags));
                // ROM Garnish03_FallingTile/overlord gen2 $09:BD6E: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                let value = self.get_random_number_with_carry().masked_adc(127, 128);
                self.overlord_slot_view_mut(k).set_gen2(value);
                let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
            } else {
                self.dungeon_moving_floor_mut().set_floor_move_flags(1);
            }
        } else {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
            if self.overlord_slot_view(k).gen2() == 0 {
                let value = 0;
                self.overlord_slot_view_mut(k).set_gen1(value);
            }
        }
    }

    pub(super) fn sprite_overlord_play_falling_sfx(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
    }

    pub(super) fn overlord05_falling_stalfos(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x & 0xff00 != 0 || y & 0xff00 != 0 {
            return;
        }
        if self.overlord_slot_view(k).gen1() == 0 {
            if self.game_state.dungeon.room_effects.trap_trigger_latch() != 0 {
                let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
            }
            return;
        }
        let old = self.overlord_slot_view(k).gen1();
        let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if old == FALLING_STALFOS_TRAP_TRIGGER_TIMERS[k] {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            let Some((j, info)) = self.try_spawn_sprite_with_info_up_to_slot(k, 0x85, 12) else {
                return;
            };
            self.sprite_set_x(j, info.r5_overlord_x);
            self.sprite_set_y(j, info.r7_overlord_y);
            let floor = self.overlord_slot_view(k).floor();
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_z(224);
            sprite.set_floor(floor);
            // The vanilla ROM leaves sprite_D uninitialized here: the spawn
            // already copied sprite_D[k] with the OVERLORD index aliased into
            // the sprite table (oracle D=0x24 at route frame 90920). The C
            // port's `sprite_D[j] = 0` bug-fix is not ROM behavior.
            self.sprite_overlord_play_falling_sfx(j);
        }
    }

    pub(super) fn overlord06_bad_switch_snake(&mut self, k: usize) {
        let a = self.overlord_slot_view(k).gen1();
        if a == 0 {
            if self
                .game_state
                .dungeon
                .room_effects
                .has_bomb_trap_activation()
            {
                let value = 1;
                self.overlord_slot_view_mut(k).set_gen1(value);
            }
            return;
        }
        let value = a.wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if a != SNAKE_TRAP_SPAWN_TIMERS_BY_SLOT[k] {
            return;
        }
        let Some((j, info)) = self.try_spawn_sprite_with_info(k, 0x6e) else {
            return;
        };
        self.sprite_set_x(j, info.r5_overlord_x);
        self.sprite_set_y(j, info.r7_overlord_y);
        let floor = self.overlord_slot_view(k).floor();
        {
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_z(192);
            sprite.set_e(192);
            sprite.or_flags3(0x10);
            sprite.set_floor(floor);
        }
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        let sprite_type = self.overlord_slot_view(k).overlord_type();
        let value = 0;
        self.overlord_slot_view_mut(k).set_overlord_type(value);
        if sprite_type == 26 {
            self.sprite_slot_view_mut(j).set_sprite_type(74);
            self.sprite_transmute_to_bomb(j);
            self.sprite_slot_view_mut(j).set_delay_aux1(112);
        }
    }

    pub(super) fn overlord02_full_room_cannons(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if (x | y) & 0xff00 != 0 || self.game_state.frame.frame_counter & 0x0f != 0 {
            return;
        }
        self.sprite_workspace_mut().set_shared_scratch_a(0);
        let j = (self.get_random_number() & 15) as usize;
        self.temp_counter_mut()
            .set(FULL_ROOM_CANNON_BALL_DIRECTIONS[j]);
        let value = FULL_ROOM_CANNON_BALL_X_POSITIONS[j];
        self.overlord_slot_view_mut(k).set_x_low(value);
        let value = self.game_state.sprites.workspace.room_origin_x_high();
        self.overlord_slot_view_mut(k).set_x_high(value);
        let value = FULL_ROOM_CANNON_BALL_Y_POSITIONS[j];
        self.overlord_slot_view_mut(k).set_y_low(value);
        let value = self
            .game_state
            .sprites
            .workspace
            .room_origin_y_high()
            .wrapping_add(1);
        self.overlord_slot_view_mut(k).set_y_high(value);
        self.overlord_spawn_cannon_ball(k, 0);
    }

    pub(super) fn overlord03_vertical_cannon(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        if x & 0xff00 != 0 {
            let value = 255;
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        if self.game_state.frame.frame_counter & 1 == 0 && self.overlord_slot_view(k).gen2() != 0 {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
        }
        self.temp_counter_mut().set(2);
        self.sprite_workspace_mut().set_shared_scratch_a(0);
        let value = self.overlord_slot_view(k).gen1().wrapping_sub(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if !sign8(self.overlord_slot_view(k).gen1()) {
            return;
        }
        let value = 56;
        self.overlord_slot_view_mut(k).set_gen1(value);
        let xd = if self.overlord_slot_view(k).gen2() == 0 {
            let value = 160;
            self.overlord_slot_view_mut(k).set_gen2(value);
            self.sprite_workspace_mut().set_shared_scratch_a(160);
            8
        } else {
            ((self.get_random_number() & 2) * 8) as i32
        };
        self.overlord_spawn_cannon_ball(k, xd);
    }

    pub(super) fn overlord_spawn_cannon_ball(&mut self, k: usize, xd: i32) {
        let Some((j, info)) = self.try_spawn_sprite_with_info(k, 0x50) else {
            return;
        };
        self.sprite_set_x(j, info.r5_overlord_x.wrapping_add(xd as i16 as u16));
        self.sprite_set_y(j, info.r7_overlord_y.wrapping_sub(1));
        let counter = self.game_state.scratch_counter.value() as usize;
        let floor = self.overlord_slot_view(k).floor();
        let mut sprite = self.sprite_slot_view_mut(j);
        sprite.set_x_velocity(CANNON_BALL_X_VELOCITIES[counter] as u8);
        sprite.set_y_velocity(CANNON_BALL_Y_VELOCITIES[counter] as u8);
        sprite.set_floor(floor);
        if self.game_state.sprites.workspace.shared_scratch_a() != 0 {
            let scratch = self.game_state.sprites.workspace.shared_scratch_a();
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_ai_state(scratch);
            sprite.add_y_low(8);
            sprite.set_flags2(3);
            sprite.set_flags4(9);
        }
        self.sprite_slot_view_mut(j).set_delay_aux2(64);
        self.sprite_sfx_queue_sfx3_with_pan(j, 0x07);
    }

    pub(super) fn overlord01_position_target(&mut self, k: usize) {
        self.garnish_state_mut().set_active_overlord_index(k as u8);
    }

    pub(super) fn overlord_check_if_active(&mut self, k: usize) {
        if self.game_state.world.location.is_indoors() {
            return;
        }
        let j = (self.game_state.frame.frame_counter & 1) as usize;
        let x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_add(OVERLORD_ACTIVE_RANGE_OFFSETS[j])
            .wrapping_sub(self.overlord_get_x(k));
        let y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(OVERLORD_ACTIVE_RANGE_OFFSETS[j])
            .wrapping_sub(self.overlord_get_y(k));
        if ((x >> 15) as usize) != j || ((y >> 15) as usize) != j {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            let blk = self.overlord_slot_view(k).sprite_block_pos();
            if blk != 0xffff {
                let loadedmask = 0x80 >> (blk & 7);
                self.clear_overworld_sprite_loaded_mask(blk, loadedmask as u8);
            }
        }
    }

    pub(super) fn armos_coordinator_rotate_knights(&mut self, k: usize) {
        // $1D:ECCC..ECD1; Rotate is a shared fall-through, not another call.
        if self.lag_free { crate::cycle_ledger::charge(48 + if self.overlord_slot_view(k).gen2() == 0 { 52 } else { 6 }); }
        if self.overlord_slot_view(k).gen2() == 0 {
            let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
            self.overlord_slot_view_mut(k).set_gen1(value);
        }
        self.armos_coordinator_rotate(k);
    }

    pub(super) fn armos_coordinator_rotate(&mut self, k: usize) {
        // $1D:ECD4..ECEA, M8/X8. All eight overlord indices stay in-page.
        if self.lag_free { crate::cycle_ledger::charge(64 + if self.overlord_slot_view(k).floor() & 0x80 != 0 { 14 } else { 6 } + 188); }
        let angle = self
            .overlord_slot_view(0)
            .adjacent_x_low_word()
            .wrapping_add(self.overlord_slot_view(k).floor() as i8 as i16 as u16);
        self.overlord_slot_view_mut(0)
            .set_adjacent_x_low_word(angle);

        for (i, offset) in ARMOS_KNIGHT_RING_ANGLE_OFFSETS.iter().enumerate() {
            let t0 = angle.wrapping_add(*offset);
            let size = self.overlord_slot_view(2).x_low();
            if self.lag_free {
                // $1D:ECED..EDB7: two sine-table/multiply/sign paths per
                // knight. System-bank $4202/$4203/$4216/$4217 bus accesses
                // cost six clocks; the eight actual NOPs remain priced.
                let axis = |angle: u16| {
                    let magnitude = ARMOS_SINE_LOOKUP_TABLE[(angle & 255) as usize];
                    let value = armos_sin(angle, size);
                    (if magnitude < 256 { 230 } else { 6 }) + 54
                        + if angle & 256 != 0 { 30 } else { 6 }
                        + 56 + if value < 0 { 38 } else { 6 }
                };
                crate::cycle_ledger::charge(946 + axis(t0) + 328
                    + axis(t0.wrapping_add(0x80)) + 320 + if i == 5 { 6 + 42 } else { 24 });
            }
            let tx = self
                .overlord_get_x(k)
                .wrapping_add(armos_sin(t0, size) as i16 as u16);
            let value = tx as u8;
            self.overlord_slot_view_mut(i).set_x_high(value);
            let value = (tx >> 8) as u8;
            self.overlord_slot_view_mut(i).set_y_high(value);
            let ty = self
                .overlord_get_y(k)
                .wrapping_add(armos_sin(t0.wrapping_add(0x80), size) as i16 as u16);
            let value = ty as u8;
            self.overlord_slot_view_mut(i).set_gen2(value);
            let value = (ty >> 8) as u8;
            self.overlord_slot_view_mut(i).set_floor(value);
        }
        self.temp_counter_mut().set(6);
    }

    pub(super) fn armos_coordinator_check_knights(&self) -> bool {
        let _scope = crate::cycle_ledger::routine(0x1d_edb8);
        if self.lag_free { crate::cycle_ledger::charge(16); } // $1D:EDB8.
        for j in (0..=5).rev() {
            let sprite = self.sprite_slot_view(j);
            if self.lag_free {
                crate::cycle_ledger::charge(48 + if sprite.state() == 0 { 6 } else { 48 });
            }
            if sprite.state() != 0 && sprite.ai_state() == 0 {
                if self.lag_free { crate::cycle_ledger::charge(56); }
                return false;
            }
            if self.lag_free { crate::cycle_ledger::charge(if sprite.state() != 0 { 6 } else { 0 } + 30 + if j != 0 { 6 } else { 0 }); }
        }
        if self.lag_free { crate::cycle_ledger::charge(56); } // $1D:EDC9..EDCA.
        true
    }

    pub(super) fn armos_coordinator_disable_coercion(&mut self, _k: usize) {
        // $1D:EDCB..EDD5: six stores, five taken branches and RTS.
        if self.lag_free { crate::cycle_ledger::charge_routine(0x1d_edcb, 16 + 6 * 84 + 5 * 6 + 42); }
        for j in (0..=5).rev() {
            self.sprite_slot_view_mut(j).set_ai_state(0);
        }
    }

    /// ROM: `Sprite_SpawnDynamically`
    fn try_spawn_sprite_with_info(
        &mut self,
        k: usize,
        sprite: u8,
    ) -> Option<(usize, super::sprite::SpriteSpawnInfo)> {
        self.try_spawn_sprite_with_info_up_to_slot(k, sprite, 15)
    }

    /// ROM: `Sprite_SpawnDynamicallyEx`
    fn try_spawn_sprite_with_info_up_to_slot(
        &mut self,
        k: usize,
        sprite: u8,
        j_in: i32,
    ) -> Option<(usize, super::sprite::SpriteSpawnInfo)> {
        let mut info = super::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, sprite, &mut info, j_in);
        if j >= 0 {
            Some((j as usize, info))
        } else {
            None
        }
    }

    fn GarnishAlloc(&self) -> Option<usize> {
        (0..30)
            .rev()
            .find(|&j| self.garnish_slot_view(j).is_empty())
    }
}

fn armos_mult(a: u16, b: u8) -> u8 {
    if a >= 256 {
        return b;
    }
    let p = a * b as u16;
    ((p >> 8) + ((p >> 7) & 1)) as u8
}

fn armos_sin(a: u16, b: u8) -> i8 {
    let t = armos_mult(ARMOS_SINE_LOOKUP_TABLE[(a & 0xff) as usize], b);
    if a & 0x100 != 0 {
        (0u8.wrapping_sub(t)) as i8
    } else {
        t as i8
    }
}

impl ZeldaState {
    pub(super) fn finish_native_overlord_caller(&mut self, tail: crate::cycle_models::dungeon_overlord::OverlordTail) {
        use crate::cycle_models::dungeon_overlord::OverlordTail;
        let _scope=crate::cycle_ledger::routine(0x09_b773);
        match tail {
            OverlordTail::Returned=>return,
            OverlordTail::Slot {slot,kind}=>{
                assert_eq!(self.overlord_slot_view(usize::from(slot)).overlord_type(),kind);
                // Selected JSR has retired; retain its body, DEX/BPL, lower
                // slots, next JSR, bank restore and far return exactly once.
                crate::cycle_ledger::charge(118+if slot==0 {30}else{36});
                self.overlord_execute_single(usize::from(slot));
                self.overlord_execute_slots_below(usize::from(slot));
                self.overlord_spawn_boulder();
            },
            OverlordTail::Boulder=>{
                crate::cycle_ledger::charge(72);
                self.overlord_spawn_boulder_after_guard();
            },
        }
        crate::cycle_ledger::charge(48);
        self.archery_game_mut().clear_out_of_arrows();
    }
}

#[cfg(test)]
mod pacing_cost_tests {
    use super::*;
#[test]
fn armos_prices_match_executed_rom_paths() {
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (Ok(assets), Ok(rom)) = (std::fs::read(root.join("zelda3_assets.dat")),
        std::fs::read(root.join("saves/zelda3.sfc"))) else { return; };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    base.set_lag_free(true);
    for phase in 0..8u8 {
        for timer in [0u8, 1, 2, 0xbf] {
            for direction in [1u8, 0xff] {
                let mut native = base.clone();
                let mut image = native.wram_image().into_owned();
                let slot = 7;
                image[0xb28 + slot] = phase;
                image[0xb30 + slot] = timer;
                image[0xb40 + slot] = direction;
                image[0xb0a] = 0x30;
                image[0xb08] = 0x80;
                image[0xb09] = 1;
                image[0xd90] = 1;
                native.import_wram_image(&image);
                native.sync_native_game_state_from_ram();
                let entry = 0x1debf3u32;
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.ram.copy_from_slice(&native.wram_image());
                machine.ram[0x1fd..0x200].copy_from_slice(&[0xff, 0x7f, 0]);
                machine.cpu.e = false;
                machine.cpu.unpack_flags(0x30);
                machine.cpu.k = (entry >> 16) as u8;
                machine.cpu.db = 0x1d;
                machine.cpu.x = 7;
                machine.cpu.dp = 0;
                machine.cpu.pc = entry as u16;
                machine.cpu.sp = 0x1fc;
                let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                    CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                let mut source = RomCpuTimingProbe::new(machine, clock,
                    SourcePpuReadState::snes9x_reset()).unwrap();
                let mut measured = 0;
                for _ in 0..10000 {
                    let step = source.step().unwrap();
                    measured += step.transactions.iter()
                        .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                    if source.snes().cpu.pc == 0x8000 { break; }
                }
                assert_eq!(source.snes().cpu.pc, 0x8000);
                let start = crate::cycle_ledger::master();
                native.overlord19_armos_coordinator_bounce(7);
                assert_eq!(crate::cycle_ledger::master() - start, measured,
                    "phase={phase} timer={timer} direction={direction}");
            }
        }
    }
}

}
