//! Octoroks, Moblins, Lynels, Hinox, and other overworld enemies.

use crate::zelda_rtl::sprite::{
    PrepOamCoordsRet, SpriteDamageBothEntry, SpriteDamageFromEntry, SpriteDamageToEntry,
    SpriteInactiveEntry, SpriteMoveEntry, SpriteMoveZEntry, SpritePausedEntry, SpriteRecoilEntry,
    SpriteShadowEntry, SpriteSingleLargeEntry, SpriteSingleSmallEntry, SpriteSpawnInfo,
    SpriteTileCollisionEntry,
};
use super::*;
use crate::tile_definition::NativeTile;
use crate::types::sign8;

impl ZeldaState {
    pub(in crate::zelda_rtl) fn zazak_draw_marked(&mut self, k: usize) {
        // $0D:C0A6..C0F2; DrawMultiple and the far shadow own their bodies.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x0dc0a6));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x0dc0a6);
        self.zazak_draw_with_markers(k, b);
    }



    // -----------------------------------------------------------------------
    // void SpriteDraw_ZirroBomb(int k) {  // 8dd606 — sprite_main.c:13246
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_zirro_bomb(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        let base = ((self.sprite_slot_view(k).delay_main() >> 2) as usize) * 3;
        self.sprite_draw_multiple(k, &BOMBER_PELLET_DRAW_FRAMES[base..base + 3]);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Pikit_Tongue(int k, PrepOamCoordsRet *info) {  // 8dd74a — sprite_main.c:13316
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_pikit_tongue(&mut self, k: usize, info: &PrepOamCoordsRet) {
        if self.sprite_slot_view(k).ai_state() != 2 || self.sprite_slot_view(k).pause() != 0 {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let x = info.x.wrapping_add(4) as u8;
        let y = info.y.wrapping_add(3) as u8;
        // oam[5]: 20 byte offset from oam base.
        let head_x = x.wrapping_add(self.sprite_slot_view(k).a());
        let head_y = y.wrapping_add(self.sprite_slot_view(k).b());
        let mut oam_view = self.oam_state_mut();
        oam_view.write_entry(oam + 5 * 4, x, y, 0xfe, info.flags);
        // oam[0]:
        oam_view.write_entry(oam, head_x, head_y, 0xfe, info.flags);
        // oam++; loop over i=3..=0 starting at oam[1].
        let mut cur = oam + 4;
        let g = self.sprite_slot_view(k).direction() as usize;
        let mut i: i32 = 3;
        loop {
            let sa = self.sprite_slot_view(k).a() as i8 as i32;
            let sb = self.sprite_slot_view(k).b() as i8 as i32;
            let xv = x as i32 + (sa * PIKIT_TONGUE_LENGTH_MULTIPLIERS[i as usize] as i32) / 256;
            let yv = y as i32 + (sb * PIKIT_TONGUE_LENGTH_MULTIPLIERS[i as usize] as i32) / 256;
            self.oam_state_mut().write_entry(
                cur,
                xv as u8,
                yv as u8,
                PIKIT_TONGUE_DRAW_CHARS[g],
                PIKIT_TONGUE_DRAW_FLAGS[g] | info.flags,
            );
            cur += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.sprite_correct_oam_entries(k, 5, 0);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Pikit_Loot(int k, PrepOamCoordsRet *info) {  // 8dd858 — sprite_main.c:13341
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_pikit_loot(&mut self, k: usize, _info: &PrepOamCoordsRet) {
        if self.sprite_slot_view(k).g() == 0 {
            return;
        }
        let mut g = (self.sprite_slot_view(k).g() as usize) - 1;
        if g == 3 {
            g = (self.sprite_slot_view(k).subtype() as usize) + 2;
        }
        self.oam_allocate_from_region_c(0x10);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let tmp_x = self.game_state.scratch_counter.value();
        let tmp_y = self.game_state.sprites.workspace.shared_scratch_a();
        let mut i: i32 = 3;
        loop {
            let j = g * 4 + i as usize;
            self.oam_state_mut().write_entry(
                oam,
                tmp_x.wrapping_add(PIKIT_GRABBED_ITEM_X_OFFSETS[j] as u8),
                tmp_y.wrapping_add(PIKIT_GRABBED_ITEM_Y_OFFSETS[j] as u8),
                PIKIT_GRABBED_ITEM_CHARS[j],
                PIKIT_GRABBED_ITEM_FLAGS[g],
            );
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.sprite_correct_oam_entries(k, 3, 0);
    }

    // -----------------------------------------------------------------------
    // void Octorock_Draw(int k) {  // 86d54a
    //   Optional mouth tile followed by the canonical large body tile.
    // }
    pub(super) fn octorock_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
        if self.sprite_slot_view(k).direction() != 3 {
            let oam = self.game_state.oam.current_pointer_usize();
            let j = (self.sprite_slot_view(k).c() as usize) * 3
                + self.sprite_slot_view(k).direction() as usize;
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(OCTOROCK_DRAW_X_OFFSETS[j] as i16 as u16),
                y.wrapping_add(OCTOROCK_DRAW_Y_OFFSETS[j] as i16 as u16),
                OCTOROCK_DRAW_CHARS[j],
                OCTOROCK_DRAW_FLAGS[j] | flags,
                0,
            );
        }
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        self.sprite_slot_view_mut(k).subtract_flags2(1);
        self.sprite_prep_and_draw_single_large_no_prep(k, &mut info);
        self.sprite_slot_view_mut(k).add_flags2(1);
    }

    // -----------------------------------------------------------------------
    // void Boulder_Draw(int k) {  // 9dd185
    //   Four large rotating tiles plus large shadow.
    // }
    pub(in crate::zelda_rtl) fn boulder_draw(&mut self, k: usize) {
        // $1D:D02F JSR $D185: complete draw caller, excluding its children.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1dd185));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1dd185);
        let base = (((self.sprite_slot_view(k).subtype2() >> 3) & 3) as usize) * 4;
        self.sprite_draw_multiple(k, &BOULDER_DRAW_FRAMES[base..base + 4]);
        b.block(0x1dd1a3);
        self.sprite_draw_large_shadow2(k);
        b.block(0x1dd1a7);
    }

    // -----------------------------------------------------------------------
    // void Zazak_Draw(int k) {  // 8dc0a6
    //   Three body tiles, head-char patch, and common shadow.
    // }
    fn zazak_draw_with_markers(&mut self, k: usize, b: crate::cycle_ledger::BlockMarkers) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 3;
        let mut info = self.sprite_draw_multiple(k, &ZAZAK_DRAW_FRAMES[base..base + 3]);
        b.block(0x0dc0c6);
        if self.sprite_slot_view(k).pause() != 0 {
            b.taken(0x0dc0c9); b.block(0x0dc0f1);
            return;
        }
        b.block(0x0dc0cb);
        if self.sprite_slot_view(k).delay_aux1() == 0 { b.taken(0x0dc0d5); }
        else { b.block(0x0dc0d7); }
        let i = (self.sprite_slot_view(k).head_direction() as usize)
            + if self.sprite_slot_view(k).delay_aux1() == 0 {
                0
            } else {
                4
            };
        b.block(0x0dc0db);
        let oam = self.game_state.oam.current_pointer_usize();
        if (oam & 255) + 3 > 255 { b.crossing(0x0dc0e3); }
        self.oam_state_mut()
            .set_entry_char(oam, ZAZAK_DRAW_CHARS[i]);
        self.oam_state_mut()
            .merge_entry_flags(oam, !0x40, ZAZAK_DRAW_FLAGS[i]);
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Unpriced);
        b.block(0x0dc0f1);
    }

    // -----------------------------------------------------------------------
    // void Leever_Draw(int k) {  // 86ce45
    //   Variable one/four tile body selected by sprite_graphics.
    // }
    pub(super) fn leever_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = (self.sprite_slot_view(k).graphics() as usize).min(13);
        for i in (0..=LEEVER_DRAW_COUNTS[d] as usize).rev() {
            let j = d * 4 + i;
            let charnum = LEEVER_DRAW_CHARS[j];
            let mut f = flags;
            if charnum >= 0x60 || charnum == 0x28 || charnum == 0x38 {
                f &= 0xf0;
            }
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(LEEVER_DRAW_X_OFFSETS[j] as i16 as u16),
                y.wrapping_add(LEEVER_DRAW_Y_OFFSETS[j] as i16 as u16),
                charnum,
                LEEVER_DRAW_FLAGS[j] | f,
                LEEVER_DRAW_SIZES[j],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Bomber_Draw(int k) {  // 8dd56c
    //   Two body tiles selected by sprite_graphics plus common shadow.
    // }
    pub(in crate::zelda_rtl) fn bomber_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 2;
        let mut info = self.sprite_draw_multiple(k, &BOMBER_DRAW_FRAMES[base..base + 2]);
        // ROM $0D:D588
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
    }

    // -----------------------------------------------------------------------
    // void Sprite_D0_Lynel(int k) {  // 9d866a
    pub(super) fn sprite_d0_lynel(&mut self, k: usize) {
        self.lynel_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Rom1DE8C5) {
            return;
        }
        let value = self.sprite_direction_to_face_link(k);
        self.sprite_slot_view_mut(k).set_direction(value);
        // $1D:867A JSR $1D:C211: both complete damage calls.
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom1DC211);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = usize::from(self.sprite_slot_view(k).direction());
                    let x = self
                        .game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_add(SPRITE_D0_LYNEL_X_TARGETS[j] as i16 as u16);
                    let value = x as u8;
                    self.sprite_slot_view_mut(k).set_a(value);
                    let value = (x >> 8) as u8;
                    self.sprite_slot_view_mut(k).set_b(value);
                    let y = self
                        .game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(SPRITE_D0_LYNEL_Y_TARGETS[j] as i16 as u16);
                    let value = y as u8;
                    self.sprite_slot_view_mut(k).set_c(value);
                    let value = (y >> 8) as u8;
                    self.sprite_slot_view_mut(k).set_e(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 80;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                let idx = usize::from(
                    (self.sprite_slot_view(k).subtype2() & 4)
                        | self.sprite_slot_view(k).direction(),
                );
                let value = SPRITE_D0_LYNEL_LOCAL_GRAPHICS[idx];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
                        let x = u16::from(self.sprite_slot_view(k).a())
                            | (u16::from(self.sprite_slot_view(k).b()) << 8);
                        let y = u16::from(self.sprite_slot_view(k).c())
                            | (u16::from(self.sprite_slot_view(k).e()) << 8);
                        if x.wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                            .wrapping_add(5)
                            < 10
                            && y.wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                                .wrapping_add(5)
                                < 10
                        {
                            self.lynel_increment_state(k);
                            return;
                        }
                        let pt = self.sprite_project_speed_towards_location(k, x, y, 24);
                        let value = pt.x;
                        self.sprite_slot_view_mut(k).set_x_velocity(value);
                        let value = pt.y;
                        self.sprite_slot_view_mut(k).set_y_velocity(value);
                    }
                    self.sprite_move_xy(k, SpriteMoveEntry::Rom1DE95D);
                    if self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced) != 0 {
                        self.lynel_increment_state(k);
                        return;
                    }
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    let idx = usize::from(
                        (self.sprite_slot_view(k).subtype2() & 4)
                            | self.sprite_slot_view(k).direction(),
                    );
                    let value = SPRITE_D0_LYNEL_LOCAL_GRAPHICS[idx];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                } else {
                    self.lynel_increment_state(k);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    // ROM Sprite_D0_Lynel $1D:8745: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(15, 16);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    return;
                }
                if self.sprite_slot_view(k).delay_main() == 16 {
                    let j = self.sprite_spawn_fire_phlegm(k);
                    if j >= 0 && self.game_state.inventory.items.shield_type() != 3 {
                        let value = 0;
                        self.sprite_slot_view_mut(j as usize).set_flags5(value);
                    }
                }
                let value =
                    SPRITE_D0_LYNEL_ATTACK_GFX[self.sprite_slot_view(k).direction() as usize];
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced);
            }
            _ => {}
        }
    }

    fn lynel_increment_state(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_ai_state(1);
        let value = 32;
        self.sprite_slot_view_mut(k).set_delay_main(value);
    }

    // -----------------------------------------------------------------------
    // void BuzzBlob_Draw(int k) {  // 86d953
    pub(super) fn buzz_blob_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = PrepOamCoordsRet {
            x: info_x,
            y: info_y,
            r4: 0,
            flags: info_flags,
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        for i in (0..3).rev() {
            let ch = BUZZ_BLOB_DRAW_CHARS[g * 3 + i];
            self.set_oam_helper0_at(
                oam,
                info_x.wrapping_add(BUZZ_BLOB_DRAW_X_OFFSETS[i]),
                info_y.wrapping_add(BUZZ_BLOB_DRAW_Y_OFFSETS[i] as u16),
                ch,
                BUZZ_BLOB_DRAW_FL[g * 3 + i] | info_flags,
                BUZZ_BLOB_DRAW_EXT[i],
            );
            if ch == 0 {
                self.oam_state_mut().hide_entry(oam);
            }
            oam += 4;
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Unpriced);
    }

    // -----------------------------------------------------------------------
    // void Cukeman_Draw(int k) {  // 9afb0e
    pub(super) fn cukeman_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        self.sprite_draw_multiple(k, &CUKEMAN_DRAW_FRAMES[base..base + 3]);
    }

    // -----------------------------------------------------------------------
    // void Hokbok_Draw(int k) {  // 9dc77d
    pub(in crate::zelda_rtl) fn hokbok_draw(&mut self, k: usize) {
        let Some((info_x, mut y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1dDoubleRet) else {
            return;
        };
        let mut info = PrepOamCoordsRet {
            x: info_x,
            y,
            r4: 0,
            flags: info_flags,
        };
        let mut oam = self.game_state.oam.current_pointer_usize() + 12;
        let d = self.sprite_slot_view(k).b();
        for i in (0..=self.sprite_slot_view(k).a()).rev() {
            let ch =
                (if i == 0 { 0xa2u8 } else { 0xa0u8 }).wrapping_sub(if d < 7 { 0x20 } else { 0 });
            self.set_oam_helper0_at(oam, info_x, y, ch, info_flags, 2);
            y = y.wrapping_sub(u16::from(d));
            oam = oam.wrapping_sub(4);
        }
        // ROM $1D:C7E6
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
    }

    // -----------------------------------------------------------------------
    // void SnapDragon_Draw(int k) {  // 869e02
    pub(super) fn snap_dragon_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        let mut info = self.sprite_draw_multiple(k, &SNAP_DRAGON_DRAW_FRAMES[base..base + 4]);
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Unpriced);
    }

    // -----------------------------------------------------------------------
    // void Lynel_Draw(int k) {  // 9d8880
    pub(super) fn lynel_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        let mut info = self.sprite_draw_multiple(k, &LYNEL_DRAW_FRAMES[base..base + 3]);
        // ROM $1D:889C
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
    }

    // -----------------------------------------------------------------------
    // void Sprite_17_Hoarder(int k) {  // 86a86c
    pub(super) fn sprite_17_hoarder(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_hoarder_frantic(k);
        } else {
            self.sprite_hoarder_covered(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hoarder_Covered(int k) {  // 86a874
    pub(super) fn sprite_hoarder_covered(&mut self, k: usize) {
        self.covered_rupee_crab_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let (dir, pt) = self.sprite_direction_and_offset_to_face_link(k);
        let dir = usize::from(dir);
        if self.sprite_slot_view(k).delay_main() != 0
            || (pt.y.wrapping_add(0x30) < 0x60 && pt.x.wrapping_add(0x20) < 0x40)
        {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 32;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            let value = SPRITE_HOARDER_COVERED_LOCAL_X_VELOCITIES[dir] as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = SPRITE_HOARDER_COVERED_LOCAL_Y_VELOCITIES[dir] as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            if self.sprite_slot_view(k).wall_collision() == 0 {
                // ROM $06:A8AD
                self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
            }
            self.sprite_check_tile_collision2(k);
            self.sprite_check_damage_from_link(k, SpriteDamageFromEntry::Unpriced);
            self.sprite_slot_view_mut(k).add_subtype2(1);
            let value = SPRITE_HOARDER_COVERED_LOCAL_GRAPHICS
                [usize::from(self.sprite_slot_view(k).subtype2() >> 1) & 3];
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        if self.sprite_slot_view(k).sprite_type() != 0x3e
            || self.game_state.inventory.items.gloves() >= 1
        {
            self.sprite_return_if_lifted_permissive(k);
        }
        if self.sprite_slot_view(k).state() != 9 {
            let value = if self.sprite_slot_view(k).sprite_type() == 0x17 {
                2
            } else {
                1
            };
            self.sprite_slot_view_mut(k).set_c(value);
            let value = 0xec;
            self.sprite_slot_view_mut(k).set_sprite_type(value);
            self.sprite_slot_view_mut(k).and_oam_flags(!1);
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x3e, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_slot_view_mut(j).and_flags2(!0x80);
                let value = 128;
                self.sprite_slot_view_mut(j).set_delay_aux2(value);
                let value = 9;
                self.sprite_slot_view_mut(j).set_oam_flags(value);
                let value = 9;
                self.sprite_slot_view_mut(j).set_ai_state(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hoarder_Frantic(int k) {  // 86a91d
    pub(super) fn sprite_hoarder_frantic(&mut self, k: usize) {
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        self.sprite_check_damage_from_link(k, SpriteDamageFromEntry::Unpriced);
        if self.sprite_slot_view(k).delay_aux2() == 0 {
            self.sprite_check_damage_to_link(k, SpriteDamageToEntry::Unpriced);
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let mut j = usize::from(self.sprite_slot_view(k).subtype2() >> 1) & 3;
        let value = SPRITE_HOARDER_FRANTIC_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value =
            (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_HOARDER_FRANTIC_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if self.sprite_slot_view(k).wall_collision() != 0 {
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
            j = usize::from(self.get_random_number() & 3);
            let value = SPRITE_HOARDER_FRANTIC_LOCAL_X_VELOCITIES[j] as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = SPRITE_HOARDER_FRANTIC_LOCAL_Y_VELOCITIES[j] as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        } else {
            // ROM $06:A96B
            self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
        }
        self.sprite_check_tile_collision2(k);
        if self.sprite_slot_view(k).delay_aux4() == 0
            && (((k as u8) ^ self.game_state.frame.frame_counter) & 31) == 0
        {
            let pt = self.sprite_project_speed_towards_link(k, 16);
            let value = pt.y.wrapping_neg();
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let value = pt.x.wrapping_neg();
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }
        if (self.game_state.frame.frame_counter & 1) != 0 {
            return;
        }

        self.sprite_slot_view_mut(k).add_g(1);
        let (end, typ) = if self.sprite_slot_view(k).g() == 192 {
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 6;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_slot_view_mut(k).add_flags2(4);
            (1, 0xd9)
        } else {
            if (self.sprite_slot_view(k).g() & 15) != 0 {
                return;
            }
            (
                0,
                if self.sprite_slot_view(k).head_direction() == 6 {
                    0xdb
                } else {
                    0xd9
                },
            )
        };
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, typ, &mut info, end);
        if j >= 0 {
            let j = j as usize;
            self.sprite_slot_view_mut(k).add_head_direction(1);
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_set_x(j, info.r0_x.wrapping_add(8));
            let value = 32;
            self.sprite_slot_view_mut(j).set_z_velocity(value);
            let value = 16;
            self.sprite_slot_view_mut(j).set_delay_aux4(value);
            let pt = self.sprite_project_speed_towards_link(j, 16);
            let value = !pt.y;
            self.sprite_slot_view_mut(j).set_y_velocity(value);
            let value = !pt.x;
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x30);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_EC_ThrownItem(int k) {  // 86aae0
    pub(super) fn sprite_ec_thrown_item(&mut self, k: usize) {
        if self.game_state.sprites.system.chr_halfslot_state() < 3 {
            if self.game_state.oam.has_sprite_sorting() && self.sprite_slot_view(k).floor() != 0 {
                let spr_slot = 0x2c + (k & 3);
                self.oam_state_mut()
                    .set_current_pointer((0x0800 + spr_slot * 4) as u16);
                self.oam_state_mut()
                    .set_current_extended_pointer((0x0a20 + spr_slot) as u16);
            }
            let value = self.sprite_slot_view(k).state();
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            if self.sprite_slot_view(k).c() >= 6 {
                self.sprite_draw_thrown_item_gigantic(k);
            } else {
                self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
                let oam = self.game_state.oam.current_pointer_usize();
                let t = self
                    .game_state
                    .world
                    .location
                    .indoor_flag
                    .wrapping_add(self.game_state.world.region.dark_world_region_index());
                let j = usize::from(self.sprite_slot_view(k).c());
                let charnum = THROWABLE_SCENERY_CHARS[j + if t >= 2 { 6 } else { 0 }];
                self.oam_state_mut().set_entry_char(oam, charnum);
                self.oam_state_mut()
                    .merge_entry_flags(oam, 0xf0, THROWABLE_SCENERY_FLAGS[j]);
                let value = (self.sprite_slot_view(k).oam_flags() & 0xc0)
                    | (self.game_state.oam.entry_flags(oam) & 0x0f);
                self.sprite_slot_view_mut(k).set_oam_flags(value);
            }
        }
        if self.sprite_slot_view(k).state() == 9 {
            if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
                return;
            }
            self.throwable_scenery_interact_with_sprites_and_tiles(k);
        }
    }

    // -----------------------------------------------------------------------
    // void CoveredRupeeCrab_Draw(int k) {  // 86aa48
    pub(super) fn covered_rupee_crab_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        if self.game_state.sprites.system.chr_halfslot_state() >= 3 {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let r7 = if self.sprite_slot_view(k).sprite_type() == 0x17 {
            2
        } else {
            0
        };
        let r6 = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        for i in (0..2).rev() {
            let j = i + r6;
            let ch = COVERED_RUPEE_CRAB_DRAW_CHARS[j];
            self.set_oam_helper0_at(
                oam,
                info_x,
                info_y.wrapping_add(COVERED_RUPEE_CRAB_DRAW_Y_OFFSETS[j] as i16 as u16),
                ch.wrapping_add(if ch == 0x44 { r7 } else { 0 }),
                (info_flags & !1) | COVERED_RUPEE_CRAB_DRAW_FL[j],
                2,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_0F_Octoballoon(int k) {  // 86d6aa
    pub(super) fn sprite_0_f_octoballoon(&mut self, k: usize) {
        let value =
            SPRITE_0_F_OCTOBALLOON_Z[usize::from((self.sprite_slot_view(k).subtype2() >> 3) & 7)];

        self.sprite_slot_view_mut(k).set_z(value);
        self.octoballoon_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 3;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            if !self.octoballoon_find() {
                let value = 6;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_hit_timer(value);
                let value = 15;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                return;
            }
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        if ((k as u8) ^ self.game_state.frame.frame_counter) & 15 == 0 {
            let pt = self.sprite_project_speed_towards_link(k, 4);
            let dx = self.sprite_slot_view(k).x_velocity().wrapping_sub(pt.x);
            if dx != 0 {
                let value = self
                    .sprite_slot_view(k)
                    .x_velocity()
                    .wrapping_add(if sign8(dx) { 1 } else { 0xff });
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            let dy = self.sprite_slot_view(k).y_velocity().wrapping_sub(pt.y);
            if dy != 0 {
                let value = self
                    .sprite_slot_view(k)
                    .y_velocity()
                    .wrapping_add(if sign8(dy) { 1 } else { 0xff });
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
        }
        // ROM $06:D716
        self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
        if self.sprite_check_damage_to_link(k, SpriteDamageToEntry::Unpriced) {
            self.octoballoon_recoil_link(k);
        }
        self.sprite_check_damage_from_link(k, SpriteDamageFromEntry::Unpriced);
        let _ = self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced);
        self.sprite_bounce_off_wall(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_10_OctoballoonBaby(int k) {  // 86d853
    pub(super) fn sprite_10_octoballoon_baby(&mut self, k: usize) {
        if self.sprite_slot_view(k).subtype2() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        if self.sprite_slot_view(k).subtype2() >= 64
            || (self.sprite_slot_view(k).subtype2() & 1) == 0
        {
            // ROM $06:D863
            self.sprite_draw_single_small(k, SpriteSingleSmallEntry::Rom06DCEF);
        }
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_slot_view_mut(k).subtract_subtype2(1);
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        self.sprite_slot_view_mut(k).subtract_z_velocity(1);
        // ROM $06:D872
        self.sprite_move_z(k, SpriteMoveZEntry::Rom06E96C);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 16;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
        }
        // ROM $06:D882
        self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
        let _ = self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced);
        self.sprite_bounce_off_wall(k);
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Unpriced);
    }

    // -----------------------------------------------------------------------
    // void Sprite_0D_Buzzblob(int k) {  // 86d89a
    pub(in crate::zelda_rtl) fn sprite_0_d_buzzblob(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            let value = (self.sprite_slot_view(k).object_priority() & 0xf1)
                | SPRITE_0_D_BUZZBLOB_OBJ_PRIO
                    [usize::from((self.sprite_slot_view(k).delay_aux1() >> 1) & 3)];
            self.sprite_slot_view_mut(k).set_object_priority(value);
        }
        self.sprite_cukeman(k);
        self.buzz_blob_draw(k);
        let value = SPRITE_0_D_BUZZBLOB_LOCAL_GRAPHICS
            [usize::from((self.sprite_slot_view(k).subtype2() >> 3) & 3)]
            + if self.sprite_slot_view(k).delay_aux1() != 0 {
                3
            } else {
                0
            };
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        if self.sprite_slot_view(k).delay_main() == 0 {
            self.buzzblob_select_new_direction(k);
        }
        if self.sprite_slot_view(k).delay_aux1() == 0 {
            if let Some(SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
                slot,
                pending: None,
            }) = self.sprite_main_cpu_boundary
            {
                if usize::from(slot) == k {
                    let pending = self.sprite_move_x_through_subpixel(k);
                    self.sprite_main_cpu_boundary =
                        Some(SpriteMainCpuBoundary::BuzzblobAfterXSubpixel {
                            slot,
                            pending: Some(pending),
                        });
                    return;
                }
            }
            // ROM $06:D8E2
            self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
        }
        self.buzzblob_after_movement(k);
    }

    pub(in crate::zelda_rtl) fn buzzblob_after_movement(&mut self, k: usize) {
        let _ = self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced);
        self.sprite_bounce_off_wall(k);
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Unpriced);
    }

    // -----------------------------------------------------------------------
    // void Sprite_08_Octorok(int k) {  // 86d377
    pub(super) fn sprite_08_octorok(&mut self, k: usize) {
        let mut j = self.sprite_slot_view(k).direction();
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            let value = SPRITE_08_OCTOROK_DIRECTIONS[usize::from(j)];
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40)
            | SPRITE_08_OCTOROK_OAM_FLAGS[usize::from(j)]
            | if self.sprite_slot_view(k).graphics() == 7 {
                0x40
            } else {
                0
            };
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.octorock_draw(k);
        let value = j;
        self.sprite_slot_view_mut(k).set_direction(value);

        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        // ROM $06:D3AD
        self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
        // $06:D3B0 JSR $06:D88B: both complete damage calls.
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom06D88B);
        if (self.sprite_slot_view(k).ai_state() & 1) == 0 {
            self.sprite_slot_view_mut(k).add_subtype2(1);
            let value = ((self.sprite_slot_view(k).subtype2() >> 3) & 3)
                | ((self.sprite_slot_view(k).direction() & 2) << 1);
            self.sprite_slot_view_mut(k).set_graphics(value);
            if self.sprite_slot_view(k).delay_main() == 0 {
                self.sprite_slot_view_mut(k).add_ai_state(1);
                let value = if self.sprite_slot_view(k).sprite_type() == 8 {
                    60
                } else {
                    160
                };
                self.sprite_slot_view_mut(k).set_delay_main(value);
            } else {
                j = self.sprite_slot_view(k).direction();
                let value = SPRITE_08_OCTOROK_LOCAL_X_VELOCITIES[usize::from(j)] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = SPRITE_08_OCTOROK_LOCAL_Y_VELOCITIES[usize::from(j)] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                if self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced) != 0 {
                    self.sprite_slot_view_mut(k).xor_direction(1);
                }
            }
            return;
        }

        self.sprite_zero_velocity_xy(k);
        if self.sprite_slot_view(k).delay_main() == 0 {
            self.sprite_slot_view_mut(k).add_ai_state(1);
            // ROM $86D416: `AND #$3F : ADC #$30` inherits the RNG routine's
            // final LSR carry (oracle sample 0x8D/C=1 wrote 0x3E at route
            // frame 71350).
            let value = self.get_random_number_with_carry().masked_adc(0x3f, 48);
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = self.sprite_slot_view(k).delay_main() & 3;
            self.sprite_slot_view_mut(k).set_direction(value);
        } else {
            match self.sprite_slot_view(k).sprite_type() {
                8 => {
                    j = self.sprite_slot_view(k).delay_main();
                    if j == 28 {
                        self.octorok_fire_loogie(k);
                    }
                    let value = OCTOROK_SPIT_POSE_BY_DELAY[usize::from(j >> 3)];
                    self.sprite_slot_view_mut(k).set_c(value);
                }
                10 => {
                    j = self.sprite_slot_view(k).delay_main();
                    if j < 128 {
                        if (j & 15) == 0 {
                            let value = SPRITE_08_OCTOROK_NEXT_DIR
                                [usize::from(self.sprite_slot_view(k).direction())];
                            self.sprite_slot_view_mut(k).set_direction(value);
                        }
                        if (j & 15) == 8 {
                            self.octorok_fire_loogie(k);
                        }
                    }
                    let value = FOUR_WAY_OCTOROK_SPIT_POSE_BY_DELAY[usize::from(j >> 4)];
                    self.sprite_slot_view_mut(k).set_c(value);
                }
                _ => {}
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_0C_OctorokStone(int k) {  // 86d5b9
    pub(super) fn sprite_0_c_octorok_stone(&mut self, k: usize) {
        if self.sprite_slot_view(k).state() == 6 {
            self.sprite_draw_octorok_stone_crumbling(k);
            if self.sprite_return_if_paused(k, SpritePausedEntry::Rom06D9F3) {
                return;
            }
            if self.sprite_slot_view(k).delay_main() == 30 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
            }
        } else {
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
            if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
                return;
            }
            self.sprite_check_damage_to_link(k, SpriteDamageToEntry::Unpriced);
            // ROM $06:D5DD
            self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
                && self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced) != 0
            {
                self.sprite_func3(k);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_0E_Snapdragon(int k) {  // 869c24
    pub(super) fn sprite_0_e_snapdragon(&mut self, k: usize) {
        let value = self.sprite_slot_view(k).b().wrapping_add(
            SPRITE_0_E_SNAPDRAGON_LOCAL_GRAPHICS[usize::from(self.sprite_slot_view(k).direction())],
        );

        self.sprite_slot_view_mut(k).set_graphics(value);
        self.snap_dragon_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        // $06:9C3A JSR $06:D88B: both complete damage calls.
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom06D88B);
        let value = 0;
        self.sprite_slot_view_mut(k).set_b(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = SPRITE_0_E_SNAPDRAGON_DELAY
                        [usize::from((self.get_random_number() & 12) >> 2)];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let new_a = self.sprite_slot_view(k).a().wrapping_sub(1);
                    let value = new_a;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if sign8(new_a) {
                        let value = 3;
                        self.sprite_slot_view_mut(k).set_a(value);
                        let value = 96;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        self.sprite_slot_view_mut(k).add_c(1);
                        let value =
                            self.sprite_is_below_link(k).a * 2 + self.sprite_is_right_of_link(k).a;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    } else {
                        let value = self.get_random_number() & 3;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                } else if (self.sprite_slot_view(k).delay_main() & 0x18) != 0 {
                    self.sprite_slot_view_mut(k).add_b(1);
                }
            }
            1 => {
                self.sprite_slot_view_mut(k).add_b(1);
                self.sprite_move_xy(k, SpriteMoveEntry::Unpriced);
                if self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced) != 0 {
                    self.sprite_slot_view_mut(k).xor_direction(3);
                }
                let j = usize::from(
                    self.sprite_slot_view(k).direction()
                        + if self.sprite_slot_view(k).c() != 0 {
                            4
                        } else {
                            0
                        },
                );
                let value = SPRITE_0_E_SNAPDRAGON_LOCAL_X_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = SPRITE_0_E_SNAPDRAGON_LOCAL_Y_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                self.sprite_move_z(k, SpriteMoveZEntry::Unpriced);
                self.sprite_slot_view_mut(k).subtract_z_velocity(4);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_c(value);
                        let value = 63;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let value = 20;
                        self.sprite_slot_view_mut(k).set_z_velocity(value);
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_11_Hinox(int k) {  // 869f05
    pub(super) fn sprite_11_hinox(&mut self, k: usize) {
        self.hinox_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_slot_view(k).f() != 0 {
            self.hinox_face_link(k);
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 48;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        // $06:9F20 JSR $06:D88B: both complete damage calls.
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom06D88B);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    if (self.get_random_number() & 3) == 0 {
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 64;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        self.sprite_slot_view_mut(k).add_c(1);
                        if self.sprite_slot_view(k).c() == 4 {
                            let value = 0;
                            self.sprite_slot_view_mut(k).set_c(value);
                            self.hinox_face_link(k);
                        } else {
                            let idx = usize::from(
                                self.sprite_slot_view(k).direction() * 2
                                    + (self.get_random_number() & 1),
                            );
                            self.hinox_set_direction(k, HINOX_RANDOM_DIRECTIONS[idx]);
                        }
                    }
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    let new_a = self.sprite_slot_view(k).a().wrapping_sub(1);
                    let value = new_a;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if sign8(new_a) {
                        let value = 11;
                        self.sprite_slot_view_mut(k).set_a(value);
                        let value = self.sprite_slot_view(k).subtype2().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                    }
                    self.sprite_move_xy(k, SpriteMoveEntry::Unpriced);
                    if self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced) == 0 {
                        let value = SPRITE_11_HINOX_WALK_GFX
                            [usize::from(self.sprite_slot_view(k).direction())]
                            + (self.sprite_slot_view(k).subtype2() & 1);
                        self.sprite_slot_view_mut(k).set_graphics(value);
                        return;
                    }
                }
                let value = 16;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    return;
                }
                if self.sprite_slot_view(k).delay_main() == 32 {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
                    if j >= 0 {
                        let j = j as usize;
                        self.sprite_transmute_to_bomb(j);
                        let value = 64;
                        self.sprite_slot_view_mut(j).set_delay_aux1(value);
                        let i = usize::from(self.sprite_slot_view(k).direction());
                        self.sprite_set_x(
                            j,
                            info.r0_x
                                .wrapping_add_signed(i16::from(HINOX_BOMB_X_OFFSETS[i])),
                        );
                        self.sprite_set_y(
                            j,
                            info.r2_y
                                .wrapping_add_signed(i16::from(HINOX_BOMB_Y_OFFSETS[i])),
                        );
                        let value = HINOX_BOMB_X_VELOCITIES[i] as u8;
                        self.sprite_slot_view_mut(j).set_x_velocity(value);
                        let value = HINOX_BOMB_Y_VELOCITIES[i] as u8;
                        self.sprite_slot_view_mut(j).set_y_velocity(value);
                        let value = 40;
                        self.sprite_slot_view_mut(j).set_z_velocity(value);
                    }
                } else {
                    let i = usize::from(
                        self.sprite_slot_view(k).direction()
                            + if self.sprite_slot_view(k).delay_main() < 32 {
                                4
                            } else {
                                0
                            },
                    );
                    let value = SPRITE_11_HINOX_LOCAL_GRAPHICS[i];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_4E_Popo(int k) {  // 85b80a
    pub(super) fn sprite_4_e_popo(&mut self, k: usize) {
        self.bot_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> 4) & 3;
        self.sprite_slot_view_mut(k).set_a(value);
        // $05:B822 JSR $05:AB93: both complete damage calls.
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom05AB93);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 105;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                self.sprite_slot_view_mut(k).add_subtype2(1);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    // ROM $85B84B: `AND #$3F : ADC #$80` inherits the RNG
                    // routine's final LSR carry (oracle sample 0x43/C=1 wrote
                    // 0x84 at route frame 84855).
                    let value = self.get_random_number_with_carry().masked_adc(63, 128);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let j = usize::from(self.get_random_number() & 15);
                    let value = ((i16::from(CUCCO_CALM_CIRCLE_X_VELOCITIES[j])) << 2) as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = ((i16::from(CUCCO_CALM_CIRCLE_Y_VELOCITIES[j])) << 2) as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).add_subtype2(1);
                if self.sprite_slot_view(k).delay_main() != 0 {
                    if (((k as u8) ^ self.game_state.frame.frame_counter)
                        & self.sprite_slot_view(k).b())
                        != 0
                    {
                        // ROM $05:B88D JSL $06:E496
                        let _ = self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Rom06E496);
                        return;
                    }
                    // ROM $05:B885
                    self.sprite_move_xy(k, SpriteMoveEntry::Rom05FA00);
                    if self.sprite_slot_view(k).wall_collision() == 0 {
                        // ROM $05:B88D JSL $06:E496
                        let _ = self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Rom06E496);
                        return;
                    }
                }
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 80;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_4D_Toppo(int k) {  // 85ba85
    pub(super) fn sprite_4_d_toppo(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_view_mut(k).or_object_priority(0x30);
            self.toppo_draw(k);
        }
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let j = usize::from(self.get_random_number() & 3);
                    let x = u16::from(self.sprite_slot_view(k).a())
                        | (u16::from(self.sprite_slot_view(k).b()) << 8);
                    let y = u16::from(self.sprite_slot_view(k).c())
                        | (u16::from(self.sprite_slot_view(k).head_direction()) << 8);
                    self.sprite_set_x(k, x.wrapping_add(SPRITE_4_D_TOPPO_X_OFFS[j] as i16 as u16));
                    self.sprite_set_y(k, y.wrapping_add(SPRITE_4_D_TOPPO_Y_OFFS[j] as i16 as u16));
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = (self.sprite_slot_view(k).delay_main() >> 2) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    self.toppo_verify_tile(k);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 64;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                }
                let value = 2;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.toppo_verify_tile(k);
            }
            3 => {
                // $05:BB30 JSR $05:AB93: complete two-call body.
                self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom05AB93);
                self.sprite_move_z(k, SpriteMoveZEntry::Unpriced);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.toppo_verify_tile(k);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = (self.sprite_slot_view(k).delay_main() >> 2) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
                self.toppo_verify_tile(k);
            }
            5 => self.toppo_flustered(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Toppo_VerifyTile(int k) {  // 85bb72
    pub(super) fn toppo_verify_tile(&mut self, k: usize) {
        let x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        // ROM $05:BB88 JSL $06:E87B
        if self.probe_entity_tile_long(0, x, y).0 != NativeTile::GRASS {
            let value = 5;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Toppo_Draw(int k) {  // 85bbff
    pub(super) fn toppo_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        let ybase = self
            .sprite_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        for i in (0..=2).rev() {
            let j = i + g * 3;
            let big = TOPPO_DRAW_BIG[j];
            let mut flags = TOPPO_DRAW_FLAGS[j] | info_flags;
            if big == 0 {
                flags = (flags & !0x0f) | 2;
            }
            let base_y = if big != 0 { info_y } else { ybase };
            self.set_oam_helper0_at(
                oam,
                info_x.wrapping_add(TOPPO_DRAW_X_OFFSETS[j] as i16 as u16),
                base_y.wrapping_add(TOPPO_DRAW_Y_OFFSETS[j] as i16 as u16),
                TOPPO_DRAW_CHARS[j],
                flags,
                big,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Toppo_Flustered(int k) {  // 9df3d4
    pub(super) fn toppo_flustered(&mut self, k: usize) {
        let value = 130;
        self.sprite_slot_view_mut(k).set_flags2(value);
        let value = 130;
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        let value = 73;
        self.sprite_slot_view_mut(k).set_flags3(value);
        if self.sprite_slot_view(k).subtype() == 0 {
            // $1D:F3E9 JSL $06:F121: complete body, caller opcode unchanged.
            if self.sprite_check_damage_to_link(k, SpriteDamageToEntry::Rom06F121) {
                self.dialogue_message_index_mut().set_value(0x174);
                self.sprite_show_message_minimal_c();
                let value = 1;
                self.sprite_slot_view_mut(k).set_subtype(value);
            }
        } else if self.sprite_slot_view(k).subtype() < 16 {
            self.sprite_slot_view_mut(k).add_subtype(1);
        } else if self.sprite_slot_view(k).subtype() == 16 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_flags5(value);
            let value = 6;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).add_flags2(4);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x15);
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x4d, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.force_prize_drop(j, 6, 6);
            }
            self.sprite_slot_view_mut(k).add_subtype(1);
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = ((self.sprite_slot_view(k).subtype2() & 4) >> 2) + 3;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_4C_Geldman(int k) {  // 85b8b3
    pub(super) fn sprite_4_c_geldman(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() < 2 {
            self.sprite_prep_oam_coord(k);
        } else {
            self.gerudo_man_draw(k);
        }
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        let value = 1;
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = self.sprite_slot_view(k).a();
                    self.sprite_slot_view_mut(k).set_x_low(value);
                    let value = self.sprite_slot_view(k).b();
                    self.sprite_slot_view_mut(k).set_x_high(value);
                    let value = self.sprite_slot_view(k).c();
                    self.sprite_slot_view_mut(k).set_y_low(value);
                    let value = self.sprite_slot_view(k).head_direction();
                    self.sprite_slot_view_mut(k).set_y_high(value);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0
                    && self
                        .game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                        .wrapping_add(0x30)
                        < 0x60
                    && self
                        .game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                        .wrapping_add(0x30)
                        < 0x60
                {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 31;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 96;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_apply_speed_towards_link(k, 16);
                } else {
                    let value = SPRITE_4_C_GELDMAN_EMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 2)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_4_C_GELDMAN_PURSUE_GFX
                        [usize::from((self.sprite_slot_view(k).delay_main() >> 2) & 1)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    // $05:B988 JSR $05:AB93: complete two-call body.
                    self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom05AB93);
                    self.sprite_move_xy(k, SpriteMoveEntry::Unpriced);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_4_C_GELDMAN_SUBMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 1)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_71_Leever(int k) {  // 86cba2
    pub(super) fn sprite_71_leever(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.leever_draw(k);
        } else {
            self.sprite_prep_oam_coord_from(k, super::sprite::PrepOamCoordEntry::Safe);
        }
        if self.sprite_slot_view(k).pause() != 0 {
            let value = 8;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    self.sprite_apply_speed_towards_link(k, 16);
                    self.sprite_move_xy(k, SpriteMoveEntry::Unpriced);
                    self.sprite_check_tile_collision2(k);
                }
            }
            1 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    // ROM `AND #63 : ADC #160` inherits GetRandomInt's
                    // final-LSR carry (route frame 125288).
                    let value = self.get_random_number_with_carry().masked_adc(63, 160);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_zero_velocity_xy(k);
                } else {
                    let value = SPRITE_71_LEEVER_EMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                // $06:CC3C JSR $06:D88B: complete two-call body.
                self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom06D88B);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    if (self.sprite_slot_view(k).subtype2() & 7) == 0 {
                        self.sprite_apply_speed_towards_link(
                            k,
                            SPRITE_71_LEEVER_ATTACK_SPD[usize::from(self.sprite_slot_view(k).a())],
                        );
                    }
                    self.sprite_move_xy(k, SpriteMoveEntry::Unpriced);
                    if self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced) != 0 {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 127;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let value = self.sprite_slot_view(k).subtype2().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                        let value = SPRITE_71_LEEVER_ATTACK_GFX
                            [usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 3)];
                        self.sprite_slot_view_mut(k).set_graphics(value);
                    }
                }
            }
            3 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    // ROM `AND #31 : ADC #64` inherits GetRandomInt's
                    // final-LSR carry.
                    let value = self.get_random_number_with_carry().masked_adc(31, 64);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_71_LEEVER_SUBMERGE_GFX
                        [usize::from((self.sprite_slot_view(k).delay_main() >> 3) ^ 15)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_12_Moblin(int k) {  // 8698e4
    pub(super) fn sprite_12_moblin(&mut self, k: usize) {
        let attribute_loaded = self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::MoblinAttributeLoaded { slot: k as u8 });
        let checkpoint = attribute_loaded
            || self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::MoblinCollisionGeometry { slot: k as u8 });
        if !self.moblin_before_tile_collision(k) {
            assert!(
                !checkpoint,
                "Moblin collision checkpoint did not reach its source call"
            );
            return;
        }
        if checkpoint {
            assert!(self.game_state.world.location.is_outdoors());
            assert_eq!(self.sprite_slot_view(k).flags2() & 0x20, 0);
            assert!(
                self.sprite_slot_view(k).flags4() & 0x80 != 0
                    || self.game_state.dungeon.room_load.header_collision() == 0
            );
            assert!(
                self.sprite_slot_view(k).y_velocity() != 0
                    && self.sprite_slot_view(k).y_velocity() < 0x80
            );
            self.sprite_slot_view_mut(k).set_wall_collision(0);
            if attribute_loaded {
                self.sprite_vertical_tile_attribute_loaded(k);
            }
            return;
        }
        let _ = self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced);
        self.moblin_after_tile_collision(k);
    }

    pub(super) fn moblin_before_tile_collision(&mut self, k: usize) -> bool {
        self.moblin_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return false;
        }
        if self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
            return false;
        }
        // $06:98ED JSR $06:D88B: both complete damage calls.
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom06D88B);
        // $06:98F0 JSR $06:E92C: complete body, caller opcode unchanged.
        self.sprite_move_xy(k, SpriteMoveEntry::Rom06E93E);
        true
    }

    pub(in crate::zelda_rtl) fn moblin_after_loaded_vertical_attribute(&mut self, k: usize) {
        self.sprite_vertical_collision_after_attribute_loaded(k);
        let velocity = self.sprite_slot_view(k).x_velocity();
        if velocity != 0 {
            self.sprite_check_for_tile_in_direction_horizontal(
                k,
                if sign8(velocity) { 2 } else { 3 },
            );
        }
        self.sprite_check_tile_collision_after_directions(k);
        self.moblin_after_tile_collision(k);
    }

    pub(in crate::zelda_rtl) fn moblin_after_tile_collision(&mut self, k: usize) {
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = SPRITE_12_MOBLIN_DELAY[usize::from(self.get_random_number() & 3)];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = self.sprite_slot_view(k).head_direction();
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let j = usize::from(self.sprite_slot_view(k).head_direction());
                    let value = SPRITE_12_MOBLIN_LOCAL_X_VELOCITIES[j] as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = SPRITE_12_MOBLIN_LOCAL_Y_VELOCITIES[j] as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            }
            1 => {
                let value = (self.sprite_slot_view(k).subtype2() & 1)
                    + SPRITE_12_MOBLIN_LOCAL_GRAPHICS
                        [usize::from(self.sprite_slot_view(k).direction())];
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).wall_collision() == 0 {
                    if self.sprite_slot_view(k).delay_main() != 0 {
                        let new_e = self.sprite_slot_view(k).e().wrapping_sub(1);
                        let value = new_e;
                        self.sprite_slot_view_mut(k).set_e(value);
                        if sign8(new_e) {
                            let value = 11;
                            self.sprite_slot_view_mut(k).set_e(value);
                            let value = self.sprite_slot_view(k).subtype2().wrapping_add(1);
                            self.sprite_slot_view_mut(k).set_subtype2(value);
                        }
                        return;
                    }
                    if self.sprite_slot_view(k).direction()
                        == self.sprite_direction_to_face_link(k)
                    {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 32;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        self.sprite_zero_velocity_xy(k);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_z_velocity(value);
                        return;
                    }
                    let value = 0x10;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = 0x0c;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                let dir_idx = usize::from(
                    (self.sprite_slot_view(k).direction() << 1) | (self.get_random_number() & 1),
                );
                let value = SPRITE_12_MOBLIN_DIRS[dir_idx];
                self.sprite_slot_view_mut(k).set_head_direction(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                self.sprite_slot_view_mut(k).add_c(1);
                if self.sprite_slot_view(k).c() == 4 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_c(value);
                    let value = self.sprite_direction_to_face_link(k);
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
                self.sprite_zero_velocity_xy(k);
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
            }
            2 => {
                let mut j = self.sprite_slot_view(k).direction();
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                if self.sprite_slot_view(k).delay_main() < 16 {
                    if self.sprite_slot_view(k).delay_main() == 15 {
                        self.moblin_materialize_spear(k);
                        let value = 32;
                        self.sprite_slot_view_mut(k).set_delay_aux1(value);
                    }
                    j = j.wrapping_add(4);
                }
                let value = SPRITE_12_MOBLIN_SECONDARY_GRAPHICS[usize::from(j)];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Cukeman(int k) {  // 9afa0c
    pub(super) fn sprite_cukeman(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() == 0 {
            return;
        }

        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        if self.sprite_slot_view(k).state() == 9
            && (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0
            && cur_x.wrapping_sub(link_x).wrapping_add(0x18) < 0x30
            && link_y.wrapping_sub(cur_y).wrapping_add(0x20) < 0x30
            && (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) != 0
        {
            let subtype = self.sprite_slot_view(k).subtype();
            self.dialogue_message_index_mut()
                .set_value(0x017a + u16::from(subtype & 1));
            let value = subtype.wrapping_add(1);
            self.sprite_slot_view_mut(k).set_subtype(value);
            self.sprite_show_message_minimal_c();
        }

        let old = self.sprite_slot_view(k).oam_flags() & 0xf0;
        let value = old | 8;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.cukeman_draw(k);
        let value = old | 0x0d;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.oam_allocate_from_region_a(0x10);
    }

    // -----------------------------------------------------------------------
    // void Octoballoon_Draw(int k) {  // 86d784
    pub(super) fn octoballoon_draw(&mut self, k: usize) {
        let mut d = 0usize;
        if self.sprite_slot_view(k).state() == 6 {
            if self.sprite_slot_view(k).delay_main() == 6 && self.game_state.frame.submodule == 0 {
                self.octoballoon_form_babby(k);
            }
            d = usize::from((self.sprite_slot_view(k).delay_main() >> 1) & 4) + 4;
        }
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = PrepOamCoordsRet {
            x: info_x,
            y: info_y,
            r4: 0,
            flags: info_flags,
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..4).rev() {
            let j = d + i;
            self.set_oam_helper0_at(
                oam,
                info_x.wrapping_add(OCTOBALLOON_DRAW_X_OFFSETS[j] as i16 as u16),
                info_y.wrapping_add(OCTOBALLOON_DRAW_Y_OFFSETS[j] as i16 as u16),
                OCTOBALLOON_DRAW_CHARS[j],
                OCTOBALLOON_DRAW_FL[j] | info_flags,
                2,
            );
            oam += 4;
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Unpriced);
    }

    // -----------------------------------------------------------------------
    // void Octoballoon_RecoilLink(int k) {  // 86d72b
    pub(super) fn octoballoon_recoil_link(&mut self, k: usize) {
        if self.game_state.player.follower_link.incapacitated_timer() == 0 {
            self.follower_link_state_mut().set_incapacitated_timer(4);
            self.sprite_apply_recoil_to_link(k, 16);
            self.sprite_invert_speed_xy(k);
        }
    }

    pub(in crate::zelda_rtl) fn pikit_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_pikit_tongue(k, &info);
        let oam = self.game_state.oam.current_pointer_usize();
        let oam_byte = self.game_state.oam.entry_x(oam);
        self.temp_counter_mut().set(oam_byte);
        let shared_scratch_a = self.game_state.oam.entry_y(oam);
        self.sprite_workspace_mut()
            .set_shared_scratch_a(shared_scratch_a);
        self.oam_state_mut()
            .set_current_pointer((oam as u16).wrapping_add(24));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(6));
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut shadow_info = self.sprite_draw_multiple(k, &PIKIT_DRAW_FRAMES[base..base + 2]);
        let bak = self.sprite_slot_view(k).flags2();
        self.sprite_slot_view_mut(k).subtract_flags2(6);
        // ROM $0D:D72C
        self.sprite_draw_shadow_custom(k, &mut shadow_info, 10, SpriteShadowEntry::Rom06DC54);
        let value = bak;
        self.sprite_slot_view_mut(k).set_flags2(value);
        self.sprite_draw_pikit_loot(k, &info);
    }

    pub(super) fn moblin_draw(&mut self, k: usize) {
        let gfx = usize::from(self.sprite_slot_view(k).graphics());
        let mut info = self.sprite_draw_multiple(
            k,
            &MOBLIN_DRAW_FRAMES[gfx * 4..gfx * 4 + 4],
        );
        if self.sprite_slot_view(k).pause() != 0 {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            for _ in 0..4 {
                let ext_index = (oam - OAM_BUF) / 4;
                if self.game_state.oam.extended_byte(ext_index) & 2 == 0 {
                    self.oam_state_mut().hide_entry(oam);
                }
                oam += 4;
            }
        }
        let oam = self.game_state.oam.current_pointer_usize()
            + usize::from(MOBLIN_DRAW_OBJ_OFFS[gfx.min(MOBLIN_DRAW_OBJ_OFFS.len() - 1)]) * 4;
        let j = usize::from(self.sprite_slot_view(k).head_direction() & 3);
        self.oam_state_mut()
            .set_entry_char(oam, MOBLIN_DRAW_HEAD_CHAR[j]);
        self.oam_state_mut()
            .merge_entry_flags(oam, !0x40, ALTAR_ZELDA_HEAD_FLAGS[j]);
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Unpriced);
    }

    pub(in crate::zelda_rtl) fn tektite_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple(k, &TEKTITE_DRAW_FRAMES[base..base + 2]);
        // ROM $1D:C40D
        self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
    }
}
