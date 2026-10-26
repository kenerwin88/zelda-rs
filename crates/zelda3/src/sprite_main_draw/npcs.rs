//! Townsfolk, quest NPCs, dialogue actors, and cutscene sprites.

use super::*;
use crate::zelda_rtl::sprite::PrepOamCoordsRet;
use crate::types::sign8;

impl ZeldaState {

    // -----------------------------------------------------------------------
    // uint8 FluteBoy_Draw(int k) {  // 8dcfd9
    //   Four-tile body draw; returns whether the prepped coordinate was offscreen.
    // }
    pub(in crate::zelda_rtl) fn flute_boy_draw(&mut self, k: usize) -> u8 {
        self.oam_allocate_from_region_b(0x10);
        let base = (self.sprite_slot_view(k).direction() as usize) * 8
            + (self.sprite_slot_view(k).graphics() as usize) * 4;
        let info = self.sprite_draw_multiple(k, &FLUTE_BOY_DRAW_FRAMES[base..base + 4]);
        ((info.x | info.y) >> 8) as u8
    }

    // -----------------------------------------------------------------------
    // void FluteAardvark_Draw(int k) {  // 8dd040
    //   Two deferred large tiles selected by sprite_graphics.
    // }
    pub(in crate::zelda_rtl) fn flute_aardvark_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &FLUTE_AARDVARK_DRAW_FRAMES[base..base + 2],
        );
    }

    // -----------------------------------------------------------------------
    // void BugNetKid_Draw(int k) {  // 8dd47b
    //   Six deferred tiles selected by sprite_graphics.
    // }
    pub(super) fn bug_net_kid_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 6;
        self.sprite_draw_multiple_player_deferred(
            k,
            &BUG_NET_KID_DRAW_FRAMES[base..base + 6],
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_Aginah(int k) {  // 85f0ea
    pub(super) fn sprite_aginah(&mut self, k: usize) {
        let msg = if (self.game_state.inventory.save_progress.progress_flags() & 0x20) == 0 {
            self.save_progress_mut().or_progress_flags(0x20);
            0x125
        } else if self.game_state.inventory.items.sword_type() >= 2 {
            0x128
        } else if (self.game_state.inventory.player_resources.pendant_flags() & 7) == 7 {
            0x126
        } else if (self.game_state.inventory.player_resources.pendant_flags() & 2) != 0 {
            0x129
        } else if self.game_state.inventory.items.book() != 0 {
            0x127
        } else {
            self.save_progress_mut().or_progress_flags(0x20);
            0x125
        };
        self.sprite_show_solicited_message(k, msg);
        let value = (self.game_state.frame.frame_counter >> 5) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Sahasrahla(int k) {  // 85f14d
    pub(super) fn sprite_sahasrahla(&mut self, k: usize) {
        match self.sprite_slot_view(k).ai_state() {
            0 => self.sasha_idle(k),
            1 => {
                self.sprite_show_message_unconditional(0x33);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                self.save_progress_mut().set_map_icons_indicator(3);
            }
            2 => {
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x4b,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::SahasrahlaBoots,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_sahasrahla_boots_item_receipt(k);
            }
            3 => {
                self.sprite_show_message_unconditional(0x37);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            _ => {}
        }
    }

    /// Source suffix after Sprite_Sahasrahla's boots handout
    /// `Link_ReceiveItem(0x4b)` call (ROM `$85f14d`). A live timing authority
    /// suspends the decompressor there; only the AI-state advance and the
    /// map-icon indicator remain.
    pub(in crate::zelda_rtl) fn complete_sahasrahla_boots_item_receipt(&mut self, k: usize) {
        let value = 3;
        self.sprite_slot_view_mut(k).set_ai_state(value);
        self.save_progress_mut().set_map_icons_indicator(3);
    }

    // -----------------------------------------------------------------------
    // void Sprite_16_Elder_bounce(int k) {  // 86c08a
    //   Dispatcher for Sahasrahla / Aginah variants.
    // }
    pub(super) fn sprite_16_elder_bounce(&mut self, k: usize) {
        self.elder_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_sahasrahla(k),
            1 => self.sprite_aginah(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Waterfall(int k) {  // 9af5b8
    pub(super) fn waterfall(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
            if self.game_state.world.location.overworld_screen_index() == 0x43 {
                self.ancilla_add_gt_cutscene();
            } else {
                self.ancilla_add_waterfall_splash();
            }
        }
    }

    /// `Waterfall` on screen $43 up to `AncillaAdd_GTCutscene`'s graphics
    /// decode: the cutscene ancilla was added, every waterfall sprite (this
    /// one included) killed and the sparkle phases filled.
    pub(in crate::zelda_rtl) fn waterfall_before_gt_cutscene_graphics(&mut self, k: usize) {
        assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x37);
        assert_eq!(self.sprite_slot_view(k).subtype2(), 0);
        assert!(!self.sprite_return_if_inactive(k));
        assert!(
            self.sprite_check_damage_to_link_same_layer(k),
            "GT cutscene graphics entry requires Link touching the waterfall"
        );
        assert_eq!(
            self.game_state.world.location.overworld_screen_index(),
            0x43
        );
        assert!(
            self.ancilla_add_gt_cutscene_before_graphics().is_some(),
            "GT cutscene graphics entry requires the cutscene ancilla"
        );
    }

    pub(in crate::zelda_rtl) fn complete_waterfall_gt_cutscene_graphics(&mut self, _k: usize) {
        self.decode_animated_sprite_tile_variable(0x28);
        let ancilla = (0..5)
            .find(|&i| self.ancilla_slot_view(i).ancilla_type() == 0x43)
            .expect("GT cutscene graphics completion requires its cutscene ancilla");
        self.ancilla_add_gt_cutscene_after_graphics(ancilla);
    }

    // -----------------------------------------------------------------------
    // void Sprite_37_Waterfall(int k) {  // 86c03a
    pub(super) fn sprite_37_waterfall(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.waterfall(k),
            1 => self.sprite_bat_crash(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BatCrash(int k) {  // 9af5d9
    pub(super) fn sprite_bat_crash(&mut self, k: usize) {
        self.retreat_bat_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.bat_crash_draw_hardcoded_garbage(k);
        self.set_bg1_y_offset(0);
        if self.sprite_slot_view(k).delay_aux3() != 0 {
            if self.sprite_slot_view(k).delay_aux3() == 1 {
                self.set_ambient_sound_effect(5);
            }
            let offs = if (self.sprite_slot_view(k).delay_aux3() & 1) != 0 {
                1u16
            } else {
                (-1i16) as u16
            };
            self.set_bg1_y_offset(offs);
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = self.sprite_slot_view(k).graphics().wrapping_add(1) & 3;
            self.sprite_slot_view_mut(k).set_graphics(value);
            if self.sprite_slot_view(k).graphics() == 0 && self.sprite_slot_view(k).ai_state() < 2 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 3);
            }
            let value = SPRITE_BAT_CRASH_DELAY[usize::from(self.sprite_slot_view(k).direction())];
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let j = usize::from(self.sprite_slot_view(k).a());
                if SPRITE_BAT_CRASH_X_POSITIONS[j]
                    < self.game_state.sprites.workspace.current_sprite_x()
                {
                    if j >= 2 {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 208;
                        self.sprite_slot_view_mut(k).set_delay_aux1(value);
                    }
                    self.sprite_slot_view_mut(k).add_a(1);
                    self.sprite_slot_view_mut(k).add_direction(1);
                }
                self.retreat_bat_update_pos(k, j, &SPRITE_BAT_CRASH_Y_POSITIONS);
            }
            1 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
                    self.sprite_slot_view_mut(k).add_direction(1);
                    let value = 232;
                    self.sprite_slot_view_mut(k).set_x_low(value);
                    let value = 7;
                    self.sprite_slot_view_mut(k).set_x_high(value);
                    let value = 224;
                    self.sprite_slot_view_mut(k).set_y_low(value);
                    let value = 5;
                    self.sprite_slot_view_mut(k).set_y_high(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 64;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 45;
                    self.sprite_slot_view_mut(k).set_delay_aux1(value);
                } else {
                    if (self.game_state.frame.frame_counter & 3) == 0 {
                        self.sprite_slot_view_mut(k).subtract_x_velocity(1);
                    }
                    let j = usize::from(self.sprite_slot_view(k).a());
                    self.retreat_bat_update_pos(k, j, &SPRITE_BAT_CRASH_Y_POSITIONS);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 96;
                    self.sprite_slot_view_mut(k).set_delay_aux1(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
                if self.sprite_slot_view(k).delay_aux1() == 9 {
                    self.bat_crash_spawn_debris(k);
                    self.CreatePyramidHole();
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                    self.increment_overworld_map_state();
                }
            }
            _ => {}
        }
    }

    fn retreat_bat_update_pos(&mut self, k: usize, j: usize, ypos: &[u16; 4]) {
        if (self.game_state.frame.frame_counter & 7) == 0 {
            let delta = if ypos[j] >= self.game_state.sprites.workspace.current_sprite_y() {
                1u8
            } else {
                (-1i8) as u8
            };
            self.sprite_slot_view_mut(k).add_y_velocity(delta);
        }
        if (self.game_state.frame.frame_counter & 15) == 0 {
            self.sprite_slot_view_mut(k).add_x_velocity(1);
        }
    }

    // -----------------------------------------------------------------------
    // void BatCrash_SpawnDebris(int k) {  // 9af7e5
    pub(super) fn bat_crash_spawn_debris(&mut self, k: usize) {
        for j in (0..30).rev() {
            self.garnish_spawn_pyramid_debris(
                BAT_CRASH_SPAWN_DEBRIS_X_OFFSETS[j],
                BAT_CRASH_SPAWN_DEBRIS_Y_OFFSETS[j],
                BAT_CRASH_SPAWN_DEBRIS_LOCAL_X_VELOCITIES[j],
                BAT_CRASH_SPAWN_DEBRIS_LOCAL_Y_VELOCITIES[j],
            );
        }
        let value = 32;
        self.sprite_slot_view_mut(k).set_delay_aux3(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_ChickenLady(int k) {  // 9afed3
    pub(super) fn sprite_chicken_lady(&mut self, k: usize) {
        let value = 1;
        self.sprite_slot_view_mut(k).set_direction(value);
        self.lady_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 1 {
            self.dialogue_message_index_mut().set_value(0x17d);
            self.sprite_show_message_minimal_c();
        }
        let value = (self.game_state.frame.frame_counter >> 4) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Bully_Draw(int k) {  // 9eed9e
    //   Sprite_DrawMultiplePlayerDeferred(k, &kBully_Dmd[sprite_D[k] * 4 +
    //                                     sprite_graphics[k] * 2], 2, &info);
    //   SpriteDraw_Shadow(k, &info);
    // }
    pub(super) fn bully_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).direction()) * 4
            + usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &BULLY_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_B9_BullyAndPinkBall(int k) {  // 9eeb33
    pub(super) fn sprite_b9_bully_and_pink_ball(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_pink_ball(k),
            1 => self.pink_ball_distress(k),
            2 => self.sprite_bully(k),
            // C Sprite_B9_BullyAndPinkBall asserts for all other subtypes.
            _ => panic!("Sprite_B9_BullyAndPinkBall invalid subtype"),
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_B6_Kiki(int k) {  // 9ee2ef
    pub(super) fn sprite_b6_kiki(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.kiki_lying_inwait(k),
            1 => self.kiki_offer_entrance_service(k),
            2 => self.kiki_offer_initial_service(k),
            3 => self.kiki_flee(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_B7_BlindMaiden(int k) {  // 9ee8b6
    pub(super) fn sprite_b7_blind_maiden(&mut self, k: usize) {
        if !self.sprite_b7_blind_maiden_before_follower_graphics(k) {
            return;
        }
        self.load_follower_graphics();
        self.sprite_b7_blind_maiden_after_follower_graphics(k);
    }

    pub(in crate::zelda_rtl) fn sprite_b7_blind_maiden_before_follower_graphics(&mut self, k: usize) -> bool {
        self.crystal_maiden_draw(k);
        if self.sprite_return_if_inactive(k) {
            return false;
        }
        self.sprite_track_body_to_head(k);
        let value = self.sprite_direction_to_face_link(k) ^ 3;
        self.sprite_slot_view_mut(k).set_head_direction(value);
        if self.sprite_slot_view(k).ai_state() == 0 {
            if self.sprite_show_message_on_contact(k, 0x0122) & 0x100 != 0 {
                let value = 1;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            false
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.follower_state_mut().set_indicator(6);
            true
        }
    }

    pub(in crate::zelda_rtl) fn sprite_b7_blind_maiden_after_follower_graphics(&mut self, k: usize) {
        self.sprite_become_follower(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_B8_DialogueTester(int k) {  // 9eeae7
    pub(super) fn sprite_b8_dialogue_tester(&mut self, _k: usize) {
        unreachable!("C Sprite_B8_DialogueTester is an assert(0) sprite slot");
    }

    // -----------------------------------------------------------------------
    // void Sprite_PinkBall(int k) {  // 9eeb40
    pub(super) fn sprite_pink_ball(&mut self, k: usize) {
        self.oam_allocate_defer_to_player(k);
        // ROM $1E:EB44 JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.pink_ball_handle_message(k);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x80)
            | self.sprite_slot_view(k).head_direction();
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_move_xyz(k);
        // ROM $1E:EB5C -> $1E:8B2E -> $06:E496
        let t = self.sprite_check_tile_collision_long(k);
        if t != 0 {
            if (t & 3) == 0 {
                let value = self.sprite_slot_view(k).y_velocity().wrapping_neg();
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                if self.sprite_slot_view(k).e() != 0 {
                    self.ball_guy_play_bounce_noise(k);
                }
            } else {
                let value = self.sprite_slot_view(k).x_velocity().wrapping_neg();
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                if self.sprite_slot_view(k).e() != 0 {
                    self.ball_guy_play_bounce_noise(k);
                }
            }
        }

        self.sprite_slot_view_mut(k).subtract_z_velocity(1);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = self.sprite_slot_view(k).z_velocity().wrapping_neg() >> 2;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            if (self.sprite_slot_view(k).z_velocity() & 0xfc) != 0 {
                self.ball_guy_play_bounce_noise(k);
            }
            self.pink_ball_handle_deceleration(k);
        }
        if self.sprite_slot_view(k).e() == 0 {
            if self.sprite_slot_view(k).head_direction() == 0 {
                self.pink_ball_distress(k);
                let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 3) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x3f) == 0 {
                    let x = (self.game_state.player.follower_link.x() & 0xff00)
                        | u16::from(self.get_random_number());
                    let y = (self.game_state.player.follower_link.y() & 0xff00)
                        | u16::from(self.get_random_number());
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 8);
                    let value = pt.x;
                    self.sprite_slot_view_mut(k).set_b(value);
                    let value = pt.y;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if pt.y != 0 {
                        self.sprite_slot_view_mut(k).or_oam_flags(0x40);
                        let value = (self.sprite_slot_view(k).x_velocity() >> 1) & 0x40;
                        self.sprite_slot_view_mut(k).xor_oam_flags(value);
                    }
                }
                let value = self.sprite_slot_view(k).b();
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = self.sprite_slot_view(k).a();
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            } else {
                self.pink_ball_distress(k);
                if ((k as u8) ^ self.game_state.frame.frame_counter) != 0 {
                    let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 2) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                } else {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
            }
        } else if (self.sprite_slot_view(k).x_velocity() | self.sprite_slot_view(k).y_velocity())
            == 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_e(value);
        } else {
            let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 2) & 1;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let value = (((k as u8) ^ self.game_state.frame.frame_counter) << 2) & 0x80;
            self.sprite_slot_view_mut(k).set_head_direction(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Bully(int k) {  // 9eec7c
    pub(super) fn sprite_bully(&mut self, k: usize) {
        self.bully_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.bully_handle_message(k);
        self.sprite_move_xyz(k);
        // ROM $1E:EC88 -> $1E:8B2E -> $06:E496
        let t = self.sprite_check_tile_collision_long(k);
        if t != 0 {
            if (t & 3) == 0 {
                let value = self.sprite_slot_view(k).y_velocity().wrapping_neg();
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            } else {
                let value = self.sprite_slot_view(k).x_velocity().wrapping_neg();
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 3) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let j = usize::from(self.sprite_slot_view(k).head_direction());
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x1f) == 0 {
                    let pt = self.sprite_project_speed_towards_location(
                        k,
                        self.sprite_get_x(j),
                        self.sprite_get_y(j),
                        14,
                    );
                    let value = pt.y;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = pt.x;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    if pt.x != 0 {
                        let value = self.sprite_slot_view(k).x_velocity() >> 7;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                }
                if self.sprite_slot_view(j).z() == 0
                    && self
                        .sprite_slot_view(k)
                        .x_low()
                        .wrapping_sub(self.sprite_slot_view(j).x_low())
                        .wrapping_add(8)
                        < 16
                    && self
                        .sprite_slot_view(k)
                        .y_low()
                        .wrapping_sub(self.sprite_slot_view(j).y_low())
                        .wrapping_add(8)
                        < 16
                {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    self.ball_guy_play_bounce_noise(k);
                }
            }
            1 => {
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let j = usize::from(self.sprite_slot_view(k).head_direction());
                let value = self.sprite_slot_view(k).x_velocity() << 1;
                self.sprite_slot_view_mut(j).set_x_velocity(value);
                let value = self.sprite_slot_view(k).y_velocity() << 1;
                self.sprite_slot_view_mut(j).set_y_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = self.get_random_number() & 31;
                self.sprite_slot_view_mut(j).set_z_velocity(value);
                let value = 96;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let value = 1;
                self.sprite_slot_view_mut(j).set_e(value);
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Lady_Draw(int k) {  // 9af92c
    pub(super) fn lady_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &LADY_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void YoungSnitchLady_Draw(int k) {  // 85e37f
    pub(super) fn young_snitch_lady_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &YOUNG_SNITCH_LADY_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_YoungSnitchLady(int k) {  // 85e2f2
    pub(super) fn sprite_young_snitch_lady(&mut self, k: usize) {
        self.sprite_old_snitch_lady(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_OldSnitchLady(int k) {  // 85e6aa
    pub(super) fn sprite_old_snitch_lady(&mut self, k: usize) {
        if self.sprite_slot_view(k).sprite_type() == 0x34 {
            if self.sprite_slot_view(k).ai_state() < 2 {
                self.young_snitch_lady_draw(k);
            }
        } else {
            if self.sprite_slot_view(k).subtype() != 0 {
                self.sprite_chicken_lady(k);
                return;
            }
            if self.sprite_slot_view(k).ai_state() < 3 {
                self.lady_draw(k);
            }
        }

        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).ai_state() < 3 {
            if self.game_state.world.location.is_indoors() {
                self.sprite_track_body_to_head(k);
                let value = self.sprite_direction_to_face_link(k) ^ 3;
                self.sprite_slot_view_mut(k).set_head_direction(value);
                self.sprite_show_solicited_message(k, 0x00ad);
                return;
            }
            if self.sprite_slot_view(k).ai_state() == 0
                && self.sprite_check_damage_to_link_same_layer(k)
            {
                let value = self.sprite_direction_to_face_link(k) ^ 3;
                self.sprite_slot_view_mut(k).set_direction(value);
                let value = 1;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            } else if self.sprite_track_body_to_head(k) {
                // ROM $05:E6F4
                self.sprite_move_xy_marked(k, 0x05fa00);
            } else {
                let value = 1;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let home_x = u16::from(self.sprite_slot_view(k).a())
                        | (u16::from(self.sprite_slot_view(k).b()) << 8);
                    let idx = usize::from(self.sprite_slot_view(k).c() & 1);
                    let t = home_x.wrapping_add_signed(i16::from(SPRITE_OLD_SNITCH_LADY_XD[idx]));
                    if t == self.sprite_get_x(k) {
                        let j = self.sprite_slot_view(k).direction() ^ 1;
                        let value = j;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                        let value = SPRITE_OLD_SNITCH_LADY_LOCAL_X_VELOCITIES[usize::from(j)] as u8;
                        self.sprite_slot_view_mut(k).set_x_velocity(value);
                        let value = SPRITE_OLD_SNITCH_LADY_LOCAL_Y_VELOCITIES[usize::from(j)] as u8;
                        self.sprite_slot_view_mut(k).set_y_velocity(value);
                        self.sprite_slot_view_mut(k).xor_c(1);
                    }
                }
                let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 4) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let bak0 = self.sprite_slot_view(k).flags4();
                let value = 3;
                self.sprite_slot_view_mut(k).set_flags4(value);
                let j = self.sprite_show_message_on_contact(k, 0x002f);
                let value = bak0;
                self.sprite_slot_view_mut(k).set_flags4(value);
                if (j & 0x0100) != 0 {
                    let value = j as u8;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    self.snitch_spawn_guard(k);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                let idx = usize::from(
                    self.game_state
                        .sprites
                        .garnish_runtime
                        .active_overlord_index(),
                );
                let overlord = self.overlord_slot_view(idx);
                let ovx = overlord.x();
                let ovy = overlord.y();
                if ovy >= self.sprite_get_y(k) {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_flags4(value);
                    let base_y = self.game_state.world.scroll.overworld_offset_base_y();
                    let mask_y = self.game_state.world.scroll.overworld_offset_mask_y();
                    let base_x = self.game_state.world.scroll.overworld_offset_base_x();
                    let mask_x = self.game_state.world.scroll.overworld_offset_mask_x();
                    let pos = ((ovy.wrapping_sub(base_y) & mask_y) * 8)
                        .wrapping_add((ovx >> 3).wrapping_sub(base_x) & mask_x);
                    self.overworld_draw_wooden_door(pos, false);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    self.follower_link_state_mut().immobilize();
                    let pt = self.sprite_project_speed_towards_location(k, ovx, ovy, 64);
                    let value = pt.x;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = pt.y;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 3) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let idx = usize::from(
                        self.game_state
                            .sprites
                            .garnish_runtime
                            .active_overlord_index(),
                    );
                    let overlord = self.overlord_slot_view(idx);
                    let ovx = overlord.x();
                    let ovy = overlord.y();
                    self.sprite_set_x(k, ovx);
                    self.sprite_set_y(k, ovy);
                    let base_y = self.game_state.world.scroll.overworld_offset_base_y();
                    let mask_y = self.game_state.world.scroll.overworld_offset_mask_y();
                    let base_x = self.game_state.world.scroll.overworld_offset_base_x();
                    let mask_x = self.game_state.world.scroll.overworld_offset_mask_x();
                    let pos = ((ovy.wrapping_sub(base_y) & mask_y) * 8)
                        .wrapping_add((ovx >> 3).wrapping_sub(base_x) & mask_x);
                    self.overworld_draw_wooden_door(pos, true);
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                // ROM $05:E6F4
                self.sprite_move_xy_marked(k, 0x05fa00);
            }
            3 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.follower_link_state_mut().clear_immobilized();
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Hobo_Draw(int k) {  // 84ea60
    pub(super) fn hobo_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple_player_deferred(k, &HOBO_DRAW_FRAMES[base..base + 4]);
    }

    // -----------------------------------------------------------------------
    // void Sprite_2B_Hobo(int k) {  // 86bdc1
    pub(super) fn sprite_2_b_hobo(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_hobo_bum(k),
            1 => self.sprite_hobo_bubble(k),
            2 => self.sprite_hobo_fire(k),
            3 => self.sprite_hobo_smoke(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hobo_Bum(int k) {  // 86bdd0
    pub(super) fn sprite_hobo_bum(&mut self, k: usize) {
        self.hobo_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = 3;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 7;
                self.sprite_slot_view_mut(k).set_flags4(value);
                if self.sprite_check_damage_to_link_same_layer(k)
                    && (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let j = usize::from(self.sprite_slot_view(k).e());
                    let value = 4;
                    self.sprite_slot_view_mut(j).set_delay_main(value);
                    self.follower_link_state_mut().immobilize();
                }
                if self.sprite_slot_view(k).delay_aux2() == 0 {
                    let value = 160;
                    self.sprite_slot_view_mut(k).set_delay_aux2(value);
                    let j = self.hobo_spawn_bubble(k);
                    let value = j as u8;
                    self.sprite_slot_view_mut(k).set_e(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = usize::from(self.sprite_slot_view(k).a());
                    if j != 7 {
                        let value = SPRITE_HOBO_BUM_LOCAL_GRAPHICS[j];
                        self.sprite_slot_view_mut(k).set_graphics(value);
                        let value = SPRITE_HOBO_BUM_DELAY[j];
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        self.sprite_slot_view_mut(k).add_a(1);
                    } else {
                        self.sprite_show_message_unconditional(0xd7);
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                }
            }
            2 => {
                let value = 3;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let screen = usize::from(self.game_state.world.location.overworld_screen_index());
                self.set_overworld_event_bits(screen, 0x20);
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x16,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::HoboBottle,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_hobo_bottle_item_receipt(k);
            }
            3 => {
                self.follower_link_state_mut().clear_immobilized();
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 160;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.hobo_spawn_bubble(k);
                }
            }
            _ => {}
        }
    }

    /// Source suffix after Sprite_Hobo_Bum's `Link_ReceiveItem(0x16, 0)`
    /// (ROM `$86bdd0` case 2): only the progress-indicator bit remains.
    pub(in crate::zelda_rtl) fn complete_hobo_bottle_item_receipt(&mut self, _k: usize) {
        self.save_progress_mut().or_progress_indicator_3(1);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hobo_Bubble(int k) {  // 86beb4
    pub(super) fn sprite_hobo_bubble(&mut self, k: usize) {
        self.oam_allocate_from_region_c(4);
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = ((self.game_state.frame.frame_counter >> 4) & 1) + 2;
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_slot_view(k).delay_aux1() == 0 {
            self.sprite_slot_view_mut(k).add_graphics(1);
            self.sprite_move_z(k);
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        }
        if self.sprite_slot_view(k).delay_main() < 4 {
            let value = 3;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hobo_Fire(int k) {  // 86bf15
    pub(super) fn sprite_hobo_fire(&mut self, k: usize) {
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = (self.game_state.frame.frame_counter >> 3) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.sprite_slot_view_mut(k).and_oam_flags(!0x40);
        if self.sprite_slot_view(k).delay_main() == 0 {
            self.hobo_spawn_smoke(k);
            let value = 47;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hobo_Smoke(int k) {  // 86bf81
    pub(super) fn sprite_hobo_smoke(&mut self, k: usize) {
        let value = 6;

        self.sprite_slot_view_mut(k).set_graphics(value);
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $06:BF8C
        self.sprite_move_xy_marked(k, 0x06e93e);
        self.sprite_move_z(k);
        let flags = SPRITE_HOBO_SMOKE_OAM_FLAGS
            [usize::from((self.game_state.frame.frame_counter >> 4) & 3)];
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | flags;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_RunningMan(int k) {  // 85e8b2
    pub(super) fn sprite_running_man(&mut self, k: usize) {
        self.running_man_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_track_body_to_head(k);
        self.sprite_behave_as_barrier(k);
        let value = 255;
        self.sprite_slot_view_mut(k).set_subtype(value);
        // ROM $05:E8C5 -> $05:B88D -> $06:E496
        self.sprite_check_tile_collision_long(k);
        let bak0 = self.sprite_slot_view(k).flags4();
        let value = 7;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.sprite_check_damage_to_link_same_layer(k) {
            let value = self.sprite_slot_view(k).ai_state();
            self.sprite_slot_view_mut(k).set_c(value);
            let value = 3;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
        let value = bak0;
        self.sprite_slot_view_mut(k).set_flags4(value);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_track_body_to_head(k);
                let j = self.sprite_direction_to_face_link(k);
                let value = j ^ 3;
                self.sprite_slot_view_mut(k).set_head_direction(value);
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.link_cancel_dash();
                    let value = j ^ 3;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = j | 2;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = (j & 1) + 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = RUNNING_MAN_RECOIL_X_VELOCITIES[usize::from(j & 1)] as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            }
            1 | 2 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    let value = (self.game_state.frame.frame_counter >> 3) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    self.sprite_move_xy(k);
                } else {
                    self.running_boy_spawn_dust_garnish(k);
                    let value = (self.game_state.frame.frame_counter >> 2) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let j = usize::from(self.sprite_slot_view(k).head_direction());
                    let value = SPRITE_RUNNING_MAN_LOCAL_X_VELOCITIES[j] as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = SPRITE_RUNNING_MAN_LOCAL_Y_VELOCITIES[j] as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    self.sprite_move_xy(k);
                    if self.sprite_slot_view(k).a() != 0 {
                        self.sprite_slot_view_mut(k).subtract_a(1);
                    } else if self.sprite_slot_view(k).ai_state() == 1 {
                        let value = 255;
                        self.sprite_slot_view_mut(k).set_a(value);
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                    } else {
                        let j = usize::from(self.sprite_slot_view(k).b());
                        self.sprite_slot_view_mut(k).add_b(1);
                        let value = SPRITE_RUNNING_MAN_A[j];
                        self.sprite_slot_view_mut(k).set_a(value);
                        if sign8(SPRITE_RUNNING_MAN_DIRECTIONS[j] as u8) {
                            let value = 0;
                            self.sprite_slot_view_mut(k).set_ai_state(value);
                            let value = 0;
                            self.sprite_slot_view_mut(k).set_subtype2(value);
                        } else {
                            let value = SPRITE_RUNNING_MAN_DIRECTIONS[j] as u8;
                            self.sprite_slot_view_mut(k).set_head_direction(value);
                        }
                    }
                }
            }
            3 => {
                self.sprite_show_message_unconditional(0x00a6);
                if self.game_state.player.follower_link.handler_state()
                    >= SPRITE_RUNNING_MAN_PLAYER_STATE_RECOIL_WALL_LOCAL
                {
                    let value = self.game_state.player.follower_link.handler_state();
                    self.sprite_slot_view_mut(k).set_direction(value);
                }
                let value = self.sprite_slot_view(k).c();
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void RunningMan_Draw(int k) {  // 85ea4d
    pub(super) fn running_man_draw(&mut self, k: usize) {
        let base = (usize::from(self.sprite_slot_view(k).direction()) * 4
            + usize::from(self.sprite_slot_view(k).graphics()) * 2)
            & 0xf;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &RUNNING_MAN_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Elder_Draw(int k) {  // 85f23a
    pub(super) fn elder_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &ELDER_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Shopkeeper_Draw(int k) {  // 85f91b
    pub(super) fn shopkeeper_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &SHOPKEEPER_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void FluteBoyFather_Draw(int k) {  // 8dc3e1
    pub(super) fn flute_boy_father_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &FLUTE_BOY_FATHER_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void FluteBoyOstrich_Draw(int k) {  // 9e9a4b
    pub(in crate::zelda_rtl) fn flute_boy_ostrich_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        let mut info = self.sprite_draw_multiple(
            k,
            &FLUTE_BOY_OSTRICH_DRAW_FRAMES[base..base + 4],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 18);
    }

    // -----------------------------------------------------------------------
    // void OldMountainMan_Draw(int k) {  // 9dff0e
    pub(in crate::zelda_rtl) fn old_mountain_man_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).subtype2() != 2 {
            let j = usize::from(self.sprite_slot_view(k).direction()) * 4
                + usize::from(self.sprite_slot_view(k).graphics()) * 2;
            self.set_sprite_dma_head_pointer(OLD_MOUNTAIN_MAN_DRAW_DMA[j]);
            self.set_sprite_dma_body_pointer(OLD_MOUNTAIN_MAN_DRAW_DMA[j + 1]);
            self.sprite_draw_multiple_player_deferred(
                k,
                &OLD_MOUNTAIN_MAN_DRAW_FRAMES_1[j..j + 2],
            );
        } else {
            self.sprite_draw_multiple_player_deferred(k, &OLD_MOUNTAIN_MAN_DRAW_FRAMES_0);
        }
    }

    // -----------------------------------------------------------------------
    // void InnKeeper_Draw(int k) {  // 85e3dc
    pub(super) fn inn_keeper_draw(&mut self, k: usize) {
        let mut info = self.sprite_draw_multiple_player_deferred(k, &INN_KEEPER_DRAW_FRAMES);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_InnKeeper(int k) {  // 85e3af
    pub(super) fn sprite_inn_keeper(&mut self, k: usize) {
        self.inn_keeper_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_show_solicited_message(
            k,
            if self.game_state.inventory.items.flippers() != 0 {
                0x0183
            } else {
                0x0182
            },
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_Witch(int k) {  // 85e3fb
    pub(super) fn sprite_witch(&mut self, k: usize) {
        self.witch_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let bak0 = self.sprite_slot_view(k).flags4();
        let value = 2;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
        }
        let value = bak0;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.game_state.frame.frame_counter == 0 {
            let value = (self.get_random_number() & 1) + 2;
            self.sprite_slot_view_mut(k).set_a(value);
        }
        let shift = self.sprite_slot_view(k).a().wrapping_add(1);
        let value = (self.game_state.frame.frame_counter >> shift) & 7;
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => match self.game_state.inventory.items.mushroom() {
                0 => {
                    if self.saved_room_flags(0x109) & 0x80 != 0 {
                        self.sprite_show_solicited_message(k, 0x004b);
                    } else {
                        self.sprite_show_solicited_message(k, 0x004a);
                    }
                }
                1 => {
                    if self.game_state.player.follower_link.joypad1h_last() & 0x40 == 0 {
                        self.sprite_show_solicited_message(k, 0x004c);
                    } else if self.sprite_check_damage_to_link_same_layer(k)
                        && self.game_state.inventory.save_progress.hud_current_item()
                            == HUD_ITEM_MUSHROOM_DRAW
                    {
                        self.witch_accept_shroom(k);
                    }
                }
                _ => {
                    self.sprite_show_solicited_message(k, 0x004a);
                }
            },
            1 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                self.follower_link_state_mut().set_item_receipt_method(0);
                self.link_receive_item(0x18, 0);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Witch_AcceptShroom(int k) {  // 85e4cf
    pub(super) fn witch_accept_shroom(&mut self, k: usize) {
        self.inventory_items_mut().set_mushroom(0);
        let dung_info = self.saved_room_flags(0x109) | 0x80;
        self.set_saved_room_flags(0x109, dung_info);
        self.set_sound_effect_1(0);
        self.hud_refresh_icon();
        self.sprite_show_message_unconditional(0x004b);
        self.sprite_sfx_queue_sfx1_with_pan(k, 0x0d);
        self.clear_flag_overworld_area_changed();
    }

    // -----------------------------------------------------------------------
    // void Witch_Draw(int k) {  // 85e55d
    pub(super) fn witch_draw(&mut self, k: usize) {
        let Some((_, _, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        self.oam_allocate_defer_to_player(k);
        let oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let scratch = &self.game_state.sprites.draw_hitbox_work;
        let x = scratch.x_low();
        let y = scratch.y_low();

        for i in 0..2 {
            let (dx, dy, chr, _fl) = WITCH_DRAW_DATA_A[g + i];
            self.set_oam_plain_at(
                oam + i * 4,
                x.wrapping_add(dx as u8),
                y.wrapping_add(dy as u8),
                chr,
                flags,
                0,
            );
        }
        for (i, &(dx, dy, chr, fl)) in WITCH_DRAW_DATA_B.iter().enumerate() {
            self.set_oam_plain_at(
                oam + (i + 2) * 4,
                x.wrapping_add(dx as u8),
                y.wrapping_add(dy as u8),
                chr,
                flags ^ fl,
                2,
            );
        }
        let i = usize::from((g as u16).wrapping_sub(6) < 6);
        let (dx, dy, chr, _fl) = WITCH_DRAW_DATA_C[i];
        self.set_oam_plain_at(
            oam + 5 * 4,
            x.wrapping_add(dx as u8),
            y.wrapping_add(dy as u8),
            chr,
            flags,
            2,
        );
        self.sprite_correct_oam_entries(k, 5, 0xff);
    }

    // -----------------------------------------------------------------------
    // void ElderWife_Draw(int k) {  // 85f505
    pub(super) fn elder_wife_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple_player_deferred(k, &ELDER_WIFE_DRAW_FRAMES[base..base + 2]);
    }

    // -----------------------------------------------------------------------
    // void Sprite_78_MrsSahasrahla(int k) {  // 86c071
    pub(super) fn sprite_78_mrs_sahasrahla(&mut self, k: usize) {
        self.elder_wife_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.inventory.items.sword_type() < 2 {
                    if self.sprite_show_solicited_message(k, 0x002b) & 0x100 != 0 {
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                } else {
                    self.sprite_show_solicited_message(k, 0x002e);
                }
                let value = self.game_state.frame.frame_counter >> 4 & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            1 => {
                self.sprite_show_message_unconditional(0x002c);
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            2 => {
                if self.multiselect_choice().value() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.sprite_show_message_unconditional(0x002d);
                } else {
                    self.sprite_show_message_unconditional(0x002c);
                }
            }
            3 => {
                self.sprite_show_solicited_message(k, 0x002d);
                let value = self.game_state.frame.frame_counter >> 4 & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void MiddleAgedMan_Draw(int k) {  // 86bdac
    pub(in crate::zelda_rtl) fn middle_aged_man_draw(&mut self, k: usize) {
        let mut info = self.sprite_draw_multiple_player_deferred(k, &MIDDLE_AGED_MAN_DRAW_FRAMES);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void BlindHideoutGuy_Draw(int k) {  // 8dc481
    pub(super) fn blind_hideout_guy_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &BLIND_HIDEOUT_GUY_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_HumanMulti_1(int k) {  // 8dc2d9
    pub(super) fn sprite_human_multi_1(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_flute_dad(k),
            1 => self.sprite_thief_hideout_guy(k),
            2 => self.sprite_blinds_hut_guy(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BlindsHutGuy(int k) {  // 8dc2e6
    pub(super) fn sprite_blinds_hut_guy(&mut self, k: usize) {
        self.blind_hideout_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_track_body_to_head(k);
        let value = 0;
        self.sprite_slot_view_mut(k).set_head_direction(value);
        let j = self.sprite_show_solicited_message(k, 0x0172);
        if j & 0x100 != 0 {
            let dir = j as u8;
            let value = dir;
            self.sprite_slot_view_mut(k).set_direction(value);
            let value = dir;
            self.sprite_slot_view_mut(k).set_head_direction(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_ThiefHideoutGuy(int k) {  // 8dc308
    pub(super) fn sprite_thief_hideout_guy(&mut self, k: usize) {
        if self.game_state.frame.frame_counter & 3 == 0 {
            let value = 2;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let dir = self.sprite_direction_to_face_link(k);
            let value = if dir == 3 { 2 } else { dir };
            self.sprite_slot_view_mut(k).set_head_direction(value);
        }
        let value = 15;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.oam_allocate_defer_to_player(k);
        self.thief_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_show_solicited_message(k, 0x0171);
        let value = 2;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_FluteDad(int k) {  // 8dc343
    pub(super) fn sprite_flute_dad(&mut self, k: usize) {
        self.flute_boy_father_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        let value = if self.game_state.frame.frame_counter < 48 {
            2
        } else {
            (self.game_state.frame.frame_counter >> 7) & 1
        };
        self.sprite_slot_view_mut(k).set_graphics(value);

        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_show_solicited_message(k, 0x00a3);
            let value = 2;
            self.sprite_slot_view_mut(k).set_graphics(value);
        } else if self.game_state.inventory.items.flute() < 2 {
            self.sprite_show_solicited_message(k, 0x00a1);
        } else if self.sprite_show_solicited_message(k, 0x00a4) & 0x100 == 0
            && self.game_state.inventory.save_progress.hud_current_item() == HUD_ITEM_FLUTE_DRAW
            && self.game_state.player.follower_link.joypad1h_last() & 0x40 != 0
            && self.sprite_check_damage_to_link_same_layer(k)
        {
            self.sprite_show_message_unconditional(0x00a2);
            self.sprite_slot_view_mut(k).add_ai_state(1);
            let value = 2;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
    }

    // -----------------------------------------------------------------------
    // void SweepingLady_Draw(int k) {  // 8dc4eb
    pub(super) fn sweeping_lady_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &SWEEPING_LADY_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_SweepingLady(int k) {  // 8dc4ad
    pub(super) fn sprite_sweeping_lady(&mut self, k: usize) {
        self.sweeping_lady_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_show_solicited_message(k, 0x00a5);
        self.sprite_behave_as_barrier(k);
        let value = self.game_state.frame.frame_counter >> 4 & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_FortuneTeller(int k) {  // 8dc762
    pub(super) fn sprite_fortune_teller(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => {
                self.fortune_teller_draw(k);
                if self.sprite_return_if_inactive(k) {
                    return;
                }
                self.fortune_teller_light_or_dark_world(
                    k,
                    ((self.game_state.inventory.save_progress.dark_world_state() >> 6) & 1) != 0,
                );
            }
            1 => {
                if self.sprite_return_if_inactive(k) {
                    return;
                }
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.sprite_nullify_hookshot_drag();
                    self.follower_link_state_mut().set_speed_setting(0);
                    self.link_cancel_dash();
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void FortuneTeller_LightOrDarkWorld(int k, bool dark_world) {
    pub(super) fn fortune_teller_light_or_dark_world(&mut self, k: usize, dark_world: bool) {
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let j = self.get_random_number() & 3;
                let value = j << 1;
                self.sprite_slot_view_mut(k).set_a(value);
                if self.game_state.inventory.player_resources.rupees_goal()
                    < u16::from(FORTUNE_TELLER_LIGHT_OR_DARK_WORLD_PRICES[usize::from(j)])
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                self.sprite_show_solicited_message(k, 0x00f2);
            }
            2 => {
                if self.sprite_show_solicited_message(k, 0x00f3) & 0x100 != 0 {
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.follower_link_state_mut().immobilize();
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            3 => {
                if self.multiselect_choice().value() == 0 {
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                    let value = (self.game_state.frame.frame_counter >> 4) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                } else {
                    self.sprite_show_message_unconditional(0x00f5);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.follower_link_state_mut().clear_immobilized();
                }
            }
            4 => {
                self.fortune_teller_perform_pseudo_science(k);
            }
            5 => {
                if !dark_world {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
                let j = FORTUNE_TELLER_LIGHT_OR_DARK_WORLD_PRICES
                    [usize::from(self.sprite_slot_view(k).a() >> 1)];
                self.dialogue_number_mut()
                    .set_packed_digits((j / 10) | ((j % 10) << 4), 0);
                self.sprite_show_message_unconditional(0x00f4);
                self.sprite_slot_view_mut(k).add_ai_state(1);
            }
            6 => {
                let price = u16::from(
                    FORTUNE_TELLER_LIGHT_OR_DARK_WORLD_PRICES
                        [usize::from(self.sprite_slot_view(k).a() >> 1)],
                );
                let rupees = self
                    .game_state
                    .inventory
                    .player_resources
                    .rupees_goal()
                    .wrapping_sub(price);
                self.player_resources_mut().set_rupees_goal(rupees);
                self.sprite_slot_view_mut(k).add_ai_state(1);
                self.player_resources_mut().set_heart_filler(160);
                self.follower_link_state_mut().clear_immobilized();
            }
            7 => {}
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void FortuneTeller_PerformPseudoScience(int k) {  // 8dc849
    pub(super) fn fortune_teller_perform_pseudo_science(&mut self, k: usize) {
        let value = 0;

        self.sprite_slot_view_mut(k).set_graphics(value);
        self.sprite_slot_view_mut(k).add_ai_state(1);

        let mut slots = [0u8; 2];
        let mut n = 0usize;
        if self
            .game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            >= 3
        {
            let inventory = &self.game_state.inventory.items;
            let resources = &self.game_state.inventory.player_resources;
            if inventory.book() == 0 {
                slots[n] = 2;
                n += 1;
            }
            if n < 2 && (resources.pendant_flags() & 2) == 0 {
                slots[n] = 1;
                n += 1;
            }
            if n < 2 && inventory.mushroom() < 2 {
                slots[n] = 3;
                n += 1;
            }
            if n < 2 && inventory.flippers() == 0 {
                slots[n] = 4;
                n += 1;
            }
            if n < 2 && inventory.moon_pearl() == 0 {
                slots[n] = 5;
                n += 1;
            }
            if n < 2 && self.game_state.inventory.save_progress.progress_indicator() < 3 {
                slots[n] = 6;
                n += 1;
            }
            if n < 2 && resources.magic_consumption_level() == 0 {
                slots[n] = 7;
                n += 1;
            }
            if n < 2 && inventory.bombos() == 0 {
                slots[n] = 8;
                n += 1;
            }
            if n < 2
                && (self
                    .game_state
                    .inventory
                    .save_progress
                    .progress_indicator_3()
                    & 0x10)
                    == 0
            {
                slots[n] = 9;
                n += 1;
            }
            if n < 2
                && (self
                    .game_state
                    .inventory
                    .save_progress
                    .progress_indicator_3()
                    & 0x20)
                    == 0
            {
                slots[n] = 10;
                n += 1;
            }
            if n < 2 && inventory.cape() == 0 {
                slots[n] = 11;
                n += 1;
            }
            if n < 2 && (self.game_state.world.overworld.event_info.event_info(0x5b) & 2) == 0 {
                slots[n] = 12;
                n += 1;
            }
            if n < 2 && inventory.sword_type() < 4 {
                slots[n] = 13;
                n += 1;
            }
            if n < 2 {
                slots[n] = 14;
                n += 1;
            }
            if n < 2 {
                slots[n] = 15;
            }
        }

        self.save_progress_mut().xor_progress_flags(0x40);
        let j = usize::from((self.game_state.inventory.save_progress.progress_flags() & 0x40) != 0);
        self.sprite_show_message_unconditional(u16::from(
            FORTUNE_TELLER_PERFORM_PSEUDO_SCIENCE_READINGS[usize::from(slots[j])],
        ));
    }

    // -----------------------------------------------------------------------
    // void FortuneTeller_Draw(int k) {  // 8dcb01
    pub(super) fn fortune_teller_draw(&mut self, k: usize) {
        let j = usize::from((self.game_state.inventory.save_progress.dark_world_state() >> 6) & 1)
            * 2
            + usize::from(self.sprite_slot_view(k).graphics());
        self.sprite_draw_multiple(k, &FORTUNE_TELLER_DRAW_FRAMES[j * 3..j * 3 + 3]);
    }

    // -----------------------------------------------------------------------
    // void Sprite_MazeGameLady(int k) {  // 8dcb5c
    pub(super) fn sprite_maze_game_lady(&mut self, k: usize) {
        self.lady_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_track_body_to_head(k);
        let value = self.sprite_direction_to_face_link(k) ^ 3;
        self.sprite_slot_view_mut(k).set_head_direction(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).x_low() < self.game_state.player.follower_link.x() as u8
                {
                    let j = self.sprite_show_message_on_contact(k, 0x00cc);
                    if j & 0x100 != 0 {
                        let dir = j as u8;
                        let value = dir;
                        self.sprite_slot_view_mut(k).set_direction(value);
                        let value = dir;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.maze_game_timer_mut().clear_elapsed();
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_a(value);
                        self.clear_flag_overworld_area_changed();
                    }
                } else {
                    self.sprite_show_message_on_contact(k, 0x00d0);
                }
            }
            1 => {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x07);
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            2 => {
                self.sprite_slot_view_mut(k).add_a(1);
                if self.sprite_slot_view(k).a() == 63 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_a(value);
                    let t = self.maze_game_timer_mut().increment_elapsed_low();
                    if t == 0 {
                        self.maze_game_timer_mut().increment_elapsed_high();
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_MazeGameGuy(int k) {  // 8dcbf2
    pub(super) fn sprite_maze_game_guy(&mut self, k: usize) {
        self.maze_game_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_track_body_to_head(k);
        let value = 0;
        self.sprite_slot_view_mut(k).set_head_direction(value);
        self.sprite_behave_as_barrier(k);
        let value = self.game_state.frame.frame_counter >> 3 & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.game_state.world.region.flag_overworld_area_changed() {
            self.sprite_show_message_on_contact(k, 0x00d0);
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let elapsed = self.game_state.sprites.maze_game_timer.elapsed_low();
                self.maze_game_timer_mut().capture_snapshot();
                let mut t = elapsed % 6000;
                let a = t / 600;
                t %= 600;
                let b = t / 60;
                t %= 60;
                let c = t / 10;
                t %= 10;
                self.dialogue_number_mut()
                    .set_packed_digits((t | (c << 4)) as u8, (b | (a << 4)) as u8);
                let j = self.sprite_show_message_on_contact(k, 0x00cb);
                if j & 0x100 != 0 {
                    let dir = j as u8;
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                let screen = usize::from(self.game_state.world.location.overworld_screen_index());
                if self
                    .game_state
                    .world
                    .overworld
                    .event_info
                    .event_info(screen)
                    & 0x40
                    != 0
                {
                    self.sprite_show_message_unconditional(0x00cf);
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else if self.game_state.sprites.maze_game_timer.snapshot_low() < 16 {
                    self.sprite_show_message_unconditional(0x00cd);
                    let dir = self.game_state.player.follower_link.handler_state();
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x00ce);
                    let dir = self.game_state.player.follower_link.handler_state();
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                let j = self.sprite_show_message_on_contact(k, 0x00ce);
                if j & 0x100 != 0 {
                    let dir = j as u8;
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
            }
            3 => {
                let j = self.sprite_show_solicited_message(k, 0x00cd);
                if j & 0x100 != 0 {
                    let dir = j as u8;
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
            }
            4 => {
                let j = self.sprite_show_solicited_message(k, 0x00cf);
                if j & 0x100 != 0 {
                    let dir = j as u8;
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void MazeGameGuy_Draw(int k) {  // 8dcda7
    pub(super) fn maze_game_guy_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &MAZE_GAME_GUY_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void DrinkingGuy_Draw(int k) {  // 9af88c
    pub(super) fn drinking_guy_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &DRINKING_GUY_DRAW_FRAMES[base..base + 3],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_25_TalkingTree(int k) {  // 86c0d5
    pub(super) fn sprite_25_talking_tree(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.talking_tree_mouth(k),
            1 => self.talking_tree_eye(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void TalkingTree_Mouth(int k) {  // 9df956
    pub(super) fn talking_tree_mouth(&mut self, k: usize) {
        self.talking_tree_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_flags4(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.link_cancel_dash();
                    let pt = self.sprite_project_speed_towards_link(k, 48);
                    self.follower_link_state_mut().set_incapacitated_timer(16);
                    self.follower_link_state_mut()
                        .set_actual_velocity_xy(pt.x, pt.y);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                let value = (self.sprite_slot_view(k).delay_main() >> 1) & 3;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            2 => {
                // The ROM indexes kTalkingTree_Gfx2 with the sprite slot, not
                // the shifted delay it just computed (see
                // TALKING_TREE_MOUTH_SECONDARY_GRAPHICS_BY_SLOT).
                let _ = TALKING_TREE_MOUTH_SECONDARY_GRAPHICS;
                let value = TALKING_TREE_MOUTH_SECONDARY_GRAPHICS_BY_SLOT[k];
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).delay_main() == 7 {
                    self.talking_tree_spawn_bomb(k);
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
            }
            3 => {
                let value = 7;
                self.sprite_slot_view_mut(k).set_flags4(value);
                if self.sprite_slot_view(k).a() == 0 {
                    let j = ((self.sprite_slot_view(k).x_low() >> 4) & 1) ^ 1;
                    let value = j;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if self
                        .sprite_show_solicited_message(k, TALKING_TREE_MOUTH_MSGS_2[usize::from(j)])
                        & 0x100
                        == 0
                    {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_a(value);
                    }
                } else {
                    let j = TALKING_TREE_MOUTH_SCREENS
                        .iter()
                        .rposition(|&screen| {
                            screen == self.game_state.world.location.overworld_screen_index()
                        })
                        .unwrap_or(0);
                    self.sprite_show_message_unconditional(TALKING_TREE_MOUTH_MSGS[j]);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_a(value);
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = self.sprite_slot_view(k).b().wrapping_add(1) & 7;
                    let value = j;
                    self.sprite_slot_view_mut(k).set_b(value);
                    let value = TALKING_TREE_MOUTH_LOCAL_GRAPHICS[usize::from(j)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = TALKING_TREE_MOUTH_DELAY[usize::from(j)];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void TalkingTree_Draw(int k) {  // 9dfadb
    pub(super) fn talking_tree_draw(&mut self, k: usize) {
        let g = self.sprite_slot_view(k).graphics().wrapping_sub(1);
        if sign8(g) {
            return;
        }
        let base = usize::from(g) * 4;
        if base + 4 > TALKING_TREE_DRAW_FRAMES.len() {
            // Slots 6, 7, and 14 receive instruction bytes from the ROM's
            // slot-indexed kTalkingTree_Gfx2 read
            // (TALKING_TREE_MOUTH_SECONDARY_GRAPHICS_BY_SLOT); the ROM then
            // draws from past the end of kTalkingTree_Dmd. That garbage is
            // not modeled yet; skip the draw instead of indexing out of range.
            return;
        }
        self.sprite_draw_multiple_player_deferred(
            k,
            &TALKING_TREE_DRAW_FRAMES[base..base + 4],
        );
    }

    // -----------------------------------------------------------------------
    // void TalkingTree_Eye(int k) {  // 9dfb0a
    pub(super) fn talking_tree_eye(&mut self, k: usize) {
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let mut j = usize::from(self.sprite_slot_view(k).head_direction());
        let x = u16::from(self.sprite_slot_view(k).a())
            | (u16::from(self.sprite_slot_view(k).b()) << 8);
        let y = u16::from(self.sprite_slot_view(k).c())
            | (u16::from(self.sprite_slot_view(k).e()) << 8);
        self.sprite_set_x(
            k,
            x.wrapping_add_signed(i16::from(TALKING_TREE_EYE_BASE_X_OFFSETS[j])),
        );
        self.sprite_set_y(k, y);
        let pt = self.sprite_project_speed_towards_link(k, 2);
        if !sign8(pt.y) {
            let value = pt.x.wrapping_add(2);
            self.sprite_slot_view_mut(k).set_direction(value);
        } else if self.sprite_slot_view(k).direction() != 2 {
            let value = self.sprite_slot_view(k).direction().wrapping_add(
                if self.sprite_slot_view(k).direction() >= 2 {
                    0xff
                } else {
                    1
                },
            );
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        j = usize::from(self.sprite_slot_view(k).direction());
        self.sprite_set_x(
            k,
            x.wrapping_add_signed(i16::from(TALKING_TREE_EYE_X_OFFSETS_1[j])),
        );
        self.sprite_set_y(
            k,
            y.wrapping_add_signed(i16::from(TALKING_TREE_EYE_Y_OFFSETS_1[j])),
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_28_DarkWorldHintNPC(int k) {  // 86ad6f
    pub(super) fn sprite_28_dark_world_hint_npc(&mut self, k: usize) {
        self.story_teller_1_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = (self.game_state.frame.frame_counter >> 4) & 1;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }

        match self.sprite_slot_view(k).subtype2() {
            0 => self.dark_world_hint_npc_run_state(k, 0x00ff),
            1 => self.dark_world_hint_npc_run_state(k, 0x0101),
            2 => self.dark_world_hint_npc_run_state(k, 0x0102),
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    if (self.game_state.frame.frame_counter & 0x3f) == 0 {
                        self.sprite_slot_view_mut(k).xor_oam_flags(0x40);
                    }
                    if self.get_random_number() == 0 {
                        let value = 32;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                }
                self.sprite_show_solicited_message(k, 0x0149);
            }
            4 => {
                let value = (self.game_state.frame.frame_counter >> 1) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_move_z(k);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                }
                let value = self.sprite_slot_view(k).z_velocity().wrapping_add(
                    if self.sprite_slot_view(k).z() >= 4 {
                        0xff
                    } else {
                        1
                    },
                );
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                self.dark_world_hint_npc_run_state(k, 0x0103);
            }
            _ => {}
        }
    }

    fn dark_world_hint_npc_run_state(&mut self, k: usize, paid_msg: u16) {
        match self.sprite_slot_view(k).ai_state() {
            0 => self.dark_world_hint_npc_idle(k),
            1 => {
                if self.multiselect_choice().value() == 0
                    && self.dark_world_hint_npc_handle_payment()
                {
                    self.sprite_show_message_unconditional(paid_msg);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x0100);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => self.dark_world_hint_npc_restore_health(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void StoryTeller_1_Draw(int k) {  // 86af1a
    pub(super) fn story_teller_1_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).subtype2()) * 2
            + usize::from(self.sprite_slot_view(k).graphics());
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &STORY_TELLER_1_DRAW_FRAMES[base..base + 1],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void SmithyFrog_Draw(int k) {  // 86b339
    pub(in crate::zelda_rtl) fn smithy_frog_draw(&mut self, k: usize) {
        let mut info = self.sprite_draw_multiple_player_deferred(k, &SMITHY_FROG_DRAW_FRAMES);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void SmithySpark_Draw(int k) {  // 86b72c
    pub(in crate::zelda_rtl) fn smithy_spark_draw(&mut self, k: usize) {
        self.oam_allocate_from_region_b(8);
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &SMITHY_SPARK_DRAW_FRAMES[base..base + 2]);
    }

    // -----------------------------------------------------------------------
    // void GerudoMan_Draw(int k) {  // 85ba24
    pub(super) fn gerudo_man_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        for i in (0..3).rev() {
            let j = g * 3 + i;
            self.set_oam_helper0_at(
                oam,
                info_x.wrapping_add(GERUDO_MAN_DRAW_X_OFFSETS[j] as i16 as u16),
                info_y.wrapping_add(GERUDO_MAN_DRAW_Y_OFFSETS[j] as i16 as u16),
                GERUDO_MAN_DRAW_CHARS[j],
                GERUDO_MAN_DRAW_FL[j] | info_flags,
                GERUDO_MAN_DRAW_BIG[j],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Recruit_Draw(int k) {  // 85bd7e
    pub(super) fn recruit_draw(&mut self, k: usize) {
        // Cycle ledger: Recruit_Draw $05:BD7E (JSR target, m8 x8): JSR
        // Sprite_PrepOamCoordOrDoubleRet__ (46; off screen it double-returns
        // past this routine). On screen: $05:BD81-BDA6 the head entry (504;
        // the closing BCC is taken +6 when head y + $10 < $100, else
        // $05:BDA8 stores $F0, 78), $05:BDAD-BDE4 the body entry (794; same
        // test on the body y, else $05:BDE6, 78), $05:BDEB-BE09 the
        // extension bytes, JSL Sprite_DrawShadow_, RTS (492).
        let _scope = crate::cycle_ledger::routine(0x05_bd7e);
        crate::cycle_ledger::charge(46);
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut info = PrepOamCoordsRet {
            x: info_x,
            y: info_y,
            r4: 0,
            flags: info_flags,
        };
        let oam = self.game_state.oam.current_pointer_usize();
        let hd = usize::from(self.sprite_slot_view(k).head_direction() & 3);
        let head_y = info_y.wrapping_sub(11);
        crate::cycle_ledger::charge(504 + if head_y.wrapping_add(0x10) < 0x100 { 6 } else { 78 });
        self.set_oam_helper0_at(
            oam,
            info_x,
            head_y,
            RECRUIT_DRAW_SOLDIER_CH[hd],
            RECRUIT_DRAW_SOLDIER_FL[hd] | info_flags,
            2,
        );
        let r6 = usize::from(self.sprite_slot_view(k).graphics());
        crate::cycle_ledger::charge(794 + if info_y.wrapping_add(0x10) < 0x100 { 6 } else { 78 });
        self.set_oam_helper0_at(
            oam + 4,
            info_x.wrapping_add(RECRUIT_DRAW_X_OFFSETS[r6] as u16),
            info_y,
            RECRUIT_DRAW_CHARS[r6],
            RECRUIT_DRAW_FL[r6] | info_flags,
            2,
        );
        crate::cycle_ledger::charge(492);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void QuarrelBros_Draw(int k) {  // 85e17f
    pub(in crate::zelda_rtl) fn quarrel_bros_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &QUARREL_BROS_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Lumberjacks_Draw(int k) {  // 8dc6ba
    pub(super) fn lumberjacks_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 11;
        self.sprite_draw_multiple(k, &LUMBERJACKS_DRAW_FRAMES[base..base + 11]);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Lumberjacks(int k) {  // 8dc51b
    pub(super) fn sprite_lumberjacks(&mut self, k: usize) {
        self.lumberjacks_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.lumberjack_check_proximity(k, 0) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
        }
        if !self.sprite_check_if_link_is_busy()
            && self.lumberjack_check_proximity(k, 1)
            && self.game_state.player.follower_link.filtered_joypad_l() & 0x80 != 0
        {
            let msg = usize::from(
                self.game_state.player.follower_link.x() as u8 >= self.sprite_slot_view(k).x_low(),
            ) + usize::from(self.game_state.inventory.items.sword_type() >= 2) * 2;
            self.sprite_show_message_unconditional(SPRITE_LUMBERJACKS_MESSAGES[msg]);
        }
        let value = self.game_state.frame.frame_counter >> 5 & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_TroughBoy(int k) {  // 85ff66
    pub(super) fn sprite_trough_boy(&mut self, k: usize) {
        self.trough_boy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_track_body_to_head(k);
        let value = self.sprite_direction_to_face_link(k) ^ 3;
        self.sprite_slot_view_mut(k).set_head_direction(value);

        if self
            .game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            < 3
        {
            if self.sprite_show_solicited_message(k, 0x147) & 0x100 != 0 {
                self.save_progress_mut().set_map_icons_indicator(2);
            }
        } else {
            self.sprite_show_solicited_message(k, 0x148);
        }
    }

    // -----------------------------------------------------------------------
    // void TroughBoy_Draw(int k) {  // 85ffdf
    pub(super) fn trough_boy_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).direction()) * 2;
        let mut info = self.sprite_draw_multiple_player_deferred(
            k,
            &TROUGH_BOY_DRAW_FRAMES[base..base + 2],
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void RetreatBat_Draw(int k) {  // 9af833
    pub(super) fn retreat_bat_draw(&mut self, k: usize) {
        self.oam_state_mut().set_current_pointer(0x960);
        self.oam_state_mut().set_current_extended_pointer(0xa78);
        let j = usize::from(self.sprite_slot_view(k).direction()) * 4
            + usize::from(self.sprite_slot_view(k).graphics());
        self.sprite_draw_multiple(
            k,
            &RETREAT_BAT_DRAW_FRAMES[RETREAT_BAT_DRAW_OFFSETS[j]
                ..RETREAT_BAT_DRAW_OFFSETS[j] + RETREAT_BAT_DRAW_COUNT[j]],
        );
    }

    // -----------------------------------------------------------------------
    // void GanonBat_Draw(int k) {  // 9d89eb
    pub(in crate::zelda_rtl) fn ganon_bat_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &GANON_BAT_DRAW_FRAMES[base..base + 2]);
    }

    // -----------------------------------------------------------------------
    // void EvilBarrier_Draw(int k) {  // 9df249
    pub(in crate::zelda_rtl) fn evil_barrier_draw(&mut self, k: usize) {
        let y = self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_add(8);
        self.sprite_workspace_mut().set_current_sprite_y(y);
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 9;
        self.sprite_draw_multiple(k, &EVIL_BARRIER_DRAW_FRAMES[base..base + 9]);
        self.sprite_get16_bit_coords(k);
    }

    // -----------------------------------------------------------------------
    // void ChattyAgahnim_Draw(int k, PrepOamCoordsRet *info) {  // 9dd451
    pub(super) fn chatty_agahnim_draw(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        if self.sprite_slot_view(k).delay_aux4() & 1 != 0 {
            return;
        }
        if self.sprite_slot_view(k).c() == 0 {
            self.oam_state_mut().set_current_pointer(0x900);
            self.oam_state_mut().set_current_extended_pointer(0xa60);
        }
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        let mut shadow_info = self.sprite_draw_multiple(
            k,
            &CHATTY_AGAHNIM_DRAW_FRAMES[base..base + 4],
        );
        self.sprite_draw_shadow_custom(k, &mut shadow_info, 18);
        info.x = shadow_info.x;
        info.y = shadow_info.y;
        info.r4 = shadow_info.r4;
        info.flags = shadow_info.flags;
    }

    // -----------------------------------------------------------------------
    // void FaerieQueen_Draw(int k) {  // 86cb26
    pub(in crate::zelda_rtl) fn faerie_queen_draw(&mut self, k: usize) {
        if self.game_state.inventory.save_progress.dark_world_state() == 0 {
            let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k)
            else {
                return;
            };
            let mut oam = self.game_state.oam.current_pointer_usize();
            let g = usize::from(self.sprite_slot_view(k).graphics());
            for i in (0..12).rev() {
                let j = g * 12 + i;
                self.set_oam_plain_at(
                    oam,
                    FAERIE_QUEEN_DRAW_X_OFFSETS[j].wrapping_add(info_x as u8),
                    FAERIE_QUEEN_DRAW_Y_OFFSETS[j].wrapping_add(info_y as u8),
                    FAERIE_QUEEN_DRAW_CHARS[j],
                    info_flags | FAERIE_QUEEN_DRAW_FL[j],
                    FAERIE_QUEEN_DRAW_BIG[j],
                );
                oam += 4;
            }
            self.sprite_correct_oam_entries(k, 11, 0xff);
        } else {
            let base = usize::from(self.sprite_slot_view(k).graphics()) * 10;
            self.sprite_draw_multiple(k, &FAERIE_QUEEN_DRAW_FRAMES[base..base + 10]);
        }
    }

    // -----------------------------------------------------------------------
    // void CrystalMaiden_Draw(int k) {  // 8dce5f
    pub(in crate::zelda_rtl) fn crystal_maiden_draw(&mut self, k: usize) {
        // $0D:CE5F..CE90: bank/table/DMA setup, JSL deferred drawing,
        // and PLB/RTL. The called draw routine owns its own instructions.
        let _scope = crate::cycle_ledger::routine(0x0d_ce5f);
        crate::cycle_ledger::charge(656);
        let j = usize::from(self.sprite_slot_view(k).direction()) * 2
            + usize::from(self.sprite_slot_view(k).graphics());
        self.set_sprite_dma_head_pointer(CRYSTAL_MAIDEN_DRAW_DMA[j * 2]);
        self.set_sprite_dma_body_pointer(CRYSTAL_MAIDEN_DRAW_DMA[j * 2 + 1]);
        self.sprite_draw_multiple_player_deferred(
            k,
            &CRYSTAL_MAIDEN_DRAW_FRAMES[j * 2..j * 2 + 2],
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_2E_FluteKid(int k) {  // 86af3b
    pub(super) fn sprite_2_e_flute_kid(&mut self, k: usize) {
        match self.sprite_slot_view(k).head_direction() {
            0 => match self.sprite_slot_view(k).subtype2() {
                0 => self.flute_kid_human(k),
                1 => self.sprite_flute_kid_stumpy(k),
                _ => {}
            },
            1 => self.sprite_flute_kid_quaver(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_1F_SickKid(int k) {  // 86b94c
    pub(in crate::zelda_rtl) fn sprite_1_f_sick_kid(&mut self, k: usize) {
        self.bug_net_kid_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_check_if_link_is_busy()
                    || !self.sprite_check_damage_to_link_same_layer(k)
                {
                    return;
                }
                let bottles = self.game_state.inventory.items.bottle(0)
                    | self.game_state.inventory.items.bottle(1)
                    | self.game_state.inventory.items.bottle(2)
                    | self.game_state.inventory.items.bottle(3);
                if bottles < 2 {
                    self.sprite_show_solicited_message(k, 0x104);
                } else {
                    self.follower_link_state_mut().increment_immobilized_flag();
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    return;
                }
                let j = usize::from(self.sprite_slot_view(k).a());
                if SPRITE_1_F_SICK_KID_LOCAL_GRAPHICS[j] >= 0 {
                    let value = SPRITE_1_F_SICK_KID_LOCAL_GRAPHICS[j] as u8;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = SPRITE_1_F_SICK_KID_DELAY[j];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_a(1);
                } else {
                    self.sprite_show_message_unconditional(0x105);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x21,
                        0,
                        ItemReceiptCaller::SpriteMain {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::SickKid,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_sick_kid_item_receipt(k);
            }
            3 => {
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_show_solicited_message(k, 0x106);
            }
            _ => {}
        }
    }

    /// Source suffix after SickKid's synchronous `Link_ReceiveItem` call.
    ///
    /// ROM $06:b9d0 restores X, increments the sprite AI state, then clears
    /// Link's immobilized flag. Keeping the suffix typed lets a live timing
    /// authority suspend the decompressor without publishing either write.
    pub(in crate::zelda_rtl) fn complete_sick_kid_item_receipt(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ai_state();
        self.follower_link_state_mut().clear_immobilized();
    }

    // -----------------------------------------------------------------------
    // void Sprite_1D_FluteQuest(int k) {  // 86c2e5
    pub(super) fn sprite_1_d_flute_quest(&mut self, k: usize) {
        self.sprite_prep_oam_coord_from(k, super::sprite::PrepOamCoordEntry::Safe);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.game_state.world.location.overworld_screen_index() == 0x18 {
            if self.game_state.inventory.items.flute() == 3 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        } else if (self.game_state.inventory.items.flute() & 2) != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void BatCrash_DrawHardcodedGarbage(int k) {  // 9af750
    pub(super) fn bat_crash_draw_hardcoded_garbage(&mut self, _k: usize) {
        let mut oam = OAM_BUF + 76 * 4;
        for &(x, y, ch, fl) in &BAT_CRASH_DRAW_HARDCODED_GARBAGE_OAMS {
            self.oam_state_mut()
                .write_entry(oam, x as u8, y as u8, ch, fl);
            oam += 4;
        }
        for i in 0..9 {
            let value = 2;
            self.oam_state_mut().set_extended_byte(76 + i, value);
        }
    }

    pub(super) fn altar_zelda_draw_body(&mut self, k: usize, info: &PrepOamCoordsRet) {
        self.oam_allocate_from_region_a(8);
        let z = self.sprite_slot_view(k).z().min(31);
        let xoffs = u16::from(ALTAR_ZELDA_DRAW_BODY_X_OFFS[usize::from(z >> 1)]);
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_helper0_at(
            oam,
            info.x.wrapping_add(xoffs),
            y.wrapping_add(7),
            0x6c,
            0x24,
            2,
        );
        self.set_oam_helper0_at(
            oam + 4,
            info.x.wrapping_sub(xoffs),
            y.wrapping_add(7),
            0x6c,
            0x24,
            2,
        );
    }
}
