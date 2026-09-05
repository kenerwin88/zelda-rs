// Methods ported from zelda3/src/sprite_main.c and included inside ZeldaState.

use super::*;
use crate::zelda_rtl::attract::{
    SOLDIER_DRAW1_CHAR, SOLDIER_DRAW1_FLAGS, SOLDIER_DRAW1_YD, SOLDIER_DRAW2_BIG,
    SOLDIER_DRAW2_CHAR, SOLDIER_DRAW2_FLAGS, SOLDIER_DRAW2_OAM_IDX, SOLDIER_DRAW2_XD,
    SOLDIER_DRAW2_YD, SOLDIER_DRAW3_CHAR, SOLDIER_DRAW3_FLAGS, SOLDIER_DRAW3_OAM_IDX,
    SOLDIER_DRAW3_XD, SOLDIER_DRAW3_YD, SOLDIER_DRAW_SHADOW,
};
use crate::zelda_rtl::sprite::PrepOamCoordsRet;

impl ZeldaState {
    pub(super) fn guard_handle_all_animation(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        if self.sprite_slot_view(k).sprite_type() == 0x41 {
            if let Some(workload) = self.last_sprite_main_timing_workload.as_mut() {
                workload.record_blue_guard_full_animation();
            }
        }
        let poc = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.guard_animate_head(k, 0, &poc);
        let sprite = self.sprite_slot_view(k);
        let direction = sprite.direction() as usize;
        let flags3 = sprite.flags3();
        self.guard_animate_body(k, SOLDIER_DRAW2_OAM_IDX[direction] >> 2, &poc);
        self.guard_animate_weapon(k, &poc);
        if flags3 & 0x10 != 0 {
            self.sprite_draw_shadow_custom_attract(
                k,
                (poc.x, poc.y, poc.flags),
                SOLDIER_DRAW_SHADOW[direction],
            );
        }
    }

    pub(super) fn guard_animate_head(&mut self, k: usize, oam_offs: u8, poc: &PrepOamCoordsRet) {
        let sprite = self.sprite_slot_view(k);
        let dir = sprite.head_direction() as usize;
        let graphics = sprite.graphics() as usize;
        self.set_sprite_main_guard_oam(
            oam_offs as usize,
            poc.x,
            poc.y.wrapping_sub(SOLDIER_DRAW1_YD[graphics] as i16 as u16),
            SOLDIER_DRAW1_CHAR[dir],
            SOLDIER_DRAW1_FLAGS[dir] | poc.flags,
            2,
        );
    }

    pub(super) fn guard_animate_body(&mut self, k: usize, oam_idx: u8, poc: &PrepOamCoordsRet) {
        let mut oam_offset = oam_idx as usize;
        for i in (0..=3).rev() {
            if self.guard_animate_body_entry(k, oam_offset, poc, i) {
                oam_offset += 1;
            }
        }
    }

    fn guard_animate_body_entry(
        &mut self,
        k: usize,
        oam_offset: usize,
        poc: &PrepOamCoordsRet,
        i: usize,
    ) -> bool {
        let sprite = self.sprite_slot_view(k);
        let g = sprite.graphics() as usize * 4;
        let sprite_type = sprite.sprite_type();
        let oam_base = (self.game_state.oam.current_pointer_usize() - OAM_BUF) / 4;
        let j = i + g;
        if sprite_type >= 0x46
            && (SOLDIER_DRAW2_BIG[j] == 0 || i == 3 && SOLDIER_DRAW2_CHAR[j] == 0x20)
        {
            return false;
        }
        let mut flags = SOLDIER_DRAW2_FLAGS[j] | poc.flags;
        if SOLDIER_DRAW2_CHAR[j] == 0x20 {
            flags = flags & 0xf1 | 2;
        } else if SOLDIER_DRAW2_BIG[j] == 0 {
            flags = flags & 0xf1 | 8;
        }
        self.set_sprite_main_guard_oam(
            oam_offset,
            poc.x.wrapping_add(SOLDIER_DRAW2_XD[j] as i16 as u16),
            poc.y.wrapping_add(SOLDIER_DRAW2_YD[j] as i16 as u16),
            SOLDIER_DRAW2_CHAR[j],
            flags,
            SOLDIER_DRAW2_BIG[j],
        );
        if SOLDIER_DRAW2_CHAR[j] == 0x20 && sprite_type == 0x46 {
            self.oam_state_mut().hide_sprite_row(oam_base + oam_offset);
        }
        true
    }

