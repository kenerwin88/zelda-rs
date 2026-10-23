//! Item receipts, milestone prizes, pond items, and fairy revival.

use super::*;
use crate::game_state::constants::{DUNG_SAVEGAME_STATE_BITS};
use crate::types::{abs8, sign8};

impl ZeldaState {
    pub(in crate::zelda_rtl) fn ancilla_add_falling_prize(&mut self, a: u8, item_idx: u8, yv: u8) -> i32 {
        self.follower_link_state_mut()
            .set_receive_item_index(item_idx);
        let Some(k) = self.ancilla_add_simple(a, yv) else {
            return -1;
        };
        let item_type = ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_TYPE[item_idx as usize];
        if item_type == 0x10 || item_type == 0x0f {
            self.decode_animated_sprite_tile_variable(RECEIVE_ITEM_GRAPHICS[item_type as usize]);
        }

        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_item_to_link(item_type);
            ancilla.set_z_velocity((-48i8) as u8);
            ancilla.set_y_velocity(0);
            ancilla.set_x_velocity(0);
            ancilla.set_z(ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_Z[item_idx as usize]);
            ancilla.set_step(0);
            ancilla.set_aux_timer(9);
            ancilla.set_work_byte_3(0);
            ancilla.set_l(0);
            ancilla.set_g(ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_G[item_idx as usize]);
        }
        self.follower_link_state_mut()
            .set_receive_item_index(item_type);

