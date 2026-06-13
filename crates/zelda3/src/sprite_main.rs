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
        let poc = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.guard_animate_head(k, 0, &poc);
        let sprite = self.sprite_slot(k);
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
        let sprite = self.sprite_slot(k);
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
        let sprite = self.sprite_slot(k);
        let g = sprite.graphics() as usize * 4;
        let sprite_type = sprite.sprite_type();
        let oam_base = (self.oam_state().current_pointer_usize() - OAM_BUF) / 4;
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
        let sprite = self.sprite_slot(k);
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

    fn set_sprite_main_guard_oam(
        &mut self,
        offset: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        let oam_cur = self.oam_state().current_pointer_usize();
        let index = (oam_cur - OAM_BUF) / 4 + offset;
        self.set_oam_helper0_index(index, x, y, charnum, flags, big);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_body_packs_skipped_oam_and_hides_type_46_blank_tiles() {
        let graphics = (0..SOLDIER_DRAW2_CHAR.len() / 4)
            .find(|&graphics| {
                let emitted: Vec<_> = (0..=3)
                    .rev()
                    .filter_map(|i| {
                        let j = graphics * 4 + i;
                        if SOLDIER_DRAW2_BIG[j] == 0 || i == 3 && SOLDIER_DRAW2_CHAR[j] == 0x20 {
                            None
                        } else {
                            Some(j)
                        }
                    })
                    .collect();
                emitted.len() < 4 && emitted.iter().any(|&j| SOLDIER_DRAW2_CHAR[j] == 0x20)
            })
            .expect("soldier draw table should contain type-46 skipped blank body entries");
        let expected: Vec<_> = (0..=3)
            .rev()
            .filter_map(|i| {
                let j = graphics * 4 + i;
                if SOLDIER_DRAW2_BIG[j] == 0 || i == 3 && SOLDIER_DRAW2_CHAR[j] == 0x20 {
                    None
                } else {
                    Some(SOLDIER_DRAW2_CHAR[j])
                }
            })
            .collect();

        let mut s = ZeldaState::new();
        s.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        for i in 0..4 {
            let base = OAM_BUF + i * 4;
            s.ram[base + 1] = 0xee;
            s.ram[base + 2] = 0xee;
        }
        let k = 0;
        s.sprite_slot_mut(k).set_sprite_type(0x46);
        s.sprite_slot_mut(k).set_graphics(graphics as u8);

        s.guard_animate_body(
            k,
            0,
            &PrepOamCoordsRet {
                x: 0x40,
                y: 0x50,
                r4: 0,
                flags: 0,
            },
        );

        for (slot, &charnum) in expected.iter().enumerate() {
            assert_eq!(s.ram[OAM_BUF + slot * 4 + 2], charnum);
            if charnum == 0x20 {
                assert_eq!(s.ram[OAM_BUF + slot * 4 + 1], 0xf0);
            }
        }
        assert_eq!(s.ram[OAM_BUF + expected.len() * 4 + 2], 0xee);
    }
}
