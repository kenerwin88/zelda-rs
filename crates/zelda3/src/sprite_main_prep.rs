//! Ported SpritePrep_* helpers from sprite_main.c.

use super::*;
use crate::types::{sign16, sign8};
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

mod sprite_main_prep_shared;
use sprite_main_prep_shared::*;

impl ZeldaState {
    pub(super) fn sprite_prep_throwable_scenery(&mut self, _k: usize) {}

    // void SpriteModule_Initialize(int k) {  // 86864d
    pub(super) fn sprite_module_initialize(&mut self, k: usize) {
        self.sprite_module_initialize_properties(k);
        self.sprite_module_initialize_after_properties(k);
    }

    pub(super) fn sprite_module_initialize_properties(&mut self, k: usize) {
        self.sprite_prep_load_properties(k);
        self.sprite_slot_view_mut(k).increment_state();
    }

    pub(super) fn sprite_module_initialize_after_properties(&mut self, k: usize) {
        match self.sprite_slot_view(k).sprite_type() {
            0x00 => self.sprite_prep_raven(k),
            0x01 => self.sprite_prep_vulture(k),
            0x02 => self.sprite_prep_do_nothing_a(k),
            0x03 => {}
            0x04 => self.sprite_prep_switch(k),
            0x05 => self.sprite_prep_do_nothing_a(k),
            0x06 => self.sprite_prep_switch(k),
            0x07 => self.sprite_prep_switch_facing_up(k),
            0x08 => self.sprite_prep_octorok(k),
            0x09 => self.sprite_prep_moldorm(k),
            0x0a => self.sprite_prep_octorok(k),
            0x0b => self.sprite_prep_do_nothing_a(k),
            0x0c => self.sprite_prep_do_nothing_a(k),
            0x0d => self.sprite_prep_do_nothing_a(k),
            0x0e => self.sprite_prep_do_nothing_a(k),
            0x0f => self.sprite_prep_octoballoon(k),
            0x10 => self.sprite_prep_do_nothing_a(k),
            0x11 => self.sprite_prep_do_nothing_a(k),
            0x12 => self.sprite_prep_do_nothing_a(k),
            0x13 => self.sprite_prep_mini_helmasaur(k),
            0x14 => self.sprite_prep_thieves_town_grate(k),
            0x15 => self.sprite_prep_antifairy(k),
            0x16 => self.sprite_prep_sage(k),
            0x17 => self.sprite_prep_do_nothing_a(k),
            0x18 => self.sprite_prep_mini_moldorm_bounce(k),
            0x19 => self.sprite_prep_poe(k),
            0x1a => self.sprite_prep_smithy(k),
            0x1b => self.sprite_prep_do_nothing_a(k),
            0x1c => self.sprite_prep_statue(k),
            0x1d => self.sprite_prep_ignore_projectiles(k),
            0x1e => self.sprite_prep_crystal_switch(k),
            0x1f => self.sprite_prep_sick_kid(k),
            0x20 => self.sprite_prep_do_nothing_a(k),
            0x21 => self.sprite_prep_water_lever(k),
            0x22 => self.sprite_prep_do_nothing_a(k),
            0x23 => self.sprite_prep_bari(k),
            0x24 => self.sprite_prep_bari(k),
            0x25 => self.sprite_prep_talking_tree(k),
            0x26 => self.sprite_prep_hardhat_beetle(k),
            0x27 => self.sprite_prep_do_nothing_a(k),
            0x28 => self.sprite_prep_storyteller(k),
            0x29 => self.sprite_prep_adults(k),
            0x2a => self.sprite_prep_ignore_projectiles(k),
            0x2b => self.sprite_prep_hobo(k),
            0x2c => self.sprite_prep_magic_bat(k),
            0x2d => self.sprite_prep_ignore_projectiles(k),
            0x2e => self.sprite_prep_flute_kid(k),
            0x2f => self.sprite_prep_ignore_projectiles(k),
            0x30 => self.sprite_prep_ignore_projectiles(k),
            0x31 => self.sprite_prep_fortune_teller(k),
            0x32 => self.sprite_prep_ignore_projectiles(k),
            0x33 => self.sprite_prep_rupee_pull(k),
            0x34 => self.sprite_prep_snitch_bounce_2(k),
            0x35 => self.sprite_prep_snitch_bounce_3(k),
            0x36 => self.sprite_prep_ignore_projectiles(k),
            0x37 => self.sprite_prep_ignore_projectiles(k),
            0x38 => self.sprite_prep_do_nothing_a(k),
            0x39 => self.sprite_prep_locksmith(k),
            0x3a => self.sprite_prep_magic_bat(k),
            0x3b => self.sprite_prep_bonk_item(k),
            0x3c => self.sprite_prep_ignore_projectiles(k),
            0x3d => self.sprite_prep_snitch_bounce_1(k),
            0x3e => self.sprite_prep_do_nothing_a(k),
            0x3f => self.sprite_prep_do_nothing_a(k),
            0x40 => self.sprite_prep_agahnims_barrier(k),
            0x41 => self.sprite_prep_standard_guard(k),
            0x42 => self.sprite_prep_standard_guard(k),
            0x43 => self.sprite_prep_standard_guard(k),
            0x44 => self.sprite_prep_trooper_and_archer_soldier(k),
            0x45 => self.sprite_prep_trooper_and_archer_soldier(k),
            0x46 => self.sprite_prep_trooper_and_archer_soldier(k),
            0x47 => self.sprite_prep_trooper_and_archer_soldier(k),
            0x48 => self.sprite_prep_trooper_and_archer_soldier(k),
            0x49 => self.sprite_prep_trooper_and_archer_soldier(k),
            0x4a => self.sprite_prep_trooper_and_archer_soldier(k),
            0x4b => self.sprite_prep_weak_guard(k),
            0x4c => self.sprite_prep_geldman(k),
            0x4d => self.sprite_prep_kyameron(k),
            0x4e => self.sprite_prep_popo(k),
            0x4f => self.sprite_prep_popo2(k),
            0x50 => self.sprite_prep_do_nothing_a(k),
            0x51 => self.sprite_prep_do_nothing_d(k),
            0x52 => self.sprite_prep_king_zora(k),
            0x53 => self.sprite_prep_armos_knight(k),
            0x54 => self.sprite_prep_lanmolas(k),
            0x55 => self.sprite_prep_swimming_zora(k),
            0x56 => self.sprite_prep_walking_zora(k),
            0x57 => self.sprite_prep_desert_statue(k),
            0x58 => self.sprite_prep_do_nothing_a(k),
            0x59 => self.sprite_prep_lost_woods_bird(k),
            0x5a => self.sprite_prep_lost_woods_squirrel(k),
            0x5b => self.sprite_prep_spark(k),
            0x5c => self.sprite_prep_spark(k),
            0x5d => self.sprite_prep_roller_vertical_down_first(k),
            0x5e => self.sprite_prep_roller_up_down(k),
            0x5f => self.sprite_prep_roller_horizontal_right_first(k),
            0x60 => self.sprite_prep_roller_left_right(k),
            0x61 => self.sprite_prep_do_nothing_a(k),
            0x62 => self.sprite_prep_master_sword(k),
            0x63 => self.sprite_prep_debirando_pit(k),
            0x64 => self.sprite_prep_fire_debirando(k),
            0x65 => self.sprite_prep_arrow_game_bounce(k),
            0x66 => self.sprite_prep_wall_cannon(k),
            0x67 => self.sprite_prep_wall_cannon(k),
            0x68 => self.sprite_prep_wall_cannon(k),
            0x69 => self.sprite_prep_wall_cannon(k),
            0x6a => self.sprite_prep_do_nothing_a(k),
            0x6b => self.sprite_prep_do_nothing_a(k),
            0x6c => self.sprite_prep_do_nothing_a(k),
            0x6d => self.sprite_prep_rat(k),
            0x6e => self.sprite_prep_rope(k),
            0x6f => self.sprite_prep_keese(k),
            0x70 => self.sprite_prep_do_nothing_g(k),
            0x71 => self.sprite_prep_fairy_pond(k),
            0x72 => self.sprite_prep_ignore_projectiles(k),
            0x73 => self.sprite_prep_uncle_and_priest_bounce(k),
            0x74 => self.sprite_prep_running_man(k),
            0x75 => self.sprite_prep_ignore_projectiles(k),
            0x76 => self.sprite_prep_zelda_bounce(k),
            0x77 => self.sprite_prep_antifairy(k),
            0x78 => self.sprite_prep_mrs_sahasrahla(k),
            0x79 => self.sprite_prep_overworld_bonk_item(k),
            0x7a => self.sprite_prep_agahnim(k),
            0x7b => self.sprite_prep_do_nothing_g(k),
            0x7c => self.sprite_prep_green_stalfos(k),
            0x7d => self.sprite_prep_big_spike(k),
            0x7e => self.sprite_prep_fire_bar(k),
            0x7f => self.sprite_prep_fire_bar(k),
            0x80 => self.sprite_prep_do_nothing_g(k),
            0x81 => self.sprite_prep_do_nothing_g(k),
            0x82 => self.sprite_prep_antifairy_circle(k),
            0x83 => self.sprite_prep_eyegore(k),
            0x84 => self.sprite_prep_eyegore(k),
            0x85 => self.sprite_prep_do_nothing_g(k),
            0x86 => self.sprite_prep_kodongo(k),
            0x87 => self.sprite_prep_do_nothing_g(k),
            0x88 => self.sprite_prep_mothula(k),
            0x89 => self.sprite_prep_do_nothing_g(k),
            0x8a => self.sprite_prep_spike(k),
            0x8b => self.sprite_prep_do_nothing_g(k),
            0x8c => self.sprite_prep_arrghus(k),
            0x8d => self.sprite_prep_arrghi(k),
            0x8e => self.sprite_prep_do_nothing_g(k),
            0x8f => self.sprite_prep_blob(k),
            0x90 => self.sprite_prep_do_nothing_g(k),
            0x91 => self.sprite_prep_do_nothing_g(k),
            0x92 => self.sprite_prep_helmasaur_king(k),
            0x93 => self.sprite_prep_bumper(k),
            0x94 => self.sprite_prep_do_nothing_a(k),
            0x95 => self.sprite_prep_laser_eye_bounce(k),
            0x96 => self.sprite_prep_laser_eye_bounce(k),
            0x97 => self.sprite_prep_laser_eye_bounce(k),
            0x98 => self.sprite_prep_laser_eye_bounce(k),
            0x99 => self.sprite_prep_do_nothing_a(k),
            0x9a => self.sprite_prep_kyameron(k),
            0x9b => self.sprite_prep_do_nothing_a(k),
            0x9c => self.sprite_prep_zoro(k),
            0x9d => self.sprite_prep_babasu(k),
            0x9e => self.sprite_prep_haunted_grove_ostritch(k),
            0x9f => self.sprite_prep_haunted_grove_animal(k),
            0xa0 => self.sprite_prep_haunted_grove_animal(k),
            0xa1 => self.sprite_prep_move_down_8px(k),
            0xa2 => self.sprite_prep_kholdstare(k),
            0xa3 => self.sprite_prep_kholdstare_shell(k),
            0xa4 => self.sprite_prep_falling_ice(k),
            0xa5 => self.sprite_prep_zazakku(k),
            0xa6 => self.sprite_prep_zazakku(k),
            0xa7 => self.sprite_prep_stalfos(k),
            0xa8 => self.sprite_prep_bomber(k),
            0xa9 => self.sprite_prep_bomber(k),
            0xaa => self.sprite_prep_do_nothing_c(k),
            0xab => self.sprite_prep_do_nothing_h(k),
            0xac => self.sprite_prep_overworld_bonk_item(k),
            0xad => self.sprite_prep_old_man_bounce(k),
            0xae => self.sprite_prep_do_nothing_a(k),
            0xaf => self.sprite_prep_do_nothing_a(k),
            0xb0 => self.sprite_prep_do_nothing_a(k),
            0xb1 => self.sprite_prep_do_nothing_a(k),
            0xb2 => self.sprite_prep_nice_bee(k),
            0xb3 => self.sprite_prep_pedestal_plaque(k),
            0xb4 => self.sprite_prep_purple_chest(k),
            0xb5 => self.sprite_prep_bomb_shoppe(k),
            0xb6 => self.sprite_prep_kiki(k),
            0xb7 => self.sprite_prep_blind_maiden(k),
            0xb8 => self.sprite_prep_do_nothing_a(k),
            0xb9 => self.sprite_prep_bully_and_victim(k),
            0xba => self.sprite_prep_whirlpool(k),
            0xbb => self.sprite_prep_shopkeeper(k),
            0xbc => self.sprite_prep_ignore_projectiles(k),
            0xbd => self.sprite_prep_vitreous(k),
            0xbe => self.sprite_prep_mini_vitreous(k),
            0xbf => self.sprite_prep_do_nothing_a(k),
            0xc0 => self.sprite_prep_catfish(k),
            0xc1 => self.sprite_prep_cutscene_agahnim(k),
            0xc2 => self.sprite_prep_do_nothing_a(k),
            0xc3 => self.sprite_prep_gibo(k),
            0xc4 => self.sprite_prep_do_nothing_a(k),
            0xc5 => self.sprite_prep_ignore_projectiles(k),
            0xc6 => self.sprite_prep_ignore_projectiles(k),
            0xc7 => self.sprite_prep_pokey(k),
            0xc8 => self.sprite_prep_big_fairy(k),
            0xc9 => self.sprite_prep_tektite(k),
            0xca => self.sprite_prep_chainchomp_bounce(k),
            0xcb => self.sprite_prep_trinexx(k),
            0xcc => self.sprite_prep_trinexx(k),
            0xcd => self.sprite_prep_trinexx(k),
            0xce => self.sprite_prep_blind(k),
            0xcf => self.sprite_prep_swamola(k),
            0xd0 => self.sprite_prep_do_nothing_a(k),
            0xd1 => self.sprite_prep_do_nothing_a(k),
            0xd2 => self.sprite_prep_ignore_projectiles(k),
            0xd3 => self.sprite_prep_rock_stal(k),
            0xd4 => self.sprite_prep_ignore_projectiles(k),
            0xd5 => self.sprite_prep_digging_game_guy_bounce(k),
            0xd6 => self.sprite_prep_ganon(k),
            0xd7 => self.sprite_prep_ganon(k),
            0xd8 => self.sprite_prep_absorbable(k),
            0xd9 => self.sprite_prep_absorbable(k),
            0xda => self.sprite_prep_absorbable(k),
            0xdb => self.sprite_prep_absorbable(k),
            0xdc => self.sprite_prep_absorbable(k),
            0xdd => self.sprite_prep_absorbable(k),
            0xde => self.sprite_prep_absorbable(k),
            0xdf => self.sprite_prep_absorbable(k),
            0xe0 => self.sprite_prep_absorbable(k),
            0xe1 => self.sprite_prep_absorbable(k),
            0xe2 => self.sprite_prep_absorbable(k),
            0xe3 => self.sprite_prep_fairy(k),
            0xe4 => self.sprite_prep_small_key(k),
            0xe5 => self.sprite_prep_big_key(k),
            0xe6 => self.sprite_prep_shield_pickup(k),
            0xe7 => self.sprite_prep_mushroom(k),
            0xe8 => self.sprite_prep_fake_sword(k),
            0xe9 => self.sprite_prep_potion_shop(k),
            0xea => self.sprite_prep_heart_container(k),
            0xeb => self.sprite_prep_heart_piece(k),
            0xec => self.sprite_prep_throwable_scenery(k),
            0xed => self.sprite_prep_do_nothing_a(k),
            0xee => self.sprite_prep_mantle(k),
            0xef => self.sprite_prep_do_nothing_a(k),
            0xf0 => self.sprite_prep_do_nothing_a(k),
            0xf1 => self.sprite_prep_do_nothing_a(k),
            0xf2 => self.sprite_prep_medallion_table(k),
            _ => {}
        }
    }

    // void SpritePrep_StandardGuard(int k) {  // 868fd6
    pub(super) fn sprite_prep_standard_guard(&mut self, k: usize) {
        let subtype = self.sprite_slot_view(k).subtype();
        if subtype != 0 {
            if (subtype & 7) >= 5 {
                let j = usize::from(if (subtype & 7) != 5 { 4 } else { 0 } + ((subtype >> 3) & 3));
                self.sprite_slot_view_mut(k)
                    .set_b(SPRITE_PREP_STANDARD_GUARD_GUARD_SUBTYPE_B_REMAP[j]);
                self.sprite_slot_view_mut(k).masked_or_flags(0x0f, 0x50);
                self.sprite_prep_trooper_and_archer_soldier(k);
                return;
            }
            self.sprite_slot_view_mut(k)
                .set_direction(((subtype & 7).wrapping_sub(1)) ^ 1);
        }
        if self.game_state.world.location.is_indoors() {
            self.sprite_slot_view_mut(k).and_flags5(!0x80);
            return;
        }
        self.sprite_slot_view_mut(k).set_ai_state(1);
        self.sprite_slot_view_mut(k).set_delay_main(112);
        let dir = self.sprite_direction_to_face_link(k, None);
        self.sprite_slot_view_mut(k).set_direction(dir);
        self.sprite_slot_view_mut(k).set_head_direction(dir);
        self.sprite_prep_trooper_and_archer_soldier(k);
    }

    // void SpritePrep_TrooperAndArcherSoldier(int k) {  // 869001
    pub(super) fn sprite_prep_trooper_and_archer_soldier(&mut self, k: usize) {
        let bak0 = self.game_state.frame.submodule;
        self.set_submodule(0);
        let deflection_bits = (self.sprite_slot_view(k).deflection_bits() >> 1) | 0x80;
        self.sprite_slot_view_mut(k)
            .set_deflection_bits(deflection_bits);
        self.sprite_active_main(k);
        self.sprite_active_main(k);
        let deflection_bits = self.sprite_slot_view(k).deflection_bits().wrapping_shl(1);
        self.sprite_slot_view_mut(k)
            .set_deflection_bits(deflection_bits);
        self.set_submodule(bak0);
    }

    pub(super) fn sprite_prep_mantle(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_y_low(3);
        self.sprite_slot_view_mut(k).add_x_low(8);
    }

    pub(super) fn sprite_prep_switch(&mut self, k: usize) {
        let room = self.game_state.dungeon.room_tracking.room_index2();
        if room == 0xce || room == 4 || room == 0x3f {
            self.sprite_slot_view_mut(k).set_oam_flags(0x0d);
        }
    }

