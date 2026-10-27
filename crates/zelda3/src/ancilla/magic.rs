//! Somaria, Byrna, medallion spells, powder, and lamp effects.

use crate::zelda_rtl::sprite::SpriteHitBoxEntry;
use super::*;
const CANE_OF_SOMARIA_PLACEMENT_Y_OFFSETS: [i8; 4] = [-8, 31, 17, 17];
const CANE_OF_SOMARIA_PLACEMENT_X_OFFSETS: [i8; 4] = [8, 8, -8, 23];
use crate::tile_definition::{DungeonRole, NativeTile};
use crate::types::{sign8, AncillaRadialProjection};

impl ZeldaState {
    pub(super) fn ancilla01_somaria_bullet(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            if self.game_state.frame.frame_counter
                & ANCILLA01_SOMARIA_BULLET_SOMARIAN_BLAST_MASK
                    [self.ancilla_slot_view(k).step() as usize]
                == 0
            {
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
            }
            if self.ancilla_slot_view(k).timer() == 0 {
                self.ancilla_slot_view_mut(k).set_timer(3);
                let mut a = self.ancilla_slot_view_mut(k).advance_step();
                if a >= 6 {
                    a = 4;
                }
                self.ancilla_slot_view_mut(k).set_step(a);
            }
            // ROM $08:8546
            if self.ancilla_check_sprite_collision_marked(k).is_some()
                || self.ancilla_check_tile_collision_staggered(k) != 0
            {
                let mut bullet = self.ancilla_slot_view_mut(k);
                bullet.set_ancilla_type(4);
                bullet.set_timer(7);
                bullet.set_num_sprites(16);
            }
        }
        self.somarian_blast_draw(k);
    }

    pub(super) fn ancilla30_byrna_windup_spark(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(self.ancilla_slot_view(k).aux_timer()) {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                if self.ancilla_slot_view_mut(k).advance_item_to_link() == 17 {
                    self.byrna_windup_spark_transmute_to_normal(k);
                    return;
                }
            }
        }
        if self.ancilla_slot_view(k).item_to_link() == 0 {
            return;
        }

        let mut j = self.game_state.player.follower_link.action_handler_timer();
        if j == 2 {
            let mut a = self.ancilla_slot_view(k).work_byte_3().wrapping_sub(1);
            if sign8(a) {
                a = 0;
                j = 3;
            }
            let value = a;
            self.ancilla_slot_view_mut(k).set_work_byte_3(value);
        }
        let j = j.wrapping_add(
            self.game_state
                .player
                .follower_link
                .facing()
                .wrapping_mul(2),
        ) as usize;
        self.ancilla_set_xy(
            k,
            self.game_state
                .player
                .follower_link
                .x()
                .wrapping_add(ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_X[j] as i16 as u16),
            self.game_state
                .player
                .follower_link
                .y()
                .wrapping_add(ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_Y[j] as i16 as u16),
        );
        let (x, y) = self.ancilla_prep_oam_coord(k);

        let a = self.ancilla_slot_view(k).item_to_link().wrapping_sub(1) & 0x0f;
        let mut j = 0usize;
        if a != 0 {
            j = 4 * if a != 15 { ((a & 1) + 1) as usize } else { 3 };
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        for _ in 0..4 {
            if ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(
                        ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_X[j] as i16 as u16,
                    ),
                    y.wrapping_add(
                        ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_Y[j] as i16 as u16,
                    ),
                    ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_CHAR[j],
                    ANCILLA30_BYRNA_WINDUP_SPARK_INITIAL_CANE_SPARK_DRAW_FLAGS[j] & !0x30
                        | self.game_state.oam.priority_high(),
                    0,
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn byrna_windup_spark_transmute_to_normal(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).set_ancilla_type(0x31);
        let j = (self.game_state.player.follower_link.facing() << 1) as usize;
        self.effect_angle_scratch_mut().set_angles4(
            &BYRNA_WINDUP_SPARK_TRANSMUTE_TO_NORMAL_CANE_SPARK_TRAILING_ANGLES,
            j,
        );
        let value = 0;
        self.ancilla_slot_view_mut(k).set_g(value);
        {
            let mut spark = self.ancilla_slot_view_mut(k);
            spark.set_aux_timer(0x17);
            spark.set_item_to_link(0);
            spark.set_work_byte_3(8);
            spark.set_step(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        let value = 2;
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        self.ancilla_slot_view_mut(k).set_timer(21);
        self.effect_angle_scratch_mut().set_radial_radius(20);
        self.ancilla_sfx3_near(0x30);
        self.ancilla31_byrna_spark(k);
    }

    pub(super) fn ancilla31_byrna_spark(&mut self, k: usize) {
        let mut flags = 2;
        if self.game_state.frame.submodule == 0 {
            if self.game_state.player.follower_link.current_item_y() != 13 {
                self.kill_byrna_spark(k);
                return;
            }
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if self.ancilla_slot_view(k).aux_timer() == 0 {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                let magic_cost =
                    ANCILLA31_BYRNA_SPARK_CANE_SPARK_MAGIC[self.magic_consumption_level() as usize];
                let r0 = self
                    .game_state
                    .inventory
                    .player_resources
                    .magic
                    .amount()
                    .wrapping_sub(magic_cost);
                if self.game_state.inventory.player_resources.magic.amount() == 0 || r0 >= 0x80 {
                    self.kill_byrna_spark(k);
                    return;
                }

                self.ancilla_slot_view_mut(k).subtract_g(1);
                if sign8(self.ancilla_slot_view(k).g()) {
                    let value = 0x17;
                    self.ancilla_slot_view_mut(k).set_g(value);
                    self.player_magic_mut().set_magic_power(r0);
                }
                if self.game_state.player.follower_link.filtered_joypad_h() & 0x40 != 0 {
                    self.kill_byrna_spark(k);
                    return;
                }
            }
            if self.ancilla_slot_view(k).step() != 3 {
                let a = self.ancilla_slot_view_mut(k).advance_item_to_link();
                let value = if a >= 4 {
                    3
                } else if a == 2 {
                    1
                } else if a == 3 {
                    2
                } else {
                    0
                };
                self.ancilla_slot_view_mut(k).set_step(value);
            }
            self.ancilla_slot_view_mut(k).subtract_work_byte_1(1);
            if sign8(self.ancilla_slot_view(k).work_byte_1()) {
                let value = 2;
                self.ancilla_slot_view_mut(k).set_work_byte_1(value);
                flags = 4;
            }
        }

        let mut z = self.game_state.player.follower_link.z() as u8 as i8 as i16;
        if z == -1 {
            z = 0;
        }
        let swordbeam_temp_y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(12)
            .wrapping_sub(z as u16);
        let swordbeam_temp_x = self.game_state.player.follower_link.x().wrapping_add(8);
        self.ether_orbit_mut()
            .set_swordbeam_temp(swordbeam_temp_x, swordbeam_temp_y);
        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).set_timer(21);
            self.ancilla_sfx3_near(0x30);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut i = self.ancilla_slot_view(k).step() as usize;
        loop {
            let angle = if self.game_state.frame.submodule == 0 {
                self.effect_angle_scratch_mut().add_angle_mod64(i, 3)
            } else {
                self.game_state.effects.angle_scratch.angle(i)
            };
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                angle,
                self.game_state.effects.angle_scratch.radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                ANCILLA31_BYRNA_SPARK_CANE_SPARK_CHAR[i],
                flags | self.game_state.oam.priority_high(),
                0,
            );
            self.ancilla_set_xy(
                k,
                pt.x.wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
                pt.y.wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
            );
            self.ancilla_slot_view_mut(k).set_direction(0);
            // ROM $08:DDBB
            self.ancilla_check_sprite_collision_marked(k);
            oam += 4;
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    fn kill_byrna_spark(&mut self, k: usize) {
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.ancilla_slot_view_mut(k).clear();
        self.follower_link_state_mut().clear_given_damage();
    }

    pub(in crate::zelda_rtl) fn ancilla_add_magic_powder(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut powder = self.ancilla_slot_view_mut(k);
                powder.set_item_to_link(0);
                powder.set_z(0);
                powder.set_aux_timer(1);
            }
            self.follower_link_state_mut()
                .set_link_dma_staging_index(80);
            let j = self.game_state.player.follower_link.facing_index();
            self.ancilla_slot_view_mut(k).set_direction(j as u8);
            let value = MAGIC_POWDER_FRAME_TIMERS[j * 10];
            self.ancilla_slot_view_mut(k).set_work_byte_25(value);
            self.ancilla_set_xy(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(ANCILLA_ADD_MAGIC_POWDER_MAGIC_POWER_X[j] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(ANCILLA_ADD_MAGIC_POWDER_MAGIC_POWER_Y[j] as i16 as u16),
            );
            self.ancilla_check_tile_collision(k);
            let target = NativeTile::from_cartridge(self.ancilla_slot_view(k).tile_attribute());
            self.dungeon_torch_mut().set_target(target);
            if self.game_state.player.follower_link.current_item_active() == 9 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            self.set_sound_effect_1_with_link_pan(0x0d);
            self.ancilla_set_xy(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(ANCILLA_ADD_MAGIC_POWDER_MAGIC_POWER_X1[j] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(ANCILLA_ADD_MAGIC_POWDER_MAGIC_POWER_Y1[j] as i16 as u16),
            );
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_lamp_flame(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            let j = self.game_state.player.follower_link.facing_index();
            {
                let mut flame = self.ancilla_slot_view_mut(k);
                flame.set_item_to_link(0);
                flame.set_aux_timer(0);
                flame.set_timer(23);
                flame.set_direction(j as u8);
            }
            self.ancilla_set_xy(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(ANCILLA_ADD_LAMP_FLAME_LAMP_FLAME_X[j] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(ANCILLA_ADD_LAMP_FLAME_LAMP_FLAME_Y[j] as i16 as u16),
            );
            self.set_sound_effect_1_with_ancilla_pan(k, 42);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_quake_spell(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut quake = self.ancilla_slot_view_mut(k);
                quake.set_step(0);
                quake.set_item_to_link(0);
                quake.set_timer(2);
            }
            self.set_chr_halfslot_request(13);
            self.set_sound_effect_1(0x35);
            for i in 0..5 {
                self.quake_bolt_mut(i).set_phase(0);
            }
            self.quake_spell_scratch_mut().set_active_bolt_limit(0);
            for i in 0..5 {
                self.quake_bolt_mut(i).set_timer(1);
            }
            self.set_custom_spell_animation_active();
            let quake_origin_y = self.game_state.player.follower_link.y().wrapping_add(26);
            let quake_origin_x = self.game_state.player.follower_link.x().wrapping_add(8);
            self.quake_spell_scratch_mut()
                .set_origin(quake_origin_x, quake_origin_y);
            self.quake_spell_scratch_mut().set_screen_shake_y(3);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_ether_spell(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut ether = self.ancilla_slot_view_mut(k);
                ether.set_item_to_link(0);
                ether.set_work_byte_25(0);
                ether.set_step(0);
                ether.set_aux_timer(2);
                ether.set_work_byte_3(3);
                ether.set_y_velocity(127);
            }
            self.set_custom_spell_animation_active();
            self.ether_orbit_mut().set_radius(40);
            self.set_chr_halfslot_request(9);
            self.ether_orbit_mut().set_spin_countdown(0x40);
            self.set_sound_effect_2_with_link_pan(0x26);
            for i in 0..8 {
                self.ether_orbit_mut().set_angle(i, (i * 8) as u8);
            }
            let ether_y = self.game_state.player.follower_link.y();
            let ether_x = self.game_state.player.follower_link.x();
            self.ether_orbit_mut().set_orb_position(ether_x, ether_y);
            let y = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_sub(16);
            self.ether_orbit_mut()
                .initialize_beam_adjusted_y(y & 0x00f0);
            let ether_y2 = ether_y.wrapping_sub(16);
            self.ether_orbit_mut().set_beam_y(ether_y2);
            self.ether_orbit_mut()
                .set_orbit_position(ether_x.wrapping_add(8), ether_y2.wrapping_add(0x24));
            self.ancilla_set_xy(k, ether_x, y);
        }
    }

    pub(super) fn ancilla18_ether_spell(&mut self, k: usize) {
        if self.game_state.frame.submodule != 0 {
            return;
        }

        if self.ancilla_slot_view(k).step() != 0 {
            let flag = if self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                == 0
            {
                self.ancilla_slot_view_mut(k).advance_work_byte_4() & 4 == 0
            } else {
                self.game_state
                    .player
                    .follower_link
                    .spin_animation_step_counter()
                    == 11
            };
            if flag {
                self.palette_electro_themed_gear();
                // ROM $08:AAC9
                self.filter_majorly_whiten_bg_marked();
            } else {
                self.load_actual_gear_palettes();
                self.palette_restore_bg_from_flash();
            }
        }

        if self.ancilla_slot_view(k).step() == 2 {
            if sign8(self.ancilla_slot_view_mut(k).tick_aux_timer()) {
                let mut ether = self.ancilla_slot_view_mut(k);
                ether.set_aux_timer(2);
                if ether.advance_item_to_link() == 2 {
                    ether.retreat_item_to_link();
                    ether.set_x_velocity(16);
                    ether.set_step(3);
                }
            }
            self.ancilla_slot_view_mut(k).add_x_velocity(1);
            self.ether_spell_handle_radial_spin(k);
            return;
        }

        if sign8(self.ancilla_slot_view_mut(k).tick_aux_timer()) {
            let mut ether = self.ancilla_slot_view_mut(k);
            ether.set_aux_timer(2);
            ether.toggle_item_to_link_bit0();
        }
        match self.ancilla_slot_view(k).step() {
            0 => self.ether_spell_handle_lightning_stroke(k),
            1 => self.ether_spell_handle_orb_pulse(k),
            3 => self.ether_spell_handle_radial_spin(k),
            4 => {
                if self.ether_orbit_mut().tick_spin_countdown() == 0 {
                    self.ancilla_slot_view_mut(k).set_step(5);
                }
                self.ether_spell_handle_radial_spin(k);
            }
            _ => {
                let mut vel = self.ancilla_slot_view(k).x_velocity().wrapping_add(0x10);
                if sign8(vel) {
                    vel = 0x7f;
                }
                self.ancilla_slot_view_mut(k).set_x_velocity(vel);
                self.ether_spell_handle_radial_spin(k);
            }
        }
    }

    fn ether_spell_handle_lightning_stroke(&mut self, k: usize) {
        self.ancilla_move_y(k);
        let y = self.ancilla_get_y(k);

        if self.game_state.sprites.ether_orbit.beam_top_bucket() != (y & 0xf0) as u8 {
            self.ether_orbit_mut().set_beam_top_bucket((y & 0xf0) as u8);
            self.ancilla_slot_view_mut(k).advance_work_byte_25();
        }
        if y < 0xe000
            && self.game_state.sprites.ether_orbit.beam_y() < 0xe000
            && self.game_state.sprites.ether_orbit.beam_y() <= y
        {
            self.ancilla_slot_view_mut(k).set_step(1);
        }
        self.ancilla_draw_ether_blitz(k);
    }

    fn ether_spell_handle_orb_pulse(&mut self, k: usize) {
        if !sign8(self.ancilla_slot_view(k).work_byte_25()) {
            if !sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
                self.ancilla_draw_ether_blitz(k);
                return;
            }
            self.ancilla_slot_view_mut(k).set_work_byte_3(3);
            if !sign8(self.ancilla_slot_view_mut(k).retreat_work_byte_25()) {
                self.ancilla_draw_ether_blitz(k);
                return;
            }
            self.ancilla_slot_view_mut(k).set_work_byte_3(9);
        }
        if sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
            {
                let mut ether = self.ancilla_slot_view_mut(k);
                ether.set_step(2);
                ether.set_y_velocity(0);
                ether.set_x_velocity(16);
                ether.set_item_to_link(0);
                ether.set_aux_timer(2);
            }
            if self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                != 0
            {
                self.medallion_check_sprite_damage(k);
            }
        }
        let oam = self.game_state.oam.current_pointer_usize();
        self.ancilla_draw_ether_orb(k, oam);
    }

    fn ether_spell_handle_radial_spin(&mut self, k: usize) {
        if self.ancilla_slot_view(k).step() == 4 {
            if self.game_state.frame.frame_counter & 7 == 0 {
                self.set_sound_effect_2(0x2a);
            } else if self.game_state.frame.frame_counter & 7 == 4 {
                self.set_sound_effect_2(0xaa);
            } else if self.game_state.frame.frame_counter & 7 == 7 {
                self.set_sound_effect_2(0x6a);
            }
        } else {
            let radius = self.game_state.sprites.ether_orbit.radius();
            self.ancilla_slot_view_mut(k).set_x(u16::from(radius));
            self.ancilla_move_x(k);
            let radius = self.ancilla_slot_view(k).x() as u8;
            self.ether_orbit_mut().set_radius(radius);
            if self.game_state.sprites.ether_orbit.radius() == 0x40 {
                self.ancilla_slot_view_mut(k).set_step(4);
            }
        }

        let sb = self.ancilla_slot_view(k).step();
        let sa = self.ancilla_slot_view(k).item_to_link() as usize;
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..=7).rev() {
            let mut angle = self.game_state.sprites.ether_orbit.angle(i);
            if sb != 2 && sb != 5 {
                angle = self.ether_orbit_mut().advance_angle(i);
            }
            let arp = self
                .ancilla_get_radial_projection(angle, self.game_state.sprites.ether_orbit.radius());
            if sb != 2 {
                oam = self.ancilla_draw_ether_blitz_ball(oam, &arp, sa);
            } else {
                oam = self.ancilla_draw_ether_blitz_segment(oam, &arp, sa, i);
            }
        }
        if self.game_state.sprites.ether_orbit.radius() < 0xf0 {
            let oam = self.game_state.oam.current_pointer_usize();
            for i in 0..8 {
                if self.game_state.oam.entry_y(oam + i * 4) != 0xf0 {
                    return;
                }
            }
        }

        self.ancilla_slot_view_mut(k).clear();
        self.set_chr_halfslot_request(1);
        self.follower_link_state_mut()
            .clear_spin_attack_sound_latch();
        self.follower_link_state_mut().clear_state_for_spin_attack();
        self.follower_link_state_mut()
            .clear_spin_animation_step_counter();
        self.follower_link_state_mut().clear_direction_lock();
        self.clear_modal_pause_flag();

        if self.game_state.world.location.overworld_screen_index() == 0x70
            && self.game_state.world.overworld.event_info.event_info(0x70) & 0x20 == 0
            && self.ancilla_check_for_entrance_trigger(2)
        {
            self.set_special_entrance_trigger(3);
            self.set_subsubmodule(0);
            self.scratch_word_mut().clear_module_transition_counter();
        }

        if self.game_state.player.follower_link.handler_state() != 25 {
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            let button_mask_b_y = if self.game_state.player.follower_link.button_b_frames() != 0 {
                self.game_state.player.follower_link.joypad1h_last() & 0x80
            } else {
                0
            };
            self.follower_link_state_mut()
                .set_button_mask_b_y(button_mask_b_y);
        }
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut()
            .clear_magic_spell_player_lock();
        self.load_actual_gear_palettes();
        self.palette_restore_bg_and_hud();
    }

    pub(in crate::zelda_rtl) fn ancilla_add_bombos_spell(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_add_ancilla_bank08(a, y) else {
            return;
        };
        for i in 0..10 {
            self.bombos_fire_column_mut(i).set_phase(0);
            self.bombos_fire_column_mut(i).set_timer(3);
        }
        for i in 0..8 {
            self.bombos_blast_mut(i).set_phase(0);
            self.bombos_blast_mut(i).set_timer(3);
        }
        self.bombos_spell_scratch_mut().set_mode(0);
        self.bombos_spell_scratch_mut()
            .set_blast_release_locked(false);
        self.bombos_spell_scratch_mut()
            .set_blast_release_countdown(0x80);
        self.bombos_fire_column_mut(0).set_radial_angle(0x10);
        self.set_chr_halfslot_request(11);
        self.set_custom_spell_animation_active();
        {
            let mut bombos = self.ancilla_slot_view_mut(k);
            bombos.set_step(0);
            bombos.set_item_to_link(0);
        }
        self.ancilla_sfx2_near(0x2a);

        let mut t = self.asset_u8(72, self.game_state.frame.frame_counter as usize);
        t = if t < 0xe0 { t } else { t & 0x7f };
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        self.bombos_spell_scratch_mut().set_blast_position(
            0,
            (link_x & !0xff) | u16::from(t),
            (link_y & !0xff) | u16::from(t),
        );

        for i in 0..1 {
            let bombos_x_coord2 =
                link_x.wrapping_add(ANCILLA_ADD_BOMBOS_SPELL_BOMBOS_X_DELTA[i] as u16);
            let bombos_y_coord2 =
                link_y.wrapping_add(ANCILLA_ADD_BOMBOS_SPELL_BOMBOS_Y_DELTA[i] as u16);
            self.bombos_spell_scratch_mut()
                .set_fire_column_seed_position(i, bombos_x_coord2, bombos_y_coord2);
            self.bombos_spell_scratch_mut().set_fire_column_radius(16);
            let arp = self.ancilla_get_radial_projection(
                self.game_state
                    .effects
                    .bombos_spell
                    .fire_column(i)
                    .radial_angle(),
                16,
            );
            let x = (if arp.r6 != 0 {
                -(arp.r4 as i32)
            } else {
                arp.r4 as i32
            }) + i32::from(bombos_x_coord2);
            let y = (if arp.r2 != 0 {
                -(arp.r0 as i32)
            } else {
                arp.r0 as i32
            }) + i32::from(bombos_y_coord2);
            self.bombos_fire_column_mut(i)
                .set_position(x as u16, y as u16);
        }
    }

    pub(super) fn ancilla19_bombos_spell(&mut self, k: usize) {
        if self.game_state.effects.bombos_spell.mode() == 0 {
            if self.game_state.frame.submodule == 0 {
                self.bombos_spell_control_fire_columns(k);
                return;
            }
            for i in (0..=9).rev() {
                self.ancilla_draw_bombos_fire_column(i);
            }
        } else if self.game_state.effects.bombos_spell.mode() != 2 {
            if self.game_state.frame.submodule == 0 {
                self.bombos_spell_finish_fire_columns(k);
                return;
            }
            for i in (0..=9).rev() {
                self.ancilla_draw_bombos_fire_column(i);
            }
        } else {
            if self.game_state.frame.submodule == 0 {
                self.bombos_spell_control_blasting(k);
                return;
            }
            let mut i = self.ancilla_slot_view(k).step() as i32;
            loop {
                self.ancilla_draw_bombos_blast(i as usize);
                i -= 1;
                if i < 0 {
                    break;
                }
            }
        }
    }

    fn bombos_spell_control_fire_columns(&mut self, k: usize) {
        let sa = self.ancilla_slot_view(k).item_to_link();
        let mut sb = self.ancilla_slot_view(k).step();

        let mut i = sb as i32;
        loop {
            let ui = i as usize;
            if self.game_state.effects.bombos_spell.fire_column(ui).phase() != 13 {
                let timer = self.bombos_fire_column_mut(ui).tick_timer();
                if sign8(timer) {
                    self.bombos_fire_column_mut(ui).set_timer(3);
                    let phase = self.bombos_fire_column_mut(ui).advance_phase();
                    if phase != 13 {
                        if phase == 2 && sa == 0 {
                            let j = if sb == 9 {
                                let mut found: Option<usize> = None;
                                for candidate in (0..=9).rev() {
                                    if self
                                        .game_state
                                        .effects
                                        .bombos_spell
                                        .fire_column(candidate)
                                        .phase()
                                        == 13
                                    {
                                        self.bombos_fire_column_mut(candidate).set_phase(0);
                                        found = Some(candidate);
                                        break;
                                    }
                                }
                                found.unwrap_or(9)
                            } else {
                                sb = if sb.wrapping_add(1) != 10 {
                                    sb.wrapping_add(1)
                                } else {
                                    9
                                };
                                sb as usize
                            };
                            let radius = self
                                .bombos_spell_scratch_mut()
                                .grow_fire_column_radius(3, 207);
                            let angle = self.bombos_fire_column_mut(0).add_radial_angle(6) & 0x3f;
                            let arp = self.ancilla_get_radial_projection(angle, radius);
                            let x = (if arp.r6 != 0 {
                                -(arp.r4 as i32)
                            } else {
                                arp.r4 as i32
                            }) + i32::from(
                                self.game_state.effects.bombos_spell.fire_column_seed_x(0),
                            );
                            let y = (if arp.r2 != 0 {
                                -(arp.r0 as i32)
                            } else {
                                arp.r0 as i32
                            }) + i32::from(
                                self.game_state.effects.bombos_spell.fire_column_seed_y(0),
                            );
                            self.bombos_fire_column_mut(j)
                                .set_position(x as u16, y as u16);

                            let t = (x as u16)
                                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                                .wrapping_add(8);
                            if t < 256 {
                                self.set_sound_effect_1(
                                    BOMBOS_PANNED_SFX_BITS[(t >> 5) as usize] | 0x2a,
                                );
                            }
                        }
                    } else {
                        i -= 1;
                        if i < 0 {
                            break;
                        }
                        continue;
                    }
                }
                self.ancilla_draw_bombos_fire_column(ui);
            }

            i -= 1;
            if i < 0 {
                break;
            }
        }
        if self
            .game_state
            .effects
            .bombos_spell
            .fire_column(0)
            .radial_angle()
            >= 0x80
        {
            self.bombos_spell_scratch_mut().set_mode(1);
        }
        self.ancilla_slot_view_mut(k).set_step(sb);
    }

    fn bombos_spell_finish_fire_columns(&mut self, kk: usize) {
        let mut k = self.ancilla_slot_view(kk).step() as i32;
        loop {
            let uk = k as usize;
            let timer = self.bombos_fire_column_mut(uk).tick_timer();
            if sign8(timer) {
                self.bombos_fire_column_mut(uk).set_timer(3);
                let phase = self.bombos_fire_column_mut(uk).advance_phase();
                if phase >= 13 {
                    self.bombos_fire_column_mut(uk).set_phase(13);
                }
            }
            self.ancilla_draw_bombos_fire_column(uk);
            k -= 1;
            if k < 0 {
                break;
            }
        }
        for k in (0..=9).rev() {
            if self.game_state.effects.bombos_spell.fire_column(k).phase() != 13 {
                return;
            }
        }
        self.bombos_spell_scratch_mut().set_mode(2);
        self.medallion_check_sprite_damage(kk);
        self.ancilla_slot_view_mut(kk).set_step(0);
    }

    fn bombos_spell_control_blasting(&mut self, kk: usize) {
        let mut k = self.ancilla_slot_view(kk).step() as i32;
        let mut sb = k;
        while k >= 0 {
            let uk = k as usize;
            if self.game_state.effects.bombos_spell.blast(uk).phase() != 8 {
                let timer = self.bombos_blast_mut(uk).tick_timer();
                if sign8(timer) {
                    self.bombos_blast_mut(uk).set_timer(3);
                    let phase = self.bombos_blast_mut(uk).advance_phase();
                    if phase == 1 && !self.game_state.effects.bombos_spell.blast_release_locked() {
                        let mut j = sb;
                        if j != 15 {
                            sb += 1;
                            j = sb;
                        } else {
                            while j >= 0
                                && self
                                    .game_state
                                    .effects
                                    .bombos_spell
                                    .blast(j as usize)
                                    .phase()
                                    != 8
                            {
                                j -= 1;
                            }
                        }
                        let uj = j as usize;
                        self.bombos_blast_mut(uj).set_phase(0);
                        self.bombos_blast_mut(uj).set_timer(3);

                        let idx = (self.game_state.frame.frame_counter & 0x3f) as usize;
                        let y = u16::from(BOMBOS_BLAST_POSITION_TABLE[idx]);
                        let x = u16::from(BOMBOS_BLAST_POSITION_TABLE[idx + 3]);
                        let bg2vofs_copy2 = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
                        let bg2hofs_copy2 = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
                        self.bombos_spell_scratch_mut().set_blast_position(
                            uj,
                            x.wrapping_add(bg2hofs_copy2),
                            y.wrapping_add(bg2vofs_copy2),
                        );
                        let bombos_x = self.game_state.effects.bombos_spell.blast_x(uj);
                        self.set_sound_effect_1(
                            0x0c | BOMBOS_PANNED_SFX_BITS[((bombos_x >> 5) & 7) as usize],
                        );
                    }
                }
            }
            self.ancilla_draw_bombos_blast(uk);
            k -= 1;
        }

        for j in (0..=15).rev() {
            if self.game_state.effects.bombos_spell.blast(j).phase() != 8 {
                self.ancilla_slot_view_mut(kk).set_step(sb as u8);
                if self
                    .bombos_spell_scratch_mut()
                    .tick_blast_release_countdown()
                    == 0
                {
                    self.bombos_spell_scratch_mut()
                        .set_blast_release_countdown(1);
                    self.bombos_spell_scratch_mut()
                        .set_blast_release_locked(true);
                }
                return;
            }
        }
        self.ancilla_slot_view_mut(kk).clear();
        self.set_chr_halfslot_request(1);
        self.follower_link_state_mut()
            .clear_spin_attack_sound_latch();
        self.follower_link_state_mut().clear_state_for_spin_attack();
        self.follower_link_state_mut()
            .clear_spin_animation_step_counter();
        self.follower_link_state_mut().clear_direction_lock();
        self.clear_modal_pause_flag();
        if self.game_state.player.follower_link.handler_state() != 26 {
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            let button_mask_b_y = if self.game_state.player.follower_link.button_b_frames() != 0 {
                self.game_state.player.follower_link.joypad1h_last() & 0x80
            } else {
                0
            };
            self.follower_link_state_mut()
                .set_button_mask_b_y(button_mask_b_y);
        }
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut()
            .clear_magic_spell_player_lock();

        if self
            .bombos_spell_scratch_mut()
            .tick_blast_release_countdown()
            == 0
        {
            self.bombos_spell_scratch_mut()
                .set_blast_release_countdown(1);
            self.bombos_spell_scratch_mut()
                .set_blast_release_locked(true);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_somaria_block(&mut self, ty: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_add_add_ancilla_bank08(ty, y)?;
        for j in (0..=4).rev() {
            if j == k || self.ancilla_slot_view(j).ancilla_type() != 0x2c {
                continue;
            }
            if j == self
                .game_state
                .player
                .follower_link
                .ancilla_pickup_flag()
                .wrapping_sub(1) as usize
            {
                self.follower_link_state_mut().clear_ancilla_pickup_flag();
            }
            self.ancilla_add_exploding_somaria_block(j);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            self.dungeon_environment_mut()
                .clear_somaria_block_switch_counter();
            if self.game_state.player.follower_link.speed_setting() == 0x12 {
                self.follower_link_state_mut().clear_defense_flags();
                self.follower_link_state_mut().set_speed_setting(0);
            }
            return Some(k);
        }

        self.ancilla_sfx3_near(0x2a);
        {
            let mut block = self.ancilla_slot_view_mut(k);
            block.set_step(0);
            block.set_y_velocity(0);
            block.set_x_velocity(0);
            block.set_item_to_link(0);
            block.set_aux_timer(0);
            block.set_work_byte_3(0);
            block.set_timer(18);
            block.set_z(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_h(value);
        let value = 12;
        self.ancilla_slot_view_mut(k).set_g(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_4(value);
        let value = 9;
        self.ancilla_slot_view_mut(k).set_s_player(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_t_player(value);
        let direction = self.game_state.player.follower_link.facing() >> 1;
        self.ancilla_slot_view_mut(k).set_direction(direction);
        if self.ancilla_check_initial_tile_collision_class2(k) {
            self.ancilla_set_xy(
                k,
                self.game_state.player.follower_link.x().wrapping_add(8),
                self.game_state.player.follower_link.y().wrapping_add(16),
            );
        } else {
            let j = self.game_state.player.follower_link.facing_index();
            self.ancilla_set_xy(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(CANE_OF_SOMARIA_PLACEMENT_X_OFFSETS[j] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(CANE_OF_SOMARIA_PLACEMENT_Y_OFFSETS[j] as i16 as u16),
            );
            self.somaria_block_check_for_transit_tile(k);
        }
        Some(k)
    }

    pub(super) fn ancilla2_d_somaria_block_fizz(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if (aux_timer as i8) < 0 {
            let item_to_link = {
                let mut fizzle = self.ancilla_slot_view_mut(k);
                fizzle.set_aux_timer(3);
                fizzle.advance_item_to_link()
            };
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut z = self.ancilla_slot_view(k).z();
        if z == 0xff {
            z = 0;
        }
        let y = y.wrapping_sub(z as i8 as i16 as u16);
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 2;
        for _ in 0..2 {
            if ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_CHAR[j] != 0xff {
                // $08:EA54 JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    x.wrapping_add(
                        ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_X[j] as i16 as u16,
                    ),
                    y.wrapping_add(
                        ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_Y[j] as i16 as u16,
                    ),
                    ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_CHAR[j],
                    ANCILLA2_D_SOMARIA_BLOCK_FIZZ_SOMARIA_BLOCK_FIZZLE_FLAGS[j] & !0x30
                        | self.game_state.oam.priority_high(),
                    0,
                );
            }
            j += 1;
            oam += 4;
        }
    }

    pub(super) fn ancilla39_somaria_platform_poof(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if (aux_timer as i8) >= 0 {
            return;
        }
        self.ancilla_slot_view_mut(k).clear();
        let x = self.ancilla_get_x(k) & !7 | 4;
        let y = self.ancilla_get_y(k) & !7 | 4;
        let floor = self.ancilla_slot_view(k).floor();
        if let Some((j, _)) = self.spawn_sprite_dynamically(k, 0xed) {
            self.follower_link_state_mut()
                .clear_somaria_platform_state();
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);

            let pos = (((x & 0x01f8) >> 3) + ((y & 0x01f8) << 3)) as usize
                + if floor >= 1 { 0x1000 } else { 0 };

            // Count closed sides (up, down, left) until the first open pipe.
            let is_pipe = |state: &Self, pos: usize| {
                matches!(
                    state
                        .game_state
                        .dungeon
                        .bg2_attributes
                        .bg2_tile(pos)
                        .dungeon_role(),
                    DungeonRole::SomariaPipe { .. }
                )
            };
            let mut t = 0usize;
            if !is_pipe(self, pos.wrapping_sub(0x40)) {
                t += 1;
                if !is_pipe(self, pos + 0x40) {
                    t += 1;
                    if !is_pipe(self, pos.wrapping_sub(1)) {
                        t += 1;
                    }
                }
            }
            let value =
                ANCILLA39_SOMARIA_PLATFORM_POOF_SOMARIAN_PLATFORM_POOF_DIRECTION_BY_OPEN_SIDE[t];
            self.sprite_slot_view_mut(j).set_direction(value);
            self.sprite_slot_view_mut(j).set_floor(0);
        } else {
            self.ancilla_draw_somaria_block(k);
        }
    }

    pub(super) fn ancilla2_c_somaria_block(&mut self, k: usize) {
        if !sign8(self.ancilla_slot_view_mut(k).tick_g()) {
            return;
        }
        self.ancilla_slot_view_mut(k).set_g(0);

        if self.ancilla_slot_view(k).h() == 0 {
            if matches!(self.game_state.frame.submodule, 0 | 8 | 16) {
                self.ancilla_handle_lift_logic(k);
            } else if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                && self.ancilla_slot_view(k).k() != 0
            {
                if self.ancilla_slot_view(k).k() != 3 {
                    self.ancilla_latch_link_coordinates(k, 3);
                    self.ancilla_latch_altitude_above_link(k);
                    self.ancilla_slot_view_mut(k).set_k(3);
                }
                self.ancilla_latch_carried_position(k);
            }
            if self.game_state.world.location.is_indoors() {
                if self.ancilla_slot_view(k).k() == 0
                    && !self
                        .game_state
                        .player
                        .follower_link
                        .is_lifting_or_carrying()
                    && (self.ancilla_slot_view(k).z() == 0 || self.ancilla_slot_view(k).z() == 0xff)
                {
                    if self
                        .game_state
                        .dungeon
                        .environment
                        .somaria_block_bg_check_flag()
                        != 0
                    {
                        let mut j = (self.game_state.frame.frame_counter & 3) as usize;
                        loop {
                            let bak = self.ancilla_slot_view(k).object_priority();
                            let x = self.ancilla_get_x(k).wrapping_add(
                                ANCILLA2_C_SOMARIA_BLOCK_SOMARIAN_BLOCK_COLL_X[j] as i16 as u16,
                            );
                            let y = self.ancilla_get_y(k).wrapping_add(
                                ANCILLA2_C_SOMARIA_BLOCK_SOMARIAN_BLOCK_COLL_Y[j] as i16 as u16,
                            );
                            self.ancilla_check_tile_collision_targeted(k, x, y);
                            self.ancilla_slot_view_mut(k).set_object_priority(bak);
                            if matches!(self.ancilla_slot_view(k).tile_attribute(), 0xb6 | 0xbc) {
                                self.ancilla_set_xy(k, x, y);
                                self.ancilla_add_somaria_platform_poof(k);
                                if k + 1
                                    == self.game_state.player.follower_link.ancilla_pickup_flag()
                                        as usize
                                {
                                    self.follower_link_state_mut().clear_ancilla_pickup_flag();
                                }
                                return;
                            }
                            j += 4;
                            if j >= 12 {
                                break;
                            }
                        }
                    } else if !self.somaria_block_check_for_switch(k)
                        && (self.ancilla_slot_view(k).z() == 0
                            || self.ancilla_slot_view(k).z() == 0xff)
                    {
                        self.dungeon_environment_mut()
                            .increment_somaria_block_switch_counter();
                    }
                } else if self.game_state.player.follower_link.ancilla_pickup_flag() == k as u8 + 1
                {
                    self.dungeon_environment_mut()
                        .clear_somaria_block_switch_counter();
                }
            }
        } else if self.game_state.world.location.is_indoors()
            && self.game_state.player.follower_link.ancilla_pickup_flag() == k as u8 + 1
        {
            self.dungeon_environment_mut()
                .clear_somaria_block_switch_counter();
        }

        let mut old_y = self.ancilla_latch_y_coord_to_z(k);
        let s1a = self.ancilla_slot_view(k).direction();
        let mut s1b = self.ancilla_slot_view(k).object_priority();
        self.ancilla_slot_view_mut(k).set_object_priority(0);
        // $08:E463 JSR $08:8BCF; full collision body, call opcode separate.
        let mut flag = self.ancilla_check_tile_collision_class2(k, AncillaClass2Entry::Rom088BCF);

        if self.game_state.world.location.is_indoors()
            && self.ancilla_slot_view(k).l() != 0
            && self.ancilla_slot_view(k).tile_attribute() == 0x1c
        {
            let value = 1;
            self.ancilla_slot_view_mut(k).set_t_player(value);
        }

        loop {
            if flag
                && (!self
                    .game_state
                    .player
                    .follower_link
                    .is_lifting_or_carrying()
                    || self
                        .game_state
                        .player
                        .follower_link
                        .has_picking_throw_state())
            {
                if s1b == 0
                    && self.ancilla_slot_view(k).work_byte_4() == 0
                    && self.ancilla_slot_view(k).z() != 0
                {
                    self.ancilla_slot_view_mut(k).set_work_byte_4(1);
                    let qq = if self.ancilla_slot_view(k).direction() == 1 {
                        16
                    } else {
                        4
                    };
                    let y_velocity = self.ancilla_slot_view(k).y_velocity();
                    if y_velocity != 0 {
                        self.ancilla_slot_view_mut(k)
                            .set_y_velocity(if sign8(y_velocity) {
                                qq
                            } else {
                                (-(qq as i8)) as u8
                            });
                    }
                    let x_velocity = self.ancilla_slot_view(k).x_velocity();
                    if x_velocity != 0 {
                        self.ancilla_slot_view_mut(k)
                            .set_x_velocity(if sign8(x_velocity) { 4 } else { (-4i8) as u8 });
                    }
                    if self.ancilla_slot_view(k).direction() == 1
                        && self.ancilla_slot_view(k).z() != 0
                    {
                        self.ancilla_slot_view_mut(k).set_y_velocity((-4i8) as u8);
                        self.ancilla_slot_view_mut(k).set_l(2);
                    }
                }
            } else if !self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
                && (self.ancilla_slot_view(k).z() == 0 || self.ancilla_slot_view(k).z() == 0xff)
            {
                self.ancilla_slot_view_mut(k).set_direction(16);
                let bak0 = self.ancilla_slot_view(k).object_priority();
                self.ancilla_check_tile_collision(k);
                self.ancilla_slot_view_mut(k).set_object_priority(bak0);
                let a = self.ancilla_slot_view(k).tile_attribute();
                if a == 0x26 {
                    flag = true;
                    continue;
                } else if a == 0x0c || a == 0x1c {
                    if self.game_state.dungeon.room_load.header_collision() != 3 {
                        if self.ancilla_slot_view(k).floor() == 0
                            && self.ancilla_slot_view(k).z() != 0
                            && self.ancilla_slot_view(k).z() != 0xff
                        {
                            self.ancilla_slot_view_mut(k).set_floor(1);
                        }
                    } else {
                        old_y = self
                            .ancilla_get_y(k)
                            .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_velocity());
                        self.ancilla_set_x(
                            k,
                            self.ancilla_get_x(k).wrapping_add(
                                self.game_state.dungeon.moving_floor.floor_x_velocity(),
                            ),
                        );
                    }
                } else if a == 0x20 || (a & 0xf0) == 0xb0 && a != 0xb6 && a != 0xbc {
                    if !self
                        .game_state
                        .player
                        .follower_link
                        .is_lifting_or_carrying()
                    {
                        if k + 1
                            == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                        {
                            self.follower_link_state_mut().clear_ancilla_pickup_flag();
                        }
                        if self.ancilla_slot_view(k).timer() == 0 {
                            if self.game_state.player.follower_link.speed_setting() == 18 {
                                self.follower_link_state_mut().set_speed_setting(0);
                                self.follower_link_state_mut().clear_defense_flags();
                            }
                            self.ancilla_slot_view_mut(k).clear();
                            return;
                        }
                    }
                } else if a == 8 {
                    if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                    {
                        self.follower_link_state_mut().clear_ancilla_pickup_flag();
                    }
                    if self.ancilla_slot_view(k).timer() == 0 {
                        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_sub(24));
                        self.ancilla_transmute_to_splash(k);
                        return;
                    }
                } else if matches!(a, 0x68..=0x6b) {
                    self.ancilla_apply_conveyor(k);
                    old_y = self.ancilla_get_y(k);
                } else {
                    let timer =
                        if (self.ancilla_slot_view(k).l() | self.ancilla_slot_view(k).h()) != 0 {
                            0
                        } else {
                            2
                        };
                    self.ancilla_slot_view_mut(k).set_timer(timer);
                }
            }
            break;
        }

        s1b |= self.ancilla_slot_view(k).object_priority();

        if !self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            && self.ancilla_slot_view_mut(k).tick_s_player() == 0
        {
            self.ancilla_slot_view_mut(k).set_s_player(1);
            self.ancilla_slot_view_mut(k).set_object_priority(0);
            if self.ancilla_check_basic_sprite_collision(k).is_some() {
                self.ancilla_slot_view_mut(k).set_s_player(7);
                if self.ancilla_slot_view_mut(k).advance_step() == 5 {
                    self.somaria_block_fizzle_away(k);
                    return;
                }
            }
        }
        self.ancilla_set_y(k, old_y);
        self.ancilla_slot_view_mut(k).set_direction(s1a);
        self.ancilla_slot_view_mut(k).set_object_priority(s1b);

        self.ancilla_draw_somaria_block(k);
    }

    fn quake_spell_shake_screen(&mut self, _k: usize) {
        let shake_y = self.quake_spell_scratch_mut().invert_screen_shake_y();
        self.set_bg1_y_offset(shake_y);
        self.follower_link_state_mut()
            .add_y_velocity_delta(shake_y as u8);
    }

    pub(super) fn ancilla1_c_quake_spell(&mut self, k: usize) {
        if self.game_state.frame.submodule != 0 {
            if self.game_state.effects.quake_bolts.slot(4).phase() != QUAKE_BOLT_TARGET_PHASES[4] {
                self.ancilla_draw_quake_initial_bolts(k);
            }
            return;
        }
        if self.ancilla_slot_view(k).step() != 2 {
            self.quake_spell_shake_screen(k);
            self.quake_spell_control_bolts(k);
            self.quake_spell_spread_bolts(k);
            return;
        }
        self.medallion_check_sprite_damage(k);
        self.prepare_apply_rumble_to_sprites();
        self.ancilla_slot_view_mut(k).clear();
        self.follower_link_state_mut().clear_handler_state();
        self.set_chr_halfslot_request(1);
        self.follower_link_state_mut()
            .clear_spin_attack_sound_latch();
        self.follower_link_state_mut().clear_state_for_spin_attack();
        self.follower_link_state_mut()
            .clear_spin_animation_step_counter();
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.clear_modal_pause_flag();
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
        if self.game_state.world.location.overworld_screen_index() == 0x47
            && self.game_state.world.overworld.event_info.event_info(0x47) & 0x20 == 0
            && self.ancilla_check_for_entrance_trigger(3)
        {
            self.set_special_entrance_trigger(4);
            self.set_subsubmodule(0);
            self.scratch_word_mut().clear_module_transition_counter();
        }
        let button_mask_b_y = if self.game_state.player.follower_link.button_b_frames() != 0 {
            self.game_state.player.follower_link.joypad1h_last() & 0x80
        } else {
            0
        };
        self.follower_link_state_mut()
            .set_button_mask_b_y(button_mask_b_y);
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut()
            .clear_magic_spell_player_lock();
    }

    fn quake_spell_control_bolts(&mut self, k: usize) {
        let pending_step = self.ancilla_slot_view(k).step();
        self.quake_spell_scratch_mut()
            .set_pending_step(pending_step);
        let mut j = self.game_state.effects.quake_spell.active_bolt_limit() as i32;
        loop {
            let uj = j as usize;
            if self.game_state.effects.quake_bolts.slot(uj).phase() != QUAKE_BOLT_TARGET_PHASES[uj]
            {
                let timer = self.quake_bolt_mut(uj).tick_timer();
                if sign8(timer) {
                    self.quake_bolt_mut(uj).set_timer(1);
                    let phase = self.quake_bolt_mut(uj).advance_phase();
                    if phase != QUAKE_BOLT_TARGET_PHASES[uj] {
                        if j == 0 && phase == 2 {
                            self.ancilla_sfx2_near(0x0c);
                            self.quake_spell_scratch_mut().set_active_bolt_limit(1);
                        } else if j == 1 && phase == 2 {
                            self.quake_spell_scratch_mut().set_active_bolt_limit(4);
                        } else if j == 4 && phase == 7 {
                            self.quake_spell_scratch_mut().set_pending_step(1);
                        }
                        self.ancilla_draw_quake_initial_bolts(uj);
                    }
                } else {
                    self.ancilla_draw_quake_initial_bolts(uj);
                }
            }
            j -= 1;
            if j < 0 {
                break;
            }
        }
        let step = self.game_state.effects.quake_spell.pending_step();
        self.ancilla_slot_view_mut(k).set_step(step);
    }

    fn ancilla_draw_quake_initial_bolts(&mut self, k: usize) {
        let t = self
            .game_state
            .effects
            .quake_bolts
            .slot(k)
            .phase()
            .wrapping_add(ANCILLA_DRAW_QUAKE_INITIAL_BOLTS_QUAKE_GROUND_BOLT_OAM_STARTS[k])
            as usize;
        let mut oam = self.game_state.oam.current_pointer_usize();
        let idx = QUAKE_INITIAL_BOLT_FRAME_RANGES[t] as usize;
        let end = QUAKE_INITIAL_BOLT_FRAME_RANGES[t + 1] as usize;
        for sprite in &QUAKE_INITIAL_BOLT_SPRITES[idx..end] {
            let x = self
                .game_state
                .effects
                .quake_spell
                .origin_x()
                .wrapping_add(sprite.x as u16)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y = self
                .game_state
                .effects
                .quake_spell
                .origin_y()
                .wrapping_add(sprite.y as u16)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());

            let mut xval = self.game_state.oam.entry_x(oam);
            let mut yval = 0xf0;
            if x < 256 && y < 256 {
                xval = x as u8;
                if y < 0xf0 {
                    yval = y as u8;
                }
            }
            self.oam_state_mut().write_entry(
                oam,
                xval,
                yval,
                QUAKE_GROUND_BOLT_CHARS[(sprite.flags & 0x0f) as usize],
                (sprite.flags & 0xc0) | 0x3c,
            );
            let value = 2;
            self.oam_state_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam += 4;
            let cur = self.game_state.oam.current_pointer().wrapping_add(4);
            let ext = self
                .game_state
                .oam
                .current_extended_pointer()
                .wrapping_add(1);
            self.oam_state_mut().set_current_pointer(cur);
            self.oam_state_mut().set_current_extended_pointer(ext);
        }
    }

    fn quake_spell_spread_bolts(&mut self, k: usize) {
        if self.ancilla_slot_view(k).step() != 1 {
            return;
        }
        if self.ancilla_slot_view(k).timer() == 0 {
            let mut quake = self.ancilla_slot_view_mut(k);
            quake.set_timer(2);
            let spread_phase = quake.advance_item_to_link();
            if spread_phase == 55 {
                quake.set_step(2);
                return;
            }
        }
        let t = self.ancilla_slot_view(k).item_to_link() as usize;
        let idx = QUAKE_SPREAD_BOLT_FRAME_RANGES[t] as usize;
        let end = QUAKE_SPREAD_BOLT_FRAME_RANGES[t + 1] as usize;
        let mut oam = self.game_state.oam.current_pointer_usize();
        for sprite in &QUAKE_SPREAD_BOLT_SPRITES[idx..end] {
            self.oam_state_mut().write_entry(
                oam,
                sprite.x as u8,
                sprite.y as u8,
                QUAKE_GROUND_BOLT_CHARS[(sprite.flags & 0x0f) as usize],
                (sprite.flags & 0xc0) | 0x3c,
            );
            let value = (sprite.flags >> 4) & 3;
            self.oam_state_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            let cur = self.game_state.oam.current_pointer().wrapping_add(4);
            let ext = self
                .game_state
                .oam
                .current_extended_pointer()
                .wrapping_add(1);
            self.oam_state_mut().set_current_pointer(cur);
            self.oam_state_mut().set_current_extended_pointer(ext);
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
        }
    }

    fn ancilla_draw_ether_blitz_ball(
        &mut self,
        oam: usize,
        arp: &AncillaRadialProjection,
        s: usize,
    ) -> usize {
        let x = self
            .game_state
            .sprites
            .ether_orbit
            .orbit_x()
            .wrapping_add(if arp.r6 != 0 {
                0u16.wrapping_sub(arp.r4 as u16)
            } else {
                arp.r4 as u16
            })
            .wrapping_sub(8)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .game_state
            .sprites
            .ether_orbit
            .orbit_y()
            .wrapping_add(if arp.r2 != 0 {
                0u16.wrapping_sub(arp.r0 as u16)
            } else {
                arp.r0 as u16
            })
            .wrapping_sub(8)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        // $08:AD27 JSR F6E1: complete XY body, once per executed draw.
        self.ancilla_set_oam_marked(
            oam,
            x,
            y,
            ANCILLA_DRAW_ETHER_BLITZ_BALL_ETHER_BLITZ_BALL_CHAR[s],
            0x3c,
            2,
        );
        self.ancilla_allocate_oam_from_custom_region(oam + 4)
    }

    fn ancilla_draw_ether_blitz_segment(
        &mut self,
        oam: usize,
        arp: &AncillaRadialProjection,
        s: usize,
        k: usize,
    ) -> usize {
        let x = if arp.r6 != 0 {
            0u16.wrapping_sub(arp.r4 as u16)
        } else {
            arp.r4 as u16
        };
        let y = if arp.r2 != 0 {
            0u16.wrapping_sub(arp.r0 as u16)
        } else {
            arp.r0 as u16
        };
        let t = s * 8 + k;
        let base_x = x
            .wrapping_add(self.game_state.sprites.ether_orbit.orbit_x())
            .wrapping_sub(8)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let base_y = y
            .wrapping_add(self.game_state.sprites.ether_orbit.orbit_y())
            .wrapping_sub(8)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        self.ancilla_set_oam(
            oam,
            base_x,
            base_y,
            ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_CHAR[t * 2],
            ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS[t * 2],
            2,
        );
        self.ancilla_set_oam(
            oam + 4,
            x.wrapping_add(self.game_state.sprites.ether_orbit.orbit_x())
                .wrapping_add(
                    ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_X[t] as i16
                        as u16,
                )
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
            y.wrapping_add(self.game_state.sprites.ether_orbit.orbit_y())
                .wrapping_add(
                    ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_Y[t] as i16
                        as u16,
                )
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
            ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_CHAR[t * 2 + 1],
            ANCILLA_DRAW_ETHER_BLITZ_SEGMENT_ETHER_SPLITTING_BLITZ_SEGMENT_FLAGS[t * 2 + 1],
            2,
        );
        self.ancilla_allocate_oam_from_custom_region(oam + 8)
    }

    fn ancilla_draw_ether_blitz(&mut self, k: usize) {
        let (x, mut y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let t = self.ancilla_slot_view(k).item_to_link() as usize;
        let mut i = self.ancilla_slot_view(k).work_byte_25();
        let mut m = 0usize;
        loop {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ANCILLA_DRAW_ETHER_BLITZ_ETHER_BLITZ_SEGMENT_CHAR[t * 2 + m],
                ANCILLA_DRAW_ETHER_BLITZ_ETHER_BLITZ_ORB_FLAGS[0]
                    | self.game_state.oam.priority_high(),
                2,
            );
            y = y.wrapping_sub(16);
            oam += 4;
            m ^= 1;
            i = i.wrapping_sub(1);
            if sign8(i) {
                break;
            }
        }
        if self.ancilla_slot_view(k).step() == 1 {
            self.ancilla_draw_ether_orb(k, oam);
        }
    }

    fn ancilla_draw_ether_orb(&mut self, k: usize, mut oam: usize) {
        let mut y = self
            .game_state
            .sprites
            .ether_orbit
            .orb_y()
            .wrapping_sub(1)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let mut x = self
            .game_state
            .sprites
            .ether_orbit
            .orb_x()
            .wrapping_sub(8)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let t = self.ancilla_slot_view(k).item_to_link() as usize * 4;

        for i in 0..4 {
            self.ancilla_set_oam(
                oam,
                x,
                y,
                ANCILLA_DRAW_ETHER_ORB_ETHER_BLITZ_ORB_CHAR[t + i],
                ANCILLA_DRAW_ETHER_ORB_ETHER_BLITZ_ORB_FLAGS[t + i],
                2,
            );
            oam += 4;
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            x = x.wrapping_add(16);
            if i == 1 {
                x = x.wrapping_sub(32);
                y = y.wrapping_add(16);
            }
        }
    }

    fn ancilla_draw_bombos_fire_column(&mut self, kk: usize) {
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(kk, 0x10);
        let mut oam = self.game_state.oam.current_pointer_usize();
        for _ in 0..1 {
            let mut k = self.game_state.effects.bombos_spell.fire_column(kk).phase() as usize;
            if k == 13 {
                continue;
            }
            k = k * 3 + 2;
            for _ in 0..3 {
                if ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_CHAR[k] != 0xff {
                    let x = self.game_state.effects.bombos_spell.fire_column(kk).x();
                    let y = self.game_state.effects.bombos_spell.fire_column(kk).y();
                    // $08:B3D6 JSR F6E1: complete XY body, once per executed draw.
                    self.ancilla_set_oam_marked(
                        oam,
                        x.wrapping_add(
                            ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_X[k] as i16
                                as u16,
                        )
                        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
                        y.wrapping_add(
                            ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_Y[k] as i16
                                as u16,
                        )
                        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
                        ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_CHAR[k],
                        ANCILLA_DRAW_BOMBOS_FIRE_COLUMN_BOMBOS_SPELL_FIRE_COLUMN_FLAGS[k],
                        2,
                    );
                    oam += 4;
                }
                oam = self.ancilla_allocate_oam_from_custom_region(oam);
                k = k.wrapping_sub(1);
            }
        }
    }

    fn ancilla_draw_bombos_blast(&mut self, k: usize) {
        let x = self.game_state.effects.bombos_spell.blast_x(k);
        let y = self.game_state.effects.bombos_spell.blast_y(k);
        if self.game_state.effects.bombos_spell.blast(k).phase() == 8 {
            return;
        }

        self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 0x10);
        let mut oam = self.game_state.oam.current_pointer_usize();

        let mut t = self.game_state.effects.bombos_spell.blast(k).phase() as usize * 4 + 3;
        for _ in 0..4 {
            if ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_CHAR[t] != 0xff {
                // $08:B644 JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    x.wrapping_add(
                        ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_X[t] as i16 as u16,
                    )
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
                    y.wrapping_add(
                        ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_Y[t] as i16 as u16,
                    )
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
                    ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_CHAR[t],
                    ANCILLA_DRAW_BOMBOS_BLAST_BOMBOS_SPELL_DRAW_BLAST_FLAGS[t],
                    2,
                );
                oam += 4;
            }
            oam = self.ancilla_allocate_oam_from_custom_region(oam);
            t = t.wrapping_sub(1);
        }
    }

    pub(super) fn ancilla_magic_powder_draw(&mut self, k: usize) {
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let b = self.ancilla_slot_view(k).work_byte_25() as usize;
        let mut j = b * 4;
        for _ in 0..4 {
            // $08:BB28 JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x.wrapping_add(ANCILLA_MAGIC_POWDER_DRAW_MAGIC_POWDER_DRAW_X[j] as i16 as u16),
                y.wrapping_add(ANCILLA_MAGIC_POWDER_DRAW_MAGIC_POWDER_DRAW_Y[j] as i16 as u16),
                ANCILLA_MAGIC_POWDER_DRAW_MAGIC_POWDER_DRAW_CHAR[b],
                ANCILLA_MAGIC_POWDER_DRAW_MAGIC_POWDER_DRAW_FLAGS[j] & !0x30
                    | self.game_state.oam.priority_high(),
                0,
            );
            oam += 4;
            j += 1;
        }
    }

    pub(super) fn ancilla1_a_powder_dust(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            self.powder_apply_damage_to_sprites(k);
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(self.ancilla_slot_view(k).aux_timer()) {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                let j = self.ancilla_slot_view(k).direction() as usize;
                if self.ancilla_slot_view(k).item_to_link() == 9 {
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                    self.dungeon_torch_mut().clear_target();
                    return;
                }
                let value = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                let value = MAGIC_POWDER_FRAME_TIMERS
                    [self.ancilla_slot_view(k).item_to_link() as usize + j * 10];
                self.ancilla_slot_view_mut(k).set_work_byte_25(value);
            }
        }
        self.ancilla_allocate_oam_from_region_b_or_e(self.ancilla_slot_view(k).num_sprites());
        self.ancilla_magic_powder_draw(k);
    }

    fn powder_apply_damage_to_sprites(&mut self, k: usize) {
        for j in (0..16).rev() {
            if ((self.game_state.frame.frame_counter ^ j as u8) & 3) != 0
                || self.sprite_slot_view(j).state() != 9
                || (self.sprite_slot_view(j).bump_damage() & 0x20) != 0
            {
                continue;
            }

            let mut hb = self.ancilla_setup_basic_hit_box(k);
            self.sprite_setup_hit_box(j, &mut hb, SpriteHitBoxEntry::Unpriced);
            if !self.check_if_hit_boxes_overlap(&hb) {
                continue;
            }

            let mut a = self.sprite_slot_view(j).sprite_type();
            if a != 0x0b
                || {
                    a = self.game_state.world.location.indoor_flag();
                    a == 0
                }
                || {
                    a = self
                        .game_state
                        .dungeon
                        .room_tracking
                        .room_index2()
                        .wrapping_sub(1);
                    a != 0
                }
            {
                if a != 0x0d {
                    self.ancilla_check_damage_to_sprite_preset(j, 10);
                    continue;
                }
                if self.sprite_slot_view(j).head_direction() != 0 {
                    continue;
                }
            }
            let value = 1;
            self.sprite_slot_view_mut(j).set_head_direction(value);
            self.sprite_spawn_poof_garnish(j);
        }
    }

    pub(super) fn ancilla2_e_somaria_block_fission(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if (aux_timer as i8) < 0 {
            let item_to_link = {
                let mut fission = self.ancilla_slot_view_mut(k);
                fission.set_aux_timer(3);
                fission.advance_item_to_link()
            };
            if item_to_link == 2 {
                self.ancilla_slot_view_mut(k).clear();
                self.somaria_block_spawn_bullets(k);
                return;
            }
        }
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();

        let z = self.ancilla_slot_view(k).z().wrapping_add(
            if self.ancilla_slot_view(k).k() == 3
                && self.game_state.player.follower_link.z() as u8 != 0xff
            {
                self.game_state.player.follower_link.z() as u8
            } else {
                0
            },
        );
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 8;
        for _ in 0..8 {
            // $08:EBB6 JSR F6E1: complete XY body, once per executed draw.
            self.ancilla_set_oam_marked(
                oam,
                x.wrapping_add(
                    ANCILLA2_E_SOMARIA_BLOCK_FISSION_SOMARIAN_BLOCK_DIVIDE_X[j] as i16 as u16,
                ),
                y.wrapping_add(
                    ANCILLA2_E_SOMARIA_BLOCK_FISSION_SOMARIAN_BLOCK_DIVIDE_Y[j] as i16 as u16,
                )
                .wrapping_sub(z as i8 as i16 as u16),
                ANCILLA2_E_SOMARIA_BLOCK_FISSION_SOMARIAN_BLOCK_DIVIDE_CHAR[j],
                ANCILLA2_E_SOMARIA_BLOCK_FISSION_SOMARIAN_BLOCK_DIVIDE_FLAGS[j] & !0x30
                    | self.game_state.oam.priority_high(),
                0,
            );
            j += 1;
            oam += 4;
        }
    }

    pub(super) fn ancilla2_f_lamp_flame(&mut self, k: usize) {
        let (x, y) = self.ancilla_prep_adjusted_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        if self.ancilla_slot_view(k).timer() == 0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            return;
        }
        let mut j = ((self.ancilla_slot_view(k).timer() & 0xf8) >> 1) as usize;
        loop {
            if ANCILLA2_F_LAMP_FLAME_LAMP_FLAME_DRAW_CHAR[j] != 0xff {
                // $08:EC51 JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    x.wrapping_add(ANCILLA2_F_LAMP_FLAME_LAMP_FLAME_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(ANCILLA2_F_LAMP_FLAME_LAMP_FLAME_DRAW_Y[j] as i16 as u16),
                    ANCILLA2_F_LAMP_FLAME_LAMP_FLAME_DRAW_CHAR[j],
                    self.game_state.oam.priority_high() | 2,
                    0,
                );
                oam += 4;
            }
            j += 1;
            if j & 3 == 0 {
                break;
            }
        }
    }

    fn somaria_block_check_for_transit_tile(&mut self, k: usize) {
        if self
            .game_state
            .dungeon
            .environment
            .somaria_block_bg_check_flag()
            == 0
        {
            return;
        }
        for j in (0..=11).rev() {
            let x = self.ancilla_get_x(k).wrapping_add(
                SOMARIA_BLOCK_CHECK_FOR_TRANSIT_TILE_SOMARIA_TRANSIT_LINE_X[j] as i16 as u16,
            );
            let y = self.ancilla_get_y(k).wrapping_add(
                SOMARIA_BLOCK_CHECK_FOR_TRANSIT_TILE_SOMARIA_TRANSIT_LINE_Y[j] as i16 as u16,
            );
            let bak = self.ancilla_slot_view(k).object_priority();
            self.ancilla_check_tile_collision_targeted(k, x, y);
            let value = bak;
            self.ancilla_slot_view_mut(k).set_object_priority(value);
            if matches!(self.ancilla_slot_view(k).tile_attribute(), 0xb6 | 0xbc) {
                self.ancilla_set_xy(k, x, y);
                self.ancilla_add_somaria_platform_poof(k);
                return;
            }
        }
    }

    fn ancilla_add_somaria_platform_poof(&mut self, k: usize) {
        {
            let mut poof = self.ancilla_slot_view_mut(k);
            poof.set_ancilla_type(0x39);
            poof.set_aux_timer(7);
        }
        for j in (0..=15).rev() {
            if self.sprite_slot_view(j).sprite_type() == 0xed {
                let value = 0;
                self.sprite_slot_view_mut(j).set_state(value);
                self.follower_link_state_mut()
                    .clear_somaria_platform_state();
            }
        }
        self.player_tile_detect_nearby();
    }

    fn ancilla_add_exploding_somaria_block(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).set_ancilla_type(0x2e);
        let value = ANCILLA_DRAW_SPRITE_COUNTS[0x2e];
        self.ancilla_slot_view_mut(k).set_num_sprites(value);
        {
            let mut block = self.ancilla_slot_view_mut(k);
            block.set_aux_timer(3);
            block.set_step(0);
            block.set_item_to_link(0);
            block.set_work_byte_3(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_object_priority(value);
        self.dungeon_environment_mut()
            .clear_somaria_block_switch_counter();
        self.set_sound_effect_2_with_ancilla_pan(k, 1);
    }

    fn somarian_blast_draw(&mut self, k: usize) {
        let Some(mut info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        info.flags |= SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_FLAGS
            [self.ancilla_slot_view(k).item_to_link() as usize];
        if self.ancilla_slot_view(k).object_priority() != 0 {
            info.flags |= 0x30;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let j = self.ancilla_slot_view(k).direction() as usize * 6
            + self.ancilla_slot_view(k).step() as usize;
        self.oam_state_mut().write_entry(
            oam,
            info.x
                .wrapping_add(SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_X0[j] as u8),
            if sign8(SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_Y0[j]) {
                0xf0
            } else {
                info.y
                    .wrapping_add(SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_Y0[j])
            },
            0x82u8.wrapping_add(SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_CHAR0[j]),
            info.flags | SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_FLAGS0[j],
        );
        let value = 0;
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
        self.oam_state_mut().write_entry(
            oam + 4,
            info.x
                .wrapping_add(SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_X1[j] as u8),
            if sign8(SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_Y1[j]) {
                0xf0
            } else {
                info.y
                    .wrapping_add(SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_Y1[j])
            },
            0x82u8.wrapping_add(SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_CHAR1[j]),
            info.flags | SOMARIAN_BLAST_DRAW_SOMARIAN_BLAST_DRAW_FLAGS1[j],
        );
        let value = 0;
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF) / 4 + 1, value);
    }

    fn somaria_block_spawn_bullets(&mut self, k: usize) {
        let z = if self.ancilla_slot_view(k).z() == 0xff {
            0
        } else {
            self.ancilla_slot_view(k).z()
        };
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k).wrapping_sub(z as u16);

        for i in (0..=3).rev() {
            if let Some(j) = self.ancilla_alloc_init(1, 4) {
                let value = 1;
                self.ancilla_slot_view_mut(j).set_ancilla_type(value);
                let value = ANCILLA_DRAW_SPRITE_COUNTS[1];
                self.ancilla_slot_view_mut(j).set_num_sprites(value);
                let value = 4;
                self.ancilla_slot_view_mut(j).set_step(value);
                let value = 0;
                self.ancilla_slot_view_mut(j).set_item_to_link(value);
                let value = 0;
                self.ancilla_slot_view_mut(j).set_object_priority(value);
                let value = i as u8;
                self.ancilla_slot_view_mut(j).set_direction(value);
                self.ancilla_set_xy(
                    j,
                    x.wrapping_add(
                        SOMARIA_BLOCK_SPAWN_BULLETS_SPAWN_CENTRIFUGAL_QUAD_X[i] as i16 as u16,
                    ),
                    y.wrapping_add(
                        SOMARIA_BLOCK_SPAWN_BULLETS_SPAWN_CENTRIFUGAL_QUAD_Y[i] as i16 as u16,
                    ),
                );
                self.ancilla_terminate_if_offscreen(j);
                let value = FIRE_ROD_SPARK_X_VELOCITIES[i] as u8;
                self.ancilla_slot_view_mut(j).set_x_velocity(value);
                let value = FIRE_ROD_SPARK_Y_VELOCITIES[i] as u8;
                self.ancilla_slot_view_mut(j).set_y_velocity(value);
                let value = self.ancilla_slot_view(k).floor();
                self.ancilla_slot_view_mut(j).set_floor(value);
                let value = self
                    .game_state
                    .player
                    .follower_link
                    .lower_level_mirror_state();
                self.ancilla_slot_view_mut(j).set_floor2(value);
            }
        }
        self.temp_counter_mut().set(0xff);
    }

    fn ancilla_draw_somaria_block(&mut self, k: usize) {
        if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
            && self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
            && self.ancilla_slot_view(k).k() != 3
            && self.game_state.player.follower_link.facing() == 0
        {
            self.ancilla_allocate_oam_from_region_b_or_e(self.ancilla_slot_view(k).num_sprites());
        } else if self.game_state.oam.has_sprite_sorting()
            && self.ancilla_slot_view(k).floor() != 0
            && (self.ancilla_slot_view(k).l() != 0
                || k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                    && self
                        .game_state
                        .player
                        .follower_link
                        .is_lifting_or_carrying())
        {
            self.oam_state_mut().set_current_pointer(0x08d0);
            self.oam_state_mut()
                .set_current_extended_pointer(0x0a20 + 0x34);
        }

        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let oam_org = self.game_state.oam.current_pointer_usize();
        let mut oam = oam_org;
        let z = self.ancilla_slot_view(k).z() as i8;
        if z != 0
            && z != -1
            && self.ancilla_slot_view(k).k() != 3
            && self.ancilla_slot_view(k).object_priority() != 0
        {
            self.oam_state_mut().set_priority_word(0x3000);
        }
        y = y.wrapping_sub(z as i16 as u16);
        let mut j = self.ancilla_slot_view(k).work_byte_1() as usize * 4;
        for _ in 0..4 {
            self.ancilla_set_oam_safe(
                oam,
                x.wrapping_add(ANCILLA_DRAW_SOMARIA_BLOCK_SOMARIAN_BLOCK_DRAW_X[j] as i16 as u16),
                y.wrapping_add(ANCILLA_DRAW_SOMARIA_BLOCK_SOMARIAN_BLOCK_DRAW_Y[j] as i16 as u16),
                0xe9,
                ANCILLA_DRAW_SOMARIA_BLOCK_SOMARIAN_BLOCK_DRAW_FLAGS[j] & !0x30
                    | 2
                    | self.game_state.oam.priority_high(),
                0,
            );
            j += 1;
            oam += 4;
        }

        if self.somarian_block_check_empty(oam_org) {
            self.dungeon_environment_mut()
                .clear_somaria_block_switch_counter();
            self.ancilla_slot_view_mut(k).set_ancilla_type(0);
            if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize {
                self.follower_link_state_mut().clear_ancilla_pickup_flag();
                if self
                    .game_state
                    .player
                    .follower_link
                    .is_lifting_or_carrying()
                {
                    self.follower_link_state_mut().clear_state_bits();
                }
            }
        }
    }

    fn somaria_block_check_for_switch(&mut self, k: usize) -> bool {
        self.dungeon_environment_mut()
            .clear_somaria_block_switch_counter();
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_24(value);
        for j in (0..=3).rev() {
            let y = self.ancilla_get_y(k).wrapping_add(
                SOMARIA_BLOCK_CHECK_FOR_SWITCH_SOMARIAN_BLOCK_CHECK_COVER_Y[j] as i16 as u16,
            );
            let x = self.ancilla_get_x(k).wrapping_add(
                SOMARIA_BLOCK_CHECK_FOR_SWITCH_SOMARIAN_BLOCK_CHECK_COVER_X[j] as i16 as u16,
            );
            let bak = self.ancilla_slot_view(k).object_priority();
            self.ancilla_check_tile_collision_targeted(k, x, y);
            let value = bak;
            self.ancilla_slot_view_mut(k).set_object_priority(value);
            if matches!(
                self.ancilla_slot_view(k).tile_attribute(),
                0x23 | 0x24 | 0x25 | 0x3b
            ) {
                self.ancilla_slot_view_mut(k).add_work_byte_24(1);
            }
        }
        self.ancilla_slot_view(k).work_byte_24() != 4
    }

    fn somaria_block_fizzle_away(&mut self, k: usize) {
        if self.game_state.player.follower_link.speed_setting() == 18 {
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
        }
        self.dungeon_environment_mut()
            .clear_somaria_block_switch_counter();
        {
            let mut block = self.ancilla_slot_view_mut(k);
            block.set_ancilla_type(0x2d);
            block.set_aux_timer(0);
            block.set_step(0);
            block.set_item_to_link(0);
            block.set_work_byte_3(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_r(value);
        if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize {
            self.follower_link_state_mut().clear_ancilla_pickup_flag();
            self.follower_link_state_mut()
                .keep_only_lifting_or_carrying_state();
        }
        self.ancilla2_d_somaria_block_fizz(k);
    }

    fn somarian_block_check_empty(&self, oam: usize) -> bool {
        for i in 0..4 {
            if self.game_state.oam.entry_y(oam + i * 4) == 0xf0 {
                continue;
            }
            for i in 0..4 {
                if self.game_state.oam.extended_byte((oam - OAM_BUF) / 4 + i) & 1 == 0 {
                    return false;
                }
            }
            break;
        }
        true
    }

    fn medallion_check_sprite_damage(&mut self, k: usize) {
        let ancilla_type = self.ancilla_slot_view(k).ancilla_type();
        self.temp_counter_mut().set(ancilla_type);
        for j in (0..16).rev() {
            if self.sprite_slot_view(j).state() >= 9
                && (self.sprite_slot_view(j).ignore_projectile() | self.sprite_slot_view(j).pause())
                    == 0
            {
                self.ancilla_check_damage_to_sprite_aggressive(
                    j,
                    self.game_state.scratch_counter.value(),
                );
            }
        }
    }

}