    pub(super) fn guard_animate_weapon(&mut self, k: usize, poc: &PrepOamCoordsRet) {
        for i in (0..=1).rev() {
            self.guard_animate_weapon_entry(k, poc, i);
        }
    }

    fn guard_animate_weapon_entry(&mut self, k: usize, poc: &PrepOamCoordsRet, i: usize) {
        let sprite = self.sprite_slot_view(k);
        let oam_idx = SOLDIER_DRAW3_OAM_IDX[sprite.direction() as usize] >> 2;
        let g = sprite.graphics() as usize * 2;
        let sprite_type = sprite.sprite_type();
        let j = i + g;
        self.hitbox_scratch_offset_mut()
            .set_offsets(SOLDIER_DRAW3_YD[j] as u8, SOLDIER_DRAW3_XD[j] as u8);
        self.set_sprite_main_guard_oam(
            oam_idx as usize + (1 - i),
            poc.x.wrapping_add(SOLDIER_DRAW3_XD[j] as i16 as u16),
            poc.y.wrapping_add(SOLDIER_DRAW3_YD[j] as i16 as u16),
            SOLDIER_DRAW3_CHAR[j].wrapping_add(if sprite_type < 0x43 { 3 } else { 0 }),
            SOLDIER_DRAW3_FLAGS[j] | poc.flags,
            0,
        );
    }

    pub(super) fn guard_animation_until_checkpoint(
        &mut self,
        k: usize,
        checkpoint: crate::GuardAnimationCheckpoint,
    ) -> GuardAnimationContinuation {
        use crate::GuardAnimationCheckpoint as Stage;
        match checkpoint {
            Stage::DrawReturned => {
                let (graphics, direction) = self.guard_main_prepare_animation_pose(k);
                self.guard_handle_all_animation(k);
                return GuardAnimationContinuation {
                    graphics,
                    direction,
                    checkpoint,
                    poc_x: 0,
                    poc_y: 0,
                    poc_flags: 0,
                };
            }
            Stage::HeadCharacterPending => return self.guard_animation_until_head_stage(k, false),
            Stage::HeadFlagsPending => return self.guard_animation_until_head_flags(k),
            Stage::WeaponCoordinates { entry } => {
                return self.guard_animation_until_weapon_coordinates(k, entry)
            }
            _ => {}
        }
        assert!(checkpoint.is_valid());
        let mut continuation = self.guard_animation_until_head_flags(k);
        self.guard_finish_head_flags(k, &continuation);
        let poc = PrepOamCoordsRet {
            x: continuation.poc_x,
            y: continuation.poc_y,
            r4: 0,
            flags: continuation.poc_flags,
        };
        let base =
            usize::from(SOLDIER_DRAW2_OAM_IDX[self.sprite_slot_view(k).direction() as usize] >> 2);
        if let Stage::WeaponBeforeCoordinates { entry } = checkpoint {
            self.guard_animate_body(k, base as u8, &poc);
            if entry == 0 {
                self.guard_animate_weapon_entry(k, &poc, 1);
            }
        } else {
            let (entry, coordinates, character) = match checkpoint {
                Stage::BodyBeforeEntry { entry } => (entry, false, false),
                Stage::BodyCoordinates { entry } => (entry, true, false),
                Stage::BodyFlagsPending { entry } => (entry, true, true),
                _ => unreachable!(),
            };
            for i in (usize::from(entry) + 1..=3).rev() {
                assert!(self.guard_animate_body_entry(k, base + 3 - i, &poc, i));
            }
            if coordinates {
                let j = self.sprite_slot_view(k).graphics() as usize * 4 + usize::from(entry);
                let x = poc.x.wrapping_add(SOLDIER_DRAW2_XD[j] as i16 as u16);
                let y = poc.y.wrapping_add(SOLDIER_DRAW2_YD[j] as i16 as u16);
                let y = if y.wrapping_add(16) < 256 { y } else { 0xf0 };
                let addr = self.game_state.oam.current_pointer_usize()
                    + (base + 3 - usize::from(entry)) * 4;
                let mut oam = self.oam_state_mut();
                oam.set_entry_x(addr, x as u8);
                oam.set_entry_y(addr, y as u8);
                oam.set_entry_char(
                    addr,
                    if character {
                        SOLDIER_DRAW2_CHAR[j]
                    } else {
                        (y >> 8) as u8
                    },
                );
            }
        }
        continuation.checkpoint = checkpoint;
        continuation
    }

