//! Sprite inactivity, damage, death, drops, carrying, stunning, and absorption.

use super::*;

impl ZeldaState {

    // void Sprite_Get16BitCoords(int k) {
    //   cur_sprite_x = sprite_x_lo[k] | sprite_x_hi[k] << 8;
    //   cur_sprite_y = sprite_y_lo[k] | sprite_y_hi[k] << 8;
    // }
    pub(in crate::zelda_rtl) fn sprite_get16_bit_coords(&mut self, k: usize) {
        // Cycle ledger: Sprite_Get16BitCoords $06:84C1 (m8 x8, JSR target):
        // four LDA abs,x / STA abs pairs + RTS, 298, input independent.
        crate::cycle_ledger::charge_routine(0x06_84c1, 298);
        let x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        self.sprite_workspace_mut().set_current_sprite_x(x);
        self.sprite_workspace_mut().set_current_sprite_y(y);
    }

    // void Sprite_inactiveSprite(int k) {  // 868510
    //   if (!player_is_indoors) {
    //     sprite_N_word[k] = 0xffff;
    //   } else {
    //     sprite_N[k] = 0xff;
    //   }
    // }
    pub(in crate::zelda_rtl) fn sprite_inactive_sprite(&mut self, k: usize) {
        // Cycle ledger: Sprite_inactiveSprite $06:8510 (m8 x8; reached by the
        // Sprite_ExecuteSingle BEQ, not a JSR). $06:8510-8512 LDA $1b : BNE
        // (40), then outdoors $06:8514-851F TXA ASL TAY LDA #$ff STA abs,y
        // STA abs,y RTS (176) or BNE taken (+6) into $06:8520-8525 LDA #$ff
        // STA abs,x RTS (96).
        // $06:84E5 BEQ enters this tail without a new call frame. Keep its
        // self work in $06:84E2 for source/native routine comparisons.
        let _scope = crate::cycle_ledger::tail_routine(0x06_8510, 0x06_84e2);
        if self.game_state.world.location.is_outdoors() {
            crate::cycle_ledger::charge(40 + 176);
            self.sprite_slot_view_mut(k).set_n_word(0xffff);
        } else {
            crate::cycle_ledger::charge(40 + 6 + 96);
            let value = 0xff;
            self.sprite_slot_view_mut(k).set_n(value);
        }
    }

    // void Sprite_KillSelf(int k) {  // 89f1f8
    //   if (!(sprite_defl_bits[k] & 0x40) && player_is_indoors)
    //     return;
    //   sprite_state[k] = 0;
    //   uint16 blk = sprite_N_word[k];
    //   g_ram[0] = blk;
    //   WORD(g_ram[1]) = (blk >> 3) + 0xef80;
    //   uint8 loadedmask = (0x80 >> (blk & 7));
    //   uint16 addr = 0xEF80 + (blk >> 3);
    //   uint8 *loadedp = &g_ram[addr + 0x10000];
    //   if (blk < 0xffff)
    //     *loadedp &= ~loadedmask;
    //   if (!player_is_indoors)
    //     sprite_N_word[k] = 0xffff;
    //   else
    //     sprite_N[k] = 0xff;
    // }
    pub(in crate::zelda_rtl) fn sprite_kill_self(&mut self, k: usize) {
        let mut work = crate::cycle_models::sprite_kill::KillWork::new();
        while self.retire_sprite_kill_instruction(k, &mut work).is_some() {}
    }

    pub(super) fn retire_sprite_kill_instruction(
        &mut self,
        k: usize,
        work: &mut crate::cycle_models::sprite_kill::KillWork,
    ) -> Option<u32> {
        use crate::cycle_models::sprite_kill::{Inputs, Publication};
        let scratch = [self.ram[0], self.ram[1], self.ram[2], self.ram[3]];
        let pointer = usize::from(u16::from_le_bytes([scratch[1], scratch[2]]));
        let loaded = if work.reads_loaded() {
            assert_eq!(scratch[3], 0x7f, "KillSelf load pointer left its WRAM bank");
            const WRAM_BANK_BYTES: usize = 0x10000;
            self.ram[WRAM_BANK_BYTES + pointer]
        } else { 0 };
        let slot = self.sprite_slot_view(k);
        let instruction = work.step(Inputs {
            slot: k as u8, deflection: slot.deflection_bits(),
            indoors: self.game_state.world.location.indoor_flag(),
            block: slot.n_word(), scratch, loaded,
        })?;
        crate::cycle_ledger::charge_routine(0x09_f1f8, u64::from(instruction.master_cycles));
        match instruction.publication {
            Some(Publication::ClearState) => self.sprite_slot_view_mut(k).set_state(0),
            Some(Publication::ScratchWord { offset, value }) => {
                let mut workspace = self.sprite_workspace_mut();
                workspace.set_oam_prep_scratch_byte(offset, value as u8);
                workspace.set_oam_prep_scratch_byte(offset + 1, (value >> 8) as u8);
            }
            Some(Publication::PointerBank(value)) => {
                self.sprite_workspace_mut().set_oam_prep_scratch_byte(3, value);
            }
            Some(Publication::Loaded(value)) => {
                assert_eq!(scratch[3], 0x7f, "KillSelf store pointer left its WRAM bank");
                self.store_overworld_sprite_loaded_byte_wrapped(pointer as u16, value);
            }
            Some(Publication::BlockLow(value)) => {
                let word = (self.sprite_slot_view(k).n_word() & 0xff00) | u16::from(value);
                self.sprite_slot_view_mut(k).set_n_word(word);
            }
            Some(Publication::BlockHigh(value)) => {
                let word = (self.sprite_slot_view(k).n_word() & 0xff) | (u16::from(value) << 8);
                self.sprite_slot_view_mut(k).set_n_word(word);
            }
            Some(Publication::IndoorBlock(value)) => self.sprite_slot_view_mut(k).set_n(value),
            None => {}
        }
        Some(instruction.master_cycles)
    }

    // void Sprite_HitTimer31(int k) {
    //   if (sprite_type[k] != 0x7a || is_in_dark_world)
    //     return;
    //   if (sprite_health[k] <= sprite_give_damage[k]) {
    //     dialogue_message_index = 0x140;
    //     Sprite_ShowMessageMinimal();
    //   }
    // }
    pub(in crate::zelda_rtl) fn sprite_hit_timer31(&mut self, k: usize) {
        // Cycle ledger: this is the inline $06:8453-8479 stretch of
        // Sprite_TimersAndOam (no routine of its own). $06:8453-8459 PHA, LDA
        // $e20,x, CMP #$7a, BNE (86); a foreign type takes the BNE (+6) to
        // $06:8479 PLA (28).
        if self.sprite_slot_view(k).sprite_type() != 0x7a {
            crate::cycle_ledger::charge(86 + 6 + 28);
        } else if self.game_state.world.region.is_in_dark_world() {
            // $06:845B-845E LDA $0fff : BNE taken (48 + 6), then PLA (28).
            crate::cycle_ledger::charge(86 + 48 + 6 + 28);
        } else {
            // $06:845B-845E not taken (48), $06:8460-8467 LDA $e50,x SEC SBC
            // $ce2,x BEQ (94): equal takes the BEQ (+6) into $06:846B; less
            // falls through $06:8469 BCS not taken (16) into $06:846B (158,
            // the JSL Sprite_ShowMessageMinimal included); greater takes the
            // BCS (16 + 6). Then $06:8479 PLA (28).
            let health = self.sprite_slot_view(k).health();
            let incoming_damage = self.sprite_slot_view(k).incoming_damage();
            crate::cycle_ledger::charge(86 + 48 + 94);
            crate::cycle_ledger::charge(if health == incoming_damage {
                6 + 158 + 28
            } else if health < incoming_damage {
                16 + 158 + 28
            } else {
                16 + 6 + 28
            });
        }
        if self.sprite_slot_view(k).sprite_type() != 0x7a
            || self.game_state.world.region.is_in_dark_world()
        {
            return;
        }
        if self.sprite_slot_view(k).health() <= self.sprite_slot_view(k).incoming_damage() {
            self.dialogue_message_index_mut().set_value(0x140);
            self.sprite_show_message_minimal_c();
        }
    }

    // bool Sprite_TrackBodyToHead(int k) {  // 85dca2
    //   if (sprite_head_dir[k] != sprite_D[k]) {
    //     if (frame_counter & 0x1f)
    //       return false;
    //     if (!((sprite_head_dir[k] ^ sprite_D[k]) & 2)) {
    //       sprite_D[k] = (((k ^ frame_counter) >> 5 | 2) & 3) ^ (sprite_head_dir[k] & 2);
    //       return false;
    //     }
    //   }
    //   sprite_D[k] = sprite_head_dir[k];
    //   return true;
    // }
    pub(in crate::zelda_rtl) fn sprite_track_body_to_head(&mut self, k: usize) -> bool {
        if self.sprite_slot_view(k).head_direction() != self.sprite_slot_view(k).direction() {
            if (self.game_state.frame.frame_counter & 0x1f) != 0 {
                return false;
            }
            if ((self.sprite_slot_view(k).head_direction() ^ self.sprite_slot_view(k).direction())
                & 2)
                == 0
            {
                let value = ((((k as u8) ^ self.game_state.frame.frame_counter) >> 5) | 2) & 3
                    ^ (self.sprite_slot_view(k).head_direction() & 2);
                self.sprite_slot_view_mut(k).set_direction(value);
                return false;
            }
        }
        let value = self.sprite_slot_view(k).head_direction();
        self.sprite_slot_view_mut(k).set_direction(value);
        true
    }

