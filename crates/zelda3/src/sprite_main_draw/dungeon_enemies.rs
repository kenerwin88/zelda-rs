//! Dungeon enemies, undead, beetles, rats, ropes, and their draw helpers.

use super::*;
use crate::zelda_rtl::sprite::PrepOamCoordsRet;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;
use crate::tile_definition::NativeTile;
use crate::types::sign8;

impl ZeldaState {

    // -----------------------------------------------------------------------
    // void SpriteDraw_StalfosKnight_Head(int k, PrepOamCoordsRet *info) {  // 9eae4e
    //   sprite_main.c:21654.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_stalfos_knight_head(&mut self, k: usize, info: &PrepOamCoordsRet) {
        if self.sprite_slot_view(k).graphics() == 2 {
            return;
        }
        let i = (self.sprite_slot_view(k).head_direction() & 3) as usize;
        let oam = self.game_state.oam.current_pointer_usize();
        let y = info
            .y
            .wrapping_add(self.sprite_slot_view(k).c() as u16)
            .wrapping_sub(12);
        self.set_oam_helper0_at(
            oam,
            info.x,
            y,
            STALFOS_KNIGHT_HEAD_CHARS[i],
            info.flags | STALFOS_KNIGHT_HEAD_FLAGS[i],
            2,
        );
    }

    // -----------------------------------------------------------------------
    // void Poe_Draw(int k) {  // 869786
    //   Single small flame body tile.
    // }
    pub(super) fn poe_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let d = usize::from(self.sprite_slot_view(k).direction() & 1);
        let ch = POE_DRAW_CHARS[((self.sprite_slot_view(k).subtype2() >> 3) & 3) as usize];
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_helper0_at(
            oam,
            x.wrapping_add(POE_DRAW_X_OFFSETS[d] as i16 as u16),
            y.wrapping_add(9),
            ch,
            (flags & 0xf0) | 2,
            0,
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_23_RedBari(int k) {  // 86a23d
    //   Red Bari movement, electrocuted phase, and split behavior.
    // }
    pub(super) fn sprite_23_red_bari(&mut self, k: usize) {
        if sign8(self.sprite_slot_view(k).c()) {
            if self.sprite_slot_view(k).head_direction() != 16 {
                self.sprite_slot_view_mut(k).add_head_direction(1);
            } else {
                let value = 255;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 255;
                self.sprite_slot_view_mut(k).set_subtype(value);
                self.sprite_check_tile_collision2(k);
                let value = 0;
                self.sprite_slot_view_mut(k).set_subtype(value);
                let tile = self.game_state.sprites.workspace.tile();
                if tile == NativeTile::GROUND {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_c(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                    self.red_bari_set_electrocute_delay(k);
                } else {
                    // The original parks the probed identity in this slot byte.
                    self.sprite_slot_view_mut(k)
                        .set_ignore_projectile(tile.cartridge_attribute());
                }
            }
            return;
        }
        if self.sprite_slot_view(k).c() != 0 {
            self.sprite_draw_single_small(k);
        } else if self.sprite_slot_view(k).graphics() >= 2 {
            self.sprite_draw_single_large(k);
        } else {
            self.red_bari_draw(k);
        }

        self.sprite_23_red_bari_after_draw(k);
    }

    pub(in crate::zelda_rtl) fn sprite_23_red_bari_after_draw(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        let recoil_from_split = self.sprite_slot_view(k).delay_aux2() != 0;

        if self.sprite_slot_view(k).ai_state() == 2 {
            let value = self.sprite_slot_view(k).ai_state();
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            let value = SPRITE_23_RED_BARI_BARI_IDLE_X_VELOCITIES
                [usize::from((self.game_state.frame.frame_counter >> 1) & 1)];
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            self.sprite_move_x(k);
            if self.sprite_slot_view(k).delay_main() == 0 {
                self.red_bari_split(k);
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            return;
        }

        if !recoil_from_split {
            self.sprite_check_damage_to_and_from_link(k);
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
                let value = self.sprite_slot_view(k).a().wrapping_add(
                    if (self.sprite_slot_view(k).b() & 1) != 0 {
                        0xff
                    } else {
                        1
                    },
                );
                self.sprite_slot_view_mut(k).set_a(value);
                if (self.get_random_number() & 3) == 0 {
                    self.sprite_slot_view_mut(k).add_b(1);
                }
            }
            let j = usize::from(self.sprite_slot_view(k).a() & 15);
            let value = BARI_BURST_X_VELOCITIES[j];
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = BARI_BURST_Y_VELOCITIES[j];
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }

        if recoil_from_split
            || ((((k as u8) ^ self.game_state.frame.frame_counter) & 3)
                | self.sprite_slot_view(k).delay_main())
                == 0
        {
            if self.sprite_slot_view(k).wall_collision() == 0 {
                self.sprite_move_xy(k);
            }
            self.sprite_check_tile_collision2(k);
        }
        let j = usize::from(self.sprite_slot_view(k).c());
        let value = ((self.game_state.frame.frame_counter >> 3) & 1) + BARI_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).delay_main() != 0 {
                let value = ((self.game_state.frame.frame_counter >> 1) & 2) + BARI_GRAPHICS[j];
                self.sprite_slot_view_mut(k).set_graphics(value);
                return;
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        } else if self.sprite_slot_view(k).delay_aux1() != 0 {
            return;
        } else if (self.get_random_number() & 1) == 0 {
            let value = 128;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).add_ai_state(1);
            return;
        }
        self.red_bari_set_electrocute_delay(k);
    }

    fn red_bari_set_electrocute_delay(&mut self, k: usize) {
        // ROM Sprite_23_RedBari $06:A342: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
        let value = self.get_random_number_with_carry().masked_adc(63, 128);
        self.sprite_slot_view_mut(k).set_delay_aux1(value);
    }

    // -----------------------------------------------------------------------
    // void RedBari_Draw(int k) {  // 86a3dc
    //   Four 8x8 body tiles selected by sprite_graphics.
    // }
    pub(super) fn red_bari_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize).min(1) * 4;
        let mut info = self.sprite_draw_multiple(k, &RED_BARI_DRAW_FRAMES[base..base + 4]);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void HardHatBeetle_Draw(int k) {  // 86a4f2
    //   Two large body tiles, with a shadow only for shadow-enabled sprites.
    // }
    pub(super) fn hard_hat_beetle_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize).min(1) * 2;
        let mut info = self.sprite_draw_multiple(
            k,
            &HARD_HAT_BEETLE_DRAW_FRAMES[base..base + 2],
        );
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
    }

    // -----------------------------------------------------------------------
    // void Armos_Draw(int k) {  // 85b7ef
    //   Two large stacked tiles plus a common shadow.
    // }
    pub(super) fn armos_draw(&mut self, k: usize) {
        let mut info = self.sprite_draw_multiple(k, &ARMOS_DRAW_FRAMES);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Freezor_Draw(int k) {  // 8dbfa6
    //   Four/eight-tile ice body selected by sprite_graphics.
    // }
    pub(in crate::zelda_rtl) fn freezor_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).graphics() != 7 {
            let base = (self.sprite_slot_view(k).graphics() as usize) * 4;
            self.sprite_draw_multiple(k, &FREEZOR_DRAW_FRAMES0[base..base + 4]);
        } else {
            self.sprite_draw_multiple(k, &FREEZOR_DRAW_FRAMES1);
        }
    }

    // -----------------------------------------------------------------------
    // void Ropa_Draw(int k) {  // 869ee5
    //   Three body tiles plus common shadow.
    // }
    pub(super) fn ropa_draw(&mut self, k: usize) {
        let base =
            ((self.sprite_slot_view(k).graphics() as usize) * 3).min(ROPA_DRAW_FRAMES.len() - 3);
        let mut info = self.sprite_draw_multiple(k, &ROPA_DRAW_FRAMES[base..base + 3]);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void WallMaster_Draw(int k) {  // 9eafe4
    pub(in crate::zelda_rtl) fn wall_master_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple(k, &WALL_MASTER_DRAW_FRAMES[base..base + 4]);
        self.sprite_draw_large_shadow2(k);
    }

    // -----------------------------------------------------------------------
    // void Zol_Draw(int k) {  // 9eb1c5
    pub(in crate::zelda_rtl) fn zol_draw(&mut self, k: usize) {
        if (self.sprite_slot_view(k).oam_flags() & 1) == 0
            && self.game_state.sprites.system.chr_halfslot_state() >= 3
        {
            return;
        }
        if self.sprite_slot_view(k).delay_aux4() != 0 {
            self.oam_allocate_from_region_b(8);
        }
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_prep_oam_coord(k);
            return;
        }
        let gfx = self.sprite_slot_view(k).graphics();
        if gfx < 4 {
            let bak1 = self.sprite_slot_view(k).oam_flags();
            let value = bak1 ^ ZOL_DRAW_OAM_FLAGS[usize::from(gfx)];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = gfx.wrapping_add(((self.sprite_slot_view(k).oam_flags() & 1) ^ 1) << 2);
            self.sprite_slot_view_mut(k).set_graphics(value);
            // ROM $1E:B202 JSL $06:DBF0
            self.sprite_draw_single_large_long(k);
            let value = gfx;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let value = bak1;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        } else {
            let base = usize::from(gfx.wrapping_sub(4)) * 2;
            self.sprite_draw_multiple(k, &ZOL_DRAW_FRAMES[base..base + 2]);
        }
    }

    // -----------------------------------------------------------------------
    // void Zoro(int k) {  // 9e9bd0
    pub(in crate::zelda_rtl) fn zoro(&mut self, k: usize) {
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        // $1E:9BD0..9BDD: C gate, optional INC/JSR/CPY/BEQ.
        blocks.block(0x1e9bd0);
        if self.sprite_slot_view(k).c() == 0 {
            blocks.block(0x1e9bd5);
            blocks.block(0x1e9bdb);
            self.sprite_slot_view_mut(k).add_c(1);
            if self.sprite_is_below_link(k).a != 0 {
                blocks.block(0x1e9bdf); // $1E:9BDF..9BE2 STZ/RTS
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                return;
            }
            blocks.taken(0x1e9bdd);
        } else {
            blocks.taken(0x1e9bd3);
        }
        blocks.block(0x1e9be3);
        blocks.block(0x1e9be7); // $1E:9BE3..9BE7 JSL draw, JSR inactive
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // $1E:9BEA..9C0D: damage JSR, animation/velocity, move JSR,
        // delay gate. Child bodies retain their existing scopes.
        blocks.block(0x1e9bea);
        blocks.block(0x1e9bed);
        blocks.block(0x1e9c0a);
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> 1) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value =
            ZORO_X_VELOCITIES[usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 1)] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        self.sprite_move_xy_bank(k, 0x1eff34);
        if self.sprite_slot_view(k).delay_main() == 0 {
            blocks.block(0x1e9c0f);
            blocks.block(0x1e9c12); // $1E:9C0F..9C12 JSR collision, BEQ
            // ROM $1E:9C0F -> $1E:8B2E -> $06:E496
            if self.sprite_check_tile_collision_long(k) != 0 {
                blocks.block(0x1e9c14); // $1E:9C14 STZ
                self.sprite_slot_view_mut(k).set_state(0);
            } else {
                blocks.taken(0x1e9c12);
            }
        } else {
            blocks.taken(0x1e9c0d);
        }
        blocks.block(0x1e9c17); // $1E:9C17..9C1C subtype test
        if (self.sprite_slot_view(k).subtype2() & 3) != 0 {
            blocks.taken(0x1e9c1c);
            blocks.block(0x1e9c2c); // $1E:9C2C RTS
            return;
        }
        let j = self.garnish_alloc();
        // $1E:9C1E..9C29: setup, then the actual descending search.
        // Native garnish_alloc supplies the number of rejected source slots.
        blocks.block(0x1e9c1e);
        for _ in 0..if j < 0 { 29 } else { 29 - j } {
            blocks.block(0x1e9c22);
            blocks.block(0x1e9c28);
            blocks.taken(0x1e9c29);
        }
        if j < 0 {
            blocks.block(0x1e9c22);
            blocks.block(0x1e9c28);
            blocks.block(0x1e9c2b);
            blocks.block(0x1e9c2c); // last BPL falls through, PLX/RTS
            return;
        }
        // $1E:9C22/9C26 free-slot branch; $9C2D..9C6A trail writes.
        // All abs,Y reads use the original sprite slot (0..15): no crossing.
        blocks.block(0x1e9c22);
        blocks.taken(0x1e9c26);
        blocks.block(0x1e9c2d);
        let j = j as usize;
        let value = 6;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(6);
        self.garnish_set_x(j, self.sprite_get_x(k));
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(16));
        let value = 10;
        self.garnish_slot_view_mut(j).set_countdown(value);
        let value = k as u8;
        self.garnish_slot_view_mut(j).set_sprite(value);
        let value = self.sprite_slot_view(k).floor();
        self.garnish_slot_view_mut(j).set_floor(value);
    }

    // -----------------------------------------------------------------------
    // void Wizzrobe_Draw(int k) {  // 8dbe06
    pub(in crate::zelda_rtl) fn wizzrobe_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        self.sprite_draw_multiple(k, &WIZZROBE_DRAW_FRAMES[base..base + 3]);
    }

    // -----------------------------------------------------------------------
    // void Wizzrobe_FireBeam(int k) {  // 9e9e15
    pub(in crate::zelda_rtl) fn wizzrobe_fire_beam(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x9b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
            let value = 1;
            self.sprite_slot_view_mut(j).set_c(value);
            let value = 1;
            self.sprite_slot_view_mut(j).set_ignore_projectile(value);
            self.sprite_set_x(j, info.r0_x.wrapping_add(4));
            self.sprite_set_y(j, info.r2_y);
            let i = usize::from(self.sprite_slot_view(k).direction());
            let value = WIZZROBE_BEAM_XY_VELOCITIES[i] as u8;
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            let value = WIZZROBE_BEAM_XY_VELOCITIES[i + 2] as u8;
            self.sprite_slot_view_mut(j).set_y_velocity(value);
            let value = 0x48;
            self.sprite_slot_view_mut(j).set_deflection_bits(value);
            let value = 2;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let value = if self.game_state.inventory.items.shield_type() == 3 {
                0x20
            } else {
                0
            };
            self.sprite_slot_view_mut(j).set_flags5(value);
            let value = 0x14;
            self.sprite_slot_view_mut(j).set_flags4(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_StalfosBone(int k) {  // 9e8fdf
    pub(in crate::zelda_rtl) fn sprite_stalfos_bone(&mut self, k: usize) {
        self.stalfos_bone_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // $1E:8FE5 JSR $C26B (far damage wrapper).
        self.sprite_check_damage_to_link_bank1e(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.sprite_move_xy_bank(k, 0x1eff34);
        // ROM $1E:8FF3 -> $1E:8B2E -> $06:E496
        if self.sprite_slot_view(k).delay_main() == 0 && self.sprite_check_tile_collision_long(k) != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_place_weapon_tink(k);
        }
    }

    // -----------------------------------------------------------------------
    // void StalfosBone_Draw(int k) {  // 9e9040
    pub(super) fn stalfos_bone_draw(&mut self, k: usize) {
        let base = usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 3) * 2;
        self.sprite_draw_multiple(k, &STALFOS_BONE_DRAW_FRAMES[base..base + 2]);
    }

    // -----------------------------------------------------------------------
    // void Stalfos_ThrowBone(int k) {  // 9e9379
    pub(in crate::zelda_rtl) fn stalfos_throw_bone(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xa7, &mut info);
        if j >= 0 {
            let j = j as usize;
            let value = 1;
            self.sprite_slot_view_mut(j).set_a(value);
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_apply_speed_towards_link(j, 32);
            let value = 0x21;
            self.sprite_slot_view_mut(j).set_flags2(value);
            let value = 33;
            self.sprite_slot_view_mut(j).set_ignore_projectile(value);
            self.sprite_slot_view_mut(j).or_flags3(0x40);
            let value = 0x48;
            self.sprite_slot_view_mut(j).set_deflection_bits(value);
            let value = 16;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 0x14;
            self.sprite_slot_view_mut(j).set_flags4(value);
            let value = 7;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let value = 32;
            self.sprite_slot_view_mut(j).set_bump_damage(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x02);
        }
    }

    // -----------------------------------------------------------------------
    // void Stalfos_Draw(int k) {  // 8dc21c
    pub(in crate::zelda_rtl) fn stalfos_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux2() != 0 {
            self.sprite_prep_oam_coord(k);
            return;
        }
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        let mut info = self.sprite_draw_multiple(k, &STALFOS_DRAW_FRAMES[base..base + 3]);
        if self.sprite_slot_view(k).graphics() < 8 && self.sprite_slot_view(k).pause() == 0 {
            let oam = self.game_state.oam.current_pointer_usize();
            let i = usize::from(self.sprite_slot_view(k).head_direction());
            self.oam_state_mut()
                .set_entry_char(oam, STALFOS_DRAW_CHARS[i]);
            self.oam_state_mut()
                .merge_entry_flags(oam, !0x70, STALFOS_DRAW_FLAGS[i]);
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void StalfosKnight_Draw(int k) {  // 9eae04
    pub(in crate::zelda_rtl) fn stalfos_knight_draw(&mut self, k: usize) {
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1eDoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        self.sprite_draw_stalfos_knight_head(k, &info);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 5;
        self.sprite_draw_multiple_with_info(k, &STALFOS_KNIGHT_DRAW_FRAMES[base..base + 5], tuple);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_sub(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_sub(1));
        let mut shadow_info = PrepOamCoordsRet {
            x: info.x,
            y: info.y,
            r4: info.r4,
            flags: info.flags,
        };
        self.sprite_draw_shadow_custom(k, &mut shadow_info, 18);
    }

    // -----------------------------------------------------------------------
    // void Sprite_D3_Stal(int k) {  // 9d8129
    pub(super) fn sprite_d3_stal(&mut self, k: usize) {
        if self.game_state.sprites.system.chr_halfslot_state() < 3 {
            if self.sprite_slot_view(k).ai_state() == 0 {
                self.oam_allocate_from_region_b(4);
            }
            self.stal_draw(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling_bank(k, 0x1de95d) {
            return;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 1;
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.sprite_nullify_hookshot_drag();
                    self.sprite_repel_dash();
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        let value = 64;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
                    }
                }
                if self.sprite_slot_view(k).delay_main() != 0 {
                    let delay = self.sprite_slot_view(k).delay_main().wrapping_sub(1);
                    if delay != 0 {
                        let value = delay | 64;
                        self.sprite_slot_view_mut(k).set_hit_timer(value);
                    } else {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_hit_timer(value);
                        self.sprite_slot_view_mut(k).and_flags3(!0x40);
                        self.sprite_slot_view_mut(k).and_flags2(!0x80);
                    }
                }
            }
            1 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_xyz_bank(k, 0x1de95d);
                self.sprite_check_tile_collision(k);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    self.sprite_apply_speed_towards_link(k, 12);
                }
                if (self.game_state.frame.frame_counter & 3) == 0 {
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    if self.sprite_slot_view(k).subtype2() == 5 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                    }
                }
                let value = SPRITE_D3_STAL_STAL_GFX[self.sprite_slot_view(k).subtype2() as usize];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Stal_Draw(int k) {  // 9d820c
    pub(super) fn stal_draw(&mut self, k: usize) {
        let n = if self.sprite_slot_view(k).ai_state() != 0 {
            2
        } else {
            1
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple(k, &STAL_DRAW_FRAMES[base..base + n]);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
    }

    // -----------------------------------------------------------------------
    // void Gibo_Draw(int k) {  // 9dcf5e
    //   Optional single-large nucleus plus four segment tiles.
    // }
    pub(in crate::zelda_rtl) fn gibo_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() == 0 {
            let bak0 = self.sprite_slot_view(k).flags2();
            let value = 1;
            self.sprite_slot_view_mut(k).set_flags2(value);
            let bak1 = self.sprite_slot_view(k).oam_flags();
            let value = GIBO_OAM_FLAGS[((self.sprite_slot_view(k).anim_clock() >> 2) & 3) as usize]
                | GIBO_ALT_OAM_FLAGS[((self.sprite_slot_view(k).delay_aux1() >> 2) & 1) as usize];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            // ROM $1D:CF8C JSL $06:DBF0
            self.sprite_draw_single_large_long(k);
            let value = bak1;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = bak0;
            self.sprite_slot_view_mut(k).set_flags2(value);
        }
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));
        let base = (usize::from(self.sprite_slot_view(k).subtype2())
            + usize::from(self.sprite_slot_view(k).direction()))
            * 4;
        self.sprite_draw_multiple(k, &GIBO_DRAW_FRAMES[base..base + 4]);
    }

    // -----------------------------------------------------------------------
    // void LaserEye_Draw(int k) {  // 9ea708
    //   Three tiles selected by direction and active beam blink state.
    // }
    /// `LaserEye_Draw` before its `Sprite_DrawMultiple`: the graphics and
    /// object-priority stores.
    pub(in crate::zelda_rtl) fn laser_eye_draw_prologue(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() != 0 {
            let value = u8::from(self.sprite_slot_view(k).delay_aux4() == 0);
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        let value = 0x30;
        self.sprite_slot_view_mut(k).set_object_priority(value);
    }

    /// `LaserEye_Draw`'s `Sprite_DrawMultiple`.
    pub(in crate::zelda_rtl) fn laser_eye_draw_multiple(&mut self, k: usize) {
        let base = (usize::from(self.sprite_slot_view(k).graphics())
            + usize::from(self.sprite_slot_view(k).direction()) * 2)
            * 3;
        self.sprite_draw_multiple(k, &LASER_EYE_DRAW_FRAMES[base..base + 3]);
    }

    // -----------------------------------------------------------------------
    // void Gibdo_Draw(int k) {  // 9ebb20
    //   Sprite_DrawMultiple(k, &kGibdo_Dmd[sprite_graphics[k] * 2], 2, &info);
    //   if (!sprite_pause[k]) SpriteDraw_Shadow(k, &info);
    // }
    pub(in crate::zelda_rtl) fn gibdo_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple(k, &GIBDO_DRAW_FRAMES[base..base + 2]);
        if self.sprite_slot_view(k).pause() == 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
    }

    // -----------------------------------------------------------------------
    // void Babusu_Draw(int k) {  // 8dbd20
    pub(in crate::zelda_rtl) fn babusu_draw(&mut self, k: usize) {
        let g = self.sprite_slot_view(k).graphics();
        if g != 0xff {
            let base = usize::from(g) * 2;
            self.sprite_draw_multiple(k, &BABUSU_DRAW_FRAMES[base..base + 2]);
        } else {
            self.sprite_prep_oam_coord(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Goriya_Draw(int k) {  // 9df589
    pub(in crate::zelda_rtl) fn goriya_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 && self.sprite_slot_view(k).direction() != 3 {
            let d = usize::from(self.sprite_slot_view(k).direction());
            self.sprite_draw_multiple(k, &GORIYA_DRAW_FRAMES_2[d..d + 1]);
        }

        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        let g = usize::from(self.sprite_slot_view(k).graphics());
        let mut info = self.sprite_draw_multiple(
            k,
            &GORIYA_DRAW_FRAMES[GORIYA_DRAW_OFFSETS[g]..GORIYA_DRAW_OFFSETS[g + 1]],
        );
        self.sprite_slot_view_mut(k).subtract_flags2(1);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
        self.sprite_slot_view_mut(k).add_flags2(1);
    }

    // -----------------------------------------------------------------------
    // void Sprite_63_DebirandoPit(int k) {  // 858531
    pub(super) fn sprite_63_debirando_pit(&mut self, k: usize) {
        let (_, pt) = self.sprite_direction_and_offset_to_face_link(k);
        if pt.y.wrapping_add(0x20) < 0x40 && pt.x.wrapping_add(0x20) < 0x40 {
            self.oam_allocate_from_region_b(16);
        }

        self.debirando_pit_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let j = usize::from(self.sprite_slot_view(k).head_direction());
        if self.sprite_slot_view(j).state() == 6 {
            let value = self.sprite_slot_view(j).state();
            self.sprite_slot_view_mut(k).set_state(value);
            let value = self.sprite_slot_view(j).delay_main();
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).add_flags2(4);
            return;
        }
        if self.sprite_slot_view(k).graphics() < 3 && self.sprite_check_damage_to_link_same_layer(k)
        {
            self.link_cancel_dash();
            if (self.game_state.player.follower_link.filtered_joypad_l() & 16) == 0 {
                self.follower_link_state_mut().prevent_movement();
            }
            self.sprite_apply_speed_towards_link(k, 16);
            let v = self.sprite_slot_view(k).y_velocity();
            let mag = if sign8(v) { v.wrapping_neg() } else { v };
            let t = u16::from(mag) + u16::from(self.sprite_slot_view(k).a());
            let value = t as u8;
            self.sprite_slot_view_mut(k).set_a(value);
            if t >= 256 {
                self.follower_link_state_mut()
                    .set_drag_player_y(if sign8(v) { 1 } else { (-1i16) as u16 });
            }

            let v = self.sprite_slot_view(k).x_velocity();
            let mag = if sign8(v) { v.wrapping_neg() } else { v };
            let t = u16::from(mag) + u16::from(self.sprite_slot_view(k).b());
            let value = t as u8;
            self.sprite_slot_view_mut(k).set_b(value);
            if t >= 256 {
                self.follower_link_state_mut()
                    .set_drag_player_x(if sign8(v) { 1 } else { (-1i16) as u16 });
            }
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 6;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 63;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_63_DEBIRANDO_PIT_OPENING_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                if (self.game_state.frame.frame_counter & 15) == 0 {
                    self.sprite_slot_view_mut(k).add_graphics(1);
                    if self.sprite_slot_view(k).graphics() >= 3 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_graphics(value);
                    }
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 63;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_63_DEBIRANDO_PIT_CLOSING_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_64_Debirando(int k) {  // 85874d
    pub(super) fn sprite_64_debirando(&mut self, k: usize) {
        self.debirando_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 31;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 128;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_64_DEBIRANDO_EMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 31;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                } else {
                    if (self.sprite_slot_view(k).delay_main() & 31)
                        | self.sprite_slot_view(k).g()
                        | self.game_state.frame.submodule
                        | self.sprite_slot_view(k).pause()
                        | self.game_state.frame.modal_pause_flag
                        == 0
                    {
                        let _ = self.sprite_spawn_fireball(k);
                    }
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    let value = ((self.sprite_slot_view(k).subtype2() >> 3) & 1) + 2;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 223;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_64_DEBIRANDO_SUBMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void DebirandoPit_Draw(int k) {  // 8586e4
    pub(super) fn debirando_pit_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let g = usize::from(self.sprite_slot_view(k).graphics());
        if g == 6 {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let big = DEBIRANDO_PIT_DRAW_BIG[g];
        for i in (0..4).rev() {
            let j = g * 4 + i;
            self.set_oam_helper0_at(
                oam,
                info_x.wrapping_add(DEBIRANDO_PIT_DRAW_X_OFFSETS[j] as u16),
                info_y.wrapping_add(DEBIRANDO_PIT_DRAW_Y_OFFSETS[j] as u16),
                DEBIRANDO_PIT_DRAW_CHARS[j],
                DEBIRANDO_PIT_DRAW_FL[j] | info_flags,
                big,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Debirando_Draw(int k) {  // 858857
    pub(super) fn debirando_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            return;
        }

        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        for i in (0..4).rev() {
            let j = d + i;
            let f = DEBIRANDO_DRAW_FL[j];
            let flags = (f ^ info_flags) & if (f & 0x0f) == 0 { 0xf0 } else { 0xff };
            self.set_oam_helper0_at(
                oam,
                info_x.wrapping_add(DEBIRANDO_DRAW_X_OFFSETS[j] as i16 as u16),
                info_y.wrapping_add(DEBIRANDO_DRAW_Y_OFFSETS[j] as i16 as u16),
                DEBIRANDO_DRAW_CHARS[j],
                flags,
                DEBIRANDO_DRAW_BIG[j],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_02_StalfosHead(int k) {  // 86ddb7
    pub(super) fn sprite_02_stalfos_head(&mut self, k: usize) {
        let value = self.game_state.player.follower_link.lower_level_state();

        self.sprite_slot_view_mut(k).set_floor(value);
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            self.oam_allocate_from_region_c(8);
        }
        let j = usize::from((self.sprite_slot_view(k).subtype2() >> 3) & 3);
        let value =
            (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_02_STALFOS_HEAD_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = SPRITE_02_STALFOS_HEAD_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = 0x30;
        self.sprite_slot_view_mut(k).set_object_priority(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // $06:DDEC JSR $FF78: bank-06 recoil (not yet marker-priced).
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if self.sprite_slot_view(k).f() != 0 {
            self.sprite_zero_velocity_xy(k);
        }
        // ROM $06:DDFA
        self.sprite_move_xy_marked(k, 0x06e93e);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let (pt_x, pt_y) = if self.sprite_slot_view(k).delay_main() != 0 {
            if (self.sprite_slot_view(k).delay_main() & 1) != 0 {
                return;
            }
            let pt = self.sprite_project_speed_towards_link(k, 16);
            (pt.x, pt.y)
        } else {
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) != 0 {
                return;
            }
            let pt = self.sprite_project_speed_towards_link(k, 16);
            (pt.x.wrapping_neg(), pt.y.wrapping_neg())
        };
        let dx = self.sprite_slot_view(k).x_velocity().wrapping_sub(pt_x);
        if dx != 0 {
            let value = self
                .sprite_slot_view(k)
                .x_velocity()
                .wrapping_add(if sign8(dx) { 1 } else { 0xff });
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }
        let dy = self.sprite_slot_view(k).y_velocity().wrapping_sub(pt_y);
        if dy != 0 {
            let value = self
                .sprite_slot_view(k)
                .y_velocity()
                .wrapping_add(if sign8(dy) { 1 } else { 0xff });
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_18_MiniMoldorm(int k) {  // 869808
    pub(super) fn sprite_18_mini_moldorm(&mut self, k: usize) {
        self.moldorm_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).f() != 0 {
            self.sprite_prep_mini_moldorm_bounce(k);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let j = usize::from(self.sprite_slot_view(k).direction());
        let value = SPRITE_18_MINI_MOLDORM_LOCAL_X_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SPRITE_18_MINI_MOLDORM_LOCAL_Y_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        self.sprite_move_xy(k);
        if self.sprite_check_tile_collision(k) != 0 {
            if (self.get_random_number() & 1) != 0 {
                let value = self.sprite_slot_view(k).head_direction().wrapping_neg();
                self.sprite_slot_view_mut(k).set_head_direction(value);
            }
            let value =
                SPRITE_18_MINI_MOLDORM_NEXT_DIR[usize::from(self.sprite_slot_view(k).direction())];
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::MiniMoldormAiPending { slot: k as u8 })
        {
            return;
        }
        self.mini_moldorm_ai_after_collision(k);
    }

    pub(in crate::zelda_rtl) fn mini_moldorm_ai_after_collision(&mut self, k: usize) {
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_g(1);
                    if self.sprite_slot_view(k).g() == 6 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_g(value);
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    } else {
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                    let value = (self.get_random_number() & 2).wrapping_sub(1);
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    // ROM Sprite_18_MiniMoldorm $06:9881: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(0x1f, 0x20);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    // ROM Sprite_18_MiniMoldorm $06:9892: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(15, 8);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else if (self.sprite_slot_view(k).delay_main() & 3) == 0 {
                    let value = self
                        .sprite_slot_view(k)
                        .direction()
                        .wrapping_add(self.sprite_slot_view(k).head_direction())
                        & 0x0f;
                    self.sprite_slot_view_mut(k).set_direction(value);
                }
            }
            2 => {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
                    self.sprite_apply_speed_towards_link(k, 31);
                    let d = Self::sprite_convert_velocity_to_angle(
                        self.sprite_slot_view(k).x_velocity(),
                        self.sprite_slot_view(k).y_velocity(),
                    )
                    .wrapping_sub(self.sprite_slot_view(k).direction());
                    if d == 0 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 48;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let value = self
                            .sprite_slot_view(k)
                            .direction()
                            .wrapping_add(if sign8(d) { 0xff } else { 1 })
                            & 0x0f;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_13_MiniHelmasaur(int k) {  // 86a409
    pub(super) fn sprite_13_mini_helmasaur(&mut self, k: usize) {
        if self.sprite_13_mini_helmasaur_through_subtype2_increment(k) {
            self.helmasaur_hard_hat_beetle_common_after_subtype2_increment(k);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_13_mini_helmasaur_through_subtype2_increment(&mut self, k: usize) -> bool {
        let j = usize::from(
            ((self.sprite_slot_view(k).subtype2() >> 2) & 1)
                | (self.sprite_slot_view(k).direction() << 1),
        );
        let value = SPRITE_13_MINI_HELMASAUR_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value =
            (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_13_MINI_HELMASAUR_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
            let mut x = self.sprite_slot_view(k).x_velocity();
            if sign8(x) {
                x = x.wrapping_neg();
            }
            let mut y = self.sprite_slot_view(k).y_velocity();
            if sign8(y) {
                y = y.wrapping_neg();
            }
            let value = if x >= y {
                self.sprite_slot_view(k).x_velocity() >> 7
            } else {
                (self.sprite_slot_view(k).y_velocity() >> 7) + 2
            };
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        self.sprite_draw_single_large(k);
        self.helmasaur_hard_hat_beetle_common_through_subtype2_increment(k)
    }

    // -----------------------------------------------------------------------
    // void Sprite_26_HardhatBeetle(int k) {  // 86a460
    pub(in crate::zelda_rtl) fn sprite_26_hardhat_beetle(&mut self, k: usize) {
        if self.sprite_26_hardhat_beetle_through_subtype2_increment(k) {
            self.helmasaur_hard_hat_beetle_common_after_subtype2_increment(k);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_26_hardhat_beetle_through_subtype2_increment(&mut self, k: usize) -> bool {
        let value = (self.sprite_slot_view(k).subtype2() >> 2) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.hard_hat_beetle_draw(k);
        self.helmasaur_hard_hat_beetle_common_through_subtype2_increment(k)
    }

    // -----------------------------------------------------------------------
    // void HelmasaurHardHatBeetleCommon(int k) {  // 86a46d
    fn helmasaur_hard_hat_beetle_common_through_subtype2_increment(&mut self, k: usize) -> bool {
        if self.sprite_return_if_inactive(k) {
            return false;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        true
    }

    pub(in crate::zelda_rtl) fn helmasaur_hard_hat_beetle_common_after_subtype2_increment(&mut self, k: usize) {
        if !self.helmasaur_hard_hat_until_tile_collision(k) {
            return;
        }
        self.sprite_check_tile_collision(k);
        self.helmasaur_hard_hat_after_tile_collision(k);
    }

    /// `HelmasaurHardHatBeetleCommon` after its subtype2 increment up to
    /// `Sprite_CheckTileCollision`: the recoil return, Link damage and the
    /// wall-collision-gated movement. Returns `false` on the recoil return.
    fn helmasaur_hard_hat_until_tile_collision(&mut self, k: usize) -> bool {
        if self.sprite_return_if_recoiling(k) {
            return false;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if (self.sprite_slot_view(k).wall_collision() & 15) != 0 {
            if (self.sprite_slot_view(k).wall_collision() & 3) != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        } else {
            self.sprite_move_xy(k);
        }
        true
    }

    /// `HelmasaurHardHatBeetleCommon` after `Sprite_CheckTileCollision`: the
    /// periodic speed targeting and the velocity steps.
    fn helmasaur_hard_hat_after_tile_collision(&mut self, k: usize) {
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 31) == 0 {
            let pt = self.sprite_project_speed_towards_link(k, self.sprite_slot_view(k).a());
            let value = pt.y;
            self.sprite_slot_view_mut(k).set_b(value);
            let value = pt.x;
            self.sprite_slot_view_mut(k).set_c(value);
        }
        if (((k as u8) ^ self.game_state.frame.frame_counter) & self.sprite_slot_view(k).ai_state())
            != 0
        {
            return;
        }
        let dy = self
            .sprite_slot_view(k)
            .y_velocity()
            .wrapping_sub(self.sprite_slot_view(k).b());
        let value = self
            .sprite_slot_view(k)
            .y_velocity()
            .wrapping_add(if sign8(dy) { 1 } else { 0xff });
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let dx = self
            .sprite_slot_view(k)
            .x_velocity()
            .wrapping_sub(self.sprite_slot_view(k).c());
        let value = self
            .sprite_slot_view(k)
            .x_velocity()
            .wrapping_add(if sign8(dx) { 1 } else { 0xff });
        self.sprite_slot_view_mut(k).set_x_velocity(value);
    }

    /// Runs the Helmasaur/Hardhat body on the interrupted host through
    /// `Sprite_CheckTileCollision` stage `stage`.
    pub(in crate::zelda_rtl) fn begin_helmasaur_hard_hat_tile_collision_checkpoint(
        &mut self,
        k: usize,
        stage: crate::SpriteTileCollisionStage,
    ) {
        let reached_increment = match self.sprite_slot_view(k).sprite_type() {
            0x13 => self.sprite_13_mini_helmasaur_through_subtype2_increment(k),
            0x26 => self.sprite_26_hardhat_beetle_through_subtype2_increment(k),
            sprite_type => panic!(
                "Helmasaur/Hardhat tile-collision checkpoint used sprite type ${sprite_type:02x}"
            ),
        };
        assert!(
            reached_increment,
            "Helmasaur/Hardhat tile-collision checkpoint requires an active sprite"
        );
        assert!(
            self.helmasaur_hard_hat_until_tile_collision(k),
            "Helmasaur/Hardhat tile-collision checkpoint requires a non-recoiling sprite"
        );
        self.sprite_check_tile_collision_until_stage(k, stage);
    }

    pub(in crate::zelda_rtl) fn resume_helmasaur_hard_hat_tile_collision(
        &mut self,
        k: usize,
        stage: crate::SpriteTileCollisionStage,
    ) {
        self.sprite_check_tile_collision_from_stage(k, stage);
        self.helmasaur_hard_hat_after_tile_collision(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_19_Poe(int k) {  // 869688
    pub(super) fn sprite_19_poe(&mut self, k: usize) {
        let mut j = usize::from(self.sprite_slot_view(k).x_velocity() >> 7);
        let value = j as u8;
        self.sprite_slot_view_mut(k).set_direction(value);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_19_POE_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if self.sprite_slot_view(k).e() == 0 {
            self.sprite_slot_view_mut(k).or_object_priority(0x30);
        }
        self.poe_draw(k);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        self.sprite_slot_view_mut(k).subtract_flags2(1);
        self.sprite_draw_single_large(k);
        self.sprite_slot_view_mut(k).add_flags2(1);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.sprite_slot_view(k).e() != 0 {
            self.sprite_slot_view_mut(k).add_z(1);
            if self.sprite_slot_view(k).z() == 12 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_e(value);
            }
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        // ROM $06:96E4
        self.sprite_move_xy_marked(k, 0x06e93e);
        if (self.game_state.frame.frame_counter & 1) == 0 {
            j = usize::from(self.sprite_slot_view(k).g() & 1);
            self.sprite_slot_view_mut(k)
                .add_z_velocity(SPRITE_19_POE_ACCEL[j] as u8);
            if self.sprite_slot_view(k).z_velocity() == CHOMP_Z_VELOCITY_TARGETS[j] as u8 {
                self.sprite_slot_view_mut(k).add_g(1);
            }
        }
        self.sprite_move_z(k);
        let value = 0;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    if (self.get_random_number() & 12) == 0 {
                        let value = self.sprite_is_below_link(k).a;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                    } else {
                        let value = self.get_random_number() & 1;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                    }
                }
            }
            1 => {
                if (self.game_state.frame.frame_counter & 1) == 0 {
                    j = usize::from(
                        (self.sprite_slot_view(k).anim_clock() & 1)
                            + self.game_state.world.region.dark_world_region_index() * 2,
                    );
                    let value = self
                        .sprite_slot_view(k)
                        .x_velocity()
                        .wrapping_add(SPRITE_19_POE_ACCEL[j] as u8);
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    if self.sprite_slot_view(k).x_velocity() == CHOMP_X_VELOCITY_TARGETS[j] as u8 {
                        let value = self.sprite_slot_view(k).anim_clock().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_anim_clock(value);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        // ROM Sprite_19_Poe $06:9769: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                        let value = self.get_random_number_with_carry().masked_adc(31, 16);
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                }
                let value = SPRITE_19_POE_LOCAL_Y_VELOCITIES
                    [usize::from(self.sprite_slot_view(k).head_direction())]
                    as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_6D_Rat(int k) {  // 85a8b0
    pub(super) fn sprite_6_d_rat(&mut self, k: usize) {
        let flags = self.rat_animation_until_oam_flags_store(k);
        self.complete_rat_animation_after_graphics(k, flags);
    }

    /// $05:A8B0-A8BE publishes graphics and computes the pending flags byte.
    /// The byte belongs to this native call, including across an NMI before
    /// $05:A8C1's store; the flags and drawing have not been published yet.
    pub(in crate::zelda_rtl) fn rat_animation_until_oam_flags_store(&mut self, k: usize) -> u8 {
        // $05:A8B0-A8BE: LDY, LDA, STA, LDA, AND, ORA. The animation
        // tables and the sixteen sprite slots stay within their pages.
        crate::cycle_ledger::charge(32 + 32 + 38 + 32 + 16 + 32);
        let j = usize::from(self.sprite_slot_view(k).a());
        let value = RAT_GRAPHICS_BY_ANIM_STATE[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        (self.sprite_slot_view(k).oam_flags() & 0x3f) | RAT_OAM_FLIP_BY_ANIM_STATE[j]
    }

    pub(in crate::zelda_rtl) fn complete_rat_animation_after_graphics(&mut self, k: usize, flags: u8) {
        // $05:A8C1 STA flags; $05:A8C4 JSL the bank-6 drawing wrapper.
        crate::cycle_ledger::charge(38 + 62);
        self.sprite_slot_view_mut(k).set_oam_flags(flags);
        {
            let _wrapper = crate::cycle_ledger::routine(0x06_dbf0);
            crate::cycle_ledger::charge(190); // PHB; PHK; PLB; JSR; PLB; RTL.
            self.sprite_draw_single_large(k);
        }
        crate::cycle_ledger::charge(46); // $05:A8C8 JSR Sprite_ReturnIfInactive_.
        if self.sprite_return_if_inactive_bank5(k) {
            return;
        }
        // $05:A8CB..A95A: executed rat control suffix, M8/X8.
        let lag_free = self.lag_free;
        let charge = |master| { if lag_free { crate::cycle_ledger::charge(master); } };
        charge(46); // $A8CB recoil JSR; double return skips the suffix.
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        charge(3 * 46); // $A8CE/A8D1/A8D4 child JSRs.
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        let _ = self.sprite_check_tile_collision(k);
        charge(48); // $A8D7..A8DA state test.
        if self.sprite_slot_view(k).ai_state() != 0 {
            charge(6 + 48); // Taken BNE, $A917..A91A delay test.
            let delay_expired = self.sprite_slot_view(k).delay_main() == 0;
            if delay_expired {
                charge(48 + 92); // $A91C..A91F world test, $A927..A92C stores.
                if !self.game_state.world.region.is_in_dark_world() {
                    charge(78); // $A921..A923 sound argument/JSL.
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x17);
                } else {
                    charge(6); // $A91F taken BNE.
                }
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 80;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            if !delay_expired { charge(6); } // $A91A taken BNE.
            charge(80); // $A92F..A935 direction and wall test.
            let mut j = usize::from(self.sprite_slot_view(k).direction());
            if self.sprite_slot_view(k).wall_collision() != 0 {
                charge(84); // $A937..A93D turn stores.
                let value = RAT_WALL_TURN_DIRECTIONS[j];
                self.sprite_slot_view_mut(k).set_direction(value);
                j = usize::from(self.sprite_slot_view(k).direction());
            } else { charge(6); } // $A935 taken BEQ.
            charge(378); // $A93E..A95A velocities, animation, RTS.
            let value = SPRITE_6_D_RAT_LOCAL_X_VELOCITIES[j] as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = SPRITE_6_D_RAT_LOCAL_Y_VELOCITIES[j] as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let value = RAT_RUN_ANIM_STATES[usize::from(
                self.sprite_slot_view(k).direction() * 2
                    + ((self.game_state.frame.frame_counter >> 2) & 1),
            )];
            self.sprite_slot_view_mut(k).set_a(value);
        } else {
            charge(94); // $A8DC..A8E2 zero-velocity JSR and delay test.
            self.sprite_zero_velocity_xy(k);
            if self.sprite_slot_view(k).delay_main() == 0 {
                charge(288); // $A8E4..A8F6 RNG JSL, direction, delay.
                let random = self.get_random_number_with_carry();
                let a = random.value();
                let value = a & 3;
                self.sprite_slot_view_mut(k).set_direction(value);
                self.sprite_slot_view_mut(k).add_ai_state(1);
                let value = rat_random_run_delay(random);
                self.sprite_slot_view_mut(k).set_delay_main(value);
            } else { charge(6); } // $A8E2 taken BNE.
            charge(252); // $A8F9..A90A idle animation and RTS.
            let value = RAT_IDLE_ANIM_STATES[usize::from(
                self.sprite_slot_view(k).direction() * 2
                    + ((self.game_state.frame.frame_counter >> 3) & 1),
            )];
            self.sprite_slot_view_mut(k).set_a(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_6E_Rope(int k) {  // 85a973
    pub(super) fn sprite_6_e_rope(&mut self, k: usize) {
        let mut j = usize::from(self.sprite_slot_view(k).a());
        let value = ROPE_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | ROPE_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        // ROM $05:A987 JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).e() != 0 {
            let oam = self.game_state.oam.current_pointer_usize();
            self.oam_state_mut().or_entry_flags(oam, 0x30);

            let old_z = self.sprite_slot_view(k).z();
            self.sprite_move_z(k);
            if !sign8(self.sprite_slot_view(k).z_velocity().wrapping_sub(0xc0)) {
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
            }
            if !sign8(old_z ^ self.sprite_slot_view(k).z()) || !sign8(self.sprite_slot_view(k).z())
            {
                return;
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_e(value);
            self.sprite_slot_view_mut(k).and_flags3(!0x10);
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_flags2(value);
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            self.sprite_check_damage_to_and_from_link(k);
            // ROM $05:A9D5
            self.sprite_move_xy_marked(k, 0x05fa00);
            // ROM $05:A9D8 -> $05:B88D -> $06:E496
            let _ = self.sprite_check_tile_collision_long(k);
            if self.sprite_slot_view(k).ai_state() != 0 {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                j = usize::from(self.sprite_slot_view(k).direction());
                if self.sprite_slot_view(k).wall_collision() != 0 {
                    let value = ROPE_IDLE_GFX_BY_DIRECTION[j];
                    self.sprite_slot_view_mut(k).set_direction(value);
                    j = usize::from(self.sprite_slot_view(k).direction());
                }

                j += usize::from(self.sprite_slot_view(k).g());
                let value = ROPE_X_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = ROPE_Y_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);

                let mut i = self.game_state.frame.frame_counter;
                if j < 4 {
                    i >>= 1;
                }

                let value = ROPE_FAST_GFX_BY_DIRECTION
                    [usize::from(self.sprite_slot_view(k).direction() * 2 + ((i >> 1) & 1))];

                self.sprite_slot_view_mut(k).set_a(value);
            } else {
                self.sprite_zero_velocity_xy(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_g(value);
                    let random = self.get_random_number_with_carry();
                    let a = random.value();
                    let value = a & 3;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = rope_random_run_delay(random);
                    self.sprite_slot_view_mut(k).set_delay_main(value);

                    let (dir, pt) = self.sprite_direction_and_offset_to_face_link(k);
                    if pt.y.wrapping_add(0x10) < 0x20 || pt.x.wrapping_add(0x18) < 0x20 {
                        let value = 4;
                        self.sprite_slot_view_mut(k).set_g(value);
                        let value = dir;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                }
                let value = ROPE_FAST_GFX_BY_DIRECTION[usize::from(
                    self.sprite_slot_view(k).direction() * 2
                        + ((self.game_state.frame.frame_counter >> 3) & 1),
                )];
                self.sprite_slot_view_mut(k).set_a(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_6F_Keese(int k) {  // 85aa8b
    pub(super) fn sprite_6_f_keese(&mut self, k: usize) {
        self.keese_draw_before_inactive_check(k);
        self.complete_keese_after_draw(k);
    }

    pub(in crate::zelda_rtl) fn keese_draw_before_inactive_check(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).or_object_priority(0x30);
        self.sprite_draw_single_large(k);
    }

    pub(in crate::zelda_rtl) fn complete_keese_after_draw(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 64;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_zero_velocity_xy(k);
            } else {
                if (self.sprite_slot_view(k).delay_main() & 7) == 0 {
                    let value = self.sprite_slot_view(k).a().wrapping_add(
                        KEESE_ORBIT_DIRECTION_STEPS[usize::from(self.sprite_slot_view(k).b() & 1)]
                            as u8,
                    );
                    self.sprite_slot_view_mut(k).set_a(value);
                    if (self.get_random_number() & 3) == 0 {
                        self.sprite_slot_view_mut(k).add_b(1);
                    }
                }
                let j = usize::from(self.sprite_slot_view(k).a() & 0x0f);
                let value = KEESE_ATTACK_X_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = KEESE_ATTACK_Y_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = ((self.game_state.frame.frame_counter >> 2) & 1) + 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
        } else {
            if ((((k as u8) ^ self.game_state.frame.frame_counter) & 3)
                | self.sprite_slot_view(k).delay_main())
                != 0
            {
                return;
            }

            let (dir, pt) = self.sprite_direction_and_offset_to_face_link(k);
            if pt.y.wrapping_add(0x28) >= 0x50 || pt.x.wrapping_add(0x28) >= 0x50 {
                return;
            }
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
            self.sprite_slot_view_mut(k).add_ai_state(1);
            let value = 64;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 64;
            self.sprite_slot_view_mut(k).set_b(value);
            let value = KEESE_ATTACK_START_PHASES[usize::from(dir)] as u8;
            self.sprite_slot_view_mut(k).set_a(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_51_ArmosStatue(int k) {  // 85b703
    pub(super) fn sprite_51_armos_statue(&mut self, k: usize) {
        self.armos_draw(k);
        if self.sprite_slot_view(k).f() != 0 {
            self.sprite_zero_velocity_xy(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_z(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            self.sprite_zero_velocity_xy(k);
        }
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_slot_view_mut(k).or_flags3(0x40);
            if self.sprite_slot_view(k).delay_main() == 1 {
                self.sprite_slot_view_mut(k).and_flags3(!0x40);
                self.sprite_slot_view_mut(k).add_ai_state(1);
                self.sprite_slot_view_mut(k).and_flags2(!0x80);
                self.sprite_slot_view_mut(k).and_flags3(!0x40);
                let value = 0x0b;
                self.sprite_slot_view_mut(k).set_oam_flags(value);
            } else {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
                    && self
                        .game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                        .wrapping_add(31)
                        < 62
                    && self
                        .game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(8)
                        .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                        .wrapping_add(48)
                        < 88
                    && self.sprite_slot_view(k).delay_main() == 0
                {
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
                }
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.sprite_nullify_hookshot_drag();
                    self.sprite_repel_dash();
                }
                if self.sprite_slot_view(k).delay_main() != 0 {
                    let value = (self.sprite_slot_view(k).delay_main() >> 1) & 0x0e;
                    self.sprite_slot_view_mut(k).xor_oam_flags(value);
                }
            }
        } else {
            self.sprite_check_damage_to_and_from_link(k);
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            // ROM $05:B7C0
            self.sprite_move_xy_marked(k, 0x05fa00);
            // ROM $05:B7C3 -> $05:B88D -> $06:E496
            let _ = self.sprite_check_tile_collision_long(k);
            if (self.sprite_slot_view(k).delay_main() | self.sprite_slot_view(k).z()) == 0 {
                let value = 8;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 16;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                self.sprite_apply_speed_towards_link(k, 12);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_7C_GreenStalfos(int k) {  // 9ed299
    pub(super) fn sprite_7_c_green_stalfos(&mut self, k: usize) {
        let mut j = usize::from(self.sprite_slot_view(k).direction() & 3);
        let value =
            (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_7_C_GREEN_STALFOS_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = SPRITE_7_C_GREEN_STALFOS_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        // ROM $1E:D2AD JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        j = usize::from(self.sprite_direction_to_face_link(k) & 3);
        if SPRITE_7_C_GREEN_STALFOS_DIRECTIONS[j] != self.game_state.player.follower_link.facing() {
            let value = 0;
            self.sprite_slot_view_mut(k).set_a(value);
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0 {
                let vel = self.sprite_slot_view(k).b();
                if vel != 4 {
                    self.sprite_slot_view_mut(k).add_b(1);
                }
                self.sprite_apply_speed_towards_link(k, vel);
                let value = self.sprite_is_right_of_link(k).a;
                self.sprite_slot_view_mut(k).set_direction(value);
            }
        } else {
            let value = 1;
            self.sprite_slot_view_mut(k).set_a(value);
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
                let vel = self.sprite_slot_view(k).b();
                if vel != 0 {
                    self.sprite_slot_view_mut(k).subtract_b(1);
                }
                self.sprite_apply_speed_towards_link(k, vel);
                let value = self.sprite_is_right_of_link(k).a;
                self.sprite_slot_view_mut(k).set_direction(value);
            }
        }
        self.sprite_move_xy_bank(k, 0x1eff34);
    }

    pub(in crate::zelda_rtl) fn eyegore_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        let mut info = self.sprite_draw_multiple(k, &EYEGORE_DRAW_FRAMES[base..base + 4]);
        if self.sprite_slot_view(k).pause() == 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 14);
        }
    }

    pub(in crate::zelda_rtl) fn chain_chomp_draw(&mut self, k: usize) {
        let j = usize::from(self.sprite_slot_view(k).direction() & 0x0f);
        let value = CHAIN_CHOMP_DRAW_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | CHAIN_CHOMP_DRAW_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        // ROM $1D:C1A6 JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        let mut oam = self.game_state.oam.current_pointer_usize() + 4;
        let flags =
            self.sprite_slot_view(k).oam_flags() ^ self.sprite_slot_view(k).object_priority();
        let r8 = u16::from(self.sprite_slot_view(k).delay_aux1() & 1) + 4;
        let mut pos = k * 8;
        for _ in (0..6).rev() {
            let x = self
                .game_state
                .sprites
                .chain_chomp_history
                .x(pos)
                .wrapping_add(r8)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y = self
                .game_state
                .sprites
                .chain_chomp_history
                .y(pos)
                .wrapping_add(r8)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            self.set_oam_helper0_at(oam, x, y, 0x8b, (flags & 0xf0) | 0x0d, 0);
            pos += 1;
            oam += 4;
        }
    }
}