    fn guard_finish_head_flags(&mut self, k: usize, continuation: &GuardAnimationContinuation) {
        let head = self.sprite_slot_view(k).head_direction() as usize;
        let addr = self.game_state.oam.current_pointer_usize();
        let mut oam = self.oam_state_mut();
        oam.set_entry_flags(addr, SOLDIER_DRAW1_FLAGS[head] | continuation.poc_flags);
        oam.set_extended_byte(
            (addr - OAM_BUF) / 4,
            2 | ((continuation.poc_x >> 8) as u8 & 1),
        );
    }

    pub(super) fn guard_animation_until_head_flags(
        &mut self,
        k: usize,
    ) -> GuardAnimationContinuation {
        self.guard_animation_until_head_stage(k, true)
    }

    fn guard_animation_until_head_stage(
        &mut self,
        k: usize,
        character_stored: bool,
    ) -> GuardAnimationContinuation {
        let (graphics, direction) = self.guard_main_prepare_animation_pose(k);
        let (x, y, flags) = self
            .sprite_prep_oam_coord_or_double_ret(k)
            .expect("source guard head draw requires visible OAM coordinates");
        if self.sprite_slot_view(k).sprite_type() == 0x41 {
            if let Some(workload) = self.last_sprite_main_timing_workload.as_mut() {
                workload.record_blue_guard_full_animation();
            }
        }
        let sprite = self.sprite_slot_view(k);
        let head = sprite.head_direction() as usize;
        let head_y = y.wrapping_sub(SOLDIER_DRAW1_YD[sprite.graphics() as usize] as i16 as u16);
        let head_y = if head_y.wrapping_add(16) < 256 {
            head_y
        } else {
            0xf0
        };
        let addr = self.game_state.oam.current_pointer_usize();
        let mut oam = self.oam_state_mut();
        oam.set_entry_x(addr, x as u8);
        oam.set_entry_y(addr, head_y as u8);
        oam.set_entry_char(
            addr,
            if character_stored {
                SOLDIER_DRAW1_CHAR[head]
            } else {
                (head_y >> 8) as u8
            },
        );
        GuardAnimationContinuation {
            graphics,
            direction,
            checkpoint: if character_stored {
                crate::GuardAnimationCheckpoint::HeadFlagsPending
            } else {
                crate::GuardAnimationCheckpoint::HeadCharacterPending
            },
            poc_x: x,
            poc_y: y,
            poc_flags: flags,
        }
    }

