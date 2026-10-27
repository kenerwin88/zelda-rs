//! Zoras, King Zora, Catfish, Swamola, and other aquatic sprites.

use crate::zelda_rtl::sprite::{
    PrepOamCoordsRet, SpriteDamageBothEntry, SpriteDamageFromEntry, SpriteDamageToEntry,
    SpriteHitBoxEntry, SpriteInactiveEntry, SpriteMoveEntry, SpriteMoveZEntry, SpriteRecoilEntry,
    SpriteShadowEntry, SpriteSingleLargeEntry, SpriteSingleSmallEntry, SpriteSpawnInfo,
    SpriteTileCollisionEntry,
};
use super::*;
use crate::tile_definition::NativeTile;
use crate::types::sign8;
use crate::types::ProjectSpeedRet;
use crate::types::SpriteHitBox;

impl ZeldaState {

    // -----------------------------------------------------------------------
    // void SpriteDraw_WaterRipple_WithOamAdjust(int k) {  // 859fe5
    //   SpriteDraw_WaterRipple(k);
    //   oam_cur_ptr += 8;
    //   oam_ext_cur_ptr += 2;
    // }
    pub(super) fn sprite_draw_water_ripple_with_oam_adjust(&mut self, k: usize) {
        self.sprite_draw_water_ripple(k);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));
    }

    // -----------------------------------------------------------------------
    // void WalkingZora_AdjustShadow(int k) {  // 859edb
    pub(super) fn walking_zora_adjust_shadow(&mut self, k: usize) {
        let value = u8::from(
            self.sprite_slot_view(k).z() == 0
                && self.game_state.sprites.workspace.tile() == NativeTile::SHALLOW_WATER,
        );
        self.sprite_slot_view_mut(k).set_anim_clock(value);
    }

    // -----------------------------------------------------------------------
    // void Zora_Draw(int k) {  // 8598f5
    pub(super) fn zora_draw(&mut self, k: usize) {
        let Some((x, y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        for i in (0..2usize).rev() {
            let j = d + i;
            let f = ZORA_DRAW_FLAGS[j];
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(ZORA_DRAW_X_OFFSETS[j] as i16 as u16),
                y.wrapping_add(ZORA_DRAW_Y_OFFSETS[j] as i16 as u16),
                ZORA_DRAW_CHARS[j],
                f | if f & 0x0f != 0 { 0 } else { info_flags },
                ZORA_DRAW_BIG[j],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_55_Zora(int k) {  // 85967b
    pub(super) fn sprite_55_zora(&mut self, k: usize) {
        if self.sprite_slot_view(k).e() != 0 {
            self.sprite_fireball(k);
        } else {
            self.sprite_zora_main(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Fireball(int k) {  // 859683
    pub(in crate::zelda_rtl) fn sprite_fireball(&mut self, k: usize) {
        if self.sprite_fireball_until_after_movement(k) {
            self.sprite_fireball_after_movement(k);
        }
    }

    /// `Sprite_Fireball` through its `Sprite_MoveXY`; returns `false` on the
    /// inactive return and the Link-damage kill.
    pub(super) fn sprite_fireball_until_after_movement(&mut self, k: usize) -> bool {
        let value = self.sprite_slot_view(k).e();

        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        if self.sprite_slot_view(k).delay_main() != 0 {
            self.oam_allocate_from_region_c(4);
        }
        self.sprite_draw_single_small(k, SpriteSingleSmallEntry::Unpriced);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return false;
        }
        self.fireball_spawn_trail_garnish(k);
        // $05:969C JSL $06:F121: complete body, caller opcode unchanged.
        if self.sprite_check_damage_to_link(k, SpriteDamageToEntry::Rom06F121) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return false;
        }
        if let Some(SpriteMainCpuBoundary::ZoraFireballMovement {
            slot,
            checkpoint,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if usize::from(slot) == k {
                // Bind the Sprite_MoveXY bytes through the source checkpoint
                // and park; the resumed host publishes the rest.
                let continuation = self.sprite_move_xy_through_checkpoint(k, checkpoint);
                self.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::ZoraFireballMovement {
                    slot,
                    checkpoint,
                    continuation: Some(continuation),
                });
                return false;
            }
        }
        // $05:96A6 JSR $05:F9ED: complete body, caller opcode unchanged.
        self.sprite_move_xy(k, SpriteMoveEntry::Rom05FA00);
        true
    }

    /// Resumes `Sprite_Fireball` from a `Sprite_MoveXY` checkpoint with the
    /// bytes bound by `sprite_fireball_until_after_movement`.
    pub(in crate::zelda_rtl) fn complete_zora_fireball_movement(
        &mut self,
        k: usize,
        checkpoint: crate::SpriteMoveXYCheckpoint,
        continuation: SpriteMoveXYContinuation,
    ) {
        self.sprite_move_xy_from_checkpoint(k, checkpoint, continuation);
        self.sprite_fireball_after_movement(k);
    }

    /// `Sprite_Fireball` after `Sprite_MoveXY`: the frame-gated tile
    /// collision kill and the shield-reflect check.
    pub(super) fn sprite_fireball_after_movement(&mut self, k: usize) {
        if self.game_state.world.location.is_indoors()
            && self.sprite_slot_view(k).delay_aux1() == 0
            && (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
            && self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced) != 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }

        if self.game_state.player.follower_link.is_bunny_mirror()
            || self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer()
                != 0
            || sign8(self.game_state.player.follower_link.state_bits())
            || self.game_state.inventory.items.shield_type() < 2
            || self.game_state.player.follower_link.lower_level_state()
                != self.sprite_slot_view(k).floor()
        {
            return;
        }

        let mut hb = SpriteHitBox {
            r0_xlo: 0,
            r8_xhi: 0,
            r1_ylo: 0,
            r9_yhi: 0,
            r2: 0,
            r3: 0,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        };
        self.sprite_setup_hit_box(k, &mut hb, SpriteHitBoxEntry::Unpriced);
        let mut j = self.game_state.player.follower_link.facing_index() & 3;
        if self.game_state.player.follower_link.button_b_frames() != 0 {
            j = usize::from(SPRITE_FIREBALL_OFFSETS[j]);
        }
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(SPRITE_FIREBALL_X_OFFSETS[j] as i16 as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(SPRITE_FIREBALL_Y_OFFSETS[j] as i16 as u16);
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        hb.r2 = SPRITE_FIREBALL_WIDTHS[j];
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
        hb.r3 = SPRITE_FIREBALL_HEIGHTS[j];
        if self.check_if_hit_boxes_overlap(&hb) {
            self.sprite_place_rupulse_spark_2(k);
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 6);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Zora_Main(int k) {  // 859725
    pub(super) fn sprite_zora_main(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_prep_oam_coord(k);
        } else {
            self.zora_draw(k);
        }
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let org_x = u16::from(self.sprite_slot_view(k).a())
                        | (u16::from(self.sprite_slot_view(k).b()) << 8);
                    let org_y = u16::from(self.sprite_slot_view(k).c())
                        | (u16::from(self.sprite_slot_view(k).head_direction()) << 8);
                    let x_idx = usize::from(self.get_random_number() & 7);
                    self.sprite_set_x(k, org_x.wrapping_add(SURFACE_XY[x_idx] as i16 as u16));
                    let y_idx = usize::from(self.get_random_number() & 7);
                    self.sprite_set_y(k, org_y.wrapping_add(SURFACE_XY[y_idx] as i16 as u16));
                    self.sprite_get16_bit_coords(k);
                    // ROM $05:979A -> $05:B88D -> $06:E496
                    self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Rom06E496);
                    let spawn_anyway = self.game_state.sprites.workspace.tile()
                        == NativeTile::DEEP_WATER
                        || (self.game_state.sprites.workspace.tile() == NativeTile::SHALLOW_WATER
                            && self.sprite_slot_view(k).delay_aux2() == 1
                            && self
                                .game_state
                                .enhanced_features
                                .has(FEATURE_GAME_CHANGING_BUG_FIXES_DRAW));
                    if spawn_anyway {
                        let value = 127;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.sprite_slot_view_mut(k).or_flags3(0x40);
                    } else if self
                        .game_state
                        .enhanced_features
                        .has(FEATURE_GAME_CHANGING_BUG_FIXES_DRAW)
                    {
                        self.sprite_set_x(k, org_x);
                        self.sprite_set_y(k, org_y);
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                        if self.sprite_slot_view(k).delay_aux2() == 0 {
                            let value = 32;
                            self.sprite_slot_view_mut(k).set_delay_aux2(value);
                        }
                    }
                }
            }
            1 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).and_flags3(!0x40);
                } else {
                    let value = ZORA_SURFACING_GRAPHICS
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                // $05:97F1 JSR $05:AB93: both complete damage calls.
                self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom05AB93);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 23;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    if self.sprite_slot_view(k).delay_main() == 48 {
                        let _ = self.sprite_spawn_fireball(k);
                    }
                    let value = SPRITE_ZORA_MAIN_ATTACK_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 128;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    let value = SPRITE_ZORA_MAIN_SUBMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 2)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_SpawnBigSplash(int k) {  // 859b40
    pub(in crate::zelda_rtl) fn sprite_spawn_big_splash(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x24);
        for i in (0..8usize).rev() {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 8, &mut info);
            if j >= 0 {
                let j = j as usize;
                let value = 3;
                self.sprite_slot_view_mut(j).set_state(value);
                self.sprite_set_x(
                    j,
                    info.r0_x
                        .wrapping_add(SPRITE_SPAWN_BIG_SPLASH_X_OFFSETS[i] as i16 as u16)
                        .wrapping_sub(4),
                );
                self.sprite_set_y(
                    j,
                    info.r2_y
                        .wrapping_add(SPRITE_SPAWN_BIG_SPLASH_Y_OFFSETS[i] as i16 as u16)
                        .wrapping_sub(4),
                );
                let value = SPRITE_SPAWN_BIG_SPLASH_LOCAL_X_VELOCITIES[i] as u8;
                self.sprite_slot_view_mut(j).set_x_velocity(value);
                let value = SPRITE_SPAWN_BIG_SPLASH_LOCAL_Y_VELOCITIES[i] as u8;
                self.sprite_slot_view_mut(j).set_y_velocity(value);
                let value = i as u8;
                self.sprite_slot_view_mut(j).set_a(value);
                // ROM Sprite_SpawnBigSplash $05:9B94: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                let value = self.get_random_number_with_carry().masked_adc(15, 24);
                self.sprite_slot_view_mut(j).set_z_velocity(value);
                let value = 1;
                self.sprite_slot_view_mut(j).set_ai_state(value);
                let value = 0;
                self.sprite_slot_view_mut(j).set_z(value);
                self.sprite_slot_view_mut(j).or_flags3(0x40);
                let value = self.sprite_slot_view(j).flags3();
                self.sprite_slot_view_mut(j).set_ignore_projectile(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_52_KingZora(int k) {  // 85995b
    pub(super) fn sprite_52_king_zora(&mut self, k: usize) {
        assert!(!self.sprite_52_king_zora_impl(k, false));
    }

    /// Execute King Zora through the exact call site immediately before the
    /// purchased-flippers sheet decode. This is used only when the timing
    /// authority proves that the source CPU suspended inside that synchronous
    /// call; the returned `true` proves the live sprite took that path.
    pub(in crate::zelda_rtl) fn sprite_52_king_zora_before_flippers_graphics(&mut self, k: usize) -> bool {
        self.sprite_52_king_zora_impl(k, true)
    }

    fn sprite_52_king_zora_impl(&mut self, k: usize, stop_before_flippers_graphics: bool) -> bool {
        self.zora_king_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return false;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let dx = self.game_state.player.follower_link.x();
                let dy = self.game_state.player.follower_link.y();
                if dx.wrapping_sub(self.sprite_get_x(k)).wrapping_add(16) < 32
                    && dy.wrapping_sub(self.sprite_get_y(k)).wrapping_add(48) < 96
                {
                    self.link_cancel_dash();
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.set_sound_effect_1(0x35);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    for j in (0..16usize).rev() {
                        if j != k && (self.sprite_slot_view(j).deflection_bits() & 0x80) == 0 {
                            if self.sprite_slot_view(j).state() == 10 {
                                self.follower_link_state_mut().clear_state_bits();
                                self.follower_link_state_mut().clear_picking_throw_state();
                            }
                            self.sprite_kill_self(j);
                        }
                    }
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.set_bg1_x_offset(0);
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                } else {
                    let offs = if self.sprite_slot_view(k).delay_main() & 1 != 0 {
                        (-1i16) as u16
                    } else {
                        1
                    };
                    self.set_bg1_x_offset(offs);
                    self.follower_link_state_mut().immobilize();
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    if self.sprite_slot_view(k).delay_main() == 28 {
                        let value = 15;
                        self.sprite_slot_view_mut(k).set_delay_aux2(value);
                        self.sprite_spawn_big_splash(k);
                    }
                    let value = SPRITE_52_KING_ZORA_SURFACING_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                let j = self.sprite_slot_view(k).delay_main();
                if j == 0 {
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 36;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    return false;
                }
                let value = SPRITE_52_KING_ZORA_DIALOGUE_GFX[usize::from(j >> 4)];
                self.sprite_slot_view_mut(k).set_graphics(value);
                if j == 80 {
                    self.dialogue_message_index_mut().set_value(0x142);
                    self.sprite_show_message_minimal_c();
                } else if j == 79 {
                    if self.multiselect_choice().value() == 0 {
                        self.dialogue_message_index_mut().set_value(0x143);
                        self.sprite_show_message_minimal_c();
                    } else {
                        self.dialogue_message_index_mut().set_value(0x146);
                        self.sprite_show_message_minimal_c();
                        let value = 0x30;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                } else if j == 78 {
                    if self.multiselect_choice().value() == 0
                        && self.game_state.inventory.player_resources.rupees_goal() >= 500
                    {
                        let rupees = self
                            .game_state
                            .inventory
                            .player_resources
                            .rupees_goal()
                            .wrapping_sub(500);
                        self.player_resources_mut().set_rupees_goal(rupees);
                        self.dialogue_message_index_mut().set_value(0x144);
                        self.sprite_show_message_minimal_c();
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_e(value);
                    } else {
                        self.dialogue_message_index_mut().set_value(0x145);
                        self.sprite_show_message_minimal_c();
                        let value = 0x30;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                } else if j == 77 && self.sprite_slot_view(k).e() != 0 {
                    if stop_before_flippers_graphics {
                        if self.sprite_zora_regurgitate_flippers_before_graphics(k) {
                            return true;
                        }
                    } else {
                        self.sprite_zora_regurgitate_flippers(k);
                    }
                }
            }
            4 => {
                let j = self.sprite_slot_view(k).delay_main();
                if j == 0 {
                    self.sprite_kill_self(k);
                    self.follower_link_state_mut().clear_immobilized();
                } else {
                    if j == 29 {
                        let value = 15;
                        self.sprite_slot_view_mut(k).set_delay_aux2(value);
                        self.sprite_spawn_big_splash(k);
                    }
                    let value = SPRITE_52_KING_ZORA_SUBMERGE_GFX[usize::from(j >> 1)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
        false
    }

    // -----------------------------------------------------------------------
    // void Sprite_Zora_RegurgitateFlippers(int k) {  // 9de1aa
    pub(super) fn sprite_zora_regurgitate_flippers(&mut self, k: usize) {
        if self.sprite_zora_regurgitate_flippers_before_graphics(k) {
            self.decode_animated_sprite_tile_variable(0x11);
        }
    }

    fn sprite_zora_regurgitate_flippers_before_graphics(&mut self, k: usize) -> bool {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc0, &mut info);
        if j < 0 {
            return false;
        }
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        let value = 32;
        self.sprite_slot_view_mut(j).set_z_velocity(value);
        let value = 16;
        self.sprite_slot_view_mut(j).set_y_velocity(value);
        let value = 30;
        self.sprite_slot_view_mut(j).set_a(value);
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        let value = 0x83;
        self.sprite_slot_view_mut(j).set_flags2(value);
        let value = 0x54;
        self.sprite_slot_view_mut(j).set_flags3(value);
        let value = 0x54 & 15;
        self.sprite_slot_view_mut(j).set_oam_flags(value);
        let value = 0x30;
        self.sprite_slot_view_mut(j).set_delay_aux3(value);
        true
    }

    // -----------------------------------------------------------------------
    // void ZoraKing_Draw(int k) {  // 859cab
    pub(super) fn zora_king_draw(&mut self, k: usize) {
        let Some((mut x, mut y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };

        if self.sprite_slot_view(k).ai_state() >= 2 {
            let mut oam = self.game_state.oam.current_pointer_usize();
            let g = usize::from(self.sprite_slot_view(k).graphics());
            for i in (0..4usize).rev() {
                let j = g * 4 + i;
                let f = ZORA_KING_DRAW_FLAGS_0[j];
                self.oam_state_mut().write_entry(
                    oam,
                    x.wrapping_add(ZORA_KING_DRAW_X_OFFSETS_0[j] as i16 as u16) as u8,
                    y.wrapping_add(ZORA_KING_DRAW_Y_OFFSETS_0[j] as i16 as u16) as u8,
                    ZORA_KING_DRAW_CHARS_0[j],
                    (if f & 0x0f != 0 { f } else { f | info_flags }) | 0x20,
                );
                oam += 4;
            }
            self.sprite_correct_oam_entries(k, 3, 2);
            let Some(poc) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
                return;
            };
            x = poc.0;
            y = poc.1;
        }

        if self.sprite_slot_view(k).delay_aux2() == 0 {
            return;
        }

        self.oam_allocate_from_region_c(0x10);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from((self.sprite_slot_view(k).delay_aux2() >> 1) & 4);
        for i in (0..4usize).rev() {
            let j = g + i;
            self.set_oam_plain_at(
                oam,
                x.wrapping_add(ZORA_KING_DRAW_X_OFFSETS_1[j] as i16 as u16) as u8,
                y.wrapping_add(ZORA_KING_DRAW_Y_OFFSETS_1[j] as i16 as u16) as u8,
                ZORA_KING_DRAW_CHARS_1[j],
                ZORA_KING_DRAW_FLAGS_1[j] | 0x24,
                2,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_C0_Catfish(int k) {  // 9ddf49
    pub(super) fn sprite_c0_catfish(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() & 0x80 != 0 {
            self.sprite_catfish_splash_of_water(k);
        } else if self.sprite_slot_view(k).a() == 0 {
            self.catfish_big_fish(k);
        } else {
            self.sprite_catfish_quake_medallion(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Catfish_QuakeMedallion(int k) {  // 9ddf54
    pub(super) fn sprite_catfish_quake_medallion(&mut self, k: usize) {
        if self.sprite_slot_view(k).z() == 0 {
            self.sprite_draw_water_ripple_with_oam_adjust(k);
            if self.game_state.frame.submodule == 0
                // ROM $1D:DF61
                && self.sprite_check_damage_same_layer_long_marked(k)
            {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.follower_link_state_mut().set_item_receipt_method(0);
                let item = self.sprite_slot_view(k).a();
                if self
                    .link_receive_item_from(
                        item,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::CatfishMedallion,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_catfish_medallion_item_receipt(k);
                return;
            }
        }
        if self.sprite_slot_view(k).delay_aux3() != 0 {
            self.oam_allocate_from_region_c(8);
        }
        // ROM $1D:DF81 JSL $06:DBF0
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_move_xyz(k, SpriteMoveEntry::Rom1DE95D);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = ((self.sprite_slot_view(k).x_velocity() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = ((self.sprite_slot_view(k).y_velocity() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let j = self.sprite_slot_view(k).ai_state();
            if j == 4 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
            } else {
                self.sprite_slot_view_mut(k).add_ai_state(1);

                let value = SPRITE_CATFISH_QUAKE_MEDALLION_Z_VELOCITIES[usize::from(j)];
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                if j < 2 {
                    let splash = self.sprite_spawn_water_splash(k);
                    if splash >= 0 {
                        let value = 16;
                        self.sprite_slot_view_mut(splash as usize)
                            .set_delay_main(value);
                    }
                }
            }
        }
    }

    /// Source suffix after Sprite_Catfish_QuakeMedallion's
    /// `Link_ReceiveItem(sprite_A, 0)` (ROM `$9ddf54`): the sprite was killed
    /// before the call, so only the OAM-region allocation and the single large
    /// draw run before `Sprite_ReturnIfInactive` returns.
    pub(in crate::zelda_rtl) fn complete_catfish_medallion_item_receipt(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux3() != 0 {
            self.oam_allocate_from_region_c(8);
        }
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
        debug_assert!(self.sprite_slot_view(k).state() == 0);
    }

    // -----------------------------------------------------------------------
    // void Catfish_BigFish(int k) {  // 9ddfd1
    pub(in crate::zelda_rtl) fn catfish_big_fish(&mut self, k: usize) {
        self.great_catfish_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                for j in (0..16usize).rev() {
                    if j == k || self.sprite_slot_view(j).state() != 3 {
                        continue;
                    }
                    if self
                        .sprite_get_x(k)
                        .wrapping_sub(self.sprite_get_x(j))
                        .wrapping_add(32)
                        < 64
                        && self
                            .sprite_get_y(k)
                            .wrapping_sub(self.sprite_get_y(j))
                            .wrapping_add(32)
                            < 64
                    {
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 255;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        return;
                    }
                }
            }
            1 => {
                let j = self.sprite_slot_view(k).delay_main();
                if j == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.set_bg1_x_offset(0);
                    self.set_ambient_sound_effect(5);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    self.catfish_spawn_plop(k);
                } else if j < 0xc0 {
                    if j == 0xbf {
                        self.set_ambient_sound_effect(7);
                    }
                    let offs = if j & 1 != 0 { (-1i16) as u16 } else { 1 };
                    self.set_bg1_x_offset(offs);
                    self.follower_link_state_mut().immobilize();
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).add_subtype2(1);
                self.sprite_move_xyz(k, SpriteMoveEntry::Rom1DE95D);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if self.sprite_slot_view(k).z_velocity() == (-48i8) as u8 {
                    self.catfish_spawn_plop(k);
                }
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                let value = CATFISH_BIG_FISH_EMERGE_GFX
                    [usize::from(self.sprite_slot_view(k).subtype2() >> 2)];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            3 => {
                let j = self.sprite_slot_view(k).delay_main();
                if j == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                } else {
                    // The ROM branches on carry after the splash call at
                    // countdown 160. A successful outdoor spawn's slot ASL
                    // clears it; a failed or indoor spawn preserves CMP #160.
                    let mut animation_lookup = j < 160;
                    if j == 160 {
                        let spawned = self.sprite_spawn_water_splash(k);
                        animation_lookup =
                            spawned >= 0 && !self.game_state.world.location.is_indoors();
                    } else if matches!(j, 252 | 4) {
                        self.sprite_spawn_water_splash(k);
                    } else if j == 10 {
                        self.catfish_spawn_plop(k);
                    } else if j == 96 {
                        self.follower_link_state_mut().clear_immobilized();
                        let msg = if self.game_state.inventory.items.quake() != 0 {
                            0x12b
                        } else {
                            0x12a
                        };
                        self.dialogue_message_index_mut().set_value(msg);
                        self.sprite_show_message_minimal_c();
                        return;
                    } else if j == 80 {
                        if self.game_state.inventory.items.quake() != 0 {
                            if self.get_random_number() & 1 != 0 {
                                self.sprite_spawn_bomb(k);
                            } else {
                                self.sprite_spawn_fireball(k);
                            }
                        } else {
                            self.catfish_regurgitate_medallion(k);
                        }
                    }
                    if animation_lookup {
                        let value = CATFISH_BIG_FISH_CONVERSATE_GFX[usize::from(j >> 3)];
                        self.sprite_slot_view_mut(k).set_graphics(value);
                    }
                }
            }
            _ => {}
        }
    }

    /// The successful medallion spawn precedes its shared graphics decode.
    pub(in crate::zelda_rtl) fn catfish_before_medallion_graphics(&mut self, k: usize) {
        assert_eq!(self.sprite_slot_view(k).state(), 9);
        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xc0);
        assert_eq!(self.sprite_slot_view(k).a(), 0);
        assert_eq!(self.sprite_slot_view(k).ai_state(), 3);
        assert_eq!(self.sprite_slot_view(k).delay_main(), 80);
        assert_eq!(self.game_state.inventory.items.quake(), 0);
        self.great_catfish_draw(k);
        assert!(!self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC));
        assert!(
            self.catfish_regurgitate_medallion_before_graphics(k),
            "Catfish source graphics entry requires a successful native medallion spawn"
        );
    }

    pub(in crate::zelda_rtl) fn complete_catfish_medallion_graphics(&mut self, k: usize) {
        self.decode_animated_sprite_tile_variable(0x1c);
        self.sprite_slot_view_mut(k)
            .set_graphics(CATFISH_BIG_FISH_CONVERSATE_GFX[80 >> 3]);
    }

    // void Sprite_Catfish_SplashOfWater(int k) {  // 9de37d
    pub(super) fn sprite_catfish_splash_of_water(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        let base = usize::from(self.sprite_slot_view(k).delay_main() >> 3) * 2;
        self.sprite_draw_multiple(
            k,
            &SPRITE_CATFISH_SPLASH_OF_WATER_DRAW_FRAMES[base..base + 2],
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_56_WalkingZora(int k) {  // 859d4a
    pub(super) fn sprite_56_walking_zora(&mut self, k: usize) {
        if self.sprite_slot_view(k).f() != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_f(value);
            let value = 3;
            self.sprite_slot_view_mut(k).set_b(value);
            let value = 192;
            self.sprite_slot_view_mut(k).set_g(value);
            let value = ((self.sprite_slot_view(k).x_recoil() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = ((self.sprite_slot_view(k).y_recoil() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }

        match self.sprite_slot_view(k).b() {
            0 => {
                self.sprite_prep_oam_coord(k);
                if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
                    return;
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_b(1);
                    self.sprite_slot_view_mut(k).or_flags3(0x40);
                }
            }
            1 => {
                self.zora_draw(k);
                if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
                    return;
                }
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).and_flags3(!0x40);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
                    self.sprite_slot_view_mut(k).add_b(1);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let dir = self.sprite_direction_to_face_link(k);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                } else {
                    let value = ZORA_SURFACING_GRAPHICS
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                let value = RECRUIT_GRAPHICS[usize::from(
                    ((self.sprite_slot_view(k).subtype2() >> 1) & 4)
                        + self.sprite_slot_view(k).direction(),
                )];
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.walking_zora_draw(k);
                if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
                    return;
                }
                // $05:9DEC JSR $05:AB93: complete two-call body.
                self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom05AB93);
                self.sprite_move_z(k, SpriteMoveZEntry::Unpriced);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot_view(k).z().wrapping_sub(1)) {
                    if sign8(self.sprite_slot_view(k).z_velocity().wrapping_add(16)) {
                        self.sprite_zero_velocity_xy(k);
                    }
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
                        let j = self.sprite_direction_to_face_link(k);
                        let value = j;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                        if (((k as u8) ^ self.game_state.frame.frame_counter) & 31) == 0 {
                            let value = j;
                            self.sprite_slot_view_mut(k).set_direction(value);
                            self.sprite_apply_speed_towards_link(k, 8);
                        }
                    }
                }
                self.sprite_move_xy(k, SpriteMoveEntry::Unpriced);
                self.sprite_check_tile_collision(k, SpriteTileCollisionEntry::Unpriced);
                if sign8(self.sprite_slot_view(k).z().wrapping_sub(1)) {
                    self.walking_zora_adjust_shadow(k);
                    if self.game_state.sprites.workspace.tile() == NativeTile::DEEP_WATER {
                        self.sprite_kill_self(k);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
                        let value = 3;
                        self.sprite_slot_view_mut(k).set_state(value);
                        let value = 15;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 3;
                        self.sprite_slot_view_mut(k).set_flags2(value);
                    }
                }
                self.sprite_slot_view_mut(k).add_subtype2(1);
            }
            3 => {
                self.sprite_check_damage_from_link(k, SpriteDamageFromEntry::Unpriced);
                if (self.game_state.frame.frame_counter & 3) == 0 {
                    let g = self.sprite_slot_view(k).g().wrapping_sub(1);
                    let value = g;
                    self.sprite_slot_view_mut(k).set_g(value);
                    if g == 0 {
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_b(value);
                        if self.sprite_slot_view(k).state() == 10 {
                            self.follower_link_state_mut().clear_state_bits();
                            self.follower_link_state_mut().clear_picking_throw_state();
                        }
                        let value = 9;
                        self.sprite_slot_view_mut(k).set_state(value);
                    }
                }
                if self.sprite_slot_view(k).g() < 48
                    && (self.game_state.frame.frame_counter & 1) == 0
                {
                    let delta = if self.game_state.frame.frame_counter & 2 != 0 {
                        -1
                    } else {
                        1
                    };
                    self.sprite_set_x(k, self.sprite_get_x(k).wrapping_add(delta as i16 as u16));
                }
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_wall_collision(value);
                self.walking_zora_draw_water_ripples(k);
                self.sprite_slot_view_mut(k).subtract_flags2(2);
                self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
                self.sprite_slot_view_mut(k).add_flags2(2);
                let value = 0;
                self.sprite_slot_view_mut(k).set_anim_clock(value);
                if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) || self.sprite_return_if_recoiling(k, SpriteRecoilEntry::Unpriced) {
                    return;
                }
                self.sprite_move_xy(k, SpriteMoveEntry::Unpriced);
                self.thrown_sprite_tile_and_sprite_interaction(k);
                self.walking_zora_adjust_shadow(k);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void WalkingZora_Draw(int k) {  // 859f08
    pub(super) fn walking_zora_draw(&mut self, k: usize) {
        self.walking_zora_draw_water_ripples(k);
        let Some((x, mut y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        if g == 0 || g == 2 {
            y = y.wrapping_sub(1);
        }
        let i = usize::from(self.sprite_slot_view(k).head_direction());
        self.set_oam_helper0_at(
            oam,
            x,
            y.wrapping_sub(6),
            WALKING_ZORA_DRAW_CHARS[i],
            flags | WALKING_ZORA_DRAW_FLAGS[i],
            2,
        );
        self.set_oam_helper0_at(
            oam + 4,
            x,
            y.wrapping_add(2),
            WALKING_ZORA_DRAW_CHARS_2[g],
            flags | WALKING_ZORA_DRAW_FLAGS_2[g],
            2,
        );
        if self.sprite_slot_view(k).anim_clock() == 0 {
            let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
            // ROM $05:9FA7
            self.sprite_draw_shadow_custom(k, &mut info, 10, SpriteShadowEntry::Rom06DC54);
        }
    }

    // -----------------------------------------------------------------------
    // void WalkingZora_DrawWaterRipples(int k) {  // 859fe0
    pub(super) fn walking_zora_draw_water_ripples(&mut self, k: usize) {
        if self.sprite_slot_view(k).anim_clock() != 0 {
            self.sprite_draw_water_ripple_with_oam_adjust(k);
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_WaterRipple(int k) {  // 859ffa
    //   static const DrawMultipleData kWaterRipple_Dmd[6] = {
    //     {0, 10, 0x01d8, 0}, {8, 10, 0x41d8, 0},
    //     {0, 10, 0x01d9, 0}, {8, 10, 0x41d9, 0},
    //     {0, 10, 0x01da, 0}, {8, 10, 0x41da, 0},
    //   };
    //   static const uint8 kWaterRipple_Idx[4] = {0, 1, 2, 1};
    //   Sprite_DrawMultiple(k, &kWaterRipple_Dmd[kWaterRipple_Idx[frame_counter >> 2 & 3] * 2], 2, NULL);
    //   OamEnt *oam = GetOamCurPtr();
    //   uint8 t = (oam[0].flags & 0x30) | 0x4;
    //   oam[0].flags = t;
    //   oam[1].flags = t | 0x40;
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_water_ripple(&mut self, k: usize) {
        let frame = self.game_state.frame.frame_counter;
        let base = (WATER_RIPPLE_FRAME_INDICES[((frame >> 2) & 3) as usize] as usize) * 2;
        self.sprite_draw_multiple(k, &WATER_RIPPLE_DRAW_FRAMES[base..base + 2]);
        let oam = self.game_state.oam.current_pointer_usize();
        let t = (self.game_state.oam.entry_flags(oam) & 0x30) | 0x4;
        self.oam_state_mut().set_entry_flags(oam, t);
        self.oam_state_mut().set_entry_flags(oam + 4, t | 0x40);
    }

    // -----------------------------------------------------------------------
    // void Sprite_CF_Swamola(int k) {  // 9d9cb0
    pub(super) fn sprite_cf_swamola(&mut self, k: usize) {
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::SwamolaHeadDraw { slot: k as u8 })
        {
            assert!((1..0x80).contains(&self.sprite_slot_view(k).ai_state()));
            self.swamola_prepare_head(k);
            return;
        }
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::SwamolaHeadDrawCompleted { slot: k as u8 })
        {
            assert!((1..0x80).contains(&self.sprite_slot_view(k).ai_state()));
            self.swamola_prepare_head(k);
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
            return;
        }
        if let Some(SpriteMainCpuBoundary::SwamolaSegmentDraw { slot, segment }) =
            self.sprite_main_cpu_boundary
        {
            if usize::from(slot) == k {
                assert!((1..0x80).contains(&self.sprite_slot_view(k).ai_state()));
                self.swamola_draw_until_segment(k, segment);
                return;
            }
        }
        if self.sprite_slot_view(k).ai_state() != 0 {
            if sign8(self.sprite_slot_view(k).ai_state()) {
                self.sprite_swamola_ripples(k);
                return;
            }
            self.swamola_draw(k);
        }
        self.sprite_cf_swamola_after_draw(k);
    }

    pub(in crate::zelda_rtl) fn sprite_cf_swamola_after_draw(&mut self, k: usize) {
        self.sprite_get16_bit_coords(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        // $1D:9CC7 JSR $1D:C211: both complete damage calls.
        self.sprite_check_damage_to_and_from_link(k, SpriteDamageBothEntry::Rom1DC211);
        let old_vel = self.sprite_slot_view(k).y_velocity();
        let value = self
            .sprite_slot_view(k)
            .y_velocity()
            .wrapping_add(self.sprite_slot_view(k).z_velocity());
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        self.sprite_move_xy(k, SpriteMoveEntry::Rom1DE95D);
        let value = old_vel;
        self.sprite_slot_view_mut(k).set_y_velocity(value);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = SPRITE_CF_SWAMOLA_TARGET_DIR[usize::from(self.get_random_number() & 7)];
                    if j == self.sprite_slot_view(k).direction() {
                        return;
                    }
                    let t = (u16::from(self.sprite_slot_view(k).b()) << 8)
                        | u16::from(self.sprite_slot_view(k).a());
                    let target_x =
                        t.wrapping_add_signed(i16::from(SWAMOLA_TARGET_X_OFFSETS[usize::from(j)]));
                    let t = (u16::from(self.sprite_slot_view(k).head_direction()) << 8)
                        | u16::from(self.sprite_slot_view(k).c());
                    let target_y =
                        t.wrapping_add_signed(i16::from(SWAMOLA_TARGET_Y_OFFSETS[usize::from(j)]));
                    self.swamola_target_mut(k).set_position(target_x, target_y);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = (-15i8) as u8;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    self.swamola_spawn_ripples(k);
                }
            }
            1 => {
                if (self.sprite_slot_view(k).subtype2() & 3) == 0 {
                    self.sprite_slot_view_mut(k).add_z_velocity(1);
                    if self.sprite_slot_view(k).z_velocity() == 0 {
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                    let pt = self.swamola_project_velocity_towards_target(k);
                    self.sprite_approach_target_speed(k, pt.x, pt.y);
                }
            }
            2 => {
                let j = usize::from(self.sprite_slot_view(k).g() & 1);
                let value = self
                    .sprite_slot_view(k)
                    .z_velocity()
                    .wrapping_add(SPRITE_CF_SWAMOLA_Z_ACCEL[j] as u8);
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                if self.sprite_slot_view(k).z_velocity() == SPRITE_CF_SWAMOLA_Z_VEL_TARGET[j] as u8
                {
                    self.sprite_slot_view_mut(k).add_g(1);
                }
                let x = self
                    .game_state
                    .effects
                    .sprite_histories
                    .swamola_target(k)
                    .x();
                let y = self
                    .game_state
                    .effects
                    .sprite_histories
                    .swamola_target(k)
                    .y();
                if self
                    .game_state
                    .sprites
                    .workspace
                    .current_sprite_x()
                    .wrapping_sub(x)
                    .wrapping_add(8)
                    < 16
                    && self
                        .game_state
                        .sprites
                        .workspace
                        .current_sprite_y()
                        .wrapping_sub(y)
                        .wrapping_add(8)
                        < 16
                {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                let pt = self.swamola_project_velocity_towards_target(k);
                let value = pt.x;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = pt.y;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
            3 => {
                if (self.sprite_slot_view(k).subtype2() & 3) == 0 {
                    self.sprite_slot_view_mut(k).add_z_velocity(1);
                    if self.sprite_slot_view(k).z_velocity() == 16 {
                        let value = 4;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.swamola_spawn_ripples(k);
                        let value = 128;
                        self.sprite_slot_view_mut(k).set_y_high(value);
                        let value = 80;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                    self.sprite_approach_target_speed(k, 0, 0);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = SPRITE_CF_SWAMOLA_TARGET_DIR[usize::from(self.get_random_number() & 7)];
                    let value = j;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let x = (u16::from(self.sprite_slot_view(k).b()) << 8)
                        | u16::from(self.sprite_slot_view(k).a());
                    self.sprite_set_x(
                        k,
                        x.wrapping_add_signed(i16::from(SWAMOLA_TARGET_X_OFFSETS[usize::from(j)])),
                    );
                    let y = (u16::from(self.sprite_slot_view(k).head_direction()) << 8)
                        | u16::from(self.sprite_slot_view(k).c());
                    self.sprite_set_y(
                        k,
                        y.wrapping_add_signed(i16::from(SWAMOLA_TARGET_Y_OFFSETS[usize::from(j)])),
                    );
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // ProjectSpeedRet Swamola_ProjectVelocityTowardsTarget(int k) {  // 9d9e13
    pub(super) fn swamola_project_velocity_towards_target(&mut self, k: usize) -> ProjectSpeedRet {
        let x = self
            .game_state
            .effects
            .sprite_histories
            .swamola_target(k)
            .x();
        let y = self
            .game_state
            .effects
            .sprite_histories
            .swamola_target(k)
            .y();
        // ROM $1D:9E2D
        self.sprite_project_speed_towards_location_long_marked(k, x, y, 15)
    }

    // -----------------------------------------------------------------------
    // void Swamola_SpawnRipples(int k) {  // 9d9eaa
    pub(super) fn swamola_spawn_ripples(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xcf, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            let value = 128;
            self.sprite_slot_view_mut(j).set_ai_state(value);
            let value = 32;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 4;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let value = 4;
            self.sprite_slot_view_mut(j).set_ignore_projectile(value);
            let value = 0;
            self.sprite_slot_view_mut(j).set_flags2(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Swamola_Ripples(int k) {  // 9d9ece
    pub(super) fn sprite_swamola_ripples(&mut self, k: usize) {
        self.swamola_ripples_draw(k);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void SwamolaRipples_Draw(int k) {  // 9d9f1d
    pub(super) fn swamola_ripples_draw(&mut self, k: usize) {
        self.oam_allocate_from_region_b(8);
        let base = usize::from((self.sprite_slot_view(k).delay_main() >> 2) & 3) * 2;
        self.sprite_draw_multiple(k, &SWAMOLA_RIPPLES_DRAW_FRAMES[base..base + 2]);
    }

    // -----------------------------------------------------------------------
    // void Swamola_Draw(int k) {  // 9d9f64
    pub(super) fn swamola_draw(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1d_9f64));
        if self.lag_free {
            // $1D:9F64..9FD4: head setup/call opcodes, history and OAM setup.
            // Child conversion/drawing bodies keep their own instruction work.
            crate::cycle_ledger::charge(1038 + if sign8(self.sprite_slot_view(k).y_velocity()) { 24 } else { 6 } + 320);
        }
        self.swamola_draw_before_segments(k);
        for segment in 0..4 {
            if self.lag_free {
                // $1D:9FD7..A039: history loads, signed OAM step, JSL only,
                // counter and BNE. Sprite/table indices stay on their pages.
                crate::cycle_ledger::charge(642 + if sign8(self.sprite_slot_view(k).y_velocity()) { 200 } else { 6 + 178 } + 194 + if segment == 3 { 0 } else { 6 });
            }
            self.swamola_prepare_segment(k, segment);
            // ROM $1D:9F88, $1D:A02D JSL $06:DBF0
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
            self.sprite_workspace_mut().set_shared_scratch_a(segment as u8 + 1);
        }
        if self.lag_free { crate::cycle_ledger::charge(42); } // $1D:A03B RTS.
    }

    fn swamola_draw_before_segments(&mut self, k: usize) {
        self.swamola_prepare_head(k);
        self.swamola_draw_head_and_publish_history(k);
    }

    pub(in crate::zelda_rtl) fn swamola_draw_after_head_checkpoint(&mut self, k: usize) {
        self.swamola_draw_head_and_publish_history(k);
        self.swamola_draw_remaining_segments(k, 0);
    }

    pub(super) fn swamola_prepare_head(&mut self, k: usize) {
        let j = usize::from(Self::sprite_convert_velocity_to_angle(
            self.sprite_slot_view(k).x_velocity(),
            self.sprite_slot_view(k)
                .y_velocity()
                .wrapping_add(self.sprite_slot_view(k).z_velocity()),
        ));
        let value = SWAMOLA_DRAW_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 63) | SWAMOLA_DRAW_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
    }

    fn swamola_draw_head_and_publish_history(&mut self, k: usize) {
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
        self.swamola_publish_history(k);
    }

    pub(in crate::zelda_rtl) fn swamola_draw_after_completed_head(&mut self, k: usize) {
        self.swamola_publish_history(k);
        self.swamola_draw_remaining_segments(k, 0);
    }

    fn swamola_publish_history(&mut self, k: usize) {
        let hist = usize::from(self.sprite_slot_view(k).subtype2() & 0x1f) + k * 32;
        let x = self.sprite_slot_view(k).x();
        let y = self.sprite_slot_view(k).y();
        self.swamola_history_mut(hist).set_position(x, y);

        let (oam_step, ext_step): (i16, i16) = if sign8(self.sprite_slot_view(k).y_velocity()) {
            (5 * 4, 5)
        } else {
            (0, 0)
        };
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add_signed(oam_step));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add_signed(ext_step));

        self.sprite_workspace_mut().set_shared_scratch_a(0);
    }

    fn swamola_prepare_segment(&mut self, k: usize, i: usize) {
        let value = SWAMOLA_DRAW_SECONDARY_GRAPHICS[i];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let j = usize::from(
            self.sprite_slot_view(k)
                .subtype2()
                .wrapping_sub(SWAMOLA_DRAW_HIST_OFFS[i])
                & 31,
        ) + k * 32;
        let x = self
            .game_state
            .effects
            .sprite_histories
            .swamola_history(j)
            .x();
        let y = self
            .game_state
            .effects
            .sprite_histories
            .swamola_history(j)
            .y();
        self.sprite_workspace_mut().set_current_sprite_x(x);
        self.sprite_workspace_mut().set_current_sprite_y(y);
        let delta: i16 = if sign8(self.sprite_slot_view(k).y_velocity()) {
            -1
        } else {
            1
        };
        let oam_step = delta * 4;
        let ext_step = delta;
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add_signed(oam_step));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add_signed(ext_step));
    }

    fn swamola_draw_remaining_segments(&mut self, k: usize, start: u8) {
        for segment in start..4 {
            self.swamola_prepare_segment(k, usize::from(segment));
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
            self.sprite_workspace_mut()
                .set_shared_scratch_a(segment + 1);
        }
    }

    pub(super) fn swamola_draw_until_segment(&mut self, k: usize, segment: u8) {
        assert!(segment < 4);
        self.swamola_draw_before_segments(k);
        for prior in 0..segment {
            self.swamola_prepare_segment(k, usize::from(prior));
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
            self.sprite_workspace_mut().set_shared_scratch_a(prior + 1);
        }
        self.swamola_prepare_segment(k, usize::from(segment));
        // PrepOamCoord cleared pause before reaching the suspended Y calculation.
        self.sprite_slot_view_mut(k).set_pause(0);
    }

    pub(in crate::zelda_rtl) fn swamola_draw_after_segment_checkpoint(&mut self, k: usize, segment: u8) {
        assert!(segment < 4);
        self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Unpriced);
        self.sprite_workspace_mut()
            .set_shared_scratch_a(segment + 1);
        self.swamola_draw_remaining_segments(k, segment + 1);
    }

    // -----------------------------------------------------------------------
    // void Kyameron_Draw(int k) {  // 9ea158
    pub(in crate::zelda_rtl) fn kyameron_draw(&mut self, k: usize) {
        let j = self.sprite_slot_view(k).graphics();
        if j < 12 {
            let bak = self.sprite_slot_view(k).oam_flags();
            let value =
                (self.sprite_slot_view(k).oam_flags() & 0x3f) | KYAMERON_DRAW_FLAGS[usize::from(j)];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            // ROM $1E:A16E JSL $06:DBF0
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
            let value = bak;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        } else {
            let base = (usize::from(j) - 12) * 4;
            self.sprite_draw_multiple(k, &KYAMERON_DRAW_FRAMES[base..base + 4]);
        }
    }

    // -----------------------------------------------------------------------
    // void Pirogusu_Draw(int k) {  // 9ea93b
    pub(in crate::zelda_rtl) fn pirogusu_draw(&mut self, k: usize) {
        let j = usize::from(self.sprite_slot_view(k).a());
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | PIROGUSU_DRAW_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = PIROGUSU_DRAW_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        if j < 4 {
            let x = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_add(4);
            let y = self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_add(4);
            self.sprite_workspace_mut().set_current_sprite_x(x);
            self.sprite_workspace_mut().set_current_sprite_y(y);
            // ROM $1E:A975
            self.sprite_draw_single_small(k, SpriteSingleSmallEntry::Rom06DBF8);
        } else {
            // ROM $1E:A97A JSL $06:DBF0
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
        }
    }

    // -----------------------------------------------------------------------
    // void GreatCatfish_Draw(int k) {  // 9de320
    pub(super) fn great_catfish_draw(&mut self, k: usize) {
        let g = self.sprite_slot_view(k).graphics();
        if g != 0 {
            if g == CATFISH_BIG_FISH_CONVERSATE_GFX[20] {
                // The carry-dependent table overread selects the native
                // four records at $1D:F9C0 on the following draw.
                self.sprite_draw_multiple_from_encoded_records(
                    k,
                    &GREAT_CATFISH_SPLASH_CARRY_DRAW,
                );
                return;
            }
            let base = usize::from(g - 1) * 4;
            self.sprite_draw_multiple(k, &GREAT_CATFISH_DRAW_FRAMES[base..base + 4]);
        }
    }
}
