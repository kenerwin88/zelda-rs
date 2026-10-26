//! Environmental effects, debris, splashes, duck travel, and cutscenes.

use super::*;
use crate::types::{abs16, abs8, sign16, sign8};

impl ZeldaState {
    pub(super) fn ancilla06_wall_hit(&mut self, k: usize) {
        if sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
            // C: uint8 t = ancilla_item_to_link[k] + 1; if (t == 5) { free; return; }
            // The early return means item_to_link is NOT written when t==5. Peek first.
            let t = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
            if t == 5 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            self.ancilla_slot_view_mut(k).advance_item_to_link();
            self.ancilla_slot_view_mut(k).set_work_byte_3(1);
        }
        self.wall_hit_draw(k);
    }

    pub(super) fn ancilla_sword_wall_hit(&mut self, k: usize) {
        self.sprite_system_mut().set_alert_flag(3);
        if sign8(self.ancilla_slot_view_mut(k).tick_aux_timer()) {
            let t = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
            if t == 8 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            self.ancilla_slot_view_mut(k).advance_item_to_link();
            self.ancilla_slot_view_mut(k).set_aux_timer(1);
        }
        self.wall_hit_draw(k);
    }

    pub(super) fn ancilla1_d_screen_shake(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            let item_to_link = self.ancilla_slot_view(k).item_to_link().wrapping_sub(1);
            self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
            if sign8(item_to_link) {
                self.set_bg1_x_offset(0);
                self.set_bg1_y_offset(0);
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            let offs = self.dash_tremor_twiddle_offset(k);
            let j = self.ancilla_slot_view(k).direction();
            if j == 0 {
                self.set_bg1_x_offset(offs as u16);
                self.follower_link_state_mut()
                    .add_movement_velocity_delta(offs as u16, 0);
            } else {
                self.set_bg1_y_offset(offs as u16);
                self.follower_link_state_mut()
                    .add_movement_velocity_delta(0, offs as u16);
            }
        }
        self.sprite_system_mut().set_alert_flag(3);
    }

    pub(super) fn ancilla1_e_dash_dust(&mut self, k: usize) {
        // $08:BC92..BCC3: control and prep call, inside $08:833C.
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        blocks.block(0x08bc92);
        if self.ancilla_slot_view(k).step() != 0 {
            blocks.block(0x08bc97);
            self.dash_dust_motive(k);
            blocks.block(0x08bc9b);
            blocks.block(0x08bcb9);
            return;
        }
        blocks.taken(0x08bc95);
        blocks.block(0x08bc9d);
        if self.ancilla_slot_view(k).timer() == 0 {
            blocks.block(0x08bca2);
            self.ancilla_slot_view_mut(k).set_timer(3);
            let item_to_link = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if item_to_link == 5 {
                blocks.taken(0x08bcb0);
                blocks.block(0x08bcb9);
                return;
            }
            blocks.block(0x08bcb2);
            if item_to_link == 6 {
                blocks.block(0x08bcb6);
                self.ancilla_slot_view_mut(k).clear();
                blocks.block(0x08bcb9);
                return;
            }
            blocks.taken(0x08bcb4);
        } else { blocks.taken(0x08bca0); }
        blocks.block(0x08bcba);
        if self.ancilla_slot_view(k).item_to_link() == 5 {
            blocks.taken(0x08bcbf);
            blocks.block(0x08bcb9);
            return;
        }

        blocks.block(0x08bcc1);
        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        // $08:BCC4..BD4B: complete three-entry dust drawing loop.
        blocks.block(0x08bcc4);
        let mut oam = self.game_state.oam.current_pointer_usize();

        let r12 = ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_X1
            [self.game_state.player.follower_link.facing_index()] as i16;
        if self.game_state.player.follower_link.water_ripple_or_grass_state() == 1 {
            blocks.block(0x08bcea);
        } else { blocks.taken(0x08bce8); }
        blocks.block(0x08bcec);
        let mut t = 3
            * (self.ancilla_slot_view(k).item_to_link() as usize
                + if self
                    .game_state
                    .player
                    .follower_link
                    .water_ripple_or_grass_state()
                    == 1
                {
                    5
                } else {
                    0
                });

        for remaining in (0..=2).rev() {
            blocks.block(0x08bd02);
            if ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_CHAR[t] != 0xff {
                blocks.block(0x08bd0b);
                if 0xf4 + 2 * t > 0xff { blocks.crossing(0x08bd13); }
                // $08:BD25 JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    info_x
                        .wrapping_add(r12 as u16)
                        .wrapping_add(ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_X[t] as u16),
                    info_y.wrapping_add(ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_Y[t] as u16),
                    ANCILLA1_E_DASH_DUST_DASH_DUST_DRAW_CHAR[t],
                    4 | self.game_state.oam.priority_high(),
                    0,
                );
                blocks.block(0x08bd28);
                oam += 4;
            } else { blocks.taken(0x08bd09); }
            blocks.block(0x08bd44);
            if remaining != 0 { blocks.taken(0x08bd48); }
            t += 1;
        }
        blocks.block(0x08bd4a);
    }

    pub(super) fn dash_dust_motive(&mut self, k: usize) {
        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).set_timer(3);
            let item_to_link = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        if self.game_state.player.follower_link.facing() == 2 {
            self.oam_allocate_from_region_b(4);
        }
        let frame = self.ancilla_slot_view(k).item_to_link() as usize;
        if frame >= DASH_DUST_MOTIVE_MOTIVE_DASH_DUST_DRAW_CHAR.len() {
            self.ancilla_slot_view_mut(k).clear();
            return;
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        self.ancilla_set_oam(
            self.game_state.oam.current_pointer_usize(),
            x,
            y,
            DASH_DUST_MOTIVE_MOTIVE_DASH_DUST_DRAW_CHAR[frame],
            4 | self.game_state.oam.priority_high(),
            0,
        );
    }

    fn wall_hit_draw(&mut self, k: usize) {
        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        let mut t = self.ancilla_slot_view(k).item_to_link() as usize * 4;

        let mut oam = self.game_state.oam.current_pointer_usize();
        for _ in (0..=3).rev() {
            if WALL_HIT_DRAW_WALL_HIT_CHAR[t] != 0 {
                // $08:9519 JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    info_x.wrapping_add(WALL_HIT_DRAW_WALL_HIT_X[t] as i16 as u16),
                    info_y.wrapping_add(WALL_HIT_DRAW_WALL_HIT_Y[t] as i16 as u16),
                    WALL_HIT_DRAW_WALL_HIT_CHAR[t],
                    (WALL_HIT_DRAW_WALL_HIT_FLAGS[t] & !0x30) | self.game_state.oam.priority_high(),
                    0,
                );
                oam += 4;
            }
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            t += 1;
        }
    }

    pub(super) fn ancilla08_door_debris(&mut self, k: usize) {
        self.door_debris_draw(k);
        let work_byte_26 = self.ancilla_slot_view(k).work_byte_26().wrapping_sub(1);
        self.ancilla_slot_view_mut(k).set_work_byte_26(work_byte_26);
        if sign8(work_byte_26) {
            let mut debris = self.ancilla_slot_view_mut(k);
            debris.set_work_byte_26(7);
            let frame = debris.advance_work_byte_25();
            if frame == 4 {
                debris.clear();
            }
        }
    }

    fn door_debris_draw(&mut self, k: usize) {
        self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let y = self
            .door_debris_y_word(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let x = self
            .door_debris_x_word(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let j = self.ancilla_slot_view(k).work_byte_25() as usize
            + self.door_debris_direction(k) as usize * 4;

        for i in 0..2 {
            let t = j * 2 + i;
            let offset = DOOR_DEBRIS_DRAW_DOOR_DEBRIS_OFFSET[t];
            let tile = DOOR_DEBRIS_DRAW_DOOR_DEBRIS_TILE[t];
            // $08:A0F1 JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x.wrapping_add(offset.x),
                y.wrapping_add(offset.y),
                tile.char,
                (tile.flags & 0xc0) | self.game_state.oam.priority_high(),
                0,
            );
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    pub(super) fn ancilla20_blanket(&mut self, k: usize) {
        // Cycle ledger: Ancilla20_Blanket $08:C013 (m8 x8), entered by
        // Ancilla_ExecuteOne's JMP (abs), so it charges into that scope.
        // $08:C013-C028 JSR Ancilla_PrepOamCoord, REP LDA STA LDA STA STA SEP
        // PHX, LDA $037d : BNE (320); pose 0 runs $08:C02A LDA #$10 JSL
        // RegionB BRA (100), else the BNE is taken (+6) into $08:C032 LDA
        // #$10 JSL RegionA (78). Then $08:C038-C03B LDA $037d : BEQ (48),
        // taken (+6) for pose 0, else $08:C03D LDA #$04 (16); $08:C03F-C044
        // TAX LDA #$03 STA $0a LDY #$00 (70).
        let (mut x, mut y) = self.ancilla_prep_oam_coord(k);

        if self.game_state.player.follower_link.opening_pose() == 0 {
            crate::cycle_ledger::charge(320 + 100 + 48 + 6 + 70);
            self.oam_allocate_from_region_b(0x10);
        } else {
            crate::cycle_ledger::charge(320 + 6 + 78 + 48 + 16 + 70);
            self.oam_allocate_from_region_a(0x10);
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut j = if self.game_state.player.follower_link.opening_pose() != 0 {
            4
        } else {
            0
        };
        for i in (0..=3).rev() {
            // $08:C046-C075 per sprite (682, JSR Ancilla_SetOam_XY included)
            // plus Ancilla_SetOam_XY $08:F6E1 itself (a JSR target the C
            // inlined into Ancilla_SetOam): PHX LDX #$f0 LDA $01 BNE (78);
            // a high Y byte takes the BNE (+6) straight to $08:F6F7-F6FD INY
            // TXA STA INY PLX RTS (158); else LDA $03 BNE (40) does the same
            // for a high X byte; else $08:F6EC-F6F4 LDA STA LDA CMP #$f0 BCS
            // (126), taken (+6) for Y >= $f0, else $08:F6F6 TAX (14).
            crate::cycle_ledger::charge(682);
            {
                let _set_oam_xy = crate::cycle_ledger::routine(0x08_f6e1);
                crate::cycle_ledger::charge(if y >= 256 {
                    78 + 6 + 158
                } else if x >= 256 {
                    78 + 40 + 6 + 158
                } else if y >= 0xf0 {
                    78 + 40 + 126 + 6 + 158
                } else {
                    78 + 40 + 126 + 14 + 158
                });
            }
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ANCILLA20_BLANKET_BEDSPREAD_CHAR[j],
                ANCILLA20_BLANKET_BEDSPREAD_FLAGS[j] | 0x0d | self.game_state.oam.priority_high(),
                2,
            );
            // After DEC $0a : BMI (in the 682): counts 2 and 0 take the
            // $08:C077-C07B LDA CMP #$01 BNE (56 + 6) back to the loop, count
            // 1 falls into $08:C07D-C08D (232, BRA included), and the final
            // negative count takes the BMI (+6) into $08:C08F PLX : RTS (70).
            crate::cycle_ledger::charge(match i {
                3 | 1 => 56 + 6,
                2 => 56 + 232,
                _ => 6 + 70,
            });
            x = x.wrapping_add(16);
            if i == 2 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(8);
            }
            j += 1;
            oam += 4;
        }
    }

    pub(super) fn ancilla21_snore(&mut self, k: usize) {
        // $08:C094 is an Ancilla_ExecuteOne jump target, not a new call.
        crate::cycle_ledger::charge(68);
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = self.ancilla_slot_view(k).item_to_link();
            crate::cycle_ledger::charge(78 + if item_to_link == 2 { 6 } else { 38 } + 54);
            let mut snore = self.ancilla_slot_view_mut(k);
            if item_to_link != 2 {
                snore.advance_item_to_link();
            }
            snore.set_aux_timer(7);
        } else {
            crate::cycle_ledger::charge(6);
        }

        let step = self.ancilla_slot_view(k).step();
        let x_velocity = self.ancilla_slot_view_mut(k).add_x_velocity(step);
        crate::cycle_ledger::charge(132 + if sign8(x_velocity) { 30 } else { 6 } + 32);
        if abs8(x_velocity) >= 8 {
            crate::cycle_ledger::charge(100);
            self.ancilla_slot_view_mut(k)
                .set_step((-(step as i8)) as u8);
        } else {
            crate::cycle_ledger::charge(6);
        }

        crate::cycle_ledger::charge(92); // The two movement JSRs.
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
        crate::cycle_ledger::charge(344 - 92 + if k >= 6 { 6 } else { 0 });
        if self.ancilla_y(k) <= self.game_state.player.follower_link.y().wrapping_sub(24) {
            crate::cycle_ledger::charge(60);
            self.ancilla_slot_view_mut(k).clear();
        } else {
            crate::cycle_ledger::charge(6);
        }

        crate::cycle_ledger::charge(462); // DMA selector, both draw calls, stores, RTS.
        let dma_staging_index =
            ANCILLA21_SNORE_BEDSPREAD_DMA[self.ancilla_slot_view(k).item_to_link() as usize];
        self.follower_link_state_mut()
            .set_link_dma_staging_index(dma_staging_index);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let oam = self.game_state.oam.current_pointer_usize();
        let ext = self.ancilla_set_oam_xy(oam, x, y);
        let mut oam_state = self.oam_state_mut();
        oam_state.set_entry_char(oam, 9);
        oam_state.set_entry_flags(oam, 0x24);
        oam_state.set_extended_byte((oam - OAM_BUF) / 4, ext);
    }

    pub(super) fn ancilla23_link_poof(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = {
                let mut poof = self.ancilla_slot_view_mut(k);
                poof.set_aux_timer(7);
                poof.advance_item_to_link()
            };
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
                self.follower_link_state_mut().clear_transforming();
                self.follower_link_state_mut().clear_direction_lock();
                if self.ancilla_slot_view(k).step() == 0 {
                    self.follower_link_state_mut().clear_animation_step();
                    self.follower_link_state_mut().set_visibility_status(0);
                    let bunny =
                        if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
                            1
                        } else {
                            0
                        };
                    self.follower_link_state_mut().set_bunny_state(bunny);
                    if self.game_state.player.follower_link.is_bunny() {
                        self.load_gear_palettes_bunny();
                    } else {
                        self.LoadActualGearPalettes();
                    }
                }
                return;
            }
        }
        self.morph_poof_draw(k);
    }

    pub(super) fn ancilla24_gravestone(&mut self, k: usize) {
        let (mut x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.oam_allocate_from_region_b(16);
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in 0..4 {
            // $08:EE16 JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x,
                y,
                ANCILLA24_GRAVESTONE_ANCILLA_GRAVESTONE_CHAR[i],
                ANCILLA24_GRAVESTONE_ANCILLA_GRAVESTONE_FLAGS[i] | 0x3d,
                2,
            );
            x = x.wrapping_add(16);
            if i == 1 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(8);
            }
            oam += 4;
        }
    }

    pub(super) fn ancilla34_skull_woods_fire(&mut self, k: usize) {
        if self.skull_woods_fire_has_started_entrance_opening()
            && self.ancilla_slot_view(k).item_to_link() != 4
            && sign8(self.ancilla_slot_view_mut(k).tick_aux_timer())
        {
            let mut fire = self.ancilla_slot_view_mut(k);
            fire.set_aux_timer(5);
            fire.advance_item_to_link();
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..=3).rev() {
            let timer = self.skull_woods_fire_mut(i).tick_timer();
            if sign8(timer) {
                self.skull_woods_fire_mut(i).set_timer(5);
                if self
                    .game_state
                    .effects
                    .entrance_effects
                    .skull_woods_fire_slot(i)
                    .phase()
                    != 128
                {
                    let phase = self.skull_woods_fire_mut(i).advance_phase();
                    if phase == 0 || phase == 4 {
                        self.skull_woods_fire_mut(i).set_phase(0);
                        let inner_y = self.skull_woods_fire_scratch_mut().retreat_inner_y(8);
                        if inner_y < 200 && !self.skull_woods_fire_has_started_entrance_opening() {
                            self.skull_woods_fire_scratch_mut()
                                .set_entrance_opening_started();
                            let pan = (0x98u16
                                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                                as u8)
                                >> 5;
                            self.set_sound_effect_1(BOMBOS_PANNED_SFX_BITS[pan as usize] | 0x0c);
                        }
                        if inner_y < 168 {
                            self.skull_woods_fire_mut(i).set_phase(128);
                        }
                        let inner_x = self.skull_woods_fire_inner_x();
                        self.skull_woods_fire_mut(i).set_position(inner_x, inner_y);
                        if self.game_state.system_signals.sound_effect_1() == 0 {
                            let pan = (self
                                .skull_woods_fire_inner_x()
                                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                                as u8)
                                >> 5;
                            self.set_sound_effect_1(BOMBOS_PANNED_SFX_BITS[pan as usize] | 0x2a);
                        }
                    }
                }
            }

            if !self
                .game_state
                .effects
                .entrance_effects
                .skull_woods_fire_slot(i)
                .is_finished()
            {
                let j = self
                    .game_state
                    .effects
                    .entrance_effects
                    .skull_woods_fire_slot(i)
                    .phase() as usize;
                let x = self
                    .game_state
                    .effects
                    .entrance_effects
                    .skull_woods_fire_slot(i)
                    .x()
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
                let y = self
                    .game_state
                    .effects
                    .entrance_effects
                    .skull_woods_fire_slot(i)
                    .y()
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                    .wrapping_add(
                        ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_Y[j] as i16 as u16,
                    );
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_CHAR[j],
                    0x32,
                    ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_EXT[j],
                );
                oam += 4;
                if ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_EXT[j] != 2 {
                    self.ancilla_set_oam(
                        oam,
                        x.wrapping_add(8),
                        y,
                        ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_CHAR[j].wrapping_add(1),
                        0x32,
                        ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW_EXT[j],
                    );
                    oam += 4;
                }
            }
        }

        let mut i = 3i32;
        while self
            .game_state
            .effects
            .entrance_effects
            .skull_woods_fire_slot(i as usize)
            .is_finished()
        {
            i -= 1;
            if i < 0 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }

        let item_to_link = self.ancilla_slot_view(k).item_to_link();
        if !self.skull_woods_fire_has_started_entrance_opening() || item_to_link == 4 {
            return;
        }

        let mut j = item_to_link as usize * 6;
        for _ in 0..6 {
            if ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    168u16
                        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                        .wrapping_add(
                            ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_X[j] as i16 as u16,
                        ),
                    200u16
                        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                        .wrapping_add(
                            ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_Y[j] as i16 as u16,
                        ),
                    ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_CHAR[j],
                    ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_FLAGS[j] | 0x32,
                    ANCILLA34_SKULL_WOODS_FIRE_SKULL_WOODS_FIRE_DRAW2_EXT[j],
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn morph_poof_draw(&mut self, k: usize) {
        if self.game_state.oam.has_sprite_sorting()
            && self.ancilla_slot_view(k).floor() != 0
            && (self.game_state.minigame.flag_boomerang_in_place() == 0
                || self.game_state.frame.frame_counter & 1 == 0)
        {
            self.oam_state_mut().set_current_pointer(0x08d0);
            self.oam_state_mut()
                .set_current_extended_pointer(0x0a20 + (0x0d0 >> 2));
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let j = self.ancilla_slot_view(k).item_to_link() as usize;
        let ext = MORPH_POOF_DRAW_MORPH_POOF_EXT[j];
        let chr = MORPH_POOF_DRAW_MORPH_POOF_CHAR[j];
        for i in 0..4 {
            let offset = MORPH_POOF_DRAW_MORPH_POOF_OFFSET[j * 4 + i];
            // $08:D46A JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x.wrapping_add(offset.x as i16 as u16),
                y.wrapping_add(offset.y as i16 as u16),
                chr,
                MORPH_POOF_DRAW_MORPH_POOF_FLAGS[j * 4 + i]
                    | 4
                    | self.game_state.oam.priority_high(),
                ext,
            );
            if ext == 2 {
                break;
            }
            oam += 4;
        }
    }

    pub(super) fn ancilla40_dwarf_poof(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = {
                let mut poof = self.ancilla_slot_view_mut(k);
                poof.set_aux_timer(7);
                poof.advance_item_to_link()
            };
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
                self.follower_state_mut().set_appearance_none_flag(0);
                return;
            }
        }
        self.morph_poof_draw(k);
    }

    pub(super) fn ancilla3_f_bush_poof(&mut self, k: usize) {
        if self.ancilla_slot_view(k).timer() == 0 {
            let item_to_link = {
                let mut poof = self.ancilla_slot_view_mut(k);
                poof.set_timer(7);
                poof.advance_item_to_link()
            };
            if item_to_link == 4 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        self.oam_allocate_from_region_c(0x10);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();

        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 4;
        for _ in 0..4 {
            // $08:D56A JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x.wrapping_add(ANCILLA3_F_BUSH_POOF_BUSH_POOF_DRAW_X[j] as i16 as u16),
                y.wrapping_add(ANCILLA3_F_BUSH_POOF_BUSH_POOF_DRAW_Y[j] as i16 as u16),
                ANCILLA3_F_BUSH_POOF_BUSH_POOF_DRAW_CHAR[j],
                ANCILLA3_F_BUSH_POOF_BUSH_POOF_DRAW_FLAGS[j]
                    | 4
                    | self.game_state.oam.priority_high(),
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    pub(super) fn ancilla37_weathervane_explosion(&mut self, k: usize) {
        if self.tick_weather_vane_countdown() != 0 {
            return;
        }
        self.set_weather_vane_countdown(1);
        if self.weather_vane_music_latch() == 0 {
            self.set_weather_vane_music_latch(1);
            self.set_music_control(0xf3);
        }
        if self.ancilla_slot_view_mut(k).tick_g() != 0 {
            return;
        }
        self.ancilla_slot_view_mut(k).set_g(1);
        if self.ancilla_slot_view(k).work_byte_3() == 0 {
            self.ancilla_slot_view_mut(k).advance_work_byte_3();
            self.ancilla_sfx2_near(0x0c);
        }
        if self.ancilla_slot_view(k).step() == 0 {
            let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(aux_timer) {
                self.ancilla_slot_view_mut(k).set_step(1);
                self.overworld_alter_weathervane();
                self.ancilla_add_cutscene_duck(0x38, 0);
            }
        }
        self.set_weather_vane_source_slot(k as u8);
        self.reset_weather_vane_oam_offset();
        for i in (0..=11).rev() {
            if self
                .game_state
                .effects
                .weather_vane_debris
                .debris(i)
                .is_finished()
            {
                continue;
            }
            let draw_state = self.weather_vane_debris_mut(i).tick_animation();
            let z_velocity = self.weather_vane_debris_mut(i).tick_z_velocity();

            let mut debris = self
                .game_state
                .effects
                .weather_vane_debris
                .debris(i)
                .snapshot();
            debris.z_velocity = z_velocity;
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_item_to_link(draw_state);
                ancilla.set_y(debris.y);
                ancilla.set_x(debris.x);
                ancilla.set_z(debris.z);
                ancilla.set_y_velocity(debris.y_velocity);
                ancilla.set_x_velocity(debris.x_velocity);
                ancilla.set_z_velocity(debris.z_velocity);
            }

            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            self.ancilla_move_z(k);

            let landed_z = self.ancilla_slot_view(k).z();
            self.ancilla_draw_weathervane_explosion_wood_debris(k);
            self.weather_vane_debris_mut(i)
                .mark_finished_if_landed(landed_z);
            let ancilla = self.ancilla_slot_view(k);
            let debris_y = ancilla.y();
            let debris_x = ancilla.x();
            let debris_z = ancilla.z();
            self.weather_vane_debris_mut(i)
                .save_position(debris_x, debris_y, debris_z);
        }
        for i in (0..=11).rev() {
            if !self
                .game_state
                .effects
                .weather_vane_debris
                .debris(i)
                .is_finished()
            {
                return;
            }
        }
        self.ancilla_slot_view_mut(k).clear();
    }

    pub(super) fn ancilla27_duck(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            if self.ancilla_slot_view(k).timer() != 0 {
                let xt: u16 = if self.game_state.enhanced_features.has(1) {
                    0x40
                } else {
                    0
                };
                self.ancilla_set_xy(
                    k,
                    self.game_state
                        .display
                        .ppu_scroll_copy
                        .bg2_h_copy2()
                        .wrapping_sub(16)
                        .wrapping_sub(xt),
                    self.game_state.player.follower_link.y().wrapping_sub(8),
                );
                return;
            }

            self.ancilla_slot_view_mut(k).subtract_g(1);
            if sign8(self.ancilla_slot_view(k).g()) {
                let value = 0x28;
                self.ancilla_slot_view_mut(k).set_g(value);
                self.ancilla_sfx3_pan(k, 0x1e);
            }

            if self.ancilla_slot_view(k).l() != 0 || self.ancilla_slot_view(k).step() != 0 {
                if self.ancilla_slot_view(k).l() == 0 && self.ancilla_slot_view(k).step() != 0 {
                    self.increment_modal_pause_flag();
                }
                self.ancilla_slot_view_mut(k).tick_z_velocity();
                self.ancilla_move_z(k);
            }
            self.ancilla_move_x(k);

            if self.ancilla_slot_view(k).l() != 0 {
                let x = self.ancilla_get_x(k);
                if self.ancilla_slot_view(k).step() != 0 {
                    self.increment_modal_pause_flag();
                }
                if !sign16(x) && x >= self.game_state.player.follower_link.x() {
                    if self.ancilla_slot_view(k).step() != 0 {
                        let value = 0;
                        self.ancilla_slot_view_mut(k).set_step(value);
                        self.follower_link_state_mut().set_visibility_status(0);
                        self.follower_state_mut().set_appearance_none_flag(0);
                        self.follower_link_state_mut().clear_item_hold_pose();
                        let value = 0;
                        self.ancilla_slot_view_mut(k).set_y_velocity(value);
                        self.follower_link_state_mut().clear_immobilized();
                        self.follower_link_state_mut()
                            .clear_sprite_damage_disable_timer();
                        self.follower_link_state_mut()
                            .clear_player_special_draw_flag();
                        self.follower_link_state_mut().set_blink_countdown(144);
                        if !((self.game_state.sprites.follower_runtime.indicator() == 12
                            || self.game_state.sprites.follower_runtime.indicator() == 13)
                            && self.game_state.sprites.follower_runtime.dropped() != 0)
                        {
                            self.follower_initialize();
                        }
                    }
                } else if self.game_state.player.follower_link.x().wrapping_sub(x) < 48 {
                    self.draw_duck(k, 3);
                    return;
                }
            } else if self.ancilla_check_link_collision(k, 1)
                && self.game_state.frame.main_module != 15
            {
                if self.game_state.world.location.is_outdoors() {
                    if self.game_state.player.follower_link.handler_state() == 8
                        || self.game_state.player.follower_link.handler_state() == 9
                        || self.game_state.player.follower_link.handler_state() == 10
                        || self.game_state.player.follower_link.near_pit_state_is(2)
                        || (self.game_state.player.follower_link.item_hold_pose()
                            | self.game_state.player.follower_link.hookshot_interlock()
                            | self
                                .game_state
                                .player
                                .follower_link
                                .force_hold_sword_up_state()
                            | self
                                .game_state
                                .player
                                .follower_link
                                .sprite_damage_disable_timer())
                            != 0
                        || self
                            .game_state
                            .player
                            .follower_link
                            .is_lifting_or_carrying()
                    {
                        self.draw_duck_default(k);
                        return;
                    }
                    for i in (0..5).rev() {
                        let a = self.ancilla_slot_view(i).ancilla_type();
                        if a == 0x2a || a == 0x1f || a == 0x30 || a == 0x31 || a == 0x41 {
                            let value = 0;
                            self.ancilla_slot_view_mut(i).set_ancilla_type(value);
                        }
                    }
                    if self.game_state.sprites.follower_runtime.indicator() == 9 {
                        self.follower_state_mut().set_indicator(0);
                        self.follower_state_mut().set_appearance_none_flag(0);
                    }
                }
                {
                    self.follower_link_state_mut().clear_state_bits();
                    self.follower_link_state_mut().clear_picking_throw_state();
                }

                self.set_bg1_x_offset(0);
                self.set_bg1_y_offset(0);
                self.link_reset_properties_a();
                self.follower_link_state_mut().clear_deep_water_state();
                self.follower_link_state_mut()
                    .clear_pull_for_rupees_sprite_need();
                self.follower_link_state_mut().set_visibility_status(12);
                self.follower_link_state_mut().clear_handler_state();
                self.follower_link_state_mut().set_item_hold_pose(1);
                self.follower_link_state_mut().immobilize();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_state_mut().set_appearance_none_flag(1);
                let value = 2;
                self.ancilla_slot_view_mut(k).set_step(value);
                self.increment_modal_pause_flag();
                self.follower_link_state_mut().clear_given_damage();
                if self.game_state.world.location.is_indoors() {
                    let value = self.game_state.world.location.indoor_flag();
                    self.follower_link_state_mut()
                        .set_player_special_draw_flag(value);
                }
            }
        }
        self.draw_duck_default(k);
    }

    fn draw_duck_default(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).tick_work_byte_3();
        if sign8(self.ancilla_slot_view(k).work_byte_3()) {
            let value = 3;
            self.ancilla_slot_view_mut(k).set_work_byte_3(value);
            self.ancilla_slot_view_mut(k).add_k(1);
            if self.ancilla_slot_view(k).k() == 3 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_k(value);
            }
        }
        self.draw_duck(k, self.ancilla_slot_view(k).k());
    }

    fn draw_duck(&mut self, k: usize, j: u8) {
        self.set_flag_travel_bird(TRAVEL_BIRD_DMA_TILE_OFFSETS[j as usize]);

        let (x, y) = self.ancilla_prep_oam_coord(k);

        let mut oam = self.game_state.oam.current_pointer_usize();
        let z = if self.ancilla_slot_view(k).z() != 0 {
            self.ancilla_slot_view(k).z() as i8 as i16 as u16
        } else {
            0
        };
        let n = self.ancilla_slot_view(k).step() as usize + 1;
        for i in 0..n {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(TRAVEL_BIRD_DRAW_X_OFFSETS[i] as i16 as u16),
                y.wrapping_add(z)
                    .wrapping_add(TRAVEL_BIRD_DRAW_Y_OFFSETS[i] as i16 as u16),
                TRAVEL_BIRD_DRAW_CHARS[i],
                TRAVEL_BIRD_DRAW_FLAGS[i] | 0x30,
                2,
            );
            oam += 4;
        }

        self.ancilla_draw_shadow(oam, 1, x, y.wrapping_add(28), 0x30);
        oam += 8;
        if self.ancilla_slot_view(k).step() != 0 {
            self.ancilla_draw_shadow(oam, 1, x.wrapping_sub(7), y.wrapping_add(28), 0x30);
        }

        if !sign16(x) && x >= 0x0130 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            if self.ancilla_slot_view(k).l() == 0 && self.ancilla_slot_view(k).step() != 0 {
                let main_module = self.game_state.frame.main_module;
                self.set_submodule(10);
                self.set_saved_module_for_menu(main_module);
                self.set_main_module(14);
            }
        }
    }

    fn ancilla_draw_weathervane_explosion_wood_debris(&mut self, k: usize) {
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let y = y.wrapping_sub(self.ancilla_slot_view(k).z() as i8 as i16 as u16);
        let i = self.ancilla_slot_view(k).item_to_link();
        if sign8(i) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize()
            + ((self.game_state.world.overworld.weather_vane.oam_offset >> 2) as usize) * 4;
        // $08:D1B3 JSR F6E1: complete XY body, once per executed draw.
        self.ancilla_set_oam_marked(
            oam,
            x,
            y,
            ANCILLA_DRAW_WEATHERVANE_EXPLOSION_WOOD_DEBRIS_WEATHERVANE_EXPLODE_CHAR[i as usize],
            0x3c,
            0,
        );
        self.advance_weather_vane_oam_offset(4);
    }

    pub(super) fn ancilla38_cutscene_duck(&mut self, k: usize) {
        if self.game_state.frame.frame_counter & 31 == 0 {
            self.ancilla_sfx3_pan(k, 0x1e);
        }

        if sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
            let mut duck = self.ancilla_slot_view_mut(k);
            duck.set_work_byte_3(3);
            duck.toggle_k_bit0();
        }

        if self.ancilla_slot_view_mut(k).tick_aux_timer() == 0 {
            self.ancilla_slot_view_mut(k).set_aux_timer(1);
            if self.ancilla_slot_view(k).l() == 0 {
                let item_to_link = self.ancilla_slot_view_mut(k).retreat_item_to_link();
                if !sign8(item_to_link) {
                    let z_delta = if self.ancilla_slot_view(k).step() != 0 {
                        1
                    } else {
                        (-1i8) as u8
                    };
                    self.ancilla_slot_view_mut(k).add_z_velocity(z_delta);
                    if abs8(self.ancilla_slot_view(k).z_velocity()) >= 12 {
                        let step = self.ancilla_slot_view(k).step() ^ 1;
                        self.ancilla_slot_view_mut(k).set_step(step);
                    }
                    self.ancilla38_cutscene_duck_after_stuff(k);
                    return;
                }
                let mut duck = self.ancilla_slot_view_mut(k);
                duck.set_item_to_link(0);
                duck.set_step(0);
                duck.set_x_velocity(ANCILLA38_CUTSCENE_DUCK_TRAVEL_BIRD_INTRO_X_SPEED_LIMITS[0]);
                duck.set_z_velocity((-16i8) as u8);
                duck.advance_l();
                duck.set_step(3);
            }
            let x_delta = if self.ancilla_slot_view(k).step() & 1 == 0 {
                1
            } else {
                (-1i8) as u8
            };
            let x_velocity = self.ancilla_slot_view_mut(k).add_x_velocity(x_delta);
            let absx = abs8(x_velocity);
            if absx == 0 {
                let l = self.ancilla_slot_view_mut(k).advance_l();
                if l == 7 {
                    self.ancilla_slot_view_mut(k).set_s_player(1);
                }
            }
            if absx
                >= ANCILLA38_CUTSCENE_DUCK_TRAVEL_BIRD_INTRO_X_SPEED_LIMITS
                    [self.ancilla_slot_view(k).s_player() as usize]
            {
                let step = self.ancilla_slot_view(k).step() ^ 3;
                self.ancilla_slot_view_mut(k).set_step(step);
            }
            let direction = if sign8(self.ancilla_slot_view(k).x_velocity()) {
                2
            } else {
                3
            };
            self.ancilla_slot_view_mut(k).set_direction(direction);
            let t = ANCILLA38_CUTSCENE_DUCK_TRAVEL_BIRD_INTRO_X_SPEED_LIMITS
                [self.ancilla_slot_view(k).s_player() as usize]
                .wrapping_sub(absx)
                >> 1;
            let z_velocity = if self.ancilla_slot_view(k).step() & 2 != 0 {
                0u8.wrapping_sub(t)
            } else {
                t
            };
            self.ancilla_slot_view_mut(k).set_z_velocity(z_velocity);
        }
        self.ancilla38_cutscene_duck_after_stuff(k);
    }

    fn ancilla38_cutscene_duck_after_stuff(&mut self, k: usize) {
        self.ancilla_move_x(k);
        self.ancilla_move_z(k);
        let value = TRAVEL_BIRD_DMA_TILE_OFFSETS[self.ancilla_slot_view(k).k() as usize + 1];
        self.set_travel_bird_tile_offset(value);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let oam = self.game_state.oam.current_pointer_usize();
        self.ancilla_set_oam(
            oam,
            x.wrapping_add(TRAVEL_BIRD_DRAW_X_OFFSETS[0] as i16 as u16),
            y.wrapping_add(self.ancilla_slot_view(k).z() as i8 as i16 as u16)
                .wrapping_add(TRAVEL_BIRD_DRAW_Y_OFFSETS[0] as i16 as u16),
            TRAVEL_BIRD_DRAW_CHARS[0],
            TRAVEL_BIRD_DRAW_FLAGS[0]
                | 0x30
                | ANCILLA38_CUTSCENE_DUCK_AFTER_STUFF_TRAVEL_BIRD_INTRO_FLAGS_BY_DIRECTION
                    [(self.ancilla_slot_view(k).direction() & 1) as usize],
            2,
        );
        self.ancilla_draw_shadow(oam + 4, 1, x, y.wrapping_add(48), 0x30);
        if !sign16(x) && x >= 248 {
            self.ancilla_slot_view_mut(k).clear();
            self.set_submodule(0);
            self.inventory_items_mut().set_flute(3);
        }
    }

    pub(super) fn ancilla16_hit_stars(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).tick_work_byte_3();
        if !sign8(self.ancilla_slot_view(k).work_byte_3()) {
            return;
        }

        let value = 0;

        self.ancilla_slot_view_mut(k).set_work_byte_3(value);
        if self.game_state.frame.submodule == 0 {
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(self.ancilla_slot_view(k).aux_timer()) {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                let value = 1;
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
            }
            if self.ancilla_slot_view(k).item_to_link() != 0 {
                self.ancilla_slot_view_mut(k).subtract_y_velocity(4);
                let value = self.ancilla_slot_view(k).y_velocity();
                self.ancilla_slot_view_mut(k).set_x_velocity(value);
                if self.ancilla_slot_view(k).y_velocity() < 232 {
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                    return;
                }
                self.ancilla_move_y(k);
                self.ancilla_move_x(k);
            }
        }
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let ax = self.ancilla_get_x(k);
        let tt = u16::from(self.ancilla_slot_view(k).a())
            | (u16::from(self.ancilla_slot_view(k).b()) << 8);
        let r8 = tt
            .wrapping_mul(2)
            .wrapping_sub(ax)
            .wrapping_sub(8)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());

        if self.ancilla_slot_view(k).step() == 2 {
            self.ancilla_allocate_oam_from_region_b_or_e(8);
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut x = x;
        let mut flags = 0;
        for _ in (0..=1).rev() {
            // $08:A967 JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x,
                y,
                ANCILLA16_HIT_STARS_ANCILLA_HIT_STARS_CHAR
                    [self.ancilla_slot_view(k).item_to_link() as usize],
                self.game_state.oam.priority_high() | 4 | flags,
                0,
            );
            flags = 0x40;
            x = (x & 0xff00) | (r8 & 0x00ff);
            oam = self.hit_stars_update_oam_buffer_position(oam + 4);
        }
    }

    pub(super) fn ancilla17_shovel_dirt(&mut self, k: usize) {
        let (mut x, mut y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        if self.ancilla_slot_view(k).timer() == 0 {
            let value = 8;
            self.ancilla_slot_view_mut(k).set_timer(value);
            self.ancilla_slot_view_mut(k).add_item_to_link(1);
            if self.ancilla_slot_view(k).item_to_link() == 2 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }
        }
        let b = self.ancilla_slot_view(k).item_to_link() as usize;
        let j = b + if self.game_state.player.follower_link.facing() == 4 {
            0
        } else {
            2
        };
        x = x.wrapping_add(ANCILLA17_SHOVEL_DIRT_SHOVEL_DIRT_XY[j * 2 + 1] as i16 as u16);
        y = y.wrapping_add(ANCILLA17_SHOVEL_DIRT_SHOVEL_DIRT_XY[j * 2] as i16 as u16);
        for i in 0..2 {
            // $08:A9FB JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x.wrapping_add((i * 8) as u16),
                y,
                ANCILLA17_SHOVEL_DIRT_SHOVEL_DIRT_CHAR[b].wrapping_add(i as u8),
                4 | self.game_state.oam.priority_high(),
                0,
            );
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    pub(super) fn ancilla41_waterfall_splash(&mut self, k: usize) {
        if !self.ancilla_check_for_entrance_trigger(
            if self.game_state.world.location.is_indoors() {
                0
            } else {
                1
            },
        ) {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            return;
        }

        if self.game_state.frame.submodule == 0 && self.game_state.frame.frame_counter & 7 == 0 {
            self.ancilla_sfx2_near(0x1c);
        }

        self.follower_link_state_mut()
            .set_water_ripple_or_grass_state(1);
        self.follower_link_state_mut()
            .subtract_animation_step_if_at_least(6, 6);

        if self.ancilla_slot_view(k).timer() == 0 {
            let mut splash = self.ancilla_slot_view_mut(k);
            splash.set_timer(2);
            let item_to_link = splash.advance_item_to_link();
            splash.set_item_to_link(item_to_link & 3);
        }

        if self.game_state.world.location.is_indoors()
            && self.game_state.player.follower_link.y_low() < 0x38
        {
            self.ancilla_set_y(k, 0x0d38);
        } else {
            self.ancilla_set_y(k, self.game_state.player.follower_link.y());
        }
        self.ancilla_set_x(k, self.game_state.player.follower_link.x());

        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let z = self.game_state.player.follower_link.z() as u8;
        y = y.wrapping_sub(if sign8(z) { 0 } else { z } as u16);

        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 2;
        for _ in 0..2 {
            if ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_CHAR[j] != 0xff {
                // $08:ED61 JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    x.wrapping_add(ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_X[j] as i16 as u16),
                    y.wrapping_add(ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_Y[j] as i16 as u16),
                    ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_CHAR[j],
                    ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_FLAGS[j] | 0x30,
                    ANCILLA41_WATERFALL_SPLASH_WATERFALL_SPLASH_EXT[j],
                );
            }
            j += 1;
            oam += 4;
        }
    }

    pub(super) fn ancilla3_d_item_splash(&mut self, k: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 8);
        if self.game_state.frame.submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
            let mut splash = self.ancilla_slot_view_mut(k);
            splash.set_timer(6);
            if splash.advance_item_to_link() == 5 {
                splash.clear();
                return;
            }
        }
        self.object_splash_draw(k);
    }

    pub(super) fn ancilla15_jump_splash(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(aux_timer) {
                let mut splash = self.ancilla_slot_view_mut(k);
                splash.set_aux_timer(0);
                splash.set_item_to_link(1);
            }
            if self.ancilla_slot_view(k).item_to_link() != 0 {
                let y_velocity = self.ancilla_slot_view(k).y_velocity().wrapping_sub(4);
                {
                    let mut splash = self.ancilla_slot_view_mut(k);
                    splash.set_y_velocity(y_velocity);
                    splash.set_x_velocity(y_velocity);
                }
                if y_velocity < 232 {
                    self.ancilla_slot_view_mut(k).clear();
                    if (self.game_state.player.follower_link.is_bunny_mirror()
                        || self.game_state.player.follower_link.handler_state() == 4)
                        && self.game_state.player.follower_link.is_in_deep_water()
                    {
                        self.check_ability_to_swim();
                    }
                    return;
                }
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
            }
        }

        let (mut x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let ax = self.ancilla_get_x(k);
        let x8 = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_mul(2)
            .wrapping_sub(ax)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let x6 = ax
            .wrapping_add(12)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let j = self.ancilla_slot_view(k).item_to_link() as usize;
        let mut flags = 0;
        for _ in 0..2 {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ANCILLA15_JUMP_SPLASH_ANCILLA_JUMP_SPLASH_CHAR[j],
                0x24 | flags,
                2,
            );
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
            x = x8;
            flags = 0x40;
        }
        self.ancilla_set_oam(oam, x6, y, 0xc0, 0x24, if j == 1 { 1 } else { 2 });
    }

    pub(super) fn ancilla_transmute_to_splash(&mut self, k: usize) {
        let value = 0x3d;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_item_to_link(value);
        let value = 6;
        self.ancilla_slot_view_mut(k).set_timer(value);
        self.ancilla_set_xy(
            k,
            self.ancilla_get_x(k).wrapping_sub(8),
            self.ancilla_get_y(k).wrapping_add(12),
        );
        self.ancilla_sfx2_pan(k, 0x28);
        self.ancilla3_d_item_splash(k);
    }

    pub(super) fn object_splash_draw(&mut self, k: usize) {
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 2;
        for _ in 0..2 {
            if OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_CHAR[j] != 0xff {
                // $08:CA5B JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    x.wrapping_add(OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_Y[j] as i16 as u16),
                    OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_CHAR[j],
                    OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_FLAGS[j] | 0x24,
                    OBJECT_SPLASH_DRAW_OBJECT_SPLASH_DRAW_EXT[j],
                );
                oam += 4;
            }
            j += 1;
        }
    }

    pub(super) fn ancilla43_ganons_tower_cutscene(&mut self, k: usize) {
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut draw_ring = true;

        if self.ancilla_slot_view(k).step() == 0 {
            let yy = self.ancilla_slot_view(k).y_velocity().wrapping_sub(1);
            let value = if yy < 0xf0 { 0xf0 } else { yy };
            self.ancilla_slot_view_mut(k).set_y_velocity(value);
            self.ancilla_move_y(k);
            let x = self.ancilla_get_x(k);
            let y = self.ancilla_get_y(k);
            let bg2vofs = self.game_state.display.ppu_scroll_copy.bg2_v_copy();
            if y.wrapping_sub(bg2vofs) < 0x38 {
                self.tower_seal_scratch_mut().set_center(
                    x.wrapping_add(8),
                    0x38u16.wrapping_add(8).wrapping_add(bg2vofs),
                );
                self.ancilla_set_y(k, 0x38u16.wrapping_add(bg2vofs));
                self.ancilla_slot_view_mut(k).add_step(1);
                self.set_ambient_sound_effect(5);
                self.set_music_control(0xf1);
                self.dialogue_message_index_mut().set_value(0x013b);
                self.main_show_text_message();
                draw_ring = false;
            } else if self.game_state.frame.submodule == 0 {
                draw_ring = false;
            }
        }

        if draw_ring {
            if self.ancilla_slot_view(k).step() == 1 && self.game_state.frame.submodule == 0 {
                let value = 16;
                self.ancilla_slot_view_mut(k).set_x_velocity(value);
                let bak0 = self.ancilla_slot_view(k).x_low();
                let bak1 = self.ancilla_slot_view(k).x_high();
                let value = self.game_state.effects.tower_seal.ring_radius();
                self.ancilla_slot_view_mut(k).set_x_low(value);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_x_high(value);
                self.ancilla_move_x(k);
                let radius = self.ancilla_slot_view(k).x_low();
                self.tower_seal_scratch_mut().set_ring_radius(radius);
                let value = bak0;
                self.ancilla_slot_view_mut(k).set_x_low(value);
                let value = bak1;
                self.ancilla_slot_view_mut(k).set_x_high(value);
                if self.game_state.effects.tower_seal.ring_radius() >= 48 {
                    self.tower_seal_scratch_mut().set_ring_radius(48);
                    self.ancilla_slot_view_mut(k).add_step(1);
                }
            }

            if self.game_state.frame.submodule == 0
                && self.ancilla_slot_view(k).step() != 0
                && self.ancilla_slot_view(k).step() != 1
            {
                if self.ancilla_slot_view(k).step() == 2 {
                    if self.tower_seal_scratch_mut().tick_wait_countdown() == 0 {
                        self.set_special_entrance_trigger(5);
                        self.set_subsubmodule(0);
                        self.scratch_word_mut().clear_module_transition_counter();
                        self.ancilla_slot_view_mut(k).add_step(1);
                    }
                } else {
                    let value = 48;
                    self.ancilla_slot_view_mut(k).set_x_velocity(value);
                    let bak0 = self.ancilla_slot_view(k).x_low();
                    let bak1 = self.ancilla_slot_view(k).x_high();
                    let value = self.game_state.effects.tower_seal.ring_radius();
                    self.ancilla_slot_view_mut(k).set_x_low(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_x_high(value);
                    self.ancilla_move_x(k);
                    let radius = self.ancilla_slot_view(k).x_low();
                    self.tower_seal_scratch_mut().set_ring_radius(radius);
                    let value = bak0;
                    self.ancilla_slot_view_mut(k).set_x_low(value);
                    let value = bak1;
                    self.ancilla_slot_view_mut(k).set_x_high(value);
                    if self.game_state.effects.tower_seal.ring_radius() >= 240 {
                        self.set_sp6r_indoors(0);
                        self.select_overworld_aux_palette_offset();
                        self.palette_load_sprite_environment_dungeon_with_cycle_scope();
                        self.increment_cgram_update_flag();
                        let value = 0;
                        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                        return;
                    }
                }
            }

            let astep = self.ancilla_slot_view(k).step();
            if astep != 0 {
                oam = self.gt_cutscene_sparkle_a_lot(oam);
            }

            for j in (0..=6).rev() {
                if self.game_state.frame.submodule == 0
                    && astep != 1
                    && self.game_state.frame.frame_counter & 1 == 0
                {
                    self.tower_seal_orbit_mut(j).advance_angle_mod64();
                }
                let arp = self.ancilla_get_radial_projection(
                    self.game_state.effects.tower_seal.orbit(j).angle(),
                    self.game_state.effects.tower_seal.ring_radius(),
                );
                let x = (if arp.r6 != 0 {
                    -(arp.r4 as i32)
                } else {
                    arp.r4 as i32
                }) + i32::from(self.game_state.effects.tower_seal.center_x())
                    - 8
                    - i32::from(self.game_state.display.ppu_scroll_copy.bg2_h_copy());
                let y = (if arp.r2 != 0 {
                    -(arp.r0 as i32)
                } else {
                    arp.r0 as i32
                }) + i32::from(self.game_state.effects.tower_seal.center_y())
                    - 8
                    - i32::from(self.game_state.display.ppu_scroll_copy.bg2_v_copy());

                self.tower_seal_orbit_mut(j)
                    .set_base_sparkle_position(x as u16, y as u16);

                self.ancilla_draw_gt_cutscene_crystal(oam, x as u16, y as u16);
                oam += 4;
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.tower_seal_orbit_mut(7).set_base_sparkle_position(x, y);

        self.ancilla_draw_gt_cutscene_crystal(oam, x, y);

        if self.ancilla_slot_view(k).step() == 0 {
            self.ancilla_add_occasional_sparkle(k);
        } else if self.game_state.frame.submodule == 0 {
            self.gt_cutscene_activate_sparkle();
        }
    }

    fn ancilla_draw_gt_cutscene_crystal(&mut self, oam: usize, x: u16, y: u16) {
        self.ancilla_set_oam_safe(oam, x, y, 0x24, 0x3c, 2);
    }

    fn gt_cutscene_activate_sparkle(&mut self) {
        for k in (0..=0x17).rev() {
            if self.game_state.effects.tower_seal.sparkle(k).is_free() {
                self.tower_seal_sparkle_mut(k).set_phase(0);
                self.tower_seal_sparkle_mut(k).set_timer(4);
                let r = self.get_random_number();
                let base = k & 7;
                let (mut x, mut y) = self.tower_seal_sparkle_mut(k).base_sparkle_position(base);
                x = x.wrapping_add((r >> 4) as u16);
                y = y.wrapping_add((r & 0x0f) as u16);
                self.tower_seal_sparkle_mut(k).set_position(x, y);
                return;
            }
        }
    }

    fn gt_cutscene_sparkle_a_lot(&mut self, mut oam: usize) -> usize {
        // $08:CDEA JSR CF35: all 24 actual slots; F6E1 owns each XY child.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x08cf35));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x08cf35);
        for k in (0..=0x17).rev() {
            b.block(0x08cf37);
            if self.game_state.effects.tower_seal.sparkle(k).is_free() {
                b.taken(0x08cf3d);
                b.block(0x08cfa2); if k != 0 { b.taken(0x08cfa3); }
                continue;
            }

            b.block(0x08cf3f);
            let timer = self.tower_seal_sparkle_mut(k).tick_timer();
            if sign8(timer) {
                b.block(0x08cf4a);
                self.tower_seal_sparkle_mut(k).set_timer(4);
                let phase = self.tower_seal_sparkle_mut(k).advance_phase();
                if phase == 3 {
                    self.tower_seal_sparkle_mut(k).set_phase(0xff);
                    b.block(0x08cf5d); b.block(0x08cfa2);
                    if k != 0 { b.taken(0x08cfa3); }
                    continue;
                }
                b.taken(0x08cf5b);
            } else { b.taken(0x08cf48); }

            b.block(0x08cf65);
            let x = self.game_state.effects.tower_seal.sparkle(k).x();
            let y = self.game_state.effects.tower_seal.sparkle(k).y();
            let j = self.game_state.effects.tower_seal.sparkle(k).phase() as usize;
            // $08:CF83 JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x,
                y,
                GT_CUTSCENE_SPARKLE_A_LOT_SWORD_CHARGE_SPARK_CHAR[j],
                GT_CUTSCENE_SPARKLE_A_LOT_SWORD_CHARGE_SPARK_FLAGS[j] | 0x30,
                0,
            );
            b.block(0x08cf86);
            oam += 4;
            b.block(0x08cfa2); if k != 0 { b.taken(0x08cfa3); }
        }
        b.block(0x08cfa5);
        oam
    }

    pub(super) fn ancilla_check_for_entrance_trigger(&self, what: usize) -> bool {
        abs16(
            self.game_state
                .player
                .follower_link
                .y()
                .wrapping_add(12)
                .wrapping_sub(ANCILLA_CHECK_FOR_ENTRANCE_TRIGGER_ENTRANCE_TRIGGER_BASE_Y[what]),
        ) < ANCILLA_CHECK_FOR_ENTRANCE_TRIGGER_ENTRANCE_TRIGGER_SIZE_Y[what]
            && abs16(
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(8)
                    .wrapping_sub(ANCILLA_CHECK_FOR_ENTRANCE_TRIGGER_ENTRANCE_TRIGGER_BASE_X[what]),
            ) < ANCILLA_CHECK_FOR_ENTRANCE_TRIGGER_ENTRANCE_TRIGGER_SIZE_X[what]
    }

    /// ROM: `GameOverText_Draw`
    pub(in crate::zelda_rtl) fn game_over_text_draw(&mut self) {
        self.oam_state_mut().set_current_pointer(0x0800);
        self.oam_state_mut().set_current_extended_pointer(0x0a20);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut k = self.game_state.minigame.flag_boomerang_in_place() as i32;
        loop {
            let j = k as usize * 2;
            let x = self.ancilla_get_x(k as usize);
            self.ancilla_set_oam(
                oam,
                x,
                0x57,
                GAME_OVER_TEXT_DRAW_GAME_OVER_TEXT_CHARS[j],
                0x3c,
                0,
            );
            self.ancilla_set_oam(
                oam + 4,
                x,
                0x5f,
                GAME_OVER_TEXT_DRAW_GAME_OVER_TEXT_CHARS[j + 1],
                0x3c,
                0,
            );
            oam += 8;
            k -= 1;
            if k < 0 {
                break;
            }
        }
    }

    fn dash_tremor_twiddle_offset(&mut self, k: usize) -> i32 {
        let j = self.ancilla_slot_view(k).direction();
        let y = 0u16.wrapping_sub(self.ancilla_get_y(k));
        self.ancilla_set_y(k, y);
        if self.game_state.world.location.is_indoors() {
            return y as i32;
        }
        if j == 2 {
            let start = self
                .game_state
                .world
                .room_bounds
                .packed_top()
                .wrapping_add(1);
            let end = self
                .game_state
                .world
                .room_bounds
                .packed_bottom()
                .wrapping_sub(1);
            let a = y.wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            if a <= start || a >= end {
                0
            } else {
                y as i32
            }
        } else {
            let start = self
                .game_state
                .world
                .room_bounds
                .packed_left()
                .wrapping_add(1);
            let end = self
                .game_state
                .world
                .room_bounds
                .packed_right()
                .wrapping_sub(1);
            let a = y.wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            if a <= start || a >= end {
                0
            } else {
                y as i32
            }
        }
    }

}

impl ZeldaState {
    /// `door_debris_x[k]` ($03B6,X words): RAM-resident, owned by the ancilla bank.
    pub(crate) fn door_debris_x_word(&self, slot: usize) -> u16 {
        crate::types::read_le_u16(
            &self.ram,
            crate::game_state::constants::DOOR_DEBRIS_X + slot * 2,
        )
    }

    /// `door_debris_y[k]` ($03BA,X words).
    pub(crate) fn door_debris_y_word(&self, slot: usize) -> u16 {
        crate::types::read_le_u16(
            &self.ram,
            crate::game_state::constants::DOOR_DEBRIS_Y + slot * 2,
        )
    }

    /// `door_debris_direction[k]` ($03BE,X bytes).
    pub(crate) fn door_debris_direction(&self, slot: usize) -> u8 {
        self.ram[crate::game_state::constants::DOOR_DEBRIS_DIRECTION + slot]
    }

}

#[cfg(test)]
#[test]
fn marker38_tower_sparkle_matches_rom() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
    let Ok(rom) = std::fs::read(path) else { return };
    for active in [false, true] {
        let mut native = ZeldaState::new();
        for k in 0..24 { native.write_wram_byte(0x15837 + k, 0xff); }
        if active {
            for (k, phase, timer) in [(23, 0, 2), (2, 0, 0), (1, 2, 0), (0, 1, 1)] {
                native.write_wram_byte(0x15837 + k, phase);
                native.write_wram_byte(0x158af + k, timer);
                native.write_wram_byte(0x1584f + k, 0x40);
                native.write_wram_byte(0x1587f + k, 0x40);
            }
        }
        native.write_wram_word(0x90, 0x800); native.write_wram_word(0x92, 0xa20);
        native.sync_native_game_state_from_ram(); native.set_lag_free(true);
        let mut machine = snes::Snes::new(); machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        machine.ram.copy_from_slice(&native.wram_image());
        machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
        machine.cpu.e = false; machine.cpu.unpack_flags(0x30);
        machine.cpu.k = 8; machine.cpu.db = 8; machine.cpu.pc = 0xcf35; machine.cpu.sp = 0x1fc;
        crate::cycle_ledger::assert_rom_marker_work(0x08cf35, machine,
            |s| s.cpu.k == 8 && (0xcf35..0xcfa6).contains(&s.cpu.pc), || {
                native.gt_cutscene_sparkle_a_lot(0x800);
            });
    }
}