    pub(super) fn complete_guard_animation_at_checkpoint(
        &mut self,
        k: usize,
        continuation: GuardAnimationContinuation,
    ) {
        if continuation.checkpoint == crate::GuardAnimationCheckpoint::DrawReturned {
            self.guard_main_after_animation(k, continuation.graphics, continuation.direction);
            return;
        }
        if matches!(
            continuation.checkpoint,
            crate::GuardAnimationCheckpoint::WeaponCoordinates { .. }
        ) {
            self.complete_guard_animation_after_weapon_coordinates(k, continuation);
            return;
        }
        let poc = PrepOamCoordsRet {
            x: continuation.poc_x,
            y: continuation.poc_y,
            r4: 0,
            flags: continuation.poc_flags,
        };
        let sprite = self.sprite_slot_view(k);
        let direction = sprite.direction() as usize;
        let flags3 = sprite.flags3();
        use crate::GuardAnimationCheckpoint as Stage;
        match continuation.checkpoint {
            Stage::HeadCharacterPending | Stage::HeadFlagsPending => {
                if continuation.checkpoint == Stage::HeadCharacterPending {
                    let head = self.sprite_slot_view(k).head_direction() as usize;
                    let addr = self.game_state.oam.current_pointer_usize();
                    self.oam_state_mut()
                        .set_entry_char(addr, SOLDIER_DRAW1_CHAR[head]);
                }
                self.guard_finish_head_flags(k, &continuation);
                self.guard_animate_body(k, SOLDIER_DRAW2_OAM_IDX[direction] >> 2, &poc);
                self.guard_animate_weapon(k, &poc);
            }
            Stage::WeaponBeforeCoordinates { entry } => {
                for i in (0..=usize::from(entry)).rev() {
                    self.guard_animate_weapon_entry(k, &poc, i);
                }
            }
            Stage::BodyBeforeEntry { entry }
            | Stage::BodyCoordinates { entry }
            | Stage::BodyFlagsPending { entry } => {
                let base = usize::from(SOLDIER_DRAW2_OAM_IDX[direction] >> 2);
                let i = usize::from(entry);
                if matches!(continuation.checkpoint, Stage::BodyBeforeEntry { .. }) {
                    assert!(self.guard_animate_body_entry(k, base + 3 - i, &poc, i));
                } else {
                    let j = self.sprite_slot_view(k).graphics() as usize * 4 + i;
                    let mut flags = SOLDIER_DRAW2_FLAGS[j] | poc.flags;
                    if SOLDIER_DRAW2_CHAR[j] == 0x20 {
                        flags = flags & 0xf1 | 2;
                    } else if SOLDIER_DRAW2_BIG[j] == 0 {
                        flags = flags & 0xf1 | 8;
                    }
                    let x = poc.x.wrapping_add(SOLDIER_DRAW2_XD[j] as i16 as u16);
                    let addr = self.game_state.oam.current_pointer_usize() + (base + 3 - i) * 4;
                    let mut oam = self.oam_state_mut();
                    if matches!(continuation.checkpoint, Stage::BodyCoordinates { .. }) {
                        oam.set_entry_char(addr, SOLDIER_DRAW2_CHAR[j]);
                    }
                    oam.set_entry_flags(addr, flags);
                    oam.set_extended_byte(
                        (addr - OAM_BUF) / 4,
                        SOLDIER_DRAW2_BIG[j] | ((x >> 8) as u8 & 1),
                    );
                }
                for remaining in (0..i).rev() {
                    assert!(self.guard_animate_body_entry(
                        k,
                        base + 3 - remaining,
                        &poc,
                        remaining
                    ));
                }
                self.guard_animate_weapon(k, &poc);
            }
            Stage::WeaponCoordinates { .. } | Stage::DrawReturned => unreachable!(),
        }
        if flags3 & 0x10 != 0 {
            self.sprite_draw_shadow_custom_attract(
                k,
                (poc.x, poc.y, poc.flags),
                SOLDIER_DRAW_SHADOW[direction],
            );
        }
        self.guard_main_after_animation(k, continuation.graphics, continuation.direction);
    }

    /// Guard_AnimateWeapon's current entry has stored its 16-bit coordinates
    /// and clipped Y, but has not stored hitbox offsets, character, flags or ext.
    pub(super) fn guard_animation_until_weapon_coordinates(
        &mut self,
        k: usize,
        entry: u8,
    ) -> GuardAnimationContinuation {
        assert!(entry <= 1);
        let (graphics, direction) = self.guard_main_prepare_animation_pose(k);
        let (x, y, flags) = self
            .sprite_prep_oam_coord_or_double_ret(k)
            .expect("source guard weapon draw requires visible OAM coordinates");
        if self.sprite_slot_view(k).sprite_type() == 0x41 {
            if let Some(workload) = self.last_sprite_main_timing_workload.as_mut() {
                workload.record_blue_guard_full_animation();
            }
        }
        let poc = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.guard_animate_head(k, 0, &poc);
        let direction_index = self.sprite_slot_view(k).direction() as usize;
        self.guard_animate_body(k, SOLDIER_DRAW2_OAM_IDX[direction_index] >> 2, &poc);
        if entry == 0 {
            self.guard_animate_weapon_entry(k, &poc, 1);
        }
        let j = self.sprite_slot_view(k).graphics() as usize * 2 + usize::from(entry);
        let x = poc.x.wrapping_add(SOLDIER_DRAW3_XD[j] as i16 as u16);
        let y = poc.y.wrapping_add(SOLDIER_DRAW3_YD[j] as i16 as u16);
        let y = if y.wrapping_add(16) < 256 { y } else { 0xf0 };
        let index = (self.game_state.oam.current_pointer_usize() - OAM_BUF) / 4
            + usize::from(SOLDIER_DRAW3_OAM_IDX[direction_index] >> 2)
            + usize::from(1 - entry);
        let addr = OAM_BUF + index * 4;
        let mut oam = self.oam_state_mut();
        oam.set_entry_x(addr, x as u8);
        oam.set_entry_y(addr, y as u8);
        // The ROM's 16-bit Y store also touches the next character byte.
        oam.set_entry_char(addr, (y >> 8) as u8);
        GuardAnimationContinuation {
            graphics,
            direction,
            checkpoint: crate::GuardAnimationCheckpoint::WeaponCoordinates { entry },
            poc_x: poc.x,
            poc_y: poc.y,
            poc_flags: poc.flags,
        }
    }