    // bool Sprite_CheckIfLinkIsBusy() {  // 87f4d0
    //   if (link_auxiliary_state | link_pose_for_item | (link_state_bits & 0x80))
    //     return true;
    //   for (int i = 4; i >= 0; i--) {
    //     if (ancilla_type[i] == 0x27)
    //       return true;
    //   }
    //   return false;
    // }
    pub(in crate::zelda_rtl) fn sprite_check_if_link_is_busy(&self) -> bool {
        if self.game_state.player.follower_link.has_auxiliary_state()
            || self.game_state.player.follower_link.item_hold_pose() != 0
            || self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
        {
            return true;
        }
        for i in (0..=4usize).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x27 {
                return true;
            }
        }
        false
    }

    // bool Sprite_ReturnIfInactive(int k) {
    //   return (sprite_state[k] != 9 || modal_pause_flag || submodule_index
    //           || !(sprite_defl_bits[k] & 0x80) && sprite_pause[k]);
    // }
    // Note: in C this returns true when the caller should bail. Same here.
    pub(in crate::zelda_rtl) fn sprite_return_if_inactive(&self, k: usize) -> bool {
        if self.lag_free {
            // $06:D9EC..DA08, the unpriced raw-call entry. The already
            // priced bank-5/1A entries bypass this charge below.
            return self.sprite_return_if_inactive_with_cost(k, 0x06_d9ec);
        }
        self.sprite_inactive_predicate(k)
    }

    fn sprite_inactive_predicate(&self, k: usize) -> bool {
        if self.sprite_slot_view(k).state() != 9 {
            return true;
        }
        if self.game_state.frame.modal_pause_flag != 0 || self.game_state.frame.submodule != 0 {
            return true;
        }
        (self.sprite_slot_view(k).deflection_bits() & 0x80) == 0
            && self.sprite_slot_view(k).pause() != 0
    }

    /// `Sprite_ReturnIfInactive_` $05:F94E, the bank-5 double-return wrapper
    /// (cycle ledger only; the result is `sprite_return_if_inactive`'s).
    /// $05:F94E LDA $DD0,x : CMP #$09 : BNE (64; a non-active state takes it
    /// +6 into the double return), $05:F955 LDA $0FC1 : BNE (48; +6 when
    /// modal-paused), $05:F95A LDA $11 : BNE (40; +6 in a submodule),
    /// $05:F95E LDA $CAA,x : BMI (48; +6 into the plain RTS for a
    /// pause-immune sprite), $05:F963 LDA $F00,x : BEQ (48; +6 into the plain
    /// RTS when not paused). The plain return is $05:F96A RTS (42). The
    /// double return is $05:F968 PLA : PLA : RTS: the profiler closes the
    /// wrapper's frame after the first PLA (28), so the second PLA and the
    /// RTS (70) are measured in the caller's frame and charged there, as
    /// with the coordinate-prep wrappers (host 2297: wrapper self 186 in a
    /// dialogue submodule, the caller +70).
    pub(in crate::zelda_rtl) fn sprite_return_if_inactive_bank5(&self, k: usize) -> bool {
        self.sprite_return_if_inactive_with_cost(k, 0x05_f94e)
    }

    /// $1A:F954..F970 has the same branch costs and double-return ownership
    /// as the bank-5 wrapper above.
    pub(in crate::zelda_rtl) fn sprite_return_if_inactive_bank1a(&self, k: usize) -> bool {
        self.sprite_return_if_inactive_with_cost(k, 0x1a_f954)
    }

    fn sprite_return_if_inactive_with_cost(&self, k: usize, address: u32) -> bool {
        let inactive = self.sprite_inactive_predicate(k);
        {
            let _wrapper = crate::cycle_ledger::routine(address);
            let slot = self.sprite_slot_view(k);
            crate::cycle_ledger::charge(if slot.state() != 9 {
                64 + 6 + 28
            } else {
                64 + self.sprite_pause_test_master_cycles(k)
            });
        }
        if inactive {
            crate::cycle_ledger::charge(28 + 42);
        }
        inactive
    }

    fn sprite_pause_test_master_cycles(&self, k: usize) -> u64 {
        if self.game_state.frame.modal_pause_flag != 0 {
            48 + 6 + 28
        } else if self.game_state.frame.submodule != 0 {
            48 + 40 + 6 + 28
        } else if self.sprite_slot_view(k).deflection_bits() & 0x80 != 0 {
            48 + 40 + 48 + 6 + 42
        } else if self.sprite_slot_view(k).pause() == 0 {
            48 + 40 + 48 + 48 + 6 + 42
        } else {
            48 + 40 + 48 + 48 + 28
        }
    }

    /// Resume ReturnIfInactive after its state-9 test, at $05:F955.
    pub(in crate::zelda_rtl) fn sprite_return_if_paused_bank5(&self, k: usize) -> bool {
        let paused = self.sprite_paused_predicate(k);
        {
            let _wrapper = crate::cycle_ledger::routine(0x05_f955);
            crate::cycle_ledger::charge(self.sprite_pause_test_master_cycles(k));
        }
        if paused { crate::cycle_ledger::charge(28 + 42); }
        paused
    }

    // bool Sprite_ReturnIfPaused(int k) {  // 86d9f3
    //   return (modal_pause_flag || submodule_index || !(sprite_defl_bits[k] & 0x80) && sprite_pause[k]);
    // }
    pub(in crate::zelda_rtl) fn sprite_return_if_paused(&self, k: usize) -> bool {
        if self.lag_free {
            // $06:D9F3..DA08: omit the state-9 test for this entry. The
            // double-return tail's second PLA/RTS belongs to the caller.
            let paused = self.sprite_paused_predicate(k);
            {
                let _scope = crate::cycle_ledger::routine(0x06_d9f3);
                crate::cycle_ledger::charge(self.sprite_pause_test_master_cycles(k));
            }
            if paused { crate::cycle_ledger::charge(28 + 42); }
            return paused;
        }
        self.sprite_paused_predicate(k)
    }

    fn sprite_paused_predicate(&self, k: usize) -> bool {
        self.game_state.frame.modal_pause_flag != 0
            || self.game_state.frame.submodule != 0
            || ((self.sprite_slot_view(k).deflection_bits() & 0x80) == 0
                && self.sprite_slot_view(k).pause() != 0)
    }

    // bool Sprite_ReturnIfPhasingOut(int k) {  // 86d0ed
    //   if (!sprite_stunned[k] || (submodule_index | modal_pause_flag))
    //     return false;
    //   if (!(frame_counter & 1))
    //     sprite_stunned[k]--;
    //   uint8 a = sprite_stunned[k];
    //   if (a == 0)
    //     sprite_state[k] = 0;
    //   else if (a >= 0x28 || (a & 1) != 0)
    //     return false;
    //   PrepOamCoordsRet info;
    //   Sprite_PrepOamCoordOrDoubleRet(k, &info);
    //   return true;
    // }
    pub(in crate::zelda_rtl) fn sprite_return_if_phasing_out(&mut self, k: usize) -> bool {
        if self.sprite_slot_view(k).stunned() == 0
            || (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) != 0
        {
            return false;
        }
        if (self.game_state.frame.frame_counter & 1) == 0 {
            let value = self.sprite_slot_view(k).stunned().wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_stunned(value);
        }
        let a = self.sprite_slot_view(k).stunned();
        if a == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        } else if a >= 0x28 || (a & 1) != 0 {
            return false;
        }
        let _ = self.sprite_prep_oam_coord_or_double_ret_from(k, PrepOamCoordEntry::Safe);
        true
    }

    // bool SpriteDraw_AbsorbableTransient(int k, bool transient) {  // 86d22f
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_absorbable_transient(&mut self, k: usize, transient: bool) -> bool {
        // $06:D22F..D289: selection only; JMP drawing tails remain separate.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(
            if transient { 0x06_d22f } else { 0x06_d232 }));
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        if transient { blocks.block(0x06d22f); }
        if transient && self.sprite_return_if_phasing_out(k) {
            return false;
        }
        blocks.block(0x06d232);
        if self.game_state.oam.has_sprite_sorting() {
            blocks.taken(0x06d235);
        } else {
            blocks.block(0x06d237);
            if self.game_state.world.location.is_outdoors() { blocks.taken(0x06d239); }
            else { blocks.block(0x06d23b); }
        }
        if !self.game_state.oam.has_sprite_sorting() && self.game_state.world.location.is_indoors()
        {
            let value = 0x30;
            self.sprite_slot_view_mut(k).set_object_priority(value);
        }
        blocks.block(0x06d240);
        if self.game_state.sprites.system.chr_halfslot_state() >= 3 {
            blocks.taken(0x06d245);
            blocks.block(0x06d289);
            return false;
        }
        blocks.block(0x06d247);
        if self.sprite_slot_view(k).delay_aux2() != 0 {
            blocks.block(0x06d24c);
            self.oam_allocate_from_region_c(12);
        } else { blocks.taken(0x06d24a); }
        blocks.block(0x06d252);
        if self.sprite_slot_view(k).e() != 0 {
            blocks.taken(0x06d255);
            blocks.block(0x06d287);
            // First PLA closes this frame; second PLA/RTS belongs to its caller.
            drop(_scope);
            blocks.block(0x06d288);
            blocks.block(0x06d289);
            if self.game_state.enhanced_features.has(4096) {
                let value = 0;
                self.sprite_slot_view_mut(k).set_b(value);
            }
            return true;
        }

        let j = self.sprite_slot_view(k).sprite_type().wrapping_sub(0xd8) as usize;
        let a = SPRITE_DRAW_ABSORBABLE_TRANSIENT_ABSORBABLE_GFX_BY_TYPE[j];
        blocks.block(0x06d257);
        // The source tables use the raw type as Y ($D144/$D135 + type).
        if 0x44 + u16::from(self.sprite_slot_view(k).sprite_type()) > 0xff {
            blocks.crossing(0x06d25a);
        }
        if a != 0 {
            blocks.block(0x06d25f);
            self.sprite_draw_numbered_absorbable(k, i32::from(a));
            return false;
        }
        blocks.taken(0x06d25d);

        let t = SPRITE_DRAW_ABSORBABLE_TRANSIENT_ABSORBABLE_OAM_EXT_SIZE_BY_TYPE[j];
        blocks.block(0x06d262);
        if 0x35 + u16::from(self.sprite_slot_view(k).sprite_type()) > 0xff {
            blocks.crossing(0x06d262);
        }
        if t == 0 {
            blocks.block(0x06d267);
            self.sprite_draw_single_small(k);
            return false;
        }
        blocks.taken(0x06d265);
        blocks.block(0x06d26a);
        if t == 2 {
            blocks.block(0x06d26e);
            if self.sprite_slot_view(k).sprite_type() == 0xe6 {
                blocks.block(0x06d275);
                if self.sprite_slot_view(k).subtype() == 1 {
                    blocks.taken(0x06d27a);
                    blocks.block(0x06d284);
                    self.sprite_draw_thin_and_tall(k);
                    return false;
                }
                blocks.block(0x06d27c);
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            } else { blocks.taken(0x06d273); }
            blocks.block(0x06d281);
            self.sprite_draw_single_large(k);
            return false;
        }
        blocks.taken(0x06d26c);
        blocks.block(0x06d284);
        self.sprite_draw_thin_and_tall(k);
        false
    }

    // void Sprite_DrawNumberedAbsorbable(int k, int a) {  // 86d2fa
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_numbered_absorbable(&mut self, k: usize, a: i32) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let base = ((a - 1) * 3).max(0) as usize;
        let n = if self.sprite_slot_view(k).head_direction() < 1 {
            2
        } else {
            1
        };
        for i in (0..=n).rev() {
            let j = (base + i).min(SPRITE_DRAW_NUMBERED_ABSORBABLE_CHARS.len() - 1);
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(SPRITE_DRAW_NUMBERED_ABSORBABLE_X_OFFSETS[j] as u16),
                y.wrapping_add(SPRITE_DRAW_NUMBERED_ABSORBABLE_Y_OFFSETS[j] as u16),
                SPRITE_DRAW_NUMBERED_ABSORBABLE_CHARS[j],
                flags,
                SPRITE_DRAW_NUMBERED_ABSORBABLE_EXT_SIZES[j],
            );
            oam += 4;
        }
        let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // void Sprite_HalveSpeed_XY(int k) {
    //   sprite_x_vel[k] = (int8)sprite_x_vel[k] >> 1;
    //   sprite_y_vel[k] = (int8)sprite_y_vel[k] >> 1;
    // }
    pub(in crate::zelda_rtl) fn sprite_halve_speed_xy(&mut self, k: usize) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.halve_x_velocity();
        sprite.halve_y_velocity();
    }

    // void Sprite_ApplyRicochet(int k) {  // 86e229
    //   Sprite_InvertSpeed_XY(k);
    //   Sprite_HalveSpeed_XY(k);
    //   ThrowableScenery_TransmuteIfValid(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_apply_ricochet(&mut self, k: usize) {
        self.sprite_invert_speed_xy(k);
        self.sprite_halve_speed_xy(k);
        self.throwable_scenery_transmute_if_valid(k);
    }

    // void ThrowableScenery_TransmuteIfValid(int k) {  // 86e22f
    //   if (sprite_type[k] != 0xec)
    //     return;
    //   repulsespark_timer = 0;
    //   ThrowableScenery_TransmuteToDebris(k);
    // }
    pub(in crate::zelda_rtl) fn throwable_scenery_transmute_if_valid(&mut self, k: usize) {
        if self.sprite_slot_view(k).sprite_type() != 0xec {
            return;
        }
        self.garnish_state_mut().set_repulsespark_timer(0);
        self.throwable_scenery_transmute_to_debris(k);
    }

    // void ThrowableScenery_TransmuteToDebris(int k) {  // 86e239
    //   uint8 a = sprite_graphics[k];
    //   if (a != 0) {
    //     BYTE(dung_secrets_unk1) = a;
    //     Sprite_SpawnSecret(k);
    //     BYTE(dung_secrets_unk1) = 0;
    //   }
    //   a = player_is_indoors ? 0 : sprite_C[k];
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, kSprite_Func21_Sfx[a]);
    //   Sprite_ScheduleForBreakage(k);
    // }
    pub(in crate::zelda_rtl) fn throwable_scenery_transmute_to_debris(&mut self, k: usize) {
        let mut a = self.sprite_slot_view(k).graphics();
        if a != 0 {
            self.dungeon_secret_scratch_mut().set_pending_kind(a);
            self.sprite_spawn_secret(k);
            self.dungeon_secret_scratch_mut().clear_pending_kind();
        }
        a = if self.game_state.world.location.is_indoors() {
            0
        } else {
            self.sprite_slot_view(k).c()
        };
        self.set_sound_effect_1(0);
        self.sprite_sfx_queue_sfx2_with_pan(
            k,
            THROWABLE_SCENERY_TRANSMUTE_TO_DEBRIS_THROWN_SPRITE_IMPACT_SFX[a as usize],
        );
        self.sprite_schedule_for_breakage(k);
    }

    // void Sprite_Func18(int k, uint8 new_type) {  // 86edcb
    //   sprite_type[k] = new_type;
    //   SpritePrep_LoadProperties(k);
    //   Sprite_SpawnPoofGarnish(k);
    //   sound_effect_2 = 0;
    //   SpriteSfx_QueueSfx3WithPan(k, 0x32);
    //   sprite_hit_timer[k] = 0;
    //   sprite_give_damage[k] = 0;
    // }
    pub(in crate::zelda_rtl) fn sprite_func18(&mut self, k: usize, new_type: u8) {
        let value = new_type;
        self.sprite_slot_view_mut(k).set_sprite_type(value);
        self.sprite_prep_load_properties(k);
        self.sprite_spawn_poof_garnish(k);
        self.set_sound_effect_2(0);
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
        let value = 0;
        self.sprite_slot_view_mut(k).set_hit_timer(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_incoming_damage(value);
    }

    // void Sprite_Func15(int k, int a) {  // 86ed25
    //   damage_type_determiner = a;
    //   Sprite_ApplyCalculatedDamage(k, a == 8 ? 0x35 : 0x20);
    // }
    pub(in crate::zelda_rtl) fn sprite_func15(&mut self, k: usize, a: u8) {
        self.sprite_battle_mut().set_damage_type_determiner(a);
        self.sprite_apply_calculated_damage(k, if a == 8 { 0x35 } else { 0x20 });
    }

    // void Sprite_CalculateSwordDamage(int k) {  // 86ed3f
    //   if (sprite_flags3[k] & 0x40)
    //     return;
    //   sprite_unk1[k] = link_is_running;
    //   uint8 a = link_sword_type - 1;
    //   if (!link_is_running)
    //     a |= sign8(button_b_frames) ? 4 : sign8(button_b_frames - 9) ? 0 : 8;
    //   damage_type_determiner = kSprite_Func14_Damage[a];
    //   if (link_item_in_hand & 10)
    //     damage_type_determiner = 3;
    //   link_sword_delay_timer = 4;
    //   set_when_damaging_enemies = 16;
    //   Sprite_ApplyCalculatedDamage(k, 0x9d);
    // }
    pub(in crate::zelda_rtl) fn sprite_calculate_sword_damage(&mut self, k: usize) {
        if self.sprite_slot_view(k).flags3() & 0x40 != 0 {
            return;
        }
        let is_running = self.game_state.player.follower_link.is_running();
        let item_in_hand_has_sword_mask = self.game_state.player.follower_link.item_in_hand_has(10);
        let value = u8::from(is_running);
        self.sprite_slot_view_mut(k).set_draw_work_byte_1(value);
        let mut a = self.game_state.inventory.items.sword_type().wrapping_sub(1);
        if !is_running {
            a |= if sign8(self.game_state.player.follower_link.button_b_frames()) {
                4
            } else if sign8(
                self.game_state
                    .player
                    .follower_link
                    .button_b_frames()
                    .wrapping_sub(9),
            ) {
                0
            } else {
                8
            };
        }
        self.sprite_battle_mut().set_damage_type_determiner(
            SPRITE_CALCULATE_SWORD_DAMAGE_SPRITE_DAMAGE_BY_PLAYER_WEAPON[a as usize],
        );
        if item_in_hand_has_sword_mask {
            self.sprite_battle_mut().set_damage_type_determiner(3);
        }
        self.follower_link_state_mut().set_sword_delay_timer(4);
        self.sprite_battle_mut().set_damaging_enemies_timer(16);
        self.sprite_apply_calculated_damage(k, 0x9d);
    }

    // void Sprite_ApplyCalculatedDamage(int k, int a) {  // 86ed89
    //   if ((sprite_flags3[k] & 0x40) || sprite_type[k] >= 0xD8)
    //     return;
    //   uint8 dmg = kEnemyDamages[damage_type_determiner * 8 | enemy_damage_data[sprite_type[k] * 16 | damage_type_determiner]];
    //   Sprite_GiveDamage(k, dmg, a);
    // }
    pub(in crate::zelda_rtl) fn sprite_apply_calculated_damage(&mut self, k: usize, a: u8) {
        if self.sprite_slot_view(k).flags3() & 0x40 != 0
            || self.sprite_slot_view(k).sprite_type() >= 0xd8
        {
            return;
        }
        let damage_type = self.game_state.sprite_battle.damage_type_determiner();
        let dmg = self.sprite_calculated_damage(k, damage_type);
        self.sprite_give_damage(k, dmg, a);
    }

    fn sprite_calculated_damage(&self, k: usize, damage_type: u8) -> u8 {
        let damage_type = usize::from(damage_type);
        let enemy_damage_index = self.sprite_slot_view(k).sprite_type() as usize * 16 + damage_type;
        SPRITE_APPLY_CALCULATED_DAMAGE_ENEMY_CONTACT_DAMAGE_BY_TYPE[(damage_type * 8)
            | self
                .game_state
                .sprites
                .enemy_damage_subclasses
                .entry(enemy_damage_index) as usize]
    }

    /// The carry `Sprite_Func15(k, a)` ($06:ED25) returns with on the paths
    /// that leave the sprite's incoming damage clear, for `a` 0 or 7 (taken
    /// before the call). `$06:ED0B ADC #$04` adds it into the repulse spark's
    /// x without `CLC`. The magic-powder transformations (damage $F9/$FA) are
    /// not modeled and count as clear.
    pub(super) fn sprite_func15_carry_out(&self, k: usize, a: u8) -> bool {
        let sprite = self.sprite_slot_view(k);
        let ty = sprite.sprite_type();
        if sprite.flags3() & 0x40 != 0 {
            return false; // `CPY #8` with a < 8.
        }
        if ty >= 0xd8 {
            return true; // `CMP #$D8 : BCC`.
        }
        match self.sprite_calculated_damage(k, a) {
            // `CMP #$0A` on the damage class, or the stun table's
            // `CPY #$09` / `CPY #$53` / `CPY #$18` on the sprite type.
            0 => sprite.flags() & 4 != 0 && (ty == 9 || ty >= 0x18),
            // `LDA $0DD0,X : CMP #$0B : BEQ` into the clearing tail.
            dmg if dmg >= 0xfe && sprite.state() == 11 => true,
            _ => false,
        }
    }

    // void Sprite_GiveDamage(int k, uint8 dmg, uint8 r0_hit_timer) {  // 86edc5
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_give_damage(&mut self, k: usize, dmg: u8, r0_hit_timer: u8) {
        if dmg == 249 {
            self.sprite_func18(k, 0xe3);
            return;
        }
        if dmg == 250 {
            self.sprite_func18(k, 0x8f);
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 32;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = 8;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_f(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_health(value);
            let value = 1;
            self.sprite_slot_view_mut(k).set_bump_damage(value);
            let value = 1;
            self.sprite_slot_view_mut(k).set_flags5(value);
            return;
        }
        if dmg >= self.sprite_slot_view(k).incoming_damage() {
            let value = dmg;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
        }
        if dmg == 0 {
            if self.game_state.sprite_battle.damage_type_determiner() != 10 {
                if self.sprite_slot_view(k).flags() & 4 != 0 {
                    self.sprite_set_damage_stun(k);
                    return;
                }
                self.follower_link_state_mut().clear_sword_delay_timer();
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            return;
        }
        if dmg >= 254 && self.sprite_slot_view(k).state() == 11 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            return;
        }
        if self.sprite_slot_view(k).sprite_type() == 0x9a
            && self.sprite_slot_view(k).incoming_damage() < 0xf0
        {
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 4;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
            return;
        }
        if self.sprite_slot_view(k).sprite_type() == 0x1b {
            self.sprite_sfx_queue_sfx2_with_pan(k, 5);
            self.sprite_schedule_for_breakage(k);
            self.sprite_place_weapon_tink(k);
            return;
        }
        let value = r0_hit_timer;
        self.sprite_slot_view_mut(k).set_hit_timer(value);
        if self.sprite_slot_view(k).sprite_type() != 0x92 || self.sprite_slot_view(k).c() >= 3 {
            let sfx = if self.sprite_slot_view(k).flags() & 2 != 0 {
                0x21
            } else if self.sprite_slot_view(k).flags5() & 0x10 != 0 {
                0x1c
            } else {
                8
            };
            self.set_sound_effect_2_with_sprite_pan(k, sfx);
        }
        self.sprite_set_damage_stun(k);
    }

    fn sprite_set_damage_stun(&mut self, k: usize) {
        let ty = self.sprite_slot_view(k).sprite_type();
        let value = if self.game_state.sprite_battle.damage_type_determiner() >= 13 {
            0
        } else if ty == 9 {
            20
        } else if ty == 0x53 || ty == 0x18 {
            11
        } else {
            15
        };
        self.sprite_slot_view_mut(k).set_f(value);
    }

    // void Sprite_ScheduleForBreakage(int k) {  // 86e25a
    //   sprite_delay_main[k] = 31;
    //   sprite_state[k] = 6;
    //   sprite_flags2[k] += 4;
    // }
    pub(in crate::zelda_rtl) fn sprite_schedule_for_breakage(&mut self, k: usize) {
        let value = 31;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let value = 6;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = self.sprite_slot_view(k).flags2().wrapping_add(4);
        self.sprite_slot_view_mut(k).set_flags2(value);
    }

    // void Sprite_ZeroVelocity_XY(int k) {  // 86cf5d
    //   sprite_x_vel[k] = sprite_y_vel[k] = 0;
    // }
    pub(in crate::zelda_rtl) fn sprite_zero_velocity_xy(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let value = self.sprite_slot_view(k).y_velocity();
        self.sprite_slot_view_mut(k).set_x_velocity(value);
    }

    // void Sprite_Invert_XY_Speeds(int k) {
    //   sprite_x_vel[k] = -sprite_x_vel[k];
    //   sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(in crate::zelda_rtl) fn sprite_invert_xy_speeds(&mut self, k: usize) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.negate_x_velocity();
        sprite.negate_y_velocity();
    }

    // void Sprite_BounceOffWall(int k) {  // 86d9c0
    //   if (sprite_wallcoll[k] & 3)
    //     sprite_x_vel[k] = -sprite_x_vel[k];
    //   if (sprite_wallcoll[k] & 12)
    //     sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(in crate::zelda_rtl) fn sprite_bounce_off_wall(&mut self, k: usize) {
        if (self.sprite_slot_view(k).wall_collision() & 3) != 0 {
            self.sprite_slot_view_mut(k).negate_x_velocity();
        }
        if (self.sprite_slot_view(k).wall_collision() & 12) != 0 {
            self.sprite_slot_view_mut(k).negate_y_velocity();
        }
    }

    // void Sprite_InvertSpeed_XY(int k) {  // 86d9d5
    //   sprite_x_vel[k] = -sprite_x_vel[k];
    //   sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(in crate::zelda_rtl) fn sprite_invert_speed_xy(&mut self, k: usize) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.negate_x_velocity();
        sprite.negate_y_velocity();
    }

    // void Sprite_MoveZ(int k) {
    //   uint16 z = (sprite_z[k] << 8 | sprite_z_subpos[k]) + ((int8)sprite_z_vel[k] << 4);
    //   sprite_z_subpos[k] = z;
    //   sprite_z[k] = z >> 8;
    // }
    pub(in crate::zelda_rtl) fn sprite_move_z(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_z();
    }

    // void Sprite_ApplySpeedTowardsLink(int k, uint8 vel) {
    //   ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, vel);
    //   sprite_x_vel[k] = pt.x;
    //   sprite_y_vel[k] = pt.y;
    // }
    pub(in crate::zelda_rtl) fn sprite_apply_speed_towards_link(&mut self, k: usize, vel: u8) {
        let pt = self.sprite_project_speed_towards_link(k, vel);
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.set_x_velocity(pt.x);
        sprite.set_y_velocity(pt.y);
    }

    // void Sprite_SetSpawnedCoordinates(int k, SpriteSpawnInfo *info) {
    //   sprite_x_lo[k] = info->r0_x;
    //   sprite_x_hi[k] = info->r0_x >> 8;
    //   sprite_y_lo[k] = info->r2_y;
    //   sprite_y_hi[k] = info->r2_y >> 8;
    //   sprite_z[k] = info->r4_z;
    // }
    pub(in crate::zelda_rtl) fn sprite_set_spawned_coordinates(&mut self, k: usize, info: &SpriteSpawnInfo) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.set_x(info.r0_x);
        sprite.set_y(info.r2_y);
        sprite.set_z(info.r4_z);
    }

    pub(in crate::zelda_rtl) fn sprite_explode_spawn_ea(&mut self, k: usize) {
        let sprite_type = self.sprite_slot_view(k).sprite_type();
        self.temp_counter_mut().set(sprite_type);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0xea, &mut info, 14);
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        let value = 32;
        self.sprite_slot_view_mut(j).set_z_velocity(value);
        let value = self.game_state.player.follower_link.lower_level_state();
        self.sprite_slot_view_mut(j).set_floor(value);
        let value = if j == 9 { 2 } else { 6 };
        self.sprite_slot_view_mut(j).set_a(value);
        self.sprite_set_y(j, info.r2_y.wrapping_add(3));
        if self.game_state.scratch_counter.value() == 0xce {
            self.sprite_set_y(j, info.r2_y.wrapping_add(16));
            return;
        }
        if self.game_state.scratch_counter.value() == 0xcb {
            let link_x_hi = (self.game_state.player.follower_link.x() >> 8) as u8;
            let link_y_hi = (self.game_state.player.follower_link.y() >> 8) as u8;
            let value = 0x78;
            self.sprite_slot_view_mut(j).set_y_low(value);
            let value = 0x78;
            self.sprite_slot_view_mut(j).set_x_low(value);
            let value = link_x_hi;
            self.sprite_slot_view_mut(j).set_x_high(value);
            let value = link_y_hi;
            self.sprite_slot_view_mut(j).set_y_high(value);
        }
    }

    // void SpriteModule_Die(int k) {  // 86f8a2
    //   SpriteDeath_MainEx(k, false);
    // }
    pub(in crate::zelda_rtl) fn sprite_module_die(&mut self, k: usize) {
        self.sprite_death_main_ex(k, false);
    }

    // void SpriteDeath_MainEx(int k, bool second_entry) {  // 86823a
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_death_main_ex(&mut self, k: usize, second_entry: bool) {
        if !second_entry {
            let type_ = self.sprite_slot_view(k).sprite_type();
            if type_ == 0xec {
                self.throwable_scenery_scatter_into_debris(k);
                return;
            }
            if type_ == 0x53
                || type_ == 0x54
                || type_ == 0x92
                || (type_ == 0x4a && self.sprite_slot_view(k).c() >= 2)
            {
                self.sprite_active_main_jsr(k);
                return;
            }
            if self.sprite_slot_view(k).delay_main() == 0 {
                self.sprite_do_the_death(k);
                return;
            }
        }
        if sign8(self.sprite_slot_view(k).flags3()) {
            self.sprite_active_main_jsr(k);
            return;
        }
        if ((self.game_state.frame.frame_counter & 3)
            | self.game_state.frame.submodule
            | self.game_state.frame.modal_pause_flag)
            == 0
        {
            let value = self.sprite_slot_view(k).delay_main().wrapping_add(1);
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        self.sprite_death_draw_poof(k);

        if self.sprite_slot_view(k).sprite_type() != 0x40
            && self.sprite_slot_view(k).delay_main() < 10
        {
            return;
        }
        let oam = self.game_state.oam.current_pointer().wrapping_add(16);
        let ext = self
            .game_state
            .oam
            .current_extended_pointer()
            .wrapping_add(4);
        self.oam_state_mut().set_current_pointer(oam);
        self.oam_state_mut().set_current_extended_pointer(ext);
        let bak = self.sprite_slot_view(k).flags2();
        let value = self.sprite_slot_view(k).flags2().wrapping_sub(4);
        self.sprite_slot_view_mut(k).set_flags2(value);
        self.sprite_active_main_jsr(k);
        let value = bak;
        self.sprite_slot_view_mut(k).set_flags2(value);
    }

    // void Sprite_DoTheDeath(int k) {  // 86f923
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_do_the_death(&mut self, k: usize) {
        let type_ = self.sprite_slot_view(k).sprite_type();
        if type_ == 0xbe {
            let g = self.sprite_slot_view(0).g().wrapping_sub(1);
            self.sprite_slot_view_mut(0).set_g(g);
        }

        if type_ == 0xaa && self.sprite_slot_view(k).e() != 0 {
            let bak = self.sprite_slot_view(k).subtype();
            let item = SPRITE_DO_THE_DEATH_PIKIT_DROP_ITEMS
                [usize::from(self.sprite_slot_view(k).e().wrapping_sub(1))];
            self.prepare_enemy_drop(k, item);
            let value = bak;
            self.sprite_slot_view_mut(k).set_subtype(value);
            if bak == 1 {
                let value = 9;
                self.sprite_slot_view_mut(k).set_oam_flags(value);
                let value = 0xf0;
                self.sprite_slot_view_mut(k).set_flags3(value);
            }
            let value = self.sprite_slot_view(k).head_direction().wrapping_add(1);
            self.sprite_slot_view_mut(k).set_head_direction(value);
            return;
        }

        if type_ == 0x45
            && self.game_state.inventory.save_progress.progress_indicator() == 2
            && self.game_state.world.region.overworld_area_low() == 0x18
        {
            self.set_music_control(7);
        }

        let drop_item = self.sprite_slot_view(k).die_action();
        if drop_item != 0 {
            let value = self.sprite_slot_view(k).n();
            self.sprite_slot_view_mut(k).set_subtype(value);
            let value = 255;
            self.sprite_slot_view_mut(k).set_n(value);
            let arg = if drop_item == 1 {
                0xe4
            } else if drop_item == 3 {
                0xd9
            } else {
                0xe5
            };
            self.prepare_enemy_drop(k, arg);
            return;
        }

        let mut prize = self.sprite_slot_view(k).flags5() & 0x0f;
        if prize != 0 {
            prize = prize.wrapping_sub(1);
            let luck = self.game_state.sprite_battle.item_drop_luck();
            if luck != 0 {
                self.sprite_battle_mut().increment_luck_kill_counter();
                if self.game_state.sprite_battle.luck_kill_counter() >= 10 {
                    self.sprite_battle_mut().set_item_drop_luck(0);
                }
                if luck == 1 {
                    self.force_prize_drop(k, prize, 1);
                    return;
                }
            } else {
                if (self.get_random_number() & SPRITE_DO_THE_DEATH_PRIZE_MASKS[usize::from(prize)])
                    == 0
                {
                    self.force_prize_drop(k, prize, prize);
                    return;
                }
            }
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        self.sprite_death_func4(k);
    }

    // void ForcePrizeDrop(int k, uint8 prize, uint8 slot) {  // 86f9bc
    //   prize = prize * 8 | prizes_arr1[slot];
    //   prizes_arr1[slot] = (prizes_arr1[slot] + 1) & 7;
    //   PrepareEnemyDrop(k, kPrizeItems[prize]);
    // }
    pub(in crate::zelda_rtl) fn force_prize_drop(&mut self, k: usize, prize: u8, slot: u8) {
        let slot = usize::from(slot);
        let cycle_index = self.prize_drop_cycle_mut().take_next_index(slot);
        let prize = (usize::from(prize) * 8) | usize::from(cycle_index);
        self.prepare_enemy_drop(k, FORCE_PRIZE_DROP_PRIZE_ITEMS[prize]);
    }

    // void PrepareEnemyDrop(int k, uint8 item) {  // 86f9d1
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn prepare_enemy_drop(&mut self, k: usize, item: u8) {
        let value = item;
        self.sprite_slot_view_mut(k).set_sprite_type(value);
        if item == 0xe5 {
            if self.begin_big_key_drop_graphics_work(k) {
                return;
            }
            self.sprite_prep_big_key_load_graphics(k);
        } else if item == 0xe4 {
            self.sprite_prep_key_set_item_drop(k);
        }

        self.complete_prepare_enemy_drop(k);
    }

    pub(in crate::zelda_rtl) fn complete_prepare_enemy_drop(&mut self, k: usize) {
        let value = 9;
        self.sprite_slot_view_mut(k).set_state(value);
        let zbak = self.sprite_slot_view(k).z();
        self.sprite_prep_load_properties(k);
        let value = self.sprite_slot_view(k).ignore_projectile().wrapping_add(1);
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);

        let pz = PREPARE_ENEMY_DROP_PRIZE_Z
            [usize::from(self.sprite_slot_view(k).sprite_type().wrapping_sub(0xd8))];
        let value = pz & 0xf0;
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_add(u16::from(pz & 0x0f)));
        let value = zbak;
        self.sprite_slot_view_mut(k).set_z(value);
        let value = 21;
        self.sprite_slot_view_mut(k).set_delay_aux4(value);
        let value = 255;
        self.sprite_slot_view_mut(k).set_stunned(value);
        self.sprite_death_func4(k);
    }

    // void SpriteDeath_Func4(int k) {  // 86fa25
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_death_func4(&mut self, k: usize) {
        if self.sprite_slot_view(k).sprite_type() == 0xa2 && self.sprite_check_if_screen_is_clear()
        {
            self.ancilla_spawn_falling_prize(4);
        }
        self.sprite_manually_set_death_flag_uw(k);
        self.sprite_battle_mut().increment_sprites_killed();
        if self.sprite_slot_view(k).sprite_type() == 0x40 {
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 4;
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_death_main_ex(k, true);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_death_draw_poof(&mut self, k: usize) {
        if self.game_state.dungeon.room_load.header_collision() == 4 {
            let value = 0x30;
            self.sprite_slot_view_mut(k).set_object_priority(value);
        }
        let Some((_x, _y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let r12 = (self.sprite_slot_view(k).flags3() & 0x20) >> 3;
        let scratch_position = &self.game_state.sprites.draw_hitbox_work;
        let dungmap_x = scratch_position.x_low();
        let dungmap_y = scratch_position.y_low();
        let mut i = usize::from((self.sprite_slot_view(k).delay_main() & 0x1c) ^ 0x1c) + 3;
        for _ in 0..4 {
            if SPRITE_DEATH_DRAW_POOF_CHARS[i] != 0 {
                self.oam_state_mut()
                    .set_entry_char(oam, SPRITE_DEATH_DRAW_POOF_CHARS[i]);
                self.oam_state_mut().set_entry_y(
                    oam,
                    dungmap_y
                        .wrapping_sub(r12)
                        .wrapping_add(SPRITE_DEATH_DRAW_POOF_Y_OFFSETS[i] as u8),
                );
                self.oam_state_mut().set_entry_x(
                    oam,
                    dungmap_x
                        .wrapping_sub(r12)
                        .wrapping_add(SPRITE_DEATH_DRAW_POOF_X_OFFSETS[i] as u8),
                );
                self.oam_state_mut()
                    .set_entry_flags(oam, (flags & 0x30) | SPRITE_DEATH_DRAW_POOF_FLAGS[i]);
            }
            oam += 4;
            i = i.wrapping_sub(1);
        }
        self.sprite_correct_oam_entries(k, 3, 0);
    }

    // void SpriteModule_Fall1(int k) {  // 86852e
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_module_fall1(&mut self, k: usize) {
        // $06:852E..8542 is a tail of the $06:84E2 state dispatch.
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        blocks.block(0x06852e);
        if self.sprite_slot_view(k).delay_main() == 0 {
            blocks.block(0x068533);
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_manually_set_death_flag_uw(k);
            blocks.block(0x06853a);
            return;
        }
        blocks.taken(0x068531);
        blocks.block(0x06853b);
        let (mut info, out) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        if !out {
            blocks.block(0x06853e);
            self.sprite_fall_draw(k, &mut info);
            blocks.block(0x068542);
        }
    }

    // void SpriteModule_Burn(int k) {  // sprite.c:747
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_module_burn(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_hit_timer(value);
        let j = i16::from(self.sprite_slot_view(k).delay_main()) - 1;
        if j == 0 {
            self.sprite_do_the_death(k);
            return;
        }
        let bak_graphics = self.sprite_slot_view(k).graphics();
        let bak_oam = self.sprite_slot_view(k).oam_flags();
        let value = SPRITE_MODULE_BURN_FLAME_GFX[(j >> 3) as usize];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = 3;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.flame_draw(k);
        let value = bak_oam;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = bak_graphics;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let next_oam = self.game_state.oam.current_pointer().wrapping_add(8);
        let next_ext = self
            .game_state
            .oam
            .current_extended_pointer()
            .wrapping_add(2);
        self.oam_state_mut().set_current_pointer(next_oam);
        self.oam_state_mut().set_current_extended_pointer(next_ext);
        if self.sprite_slot_view(k).delay_main() >= 0x10 {
            let bak = self.sprite_slot_view(k).flags2();
            let value = self.sprite_slot_view(k).flags2().wrapping_sub(2);
            self.sprite_slot_view_mut(k).set_flags2(value);
            self.sprite_active_main_long(k);
            let value = bak;
            self.sprite_slot_view_mut(k).set_flags2(value);
        }
    }

    // void SpriteModule_Poof(int k) {  // 86e393
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_module_poof(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            if self.sprite_slot_view(k).sprite_type() == 0x0d
                && self.sprite_slot_view(k).head_direction() != 0
            {
                let bakx = self.sprite_get_x(k);
                self.prepare_enemy_drop(k, 0x0d);
                self.sprite_set_x(k, bakx);
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            } else if self.sprite_slot_view(k).die_action() == 0 {
                self.force_prize_drop(k, 2, 2);
            } else {
                self.sprite_do_the_death(k);
            }
            return;
        }

        let Some((_x, _y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut j = usize::from(((self.sprite_slot_view(k).delay_main() >> 1) & !3) + 3).min(15);
        let scratch_position = &self.game_state.sprites.draw_hitbox_work;
        let base_x = scratch_position.x_low();
        let base_y = scratch_position.y_low();
        for _ in 0..4 {
            self.set_oam_plain_at(
                oam,
                base_x.wrapping_add(SPRITE_MODULE_POOF_X_OFFSETS[j] as u8),
                base_y.wrapping_add(SPRITE_MODULE_POOF_Y_OFFSETS[j] as u8),
                SPRITE_MODULE_POOF_CHARS[j],
                SPRITE_MODULE_POOF_FLAGS[j],
                SPRITE_MODULE_POOF_EXT_SIZES[j],
            );
            oam += 4;
            j = j.saturating_sub(1);
        }
        self.sprite_correct_oam_entries(k, 3, 0xff);
    }

    // void SpriteModule_Drown(int k) {  // 86859c
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_module_drown(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).a() == 6 {
                self.oam_allocate_from_region_c(8);
            }
            self.sprite_slot_view_mut(k).xor_flags3(16);
            self.sprite_draw_single_large(k);
            let oam = self.game_state.oam.current_pointer_usize();
            let j = self.sprite_slot_view(k).delay_main();
            if j == 1 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            if j != 0 {
                self.oam_state_mut().set_entry_char(
                    oam,
                    SPRITE_MODULE_DROWN_OAM_CHARS[usize::from((j >> 1).min(10))],
                );
                self.oam_state_mut().set_entry_flags(oam, 0x24);
                return;
            }
            self.oam_state_mut().set_entry_char(oam, 0x8a);
            let flags = SPRITE_MODULE_DROWN_OAM_FLAGS
                [usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 3)]
                | 0x24;
            self.oam_state_mut().set_entry_flags(oam, flags);
            if self.sprite_return_if_paused(k) {
                return;
            }
            self.sprite_slot_view_mut(k).increment_subtype2();
            self.sprite_move_xy(k);
            self.sprite_move_z(k);
            let value = self.sprite_slot_view(k).z_velocity().wrapping_sub(2);
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            if sign8(self.sprite_slot_view(k).z()) {
                let value = 0;
                self.sprite_slot_view_mut(k).set_z(value);
                let value = 18;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                self.sprite_slot_view_mut(k).and_flags3(!0x10);
            }
        } else {
            if self.sprite_return_if_paused(k) {
                return;
            }
            if self.game_state.frame.frame_counter & 1 == 0 {
                let value = self.sprite_slot_view(k).delay_main().wrapping_add(1);
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            let base =
                usize::from(((self.sprite_slot_view(k).delay_main() << 1) & 0xf8) >> 2).min(6);
            self.sprite_draw_multiple(
                k,
                &SPRITE_MODULE_DROWN_DROWN_DRAW_FRAMES[base..base + 2],
            );
        }
    }

    // void SpriteModule_Explode(int k) {  // sprite.c:616
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_module_explode(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() != 0 {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                if !(0..16).any(|j| self.sprite_slot_view(j).state() == 4) {
                    self.set_chr_halfslot_request(1);
                    if !self.sprite_check_if_screen_is_clear() {
                        self.follower_link_state_mut().clear_menu_block();
                    }
                }
            } else {
                let base = usize::from((self.sprite_slot_view(k).delay_main() >> 2) ^ 7) * 4;
                self.sprite_draw_multiple(
                    k,
                    &SPRITE_MODULE_EXPLODE_SPRITE_EXPLODE_DRAW_FRAMES[base..base + 4],
                );
            }
            return;
        }
        let value = 2;
        self.sprite_slot_view_mut(k).set_floor(value);
        if self.sprite_slot_view(k).delay_main() == 32 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.follower_link_state_mut().clear_immobilized();
            if !self.game_state.player.follower_link.near_pit_state_is(2)
                && self.sprite_check_if_screen_is_clear()
            {
                if self.sprite_slot_view(k).sprite_type() >= 0xd6 {
                    self.set_music_control(0x13);
                } else if self.sprite_slot_view(k).sprite_type() == 0x7a {
                    self.prepare_dungeon_exit_from_boss_fight();
                } else {
                    self.sprite_explode_spawn_ea(k);
                    return;
                }
            }
        }
        if self.sprite_slot_view(k).delay_main() >= 64
            && (self.sprite_slot_view(k).delay_main() >= 0x70
                || (self.sprite_slot_view(k).delay_main() & 1) == 0)
        {
            self.sprite_active_main_long(k);
        }

        let type_ = self.sprite_slot_view(k).sprite_type();
        let delay = self.sprite_slot_view(k).delay_main();
        if delay >= 0xc0 {
            return;
        }
        if delay & 3 == 0 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
        }
        if delay & if type_ == 0x92 { 3 } else { 7 } != 0 {
            return;
        }

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x1c, &mut info);
        if j >= 0 {
            const SPRITE_EXPLODE_RANDOM_XY: [i8; 16] =
                [0, 4, 8, 12, -4, -8, -12, 0, 0, 8, 16, 24, -24, -16, -8, 0];
            let j = j as usize;
            self.set_chr_halfslot_request(11);
            let value = 4;
            self.sprite_slot_view_mut(j).set_state(value);
            let value = 3;
            self.sprite_slot_view_mut(j).set_flags2(value);
            let value = 0x0c;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let random_base = if type_ == 0x92 { 8 } else { 0 };
            let xoff =
                SPRITE_EXPLODE_RANDOM_XY[usize::from(self.get_random_number() & 7) | random_base];
            let yoff =
                SPRITE_EXPLODE_RANDOM_XY[usize::from(self.get_random_number() & 7) | random_base];
            self.sprite_set_x(j, info.r0_x.wrapping_add(xoff as i16 as u16));
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add(yoff as i16 as u16)
                    .wrapping_sub(u16::from(info.r4_z)),
            );
            let value = 31;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 31;
            self.sprite_slot_view_mut(j).set_a(value);
        }
    }

    // void SpriteModule_Fall2(int k) {  // 86fbea
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_module_fall2(&mut self, k: usize) {
        let mut delay = self.sprite_slot_view(k).delay_main();
        if delay == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_manually_set_death_flag_uw(k);
            return;
        }
        if delay >= 0x40 {
            if self.sprite_slot_view(k).oam_flags() != 5 {
                if ((delay & 7)
                    | self.game_state.frame.submodule
                    | self.game_state.frame.modal_pause_flag)
                    == 0
                {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x31);
                }
                self.sprite_active_main_jsr(k);
                let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
                    return;
                };
                self.sprite_draw_distress_custom(x, y.wrapping_sub(8), delay.wrapping_add(20));
                return;
            }
            let value = 63;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            delay = 63;
        }
        if delay == 61 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
        }
        let j = usize::from(delay >> 1);
        if self.sprite_slot_view(k).sprite_type() == 0x26
            || self.sprite_slot_view(k).sprite_type() == 0x13
        {
            let value = SPRITE_MODULE_FALL2_FALLING_HELMA_BEETLE_GFX_BY_DELAY[j];
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_draw_falling_helma_beetle(k);
        } else {
            let mut t = SPRITE_MODULE_FALL2_FALLING_HUMANOID_GFX_BY_DELAY[j];
            if t < 12 {
                t = t.wrapping_add(
                    SPRITE_MODULE_FALL2_FALLING_DIRECTION_GFX_OFFSETS
                        [usize::from(self.sprite_slot_view(k).direction() & 3)],
                );
            }
            let value = t;
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_draw_falling_humanoid(k);
        }
        if (self.game_state.frame.frame_counter
            & SPRITE_MODULE_FALL2_FALLING_TILE_CHECK_FRAME_MASKS
                [usize::from(self.sprite_slot_view(k).delay_main() >> 3)])
            | self.game_state.frame.submodule
            != 0
        {
            return;
        }
        self.sprite_check_tile_property(k, 0x68);
        if self.game_state.sprites.workspace.tile() != NativeTile::PIT {
            let value = 0;
            self.sprite_slot_view_mut(k).set_y_recoil(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_x_recoil(value);
        }
        let value = ((self.sprite_slot_view(k).y_recoil() as i8) >> 2) as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let value = ((self.sprite_slot_view(k).x_recoil() as i8) >> 2) as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        self.sprite_move_xy(k);
    }

    // bool Sprite_CheckDamageToAndFromLink(int k) {  // 85ab93
    //   Sprite_CheckDamageFromLink(k);
    //   return Sprite_CheckDamageToLink(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_check_damage_to_and_from_link(&mut self, k: usize) -> bool {
        self.sprite_check_damage_from_link(k);
        self.sprite_check_damage_to_link(k)
    }

    // void SpriteModule_Carried(int k) {  // 86de83
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_module_carried(&mut self, k: usize) {
        let value = self.game_state.world.region.overworld_area_low();
        self.sprite_slot_view_mut(k).set_room(value);
        if self.sprite_slot_view(k).draw_work_byte_3() != 3 {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = if self.sprite_slot_view(k).c() == 6 {
                    8
                } else {
                    4
                };
                self.sprite_slot_view_mut(k).set_delay_main(value);
                self.sprite_slot_view_mut(k).increment_draw_work_byte_3();
            }
        } else {
            self.sprite_slot_view_mut(k).and_flags3(!0x10);
        }

        let t = self.sprite_slot_view(k).delay_aux4().wrapping_sub(1);
        let r0 = u16::from(t < 63 && (t & 2) != 0);
        let j = usize::from(
            self.game_state
                .player
                .follower_link
                .facing()
                .wrapping_mul(2)
                .wrapping_add(self.sprite_slot_view(k).draw_work_byte_3())
                & 0x0f,
        );
        let link_x = self.game_state.player.follower_link.x();
        let offset = SPRITE_MODULE_CARRIED_SPRITE_HELD_X[j] as i16 as u16;
        let t0 = u16::from(link_x as u8) + u16::from(offset as u8);
        let t1 = u16::from(t0 as u8) + ((t0 >> 8) & 1) + r0;
        let t2 = u16::from((link_x >> 8) as u8)
            + ((t1 >> 8) & 1)
            + ((t0 >> 8) & 1)
            + u16::from((offset >> 8) as u8);
        let value = t1 as u8;
        self.sprite_slot_view_mut(k).set_x_low(value);
        let value = t2 as u8;
        self.sprite_slot_view_mut(k).set_x_high(value);
        let value = SPRITE_MODULE_CARRIED_SPRITE_HELD_Z[j];
        self.sprite_slot_view_mut(k).set_z(value);
        let an = if self.game_state.player.follower_link.animation_step() < 6 {
            self.game_state.player.follower_link.animation_step_index()
        } else {
            0
        };
        let z = self
            .game_state
            .player
            .follower_link
            .z()
            .wrapping_add(1)
            .wrapping_add(u16::from(SPRITE_MODULE_CARRIED_SPRITE_HELD_Z_FOR_FRAME[an]));
        self.sprite_set_y(
            k,
            self.game_state
                .player
                .follower_link
                .y()
                .wrapping_add(8)
                .wrapping_sub(z),
        );
        let value = self.game_state.player.follower_link.lower_level_state() & 1;
        self.sprite_slot_view_mut(k).set_floor(value);

        self.carried_sprite_check_for_throw(k);
        self.sprite_get16_bit_coords(k);
        if self.sprite_slot_view(k).draw_work_byte_4() != 11 {
            self.sprite_active_main_jsr(k);
            if self.sprite_slot_view(k).delay_aux4() == 1 {
                let value = 9;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_b(value);
                let value = 96;
                self.sprite_slot_view_mut(k).set_delay_aux4(value);
                let value = 32;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                self.sprite_slot_view_mut(k).or_flags3(0x10);
                self.follower_link_state_mut().set_picking_throw_state(2);
            }
        } else {
            self.sprite_stunned_main_func1(k);
        }
    }

    // void CarriedSprite_CheckForThrow(int k) {  // 86df6d
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn carried_sprite_check_for_throw(&mut self, k: usize) {
        if self.game_state.frame.main_module == 14 {
            return;
        }
        if !self.game_state.player.follower_link.near_pit_state_is(2) {
            let t = (self.game_state.player.follower_link.auxiliary_state() & 1)
                | self.game_state.player.follower_link.deep_water_state()
                | u8::from(self.game_state.player.follower_link.is_bunny_mirror())
                | self.game_state.player.follower_link.item_hold_pose()
                | if self
                    .game_state
                    .player
                    .follower_link
                    .sprite_damage_disable_timer()
                    != 0
                {
                    0
                } else {
                    self.game_state.player.follower_link.incapacitated_timer()
                };
            if t == 0 {
                if self.sprite_slot_view(k).draw_work_byte_3() != 3
                    || ((self.game_state.player.follower_link.filtered_joypad_h()
                        | self.game_state.player.follower_link.filtered_joypad_l())
                        & 0x80)
                        == 0
                {
                    return;
                }
                self.follower_link_state_mut()
                    .clear_filtered_joypad_l_bits(0x80);
            }
        }

        self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
        self.follower_link_state_mut().set_picking_throw_state(2);
        let value = self.sprite_slot_view(k).draw_work_byte_4();
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_draw_work_byte_3(value);
        let value = (self.sprite_slot_view(k).flags3() & !0x10)
            | (sprite_init_value(
                SPRITE_INIT_FLAGS3_TABLE,
                self.sprite_slot_view(k).sprite_type(),
            ) & 0x10);
        self.sprite_slot_view_mut(k).set_flags3(value);
        let j = self.game_state.player.follower_link.facing_index() & 3;
        let value = CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_XVEL[j];
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_YVEL[j];
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let value = CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_ZVEL[j];
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_delay_aux4(value);
    }

    // void SpriteModule_Stunned(int k) {  // 86dffa
    //   SpriteStunned_MainEx(k, false);
    // }
    pub(in crate::zelda_rtl) fn sprite_module_stunned(&mut self, k: usize) {
        self.sprite_stunned_main_ex(k, false);
    }

    // void ThrownSprite_TileAndSpriteInteraction(int k) {  // 86e02a
    //   SpriteStunned_MainEx(k, true);
    // }
    pub(in crate::zelda_rtl) fn thrown_sprite_tile_and_sprite_interaction(&mut self, k: usize) {
        self.sprite_stunned_main_ex(k, true);
    }

    // void ThrowableScenery_InteractWithSpritesAndTiles(int k) {  // 86e164
    //   Sprite_MoveXY(k);
    //   if (!sprite_E[k])
    //     Sprite_CheckTileCollision(k);
    //   ThrownSprite_TileAndSpriteInteraction(k);
    // }
    pub(in crate::zelda_rtl) fn throwable_scenery_interact_with_sprites_and_tiles(&mut self, k: usize) {
        self.sprite_move_xy(k);
        if self.sprite_slot_view(k).e() == 0 {
            self.sprite_check_tile_collision(k);
        }
        self.thrown_sprite_tile_and_sprite_interaction(k);
    }

    // void SpriteStunned_MainEx(int k, bool second_entry) {  // 86dfcf
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_stunned_main_ex(&mut self, k: usize, second_entry: bool) {
        if !second_entry {
            self.sprite_draw_ripple_if_in_water(k);
            self.sprite_stunned_main_func1(k);
            if self.sprite_return_if_paused(k) {
                return;
            }
            if self.sprite_slot_view(k).f() != 0 {
                if sign8(self.sprite_slot_view(k).f()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_f(value);
                }
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            if self.sprite_slot_view(k).delay_main() < 0x20 {
                self.sprite_check_damage_from_link(k);
            }
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            self.sprite_move_xy(k);
            if self.sprite_slot_view(k).e() == 0 {
                self.sprite_check_tile_collision(k);
                if self.sprite_slot_view(k).state() == 0 {
                    return;
                }
            }
        }

        if (second_entry || self.sprite_slot_view(k).e() == 0)
            && (self.sprite_slot_view(k).wall_collision() & 0x0f) != 0
        {
            self.sprite_apply_ricochet(k);
            if self.sprite_slot_view(k).state() == 11 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 5);
            }
        }
        self.sprite_check_tile_property(k, 0x68);

        if sprite_init_value(
            SPRITE_INIT_FLAGS3_TABLE,
            self.sprite_slot_view(k).sprite_type(),
        ) & 0x10
            != 0
        {
            self.sprite_slot_view_mut(k).or_flags3(0x10);
            if self.game_state.sprites.workspace.tile() == NativeTile::PIT {
                self.sprite_slot_view_mut(k).and_flags3(!0x10);
            }
        }
        self.sprite_move_z(k);
        let value = self.sprite_slot_view(k).z_velocity().wrapping_sub(2);
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        if self.sprite_slot_view(k).z().wrapping_sub(1) >= 0xf0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            if self.sprite_slot_view(k).sprite_type() == 0xe8
                && (self.sprite_slot_view(k).z_velocity().wrapping_sub(0xe8) as i8).is_negative()
            {
                let value = 6;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 8;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 3;
                self.sprite_slot_view_mut(k).set_flags2(value);
                return;
            }

            self.throwable_scenery_transmute_if_valid(k);
            // Over a pit the landing surface comes from the sprite's own
            // flags instead of the probed tile, then faces the same test.
            let landing = self.game_state.sprites.workspace.tile();
            let surface = if landing == NativeTile::PIT {
                if self.sprite_slot_view(k).flags() & 1 == 0 {
                    self.sprite_func8(k);
                    return;
                }
                NativeTile::from_cartridge(self.sprite_slot_view(k).flags() >> 1)
            } else {
                landing
            };
            if surface == NativeTile::SHALLOW_WATER {
                let z_vel = self.sprite_slot_view(k).z_velocity();
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                if (z_vel.wrapping_sub(0xf0) as i8).is_negative() {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0xec, &mut info);
                    if j >= 0 {
                        let j = j as usize;
                        self.sprite_set_spawned_coordinates(j, &info);
                        self.sprite_func22(j);
                    }
                }
            } else if surface == NativeTile::DEEP_WATER {
                if self.sprite_slot_view(k).sprite_type() == 0xd2
                    || (self.get_random_number() & 1) != 0
                {
                    self.sprite_spawn_leaping_fish(k);
                }
                self.sprite_func22(k);
                return;
            }

            let z_vel = self.sprite_slot_view(k).z_velocity();
            if (z_vel as i8).is_negative() {
                let bounced = z_vel.wrapping_neg() >> 1;
                let value = if bounced < 9 { 0 } else { bounced };
                self.sprite_slot_view_mut(k).set_z_velocity(value);
            }
            let value = ((self.sprite_slot_view(k).x_velocity() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            if self.sprite_slot_view(k).x_velocity() == 0xff {
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            let value = ((self.sprite_slot_view(k).y_velocity() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            if self.sprite_slot_view(k).y_velocity() == 0xff {
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
        }
        if self.sprite_slot_view(k).state() != 11
            || self.sprite_slot_view(k).draw_work_byte_5() != 0
        {
            if self.sprite_return_if_lifted(k) {
                return;
            }
            if self.sprite_slot_view(k).sprite_type() != 0x4a {
                self.thrown_sprite_check_damage_to_sprites(k);
            }
        }
    }

    // void SpriteStunned_Main_Func1(int k) {  // 86e2ba
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_stunned_main_func1(&mut self, k: usize) {
        self.sprite_active_main_jsr(k);
        if self.sprite_slot_view(k).draw_work_byte_5() != 0 {
            if self.sprite_slot_view(k).delay_main() < 32 {
                let value = (self.sprite_slot_view(k).oam_flags() & 0xf1) | 4;
                self.sprite_slot_view_mut(k).set_oam_flags(value);
            }
            let t = (((k as u8) << 4) ^ self.game_state.frame.frame_counter)
                | self.game_state.frame.submodule;
            let mask = SPRITE_STUNNED_MAIN_FUNC1_SPRITE_STUNNED_MAIN_FUNC1_MASKS
                [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
            if t & mask != 0 {
                return;
            }
            let x = SPRITE_STUNNED_MAIN_FUNC1_SPARKLE_GARNISH_XY
                [usize::from(self.get_random_number() & 3)] as i16 as u16;
            let y = SPRITE_STUNNED_MAIN_FUNC1_SPARKLE_GARNISH_XY
                [usize::from(self.get_random_number() & 3)] as i16 as u16;
            self.sprite_garnish_spawn_sparkle(k, x, y);
            return;
        }

        if (self.game_state.frame.frame_counter & 1)
            | self.game_state.frame.submodule
            | self.game_state.frame.modal_pause_flag
            != 0
        {
            return;
        }
        let t = self.sprite_slot_view(k).stunned();
        if t != 0 {
            let value = self.sprite_slot_view(k).stunned().wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_stunned(value);
            if t < 0x38 {
                let value = if (t & 1) != 0 { (-8i8) as u8 } else { 8 };
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                self.sprite_move_x(k);
            }
            return;
        }
        let value = 9;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_x_recoil(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_y_recoil(value);
    }

    // void Sprite_SpawnLeapingFish(int k) {  // 86e286
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_spawn_leaping_fish(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xd2, &mut info);
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        let value = 2;
        self.sprite_slot_view_mut(j).set_ai_state(value);
        let value = 48;
        self.sprite_slot_view_mut(j).set_delay_main(value);
        if self.sprite_slot_view(k).sprite_type() == 0xd2 {
            let value = 0xd2;
            self.sprite_slot_view_mut(j).set_a(value);
        }
    }

    // bool Sprite_HandleDraggingByAncilla(int k) {  // 86cf64
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_handle_dragging_by_ancilla(&mut self, k: usize) -> bool {
        let mut j = self.sprite_slot_view(k).b();
        if j == 0 {
            return false;
        }
        j = j.wrapping_sub(1);
        let j = usize::from(j);
        if self.ancilla_slot_view(j).ancilla_type() == 0 {
            self.sprite_handle_absorption_by_player(k);
        } else {
            let ancilla = self.ancilla_slot_view(j);
            let x_low = ancilla.x_low();
            let x_high = ancilla.x_high();
            let y_low = ancilla.y_low();
            let y_high = ancilla.y_high();
            self.sprite_slot_view_mut(k).set_x_low(x_low);
            self.sprite_slot_view_mut(k).set_x_high(x_high);
            self.sprite_slot_view_mut(k).set_y_low(y_low);
            self.sprite_slot_view_mut(k).set_y_high(y_high);
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
        }
        true
    }

    // void Sprite_CheckAbsorptionByPlayer(int k) {  // 86d116
    //   if (!sprite_delay_aux4[k] && Sprite_CheckDamageToPlayer_1(k))
    //     Sprite_HandleAbsorptionByPlayer(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_check_absorption_by_player(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux4() == 0 && self.sprite_check_damage_to_player_1(k) {
            self.sprite_handle_absorption_by_player(k);
        }
    }

    // void Sprite_HandleAbsorptionByPlayer(int k) {  // 86d13c
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_handle_absorption_by_player(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        let t = self.sprite_slot_view(k).sprite_type().wrapping_sub(0xd8);
        if usize::from(t) < SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORPTION_SFX.len() {
            self.sprite_sfx_queue_sfx3_with_pan(
                k,
                SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORPTION_SFX[usize::from(t)],
            );
        }
        match t {
            0 => self.player_resources_mut().increment_heart_filler_by(8),
            1..=3 => {
                let rupees = self
                    .game_state
                    .inventory
                    .player_resources
                    .rupees_goal()
                    .wrapping_add(
                        SPRITE_HANDLE_ABSORPTION_BY_PLAYER_RUPEES_ABSORPTION[usize::from(t - 1)],
                    );
                self.player_resources_mut().set_rupees_goal(rupees);
            }
            4..=6 => {
                self.player_resources_mut().increment_bomb_filler_by(
                    SPRITE_HANDLE_ABSORPTION_BY_PLAYER_BOMBS_ABSORPTION[usize::from(t - 4)],
                );
            }
            7 => self.player_resources_mut().increment_magic_filler_by(0x10),
            8 => self.player_resources_mut().set_magic_filler(0x80),
            9 => {
                let arrows = if self.sprite_slot_view(k).head_direction() == 0 {
                    5
                } else {
                    self.sprite_slot_view(k).head_direction()
                };
                self.player_resources_mut()
                    .increment_arrow_filler_by(arrows);
            }
            10 => self.player_resources_mut().increment_arrow_filler_by(10),
            11 => {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x31);
                self.player_resources_mut().increment_heart_filler_by(56);
            }
            12 => {
                self.player_resources_mut().increment_keys();
                self.finish_absorbed_key_or_big_key(
                    k,
                    &SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORB_BIG_KEY,
                );
            }
            13 => {
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x32,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::BigKeyAbsorption,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.finish_absorbed_key_or_big_key(
                    k,
                    &SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORB_BIG_KEY,
                );
            }
            14 => {
                let shield = self.sprite_slot_view(k).subtype();
                self.inventory_items_mut().set_shield_type(shield);
                if self.game_state.enhanced_features.has(4096) {
                    self.palette_load_shield();
                }
            }
            _ => {}
        }
    }

    /// Source suffix after the absorption handler's synchronous
    /// `Link_ReceiveItem(0x32)` call (ROM `$86d13c` case 13). A live timing
    /// authority suspends the decompressor there; only the shared
    /// key/big-key savegame-bit tail remains.
    pub(in crate::zelda_rtl) fn complete_big_key_absorption_item_receipt(&mut self, k: usize) {
        self.finish_absorbed_key_or_big_key(k, &SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORB_BIG_KEY);
    }

    fn finish_absorbed_key_or_big_key(&mut self, k: usize, absorb_big_key: &[u16; 2]) {
        let value = self.sprite_slot_view(k).subtype();
        self.sprite_slot_view_mut(k).set_n(value);
        let idx = usize::from(self.sprite_slot_view(k).die_action());
        let bits =
            self.game_state.dungeon.savegame_state.savegame_state_bits() | absorb_big_key[idx];
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(bits);
        self.sprite_manually_set_death_flag_uw(k);
    }
}
