//! Sprite coordinate preparation, OAM correction, multi-draw, single-draw, and shadows.

use crate::zelda_rtl::sprite::{
    SpriteDistressEntry, SpriteShadowEntry, SpriteSingleLargeEntry, SpriteSingleSmallEntry,
};
use super::*;

impl ZeldaState {

    // void Oam_ResetRegionBases() {  // 8683d3
    //   memcpy(oam_region_base, kOam_ResetRegionBases, 12);
    // }
    pub(in crate::zelda_rtl) fn oam_reset_region_bases(&mut self) {
        // Cycle ledger: Oam_ResetRegionBases $06:83D3 (entry m8 x8). LDY #0 :
        // REP #$20 (38), six passes (Y = 0..10) of LDA abs,y STA abs,y INY
        // INY CPY #$0b BCC (146, no page crossing) with the BCC taken five
        // times, then SEP #$20 : RTS (64): 38 + 6 x 146 + 5 x 6 + 64 = 1,008.
        crate::cycle_ledger::charge_routine(0x06_83d3, 1_008);
        for (i, value) in OAM_RESET_REGION_BASES_OAM_RESET_REGION_BASES
            .into_iter()
            .enumerate()
        {
            self.oam_state_mut().set_region_base_word(i, value);
        }
    }

    // void Sprite_CorrectOamEntries(int k, int n, uint8 islarge) {  // 86febc
    //   OamEnt *oam = GetOamCurPtr();
    //   uint8 *extp = &g_ram[oam_ext_cur_ptr];
    //   uint16 spr_x = Sprite_GetX(k);
    //   uint16 spr_y = Sprite_GetY(k);
    //   uint8 scrollx = spr_x - BG2HOFS_copy2;
    //   uint8 scrolly = spr_y - BG2VOFS_copy2;
    //   do {
    //     uint16 x = spr_x + (int8)(oam->x - scrollx);
    //     uint16 y = spr_y + (int8)(oam->y - scrolly);
    //     uint8 ext = sign8(islarge) ? (*extp & 2) : islarge;
    //     *extp = ext + ((uint16)(x - BG2HOFS_copy2) >= 0x100);
    //     if ((uint16)(y + 0x10 - BG2VOFS_copy2) >= 0x100)
    //       oam->y = 0xf0;
    //   } while (oam++, extp++, --n >= 0);
    // }
    pub(in crate::zelda_rtl) fn sprite_correct_oam_entries(&mut self, k: usize, n: i32, islarge: u8) {
        let _scope = crate::cycle_ledger::routine(0x06_febc);
        crate::cycle_ledger::charge(218); // $FEBC..FEC8, including coordinate JSR
        // Sprite_GetScreenRelativeCoords, $FF26..FF48, fixed 8-bit slot reads.
        crate::cycle_ledger::charge_routine(0x06_ff26, 438);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut extp = self.game_state.oam.current_extended_pointer_usize();
        let spr_x = self.sprite_get_x(k);
        let spr_y = self.sprite_get_y(k);
        let scrollx =
            spr_x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()) as u8;
        let scrolly =
            spr_y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()) as u8;
        for index in 0..=n {
            crate::cycle_ledger::charge(72 + if sign8(islarge) { 46 } else { 6 }); // $FECA..FED2
            let x = spr_x.wrapping_add(
                self.game_state.oam.entry_x(oam).wrapping_sub(scrollx) as i8 as i16 as u16,
            );
            let y = spr_y.wrapping_add(
                self.game_state.oam.entry_y(oam).wrapping_sub(scrolly) as i8 as i16 as u16,
            );
            // $FED4..FEE2: sign-extend the tile's relative X displacement.
            crate::cycle_ledger::charge(170 + if sign8(self.game_state.oam.entry_x(oam).wrapping_sub(scrollx)) { 14 } else { 6 });
            let offscreen_x = x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()) >= 0x100;
            crate::cycle_ledger::charge(186 + if offscreen_x { 76 } else { 6 }); // $FEE3..FEF4
            crate::cycle_ledger::charge_routine(0x06_ff49, 188); // $FF49..FF55
            crate::cycle_ledger::charge(154 + if sign8(self.game_state.oam.entry_y(oam).wrapping_sub(scrolly)) { 14 } else { 6 }); // $FEF6..FF03
            let offscreen_y = y.wrapping_add(0x10).wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()) >= 0x100;
            crate::cycle_ledger::charge(186 + if offscreen_y { 46 } else { 6 }); // $FF04..FF15
            crate::cycle_ledger::charge_routine(0x06_ff56, 356); // $FF56..FF6C
            let ext = if sign8(islarge) {
                self.game_state.oam.extended_byte_at(extp) & 2
            } else {
                islarge
            };
            let value = ext
                + u8::from(
                    x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()) >= 0x100,
                );
            self.oam_state_mut().set_extended_byte_at(extp, value);
            if y.wrapping_add(0x10)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                >= 0x100
            {
                self.oam_state_mut().hide_entry(oam);
            }
            oam += 4;
            extp += 1;
            crate::cycle_ledger::charge(166 + if index < n { 6 } else { 0 }); // $FF17..FF20
        }
        crate::cycle_ledger::charge(92); // $FF22..FF25
    }

    pub(in crate::zelda_rtl) fn sprite_correct_oam_entries_long(&mut self, k: usize, n: i32, islarge: u8) {
        // $06:FEB4..FEBB: bank wrapper, including JSR of the shared body.
        crate::cycle_ledger::charge_routine(0x06_feb4, 190);
        self.sprite_correct_oam_entries(k, n, islarge);
    }

    pub(in crate::zelda_rtl) fn sprite_prep_oam_coord_or_double_ret(
        &mut self,
        k: usize,
    ) -> Option<(u16, u16, u8)> {
        self.sprite_prep_oam_coord_or_double_ret_from(k, PrepOamCoordEntry::Direct)
    }

    /// `Sprite_PrepOamCoordOrDoubleRet` reached through the ROM entry a
    /// caller in another bank actually uses (cycle ledger only; the result
    /// is identical for every entry).
    pub(in crate::zelda_rtl) fn sprite_prep_oam_coord_or_double_ret_from(
        &mut self,
        k: usize,
        entry: PrepOamCoordEntry,
    ) -> Option<(u16, u16, u8)> {
        let (ret, out) = self.sprite_prep_oam_coord_or_double_ret_raw_from(k, entry);
        if out {
            None
        } else {
            Some((ret.x, ret.y, ret.flags))
        }
    }

    pub(in crate::zelda_rtl) fn sprite_prep_oam_coord_or_double_ret_with_out_flag_from(
        &mut self,
        k: usize,
        entry: PrepOamCoordEntry,
    ) -> ((u16, u16, u8), bool) {
        let (ret, out) = self.sprite_prep_oam_coord_or_double_ret_raw_from(k, entry);
        ((ret.x, ret.y, ret.flags), out)
    }

    // void Sprite_PrepOamCoord(int k, PrepOamCoordsRet *ret) {  // 86e416
    //   Sprite_PrepOamCoordOrDoubleRet(k, ret);
    // }
    pub(in crate::zelda_rtl) fn sprite_prep_oam_coord(&mut self, k: usize) -> PrepOamCoordsRet {
        self.sprite_prep_oam_coord_from(k, PrepOamCoordEntry::Long)
    }

    /// `Sprite_PrepOamCoord` reached through the ROM entry the caller uses
    /// (cycle ledger only).
    pub(in crate::zelda_rtl) fn sprite_prep_oam_coord_from(
        &mut self,
        k: usize,
        entry: PrepOamCoordEntry,
    ) -> PrepOamCoordsRet {
        self.sprite_prep_oam_coord_or_double_ret_raw_from(k, entry).0
    }

    pub(super) fn sprite_prep_oam_coord_or_double_ret_raw(&mut self, k: usize) -> (PrepOamCoordsRet, bool) {
        self.sprite_prep_oam_coord_or_double_ret_raw_from(k, PrepOamCoordEntry::Direct)
    }

    /// Cycle ledger: the JSR/JSL entry chain into
    /// Sprite_PrepOamCoordOrDoubleRet $06:E41E. Every wrapper is a JSR/JSL
    /// target, so each opens its own scope around the next one.
    ///
    /// - `Safe`: Sprite_PrepOamCoordSafeWrapper $06:E41A = JSR $06:E41E (46)
    ///   : RTS (42). The double return pops this wrapper's return address,
    ///   so its RTS runs only for an in-bounds sprite.
    /// - `Long`: Sprite_PrepOamCoord $06:E416 = JSR $06:E41A (46) : RTL (44)
    ///   around `Safe`.
    /// - `Bank5DoubleRet` / `Bank1dDoubleRet` / `Bank1eDoubleRet`: the
    ///   bank-local double-return wrappers $05:FA50 / $1D:E9AD / $1E:FF84 =
    ///   JSL $06:E416 (62), BCC (16, taken +6 when in bounds), else PLA PLA
    ///   (56), RTS (42) — the RTS runs on both paths (out of bounds it
    ///   returns to the caller's caller). The profiler closes a frame as
    ///   soon as the stack rises above its entry depth, so out of bounds
    ///   the first PLA (28) is the wrapper's last own instruction and the
    ///   second PLA and the RTS (70) are measured in the caller's frame;
    ///   the ledger splits them the same way (200k profiles: $05:FA50 self
    ///   126 in bounds / 106 out, the caller +70).
    /// - `Bank5Safe`: Sprite_PrepOamCoord_wrapper $05:9257 = JSR $05:FA50
    ///   (46) : RTS (42, skipped when the double return pops it).
    fn sprite_prep_oam_coord_or_double_ret_raw_from(
        &mut self,
        k: usize,
        entry: PrepOamCoordEntry,
    ) -> (PrepOamCoordsRet, bool) {
        match entry {
            PrepOamCoordEntry::Direct => self.sprite_prep_oam_coord_or_double_ret_body(k),
            PrepOamCoordEntry::Safe => {
                let _wrapper = crate::cycle_ledger::routine(0x06_e41a);
                crate::cycle_ledger::charge(46);
                let result =
                    self.sprite_prep_oam_coord_or_double_ret_raw_from(k, PrepOamCoordEntry::Direct);
                if !result.1 {
                    crate::cycle_ledger::charge(42);
                }
                result
            }
            PrepOamCoordEntry::Long => {
                let _long = crate::cycle_ledger::routine(0x06_e416);
                crate::cycle_ledger::charge(46);
                let result =
                    self.sprite_prep_oam_coord_or_double_ret_raw_from(k, PrepOamCoordEntry::Safe);
                crate::cycle_ledger::charge(44);
                result
            }
            PrepOamCoordEntry::Bank5DoubleRet
            | PrepOamCoordEntry::Bank1dDoubleRet
            | PrepOamCoordEntry::Bank1eDoubleRet => {
                let address = match entry {
                    PrepOamCoordEntry::Bank5DoubleRet => 0x05_fa50,
                    PrepOamCoordEntry::Bank1dDoubleRet => 0x1d_e9ad,
                    _ => 0x1e_ff84,
                };
                let result = {
                    let _wrapper = crate::cycle_ledger::routine(address);
                    crate::cycle_ledger::charge(62 + 16);
                    let result = self
                        .sprite_prep_oam_coord_or_double_ret_raw_from(k, PrepOamCoordEntry::Long);
                    crate::cycle_ledger::charge(if result.1 { 28 } else { 6 + 42 });
                    result
                };
                if result.1 {
                    // The second PLA and the RTS of the double return run
                    // after the wrapper's frame has closed: the caller's cost.
                    crate::cycle_ledger::charge(28 + 42);
                }
                result
            }
            PrepOamCoordEntry::Bank5Safe => {
                let _wrapper = crate::cycle_ledger::routine(0x05_9257);
                crate::cycle_ledger::charge(46);
                let result = self.sprite_prep_oam_coord_or_double_ret_raw_from(
                    k,
                    PrepOamCoordEntry::Bank5DoubleRet,
                );
                if !result.1 {
                    crate::cycle_ledger::charge(42);
                }
                result
            }
        }
    }

    // bool Sprite_PrepOamCoordOrDoubleRet(int k, PrepOamCoordsRet *ret) {  // 86e41e
    //   sprite_pause[k] = 0;
    //   uint16 x = cur_sprite_x - BG2HOFS_copy2;
    //   uint16 y = cur_sprite_y - BG2VOFS_copy2;
    //   bool out_of_bounds = false;
    //   prep_x = x;
    //   prep_y = y - sprite_z[k];
    //   ret->flags = sprite_oam_flags[k] ^ sprite_obj_prio[k];
    //   ret->r4 = 0;
    //   if ((uint16)(x + 0x40 + xt) >= (0x170 + xt * 2) ||
    //       (uint16)(y + 0x40) >= 0x170 && !(sprite_flags4[k] & 0x20)) {
    //     sprite_pause[k]++;
    //     if (!(sprite_defl_bits[k] & 0x80))
    //       Sprite_KillSelf(k);
    //     out_of_bounds = true;
    //   }
    //   ret->x = prep_x;
    //   ret->y = prep_y;
    //   BYTE(dungmap_var7) = ret->x;
    //   HIBYTE(dungmap_var7) = ret->y;
    //   return out_of_bounds;
    // }
    fn sprite_prep_oam_coord_or_double_ret_body(&mut self, k: usize) -> (PrepOamCoordsRet, bool) {
        let mut work = CoordinatePrepContinuation::new(false);
        while self.retire_coordinate_prep_instruction(k, &mut work).is_some() {}
        (self.prepared_oam_coordinates(), work.work.out_of_bounds())
    }

    fn prepared_oam_coordinates(&self) -> PrepOamCoordsRet {
        PrepOamCoordsRet {
            x: u16::from_le_bytes([self.ram[0], self.ram[1]]),
            y: u16::from_le_bytes([self.ram[2], self.ram[3]]),
            r4: self.ram[4], flags: self.ram[5],
        }
    }

    pub(super) fn coordinate_prep_inputs(&self, k: usize) -> crate::cycle_models::sprite_prep::Inputs {
        let slot = self.sprite_slot_view(k);
        crate::cycle_models::sprite_prep::Inputs {
            world_x: self.game_state.sprites.workspace.current_sprite_x(),
            world_y: self.game_state.sprites.workspace.current_sprite_y(),
            scroll_x: self.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
            scroll_y: self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
            height: slot.z(), flags4: slot.flags4(), deflection: slot.deflection_bits(),
            flags: slot.oam_flags(), priority: slot.object_priority(), pause: slot.pause(),
            scratch: self.ram[..6].try_into().unwrap(),
            extended_screen: self.game_state.enhanced_features.has(FEATURES0_EXTEND_SCREEN64_SPRITE),
        }
    }

    fn retire_coordinate_prep_instruction(
        &mut self, k: usize, work: &mut CoordinatePrepContinuation,
    ) -> Option<u32> {
        if work.call_pending {
            work.call_pending = false;
            crate::cycle_ledger::charge_routine(0x06_dc10, 46);
            return Some(46);
        }
        if let Some(kill) = work.kill.as_mut() {
            if let Some(cycles) = self.retire_sprite_kill_instruction(k, kill) {
                return Some(cycles);
            }
            work.kill = None;
        }
        use crate::cycle_models::sprite_prep::Publication;
        let instruction = work.work.step(self.coordinate_prep_inputs(k))?;
        crate::cycle_ledger::charge_routine(0x06_e41e, u64::from(instruction.master_cycles));
        match instruction.publication {
            Some(Publication::ScratchByte { offset, value }) => {
                self.sprite_workspace_mut().set_oam_prep_scratch_byte(offset, value);
            }
            Some(Publication::ScratchWord { offset, value }) => {
                let mut workspace = self.sprite_workspace_mut();
                workspace.set_oam_prep_scratch_byte(offset, value as u8);
                workspace.set_oam_prep_scratch_byte(offset + 1, (value >> 8) as u8);
            }
            Some(Publication::Pause(value)) => self.sprite_slot_view_mut(k).set_pause(value),
            Some(Publication::DrawX(value)) => {
                let y = self.ram[crate::game_state::constants::DRAW_WORK_POSITION_Y];
                self.draw_scratch_position_mut().set_low_position(value, y);
            }
            Some(Publication::DrawY(value)) => {
                let x = self.ram[crate::game_state::constants::DRAW_WORK_POSITION_X];
                self.draw_scratch_position_mut().set_low_position(x, value);
            }
            Some(Publication::EnterKill) => work.kill = Some(crate::cycle_models::sprite_kill::KillWork::new()),
            None => {}
        }
        Some(instruction.master_cycles)
    }

    /// Coordinates and pause precede KillSelf; draw scratch belongs to the
    /// shared return path and must remain unpublished while that call is parked.
    fn sprite_prep_oam_coord_prefix(&mut self, k: usize) -> (bool, bool) {
        // Cycle ledger: Sprite_PrepOamCoordOrDoubleRet $06:E41E (entry m8 x8,
        // X <= 15 so no abs,x page crossing). The blocks are charged where
        // the bounds test resolves, below.
        let value = 0;
        self.sprite_slot_view_mut(k).set_pause(value);
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let x = cur_x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = cur_y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let xt = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_EXTEND_SCREEN64_SPRITE)
        {
            0x40
        } else {
            0
        };
        // $06:E41E-E434 STZ $f00,x, REP, LDA $0fd8 SEC SBC $e2 STA $00 CLC ADC
        // #$40 CMP #$170 SEP, BCS (278). The ROM has no extended-screen
        // margin; with the feature off these tests are the ROM's, and with it
        // on the ROM cost of the same outcome is charged.
        let x_out_of_bounds = x.wrapping_add(0x40 + xt) >= 0x170 + xt * 2;
        let y_out_of_bounds = y.wrapping_add(0x40) >= 0x170;
        let flags4_keeps_offscreen = self.sprite_slot_view(k).flags4() & 0x20 != 0;
        let height_operand = if x_out_of_bounds {
            // BCS skips $E436..E43B. The later 16-bit SBC $04 consumes
            // the word left by the preceding caller, not this sprite's Z.
            self.sprite_workspace_mut().oam_prep_scratch_word()
        } else {
            let height = self.sprite_slot_view(k).z();
            self.sprite_workspace_mut().set_oam_prep_height(height);
            u16::from(height)
        };
        self.sprite_workspace_mut().set_oam_prep_coords(x, y.wrapping_sub(height_operand));
        crate::cycle_ledger::charge(if x_out_of_bounds {
            // BCS taken (+6) into $06:E476-E483 REP LDA $0fda SEC SBC $e8 SEC
            // SBC $04 STA $02 SEP (208), falling into $06:E485.
            278 + 6 + 208
        } else {
            // $06:E436-E455 LDA $f70,x STA $04 STZ $05 REP LDA $0fda SEC SBC
            // $e8 PHA SEC SBC $04 STA $02 PLA CLC ADC #$40 CMP #$170 SEP BCC
            // (432); in range takes the BCC (+6) into $06:E45E CLC (14).
            if !y_out_of_bounds {
                278 + 432 + 6 + 14
            } else if flags4_keeps_offscreen {
                // $06:E457-E45C LDA $f60,x AND #$20 BEQ not taken (64), then
                // $06:E45E CLC (14).
                278 + 432 + 64 + 14
            } else {
                // BEQ taken (+6) into $06:E485.
                278 + 432 + 64 + 6
            }
        });
        let out = x.wrapping_add(0x40 + xt) >= 0x170 + xt * 2
            || (y.wrapping_add(0x40) >= 0x170 && self.sprite_slot_view(k).flags4() & 0x20 == 0);
        if out {
            // $06:E485-E48B INC $f00,x LDA $caa,x BMI (100): a deflecting
            // sprite takes the BMI (+6), else $06:E48D JSL Sprite_KillSelf
            // (62; the callee charges its own body). Then $06:E491-E494 PLA
            // PLA SEC BRA (92; BRA's 22 already includes the taken cycles)
            // into the shared exit.
            crate::cycle_ledger::charge(100);
            let value = self.sprite_slot_view(k).pause().wrapping_add(1);
            self.sprite_slot_view_mut(k).set_pause(value);
            if (self.sprite_slot_view(k).deflection_bits() & 0x80) == 0 {
                crate::cycle_ledger::charge(62);
                return (true, true);
            } else {
                crate::cycle_ledger::charge(6);
            }
        }
        (out, false)
    }

    fn sprite_prep_oam_coord_suffix(&mut self, k: usize, out: bool) -> (PrepOamCoordsRet, bool) {
        if out {
            crate::cycle_ledger::charge(92);
        }
        // $06:E45F-E475 LDA $f50,x EOR $b89,x STA $05 STZ $04 LDA $00 STA
        // $0fa8 LDA $02 STA $0fa9 LDY #0 RTS (282), both paths.
        crate::cycle_ledger::charge(282);
        let ret_x = self.game_state.sprites.workspace.oam_prep_x();
        let ret_y = self.game_state.sprites.workspace.oam_prep_y();
        let ret = PrepOamCoordsRet {
            x: ret_x,
            y: ret_y,
            r4: 0,
            flags: self.sprite_slot_view(k).oam_flags()
                ^ self.sprite_slot_view(k).object_priority(),
        };
        self.sprite_workspace_mut().set_oam_prep_flags(ret.flags);
        self.draw_scratch_position_mut()
            .set_low_position(ret_x as u8, ret_y as u8);
        (ret, out)
    }

    /// Bank-5 draw wrapper suspended inside its shared off-screen KillSelf call.
    pub(in crate::zelda_rtl) fn sprite_prep_oam_coord_bank5_until_kill_self(&mut self, k: usize) {
        let _wrapper = crate::cycle_ledger::routine(0x05_fa50);
        crate::cycle_ledger::charge(62 + 16);
        let _long = crate::cycle_ledger::routine(0x06_e416);
        crate::cycle_ledger::charge(46);
        let _safe = crate::cycle_ledger::routine(0x06_e41a);
        crate::cycle_ledger::charge(46);
        let _body = crate::cycle_ledger::routine(0x06_e41e);
        assert_eq!(self.sprite_prep_oam_coord_prefix(k), (true, true));
    }

    pub(in crate::zelda_rtl) fn sprite_prep_oam_coord_bank5_complete_kill_self(&mut self, k: usize) {
        {
            let _wrapper = crate::cycle_ledger::routine(0x05_fa50);
            {
                let _long = crate::cycle_ledger::routine(0x06_e416);
                {
                    let _safe = crate::cycle_ledger::routine(0x06_e41a);
                    let _body = crate::cycle_ledger::routine(0x06_e41e);
                    self.sprite_kill_self(k);
                    self.sprite_prep_oam_coord_suffix(k, true);
                }
                crate::cycle_ledger::charge(44);
            }
            crate::cycle_ledger::charge(28);
        }
        crate::cycle_ledger::charge(28 + 42);
    }

    // void Sprite_DrawMultiple(int k, const DrawMultipleData *src, int n,
    //                          PrepOamCoordsRet *info)
    //   See sprite.c:900.
    /// Draw the entries relative to the sprite and return the prepared OAM
    /// coordinates. The original always stores them, even when the sprite is
    /// off screen and nothing is drawn, so callers can position extra parts.
    pub(in crate::zelda_rtl) fn sprite_draw_multiple(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
    ) -> PrepOamCoordsRet {
        self.sprite_draw_multiple_from(k, src, DrawMultipleEntry::Direct)
    }

    /// `Sprite_DrawMultiple` reached through the ROM entry the caller uses
    /// (cycle ledger only; the drawing is identical for every entry).
    pub(in crate::zelda_rtl) fn sprite_draw_multiple_from(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
        entry: DrawMultipleEntry,
    ) -> PrepOamCoordsRet {
        let wrapper = (entry == DrawMultipleEntry::Bank1d).then(|| {
            // Sprite_DrawMultiple__ $1D:8549 (JSR target): JSL $05:DF6C (62)
            // ... RTS (42).
            let scope = crate::cycle_ledger::routine(0x1d_8549);
            crate::cycle_ledger::charge(62);
            scope
        });
        let prepared = {
            let _scope = self.sprite_draw_multiple_entry_scope(entry);
            let (prepared, drawable) = self.sprite_prepare_draw_multiple(k);
            self.sprite_draw_multiple_body(
                k,
                src.iter().map(|record| DrawMultipleWordData {
                    x: record.x as i16 as u16,
                    y: record.y as i16 as u16,
                    char_flags: record.char_flags,
                    ext: record.ext,
                }),
                drawable,
            );
            prepared
        };
        if wrapper.is_some() {
            crate::cycle_ledger::charge(42);
        }
        drop(wrapper);
        prepared
    }

    /// Cycle ledger: the Sprite_DrawMultiple scope `entry` opens, with its
    /// entry blocks charged. `Direct` (and the bank-$1D wrapper) enter at
    /// $05:DF6C STA $06 : STZ $07 (48); every entry then runs $05:DF70 JSR
    /// Sprite_DrawMultiple_prep_oam : BRA $DF78 (46 + 22) — off screen the
    /// prologue's double return lands on that BRA too.
    #[must_use = "bind the scope to a local so it lives until the draw returns"]
    fn sprite_draw_multiple_entry_scope(
        &self,
        entry: DrawMultipleEntry,
    ) -> crate::cycle_ledger::RoutineScope {
        match entry {
            DrawMultipleEntry::Direct | DrawMultipleEntry::Bank1d => {
                let scope = crate::cycle_ledger::routine(0x05_df6c);
                crate::cycle_ledger::charge(48 + 46 + 22);
                scope
            }
            DrawMultipleEntry::R6 => {
                let scope = crate::cycle_ledger::routine(0x05_df70);
                crate::cycle_ledger::charge(46 + 22);
                scope
            }
        }
    }

    /// The tail of Sprite_DrawMultiple after its prologue returned: $05:DF78
    /// BCS (16; taken +6 into the $05:DFE4 RTL, 44, when the sprite is off
    /// screen), else $05:DF7A-DF80 PHX, REP #$30, LDY #0, LDX $0090 (108), the
    /// per-entry loop, $05:DFE1 SEP #$30 : PLX (50) and the RTL (44).
    fn sprite_draw_multiple_body(
        &mut self,
        k: usize,
        src: impl IntoIterator<Item = DrawMultipleWordData>,
        drawable: Option<(u16, u16, u8)>,
    ) {
        let Some(info) = drawable else {
            crate::cycle_ledger::charge(16 + 6 + 44);
            return;
        };
        crate::cycle_ledger::charge(16 + 108);
        self.sprite_draw_multiple_words_with_info_charged(k, src, info, true, false);
        crate::cycle_ledger::charge(50 + 44);
    }

    /// The prepared coordinates, and the draw triple when the sprite is on
    /// screen.
    fn sprite_prepare_draw_multiple(
        &mut self,
        k: usize,
    ) -> (PrepOamCoordsRet, Option<(u16, u16, u8)>) {
        self.sprite_prepare_draw_multiple_from(k, false)
    }

    /// Sprite_DrawMultiple_prep_oam $05:DFE9 (a JSR target, m8 x8); with
    /// `deferred` the entry is Sprite_DrawMultiple_prep_oam_deferred
    /// $05:DFE5, which first JSLs OAM_AllocateDeferToPlayer_ (62; the callee
    /// charges its own body) and falls through. JSR $05:FA50 (46) preps the
    /// coordinates; off screen its double return leaves the prologue
    /// through the wrapper. On screen: $05:DFEC-DFF8 PHP, STZ $0CFE, STZ
    /// $0CFF, LDA $DD0,x, CMP #$0A, BNE (150; state 10 loads $7FFA2C,x, 40,
    /// else the BNE is taken +6), $05:DFFE CMP #$0B : BNE (32; state 11
    /// stores $7FFA3C,x into $0CFE, 72, else +6), $05:E009 PLP : RTS (70).
    fn sprite_prepare_draw_multiple_from(
        &mut self,
        k: usize,
        deferred: bool,
    ) -> (PrepOamCoordsRet, Option<(u16, u16, u8)>) {
        let _scope = crate::cycle_ledger::routine(if deferred { 0x05_dfe5 } else { 0x05_dfe9 });
        if deferred {
            crate::cycle_ledger::charge(62);
            // This entry JSLs the bank wrapper, not the bank-6 RTS body.
            crate::cycle_ledger::charge_routine(0x06_f864, 190);
            self.oam_allocate_defer_to_player(k);
        }
        crate::cycle_ledger::charge(46);
        let (prepped, out_of_bounds) = self.sprite_prep_oam_coord_or_double_ret_raw_from(k, PrepOamCoordEntry::Bank5DoubleRet);
        if !out_of_bounds {
            let state = self.sprite_slot_view(k).state();
            let priority_state = if state == 10 {
                self.sprite_slot_view(k).draw_work_byte_4()
            } else {
                state
            };
            crate::cycle_ledger::charge(
                150 + if state == 10 { 40 } else { 6 }
                    + 32
                    + if priority_state == 11 { 72 } else { 6 }
                    + 70,
            );
        }
        let prepared = PrepOamCoordsRet {
            x: prepped.x,
            y: prepped.y,
            r4: 0,
            flags: prepped.flags,
        };
        let drawable = (!out_of_bounds).then_some((prepped.x, prepped.y, prepped.flags));
        (prepared, drawable)
    }

    /// Decode native eight-byte records, retaining word coordinates which
    /// cannot be represented by the ordinary compact signed-byte tables.
    pub(in crate::zelda_rtl) fn sprite_draw_multiple_from_encoded_records(
        &mut self,
        k: usize,
        records: &[[u8; 8]],
    ) -> PrepOamCoordsRet {
        // Cycle ledger: the `Direct` entry (JSL $05:DF6C).
        let _scope = self.sprite_draw_multiple_entry_scope(DrawMultipleEntry::Direct);
        let (prepared, drawable) = self.sprite_prepare_draw_multiple(k);
        self.sprite_draw_multiple_body(
            k,
            records.iter().map(|bytes| DrawMultipleWordData {
                x: u16::from_le_bytes([bytes[0], bytes[1]]),
                y: u16::from_le_bytes([bytes[2], bytes[3]]),
                char_flags: u16::from_le_bytes([bytes[4], bytes[5]]),
                ext: bytes[7],
            }),
            drawable,
        );
        prepared
    }

    /// Draw the ROM's native eight-byte `DrawMultipleData` records after a
    /// 16-bit table-address calculation has wrapped into low WRAM, entering
    /// Sprite_DrawMultiple at `entry` (cycle ledger only).
    pub(in crate::zelda_rtl) fn sprite_draw_multiple_from_wram_records<const N: usize>(
        &mut self,
        k: usize,
        source: u16,
        entry: DrawMultipleEntry,
    ) -> PrepOamCoordsRet {
        let mut cursor = source;
        let entries: [DrawMultipleWordData; N] = std::array::from_fn(|_| {
            let word = |address: u16| {
                u16::from(self.ram[usize::from(address)])
                    | (u16::from(self.ram[usize::from(address.wrapping_add(1))]) << 8)
            };
            let entry = DrawMultipleWordData {
                x: word(cursor),
                y: word(cursor.wrapping_add(2)),
                char_flags: word(cursor.wrapping_add(4)),
                ext: self.ram[usize::from(cursor.wrapping_add(7))],
            };
            cursor = cursor.wrapping_add(8);
            entry
        });
        let wrapper = (entry == DrawMultipleEntry::Bank1d).then(|| {
            // Sprite_DrawMultiple__ $1D:8549: JSL $05:DF6C (62) ... RTS (42).
            let scope = crate::cycle_ledger::routine(0x1d_8549);
            crate::cycle_ledger::charge(62);
            scope
        });
        let prepared = {
            let _scope = self.sprite_draw_multiple_entry_scope(entry);
            let (prepared, drawable) = self.sprite_prepare_draw_multiple(k);
            self.sprite_draw_multiple_body(k, entries, drawable);
            prepared
        };
        if wrapper.is_some() {
            crate::cycle_ledger::charge(42);
        }
        drop(wrapper);
        prepared
    }

    // Variant that takes a precomputed PrepOamCoord triple (x, y, flags). The
    // C version mutates a caller-supplied PrepOamCoordsRet pointer; Rust
    // callers pass the triple directly so the API stays panic-free.
    pub(in crate::zelda_rtl) fn sprite_draw_multiple_with_info(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
        info: (u16, u16, u8),
    ) {
        // Not charged: the callers own a different ROM entry (they prepped
        // the coordinates themselves).
        self.sprite_draw_multiple_words_with_info_charged(
            k,
            src.iter().map(|entry| DrawMultipleWordData {
                x: entry.x as i16 as u16,
                y: entry.y as i16 as u16,
                char_flags: entry.char_flags,
                ext: entry.ext,
            }),
            info,
            false,
            false,
        );
    }

    /// $1E:AE2E JSR $87F0 -> JSL $05:DF6C. This native entry reuses
    /// prepared coordinates, so price only the complete executed draw loop
    /// ($05:DF78..DFE4), not the skipped coordinate-preparation call/body.
    pub(in crate::zelda_rtl) fn sprite_draw_multiple_prepared_marked(
        &mut self, k: usize, src: &[DrawMultipleData], info: (u16, u16, u8),
    ) {
        let _wrapper = self.lag_free.then(|| crate::cycle_ledger::routine(0x1e87f0));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e87f0);
        {
            let _body = self.lag_free.then(|| crate::cycle_ledger::routine(0x05df6c));
            b.block(0x05df78); b.block(0x05df7a);
            self.sprite_draw_multiple_words_with_info_charged(k,
                src.iter().map(|e| DrawMultipleWordData {
                    x: e.x as i16 as u16, y: e.y as i16 as u16,
                    char_flags: e.char_flags, ext: e.ext,
                }), info, false, true);
            b.block(0x05dfe1); b.block(0x05dfe4);
        }
        b.block(0x1e87f4);
    }

    /// The Sprite_DrawMultiple entry loop. With `ledger`, each entry charges
    /// $05:DF83-DFA1 (454; the BCC is taken +6 when y + $10 < $100, else
    /// $05:DFA3 LDA #$F0 : STA $1,x, 70), $05:DFA9-DFB5 (194; BCC taken +6
    /// when the priority override is 0, else $05:DFB7-DFBA, 48) and
    /// $05:DFBD-DFDF (540; the closing BNE is taken +6 before every entry
    /// but the first).
    fn sprite_draw_multiple_words_with_info_charged(
        &mut self,
        k: usize,
        src: impl IntoIterator<Item = DrawMultipleWordData>,
        info: (u16, u16, u8),
        ledger: bool,
        marked: bool,
    ) {
        let b = crate::cycle_ledger::blocks(marked && self.lag_free);
        let (info_x, info_y, info_flags) = info;
        // r4 is always 0 in C's Sprite_PrepOamCoordOrDoubleRet (sprite.c:1843).
        let info_r4: u8 = 0;
        self.sprite_workspace_mut().clear_draw_priority_override();
        let mut a = self.sprite_slot_view(k).state();
        if a == 10 {
            a = self.sprite_slot_view(k).draw_work_byte_4();
        }
        if a == 11 {
            let priority = self.sprite_slot_view(k).draw_work_byte_5();
            self.sprite_workspace_mut()
                .set_draw_priority_override_low(priority);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let combined_flags = (u16::from(info_flags) << 8) | u16::from(info_r4);
        let mut first = true;
        for entry in src {
            let mut d = entry.char_flags ^ combined_flags;
            let override_active =
                self.game_state.sprites.workspace.draw_priority_override() >= 1;
            if override_active {
                d = (d & !0x0e00) | 0x0400;
            }
            let x = info_x.wrapping_add(entry.x);
            let y = info_y.wrapping_add(entry.y);
            if ledger {
                crate::cycle_ledger::charge(
                    if first { 0 } else { 6 }
                        + 454
                        + if y.wrapping_add(0x10) < 0x100 { 6 } else { 70 }
                        + 194
                        + if override_active { 48 } else { 6 }
                        + 540,
                );
            }
            b.block(0x05df83);
            if y.wrapping_add(0x10) < 0x100 { b.taken(0x05dfa1); }
            else { b.block(0x05dfa3); }
            b.block(0x05dfa9);
            if override_active { b.block(0x05dfb7); }
            else { b.taken(0x05dfb5); }
            b.block(0x05dfbd);
            if !first { b.taken(0x05dfdf); }
            first = false;
            self.set_oam_helper0_at(oam, x, y, d as u8, (d >> 8) as u8, entry.ext);
            oam += 4;
        }
    }

    // void Sprite_DrawMultiplePlayerDeferred(int k, ...) {
    //   Oam_AllocateDeferToPlayer(k);
    //   Sprite_DrawMultiple(k, src, n, info);
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_multiple_player_deferred(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
    ) -> PrepOamCoordsRet {
        // Cycle ledger: Sprite_DrawMultiplePlayerDeferred $05:DF75 (JSL
        // target): JSR Sprite_DrawMultiple_prep_oam_deferred (46), whose
        // prologue allocates the deferred OAM region before the shared
        // coordinate prep, then the shared $05:DF78 tail.
        let _scope = crate::cycle_ledger::routine(0x05_df75);
        crate::cycle_ledger::charge(46);
        let (prepared, drawable) = self.sprite_prepare_draw_multiple_from(k, true);
        self.sprite_draw_multiple_body(
            k,
            src.iter().map(|record| DrawMultipleWordData {
                x: record.x as i16 as u16,
                y: record.y as i16 as u16,
                char_flags: record.char_flags,
                ext: record.ext,
            }),
            drawable,
        );
        prepared
    }

    fn sprite_single_draw_char(&self, k: usize) -> u8 {
        let base = SINGLE_LARGE_SPRITE_CHAR_BASE_BY_TYPE
            .get(usize::from(self.sprite_slot_view(k).sprite_type()))
            .copied()
            .unwrap_or(0);
        SINGLE_LARGE_SPRITE_CHAR_BY_BASE_AND_GFX
            .get(usize::from(base) + usize::from(self.sprite_slot_view(k).graphics()))
            .copied()
            .unwrap_or(0)
    }

    // void SpriteDraw_SingleLarge(int k) {  // 86dc10
    //   PrepOamCoordsRet info;
    //   if (Sprite_PrepOamCoordOrDoubleRet(k, &info))
    //     return;
    //   Sprite_PrepAndDrawSingleLargeNoPrep(k, &info);
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_single_large(
        &mut self,
        k: usize,
        entry: SpriteSingleLargeEntry,
    ) {
        match entry {
            SpriteSingleLargeEntry::Unpriced => {
                let mut work = self.begin_single_large_draw(k);
                while self.retire_single_large_draw_instruction(&mut work).is_some() {}

            }
            SpriteSingleLargeEntry::Rom06DBF0 => {
                let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x06_dbf0));
                let blocks = crate::cycle_ledger::blocks(self.lag_free);
                // $06:DBF0..DBF7: bank wrapper only. DC10/DC13, coordinate
                // prep and the shadow tail already retire their own instructions.
                blocks.block(0x06dbf0);
                self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
                blocks.block(0x06dbf6);

            }
        }
    }

    pub(in crate::zelda_rtl) fn begin_single_large_draw(
        &self,
        k: usize,
    ) -> SingleLargeDrawContinuation {
        let mut work = SingleLargeDrawContinuation::new(k,
            &PrepOamCoordsRet { x: 0, y: 0, r4: 0, flags: 0 });
        work.prep = Some(CoordinatePrepContinuation::new(true));
        work
    }

    // void Sprite_PrepAndDrawSingleLargeNoPrep(int k, PrepOamCoordsRet *info) {  // 86dc13
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_prep_and_draw_single_large_no_prep(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
    ) {
        let mut work = SingleLargeDrawContinuation::new(k, info);
        while self.retire_single_large_draw_instruction(&mut work).is_some() {
            info.y = work.y;
        }
    }

    pub(in crate::zelda_rtl) fn retire_single_large_draw_instruction(
        &mut self,
        work: &mut SingleLargeDrawContinuation,
    ) -> Option<u32> {
        if let Some(prep) = work.prep.as_mut() {
            if let Some(cycles) = self.retire_coordinate_prep_instruction(usize::from(work.slot), prep) {
                return Some(cycles);
            }
            work.skipped = prep.work.out_of_bounds();
            work.prep = None;
            let info = self.prepared_oam_coordinates();
            work.x = info.x;
            work.y = info.y;
            work.flags = info.flags;
        }
        if work.skipped { return None; }
        if let Some(shadow) = work.shadow.as_mut() {
            let _scope = crate::cycle_ledger::routine(0x06_dc66);
            let cycles = self.retire_shadow_instruction(shadow);
            work.y = shadow.y;
            return cycles;
        }
        use crate::cycle_models::sprite_single::{Inputs, Publication};
        let slot = self.sprite_slot_view(usize::from(work.slot));
        let index = work.work.table_index();
        let character_base = SINGLE_LARGE_SPRITE_CHAR_BASE_BY_TYPE.get(index).copied().unwrap_or(0);
        // $06:DA09's byte table ends at $06:DB03. The last five values of
        // an 8-bit index address the adjacent type table, exactly as the ROM.
        let character = SINGLE_LARGE_SPRITE_CHAR_BY_BASE_AND_GFX.get(index).copied()
            .or_else(|| index.checked_sub(SINGLE_LARGE_SPRITE_CHAR_BY_BASE_AND_GFX.len())
                .and_then(|index| SINGLE_LARGE_SPRITE_CHAR_BASE_BY_TYPE.get(index).copied()))
            .unwrap_or(0);
        let instruction = work.work.step(Inputs {
            x: work.x, y: work.y, flags: work.flags,
            sprite_type: slot.sprite_type(), graphics: slot.graphics(), flags3: slot.flags3(),
            character_base, character,
            oam_pointer: self.game_state.oam.current_pointer_usize() as u16,
            extended_pointer: self.game_state.oam.current_extended_pointer_usize() as u16,
        })?;
        crate::cycle_ledger::charge_routine(0x06_dc13, u64::from(instruction.master_cycles));
        match instruction.publication {
            Some(Publication::OamByte { address, value }) => {
                let address = usize::from(address);
                let base = address & !3;
                let mut oam = self.oam_state_mut();
                match address & 3 {
                    0 => oam.set_entry_x(base, value),
                    1 => oam.set_entry_y(base, value),
                    2 => oam.set_entry_char(base, value),
                    3 => oam.set_entry_flags(base, value),
                    _ => unreachable!(),
                }
            }
            Some(Publication::ExtendedByte { address, value }) => {
                self.oam_state_mut().set_extended_byte_at(usize::from(address), value);
            }
            Some(Publication::EnterShadow) => work.shadow = Some(ShadowOamTail {
                work: crate::cycle_models::sprite_shadow::ShadowWork::new(10),
                slot: work.slot, x: work.x, y: work.y, flags: work.flags,
            }),
            None => {}
        }
        Some(instruction.master_cycles)
    }

    /// The default-offset far entry at $06:DC54, including its bank wrapper.
    /// Same-bank and custom-offset callers have different entry work.


    // The C helper merges far and same-bank entries. Price their shared
    // $06:DC66 body here; bank-wrapper work belongs to explicit caller entries.
    pub(in crate::zelda_rtl) fn sprite_draw_shadow_custom(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
        a: u8,
        entry: SpriteShadowEntry,
    ) {
        match entry {
            SpriteShadowEntry::Unpriced => {
                let _scope = crate::cycle_ledger::routine(0x06_dc66);
                if let Some(tail) = self.sprite_shadow_body_before_extended(k, info, a) {
                    self.finish_shadow_body(tail);
                }

            }
            SpriteShadowEntry::Rom06DC54 => {
                // $06:DC54..DC5B is a far wrapper; DC64/DC66 remain unchanged.
                let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x06dc54));
                let b = crate::cycle_ledger::blocks(self.lag_free);
                b.block(0x06dc54);
                self.sprite_draw_shadow_custom(k, info, a, SpriteShadowEntry::Unpriced);
                b.block(0x06dc5a);

            }
            // Default-offset entry executes LDA #10 in its DC64 child scope.
            SpriteShadowEntry::Rom06DC54Default => {
                let _wrapper = crate::cycle_ledger::routine(0x06_dc54);
                crate::cycle_ledger::charge(118); // PHB : PHK : PLB : JSR.
                {
                    let _body = crate::cycle_ledger::routine(0x06_dc64);
                    crate::cycle_ledger::charge(16); // LDA #10.
                    if let Some(tail) = self.sprite_shadow_body_before_extended(k, info, 10) {
                        self.finish_shadow_body(tail);
                    }
                }
                crate::cycle_ledger::charge(72); // PLB : RTL after the body returns.

            }
        }
    }

    /// Stop after the ordinary OAM entry's flags store. The original ROM can
    /// accept NMI here before the separate extended-OAM byte publication.
    pub(in crate::zelda_rtl) fn sprite_draw_shadow_custom_before_extended(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
        a: u8,
    ) -> Option<ShadowOamTail> {
        let _scope = crate::cycle_ledger::routine(0x06_dc66);
        self.sprite_shadow_body_before_extended(k, info, a)
    }

    fn sprite_shadow_body_before_extended(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
        a: u8,
    ) -> Option<ShadowOamTail> {
        let mut tail = ShadowOamTail {
            work: crate::cycle_models::sprite_shadow::ShadowWork::new(a),
            slot: u8::try_from(k).expect("sprite slot fits a byte"),
            x: info.x, y: info.y, flags: info.flags,
        };
        while self.retire_shadow_instruction(&mut tail).is_some() {
            info.y = tail.y;
            if tail.work.at_extended_tail() {
                return Some(tail);
            }
        }
        None
    }

    pub(in crate::zelda_rtl) fn finish_shadow_extended_oam_write(
        &mut self,
        tail: ShadowOamTail,
    ) {
        let _scope = crate::cycle_ledger::routine(0x06_dc66);
        self.finish_shadow_body(tail);
    }

    fn finish_shadow_body(&mut self, mut tail: ShadowOamTail) {
        while self.retire_shadow_instruction(&mut tail).is_some() {}
    }

    fn retire_shadow_instruction(&mut self, tail: &mut ShadowOamTail) -> Option<u32> {
        use crate::cycle_models::sprite_shadow::{Inputs, Publication};
        let slot = self.sprite_slot_view(usize::from(tail.slot));
        let input = Inputs {
            world_y: self.sprite_get_y(usize::from(tail.slot)),
            pause: slot.pause(), state: slot.state(), draw_work: slot.draw_work_byte_3(),
            scroll_y: self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
            flags2: slot.flags2(), flags3: slot.flags3(),
            x: tail.x, y: tail.y, flags: tail.flags,
            oam_pointer: self.game_state.oam.current_pointer_usize() as u16,
            extended_pointer: self.game_state.oam.current_extended_pointer_usize() as u16,
        };
        let instruction = tail.work.step(input)?;
        crate::cycle_ledger::charge(u64::from(instruction.master_cycles));
        match instruction.publication {
            Some(Publication::YLow(value)) => {
                tail.y = (tail.y & 0xff00) | u16::from(value);
                self.sprite_workspace_mut().set_oam_prep_scratch_byte(2, value);
            }
            Some(Publication::YHigh(value)) => {
                tail.y = (tail.y & 0xff) | (u16::from(value) << 8);
                self.sprite_workspace_mut().set_oam_prep_scratch_byte(3, value);
            }
            Some(Publication::Y(value)) => {
                tail.y = value;
                let mut workspace = self.sprite_workspace_mut();
                workspace.set_oam_prep_scratch_byte(2, value as u8);
                workspace.set_oam_prep_scratch_byte(3, (value >> 8) as u8);
            }
            Some(Publication::OamByte { address, value }) => {
                let address = usize::from(address);
                let base = address & !3;
                let mut oam = self.oam_state_mut();
                match address & 3 {
                    0 => oam.set_entry_x(base, value),
                    1 => oam.set_entry_y(base, value),
                    2 => oam.set_entry_char(base, value),
                    3 => oam.set_entry_flags(base, value),
                    _ => unreachable!(),
                }
            }
            Some(Publication::ExtendedByte { address, value }) => {
                self.oam_state_mut().set_extended_byte_at(usize::from(address), value);
            }
            None => {}
        }
        Some(instruction.master_cycles)
    }

    // void SpriteDraw_SingleSmall(int k) {  // 86dcef
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_single_small(
        &mut self,
        k: usize,
        entry: SpriteSingleSmallEntry,
    ) {
        match entry {
            SpriteSingleSmallEntry::Unpriced => {
                let Some(continuation) = self.sprite_draw_single_small_position_prefix(k) else {
                    return;
                };
                self.sprite_draw_single_small_after_position(k, continuation);

            }
            SpriteSingleSmallEntry::Rom06DCEF => {
                // $06:DCEF..DD37. Position-prefix resumes retain their raw entry;
                // only a complete call receives this price. DC66's shadow is a child.
                let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x06dcef));
                let b = crate::cycle_ledger::blocks(self.lag_free);
                b.block(0x06dcef);
                let Some(continuation) = self.sprite_draw_single_small_position_prefix(k) else { return; };
                b.block(0x06dcf2);
                if continuation.visible {
                    b.block(0x06dd0f);
                    let ty = self.sprite_slot_view(k).sprite_type();
                    if 4 + usize::from(ty) > 255 {
                        b.crossing(0x06dd17);
                    }
                    let base = SINGLE_LARGE_SPRITE_CHAR_BASE_BY_TYPE.get(usize::from(ty)).copied().unwrap_or(0);
                    let index = base.wrapping_add(self.sprite_slot_view(k).graphics());
                    if 9 + usize::from(index) > 255 {
                        b.crossing(0x06dd1f);
                    }
                } else {
                    b.taken(0x06dd0d);
                }
                b.block(0x06dd2b);
                if self.sprite_slot_view(k).flags3() & 0x10 == 0 {
                    b.taken(0x06dd30);
                    b.block(0x06dd37);
                } else {
                    b.block(0x06dd32);
                }
                self.sprite_draw_single_small_after_position(k, continuation);

            }
            SpriteSingleSmallEntry::Rom06DBF8 => {
                // $06:DBF8..DBFF: bank wrapper, excluding DCEF's body.
                let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x06dbf8));
                let b = crate::cycle_ledger::blocks(self.lag_free);
                b.block(0x06dbf8);
                self.sprite_draw_single_small(k, SpriteSingleSmallEntry::Rom06DCEF);
                b.block(0x06dbfe);

            }
        }
    }

    pub(in crate::zelda_rtl) fn sprite_draw_single_small_position_prefix(
        &mut self,
        k: usize,
    ) -> Option<SingleSmallDrawContinuation> {
        let (x, y, flags) = self.sprite_prep_oam_coord_or_double_ret(k)?;
        let oam = self.game_state.oam.current_pointer_usize();
        self.oam_state_mut().set_entry_x(oam, x as u8);
        // The ROM publishes the extended-OAM size/X bit before its vertical
        // clipping branch ($86:DCFD), not after the character/flags stores as
        // the decompilation orders it. This is observable when vblank lands
        // at the source statement after the visible Y write.
        let ext_index = (oam - OAM_BUF) / 4;
        let value = u8::from(x >= 256);
        self.oam_state_mut().set_extended_byte(ext_index, value);
        let visible = y.wrapping_add(0x10) < 0x100;
        if visible {
            self.oam_state_mut().set_entry_y(oam, y as u8);
        }
        Some(SingleSmallDrawContinuation {
            x,
            y,
            oam: u16::try_from(oam).expect("single-small OAM pointer exceeded one word"),
            flags,
            visible,
        })
    }

    pub(super) fn sprite_draw_single_small_after_position(
        &mut self,
        k: usize,
        continuation: SingleSmallDrawContinuation,
    ) {
        let SingleSmallDrawContinuation {
            x,
            y,
            oam,
            flags,
            visible,
        } = continuation;
        if visible {
            let chr = self.sprite_single_draw_char(k);
            self.oam_state_mut().set_entry_char(usize::from(oam), chr);
            self.oam_state_mut()
                .set_entry_flags(usize::from(oam), flags);
        }
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
            self.sprite_draw_shadow_custom(k, &mut info, 2, SpriteShadowEntry::Unpriced);
        }
    }

    // void Sprite_DrawThinAndTall(int k) {  // 86dd40
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_thin_and_tall(&mut self, k: usize) {
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        // $06:DD40..DDAE, M8/X8; coordinate and shadow bodies stay separate.
        // JMP tails inherit the caller's owner, just as the ROM profiler does.
        blocks.block(0x06dd40);
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        blocks.block(0x06dd43);
        if y.wrapping_add(0x10) < 0x100 { blocks.taken(0x06dd68); }
        else { blocks.block(0x06dd6a); }
        blocks.block(0x06dd6f);
        if y.wrapping_add(8).wrapping_add(0x10) < 0x100 { blocks.taken(0x06dd80); }
        else { blocks.block(0x06dd82); }
        blocks.block(0x06dd87);
        let ty = u16::from(self.sprite_slot_view(k).sprite_type());
        if 0x04 + ty > 0xff { blocks.crossing(0x06dd8c); }
        let base = SINGLE_LARGE_SPRITE_CHAR_BASE_BY_TYPE.get(usize::from(ty))
            .copied().unwrap_or(0);
        let index = base.wrapping_add(self.sprite_slot_view(k).graphics());
        if 0x09 + u16::from(index) > 0xff { blocks.crossing(0x06dd94); }
        let oam = self.game_state.oam.current_pointer_usize();
        let chr = self.sprite_single_draw_char(k);
        self.set_oam_helper0_at(oam, x, y, chr, flags, 0);
        self.set_oam_helper0_at(
            oam + 4,
            x,
            y.wrapping_add(8),
            chr.wrapping_add(0x10),
            flags,
            0,
        );
        // $06:DC4C..DC65 is the shared conditional-shadow tail. DC66 already prices its body.
        blocks.block(0x06dc4c);
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            blocks.taken(0x06dc51);
            blocks.block(0x06dc64);
            let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
            self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Unpriced);
        } else { blocks.block(0x06dc53); }
    }

    // void SpriteFall_Draw(int k, PrepOamCoordsRet *info) {  // 9dffc5
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_fall_draw(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        let oam = self.game_state.oam.current_pointer_usize();
        let idx = usize::from(self.sprite_slot_view(k).delay_main() >> 2);
        self.oam_state_mut().write_entry(
            oam,
            info.x.wrapping_add(4) as u8,
            info.y.wrapping_add(4) as u8,
            SPRITE_FALL_DRAW_SPRITE_FALL_CHAR[idx],
            (info.flags & 0x30) | 0x04,
        );
        self.sprite_correct_oam_entries(k, 0, 0);
    }

    // void Sprite_DrawDistress_custom(uint16 xin, uint16 yin, uint8 time) {  // 86a733
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_distress_custom(
        &mut self, xin: u16, yin: u16, time: u8,
        entry: SpriteDistressEntry,
    ) {
        // $06:A733..A78D: complete far-call writer; RegionA owns allocation.
        let marked = self.lag_free && entry == SpriteDistressEntry::Rom06A733;
        let _scope = marked.then(|| crate::cycle_ledger::routine(0x06a733));
        let b = crate::cycle_ledger::blocks(marked);
        b.block(0x06a733);
        self.oam_allocate_from_region_a(0x10);
        b.block(0x06a739);
        if (time & 0x18) == 0 {
            b.taken(0x06a73d);
            b.block(0x06a78d);
            return;
        }
        b.block(0x06a73f);
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..4).rev() {
            b.block(0x06a744);
            let y = yin.wrapping_add(SPRITE_DRAW_DISTRESS_CUSTOM_Y_OFFSETS[i] as i16 as u16);
            if y.wrapping_add(16) < 256 { b.taken(0x06a76c); }
            else { b.block(0x06a76e); }
            b.block(0x06a772);
            if i != 0 { b.taken(0x06a78a); }
            self.set_oam_helper0_at(
                oam,
                xin.wrapping_add(SPRITE_DRAW_DISTRESS_CUSTOM_X_OFFSETS[i] as i16 as u16),
                yin.wrapping_add(SPRITE_DRAW_DISTRESS_CUSTOM_Y_OFFSETS[i] as i16 as u16),
                0x83,
                0x22,
                0,
            );
            oam += 4;
        }
        b.block(0x06a78c);
        b.block(0x06a78d);
    }

    // void SpriteDraw_FallingHelmaBeetle(int k) {  // 86fd17
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_falling_helma_beetle(&mut self, k: usize) {
        let mut base = usize::from(self.sprite_slot_view(k).graphics()).min(5);
        if self.sprite_slot_view(k).sprite_type() == 0x13 {
            base += 6;
        }
        self.sprite_draw_multiple(
            k,
            &SPRITE_DRAW_FALLING_HELMA_BEETLE_FALL0[base..base + 1],
        );
    }

    // void SpriteDraw_FallingHumanoid(int k) {  // 86fe5b
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_falling_humanoid(&mut self, k: usize) {
        // $06:FE5B..FEB3: selection, inline writer and correction call only.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x06fe5b));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x06fe5b);
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let q = usize::from(self.sprite_slot_view(k).graphics());
        let mut oam = self.game_state.oam.current_pointer_usize();
        b.block(0x06fe5e);
        if q >= 12 { b.taken(0x06fe6c); }
        else {
            b.block(0x06fe6e);
            if q & 3 != 0 { b.taken(0x06fe70); }
            else { b.block(0x06fe72); }
        }
        b.block(0x06fe74);
        let n = if q < 12 && (q & 3) == 0 { 3 } else { 0 };
        for n_cur in (0..=n).rev() {
            let i = q * 4 + n_cur;
            b.block(0x06fe76);
            for (low, pc) in [(0x43, 0x06fe7f), (0x7b, 0x06fe87),
                (0xb3, 0x06fe8d), (0xeb, 0x06fe93), (0x23, 0x06fea0)] {
                if low + i > 0xff { b.crossing(pc); }
            }
            if n_cur != 0 { b.taken(0x06fea9); }
            self.set_oam_plain_at(
                oam,
                x.wrapping_add(SPRITE_DRAW_FALLING_HUMANOID_X_OFFSETS[i] as i16 as u16) as u8,
                y.wrapping_add(SPRITE_DRAW_FALLING_HUMANOID_Y_OFFSETS[i] as i16 as u16) as u8,
                SPRITE_DRAW_FALLING_HUMANOID_CHARS[i],
                flags ^ SPRITE_DRAW_FALLING_HUMANOID_FLAGS[i],
                SPRITE_DRAW_FALLING_HUMANOID_EXT_SIZES[i],
            );
            oam += 4;
        }
        b.block(0x06feab);
        self.sprite_correct_oam_entries(k, n as i32, 0xff);
        b.block(0x06feb3);
    }

    // void ScatterDebris_Draw(int k, Point16U pt) {  // 89f198
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn scatter_debris_draw(&mut self, k: usize, pt: Point16U) {
        // $09:F198..F1F7: the three inline OAM entries; no child call.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x09f198));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x09f198);
        if self.garnish_slot_view(k).countdown() == 16 {
            b.block(0x09f1a0);
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
        } else { b.taken(0x09f19e); }
        b.block(0x09f1a6);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let base = usize::from(((self.garnish_slot_view(k).countdown() & 0x0f) >> 2) * 3);
        for i in (0..=2usize).rev() {
            b.block(0x09f1b6);
            if i != 0 { b.taken(0x09f1f4); }
            let j = base + i;
            self.set_oam_helper1_at(
                oam,
                pt.x.wrapping_add(SCATTER_DEBRIS_DRAW_X_OFFSETS[j] as i16 as u16),
                pt.y.wrapping_add(SCATTER_DEBRIS_DRAW_Y_OFFSETS[j] as i16 as u16) as u8,
                SCATTER_DEBRIS_DRAW_CHARS[j],
                SCATTER_DEBRIS_DRAW_FLAGS[j] | 0x22,
                0,
            );
            oam += 4;
        }
        b.block(0x09f1f6);
    }

    // void Garnish16_ThrownItemDebris(int k) {  // 89f0cb
    pub(in crate::zelda_rtl) fn garnish16_thrown_item_debris(&mut self, k: usize) {
        // $09:B110 RTS-dispatch -> F0CB. F198 scatter and prep remain children.
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x09f0cb);
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        b.block(0x09f0ce);
        let r5 = self.garnish_slot_view(k).oam_flags();
        if self.game_state.sprites.system.chr_halfslot_state() >= 3 {
            b.taken(0x09f0d9); b.block(0x09f0e6); return;
        }
        b.block(0x09f0db);
        if self.garnish_slot_view(k).sprite() == 3 {
            b.block(0x09f0e3);
            self.scatter_debris_draw(k, pt);
            b.block(0x09f0e6); return;
        }
        b.taken(0x09f0e1); b.block(0x09f0e7);
        let garnish_sprite = self.garnish_slot_view(k).sprite();
        self.temp_counter_mut().set(garnish_sprite);
        let mut base = ((self.garnish_slot_view(k).countdown() >> 2) ^ 7) << 2;
        if self.game_state.scratch_counter.value() == 4
            || (self.game_state.scratch_counter.value() == 2
                && self.game_state.world.location.is_outdoors())
        {
            base = base.wrapping_add(0x20);
        }
        if garnish_sprite == 4 { b.taken(0x09f0f7); b.block(0x09f101); }
        else {
            b.block(0x09f0f9);
            if garnish_sprite != 2 { b.taken(0x09f0fb); }
            else {
                b.block(0x09f0fd);
                if self.game_state.world.location.is_indoors() { b.taken(0x09f0ff); }
                else { b.block(0x09f101); }
            }
        }
        b.block(0x09f104);
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..=3usize).rev() {
            let j = usize::from(base) + i;
            b.block(0x09f10b);
            if 0x8b + 2 * j > 255 { b.crossing(0x09f118); }
            if self.game_state.scratch_counter.value() == 0 { b.block(0x09f133); }
            else {
                b.taken(0x09f131); b.block(0x09f137);
                if self.game_state.scratch_counter.value() < 0x80 { b.taken(0x09f13c); }
                else { b.block(0x09f13e); }
            }
            b.block(0x09f140); if i != 0 { b.taken(0x09f158); }
            let chr = if self.game_state.scratch_counter.value() == 0 {
                0x4e
            } else if self.game_state.scratch_counter.value() >= 0x80 {
                0xf2
            } else {
                GARNISH16_THROWN_ITEM_DEBRIS_CHARS[j]
            };
            self.set_oam_helper1_at(
                oam,
                pt.x.wrapping_add(GARNISH16_THROWN_ITEM_DEBRIS_X_OFFSETS[j] as u16),
                pt.y.wrapping_add(GARNISH16_THROWN_ITEM_DEBRIS_Y_OFFSETS[j] as i16 as u16) as u8,
                chr,
                GARNISH16_THROWN_ITEM_DEBRIS_FLAGS[j] | r5,
                0,
            );
            oam += 4;
        }
        b.block(0x09f15a);
    }

    // void Oam_AllocateDeferToPlayer(int k) — sprite.c:2920
    pub(in crate::zelda_rtl) fn oam_allocate_defer_to_player(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x06_f86c);
        crate::cycle_ledger::charge(72);
        if self.sprite_slot_view(k).floor()
            != self.game_state.player.follower_link.lower_level_state()
        {
            crate::cycle_ledger::charge(6 + 42);
            return;
        }
        crate::cycle_ledger::charge(132);
        let right = self.sprite_is_right_of_link(k);
        crate::cycle_ledger::charge_routine(0x06_ead1, if right.a != 0 { 238 } else { 230 });
        if right.b.wrapping_add(0x10) >= 0x20 {
            crate::cycle_ledger::charge(6 + 42);
            return;
        }
        crate::cycle_ledger::charge(132);
        let below = self.sprite_is_below_link(k);
        crate::cycle_ledger::charge_routine(0x06_eae8, if below.a != 0 { 446 } else { 438 });
        if below.b.wrapping_add(0x20) >= 0x48 {
            crate::cycle_ledger::charge(6 + 42);
            return;
        }
        crate::cycle_ledger::charge(122);
        let nslots = ((self.sprite_slot_view(k).flags2() & 0x1f) + 1) << 2;
        if below.a != 0 {
            crate::cycle_ledger::charge(62 + 22 + 42);
            self.oam_allocate_from_region_c(nslots);
        } else {
            crate::cycle_ledger::charge(6 + 62 + 42);
            self.oam_allocate_from_region_b(nslots);
        }
    }
}