    pub(super) fn complete_guard_animation_after_weapon_coordinates(
        &mut self,
        k: usize,
        continuation: GuardAnimationContinuation,
    ) {
        let poc = PrepOamCoordsRet {
            x: continuation.poc_x,
            y: continuation.poc_y,
            r4: 0,
            flags: continuation.poc_flags,
        };
        let crate::GuardAnimationCheckpoint::WeaponCoordinates { entry } = continuation.checkpoint
        else {
            unreachable!("weapon draw resumed at another checkpoint")
        };
        let sprite = self.sprite_slot_view(k);
        let direction = sprite.direction() as usize;
        let flags3 = sprite.flags3();
        let j = sprite.graphics() as usize * 2 + usize::from(entry);
        let charnum =
            SOLDIER_DRAW3_CHAR[j].wrapping_add(if sprite.sprite_type() < 0x43 { 3 } else { 0 });
        let index = (self.game_state.oam.current_pointer_usize() - OAM_BUF) / 4
            + usize::from(SOLDIER_DRAW3_OAM_IDX[direction] >> 2)
            + usize::from(1 - entry);
        let x = poc.x.wrapping_add(SOLDIER_DRAW3_XD[j] as i16 as u16);
        self.hitbox_scratch_offset_mut()
            .set_offsets(SOLDIER_DRAW3_YD[j] as u8, SOLDIER_DRAW3_XD[j] as u8);
        {
            let mut oam = self.oam_state_mut();
            oam.set_entry_char(OAM_BUF + index * 4, charnum);
            oam.set_entry_flags(OAM_BUF + index * 4, SOLDIER_DRAW3_FLAGS[j] | poc.flags);
            oam.set_extended_byte(index, (x >> 8) as u8 & 1);
        }
        if entry == 1 {
            self.guard_animate_weapon_entry(k, &poc, 0);
        }
        if flags3 & 0x10 != 0 {
            self.sprite_draw_shadow_custom_attract(
                k,
                (poc.x, poc.y, poc.flags),
                SOLDIER_DRAW_SHADOW[direction],
            );
        }
        self.guard_main_after_animation(k, continuation.graphics, continuation.direction);
    }