    pub(super) fn sprite_prep_switch_facing_up(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_do_nothing_a(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_rat(&mut self, k: usize) {
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_view_mut(k)
            .set_bump_damage(SPRITE_PREP_RAT_BUMP_DAMAGE_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_health(SPRITE_PREP_RAT_HEALTH_VALUES[j]);
    }

    pub(super) fn sprite_prep_keese(&mut self, k: usize) {
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_view_mut(k)
            .set_bump_damage(SPRITE_PREP_KEESE_BUMP_DAMAGE_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_health(SPRITE_PREP_KEESE_HEALTH_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_flags5(SPRITE_PREP_KEESE_FLAGS5_VALUES[j]);
    }

    pub(super) fn sprite_prep_rope(&mut self, k: usize) {
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_view_mut(k)
            .set_bump_damage(SPRITE_PREP_ROPE_BUMP_DAMAGE_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_health(SPRITE_PREP_ROPE_HEALTH_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_flags5(SPRITE_PREP_ROPE_FLAGS5_VALUES[j]);
    }

    pub(super) fn sprite_prep_babasu(&mut self, k: usize) {
        self.sprite_prep_move_down_8px(k);
        self.sprite_prep_zoro(k);
    }

    pub(super) fn sprite_prep_pokey(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_a(3);
        self.sprite_slot_view_mut(k).set_b(8);
        let j = (self.get_random_number() & 3) as usize;
        self.sprite_slot_view_mut(k)
            .set_x_velocity(SPRITE_PREP_POKEY_INITIAL_X_VELOCITIES[j] as u8);
        self.sprite_slot_view_mut(k)
            .set_y_velocity(SPRITE_PREP_POKEY_INITIAL_Y_VELOCITIES[j] as u8);
    }

    pub(super) fn sprite_prep_gibo(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(16);
        self.sprite_slot_view_mut(k).set_g(8);
    }

    pub(super) fn sprite_prep_octoballoon(&mut self, k: usize) {
        self.sprite_slot_view_mut(k)
            .set_delay_main(SPRITE_PREP_OCTOBALLOON_DELAYS[k & 3]);
    }

    pub(super) fn sprite_prep_blind(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_prep_blind_prepare_battle(k);
    }

    pub(super) fn sprite_prep_ganon(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ganon_handle_animation_idle(k);
        self.sprite_slot_view_mut(k).set_delay_main(128);
        self.sprite_slot_view_mut(k).set_room(2);
        self.set_music_control(0x1e);
    }

    pub(super) fn sprite_prep_mini_vitreous(&mut self, k: usize) {
        self.sprite_return_if_boss_finished(k);
    }

    pub(super) fn sprite_prep_agahnims_barrier(&mut self, k: usize) {
        let screen = self.game_state.world.location.overworld_screen_index() as usize;
        if self
            .game_state
            .world
            .overworld
            .event_info
            .event_info(screen)
            & 0x40
            != 0
        {
            self.sprite_slot_view_mut(k).set_graphics(4);
        }
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_slot_view_mut(k).subtract_y_low(12);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_catfish(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_slot_view_mut(k).subtract_y_low(12);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub(super) fn sprite_prep_cutscene_agahnim(&mut self, k: usize) {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x4000 != 0 {
            self.sprite_slot_view_mut(k).set_state(0);
        } else {
            self.cutscene_agahnim_spawn_zelda_on_altar(k);
            self.sprite_slot_view_mut(k).increment_ignore_projectile();
        }
    }

    pub(super) fn cutscene_agahnim_spawn_zelda_on_altar(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_x_low(8);
        self.sprite_slot_view_mut(k).add_y_low(6);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc1, &mut info);
        let j = j as usize;
        self.sprite_slot_view_mut(j).set_a(1);
        self.sprite_slot_view_mut(j).set_ignore_projectile(1);
        self.sprite_set_spawned_coordinates(j, &info);
        self.sprite_slot_view_mut(j)
            .set_y_low((info.r2_y as u8).wrapping_add(40));
        self.sprite_slot_view_mut(j).set_flags2(0);
        self.sprite_slot_view_mut(j).set_oam_flags(12);
    }

    pub(super) fn sprite_prep_vitreous(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_slot_view_mut(k).subtract_y_low(16);
        self.vitreous_spawn_smaller_eyes(k);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_raven(&mut self, k: usize) {
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_view_mut(k)
            .set_bump_damage(SPRITE_PREP_RAVEN_BUMP_DAMAGE_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_health(SPRITE_PREP_RAVEN_HEALTH_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_flags5(SPRITE_PREP_RAVEN_FLAGS5_VALUES[j]);
        self.sprite_prep_vulture(k);
    }

    pub(super) fn sprite_prep_vulture(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(0);
        let a = (self.sprite_slot_view(k).x_low() & 16) >> 4;
        self.sprite_slot_view_mut(k).set_a(a);
        self.sprite_slot_view_mut(k).set_subtype(254);
    }

    pub(super) fn sprite_prep_poe(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(12);
        self.sprite_slot_view_mut(k).set_subtype(254);
    }

    pub(super) fn sprite_prep_do_nothing_c(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_blind_maiden(&mut self, k: usize) {
        if self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(0xac)
            & 0x0800
            == 0
        {
            self.sprite_slot_view_mut(k).increment_ignore_projectile();
            if self.game_state.sprites.follower_runtime.indicator() != 6 {
                self.follower_state_mut().set_indicator(6);
                self.follower_state_mut().set_dropped(0);
                self.follower_state_mut().set_appearance_none_flag(0);
                self.load_follower_graphics();
                self.follower_initialize();
                self.follower_state_mut().set_indicator(0);
                return;
            }
        }
        self.sprite_slot_view_mut(k).set_state(0);
    }

    pub(super) fn sprite_prep_snitches(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_direction(2);
        self.sprite_slot_view_mut(k).set_head_direction(2);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        let x_low = self.sprite_slot_view(k).x_low();
        let x_high = self.sprite_slot_view(k).x_high();
        self.sprite_slot_view_mut(k).set_a(x_low);
        self.sprite_slot_view_mut(k).set_b(x_high);
        self.sprite_slot_view_mut(k).set_x_velocity((-9i8) as u8);
    }

    pub(super) fn sprite_prep_running_man(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_head_direction(2);
        self.sprite_slot_view_mut(k).set_direction(2);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_arrow_game_bounce(&mut self, k: usize) {
        self.archery_game_mut().clear_hit_counter();
        self.sprite_slot_view_mut(k).subtract_y_low(9);
        let link_x_high = (self.game_state.player.follower_link.x() >> 8) as u8;
        let link_y_high = (self.game_state.player.follower_link.y() >> 8) as u8;
        let link_floor = self.game_state.player.follower_link.lower_level_state();
        for i in (1..=7).rev() {
            self.sprite_slot_view_mut(i).set_sprite_type(0x65);
            self.sprite_slot_view_mut(i).set_state(9);
            self.sprite_prep_load_properties(i);
            self.sprite_slot_view_mut(i).set_x_high(link_x_high);
            self.sprite_slot_view_mut(i)
                .set_x_low(SPRITE_PREP_ARROW_GAME_BOUNCE_X_OFFSETS[i]);
            self.sprite_slot_view_mut(i).set_y_high(link_y_high);
            self.sprite_slot_view_mut(i)
                .set_y_low(SPRITE_PREP_ARROW_GAME_BOUNCE_Y_OFFSETS[i]);
            self.sprite_slot_view_mut(i)
                .set_a(SPRITE_PREP_ARROW_GAME_BOUNCE_ATTRS[i]);
            let j = (SPRITE_PREP_ARROW_GAME_BOUNCE_ATTRS[i] - 1) as usize;
            self.sprite_slot_view_mut(i).set_graphics(j as u8);
            self.sprite_slot_view_mut(i)
                .set_x_velocity(SPRITE_PREP_ARROW_GAME_BOUNCE_X_VELOCITIES[j] as u8);
            self.sprite_slot_view_mut(i)
                .set_flags4(SPRITE_PREP_ARROW_GAME_BOUNCE_FLAGS4_VALUES[j]);
            self.sprite_slot_view_mut(i).set_oam_flags(13);
            self.sprite_slot_view_mut(i).set_floor(link_floor);
            let subtype2 = self.get_random_number();
            self.sprite_slot_view_mut(i).set_subtype2(subtype2);
        }
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        let arrows = self.game_state.inventory.player_resources.arrows();
        self.sprite_slot_view_mut(k).set_subtype(arrows);
    }

    pub(super) fn sprite_prep_mushroom(&mut self, k: usize) {
        if self.game_state.inventory.items.mushroom() >= 2 {
            self.sprite_slot_view_mut(k).set_state(0);
        } else {
            self.sprite_slot_view_mut(k).set_graphics(0);
            self.sprite_slot_view_mut(k).or_oam_flags(8);
            self.sprite_slot_view_mut(k).increment_ignore_projectile();
        }
    }

    pub(super) fn sprite_prep_potion_shop(&mut self, k: usize) {
        self.magic_shop_assistant_spawn_powder(k);
        self.magic_shop_assistant_spawn_green_cauldron(k);
        self.magic_shop_assistant_spawn_blue_cauldron(k);
        self.magic_shop_assistant_spawn_red_cauldron(k);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn magic_shop_assistant_spawn_powder(&mut self, k: usize) {
        if !self.game_state.world.region.flag_overworld_area_changed()
            || self.game_state.inventory.items.mushroom() == 2
        {
            return;
        }
        if self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(0x109)
            & 0x80
            != 0
        {
            self.magic_shop_assistant_spawn_item(k, 1, -16, 0);
        }
    }

    pub(super) fn magic_shop_assistant_spawn_green_cauldron(&mut self, k: usize) {
        self.magic_shop_assistant_spawn_item(k, 2, -40, -72);
    }

    pub(super) fn magic_shop_assistant_spawn_blue_cauldron(&mut self, k: usize) {
        self.magic_shop_assistant_spawn_item(k, 3, 8, -72);
    }

    pub(super) fn magic_shop_assistant_spawn_red_cauldron(&mut self, k: usize) {
        self.magic_shop_assistant_spawn_item(k, 4, -88, -72);
    }

    fn magic_shop_assistant_spawn_item(&mut self, k: usize, subtype: u8, x_off: i16, y_off: i16) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xe9, &mut info);
        assert!(
            j >= 0,
            "MagicShopAssistant spawn expected Sprite_SpawnDynamically to succeed"
        );
        let j = j as usize;
        self.sprite_slot_view_mut(j).set_subtype2(subtype);
        self.sprite_set_x(j, info.r0_x.wrapping_add(x_off as u16));
        self.sprite_set_y(j, info.r2_y.wrapping_add(y_off as u16));
        self.sprite_slot_view_mut(j).set_flags4(3);
        self.sprite_slot_view_mut(j).or_deflection_bits(0x20);
    }

    pub(super) fn sprite_prep_mini_moldorm_bounce(&mut self, k: usize) {
        let mut j = 32 * k;
        let x = self.sprite_slot_view(k).x();
        let y = self.sprite_slot_view(k).y();
        for _ in 0..32 {
            self.moldorm_history_mut(j).set_position(x, y);
            j += 1;
        }
    }

    pub(super) fn sprite_prep_bomber(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(16);
        self.sprite_slot_view_mut(k).set_subtype(254);
    }

    pub(super) fn sprite_prep_bomb_shoppe(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xb5, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_x(j, info.r0_x.wrapping_sub(24));
            self.sprite_set_y(j, info.r2_y.wrapping_sub(24));
            self.sprite_slot_view_mut(j).set_subtype2(1);
            self.sprite_slot_view_mut(j).set_ignore_projectile(1);
        }

        if self.game_state.inventory.player_resources.crystal_flags() & 5 == 5
            && self
                .game_state
                .inventory
                .save_progress
                .progress_indicator_3()
                & 32
                != 0
        {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xb5, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, info.r0_x.wrapping_sub(56));
                self.sprite_set_y(j, info.r2_y.wrapping_sub(24));
                self.sprite_slot_view_mut(j).set_subtype2(2);
                self.sprite_slot_view_mut(j).set_ignore_projectile(2);
            }
        }
    }

    // void BombShop_ClerkExhalation(int k) {  // 9ee256
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0xb5, &info);
    //   if (j >= 0) {
    //     Sprite_SetX(j, info.r0_x + 4);
    //     Sprite_SetY(j, info.r2_y + 16);
    //     sprite_subtype2[j] = 3;
    //     sprite_ignore_projectile[j] = 3;
    //     sprite_z[j] = 4;
    //     sprite_z_vel[j] = -12;
    //     sprite_delay_main[j] = 23;
    //     sprite_flags3[j] &= ~0x11;
    //   }
    // }
    pub(super) fn bomb_shop_clerk_exhalation(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xb5, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_x(j, info.r0_x.wrapping_add(4));
            self.sprite_set_y(j, info.r2_y.wrapping_add(16));
            self.sprite_slot_view_mut(j).set_subtype2(3);
            self.sprite_slot_view_mut(j).set_ignore_projectile(3);
            self.sprite_slot_view_mut(j).set_z(4);
            self.sprite_slot_view_mut(j).set_z_velocity((-12i8) as u8);
            self.sprite_slot_view_mut(j).set_delay_main(23);
            self.sprite_slot_view_mut(j).and_flags3(!0x11u8);
        }
    }

    // void ArcheryGameGuy_ShowMsg(int k, int msg) {  // 8582bf
    //   dialogue_message_index = msg;
    //   Sprite_ShowMessageMinimal();
    //   sprite_delay_main[k] = 0;
    // }
    pub(super) fn archery_game_guy_show_msg(&mut self, k: usize, msg: i32) {
        self.dialogue_message_index_mut().set_value(msg as u16);
        self.sprite_show_message_minimal_c();
        self.sprite_slot_view_mut(k).set_delay_main(0);
    }

    pub(super) fn sprite_65_archery_game(&mut self, k: usize) {
        let arrows = self.sprite_slot_view(k).subtype();
        self.player_resources_mut().set_arrows(arrows);
        if self.sprite_slot_view(k).a() == 0 {
            self.archery_game_host(k);
        } else {
            self.sprite_good_or_bad_archery_target(k);
        }
    }

    pub(super) fn archery_game_host(&mut self, k: usize) {
        if self.game_state.archery_game.arrows_left() == 0 {
            self.archery_game_mut().increment_out_of_arrows();
        }
        self.archery_game_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).set_flags4(0);
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
        }
        if self.sprite_slot_view(k).delay_main() != 0 {
            if self.sprite_slot_view(k).delay_main() & 7 == 0 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x11);
            }
            let graphics = (self.sprite_slot_view(k).delay_main() & 4) >> 2;
            self.sprite_slot_view_mut(k).set_graphics(graphics);
        } else {
            let idx = if self.sprite_slot_view(k).ai_state() != 0 {
                ((self.game_state.frame.frame_counter >> 5) & 3) as usize
            } else {
                0
            };
            self.sprite_slot_view_mut(k)
                .set_graphics(ARCHERY_GAME_HOST_IDLE_GRAPHICS[idx]);
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_slot_view_mut(k).set_flags4(10);
                if self.sprite_check_damage_to_link_same_layer(k)
                    && self.game_state.player.follower_link.filtered_joypad_l() & 0x80 != 0
                {
                    self.sprite_slot_view_mut(k).set_ai_state(1);
                    self.archery_game_guy_show_msg(k, 0x85);
                }
            }
            1 | 3 => {
                if self.multiselect_choice().value() == 0
                    && self.game_state.inventory.player_resources.rupees_goal() >= 20
                {
                    self.sprite_slot_view_mut(k).set_head_direction(0);
                    self.archery_game_mut().clear_hit_counter();
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                    self.archery_game_guy_show_msg(k, 0x86);
                } else {
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                    self.archery_game_guy_show_msg(k, 0x87);
                }
            }
            2 => self.archery_game_host_proctor_game(k),
            _ => {}
        }
    }

    pub(super) fn archery_game_host_proctor_game(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() == 0 {
            self.archery_game_mut().set_arrows_left(5);
            self.sprite_initialize_secondary_item_minigame(2);
            self.sprite_slot_view_mut(k).set_delay_aux1(39);
            let rupees = self.game_state.inventory.player_resources.rupees_goal();
            self.player_resources_mut()
                .set_rupees_goal(rupees.wrapping_sub(20));
            self.sprite_slot_view_mut(k).increment_head_direction();
        }

        self.oam_allocate_from_region_a(0x34);
        let Some((info_x, info_y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let count = if self.sprite_slot_view(k).delay_aux1() != 0 {
            ARCHERY_GAME_HOST_PROCTOR_GAME_SPRITE_COUNTS
                [(self.sprite_slot_view(k).delay_aux1() >> 3) as usize]
        } else {
            self.game_state.archery_game.arrows_left()
        };
        let mut i = (count as i32) * 2 + 7;
        let mut oam = self.game_state.oam.current_pointer_usize();
        while i >= 0 {
            let idx = i as usize;
            self.set_oam_plain_at_for_prep(
                oam,
                info_x
                    .wrapping_sub(20)
                    .wrapping_add(ARCHERY_GAME_HOST_PROCTOR_GAME_X_OFFSETS[idx] as i16 as u16)
                    .wrapping_add(1) as u8,
                info_y
                    .wrapping_sub(48)
                    .wrapping_add(ARCHERY_GAME_HOST_PROCTOR_GAME_Y_OFFSETS[idx] as i16 as u16)
                    .wrapping_add(1) as u8,
                ARCHERY_GAME_HOST_PROCTOR_GAME_CHARS[idx],
                ARCHERY_GAME_HOST_PROCTOR_GAME_OAM_FLAGS[idx],
                0,
            );
            oam += 4;
            i -= 1;
        }

        let ancillas_active = (0..=4).any(|i| self.ancilla_slot_view(i).is_active());
        if self.game_state.archery_game.arrows_left()
            | self.sprite_slot_view(k).delay_aux4()
            | u8::from(ancillas_active)
            != 0
        {
            return;
        }
        self.sprite_slot_view_mut(k).set_flags4(0x0a);
        if self.sprite_check_damage_to_link_same_layer(k)
            && self.game_state.player.follower_link.filtered_joypad_l() & 0x80 != 0
        {
            self.archery_game_guy_show_msg(k, 0x88);
            self.sprite_slot_view_mut(k).set_ai_state(3);
        }
    }

    pub(super) fn sprite_good_or_bad_archery_target(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() == 1 {
            if self.sprite_slot_view(k).g() >= 5 {
                self.sprite_slot_view_mut(k).set_b(6);
            }
            self.sprite_slot_view_mut(k).and_flags2(!0x1f);
            let j = if self.sprite_slot_view(k).delay_aux2() != 0 {
                self.sprite_slot_view(k).delay_aux2()
            } else {
                self.sprite_slot_view(k).subtype2() >> 3
            };
            self.sprite_slot_view_mut(k)
                .masked_or_oam_flags(!0x40, (j & 4) << 4);
            self.sprite_workspace_mut().subtract_current_sprite_y_low(3);
            self.sprite_draw_single_large(k);
            if self.sprite_slot_view(k).delay_aux2() != 0 {
                if self.sprite_slot_view(k).delay_aux2() == 96
                    && self.game_state.frame.submodule == 0
                {
                    self.sprite_slot_view_mut(0).set_delay_main(112);
                    let prize = SPRITE_GOOD_OR_BAD_ARCHERY_TARGET_CASH_PRIZE
                        [self.sprite_slot_view(k).b().wrapping_sub(1) as usize]
                        as u16;
                    let rupees = self
                        .game_state
                        .inventory
                        .player_resources
                        .rupees_goal()
                        .wrapping_add(prize);
                    self.player_resources_mut().set_rupees_goal(rupees);
                }
                self.sprite_slot_view_mut(k).or_flags2(5);
                self.archery_game_draw_prize(k);
            }
        } else {
            self.sprite_slot_view_mut(k).and_flags2(!0x1f);
            self.sprite_workspace_mut().add_current_sprite_y_low(3);
            self.sprite_draw_single_large(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        if self.sprite_slot_view(k).delay_aux3() == 1 {
            self.set_sound_effect_1(0x3c);
        }
        self.sprite_slot_view_mut(k).increment_subtype2();
        self.sprite_move_x(k);
        if self.sprite_slot_view(k).delay_aux1() == 0 {
            let delay_main = self.sprite_slot_view(k).delay_main();
            self.sprite_slot_view_mut(k)
                .set_ignore_projectile(delay_main);
            if self.sprite_slot_view(k).delay_main() == 0 {
                if self.sprite_check_tile_collision(k) != 0 {
                    self.sprite_slot_view_mut(k).set_delay_main(16);
                    self.sprite_slot_view_mut(k).set_delay_aux2(0);
                }
            } else if self.sprite_slot_view(k).delay_main() == 1 {
                let graphics = self.sprite_slot_view(k).graphics() as usize;
                let link_x_high = (self.game_state.player.follower_link.x() >> 8) as u8;
                self.sprite_slot_view_mut(k)
                    .set_x_low(ARCHERY_TARGET_RESET_X_LOWS[graphics]);
                self.sprite_slot_view_mut(k).set_x_high(link_x_high);
                self.sprite_slot_view_mut(k).set_delay_aux1(32);
                self.sprite_slot_view_mut(k).set_g(0);
            }
        }
    }

    fn set_oam_plain_at_for_prep(
        &mut self,
        oam: usize,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut().write_entry(oam, x, y, charnum, flags);
        let value = big;
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
    }

    pub(super) fn sprite_prep_bully_and_victim(&mut self, k: usize) {
        self.spawn_bully(k);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn spawn_bully(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xb9, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_view_mut(j).set_subtype2(2);
            self.sprite_slot_view_mut(j).set_head_direction(k as u8);
            self.sprite_slot_view_mut(j).set_ignore_projectile(1);
        }
    }

    pub(super) fn ball_guy_play_bounce_noise(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
    }

    pub(super) fn garnish_alloc_force(&mut self) -> i32 {
        (0..30)
            .rev()
            .find(|&k| self.garnish_slot_view(k).is_empty())
            .unwrap_or(0) as i32
    }

    pub(super) fn garnish_alloc(&mut self) -> i32 {
        (0..30)
            .rev()
            .find(|&k| self.garnish_slot_view(k).is_empty())
            .map_or(-1, |k| k as i32)
    }

    pub(super) fn garnish_alloc_low(&mut self) -> i32 {
        (0..15)
            .rev()
            .find(|&k| self.garnish_slot_view(k).is_empty())
            .map_or(-1, |k| k as i32)
    }

    pub(super) fn garnish_alloc_limit(&mut self, k: usize) -> i32 {
        (0..=k)
            .rev()
            .find(|&k| self.garnish_slot_view(k).is_empty())
            .map_or(-1, |k| k as i32)
    }

    pub(super) fn garnish_alloc_overwrite_old_low(&mut self) -> i32 {
        if let Some(k) = (0..15)
            .rev()
            .find(|&k| self.garnish_slot_view(k).is_empty())
        {
            return k as i32;
        }
        self.sprite_workspace_mut().decrement_prep_shared_counter();
        if sign8(self.game_state.sprites.workspace.prep_shared_counter()) {
            self.sprite_workspace_mut().set_prep_shared_counter(14);
        }
        self.game_state.sprites.workspace.prep_shared_counter() as i32
    }

    pub(super) fn garnish_alloc_overwrite_old(&mut self) -> i32 {
        if let Some(k) = (0..30)
            .rev()
            .find(|&k| self.garnish_slot_view(k).is_empty())
        {
            return k as i32;
        }
        self.sprite_workspace_mut().decrement_prep_shared_counter();
        if sign8(self.game_state.sprites.workspace.prep_shared_counter()) {
            self.sprite_workspace_mut().set_prep_shared_counter(29);
        }
        self.game_state.sprites.workspace.prep_shared_counter() as i32
    }

    pub(super) fn garnish_set_x(&mut self, k: usize, x: u16) {
        let value = x as u8;
        self.garnish_slot_view_mut(k).set_x_low(value);
        let value = (x >> 8) as u8;
        self.garnish_slot_view_mut(k).set_x_high(value);
    }

    pub(super) fn garnish_set_y(&mut self, k: usize, y: u16) {
        let value = y as u8;
        self.garnish_slot_view_mut(k).set_y_low(value);
        let value = (y >> 8) as u8;
        self.garnish_slot_view_mut(k).set_y_high(value);
    }

    // void Sprite_SpawnSparkleGarnish(int k) {  // 858008
    pub(super) fn sprite_spawn_sparkle_garnish(&mut self, k: usize) {
        if (self.game_state.frame.frame_counter & 3) != 0 {
            return;
        }
        let j = self.garnish_alloc_force() as usize;
        let value = 0x12;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(0x12);
        let x = self.sprite_get_x(k).wrapping_add(
            SPRITE_SPAWN_SPARKLE_GARNISH_COORD_OFFSETS[usize::from(self.get_random_number() & 3)]
                as i16 as u16,
        );
        let y = self.sprite_get_y(k).wrapping_add(
            SPRITE_SPAWN_SPARKLE_GARNISH_COORD_OFFSETS[usize::from(self.get_random_number() & 3)]
                as i16 as u16,
        );
        self.garnish_set_x(j, x);
        self.garnish_set_y(j, y);
        let value = k as u8;
        self.garnish_slot_view_mut(j).set_sprite(value);
        let value = 15;
        self.garnish_slot_view_mut(j).set_countdown(value);
    }

    // void Sprite_SpawnDummyDeathAnimation(int k) {  // 89ae7e
    pub(super) fn sprite_spawn_dummy_death_animation(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x0b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_view_mut(j).set_state(6);
            self.sprite_slot_view_mut(j).set_delay_main(15);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x14);
            self.sprite_slot_view_mut(j).set_floor(2);
        }
    }

    // void Sprite_MagicBat_SpawnLightning(int k) {  // 89aea8
    pub(super) fn sprite_magic_bat_spawn_lightning(&mut self, k: usize) {
        for _ in 0..4 {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x3a, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_sfx_queue_sfx3_with_pan(k, 1);
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_set_x(j, info.r0_x.wrapping_add(4));
                self.sprite_set_y(
                    j,
                    info.r2_y
                        .wrapping_add(12)
                        .wrapping_sub(u16::from(self.sprite_slot_view(k).z())),
                );
                self.sprite_slot_view_mut(j).set_z(0);
                self.sprite_slot_view_mut(j).set_y_velocity(24);
                self.sprite_slot_view_mut(j).set_head_direction(24);
                self.sprite_slot_view_mut(j).set_ignore_projectile(24);
                self.sprite_slot_view_mut(j).set_flags2(0x80);
                self.sprite_slot_view_mut(j).set_flags3(3);
                self.sprite_slot_view_mut(j).set_oam_flags(3);
                self.sprite_slot_view_mut(j).set_delay_main(32);
                self.sprite_slot_view_mut(j).set_graphics(2);
                let i = usize::from(self.sprite_slot_view(k).g());
                self.sprite_slot_view_mut(j)
                    .set_x_velocity(SPRITE_MAGIC_BAT_SPAWN_LIGHTNING_X_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j)
                    .set_subtype2(SPRITE_MAGIC_BAT_SPAWN_LIGHTNING_STATE2_VALUES[i]);
                self.sprite_slot_view_mut(j).set_floor(2);
                self.sprite_slot_view_mut(k).increment_g();
            }
        }
    }

    pub(super) fn garnish_spawn_pyramid_debris(&mut self, x: i8, y: i8, xvel: i8, yvel: i8) {
        let k = self.garnish_alloc_force() as usize;
        self.set_sound_effect_2(3);
        self.set_sound_effect_1(31);
        self.set_ambient_sound_effect(5);
        let value = 19;
        self.garnish_slot_view_mut(k).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(19);
        let value = 232u8.wrapping_add_signed(x);
        self.garnish_slot_view_mut(k).set_x_low(value);
        let value = 96u8.wrapping_add_signed(y);
        self.garnish_slot_view_mut(k).set_y_low(value);
        let value = xvel as u8;
        self.garnish_slot_view_mut(k).set_x_velocity(value);
        let value = yvel as u8;
        self.garnish_slot_view_mut(k).set_y_velocity(value);
        // ROM Garnish15_ArrghusSplash countdown $09:B201: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
        let value = self.get_random_number_with_carry().masked_adc(31, 48);
        self.garnish_slot_view_mut(k).set_countdown(value);
    }

    pub(super) fn kholdstare_spawn_puff_cloud_garnish(&mut self, k: usize) {
        if (k as u8 ^ self.game_state.frame.frame_counter) & 3 != 0 {
            return;
        }
        let j = self.garnish_alloc_low();
        if j < 0 {
            return;
        }
        let j = j as usize;
        let value = 7;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(7);
        let value = 31;
        self.garnish_slot_view_mut(j).set_countdown(value);
        let x = self
            .game_state
            .sprites
            .workspace
            .current_sprite_x()
            .wrapping_add_signed(i16::from(
                KHOLDSTARE_SPAWN_PUFF_CLOUD_GARNISH_XY_OFFSETS
                    [(self.get_random_number() & 7) as usize],
            ));
        let y = self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_add_signed(
                i16::from(
                    KHOLDSTARE_SPAWN_PUFF_CLOUD_GARNISH_XY_OFFSETS
                        [(self.get_random_number() & 7) as usize],
                ) + 16,
            );
        self.garnish_set_x(j, x);
        self.garnish_set_y(j, y);
        let value = 0;
        self.garnish_slot_view_mut(j).set_floor(value);
    }

    pub(super) fn garnish_flame_trail(&mut self, k: usize, is_low: bool) -> i32 {
        let j = if is_low {
            self.garnish_alloc_overwrite_old_low()
        } else {
            self.garnish_alloc_overwrite_old()
        };
        let j_usize = j as usize;
        let value = 0x10;
        self.garnish_slot_view_mut(j_usize).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(0x10);
        let value = k as u8;
        self.garnish_slot_view_mut(j_usize).set_sprite(value);
        self.garnish_set_x(j_usize, self.sprite_get_x(k));
        self.garnish_set_y(j_usize, self.sprite_get_y(k).wrapping_add(16));
        let value = 127;
        self.garnish_slot_view_mut(j_usize).set_countdown(value);
        j
    }

    pub(super) fn fire_bat_animate(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_subtype2();
        let i = ((self.sprite_slot_view(k).subtype2() >> 2) & 3) as usize;
        self.sprite_slot_view_mut(k)
            .set_graphics(FIRE_BAT_ANIMATE_GRAPHICS[i]);
    }

    pub(super) fn fire_bat_move(&mut self, k: usize) {
        self.fire_bat_animate(k);
        self.sprite_move_xy(k);

        if self.sprite_slot_view(k).subtype2() & 7 != 0 {
            return;
        }

        let j = self.garnish_flame_trail(k, true) as usize;
        let countdown = if self.sprite_slot_view(k).anim_clock() == 5 {
            0x2f
        } else {
            0x4f
        };
        self.garnish_slot_view_mut(j).set_countdown(countdown);
    }

    pub(super) fn fireball_spawn_trail_garnish(&mut self, k: usize) {
        if (k as u8 ^ self.game_state.frame.frame_counter) & 3 != 0 {
            return;
        }
        let j = self.garnish_alloc() as usize;
        let value = 8;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(8);
        let value = 11;
        self.garnish_slot_view_mut(j).set_countdown(value);
        let x = self.game_state.sprites.workspace.current_sprite_x();
        let y = self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_add(16);
        self.garnish_set_x(j, x);
        self.garnish_set_y(j, y);
        let value = k as u8;
        self.garnish_slot_view_mut(j).set_sprite(value);
    }

    pub(super) fn firesnake_spawn_fireball(&mut self, j: usize) {
        if ((j as u8) ^ self.game_state.frame.frame_counter) & 7 != 0 {
            return;
        }

        let k = self.garnish_alloc();
        if k < 0 {
            return;
        }

        let k = k as usize;
        let value = 1;
        self.garnish_slot_view_mut(k).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(1);
        let value = self.sprite_slot_view(j).x_low();
        self.garnish_slot_view_mut(k).set_x_low(value);
        let value = self.sprite_slot_view(j).x_high();
        self.garnish_slot_view_mut(k).set_x_high(value);
        self.garnish_set_y(k, self.sprite_get_y(j).wrapping_add(16));
        let value = 32;
        self.garnish_slot_view_mut(k).set_countdown(value);
        let value = j as u8;
        self.garnish_slot_view_mut(k).set_sprite(value);
        let value = self.sprite_slot_view(j).floor();
        self.garnish_slot_view_mut(k).set_floor(value);
    }

    pub(super) fn catfish_spawn_plop(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xec, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_view_mut(j).set_state(3);
            self.sprite_slot_view_mut(j).set_delay_main(15);
            self.sprite_slot_view_mut(j).set_ai_state(0);
            self.sprite_slot_view_mut(j).set_flags2(3);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
        }
    }

    pub(super) fn catfish_regurgitate_medallion(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc0, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_view_mut(j).set_x_velocity(24);
            self.sprite_slot_view_mut(j).set_z_velocity(48);
            self.sprite_slot_view_mut(j).set_a(17);
            self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
            self.sprite_slot_view_mut(j).set_flags2(0x83);
            self.sprite_slot_view_mut(j).set_flags3(0x58);
            self.sprite_slot_view_mut(j).set_oam_flags(0x58 & 0x0f);
            self.DecodeAnimatedSpriteTile_variable(0x1c);
        }
    }

    pub(super) fn sprite_spawn_water_splash(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc0, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_view_mut(j_usize).set_a(0x80);
            self.sprite_slot_view_mut(j_usize).set_flags2(2);
            self.sprite_slot_view_mut(j_usize).set_ignore_projectile(2);
            self.sprite_slot_view_mut(j_usize).set_oam_flags(4);
            self.sprite_slot_view_mut(j_usize).set_delay_main(31);
        }
        j
    }

