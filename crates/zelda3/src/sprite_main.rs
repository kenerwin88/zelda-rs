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
        let sprite = self.sprite_slot_view(k);
        let g = sprite.graphics() as usize * 4;
        let sprite_type = sprite.sprite_type();
        let oam_base = (self.game_state.oam.current_pointer_usize() - OAM_BUF) / 4;
        let mut oam_offset = oam_idx as usize;
        for i in (0..=3).rev() {
            let j = i + g;
            if sprite_type >= 0x46
                && (SOLDIER_DRAW2_BIG[j] == 0 || i == 3 && SOLDIER_DRAW2_CHAR[j] == 0x20)
            {
                continue;
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
            oam_offset += 1;
        }
    }

    pub(super) fn guard_animate_weapon(&mut self, k: usize, poc: &PrepOamCoordsRet) {
        let sprite = self.sprite_slot_view(k);
        let oam_idx = SOLDIER_DRAW3_OAM_IDX[sprite.direction() as usize] >> 2;
        let g = sprite.graphics() as usize * 2;
        let sprite_type = sprite.sprite_type();
        for i in (0..=1).rev() {
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