        let (x, y) = if item_idx != 0 && item_idx != 5 {
            if self.game_state.inventory.save_progress.palace_index_x2() == 20 {
                (
                    (self.game_state.player.follower_link.x() & 0xff00) | 0x0100,
                    (self.game_state.player.follower_link.y() & 0xff00) | 0x0100,
                )
            } else {
                (
                    ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_X[item_idx as usize]
                        .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
                    ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_Y[item_idx as usize]
                        .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
                )
            }
        } else {
            (
                self.game_state.player.follower_link.x(),
                ANCILLA_ADD_FALLING_PRIZE_FALLING_ITEM_Y[item_idx as usize]
                    .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
            )
        };
        self.ancilla_set_xy(k, x, y);
        k as i32
    }

    pub(in crate::zelda_rtl) fn configure_revival_ancillae(&mut self) {
        self.follower_link_state_mut()
            .set_link_dma_staging_index(80);
        let mut k = 0usize;

        {
            let mut revival = self.ancilla_slot_view_mut(k);
            revival.set_work_byte_3(64);
            revival.set_step(0);
            revival.set_z_velocity(8);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        let value = 5;
        self.ancilla_slot_view_mut(k).set_g(value);
        self.ancilla_slot_view_mut(k).set_item_to_link(0);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
        );
        self.ancilla_slot_view_mut(k).set_z(0);
        k += 1;

        self.ancilla_slot_view_mut(k).set_z(0);
        {
            let mut revival = self.ancilla_slot_view_mut(k);
            revival.set_work_byte_3(240);
            revival.set_step(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        k += 1;

        self.ancilla_slot_view_mut(k).set_item_to_link(2);
        {
            let mut revival = self.ancilla_slot_view_mut(k);
            revival.set_aux_timer(3);
            revival.set_work_byte_3(8);
            revival.set_step(0);
        }
        self.ancilla_slot_view_mut(k).set_direction(3);
        let item_to_link = self.ancilla_slot_view(k).item_to_link() as usize;
        self.ancilla_slot_view_mut(k)
            .set_work_byte_25(MAGIC_POWDER_FRAME_TIMERS[30 + item_to_link]);

        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x().wrapping_add(20),
            self.game_state.player.follower_link.y().wrapping_add(2),
        );
    }

    pub(in crate::zelda_rtl) fn ancilla_add_ms_cutscene(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut cutscene = self.ancilla_slot_view_mut(k);
                cutscene.set_item_to_link(0);
                cutscene.set_aux_timer(2);
                cutscene.set_timer(64);
            }
            self.ancilla_set_xy(
                k,
                self.game_state.player.follower_link.x().wrapping_add(8),
                self.game_state.player.follower_link.y().wrapping_sub(8),
            );
        }
    }

    pub(in crate::zelda_rtl) fn add_happiness_pond_rupees(&mut self, arg: u8) {
        if !self.add_happiness_pond_rupees_before_graphics() {
            return;
        }
        self.decode_animated_sprite_tile_variable(0x24);
        self.add_happiness_pond_rupees_after_graphics(arg);
    }

    pub(in crate::zelda_rtl) fn add_happiness_pond_rupees_before_graphics(&mut self) -> bool {
        let Some(_) = self.ancilla_add_simple(0x42, 9) else {
            return false;
        };
        self.set_sound_effect_2_with_link_pan(0x13);
        true
    }

    pub(in crate::zelda_rtl) fn add_happiness_pond_rupees_after_graphics(&mut self, arg: u8) {
        self.follower_link_state_mut().enter_item_hold_pose();

        for i in 0..10 {
            self.happiness_pond_rupee_mut(i).clear();
        }

        let mut j = ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_START[arg as usize];
        let j_end = ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_END[arg as usize];
        let mut k = 9usize;
        loop {
            let x = self.game_state.player.follower_link.x().wrapping_add(4);
            let y = self.game_state.player.follower_link.y().wrapping_sub(12);
            self.happiness_pond_rupee_mut(k).initialize(
                x,
                y,
                ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_XVEL[j as usize] as u8,
                ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_YVEL[j as usize] as u8,
                ADD_HAPPINESS_POND_RUPEES_HAPPINESS_POND_ZVEL[j as usize] as u8,
            );
            if k == 0 {
                break;
            }
            k -= 1;
            j -= 1;
            if j == j_end {
                break;
            }
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_tossed_pond_item(&mut self, a: u8, xin: u8, yin: u8) {
        if let Some((slot, graphics)) =
            self.ancilla_add_tossed_pond_item_until_graphics(a, xin, yin)
        {
            self.complete_ancilla_add_tossed_pond_item_graphics(slot, xin, graphics);
        }
    }

    /// Execute `AncillaAdd_TossedPondItem` through the source call to
    /// `DecodeAnimatedSpriteTile_variable`, but do not enter that synchronous
    /// graphics call. The selected ancilla slot and decoded-sheet selector are
    /// call locals which must survive a host return inside the decompressor.
    pub(in crate::zelda_rtl) fn ancilla_add_tossed_pond_item_until_graphics(
        &mut self,
        a: u8,
        xin: u8,
        yin: u8,
    ) -> Option<(usize, u8)> {
        self.follower_link_state_mut().set_receive_item_index(xin);
        let k = self.ancilla_add_ancilla(a, yin)?;
        self.set_sound_effect_2_with_link_pan(0x13);
        let graphics = RECEIVE_ITEM_GRAPHICS[xin as usize];
        if graphics == 0x20 {
            self.DecompressShieldGraphics();
        }
        Some((k, graphics))
    }

    /// Resume `AncillaAdd_TossedPondItem` at its interrupted animated-sheet
    /// call and finish the exact caller suffix without replaying its spawn.
    pub(in crate::zelda_rtl) fn complete_ancilla_add_tossed_pond_item_graphics(
        &mut self,
        k: usize,
        receive_item: u8,
        graphics: u8,
    ) {
        self.decode_animated_sprite_tile_variable(if graphics == 0xff { 0 } else { graphics });
        if graphics == 6 {
            self.DecompressSwordGraphics();
        }

        self.follower_link_state_mut().enter_item_hold_pose();
        {
            let mut item = self.ancilla_slot_view_mut(k);
            item.set_z_velocity(20);
            item.set_y_velocity((-40i8) as u8);
            item.set_x_velocity(0);
            item.set_z(0);
            item.set_timer(16);
            item.set_item_to_link(receive_item);
        }
        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x().wrapping_add(
                ANCILLA_ADD_TOSSED_POND_ITEM_WISH_POND_ITEM_X[receive_item as usize] as u16,
            ),
            self.game_state.player.follower_link.y().wrapping_add(
                ANCILLA_ADD_TOSSED_POND_ITEM_WISH_POND_ITEM_Y[receive_item as usize] as i16 as u16,
            ),
        );
    }

    pub(super) fn ancilla35_master_sword_receipt(&mut self, k: usize) {
        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).clear();
            return;
        }
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = if self.ancilla_slot_view(k).item_to_link() == 2 {
                0
            } else {
                self.ancilla_slot_view(k).item_to_link().wrapping_add(1)
            };
            self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
        }

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let item = self.ancilla_slot_view(k).item_to_link();
        if item == 0 {
            return;
        }

        let mut j = item.wrapping_sub(1) as usize * 4;
        for _ in 0..4 {
            self.ancilla_set_oam(
                oam,
                x.wrapping_add(ANCILLA35_MASTER_SWORD_RECEIPT_SWORD_CEREMONY_X[j] as i16 as u16),
                y.wrapping_add(ANCILLA35_MASTER_SWORD_RECEIPT_SWORD_CEREMONY_Y[j] as i16 as u16),
                ANCILLA35_MASTER_SWORD_RECEIPT_SWORD_CEREMONY_CHAR[j],
                ANCILLA35_MASTER_SWORD_RECEIPT_SWORD_CEREMONY_FLAGS[j] & !0x30
                    | 4
                    | self.game_state.oam.priority_high(),
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    pub(super) fn ancilla36_flute(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            if self.ancilla_slot_view(k).step() != 3 {
                self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
                self.ancilla_move_x(k);
                self.ancilla_move_z(k);
                let z = self.ancilla_slot_view(k).z();
                if sign8(z) || z >= 0xf0 {
                    let step = self.ancilla_slot_view_mut(k).advance_step();
                    let z_velocity = ANCILLA36_FLUTE_FLUTE_VELS[step as usize];
                    let mut flute = self.ancilla_slot_view_mut(k);
                    flute.set_z_velocity(z_velocity);
                    flute.set_z(0);
                }
            } else if self.ancilla_check_link_collision(k, 2)
                && !self
                    .game_state
                    .player
                    .follower_link
                    .has_hookshot_interlock()
                && !self.game_state.player.follower_link.has_auxiliary_state()
            {
                self.ancilla_slot_view_mut(k).clear();
                self.follower_link_state_mut().set_item_receipt_method(0);
                let _ = self.link_receive_item_from(
                    0x14,
                    0,
                    ItemReceiptCaller::AncillaMilestone {
                        ancilla_slot: k as u8,
                        suffix: AncillaItemReceiptSuffix::None,
                    },
                );
                return;
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam = self.game_state.oam.current_pointer_usize();
        self.ancilla_set_oam(
            oam,
            x,
            y.wrapping_sub(self.ancilla_slot_view(k).z() as i8 as i16 as u16),
            0x24,
            self.game_state.oam.priority_high() | 4,
            2,
        );
        if self.game_state.oam.entry_y(oam) == 0xf0 {
            self.ancilla_slot_view_mut(k).clear();
        }
    }

    fn wish_pond_item_draw(&mut self, k: usize) {
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);

        if self.ancilla_slot_view(k).item_to_link() == 1 {
            let value = 5;
            self.ancilla_slot_view_mut(k).set_work_byte_4(value);
        }

        let oam = self.ancilla_receive_item_draw(
            k,
            x,
            y.wrapping_sub(self.ancilla_slot_view(k).z() as i8 as i16 as u16),
        );

        if !self
            .game_state
            .player
            .follower_link
            .picking_throw_state_has(2)
            || (!sign8(self.ancilla_slot_view(k).z_velocity())
                && self.ancilla_slot_view(k).z_velocity() >= 2)
        {
            return;
        }

        let xx = self.asset_u8(71, self.ancilla_slot_view(k).item_to_link() as usize);
        self.ancilla_draw_shadow(
            oam,
            if xx == 2 { 1 } else { 2 },
            x.wrapping_sub(if xx == 2 { 0 } else { 4 }),
            y.wrapping_add(40),
            self.game_state.oam.priority_high(),
        );
    }

    pub(super) fn ancilla_receive_item_draw(&mut self, k: usize, x: u16, y: u16) -> usize {
        let _scope = crate::cycle_ledger::routine(0x08_c690);
        crate::cycle_ledger::charge(186); // PHX, palette/item locals, LDY #0, JSR SetOam_XY.
        let mut oam = self.game_state.oam.current_pointer_usize();
        let j = self.ancilla_slot_view(k).item_to_link() as usize;
        let x_bit = self.ancilla_set_oam_xy(oam, x, y);
        crate::cycle_ledger::charge(132); // Character store and palette lookup/test.
        let mut a = WISH_POND_ITEM_OAM_FLAGS[j];
        if sign8(a) {
            crate::cycle_ledger::charge(24);
            a = self.ancilla_slot_view(k).work_byte_4();
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(344); // Flags/extended store and second-tile test.
        self.oam_state_mut().set_entry_char(oam, 0x24);
        self.oam_state_mut().set_entry_flags(oam, a.wrapping_mul(2) | 0x30);
        self.oam_state_mut().set_extended_byte((oam - OAM_BUF) / 4, RECEIVE_ITEM_OAM_EXT_SIZES[j] | x_bit);
        oam += 4;
        if RECEIVE_ITEM_OAM_EXT_SIZES[j] == 0 {
            crate::cycle_ledger::charge(154); // Load saved second Y, SEP, JSR SetOam_XY.
            let x_bit = self.ancilla_set_oam_xy(oam, x, y.wrapping_add(8));
            crate::cycle_ledger::charge(132 + if sign8(WISH_POND_ITEM_OAM_FLAGS[j]) { 24 } else { 6 });
            crate::cycle_ledger::charge(288); // Second flags/extended store.
            self.oam_state_mut().set_entry_char(oam, 0x34);
            self.oam_state_mut().set_entry_flags(oam, a.wrapping_mul(2) | 0x30);
            self.oam_state_mut().set_extended_byte((oam - OAM_BUF) / 4, x_bit);
            oam += 4;
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(70); // PLX : RTS.
        oam
    }

    fn item_receipt_transmute_to_rising_crystal(&mut self, k: usize) {
        let value = 0x3e;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_y_velocity(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_x_velocity(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_y_subpixel(value);
        self.ancilla_rising_crystal(k);
    }

    pub(super) fn ancilla22_item_receipt(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x08_c38a);
        crate::cycle_ledger::charge(64); // Immobilized-state load, compare and BEQ.
        if self.game_state.player.follower_link.immobilized_flag() != 2 {
            // The successive submodule comparisons stop at their taken branch.
            crate::cycle_ledger::charge(match self.game_state.frame.submodule {
                0 => 40 + 6,
                43 => 40 + 32 + 6,
                9 => 40 + 32 + 32 + 6,
                2 => 40 + 32 + 32 + 32,
                _ => 40 + 32 + 32 + 32 + 6,
            });
            if self.game_state.frame.submodule != 0
                && self.game_state.frame.submodule != 43
                && self.game_state.frame.submodule != 9
            {
                if self.game_state.frame.submodule == 2 {
                    crate::cycle_ledger::charge(54); // LDA #16 : STA timer,X.
                    let value = 16;
                    self.ancilla_slot_view_mut(k).set_timer(value);
                }
                crate::cycle_ledger::charge(30); // BRL drawing.
            } else {
                crate::cycle_ledger::charge(94 + match self.ancilla_slot_view(k).step() {
                    0 => 6,
                    3 => 32 + 6,
                    _ => 32 + 30,
                }); // INC modal pause, step tests, possible BRL C515.
                self.increment_modal_pause_flag();

                if self.ancilla_slot_view(k).step() != 0 && self.ancilla_slot_view(k).step() != 3 {
                    crate::cycle_ledger::charge(100); // DEC/LDA aux timer : BPL.
                    let value = self.ancilla_slot_view(k).aux_timer().wrapping_sub(1);
                    self.ancilla_slot_view_mut(k).set_aux_timer(value);
                    if sign8(self.ancilla_slot_view(k).aux_timer()) {
                        crate::cycle_ledger::charge(30); // BRL finish.
                        self.ancilla22_item_receipt_finish(k);
                        return;
                    }
                    crate::cycle_ledger::charge(6 + 32); // Taken BPL : CMP #0 : BEQ.
                    if self.ancilla_slot_view(k).aux_timer() == 0 {
                        crate::cycle_ledger::charge(6);
                        self.ancilla22_item_receipt_show_message(k);
                    } else {
                        crate::cycle_ledger::charge(32); // CMP #40 : BNE.
                        let mut play_sound = false;
                        if self.ancilla_slot_view(k).aux_timer() == 40 {
                            crate::cycle_ledger::charge(64); // LDA step : CMP #2 : BEQ.
                            if self.ancilla_slot_view(k).step() != 2 {
                                crate::cycle_ledger::charge(78); // JSL AddRupees : BCS.
                                if self.ancilla_add_rupees(k) {
                                    crate::cycle_ledger::charge(6);
                                    play_sound = true;
                                } else {
                                    crate::cycle_ledger::charge(64); // Item test after carry clear.
                                    if self.ancilla_slot_view(k).item_to_link() != 0x17 {
                                        crate::cycle_ledger::charge(6);
                                        play_sound = true;
                                    }
                                }
                            } else {
                                crate::cycle_ledger::charge(6);
                            }
                        } else {
                            crate::cycle_ledger::charge(6);
                        }
                        if play_sound {
                            crate::cycle_ledger::charge(62); // LDA #15 : JSR sound.
                            self.ancilla_sfx3_near(0x0f);
                            crate::cycle_ledger::charge(22); // BRA to the shared movement branch.
                        }
                        crate::cycle_ledger::charge(30); // BRL movement.
                        self.ancilla22_item_receipt_move_label_b(k);
                    }
                } else if self.ancilla_slot_view(k).item_to_link() == 1
                    && self.ancilla_slot_view(k).step() != 2
                {
                    crate::cycle_ledger::charge(64 + 64 + 48); // Item, step, timer tests.
                    if self.ancilla_slot_view(k).timer() == 0 {
                        crate::cycle_ledger::charge(6);
                        self.ancilla22_item_receipt_label_a(k);
                        return;
                    }
                    crate::cycle_ledger::charge(32); // CMP #17 : BNE.
                    if self.ancilla_slot_view(k).timer() == 17 {
                        crate::cycle_ledger::charge(164 + 30); // Shared message timer, follower, BRL message.
                        self.start_shared_message_timer(0x0df3);
                        self.follower_state_mut().set_indicator(0x0e);
                        self.ancilla22_item_receipt_show_message(k);
                    } else {
                        crate::cycle_ledger::charge(6 + 30); // Taken BNE : BRL drawing.
                    }
                } else {
                    crate::cycle_ledger::charge(64 + 6 + 100); // Non-sword item, DEC/LDA aux : BEQ.
                    let value = self.ancilla_slot_view(k).aux_timer().wrapping_sub(1);
                    self.ancilla_slot_view_mut(k).set_aux_timer(value);
                    let a = self.ancilla_slot_view(k).aux_timer();
                    if a == 0 {
                        crate::cycle_ledger::charge(6);
                        self.ancilla22_item_receipt_label_a(k);
                        return;
                    }
                    crate::cycle_ledger::charge(32); // CMP #1 : BNE.
                    if a == 1 {
                        let item = self.ancilla_slot_view(k).item_to_link();
                        crate::cycle_ledger::charge(match item {
                            0x37 => 64 + 6,
                            0x38 => 64 + 32 + 6,
                            0x39 => 64 + 32 + 32,
                            _ => 64 + 32 + 32 + 6,
                        });
                        let pendant = (0x37..=0x39).contains(&item);
                        if pendant { crate::cycle_ledger::charge(46); } // System-bank APUI00 read + BEQ.
                        if (item == 0x37 || item == 0x38 || item == 0x39)
                            && self.zelda_read_apui00_for_song_end_poll() != 0
                        {
                            crate::cycle_ledger::charge(74 + 30); // INC aux : BRA : BRL drawing.
                            let value = self.ancilla_slot_view(k).aux_timer().wrapping_add(1);
                            self.ancilla_slot_view_mut(k).set_aux_timer(value);
                        } else {
                            crate::cycle_ledger::charge(if pendant { 6 + 60 } else { 60 }); // Two BRLs to message.
                            self.ancilla22_item_receipt_show_message(k);
                        }
                    } else {
                        crate::cycle_ledger::charge(6 + 30); // Taken BNE : BRL drawing.
                    }
                }
            }
        } else {
            crate::cycle_ledger::charge(6 + 30); // Taken BEQ : BRL drawing.
        }

        self.ancilla22_item_receipt_draw_and_update(k);
    }

    fn ancilla22_item_receipt_label_a(&mut self, k: usize) {
        if self.ancilla_slot_view(k).item_to_link() == 1 && self.ancilla_slot_view(k).step() == 0 {
            self.set_ambient_sound_effect(5);
            self.set_music_control(2);
        }
        let handler_state = if self.game_state.player.follower_link.is_in_deep_water() {
            4
        } else {
            0
        };
        self.follower_link_state_mut()
            .set_handler_state(handler_state);
        self.follower_link_state_mut().set_receive_item_index(0);
        self.follower_link_state_mut().clear_item_hold_pose();
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.ancilla_add_rupees(k);
        self.ancilla22_item_receipt_finish(k);
    }

    fn ancilla22_item_receipt_finish(&mut self, k: usize) {
        self.follower_link_state_mut().set_item_receipt_method(0);
        let a = self.ancilla_slot_view(k).item_to_link();
        if a == 23 && self.game_state.inventory.player_resources.heart_pieces() == 0 {
            if self
                .link_receive_item_from(
                    0x26,
                    0,
                    ItemReceiptCaller::AncillaMilestone {
                        ancilla_slot: k as u8,
                        suffix: AncillaItemReceiptSuffix::ClearAncillaAndModalPause,
                    },
                )
                .is_suspended()
            {
                return;
            }
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            self.clear_modal_pause_flag();
            return;
        }

        if a == 0x26 || a == 0x3f {
            if self.game_state.inventory.player_resources.health_capacity() != 0xa0 {
                let capacity = self.player_resources_mut().increment_health_capacity_by(8);
                let filler = capacity
                    .wrapping_sub(self.game_state.inventory.player_resources.current_health());
                self.player_resources_mut()
                    .increment_heart_filler_by(filler);
                self.ancilla_sfx3_near(0x0d);
            }
        } else if a == 0x3e {
            self.follower_link_state_mut().clear_immobilized();
            if self.game_state.inventory.player_resources.health_capacity() != 0xa0 {
                self.player_resources_mut().increment_health_capacity_by(8);
                self.player_resources_mut().increment_heart_filler_by(8);
                self.ancilla_sfx3_near(0x0d);
            }
        } else if a == 0x42 {
            self.player_resources_mut().increment_heart_filler_by(8);
        } else if a == 0x45 {
            self.player_resources_mut().increment_magic_filler_by(16);
        } else if a == 0x22 || a == 0x23 {
            self.palette_load_link_armor_and_gloves();
        }

        let value = 0;

        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        self.clear_modal_pause_flag();
        let a = self.ancilla_slot_view(k).item_to_link();
        if self.ancilla_slot_view(k).step() == 3 && a != 0x10 && a != 0x26 && a != 0x0f && a != 0x20
        {
            self.prepare_dungeon_exit_from_boss_fight();
        }

        if self.ancilla_slot_view(k).step() != 2 {
            self.follower_link_state_mut().clear_immobilized();
        }
    }

    pub(super) fn ancilla22_item_receipt_show_message(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x08_c546);
        crate::cycle_ledger::charge(40); // Indoor test.
        if self.game_state.world.location.is_indoors() {
            let room = self.game_state.world.location.dungeon_room();
            crate::cycle_ledger::charge(match room {
                0x00ff => 94 + 6,
                0x010f => 94 + 40 + 6,
                0x0110 => 94 + 80 + 6,
                0x0112 => 94 + 120 + 6,
                0x011f => 94 + 160,
                _ => 94 + 160 + 6,
            });
            if room == 0x00ff
                || room == 0x010f
                || room == 0x0110
                || room == 0x0112
                || room == 0x011f
            {
                crate::cycle_ledger::charge(44); // SEP : BRA movement.
                self.ancilla22_item_receipt_move_label_b(k);
                return;
            }
        } else {
            crate::cycle_ledger::charge(6);
        }
        let item = self.ancilla_slot_view(k).item_to_link() as usize;
        crate::cycle_ledger::charge(86 + if item == 0x38 { 6 } else { 32 });
        let mut msg = -1i16;
        let mut general_message = false;
        if self.ancilla_slot_view(k).item_to_link() == 0x38
            || self.ancilla_slot_view(k).item_to_link() == 0x39
        {
            crate::cycle_ledger::charge(102); // Pendant flags test.
            if self.game_state.inventory.player_resources.pendant_flags() & 7 == 7 {
                crate::cycle_ledger::charge(218); // Special table, message store, SEP : BRA.
                msg = RECEIVE_ITEM_SPECIAL_MESSAGES[item - 0x38];
            } else {
                crate::cycle_ledger::charge(6);
                general_message = true;
            }
        } else {
            crate::cycle_ledger::charge(6 + 64); // Taken non-pendant branch and step test.
            if self.ancilla_slot_view(k).step() != 2 {
                crate::cycle_ledger::charge(64); // Heart-piece item test.
                if item == 0x17 {
                    crate::cycle_ledger::charge(170);
                    msg = RECEIVE_ITEM_HEART_PIECE_MESSAGES
                        [self.game_state.inventory.player_resources.heart_pieces() as usize];
                    crate::cycle_ledger::charge(if msg == -1 { 6 } else { 84 });
                } else {
                    crate::cycle_ledger::charge(6);
                    general_message = true;
                }
            } else {
                crate::cycle_ledger::charge(6);
            }
        }
        if general_message {
            // The word table starts at C2DD; Y = 2*item crosses the page at item 18.
            crate::cycle_ledger::charge(162 + if item >= 18 { 6 } else { 0 });
            msg = RECEIVE_ITEM_MESSAGES[item];
            if msg == -1 {
                crate::cycle_ledger::charge(6);
            } else {
                crate::cycle_ledger::charge(80 + if msg == 0x70 { 70 } else { 6 } + 22);
            }
        }
        if msg != -1 {
            self.dialogue_message_index_mut().set_value(msg as u16);
            if msg == 0x70 {
                self.set_ambient_sound_effect(9);
            }
            crate::cycle_ledger::charge(62); // JSL Main_ShowTextMessage.
            self.main_show_text_message();
            crate::cycle_ledger::charge(22); // BRA drawing after the call returns.
        }
    }

    fn ancilla22_item_receipt_move_label_b(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x08_c5e0);
        crate::cycle_ledger::charge(64); // Aux timer : CMP #24 : BCC.
        if self.ancilla_slot_view(k).aux_timer() >= 24 {
            crate::cycle_ledger::charge(94); // Velocity decrement and lower-bound test.
            let a = self.ancilla_slot_view(k).y_velocity().wrapping_sub(1);
            if a >= 248 {
                crate::cycle_ledger::charge(38);
                let value = a;
                self.ancilla_slot_view_mut(k).set_y_velocity(value);
            } else {
                crate::cycle_ledger::charge(6);
            }
            crate::cycle_ledger::charge(46); // JSR MoveY.
            self.ancilla_move_y(k);
        } else {
            crate::cycle_ledger::charge(6);
        }
    }

    pub(super) fn ancilla22_item_receipt_draw_and_update(&mut self,k:usize) {
        let _scope=crate::cycle_ledger::routine(0x08_c5f7);
        crate::cycle_ledger::charge(86);
        if self.ancilla_slot_view(k).item_to_link()==0x20 {
            crate::cycle_ledger::charge(84);
            self.ancilla_slot_view_mut(k).set_z(0);
            self.ancilla_add_occasional_sparkle(k);
            self.ancilla_receipt_after_sparkle(k);
        } else {
            crate::cycle_ledger::charge(6);
            self.ancilla_receipt_animation_and_draw(k);
        }
    }

    fn ancilla_receipt_after_sparkle(&mut self,k:usize) {
        let _scope=crate::cycle_ledger::routine(0x08_c606);
            crate::cycle_ledger::charge(46); // System-bank APUI00 read : BNE.
            if self.zelda_read_apui00_for_song_end_poll() == 0 {
                crate::cycle_ledger::charge(78); // Music store : BRL crystal continuation.
                self.set_music_control(0x1a);
                self.item_receipt_transmute_to_rising_crystal(k);
                return;
            }
            crate::cycle_ledger::charge(6);
        self.ancilla_receipt_animation_and_draw(k);
    }

    fn ancilla_receipt_animation_and_draw(&mut self,k:usize) {
        let _scope=crate::cycle_ledger::routine(0x08_c613);
        crate::cycle_ledger::charge(86); // C613..C61A: SEP and sword test.
        if self.ancilla_slot_view(k).item_to_link() == 1 {
            crate::cycle_ledger::charge(134); // First crystal frame and step test.
            let value = RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE[0];
            self.ancilla_slot_view_mut(k).set_work_byte_4(value);
            if self.ancilla_slot_view(k).step() != 2 {
                crate::cycle_ledger::charge(64); // Timer : CMP #16 : BCC.
                if self.ancilla_slot_view(k).timer() < 16 {
                    crate::cycle_ledger::charge(6 + 16 + 122);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_work_byte_1(value);
                    let value = RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE[0];
                    self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                } else {
                    crate::cycle_ledger::charge(68); // DEC frame timer : BPL.
                    self.ancilla_slot_view_mut(k).tick_work_byte_3();
                    if sign8(self.ancilla_slot_view(k).work_byte_3()) {
                        crate::cycle_ledger::charge(132); // Reset timer, advance frame and test wrap.
                        let value = 2;
                        self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                        let mut a = self.ancilla_slot_view(k).work_byte_1().wrapping_add(1);
                        if a == 3 {
                            crate::cycle_ledger::charge(16);
                            a = 0;
                        } else {
                            crate::cycle_ledger::charge(6);
                        }
                        crate::cycle_ledger::charge(122); // Frame and palette publication.
                        let value = a;
                        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
                        let value = RECEIVE_ITEM_CRYSTAL_FRAME_SEQUENCE[a as usize];
                        self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                    } else {
                        crate::cycle_ledger::charge(6);
                    }
                }
            } else {
                crate::cycle_ledger::charge(6);
            }
        } else {
            crate::cycle_ledger::charge(6);
        }

        crate::cycle_ledger::charge(match self.ancilla_slot_view(k).item_to_link() {
            0x34 => 64 + 6,
            0x35 => 64 + 32 + 6,
            0x36 => 64 + 32 + 32,
            _ => 64 + 32 + 32 + 6,
        }); // Milestone animation item tests.
        if self.ancilla_slot_view(k).item_to_link() == 0x34
            || self.ancilla_slot_view(k).item_to_link() == 0x35
            || self.ancilla_slot_view(k).item_to_link() == 0x36
        {
            crate::cycle_ledger::charge(68); // DEC frame timer : BPL.
            self.ancilla_slot_view_mut(k).tick_work_byte_3();
            if sign8(self.ancilla_slot_view(k).work_byte_3()) {
                crate::cycle_ledger::charge(116); // INC/LDA frame : CMP #3 : BNE.
                let mut a = self.ancilla_slot_view(k).work_byte_1().wrapping_add(1);
                if a == 3 {
                    crate::cycle_ledger::charge(54);
                    a = 0;
                } else {
                    crate::cycle_ledger::charge(6);
                }
                crate::cycle_ledger::charge(200); // Tables, timer store, PHX and JSL graphics.
                let value = a;
                self.ancilla_slot_view_mut(k).set_work_byte_1(value);
                let value = RECEIVE_ITEM_MILESTONE_FRAME_TIMERS[a as usize];
                self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                self.write_to4_bpp_buffer_at_7f4000(RECEIVE_ITEM_MILESTONE_GFX_SOURCES[a as usize]);
                crate::cycle_ledger::charge(28); // PLX after graphics return.
            } else {
                crate::cycle_ledger::charge(6);
            }
        }

        self.ancilla_receipt_coordinates_and_draw(k);
    }

    fn ancilla_receipt_coordinates_and_draw(&mut self,k:usize) {
        let _scope=crate::cycle_ledger::routine(0x08_c681);
        crate::cycle_ledger::charge(46); // JSR adjusted coordinates.
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        crate::cycle_ledger::charge(146); // Save y+8 for the optional second tile.
        self.ancilla_receive_item_draw(k, x, y);
    }

    pub(super) fn ancilla_rising_crystal(&mut self, k: usize) {
        let value = 0;

        self.ancilla_slot_view_mut(k).set_z(value);
        self.ancilla_add_occasional_sparkle(k);
        let mut yy = self.ancilla_slot_view(k).y_velocity().wrapping_sub(1);
        if yy < 0xf0 {
            yy = 0xf0;
        }
        let value = yy;
        self.ancilla_slot_view_mut(k).set_y_velocity(value);
        self.ancilla_move_y(k);

        let y = self
            .ancilla_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy());
        if y < 0x49 {
            self.ancilla_set_y(
                k,
                0x49u16.wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy()),
            );
            if self.game_state.frame.submodule == 0 {
                let i = (self.game_state.inventory.save_progress.palace_index_x2() >> 1) as usize;
                self.player_resources_mut()
                    .add_crystal_flags(ANCILLA_RISING_CRYSTAL_DUNGEON_CRYSTAL_PENDANT_BIT[i]);
                self.set_submodule(0x18);
                self.set_subsubmodule(0);
                self.clear_aux_visible_subpalettes();
                self.set_countdown_word(0);
                self.set_darkening_or_lightening_screen_word(0);
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        self.ancilla_receive_item_draw(k, x, y);
    }

    pub(super) fn ancilla29_milestone_item_receipt(&mut self, k: usize) {
        if crate::debug_env::var_os("ZELDA3_DEBUG_ANCILLA29").is_some() {
            eprintln!(
                "[ANC29] host={} k={} item={:#x} z={} g={} native_bits={:#06x} ram_bits={:#04x}{:02x} countdown={} work3={} link_aux={} hook={} sub={}",
                self.frame_ctr_dbg,
                k,
                self.ancilla_slot_view(k).item_to_link(),
                self.ancilla_slot_view(k).z(),
                self.ancilla_slot_view(k).g(),
                self.game_state.dungeon.savegame_state.savegame_state_bits(),
                self.ram[DUNG_SAVEGAME_STATE_BITS + 1],
                self.ram[DUNG_SAVEGAME_STATE_BITS],
                self.game_state.world.transient.boss_prize_graphics_countdown(),
                self.ancilla_slot_view(k).work_byte_3(),
                self.game_state.player.follower_link.has_auxiliary_state(),
                self.game_state
                    .player
                    .follower_link
                    .has_hookshot_interlock(),
                self.game_state.frame.submodule,
            );
        }
        if self.ancilla_slot_view(k).item_to_link() != 0x10
            && self.ancilla_slot_view(k).item_to_link() != 0x0f
        {
            let dung_savegame_state_bits =
                self.game_state.dungeon.savegame_state.savegame_state_bits();
            if dung_savegame_state_bits & 0x4000 != 0 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }

            if dung_savegame_state_bits & 0x8000 == 0 {
                return;
            }

            let countdown = self.game_state.world.transient.boss_prize_graphics_countdown();
            if countdown != 0 {
                if countdown == 1 {
                    if self.ancilla_slot_view(k).item_to_link() == 0x20 {
                        self.set_ambient_sound_effect(0x0f);
                        self.decode_animated_sprite_tile_variable(0x28);
                    } else {
                        self.decode_animated_sprite_tile_variable(0x23);
                    }
                }
                self.world_transient_mut()
                    .decrement_boss_prize_graphics_countdown();
                return;
            }
            if self.ancilla_slot_view(k).work_byte_3() == 0
                && self.ancilla_slot_view(k).item_to_link() == 0x20
            {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                self.set_sp6r_indoors(4);
                self.select_overworld_aux_palette_offset();
                self.palette_load_sprite_environment_dungeon_with_cycle_scope();
                self.increment_cgram_update_flag();
            }
        } else if self.ancilla_slot_view(k).g() != 0 {
            self.ancilla_slot_view_mut(k).subtract_g(1);
            return;
        }

        if self.ancilla_slot_view(k).item_to_link() == 0x20 {
            self.ancilla_add_occasional_sparkle(k);
        }

        if self.game_state.frame.submodule == 0 {
            if self.ancilla_slot_view(k).z() < 24
                && self.ancilla_check_link_collision(k, 2)
                && !self
                    .game_state
                    .player
                    .follower_link
                    .has_hookshot_interlock()
                && !self.game_state.player.follower_link.has_auxiliary_state()
            {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                if self.game_state.player.follower_link.handler_state() == 25
                    || self.game_state.player.follower_link.handler_state() == 26
                {
                    self.clear_custom_spell_animation();
                    self.follower_link_state_mut().clear_force_hold_sword_up();
                    self.follower_link_state_mut().clear_handler_state();
                }
                self.follower_link_state_mut().set_item_receipt_method(3);
                let item = self.ancilla_slot_view(k).item_to_link();
                let _ = self.link_receive_item_from(
                    item,
                    0,
                    ItemReceiptCaller::AncillaMilestone {
                        ancilla_slot: k as u8,
                        suffix: AncillaItemReceiptSuffix::None,
                    },
                );
                return;
            }

            if self.ancilla_slot_view(k).step() != 2 {
                if self.ancilla_slot_view(k).step() != 0 {
                    self.ancilla_slot_view_mut(k).tick_z_velocity();
                }
                self.ancilla_move_z(k);
                if self.ancilla_slot_view(k).z() >= 0xf8 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value = 0x18;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_z(value);
                }
            }
        }

        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam = self.ancilla_receive_item_draw(
            k,
            x,
            y.wrapping_sub(self.ancilla_slot_view(k).z() as u16),
        );

        let aux_timer = self.ancilla_slot_view(k).aux_timer().wrapping_sub(1);
        let value = aux_timer;
        self.ancilla_slot_view_mut(k).set_aux_timer(value);
        if sign8(aux_timer) {
            let value = 9;
            self.ancilla_slot_view_mut(k).set_aux_timer(value);
            self.ancilla_slot_view_mut(k).add_l(1);
            if self.ancilla_slot_view(k).l() == 3 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_l(value);
            }
        }

        let t = if self.ancilla_slot_view(k).z() == 0 {
            if self.game_state.world.location.dungeon_room() == 6 {
                self.ancilla_slot_view(k).l().wrapping_add(4)
            } else {
                0
            }
        } else if self.ancilla_slot_view(k).z() < 0x20 {
            1
        } else {
            2
        };
        self.ancilla_draw_shadow(oam, t as usize, x, y.wrapping_add(12), 0x20);
    }

    pub(super) fn ancilla28_wish_pond_item(&mut self, k: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);

        if self.game_state.frame.submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
            self.follower_link_state_mut().set_picking_throw_state(2);
            self.follower_link_state_mut().clear_state_bits();
            self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
            self.ancilla_move_z(k);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            if sign8(self.ancilla_slot_view(k).z()) && self.ancilla_slot_view(k).z() < 228 {
                let value = 228;
                self.ancilla_slot_view_mut(k).set_z(value);
                let j = self.ancilla_slot_view(k).item_to_link() as usize;
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k)
                        .wrapping_add(if self.asset_u8(71, j) != 0 { 8 } else { 4 }),
                    self.ancilla_get_y(k).wrapping_add(18),
                );
                self.ancilla_transmute_to_splash(k);
                return;
            }
        }
        self.wish_pond_item_draw(k);
    }

    pub(super) fn ancilla42_happiness_pond_rupees(&mut self, k: usize) {
        self.follower_link_state_mut().set_picking_throw_state(2);
        self.follower_link_state_mut().clear_state_bits();
        for i in (0..=9).rev() {
            if self
                .game_state
                .effects
                .happiness_pond_rupees
                .rupee(i)
                .is_active()
            {
                self.hapiness_pond_rupees_execute_rupee(k, i);
                if self
                    .game_state
                    .effects
                    .happiness_pond_rupees
                    .rupee(i)
                    .step()
                    == 2
                {
                    self.happiness_pond_rupee_mut(i).clear();
                }
            }
        }
        for i in (0..=9).rev() {
            if self
                .game_state
                .effects
                .happiness_pond_rupees
                .rupee(i)
                .is_active()
            {
                return;
            }
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
    }

    fn hapiness_pond_rupees_execute_rupee(&mut self, k: usize, i: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);
        self.hapiness_pond_rupees_get_state(k, i);

        if self.ancilla_slot_view(k).step() != 0 {
            if self.game_state.frame.submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
                let value = 6;
                self.ancilla_slot_view_mut(k).set_timer(value);
                let value = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                if self.ancilla_slot_view(k).item_to_link() == 5 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                } else {
                    self.object_splash_draw(k);
                }
            } else {
                self.object_splash_draw(k);
            }
        } else if self.game_state.frame.submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).add_z_velocity((-2i8) as u8);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            self.ancilla_move_z(k);
            if sign8(self.ancilla_slot_view(k).z()) && self.ancilla_slot_view(k).z() < 0xe4 {
                let value = 0xe4;
                self.ancilla_slot_view_mut(k).set_z(value);
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k).wrapping_sub(4),
                    self.ancilla_get_y(k).wrapping_add(30),
                );
                let value = 0;
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                let value = 6;
                self.ancilla_slot_view_mut(k).set_timer(value);
                self.ancilla_sfx2_pan(k, 0x28);
                self.ancilla_slot_view_mut(k).add_step(1);
                self.object_splash_draw(k);
            } else {
                let value = 2;
                self.ancilla_slot_view_mut(k).set_work_byte_4(value);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_floor(value);
                self.wish_pond_item_draw(k);
            }
        } else {
            let value = 2;
            self.ancilla_slot_view_mut(k).set_work_byte_4(value);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_floor(value);
            self.wish_pond_item_draw(k);
        }
        self.hapiness_pond_rupees_save_state(i, k);
    }

    fn hapiness_pond_rupees_get_state(&mut self, j: usize, k: usize) {
        let pond = self
            .game_state
            .effects
            .happiness_pond_rupees
            .rupee(k)
            .snapshot();
        let mut ancilla = self.ancilla_slot_view_mut(j);
        ancilla.set_y_low(pond.y_low);
        ancilla.set_y_high(pond.y_high);
        ancilla.set_x_low(pond.x_low);
        ancilla.set_x_high(pond.x_high);
        ancilla.set_z(pond.z);
        ancilla.set_y_velocity(pond.y_velocity);
        ancilla.set_x_velocity(pond.x_velocity);
        ancilla.set_z_velocity(pond.z_velocity);
        ancilla.set_y_subpixel(pond.y_subpixel);
        ancilla.set_x_subpixel(pond.x_subpixel);
        ancilla.set_z_subpixel(pond.z_subpixel);
        ancilla.set_item_to_link(pond.item_to_link);
        ancilla.set_step(pond.step);
        ancilla.set_timer(pond.timer);
    }

    fn hapiness_pond_rupees_save_state(&mut self, k: usize, j: usize) {
        let ancilla = self.ancilla_slot_view(j);
        let snapshot = HappinessPondRupeeSnapshot {
            y_low: ancilla.y_low(),
            y_high: ancilla.y_high(),
            x_low: ancilla.x_low(),
            x_high: ancilla.x_high(),
            z: ancilla.z(),
            y_velocity: ancilla.y_velocity(),
            x_velocity: ancilla.x_velocity(),
            z_velocity: ancilla.z_velocity(),
            y_subpixel: ancilla.y_subpixel(),
            x_subpixel: ancilla.x_subpixel(),
            z_subpixel: ancilla.z_subpixel(),
            item_to_link: ancilla.item_to_link(),
            timer: ancilla.timer(),
            step: ancilla.step(),
        };
        self.happiness_pond_rupee_mut(k).store_snapshot(snapshot);
    }

    fn revival_fairy_monitor_hp(&mut self) {
        if (self.game_state.inventory.player_resources.current_health()
            == self.game_state.inventory.player_resources.health_capacity()
            || self.game_state.inventory.player_resources.current_health() == 0x38)
            && !self.hud_state().is_doing_heart_animation()
        {
            if self.game_state.player.follower_link.is_in_deep_water() {
                self.follower_link_state_mut().set_swim_direction_flags(4);
                self.follower_link_state_mut().set_handler_state(4);
            } else if self.game_state.player.follower_link.is_bunny() {
                self.follower_link_state_mut().set_handler_state(23);
                self.follower_link_state_mut().set_bunny_state(1);
                if self
                    .game_state
                    .enhanced_features
                    .has(REVIVAL_FAIRY_MONITOR_HP_FEATURES0_MISC_BUG_FIXES)
                {
                    self.load_gear_palettes_bunny();
                }
            } else {
                self.follower_link_state_mut().clear_handler_state();
            }
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut()
                .clear_faint_animation_active();
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_z(0);
            self.follower_link_state_mut().set_incapacitated_timer(0);
            for i in 0..5 {
                let value = 0;
                self.ancilla_slot_view_mut(i).set_ancilla_type(value);
            }
            return;
        }

        let k = 1;
        if self.ancilla_slot_view(k).step() == 0 {
            self.ancilla_slot_view_mut(k).tick_work_byte_3();
            if self.ancilla_slot_view(k).work_byte_3() == 0 {
                self.ancilla_slot_view_mut(k).add_work_byte_3(1);
                let value = 4;
                self.ancilla_slot_view_mut(k).set_z_velocity(value);
                self.ancilla_move_z(k);
                if self.ancilla_slot_view(k).z() >= 16 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value = 2;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                }
            }
        } else {
            self.ancilla_slot_view_mut(k).tick_k();
            if sign8(self.ancilla_slot_view(k).k()) {
                let value = 32;
                self.ancilla_slot_view_mut(k).set_k(value);
                let value = 0u8.wrapping_sub(self.ancilla_slot_view(k).z_velocity());
                self.ancilla_slot_view_mut(k).set_z_velocity(value);
            }
            self.ancilla_move_z(k);
        }
        let z = self.ancilla_slot_view(k).z() as u16;
        self.follower_link_state_mut().set_z(z);
    }

    fn revival_fairy_dust(&mut self) {
        let k = 2;
        if self.ancilla_slot_view(0).step() == 0 || self.ancilla_slot_view(k).step() == 2 {
            return;
        }
        self.ancilla_slot_view_mut(k).tick_work_byte_3();
        if !sign8(self.ancilla_slot_view(k).work_byte_3()) {
            return;
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_3(value);
        if !self.game_state.oam.has_sprite_sorting() {
            self.oam_allocate_from_region_a(16);
        } else {
            self.oam_allocate_from_region_d(16);
        }
        self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(self.ancilla_slot_view(k).aux_timer()) {
            let value = 3;
            self.ancilla_slot_view_mut(k).set_aux_timer(value);
            if self.ancilla_slot_view(k).item_to_link() == 9 {
                let value = 32;
                self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                self.ancilla_slot_view_mut(k).add_step(1);
                let value = 2;
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                return;
            }
            self.ancilla_slot_view_mut(k).add_item_to_link(1);
            let value =
                MAGIC_POWDER_FRAME_TIMERS[30 + self.ancilla_slot_view(k).item_to_link() as usize];
            self.ancilla_slot_view_mut(k).set_work_byte_25(value);
        }
        self.ancilla_magic_powder_draw(k);
    }

    pub(in crate::zelda_rtl) fn revival_fairy_main(&mut self) {
        let k = 0;
        let skip_draw = match self.ancilla_slot_view(k).step() {
            0 => {
                self.ancilla_slot_view_mut(k).tick_work_byte_3();
                if self.ancilla_slot_view(k).work_byte_3() == 0 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value = REVIVAL_FAIRY_MAIN_REVIVAL_FAIRY_STEP_TIMERS
                        [self.ancilla_slot_view(k).step() as usize];
                    self.ancilla_slot_view_mut(k).set_work_byte_3(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_k(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                } else {
                    self.ancilla_move_z(k);
                }
                false
            }
            1 => {
                self.ancilla_slot_view_mut(k).tick_work_byte_3();
                if self.ancilla_slot_view(k).work_byte_3() == 0 {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_x_velocity(value);
                } else {
                    if self.ancilla_slot_view(k).work_byte_3() == 0x4f
                        || self.ancilla_slot_view(k).work_byte_3() == 0x8f
                    {
                        self.ancilla_slot_view_mut(k).add_l(1);
                        self.ancilla_sfx2_pan(k, 0x31);
                    }
                    if self.ancilla_slot_view(k).l() != 0 {
                        self.ancilla_slot_view_mut(k).subtract_g(1);
                        if sign8(self.ancilla_slot_view(k).g()) {
                            let value = 5;
                            self.ancilla_slot_view_mut(k).set_g(value);
                            let value = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                            self.ancilla_slot_view_mut(k).set_item_to_link(value);
                            if self.ancilla_slot_view(k).item_to_link() == 3 {
                                let value = 0;
                                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                                let value = 0;
                                self.ancilla_slot_view_mut(k).set_l(value);
                            }
                        }
                    }
                    let value = self.ancilla_slot_view(k).z_velocity().wrapping_add(
                        if self.ancilla_slot_view(k).k() != 0 {
                            1
                        } else {
                            0xff
                        },
                    );
                    self.ancilla_slot_view_mut(k).set_z_velocity(value);
                    if abs8(self.ancilla_slot_view(k).z_velocity()) == 8 {
                        self.ancilla_slot_view_mut(k).toggle_k_bit0();
                    }
                    self.ancilla_move_z(k);
                }
                false
            }
            2 => {
                if self.ancilla_slot_view(k).z_velocity() < 24 {
                    self.ancilla_slot_view_mut(k).add_z_velocity(1);
                }
                if self.ancilla_slot_view(k).x_velocity() < 16 {
                    self.ancilla_slot_view_mut(k).add_x_velocity(1);
                }
                self.ancilla_move_x(k);
                self.ancilla_move_z(k);
                false
            }
            3 => true,
            _ => false,
        };

        if !skip_draw {
            self.oam_allocate_from_region_c(12);
            let (x, y) = self.ancilla_prep_oam_coord(k);
            let oam = self.game_state.oam.current_pointer_usize();
            let mut t =
                if self.ancilla_slot_view(k).step() == 1 && self.ancilla_slot_view(k).l() != 0 {
                    self.ancilla_slot_view(k).item_to_link().wrapping_add(1)
                } else {
                    0
                };
            if t != 0 {
                t = t.wrapping_add(1);
            } else {
                t = (self.game_state.frame.frame_counter >> 2) & 1;
            }
            self.ancilla_set_oam(
                oam,
                x,
                y.wrapping_sub(self.ancilla_slot_view(k).z() as i8 as i16 as u16),
                REVIVAL_FAIRY_MAIN_REVIVAL_FAIRY_TILE_CHARS[t as usize],
                0x74,
                2,
            );
            if self.game_state.oam.entry_y(oam) == 0xf0 {
                let value = 3;
                self.ancilla_slot_view_mut(k).set_step(value);
                self.increment_submodule();
                let main_screen_layers = self.game_state.display.ppu_scroll_copy.mapbak_tm();
                self.set_main_screen_layers(main_screen_layers);
            }
        }

        self.revival_fairy_dust();
        self.revival_fairy_monitor_hp();
    }

    fn ancilla_add_rupees(&mut self, k: usize) -> bool {
        let a = self.ancilla_slot_view(k).item_to_link();
        let amount = if (0x34..=0x36).contains(&a) {
            ANCILLA_ADD_RUPEES_RUPEE_GIFT_AMOUNTS[(a - 0x34) as usize]
        } else if a == 0x40 || a == 0x41 {
            ANCILLA_ADD_RUPEES_RUPEE_GIFT_AMOUNTS[(a - 0x40 + 3) as usize]
        } else if a == 0x46 {
            300
        } else if a == 0x47 {
            20
        } else {
            return false;
        };
        let rupees = self
            .game_state
            .inventory
            .player_resources
            .rupees_goal()
            .wrapping_add(amount);
        self.player_resources_mut().set_rupees_goal(rupees);
        true
    }

}

impl ZeldaState {
    pub(in crate::zelda_rtl) fn finish_native_receipt_prefix(&mut self, k: usize, tail: crate::cycle_models::receipt_prefix::ReceiptTail) {
        use crate::cycle_models::receipt_prefix::ReceiptTail;
        let _scope=crate::cycle_ledger::routine(0x08_c38a);
        match tail {
            ReceiptTail::Draw => {},
            ReceiptTail::Message => self.ancilla22_item_receipt_show_message(k),
            ReceiptTail::Movement => self.ancilla22_item_receipt_move_label_b(k),
            ReceiptTail::LabelA => {self.ancilla22_item_receipt_label_a(k);return;},
            ReceiptTail::Finish => {self.ancilla22_item_receipt_finish(k);return;},
            ReceiptTail::SongPoll => {
                crate::cycle_ledger::charge(46);
                if self.zelda_read_apui00_for_song_end_poll()!=0 {
                    crate::cycle_ledger::charge(74+30);
                    let v=self.ancilla_slot_view(k).aux_timer().wrapping_add(1);
                    self.ancilla_slot_view_mut(k).set_aux_timer(v);
                } else {
                    crate::cycle_ledger::charge(6+60);
                    self.ancilla22_item_receipt_show_message(k);
                }
            },
            ReceiptTail::Rupees => {
                // JSL has already retired; retain its callee, carry branch and caller suffix.
                let carry=self.ancilla_add_rupees(k);
                crate::cycle_ledger::charge(16);
                let sound=if carry {crate::cycle_ledger::charge(6);true} else {
                    crate::cycle_ledger::charge(64);
                    if self.ancilla_slot_view(k).item_to_link()!=0x17 {crate::cycle_ledger::charge(6);true}else{false}
                };
                if sound {crate::cycle_ledger::charge(62);self.ancilla_sfx3_near(15);crate::cycle_ledger::charge(22);}
                crate::cycle_ledger::charge(30);
                self.ancilla22_item_receipt_move_label_b(k);
            },
            ReceiptTail::Sound => {
                self.ancilla_sfx3_near(15);
                crate::cycle_ledger::charge(22+30);
                self.ancilla22_item_receipt_move_label_b(k);
            },
        }
        self.ancilla22_item_receipt_draw_and_update(k);
    }

}

impl ZeldaState {
    pub(in crate::zelda_rtl) fn finish_native_receipt_drawing(&mut self,k:usize,tail:crate::cycle_models::receipt_drawing::ReceiptDrawingTail) {
        use crate::cycle_models::receipt_drawing::ReceiptDrawingTail;
        match tail {
            ReceiptDrawingTail::Sparkle=>{self.ancilla_add_occasional_sparkle(k);self.ancilla_receipt_after_sparkle(k);},
            ReceiptDrawingTail::Graphics {source}=>{self.write_to4_bpp_buffer_at_7f4000(source);crate::cycle_ledger::charge(28);self.ancilla_receipt_coordinates_and_draw(k);},
            ReceiptDrawingTail::Draw=>{let x=read_le_u16(&self.ram,2);let y=read_le_u16(&self.ram,0);self.ancilla_receive_item_draw(k,x,y);},
        }
    }

}