    pub(super) fn sprite_spawn_small_splash(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0xec, &mut info, 14);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.set_sound_effect_1(0);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
            self.sprite_slot_view_mut(j_usize).set_state(3);
            self.sprite_slot_view_mut(j_usize).set_delay_main(15);
            self.sprite_slot_view_mut(j_usize).set_ai_state(0);
            self.sprite_slot_view_mut(j_usize).set_flags2(3);
        }
        j
    }

    pub(super) fn sprite_spawn_dust_cloud(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xf2, &mut info);
        if j >= 0 {
            let y_off = u16::from(self.get_random_number() & 15);
            let x_off = i16::from(self.get_random_number() & 15) - 8;
            info.r2_y = info.r2_y.wrapping_add(y_off);
            info.r0_x = info.r0_x.wrapping_add_signed(x_off);
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_view_mut(j_usize).set_subtype2(1);
        }
        j
    }

    pub(super) fn sprite_spawn_superficial_bomb_blast(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_slot_view_mut(j_usize).set_state(6);
            self.sprite_slot_view_mut(j_usize).set_delay_aux1(31);
            self.sprite_slot_view_mut(j_usize).set_c(3);
            self.sprite_slot_view_mut(j_usize).set_flags2(3);
            self.sprite_slot_view_mut(j_usize).set_oam_flags(4);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x15);
            self.sprite_set_spawned_coordinates(j_usize, &info);
        }
        j
    }

    pub(super) fn sprite_spawn_bomb(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_view_mut(j_usize).set_sprite_type(0x4a);
            self.sprite_slot_view_mut(j_usize).set_c(1);
            self.sprite_slot_view_mut(j_usize).set_delay_aux1(255);
            self.sprite_slot_view_mut(j_usize).set_flags3(0x18);
            self.sprite_slot_view_mut(j_usize).set_oam_flags(8);
            self.sprite_slot_view_mut(j_usize).set_health(0);
            self.sprite_slot_view_mut(j_usize).set_delay_aux1(80);
            self.sprite_slot_view_mut(j_usize).set_x_velocity(24);
            self.sprite_slot_view_mut(j_usize).set_z_velocity(48);
        }
        j
    }

    pub(super) fn spawn_boss_poof(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xce, &mut info);
        let j_usize = j as usize;
        self.sprite_set_x(j_usize, info.r0_x.wrapping_add(16));
        self.sprite_set_y(j_usize, info.r2_y.wrapping_add(40));
        self.sprite_slot_view_mut(j_usize).set_graphics(0x0f);
        self.sprite_slot_view_mut(j_usize).set_a(1);
        self.sprite_slot_view_mut(j_usize).set_delay_main(47);
        self.sprite_slot_view_mut(j_usize).set_flags2(9);
        self.sprite_slot_view_mut(j_usize).set_ignore_projectile(9);
        self.set_sound_effect_1(12);
        j
    }

    pub(super) fn sprite_spawn_fireball(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
        let j = self.sprite_spawn_dynamically_ex(k, 0x55, &mut info, 13);
        if j < 0 {
            return j;
        }

        let j_usize = j as usize;
        self.sprite_set_x(j_usize, info.r0_x.wrapping_add(4));
        self.sprite_set_y(
            j_usize,
            info.r2_y.wrapping_add(4).wrapping_sub(u16::from(info.r4_z)),
        );
        self.sprite_slot_view_mut(j_usize)
            .masked_or_flags3(0xfe, 0x40);
        self.sprite_slot_view_mut(j_usize).set_oam_flags(6);
        self.sprite_slot_view_mut(j_usize).set_flags4(0x54);
        self.sprite_slot_view_mut(j_usize).set_e(0x54);
        self.sprite_slot_view_mut(j_usize).set_flags2(0x20);
        self.sprite_apply_speed_towards_link(j_usize, 0x20);
        self.sprite_slot_view_mut(j_usize).set_delay_main(20);
        self.sprite_slot_view_mut(j_usize).set_delay_aux1(16);
        self.sprite_slot_view_mut(j_usize).set_flags5(0);
        self.sprite_slot_view_mut(j_usize).set_deflection_bits(0x48);
        j
    }

    pub(super) fn sprite_spawn_fire_phlegm(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xa5, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_sfx_queue_sfx3_with_pan(k, 5);
            self.sprite_set_spawned_coordinates(j_usize, &info);
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_set_x(
                j_usize,
                info.r0_x
                    .wrapping_add_signed(i16::from(SPRITE_SPAWN_FIRE_PHLEGM_X_OFFSETS[i])),
            );
            self.sprite_set_y(
                j_usize,
                info.r2_y
                    .wrapping_add_signed(i16::from(SPRITE_SPAWN_FIRE_PHLEGM_Y_OFFSETS[i])),
            );
            self.sprite_slot_view_mut(j_usize)
                .set_x_velocity(SPRITE_SPAWN_FIRE_PHLEGM_X_VELOCITIES[i] as u8);
            self.sprite_slot_view_mut(j_usize)
                .set_y_velocity(SPRITE_SPAWN_FIRE_PHLEGM_Y_VELOCITIES[i] as u8);
            self.sprite_slot_view_mut(j_usize).or_flags3(0x40);
            self.sprite_slot_view_mut(j_usize).set_deflection_bits(0x40);
            self.sprite_slot_view_mut(j_usize).set_flags2(0x21);
            self.sprite_slot_view_mut(j_usize).set_b(0x21);
            self.sprite_slot_view_mut(j_usize).set_oam_flags(2);
            self.sprite_slot_view_mut(j_usize).set_flags4(0x14);
            self.sprite_slot_view_mut(j_usize).set_ignore_projectile(20);
            self.sprite_slot_view_mut(j_usize).set_bump_damage(37);
            if self.game_state.inventory.items.shield_type() >= 3 {
                self.sprite_slot_view_mut(j_usize).set_flags5(0x20);
            }
        }
        j
    }

    pub(super) fn lumberjack_tree_spawn_leaves(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x3b, &mut info);
        assert!(
            j >= 0,
            "LumberjackTree_SpawnLeaves expected Sprite_SpawnDynamically to succeed"
        );
        let j = j as usize;
        self.sprite_slot_view_mut(j).set_graphics(2);
        let z_velocity = self.sprite_slot_view(k).z_velocity();
        self.sprite_slot_view_mut(j).set_z_velocity(z_velocity);
        self.sprite_slot_view_mut(j).set_subtype2(1);
        self.sprite_slot_view_mut(j).set_ai_state(2);
        self.sprite_slot_view_mut(j).set_delay_main(8);
        self.sprite_set_spawned_coordinates(j, &info);
        j as i32
    }

    pub(super) fn sprite_spawn_poof_garnish(&mut self, j: usize) {
        let k = self.garnish_alloc_force() as usize;
        let value = 10;
        self.garnish_slot_view_mut(k).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(10);
        let value = self.sprite_slot_view(j).x_low();
        self.garnish_slot_view_mut(k).set_x_low(value);
        let value = self.sprite_slot_view(j).x_high();
        self.garnish_slot_view_mut(k).set_x_high(value);
        let y = self.sprite_get_y(j).wrapping_add(16);
        let value = y as u8;
        self.garnish_slot_view_mut(k).set_y_low(value);
        let value = (y >> 8) as u8;
        self.garnish_slot_view_mut(k).set_y_high(value);
        let native_floor = self.sprite_slot_view(j).floor();
        let value = native_floor;
        self.garnish_slot_view_mut(k).set_sprite(value);
        let value = 15;
        self.garnish_slot_view_mut(k).set_countdown(value);
    }

    pub(super) fn octorok_fire_loogie(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        self.sprite_sfx_queue_sfx2_with_pan(k, 7);
        let j = self.sprite_spawn_dynamically(k, 0x0c, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_set_x(
                j,
                info.r0_x
                    .wrapping_add_signed(i16::from(OCTOROK_FIRE_LOOGIE_X_OFFSETS[i])),
            );
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add_signed(i16::from(OCTOROK_FIRE_LOOGIE_Y_OFFSETS[i])),
            );
            self.sprite_slot_view_mut(j)
                .set_x_velocity(OCTOROK_FIRE_LOOGIE_X_VELOCITIES[i] as u8);
            self.sprite_slot_view_mut(j)
                .set_y_velocity(OCTOROK_FIRE_LOOGIE_Y_VELOCITIES[i] as u8);
        }
    }

    pub(super) fn moblin_materialize_spear(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x1b, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_slot_view_mut(j).set_a(3);
            self.sprite_slot_view_mut(j).set_direction(i as u8);
            self.sprite_set_x(
                j,
                info.r0_x
                    .wrapping_add_signed(i16::from(MOBLIN_MATERIALIZE_SPEAR_X_OFFSETS[i])),
            );
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add_signed(i16::from(MOBLIN_MATERIALIZE_SPEAR_Y_OFFSETS[i])),
            );
            self.sprite_slot_view_mut(j)
                .set_x_velocity(MOBLIN_MATERIALIZE_SPEAR_X_VELOCITIES[i] as u8);
            self.sprite_slot_view_mut(j)
                .set_y_velocity(MOBLIN_MATERIALIZE_SPEAR_Y_VELOCITIES[i] as u8);
        }
    }

    pub(super) fn snitch_spawn_guard(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0x45, &mut info, 0);
        if j < 0 {
            return;
        }

        let j = j as usize;
        let i = match self.sprite_slot_view(k).sprite_type() {
            0x3d => 0,
            0x35 => 1,
            _ => 2,
        };
        let x_base = self.game_state.sprites.garnish_runtime.sprcoll_x_word() & 0xff00;
        let y_base = self.game_state.sprites.garnish_runtime.sprcoll_y_word() & 0xff00;
        self.sprite_set_x(j, SNITCH_SPAWN_GUARD_X_OFFSETS[i].wrapping_add(x_base));
        self.sprite_set_y(j, SNITCH_SPAWN_GUARD_Y_OFFSETS[i].wrapping_add(y_base));
        self.sprite_slot_view_mut(j).set_floor(0);
        self.sprite_slot_view_mut(j).set_health(4);
        self.sprite_slot_view_mut(j).set_deflection_bits(0x80);
        self.sprite_slot_view_mut(j).set_flags5(0x90);
        self.sprite_slot_view_mut(j).set_oam_flags(0x0b);
    }

    pub(super) fn ancilla_terminate_sparkle_objects(&mut self) {
        for i in (0..=4).rev() {
            let t = self.ancilla_slot_view(i).ancilla_type();
            if matches!(t, 0x2a | 0x2b | 0x30 | 0x31 | 0x18 | 0x19 | 0x0c) {
                self.ancilla_slot_view_mut(i).clear();
            }
        }
    }

    pub(super) fn kodongo_set_direction(&mut self, k: usize) {
        let j = self.sprite_slot_view(k).direction() as usize;
        self.sprite_slot_view_mut(k)
            .set_x_velocity(KODONGO_SET_DIRECTION_X_VELOCITIES[j] as u8);
        self.sprite_slot_view_mut(k)
            .set_y_velocity(KODONGO_SET_DIRECTION_Y_VELOCITIES[j] as u8);
    }

    pub(super) fn kodongo_spawn_fire(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0x87, &mut info, 13);
        if j >= 0 {
            let j = j as usize;
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_set_x(
                j,
                info.r0_x
                    .wrapping_add_signed(i16::from(KODONGO_SPAWN_FIRE_X_OFFSETS[i])),
            );
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add_signed(i16::from(KODONGO_SPAWN_FIRE_Y_OFFSETS[i])),
            );
            self.sprite_slot_view_mut(j)
                .set_x_velocity(KODONGO_SPAWN_FIRE_X_VELOCITIES[i] as u8);
            self.sprite_slot_view_mut(j)
                .set_y_velocity(KODONGO_SPAWN_FIRE_Y_VELOCITIES[i] as u8);
            self.sprite_slot_view_mut(j).set_ignore_projectile(1);
        }
    }

    pub(super) fn create_six_blue_balls(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
        self.temp_counter_mut().set(5);
        loop {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x55, &mut info);
            if j >= 0 {
                let j = j as usize;
                let i = self.game_state.scratch_counter.value() as usize;
                self.sprite_set_x(j, info.r0_x.wrapping_add(4));
                self.sprite_set_y(j, info.r2_y.wrapping_add(4));
                self.sprite_slot_view_mut(j).masked_or_flags3(!1, 0x40);
                self.sprite_slot_view_mut(j).set_oam_flags(4);
                self.sprite_slot_view_mut(j).set_delay_aux1(4);
                self.sprite_slot_view_mut(j).set_flags4(20);
                self.sprite_slot_view_mut(j).set_c(20);
                self.sprite_slot_view_mut(j).set_e(20);
                self.sprite_slot_view_mut(j)
                    .set_x_velocity(CREATE_SIX_BLUE_BALLS_X_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j)
                    .set_y_velocity(CREATE_SIX_BLUE_BALLS_Y_VELOCITIES[i] as u8);
            }

            self.temp_counter_mut().decrement();
            if sign8(self.game_state.scratch_counter.value()) {
                break;
            }
        }
        self.temp_counter_mut().set(0);
    }

    pub(super) fn lanmola_spawn_shrapnel(&mut self, k: usize) {
        let state_sum = self
            .sprite_slot_view(0)
            .state()
            .wrapping_add(self.sprite_slot_view(1).state())
            .wrapping_add(self.sprite_slot_view(2).state());
        let shrapnel_countdown = if state_sum < 10 { 7 } else { 3 };
        self.temp_counter_mut().set(shrapnel_countdown);

        // The ROM's coordinate stores are `ADC #$04` with NO preceding CLC
        // ($1A:F9A2/$1A:F9A8; the C port patches the ROM to hide this). The
        // countdown selection's `CMP #$0A : BCS` leaves carry = (state sum
        // >= 10), and nothing on the indoor spawn path
        // (Sprite_SpawnDynamically, Sprite_SetSpawnedCoordinates,
        // SpritePrep_LoadProperties) touches carry, so the first spawn adds
        // it; every later spawn adds the previous GetRandomNumber's
        // final-LSR carry, and y_lo chains the x_lo add's carry-out (route
        // frame 148432).
        let mut carry = state_sum >= 10;
        loop {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xc2, &mut info);
            if j >= 0 {
                let j = j as usize;
                let i = self.game_state.scratch_counter.value() as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                let x_sum = u16::from(info.r0_x as u8) + 4 + u16::from(carry as u8);
                self.sprite_slot_view_mut(j).set_x_low(x_sum as u8);
                let y_sum = u16::from(info.r2_y as u8) + 4 + u16::from(x_sum > 0xff);
                self.sprite_slot_view_mut(j).set_y_low(y_sum as u8);
                self.sprite_slot_view_mut(j).set_ignore_projectile(1);
                self.sprite_slot_view_mut(j).set_bump_damage(1);
                self.sprite_slot_view_mut(j).set_flags4(1);
                self.sprite_slot_view_mut(j).set_z(0);
                self.sprite_slot_view_mut(j).set_flags2(0x20);
                self.sprite_slot_view_mut(j)
                    .set_x_velocity(LANMOLA_SPAWN_SHRAPNEL_X_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j)
                    .set_y_velocity(LANMOLA_SPAWN_SHRAPNEL_Y_VELOCITIES[i] as u8);
                let random = self.get_random_number_with_carry();
                self.sprite_slot_view_mut(j)
                    .set_graphics(random.value() & 1);
                carry = random.carry();
            }

            self.temp_counter_mut().decrement();
            if sign8(self.game_state.scratch_counter.value()) {
                break;
            }
        }
    }

    pub(super) fn octoballoon_form_babby(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
        for i in (0..=5).rev() {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x10, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_slot_view_mut(j)
                    .set_x_velocity(OCTOBALLOON_FORM_BABBY_X_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j)
                    .set_y_velocity(OCTOBALLOON_FORM_BABBY_Y_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j).set_z_velocity(48);
                self.sprite_slot_view_mut(j).set_subtype2(255);
            }
        }
    }

    pub(super) fn pink_ball_handle_message(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux4() != 0 {
            return;
        }
        let msg = if self.game_state.inventory.items.moon_pearl() & 1 != 0 {
            0x15c
        } else {
            0x15b
        };
        if self.sprite_show_message_on_contact(k, msg) & 0x100 != 0 {
            self.sprite_slot_view_mut(k).xor_x_velocity(255);
            self.sprite_slot_view_mut(k).xor_y_velocity(255);
            if self.sprite_slot_view(k).e() != 0 {
                self.ball_guy_play_bounce_noise(k);
            }
            self.sprite_slot_view_mut(k).set_delay_aux4(64);
        }
    }

    pub(super) fn bully_handle_message(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux4() != 0 {
            return;
        }
        let msg = if self.game_state.inventory.items.moon_pearl() & 1 != 0 {
            0x15e
        } else {
            0x15d
        };
        if self.sprite_show_message_on_contact(k, msg) & 0x100 != 0 {
            self.sprite_slot_view_mut(k).xor_x_velocity(255);
            self.sprite_slot_view_mut(k).xor_y_velocity(255);
            self.sprite_slot_view_mut(k).set_delay_aux4(64);
        }
    }

    pub(super) fn rupee_pull_spawn_prize(&mut self, k: usize) {
        if self.game_state.sprite_battle.sprites_killed() != 0 {
            let shared_scratch_a = if self.game_state.sprite_battle.sprites_killed() < 4 {
                0
            } else if self.game_state.sprite_battle.times_hurt_by_sprites() != 0 {
                1
            } else {
                2
            };
            self.sprite_workspace_mut()
                .set_shared_scratch_a(shared_scratch_a);
            self.temp_counter_mut().set(3);
            loop {
                let mut info = SpriteSpawnInfo::default();
                let what = RUPEE_PULL_SPAWN_PRIZE_TYPES
                    [self.game_state.sprites.workspace.shared_scratch_a() as usize];
                let j = self.sprite_spawn_dynamically(k, what, &mut info);
                if j < 0 {
                    break;
                }

                let j = j as usize;
                let i = self.game_state.scratch_counter.value() as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_slot_view_mut(j)
                    .set_x_velocity(RUPEE_PULL_SPAWN_PRIZE_X_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j)
                    .set_y_velocity(RUPEE_PULL_SPAWN_PRIZE_Y_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j).set_stunned(255);
                self.sprite_slot_view_mut(j).set_delay_aux4(32);
                let value = 32;
                self.sprite_slot_view_mut(j).set_delay_aux3(value);
                self.sprite_slot_view_mut(j).set_z_velocity(32);

                self.temp_counter_mut().decrement();
                if sign8(self.game_state.scratch_counter.value()) {
                    break;
                }
            }
        }
        self.sprite_battle_mut().clear_sprites_killed();
        self.sprite_battle_mut().clear_times_hurt_by_sprites();
    }

    pub(super) fn sluggula_drop_bomb(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0x4a, &mut info, 11);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_view_mut(j_usize).set_sprite_type(0x4a);
            self.sprite_slot_view_mut(j_usize).set_c(1);
            self.sprite_slot_view_mut(j_usize).set_delay_aux1(255);
            self.sprite_slot_view_mut(j_usize).set_flags3(0x18);
            self.sprite_slot_view_mut(j_usize).set_oam_flags(8);
            self.sprite_slot_view_mut(j_usize).set_health(0);
        }
    }

    pub(super) fn talking_tree_spawn_bomb(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_slot_view_mut(j_usize).set_sprite_type(0x4a);
            self.sprite_slot_view_mut(j_usize).set_c(1);
            self.sprite_slot_view_mut(j_usize).set_delay_aux1(255);
            self.sprite_slot_view_mut(j_usize).set_flags3(0x18);
            self.sprite_slot_view_mut(j_usize).set_oam_flags(8);
            self.sprite_slot_view_mut(j_usize).set_health(0);
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_view_mut(j_usize).set_delay_aux1(64);
            self.sprite_slot_view_mut(j_usize).set_y_velocity(24);
            self.sprite_slot_view_mut(j_usize).set_z_velocity(18);
        }
    }

    pub(super) fn pirogusu_spawn_splash(&mut self, k: usize) {
        if (k as u8 ^ self.game_state.frame.frame_counter) & 3 != 0 {
            return;
        }
        let x =
            PIROGUSU_SPAWN_SPLASH_SPLASH_JITTER_OFFSETS[(self.get_random_number() & 3) as usize];
        let y =
            PIROGUSU_SPAWN_SPLASH_SPLASH_JITTER_OFFSETS[(self.get_random_number() & 3) as usize];
        let j = self.garnish_alloc_low();
        if j >= 0 {
            let j_usize = j as usize;
            let value = 11;
            self.garnish_slot_view_mut(j_usize).set_garnish_type(value);
            self.garnish_state_mut().set_active_type(11);
            self.garnish_set_x(j_usize, self.sprite_get_x(k).wrapping_add(u16::from(x)));
            self.garnish_set_y(
                j_usize,
                self.sprite_get_y(k)
                    .wrapping_add(u16::from(y))
                    .wrapping_add(16),
            );
            let value = 15;
            self.garnish_slot_view_mut(j_usize).set_countdown(value);
        }
    }

    pub(super) fn lightning_spawn_garnish(&mut self, k: usize) {
        let j = self.garnish_alloc_overwrite_old() as usize;
        let value = 9;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(9);
        let value = self.sprite_slot_view(k).a();
        self.garnish_slot_view_mut(j).set_sprite(value);
        let value = self.sprite_slot_view(k).x_low();
        self.garnish_slot_view_mut(j).set_x_low(value);
        let value = self.sprite_slot_view(k).x_high();
        self.garnish_slot_view_mut(j).set_x_high(value);
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(16));
        let value = 32;
        self.garnish_slot_view_mut(j).set_countdown(value);
    }

    pub(super) fn laser_beam_build_up_garnish(&mut self, k: usize) {
        let j = self.garnish_alloc_overwrite_old() as usize;
        let value = 4;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(4);
        self.garnish_set_x(j, self.sprite_get_x(k));
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(16));
        let value = 16;
        self.garnish_slot_view_mut(j).set_countdown(value);
        let value = self.sprite_slot_view(k).graphics();
        self.garnish_slot_view_mut(j).set_oam_flags(value);
        let value = k as u8;
        self.garnish_slot_view_mut(j).set_sprite(value);
        let value = self.sprite_slot_view(k).floor();
        self.garnish_slot_view_mut(j).set_floor(value);
    }

    pub(super) fn laser_eye_fire_beam(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x95, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_slot_view_mut(j)
                .set_graphics((i as u8 & 2) >> 1);
            self.sprite_set_x(
                j,
                info.r0_x
                    .wrapping_add_signed(i16::from(LASER_EYE_FIRE_BEAM_SPAWN_XY[i])),
            );
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add_signed(i16::from(LASER_EYE_FIRE_BEAM_SPAWN_XY[i + 2])),
            );
            self.sprite_slot_view_mut(j)
                .set_x_velocity(LASER_EYE_FIRE_BEAM_SPAWN_XYVEL[i] as u8);
            self.sprite_slot_view_mut(j)
                .set_y_velocity(LASER_EYE_FIRE_BEAM_SPAWN_XYVEL[i + 2] as u8);
            self.sprite_slot_view_mut(j).set_flags2(0x20);
            self.sprite_slot_view_mut(j).set_a(0x20);
            self.sprite_slot_view_mut(j).set_oam_flags(5);
            self.sprite_slot_view_mut(j).set_deflection_bits(0x48);
            self.sprite_slot_view_mut(j).set_ignore_projectile(0x48);
            self.sprite_slot_view_mut(j).set_delay_main(5);
            if self.game_state.inventory.items.shield_type() == 3 {
                self.sprite_slot_view_mut(j).set_flags5(32);
            }
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
        }
    }

    pub(super) fn get_position_relative_to_the_great_overlord_ganon(&mut self, k: usize) {
        let j = self.sprite_slot_view(0).direction() as usize;
        let home = self.armos_knight_home_position(k);
        let x = home.x();
        let y = home.y();
        self.sprite_set_x(
            k,
            x.wrapping_add_signed(i16::from(
                GET_POSITION_RELATIVE_TO_THE_GREAT_OVERLORD_GANON_X_OFFSETS[j],
            )),
        );
        self.sprite_set_y(
            k,
            y.wrapping_add_signed(i16::from(
                GET_POSITION_RELATIVE_TO_THE_GREAT_OVERLORD_GANON_Y_OFFSETS[j],
            )),
        );
    }

    pub(super) fn sasha_idle(&mut self, k: usize) {
        let inventory = &self.game_state.inventory.items;
        let resources = &self.game_state.inventory.player_resources;
        if resources.pendant_flags() & 4 == 0 {
            if self.sprite_show_solicited_message(k, 0x32) & 0x100 != 0 {
                self.sprite_slot_view_mut(k).set_ai_state(1);
            }
        } else if !inventory.has_boots() {
            let m = if self
                .game_state
                .inventory
                .save_progress
                .map_icons_indicator()
                >= 3
            {
                0x38
            } else {
                0x39
            };
            if self.sprite_show_solicited_message(k, m) & 0x100 != 0 {
                self.sprite_slot_view_mut(k).set_ai_state(2);
            }
        } else if inventory.ice_rod() == 0 {
            self.sprite_show_solicited_message(k, 0x37);
        } else if resources.pendant_flags() & 7 != 7 {
            self.sprite_show_solicited_message(k, 0x34);
        } else if inventory.sword_type() < 2 {
            self.sprite_show_solicited_message(k, 0x30);
        } else {
            self.sprite_show_solicited_message(k, 0x31);
        }
        let graphics = self.game_state.frame.frame_counter >> 5 & 1;
        self.sprite_slot_view_mut(k).set_graphics(graphics);
    }

    pub(super) fn old_man_revert_to_sprite(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xad, &mut info);
        if j < 0 {
            return;
        }
        let j = j as usize;
        let tagalong = self.tagalong_slot(k);
        let direction = tagalong.direction();
        let y = tagalong.y();
        let x = tagalong.x();
        self.sprite_slot_view_mut(j).set_direction(direction);
        self.sprite_slot_view_mut(j).set_head_direction(direction);
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.sprite_set_y(j, y.wrapping_add(2));
        self.sprite_set_x(j, x.wrapping_add(2));
        self.sprite_slot_view_mut(j).set_floor(floor);
        self.sprite_slot_view_mut(j).set_ignore_projectile(1);
        self.sprite_slot_view_mut(j).set_subtype2(1);
        self.old_man_enable_cutscene();
        self.follower_state_mut().set_indicator(0);
        self.follower_link_state_mut().set_speed_setting(0);
    }

    pub(super) fn old_man_enable_cutscene(&mut self) {
        self.follower_link_state_mut().immobilize();
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
    }

    /// Source suffix after Sprite_OldMan's `Link_ReceiveItem(0x1a, 0)`:
    /// the starting point, cutscene enable, and the shuffle-away velocities.
    pub(super) fn complete_old_man_mirror_item_receipt(&mut self, k: usize) {
        self.save_progress_mut().set_which_starting_point(1);
        self.old_man_enable_cutscene();
        self.sprite_slot_view_mut(k).set_delay_main(48);
        self.sprite_slot_view_mut(k).set_x_velocity(8);
        self.sprite_slot_view_mut(k).set_y_velocity(4);
        self.sprite_slot_view_mut(k).set_direction(3);
        self.sprite_slot_view_mut(k).set_head_direction(3);
    }

    pub(super) fn sprite_ad_old_man(&mut self, k: usize) {
        self.old_mountain_man_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.sprite_slot_view(k).subtype2() {
            0 => match self.sprite_slot_view(k).ai_state() {
                0 => {
                    self.sprite_track_body_to_head(k);
                    let dir = self.sprite_direction_to_face_link(k, None) ^ 3;
                    self.sprite_slot_view_mut(k).set_head_direction(dir);
                    let j = self.sprite_show_message_on_contact(k, 0x9c);
                    if j & 0x100 != 0 {
                        self.sprite_slot_view_mut(k).set_direction(j as u8);
                        self.sprite_slot_view_mut(k).set_head_direction(j as u8);
                        self.sprite_slot_view_mut(k).set_ai_state(1);
                    }
                }
                1 => {
                    self.follower_state_mut().set_indicator(4);
                    self.sprite_become_follower(k);
                    self.save_progress_mut().set_which_starting_point(5);
                    self.sprite_slot_view_mut(k).set_state(0);
                    self.cache_camera_properties();
                }
                _ => {}
            },
            1 => {
                self.sprite_move_xy(k);
                match self.sprite_slot_view(k).ai_state() {
                    0 => {
                        self.sprite_slot_view_mut(k).increment_ai_state();
                        self.follower_link_state_mut().set_item_receipt_method(0);
                        if self
                            .link_receive_item_from(
                                0x1a,
                                0,
                                ItemReceiptCaller::SpriteMainDirect {
                                    sprite_slot: k as u8,
                                    suffix: SpriteMainItemReceiptSuffix::OldManMirror,
                                },
                            )
                            .is_suspended()
                        {
                            return;
                        }
                        self.complete_old_man_mirror_item_receipt(k);
                    }
                    1 => {
                        self.old_man_enable_cutscene();
                        if self.sprite_slot_view(k).delay_main() == 0 {
                            self.sprite_slot_view_mut(k).increment_ai_state();
                        }
                        let graphics = ((k as u8) ^ self.game_state.frame.frame_counter) >> 3 & 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    }
                    2 => {
                        self.sprite_slot_view_mut(k).set_head_direction(0);
                        self.sprite_slot_view_mut(k).set_direction(0);
                        let j = self
                            .game_state
                            .sprites
                            .garnish_runtime
                            .active_overlord_index() as usize;
                        let overlord = self.overlord_slot_view(j);
                        let x = overlord.x();
                        let y = overlord.y();
                        if y >= self.sprite_get_y(k) {
                            self.sprite_slot_view_mut(k).increment_ai_state();
                            self.sprite_slot_view_mut(k).set_y_velocity(0);
                            self.sprite_slot_view_mut(k).set_x_velocity(0);
                        } else {
                            let pt = self.sprite_project_speed_towards_location(k, x, y, 8);
                            self.sprite_slot_view_mut(k).set_y_velocity(pt.y);
                            self.sprite_slot_view_mut(k).set_x_velocity(pt.x);
                            let graphics =
                                ((k as u8) ^ self.game_state.frame.frame_counter) >> 3 & 1;
                            self.sprite_slot_view_mut(k).set_graphics(graphics);
                            self.old_man_enable_cutscene();
                        }
                    }
                    3 => {
                        self.sprite_slot_view_mut(k).set_state(0);
                        self.follower_link_state_mut().clear_immobilized();
                        self.follower_link_state_mut()
                            .clear_sprite_damage_disable_timer();
                    }
                    _ => {}
                }
            }
            2 => {
                self.sprite_behave_as_barrier(k);
                if self.sprite_slot_view(k).ai_state() != 0 {
                    self.player_resources_mut().set_heart_filler(160);
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                }
                let j = if self.game_state.inventory.save_progress.progress_indicator() >= 3 {
                    2
                } else {
                    self.game_state.inventory.items.moon_pearl() as usize
                };
                if self.sprite_show_solicited_message(k, SPRITE_AD_OLD_MAN_OLD_MOUNTAIN_MAN_MSGS[j])
                    & 0x100
                    != 0
                {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                }
            }
            _ => {}
        }
    }

    pub(super) fn sprite_39_locksmith(&mut self, k: usize) {
        self.middle_aged_man_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_show_solicited_message(k, 0x107);
                let bak = self.sprite_slot_view(k).x_low();
                self.sprite_slot_view_mut(k).subtract_x_low(16);
                self.sprite_get_16bit_coords_for_prep(k);
                self.sprite_slot_view_mut(k).set_x_velocity(1);
                self.sprite_slot_view_mut(k).set_y_velocity(1);
                if self.sprite_check_tile_collision(k) == 0 {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    if self.game_state.sprites.follower_runtime.indicator() != 0 {
                        self.sprite_slot_view_mut(k).set_ai_state(5);
                    }
                }
                self.sprite_slot_view_mut(k).set_x_low(bak);
            }
            1 => {
                self.follower_state_mut().set_indicator(9);
                self.follower_state_mut().set_appearance_none_flag(0);
                self.load_follower_graphics();
                self.follower_initialize();
                self.start_shared_message_timer(0x40);
                self.sprite_slot_view_mut(k).set_state(0);
            }
            2 => {
                if self.sprite_check_if_link_is_busy() {
                    return;
                }
                let j = if self.game_state.sprites.follower_runtime.dropped() != 0 {
                    self.sprite_show_solicited_message(k, 0x109)
                } else {
                    self.sprite_show_message_on_contact(k, 0x109)
                };
                if j & 0x100 != 0 {
                    self.sprite_slot_view_mut(k).set_ai_state(3);
                }
            }
            3 => {
                if self.multiselect_choice().value() == 0 {
                    if self.game_state.sprites.follower_runtime.dropped() != 0 {
                        self.sprite_show_message_unconditional(0x10c);
                        self.sprite_slot_view_mut(k).set_ai_state(2);
                    } else {
                        self.follower_link_state_mut().set_item_receipt_method(0);
                        self.link_receive_item(0x16, 0);
                        self.save_progress_mut().or_progress_indicator_3(0x10);
                        self.sprite_slot_view_mut(k).set_ai_state(4);
                        self.follower_state_mut().set_indicator(0);
                    }
                } else {
                    self.sprite_show_message_unconditional(0x10a);
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                }
            }
            4 => {
                self.sprite_show_solicited_message(k, 0x10b);
            }
            5 => {
                self.sprite_show_solicited_message(k, 0x107);
            }
            _ => {}
        }
    }

    pub(super) fn sprite_3_a_magic_bat(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() != 0 {
            self.sprite_mad_batter_bolt(k);
            return;
        }

        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_draw_single_large(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_move_z(k);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self
                    .game_state
                    .inventory
                    .player_resources
                    .magic_consumption_level()
                    >= 2
                    || !self.sprite_check_damage_to_link_same_layer(k)
                {
                    return;
                }
                for i in (0..=4).rev() {
                    if self.ancilla_slot_view(i).ancilla_type() == 0x1a {
                        self.sprite_spawn_superficial_bomb_blast(k);
                        self.sprite_sfx_queue_sfx1_with_pan(k, 0x0d);
                        self.sprite_slot_view_mut(k).increment_ai_state();
                        self.sprite_slot_view_mut(k).set_a(20);
                        self.follower_link_state_mut().immobilize();
                        self.sprite_slot_view_mut(k).or_oam_flags(32);
                        return;
                    }
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).decrement_a();
                    let delay_main = self.sprite_slot_view(k).a();
                    self.sprite_slot_view_mut(k).set_delay_main(delay_main);
                    if self.sprite_slot_view(k).delay_main() != 1 {
                        let z_velocity = self.sprite_slot_view(k).delay_main() >> 2;
                        self.sprite_slot_view_mut(k).set_z_velocity(z_velocity);
                        let idx = (self.sprite_slot_view(k).a() & 1) as usize;
                        self.sprite_slot_view_mut(k)
                            .add_x_velocity(MAGIC_BAT_RISING_UP_X_ACCELERATIONS[idx] as u8);
                        self.sprite_slot_view_mut(k).xor_graphics(1);
                    } else {
                        self.sprite_show_message_unconditional(0x110);
                        self.sprite_slot_view_mut(k).increment_ai_state();
                        self.sprite_slot_view_mut(k).set_graphics(0);
                        self.sprite_slot_view_mut(k).set_z_velocity(0);
                        self.sprite_slot_view_mut(k).set_x_velocity(0);
                        self.sprite_slot_view_mut(k).set_delay_main(255);
                    }
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).set_delay_aux1(64);
                }
                let idx = ((self.sprite_slot_view(k).delay_main() >> 1) & 7) as usize;
                self.sprite_slot_view_mut(k)
                    .masked_or_oam_flags(!0x0e, MAGIC_BAT_LIGHTNING_OAM_FLAG_SEQUENCE[idx]);
                if self.sprite_slot_view(k).delay_main() == 240 {
                    self.sprite_magic_bat_spawn_lightning(k);
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    self.sprite_show_message_unconditional(0x111);
                    self.Palette_Restore_BG_And_HUD();
                    self.increment_cgram_update_flag();
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.player_resources_mut().set_magic_consumption_level(1);
                    self.hud_refresh_icon();
                } else if self.sprite_slot_view(k).delay_aux1() == 0x10 {
                    self.attract_scene_mut().set_intro_palette_flash_count(0x10);
                }
            }
            4 => {
                self.sprite_spawn_dummy_death_animation(k);
                self.sprite_slot_view_mut(k).set_state(0);
                self.follower_link_state_mut().clear_immobilized();
            }
            _ => {}
        }
    }

    pub(super) fn sprite_72_fairy_pond(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() != 0 {
            self.sprite_slot_view_mut(k).decrement_c();
            if self.sprite_slot_view(k).c() == 0 {
                self.sprite_slot_view_mut(k).set_state(0);
            }
            let graphics = self.sprite_slot_view(k).c() >> 3;
            self.sprite_slot_view_mut(k).set_graphics(graphics);
            self.oam_allocate_from_region_c(4);
            self.sprite_draw_single_small(k);
            return;
        }
        if self.sprite_slot_view(k).b() != 0 {
            self.faerie_queen_draw(k);
            let graphics = self.game_state.frame.frame_counter >> 4 & 1;
            self.sprite_slot_view_mut(k).set_graphics(graphics);
            if self.game_state.frame.frame_counter & 15 != 0 {
                return;
            }
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x72, &mut info);
            if j >= 0 {
                let j = j as usize;
                let xoff =
                    WISH_POND_SPARKLE_X_OFFSETS[(self.get_random_number() & 7) as usize] as u16;
                let yoff =
                    WISH_POND_SPARKLE_Y_OFFSETS[(self.get_random_number() & 7) as usize] as u16;
                self.sprite_set_x(j, info.r0_x.wrapping_add(xoff));
                self.sprite_set_y(j, info.r2_y.wrapping_add(yoff));
                self.sprite_slot_view_mut(j).set_c(31);
                self.sprite_slot_view_mut(j).set_a(31);
                self.sprite_slot_view_mut(j).set_flags2(0);
                self.sprite_slot_view_mut(j).set_flags3(0x48);
                self.sprite_slot_view_mut(j).set_oam_flags(0x48 & 0x0f);
                self.sprite_slot_view_mut(j).set_b(1);
            }
            return;
        }
        let _ = self.sprite_prep_oam_coord_or_double_ret(k);
        self.sprite_wish_pond2(k);
    }

    /// Source suffix after Sprite_HappinessPond case 9's
    /// `Link_ReceiveItem(gfx, 0)` (ROM `$86c44c`): nothing observable remains
    /// (the AI-state advance precedes the call in the ROM).
    pub(super) fn complete_happiness_pond_item_receipt(&mut self, k: usize) {
        debug_assert_eq!(self.sprite_slot_view(k).ai_state(), 10);
    }

    pub(super) fn sprite_wish_pond2(&mut self, k: usize) {
        self.wish_pond2_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.game_state.world.location.dungeon_room() as u8 != 21 {
            self.sprite_wish_pond3(k);
        } else {
            self.sprite_happiness_pond(k);
        }
    }

    pub(super) fn sprite_wish_pond3(&mut self, k: usize) {
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.follower_link_state_mut().clear_immobilized();
                if self.sprite_slot_view(k).delay_main() != 0 || self.sprite_check_if_link_is_busy()
                {
                    return;
                }
                if self.sprite_show_message_on_contact(k, 0x14a) & 0x100 != 0 {
                    self.sprite_slot_view_mut(k).set_ai_state(1);
                    self.link_reset_properties_a();
                    self.follower_link_state_mut().set_facing(0);
                    self.sprite_slot_view_mut(k).set_head_direction(0);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 {
                    self.sprite_show_message_unconditional(0x8a);
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                    self.follower_link_state_mut().immobilize();
                } else {
                    self.sprite_show_message_unconditional(0x14b);
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                    self.sprite_slot_view_mut(k).set_delay_main(255);
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).set_ai_state(3);
                let j = self.multiselect_choice().value() as usize;
                self.sprite_slot_view_mut(k).set_c(j as u8);
                // C reads/clears raw ram[LINK_ITEM_BOW + j]. j (multiselect choice) can be 28-32,
                // beyond the 28-slot native item model, where inventory_item(j)/set_inventory_item(j)
                // silently return 0 / no-op. Use raw RAM for the read and the >=28 clear (the
                // wish-pond grant `t` is derived from this item value, so it diverged).
                let item = self.ram[crate::game_state::constants::LINK_ITEM_BOW + j];
                // Route the clear to the right native owner so its projection doesn't undo it:
                // bombs (idx 3) -> PlayerResources; items 0..28 -> item_slots; bottles 28..32
                // (LINK_BOTTLE_INFO = LINK_ITEM_BOW+28) -> bottles model; beyond -> raw RAM.
                let value = 0u8;
                if j == 3 {
                    self.player_resources_mut().set_bombs(value);
                } else if j < 28 {
                    self.inventory_items_mut().set_inventory_item(j, value);
                } else if j < 32 {
                    self.inventory_items_mut().set_bottle(j - 28, value);
                } else {
                    self.ram[crate::game_state::constants::LINK_ITEM_BOW + j] = value;
                }
                let item_idx = if j == 3 || j == 32 { 1 } else { item };
                let data_idx = WISH_POND_ITEM_DATA_OFFSETS[j]
                    .wrapping_add(item_idx)
                    .wrapping_sub(1) as usize;
                let t = WISH_POND_ITEM_DATA[data_idx];
                self.ancilla_add_tossed_pond_item(0x28, t, 4);
                self.hud_refresh_icon();
                self.sprite_slot_view_mut(k).set_graphics(t);
                self.sprite_slot_view_mut(k).set_direction(item);
                self.sprite_slot_view_mut(k).set_delay_main(255);
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0x72, &mut info);
                    if j >= 0 {
                        let j = j as usize;
                        self.sprite_set_x(j, info.r0_x);
                        self.sprite_set_y(j, info.r2_y.wrapping_sub(80));
                        self.set_music_control(0x1b);
                        self.set_last_music_control(0);
                        self.sprite_slot_view_mut(j).set_b(1);
                        self.Palette_AssertTranslucencySwap();
                        self.PaletteFilter_WishPonds();
                        self.sprite_slot_view_mut(k).set_e(j as u8);
                        self.sprite_slot_view_mut(k).set_ai_state(4);
                        self.sprite_slot_view_mut(k).set_delay_main(255);
                    }
                }
            }
            4 => {
                if self.game_state.frame.frame_counter & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.game_state.display.palette_filter.countdown() == 0 {
                        self.sprite_show_message_unconditional(0x8b);
                        self.Palette_RevertTranslucencySwap();
                        self.set_sub_screen_layers(0);
                        self.set_color_math_control(0x20);
                        self.increment_cgram_update_flag();
                        self.sprite_slot_view_mut(k).set_ai_state(5);
                    }
                }
            }
            5 => {
                let ai_state = if self.multiselect_choice().value() == 0 {
                    6
                } else {
                    11
                };
                self.sprite_slot_view_mut(k).set_ai_state(ai_state);
            }
            6 => {
                self.sprite_slot_view_mut(k).set_ai_state(7);
                if self.game_state.inventory.save_progress.dark_world_state() == 0 {
                    match self.sprite_slot_view(k).graphics() {
                        12 => {
                            self.sprite_slot_view_mut(k).set_graphics(42);
                            self.sprite_slot_view_mut(k).set_head_direction(1);
                        }
                        4 => {
                            self.sprite_slot_view_mut(k).set_graphics(5);
                            self.sprite_slot_view_mut(k).set_head_direction(2);
                        }
                        22 => {
                            self.sprite_slot_view_mut(k).set_graphics(44);
                            self.sprite_slot_view_mut(k).set_head_direction(3);
                        }
                        _ => {
                            self.sprite_show_message_unconditional(0x14d);
                            return;
                        }
                    }
                } else {
                    match self.sprite_slot_view(k).graphics() {
                        58 => {
                            self.sprite_slot_view_mut(k).set_graphics(59);
                            self.sprite_slot_view_mut(k).set_head_direction(4);
                            self.sprite_show_message_unconditional(0x14f);
                            return;
                        }
                        2 => {
                            self.sprite_slot_view_mut(k).set_graphics(3);
                            self.sprite_slot_view_mut(k).set_head_direction(5);
                        }
                        22 => {
                            self.sprite_slot_view_mut(k).set_graphics(44);
                            self.sprite_slot_view_mut(k).set_head_direction(3);
                        }
                        _ => {
                            self.sprite_show_message_unconditional(0x14d);
                            return;
                        }
                    }
                }
                self.sprite_show_message_unconditional(0x8c);
            }
            7 => {
                if self.sprite_slot_view(k).c() == 3 {
                    let value = self.sprite_slot_view(k).direction();
                    self.player_resources_mut().set_bombs(value);
                }
                self.Palette_AssertTranslucencySwap();
                self.set_sub_screen_layers(2);
                self.set_color_math_control(0x30);
                self.increment_cgram_update_flag();
                self.sprite_slot_view_mut(k).set_ai_state(8);
            }
            8 => {
                if self.game_state.frame.frame_counter & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.game_state.display.palette_filter.countdown() == 30 {
                        let j = self.sprite_slot_view(k).e() as usize;
                        self.sprite_slot_view_mut(j).set_state(0);
                    } else if self.game_state.display.palette_filter.countdown() == 0 {
                        self.sprite_slot_view_mut(k).set_ai_state(9);
                    }
                }
            }
            9 => {
                self.PaletteFilter_RestoreSP5F();
                self.Palette_RevertTranslucencySwap();
                self.follower_link_state_mut().set_item_receipt_method(2);
                // The ROM advances the pond's AI state BEFORE calling
                // Link_ReceiveItem: the oracle already reads $0A while the
                // item graphics decompression is suspended (route host
                // 182366); the C port's statement order is a reordering.
                self.sprite_slot_view_mut(k).set_ai_state(10);
                let item = self.sprite_slot_view(k).graphics();
                if self
                    .link_receive_item_from(
                        item,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::HappinessPondReward,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_happiness_pond_item_receipt(k);
            }
            10 => {
                let head = self.sprite_slot_view(k).head_direction();
                if head != 0 {
                    self.sprite_show_message_unconditional(
                        HAPPINESS_POND_REWARD_MESSAGES[head.wrapping_sub(1) as usize],
                    );
                }
                self.sprite_slot_view_mut(k).set_ai_state(0);
                self.sprite_slot_view_mut(k).set_delay_main(255);
            }
            11 => {
                self.sprite_show_message_unconditional(0x8d);
                self.sprite_slot_view_mut(k).set_ai_state(12);
            }
            12 => {
                let ai_state = if self.multiselect_choice().value() == 0 {
                    13
                } else {
                    6
                };
                self.sprite_slot_view_mut(k).set_ai_state(ai_state);
            }
            13 => {
                self.sprite_show_message_unconditional(0x8e);
                self.sprite_slot_view_mut(k).set_ai_state(7);
            }
            _ => {}
        }
    }

    pub(super) fn sprite_happiness_pond(&mut self, k: usize) {
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.follower_link_state_mut().clear_immobilized();
                if self.sprite_slot_view(k).delay_main() != 0 || self.sprite_check_if_link_is_busy()
                {
                    return;
                }
                if self.sprite_show_message_on_contact(k, 0x89) & 0x100 != 0 {
                    self.sprite_slot_view_mut(k).set_ai_state(1);
                    self.link_reset_properties_a();
                    self.ancilla_terminate_sparkle_objects();
                    self.follower_link_state_mut().set_facing(0);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 {
                    let i = u8::from(
                        self.game_state
                            .inventory
                            .player_resources
                            .has_bomb_or_arrow_upgrade(),
                    );
                    self.sprite_slot_view_mut(k).set_graphics(i * 2);
                    let cost_index = (i * 2) as usize;
                    self.dialogue_number_mut().set_packed_digits(
                        SPRITE_HAPPINESS_POND_COST_HEX_VALUES[cost_index],
                        SPRITE_HAPPINESS_POND_COST_HEX_VALUES[cost_index + 1],
                    );
                    self.sprite_show_message_unconditional(0x14e);
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                    self.follower_link_state_mut().immobilize();
                } else {
                    self.happiness_pond_show_later(k);
                }
            }
            2 => {
                let i = self
                    .sprite_slot_view(k)
                    .graphics()
                    .wrapping_add(self.multiselect_choice().value());
                self.dialogue_number_mut()
                    .set_high_pair(SPRITE_HAPPINESS_POND_COST_HEX_VALUES[i as usize]);
                if self.game_state.inventory.player_resources.rupees_goal()
                    < SPRITE_HAPPINESS_POND_COSTS[i as usize] as u16
                {
                    self.happiness_pond_show_later(k);
                } else {
                    self.sprite_slot_view_mut(k)
                        .set_direction(SPRITE_HAPPINESS_POND_COSTS[i as usize]);
                    self.sprite_slot_view_mut(k).set_head_direction(i);
                    self.sprite_slot_view_mut(k).set_ai_state(3);
                }
            }
            3 => {
                self.sprite_slot_view_mut(k).set_delay_main(80);
                let i = self.sprite_slot_view(k).direction();
                let rupees = self
                    .game_state
                    .inventory
                    .player_resources
                    .rupees_goal()
                    .wrapping_sub(i as u16);
                self.player_resources_mut().set_rupees_goal(rupees);
                let pond = self.player_resources_mut().add_rupees_to_pond(i);
                self.add_happiness_pond_rupees(self.sprite_slot_view(k).head_direction());
                if pond >= 100 {
                    self.player_resources_mut().subtract_pond_reward_threshold();
                    self.sprite_slot_view_mut(k).set_ai_state(5);
                    return;
                }
                let pond = self.game_state.inventory.player_resources.rupees_in_pond();
                self.dialogue_number_mut()
                    .set_low_pair((pond / 10) * 16 + (pond % 10));
                self.sprite_slot_view_mut(k).set_ai_state(4);
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_show_message_unconditional(0x94);
                    self.sprite_slot_view_mut(k).set_ai_state(13);
                }
            }
            5 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0x72, &mut info);
                    assert!(
                        j >= 0,
                        "Sprite_HappinessPond expected Sprite_SpawnDynamically to succeed"
                    );
                    let j = j as usize;
                    self.sprite_set_x(j, info.r0_x);
                    self.sprite_set_y(j, info.r2_y.wrapping_sub(80));
                    self.set_music_control(0x1b);
                    self.set_last_music_control(0);
                    self.sprite_slot_view_mut(j).set_b(1);
                    self.Palette_AssertTranslucencySwap();
                    self.PaletteFilter_WishPonds();
                    self.sprite_slot_view_mut(k).set_e(j as u8);
                    self.sprite_slot_view_mut(k).set_ai_state(6);
                    self.sprite_slot_view_mut(k).set_delay_main(255);
                }
            }
            6 => {
                if self.game_state.frame.frame_counter & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.game_state.display.palette_filter.countdown() == 0 {
                        self.sprite_show_message_unconditional(0x95);
                        self.Palette_RevertTranslucencySwap();
                        self.set_sub_screen_layers(0);
                        self.set_color_math_control(0x20);
                        self.increment_cgram_update_flag();
                        self.sprite_slot_view_mut(k).set_ai_state(7);
                    }
                }
            }
            7 => {
                let ai_state = if self.multiselect_choice().value() == 0 {
                    8
                } else {
                    12
                };
                self.sprite_slot_view_mut(k).set_ai_state(ai_state);
            }
            8 => {
                let i = self
                    .game_state
                    .inventory
                    .player_resources
                    .next_bomb_upgrade_level();
                if i != 8 {
                    let filler = HAPPINESS_POND_MAX_BOMBS_HEX[i as usize];
                    {
                        let mut resources = self.player_resources_mut();
                        resources.set_bomb_upgrade_level(i);
                        resources.set_bomb_filler(filler);
                    }
                    self.dialogue_number_mut().set_low_pair(filler);
                    self.sprite_show_message_unconditional(0x96);
                } else {
                    let rupees = self
                        .game_state
                        .inventory
                        .player_resources
                        .rupees_goal()
                        .wrapping_add(100);
                    self.player_resources_mut().set_rupees_goal(rupees);
                    self.sprite_show_message_unconditional(0x98);
                }
                self.sprite_slot_view_mut(k).set_ai_state(9);
            }
            9 => {
                self.Palette_AssertTranslucencySwap();
                self.set_sub_screen_layers(2);
                self.set_color_math_control(0x30);
                self.increment_cgram_update_flag();
                self.sprite_slot_view_mut(k).set_ai_state(10);
            }
            10 => {
                if self.game_state.frame.frame_counter & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.game_state.display.palette_filter.countdown() == 30 {
                        let j = self.sprite_slot_view(k).e() as usize;
                        self.sprite_slot_view_mut(j).set_state(0);
                    } else if self.game_state.display.palette_filter.countdown() == 0 {
                        self.sprite_slot_view_mut(k).set_ai_state(11);
                    }
                }
            }
            11 => {
                self.PaletteFilter_RestoreSP5F();
                self.Palette_RevertTranslucencySwap();
                self.sprite_slot_view_mut(k).set_ai_state(0);
                self.sprite_slot_view_mut(k).set_delay_main(255);
            }
            12 => {
                let i = self
                    .game_state
                    .inventory
                    .player_resources
                    .next_arrow_upgrade_level();
                if i != 8 {
                    let filler = HAPPINESS_POND_ARROW_REFILL_AMOUNTS[i as usize];
                    {
                        let mut resources = self.player_resources_mut();
                        resources.set_arrow_upgrade_level(i);
                        resources.set_arrow_filler(filler);
                    }
                    self.dialogue_number_mut().set_low_pair(filler);
                    self.sprite_show_message_unconditional(0x97);
                } else {
                    let rupees = self
                        .game_state
                        .inventory
                        .player_resources
                        .rupees_goal()
                        .wrapping_add(100);
                    self.player_resources_mut().set_rupees_goal(rupees);
                    self.sprite_show_message_unconditional(0x98);
                }
                self.sprite_slot_view_mut(k).set_ai_state(9);
            }
            13 => {
                self.sprite_show_message_unconditional(0x154);
                self.sprite_slot_view_mut(k).set_ai_state(14);
            }
            14 => {
                let i = (self.get_random_number() & 3) as usize;
                self.sprite_battle_mut()
                    .set_item_drop_luck(HAPPINESS_POND_LUCK_VALUES[i]);
                self.sprite_battle_mut().clear_luck_kill_counter();
                self.sprite_show_message_unconditional(HAPPINESS_POND_LUCK_MESSAGES[i]);
                self.sprite_slot_view_mut(k).set_ai_state(0);
                self.sprite_slot_view_mut(k).set_delay_main(255);
            }
            _ => {}
        }
    }

    fn happiness_pond_show_later(&mut self, k: usize) {
        self.sprite_show_message_unconditional(0x14c);
        self.sprite_slot_view_mut(k).set_ai_state(0);
        self.sprite_slot_view_mut(k).set_delay_main(255);
    }

    pub(super) fn wish_pond2_draw(&mut self, k: usize) {
        if self.game_state.world.location.dungeon_room() as u8 == 21 {
            return;
        }
        let t = self.sprite_slot_view(k).ai_state();
        if !matches!(t, 5 | 6 | 11 | 12) {
            return;
        }
        let g = self.sprite_slot_view(k).graphics() as usize;
        let mut f = WISH_POND_ITEM_OAM_FLAGS[g];
        if f == 0xff {
            f = 5;
        }
        self.sprite_slot_view_mut(k).set_oam_flags((f & 7) * 2);
        let start = ((RECEIVE_ITEM_PREP_DRAW_FRAME_START_BYTES[g] >> 1) * 4) as usize;
        self.sprite_draw_multiple(
            k,
            &WISH_POND2_DRAW_WISH_POND_ITEM_DRAW_FRAMES[start..start + 4],
            None,
        );
    }

    fn sprite_get_16bit_coords_for_prep(&mut self, k: usize) {
        let x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        self.sprite_workspace_mut().set_current_sprite_x(x);
        self.sprite_workspace_mut().set_current_sprite_y(y);
    }

    pub(super) fn pink_ball_handle_deceleration(&mut self, k: usize) {
        if self.sprite_slot_view(k).x_velocity() != 0 {
            let x_velocity = self.sprite_slot_view(k).x_velocity();
            let delta = if sign8(x_velocity) {
                2
            } else {
                0u8.wrapping_sub(2)
            };
            self.sprite_slot_view_mut(k).add_x_velocity(delta);
        }
        if self.sprite_slot_view(k).y_velocity() != 0 {
            let y_velocity = self.sprite_slot_view(k).y_velocity();
            let delta = if sign8(y_velocity) {
                2
            } else {
                0u8.wrapping_sub(2)
            };
            self.sprite_slot_view_mut(k).add_y_velocity(delta);
        }
    }

    pub(super) fn pink_ball_distress(&mut self, k: usize) {
        let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        self.sprite_draw_distress_custom(x, y, self.game_state.frame.frame_counter);
    }

    pub(super) fn spawn_apple(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xac, &mut info);
        if j < 0 {
            return;
        }

        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        self.sprite_slot_view_mut(j).set_ai_state(1);
        self.sprite_slot_view_mut(j).set_a(255);
        self.sprite_slot_view_mut(j).set_z(8);
        self.sprite_slot_view_mut(j).set_z_velocity(22);
        let x = (info.r0_x & !0xff).wrapping_add(u16::from(self.get_random_number()));
        let y = (info.r2_y & !0xff).wrapping_add(u16::from(self.get_random_number()));
        let pt = self.sprite_project_speed_towards_location(k, x, y, 10);
        self.sprite_slot_view_mut(j).set_x_velocity(pt.x);
        self.sprite_slot_view_mut(j).set_y_velocity(pt.y);
    }

    pub(super) fn sprite_transmute_to_bomb(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_sprite_type(0x4a);
        self.sprite_slot_view_mut(k).set_c(1);
        self.sprite_slot_view_mut(k).set_delay_aux1(255);
        self.sprite_slot_view_mut(k).set_flags3(0x18);
        self.sprite_slot_view_mut(k).set_oam_flags(8);
        self.sprite_slot_view_mut(k).set_health(0);
    }

    pub(super) fn beamos_fire_laser(&mut self, k: usize) {
        if self.game_state.sprites.system.limit_instance() >= 4 {
            return;
        }

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x61, &mut info);
        if j < 0 {
            return;
        }

        let j = j as usize;
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
        self.sprite_set_x(
            j,
            info.r0_x.wrapping_add_signed(i16::from(
                self.game_state.sprites.draw_hitbox_work.x_low() as i8,
            )),
        );
        self.sprite_set_y(
            j,
            info.r2_y.wrapping_add_signed(i16::from(
                self.game_state.sprites.draw_hitbox_work.y_low() as i8,
            )),
        );
        self.sprite_apply_speed_towards_link(j, 0x20);
        self.sprite_slot_view_mut(j).set_flags2(0x3f);
        self.sprite_slot_view_mut(j).set_flags4(0x54);
        self.sprite_slot_view_mut(j).set_c(1);
        self.sprite_slot_view_mut(j).set_deflection_bits(0x48);
        self.sprite_slot_view_mut(j).set_oam_flags(3);
        self.sprite_slot_view_mut(j).set_bump_damage(4);
        self.sprite_slot_view_mut(j).set_delay_aux1(12);
        let t = self.game_state.sprites.system.limit_instance() as usize;
        self.sprite_slot_view_mut(j).set_graphics(t as u8);
        self.sprite_system_mut().increment_limit_instance();

        let x = self.sprite_slot_view(j).x();
        let y = self.sprite_slot_view(j).y();
        for i in 0..32 {
            let o = t * 32 + i;
            self.beamos_laser_history_mut(o).set_position(x, y);
        }
    }

    pub(super) fn octoballoon_find(&mut self) -> bool {
        (0..16).rev().any(|i| {
            self.sprite_slot_view(i).state() != 0 && self.sprite_slot_view(i).sprite_type() == 0x10
        })
    }

    pub(super) fn potion_cauldron_go_beep(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x3c);
    }

    pub(super) fn potion_cauldron_check_bottles(&mut self) -> bool {
        (self.game_state.inventory.items.bottle(0)
            | self.game_state.inventory.items.bottle(1)
            | self.game_state.inventory.items.bottle(2)
            | self.game_state.inventory.items.bottle(3))
            >= 2
    }

    pub(super) fn dark_world_hint_npc_handle_payment(&mut self) -> bool {
        let rupees_goal = self.game_state.inventory.player_resources.rupees_goal();
        if rupees_goal < 20 {
            return false;
        }
        self.player_resources_mut().subtract_rupees_goal(20);
        true
    }

    pub(super) fn dark_world_hint_npc_idle(&mut self, k: usize) {
        if self.sprite_show_solicited_message(k, 0xfe) & 0x100 != 0 {
            self.sprite_slot_view_mut(k).set_ai_state(1);
        }
    }

    pub(super) fn fairy_check_if_touchable(&mut self, k: usize) {
        let msg = self.game_state.messaging.dialogue_message_index.value();
        if self.game_state.frame.submodule == 2 && (msg == 0xc9 || msg == 0xca) {
            self.sprite_slot_view_mut(k).set_delay_aux4(40);
        }
    }

    pub(super) fn buzzblob_select_new_direction(&mut self, k: usize) {
        let j = (self.get_random_number() & 7) as usize;
        self.sprite_slot_view_mut(k)
            .set_x_velocity(BUZZBLOB_SELECT_NEW_DIRECTION_X_VELOCITIES[j] as u8);
        self.sprite_slot_view_mut(k)
            .set_y_velocity(BUZZBLOB_SELECT_NEW_DIRECTION_Y_VELOCITIES[j] as u8);
        self.sprite_slot_view_mut(k)
            .set_delay_main(BUZZBLOB_SELECT_NEW_DIRECTION_DELAYS[j]);
    }

    pub(super) fn lumberjack_check_proximity(&mut self, _k: usize, j: usize) -> bool {
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        cur_x
            .wrapping_sub(link_x)
            .wrapping_add(LUMBERJACK_CHECK_PROXIMITY_X_OFFSETS[j])
            < LUMBERJACK_CHECK_PROXIMITY_WIDTHS[j]
            && cur_y
                .wrapping_sub(link_y)
                .wrapping_add(LUMBERJACK_CHECK_PROXIMITY_Y_OFFSETS[j])
                < LUMBERJACK_CHECK_PROXIMITY_HEIGHTS[j]
    }

    pub(super) fn blind_laser_spawn_trail_garnish(&mut self, j: usize) {
        let k = self.garnish_alloc_overwrite_old() as usize;
        let value = 15;
        self.garnish_slot_view_mut(k).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(15);
        let value = self.sprite_slot_view(j).graphics();
        self.garnish_slot_view_mut(k).set_oam_flags(value);
        let value = j as u8;
        self.garnish_slot_view_mut(k).set_sprite(value);
        let value = self.sprite_slot_view(j).x_low();
        self.garnish_slot_view_mut(k).set_x_low(value);
        let value = self.sprite_slot_view(j).x_high();
        self.garnish_slot_view_mut(k).set_x_high(value);
        self.garnish_set_y(k, self.sprite_get_y(j).wrapping_add(16));
        let value = 10;
        self.garnish_slot_view_mut(k).set_countdown(value);
    }

    pub(super) fn running_boy_spawn_dust_garnish(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_die_action();
        if self.sprite_slot_view(k).die_action() & 0x0f != 0 {
            return;
        }
        let j = self.garnish_alloc_force() as usize;
        let value = 20;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(20);
        self.garnish_set_x(j, self.sprite_get_x(k).wrapping_add(4));
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(28));
        let value = 10;
        self.garnish_slot_view_mut(j).set_countdown(value);
    }

    pub(super) fn sprite_cd_spawn_garnish(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_subtype2();
        if self.sprite_slot_view(k).subtype2() & 7 != 0 {
            return;
        }
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x14);
        let j = self.garnish_alloc_overwrite_old() as usize;
        let value = 0x0c;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(0x0c);
        let value = k as u8;
        self.garnish_slot_view_mut(j).set_sprite(value);
        self.garnish_set_x(j, self.sprite_get_x(k));
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(16));
        let value = 127;
        self.garnish_slot_view_mut(j).set_countdown(value);
    }

    pub(super) fn dark_world_hint_npc_restore_health(&mut self, k: usize) {
        self.player_resources_mut().set_heart_filler(0xa0);
        self.sprite_slot_view_mut(k).set_ai_state(0);
    }

    pub(super) fn pipe_validate_entry(&mut self) -> bool {
        for k in (0..=4).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 0x31 {
                self.follower_link_state_mut().clear_position_mode();
                self.follower_link_state_mut().clear_direction_lock();
                self.ancilla_slot_view_mut(k).clear();
                break;
            }
        }
        self.game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            || self.game_state.player.follower_link.has_auxiliary_state()
    }

    pub(super) fn sprite_prep_zoro(&mut self, k: usize) {
        let direction = self.sprite_slot_view(k).sprite_type().wrapping_sub(0x9c) << 1;
        self.sprite_slot_view_mut(k).set_direction(direction);
        self.sprite_slot_view_mut(k).decrement_graphics();
    }

    pub(super) fn sprite_prep_popo(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_b(7);
    }

    pub(super) fn sprite_prep_popo2(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_b(15);
    }

    pub(super) fn sprite_prep_statue(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_y_low(7);
    }

    pub(super) fn sprite_prep_bari(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(6);
        if self.game_state.dungeon.room_tracking.room_index2() == 206 {
            self.sprite_slot_view_mut(k).decrement_c();
        }
        // ROM $86:8B1C: JSL GetRandomNumber / AND #$3F / ADC #$80 keeps the
        // carry GetRandomNumber leaves set (route hosts 357284 and 583864:
        // Blue/Red Bari aux1 0x90 vs a carry-dropped 0x8f).
        let delay_aux1 = self.get_random_number_with_carry().masked_adc(63, 128);
        self.sprite_slot_view_mut(k).set_delay_aux1(delay_aux1);
    }

    pub(super) fn sprite_prep_green_stalfos(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(9);
    }

    pub(super) fn sprite_prep_water_lever(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_y_low(5);
    }

    pub(super) fn sprite_prep_fire_debirando(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_sprite_type(0x63);
        self.sprite_prep_load_properties(k);
        self.sprite_slot_view_mut(k).decrement_g();
        self.sprite_prep_debirando_pit(k);
    }

    pub(super) fn sprite_prep_debirando_pit(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_g();
        self.sprite_slot_view_mut(k).set_delay_main(0);
        self.sprite_slot_view_mut(k).set_graphics(6);
        self.sprite_prep_ignore_projectiles(k);

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x64, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_view_mut(j).set_delay_main(96);
            self.sprite_slot_view_mut(k).set_head_direction(j as u8);
            let g = self.sprite_slot_view(k).g();
            self.sprite_slot_view_mut(j).set_g(g);
            self.sprite_slot_view_mut(j)
                .set_oam_flags(SPRITE_PREP_DEBIRANDO_PIT_DEBIRANDO_OAM_FLAGS[g as usize]);
        }
    }

    pub(super) fn sprite_prep_weak_guard(&mut self, k: usize) {
        let dir = self.get_random_number() & 3;
        self.sprite_slot_view_mut(k).set_direction(dir);
        self.sprite_slot_view_mut(k).set_head_direction(dir);
        self.sprite_slot_view_mut(k).set_delay_main(16);
    }

    pub(super) fn sprite_prep_laser_eye_bounce(&mut self, k: usize) {
        let t = self.sprite_slot_view(k).sprite_type();
        self.sprite_slot_view_mut(k)
            .set_direction(t.wrapping_sub(0x95));
        if t >= 0x97 {
            self.sprite_slot_view_mut(k).add_x_low(8);
            let head_direction = (self.sprite_slot_view(k).x_low() & 16) ^ 16;
            self.sprite_slot_view_mut(k)
                .set_head_direction(head_direction);
            if self.sprite_slot_view(k).head_direction() == 0 {
                let y_low = self
                    .sprite_slot_view(k)
                    .y_low()
                    .wrapping_add(if (t & 1) != 0 { (-8i8) as u8 } else { 8 });
                self.sprite_slot_view_mut(k).set_y_low(y_low);
            }
        } else {
            let head_direction = self.sprite_slot_view(k).y_low() & 16;
            self.sprite_slot_view_mut(k)
                .set_head_direction(head_direction);
            if self.sprite_slot_view(k).head_direction() == 0 {
                let x_low = self
                    .sprite_slot_view(k)
                    .x_low()
                    .wrapping_add(if (t & 1) != 0 { (-8i8) as u8 } else { 8 });
                self.sprite_slot_view_mut(k).set_x_low(x_low);
            }
        }
    }

    pub(super) fn sprite_prep_wall_cannon(&mut self, k: usize) {
        let direction = self.sprite_slot_view(k).sprite_type().wrapping_sub(0x66);
        self.sprite_slot_view_mut(k).set_direction(direction);
        self.sprite_slot_view_mut(k).set_a(direction & 2);
    }

    pub(super) fn sprite_prep_purple_chest(&mut self, k: usize) {
        if self.game_state.sprites.follower_runtime.indicator() != 12
            && self
                .game_state
                .inventory
                .save_progress
                .progress_indicator_3()
                & 16
                == 0
            && self
                .game_state
                .inventory
                .save_progress
                .progress_indicator_3()
                & 32
                != 0
        {
            self.sprite_slot_view_mut(k).increment_ignore_projectile();
        } else {
            self.sprite_slot_view_mut(k).set_state(0);
        }
    }

    pub(super) fn sprite_prep_smithy(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        if self.game_state.inventory.save_progress.dark_world_state() & 64 != 0 {
            if self
                .game_state
                .inventory
                .save_progress
                .progress_indicator_3()
                & 32
                != 0
                || self.game_state.sprites.follower_runtime.indicator() != 0
            {
                self.sprite_slot_view_mut(k).set_state(0);
            } else {
                self.sprite_slot_view_mut(k).set_subtype2(2);
            }
            return;
        }

        self.sprite_prep_smithy_spawn_dumb_barrier_sprite(k);
        self.sprite_slot_view_mut(k).add_x_low(2);
        self.sprite_slot_view_mut(k).subtract_y_low(3);
        if self
            .game_state
            .inventory
            .save_progress
            .progress_indicator_3()
            & 32
            == 0
        {
            return;
        }

        let j = self.sprite_prep_smithy_spawn_dwarf_pal(k);
        let j = j as usize;
        self.sprite_prep_smithy_spawn_dumb_barrier_sprite(j);
        self.sprite_slot_view_mut(j).set_e(k as u8);
        self.sprite_slot_view_mut(k).set_e(j as u8);

        if self
            .game_state
            .inventory
            .save_progress
            .progress_indicator_3()
            & 0x80
            != 0
        {
            self.sprite_slot_view_mut(k).set_ai_state(5);
            self.sprite_slot_view_mut(j).set_ai_state(5);
        }
    }

    fn sprite_prep_smithy_spawn_dwarf_pal(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x1a, &mut info);
        if j < 0 {
            return j;
        }
        let j = j as usize;
        self.sprite_set_x(j, info.r0_x);
        self.sprite_set_y(j, info.r2_y);
        self.sprite_slot_view_mut(j).add_x_low(0x2c);
        self.sprite_slot_view_mut(j).set_direction(1);
        self.sprite_slot_view_mut(j).set_a(4);
        self.sprite_slot_view_mut(j).set_ignore_projectile(4);
        j as i32
    }

    fn sprite_prep_smithy_spawn_dumb_barrier_sprite(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x31, &mut info);
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.sprite_set_x(j, info.r0_x);
        self.sprite_set_y(j, info.r2_y);
        self.sprite_slot_view_mut(j).set_subtype2(1);
        self.sprite_slot_view_mut(j).set_flags4(0);
        self.sprite_slot_view_mut(j).set_ignore_projectile(1);
    }

    pub(super) fn sprite_prep_ignore_projectiles(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_haunted_grove_animal(&mut self, k: usize) {
        let direction = self.sprite_is_right_of_link(k).a;
        self.sprite_slot_view_mut(k).set_direction(direction);
        self.sprite_prep_haunted_grove_ostritch(k);
    }

    pub(super) fn sprite_prep_haunted_grove_ostritch(&mut self, k: usize) {
        if self.game_state.inventory.items.flute() >= 2 {
            self.sprite_slot_view_mut(k).set_state(0);
        }
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_whirlpool(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        self.sprite_slot_view_mut(k).set_a(1);
    }

    pub(super) fn sprite_prep_bonk_item(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors() {
            self.sprite_slot_view_mut(k).set_graphics(2);
            return;
        }

        self.sprite_slot_view_mut(k).set_floor(2);
        if self.game_state.world.location.dungeon_room() == 0x0107 {
            if self.game_state.inventory.items.book() != 0 {
                self.sprite_slot_view_mut(k).set_state(0);
            } else {
                self.DecodeAnimatedSpriteTile_variable(0x0e);
            }
        } else {
            let j = self.game_state.sprite_battle.item_drop_counter();
            self.sprite_battle_mut().increment_item_drop_counter();
            self.sprite_slot_view_mut(k).set_die_action(j);
            if self.game_state.dungeon.savegame_state.savegame_state_bits()
                & SPRITE_PREP_BONK_ITEM_DASH_ITEM_MASK[j as usize]
                != 0
            {
                self.sprite_slot_view_mut(k).set_state(0);
            }
            self.sprite_slot_view_mut(k).increment_graphics();
            self.sprite_slot_view_mut(k).set_oam_flags(8);
            self.sprite_slot_view_mut(k).or_flags3(0x20);
        }
    }

    pub(super) fn sprite_prep_digging_game_guy_bounce(&mut self, k: usize) {
        if self.game_state.player.follower_link.y() < self.sprite_get_y(k) {
            self.sprite_slot_view_mut(k).set_ai_state(5);
            self.sprite_slot_view_mut(k).subtract_x_low(9);
            self.sprite_slot_view_mut(k).set_graphics(1);
        }
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    // void Sprite_D5_DigGameGuy(int k) {  // 9dfc38
    //   DiggingGameGuy_Draw(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   Sprite_BehaveAsBarrier(k);
    //   Sprite_MoveXY(k);
    //   sprite_x_vel[k] = 0;
    //   switch(sprite_ai_state[k]) {
    //   case 0:  // intro
    //     if ((uint8)(sprite_y_lo[k] + 7) < BYTE(link_y_coord) && Sprite_DirectionToFaceLink(k, NULL) == 2) {
    //       if (follower_indicator == 0) {
    //         if (Sprite_ShowSolicitedMessage(k, 0x187) & 0x100)
    //           sprite_ai_state[k]++;
    //       } else {
    //         Sprite_ShowSolicitedMessage(k, 0x18c);
    //       }
    //     }
    //     break;
    //   case 1:  // do you want to play
    //     if (choice_in_multiselect_box == 0 && link_rupees_goal >= 80) {
    //       link_rupees_goal -= 80;
    //       Sprite_ShowMessageUnconditional(0x188);
    //       sprite_ai_state[k] = 2;
    //       sprite_graphics[k] = 1;
    //       sprite_delay_main[k] = 80;
    //       beamos_x_hi[0] = 0;
    //       beamos_x_hi[1] = 0;
    //       sprite_delay_aux1[k] = 5;
    //       Sprite_InitializeSecondaryItemMinigame(1);
    //       music_control = 14;
    //     } else {
    //       Sprite_ShowMessageUnconditional(0x189);
    //       sprite_ai_state[k] = 0;
    //     }
    //     break;
    //   case 2:  // move out of the way
    //     if (!sprite_delay_main[k]) {
    //       sprite_ai_state[k]++;
    //       sprite_graphics[k] = 1;
    //     } else if (!sprite_delay_aux1[k]) {
    //       sprite_graphics[k] ^= 3;
    //       if (sprite_graphics[k] & 1)
    //         sprite_x_vel[k] = -16;
    //       sprite_delay_aux1[k] = 5;
    //     }
    //     break;
    //   case 3:  // start timer
    //     sprite_ai_state[k]++;
    //     super_bomb_indicator_unk1 = 0;
    //     super_bomb_indicator_unk2 = 30;
    //     break;
    //   case 4:  // terminate
    //     if ((int8)super_bomb_indicator_unk2 > 0 || link_position_mode & 1)
    //       return;
    //     music_control = 9;
    //     sprite_ai_state[k]++;
    //     is_archer_or_shovel_game = 0;
    //     dialogue_message_index = 0x18a;
    //     Sprite_ShowMessageMinimal();
    //     super_bomb_indicator_unk2 = 254;
    //     break;
    //   case 5:  // come back later
    //     Sprite_ShowSolicitedMessage(k, 0x18b);
    //     break;
    //   }
    // }
    pub(super) fn sprite_d5_dig_game_guy(&mut self, k: usize) {
        self.digging_game_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_move_xy(k);
        self.sprite_slot_view_mut(k).set_x_velocity(0);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).y_low().wrapping_add(7)
                    < self.game_state.player.follower_link.y() as u8
                    && self.sprite_direction_to_face_link(k, None) == 2
                {
                    if self.game_state.sprites.follower_runtime.indicator() == 0 {
                        if self.sprite_show_solicited_message(k, 0x187) & 0x100 != 0 {
                            self.sprite_slot_view_mut(k).increment_ai_state();
                        }
                    } else {
                        self.sprite_show_solicited_message(k, 0x18c);
                    }
                }
            }
            1 => {
                let rupees = self.game_state.inventory.player_resources.rupees_goal();
                if self.multiselect_choice().value() == 0 && rupees >= 80 {
                    self.player_resources_mut()
                        .set_rupees_goal(rupees.wrapping_sub(80));
                    self.sprite_show_message_unconditional(0x188);
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                    self.sprite_slot_view_mut(k).set_graphics(1);
                    self.sprite_slot_view_mut(k).set_delay_main(80);
                    self.digging_game_prize_mut().clear_prize_spawned();
                    self.lanmola_segment_motion_mut(1).set_z_offset(0);
                    self.sprite_slot_view_mut(k).set_delay_aux1(5);
                    self.sprite_initialize_secondary_item_minigame(1);
                    self.set_music_control(14);
                } else {
                    self.sprite_show_message_unconditional(0x189);
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).increment_ai_state();
                    self.sprite_slot_view_mut(k).set_graphics(1);
                } else if self.sprite_slot_view(k).delay_aux1() == 0 {
                    self.sprite_slot_view_mut(k).xor_graphics(3);
                    if self.sprite_slot_view(k).graphics() & 1 != 0 {
                        self.sprite_slot_view_mut(k).set_x_velocity((-16i8) as u8);
                    }
                    self.sprite_slot_view_mut(k).set_delay_aux1(5);
                }
            }
            3 => {
                self.sprite_slot_view_mut(k).increment_ai_state();
                self.set_super_bomb_indicator_counter(0);
                self.set_super_bomb_indicator_timer(30);
            }
            4 => {
                if (self.hud_state().super_bomb_indicator_timer() as i8) > 0
                    || self.game_state.player.follower_link.position_mode_has(1)
                {
                    return;
                }
                self.set_music_control(9);
                self.sprite_slot_view_mut(k).increment_ai_state();
                self.minigame_state_mut().clear_is_archer_or_shovel_game();
                self.dialogue_message_index_mut().set_value(0x18a);
                self.sprite_show_message_minimal_c();
                self.set_super_bomb_indicator_timer(254);
            }
            5 => {
                self.sprite_show_solicited_message(k, 0x18b);
            }
            _ => {}
        }
    }

    // void Sprite_InitializeSecondaryItemMinigame(int what) {  // 8ffd86
    //   is_archer_or_shovel_game = what;
    //   Link_ResetProperties_C();
    //   for (int k = 4; k >= 0; k--) {
    //     if (ancilla_type[k] == 0x30 || ancilla_type[k] == 0x31) {
    //       ancilla_type[k] = 0;
    //     } else if (ancilla_type[k] == 5) {
    //       flag_for_boomerang_in_place = 0;
    //       ancilla_type[k] = 0;
    //     }
    //   }
    // }
    pub(super) fn sprite_initialize_secondary_item_minigame(&mut self, what: u8) {
        self.minigame_state_mut().set_is_archer_or_shovel_game(what);
        self.link_reset_properties_c();
        for k in (0..=4).rev() {
            match self.ancilla_slot_view(k).ancilla_type() {
                0x30 | 0x31 => self.ancilla_slot_view_mut(k).clear(),
                5 => {
                    self.minigame_state_mut().clear_flag_boomerang_in_place();
                    self.ancilla_slot_view_mut(k).clear();
                }
                _ => {}
            }
        }
    }

    pub(super) fn sprite_prep_thieves_town_grate(&mut self, k: usize) {
        if self.game_state.world.overworld.event_info.event_info(0x58) & 0x20 != 0 {
            self.sprite_slot_view_mut(k).set_state(0);
        }
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(8));
    }

    pub(super) fn sprite_prep_rupee_pull(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(8));
    }

    pub(super) fn sprite_prep_shopkeeper(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        self.sprite_slot_view_mut(k).or_flags2(2);
        self.sprite_slot_view_mut(k).or_oam_flags(12);
        self.sprite_slot_view_mut(k).or_flags3(16);

        let room = self.game_state.world.location.dungeon_room_index();
        let j = SPRITE_PREP_SHOPKEEPER_SHOP_KEEPER_WHERE
            .iter()
            .position(|&candidate| candidate == room)
            .expect("SpritePrep_Shopkeeper room not in kShopKeeperWhere");
        match j {
            0 => {
                self.shop_keeper_spawn_shop_item(k, 0, 7);
                self.shop_keeper_spawn_shop_item(k, 1, 8);
                self.shop_keeper_spawn_shop_item(k, 2, 12);
            }
            1 => {
                self.shop_keeper_spawn_shop_item(k, 0, 9);
                self.shop_keeper_spawn_shop_item(k, 1, 13);
                self.shop_keeper_spawn_shop_item(k, 2, 11);
            }
            2 => {
                self.sprite_slot_view_mut(k).set_subtype2(4);
                self.minigame_state_mut().set_credits(0xff);
            }
            3 => {
                self.sprite_slot_view_mut(k).set_subtype2(1);
                self.sprite_slot_view_mut(k).set_graphics(1);
                self.minigame_state_mut().set_credits(0xff);
            }
            4 => {
                self.sprite_slot_view_mut(k).set_subtype2(3);
                self.minigame_state_mut().set_credits(0xff);
            }
            5 | 7 | 8 => {
                self.shop_keeper_spawn_shop_item(k, 0, 7);
                self.shop_keeper_spawn_shop_item(k, 1, 10);
                self.shop_keeper_spawn_shop_item(k, 2, 12);
            }
            6 | 9 | 12 => self.sprite_slot_view_mut(k).set_subtype2(2),
            10 => self.sprite_slot_view_mut(k).set_subtype2(5),
            11 => self.sprite_slot_view_mut(k).set_subtype2(6),
            _ => unreachable!(),
        }
    }

    pub(super) fn shop_keeper_spawn_shop_item(&mut self, k: usize, pos: usize, what: u8) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0xbb, &mut info, 12);
        assert!(j >= 0);
        let j = j as usize;
        self.sprite_slot_view_mut(j).set_ignore_projectile(what);
        self.sprite_slot_view_mut(j).set_subtype2(what);
        self.sprite_set_x(
            j,
            info.r0_x
                .wrapping_add(SHOP_KEEPER_SPAWN_SHOP_ITEM_SHOP_KEEPER_ITEM_X[pos] as u16),
        );
        self.sprite_set_y(j, info.r2_y.wrapping_add(0x27));
        self.sprite_slot_view_mut(j).or_flags2(4);
    }

    pub(super) fn shop_keeper_rapid_terminate_receive_item(&mut self) {
        for i in (0..=4).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x22 {
                let value = 1;
                self.ancilla_slot_view_mut(i).set_aux_timer(value);
            }
        }
    }

    pub(super) fn sprite_spawn_bat_crash_cutscene(&mut self) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0x37, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_slot_view_mut(j).set_y_velocity(0);
            self.sprite_slot_view_mut(j).set_b(0);
            self.sprite_slot_view_mut(j).set_direction(0);
            self.sprite_slot_view_mut(j).set_floor(0);
            self.sprite_slot_view_mut(j).set_subtype2(1);
            self.sprite_slot_view_mut(j).set_flags2(1);
            self.sprite_slot_view_mut(j).set_flags3(1);
            self.sprite_slot_view_mut(j).set_oam_flags(1);
            self.sprite_slot_view_mut(j).set_x_low(204);
            self.sprite_slot_view_mut(j).set_x_high(7);
            self.sprite_slot_view_mut(j).set_y_low(50);
            self.sprite_slot_view_mut(j).set_y_high(6);
            self.sprite_slot_view_mut(j).set_deflection_bits(128);
        }
    }

    pub(super) fn sprite_prep_storyteller(&mut self, k: usize) {
        let mut r = SPRITE_PREP_STORYTELLER_ROOMS
            .iter()
            .position(|&room| room == self.game_state.world.location.dungeon_room_index())
            .map_or(0xff, |idx| idx as u8);
        if r == 0 && self.sprite_slot_view(k).x_high() & 1 != 0 {
            r = 1;
        }
        self.sprite_slot_view_mut(k).set_subtype2(r);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_adults(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        let dungeon_room = self.game_state.world.location.dungeon_room_index();
        let subtype2 = SPRITE_PREP_ADULTS_HUMAN_MULTI_TYPES
            .iter()
            .position(|&room| room == dungeon_room)
            .map_or(0xff, |idx| idx as u8);
        self.sprite_slot_view_mut(k).set_subtype2(subtype2);
    }

    pub(super) fn sprite_prep_sage(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        if self.game_state.world.location.dungeon_room_index() == 10 {
            self.sprite_slot_view_mut(k).increment_subtype2();
            self.sprite_slot_view_mut(k).set_oam_flags(11);
        }
    }

    pub(super) fn sprite_prep_kiki(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        let screen = self.game_state.world.location.overworld_screen_index() as usize;
        if self
            .game_state
            .world
            .overworld
            .event_info
            .event_info(screen)
            & 0x20
            != 0
        {
            self.sprite_slot_view_mut(k).set_state(0);
        }
    }

    pub(super) fn sprite_prep_locksmith(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        if self.game_state.sprites.follower_runtime.indicator() == 9 {
            self.sprite_slot_view_mut(k).set_state(0);
            return;
        }
        if self.game_state.sprites.follower_runtime.indicator() == 12 {
            self.sprite_slot_view_mut(k).set_ai_state(2);
        }
        if self
            .game_state
            .inventory
            .save_progress
            .progress_indicator_3()
            & 0x10
            != 0
        {
            self.sprite_slot_view_mut(k).set_ai_state(4);
        }
    }

    pub(super) fn sprite_prep_sick_kid(&mut self, k: usize) {
        if self.game_state.inventory.items.bug_net() != 0 {
            self.sprite_slot_view_mut(k).set_ai_state(3);
        }
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_tektite(&mut self, k: usize) {
        let j = ((self.sprite_slot_view(k).x_low() >> 4) & 1) as usize;
        self.sprite_slot_view_mut(k).set_a(j as u8);
        self.sprite_slot_view_mut(k)
            .set_oam_flags(SPRITE_PREP_TEKTITE_OAM_FLAGS[j]);
        self.sprite_slot_view_mut(k)
            .set_health(SPRITE_PREP_TEKTITE_HEALTH_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_bump_damage(SPRITE_PREP_TEKTITE_BUMP_DAMAGE_VALUES[j]);
        self.sprite_apply_speed_towards_link(k, 16);
        self.sprite_slot_view_mut(k).set_z_velocity(32);
        self.sprite_slot_view_mut(k).increment_ai_state();
    }

    pub(super) fn sprite_prep_chainchomp_bounce(&mut self, k: usize) {
        let mut i = k * 8;
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        for _ in (0..=5).rev() {
            self.chain_chomp_history_mut().set_x(i, cur_x);
            self.chain_chomp_history_mut().set_y(i, cur_y);
            i += 1;
        }
        let x_low = self.sprite_slot_view(k).x_low();
        let x_high = self.sprite_slot_view(k).x_high();
        let y_low = self.sprite_slot_view(k).y_low();
        let y_high = self.sprite_slot_view(k).y_high();
        self.sprite_slot_view_mut(k).set_a(x_low);
        self.sprite_slot_view_mut(k).set_b(x_high);
        self.sprite_slot_view_mut(k).set_c(y_low);
        self.sprite_slot_view_mut(k).set_g(y_high);
    }

    pub(super) fn chain_chomp_move_chain(&mut self, k: usize) {
        let x = u16::from(self.sprite_slot_view(k).a())
            | (u16::from(self.sprite_slot_view(k).b()) << 8);
        let y = u16::from(self.sprite_slot_view(k).c())
            | (u16::from(self.sprite_slot_view(k).g()) << 8);
        let mut pos = k * 8;
        let x2 = self
            .game_state
            .sprites
            .chain_chomp_history
            .x(pos)
            .wrapping_sub(x);
        let y2 = self
            .game_state
            .sprites
            .chain_chomp_history
            .y(pos)
            .wrapping_sub(y);
        pos += 1;

        for _ in (0..=5).rev() {
            let mul = CHAIN_CHOMP_MOVE_CHAIN_MULS[(pos & 7) - 1];
            let x3 = x.wrapping_add_signed(chain_chomp_one_mult_prep(x2 as u8, mul) as i16);
            let y3 = y.wrapping_add_signed(chain_chomp_one_mult_prep(y2 as u8, mul) as i16);

            let old_x = self.game_state.sprites.chain_chomp_history.x(pos);
            let dx = old_x.wrapping_sub(x3);
            if dx != 0 {
                let new_x = if sign16(dx) {
                    old_x.wrapping_add(1)
                } else {
                    old_x.wrapping_sub(1)
                };
                self.chain_chomp_history_mut().set_x(pos, new_x);
            }

            let old_y = self.game_state.sprites.chain_chomp_history.y(pos);
            let dy = old_y.wrapping_sub(y3);
            if dy != 0 {
                let new_y = if sign16(dy) {
                    old_y.wrapping_add(1)
                } else {
                    old_y.wrapping_sub(1)
                };
                self.chain_chomp_history_mut().set_y(pos, new_y);
            }

            pos += 1;
        }
    }

    pub(super) fn chain_chomp_handle_leash(&mut self, k: usize) {
        let mut pos = k * 8;
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        self.chain_chomp_history_mut().set_x(pos, cur_x);
        self.chain_chomp_history_mut().set_y(pos, cur_y);

        for _ in 0..6 {
            let x = self.game_state.sprites.chain_chomp_history.x(pos);
            let next_x = self.game_state.sprites.chain_chomp_history.x(pos + 1);
            let dx = x.wrapping_sub(next_x);
            if !sign16(dx.wrapping_sub(8)) {
                self.chain_chomp_history_mut()
                    .set_x(pos + 1, x.wrapping_sub(8));
            } else if sign16(dx.wrapping_add(8)) {
                self.chain_chomp_history_mut()
                    .set_x(pos + 1, x.wrapping_add(8));
            }

            let y = self.game_state.sprites.chain_chomp_history.y(pos);
            let next_y = self.game_state.sprites.chain_chomp_history.y(pos + 1);
            let dy = y.wrapping_sub(next_y);
            if !sign16(dy.wrapping_sub(8)) {
                self.chain_chomp_history_mut()
                    .set_y(pos + 1, y.wrapping_sub(8));
            } else if sign16(dy.wrapping_add(8)) {
                self.chain_chomp_history_mut()
                    .set_y(pos + 1, y.wrapping_add(8));
            }

            pos += 1;
        }
    }

    pub(super) fn sprite_prep_big_fairy(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(24);
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_mrs_sahasrahla(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_y_low(8);
        self.sprite_prep_magic_bat(k);
    }

    pub(super) fn sprite_prep_magic_bat(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_x_low(8);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_fortune_teller(&mut self, k: usize) {
        self.sprite_prep_incr_xy_low8(k);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_fairy_pond(&mut self, k: usize) {
        let j = ((self.sprite_slot_view(k).x_low() >> 4) & 1) as usize;
        self.sprite_slot_view_mut(k).set_a(j as u8);
        self.sprite_slot_view_mut(k)
            .set_oam_flags(SPRITE_PREP_FAIRY_POND_OAM_FLAGS[j]);
    }

    pub(super) fn sprite_prep_hobo(&mut self, k: usize) {
        for _ in 1..=15 {
            self.sprite_prep_hobo_spawn_smoke(k);
        }
        for i in (1..=15).rev() {
            if self.sprite_slot_view(i).sprite_type() == 0x2b {
                self.sprite_slot_view_mut(i).set_state(0);
            }
        }
        self.sprite_prep_hobo_spawn_fire(k);
        if self
            .game_state
            .inventory
            .save_progress
            .progress_indicator_3()
            & 1
            != 0
        {
            self.sprite_slot_view_mut(0).set_ai_state(3);
        }
        self.sprite_slot_view_mut(0).set_ignore_projectile(1);
    }

    pub(super) fn sprite_prep_hobo_spawn_smoke(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_view_mut(j).set_subtype2(0);
            self.sprite_slot_view_mut(j).set_ignore_projectile(0);
        }
    }

    pub(super) fn sprite_prep_hobo_spawn_fire(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_x(j, 0x0194);
            self.sprite_set_y(j, 0x003f);
            self.sprite_slot_view_mut(j).set_subtype2(2);
            self.sprite_slot_view_mut(j).set_ignore_projectile(2);
            self.sprite_slot_view_mut(j).set_flags2(0);
            self.sprite_slot_view_mut(j).masked_or_oam_flags(!0x0e, 2);
        }
    }

    pub(super) fn hobo_spawn_bubble(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_view_mut(j_usize).set_subtype2(1);
            self.sprite_slot_view_mut(j_usize).set_z_velocity(2);
            self.sprite_slot_view_mut(j_usize).set_delay_main(96);
            self.sprite_slot_view_mut(j_usize).set_delay_aux1(48);
            self.sprite_slot_view_mut(j_usize).set_ignore_projectile(48);
            self.sprite_slot_view_mut(j_usize).set_flags2(0);
        }
        j
    }

    pub(super) fn hobo_spawn_smoke(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_set_y(j, info.r2_y.wrapping_sub(4));
            self.sprite_slot_view_mut(j).set_subtype2(3);
            self.sprite_slot_view_mut(j).set_z_velocity(7);
            self.sprite_slot_view_mut(j).set_delay_main(96);
            self.sprite_slot_view_mut(j).set_ignore_projectile(96);
            self.sprite_slot_view_mut(j).set_flags2(0);
        }
    }

    pub(super) fn sprite_prep_master_sword(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_x_low(6);
        self.sprite_slot_view_mut(k).add_y_low(6);
    }

    pub(super) fn sprite_prep_roller_horizontal_right_first(&mut self, k: usize) {
        let ai_state = (!self.sprite_slot_view(k).x_low() & 16) >> 4;
        self.sprite_slot_view_mut(k).set_ai_state(ai_state);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_view_mut(k).increment_flags4();
        }
        self.sprite_slot_view_mut(k).set_direction(0);
    }

    pub(super) fn sprite_prep_roller_left_right(&mut self, k: usize) {
        let ai_state = (!self.sprite_slot_view(k).x_low() & 16) >> 4;
        self.sprite_slot_view_mut(k).set_ai_state(ai_state);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_view_mut(k).increment_flags4();
        }
        self.sprite_slot_view_mut(k).set_direction(1);
    }

    pub(super) fn sprite_prep_roller_vertical_down_first(&mut self, k: usize) {
        let ai_state = (self.sprite_slot_view(k).y_low() & 16) >> 4;
        self.sprite_slot_view_mut(k).set_ai_state(ai_state);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_view_mut(k).increment_flags4();
        }
        self.sprite_slot_view_mut(k).set_direction(2);
    }

    pub(super) fn sprite_prep_roller_up_down(&mut self, k: usize) {
        let ai_state = (self.sprite_slot_view(k).y_low() & 16) >> 4;
        self.sprite_slot_view_mut(k).set_ai_state(ai_state);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_view_mut(k).increment_flags4();
        }
        self.sprite_slot_view_mut(k).set_direction(3);
    }

    pub(super) fn sprite_prep_kodongo(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_x_low(4);
        self.sprite_set_y(k, self.sprite_get_y(k).wrapping_sub(5));
        self.sprite_slot_view_mut(k).decrement_subtype();
    }

    pub(super) fn sprite_prep_spark(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).decrement_subtype();
    }

    pub(super) fn sprite_prep_lost_woods_bird(&mut self, k: usize) {
        let z_velocity = (self.get_random_number() & 0x1f).wrapping_sub(0x10);
        self.sprite_slot_view_mut(k).set_z_velocity(z_velocity);
        self.sprite_slot_view_mut(k).set_z(64);
        self.sprite_prep_lost_woods_squirrel(k);
    }

    pub(super) fn sprite_prep_lost_woods_squirrel(&mut self, k: usize) {
        let x_velocity = if self.sprite_is_right_of_link(k).a != 0 {
            (-16i8) as u8
        } else {
            16
        };
        self.sprite_slot_view_mut(k).set_x_velocity(x_velocity);
        let y_vel = if sign8(self.overworld_vertical_scroll_delta_low()) {
            4
        } else {
            (-4i8) as u8
        };
        self.sprite_slot_view_mut(k).set_y_velocity(y_vel);
        self.sprite_slot_view_mut(k).set_ignore_projectile(y_vel);
    }

    pub(super) fn sprite_prep_antifairy(&mut self, k: usize) {
        let idx = ((self.sprite_slot_view(k).x_low() >> 4) & 1) as usize;
        self.sprite_slot_view_mut(k)
            .set_x_velocity(SPRITE_PREP_ANTIFAIRY_X_VELOCITIES[idx] as u8);
        self.sprite_slot_view_mut(k).set_y_velocity((-16i8) as u8);
    }

    pub(super) fn sprite_prep_antifairy_circle(&mut self, k: usize) {
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(10));
        self.sprite_slot_view_mut(k).set_y_velocity((-18i8) as u8);
        self.sprite_slot_view_mut(k).set_x_velocity(0);
        self.sprite_slot_view_mut(k).set_a(0);
        self.sprite_slot_view_mut(k).set_b(0);
        self.temp_counter_mut().set(2);
        loop {
            let i = self.game_state.scratch_counter.value() as usize;
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x82, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(
                    j,
                    info.r0_x
                        .wrapping_add(SPRITE_PREP_ANTIFAIRY_CIRCLE_X_OFFSETS[i] as u16),
                );
                self.sprite_set_y(
                    j,
                    info.r2_y
                        .wrapping_add(SPRITE_PREP_ANTIFAIRY_CIRCLE_Y_OFFSETS[i] as u16),
                );
                self.sprite_slot_view_mut(j)
                    .set_x_velocity(SPRITE_PREP_ANTIFAIRY_CIRCLE_X_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j)
                    .set_y_velocity(SPRITE_PREP_ANTIFAIRY_CIRCLE_Y_VELOCITIES[i] as u8);
                self.sprite_slot_view_mut(j)
                    .set_a(SPRITE_PREP_ANTIFAIRY_CIRCLE_ATTRS[i]);
                self.sprite_slot_view_mut(j)
                    .set_b(SPRITE_PREP_ANTIFAIRY_CIRCLE_B[i]);
            }
            self.temp_counter_mut().decrement();
            if sign8(self.game_state.scratch_counter.value()) {
                break;
            }
        }
    }

    pub(super) fn sprite_prep_king_zora(&mut self, k: usize) {
        if self.game_state.inventory.items.flippers() != 0 {
            self.sprite_slot_view_mut(k).set_state(0);
        } else {
            self.sprite_slot_view_mut(k).increment_ignore_projectile();
        }
    }

    pub(super) fn sprite_prep_do_nothing_d(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_octorok(&mut self, k: usize) {
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_view_mut(k)
            .set_health(SPRITE_PREP_OCTOROK_HEALTH_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_bump_damage(SPRITE_PREP_OCTOROK_BUMP_DAMAGE_VALUES[j]);
        let delay_main = self.get_random_number() & 127;
        self.sprite_slot_view_mut(k).set_delay_main(delay_main);
    }

    pub(super) fn sprite_prep_swimming_zora(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_delay_main(64);
        self.sprite_prep_geldman(k);
    }

    pub(super) fn sprite_prep_geldman(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_x_low(8);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_kyameron(&mut self, k: usize) {
        let x_low = self.sprite_slot_view(k).x_low();
        let x_high = self.sprite_slot_view(k).x_high();
        let y_low = self.sprite_slot_view(k).y_low();
        let y_high = self.sprite_slot_view(k).y_high();
        self.sprite_slot_view_mut(k).set_a(x_low);
        self.sprite_slot_view_mut(k).set_b(x_high);
        self.sprite_slot_view_mut(k).set_c(y_low);
        self.sprite_slot_view_mut(k).set_head_direction(y_high);
    }

    pub(super) fn sprite_prep_walking_zora(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_delay_main(96);
    }

    pub(super) fn sprite_prep_talking_tree(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        let x = self.sprite_get_x(k).wrapping_sub(8);
        self.sprite_set_x(k, x);
        self.sprite_prep_talking_tree_spawn_eyeball(k, 0);
        self.sprite_prep_talking_tree_spawn_eyeball(k, 1);
    }

    pub(super) fn sprite_prep_talking_tree_spawn_eyeball(&mut self, k: usize, dir: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x25, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_slot_view_mut(j).set_head_direction(dir as u8);
            let x = info.r0_x.wrapping_add(
                SPRITE_PREP_TALKING_TREE_SPAWN_EYEBALL_TALKING_TREE_SPAWN_X[dir] as u16,
            );
            let y = info.r2_y.wrapping_sub(11);
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);
            self.sprite_slot_view_mut(j).set_a(x as u8);
            self.sprite_slot_view_mut(j).set_b((x >> 8) as u8);
            self.sprite_slot_view_mut(j).set_c(y as u8);
            self.sprite_slot_view_mut(j).set_e((y >> 8) as u8);
            self.sprite_slot_view_mut(j).set_subtype2(1);
        }
    }

    pub(super) fn sprite_prep_swamola(&mut self, k: usize) {
        self.sprite_prep_swamola_initialize_segments(k);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_swamola_initialize_segments(&mut self, k: usize) {
        let mut j = if self
            .game_state
            .enhanced_features
            .has(FEATURE_MISC_BUG_FIXES_PREP)
        {
            k * 32
        } else {
            SPRITE_PREP_SWAMOLA_INITIALIZE_SEGMENTS_BUGGY_SWAMOLA_LOOKUP[k]
        };
        let x = self.sprite_slot_view(k).x();
        let y = self.sprite_slot_view(k).y();
        for _ in 0..32 {
            self.swamola_history_mut(j).set_position(x, y);
            j += 1;
        }
    }

    pub(super) fn sprite_prep_flute_kid(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        let subtype2 = (self.game_state.inventory.save_progress.dark_world_state() >> 6) & 1;
        self.sprite_slot_view_mut(k).set_subtype2(subtype2);
        let flute = self.game_state.inventory.items.flute();
        if self.sprite_slot_view(k).subtype2() != 0 {
            if self
                .game_state
                .inventory
                .save_progress
                .progress_indicator_3()
                & 8
                != 0
                || flute > 2
            {
                self.sprite_slot_view_mut(k).set_graphics(3);
                self.sprite_slot_view_mut(k).set_ai_state(5);
            } else if flute == 2 {
                self.sprite_slot_view_mut(k).set_graphics(1);
            }
            self.sprite_slot_view_mut(k).add_x_low(8);
            self.sprite_slot_view_mut(k).subtract_y_low(8);
        } else if flute >= 2 {
            self.sprite_slot_view_mut(k).set_state(0);
        } else {
            self.sprite_slot_view_mut(k).add_x_low(7);
        }
    }

    pub(super) fn sprite_prep_move_down_8px(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_y_low(8);
    }

    pub(super) fn sprite_prep_zazakku(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_pedestal_plaque(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        if self.game_state.world.location.overworld_screen_index() == 48 {
            self.sprite_slot_view_mut(k).add_x_low(7);
        }
    }

    pub(super) fn sprite_prep_stalfos(&mut self, k: usize) {
        let subtype = self.sprite_slot_view(k).x_low() & 16;
        self.sprite_slot_view_mut(k).set_subtype(subtype);
        if self.sprite_slot_view(k).subtype() != 0 {
            self.sprite_slot_view_mut(k).set_oam_flags(7);
        }
    }

    pub(super) fn sprite_prep_moldorm(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        self.sprite_initialized_segmented(k);
    }

    pub(super) fn sprite_prep_lanmolas(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k)
            .set_delay_main(SPRITE_PREP_LANMOLAS_INIT_DELAY[k]);
        self.sprite_slot_view_mut(k).set_z(0xff);
        for i in 0..64 {
            self.lanmola_segment_motion_mut(k * 0x40 + i)
                .set_z_offset(0xff);
        }
        let value = 7;
        self.garnish_slot_view_mut(k).set_y_low(value);
    }

    pub(super) fn sprite_prep_bumper(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_move_down_8px_right8px(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_x_low(8);
        self.sprite_slot_view_mut(k).add_y_low(8);
    }

    pub(super) fn sprite_prep_hardhat_beetle(&mut self, k: usize) {
        let j = usize::from((self.sprite_slot_view(k).x_low() & 0x10) != 0);
        self.sprite_slot_view_mut(k)
            .set_oam_flags(SPRITE_PREP_HARDHAT_BEETLE_OAM_FLAGS[j]);
        self.sprite_slot_view_mut(k)
            .set_health(SPRITE_PREP_HARDHAT_BEETLE_HEALTH_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_a(SPRITE_PREP_HARDHAT_BEETLE_ATTRS[j]);
        self.sprite_slot_view_mut(k)
            .set_ai_state(SPRITE_PREP_HARDHAT_BEETLE_STATE[j]);
        self.sprite_slot_view_mut(k)
            .set_flags5(SPRITE_PREP_HARDHAT_BEETLE_FLAGS5_VALUES[j]);
        self.sprite_slot_view_mut(k)
            .set_bump_damage(SPRITE_PREP_HARDHAT_BEETLE_BUMP_DAMAGE_VALUES[j]);
    }

    pub(super) fn sprite_prep_mini_helmasaur(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_a(16);
        self.sprite_slot_view_mut(k).set_ai_state(1);
    }

    pub(super) fn sprite_prep_fairy(&mut self, k: usize) {
        let a = self.get_random_number() & 1;
        self.sprite_slot_view_mut(k).set_a(a);
        self.sprite_slot_view_mut(k).set_direction(a ^ 1);
        self.sprite_prep_absorbable(k);
    }

    pub(super) fn sprite_prep_falling_ice(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_armos_knight(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k).set_delay_main(255);
        self.sprite_workspace_mut().increment_prep_shared_counter();
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_desert_statue(&mut self, k: usize) {
        let limit_instance = self.game_state.sprites.system.limit_instance();
        self.sprite_slot_view_mut(k).set_a(limit_instance);
        self.sprite_system_mut().increment_limit_instance();
        self.sprite_prep_move_down_8px_right8px(k);
        let x_low = self.sprite_slot_view(k).x_low();
        let direction = if x_low < 0x30 {
            1
        } else if x_low < 0xe0 {
            3
        } else {
            2
        };
        self.sprite_slot_view_mut(k).set_direction(direction);
    }

    pub(super) fn sprite_prep_big_spike(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_crystal_switch(&mut self, k: usize) {
        let oam_flags = SPRITE_PREP_CRYSTAL_SWITCH_CRYSTAL_SWITCH_PAL[(self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            & 1) as usize];
        self.sprite_slot_view_mut(k).or_oam_flags(oam_flags);
    }

    pub(super) fn sprite_prep_kholdstare_shell(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k).set_delay_aux1(192);
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_kholdstare(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k).set_ai_state(3);
        self.sprite_prep_ignore_projectiles(k);
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_agahnim(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k).set_graphics(0);
        self.sprite_slot_view_mut(k).set_direction(3);
        self.sprite_prep_move_down_8px_right8px(k);
        let oam_flags = SPRITE_PREP_AGAHNIM_OAM_FLAGS
            [self.game_state.world.region.dark_world_region_index() as usize];
        self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
    }

    pub(super) fn sprite_prep_trinexx(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.trinexx_components_initialize(k);
        for i in (0..=15).rev() {
            self.alt_sprite_slot_mut(i).clear_state();
        }
    }

    pub(super) fn trinexx_components_initialize(&mut self, k: usize) {
        match self.sprite_slot_view(k).sprite_type() {
            0xcb => {
                self.sprite_slot_view_mut(k).add_x_low(8);
                self.sprite_slot_view_mut(k).add_y_low(16);
                self.trinexx_cache_position(k);
                self.overlord_slot_view_mut(2).set_x_low(0);
                self.overlord_slot_view_mut(3).set_x_low(0);
                self.overlord_slot_view_mut(5).set_x_low(0);
                self.overlord_slot_view_mut(7).set_x_low(0);
                self.overlord_slot_view_mut(0).set_x_high(0);
                self.overlord_slot_view_mut(6).set_x_low(255);
                self.trinexx_restore_xy(k);
            }
            0xcc => {
                self.sprite_slot_view_mut(k).set_graphics(3);
                self.sprite_slot_view_mut(k).set_delay_main(128);
                self.trinexx_initialize_alt_sprites(k);
            }
            0xcd => {
                self.sprite_slot_view_mut(k).set_delay_main(255);
                self.trinexx_initialize_alt_sprites(k);
            }
            _ => {}
        }
    }

    fn trinexx_initialize_alt_sprites(&mut self, k: usize) {
        for j in (0..=0x1a).rev() {
            self.alt_sprite_slot_mut(j).initialize_trinexx_component();
        }
        self.sprite_slot_view_mut(k).set_subtype2(1);
        self.trinexx_cache_position(k);
    }

    pub(super) fn sprite_prep_helmasaur_king(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.helmasaur_king_initialize(k);
        for i in 0..16 {
            self.alt_sprite_slot_mut(i).clear_state();
        }
    }

    pub(super) fn sprite_prep_absorbable(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors() {
            self.sprite_slot_view_mut(k).increment_e();
            self.sprite_slot_view_mut(k).increment_ignore_projectile();
        }
    }

    pub(super) fn sprite_prep_overworld_bonk_item(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_e();
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_shield_pickup(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_nice_bee(&mut self, k: usize) {
        let or_bottle = self.game_state.inventory.items.bottle(0)
            | self.game_state.inventory.items.bottle(1)
            | self.game_state.inventory.items.bottle(2)
            | self.game_state.inventory.items.bottle(3);
        if or_bottle & 8 != 0 {
            self.sprite_slot_view_mut(k).set_state(0);
        }
        self.sprite_slot_view_mut(k).increment_e();
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_do_nothing_g(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_fire_bar(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_b();
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_spike(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_x_velocity(32);
        self.sprite_slot_view_mut(k).set_y_velocity((-16i8) as u8);
        self.sprite_move_y(k);
        self.sprite_slot_view_mut(k).set_y_velocity(0);
    }

    pub(super) fn sprite_prep_rock_stal(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_y_velocity((-16i8) as u8);
        self.sprite_move_y(k);
        self.sprite_slot_view_mut(k).set_y_velocity(0);
    }

    pub(super) fn sprite_prep_blob(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_graphics(4);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub(super) fn sprite_prep_arrghus(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k).set_z(24);
    }

    pub(super) fn sprite_prep_arrghi(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        let subtype2 = self.get_random_number();
        self.sprite_slot_view_mut(k).set_subtype2(subtype2);
        if k == 13 {
            self.overlord_slot_view_mut(2).set_x_low(0);
            self.overlord_slot_view_mut(3).set_x_low(0);
            self.arrghus_handle_puffs(0);
        }
        let puff_home = self.arrghus_puff_home_position(k);
        let x_low = puff_home.x_low();
        let x_high = puff_home.x_high();
        let y_low = puff_home.y_low();
        let y_high = puff_home.y_high();
        self.sprite_slot_view_mut(k).set_x_low(x_low);
        self.sprite_slot_view_mut(k).set_x_high(x_high);
        self.sprite_slot_view_mut(k).set_y_low(y_low);
        self.sprite_slot_view_mut(k).set_y_high(y_high);
    }

    pub(super) fn arrghus_handle_puffs(&mut self, k: usize) {
        // The orbit-phase accumulator is a 16-bit value at OVERLORD_X_LO that the C
        // reads/writes raw, spanning overlord slots 0 and 1's x_low bytes (NOT slot 0's
        // packed x_low|x_high). Use the adjacent-x-low-word accessor, exactly like
        // armos_coordinator_rotate, so we read/write the same bytes the C does.
        let base = self
            .game_state
            .sprites
            .overlord_slots
            .slot(0)
            .adjacent_x_low_word()
            .wrapping_add(self.overlord_slot_view(4).x_low() as u16);
        self.overlord_slot_view_mut(0).set_adjacent_x_low_word(base);

        if self.game_state.frame.frame_counter & 3 == 0 {
            self.sprite_slot_view_mut(k).increment_a();
            if self.sprite_slot_view(k).a() == 13 {
                self.sprite_slot_view_mut(k).set_a(0);
            }
        }
        if self.game_state.frame.frame_counter & 7 == 0 {
            self.sprite_slot_view_mut(k).increment_b();
            if self.sprite_slot_view(k).b() == 13 {
                self.sprite_slot_view_mut(k).set_b(0);
            }
        }

        let sprite_x = self.sprite_get_x(k) as i32;
        let sprite_y = self.sprite_get_y(k) as i32;
        for i in 0..13 {
            let r0 = base.wrapping_add(ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_BASE_ANGLES[i])
                ^ ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_ANGLE_XOR_MASKS[i];
            let r14 = self
                .game_state
                .sprites
                .overlord_slots
                .slot(2)
                .x_low()
                .wrapping_add(ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_PHASE_OFFSETS[i]);
            let sin_arg = r14.wrapping_add_signed(
                ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_WAVE_SHIFTS
                    [self.sprite_slot_view(k).a() as usize + i],
            );
            let cos_arg = r14.wrapping_add_signed(
                ARRGHUS_HANDLE_PUFFS_PUFF_ORBIT_WAVE_SHIFTS
                    [self.sprite_slot_view(k).b() as usize + i],
            );
            let sin_val = super::sprite_main_draw::arrgi_sin(r0, sin_arg) as i32;
            let cos_val = super::sprite_main_draw::arrgi_sin(r0.wrapping_add(0x80), cos_arg) as i32;

            let tx = (sprite_x + sin_val) as u16;
            let ty = (sprite_y + cos_val - 0x10) as u16;
            self.armos_knight_home_position_mut(i).set_position(tx, ty);
        }
        // The puff homes are written into the overlord slot array via the armos-knight
        // home bridge (raw RAM), which leaves the overlord native model stale. A later
        // overlord setter's sync() projects the WHOLE overlord block (write_to_ram) and
        // would re-stamp the stale home bytes over the fresh ones. Resync the native from
        // RAM now so it re-projects the homes we just wrote, not last frame's.
        self.game_state
            .sprites
            .reload_overlord_slots_from_ram(&self.ram);
        self.temp_counter_mut().set(13);
    }

    pub(super) fn sprite_prep_mothula(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_view_mut(k).set_delay_main(80);
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        self.sprite_slot_view_mut(k).set_graphics(2);
        self.dungeon_moving_floor_mut().increment_floor_move_flags();
        self.sprite_slot_view_mut(k).set_c(112);
    }

    pub(super) fn sprite_prep_do_nothing_h(&mut self, _k: usize) {}

    pub(super) fn heart_upgrade_check_if_already_obtained(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors() {
            let screen = self.game_state.world.location.overworld_screen_index() as usize;
            if (screen == 0x3b
                && self.game_state.world.overworld.event_info.event_info(0x3b) & 0x20 == 0)
                || self
                    .game_state
                    .world
                    .overworld
                    .event_info
                    .event_info(screen)
                    & 0x40
                    != 0
            {
                self.sprite_slot_view_mut(k).set_state(0);
            }
        } else {
            let j = self.sprite_slot_view(k).x_high() & 1;
            let mask = if j != 0 { 0x2000 } else { 0x4000 };
            if self.game_state.dungeon.savegame_state.savegame_state_bits() & mask != 0 {
                self.sprite_slot_view_mut(k).set_state(0);
            }
        }
    }

    pub(super) fn heart_upgrade_set_obtained_flag(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors() {
            let screen = self.game_state.world.location.overworld_screen_index() as usize;
            self.set_overworld_event_bits(screen, 0x40);
        } else {
            let mask = if self.sprite_slot_view(k).x_high() & 1 != 0 {
                0x2000
            } else {
                0x4000
            };
            let bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | mask;
            self.dungeon_savegame_state_mut()
                .set_savegame_state_bits(bits);
        }
    }

    pub(super) fn sprite_prep_heart_container(&mut self, k: usize) {
        self.heart_upgrade_check_if_already_obtained(k);
    }

    pub(super) fn sprite_prep_heart_piece(&mut self, k: usize) {
        self.heart_upgrade_check_if_already_obtained(k);
    }

    pub(super) fn sprite_prep_small_key(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_subtype(255);
        let j = self.game_state.sprite_battle.item_drop_counter();
        self.sprite_battle_mut().increment_item_drop_counter();
        self.sprite_slot_view_mut(k).set_die_action(j);
    }

    pub(super) fn sprite_prep_key_set_item_drop(&mut self, k: usize) {
        let die_action = self.game_state.sprite_battle.item_drop_counter();
        self.sprite_slot_view_mut(k).set_die_action(die_action);
        self.sprite_battle_mut().increment_item_drop_counter();
    }

    pub(super) fn sprite_prep_big_key(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_x_low(8);
        self.sprite_slot_view_mut(k).set_subtype(0xff);
        self.sprite_prep_big_key_load_graphics(k);
    }

    pub(super) fn sprite_prep_big_key_load_graphics(&mut self, k: usize) {
        self.DecodeAnimatedSpriteTile_variable(0x22);
        self.sprite_prep_key_set_item_drop(k);
    }

    pub(super) fn sprite_prep_incr_xy_low8(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_x_low(8);
        self.sprite_slot_view_mut(k).add_y_low(8);
    }

    pub(super) fn sprite_prep_fake_sword(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_old_man_bounce(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        if self.game_state.world.location.dungeon_room_index() == 0xe4 {
            self.sprite_slot_view_mut(k).set_subtype2(2);
            return;
        }
        if self.game_state.sprites.follower_runtime.indicator() == 0 {
            if self.game_state.inventory.items.mirror() == 2 {
                self.sprite_slot_view_mut(k).set_state(0);
            }
            self.follower_state_mut().set_indicator(4);
            self.load_follower_graphics();
            self.follower_state_mut().set_indicator(0);
        } else {
            self.sprite_slot_view_mut(k).set_state(0);
            self.load_follower_graphics();
        }
    }

    pub(super) fn sprite_prep_snitch_bounce_1(&mut self, k: usize) {
        self.sprite_prep_snitches(k);
    }

    pub(super) fn sprite_prep_snitch_bounce_2(&mut self, k: usize) {
        self.sprite_prep_snitches(k);
    }

    pub(super) fn sprite_prep_snitch_bounce_3(&mut self, k: usize) {
        self.sprite_prep_snitches(k);
    }

    pub(super) fn sprite_prep_zelda_bounce(&mut self, k: usize) {
        let Some(saved_follower_indicator) = self.sprite_prep_zelda_before_follower_graphics(k)
        else {
            return;
        };
        self.load_follower_graphics();
        self.sprite_prep_zelda_after_follower_graphics(k, saved_follower_indicator);
    }

    pub(super) fn sprite_prep_zelda_before_follower_graphics(&mut self, k: usize) -> Option<u8> {
        if self.game_state.inventory.items.sword_type() >= 2 {
            self.sprite_slot_view_mut(k).set_state(0);
            return None;
        }
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        let dir = self.sprite_direction_to_face_link(k, None) ^ 3;
        self.sprite_slot_view_mut(k).set_direction(dir);
        self.sprite_slot_view_mut(k).set_head_direction(dir);

        let follower = self.game_state.sprites.follower_runtime.indicator();
        self.follower_state_mut().set_indicator(1);
        Some(follower)
    }

    pub(super) fn sprite_prep_zelda_after_follower_graphics(
        &mut self,
        k: usize,
        saved_follower_indicator: u8,
    ) {
        self.follower_state_mut()
            .set_indicator(saved_follower_indicator);

        if self.game_state.world.location.dungeon_room_index() == 0x12 {
            self.sprite_slot_view_mut(k).set_subtype2(2);
            if self.game_state.inventory.save_progress.progress_flags() & 4 == 0 {
                self.sprite_slot_view_mut(k).set_state(0);
            } else {
                let x = self.sprite_get_x(k).wrapping_add(6);
                let y = self.sprite_get_y(k).wrapping_add(15);
                self.sprite_set_x(k, x);
                self.sprite_set_y(k, y);
                self.sprite_slot_view_mut(k).set_flags4(3);
            }
        } else {
            self.sprite_slot_view_mut(k).set_subtype2(0);
            if self.game_state.sprites.follower_runtime.indicator() == 1
                || self.game_state.inventory.save_progress.progress_flags() & 4 != 0
            {
                self.sprite_slot_view_mut(k).set_state(0);
            }
        }
    }

    pub(super) fn sprite_prep_medallion_table(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ignore_projectile();
        if self.game_state.world.location.overworld_screen_index() != 3 {
            self.sprite_slot_view_mut(k).add_x_low(8);
            if self.game_state.inventory.items.bombos() != 0 {
                self.sprite_slot_view_mut(k).set_graphics(4);
                self.sprite_slot_view_mut(k).set_ai_state(3);
            }
        } else if self.game_state.inventory.items.ether() != 0 {
            self.sprite_slot_view_mut(k).set_graphics(4);
            self.sprite_slot_view_mut(k).set_ai_state(3);
        }
    }

    pub(super) fn sprite_prep_eyegore(&mut self, k: usize) {
        let room = self.game_state.dungeon.room_tracking.room_index2();
        if room == 12 || room == 27 || room == 75 || room == 107 {
            self.sprite_slot_view_mut(k).increment_b();
            if self.sprite_slot_view(k).sprite_type() == 0x83 {
                self.sprite_slot_view_mut(k).set_deflection_bits(0);
            }
        }
    }

    fn sprite_return_if_boss_finished(&mut self, k: usize) -> bool {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 != 0 {
            self.sprite_slot_view_mut(k).set_state(0);
            return true;
        }
        for j in (0..16).rev() {
            if SPRITE_INITIAL_BUMP_DAMAGE[self.sprite_slot_view(j).sprite_type() as usize] & 0x10
                == 0
            {
                self.sprite_slot_view_mut(j).set_state(0);
            }
        }
        false
    }

    pub(super) fn sprite_initialized_segmented(&mut self, k: usize) {
        let x = self.sprite_slot_view(k).x();
        let y = self.sprite_slot_view(k).y();
        for i in 0..128 {
            self.moldorm_history_mut(i).set_position(x, y);
        }
    }
}

#[cfg(test)]
#[path = "sprite_main_prep_tests.rs"]
mod tests;