    /// Execute the first nested guard draw in
    /// `SpritePrep_TrooperAndArcherSoldier` through the character store of
    /// the final weapon entry. The next ROM instruction is that entry's flags
    /// store (`$05:cbcd`), so bytewise extended OAM is deliberately untouched.
    pub(super) fn guard_prep_first_animation_until_weapon_flags(
        &mut self,
        k: usize,
        saved_submodule: u8,
    ) -> Option<GuardPrepWeaponDrawContinuation> {
        let guard_bak_graphics = self.sprite_slot_view(k).graphics();
        let guard_bak_direction = self.sprite_slot_view(k).direction();
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.set_direction(
                crate::zelda_rtl::sprite_main_guard::SOLDIER_DIRECTION_LOCK_SETTINGS
                    [(guard_bak_direction as usize) & 3],
            );
            sprite.set_graphics(
                crate::zelda_rtl::sprite_main_guard::SOLDIER_GRAPHICS_BY_DIRECTION
                    [(guard_bak_direction as usize) & 3],
            );
        }
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return None;
        };
        if self.sprite_slot_view(k).sprite_type() == 0x41 {
            if let Some(workload) = self.last_sprite_main_timing_workload.as_mut() {
                workload.record_blue_guard_full_animation();
            }
        }
        let poc = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.guard_animate_head(k, 0, &poc);
        let sprite = self.sprite_slot_view(k);
        let draw_direction = sprite.direction();
        let draw_flags3 = sprite.flags3();
        self.guard_animate_body(k, SOLDIER_DRAW2_OAM_IDX[draw_direction as usize] >> 2, &poc);

        let oam_idx = SOLDIER_DRAW3_OAM_IDX[draw_direction as usize] >> 2;
        let g = self.sprite_slot_view(k).graphics() as usize * 2;
        let sprite_type = self.sprite_slot_view(k).sprite_type();
        // i=1 is the first complete source loop iteration.
        let first_j = 1 + g;
        self.hitbox_scratch_offset_mut().set_offsets(
            SOLDIER_DRAW3_YD[first_j] as u8,
            SOLDIER_DRAW3_XD[first_j] as u8,
        );
        self.set_sprite_main_guard_oam(
            oam_idx as usize,
            poc.x.wrapping_add(SOLDIER_DRAW3_XD[first_j] as i16 as u16),
            poc.y.wrapping_add(SOLDIER_DRAW3_YD[first_j] as i16 as u16),
            SOLDIER_DRAW3_CHAR[first_j].wrapping_add(if sprite_type < 0x43 { 3 } else { 0 }),
            SOLDIER_DRAW3_FLAGS[first_j] | poc.flags,
            0,
        );

        // i=0 has committed X, clipped Y and character, but not flags/ext.
        let pending_j = g;
        self.hitbox_scratch_offset_mut().set_offsets(
            SOLDIER_DRAW3_YD[pending_j] as u8,
            SOLDIER_DRAW3_XD[pending_j] as u8,
        );
        let pending_oam_index =
            ((self.game_state.oam.current_pointer_usize() - OAM_BUF) / 4) + oam_idx as usize + 1;
        let pending_oam_x = poc
            .x
            .wrapping_add(SOLDIER_DRAW3_XD[pending_j] as i16 as u16);
        let pending_oam_y = poc
            .y
            .wrapping_add(SOLDIER_DRAW3_YD[pending_j] as i16 as u16);
        let clipped_y = if pending_oam_y.wrapping_add(0x10) < 0x100 {
            pending_oam_y as u8
        } else {
            0xf0
        };
        let pending_oam_char =
            SOLDIER_DRAW3_CHAR[pending_j].wrapping_add(if sprite_type < 0x43 { 3 } else { 0 });
        let addr = OAM_BUF + pending_oam_index * 4;
        {
            let mut oam = self.oam_state_mut();
            oam.set_entry_x(addr, pending_oam_x as u8);
            oam.set_entry_y(addr, clipped_y);
            oam.set_entry_char(addr, pending_oam_char);
        }
        Some(GuardPrepWeaponDrawContinuation {
            saved_submodule,
            guard_bak_graphics,
            guard_bak_direction,
            draw_flags3,
            draw_direction,
            poc_x: poc.x,
            poc_y: poc.y,
            poc_flags: poc.flags,
            pending_oam_index: pending_oam_index as u16,
            pending_oam_x,
            pending_oam_flags: SOLDIER_DRAW3_FLAGS[pending_j] | poc.flags,
        })
    }

    pub(super) fn complete_guard_prep_first_animation_after_weapon_flags(
        &mut self,
        k: usize,
        continuation: GuardPrepWeaponDrawContinuation,
    ) {
        let oam_index = continuation.pending_oam_index as usize;
        let addr = OAM_BUF + oam_index * 4;
        {
            let mut oam = self.oam_state_mut();
            oam.set_entry_flags(addr, continuation.pending_oam_flags);
            oam.set_extended_byte(oam_index, (continuation.pending_oam_x >> 8) as u8 & 1);
        }
        if continuation.draw_flags3 & 0x10 != 0 {
            self.sprite_draw_shadow_custom_attract(
                k,
                (
                    continuation.poc_x,
                    continuation.poc_y,
                    continuation.poc_flags,
                ),
                SOLDIER_DRAW_SHADOW[continuation.draw_direction as usize],
            );
        }
        self.guard_main_after_animation(
            k,
            continuation.guard_bak_graphics,
            continuation.guard_bak_direction,
        );
    }

    fn set_sprite_main_guard_oam(
        &mut self,
        offset: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        let oam_cur = self.game_state.oam.current_pointer_usize();
        let index = (oam_cur - OAM_BUF) / 4 + offset;
        self.set_oam_helper0_index(index, x, y, charnum, flags, big);
    }
}

#[cfg(test)]
#[path = "sprite_main_tests.rs"]
mod tests;
