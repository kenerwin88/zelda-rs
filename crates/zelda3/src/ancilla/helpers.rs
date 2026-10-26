//! Shared ancilla movement, lifting, collision, OAM, and sound helpers.

use super::*;
const ANCILLA_LIFTABLE_THROW_Y_VELOCITIES: [i8; 4] = [-32, 32, 0, 0];
const ANCILLA_LIFTABLE_THROW_X_VELOCITIES: [i8; 4] = [0, 0, -32, 32];
const ANCILLA_SPRITE_COLLISION_DEFLECTED_DIRECTIONS: [u8; 4] = [2, 3, 0, 1];
const ANCILLA_SPRITE_COLLISION_RECOIL_X: [u8; 4] = [0, 0, 0xc0, 0x40];
const ANCILLA_SPRITE_COLLISION_RECOIL_Y: [u8; 4] = [0xc0, 0x40, 0, 0];
use crate::tile_definition::{EntityCollision, NativeTile};
use crate::types::{abs16, project_speed_from_differences, sign8, AncillaRadialProjection, PairU8, Point16U, ProjectSpeedRet, SpriteHitBox};

impl ZeldaState {
    /// ROM: `Ancilla_GetY`
    pub(in crate::zelda_rtl) fn ancilla_get_y(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).y()
    }

    /// ROM: `Ancilla_GetX`
    pub(in crate::zelda_rtl) fn ancilla_get_x(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).x()
    }

    pub(super) fn ancilla_project_reflexive_speed_onto_sprite(
        &mut self,
        k: usize,
        x: u16,
        y: u16,
        vel: u8,
    ) -> ProjectSpeedRet {
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut().set_x(x);
        self.follower_link_state_mut().set_y(y);
        let pt = self.sprite_project_speed_towards_link(k, vel);
        self.follower_link_state_mut().set_x(old_x);
        self.follower_link_state_mut().set_y(old_y);
        pt
    }

    pub(in crate::zelda_rtl) fn ancilla_terminate_select_interactives(&mut self, mut y: u8) -> u8 {
        let _scope = crate::cycle_ledger::routine(0x09_ac6b);
        crate::cycle_ledger::charge(88); // PHB/PHK/PLB; LDX #5.
        for i in (0..=5).rev() {
            if let Some(slot) = self.ancilla_interactive_cleanup_slot_prefix(i) {
                y = slot;
            }
            self.ancilla_interactive_cleanup_slot_finish(i);
            crate::cycle_ledger::charge(if i == 0 { 30 } else { 36 }); // DEX; BPL.
        }
        self.ancilla_interactive_cleanup_finish();
        y
    }

    pub(in crate::zelda_rtl) fn ancilla_interactive_cleanup_before_pickup(&mut self, slot: u8) {
        assert!(slot < 6);
        for i in (usize::from(slot)..=5).rev() {
            self.ancilla_interactive_cleanup_slot_prefix(i);
            if i != usize::from(slot) {
                self.ancilla_interactive_cleanup_slot_finish(i);
            }
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_interactive_cleanup_after_pickup(&mut self, slot: u8) {
        assert!(slot < 6);
        self.ancilla_interactive_cleanup_slot_finish(usize::from(slot));
        self.ancilla_interactive_cleanup_after_current_slot(slot);
    }

    pub(in crate::zelda_rtl) fn ancilla_interactive_cleanup_before_type_clear(&mut self, slot: u8) {
        self.ancilla_interactive_cleanup_before_pickup(slot);
        assert!(self.ancilla_interactive_cleanup_pickup_test(usize::from(slot)));
    }

    pub(in crate::zelda_rtl) fn ancilla_interactive_cleanup_at_type_clear(&mut self, slot: u8) {
        assert!(slot < 6);
        self.ancilla_slot_view_mut(usize::from(slot))
            .set_ancilla_type(0);
        self.ancilla_interactive_cleanup_after_current_slot(slot);
    }

    fn ancilla_interactive_cleanup_after_current_slot(&mut self, slot: u8) {
        for i in (0..usize::from(slot)).rev() {
            self.ancilla_interactive_cleanup_slot_prefix(i);
            self.ancilla_interactive_cleanup_slot_finish(i);
        }
        self.ancilla_interactive_cleanup_finish();
    }

    /// Returns the slot when it holds the interactive that the caller keeps.
    fn ancilla_interactive_cleanup_slot_prefix(&mut self, i: usize) -> Option<u8> {
        crate::cycle_ledger::charge(64); // LDA type; CMP #$3E; BNE.
        if self.ancilla_slot_view(i).ancilla_type() == 0x3e {
            crate::cycle_ledger::charge(14 + 22); // TXY; BRA.
            return Some(i as u8);
        }
        crate::cycle_ledger::charge(6 + 64); // Taken BNE; LDA type; CMP #$2C; BNE.
        if self.ancilla_slot_view(i).ancilla_type() == 0x2c {
            crate::cycle_ledger::charge(88);
            self.dungeon_environment_mut()
                .clear_somaria_block_switch_counter();
            if self.game_state.player.follower_link.defense_flags() & 0x80 != 0 {
                crate::cycle_ledger::charge(48);
                self.follower_link_state_mut().clear_defense_flags();
                self.follower_link_state_mut().set_speed_setting(0);
            } else {
                crate::cycle_ledger::charge(6);
            }
        } else {
            crate::cycle_ledger::charge(6);
        }
        None
    }

    fn ancilla_interactive_cleanup_slot_finish(&mut self, i: usize) {
        if self.ancilla_interactive_cleanup_pickup_test(i) {
            crate::cycle_ledger::charge(38); // STZ type,X.
            self.ancilla_slot_view_mut(i).set_ancilla_type(0);
        }
    }

    fn ancilla_interactive_cleanup_pickup_test(&mut self, i: usize) -> bool {
        crate::cycle_ledger::charge(48 + 76); // LDA state/BPL; TXA/INC/CMP/branch.
        if sign8(self.game_state.player.follower_link.state_bits()) {
            crate::cycle_ledger::charge(if i + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize { 6 } else { 22 });
            i + 1 != self.game_state.player.follower_link.ancilla_pickup_flag() as usize
        } else {
            crate::cycle_ledger::charge(6); // BPL taken.
            if i + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize {
                crate::cycle_ledger::charge(32);
                self.follower_link_state_mut().clear_ancilla_pickup_flag();
            } else {
                crate::cycle_ledger::charge(6);
            }
            true
        }
    }

    fn ancilla_interactive_cleanup_finish(&mut self) {
        crate::cycle_ledger::charge(64); // Position-mode branch.
        if self.game_state.player.follower_link.position_mode_has(0x10) {
            crate::cycle_ledger::charge(56);
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().clear_position_mode();
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(312); // Reset flags and test hookshot handler.
        self.follower_link_state_mut().clear_flute_countdown();
        self.follower_state_mut().clear_event_flags();
        self.follower_link_state_mut()
            .clear_ancilla_interactive_reset_flag();
        self.minigame_state_mut().clear_flag_boomerang_in_place();
        self.minigame_state_mut().clear_is_archer_or_shovel_game();
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.follower_link_state_mut()
            .clear_player_special_draw_flag();
        self.follower_link_state_mut().clear_electrocute_on_touch();
        if self.game_state.player.follower_link.handler_state() == 19 {
            crate::cycle_ledger::charge(280);
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut().clear_position_mode_bits(4);
            self.follower_link_state_mut().clear_hookshot_interlock();
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(72); // PLB; RTL.
    }

    pub(in crate::zelda_rtl) fn ancilla_allocate_oam_from_region_a_or_d_or_f(
        &mut self,
        k: usize,
        size: u8,
    ) -> u16 {
        if self.game_state.oam.has_sprite_sorting() {
            if self.ancilla_slot_view(k).floor() != 0 {
                self.oam_allocate_from_region_f(size)
            } else {
                self.oam_allocate_from_region_d(size)
            }
        } else {
            self.oam_allocate_from_region_a(size)
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_prep_oam_coord(&mut self, k: usize) -> (u16, u16) {
        // Cycle ledger: Ancilla_PrepOamCoord $08:F671 (m8 x8, JSR target):
        // layer bits, four abs,x coordinate loads, two 16-bit scroll
        // subtractions, RTS: 674. The $0BFA,X read crosses a page at slot 6.
        crate::cycle_ledger::charge_routine(0x08_f671, 674 + if k >= 6 { 6 } else { 0 });
        let floor = self.ancilla_slot_view(k).floor() as usize;
        self.oam_state_mut()
            .set_priority_word((ANCILLA_PREP_OAM_COORD_TAGALONG_LAYER_BITS[floor] as u16) << 8);
        (
            self.ancilla_x(k)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
            self.ancilla_y(k)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
        )
    }

    /// ROM: `Ancilla_MoveX`
    pub(in crate::zelda_rtl) fn ancilla_move_x(&mut self, k: usize) {
        // $08:9080 wraps the shared Y-axis body with X+10, then restores
        // the live slot through $08:A752. The low-coordinate read crosses
        // $0BFF for that indexed axis.
        let _scope = crate::cycle_ledger::routine(0x08_9080);
        crate::cycle_ledger::charge(208);
        crate::cycle_ledger::charge_routine(0x08_908b,
            Self::ancilla_axis_move_master_cycles(k + 10, self.ancilla_slot_view(k).x_velocity()));
        self.ancilla_slot_view_mut(k).move_x();
    }

    /// ROM: `Ancilla_MoveY`
    pub(in crate::zelda_rtl) fn ancilla_move_y(&mut self, k: usize) {
        crate::cycle_ledger::charge_routine(0x08_908b,
            Self::ancilla_axis_move_master_cycles(k, self.ancilla_slot_view(k).y_velocity()));
        self.ancilla_slot_view_mut(k).move_y();
    }

    fn ancilla_axis_move_master_cycles(index: usize, velocity: u8) -> u64 {
        // $08:908B-90B6: fractional carry, signed velocity/16, and two
        // coordinate stores. BPL replaces ORA/DEY for a positive velocity;
        // ADC $0BFA,X adds six clocks when the indexed read crosses a page.
        342 + if velocity & 0x80 == 0 { 6 } else { 30 }
            + 196 + if index >= 6 { 6 } else { 0 }
    }

    pub(in crate::zelda_rtl) fn ancilla_move_z(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).move_z();
    }

    pub(in crate::zelda_rtl) fn ancilla_check_link_collision(&self, k: usize, j: usize) -> bool {
        self.ancilla_check_link_collision_out(k, j).is_some()
    }

    fn ancilla_check_link_collision_out(&self, k: usize, j: usize) -> Option<CheckPlayerCollOut> {
        let y = self
            .ancilla_y(k)
            .wrapping_add(ANCILLA_CHECK_LINK_COLLISION_OUT_YOFFS[j])
            .wrapping_add(self.ancilla_slot_view(k).z() as i8 as i16 as u16);
        let x = self
            .ancilla_x(k)
            .wrapping_add(ANCILLA_CHECK_LINK_COLLISION_OUT_XOFFS[j]);
        let r4 = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(ANCILLA_CHECK_LINK_COLLISION_OUT_LINK_YOFFS[j])
            .wrapping_sub(y);
        let r6 = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(ANCILLA_CHECK_LINK_COLLISION_OUT_LINK_XOFFS[j])
            .wrapping_sub(x);
        let r8 = abs16(r4);
        let r10 = abs16(r6);
        if r8 < ANCILLA_CHECK_LINK_COLLISION_OUT_H[j] && r10 < ANCILLA_CHECK_LINK_COLLISION_OUT_W[j]
        {
            Some(CheckPlayerCollOut { r4, r6, r8, r10 })
        } else {
            None
        }
    }

    pub(super) fn ancilla_check_tile_collision(&mut self, k: usize) -> u8 {
        if self.game_state.world.location.is_outdoors()
            && self.ancilla_slot_view(k).object_priority() != 0
        {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_tile_attribute(value);
            return 0;
        }
        if self.game_state.dungeon.room_load.header_collision() == 0 {
            return self.ancilla_check_tile_collision_one_floor(k) as u8;
        }

        let mut x = 0u16;
        let mut y = 0u16;
        if self.game_state.dungeon.room_load.header_collision() < 3 {
            x = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg1_h_copy2()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            y = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg1_v_copy2()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        }
        let oldx = self.ancilla_get_x(k);
        let oldy = self.ancilla_get_y(k);
        self.ancilla_set_xy(k, oldx.wrapping_add(x), oldy.wrapping_add(y));
        let value = 1;
        self.ancilla_slot_view_mut(k).set_floor(value);
        let b = self.ancilla_check_tile_collision_one_floor(k) as u8;
        let value = 0;
        self.ancilla_slot_view_mut(k).set_floor(value);
        self.ancilla_set_xy(k, oldx, oldy);
        (b << 1) | self.ancilla_check_tile_collision_one_floor(k) as u8
    }

    pub(super) fn ancilla_check_tile_collision_staggered(&mut self, k: usize) -> u8 {
        if (self.game_state.frame.frame_counter ^ k as u8) & 1 != 0 {
            self.ancilla_check_tile_collision(k)
        } else {
            0
        }
    }

    fn ancilla_check_tile_collision_one_floor(&mut self, k: usize) -> bool {
        let j = self.ancilla_slot_view(k).direction() as usize;
        let x = self.ancilla_get_x(k).wrapping_add(
            ANCILLA_CHECK_TILE_COLLISION_ONE_FLOOR_CHECK_TILE_COLL0_X[j] as i16 as u16,
        );
        let y = self.ancilla_get_y(k).wrapping_add(
            ANCILLA_CHECK_TILE_COLLISION_ONE_FLOOR_CHECK_TILE_COLL0_Y[j] as i16 as u16,
        );
        self.ancilla_check_tile_collision_targeted(k, x, y)
    }

    pub(super) fn ancilla_check_initial_tile_a(&mut self, k: usize) -> i32 {
        let mut j = self.ancilla_slot_view(k).direction() as usize * 3;
        let mut i = 2i32;
        loop {
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(ANCILLA_CHECK_INITIAL_TILE_A_XOFFS_HB[j] as i16 as u16);
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(ANCILLA_CHECK_INITIAL_TILE_A_YOFFS_HB[j] as i16 as u16);
            self.ancilla_set_xy(k, x, y);
            if self.ancilla_check_tile_collision(k) != 0 {
                break;
            }
            i -= 1;
            if i < 0 {
                break;
            }
            j += 1;
        }
        i
    }

    pub(super) fn ancilla_return_if_outside_bounds(&mut self, k: usize) -> Option<AncillaOamInfo> {
        let info = AncillaOamInfo {
            x: self
                .ancilla_slot_view(k)
                .x_low()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low()),
            y: self
                .ancilla_slot_view(k)
                .y_low()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low()),
            flags: ANCILLA_RETURN_IF_OUTSIDE_BOUNDS_ANCILLA_FLOOR_FLAGS
                [self.ancilla_slot_view(k).floor() as usize],
        };
        if info.x >= 0xf4 || info.y >= 0xf0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            None
        } else {
            Some(info)
        }
    }

    pub(super) fn ancilla_apply_conveyor(&mut self, k: usize) {
        let j = self
            .ancilla_slot_view(k)
            .tile_attribute()
            .wrapping_sub(0x68) as usize;
        let value = ANCILLA_APPLY_CONVEYOR_ANCILLA_BELT_YVEL[j] as u8;
        self.ancilla_slot_view_mut(k).set_y_velocity(value);
        let value = ANCILLA_APPLY_CONVEYOR_ANCILLA_BELT_XVEL[j] as u8;
        self.ancilla_slot_view_mut(k).set_x_velocity(value);
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
    }

    /// ROM: `Ancilla_ProjectSpeedTowardsPlayer`
    pub(in crate::zelda_rtl) fn ancilla_project_speed_towards_player(&self, k: usize, vel: u8) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet::ZERO;
        }
        let below = self.ancilla_is_below_link(k);
        let right = self.ancilla_is_right_of_link(k);
        project_speed_from_differences(vel, below, right)
    }

    pub(super) fn ancilla_get_radial_projection(&self, a: u8, r8: u8) -> AncillaRadialProjection {
        let a = a as usize;
        let p0 = u16::from(ANCILLA_GET_RADIAL_PROJECTION_RADIAL_PROJECTION_PRIMARY_MAGNITUDE[a])
            * u16::from(r8);
        let p1 = u16::from(ANCILLA_GET_RADIAL_PROJECTION_RADIAL_PROJECTION_SECONDARY_MAGNITUDE[a])
            * u16::from(r8);
        AncillaRadialProjection {
            r0: ((p0 >> 8) + ((p0 >> 7) & 1)) as u8,
            r2: ANCILLA_GET_RADIAL_PROJECTION_RADIAL_PROJECTION_PRIMARY_SIGN[a],
            r4: ((p1 >> 8) + ((p1 >> 7) & 1)) as u8,
            r6: ANCILLA_GET_RADIAL_PROJECTION_RADIAL_PROJECTION_SECONDARY_SIGN[a],
        }
    }

    pub(super) fn sparkle_prep_oam_from_radial(&self, p: AncillaRadialProjection) -> Point16U {
        Point16U {
            y: self
                .game_state
                .sprites
                .ether_orbit
                .swordbeam_temp_y()
                .wrapping_add(if p.r2 != 0 {
                    -(p.r0 as i16)
                } else {
                    p.r0 as i16
                } as u16)
                .wrapping_sub(4)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
            x: self
                .game_state
                .sprites
                .ether_orbit
                .swordbeam_temp_x()
                .wrapping_add(if p.r6 != 0 {
                    -(p.r4 as i16)
                } else {
                    p.r4 as i16
                } as u16)
                .wrapping_sub(4)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
        }
    }

    fn ancilla_is_right_of_link(&self, k: usize) -> PairU8 {
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.ancilla_get_x(k));
        PairU8 {
            a: u8::from((x as i16).is_negative()),
            b: x as u8,
        }
    }

    fn ancilla_is_below_link(&self, k: usize) -> PairU8 {
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.ancilla_get_y(k));
        PairU8 {
            a: u8::from((y as i16).is_negative()),
            b: y as u8,
        }
    }

    pub(super) fn ancilla_handle_lift_logic(&mut self, k: usize) {
        if self.ancilla_slot_view(k).r() != 0 {
            self.ancilla_handle_lift_logic_label_6(k);
            return;
        }
        if self.ancilla_slot_view(k).l() == 0 {
            if self.game_state.player.follower_link.ancilla_pickup_flag() == 0 {
                if self.ancilla_handle_lift_logic_clear_pickup_item(
                    k,
                    &ANCILLA_HANDLE_LIFT_LOGIC_ANCILLA_LIFTABLE_DELAY,
                ) {
                    return;
                }
            } else {
                if self.game_state.player.follower_link.ancilla_pickup_flag() != k as u8 + 1 {
                    return;
                }
                if (self
                    .game_state
                    .player
                    .follower_link
                    .sprite_damage_disable_timer()
                    == 0
                    && self.game_state.player.follower_link.incapacitated_timer() != 0)
                    || self
                        .game_state
                        .player
                        .follower_link
                        .player_special_draw_flag()
                        != 0
                    || self
                        .game_state
                        .player
                        .follower_link
                        .is_in_auxiliary_state(1)
                {
                    let value = 1;
                    self.ancilla_slot_view_mut(k).set_r(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                    self.follower_link_state_mut().clear_ancilla_pickup_flag();
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                    self.ancilla_handle_lift_logic_label_6(k);
                    return;
                }
                if !self
                    .game_state
                    .player
                    .follower_link
                    .is_lifting_or_carrying()
                {
                    if self.ancilla_handle_lift_logic_clear_pickup_item(
                        k,
                        &ANCILLA_HANDLE_LIFT_LOGIC_ANCILLA_LIFTABLE_DELAY,
                    ) {
                        return;
                    }
                } else {
                    let mut j = self.ancilla_slot_view(k).k();
                    if !self
                        .game_state
                        .player
                        .follower_link
                        .picking_throw_state_has(2)
                        && self.game_state.player.follower_link.ancilla_pickup_flag() != 0
                        && j != 3
                    {
                        if j == 0 && self.ancilla_slot_view(k).aux_timer() == 16 {
                            self.ancilla_sfx2_pan(k, 0x1d);
                        }
                        let value = self.ancilla_slot_view(k).aux_timer().wrapping_sub(1);
                        self.ancilla_slot_view_mut(k).set_aux_timer(value);
                        if (self.ancilla_slot_view(k).aux_timer() as i8).is_negative() {
                            j = j.wrapping_add(1);
                            let value = j;
                            self.ancilla_slot_view_mut(k).set_k(value);
                            let value = if j == 3 {
                                (-2i8) as u8
                            } else {
                                ANCILLA_HANDLE_LIFT_LOGIC_ANCILLA_LIFTABLE_DELAY[j as usize]
                            };
                            self.ancilla_slot_view_mut(k).set_aux_timer(value);
                            if j == 3 {
                                self.ancilla_latch_altitude_above_link(k);
                                return;
                            }
                        }
                        self.ancilla_latch_link_coordinates(k, j as usize);
                        return;
                    }
                    if j != 3 {
                        return;
                    }

                    if !self
                        .game_state
                        .player
                        .follower_link
                        .picking_throw_state_has(2)
                        && (self.game_state.frame.submodule != 0
                            || ((self.game_state.player.follower_link.filtered_joypad_l()
                                | self.game_state.player.follower_link.filtered_joypad_h())
                                & 0x80)
                                == 0)
                    {
                        if self.ancilla_slot_view(k).item_to_link() != 0 {
                            return;
                        }
                        if self
                            .game_state
                            .player
                            .follower_link
                            .near_pit_state_at_least(2)
                        {
                            self.follower_link_state_mut().set_speed_setting(0);
                            if k + 1
                                == self.game_state.player.follower_link.ancilla_pickup_flag()
                                    as usize
                            {
                                self.follower_link_state_mut().clear_ancilla_pickup_flag();
                                let value = 0;
                                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                            }
                            return;
                        }
                        if !self.game_state.player.follower_link.is_in_deep_water()
                            && !self.game_state.player.follower_link.is_bunny_mirror()
                        {
                            self.ancilla_latch_carried_position(k);
                            return;
                        }
                        self.follower_link_state_mut().clear_state_bits();
                    }
                    let j = self.game_state.player.follower_link.facing_index();
                    let value = j as u8;
                    self.ancilla_slot_view_mut(k).set_direction(value);
                    {
                        let mut liftable = self.ancilla_slot_view_mut(k);
                        liftable.set_z_velocity(24);
                        liftable.set_y_velocity(ANCILLA_LIFTABLE_THROW_Y_VELOCITIES[j] as u8);
                        liftable.set_x_velocity(ANCILLA_LIFTABLE_THROW_X_VELOCITIES[j] as u8);
                    }
                    self.follower_link_state_mut().set_picking_throw_state(2);
                    let value = 1;
                    self.ancilla_slot_view_mut(k).set_l(value);
                    self.follower_link_state_mut().clear_ancilla_pickup_flag();
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_k(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_object_priority(value);
                    self.ancilla_sfx3_pan(k, 0x13);
                }
            }
        }

        if self.ancilla_slot_view(k).item_to_link() == 0 {
            self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            let old_z = self.ancilla_slot_view(k).z();
            self.ancilla_move_z(k);
            let z = self.ancilla_slot_view(k).z();
            if self.ancilla_slot_view(k).work_byte_4() != 0
                && self.ancilla_slot_view(k).direction() == 1
                && !(z as i8).is_negative()
            {
                self.ancilla_set_y(
                    k,
                    self.ancilla_get_y(k)
                        .wrapping_add(z.wrapping_sub(old_z) as i8 as i16 as u16),
                );
            }
            if !(z as i8).is_negative() || z == 0xff {
                return;
            }
            self.ancilla_slot_view_mut(k).set_z(0);
            self.ancilla_sfx2_pan(k, 0x21);
            self.ancilla_slot_view_mut(k).add_l(1);
            if self.ancilla_slot_view(k).l() != 3 {
                let mut liftable = self.ancilla_slot_view_mut(k);
                let y_velocity = ((liftable.y_velocity() as i8) / 2) as u8;
                let x_velocity = ((liftable.x_velocity() as i8) / 2) as u8;
                liftable.set_y_velocity(y_velocity);
                liftable.set_x_velocity(x_velocity);
                liftable.set_z_velocity(16);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_work_byte_4(value);
            } else {
                self.ancilla_slot_view_mut(k).set_z(0);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_l(value);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                self.follower_link_state_mut().set_speed_setting(0);
                {
                    let mut liftable = self.ancilla_slot_view_mut(k);
                    liftable.set_y_velocity(0);
                    liftable.set_x_velocity(0);
                    liftable.set_z_velocity(0);
                }
                if self.ancilla_slot_view(k).t_player() != 0 {
                    let value = self.ancilla_slot_view(k).t_player();
                    self.ancilla_slot_view_mut(k).set_floor(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_t_player(value);
                }
            }
        }
    }

    fn ancilla_handle_lift_logic_clear_pickup_item(
        &mut self,
        k: usize,
        liftable_delay: &[u8; 3],
    ) -> bool {
        self.follower_link_state_mut().clear_ancilla_pickup_flag();
        if self.ancilla_slot_view(k).item_to_link() != 0
            || self.game_state.player.follower_link.has_action_state()
        {
            return true;
        }
        let Some(coll) = self.ancilla_check_link_collision_out(k, 0) else {
            return true;
        };
        if self.ancilla_slot_view(k).floor()
            != self.game_state.player.follower_link.lower_level_state()
        {
            return true;
        }
        if coll.r8 >= 16 || coll.r10 >= 12 {
            let j = if coll.r8 >= coll.r10 {
                if (coll.r4 as i16).is_negative() {
                    1
                } else {
                    0
                }
            } else if (coll.r6 as i16).is_negative() {
                3
            } else {
                2
            };
            if j * 2 != self.game_state.player.follower_link.facing() {
                return true;
            }
        }
        self.follower_link_state_mut()
            .set_ancilla_pickup_flag(k as u8 + 1);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        let value = liftable_delay[0];
        self.ancilla_slot_view_mut(k).set_aux_timer(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        self.ancilla_slot_view_mut(k).set_z(0);
        true
    }

    fn ancilla_handle_lift_logic_label_6(&mut self, k: usize) {
        if self.ancilla_slot_view(k).item_to_link() != 0 {
            return;
        }
        if self.ancilla_slot_view(k).k() == 3 {
            self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
            self.ancilla_move_z(k);
            if self.ancilla_slot_view(k).z() != 0 && self.ancilla_slot_view(k).z() < 252 {
                return;
            }
            let value = 0;
            self.ancilla_slot_view_mut(k).set_z(value);
            self.ancilla_slot_view_mut(k).add_r(1);
            if self.ancilla_slot_view(k).r() != 3 {
                let value = 24;
                self.ancilla_slot_view_mut(k).set_z_velocity(value);
                return;
            }
            let value = 0;
            self.ancilla_slot_view_mut(k).set_k(value);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        self.follower_link_state_mut().set_speed_setting(0);
    }

    pub(super) fn ancilla_latch_altitude_above_link(&mut self, k: usize) {
        let value = 17;
        self.ancilla_slot_view_mut(k).set_z(value);
        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_add(17));
        let value = 0;
        self.ancilla_slot_view_mut(k).set_object_priority(value);
    }

    pub(super) fn ancilla_latch_link_coordinates(&mut self, k: usize, mut j: usize) {
        j = j * 4 + self.game_state.player.follower_link.facing_index();
        self.ancilla_set_xy(
            k,
            self.game_state
                .player
                .follower_link
                .x()
                .wrapping_add(ANCILLA_LATCH_LINK_COORDINATES_ANCILLA_FUNC3_X[j] as i16 as u16),
            self.game_state
                .player
                .follower_link
                .y()
                .wrapping_add(ANCILLA_LATCH_LINK_COORDINATES_ANCILLA_FUNC3_Y[j] as i16 as u16),
        );
    }

    pub(super) fn ancilla_latch_carried_position(&mut self, k: usize) {
        self.follower_link_state_mut().set_speed_setting(12);
        let value = self.game_state.player.follower_link.lower_level_state();
        self.ancilla_slot_view_mut(k).set_floor(value);
        let value = self
            .game_state
            .player
            .follower_link
            .lower_level_mirror_state();
        self.ancilla_slot_view_mut(k).set_floor2(value);
        let mut z = self.game_state.player.follower_link.z();
        if z == 0xffff {
            z = 0;
        }
        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x().wrapping_add(8),
            self.game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(z)
                .wrapping_add(18)
                .wrapping_add(
                    ANCILLA_LATCH_CARRIED_POSITION_ANCILLA_FUNC2_Y
                        [self.game_state.player.follower_link.animation_step_index()]
                        as i16 as u16,
                ),
        );
    }

    pub(super) fn ancilla_latch_y_coord_to_z(&mut self, k: usize) -> u16 {
        let y = self.ancilla_get_y(k);
        let z = self.ancilla_slot_view(k).z();
        if self.ancilla_slot_view(k).direction() == 1 && z != 0xff {
            self.ancilla_set_y(k, y.wrapping_sub(z as i8 as i16 as u16));
        }
        y
    }

    pub(in crate::zelda_rtl) fn ancilla_check_tile_collision_class2(&mut self, k: usize) -> bool {
        if self.game_state.dungeon.room_load.header_collision() == 0 {
            return self.ancilla_check_tile_collision_class2_inner(k);
        }

        let mut x = 0u16;
        let mut y = 0u16;
        if self.game_state.dungeon.room_load.header_collision() < 3 {
            x = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg1_h_copy2()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            y = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg1_v_copy2()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        }

        let oldx = self.ancilla_x(k);
        let oldy = self.ancilla_y(k);
        self.ancilla_set_xy(k, oldx.wrapping_add(x), oldy.wrapping_add(y));
        let value = 1;
        self.ancilla_slot_view_mut(k).set_floor(value);
        let b = self.ancilla_check_tile_collision_class2_inner(k);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_floor(value);
        self.ancilla_set_xy(k, oldx, oldy);
        b | self.ancilla_check_tile_collision_class2_inner(k)
    }

    fn ancilla_check_tile_collision_class2_inner(&mut self, k: usize) -> bool {
        let dir = self.ancilla_slot_view(k).direction() as usize;
        let x = self
            .ancilla_x(k)
            .wrapping_add(ANCILLA_CHECK_TILE_COLLISION_CLASS2_INNER_X_OFFSETS[dir] as i16 as u16);
        let y = self
            .ancilla_y(k)
            .wrapping_add(ANCILLA_CHECK_TILE_COLLISION_CLASS2_INNER_Y_OFFSETS[dir] as i16 as u16);

        if y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()) >= 224
            || x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()) >= 256
        {
            return false;
        }

        let (tile, x) = self.ancilla_probe_tile(k, x, y);
        // Wall identity 3 never stops an ancilla travelling on the upper layer.
        if tile == NativeTile::from_cartridge(3) && self.ancilla_slot_view(k).floor2() != 0 {
            return false;
        }

        match tile.ancilla_collision() {
            EntityCollision::Passable => false,
            EntityCollision::Solid => true,
            EntityCollision::Slope => self.ancilla_sloped_tile_collision(x, y),
            EntityCollision::LayerBoundary => self.ancilla_slot_view(k).floor2() != 0,
            EntityCollision::Ledge => {
                if self.ancilla_slot_view(k).floor2() != 0 {
                    true
                } else {
                    let value = 1;
                    self.ancilla_slot_view_mut(k).set_object_priority(value);
                    false
                }
            }
        }
    }

    pub(super) fn ancilla_check_initial_tile_collision_class2(&mut self, k: usize) -> bool {
        let mut j = self.ancilla_slot_view(k).direction() as usize * 2;
        for _ in (0..=2).rev() {
            self.ancilla_set_xy(
                k,
                self.game_state.player.follower_link.x().wrapping_add(
                    ANCILLA_CHECK_INITIAL_TILE_COLLISION_CLASS2_INITIAL_TILE_COLL_X[j] as u16,
                ),
                self.game_state.player.follower_link.y().wrapping_add(
                    ANCILLA_CHECK_INITIAL_TILE_COLLISION_CLASS2_INITIAL_TILE_COLL_Y[j] as u16,
                ),
            );
            if self.ancilla_check_tile_collision_class2(k) {
                return true;
            }
            j += 1;
        }
        false
    }

    pub(super) fn ancilla_check_tile_collision_targeted(&mut self, k: usize, x: u16, y: u16) -> bool {
        if y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()) >= 224
            || x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()) >= 256
        {
            return false;
        }
        let (tile, x) = self.ancilla_probe_tile(k, x, y);
        // Wall identity 3 never stops an ancilla travelling on the upper layer.
        if tile == NativeTile::from_cartridge(3) && self.ancilla_slot_view(k).floor2() != 0 {
            return false;
        }

        let mut collision = tile.ancilla_ground_layer_collision();
        // The boomerang passes the interactable range 0xc0..=0xcf.
        if self.ancilla_slot_view(k).ancilla_type() == 2
            && tile.cartridge_attribute() & 0xf0 == 0xc0
        {
            collision = EntityCollision::Passable;
        }

        if self.ancilla_slot_view(k).object_priority() == 0 {
            match collision {
                EntityCollision::Passable => return false,
                EntityCollision::Solid => {
                    self.sprite_system_mut().set_alert_flag(3);
                    return true;
                }
                EntityCollision::Slope => return self.ancilla_sloped_tile_collision(x, y),
                EntityCollision::LayerBoundary => {
                    if self.ancilla_slot_view(k).floor2() != 0 {
                        self.sprite_system_mut().set_alert_flag(3);
                        return true;
                    }
                    return false;
                }
                EntityCollision::Ledge => {}
            }
        }
        self.ancilla_slot_view_mut(k).subtract_u(1);
        if (self.ancilla_slot_view(k).u() as i8) < 0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_u(value);
            if collision == EntityCollision::Ledge {
                let value = 6;
                self.ancilla_slot_view_mut(k).set_u(value);
                self.ancilla_slot_view_mut(k).xor_object_priority(1);
            }
        }
        false
    }

    pub(super) fn ancilla_terminate_if_offscreen(&mut self, j: usize) {
        let xt: u16 = if self.game_state.enhanced_features.has(1) {
            0x40
        } else {
            0
        };
        let x = self
            .ancilla_get_x(j)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
            .wrapping_add(xt);
        let y = self
            .ancilla_get_y(j)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x >= 244 + xt * 2 || y >= 240 {
            let value = 0;
            self.ancilla_slot_view_mut(j).set_ancilla_type(value);
        }
    }

    pub(super) fn ancilla_setup_basic_hit_box(&self, k: usize) -> SpriteHitBox {
        let x = self.ancilla_get_x(k).wrapping_sub(8);
        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(8)
            .wrapping_sub(self.ancilla_slot_view(k).z() as u16);
        SpriteHitBox {
            r0_xlo: x as u8,
            r8_xhi: (x >> 8) as u8,
            r1_ylo: y as u8,
            r9_yhi: (y >> 8) as u8,
            r2: 15,
            r3: 15,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        }
    }

    fn ancilla_setup_hit_box(&self, k: usize) -> SpriteHitBox {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x08_8ead));
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        // $08:8EAD..8EEB, M8/X8: full hitbox setup; only $0BFA,X
        // crosses a page for the ten ancilla slots. Tables stay in-page.
        blocks.block(0x088ead);
        let mut j = self.ancilla_slot_view(k).direction() as usize;
        if self.ancilla_slot_view(k).ancilla_type() == 0x0c {
            blocks.block(0x088eba);
            j |= 8;
        } else {
            blocks.taken(0x088eb8);
        }
        blocks.block(0x088ec0);
        if k >= 6 { blocks.crossing(0x088ed0); }
        let x = self
            .ancilla_get_x(k)
            .wrapping_add(ANCILLA_SETUP_HIT_BOX_ANCILLA_HIT_BOX_X[j] as i16 as u16);
        let y = self
            .ancilla_get_y(k)
            .wrapping_add(ANCILLA_SETUP_HIT_BOX_ANCILLA_HIT_BOX_Y[j] as i16 as u16);
        SpriteHitBox {
            r0_xlo: x as u8,
            r8_xhi: (x >> 8) as u8,
            r1_ylo: y as u8,
            r9_yhi: (y >> 8) as u8,
            r2: ANCILLA_SETUP_HIT_BOX_ANCILLA_HIT_BOX_W[j],
            r3: ANCILLA_SETUP_HIT_BOX_ANCILLA_HIT_BOX_H[j],
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        }
    }

    pub(super) fn ancilla_prep_adjusted_oam_coord(&mut self, k: usize) -> (u16, u16) {
        let _scope = crate::cycle_ledger::routine(0x08_f6a4);
        // F6A4..F6D8 is straight-line priority setup, coordinate loads,
        // scroll subtraction and RTS (m8/x8, with the REP/SEP pair).
        // LDA $0BFA,X crosses its page for slots 6..9; the other slot
        // arrays and the four floor priorities do not cross a page.
        crate::cycle_ledger::charge(690 + if k >= 6 { 6 } else { 0 });
        let floor = self.ancilla_slot_view(k).floor() as usize;
        self.oam_state_mut().set_priority_word(
            (ANCILLA_PREP_ADJUSTED_OAM_COORD_TAGALONG_LAYER_BITS[floor] as u16) << 8,
        );
        (
            self.ancilla_get_x(k)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy()),
            self.ancilla_get_y(k)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy()),
        )
    }

    pub(super) fn ancilla_allocate_oam_from_region_b_or_e(&mut self, size: u8) {
        if !self.game_state.oam.has_sprite_sorting() {
            self.oam_allocate_from_region_b(size);
        } else {
            self.oam_allocate_from_region_e(size);
        }
    }

    pub(super) fn ancilla_allocate_oam_from_custom_region(&mut self, oam: usize) -> usize {
        let mut a = oam;
        if self.game_state.oam.has_sprite_sorting() {
            if a < 0x900 {
                if a < 0x8e0 {
                    return oam;
                }
                a = 0x820;
            } else {
                if a < 0x9d0 {
                    return oam;
                }
                a = 0x940;
            }
        } else {
            if a < 0x990 {
                return oam;
            }
            a = 0x820;
        }
        self.oam_state_mut().set_current_pointer(a as u16);
        self.oam_state_mut()
            .set_current_extended_pointer((((a - 0x800) >> 2) + 0xa20) as u16);
        self.game_state.oam.current_pointer_usize()
    }

    pub(super) fn hit_stars_update_oam_buffer_position(&mut self, oam: usize) -> usize {
        let mut oam = oam;
        if !self.game_state.oam.has_sprite_sorting() && oam >= 0x9d0 {
            self.oam_state_mut().set_current_pointer(0x820);
            self.oam_state_mut()
                .set_current_extended_pointer(0xa20 + (0x20 >> 2));
            oam = self.game_state.oam.current_pointer_usize();
        }
        oam
    }

    pub(super) fn ancilla_draw_shadow(&mut self, oam: usize, k: usize, mut x: u16, y: u16, pal: u8) {
        if k == 2 {
            x = x.wrapping_add(4);
        }
        self.ancilla_set_oam_safe(
            oam,
            x,
            y,
            ANCILLA_DRAW_SHADOW_ANCILLA_DRAW_SHADOW_CHAR[k * 2],
            ANCILLA_DRAW_SHADOW_ANCILLA_DRAW_SHADOW_FLAGS[k * 2] & !0x30 | pal,
            0,
        );
        let ch = ANCILLA_DRAW_SHADOW_ANCILLA_DRAW_SHADOW_CHAR[k * 2 + 1];
        if ch != 0xff {
            x = x.wrapping_add(8);
            self.ancilla_set_oam_safe(
                oam + 4,
                x,
                y,
                ch,
                ANCILLA_DRAW_SHADOW_ANCILLA_DRAW_SHADOW_FLAGS[k * 2 + 1] & !0x30 | pal,
                0,
            );
        }
    }

    pub(super) fn ancilla_check_damage_to_sprite(&mut self, k: usize, ty: u8) {
        if !sign8(self.sprite_slot_view(k).hit_timer()) {
            self.ancilla_check_damage_to_sprite_aggressive(k, ty);
        }
    }

    pub(super) fn ancilla_check_damage_to_sprite_aggressive(&mut self, k: usize, ty: u8) {
        let mut dmg = ANCILLA_CHECK_DAMAGE_TO_SPRITE_AGGRESSIVE_ANCILLA_DAMAGE[ty as usize];
        if dmg == 6 && self.game_state.inventory.items.has_upgraded_bow() {
            if self.sprite_slot_view(k).sprite_type() == 0xd7 {
                let value = 32;
                self.sprite_slot_view_mut(k).set_delay_aux4(value);
            }
            dmg = 9;
        }
        self.ancilla_check_damage_to_sprite_preset(k, dmg);
    }

    pub(super) fn ancilla_check_sprite_collision(&mut self, k: usize) -> Option<usize> {
        (0..16).rev().find(|&j| {
            (self.ancilla_slot_view(k).ancilla_type() == 9
                || self.ancilla_slot_view(k).ancilla_type() == 0x1f
                || (((j as u8 ^ self.game_state.frame.frame_counter) & 3)
                    | self.sprite_slot_view(j).pause())
                    == 0)
                && self.sprite_slot_view(j).state() >= 9
                && (self.sprite_slot_view(j).deflection_bits() & 2 != 0
                    || self.ancilla_slot_view(k).object_priority() == 0)
                && self.ancilla_slot_view(k).floor() == self.sprite_slot_view(j).floor()
                && self.ancilla_check_sprite_collision_single(k, j)
        })
    }

    fn ancilla_check_sprite_collision_single(&mut self, k: usize, j: usize) -> bool {
        let mut hb = self.ancilla_setup_hit_box(k);
        self.sprite_setup_hit_box(j, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if !overlap {
            return false;
        }

        let mut return_value = true;
        if self.sprite_slot_view(j).flags() & 8 != 0
            && self.ancilla_slot_view(k).ancilla_type() == 9
        {
            if self.sprite_slot_view(j).sprite_type() != 0x1b {
                self.sprite_create_deflected_arrow(k);
                return false;
            }
            if !self.game_state.inventory.items.has_upgraded_bow() {
                self.sprite_create_deflected_arrow(k);
            } else {
                return_value = false;
            }
        }

        let mut return_true_set_alert = false;
        if self.sprite_slot_view(j).deflection_bits() & 0x10 != 0 {
            self.ancilla_slot_view_mut(k).and_direction(3);
            if self.ancilla_slot_view(k).direction()
                == ANCILLA_SPRITE_COLLISION_DEFLECTED_DIRECTIONS
                    [self.ancilla_slot_view(k).direction() as usize]
            {
                return_true_set_alert = true;
            }
        }

        if !return_true_set_alert
            && (self.ancilla_slot_view(k).ancilla_type() == 5
                || self.ancilla_slot_view(k).ancilla_type() == 0x1f)
        {
            let skip = self.ancilla_slot_view(k).ancilla_type() == 0x1f
                && self.sprite_slot_view(j).sprite_type() == 0x8d;
            if !skip && self.sprite_slot_view(j).hit_timer() != 0 {
                return_true_set_alert = true;
            } else if skip || self.sprite_slot_view(j).deflection_bits() & 2 != 0 {
                let value = k as u8 + 1;
                self.sprite_slot_view_mut(j).set_b(value);
                let value = self.ancilla_slot_view(k).ancilla_type();
                self.sprite_slot_view_mut(j).set_draw_work_byte_2(value);
                return_true_set_alert = true;
            }
        }

        if !return_true_set_alert && self.sprite_slot_view(j).ignore_projectile() == 0 {
            if self.sprite_slot_view(j).sprite_type() == 0x92 && self.sprite_slot_view(j).c() < 3 {
                return_true_set_alert = true;
            } else {
                let i = (self.ancilla_slot_view(k).direction() & 3) as usize;
                let value = ANCILLA_SPRITE_COLLISION_RECOIL_X[i];
                self.sprite_slot_view_mut(j).set_x_recoil(value);
                let value = ANCILLA_SPRITE_COLLISION_RECOIL_Y[i];
                self.sprite_slot_view_mut(j).set_y_recoil(value);
                self.sprite_workspace_mut().set_shared_scratch_a(k as u8);
                self.ancilla_check_damage_to_sprite(j, self.ancilla_slot_view(k).ancilla_type());
                return_true_set_alert = true;
            }
        } else if !return_true_set_alert {
            return false;
        }

        if return_true_set_alert {
            let value = self.ancilla_slot_view(k).ancilla_type();
            self.sprite_slot_view_mut(j).set_draw_work_byte_2(value);
            self.sprite_system_mut().set_alert_flag(3);
            return return_value;
        }
        false
    }

    pub(super) fn ancilla_check_basic_sprite_collision(&mut self, k: usize) -> Option<usize> {
        for j in (0..16).rev() {
            if (((j as u8 ^ self.game_state.frame.frame_counter) & 3)
                | self.sprite_slot_view(j).pause()
                | self.sprite_slot_view(j).hit_timer())
                != 0
            {
                continue;
            }
            if self.sprite_slot_view(j).state() < 9
                || (self.sprite_slot_view(j).deflection_bits() & 2 == 0
                    && self.ancilla_slot_view(k).object_priority() != 0)
                || self.ancilla_slot_view(k).floor() != self.sprite_slot_view(j).floor()
                || self.ancilla_slot_view(k).ancilla_type() == 0x2c
                    && (self.sprite_slot_view(j).sprite_type() == 0x1e
                        || self.sprite_slot_view(j).sprite_type() == 0x90)
            {
                continue;
            }
            if self.ancilla_check_basic_sprite_collision_single(k, j) {
                return Some(j);
            }
        }
        None
    }

    fn ancilla_check_basic_sprite_collision_single(&mut self, k: usize, j: usize) -> bool {
        let mut hb = self.ancilla_setup_basic_hit_box(k);
        self.sprite_setup_hit_box(j, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if !overlap {
            return false;
        }
        if self.sprite_slot_view(j).sprite_type() == 0x92 && self.sprite_slot_view(j).c() < 3 {
            return true;
        }
        if self.sprite_slot_view(j).sprite_type() == 0x80
            && self.sprite_slot_view(j).delay_aux4() == 0
        {
            let value = 24;
            self.sprite_slot_view_mut(j).set_delay_aux4(value);
            self.sprite_slot_view_mut(j).xor_direction(1);
        }
        if self.sprite_slot_view(j).ignore_projectile() != 0 {
            return false;
        }

        let x = self.ancilla_get_x(k).wrapping_sub(8);
        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(8)
            .wrapping_sub(self.ancilla_slot_view(k).z() as u16);
        let pt = self.sprite_project_speed_towards_location(j, x, y, 80);
        let value = !pt.y;
        self.sprite_slot_view_mut(j).set_y_recoil(value);
        let value = !pt.x;
        self.sprite_slot_view_mut(j).set_x_recoil(value);
        self.ancilla_check_damage_to_sprite(j, self.ancilla_slot_view(k).ancilla_type());
        true
    }

    pub(crate) fn ancilla_calculate_sfx_pan(&self, k: usize) -> u8 {
        Self::calculate_sfx_pan_with_scroll(
            self.ancilla_get_x(k),
            self.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
        )
    }

    /// Probe the tile ahead of an ancilla and record it on the slot. The
    /// original published the sprite scratch tile only indoors, so the
    /// outdoor slope test still reads whatever the scratch last held.
    fn ancilla_probe_tile(&mut self, k: usize, x: u16, y: u16) -> (NativeTile, u16) {
        let floor = self.ancilla_slot_view(k).floor();
        let (tile, x) = if self.game_state.world.location.is_indoors() {
            // $08:8A59 / $08:8C9B JSL $06:E87B. Both indoor
            // paths call the same complete lookup; neither caller prices it.
            self.probe_entity_tile_long(floor, x, y)
        } else {
            self.entity_tile_at(floor, x, y)
        };
        self.ancilla_slot_view_mut(k)
            .set_tile_attribute(tile.cartridge_attribute());
        (tile, x)
    }

    fn ancilla_sloped_tile_collision(&self, x: u16, y: u16) -> bool {
        self.entity_sloped_tile_collision(x, y).unwrap_or(true)
    }

    /// ROM SetOam_XY stops before character, flags and extended publication.
    pub(super) fn ancilla_set_oam_xy(&mut self, oam: usize, x: u16, y: u16) -> u8 {
        let _scope = crate::cycle_ledger::routine(0x08_f6e1);
        crate::cycle_ledger::charge(78 + if y >= 256 {
            6
        } else {
            40 + if x >= 256 { 6 } else { 126 + if y < 0xf0 { 14 } else { 6 } }
        } + 158);
        self.ancilla_write_oam_xy(oam, x, y)
    }

    // Shared semantic writer; other merged draw helpers still need their
    // source call boundaries mapped before charging SetOam_XY at those sites.
    fn ancilla_write_oam_xy(&mut self, oam: usize, x: u16, y: u16) -> u8 {
        let mut yval = 0xf0;
        let mut x_bit = 0;
        let xt: u16 = if self.game_state.enhanced_features.has(1) {
            0x40
        } else {
            0
        };
        if x.wrapping_add(xt) < 256 + xt * 2 && y < 256 {
            x_bit = ((x >> 8) as u8) & 1;
            self.oam_state_mut().set_entry_x(oam, x as u8);
            if y < 0xf0 {
                yval = y as u8;
            }
        }
        self.oam_state_mut().set_entry_y(oam, yval);
        x_bit
    }

    pub(super) fn ancilla_set_oam(&mut self, oam: usize, x: u16, y: u16, charnum: u8, flags: u8, big: u8) {
        let big = big | self.ancilla_write_oam_xy(oam, x, y);
        self.oam_state_mut().set_entry_char(oam, charnum);
        self.oam_state_mut().set_entry_flags(oam, flags);
        let value = big;
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
    }

    pub(super) fn ancilla_set_oam_plain(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut()
            .write_entry(oam, x as u8, y as u8, charnum, flags);
        let value = big;
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
    }

    pub(super) fn ancilla_set_oam_safe(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        mut big: u8,
    ) {
        // $08:F702..F72E owns coordinates only, before char/flags/ext stores.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x08_f702));
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        blocks.block(0x08f702);
        if x.wrapping_add(0x80) >= 0x180 {
            blocks.taken(0x08f710);
            blocks.block(0x08f726);
        } else {
            blocks.block(0x08f712);
            if y.wrapping_add(0x10) < 0x100 { blocks.taken(0x08f724); }
            else { blocks.block(0x08f726); }
        }
        blocks.block(0x08f72b);
        let mut yval = 0xf0;
        self.oam_state_mut().set_entry_x(oam, x as u8);
        let xt: u16 = if self.game_state.enhanced_features.has(1) {
            0x48
        } else {
            0
        };
        if x.wrapping_add(0x80) < 0x180 + xt {
            big |= ((x >> 8) as u8) & 1;
            if y.wrapping_add(0x10) < 0x100 {
                yval = y as u8;
            }
        }
        self.oam_state_mut().set_entry_y(oam, yval);
        self.oam_state_mut().set_entry_char(oam, charnum);
        self.oam_state_mut().set_entry_flags(oam, flags);
        let value = big;
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
    }

    pub(in crate::zelda_rtl) fn ancilla_sfx2_pan(&mut self, k: usize, sfx: u8) {
        self.set_raw_sfx_pan_value(sfx);
        let out = sfx | self.ancilla_calculate_sfx_pan(k);
        self.set_sound_effect_1(out);
    }

    pub(in crate::zelda_rtl) fn ancilla_sfx3_pan(&mut self, k: usize, sfx: u8) {
        self.set_raw_sfx_pan_value(sfx);
        let out = sfx | self.ancilla_calculate_sfx_pan(k);
        self.set_sound_effect_2(out);
    }

    /// ROM: `Ancilla_SetXY`
    pub(in crate::zelda_rtl) fn ancilla_set_xy(&mut self, k: usize, x: u16, y: u16) {
        self.ancilla_set_x(k, x);
        self.ancilla_set_y(k, y);
    }

    pub(super) fn ancilla_set_x(&mut self, k: usize, x: u16) {
        self.ancilla_slot_view_mut(k).set_x(x);
    }

    pub(super) fn ancilla_set_y(&mut self, k: usize, y: u16) {
        self.ancilla_slot_view_mut(k).set_y(y);
    }

}

#[cfg(test)]
mod marker31_tests {

#[test]
fn marker31_ancilla_hitbox_matches_rom() {
    for k in 0..10 {
        for kind in [1u8, 12] {
            for direction in 0..4 {
                let mut native = super::super::tests::marker29_state();
                native.write_wram_byte(0xc4a + k, kind);
                native.write_wram_byte(0xc72 + k, direction);
                native.sync_native_game_state_from_ram();
                let Some(machine) = super::super::tests::marker29_machine(&native, 0x088ead, k) else { return };
                crate::cycle_ledger::assert_rom_marker_work(0x088ead, machine,
                    |s| s.cpu.k == 8 && (0x8ead..0x8eec).contains(&s.cpu.pc), || {
                        native.ancilla_setup_hit_box(k);
                    });
            }
        }
    }
}
}
