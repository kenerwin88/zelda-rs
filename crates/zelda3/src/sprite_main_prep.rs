//! Ported SpritePrep_* helpers from sprite_main.c.

use super::*;
use crate::types::{sign16, sign8};
use crate::zelda_rtl::sprite::{DrawMultipleData, SpriteSpawnInfo};

const IS_IN_DARK_WORLD_PREP: usize = 0x0fff;
const DUNG_FLOOR_MOVE_FLAGS_PREP: usize = 0x041a;
const ACTIVE_OVERLORD_INDEX_PREP: usize = 0x0fde;
const SPRITE_PREP_SHARED_COUNTER: usize = 0x0ff8;
const LINK_RUPEES_IN_POND_PREP: usize = 0x0f36a;
const ITEM_DROP_LUCK_PREP: usize = 0x0cf9;
const LUCK_KILL_COUNTER_PREP: usize = 0x0cfa;
const NUM_SPRITES_KILLED_PREP: usize = 0x0cfb;
const SPRITE_DELAY_AUX3_PREP: usize = 0x0ee0;
const MINIGAME_CREDITS_PREP: usize = 0x04c4;
const FLAG_OVERWORLD_AREA_DID_CHANGE_PREP: usize = 0x0abf;
const SRAM_PROGRESS_INDICATOR_3_PREP: usize = 0x0f3c9;
const SPRCOLL_X_BASE_PREP: usize = 0x0fbc;
const SPRCOLL_Y_BASE_PREP: usize = 0x0fbe;
const CHAINCHOMP_X_HIST_PREP: usize = 0x1fc00;
const CHAINCHOMP_Y_HIST_PREP: usize = 0x1fd00;
const FEATURE_MISC_BUG_FIXES_PREP: u32 = 4096;

#[cfg(test)]
const ALT_SPRITE_STATE_PREP: usize = 0x1d00;
#[cfg(test)]
const ALT_SPRITE_TYPE_PREP: usize = 0x1d10;
#[cfg(test)]
const ALT_SPRITE_X_HI_PREP: usize = 0x1d30;
#[cfg(test)]
const ALT_SPRITE_Y_HI_PREP: usize = 0x1d50;
#[cfg(test)]
const BEAMOS_X_LO_PREP: usize = 0x1fd80;
#[cfg(test)]
const BEAMOS_Y_LO_PREP: usize = 0x1fe80;
#[cfg(test)]
const BEAMOS_Y_HI_PREP: usize = 0x1ff00;
#[cfg(test)]
const MOLDORM_X_LO_PREP: usize = 0x1fc00;
#[cfg(test)]
const MOLDORM_X_HI_PREP: usize = 0x1fc80;
#[cfg(test)]
const MOLDORM_Y_LO_PREP: usize = 0x1fd00;
#[cfg(test)]
const MOLDORM_Y_HI_PREP: usize = 0x1fd80;
const OVERLORD_X_HI_PREP: usize = 0x0b10;
#[cfg(test)]
const OVERLORD_Y_LO_PREP: usize = 0x0b18;
#[cfg(test)]
const OVERLORD_Y_HI_PREP: usize = 0x0b20;
#[cfg(test)]
const OVERLORD_GEN1_PREP: usize = 0x0b28;
#[cfg(test)]
const OVERLORD_GEN2_PREP: usize = 0x0b30;
#[cfg(test)]
const OVERLORD_GEN3_PREP: usize = 0x0b38;
#[cfg(test)]
const OVERLORD_FLOOR_PREP: usize = 0x0b40;
#[cfg(test)]
const SWAMOLA_X_LO_PREP: usize = 0x1fa5c;
#[cfg(test)]
const SWAMOLA_X_HI_PREP: usize = 0x1fb1c;
#[cfg(test)]
const SWAMOLA_Y_LO_PREP: usize = 0x1fbdc;
#[cfg(test)]
const SWAMOLA_Y_HI_PREP: usize = 0x1fc9c;

const WISH_POND_SPARKLE_X_OFFSETS: [u8; 8] = [0, 4, 8, 12, 16, 20, 24, 0];
const WISH_POND_SPARKLE_Y_OFFSETS: [u8; 8] = [0, 8, 16, 24, 32, 40, 4, 36];
const WISH_POND_ITEM_OAM_FLAGS: [u8; 76] = [
    5, 0xff, 5, 5, 5, 5, 5, 1, 2, 1, 1, 1, 2, 2, 2, 4, 4, 4, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 4, 4, 2,
    1, 6, 1, 2, 1, 2, 2, 1, 2, 2, 4, 1, 1, 4, 2, 1, 4, 2, 2, 4, 4, 4, 2, 1, 4, 1, 2, 2, 1, 2, 2, 1,
    1, 4, 4, 1, 2, 2, 4, 4, 4, 2, 5, 2, 1,
];
const RECEIVE_ITEM_PREP_DRAW_FRAME_START_BYTES: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
const WISH_POND_ITEM_DATA_OFFSETS: [u8; 32] = [
    0, 4, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 20, 21, 22, 22, 23, 24, 25, 28, 30, 31, 32, 33,
    33, 37, 40, 42, 42, 42, 42,
];
const WISH_POND_ITEM_DATA: [u8; 50] = [
    0x3a, 0x3a, 0x3b, 0x3b, 0x0c, 0x2a, 0x0a, 0x27, 0x29, 0x0d, 0x07, 0x08, 0x0f, 0x10, 0x11, 0x12,
    0x09, 0x13, 0x14, 0x4a, 0x21, 0x1d, 0x15, 0x18, 0x19, 0x31, 0x1a, 0x1a, 0x1b, 0x1c, 0x4b, 0x1e,
    0x1f, 0x49, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x22, 0x23, 0x29, 0x16, 0x2b, 0x2c, 0x2d, 0x3d,
    0x3c, 0x48,
];

const SPRITE_INITIAL_BUMP_DAMAGE: [u8; 243] = [
    0x83, 0x83, 0x81, 2, 2, 2, 2, 2, 1, 0x13, 1, 1, 1, 1, 8, 1, 1, 8, 5, 3, 0x40, 4, 0, 2, 3, 0x85,
    0, 1, 0, 0x40, 0, 0, 6, 0, 5, 3, 1, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
    0, 0, 0, 0, 0, 0, 2, 2, 0, 1, 1, 3, 1, 3, 1, 1, 3, 3, 3, 1, 3, 1, 1, 1, 1, 1, 1, 0x11, 0x14, 1,
    1, 2, 5, 0, 0, 4, 4, 8, 8, 8, 8, 4, 0, 4, 3, 2, 2, 2, 2, 2, 3, 1, 0, 0, 1, 0x80, 5, 1, 0, 0, 0,
    0x40, 0, 4, 0, 0, 0x14, 4, 6, 4, 4, 4, 4, 3, 4, 4, 4, 1, 4, 4, 0x15, 5, 4, 5, 0x15, 0x15, 3, 5,
    0, 5, 0x15, 5, 5, 6, 6, 6, 6, 5, 3, 6, 5, 5, 3, 3, 3, 6, 0x17, 0x15, 0x15, 5, 5, 1, 0x85, 0x83,
    5, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x17, 0x17, 5, 5, 5, 4, 3, 2, 0x10, 0,
    6, 0, 5, 7, 0x17, 0x17, 0x17, 0x15, 7, 6, 0x10, 0, 3, 3, 0, 0x19, 0x19, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn chain_chomp_one_mult_prep(a: u8, b: u8) -> i32 {
    let at = if sign8(a) { 0u8.wrapping_sub(a) } else { a };
    let prod = (((at as u16) * (b as u16)) >> 8) as u8;
    if sign8(a) {
        !(prod as i32)
    } else {
        prod as i32
    }
}

impl ZeldaState {
    pub(super) fn sprite_prep_throwable_scenery(&mut self, _k: usize) {}

    // void SpriteModule_Initialize(int k) {  // 86864d
    pub(super) fn sprite_module_initialize(&mut self, k: usize) {
        self.sprite_prep_load_properties(k);
        self.sprite_slot_mut(k).increment_state();
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
        const GUARD_SUBTYPE_B_REMAP: [u8; 8] = [0, 2, 1, 3, 6, 4, 5, 7];

        let subtype = self.sprite_slot_view(k).subtype();
        if subtype != 0 {
            if (subtype & 7) >= 5 {
                let j = usize::from(if (subtype & 7) != 5 { 4 } else { 0 } + ((subtype >> 3) & 3));
                self.sprite_slot_mut(k).set_b(GUARD_SUBTYPE_B_REMAP[j]);
                self.sprite_slot_mut(k).masked_or_flags(0x0f, 0x50);
                self.sprite_prep_trooper_and_archer_soldier(k);
                return;
            }
            self.sprite_slot_mut(k)
                .set_direction(((subtype & 7).wrapping_sub(1)) ^ 1);
        }
        if self.game_state.world.location.is_indoors() {
            self.sprite_slot_mut(k).and_flags5(!0x80);
            return;
        }
        self.sprite_slot_mut(k).set_ai_state(1);
        self.sprite_slot_mut(k).set_delay_main(112);
        let dir = self.sprite_direction_to_face_link(k, None);
        self.sprite_slot_mut(k).set_direction(dir);
        self.sprite_slot_mut(k).set_head_direction(dir);
        self.sprite_prep_trooper_and_archer_soldier(k);
    }

    // void SpritePrep_TrooperAndArcherSoldier(int k) {  // 869001
    pub(super) fn sprite_prep_trooper_and_archer_soldier(&mut self, k: usize) {
        let bak0 = self.game_state.frame.submodule;
        self.set_submodule(0);
        let deflection_bits = (self.sprite_slot_view(k).deflection_bits() >> 1) | 0x80;
        self.sprite_slot_mut(k).set_deflection_bits(deflection_bits);
        self.sprite_active_main(k);
        self.sprite_active_main(k);
        let deflection_bits = self.sprite_slot_view(k).deflection_bits().wrapping_shl(1);
        self.sprite_slot_mut(k).set_deflection_bits(deflection_bits);
        self.set_submodule(bak0);
    }

    pub(super) fn sprite_prep_mantle(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_y_low(3);
        self.sprite_slot_mut(k).add_x_low(8);
    }

    pub(super) fn sprite_prep_switch(&mut self, k: usize) {
        let room = self.game_state.dungeon.room_tracking.room_index2();
        if room == 0xce || room == 4 || room == 0x3f {
            self.sprite_slot_mut(k).set_oam_flags(0x0d);
        }
    }

    pub(super) fn sprite_prep_switch_facing_up(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_do_nothing_a(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_rat(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [0, 5];
        const HEALTH: [u8; 2] = [2, 8];
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_mut(k).set_bump_damage(BUMP_DAMAGE[j]);
        self.sprite_slot_mut(k).set_health(HEALTH[j]);
    }

    pub(super) fn sprite_prep_keese(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [0x80, 0x85];
        const HEALTH: [u8; 2] = [1, 4];
        const FLAGS5: [u8; 2] = [0, 7];
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_mut(k).set_bump_damage(BUMP_DAMAGE[j]);
        self.sprite_slot_mut(k).set_health(HEALTH[j]);
        self.sprite_slot_mut(k).set_flags5(FLAGS5[j]);
    }

    pub(super) fn sprite_prep_rope(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [1, 5];
        const HEALTH: [u8; 2] = [4, 8];
        const FLAGS5: [u8; 2] = [1, 7];
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_mut(k).set_bump_damage(BUMP_DAMAGE[j]);
        self.sprite_slot_mut(k).set_health(HEALTH[j]);
        self.sprite_slot_mut(k).set_flags5(FLAGS5[j]);
    }

    pub(super) fn sprite_prep_babasu(&mut self, k: usize) {
        self.sprite_prep_move_down_8px(k);
        self.sprite_prep_zoro(k);
    }

    pub(super) fn sprite_prep_pokey(&mut self, k: usize) {
        const INIT_XVEL: [i8; 4] = [16, -16, 16, -16];
        const INIT_YVEL: [i8; 4] = [16, 16, -16, -16];
        self.sprite_slot_mut(k).set_a(3);
        self.sprite_slot_mut(k).set_b(8);
        let j = (self.get_random_number() & 3) as usize;
        self.sprite_slot_mut(k).set_x_velocity(INIT_XVEL[j] as u8);
        self.sprite_slot_mut(k).set_y_velocity(INIT_YVEL[j] as u8);
    }

    pub(super) fn sprite_prep_gibo(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_z(16);
        self.sprite_slot_mut(k).set_g(8);
    }

    pub(super) fn sprite_prep_octoballoon(&mut self, k: usize) {
        const DELAY: [u8; 4] = [192, 208, 224, 240];
        self.sprite_slot_mut(k).set_delay_main(DELAY[k & 3]);
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
        self.sprite_slot_mut(k).set_delay_main(128);
        self.sprite_slot_mut(k).set_room(2);
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
            self.sprite_slot_mut(k).set_graphics(4);
        }
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_slot_mut(k).subtract_y_low(12);
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_catfish(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_slot_mut(k).subtract_y_low(12);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub(super) fn sprite_prep_cutscene_agahnim(&mut self, k: usize) {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x4000 != 0 {
            self.sprite_slot_mut(k).set_state(0);
        } else {
            self.cutscene_agahnim_spawn_zelda_on_altar(k);
            self.sprite_slot_mut(k).increment_ignore_projectile();
        }
    }

    pub(super) fn cutscene_agahnim_spawn_zelda_on_altar(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_x_low(8);
        self.sprite_slot_mut(k).add_y_low(6);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc1, &mut info);
        let j = j as usize;
        self.sprite_slot_mut(j).set_a(1);
        self.sprite_slot_mut(j).set_ignore_projectile(1);
        self.sprite_set_spawned_coordinates(j, &info);
        self.sprite_slot_mut(j)
            .set_y_low((info.r2_y as u8).wrapping_add(40));
        self.sprite_slot_mut(j).set_flags2(0);
        self.sprite_slot_mut(j).set_oam_flags(12);
    }

    pub(super) fn sprite_prep_vitreous(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_slot_mut(k).subtract_y_low(16);
        self.vitreous_spawn_smaller_eyes(k);
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_raven(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [0x81, 0x88];
        const HEALTH: [u8; 2] = [4, 8];
        const FLAGS5: [u8; 2] = [6, 2];
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_mut(k).set_bump_damage(BUMP_DAMAGE[j]);
        self.sprite_slot_mut(k).set_health(HEALTH[j]);
        self.sprite_slot_mut(k).set_flags5(FLAGS5[j]);
        self.sprite_prep_vulture(k);
    }

    pub(super) fn sprite_prep_vulture(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_z(0);
        let a = (self.sprite_slot_view(k).x_low() & 16) >> 4;
        self.sprite_slot_mut(k).set_a(a);
        self.sprite_slot_mut(k).set_subtype(254);
    }

    pub(super) fn sprite_prep_poe(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_z(12);
        self.sprite_slot_mut(k).set_subtype(254);
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
            self.sprite_slot_mut(k).increment_ignore_projectile();
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
        self.sprite_slot_mut(k).set_state(0);
    }

    pub(super) fn sprite_prep_snitches(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_direction(2);
        self.sprite_slot_mut(k).set_head_direction(2);
        self.sprite_slot_mut(k).increment_ignore_projectile();
        let x_low = self.sprite_slot_view(k).x_low();
        let x_high = self.sprite_slot_view(k).x_high();
        self.sprite_slot_mut(k).set_a(x_low);
        self.sprite_slot_mut(k).set_b(x_high);
        self.sprite_slot_mut(k).set_x_velocity((-9i8) as u8);
    }

    pub(super) fn sprite_prep_running_man(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_head_direction(2);
        self.sprite_slot_mut(k).set_direction(2);
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_arrow_game_bounce(&mut self, k: usize) {
        const X: [u8; 8] = [0, 0x40, 0x80, 0xc0, 0x30, 0x60, 0x90, 0xc0];
        const Y: [u8; 8] = [0, 0x4f, 0x4f, 0x4f, 0x5a, 0x5a, 0x5a, 0x5a];
        const A: [u8; 8] = [0, 1, 1, 1, 2, 2, 2, 2];
        const LOCAL_X_VELOCITIES: [i8; 2] = [-8, 12];
        const FLAGS4: [u8; 2] = [0x1c, 0x15];

        self.archery_game_mut().clear_hit_counter();
        self.sprite_slot_mut(k).subtract_y_low(9);
        let link_x_high = (self.game_state.player.follower_link.x() >> 8) as u8;
        let link_y_high = (self.game_state.player.follower_link.y() >> 8) as u8;
        let link_floor = self.game_state.player.follower_link.lower_level_state();
        for i in (1..=7).rev() {
            self.sprite_slot_mut(i).set_sprite_type(0x65);
            self.sprite_slot_mut(i).set_state(9);
            self.sprite_prep_load_properties(i);
            self.sprite_slot_mut(i).set_x_high(link_x_high);
            self.sprite_slot_mut(i).set_x_low(X[i]);
            self.sprite_slot_mut(i).set_y_high(link_y_high);
            self.sprite_slot_mut(i).set_y_low(Y[i]);
            self.sprite_slot_mut(i).set_a(A[i]);
            let j = (A[i] - 1) as usize;
            self.sprite_slot_mut(i).set_graphics(j as u8);
            self.sprite_slot_mut(i)
                .set_x_velocity(LOCAL_X_VELOCITIES[j] as u8);
            self.sprite_slot_mut(i).set_flags4(FLAGS4[j]);
            self.sprite_slot_mut(i).set_oam_flags(13);
            self.sprite_slot_mut(i).set_floor(link_floor);
            let subtype2 = self.get_random_number();
            self.sprite_slot_mut(i).set_subtype2(subtype2);
        }
        self.sprite_slot_mut(k).increment_ignore_projectile();
        let arrows = self.game_state.inventory.player_resources.arrows();
        self.sprite_slot_mut(k).set_subtype(arrows);
    }

    pub(super) fn sprite_prep_mushroom(&mut self, k: usize) {
        if self.game_state.inventory.items.mushroom() >= 2 {
            self.sprite_slot_mut(k).set_state(0);
        } else {
            self.sprite_slot_mut(k).set_graphics(0);
            self.sprite_slot_mut(k).or_oam_flags(8);
            self.sprite_slot_mut(k).increment_ignore_projectile();
        }
    }

    pub(super) fn sprite_prep_potion_shop(&mut self, k: usize) {
        self.magic_shop_assistant_spawn_powder(k);
        self.magic_shop_assistant_spawn_green_cauldron(k);
        self.magic_shop_assistant_spawn_blue_cauldron(k);
        self.magic_shop_assistant_spawn_red_cauldron(k);
        self.sprite_slot_mut(k).increment_ignore_projectile();
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
        self.sprite_slot_mut(j).set_subtype2(subtype);
        self.sprite_set_x(j, info.r0_x.wrapping_add(x_off as u16));
        self.sprite_set_y(j, info.r2_y.wrapping_add(y_off as u16));
        self.sprite_slot_mut(j).set_flags4(3);
        self.sprite_slot_mut(j).or_deflection_bits(0x20);
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
        self.sprite_slot_mut(k).set_z(16);
        self.sprite_slot_mut(k).set_subtype(254);
    }

    pub(super) fn sprite_prep_bomb_shoppe(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xb5, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_x(j, info.r0_x.wrapping_sub(24));
            self.sprite_set_y(j, info.r2_y.wrapping_sub(24));
            self.sprite_slot_mut(j).set_subtype2(1);
            self.sprite_slot_mut(j).set_ignore_projectile(1);
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
                self.sprite_slot_mut(j).set_subtype2(2);
                self.sprite_slot_mut(j).set_ignore_projectile(2);
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
            self.sprite_slot_mut(j).set_subtype2(3);
            self.sprite_slot_mut(j).set_ignore_projectile(3);
            self.sprite_slot_mut(j).set_z(4);
            self.sprite_slot_mut(j).set_z_velocity((-12i8) as u8);
            self.sprite_slot_mut(j).set_delay_main(23);
            self.sprite_slot_mut(j).and_flags3(!0x11u8);
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
        self.sprite_slot_mut(k).set_delay_main(0);
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
        self.sprite_slot_mut(k).set_flags4(0);
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
            self.sprite_slot_mut(k).set_graphics(graphics);
        } else {
            const LOCAL_GRAPHICS: [u8; 4] = [3, 4, 3, 2];
            let idx = if self.sprite_slot_view(k).ai_state() != 0 {
                ((self.game_state.frame.frame_counter >> 5) & 3) as usize
            } else {
                0
            };
            self.sprite_slot_mut(k).set_graphics(LOCAL_GRAPHICS[idx]);
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_slot_mut(k).set_flags4(10);
                if self.sprite_check_damage_to_link_same_layer(k)
                    && self.game_state.player.follower_link.filtered_joypad_l() & 0x80 != 0
                {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.archery_game_guy_show_msg(k, 0x85);
                }
            }
            1 | 3 => {
                if self.multiselect_choice().value() == 0
                    && self.game_state.inventory.player_resources.rupees_goal() >= 20
                {
                    self.sprite_slot_mut(k).set_head_direction(0);
                    self.archery_game_mut().clear_hit_counter();
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.archery_game_guy_show_msg(k, 0x86);
                } else {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.archery_game_guy_show_msg(k, 0x87);
                }
            }
            2 => self.archery_game_host_proctor_game(k),
            _ => {}
        }
    }

    pub(super) fn archery_game_host_proctor_game(&mut self, k: usize) {
        const NUM_SPR: [u8; 6] = [5, 4, 3, 2, 1, 0];
        const X: [i8; 18] = [
            0, 0, 0, 0, 48, 48, 48, 48, 8, 8, 16, 16, 24, 24, 32, 32, 40, 40,
        ];
        const Y: [i8; 18] = [-8, 0, 8, 16, -8, 0, 8, 16, 0, 8, 0, 8, 0, 8, 0, 8, 0, 8];
        const CHAR: [u8; 18] = [
            0x2b, 0x3b, 0x3b, 0x2b, 0x2b, 0x3b, 0x3b, 0x2b, 0x63, 0x73, 0x63, 0x73, 0x63, 0x73,
            0x63, 0x73, 0x63, 0x73,
        ];
        const FLAGS: [u8; 18] = [
            0x33, 0x33, 0xb3, 0xb3, 0x73, 0x73, 0xf3, 0xf3, 0x32, 0x32, 0x32, 0x32, 0x32, 0x32,
            0x32, 0x32, 0x32, 0x32,
        ];

        if self.sprite_slot_view(k).head_direction() == 0 {
            self.archery_game_mut().set_arrows_left(5);
            self.sprite_initialize_secondary_item_minigame(2);
            self.sprite_slot_mut(k).set_delay_aux1(39);
            let rupees = self.game_state.inventory.player_resources.rupees_goal();
            self.player_resources_mut()
                .set_rupees_goal(rupees.wrapping_sub(20));
            self.sprite_slot_mut(k).increment_head_direction();
        }

        self.oam_allocate_from_region_a(0x34);
        let Some((info_x, info_y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let count = if self.sprite_slot_view(k).delay_aux1() != 0 {
            NUM_SPR[(self.sprite_slot_view(k).delay_aux1() >> 3) as usize]
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
                    .wrapping_add(X[idx] as i16 as u16)
                    .wrapping_add(1) as u8,
                info_y
                    .wrapping_sub(48)
                    .wrapping_add(Y[idx] as i16 as u16)
                    .wrapping_add(1) as u8,
                CHAR[idx],
                FLAGS[idx],
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
        self.sprite_slot_mut(k).set_flags4(0x0a);
        if self.sprite_check_damage_to_link_same_layer(k)
            && self.game_state.player.follower_link.filtered_joypad_l() & 0x80 != 0
        {
            self.archery_game_guy_show_msg(k, 0x88);
            self.sprite_slot_mut(k).set_ai_state(3);
        }
    }

    pub(super) fn sprite_good_or_bad_archery_target(&mut self, k: usize) {
        const CASH_PRIZE: [u8; 10] = [4, 8, 16, 32, 64, 99, 99, 99, 99, 99];
        if self.sprite_slot_view(k).a() == 1 {
            if self.sprite_slot_view(k).g() >= 5 {
                self.sprite_slot_mut(k).set_b(6);
            }
            self.sprite_slot_mut(k).and_flags2(!0x1f);
            let j = if self.sprite_slot_view(k).delay_aux2() != 0 {
                self.sprite_slot_view(k).delay_aux2()
            } else {
                self.sprite_slot_view(k).subtype2() >> 3
            };
            self.sprite_slot_mut(k)
                .masked_or_oam_flags(!0x40, (j & 4) << 4);
            self.sprite_workspace_mut().subtract_current_sprite_y_low(3);
            self.sprite_draw_single_large(k);
            if self.sprite_slot_view(k).delay_aux2() != 0 {
                if self.sprite_slot_view(k).delay_aux2() == 96
                    && self.game_state.frame.submodule == 0
                {
                    self.sprite_slot_mut(0).set_delay_main(112);
                    let prize =
                        CASH_PRIZE[self.sprite_slot_view(k).b().wrapping_sub(1) as usize] as u16;
                    let rupees = self
                        .game_state
                        .inventory
                        .player_resources
                        .rupees_goal()
                        .wrapping_add(prize);
                    self.player_resources_mut().set_rupees_goal(rupees);
                }
                self.sprite_slot_mut(k).or_flags2(5);
                self.archery_game_draw_prize(k);
            }
        } else {
            self.sprite_slot_mut(k).and_flags2(!0x1f);
            self.sprite_workspace_mut().add_current_sprite_y_low(3);
            self.sprite_draw_single_large(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        if self.sprite_slot_view(k).delay_aux3() == 1 {
            self.set_sound_effect_1(0x3c);
        }
        self.sprite_slot_mut(k).increment_subtype2();
        self.sprite_move_x(k);
        if self.sprite_slot_view(k).delay_aux1() == 0 {
            let delay_main = self.sprite_slot_view(k).delay_main();
            self.sprite_slot_mut(k).set_ignore_projectile(delay_main);
            if self.sprite_slot_view(k).delay_main() == 0 {
                if self.sprite_check_tile_collision(k) != 0 {
                    self.sprite_slot_mut(k).set_delay_main(16);
                    self.sprite_slot_mut(k).set_delay_aux2(0);
                }
            } else if self.sprite_slot_view(k).delay_main() == 1 {
                const TARGET_X: [u8; 2] = [(-24i8) as u8, 8];
                let graphics = self.sprite_slot_view(k).graphics() as usize;
                let link_x_high = (self.game_state.player.follower_link.x() >> 8) as u8;
                self.sprite_slot_mut(k).set_x_low(TARGET_X[graphics]);
                self.sprite_slot_mut(k).set_x_high(link_x_high);
                self.sprite_slot_mut(k).set_delay_aux1(32);
                self.sprite_slot_mut(k).set_g(0);
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
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn spawn_bully(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xb9, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_mut(j).set_subtype2(2);
            self.sprite_slot_mut(j).set_head_direction(k as u8);
            self.sprite_slot_mut(j).set_ignore_projectile(1);
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
        const COORD: [i8; 4] = [-4, 0, 4, 8];
        if (self.game_state.frame.frame_counter & 3) != 0 {
            return;
        }
        let j = self.garnish_alloc_force() as usize;
        let value = 0x12;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(0x12);
        let x = self
            .sprite_get_x(k)
            .wrapping_add(COORD[usize::from(self.get_random_number() & 3)] as i16 as u16);
        let y = self
            .sprite_get_y(k)
            .wrapping_add(COORD[usize::from(self.get_random_number() & 3)] as i16 as u16);
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
            self.sprite_slot_mut(j).set_state(6);
            self.sprite_slot_mut(j).set_delay_main(15);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x14);
            self.sprite_slot_mut(j).set_floor(2);
        }
    }

    // void Sprite_MagicBat_SpawnLightning(int k) {  // 89aea8
    pub(super) fn sprite_magic_bat_spawn_lightning(&mut self, k: usize) {
        const LOCAL_X_VELOCITIES: [i8; 4] = [-8, -4, 4, 8];
        const ST2: [u8; 4] = [0, 0x11, 0x22, 0x33];

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
                self.sprite_slot_mut(j).set_z(0);
                self.sprite_slot_mut(j).set_y_velocity(24);
                self.sprite_slot_mut(j).set_head_direction(24);
                self.sprite_slot_mut(j).set_ignore_projectile(24);
                self.sprite_slot_mut(j).set_flags2(0x80);
                self.sprite_slot_mut(j).set_flags3(3);
                self.sprite_slot_mut(j).set_oam_flags(3);
                self.sprite_slot_mut(j).set_delay_main(32);
                self.sprite_slot_mut(j).set_graphics(2);
                let i = usize::from(self.sprite_slot_view(k).g());
                self.sprite_slot_mut(j)
                    .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j).set_subtype2(ST2[i]);
                self.sprite_slot_mut(j).set_floor(2);
                self.sprite_slot_mut(k).increment_g();
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
        let value = (self.get_random_number() & 31).wrapping_add(48);
        self.garnish_slot_view_mut(k).set_countdown(value);
    }

    pub(super) fn kholdstare_spawn_puff_cloud_garnish(&mut self, k: usize) {
        const XY: [i8; 8] = [-8, -6, -4, -2, 0, 2, 4, 6];
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
            .wrapping_add_signed(i16::from(XY[(self.get_random_number() & 7) as usize]));
        let y = self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_add_signed(i16::from(XY[(self.get_random_number() & 7) as usize]) + 16);
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
        const LOCAL_GRAPHICS: [u8; 4] = [4, 5, 6, 5];

        self.sprite_slot_mut(k).increment_subtype2();
        let i = ((self.sprite_slot_view(k).subtype2() >> 2) & 3) as usize;
        self.sprite_slot_mut(k).set_graphics(LOCAL_GRAPHICS[i]);
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
            self.sprite_slot_mut(j).set_state(3);
            self.sprite_slot_mut(j).set_delay_main(15);
            self.sprite_slot_mut(j).set_ai_state(0);
            self.sprite_slot_mut(j).set_flags2(3);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
        }
    }

    pub(super) fn catfish_regurgitate_medallion(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc0, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_mut(j).set_x_velocity(24);
            self.sprite_slot_mut(j).set_z_velocity(48);
            self.sprite_slot_mut(j).set_a(17);
            self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
            self.sprite_slot_mut(j).set_flags2(0x83);
            self.sprite_slot_mut(j).set_flags3(0x58);
            self.sprite_slot_mut(j).set_oam_flags(0x58 & 0x0f);
            self.DecodeAnimatedSpriteTile_variable(0x1c);
        }
    }

    pub(super) fn sprite_spawn_water_splash(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc0, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_mut(j_usize).set_a(0x80);
            self.sprite_slot_mut(j_usize).set_flags2(2);
            self.sprite_slot_mut(j_usize).set_ignore_projectile(2);
            self.sprite_slot_mut(j_usize).set_oam_flags(4);
            self.sprite_slot_mut(j_usize).set_delay_main(31);
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
            self.sprite_slot_mut(j_usize).set_state(3);
            self.sprite_slot_mut(j_usize).set_delay_main(15);
            self.sprite_slot_mut(j_usize).set_ai_state(0);
            self.sprite_slot_mut(j_usize).set_flags2(3);
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
            self.sprite_slot_mut(j_usize).set_subtype2(1);
        }
        j
    }

    pub(super) fn sprite_spawn_superficial_bomb_blast(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_slot_mut(j_usize).set_state(6);
            self.sprite_slot_mut(j_usize).set_delay_aux1(31);
            self.sprite_slot_mut(j_usize).set_c(3);
            self.sprite_slot_mut(j_usize).set_flags2(3);
            self.sprite_slot_mut(j_usize).set_oam_flags(4);
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
            self.sprite_slot_mut(j_usize).set_sprite_type(0x4a);
            self.sprite_slot_mut(j_usize).set_c(1);
            self.sprite_slot_mut(j_usize).set_delay_aux1(255);
            self.sprite_slot_mut(j_usize).set_flags3(0x18);
            self.sprite_slot_mut(j_usize).set_oam_flags(8);
            self.sprite_slot_mut(j_usize).set_health(0);
            self.sprite_slot_mut(j_usize).set_delay_aux1(80);
            self.sprite_slot_mut(j_usize).set_x_velocity(24);
            self.sprite_slot_mut(j_usize).set_z_velocity(48);
        }
        j
    }

    pub(super) fn spawn_boss_poof(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xce, &mut info);
        let j_usize = j as usize;
        self.sprite_set_x(j_usize, info.r0_x.wrapping_add(16));
        self.sprite_set_y(j_usize, info.r2_y.wrapping_add(40));
        self.sprite_slot_mut(j_usize).set_graphics(0x0f);
        self.sprite_slot_mut(j_usize).set_a(1);
        self.sprite_slot_mut(j_usize).set_delay_main(47);
        self.sprite_slot_mut(j_usize).set_flags2(9);
        self.sprite_slot_mut(j_usize).set_ignore_projectile(9);
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
        self.sprite_slot_mut(j_usize).masked_or_flags3(0xfe, 0x40);
        self.sprite_slot_mut(j_usize).set_oam_flags(6);
        self.sprite_slot_mut(j_usize).set_flags4(0x54);
        self.sprite_slot_mut(j_usize).set_e(0x54);
        self.sprite_slot_mut(j_usize).set_flags2(0x20);
        self.sprite_apply_speed_towards_link(j_usize, 0x20);
        self.sprite_slot_mut(j_usize).set_delay_main(20);
        self.sprite_slot_mut(j_usize).set_delay_aux1(16);
        self.sprite_slot_mut(j_usize).set_flags5(0);
        self.sprite_slot_mut(j_usize).set_deflection_bits(0x48);
        j
    }

    pub(super) fn sprite_spawn_fire_phlegm(&mut self, k: usize) -> i32 {
        const X: [i8; 4] = [16, -8, 4, 4];
        const Y: [i8; 4] = [-2, -2, 8, -20];
        const LOCAL_X_VELOCITIES: [i8; 4] = [48, -48, 0, 0];
        const LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 48, -48];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xa5, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_sfx_queue_sfx3_with_pan(k, 5);
            self.sprite_set_spawned_coordinates(j_usize, &info);
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_set_x(j_usize, info.r0_x.wrapping_add_signed(i16::from(X[i])));
            self.sprite_set_y(j_usize, info.r2_y.wrapping_add_signed(i16::from(Y[i])));
            self.sprite_slot_mut(j_usize)
                .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
            self.sprite_slot_mut(j_usize)
                .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
            self.sprite_slot_mut(j_usize).or_flags3(0x40);
            self.sprite_slot_mut(j_usize).set_deflection_bits(0x40);
            self.sprite_slot_mut(j_usize).set_flags2(0x21);
            self.sprite_slot_mut(j_usize).set_b(0x21);
            self.sprite_slot_mut(j_usize).set_oam_flags(2);
            self.sprite_slot_mut(j_usize).set_flags4(0x14);
            self.sprite_slot_mut(j_usize).set_ignore_projectile(20);
            self.sprite_slot_mut(j_usize).set_bump_damage(37);
            if self.game_state.inventory.items.shield_type() >= 3 {
                self.sprite_slot_mut(j_usize).set_flags5(0x20);
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
        self.sprite_slot_mut(j).set_graphics(2);
        let z_velocity = self.sprite_slot_view(k).z_velocity();
        self.sprite_slot_mut(j).set_z_velocity(z_velocity);
        self.sprite_slot_mut(j).set_subtype2(1);
        self.sprite_slot_mut(j).set_ai_state(2);
        self.sprite_slot_mut(j).set_delay_main(8);
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
        const X: [i8; 4] = [12, -12, 0, 0];
        const Y: [i8; 4] = [4, 4, 12, -12];
        const LOCAL_X_VELOCITIES: [i8; 4] = [44, -44, 0, 0];
        const LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 44, -44];

        let mut info = SpriteSpawnInfo::default();
        self.sprite_sfx_queue_sfx2_with_pan(k, 7);
        let j = self.sprite_spawn_dynamically(k, 0x0c, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(X[i])));
            self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(Y[i])));
            self.sprite_slot_mut(j)
                .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
            self.sprite_slot_mut(j)
                .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
        }
    }

    pub(super) fn moblin_materialize_spear(&mut self, k: usize) {
        const X: [i8; 4] = [11, -2, -3, 11];
        const Y: [i8; 4] = [-3, -3, 3, -11];
        const LOCAL_X_VELOCITIES: [i8; 4] = [32, -32, 0, 0];
        const LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 32, -32];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x1b, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_slot_mut(j).set_a(3);
            self.sprite_slot_mut(j).set_direction(i as u8);
            self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(X[i])));
            self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(Y[i])));
            self.sprite_slot_mut(j)
                .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
            self.sprite_slot_mut(j)
                .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
        }
    }

    pub(super) fn snitch_spawn_guard(&mut self, k: usize) {
        const X: [u16; 3] = [0x0120, 0x0340, 0x02e0];
        const Y: [u16; 3] = [0x0100, 0x03b0, 0x0160];

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
        self.sprite_set_x(j, X[i].wrapping_add(x_base));
        self.sprite_set_y(j, Y[i].wrapping_add(y_base));
        self.sprite_slot_mut(j).set_floor(0);
        self.sprite_slot_mut(j).set_health(4);
        self.sprite_slot_mut(j).set_deflection_bits(0x80);
        self.sprite_slot_mut(j).set_flags5(0x90);
        self.sprite_slot_mut(j).set_oam_flags(0x0b);
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
        const LOCAL_X_VELOCITIES: [i8; 4] = [16, -16, 0, 0];
        const LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 16, -16];

        let j = self.sprite_slot_view(k).direction() as usize;
        self.sprite_slot_mut(k)
            .set_x_velocity(LOCAL_X_VELOCITIES[j] as u8);
        self.sprite_slot_mut(k)
            .set_y_velocity(LOCAL_Y_VELOCITIES[j] as u8);
    }

    pub(super) fn kodongo_spawn_fire(&mut self, k: usize) {
        const X: [i8; 4] = [8, -8, 0, 0];
        const Y: [i8; 4] = [0, 0, 8, -8];
        const LOCAL_X_VELOCITIES: [i8; 4] = [24, -24, 0, 0];
        const LOCAL_Y_VELOCITIES: [i8; 4] = [0, 0, 24, -24];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0x87, &mut info, 13);
        if j >= 0 {
            let j = j as usize;
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(X[i])));
            self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(Y[i])));
            self.sprite_slot_mut(j)
                .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
            self.sprite_slot_mut(j)
                .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
            self.sprite_slot_mut(j).set_ignore_projectile(1);
        }
    }

    pub(super) fn create_six_blue_balls(&mut self, k: usize) {
        const LOCAL_X_VELOCITIES: [i8; 6] = [0, 24, 24, 0, -24, -24];
        const LOCAL_Y_VELOCITIES: [i8; 6] = [-32, -16, 16, 32, 16, -16];

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
                self.sprite_slot_mut(j).masked_or_flags3(!1, 0x40);
                self.sprite_slot_mut(j).set_oam_flags(4);
                self.sprite_slot_mut(j).set_delay_aux1(4);
                self.sprite_slot_mut(j).set_flags4(20);
                self.sprite_slot_mut(j).set_c(20);
                self.sprite_slot_mut(j).set_e(20);
                self.sprite_slot_mut(j)
                    .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j)
                    .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
            }

            self.temp_counter_mut().decrement();
            if sign8(self.game_state.scratch_counter.value()) {
                break;
            }
        }
        self.temp_counter_mut().set(0);
    }

    pub(super) fn lanmola_spawn_shrapnel(&mut self, k: usize) {
        const LOCAL_Y_VELOCITIES: [i8; 8] = [28, -28, 28, -28, 0, 36, 0, -36];
        const LOCAL_X_VELOCITIES: [i8; 8] = [-28, -28, 28, 28, -36, 0, 36, 0];

        let shrapnel_countdown = if self
            .sprite_slot_view(0)
            .state()
            .wrapping_add(self.sprite_slot_view(1).state())
            .wrapping_add(self.sprite_slot_view(2).state())
            < 10
        {
            7
        } else {
            3
        };
        self.temp_counter_mut().set(shrapnel_countdown);

        loop {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xc2, &mut info);
            if j >= 0 {
                let j = j as usize;
                let i = self.game_state.scratch_counter.value() as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_slot_mut(j)
                    .set_x_low((info.r0_x as u8).wrapping_add(4));
                self.sprite_slot_mut(j)
                    .set_y_low((info.r2_y as u8).wrapping_add(4));
                self.sprite_slot_mut(j).set_ignore_projectile(1);
                self.sprite_slot_mut(j).set_bump_damage(1);
                self.sprite_slot_mut(j).set_flags4(1);
                self.sprite_slot_mut(j).set_z(0);
                self.sprite_slot_mut(j).set_flags2(0x20);
                self.sprite_slot_mut(j)
                    .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j)
                    .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
                let graphics = self.get_random_number() & 1;
                self.sprite_slot_mut(j).set_graphics(graphics);
            }

            self.temp_counter_mut().decrement();
            if sign8(self.game_state.scratch_counter.value()) {
                break;
            }
        }
    }

    pub(super) fn octoballoon_form_babby(&mut self, k: usize) {
        const LOCAL_X_VELOCITIES: [i8; 6] = [16, 11, -11, -16, -11, 11];
        const LOCAL_Y_VELOCITIES: [i8; 6] = [0, 11, 11, 0, -11, -11];

        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
        for i in (0..=5).rev() {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x10, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_slot_mut(j)
                    .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j)
                    .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j).set_z_velocity(48);
                self.sprite_slot_mut(j).set_subtype2(255);
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
            self.sprite_slot_mut(k).xor_x_velocity(255);
            self.sprite_slot_mut(k).xor_y_velocity(255);
            if self.sprite_slot_view(k).e() != 0 {
                self.ball_guy_play_bounce_noise(k);
            }
            self.sprite_slot_mut(k).set_delay_aux4(64);
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
            self.sprite_slot_mut(k).xor_x_velocity(255);
            self.sprite_slot_mut(k).xor_y_velocity(255);
            self.sprite_slot_mut(k).set_delay_aux4(64);
        }
    }

    pub(super) fn rupee_pull_spawn_prize(&mut self, k: usize) {
        const LOCAL_X_VELOCITIES: [i8; 4] = [-18, -12, 12, 18];
        const LOCAL_Y_VELOCITIES: [i8; 4] = [16, 24, 24, 16];
        const TYPE: [u8; 3] = [0xd9, 0xda, 0xdb];

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
                let what = TYPE[self.game_state.sprites.workspace.shared_scratch_a() as usize];
                let j = self.sprite_spawn_dynamically(k, what, &mut info);
                if j < 0 {
                    break;
                }

                let j = j as usize;
                let i = self.game_state.scratch_counter.value() as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_slot_mut(j)
                    .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j)
                    .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j).set_stunned(255);
                self.sprite_slot_mut(j).set_delay_aux4(32);
                let value = 32;
                self.sprite_slot_mut(j).set_delay_aux3(value);
                self.sprite_slot_mut(j).set_z_velocity(32);

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
            self.sprite_slot_mut(j_usize).set_sprite_type(0x4a);
            self.sprite_slot_mut(j_usize).set_c(1);
            self.sprite_slot_mut(j_usize).set_delay_aux1(255);
            self.sprite_slot_mut(j_usize).set_flags3(0x18);
            self.sprite_slot_mut(j_usize).set_oam_flags(8);
            self.sprite_slot_mut(j_usize).set_health(0);
        }
    }

    pub(super) fn talking_tree_spawn_bomb(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_slot_mut(j_usize).set_sprite_type(0x4a);
            self.sprite_slot_mut(j_usize).set_c(1);
            self.sprite_slot_mut(j_usize).set_delay_aux1(255);
            self.sprite_slot_mut(j_usize).set_flags3(0x18);
            self.sprite_slot_mut(j_usize).set_oam_flags(8);
            self.sprite_slot_mut(j_usize).set_health(0);
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_mut(j_usize).set_delay_aux1(64);
            self.sprite_slot_mut(j_usize).set_y_velocity(24);
            self.sprite_slot_mut(j_usize).set_z_velocity(18);
        }
    }

    pub(super) fn pirogusu_spawn_splash(&mut self, k: usize) {
        const SPLASH_JITTER_OFFSETS: [u8; 4] = [3, 4, 5, 4];
        if (k as u8 ^ self.game_state.frame.frame_counter) & 3 != 0 {
            return;
        }
        let x = SPLASH_JITTER_OFFSETS[(self.get_random_number() & 3) as usize];
        let y = SPLASH_JITTER_OFFSETS[(self.get_random_number() & 3) as usize];
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
        const SPAWN_XY: [i8; 6] = [12, -4, 4, 4, 12, -4];
        const SPAWN_XYVEL: [i8; 6] = [112, -112, 0, 0, 112, -112];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x95, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.sprite_slot_view(k).direction() as usize;
            self.sprite_slot_mut(j).set_graphics((i as u8 & 2) >> 1);
            self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(SPAWN_XY[i])));
            self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(SPAWN_XY[i + 2])));
            self.sprite_slot_mut(j).set_x_velocity(SPAWN_XYVEL[i] as u8);
            self.sprite_slot_mut(j)
                .set_y_velocity(SPAWN_XYVEL[i + 2] as u8);
            self.sprite_slot_mut(j).set_flags2(0x20);
            self.sprite_slot_mut(j).set_a(0x20);
            self.sprite_slot_mut(j).set_oam_flags(5);
            self.sprite_slot_mut(j).set_deflection_bits(0x48);
            self.sprite_slot_mut(j).set_ignore_projectile(0x48);
            self.sprite_slot_mut(j).set_delay_main(5);
            if self.game_state.inventory.items.shield_type() == 3 {
                self.sprite_slot_mut(j).set_flags5(32);
            }
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
        }
    }

    pub(super) fn get_position_relative_to_the_great_overlord_ganon(&mut self, k: usize) {
        const X: [i8; 2] = [20, -18];
        const Y: [i8; 2] = [-20, -20];

        let j = self.sprite_slot_view(0).direction() as usize;
        let home = self.armos_knight_home_position(k);
        let x = home.x();
        let y = home.y();
        self.sprite_set_x(k, x.wrapping_add_signed(i16::from(X[j])));
        self.sprite_set_y(k, y.wrapping_add_signed(i16::from(Y[j])));
    }

    pub(super) fn sasha_idle(&mut self, k: usize) {
        let inventory = &self.game_state.inventory.items;
        let resources = &self.game_state.inventory.player_resources;
        if resources.pendant_flags() & 4 == 0 {
            if self.sprite_show_solicited_message(k, 0x32) & 0x100 != 0 {
                self.sprite_slot_mut(k).set_ai_state(1);
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
                self.sprite_slot_mut(k).set_ai_state(2);
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
        self.sprite_slot_mut(k).set_graphics(graphics);
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
        self.sprite_slot_mut(j).set_direction(direction);
        self.sprite_slot_mut(j).set_head_direction(direction);
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.sprite_set_y(j, y.wrapping_add(2));
        self.sprite_set_x(j, x.wrapping_add(2));
        self.sprite_slot_mut(j).set_floor(floor);
        self.sprite_slot_mut(j).set_ignore_projectile(1);
        self.sprite_slot_mut(j).set_subtype2(1);
        self.old_man_enable_cutscene();
        self.follower_state_mut().set_indicator(0);
        self.follower_link_state_mut().set_speed_setting(0);
    }

    pub(super) fn old_man_enable_cutscene(&mut self) {
        self.follower_link_state_mut().immobilize();
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
    }

    pub(super) fn sprite_ad_old_man(&mut self, k: usize) {
        const OLD_MOUNTAIN_MAN_MSGS: [u16; 3] = [0x9e, 0x9f, 0xa0];

        self.old_mountain_man_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.sprite_slot_view(k).subtype2() {
            0 => match self.sprite_slot_view(k).ai_state() {
                0 => {
                    self.sprite_track_body_to_head(k);
                    let dir = self.sprite_direction_to_face_link(k, None) ^ 3;
                    self.sprite_slot_mut(k).set_head_direction(dir);
                    let j = self.sprite_show_message_on_contact(k, 0x9c);
                    if j & 0x100 != 0 {
                        self.sprite_slot_mut(k).set_direction(j as u8);
                        self.sprite_slot_mut(k).set_head_direction(j as u8);
                        self.sprite_slot_mut(k).set_ai_state(1);
                    }
                }
                1 => {
                    self.follower_state_mut().set_indicator(4);
                    self.sprite_become_follower(k);
                    self.save_progress_mut().set_which_starting_point(5);
                    self.sprite_slot_mut(k).set_state(0);
                    self.cache_camera_properties();
                }
                _ => {}
            },
            1 => {
                self.sprite_move_xy(k);
                match self.sprite_slot_view(k).ai_state() {
                    0 => {
                        self.sprite_slot_mut(k).increment_ai_state();
                        self.follower_link_state_mut().set_item_receipt_method(0);
                        self.link_receive_item(0x1a, 0);
                        self.save_progress_mut().set_which_starting_point(1);
                        self.old_man_enable_cutscene();
                        self.sprite_slot_mut(k).set_delay_main(48);
                        self.sprite_slot_mut(k).set_x_velocity(8);
                        self.sprite_slot_mut(k).set_y_velocity(4);
                        self.sprite_slot_mut(k).set_direction(3);
                        self.sprite_slot_mut(k).set_head_direction(3);
                    }
                    1 => {
                        self.old_man_enable_cutscene();
                        if self.sprite_slot_view(k).delay_main() == 0 {
                            self.sprite_slot_mut(k).increment_ai_state();
                        }
                        let graphics = ((k as u8) ^ self.game_state.frame.frame_counter) >> 3 & 1;
                        self.sprite_slot_mut(k).set_graphics(graphics);
                    }
                    2 => {
                        self.sprite_slot_mut(k).set_head_direction(0);
                        self.sprite_slot_mut(k).set_direction(0);
                        let j = self
                            .game_state
                            .sprites
                            .garnish_runtime
                            .active_overlord_index() as usize;
                        let overlord = self.overlord_slot_view(j);
                        let x = overlord.x();
                        let y = overlord.y();
                        if y >= self.sprite_get_y(k) {
                            self.sprite_slot_mut(k).increment_ai_state();
                            self.sprite_slot_mut(k).set_y_velocity(0);
                            self.sprite_slot_mut(k).set_x_velocity(0);
                        } else {
                            let pt = self.sprite_project_speed_towards_location(k, x, y, 8);
                            self.sprite_slot_mut(k).set_y_velocity(pt.y);
                            self.sprite_slot_mut(k).set_x_velocity(pt.x);
                            let graphics =
                                ((k as u8) ^ self.game_state.frame.frame_counter) >> 3 & 1;
                            self.sprite_slot_mut(k).set_graphics(graphics);
                            self.old_man_enable_cutscene();
                        }
                    }
                    3 => {
                        self.sprite_slot_mut(k).set_state(0);
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
                    self.sprite_slot_mut(k).set_ai_state(0);
                }
                let j = if self.game_state.inventory.save_progress.progress_indicator() >= 3 {
                    2
                } else {
                    self.game_state.inventory.items.moon_pearl() as usize
                };
                if self.sprite_show_solicited_message(k, OLD_MOUNTAIN_MAN_MSGS[j]) & 0x100 != 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
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
                self.sprite_slot_mut(k).subtract_x_low(16);
                self.sprite_get_16bit_coords_for_prep(k);
                self.sprite_slot_mut(k).set_x_velocity(1);
                self.sprite_slot_mut(k).set_y_velocity(1);
                if self.sprite_check_tile_collision(k) == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    if self.game_state.sprites.follower_runtime.indicator() != 0 {
                        self.sprite_slot_mut(k).set_ai_state(5);
                    }
                }
                self.sprite_slot_mut(k).set_x_low(bak);
            }
            1 => {
                self.follower_state_mut().set_indicator(9);
                self.follower_state_mut().set_appearance_none_flag(0);
                self.load_follower_graphics();
                self.follower_initialize();
                self.start_shared_message_timer(0x40);
                self.sprite_slot_mut(k).set_state(0);
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
                    self.sprite_slot_mut(k).set_ai_state(3);
                }
            }
            3 => {
                if self.multiselect_choice().value() == 0 {
                    if self.game_state.sprites.follower_runtime.dropped() != 0 {
                        self.sprite_show_message_unconditional(0x10c);
                        self.sprite_slot_mut(k).set_ai_state(2);
                    } else {
                        self.follower_link_state_mut().set_item_receipt_method(0);
                        self.link_receive_item(0x16, 0);
                        self.save_progress_mut().or_progress_indicator_3(0x10);
                        self.sprite_slot_mut(k).set_ai_state(4);
                        self.follower_state_mut().set_indicator(0);
                    }
                } else {
                    self.sprite_show_message_unconditional(0x10a);
                    self.sprite_slot_mut(k).set_ai_state(2);
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
                        self.sprite_slot_mut(k).increment_ai_state();
                        self.sprite_slot_mut(k).set_a(20);
                        self.follower_link_state_mut().immobilize();
                        self.sprite_slot_mut(k).or_oam_flags(32);
                        return;
                    }
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).decrement_a();
                    let delay_main = self.sprite_slot_view(k).a();
                    self.sprite_slot_mut(k).set_delay_main(delay_main);
                    if self.sprite_slot_view(k).delay_main() != 1 {
                        const RISING_UP_X_ACCEL: [i8; 2] = [-8, 7];
                        let z_velocity = self.sprite_slot_view(k).delay_main() >> 2;
                        self.sprite_slot_mut(k).set_z_velocity(z_velocity);
                        let idx = (self.sprite_slot_view(k).a() & 1) as usize;
                        self.sprite_slot_mut(k)
                            .add_x_velocity(RISING_UP_X_ACCEL[idx] as u8);
                        self.sprite_slot_mut(k).xor_graphics(1);
                    } else {
                        self.sprite_show_message_unconditional(0x110);
                        self.sprite_slot_mut(k).increment_ai_state();
                        self.sprite_slot_mut(k).set_graphics(0);
                        self.sprite_slot_mut(k).set_z_velocity(0);
                        self.sprite_slot_mut(k).set_x_velocity(0);
                        self.sprite_slot_mut(k).set_delay_main(255);
                    }
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_delay_aux1(64);
                }
                const OAM_FLAGS: [u8; 8] = [0x0a, 4, 2, 4, 2, 0x0a, 4, 2];
                let idx = ((self.sprite_slot_view(k).delay_main() >> 1) & 7) as usize;
                self.sprite_slot_mut(k)
                    .masked_or_oam_flags(!0x0e, OAM_FLAGS[idx]);
                if self.sprite_slot_view(k).delay_main() == 240 {
                    self.sprite_magic_bat_spawn_lightning(k);
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    self.sprite_show_message_unconditional(0x111);
                    self.Palette_Restore_BG_And_HUD();
                    self.increment_cgram_update_flag();
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.player_resources_mut().set_magic_consumption_level(1);
                    self.hud_refresh_icon();
                } else if self.sprite_slot_view(k).delay_aux1() == 0x10 {
                    self.attract_scene_mut().set_intro_palette_flash_count(0x10);
                }
            }
            4 => {
                self.sprite_spawn_dummy_death_animation(k);
                self.sprite_slot_mut(k).set_state(0);
                self.follower_link_state_mut().clear_immobilized();
            }
            _ => {}
        }
    }

    pub(super) fn sprite_72_fairy_pond(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() != 0 {
            self.sprite_slot_mut(k).decrement_c();
            if self.sprite_slot_view(k).c() == 0 {
                self.sprite_slot_mut(k).set_state(0);
            }
            let graphics = self.sprite_slot_view(k).c() >> 3;
            self.sprite_slot_mut(k).set_graphics(graphics);
            self.oam_allocate_from_region_c(4);
            self.sprite_draw_single_small(k);
            return;
        }
        if self.sprite_slot_view(k).b() != 0 {
            self.faerie_queen_draw(k);
            let graphics = self.game_state.frame.frame_counter >> 4 & 1;
            self.sprite_slot_mut(k).set_graphics(graphics);
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
                self.sprite_slot_mut(j).set_c(31);
                self.sprite_slot_mut(j).set_a(31);
                self.sprite_slot_mut(j).set_flags2(0);
                self.sprite_slot_mut(j).set_flags3(0x48);
                self.sprite_slot_mut(j).set_oam_flags(0x48 & 0x0f);
                self.sprite_slot_mut(j).set_b(1);
            }
            return;
        }
        let _ = self.sprite_prep_oam_coord_or_double_ret(k);
        self.sprite_wish_pond2(k);
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
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.link_reset_properties_a();
                    self.follower_link_state_mut().set_facing(0);
                    self.sprite_slot_mut(k).set_head_direction(0);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 {
                    self.sprite_show_message_unconditional(0x8a);
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.follower_link_state_mut().immobilize();
                } else {
                    self.sprite_show_message_unconditional(0x14b);
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.sprite_slot_mut(k).set_delay_main(255);
                }
            }
            2 => {
                self.sprite_slot_mut(k).set_ai_state(3);
                let j = self.multiselect_choice().value() as usize;
                self.sprite_slot_mut(k).set_c(j as u8);
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
                self.sprite_slot_mut(k).set_graphics(t);
                self.sprite_slot_mut(k).set_direction(item);
                self.sprite_slot_mut(k).set_delay_main(255);
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
                        self.sprite_slot_mut(j).set_b(1);
                        self.Palette_AssertTranslucencySwap();
                        self.PaletteFilter_WishPonds();
                        self.sprite_slot_mut(k).set_e(j as u8);
                        self.sprite_slot_mut(k).set_ai_state(4);
                        self.sprite_slot_mut(k).set_delay_main(255);
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
                        self.sprite_slot_mut(k).set_ai_state(5);
                    }
                }
            }
            5 => {
                let ai_state = if self.multiselect_choice().value() == 0 {
                    6
                } else {
                    11
                };
                self.sprite_slot_mut(k).set_ai_state(ai_state);
            }
            6 => {
                self.sprite_slot_mut(k).set_ai_state(7);
                if self.game_state.inventory.save_progress.dark_world_state() == 0 {
                    match self.sprite_slot_view(k).graphics() {
                        12 => {
                            self.sprite_slot_mut(k).set_graphics(42);
                            self.sprite_slot_mut(k).set_head_direction(1);
                        }
                        4 => {
                            self.sprite_slot_mut(k).set_graphics(5);
                            self.sprite_slot_mut(k).set_head_direction(2);
                        }
                        22 => {
                            self.sprite_slot_mut(k).set_graphics(44);
                            self.sprite_slot_mut(k).set_head_direction(3);
                        }
                        _ => {
                            self.sprite_show_message_unconditional(0x14d);
                            return;
                        }
                    }
                } else {
                    match self.sprite_slot_view(k).graphics() {
                        58 => {
                            self.sprite_slot_mut(k).set_graphics(59);
                            self.sprite_slot_mut(k).set_head_direction(4);
                            self.sprite_show_message_unconditional(0x14f);
                            return;
                        }
                        2 => {
                            self.sprite_slot_mut(k).set_graphics(3);
                            self.sprite_slot_mut(k).set_head_direction(5);
                        }
                        22 => {
                            self.sprite_slot_mut(k).set_graphics(44);
                            self.sprite_slot_mut(k).set_head_direction(3);
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
                self.sprite_slot_mut(k).set_ai_state(8);
            }
            8 => {
                if self.game_state.frame.frame_counter & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.game_state.display.palette_filter.countdown() == 30 {
                        let j = self.sprite_slot_view(k).e() as usize;
                        self.sprite_slot_mut(j).set_state(0);
                    } else if self.game_state.display.palette_filter.countdown() == 0 {
                        self.sprite_slot_mut(k).set_ai_state(9);
                    }
                }
            }
            9 => {
                self.PaletteFilter_RestoreSP5F();
                self.Palette_RevertTranslucencySwap();
                self.follower_link_state_mut().set_item_receipt_method(2);
                self.link_receive_item(self.sprite_slot_view(k).graphics(), 0);
                self.sprite_slot_mut(k).set_ai_state(10);
            }
            10 => {
                const MSGS: [u16; 5] = [0x8f, 0x90, 0x92, 0x91, 0x93];
                let head = self.sprite_slot_view(k).head_direction();
                if head != 0 {
                    self.sprite_show_message_unconditional(MSGS[head.wrapping_sub(1) as usize]);
                }
                self.sprite_slot_mut(k).set_ai_state(0);
                self.sprite_slot_mut(k).set_delay_main(255);
            }
            11 => {
                self.sprite_show_message_unconditional(0x8d);
                self.sprite_slot_mut(k).set_ai_state(12);
            }
            12 => {
                let ai_state = if self.multiselect_choice().value() == 0 {
                    13
                } else {
                    6
                };
                self.sprite_slot_mut(k).set_ai_state(ai_state);
            }
            13 => {
                self.sprite_show_message_unconditional(0x8e);
                self.sprite_slot_mut(k).set_ai_state(7);
            }
            _ => {}
        }
    }

    pub(super) fn sprite_happiness_pond(&mut self, k: usize) {
        const COST: [u8; 4] = [5, 20, 25, 50];
        const COST_HEX: [u8; 4] = [5, 0x20, 0x25, 0x50];
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.follower_link_state_mut().clear_immobilized();
                if self.sprite_slot_view(k).delay_main() != 0 || self.sprite_check_if_link_is_busy()
                {
                    return;
                }
                if self.sprite_show_message_on_contact(k, 0x89) & 0x100 != 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
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
                    self.sprite_slot_mut(k).set_graphics(i * 2);
                    let cost_index = (i * 2) as usize;
                    self.dialogue_number_mut()
                        .set_packed_digits(COST_HEX[cost_index], COST_HEX[cost_index + 1]);
                    self.sprite_show_message_unconditional(0x14e);
                    self.sprite_slot_mut(k).set_ai_state(2);
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
                    .set_high_pair(COST_HEX[i as usize]);
                if self.game_state.inventory.player_resources.rupees_goal()
                    < COST[i as usize] as u16
                {
                    self.happiness_pond_show_later(k);
                } else {
                    self.sprite_slot_mut(k).set_direction(COST[i as usize]);
                    self.sprite_slot_mut(k).set_head_direction(i);
                    self.sprite_slot_mut(k).set_ai_state(3);
                }
            }
            3 => {
                self.sprite_slot_mut(k).set_delay_main(80);
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
                    self.sprite_slot_mut(k).set_ai_state(5);
                    return;
                }
                let pond = self.game_state.inventory.player_resources.rupees_in_pond();
                self.dialogue_number_mut()
                    .set_low_pair((pond / 10) * 16 + (pond % 10));
                self.sprite_slot_mut(k).set_ai_state(4);
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_show_message_unconditional(0x94);
                    self.sprite_slot_mut(k).set_ai_state(13);
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
                    self.sprite_slot_mut(j).set_b(1);
                    self.Palette_AssertTranslucencySwap();
                    self.PaletteFilter_WishPonds();
                    self.sprite_slot_mut(k).set_e(j as u8);
                    self.sprite_slot_mut(k).set_ai_state(6);
                    self.sprite_slot_mut(k).set_delay_main(255);
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
                        self.sprite_slot_mut(k).set_ai_state(7);
                    }
                }
            }
            7 => {
                let ai_state = if self.multiselect_choice().value() == 0 {
                    8
                } else {
                    12
                };
                self.sprite_slot_mut(k).set_ai_state(ai_state);
            }
            8 => {
                const MAX_BOMBS_HEX: [u8; 8] = [0x10, 0x15, 0x20, 0x25, 0x30, 0x35, 0x40, 0x50];
                let i = self
                    .game_state
                    .inventory
                    .player_resources
                    .next_bomb_upgrade_level();
                if i != 8 {
                    let filler = MAX_BOMBS_HEX[i as usize];
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
                self.sprite_slot_mut(k).set_ai_state(9);
            }
            9 => {
                self.Palette_AssertTranslucencySwap();
                self.set_sub_screen_layers(2);
                self.set_color_math_control(0x30);
                self.increment_cgram_update_flag();
                self.sprite_slot_mut(k).set_ai_state(10);
            }
            10 => {
                if self.game_state.frame.frame_counter & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.game_state.display.palette_filter.countdown() == 30 {
                        let j = self.sprite_slot_view(k).e() as usize;
                        self.sprite_slot_mut(j).set_state(0);
                    } else if self.game_state.display.palette_filter.countdown() == 0 {
                        self.sprite_slot_mut(k).set_ai_state(11);
                    }
                }
            }
            11 => {
                self.PaletteFilter_RestoreSP5F();
                self.Palette_RevertTranslucencySwap();
                self.sprite_slot_mut(k).set_ai_state(0);
                self.sprite_slot_mut(k).set_delay_main(255);
            }
            12 => {
                const ARROW_UPGRADE_REFILL_AMOUNTS: [u8; 8] =
                    [0x30, 0x35, 0x40, 0x45, 0x50, 0x55, 0x60, 0x70];
                let i = self
                    .game_state
                    .inventory
                    .player_resources
                    .next_arrow_upgrade_level();
                if i != 8 {
                    let filler = ARROW_UPGRADE_REFILL_AMOUNTS[i as usize];
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
                self.sprite_slot_mut(k).set_ai_state(9);
            }
            13 => {
                self.sprite_show_message_unconditional(0x154);
                self.sprite_slot_mut(k).set_ai_state(14);
            }
            14 => {
                const LUCK_MSG: [u16; 4] = [0x150, 0x151, 0x152, 0x153];
                const LUCK: [u8; 4] = [1, 0, 0, 2];
                let i = (self.get_random_number() & 3) as usize;
                self.sprite_battle_mut().set_item_drop_luck(LUCK[i]);
                self.sprite_battle_mut().clear_luck_kill_counter();
                self.sprite_show_message_unconditional(LUCK_MSG[i]);
                self.sprite_slot_mut(k).set_ai_state(0);
                self.sprite_slot_mut(k).set_delay_main(255);
            }
            _ => {}
        }
    }

    fn happiness_pond_show_later(&mut self, k: usize) {
        self.sprite_show_message_unconditional(0x14c);
        self.sprite_slot_mut(k).set_ai_state(0);
        self.sprite_slot_mut(k).set_delay_main(255);
    }

    pub(super) fn wish_pond2_draw(&mut self, k: usize) {
        const WISH_POND_ITEM_DRAW_FRAMES: [DrawMultipleData; 8] = [
            DrawMultipleData {
                x: 32,
                y: -64,
                char_flags: 0x0024,
                ext: 0,
            },
            DrawMultipleData {
                x: 32,
                y: -56,
                char_flags: 0x0034,
                ext: 0,
            },
            DrawMultipleData {
                x: 32,
                y: -64,
                char_flags: 0x0024,
                ext: 0,
            },
            DrawMultipleData {
                x: 32,
                y: -56,
                char_flags: 0x0034,
                ext: 0,
            },
            DrawMultipleData {
                x: 32,
                y: -64,
                char_flags: 0x0024,
                ext: 2,
            },
            DrawMultipleData {
                x: 32,
                y: -64,
                char_flags: 0x0024,
                ext: 2,
            },
            DrawMultipleData {
                x: 32,
                y: -64,
                char_flags: 0x0024,
                ext: 2,
            },
            DrawMultipleData {
                x: 32,
                y: -64,
                char_flags: 0x0024,
                ext: 2,
            },
        ];
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
        self.sprite_slot_mut(k).set_oam_flags((f & 7) * 2);
        let start = ((RECEIVE_ITEM_PREP_DRAW_FRAME_START_BYTES[g] >> 1) * 4) as usize;
        self.sprite_draw_multiple(k, &WISH_POND_ITEM_DRAW_FRAMES[start..start + 4], None);
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
            self.sprite_slot_mut(k).add_x_velocity(delta);
        }
        if self.sprite_slot_view(k).y_velocity() != 0 {
            let y_velocity = self.sprite_slot_view(k).y_velocity();
            let delta = if sign8(y_velocity) {
                2
            } else {
                0u8.wrapping_sub(2)
            };
            self.sprite_slot_mut(k).add_y_velocity(delta);
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
        self.sprite_slot_mut(j).set_ai_state(1);
        self.sprite_slot_mut(j).set_a(255);
        self.sprite_slot_mut(j).set_z(8);
        self.sprite_slot_mut(j).set_z_velocity(22);
        let x = (info.r0_x & !0xff).wrapping_add(u16::from(self.get_random_number()));
        let y = (info.r2_y & !0xff).wrapping_add(u16::from(self.get_random_number()));
        let pt = self.sprite_project_speed_towards_location(k, x, y, 10);
        self.sprite_slot_mut(j).set_x_velocity(pt.x);
        self.sprite_slot_mut(j).set_y_velocity(pt.y);
    }

    pub(super) fn sprite_transmute_to_bomb(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_sprite_type(0x4a);
        self.sprite_slot_mut(k).set_c(1);
        self.sprite_slot_mut(k).set_delay_aux1(255);
        self.sprite_slot_mut(k).set_flags3(0x18);
        self.sprite_slot_mut(k).set_oam_flags(8);
        self.sprite_slot_mut(k).set_health(0);
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
        self.sprite_slot_mut(j).set_flags2(0x3f);
        self.sprite_slot_mut(j).set_flags4(0x54);
        self.sprite_slot_mut(j).set_c(1);
        self.sprite_slot_mut(j).set_deflection_bits(0x48);
        self.sprite_slot_mut(j).set_oam_flags(3);
        self.sprite_slot_mut(j).set_bump_damage(4);
        self.sprite_slot_mut(j).set_delay_aux1(12);
        let t = self.game_state.sprites.system.limit_instance() as usize;
        self.sprite_slot_mut(j).set_graphics(t as u8);
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
            self.sprite_slot_mut(k).set_ai_state(1);
        }
    }

    pub(super) fn fairy_check_if_touchable(&mut self, k: usize) {
        let msg = self.game_state.messaging.dialogue_message_index.value();
        if self.game_state.frame.submodule == 2 && (msg == 0xc9 || msg == 0xca) {
            self.sprite_slot_mut(k).set_delay_aux4(40);
        }
    }

    pub(super) fn buzzblob_select_new_direction(&mut self, k: usize) {
        const LOCAL_X_VELOCITIES: [i8; 8] = [3, 2, -2, -3, -2, 2, 0, 0];
        const LOCAL_Y_VELOCITIES: [i8; 8] = [0, 2, 2, 0, -2, -2, 0, 0];
        const DELAY: [u8; 8] = [48, 48, 48, 48, 48, 48, 64, 64];
        let j = (self.get_random_number() & 7) as usize;
        self.sprite_slot_mut(k)
            .set_x_velocity(LOCAL_X_VELOCITIES[j] as u8);
        self.sprite_slot_mut(k)
            .set_y_velocity(LOCAL_Y_VELOCITIES[j] as u8);
        self.sprite_slot_mut(k).set_delay_main(DELAY[j]);
    }

    pub(super) fn lumberjack_check_proximity(&mut self, _k: usize, j: usize) -> bool {
        const X: [u16; 2] = [48, 52];
        const Y: [u16; 2] = [19, 20];
        const W: [u16; 2] = [98, 106];
        const H: [u16; 2] = [37, 40];
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        cur_x.wrapping_sub(link_x).wrapping_add(X[j]) < W[j]
            && cur_y.wrapping_sub(link_y).wrapping_add(Y[j]) < H[j]
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
        self.sprite_slot_mut(k).increment_die_action();
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
        self.sprite_slot_mut(k).increment_subtype2();
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
        self.sprite_slot_mut(k).set_ai_state(0);
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
        self.sprite_slot_mut(k).set_direction(direction);
        self.sprite_slot_mut(k).decrement_graphics();
    }

    pub(super) fn sprite_prep_popo(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_b(7);
    }

    pub(super) fn sprite_prep_popo2(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_b(15);
    }

    pub(super) fn sprite_prep_statue(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_y_low(7);
    }

    pub(super) fn sprite_prep_bari(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_z(6);
        if self.game_state.dungeon.room_tracking.room_index2() == 206 {
            self.sprite_slot_mut(k).decrement_c();
        }
        let delay_aux1 = (self.get_random_number() & 63).wrapping_add(128);
        self.sprite_slot_mut(k).set_delay_aux1(delay_aux1);
    }

    pub(super) fn sprite_prep_green_stalfos(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_z(9);
    }

    pub(super) fn sprite_prep_water_lever(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_y_low(5);
    }

    pub(super) fn sprite_prep_fire_debirando(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_sprite_type(0x63);
        self.sprite_prep_load_properties(k);
        self.sprite_slot_mut(k).decrement_g();
        self.sprite_prep_debirando_pit(k);
    }

    pub(super) fn sprite_prep_debirando_pit(&mut self, k: usize) {
        const DEBIRANDO_OAM_FLAGS: [u8; 2] = [6, 8];

        self.sprite_slot_mut(k).increment_g();
        self.sprite_slot_mut(k).set_delay_main(0);
        self.sprite_slot_mut(k).set_graphics(6);
        self.sprite_prep_ignore_projectiles(k);

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x64, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_mut(j).set_delay_main(96);
            self.sprite_slot_mut(k).set_head_direction(j as u8);
            let g = self.sprite_slot_view(k).g();
            self.sprite_slot_mut(j).set_g(g);
            self.sprite_slot_mut(j)
                .set_oam_flags(DEBIRANDO_OAM_FLAGS[g as usize]);
        }
    }

    pub(super) fn sprite_prep_weak_guard(&mut self, k: usize) {
        let dir = self.get_random_number() & 3;
        self.sprite_slot_mut(k).set_direction(dir);
        self.sprite_slot_mut(k).set_head_direction(dir);
        self.sprite_slot_mut(k).set_delay_main(16);
    }

    pub(super) fn sprite_prep_laser_eye_bounce(&mut self, k: usize) {
        let t = self.sprite_slot_view(k).sprite_type();
        self.sprite_slot_mut(k).set_direction(t.wrapping_sub(0x95));
        if t >= 0x97 {
            self.sprite_slot_mut(k).add_x_low(8);
            let head_direction = (self.sprite_slot_view(k).x_low() & 16) ^ 16;
            self.sprite_slot_mut(k).set_head_direction(head_direction);
            if self.sprite_slot_view(k).head_direction() == 0 {
                let y_low = self
                    .sprite_slot_view(k)
                    .y_low()
                    .wrapping_add(if (t & 1) != 0 { (-8i8) as u8 } else { 8 });
                self.sprite_slot_mut(k).set_y_low(y_low);
            }
        } else {
            let head_direction = self.sprite_slot_view(k).y_low() & 16;
            self.sprite_slot_mut(k).set_head_direction(head_direction);
            if self.sprite_slot_view(k).head_direction() == 0 {
                let x_low = self
                    .sprite_slot_view(k)
                    .x_low()
                    .wrapping_add(if (t & 1) != 0 { (-8i8) as u8 } else { 8 });
                self.sprite_slot_mut(k).set_x_low(x_low);
            }
        }
    }

    pub(super) fn sprite_prep_wall_cannon(&mut self, k: usize) {
        let direction = self.sprite_slot_view(k).sprite_type().wrapping_sub(0x66);
        self.sprite_slot_mut(k).set_direction(direction);
        self.sprite_slot_mut(k).set_a(direction & 2);
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
            self.sprite_slot_mut(k).increment_ignore_projectile();
        } else {
            self.sprite_slot_mut(k).set_state(0);
        }
    }

    pub(super) fn sprite_prep_smithy(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
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
                self.sprite_slot_mut(k).set_state(0);
            } else {
                self.sprite_slot_mut(k).set_subtype2(2);
            }
            return;
        }

        self.sprite_prep_smithy_spawn_dumb_barrier_sprite(k);
        self.sprite_slot_mut(k).add_x_low(2);
        self.sprite_slot_mut(k).subtract_y_low(3);
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
        self.sprite_slot_mut(j).set_e(k as u8);
        self.sprite_slot_mut(k).set_e(j as u8);

        if self
            .game_state
            .inventory
            .save_progress
            .progress_indicator_3()
            & 0x80
            != 0
        {
            self.sprite_slot_mut(k).set_ai_state(5);
            self.sprite_slot_mut(j).set_ai_state(5);
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
        self.sprite_slot_mut(j).add_x_low(0x2c);
        self.sprite_slot_mut(j).set_direction(1);
        self.sprite_slot_mut(j).set_a(4);
        self.sprite_slot_mut(j).set_ignore_projectile(4);
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
        self.sprite_slot_mut(j).set_subtype2(1);
        self.sprite_slot_mut(j).set_flags4(0);
        self.sprite_slot_mut(j).set_ignore_projectile(1);
    }

    pub(super) fn sprite_prep_ignore_projectiles(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_haunted_grove_animal(&mut self, k: usize) {
        let direction = self.sprite_is_right_of_link(k).a;
        self.sprite_slot_mut(k).set_direction(direction);
        self.sprite_prep_haunted_grove_ostritch(k);
    }

    pub(super) fn sprite_prep_haunted_grove_ostritch(&mut self, k: usize) {
        if self.game_state.inventory.items.flute() >= 2 {
            self.sprite_slot_mut(k).set_state(0);
        }
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_whirlpool(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        self.sprite_slot_mut(k).set_a(1);
    }

    pub(super) fn sprite_prep_bonk_item(&mut self, k: usize) {
        const DASH_ITEM_MASK: [u16; 2] = [0x4000, 0x2000];
        if self.game_state.world.location.is_outdoors() {
            self.sprite_slot_mut(k).set_graphics(2);
            return;
        }

        self.sprite_slot_mut(k).set_floor(2);
        if self.game_state.world.location.dungeon_room() == 0x0107 {
            if self.game_state.inventory.items.book() != 0 {
                self.sprite_slot_mut(k).set_state(0);
            } else {
                self.DecodeAnimatedSpriteTile_variable(0x0e);
            }
        } else {
            let j = self.game_state.sprite_battle.item_drop_counter();
            self.sprite_battle_mut().increment_item_drop_counter();
            self.sprite_slot_mut(k).set_die_action(j);
            if self.game_state.dungeon.savegame_state.savegame_state_bits()
                & DASH_ITEM_MASK[j as usize]
                != 0
            {
                self.sprite_slot_mut(k).set_state(0);
            }
            self.sprite_slot_mut(k).increment_graphics();
            self.sprite_slot_mut(k).set_oam_flags(8);
            self.sprite_slot_mut(k).or_flags3(0x20);
        }
    }

    pub(super) fn sprite_prep_digging_game_guy_bounce(&mut self, k: usize) {
        if self.game_state.player.follower_link.y() < self.sprite_get_y(k) {
            self.sprite_slot_mut(k).set_ai_state(5);
            self.sprite_slot_mut(k).subtract_x_low(9);
            self.sprite_slot_mut(k).set_graphics(1);
        }
        self.sprite_slot_mut(k).increment_ignore_projectile();
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
        self.sprite_slot_mut(k).set_x_velocity(0);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).y_low().wrapping_add(7)
                    < self.game_state.player.follower_link.y() as u8
                    && self.sprite_direction_to_face_link(k, None) == 2
                {
                    if self.game_state.sprites.follower_runtime.indicator() == 0 {
                        if self.sprite_show_solicited_message(k, 0x187) & 0x100 != 0 {
                            self.sprite_slot_mut(k).increment_ai_state();
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
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.sprite_slot_mut(k).set_graphics(1);
                    self.sprite_slot_mut(k).set_delay_main(80);
                    self.digging_game_prize_mut().clear_prize_spawned();
                    self.lanmola_segment_motion_mut(1).set_z_offset(0);
                    self.sprite_slot_mut(k).set_delay_aux1(5);
                    self.sprite_initialize_secondary_item_minigame(1);
                    self.set_music_control(14);
                } else {
                    self.sprite_show_message_unconditional(0x189);
                    self.sprite_slot_mut(k).set_ai_state(0);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_graphics(1);
                } else if self.sprite_slot_view(k).delay_aux1() == 0 {
                    self.sprite_slot_mut(k).xor_graphics(3);
                    if self.sprite_slot_view(k).graphics() & 1 != 0 {
                        self.sprite_slot_mut(k).set_x_velocity((-16i8) as u8);
                    }
                    self.sprite_slot_mut(k).set_delay_aux1(5);
                }
            }
            3 => {
                self.sprite_slot_mut(k).increment_ai_state();
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
                self.sprite_slot_mut(k).increment_ai_state();
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
            self.sprite_slot_mut(k).set_state(0);
        }
        self.sprite_slot_mut(k).increment_ignore_projectile();
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(8));
    }

    pub(super) fn sprite_prep_rupee_pull(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(8));
    }

    pub(super) fn sprite_prep_shopkeeper(&mut self, k: usize) {
        const SHOP_KEEPER_WHERE: [u8; 13] = [
            0x0f, 0x10, 0x00, 0x06, 0x18, 0x12, 0x1e, 0xff, 0x1f, 0x23, 0x24, 0x25, 0x27,
        ];

        self.sprite_slot_mut(k).increment_ignore_projectile();
        self.sprite_slot_mut(k).or_flags2(2);
        self.sprite_slot_mut(k).or_oam_flags(12);
        self.sprite_slot_mut(k).or_flags3(16);

        let room = self.game_state.world.location.dungeon_room_index();
        let j = SHOP_KEEPER_WHERE
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
                self.sprite_slot_mut(k).set_subtype2(4);
                self.minigame_state_mut().set_credits(0xff);
            }
            3 => {
                self.sprite_slot_mut(k).set_subtype2(1);
                self.sprite_slot_mut(k).set_graphics(1);
                self.minigame_state_mut().set_credits(0xff);
            }
            4 => {
                self.sprite_slot_mut(k).set_subtype2(3);
                self.minigame_state_mut().set_credits(0xff);
            }
            5 | 7 | 8 => {
                self.shop_keeper_spawn_shop_item(k, 0, 7);
                self.shop_keeper_spawn_shop_item(k, 1, 10);
                self.shop_keeper_spawn_shop_item(k, 2, 12);
            }
            6 | 9 | 12 => self.sprite_slot_mut(k).set_subtype2(2),
            10 => self.sprite_slot_mut(k).set_subtype2(5),
            11 => self.sprite_slot_mut(k).set_subtype2(6),
            _ => unreachable!(),
        }
    }

    pub(super) fn shop_keeper_spawn_shop_item(&mut self, k: usize, pos: usize, what: u8) {
        const SHOP_KEEPER_ITEM_X: [i16; 3] = [-44, 8, 60];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0xbb, &mut info, 12);
        assert!(j >= 0);
        let j = j as usize;
        self.sprite_slot_mut(j).set_ignore_projectile(what);
        self.sprite_slot_mut(j).set_subtype2(what);
        self.sprite_set_x(j, info.r0_x.wrapping_add(SHOP_KEEPER_ITEM_X[pos] as u16));
        self.sprite_set_y(j, info.r2_y.wrapping_add(0x27));
        self.sprite_slot_mut(j).or_flags2(4);
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
            self.sprite_slot_mut(j).set_y_velocity(0);
            self.sprite_slot_mut(j).set_b(0);
            self.sprite_slot_mut(j).set_direction(0);
            self.sprite_slot_mut(j).set_floor(0);
            self.sprite_slot_mut(j).set_subtype2(1);
            self.sprite_slot_mut(j).set_flags2(1);
            self.sprite_slot_mut(j).set_flags3(1);
            self.sprite_slot_mut(j).set_oam_flags(1);
            self.sprite_slot_mut(j).set_x_low(204);
            self.sprite_slot_mut(j).set_x_high(7);
            self.sprite_slot_mut(j).set_y_low(50);
            self.sprite_slot_mut(j).set_y_high(6);
            self.sprite_slot_mut(j).set_deflection_bits(128);
        }
    }

    pub(super) fn sprite_prep_storyteller(&mut self, k: usize) {
        const ROOMS: [u8; 5] = [0x0e, 0x0e, 0x12, 0x1a, 0x14];
        let mut r = ROOMS
            .iter()
            .position(|&room| room == self.game_state.world.location.dungeon_room_index())
            .map_or(0xff, |idx| idx as u8);
        if r == 0 && self.sprite_slot_view(k).x_high() & 1 != 0 {
            r = 1;
        }
        self.sprite_slot_mut(k).set_subtype2(r);
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_adults(&mut self, k: usize) {
        const HUMAN_MULTI_TYPES: [u8; 3] = [3, 0xe1, 0x19];
        self.sprite_slot_mut(k).increment_ignore_projectile();
        let dungeon_room = self.game_state.world.location.dungeon_room_index();
        let subtype2 = HUMAN_MULTI_TYPES
            .iter()
            .position(|&room| room == dungeon_room)
            .map_or(0xff, |idx| idx as u8);
        self.sprite_slot_mut(k).set_subtype2(subtype2);
    }

    pub(super) fn sprite_prep_sage(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        if self.game_state.world.location.dungeon_room_index() == 10 {
            self.sprite_slot_mut(k).increment_subtype2();
            self.sprite_slot_mut(k).set_oam_flags(11);
        }
    }

    pub(super) fn sprite_prep_kiki(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
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
            self.sprite_slot_mut(k).set_state(0);
        }
    }

    pub(super) fn sprite_prep_locksmith(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        if self.game_state.sprites.follower_runtime.indicator() == 9 {
            self.sprite_slot_mut(k).set_state(0);
            return;
        }
        if self.game_state.sprites.follower_runtime.indicator() == 12 {
            self.sprite_slot_mut(k).set_ai_state(2);
        }
        if self
            .game_state
            .inventory
            .save_progress
            .progress_indicator_3()
            & 0x10
            != 0
        {
            self.sprite_slot_mut(k).set_ai_state(4);
        }
    }

    pub(super) fn sprite_prep_sick_kid(&mut self, k: usize) {
        if self.game_state.inventory.items.bug_net() != 0 {
            self.sprite_slot_mut(k).set_ai_state(3);
        }
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_tektite(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [9, 7];
        const HEALTH: [u8; 2] = [8, 12];
        const BUMP_DAMAGE: [u8; 2] = [3, 5];
        let j = ((self.sprite_slot_view(k).x_low() >> 4) & 1) as usize;
        self.sprite_slot_mut(k).set_a(j as u8);
        self.sprite_slot_mut(k).set_oam_flags(OAM_FLAGS[j]);
        self.sprite_slot_mut(k).set_health(HEALTH[j]);
        self.sprite_slot_mut(k).set_bump_damage(BUMP_DAMAGE[j]);
        self.sprite_apply_speed_towards_link(k, 16);
        self.sprite_slot_mut(k).set_z_velocity(32);
        self.sprite_slot_mut(k).increment_ai_state();
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
        self.sprite_slot_mut(k).set_a(x_low);
        self.sprite_slot_mut(k).set_b(x_high);
        self.sprite_slot_mut(k).set_c(y_low);
        self.sprite_slot_mut(k).set_g(y_high);
    }

    pub(super) fn chain_chomp_move_chain(&mut self, k: usize) {
        const MULS: [u8; 6] = [205, 154, 102, 51, 8, 0xbd];

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
            let mul = MULS[(pos & 7) - 1];
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
        self.sprite_slot_mut(k).set_z(24);
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_mrs_sahasrahla(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_y_low(8);
        self.sprite_prep_magic_bat(k);
    }

    pub(super) fn sprite_prep_magic_bat(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_x_low(8);
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_fortune_teller(&mut self, k: usize) {
        self.sprite_prep_incr_xy_low8(k);
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_fairy_pond(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [10, 2];
        let j = ((self.sprite_slot_view(k).x_low() >> 4) & 1) as usize;
        self.sprite_slot_mut(k).set_a(j as u8);
        self.sprite_slot_mut(k).set_oam_flags(OAM_FLAGS[j]);
    }

    pub(super) fn sprite_prep_hobo(&mut self, k: usize) {
        for _ in 1..=15 {
            self.sprite_prep_hobo_spawn_smoke(k);
        }
        for i in (1..=15).rev() {
            if self.sprite_slot_view(i).sprite_type() == 0x2b {
                self.sprite_slot_mut(i).set_state(0);
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
            self.sprite_slot_mut(0).set_ai_state(3);
        }
        self.sprite_slot_mut(0).set_ignore_projectile(1);
    }

    pub(super) fn sprite_prep_hobo_spawn_smoke(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_slot_mut(j).set_subtype2(0);
            self.sprite_slot_mut(j).set_ignore_projectile(0);
        }
    }

    pub(super) fn sprite_prep_hobo_spawn_fire(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_x(j, 0x0194);
            self.sprite_set_y(j, 0x003f);
            self.sprite_slot_mut(j).set_subtype2(2);
            self.sprite_slot_mut(j).set_ignore_projectile(2);
            self.sprite_slot_mut(j).set_flags2(0);
            self.sprite_slot_mut(j).masked_or_oam_flags(!0x0e, 2);
        }
    }

    pub(super) fn hobo_spawn_bubble(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.sprite_slot_mut(j_usize).set_subtype2(1);
            self.sprite_slot_mut(j_usize).set_z_velocity(2);
            self.sprite_slot_mut(j_usize).set_delay_main(96);
            self.sprite_slot_mut(j_usize).set_delay_aux1(48);
            self.sprite_slot_mut(j_usize).set_ignore_projectile(48);
            self.sprite_slot_mut(j_usize).set_flags2(0);
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
            self.sprite_slot_mut(j).set_subtype2(3);
            self.sprite_slot_mut(j).set_z_velocity(7);
            self.sprite_slot_mut(j).set_delay_main(96);
            self.sprite_slot_mut(j).set_ignore_projectile(96);
            self.sprite_slot_mut(j).set_flags2(0);
        }
    }

    pub(super) fn sprite_prep_master_sword(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_x_low(6);
        self.sprite_slot_mut(k).add_y_low(6);
    }

    pub(super) fn sprite_prep_roller_horizontal_right_first(&mut self, k: usize) {
        let ai_state = (!self.sprite_slot_view(k).x_low() & 16) >> 4;
        self.sprite_slot_mut(k).set_ai_state(ai_state);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_mut(k).increment_flags4();
        }
        self.sprite_slot_mut(k).set_direction(0);
    }

    pub(super) fn sprite_prep_roller_left_right(&mut self, k: usize) {
        let ai_state = (!self.sprite_slot_view(k).x_low() & 16) >> 4;
        self.sprite_slot_mut(k).set_ai_state(ai_state);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_mut(k).increment_flags4();
        }
        self.sprite_slot_mut(k).set_direction(1);
    }

    pub(super) fn sprite_prep_roller_vertical_down_first(&mut self, k: usize) {
        let ai_state = (self.sprite_slot_view(k).y_low() & 16) >> 4;
        self.sprite_slot_mut(k).set_ai_state(ai_state);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_mut(k).increment_flags4();
        }
        self.sprite_slot_mut(k).set_direction(2);
    }

    pub(super) fn sprite_prep_roller_up_down(&mut self, k: usize) {
        let ai_state = (self.sprite_slot_view(k).y_low() & 16) >> 4;
        self.sprite_slot_mut(k).set_ai_state(ai_state);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_mut(k).increment_flags4();
        }
        self.sprite_slot_mut(k).set_direction(3);
    }

    pub(super) fn sprite_prep_kodongo(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_x_low(4);
        self.sprite_set_y(k, self.sprite_get_y(k).wrapping_sub(5));
        self.sprite_slot_mut(k).decrement_subtype();
    }

    pub(super) fn sprite_prep_spark(&mut self, k: usize) {
        self.sprite_slot_mut(k).decrement_subtype();
    }

    pub(super) fn sprite_prep_lost_woods_bird(&mut self, k: usize) {
        let z_velocity = (self.get_random_number() & 0x1f).wrapping_sub(0x10);
        self.sprite_slot_mut(k).set_z_velocity(z_velocity);
        self.sprite_slot_mut(k).set_z(64);
        self.sprite_prep_lost_woods_squirrel(k);
    }

    pub(super) fn sprite_prep_lost_woods_squirrel(&mut self, k: usize) {
        let x_velocity = if self.sprite_is_right_of_link(k).a != 0 {
            (-16i8) as u8
        } else {
            16
        };
        self.sprite_slot_mut(k).set_x_velocity(x_velocity);
        let y_vel = if sign8(self.overworld_vertical_scroll_delta_low()) {
            4
        } else {
            (-4i8) as u8
        };
        self.sprite_slot_mut(k).set_y_velocity(y_vel);
        self.sprite_slot_mut(k).set_ignore_projectile(y_vel);
    }

    pub(super) fn sprite_prep_antifairy(&mut self, k: usize) {
        const LOCAL_X_VELOCITIES: [i8; 2] = [16, -16];
        let idx = ((self.sprite_slot_view(k).x_low() >> 4) & 1) as usize;
        self.sprite_slot_mut(k)
            .set_x_velocity(LOCAL_X_VELOCITIES[idx] as u8);
        self.sprite_slot_mut(k).set_y_velocity((-16i8) as u8);
    }

    pub(super) fn sprite_prep_antifairy_circle(&mut self, k: usize) {
        const X: [i16; 3] = [10, 20, 10];
        const Y: [i16; 3] = [-10, 0, 10];
        const LOCAL_X_VELOCITIES: [i8; 3] = [18, 0, -18];
        const LOCAL_Y_VELOCITIES: [i8; 3] = [0, 18, 0];
        const A: [u8; 3] = [1, 1, 0];
        const B: [u8; 3] = [0, 1, 1];

        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(10));
        self.sprite_slot_mut(k).set_y_velocity((-18i8) as u8);
        self.sprite_slot_mut(k).set_x_velocity(0);
        self.sprite_slot_mut(k).set_a(0);
        self.sprite_slot_mut(k).set_b(0);
        self.temp_counter_mut().set(2);
        loop {
            let i = self.game_state.scratch_counter.value() as usize;
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x82, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, info.r0_x.wrapping_add(X[i] as u16));
                self.sprite_set_y(j, info.r2_y.wrapping_add(Y[i] as u16));
                self.sprite_slot_mut(j)
                    .set_x_velocity(LOCAL_X_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j)
                    .set_y_velocity(LOCAL_Y_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j).set_a(A[i]);
                self.sprite_slot_mut(j).set_b(B[i]);
            }
            self.temp_counter_mut().decrement();
            if sign8(self.game_state.scratch_counter.value()) {
                break;
            }
        }
    }

    pub(super) fn sprite_prep_king_zora(&mut self, k: usize) {
        if self.game_state.inventory.items.flippers() != 0 {
            self.sprite_slot_mut(k).set_state(0);
        } else {
            self.sprite_slot_mut(k).increment_ignore_projectile();
        }
    }

    pub(super) fn sprite_prep_do_nothing_d(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_octorok(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [3, 5];
        const HEALTH: [u8; 2] = [2, 4];
        let j = self.game_state.world.region.dark_world_region_index() as usize;
        self.sprite_slot_mut(k).set_health(HEALTH[j]);
        self.sprite_slot_mut(k).set_bump_damage(BUMP_DAMAGE[j]);
        let delay_main = self.get_random_number() & 127;
        self.sprite_slot_mut(k).set_delay_main(delay_main);
    }

    pub(super) fn sprite_prep_swimming_zora(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_delay_main(64);
        self.sprite_prep_geldman(k);
    }

    pub(super) fn sprite_prep_geldman(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_x_low(8);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_kyameron(&mut self, k: usize) {
        let x_low = self.sprite_slot_view(k).x_low();
        let x_high = self.sprite_slot_view(k).x_high();
        let y_low = self.sprite_slot_view(k).y_low();
        let y_high = self.sprite_slot_view(k).y_high();
        self.sprite_slot_mut(k).set_a(x_low);
        self.sprite_slot_mut(k).set_b(x_high);
        self.sprite_slot_mut(k).set_c(y_low);
        self.sprite_slot_mut(k).set_head_direction(y_high);
    }

    pub(super) fn sprite_prep_walking_zora(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_delay_main(96);
    }

    pub(super) fn sprite_prep_talking_tree(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        let x = self.sprite_get_x(k).wrapping_sub(8);
        self.sprite_set_x(k, x);
        self.sprite_prep_talking_tree_spawn_eyeball(k, 0);
        self.sprite_prep_talking_tree_spawn_eyeball(k, 1);
    }

    pub(super) fn sprite_prep_talking_tree_spawn_eyeball(&mut self, k: usize, dir: usize) {
        const TALKING_TREE_SPAWN_X: [i16; 2] = [-4, 14];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x25, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_slot_mut(j).set_head_direction(dir as u8);
            let x = info.r0_x.wrapping_add(TALKING_TREE_SPAWN_X[dir] as u16);
            let y = info.r2_y.wrapping_sub(11);
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);
            self.sprite_slot_mut(j).set_a(x as u8);
            self.sprite_slot_mut(j).set_b((x >> 8) as u8);
            self.sprite_slot_mut(j).set_c(y as u8);
            self.sprite_slot_mut(j).set_e((y >> 8) as u8);
            self.sprite_slot_mut(j).set_subtype2(1);
        }
    }

    pub(super) fn sprite_prep_swamola(&mut self, k: usize) {
        self.sprite_prep_swamola_initialize_segments(k);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_swamola_initialize_segments(&mut self, k: usize) {
        const BUGGY_SWAMOLA_LOOKUP: [usize; 6] = [0x1c, 0xa9, 0x03, 0x9d, 0x90, 0x0d];
        let mut j = if self
            .game_state
            .enhanced_features
            .has(FEATURE_MISC_BUG_FIXES_PREP)
        {
            k * 32
        } else {
            BUGGY_SWAMOLA_LOOKUP[k]
        };
        let x = self.sprite_slot_view(k).x();
        let y = self.sprite_slot_view(k).y();
        for _ in 0..32 {
            self.swamola_history_mut(j).set_position(x, y);
            j += 1;
        }
    }

    pub(super) fn sprite_prep_flute_kid(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        let subtype2 = (self.game_state.inventory.save_progress.dark_world_state() >> 6) & 1;
        self.sprite_slot_mut(k).set_subtype2(subtype2);
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
                self.sprite_slot_mut(k).set_graphics(3);
                self.sprite_slot_mut(k).set_ai_state(5);
            } else if flute == 2 {
                self.sprite_slot_mut(k).set_graphics(1);
            }
            self.sprite_slot_mut(k).add_x_low(8);
            self.sprite_slot_mut(k).subtract_y_low(8);
        } else if flute >= 2 {
            self.sprite_slot_mut(k).set_state(0);
        } else {
            self.sprite_slot_mut(k).add_x_low(7);
        }
    }

    pub(super) fn sprite_prep_move_down_8px(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_y_low(8);
    }

    pub(super) fn sprite_prep_zazakku(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_pedestal_plaque(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        if self.game_state.world.location.overworld_screen_index() == 48 {
            self.sprite_slot_mut(k).add_x_low(7);
        }
    }

    pub(super) fn sprite_prep_stalfos(&mut self, k: usize) {
        let subtype = self.sprite_slot_view(k).x_low() & 16;
        self.sprite_slot_mut(k).set_subtype(subtype);
        if self.sprite_slot_view(k).subtype() != 0 {
            self.sprite_slot_mut(k).set_oam_flags(7);
        }
    }

    pub(super) fn sprite_prep_moldorm(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_mut(k).increment_ignore_projectile();
        self.sprite_initialized_segmented(k);
    }

    pub(super) fn sprite_prep_lanmolas(&mut self, k: usize) {
        const INIT_DELAY: [u8; 3] = [128, 192, 255];
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_mut(k).set_delay_main(INIT_DELAY[k]);
        self.sprite_slot_mut(k).set_z(0xff);
        for i in 0..64 {
            self.lanmola_segment_motion_mut(k * 0x40 + i)
                .set_z_offset(0xff);
        }
        let value = 7;
        self.garnish_slot_view_mut(k).set_y_low(value);
    }

    pub(super) fn sprite_prep_bumper(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_move_down_8px_right8px(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_x_low(8);
        self.sprite_slot_mut(k).add_y_low(8);
    }

    pub(super) fn sprite_prep_hardhat_beetle(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [6, 8];
        const HEALTH: [u8; 2] = [32, 6];
        const A: [u8; 2] = [16, 12];
        const STATE: [u8; 2] = [1, 3];
        const FLAGS5: [u8; 2] = [2, 6];
        const BUMP_DAMAGE: [u8; 2] = [5, 3];
        let j = usize::from((self.sprite_slot_view(k).x_low() & 0x10) != 0);
        self.sprite_slot_mut(k).set_oam_flags(OAM_FLAGS[j]);
        self.sprite_slot_mut(k).set_health(HEALTH[j]);
        self.sprite_slot_mut(k).set_a(A[j]);
        self.sprite_slot_mut(k).set_ai_state(STATE[j]);
        self.sprite_slot_mut(k).set_flags5(FLAGS5[j]);
        self.sprite_slot_mut(k).set_bump_damage(BUMP_DAMAGE[j]);
    }

    pub(super) fn sprite_prep_mini_helmasaur(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_a(16);
        self.sprite_slot_mut(k).set_ai_state(1);
    }

    pub(super) fn sprite_prep_fairy(&mut self, k: usize) {
        let a = self.get_random_number() & 1;
        self.sprite_slot_mut(k).set_a(a);
        self.sprite_slot_mut(k).set_direction(a ^ 1);
        self.sprite_prep_absorbable(k);
    }

    pub(super) fn sprite_prep_falling_ice(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_armos_knight(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_mut(k).set_delay_main(255);
        self.sprite_workspace_mut().increment_prep_shared_counter();
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_desert_statue(&mut self, k: usize) {
        let limit_instance = self.game_state.sprites.system.limit_instance();
        self.sprite_slot_mut(k).set_a(limit_instance);
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
        self.sprite_slot_mut(k).set_direction(direction);
    }

    pub(super) fn sprite_prep_big_spike(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_crystal_switch(&mut self, k: usize) {
        const CRYSTAL_SWITCH_PAL: [u8; 2] = [2, 4];
        let oam_flags = CRYSTAL_SWITCH_PAL[(self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            & 1) as usize];
        self.sprite_slot_mut(k).or_oam_flags(oam_flags);
    }

    pub(super) fn sprite_prep_kholdstare_shell(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_mut(k).set_delay_aux1(192);
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_kholdstare(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_mut(k).set_ai_state(3);
        self.sprite_prep_ignore_projectiles(k);
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_agahnim(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [11, 7];
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_mut(k).set_graphics(0);
        self.sprite_slot_mut(k).set_direction(3);
        self.sprite_prep_move_down_8px_right8px(k);
        let oam_flags = OAM_FLAGS[self.game_state.world.region.dark_world_region_index() as usize];
        self.sprite_slot_mut(k).set_oam_flags(oam_flags);
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
                self.sprite_slot_mut(k).add_x_low(8);
                self.sprite_slot_mut(k).add_y_low(16);
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
                self.sprite_slot_mut(k).set_graphics(3);
                self.sprite_slot_mut(k).set_delay_main(128);
                self.trinexx_initialize_alt_sprites(k);
            }
            0xcd => {
                self.sprite_slot_mut(k).set_delay_main(255);
                self.trinexx_initialize_alt_sprites(k);
            }
            _ => {}
        }
    }

    fn trinexx_initialize_alt_sprites(&mut self, k: usize) {
        for j in (0..=0x1a).rev() {
            self.alt_sprite_slot_mut(j).initialize_trinexx_component();
        }
        self.sprite_slot_mut(k).set_subtype2(1);
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
            self.sprite_slot_mut(k).increment_e();
            self.sprite_slot_mut(k).increment_ignore_projectile();
        }
    }

    pub(super) fn sprite_prep_overworld_bonk_item(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_e();
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_shield_pickup(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_nice_bee(&mut self, k: usize) {
        let or_bottle = self.game_state.inventory.items.bottle(0)
            | self.game_state.inventory.items.bottle(1)
            | self.game_state.inventory.items.bottle(2)
            | self.game_state.inventory.items.bottle(3);
        if or_bottle & 8 != 0 {
            self.sprite_slot_mut(k).set_state(0);
        }
        self.sprite_slot_mut(k).increment_e();
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_do_nothing_g(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_fire_bar(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_b();
        self.sprite_slot_mut(k).increment_ignore_projectile();
    }

    pub(super) fn sprite_prep_spike(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_x_velocity(32);
        self.sprite_slot_mut(k).set_y_velocity((-16i8) as u8);
        self.sprite_move_y(k);
        self.sprite_slot_mut(k).set_y_velocity(0);
    }

    pub(super) fn sprite_prep_rock_stal(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_y_velocity((-16i8) as u8);
        self.sprite_move_y(k);
        self.sprite_slot_mut(k).set_y_velocity(0);
    }

    pub(super) fn sprite_prep_blob(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_graphics(4);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub(super) fn sprite_prep_arrghus(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_slot_mut(k).set_z(24);
    }

    pub(super) fn sprite_prep_arrghi(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        let subtype2 = self.get_random_number();
        self.sprite_slot_mut(k).set_subtype2(subtype2);
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
        self.sprite_slot_mut(k).set_x_low(x_low);
        self.sprite_slot_mut(k).set_x_high(x_high);
        self.sprite_slot_mut(k).set_y_low(y_low);
        self.sprite_slot_mut(k).set_y_high(y_high);
    }

    pub(super) fn arrghus_handle_puffs(&mut self, k: usize) {
        const PUFF_ORBIT_BASE_ANGLES: [u16; 13] = [
            0, 0x40, 0x80, 0xc0, 0x100, 0x140, 0x180, 0x1c0, 0, 0x66, 0xcc, 0x132, 0x198,
        ];
        const PUFF_ORBIT_ANGLE_XOR_MASKS: [u16; 13] =
            [0, 0, 0, 0, 0, 0, 0, 0, 0x1ff, 0x1ff, 0x1ff, 0x1ff, 0x1ff];
        const PUFF_ORBIT_PHASE_OFFSETS: [u8; 13] = [
            0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c,
        ];
        const PUFF_ORBIT_WAVE_SHIFTS: [i8; 52] = [
            0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3, -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5,
            -4, -3, -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3, -2, -1, 0, -1, -2, -3, -4,
            -5, -6, -6, -5, -4, -3, -2, -1,
        ];

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
            self.sprite_slot_mut(k).increment_a();
            if self.sprite_slot_view(k).a() == 13 {
                self.sprite_slot_mut(k).set_a(0);
            }
        }
        if self.game_state.frame.frame_counter & 7 == 0 {
            self.sprite_slot_mut(k).increment_b();
            if self.sprite_slot_view(k).b() == 13 {
                self.sprite_slot_mut(k).set_b(0);
            }
        }

        let sprite_x = self.sprite_get_x(k) as i32;
        let sprite_y = self.sprite_get_y(k) as i32;
        for i in 0..13 {
            let r0 = base.wrapping_add(PUFF_ORBIT_BASE_ANGLES[i]) ^ PUFF_ORBIT_ANGLE_XOR_MASKS[i];
            let r14 = self
                .game_state
                .sprites
                .overlord_slots
                .slot(2)
                .x_low()
                .wrapping_add(PUFF_ORBIT_PHASE_OFFSETS[i]);
            let sin_arg = r14.wrapping_add_signed(
                PUFF_ORBIT_WAVE_SHIFTS[self.sprite_slot_view(k).a() as usize + i],
            );
            let cos_arg = r14.wrapping_add_signed(
                PUFF_ORBIT_WAVE_SHIFTS[self.sprite_slot_view(k).b() as usize + i],
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
        self.sprite_slot_mut(k).set_delay_main(80);
        self.sprite_slot_mut(k).increment_ignore_projectile();
        self.sprite_slot_mut(k).set_graphics(2);
        self.dungeon_moving_floor_mut().increment_floor_move_flags();
        self.sprite_slot_mut(k).set_c(112);
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
                self.sprite_slot_mut(k).set_state(0);
            }
        } else {
            let j = self.sprite_slot_view(k).x_high() & 1;
            let mask = if j != 0 { 0x2000 } else { 0x4000 };
            if self.game_state.dungeon.savegame_state.savegame_state_bits() & mask != 0 {
                self.sprite_slot_mut(k).set_state(0);
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
        self.sprite_slot_mut(k).set_subtype(255);
        let j = self.game_state.sprite_battle.item_drop_counter();
        self.sprite_battle_mut().increment_item_drop_counter();
        self.sprite_slot_mut(k).set_die_action(j);
    }

    pub(super) fn sprite_prep_key_set_item_drop(&mut self, k: usize) {
        let die_action = self.game_state.sprite_battle.item_drop_counter();
        self.sprite_slot_mut(k).set_die_action(die_action);
        self.sprite_battle_mut().increment_item_drop_counter();
    }

    pub(super) fn sprite_prep_big_key(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_x_low(8);
        self.sprite_slot_mut(k).set_subtype(0xff);
        self.sprite_prep_big_key_load_graphics(k);
    }

    pub(super) fn sprite_prep_big_key_load_graphics(&mut self, k: usize) {
        self.DecodeAnimatedSpriteTile_variable(0x22);
        self.sprite_prep_key_set_item_drop(k);
    }

    pub(super) fn sprite_prep_incr_xy_low8(&mut self, k: usize) {
        self.sprite_slot_mut(k).add_x_low(8);
        self.sprite_slot_mut(k).add_y_low(8);
    }

    pub(super) fn sprite_prep_fake_sword(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_old_man_bounce(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        if self.game_state.world.location.dungeon_room_index() == 0xe4 {
            self.sprite_slot_mut(k).set_subtype2(2);
            return;
        }
        if self.game_state.sprites.follower_runtime.indicator() == 0 {
            if self.game_state.inventory.items.mirror() == 2 {
                self.sprite_slot_mut(k).set_state(0);
            }
            self.follower_state_mut().set_indicator(4);
            self.load_follower_graphics();
            self.follower_state_mut().set_indicator(0);
        } else {
            self.sprite_slot_mut(k).set_state(0);
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
        if self.game_state.inventory.items.sword_type() >= 2 {
            self.sprite_slot_mut(k).set_state(0);
            return;
        }
        self.sprite_slot_mut(k).increment_ignore_projectile();
        let dir = self.sprite_direction_to_face_link(k, None) ^ 3;
        self.sprite_slot_mut(k).set_direction(dir);
        self.sprite_slot_mut(k).set_head_direction(dir);

        let follower = self.game_state.sprites.follower_runtime.indicator();
        self.follower_state_mut().set_indicator(1);
        self.load_follower_graphics();
        self.follower_state_mut().set_indicator(follower);

        if self.game_state.world.location.dungeon_room_index() == 0x12 {
            self.sprite_slot_mut(k).set_subtype2(2);
            if self.game_state.inventory.save_progress.progress_flags() & 4 == 0 {
                self.sprite_slot_mut(k).set_state(0);
            } else {
                let x = self.sprite_get_x(k).wrapping_add(6);
                let y = self.sprite_get_y(k).wrapping_add(15);
                self.sprite_set_x(k, x);
                self.sprite_set_y(k, y);
                self.sprite_slot_mut(k).set_flags4(3);
            }
        } else {
            self.sprite_slot_mut(k).set_subtype2(0);
            if self.game_state.sprites.follower_runtime.indicator() == 1
                || self.game_state.inventory.save_progress.progress_flags() & 4 != 0
            {
                self.sprite_slot_mut(k).set_state(0);
            }
        }
    }

    pub(super) fn sprite_prep_medallion_table(&mut self, k: usize) {
        self.sprite_slot_mut(k).increment_ignore_projectile();
        if self.game_state.world.location.overworld_screen_index() != 3 {
            self.sprite_slot_mut(k).add_x_low(8);
            if self.game_state.inventory.items.bombos() != 0 {
                self.sprite_slot_mut(k).set_graphics(4);
                self.sprite_slot_mut(k).set_ai_state(3);
            }
        } else if self.game_state.inventory.items.ether() != 0 {
            self.sprite_slot_mut(k).set_graphics(4);
            self.sprite_slot_mut(k).set_ai_state(3);
        }
    }

    pub(super) fn sprite_prep_eyegore(&mut self, k: usize) {
        let room = self.game_state.dungeon.room_tracking.room_index2();
        if room == 12 || room == 27 || room == 75 || room == 107 {
            self.sprite_slot_mut(k).increment_b();
            if self.sprite_slot_view(k).sprite_type() == 0x83 {
                self.sprite_slot_mut(k).set_deflection_bits(0);
            }
        }
    }

    fn sprite_return_if_boss_finished(&mut self, k: usize) -> bool {
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 != 0 {
            self.sprite_slot_mut(k).set_state(0);
            return true;
        }
        for j in (0..16).rev() {
            if SPRITE_INITIAL_BUMP_DAMAGE[self.sprite_slot_view(j).sprite_type() as usize] & 0x10
                == 0
            {
                self.sprite_slot_mut(j).set_state(0);
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
mod tests {
    use super::*;
    use crate::types::{read_le_u16, write_le_u16};

    fn fresh_state() -> Box<ZeldaState> {
        Box::new(ZeldaState::new())
    }

    #[test]
    fn simple_sprite_prep_offsets_and_flags_match_c() {
        let mut s = fresh_state();
        let k = 2;
        s.sprite_slot_mut(k).set_x_low(0xf9);
        s.sprite_slot_mut(k).set_y_low(0xfb);
        s.sprite_prep_mantle(k);
        assert_eq!(s.sprite_slot_view(k).x_low(), 1);
        assert_eq!(s.sprite_slot_view(k).y_low(), 0xfe);

        s.sprite_prep_move_down_8px_right8px(k);
        assert_eq!(s.sprite_slot_view(k).x_low(), 9);
        assert_eq!(s.sprite_slot_view(k).y_low(), 6);

        s.sprite_slot_mut(k).set_ignore_projectile(0xff);
        s.sprite_prep_ignore_projectiles(k);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 0);
    }

    #[test]
    fn dark_world_enemy_prep_uses_second_property_row() {
        let mut s = fresh_state();
        let k = 3;
        s.set_dark_world_region_index(1);
        s.sprite_prep_keese(k);
        assert_eq!(s.sprite_slot_view(k).bump_damage(), 0x85);
        assert_eq!(s.sprite_slot_view(k).health(), 4);
        assert_eq!(s.sprite_slot_view(k).flags5(), 7);

        s.sprite_prep_rope(k);
        assert_eq!(s.sprite_slot_view(k).bump_damage(), 5);
        assert_eq!(s.sprite_slot_view(k).health(), 8);
        assert_eq!(s.sprite_slot_view(k).flags5(), 7);
    }

    #[test]
    fn position_snapshot_prep_copies_low_high_coords() {
        let mut s = fresh_state();
        let k = 4;
        s.sprite_slot_mut(k).set_x_low(0x12);
        s.sprite_slot_mut(k).set_x_high(0x01);
        s.sprite_slot_mut(k).set_y_low(0x34);
        s.sprite_slot_mut(k).set_y_high(0x02);
        s.sprite_prep_kyameron(k);
        assert_eq!(s.sprite_slot_view(k).a(), 0x12);
        assert_eq!(s.sprite_slot_view(k).b(), 0x01);
        assert_eq!(s.sprite_slot_view(k).c(), 0x34);
        assert_eq!(s.sprite_slot_view(k).head_direction(), 0x02);
    }

    #[test]
    fn key_prep_consumes_item_drop_counter() {
        let mut s = fresh_state();
        let k = 5;
        s.sprite_battle_mut().set_item_drop_counter(0x7e);
        s.sprite_prep_small_key(k);
        assert_eq!(s.sprite_slot_view(k).subtype(), 0xff);
        assert_eq!(s.sprite_slot_view(k).die_action(), 0x7e);
        assert_eq!(s.ram[ITEM_DROP_COUNTER], 0x7f);

        s.sprite_prep_key_set_item_drop(k);
        assert_eq!(s.sprite_slot_view(k).die_action(), 0x7f);
        assert_eq!(s.ram[ITEM_DROP_COUNTER], 0x80);
    }

    #[test]
    fn flute_kid_prep_handles_light_and_dark_world_branches() {
        let mut light = fresh_state();
        let k = 6;
        light.inventory_items_mut().set_flute(2);
        light.sprite_slot_mut(k).set_state(9);
        light.sprite_prep_flute_kid(k);
        assert_eq!(light.sprite_slot_view(k).state(), 0);

        let mut dark = fresh_state();
        dark.save_progress_mut().set_dark_world_state(0x40);
        dark.save_progress_mut().set_progress_indicator_3(8);
        dark.sprite_slot_mut(k).set_x_low(10);
        dark.sprite_slot_mut(k).set_y_low(20);
        dark.sprite_prep_flute_kid(k);
        assert_eq!(dark.sprite_slot_view(k).subtype2(), 1);
        assert_eq!(dark.sprite_slot_view(k).graphics(), 3);
        assert_eq!(dark.sprite_slot_view(k).ai_state(), 5);
        assert_eq!(dark.sprite_slot_view(k).x_low(), 18);
        assert_eq!(dark.sprite_slot_view(k).y_low(), 12);
    }

    #[test]
    fn return_if_boss_finished_clears_non_boss_sprites_or_self_when_finished() {
        let mut s = fresh_state();
        for k in 0..16 {
            s.sprite_slot_mut(k).set_state(9);
            s.sprite_slot_mut(k).set_sprite_type(0);
        }
        s.sprite_slot_mut(3).set_sprite_type(9); // bump damage 0x13 keeps state.
        assert!(!s.sprite_return_if_boss_finished(2));
        assert_eq!(s.sprite_slot_view(0).state(), 0);
        assert_eq!(s.sprite_slot_view(3).state(), 9);

        let mut finished = fresh_state();
        finished.sprite_slot_mut(2).set_state(9);
        finished
            .dungeon_savegame_state_mut()
            .set_savegame_state_bits(0x8000);
        assert!(finished.sprite_return_if_boss_finished(2));
        assert_eq!(finished.sprite_slot_view(2).state(), 0);
    }

    #[test]
    fn room_lookup_prep_sets_subtype_and_ignore_projectile() {
        let mut s = fresh_state();
        let k = 7;
        s.set_dungeon_room_index(0x12);
        s.sprite_prep_storyteller(k);
        assert_eq!(s.sprite_slot_view(k).subtype2(), 2);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);

        s.set_dungeon_room_index(0x03);
        s.sprite_slot_mut(k).set_ignore_projectile(0);
        s.sprite_prep_adults(k);
        assert_eq!(s.sprite_slot_view(k).subtype2(), 0);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
    }

    #[test]
    fn rupee_pull_and_grate_shift_x_16_bit_left() {
        let mut s = fresh_state();
        let k = 8;
        s.sprite_set_x(k, 0x0104);
        s.sprite_prep_rupee_pull(k);
        assert_eq!(s.sprite_get_x(k), 0x00fc);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);

        s.sprite_set_x(k, 0x0004);
        s.set_overworld_event_info(0x58, 0x20);
        s.sprite_slot_mut(k).set_state(9);
        s.sprite_prep_thieves_town_grate(k);
        assert_eq!(s.sprite_get_x(k), 0xfffc);
        assert_eq!(s.sprite_slot_view(k).state(), 0);
    }

    #[test]
    fn boss_gated_prep_sets_expected_state_when_unfinished() {
        let mut s = fresh_state();
        let k = 10;
        s.sprite_slot_mut(k).set_x_low(0x20);
        s.sprite_slot_mut(k).set_y_low(0x30);
        s.set_dark_world_region_index(1);
        s.sprite_prep_agahnim(k);
        assert_eq!(s.sprite_slot_view(k).graphics(), 0);
        assert_eq!(s.sprite_slot_view(k).direction(), 3);
        assert_eq!(s.sprite_slot_view(k).oam_flags(), 7);
        assert_eq!(s.sprite_slot_view(k).x_low(), 0x28);
        assert_eq!(s.sprite_slot_view(k).y_low(), 0x38);

        s.sprite_prep_kholdstare(k);
        assert_eq!(s.sprite_slot_view(k).ai_state(), 3);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
    }

    #[test]
    fn armos_desert_and_big_spike_prep_update_state() {
        let mut s = fresh_state();
        let k = 11;
        s.sprite_slot_mut(k).set_x_low(0x2f);
        s.sprite_slot_mut(k).set_y_low(0x40);
        s.sprite_system_mut().set_limit_instance(5);
        s.sprite_prep_desert_statue(k);
        assert_eq!(s.sprite_slot_view(k).a(), 5);
        assert_eq!(s.game_state.sprites.system.limit_instance(), 6);
        assert_eq!(s.sprite_slot_view(k).direction(), 3); // after +8, x is now 0x37.

        s.sprite_prep_armos_knight(k);
        assert_eq!(s.sprite_slot_view(k).delay_main(), 255);
        assert_eq!(s.game_state.sprites.workspace.prep_shared_counter(), 1);

        s.sprite_slot_mut(k).set_x_low(0x10);
        s.sprite_slot_mut(k).set_x_high(1);
        s.sprite_slot_mut(k).set_y_low(0x20);
        s.sprite_slot_mut(k).set_y_high(2);
        s.sprite_prep_big_spike(k);
        assert_eq!(s.sprite_slot_view(k).a(), 0x18);
        assert_eq!(s.sprite_slot_view(k).b(), 1);
        assert_eq!(s.sprite_slot_view(k).c(), 0x28);
        assert_eq!(s.sprite_slot_view(k).head_direction(), 2);
    }

    #[test]
    fn barrier_catfish_and_mini_vitreous_prep_match_simple_branches() {
        let mut s = fresh_state();
        let k = 12;
        s.set_overworld_screen(5);
        s.set_overworld_event_info(5, 0x40);
        s.sprite_slot_mut(k).set_x_low(0x10);
        s.sprite_slot_mut(k).set_y_low(0x20);
        s.sprite_prep_agahnims_barrier(k);
        assert_eq!(s.sprite_slot_view(k).graphics(), 4);
        assert_eq!(s.sprite_slot_view(k).x_low(), 0x18);
        assert_eq!(s.sprite_slot_view(k).y_low(), 0x1c);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);

        s.sprite_slot_mut(k).set_x_low(0x30);
        s.sprite_slot_mut(k).set_y_low(0x40);
        s.sprite_slot_mut(k).set_ignore_projectile(0);
        s.sprite_prep_catfish(k);
        assert_eq!(s.sprite_slot_view(k).x_low(), 0x38);
        assert_eq!(s.sprite_slot_view(k).y_low(), 0x3c);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);

        s.sprite_slot_mut(k).set_state(9);
        s.dungeon_savegame_state_mut()
            .set_savegame_state_bits(0x8000);
        s.sprite_prep_mini_vitreous(k);
        assert_eq!(s.sprite_slot_view(k).state(), 0);

        let mut cutscene = fresh_state();
        cutscene.sprite_slot_mut(k).set_state(9);
        cutscene.sprite_set_x(k, 0x0100);
        cutscene.sprite_set_y(k, 0x0200);
        cutscene.sprite_prep_cutscene_agahnim(k);
        assert_eq!(cutscene.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(cutscene.sprite_get_x(k), 0x0108);
        assert_eq!(cutscene.sprite_get_y(k), 0x0206);
        assert_eq!(cutscene.sprite_slot_view(15).sprite_type(), 0xc1);
        assert_eq!(cutscene.sprite_slot_view(15).a(), 1);
        assert_eq!(cutscene.sprite_slot_view(15).ignore_projectile(), 1);
        assert_eq!(cutscene.sprite_get_x(15), 0x0108);
        assert_eq!(cutscene.sprite_slot_view(15).y_high(), 0x02);
        assert_eq!(cutscene.sprite_slot_view(15).y_low(), 0x2e);
        assert_eq!(cutscene.sprite_slot_view(15).flags2(), 0);
        assert_eq!(cutscene.sprite_slot_view(15).oam_flags(), 12);

        let mut cutscene_done = fresh_state();
        cutscene_done.sprite_slot_mut(k).set_state(9);
        cutscene_done
            .dungeon_savegame_state_mut()
            .set_savegame_state_bits(0x4000);
        cutscene_done.sprite_prep_cutscene_agahnim(k);
        assert_eq!(cutscene_done.sprite_slot_view(k).state(), 0);
        assert_eq!(cutscene_done.sprite_slot_view(15).state(), 0);
    }

    #[test]
    fn ganon_helmasaur_and_trinexx_prep_call_existing_initializers() {
        let mut ganon = fresh_state();
        let k = 13;
        ganon.sprite_slot_mut(k).set_direction(1);
        ganon.sprite_prep_ganon(k);
        assert_eq!(ganon.sprite_slot_view(k).delay_main(), 128);
        assert_eq!(ganon.sprite_slot_view(k).room(), 2);
        assert_eq!(ganon.game_state.system_signals.music_control(), 0x1e);

        let mut helmasaur = fresh_state();
        for i in 0..16 {
            helmasaur.ram[ALT_SPRITE_STATE_PREP + i] = 0xff;
        }
        helmasaur.sprite_prep_helmasaur_king(k);
        assert_eq!(
            &helmasaur.ram[ALT_SPRITE_STATE_PREP..ALT_SPRITE_STATE_PREP + 16],
            &[0; 16]
        );

        let mut trinexx_body = fresh_state();
        let k = 5;
        trinexx_body.sprite_slot_mut(k).set_sprite_type(0xcb);
        trinexx_body.sprite_slot_mut(k).set_x_low(0x20);
        trinexx_body.sprite_slot_mut(k).set_x_high(1);
        trinexx_body.sprite_slot_mut(k).set_y_low(0x30);
        trinexx_body.sprite_slot_mut(k).set_y_high(2);
        trinexx_body.overlord_slot_view_mut(0).set_y_high(0x0c);
        trinexx_body.overlord_slot_view_mut(0).set_gen2(0x97);
        trinexx_body.overlord_slot_view_mut(0).set_floor(0x01);
        trinexx_body.ram[ALT_SPRITE_STATE_PREP + 3] = 0xaa;
        trinexx_body.sprite_prep_trinexx(k);
        assert_eq!(trinexx_body.sprite_slot_view(k).a(), 0x28);
        assert_eq!(trinexx_body.sprite_slot_view(k).b(), 1);
        assert_eq!(trinexx_body.sprite_slot_view(k).c(), 0x40);
        assert_eq!(trinexx_body.sprite_slot_view(k).g(), 2);
        assert_eq!(trinexx_body.sprite_get_x(k), 0x0128);
        assert_eq!(trinexx_body.sprite_get_y(k), 0x024c);
        assert_eq!(
            trinexx_body
                .game_state
                .sprites
                .overlord_slots
                .slot(2)
                .x_low(),
            0
        );
        assert_eq!(
            trinexx_body
                .game_state
                .sprites
                .overlord_slots
                .slot(6)
                .x_low(),
            255
        );
        assert_eq!(trinexx_body.ram[OVERLORD_X_HI_PREP], 0);
        assert_eq!(trinexx_body.ram[OVERLORD_Y_HI_PREP], 0x0c);
        assert_eq!(trinexx_body.ram[OVERLORD_GEN2_PREP], 0x97);
        assert_eq!(trinexx_body.ram[OVERLORD_FLOOR_PREP], 0x01);
        assert_eq!(trinexx_body.ram[ALT_SPRITE_STATE_PREP + 3], 0);

        let mut trinexx_head = fresh_state();
        trinexx_head.sprite_slot_mut(k).set_sprite_type(0xcc);
        trinexx_head.sprite_slot_mut(k).set_x_low(0x44);
        trinexx_head.sprite_slot_mut(k).set_x_high(3);
        trinexx_head.sprite_slot_mut(k).set_y_low(0x55);
        trinexx_head.sprite_slot_mut(k).set_y_high(4);
        trinexx_head.ram[ALT_SPRITE_TYPE_PREP + 0x1a] = 0;
        trinexx_head.ram[ALT_SPRITE_X_HI_PREP + 0x1a] = 0xff;
        trinexx_head.ram[ALT_SPRITE_Y_HI_PREP + 0x1a] = 0xff;
        trinexx_head.sprite_prep_trinexx(k);
        assert_eq!(trinexx_head.sprite_slot_view(k).graphics(), 3);
        assert_eq!(trinexx_head.sprite_slot_view(k).delay_main(), 128);
        assert_eq!(trinexx_head.sprite_slot_view(k).subtype2(), 1);
        assert_eq!(trinexx_head.ram[ALT_SPRITE_TYPE_PREP + 0x1a], 0x40);
        assert_eq!(trinexx_head.ram[ALT_SPRITE_X_HI_PREP + 0x1a], 0);
        assert_eq!(trinexx_head.ram[ALT_SPRITE_Y_HI_PREP + 0x1a], 0);
        assert_eq!(trinexx_head.sprite_slot_view(k).a(), 0x44);
        assert_eq!(trinexx_head.sprite_slot_view(k).g(), 4);
    }

    #[test]
    fn moldorm_and_chainchomp_history_buffers_are_seeded_from_sprite_position() {
        let mut s = fresh_state();
        let k = 2;
        s.sprite_slot_mut(k).set_x_low(0x44);
        s.sprite_slot_mut(k).set_x_high(0x01);
        s.sprite_slot_mut(k).set_y_low(0x55);
        s.sprite_slot_mut(k).set_y_high(0x02);
        s.sprite_prep_mini_moldorm_bounce(k);
        let base = 32 * k;
        assert_eq!(s.ram[MOLDORM_X_LO_PREP + base], 0x44);
        assert_eq!(s.ram[MOLDORM_X_HI_PREP + base + 31], 0x01);
        assert_eq!(s.ram[MOLDORM_Y_LO_PREP + base + 15], 0x55);
        assert_eq!(s.ram[MOLDORM_Y_HI_PREP + base + 31], 0x02);

        s.sprite_workspace_mut().set_current_sprite_x(0x1234);
        s.sprite_workspace_mut().set_current_sprite_y(0x5678);
        s.sprite_prep_chainchomp_bounce(k);
        let hist = k * 8;
        assert_eq!(
            read_le_u16(&s.ram, CHAINCHOMP_X_HIST_PREP + hist * 2),
            0x1234
        );
        assert_eq!(
            read_le_u16(&s.ram, CHAINCHOMP_Y_HIST_PREP + (hist + 5) * 2),
            0x5678
        );
        assert_eq!(s.sprite_slot_view(k).a(), 0x44);
        assert_eq!(s.sprite_slot_view(k).g(), 0x02);

        let mut leash = fresh_state();
        leash.sprite_workspace_mut().set_current_sprite_x(0x0100);
        leash.sprite_workspace_mut().set_current_sprite_y(0x0200);
        leash.chain_chomp_history_mut().set_x(hist + 1, 0x0120);
        leash.chain_chomp_history_mut().set_y(hist + 1, 0x01e0);
        leash.chain_chomp_handle_leash(k);
        assert_eq!(
            read_le_u16(&leash.ram, CHAINCHOMP_X_HIST_PREP + hist * 2),
            0x0100
        );
        assert_eq!(
            read_le_u16(&leash.ram, CHAINCHOMP_Y_HIST_PREP + hist * 2),
            0x0200
        );
        assert_eq!(
            read_le_u16(&leash.ram, CHAINCHOMP_X_HIST_PREP + (hist + 1) * 2),
            0x0108
        );
        assert_eq!(
            read_le_u16(&leash.ram, CHAINCHOMP_Y_HIST_PREP + (hist + 1) * 2),
            0x01f8
        );

        let mut moving_chain = fresh_state();
        moving_chain.sprite_slot_mut(k).set_a(0x00);
        moving_chain.sprite_slot_mut(k).set_b(0x01);
        moving_chain.sprite_slot_mut(k).set_c(0x00);
        moving_chain.sprite_slot_mut(k).set_g(0x02);
        moving_chain.chain_chomp_history_mut().set_x(hist, 0x0110);
        moving_chain.chain_chomp_history_mut().set_y(hist, 0x0220);
        moving_chain
            .chain_chomp_history_mut()
            .set_x(hist + 1, 0x0100);
        moving_chain
            .chain_chomp_history_mut()
            .set_y(hist + 1, 0x0230);
        moving_chain.chain_chomp_move_chain(k);
        assert_eq!(
            read_le_u16(&moving_chain.ram, CHAINCHOMP_X_HIST_PREP + (hist + 1) * 2),
            0x0101
        );
        assert_eq!(
            read_le_u16(&moving_chain.ram, CHAINCHOMP_Y_HIST_PREP + (hist + 1) * 2),
            0x022f
        );
    }

    #[test]
    fn bonk_big_key_and_purple_chest_prep_match_state_gates() {
        let mut outdoor = fresh_state();
        let k = 3;
        outdoor.sprite_prep_bonk_item(k);
        assert_eq!(outdoor.sprite_slot_view(k).graphics(), 2);

        let mut indoor = fresh_state();
        indoor.set_indoor_flag(1);
        indoor.sprite_battle_mut().set_item_drop_counter(1);
        indoor.sprite_slot_mut(k).set_graphics(4);
        indoor
            .dungeon_savegame_state_mut()
            .set_savegame_state_bits(0x2000);
        indoor.sprite_slot_mut(k).set_state(9);
        indoor.sprite_prep_bonk_item(k);
        assert_eq!(indoor.sprite_slot_view(k).floor(), 2);
        assert_eq!(indoor.sprite_slot_view(k).die_action(), 1);
        assert_eq!(indoor.sprite_slot_view(k).state(), 0);
        assert_eq!(indoor.sprite_slot_view(k).graphics(), 5);
        assert_eq!(indoor.sprite_slot_view(k).oam_flags(), 8);
        assert_eq!(indoor.sprite_slot_view(k).flags3() & 0x20, 0x20);

        let mut key = fresh_state();
        key.sprite_slot_mut(k).set_x_low(0x20);
        key.sprite_battle_mut().set_item_drop_counter(7);
        key.sprite_prep_big_key(k);
        assert_eq!(key.sprite_slot_view(k).x_low(), 0x28);
        assert_eq!(key.sprite_slot_view(k).subtype(), 0xff);
        assert_eq!(key.sprite_slot_view(k).die_action(), 7);
        assert_eq!(key.ram[ITEM_DROP_COUNTER], 8);

        let mut chest = fresh_state();
        chest.save_progress_mut().set_progress_indicator_3(32);
        chest.sprite_prep_purple_chest(k);
        assert_eq!(chest.sprite_slot_view(k).ignore_projectile(), 1);
        chest.follower_state_mut().set_indicator(12);
        chest.sprite_slot_mut(k).set_state(9);
        chest.sprite_prep_purple_chest(k);
        assert_eq!(chest.sprite_slot_view(k).state(), 0);
    }

    #[test]
    fn smithy_prep_matches_world_and_progress_gates() {
        let k = 6;

        let mut dark_waiting = fresh_state();
        dark_waiting.save_progress_mut().set_dark_world_state(0x40);
        dark_waiting.sprite_slot_mut(k).set_state(9);
        dark_waiting.sprite_prep_smithy(k);
        assert_eq!(dark_waiting.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(dark_waiting.sprite_slot_view(k).subtype2(), 2);
        assert_eq!(dark_waiting.sprite_slot_view(k).state(), 9);

        let mut dark_done = fresh_state();
        dark_done.save_progress_mut().set_dark_world_state(0x40);
        dark_done.save_progress_mut().set_progress_indicator_3(32);
        dark_done.sprite_slot_mut(k).set_state(9);
        dark_done.sprite_prep_smithy(k);
        assert_eq!(dark_done.sprite_slot_view(k).state(), 0);

        let mut light_alone = fresh_state();
        light_alone.sprite_slot_mut(k).set_state(9);
        light_alone.sprite_set_x(k, 0x0100);
        light_alone.sprite_set_y(k, 0x0200);
        light_alone.sprite_prep_smithy(k);
        assert_eq!(light_alone.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(light_alone.sprite_get_x(k), 0x0102);
        assert_eq!(light_alone.sprite_get_y(k), 0x02fd);
        assert_eq!(light_alone.sprite_slot_view(15).sprite_type(), 0x31);
        assert_eq!(light_alone.sprite_get_x(15), 0x0100);
        assert_eq!(light_alone.sprite_get_y(15), 0x0200);
        assert_eq!(light_alone.sprite_slot_view(15).subtype2(), 1);
        assert_eq!(light_alone.sprite_slot_view(15).ignore_projectile(), 1);

        let mut light_reunited = fresh_state();
        light_reunited.sprite_slot_mut(k).set_state(9);
        light_reunited
            .save_progress_mut()
            .set_progress_indicator_3(0xa0);
        light_reunited.sprite_set_x(k, 0x0100);
        light_reunited.sprite_set_y(k, 0x0200);
        light_reunited.sprite_prep_smithy(k);
        assert_eq!(light_reunited.sprite_slot_view(15).sprite_type(), 0x31);
        assert_eq!(light_reunited.sprite_slot_view(14).sprite_type(), 0x1a);
        assert_eq!(light_reunited.sprite_get_x(14), 0x012e);
        assert_eq!(light_reunited.sprite_get_y(14), 0x02fd);
        assert_eq!(light_reunited.sprite_slot_view(14).direction(), 1);
        assert_eq!(light_reunited.sprite_slot_view(14).a(), 4);
        assert_eq!(light_reunited.sprite_slot_view(14).ignore_projectile(), 4);
        assert_eq!(light_reunited.sprite_slot_view(13).sprite_type(), 0x31);
        assert_eq!(light_reunited.sprite_get_x(13), 0x012e);
        assert_eq!(light_reunited.sprite_get_y(13), 0x02fd);
        assert_eq!(light_reunited.sprite_slot_view(14).e(), k as u8);
        assert_eq!(light_reunited.sprite_slot_view(k).e(), 14);
        assert_eq!(light_reunited.sprite_slot_view(k).ai_state(), 5);
        assert_eq!(light_reunited.sprite_slot_view(14).ai_state(), 5);
    }

    #[test]
    fn lanmolas_moldorm_and_tektite_prep_initialize_state() {
        let mut s = fresh_state();
        let k = 1;
        s.sprite_slot_mut(k).set_x_low(0x66);
        s.sprite_slot_mut(k).set_x_high(0x03);
        s.sprite_slot_mut(k).set_y_low(0x77);
        s.sprite_slot_mut(k).set_y_high(0x04);
        s.sprite_prep_moldorm(k);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(s.ram[MOLDORM_X_LO_PREP], 0x66);
        assert_eq!(s.ram[MOLDORM_Y_HI_PREP + 127], 0x04);

        let mut lanmolas = fresh_state();
        let k = 2;
        lanmolas.sprite_prep_lanmolas(k);
        assert_eq!(lanmolas.sprite_slot_view(k).delay_main(), 255);
        assert_eq!(lanmolas.sprite_slot_view(k).z(), 0xff);
        assert_eq!(lanmolas.ram[BEAMOS_X_HI + k * 0x40], 0xff);
        assert_eq!(lanmolas.ram[BEAMOS_X_HI + k * 0x40 + 63], 0xff);
        assert_eq!(lanmolas.garnish_slot_view(k).y_low(), 7);

        let mut shrapnel = fresh_state();
        shrapnel.sprite_slot_mut(k).set_state(9);
        shrapnel.sprite_set_x(k, 0x0120);
        shrapnel.sprite_set_y(k, 0x0340);
        shrapnel.lanmola_spawn_shrapnel(k);
        assert_eq!(shrapnel.game_state.scratch_counter.value(), 0xff);
        assert_eq!(shrapnel.sprite_slot_view(15).sprite_type(), 0xc2);
        assert_eq!(shrapnel.sprite_get_x(15), 0x0124);
        assert_eq!(shrapnel.sprite_get_y(15), 0x0344);
        assert_eq!(shrapnel.sprite_slot_view(15).ignore_projectile(), 1);
        assert_eq!(shrapnel.sprite_slot_view(15).bump_damage(), 1);
        assert_eq!(shrapnel.sprite_slot_view(15).flags4(), 1);
        assert_eq!(shrapnel.sprite_slot_view(15).z(), 0);
        assert_eq!(shrapnel.sprite_slot_view(15).flags2(), 0x20);
        assert_eq!(shrapnel.sprite_slot_view(15).x_velocity(), 0);
        assert_eq!(shrapnel.sprite_slot_view(15).y_velocity(), (-36i8) as u8);
        assert_eq!(shrapnel.sprite_slot_view(15).graphics(), 0);
        assert_eq!(shrapnel.sprite_slot_view(8).sprite_type(), 0xc2);
        assert_eq!(shrapnel.sprite_slot_view(8).x_velocity(), (-28i8) as u8);
        assert_eq!(shrapnel.sprite_slot_view(8).y_velocity(), 28);

        let mut short_shrapnel = fresh_state();
        short_shrapnel.sprite_slot_mut(0).set_state(9);
        short_shrapnel.sprite_slot_mut(1).set_state(9);
        short_shrapnel.sprite_slot_mut(2).set_state(9);
        short_shrapnel.sprite_slot_mut(k).set_state(9);
        short_shrapnel.sprite_set_x(k, 0x0050);
        short_shrapnel.sprite_set_y(k, 0x0060);
        short_shrapnel.lanmola_spawn_shrapnel(k);
        assert_eq!(short_shrapnel.sprite_slot_view(15).sprite_type(), 0xc2);
        assert_eq!(short_shrapnel.sprite_slot_view(15).x_velocity(), 28);
        assert_eq!(
            short_shrapnel.sprite_slot_view(15).y_velocity(),
            (-28i8) as u8
        );
        assert_eq!(short_shrapnel.sprite_slot_view(12).sprite_type(), 0xc2);
        assert_eq!(short_shrapnel.sprite_slot_view(11).sprite_type(), 0);

        let mut tektite = fresh_state();
        let k = 4;
        tektite.sprite_slot_mut(k).set_x_low(0x10);
        tektite.sprite_prep_tektite(k);
        assert_eq!(tektite.sprite_slot_view(k).a(), 1);
        assert_eq!(tektite.sprite_slot_view(k).oam_flags(), 7);
        assert_eq!(tektite.sprite_slot_view(k).health(), 12);
        assert_eq!(tektite.sprite_slot_view(k).bump_damage(), 5);
        assert_eq!(tektite.sprite_slot_view(k).z_velocity(), 32);
        assert_eq!(tektite.sprite_slot_view(k).ai_state(), 1);
    }

    #[test]
    fn snitch_running_man_and_mushroom_prep_match_simple_gates() {
        let k = 6;

        let mut snitch = fresh_state();
        snitch.sprite_slot_mut(k).set_x_low(0x34);
        snitch.sprite_slot_mut(k).set_x_high(0x12);
        snitch.sprite_prep_snitches(k);
        assert_eq!(snitch.sprite_slot_view(k).direction(), 2);
        assert_eq!(snitch.sprite_slot_view(k).head_direction(), 2);
        assert_eq!(snitch.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(snitch.sprite_slot_view(k).a(), 0x34);
        assert_eq!(snitch.sprite_slot_view(k).b(), 0x12);
        assert_eq!(snitch.sprite_slot_view(k).x_velocity(), (-9i8) as u8);

        let mut bounce = fresh_state();
        bounce.sprite_slot_mut(k).set_x_low(0x55);
        bounce.sprite_prep_snitch_bounce_2(k);
        assert_eq!(bounce.sprite_slot_view(k).a(), 0x55);
        bounce.sprite_prep_snitch_bounce_3(k);
        assert_eq!(bounce.sprite_slot_view(k).ignore_projectile(), 2);

        let mut runner = fresh_state();
        runner.sprite_prep_running_man(k);
        assert_eq!(runner.sprite_slot_view(k).direction(), 2);
        assert_eq!(runner.sprite_slot_view(k).head_direction(), 2);
        assert_eq!(runner.sprite_slot_view(k).ignore_projectile(), 1);

        let mut mushroom = fresh_state();
        mushroom.inventory_items_mut().set_mushroom(1);
        mushroom.sprite_slot_mut(k).set_graphics(7);
        mushroom.sprite_prep_mushroom(k);
        assert_eq!(mushroom.sprite_slot_view(k).graphics(), 0);
        assert_eq!(mushroom.sprite_slot_view(k).oam_flags() & 8, 8);
        assert_eq!(mushroom.sprite_slot_view(k).ignore_projectile(), 1);

        mushroom.inventory_items_mut().set_mushroom(2);
        mushroom.sprite_slot_mut(k).set_state(9);
        mushroom.sprite_prep_mushroom(k);
        assert_eq!(mushroom.sprite_slot_view(k).state(), 0);
    }

    #[test]
    fn potion_shop_prep_spawns_powder_and_cauldrons_with_barrier_flags() {
        let k = 4;
        let mut s = fresh_state();
        s.sprite_slot_mut(k).set_state(9);
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        s.set_flag_overworld_area_changed(1);
        s.inventory_items_mut().set_mushroom(1);
        s.save_progress_mut().set_dungeon_info_word(0x109, 0x80);

        s.sprite_prep_potion_shop(k);

        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
        for (slot, subtype, x, y) in [
            (15, 1, 0x0100u16.wrapping_sub(16), 0x0200),
            (
                14,
                2,
                0x0100u16.wrapping_sub(40),
                0x0200u16.wrapping_sub(72),
            ),
            (13, 3, 0x0100u16.wrapping_add(8), 0x0200u16.wrapping_sub(72)),
            (
                12,
                4,
                0x0100u16.wrapping_sub(88),
                0x0200u16.wrapping_sub(72),
            ),
        ] {
            assert_eq!(s.sprite_slot_view(slot).state(), 9);
            assert_eq!(s.sprite_slot_view(slot).sprite_type(), 0xe9);
            assert_eq!(s.sprite_slot_view(slot).subtype2(), subtype);
            assert_eq!(s.sprite_get_x(slot), x);
            assert_eq!(s.sprite_get_y(slot), y);
            assert_eq!(s.sprite_slot_view(slot).flags4(), 3);
            assert_eq!(s.sprite_slot_view(slot).deflection_bits() & 0x20, 0x20);
        }

        let mut skipped_powder = fresh_state();
        skipped_powder.sprite_slot_mut(k).set_state(9);
        skipped_powder.clear_flag_overworld_area_changed();
        skipped_powder.inventory_items_mut().set_mushroom(1);
        skipped_powder
            .save_progress_mut()
            .set_dungeon_info_word(0x109, 0x80);
        skipped_powder.sprite_prep_potion_shop(k);
        assert_eq!(skipped_powder.sprite_slot_view(15).subtype2(), 2);
        assert_eq!(skipped_powder.sprite_slot_view(14).subtype2(), 3);
        assert_eq!(skipped_powder.sprite_slot_view(13).subtype2(), 4);
        assert_eq!(skipped_powder.sprite_slot_view(12).state(), 0);
    }

    #[test]
    fn arrow_game_prep_seeds_archery_sprites_from_link_state() {
        let k = 0;
        let mut s = fresh_state();
        s.sprite_slot_mut(k).set_y_low(0x30);
        s.ram[ARCHERY_GAME_HIT_COUNTER] = 0xaa;
        s.follower_link_state_mut().set_position(0x1200, 0x3400);
        s.follower_link_state_mut().mark_lower_level();
        s.player_resources_mut().set_arrows(17);

        s.sprite_prep_arrow_game_bounce(k);

        assert_eq!(s.ram[ARCHERY_GAME_HIT_COUNTER], 0);
        assert_eq!(s.sprite_slot_view(k).y_low(), 0x27);
        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(s.sprite_slot_view(k).subtype(), 17);

        assert_eq!(s.sprite_slot_view(1).sprite_type(), 0x65);
        assert_eq!(s.sprite_slot_view(1).state(), 9);
        assert_eq!(s.sprite_slot_view(1).x_high(), 0x12);
        assert_eq!(s.sprite_slot_view(1).x_low(), 0x40);
        assert_eq!(s.sprite_slot_view(1).y_high(), 0x34);
        assert_eq!(s.sprite_slot_view(1).y_low(), 0x4f);
        assert_eq!(s.sprite_slot_view(1).a(), 1);
        assert_eq!(s.sprite_slot_view(1).graphics(), 0);
        assert_eq!(s.sprite_slot_view(1).x_velocity(), (-8i8) as u8);
        assert_eq!(s.sprite_slot_view(1).flags4(), 0x1c);
        assert_eq!(s.sprite_slot_view(1).oam_flags(), 13);
        assert_eq!(s.sprite_slot_view(1).floor(), 1);

        assert_eq!(s.sprite_slot_view(7).x_low(), 0xc0);
        assert_eq!(s.sprite_slot_view(7).y_low(), 0x5a);
        assert_eq!(s.sprite_slot_view(7).a(), 2);
        assert_eq!(s.sprite_slot_view(7).graphics(), 1);
        assert_eq!(s.sprite_slot_view(7).x_velocity(), 12);
        assert_eq!(s.sprite_slot_view(7).flags4(), 0x15);
    }

    #[test]
    fn heart_upgrade_prep_clears_already_obtained_entries() {
        let k = 4;

        let mut overworld = fresh_state();
        overworld.sprite_slot_mut(k).set_state(9);
        overworld.set_overworld_screen(0x22);
        overworld.set_overworld_event_info(0x22, 0x40);
        overworld.sprite_prep_heart_container(k);
        assert_eq!(overworld.sprite_slot_view(k).state(), 0);
        overworld.set_overworld_event_info(0x22, 0x10);
        overworld.heart_upgrade_set_obtained_flag(k);
        assert_eq!(
            overworld
                .game_state
                .world
                .overworld
                .event_info
                .event_info(0x22),
            0x50
        );

        let mut lumberjack = fresh_state();
        lumberjack.sprite_slot_mut(k).set_state(9);
        lumberjack.set_overworld_screen(0x3b);
        lumberjack.set_overworld_event_info(0x3b, 0);
        lumberjack.sprite_prep_heart_piece(k);
        assert_eq!(lumberjack.sprite_slot_view(k).state(), 0);

        let mut dungeon = fresh_state();
        dungeon.set_indoor_flag(1);
        dungeon.sprite_slot_mut(k).set_state(9);
        dungeon.sprite_slot_mut(k).set_x_high(0);
        dungeon
            .dungeon_savegame_state_mut()
            .set_savegame_state_bits(0x4000);
        dungeon.heart_upgrade_check_if_already_obtained(k);
        assert_eq!(dungeon.sprite_slot_view(k).state(), 0);
        dungeon
            .dungeon_savegame_state_mut()
            .set_savegame_state_bits(0x0001);
        dungeon.heart_upgrade_set_obtained_flag(k);
        assert_eq!(
            dungeon
                .game_state
                .dungeon
                .savegame_state
                .savegame_state_bits(),
            0x4001
        );

        dungeon.sprite_slot_mut(k).set_x_high(1);
        dungeon
            .dungeon_savegame_state_mut()
            .set_savegame_state_bits(0x0002);
        dungeon.heart_upgrade_set_obtained_flag(k);
        assert_eq!(
            dungeon
                .game_state
                .dungeon
                .savegame_state
                .savegame_state_bits(),
            0x2002
        );

        let mut untouched = fresh_state();
        untouched.sprite_slot_mut(k).set_state(9);
        untouched.set_overworld_screen(0x11);
        untouched.heart_upgrade_check_if_already_obtained(k);
        assert_eq!(untouched.sprite_slot_view(k).state(), 9);
    }

    #[test]
    fn swamola_prep_initializes_segment_history_and_position_snapshot() {
        let k = 2;
        let mut buggy = fresh_state();
        buggy.sprite_slot_mut(k).set_x_low(0x44);
        buggy.sprite_slot_mut(k).set_x_high(0x01);
        buggy.sprite_slot_mut(k).set_y_low(0x88);
        buggy.sprite_slot_mut(k).set_y_high(0x02);
        buggy.sprite_prep_swamola(k);
        let buggy_start = 0x03;
        assert_eq!(buggy.ram[SWAMOLA_X_LO_PREP + buggy_start], 0x44);
        assert_eq!(buggy.ram[SWAMOLA_X_HI_PREP + buggy_start + 31], 0x01);
        assert_eq!(buggy.ram[SWAMOLA_Y_LO_PREP + buggy_start], 0x88);
        assert_eq!(buggy.ram[SWAMOLA_Y_HI_PREP + buggy_start + 31], 0x02);
        assert_eq!(buggy.sprite_slot_view(k).a(), 0x44);
        assert_eq!(buggy.sprite_slot_view(k).b(), 0x01);
        assert_eq!(buggy.sprite_slot_view(k).c(), 0x88);
        assert_eq!(buggy.sprite_slot_view(k).head_direction(), 0x02);

        let mut fixed = fresh_state();
        fixed
            .enhanced_features_mut()
            .set_bits(FEATURE_MISC_BUG_FIXES_PREP);
        fixed.sprite_slot_mut(k).set_x_low(0x77);
        fixed.sprite_slot_mut(k).set_x_high(0x03);
        fixed.sprite_slot_mut(k).set_y_low(0x99);
        fixed.sprite_slot_mut(k).set_y_high(0x04);
        fixed.sprite_prep_swamola_initialize_segments(k);
        let fixed_start = k * 32;
        assert_eq!(fixed.ram[SWAMOLA_X_LO_PREP + fixed_start], 0x77);
        assert_eq!(fixed.ram[SWAMOLA_X_HI_PREP + fixed_start + 31], 0x03);
        assert_eq!(fixed.ram[SWAMOLA_Y_LO_PREP + fixed_start], 0x99);
        assert_eq!(fixed.ram[SWAMOLA_Y_HI_PREP + fixed_start + 31], 0x04);
    }

    #[test]
    fn blind_maiden_and_old_man_prep_follow_follower_gates() {
        let k = 5;

        let mut maiden = fresh_state();
        maiden.sprite_slot_mut(k).set_state(9);
        maiden.follower_state_mut().set_indicator(0);
        maiden.follower_state_mut().set_dropped(0x80);
        maiden.follower_state_mut().set_appearance_none_flag(7);
        maiden.sprite_prep_blind_maiden(k);
        assert_eq!(maiden.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(maiden.game_state.sprites.follower_runtime.indicator(), 0);
        assert_eq!(maiden.game_state.sprites.follower_runtime.dropped(), 0);
        assert_eq!(
            maiden
                .game_state
                .sprites
                .follower_runtime
                .appearance_none_flag(),
            0
        );
        assert_eq!(maiden.sprite_slot_view(k).state(), 9);

        let mut maiden_finished = fresh_state();
        maiden_finished.sprite_slot_mut(k).set_state(9);
        maiden_finished
            .save_progress_mut()
            .set_dungeon_info_word(0xac, 0x0800);
        maiden_finished.sprite_prep_blind_maiden(k);
        assert_eq!(maiden_finished.sprite_slot_view(k).state(), 0);

        let mut old_man_room = fresh_state();
        old_man_room.set_dungeon_room_index(0xe4);
        old_man_room.sprite_prep_old_man_bounce(k);
        assert_eq!(old_man_room.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(old_man_room.sprite_slot_view(k).subtype2(), 2);

        let mut old_man_mirror = fresh_state();
        old_man_mirror.inventory_items_mut().set_mirror(2);
        old_man_mirror.sprite_slot_mut(k).set_state(9);
        old_man_mirror.sprite_prep_old_man_bounce(k);
        assert_eq!(old_man_mirror.sprite_slot_view(k).state(), 0);
        assert_eq!(
            old_man_mirror
                .game_state
                .sprites
                .follower_runtime
                .indicator(),
            0
        );

        let mut old_man_followed = fresh_state();
        old_man_followed.follower_state_mut().set_indicator(1);
        old_man_followed.sprite_slot_mut(k).set_state(9);
        old_man_followed.sprite_prep_old_man_bounce(k);
        assert_eq!(old_man_followed.sprite_slot_view(k).state(), 0);
        assert_eq!(
            old_man_followed
                .game_state
                .sprites
                .follower_runtime
                .indicator(),
            1
        );
    }

    #[test]
    fn zelda_bounce_prep_matches_sword_room_and_progress_gates() {
        let k = 6;

        let mut has_sword = fresh_state();
        has_sword.inventory_items_mut().set_sword_type(2);
        has_sword.sprite_slot_mut(k).set_state(9);
        has_sword.sprite_prep_zelda_bounce(k);
        assert_eq!(has_sword.sprite_slot_view(k).state(), 0);

        let mut cell = fresh_state();
        cell.sprite_slot_mut(k).set_state(9);
        cell.set_dungeon_room_index(0x12);
        cell.save_progress_mut().set_progress_flags(4);
        cell.follower_state_mut().set_indicator(7);
        cell.sprite_set_x(k, 0x0100);
        cell.sprite_set_y(k, 0x0200);
        cell.follower_link_state_mut().set_x(0x0180);
        cell.follower_link_state_mut().set_y(0x0200);
        cell.sprite_prep_zelda_bounce(k);
        assert_eq!(cell.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(cell.sprite_slot_view(k).direction(), 3);
        assert_eq!(cell.sprite_slot_view(k).head_direction(), 3);
        assert_eq!(cell.game_state.sprites.follower_runtime.indicator(), 7);
        assert_eq!(cell.sprite_slot_view(k).subtype2(), 2);
        assert_eq!(cell.sprite_get_x(k), 0x0106);
        assert_eq!(cell.sprite_get_y(k), 0x020f);
        assert_eq!(cell.sprite_slot_view(k).flags4(), 3);
        assert_eq!(cell.sprite_slot_view(k).state(), 9);

        let mut not_rescued = fresh_state();
        not_rescued.sprite_slot_mut(k).set_state(9);
        not_rescued.set_dungeon_room_index(0x12);
        not_rescued.save_progress_mut().set_progress_flags(0);
        not_rescued.sprite_prep_zelda_bounce(k);
        assert_eq!(not_rescued.sprite_slot_view(k).state(), 0);

        let mut follower_present = fresh_state();
        follower_present.sprite_slot_mut(k).set_state(9);
        follower_present.set_dungeon_room_index(0x20);
        follower_present.follower_state_mut().set_indicator(1);
        follower_present.sprite_prep_zelda_bounce(k);
        assert_eq!(follower_present.sprite_slot_view(k).subtype2(), 0);
        assert_eq!(follower_present.sprite_slot_view(k).state(), 0);
    }

    #[test]
    fn bomb_shoppe_prep_spawns_visible_bombs_and_big_bomb_when_unlocked() {
        let k = 2;
        let mut s = fresh_state();
        s.sprite_slot_mut(k).set_state(9);
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0230);
        s.player_resources_mut().set_crystal_flags(5);
        s.save_progress_mut().set_progress_indicator_3(32);

        s.sprite_prep_bomb_shoppe(k);

        assert_eq!(s.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(s.sprite_slot_view(15).state(), 9);
        assert_eq!(s.sprite_slot_view(15).sprite_type(), 0xb5);
        assert_eq!(s.sprite_get_x(15), 0x0120u16.wrapping_sub(24));
        assert_eq!(s.sprite_get_y(15), 0x0230u16.wrapping_sub(24));
        assert_eq!(s.sprite_slot_view(15).subtype2(), 1);
        assert_eq!(s.sprite_slot_view(15).ignore_projectile(), 1);
        assert_eq!(s.sprite_slot_view(14).state(), 9);
        assert_eq!(s.sprite_slot_view(14).sprite_type(), 0xb5);
        assert_eq!(s.sprite_get_x(14), 0x0120u16.wrapping_sub(56));
        assert_eq!(s.sprite_get_y(14), 0x0230u16.wrapping_sub(24));
        assert_eq!(s.sprite_slot_view(14).subtype2(), 2);
        assert_eq!(s.sprite_slot_view(14).ignore_projectile(), 2);

        let mut locked = fresh_state();
        locked.sprite_slot_mut(k).set_state(9);
        locked.sprite_set_x(k, 0x0040);
        locked.sprite_set_y(k, 0x0050);
        locked.player_resources_mut().set_crystal_flags(4);
        locked.save_progress_mut().set_progress_indicator_3(32);
        locked.sprite_prep_bomb_shoppe(k);
        assert_eq!(locked.sprite_slot_view(15).state(), 9);
        assert_eq!(locked.sprite_slot_view(14).state(), 0);
    }

    #[test]
    fn bomb_shop_clerk_exhalation_spawns_huff_with_exact_state() {
        let k = 2;
        let mut s = fresh_state();
        s.sprite_slot_mut(k).set_state(9);
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0230);
        s.sprite_slot_mut(k).set_z(9);
        s.sprite_slot_mut(15).set_flags3(0xff);

        s.bomb_shop_clerk_exhalation(k);

        assert_eq!(s.sprite_slot_view(15).state(), 9);
        assert_eq!(s.sprite_slot_view(15).sprite_type(), 0xb5);
        assert_eq!(s.sprite_get_x(15), 0x0124);
        assert_eq!(s.sprite_get_y(15), 0x0240);
        assert_eq!(s.sprite_slot_view(15).subtype2(), 3);
        assert_eq!(s.sprite_slot_view(15).ignore_projectile(), 3);
        assert_eq!(s.sprite_slot_view(15).z(), 4);
        assert_eq!(s.sprite_slot_view(15).z_velocity(), (-12i8) as u8);
        assert_eq!(s.sprite_slot_view(15).delay_main(), 23);
        assert_eq!(s.sprite_slot_view(15).flags3() & 0x11, 0);
    }

    #[test]
    fn bomb_shop_clerk_exhalation_noops_when_no_spawn_slot_exists() {
        let k = 2;
        let mut s = fresh_state();
        for slot in 0..16 {
            s.sprite_slot_mut(slot).set_state(9);
            s.sprite_slot_mut(slot).set_sprite_type(0xa0 + slot as u8);
        }
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        let before = s.sprite_slot_view(15).sprite_type();

        s.bomb_shop_clerk_exhalation(k);

        assert_eq!(s.sprite_slot_view(15).sprite_type(), before);
        assert_eq!(s.sprite_get_x(15), 0);
        assert_eq!(s.sprite_get_y(15), 0);
    }

    #[test]
    fn archery_game_guy_show_msg_sets_message_module_and_clears_delay() {
        let k = 4;
        let mut s = fresh_state();
        s.ram[TILE_INTERACTION_SHARED_FLAG] = 7;
        s.ram[MESSAGING_MODULE] = 9;
        s.set_submodule(1);
        s.set_main_module(3);
        s.clear_saved_module_for_menu();
        s.sprite_slot_mut(k).set_delay_main(88);

        s.archery_game_guy_show_msg(k, 0x86);

        assert_eq!(s.game_state.messaging.dialogue_message_index.value(), 0x86);
        assert_eq!(s.ram[TILE_INTERACTION_SHARED_FLAG], 0);
        assert_eq!(s.ram[MESSAGING_MODULE], 0);
        assert_eq!(s.game_state.frame.submodule, 2);
        assert_eq!(s.game_state.frame.saved_module_for_menu, 3);
        assert_eq!(s.game_state.frame.main_module, 14);
        assert_eq!(s.sprite_slot_view(k).delay_main(), 0);
    }

    #[test]
    fn debirando_prep_spawns_pit_pair_and_fire_variant_reloads_properties() {
        let k = 3;
        let mut pit = fresh_state();
        pit.sprite_slot_mut(k).set_state(9);
        pit.sprite_slot_mut(k).set_g(0);
        pit.sprite_slot_mut(k).set_delay_main(7);
        pit.sprite_slot_mut(k).set_graphics(2);
        pit.sprite_slot_mut(k).set_x_low(0x70);
        pit.sprite_slot_mut(k).set_y_low(0x80);

        pit.sprite_prep_debirando_pit(k);

        assert_eq!(pit.sprite_slot_view(k).g(), 1);
        assert_eq!(pit.sprite_slot_view(k).delay_main(), 0);
        assert_eq!(pit.sprite_slot_view(k).graphics(), 6);
        assert_eq!(pit.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(pit.sprite_slot_view(k).head_direction(), 15);
        assert_eq!(pit.sprite_slot_view(15).state(), 9);
        assert_eq!(pit.sprite_slot_view(15).sprite_type(), 0x64);
        assert_eq!(pit.sprite_slot_view(15).delay_main(), 96);
        assert_eq!(pit.sprite_slot_view(15).g(), 1);
        assert_eq!(pit.sprite_slot_view(15).oam_flags(), 8);
        assert_eq!(pit.sprite_get_x(15), pit.sprite_get_x(k));
        assert_eq!(pit.sprite_get_y(15), pit.sprite_get_y(k));

        let mut fire = fresh_state();
        fire.sprite_slot_mut(k).set_state(9);
        fire.sprite_slot_mut(k).set_sprite_type(0x64);
        fire.sprite_slot_mut(k).set_g(7);
        fire.sprite_slot_mut(k).set_delay_main(9);
        fire.sprite_slot_mut(k).set_x_low(0x44);
        fire.sprite_slot_mut(k).set_y_low(0x55);
        fire.sprite_prep_fire_debirando(k);
        assert_eq!(fire.sprite_slot_view(k).sprite_type(), 0x63);
        assert_eq!(fire.sprite_slot_view(k).g(), 0);
        assert_eq!(fire.sprite_slot_view(k).graphics(), 6);
        assert_eq!(fire.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(fire.sprite_slot_view(15).sprite_type(), 0x64);
        assert_eq!(fire.sprite_slot_view(15).g(), 0);
        assert_eq!(fire.sprite_slot_view(15).oam_flags(), 6);
    }

    #[test]
    fn bully_hobo_and_talking_tree_prep_spawn_helper_sprites() {
        let k = 4;

        let mut bully = fresh_state();
        bully.sprite_slot_mut(k).set_state(9);
        bully.sprite_set_x(k, 0x0110);
        bully.sprite_set_y(k, 0x0220);
        bully.sprite_prep_bully_and_victim(k);
        assert_eq!(bully.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(bully.sprite_slot_view(15).state(), 9);
        assert_eq!(bully.sprite_slot_view(15).sprite_type(), 0xb9);
        assert_eq!(bully.sprite_get_x(15), 0x0110);
        assert_eq!(bully.sprite_get_y(15), 0x0220);
        assert_eq!(bully.sprite_slot_view(15).subtype2(), 2);
        assert_eq!(bully.sprite_slot_view(15).head_direction(), k as u8);
        assert_eq!(bully.sprite_slot_view(15).ignore_projectile(), 1);
        bully.ball_guy_play_bounce_noise(k);
        assert_eq!(
            bully.game_state.system_signals.sound_effect_2() & 0x3f,
            0x32
        );

        let mut garnish = fresh_state();
        garnish.garnish_slot_view_mut(29).set_garnish_type(1);
        garnish.garnish_slot_view_mut(14).set_garnish_type(1);
        assert_eq!(garnish.garnish_alloc_force(), 28);
        assert_eq!(garnish.garnish_alloc(), 28);
        assert_eq!(garnish.garnish_alloc_low(), 13);
        assert_eq!(garnish.garnish_alloc_limit(12), 12);

        for slot in 0..30 {
            garnish.garnish_slot_view_mut(slot).set_garnish_type(1);
        }
        assert_eq!(garnish.garnish_alloc_force(), 0);
        assert_eq!(garnish.garnish_alloc(), -1);
        assert_eq!(garnish.garnish_alloc_low(), -1);
        assert_eq!(garnish.garnish_alloc_limit(12), -1);
        assert_eq!(garnish.garnish_alloc_overwrite_old_low(), 14);
        assert_eq!(garnish.garnish_alloc_overwrite_old(), 13);

        let mut coords = fresh_state();
        coords.garnish_set_x(3, 0x1234);
        coords.garnish_set_y(3, 0xabcd);
        assert_eq!(coords.garnish_slot_view(3).x_low(), 0x34);
        assert_eq!(coords.garnish_slot_view(3).x_high(), 0x12);
        assert_eq!(coords.garnish_slot_view(3).y_low(), 0xcd);
        assert_eq!(coords.garnish_slot_view(3).y_high(), 0xab);

        let mut debris = fresh_state();
        debris.garnish_slot_view_mut(29).set_garnish_type(1);
        debris.garnish_spawn_pyramid_debris(-4, 5, -7, 9);
        assert_eq!(debris.game_state.system_signals.sound_effect_2(), 3);
        assert_eq!(debris.game_state.system_signals.sound_effect_1(), 31);
        assert_eq!(debris.game_state.system_signals.ambient_sound_effect(), 5);
        assert_eq!(debris.garnish_slot_view(28).garnish_type(), 19);
        assert_eq!(debris.game_state.sprites.garnish_runtime.active_type(), 19);
        assert_eq!(debris.garnish_slot_view(28).x_low(), 228);
        assert_eq!(debris.garnish_slot_view(28).y_low(), 101);
        assert_eq!(debris.garnish_slot_view(28).x_velocity(), (-7i8) as u8);
        assert_eq!(debris.garnish_slot_view(28).y_velocity(), 9);
        assert_eq!(debris.garnish_slot_view(28).countdown(), 72);

        let mut puff = fresh_state();
        let puff_owner = 6;
        puff.set_frame_counter(2);
        puff.garnish_slot_view_mut(14).set_garnish_type(1);
        puff.sprite_workspace_mut().set_current_sprite_x(0x0200);
        puff.sprite_workspace_mut().set_current_sprite_y(0x0300);
        puff.kholdstare_spawn_puff_cloud_garnish(puff_owner);
        assert_eq!(puff.garnish_slot_view(13).garnish_type(), 7);
        assert_eq!(puff.game_state.sprites.garnish_runtime.active_type(), 7);
        assert_eq!(puff.garnish_slot_view(13).countdown(), 31);
        assert_eq!(puff.garnish_slot_view(13).x_low(), 0xfa);
        assert_eq!(puff.garnish_slot_view(13).x_high(), 0x01);
        assert_eq!(puff.garnish_slot_view(13).y_low(), 0x12);
        assert_eq!(puff.garnish_slot_view(13).y_high(), 0x03);
        assert_eq!(puff.garnish_slot_view(13).floor(), 0);

        let mut flame = fresh_state();
        flame.garnish_slot_view_mut(29).set_garnish_type(1);
        flame.sprite_set_x(k, 0x0456);
        flame.sprite_set_y(k, 0x0789);
        assert_eq!(flame.garnish_flame_trail(k, false), 28);
        assert_eq!(flame.garnish_slot_view(28).garnish_type(), 0x10);
        assert_eq!(flame.game_state.sprites.garnish_runtime.active_type(), 0x10);
        assert_eq!(flame.garnish_slot_view(28).sprite(), k as u8);
        assert_eq!(flame.garnish_slot_view(28).x_low(), 0x56);
        assert_eq!(flame.garnish_slot_view(28).x_high(), 0x04);
        assert_eq!(flame.garnish_slot_view(28).y_low(), 0x99);
        assert_eq!(flame.garnish_slot_view(28).y_high(), 0x07);
        assert_eq!(flame.garnish_slot_view(28).countdown(), 127);

        let mut low_flame = fresh_state();
        low_flame.garnish_slot_view_mut(14).set_garnish_type(1);
        low_flame.sprite_set_x(k, 0x0012);
        low_flame.sprite_set_y(k, 0x00f8);
        assert_eq!(low_flame.garnish_flame_trail(k, true), 13);
        assert_eq!(low_flame.garnish_slot_view(13).garnish_type(), 0x10);
        assert_eq!(low_flame.garnish_slot_view(13).y_low(), 0x08);
        assert_eq!(low_flame.garnish_slot_view(13).y_high(), 0x01);

        let mut fire_bat = fresh_state();
        fire_bat.sprite_slot_mut(k).set_subtype2(3);
        fire_bat.fire_bat_animate(k);
        assert_eq!(fire_bat.sprite_slot_view(k).subtype2(), 4);
        assert_eq!(fire_bat.sprite_slot_view(k).graphics(), 5);

        let mut moving_fire_bat = fresh_state();
        moving_fire_bat
            .garnish_slot_view_mut(14)
            .set_garnish_type(1);
        moving_fire_bat.sprite_slot_mut(k).set_subtype2(7);
        moving_fire_bat.sprite_slot_mut(k).set_anim_clock(5);
        moving_fire_bat.sprite_set_x(k, 0x0124);
        moving_fire_bat.sprite_set_y(k, 0x0340);
        moving_fire_bat.fire_bat_move(k);
        assert_eq!(moving_fire_bat.sprite_slot_view(k).subtype2(), 8);
        assert_eq!(moving_fire_bat.sprite_slot_view(k).graphics(), 6);
        assert_eq!(moving_fire_bat.garnish_slot_view(13).garnish_type(), 0x10);
        assert_eq!(
            moving_fire_bat
                .game_state
                .sprites
                .garnish_runtime
                .active_type(),
            0x10
        );
        assert_eq!(moving_fire_bat.garnish_slot_view(13).sprite(), k as u8);
        assert_eq!(moving_fire_bat.garnish_slot_view(13).x_low(), 0x24);
        assert_eq!(moving_fire_bat.garnish_slot_view(13).x_high(), 0x01);
        assert_eq!(moving_fire_bat.garnish_slot_view(13).y_low(), 0x50);
        assert_eq!(moving_fire_bat.garnish_slot_view(13).y_high(), 0x03);
        assert_eq!(moving_fire_bat.garnish_slot_view(13).countdown(), 0x2f);

        let mut skipped_fire_bat = fresh_state();
        skipped_fire_bat.sprite_slot_mut(k).set_subtype2(0);
        skipped_fire_bat.fire_bat_move(k);
        assert_eq!(skipped_fire_bat.sprite_slot_view(k).subtype2(), 1);
        assert_eq!(
            skipped_fire_bat
                .game_state
                .sprites
                .garnish_runtime
                .active_type(),
            0
        );

        let mut fireball = fresh_state();
        fireball.set_frame_counter(0);
        fireball.garnish_slot_view_mut(29).set_garnish_type(1);
        fireball.sprite_workspace_mut().set_current_sprite_x(0x0123);
        fireball.sprite_workspace_mut().set_current_sprite_y(0x02f5);
        fireball.fireball_spawn_trail_garnish(k);
        assert_eq!(fireball.garnish_slot_view(28).garnish_type(), 8);
        assert_eq!(fireball.game_state.sprites.garnish_runtime.active_type(), 8);
        assert_eq!(fireball.garnish_slot_view(28).countdown(), 11);
        assert_eq!(fireball.garnish_slot_view(28).x_low(), 0x23);
        assert_eq!(fireball.garnish_slot_view(28).x_high(), 0x01);
        assert_eq!(fireball.garnish_slot_view(28).y_low(), 0x05);
        assert_eq!(fireball.garnish_slot_view(28).y_high(), 0x03);
        assert_eq!(fireball.garnish_slot_view(28).sprite(), k as u8);

        let mut skipped_fireball = fresh_state();
        skipped_fireball.set_frame_counter(1);
        skipped_fireball.fireball_spawn_trail_garnish(k);
        assert_eq!(
            skipped_fireball
                .game_state
                .sprites
                .garnish_runtime
                .active_type(),
            0
        );

        let mut firesnake = fresh_state();
        firesnake.set_frame_counter(k as u8);
        firesnake.garnish_slot_view_mut(29).set_garnish_type(1);
        firesnake.sprite_set_x(k, 0x0167);
        firesnake.sprite_set_y(k, 0x02f0);
        firesnake.sprite_slot_mut(k).set_floor(2);
        firesnake.firesnake_spawn_fireball(k);
        assert_eq!(firesnake.garnish_slot_view(28).garnish_type(), 1);
        assert_eq!(
            firesnake.game_state.sprites.garnish_runtime.active_type(),
            1
        );
        assert_eq!(firesnake.garnish_slot_view(28).x_low(), 0x67);
        assert_eq!(firesnake.garnish_slot_view(28).x_high(), 0x01);
        assert_eq!(firesnake.garnish_slot_view(28).y_low(), 0x00);
        assert_eq!(firesnake.garnish_slot_view(28).y_high(), 0x03);
        assert_eq!(firesnake.garnish_slot_view(28).countdown(), 32);
        assert_eq!(firesnake.garnish_slot_view(28).sprite(), k as u8);
        assert_eq!(firesnake.garnish_slot_view(28).floor(), 2);

        let mut skipped_firesnake = fresh_state();
        skipped_firesnake.set_frame_counter((k as u8) ^ 1);
        skipped_firesnake.firesnake_spawn_fireball(k);
        assert_eq!(
            skipped_firesnake
                .game_state
                .sprites
                .garnish_runtime
                .active_type(),
            0
        );

        let mut plop = fresh_state();
        plop.sprite_slot_mut(k).set_state(9);
        plop.sprite_set_x(k, 0x0100);
        plop.sprite_set_y(k, 0x0200);
        plop.catfish_spawn_plop(k);
        assert_eq!(plop.sprite_slot_view(15).sprite_type(), 0xec);
        assert_eq!(plop.sprite_get_x(15), 0x0100);
        assert_eq!(plop.sprite_get_y(15), 0x0200);
        assert_eq!(plop.sprite_slot_view(15).state(), 3);
        assert_eq!(plop.sprite_slot_view(15).delay_main(), 15);
        assert_eq!(plop.sprite_slot_view(15).ai_state(), 0);
        assert_eq!(plop.sprite_slot_view(15).flags2(), 3);
        assert_eq!(plop.game_state.system_signals.sound_effect_1() & 0x3f, 0x28);

        let mut medallion = fresh_state();
        medallion.sprite_slot_mut(k).set_state(9);
        medallion.sprite_set_x(k, 0x0100);
        medallion.sprite_set_y(k, 0x0200);
        medallion.catfish_regurgitate_medallion(k);
        assert_eq!(medallion.sprite_slot_view(15).sprite_type(), 0xc0);
        assert_eq!(medallion.sprite_get_x(15), 0x0100);
        assert_eq!(medallion.sprite_get_y(15), 0x0200);
        assert_eq!(medallion.sprite_slot_view(15).x_velocity(), 24);
        assert_eq!(medallion.sprite_slot_view(15).z_velocity(), 48);
        assert_eq!(medallion.sprite_slot_view(15).a(), 17);
        assert_eq!(
            medallion.game_state.system_signals.sound_effect_1() & 0x3f,
            0x20
        );
        assert_eq!(medallion.sprite_slot_view(15).flags2(), 0x83);
        assert_eq!(medallion.sprite_slot_view(15).flags3(), 0x58);
        assert_eq!(medallion.sprite_slot_view(15).oam_flags(), 8);

        let mut splash = fresh_state();
        splash.sprite_slot_mut(k).set_state(9);
        splash.sprite_set_x(k, 0x0030);
        splash.sprite_set_y(k, 0x0040);
        assert_eq!(splash.sprite_spawn_water_splash(k), 15);
        assert_eq!(splash.sprite_slot_view(15).sprite_type(), 0xc0);
        assert_eq!(splash.sprite_get_x(15), 0x0030);
        assert_eq!(splash.sprite_get_y(15), 0x0040);
        assert_eq!(splash.sprite_slot_view(15).a(), 0x80);
        assert_eq!(splash.sprite_slot_view(15).flags2(), 2);
        assert_eq!(splash.sprite_slot_view(15).ignore_projectile(), 2);
        assert_eq!(splash.sprite_slot_view(15).oam_flags(), 4);
        assert_eq!(splash.sprite_slot_view(15).delay_main(), 31);

        let mut small_splash = fresh_state();
        small_splash.sprite_slot_mut(k).set_state(9);
        small_splash.sprite_set_x(k, 0x0060);
        small_splash.sprite_set_y(k, 0x0070);
        small_splash.set_sound_effect_1(0xff);
        assert_eq!(small_splash.sprite_spawn_small_splash(k), 14);
        assert_eq!(small_splash.sprite_slot_view(14).sprite_type(), 0xec);
        assert_eq!(small_splash.sprite_get_x(14), 0x0060);
        assert_eq!(small_splash.sprite_get_y(14), 0x0070);
        assert_eq!(
            small_splash.game_state.system_signals.sound_effect_1() & 0x3f,
            0x28
        );
        assert_eq!(small_splash.sprite_slot_view(14).state(), 3);
        assert_eq!(small_splash.sprite_slot_view(14).delay_main(), 15);
        assert_eq!(small_splash.sprite_slot_view(14).ai_state(), 0);
        assert_eq!(small_splash.sprite_slot_view(14).flags2(), 3);

        let mut dust = fresh_state();
        dust.sprite_slot_mut(k).set_state(9);
        dust.sprite_set_x(k, 0x0100);
        dust.sprite_set_y(k, 0x0200);
        assert_eq!(dust.sprite_spawn_dust_cloud(k), 15);
        assert_eq!(dust.sprite_slot_view(15).sprite_type(), 0xf2);
        assert_eq!(dust.sprite_get_x(15), 0x00fc);
        assert_eq!(dust.sprite_get_y(15), 0x0208);
        assert_eq!(dust.sprite_slot_view(15).subtype2(), 1);

        let mut blast = fresh_state();
        blast.sprite_slot_mut(k).set_state(9);
        blast.sprite_set_x(k, 0x0018);
        blast.sprite_set_y(k, 0x0028);
        assert_eq!(blast.sprite_spawn_superficial_bomb_blast(k), 15);
        assert_eq!(blast.sprite_slot_view(15).sprite_type(), 0x4a);
        assert_eq!(blast.sprite_get_x(15), 0x0018);
        assert_eq!(blast.sprite_get_y(15), 0x0028);
        assert_eq!(blast.sprite_slot_view(15).state(), 6);
        assert_eq!(blast.sprite_slot_view(15).delay_aux1(), 31);
        assert_eq!(blast.sprite_slot_view(15).c(), 3);
        assert_eq!(blast.sprite_slot_view(15).flags2(), 3);
        assert_eq!(blast.sprite_slot_view(15).oam_flags(), 4);
        assert_eq!(
            blast.game_state.system_signals.sound_effect_1() & 0x3f,
            0x15
        );

        let mut bomb = fresh_state();
        bomb.sprite_slot_mut(k).set_state(9);
        bomb.sprite_set_x(k, 0x0044);
        bomb.sprite_set_y(k, 0x0055);
        assert_eq!(bomb.sprite_spawn_bomb(k), 15);
        assert_eq!(bomb.sprite_slot_view(15).sprite_type(), 0x4a);
        assert_eq!(bomb.sprite_get_x(15), 0x0044);
        assert_eq!(bomb.sprite_get_y(15), 0x0055);
        assert_eq!(bomb.sprite_slot_view(15).c(), 1);
        assert_eq!(bomb.sprite_slot_view(15).delay_aux1(), 80);
        assert_eq!(bomb.sprite_slot_view(15).flags3(), 0x18);
        assert_eq!(bomb.sprite_slot_view(15).oam_flags(), 8);
        assert_eq!(bomb.sprite_slot_view(15).health(), 0);
        assert_eq!(bomb.sprite_slot_view(15).x_velocity(), 24);
        assert_eq!(bomb.sprite_slot_view(15).z_velocity(), 48);

        let mut poof = fresh_state();
        poof.sprite_slot_mut(k).set_state(9);
        poof.sprite_set_x(k, 0x0100);
        poof.sprite_set_y(k, 0x0200);
        assert_eq!(poof.spawn_boss_poof(k), 15);
        assert_eq!(poof.sprite_slot_view(15).sprite_type(), 0xce);
        assert_eq!(poof.sprite_get_x(15), 0x0110);
        assert_eq!(poof.sprite_get_y(15), 0x0228);
        assert_eq!(poof.sprite_slot_view(15).graphics(), 0x0f);
        assert_eq!(poof.sprite_slot_view(15).a(), 1);
        assert_eq!(poof.sprite_slot_view(15).delay_main(), 47);
        assert_eq!(poof.sprite_slot_view(15).flags2(), 9);
        assert_eq!(poof.sprite_slot_view(15).ignore_projectile(), 9);
        assert_eq!(poof.game_state.system_signals.sound_effect_1(), 12);

        let mut fireball = fresh_state();
        fireball.sprite_slot_mut(k).set_state(9);
        fireball.sprite_set_x(k, 0x0100);
        fireball.sprite_set_y(k, 0x0200);
        fireball.sprite_slot_mut(k).set_z(16);
        fireball.follower_link_state_mut().set_x(0x0124);
        fireball.follower_link_state_mut().set_y(0x01ec);
        assert_eq!(fireball.sprite_spawn_fireball(k), 13);
        assert_eq!(fireball.sprite_slot_view(13).sprite_type(), 0x55);
        assert_eq!(fireball.sprite_get_x(13), 0x0104);
        assert_eq!(fireball.sprite_get_y(13), 0x01f4);
        assert_eq!(fireball.sprite_slot_view(13).flags3(), 0x42);
        assert_eq!(fireball.sprite_slot_view(13).oam_flags(), 6);
        assert_eq!(fireball.sprite_slot_view(13).flags4(), 0x54);
        assert_eq!(fireball.sprite_slot_view(13).e(), 0x54);
        assert_eq!(fireball.sprite_slot_view(13).flags2(), 0x20);
        assert_eq!(fireball.sprite_slot_view(13).x_velocity(), 0x20);
        assert_eq!(fireball.sprite_slot_view(13).y_velocity(), 0);
        assert_eq!(fireball.sprite_slot_view(13).delay_main(), 20);
        assert_eq!(fireball.sprite_slot_view(13).delay_aux1(), 16);
        assert_eq!(fireball.sprite_slot_view(13).flags5(), 0);
        assert_eq!(fireball.sprite_slot_view(13).deflection_bits(), 0x48);
        assert_eq!(
            fireball.game_state.system_signals.sound_effect_2() & 0x3f,
            0x19
        );

        let mut phlegm = fresh_state();
        phlegm.sprite_slot_mut(k).set_state(9);
        phlegm.sprite_set_x(k, 0x0040);
        phlegm.sprite_set_y(k, 0x0060);
        phlegm.sprite_slot_mut(k).set_z(7);
        phlegm.sprite_slot_mut(k).set_direction(1);
        phlegm.inventory_items_mut().set_shield_type(3);
        assert_eq!(phlegm.sprite_spawn_fire_phlegm(k), 15);
        assert_eq!(phlegm.sprite_slot_view(15).sprite_type(), 0xa5);
        assert_eq!(phlegm.sprite_get_x(15), 0x0038);
        assert_eq!(phlegm.sprite_get_y(15), 0x005e);
        assert_eq!(phlegm.sprite_slot_view(15).x_velocity(), (-48i8) as u8);
        assert_eq!(phlegm.sprite_slot_view(15).y_velocity(), 0);
        assert_eq!(phlegm.sprite_slot_view(15).flags3() & 0x40, 0x40);
        assert_eq!(phlegm.sprite_slot_view(15).deflection_bits(), 0x40);
        assert_eq!(phlegm.sprite_slot_view(15).flags2(), 0x21);
        assert_eq!(phlegm.sprite_slot_view(15).b(), 0x21);
        assert_eq!(phlegm.sprite_slot_view(15).oam_flags(), 2);
        assert_eq!(phlegm.sprite_slot_view(15).flags4(), 0x14);
        assert_eq!(phlegm.sprite_slot_view(15).ignore_projectile(), 20);
        assert_eq!(phlegm.sprite_slot_view(15).bump_damage(), 37);
        assert_eq!(phlegm.sprite_slot_view(15).flags5(), 0x20);
        assert_eq!(phlegm.game_state.system_signals.sound_effect_2() & 0x3f, 5);

        let mut leaves = fresh_state();
        leaves.sprite_slot_mut(k).set_state(9);
        leaves.sprite_set_x(k, 0x0120);
        leaves.sprite_set_y(k, 0x0340);
        leaves.sprite_slot_mut(k).set_z_velocity(0x24);
        assert_eq!(leaves.lumberjack_tree_spawn_leaves(k), 15);
        assert_eq!(leaves.sprite_slot_view(15).sprite_type(), 0x3b);
        assert_eq!(leaves.sprite_get_x(15), 0x0120);
        assert_eq!(leaves.sprite_get_y(15), 0x0340);
        assert_eq!(leaves.sprite_slot_view(15).graphics(), 2);
        assert_eq!(leaves.sprite_slot_view(15).z_velocity(), 0x24);
        assert_eq!(leaves.sprite_slot_view(15).subtype2(), 1);
        assert_eq!(leaves.sprite_slot_view(15).ai_state(), 2);
        assert_eq!(leaves.sprite_slot_view(15).delay_main(), 8);

        let mut garnish_poof = fresh_state();
        garnish_poof.sprite_set_x(k, 0x0234);
        garnish_poof.sprite_set_y(k, 0x0456);
        garnish_poof.sprite_slot_mut(k).set_floor(2);
        garnish_poof.sprite_spawn_poof_garnish(k);
        assert_eq!(garnish_poof.garnish_slot_view(29).garnish_type(), 10);
        assert_eq!(
            garnish_poof
                .game_state
                .sprites
                .garnish_runtime
                .active_type(),
            10
        );
        assert_eq!(garnish_poof.garnish_slot_view(29).x_low(), 0x34);
        assert_eq!(garnish_poof.garnish_slot_view(29).x_high(), 0x02);
        assert_eq!(garnish_poof.garnish_slot_view(29).y_low(), 0x66);
        assert_eq!(garnish_poof.garnish_slot_view(29).y_high(), 0x04);
        assert_eq!(garnish_poof.garnish_slot_view(29).sprite(), 2);
        assert_eq!(garnish_poof.garnish_slot_view(29).countdown(), 15);

        let mut octorok = fresh_state();
        octorok.sprite_slot_mut(k).set_state(9);
        octorok.sprite_set_x(k, 0x0100);
        octorok.sprite_set_y(k, 0x0200);
        octorok.sprite_slot_mut(k).set_direction(0);
        octorok.octorok_fire_loogie(k);
        assert_eq!(octorok.sprite_slot_view(15).sprite_type(), 0x0c);
        assert_eq!(octorok.sprite_get_x(15), 0x010c);
        assert_eq!(octorok.sprite_get_y(15), 0x0204);
        assert_eq!(octorok.sprite_slot_view(15).x_velocity(), 44);
        assert_eq!(octorok.sprite_slot_view(15).y_velocity(), 0);
        assert_eq!(octorok.game_state.system_signals.sound_effect_1() & 0x3f, 7);

        let mut moblin = fresh_state();
        moblin.sprite_slot_mut(k).set_state(9);
        moblin.sprite_set_x(k, 0x0200);
        moblin.sprite_set_y(k, 0x0100);
        moblin.sprite_slot_mut(k).set_direction(3);
        moblin.moblin_materialize_spear(k);
        assert_eq!(moblin.sprite_slot_view(15).sprite_type(), 0x1b);
        assert_eq!(moblin.sprite_slot_view(15).a(), 3);
        assert_eq!(moblin.sprite_slot_view(15).direction(), 3);
        assert_eq!(moblin.sprite_get_x(15), 0x020b);
        assert_eq!(moblin.sprite_get_y(15), 0x00f5);
        assert_eq!(moblin.sprite_slot_view(15).x_velocity(), 0);
        assert_eq!(moblin.sprite_slot_view(15).y_velocity(), (-32i8) as u8);

        let mut snitch = fresh_state();
        snitch.sprite_slot_mut(k).set_state(9);
        snitch.sprite_slot_mut(k).set_sprite_type(0x35);
        snitch.garnish_state_mut().set_sprcoll_x_base(0x1200);
        snitch.garnish_state_mut().set_sprcoll_y_base(0x3400);
        snitch.snitch_spawn_guard(k);
        assert_eq!(snitch.sprite_slot_view(0).sprite_type(), 0x45);
        assert_eq!(snitch.sprite_slot_view(0).state(), 9);
        assert_eq!(snitch.sprite_get_x(0), 0x1540);
        assert_eq!(snitch.sprite_get_y(0), 0x37b0);
        assert_eq!(snitch.sprite_slot_view(0).floor(), 0);
        assert_eq!(snitch.sprite_slot_view(0).health(), 4);
        assert_eq!(snitch.sprite_slot_view(0).deflection_bits(), 0x80);
        assert_eq!(snitch.sprite_slot_view(0).flags5(), 0x90);
        assert_eq!(snitch.sprite_slot_view(0).oam_flags(), 0x0b);

        let mut sparkle = fresh_state();
        for (idx, ty) in [0x2a, 0x21, 0x30, 0x19, 0x0c].into_iter().enumerate() {
            sparkle.ancilla_slot_view_mut(idx).set_ancilla_type(ty);
        }
        sparkle.ancilla_terminate_sparkle_objects();
        assert_eq!(sparkle.ancilla_slot_view(0).ancilla_type(), 0);
        assert_eq!(sparkle.ancilla_slot_view(1).ancilla_type(), 0x21);
        assert_eq!(sparkle.ancilla_slot_view(2).ancilla_type(), 0);
        assert_eq!(sparkle.ancilla_slot_view(3).ancilla_type(), 0);
        assert_eq!(sparkle.ancilla_slot_view(4).ancilla_type(), 0);

        let mut kodongo = fresh_state();
        kodongo.sprite_slot_mut(k).set_direction(2);
        kodongo.kodongo_set_direction(k);
        assert_eq!(kodongo.sprite_slot_view(k).x_velocity(), 0);
        assert_eq!(kodongo.sprite_slot_view(k).y_velocity(), 16);

        let mut kodongo_fire = fresh_state();
        kodongo_fire.sprite_slot_mut(k).set_state(9);
        kodongo_fire.sprite_set_x(k, 0x0300);
        kodongo_fire.sprite_set_y(k, 0x0040);
        kodongo_fire.sprite_slot_mut(k).set_direction(1);
        kodongo_fire.kodongo_spawn_fire(k);
        assert_eq!(kodongo_fire.sprite_slot_view(13).sprite_type(), 0x87);
        assert_eq!(kodongo_fire.sprite_get_x(13), 0x02f8);
        assert_eq!(kodongo_fire.sprite_get_y(13), 0x0040);
        assert_eq!(
            kodongo_fire.sprite_slot_view(13).x_velocity(),
            (-24i8) as u8
        );
        assert_eq!(kodongo_fire.sprite_slot_view(13).y_velocity(), 0);
        assert_eq!(kodongo_fire.sprite_slot_view(13).ignore_projectile(), 1);

        let mut blue_balls = fresh_state();
        blue_balls.sprite_slot_mut(k).set_state(9);
        blue_balls.sprite_set_x(k, 0x0120);
        blue_balls.sprite_set_y(k, 0x0340);
        blue_balls.create_six_blue_balls(k);
        assert_eq!(
            blue_balls.game_state.system_signals.sound_effect_2() & 0x3f,
            0x36
        );
        assert_eq!(blue_balls.game_state.scratch_counter.value(), 0);
        assert_eq!(blue_balls.sprite_slot_view(15).sprite_type(), 0x55);
        assert_eq!(blue_balls.sprite_get_x(15), 0x0124);
        assert_eq!(blue_balls.sprite_get_y(15), 0x0344);
        assert_eq!(blue_balls.sprite_slot_view(15).flags3(), 0x42);
        assert_eq!(blue_balls.sprite_slot_view(15).oam_flags(), 4);
        assert_eq!(blue_balls.sprite_slot_view(15).delay_aux1(), 4);
        assert_eq!(blue_balls.sprite_slot_view(15).flags4(), 20);
        assert_eq!(blue_balls.sprite_slot_view(15).c(), 20);
        assert_eq!(blue_balls.sprite_slot_view(15).e(), 20);
        assert_eq!(blue_balls.sprite_slot_view(15).x_velocity(), (-24i8) as u8);
        assert_eq!(blue_balls.sprite_slot_view(15).y_velocity(), (-16i8) as u8);
        assert_eq!(blue_balls.sprite_slot_view(10).sprite_type(), 0x55);
        assert_eq!(blue_balls.sprite_slot_view(10).x_velocity(), 0);
        assert_eq!(blue_balls.sprite_slot_view(10).y_velocity(), (-32i8) as u8);

        let mut octoballoon = fresh_state();
        octoballoon.sprite_slot_mut(k).set_state(9);
        octoballoon.sprite_set_x(k, 0x0110);
        octoballoon.sprite_set_y(k, 0x0220);
        octoballoon.octoballoon_form_babby(k);
        assert_eq!(
            octoballoon.game_state.system_signals.sound_effect_1() & 0x3f,
            0x0c
        );
        assert_eq!(octoballoon.sprite_slot_view(15).sprite_type(), 0x10);
        assert_eq!(octoballoon.sprite_get_x(15), 0x0110);
        assert_eq!(octoballoon.sprite_get_y(15), 0x0220);
        assert_eq!(octoballoon.sprite_slot_view(15).x_velocity(), 11);
        assert_eq!(octoballoon.sprite_slot_view(15).y_velocity(), (-11i8) as u8);
        assert_eq!(octoballoon.sprite_slot_view(15).z_velocity(), 48);
        assert_eq!(octoballoon.sprite_slot_view(15).subtype2(), 255);
        assert_eq!(octoballoon.sprite_slot_view(10).sprite_type(), 0x10);
        assert_eq!(octoballoon.sprite_slot_view(10).x_velocity(), 16);
        assert_eq!(octoballoon.sprite_slot_view(10).y_velocity(), 0);

        let mut bully = fresh_state();
        bully.sprite_slot_mut(k).set_state(9);
        bully.sprite_set_x(k, 0x0440);
        bully.sprite_set_y(k, 0x0550);
        bully.ball_guy_play_bounce_noise(k);
        assert_eq!(
            bully.game_state.system_signals.sound_effect_2() & 0x3f,
            0x32
        );
        bully.spawn_bully(k);
        assert_eq!(bully.sprite_slot_view(15).sprite_type(), 0xb9);
        assert_eq!(bully.sprite_get_x(15), 0x0440);
        assert_eq!(bully.sprite_get_y(15), 0x0550);
        assert_eq!(bully.sprite_slot_view(15).subtype2(), 2);
        assert_eq!(bully.sprite_slot_view(15).head_direction(), k as u8);
        assert_eq!(bully.sprite_slot_view(15).ignore_projectile(), 1);

        let mut rupees = fresh_state();
        rupees.sprite_slot_mut(k).set_state(9);
        rupees.sprite_set_x(k, 0x0180);
        rupees.sprite_set_y(k, 0x0280);
        rupees.sprite_battle_mut().set_sprites_killed(4);
        rupees.sprite_battle_mut().set_times_hurt_by_sprites(0);
        rupees.rupee_pull_spawn_prize(k);
        assert_eq!(rupees.game_state.sprites.workspace.shared_scratch_a(), 2);
        assert_eq!(rupees.game_state.scratch_counter.value(), 0xff);
        assert_eq!(rupees.ram[NUM_SPRITES_KILLED_PREP], 0);
        assert_eq!(rupees.ram[NUMBER_OF_TIMES_HURT_BY_SPRITES], 0);
        assert_eq!(rupees.sprite_slot_view(15).sprite_type(), 0xdb);
        assert_eq!(rupees.sprite_get_x(15), 0x0180);
        assert_eq!(rupees.sprite_get_y(15), 0x0280);
        assert_eq!(rupees.sprite_slot_view(15).x_velocity(), 18);
        assert_eq!(rupees.sprite_slot_view(15).y_velocity(), 16);
        assert_eq!(rupees.sprite_slot_view(15).stunned(), 255);
        assert_eq!(rupees.sprite_slot_view(15).delay_aux4(), 32);
        assert_eq!(rupees.sprite_slot_view(15).delay_aux3(), 32);
        assert_eq!(rupees.sprite_slot_view(15).z_velocity(), 32);
        assert_eq!(rupees.sprite_slot_view(12).sprite_type(), 0xdb);
        assert_eq!(rupees.sprite_slot_view(12).x_velocity(), (-18i8) as u8);
        assert_eq!(rupees.sprite_slot_view(12).y_velocity(), 16);

        let mut pink = fresh_state();
        pink.sprite_slot_mut(k).set_x_velocity(10);
        pink.sprite_slot_mut(k).set_y_velocity((-10i8) as u8);
        pink.pink_ball_handle_deceleration(k);
        assert_eq!(pink.sprite_slot_view(k).x_velocity(), 8);
        assert_eq!(pink.sprite_slot_view(k).y_velocity(), (-8i8) as u8);
        write_le_u16(&mut pink.ram, OAM_CUR_PTR, 0x0800);
        pink.sprite_set_x(k, 0x0100);
        pink.sprite_set_y(k, 0x0120);
        pink.set_frame_counter(0x18);
        pink.pink_ball_distress(k);
        assert_eq!(pink.sprite_slot_view(k).pause(), 0);

        let mut pink_msg = fresh_state();
        pink_msg.sprite_slot_mut(k).set_direction(3);
        pink_msg.sprite_slot_mut(k).set_x_velocity(0x12);
        pink_msg.sprite_slot_mut(k).set_y_velocity(0x34);
        pink_msg.pink_ball_handle_message(k);
        assert_eq!(
            pink_msg.game_state.messaging.dialogue_message_index.value(),
            0x15b
        );
        assert_eq!(pink_msg.sprite_slot_view(k).x_velocity(), 0xed);
        assert_eq!(pink_msg.sprite_slot_view(k).y_velocity(), 0xcb);
        assert_eq!(pink_msg.sprite_slot_view(k).delay_aux4(), 64);
        pink_msg.sprite_slot_mut(k).set_delay_aux4(0);
        pink_msg.inventory_items_mut().set_moon_pearl(1);
        pink_msg.pink_ball_handle_message(k);
        assert_eq!(
            pink_msg.game_state.messaging.dialogue_message_index.value(),
            0x15c
        );

        let mut bully_msg = fresh_state();
        bully_msg.sprite_slot_mut(k).set_direction(2);
        bully_msg.sprite_slot_mut(k).set_x_velocity(0x12);
        bully_msg.sprite_slot_mut(k).set_y_velocity(0x34);
        bully_msg.bully_handle_message(k);
        assert_eq!(
            bully_msg
                .game_state
                .messaging
                .dialogue_message_index
                .value(),
            0x15d
        );
        assert_eq!(bully_msg.sprite_slot_view(k).x_velocity(), 0xed);
        assert_eq!(bully_msg.sprite_slot_view(k).y_velocity(), 0xcb);
        assert_eq!(bully_msg.sprite_slot_view(k).delay_aux4(), 64);
        bully_msg.sprite_slot_mut(k).set_delay_aux4(0);
        bully_msg.inventory_items_mut().set_moon_pearl(1);
        bully_msg.bully_handle_message(k);
        assert_eq!(
            bully_msg
                .game_state
                .messaging
                .dialogue_message_index
                .value(),
            0x15e
        );

        let mut sasha = fresh_state();
        sasha.sprite_slot_mut(k).set_state(9);
        sasha.set_frame_counter(0x20);
        sasha.sasha_idle(k);
        assert_eq!(
            sasha.game_state.messaging.dialogue_message_index.value(),
            0x32
        );
        assert_eq!(sasha.sprite_slot_view(k).graphics(), 1);
        sasha.player_resources_mut().set_pendant_flags(4);
        sasha.save_progress_mut().set_map_icons_indicator(3);
        sasha.sasha_idle(k);
        assert_eq!(
            sasha.game_state.messaging.dialogue_message_index.value(),
            0x38
        );
        sasha.inventory_items_mut().set_boots(1);
        sasha.sasha_idle(k);
        assert_eq!(
            sasha.game_state.messaging.dialogue_message_index.value(),
            0x37
        );
        sasha.inventory_items_mut().set_ice_rod(1);
        sasha.sasha_idle(k);
        assert_eq!(
            sasha.game_state.messaging.dialogue_message_index.value(),
            0x34
        );
        sasha.player_resources_mut().set_pendant_flags(7);
        sasha.sasha_idle(k);
        assert_eq!(
            sasha.game_state.messaging.dialogue_message_index.value(),
            0x30
        );
        sasha.inventory_items_mut().set_sword_type(2);
        sasha.sasha_idle(k);
        assert_eq!(
            sasha.game_state.messaging.dialogue_message_index.value(),
            0x31
        );

        let mut old_man = fresh_state();
        let t = 2;
        old_man.tagalong_slot_mut(t).set_layer_bits(2);
        old_man.tagalong_slot_mut(t).set_position(0x0420, 0x0340);
        old_man.follower_link_state_mut().mark_lower_level();
        old_man.follower_state_mut().set_indicator(6);
        old_man.follower_link_state_mut().set_speed_setting(9);
        old_man.old_man_revert_to_sprite(t);
        assert_eq!(old_man.sprite_slot_view(15).sprite_type(), 0xad);
        assert_eq!(old_man.sprite_slot_view(15).direction(), 2);
        assert_eq!(old_man.sprite_slot_view(15).head_direction(), 2);
        assert_eq!(old_man.sprite_get_y(15), 0x0342);
        assert_eq!(old_man.sprite_get_x(15), 0x0422);
        assert_eq!(old_man.sprite_slot_view(15).floor(), 1);
        assert_eq!(old_man.sprite_slot_view(15).ignore_projectile(), 1);
        assert_eq!(old_man.sprite_slot_view(15).subtype2(), 1);
        assert_eq!(
            old_man.game_state.player.follower_link.immobilized_flag(),
            1
        );
        assert_eq!(
            old_man
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer(),
            1
        );
        assert_eq!(old_man.game_state.sprites.follower_runtime.indicator(), 0);
        assert_eq!(old_man.game_state.player.follower_link.speed_setting(), 0);

        let mut apple = fresh_state();
        apple.sprite_slot_mut(k).set_state(9);
        apple.sprite_set_x(k, 0x0200);
        apple.sprite_set_y(k, 0x0300);
        apple.set_frame_counter(0);
        apple.ram[0x0fa1] = 0;
        apple.spawn_apple(k);
        assert_eq!(apple.sprite_slot_view(15).sprite_type(), 0xac);
        assert_eq!(apple.sprite_get_x(15), 0x0200);
        assert_eq!(apple.sprite_get_y(15), 0x0300);
        assert_eq!(apple.sprite_slot_view(15).ai_state(), 1);
        assert_eq!(apple.sprite_slot_view(15).a(), 255);
        assert_eq!(apple.sprite_slot_view(15).z(), 8);
        assert_eq!(apple.sprite_slot_view(15).z_velocity(), 22);
        assert_eq!(apple.sprite_slot_view(15).x_velocity(), 10);
        assert_eq!(apple.sprite_slot_view(15).y_velocity(), 3);

        let mut transmute = fresh_state();
        transmute.sprite_slot_mut(k).set_sprite_type(0xd8);
        transmute.sprite_slot_mut(k).set_health(7);
        transmute.sprite_transmute_to_bomb(k);
        assert_eq!(transmute.sprite_slot_view(k).sprite_type(), 0x4a);
        assert_eq!(transmute.sprite_slot_view(k).c(), 1);
        assert_eq!(transmute.sprite_slot_view(k).delay_aux1(), 255);
        assert_eq!(transmute.sprite_slot_view(k).flags3(), 0x18);
        assert_eq!(transmute.sprite_slot_view(k).oam_flags(), 8);
        assert_eq!(transmute.sprite_slot_view(k).health(), 0);

        let mut sluggula = fresh_state();
        sluggula.sprite_slot_mut(k).set_state(9);
        sluggula.sprite_set_x(k, 0x0120);
        sluggula.sprite_set_y(k, 0x0340);
        sluggula.sluggula_drop_bomb(k);
        assert_eq!(sluggula.sprite_slot_view(11).sprite_type(), 0x4a);
        assert_eq!(sluggula.sprite_get_x(11), 0x0120);
        assert_eq!(sluggula.sprite_get_y(11), 0x0340);
        assert_eq!(sluggula.sprite_slot_view(11).c(), 1);
        assert_eq!(sluggula.sprite_slot_view(11).delay_aux1(), 255);
        assert_eq!(sluggula.sprite_slot_view(11).flags3(), 0x18);
        assert_eq!(sluggula.sprite_slot_view(11).oam_flags(), 8);
        assert_eq!(sluggula.sprite_slot_view(11).health(), 0);

        let mut tree_bomb = fresh_state();
        tree_bomb.sprite_slot_mut(k).set_state(9);
        tree_bomb.sprite_set_x(k, 0x0048);
        tree_bomb.sprite_set_y(k, 0x0058);
        tree_bomb.talking_tree_spawn_bomb(k);
        assert_eq!(tree_bomb.sprite_slot_view(15).sprite_type(), 0x4a);
        assert_eq!(tree_bomb.sprite_get_x(15), 0x0048);
        assert_eq!(tree_bomb.sprite_get_y(15), 0x0058);
        assert_eq!(tree_bomb.sprite_slot_view(15).c(), 1);
        assert_eq!(tree_bomb.sprite_slot_view(15).delay_aux1(), 64);
        assert_eq!(tree_bomb.sprite_slot_view(15).flags3(), 0x18);
        assert_eq!(tree_bomb.sprite_slot_view(15).oam_flags(), 8);
        assert_eq!(tree_bomb.sprite_slot_view(15).health(), 0);
        assert_eq!(tree_bomb.sprite_slot_view(15).y_velocity(), 24);
        assert_eq!(tree_bomb.sprite_slot_view(15).z_velocity(), 18);

        let mut tree_eye = fresh_state();
        tree_eye.sprite_slot_mut(k).set_state(9);
        tree_eye.sprite_set_x(k, 0x0200);
        tree_eye.sprite_set_y(k, 0x0300);
        tree_eye.sprite_prep_talking_tree_spawn_eyeball(k, 1);
        assert_eq!(tree_eye.sprite_slot_view(15).sprite_type(), 0x25);
        assert_eq!(tree_eye.sprite_slot_view(15).head_direction(), 1);
        assert_eq!(tree_eye.sprite_get_x(15), 0x020e);
        assert_eq!(tree_eye.sprite_get_y(15), 0x02f5);
        assert_eq!(tree_eye.sprite_slot_view(15).a(), 0x0e);
        assert_eq!(tree_eye.sprite_slot_view(15).b(), 0x02);
        assert_eq!(tree_eye.sprite_slot_view(15).c(), 0xf5);
        assert_eq!(tree_eye.sprite_slot_view(15).e(), 0x02);
        assert_eq!(tree_eye.sprite_slot_view(15).subtype2(), 1);

        let mut pirogusu = fresh_state();
        pirogusu.set_frame_counter(k as u8);
        pirogusu.garnish_slot_view_mut(14).set_garnish_type(1);
        pirogusu.sprite_set_x(k, 0x0110);
        pirogusu.sprite_set_y(k, 0x0220);
        pirogusu.pirogusu_spawn_splash(k);
        assert_eq!(pirogusu.garnish_slot_view(13).garnish_type(), 11);
        assert_eq!(
            pirogusu.game_state.sprites.garnish_runtime.active_type(),
            11
        );
        assert_eq!(pirogusu.garnish_slot_view(13).x_low(), 0x15);
        assert_eq!(pirogusu.garnish_slot_view(13).x_high(), 0x01);
        assert_eq!(pirogusu.garnish_slot_view(13).y_low(), 0x34);
        assert_eq!(pirogusu.garnish_slot_view(13).y_high(), 0x02);
        assert_eq!(pirogusu.garnish_slot_view(13).countdown(), 15);

        let mut lightning = fresh_state();
        lightning.garnish_slot_view_mut(29).set_garnish_type(1);
        lightning.sprite_set_x(k, 0x0123);
        lightning.sprite_set_y(k, 0x02f4);
        lightning.sprite_slot_mut(k).set_a(7);
        lightning.lightning_spawn_garnish(k);
        assert_eq!(lightning.garnish_slot_view(28).garnish_type(), 9);
        assert_eq!(
            lightning.game_state.sprites.garnish_runtime.active_type(),
            9
        );
        assert_eq!(lightning.garnish_slot_view(28).sprite(), 7);
        assert_eq!(lightning.garnish_slot_view(28).x_low(), 0x23);
        assert_eq!(lightning.garnish_slot_view(28).x_high(), 0x01);
        assert_eq!(lightning.garnish_slot_view(28).y_low(), 0x04);
        assert_eq!(lightning.garnish_slot_view(28).y_high(), 0x03);
        assert_eq!(lightning.garnish_slot_view(28).countdown(), 32);

        let mut laser = fresh_state();
        laser.garnish_slot_view_mut(29).set_garnish_type(1);
        laser.sprite_set_x(k, 0x0034);
        laser.sprite_set_y(k, 0x00f0);
        laser.sprite_slot_mut(k).set_graphics(5);
        laser.sprite_slot_mut(k).set_floor(2);
        laser.laser_beam_build_up_garnish(k);
        assert_eq!(laser.garnish_slot_view(28).garnish_type(), 4);
        assert_eq!(laser.game_state.sprites.garnish_runtime.active_type(), 4);
        assert_eq!(laser.garnish_slot_view(28).x_low(), 0x34);
        assert_eq!(laser.garnish_slot_view(28).x_high(), 0x00);
        assert_eq!(laser.garnish_slot_view(28).y_low(), 0x00);
        assert_eq!(laser.garnish_slot_view(28).y_high(), 0x01);
        assert_eq!(laser.garnish_slot_view(28).countdown(), 16);
        assert_eq!(laser.garnish_slot_view(28).oam_flags(), 5);
        assert_eq!(laser.garnish_slot_view(28).sprite(), k as u8);
        assert_eq!(laser.garnish_slot_view(28).floor(), 2);

        let mut logic = fresh_state();
        assert!(!logic.octoballoon_find());
        logic.sprite_slot_mut(10).set_state(9);
        logic.sprite_slot_mut(10).set_sprite_type(0x10);
        assert!(logic.octoballoon_find());

        assert!(!logic.potion_cauldron_check_bottles());
        logic.inventory_items_mut().set_bottle(2, 2);
        assert!(logic.potion_cauldron_check_bottles());
        logic.potion_cauldron_go_beep(k);
        assert_eq!(
            logic.game_state.system_signals.sound_effect_1() & 0x3f,
            0x3c
        );

        logic.player_resources_mut().set_rupees_goal(19);
        assert!(!logic.dark_world_hint_npc_handle_payment());
        assert_eq!(
            logic.game_state.inventory.player_resources.rupees_goal(),
            19
        );
        logic.player_resources_mut().set_rupees_goal(20);
        assert!(logic.dark_world_hint_npc_handle_payment());
        assert_eq!(logic.game_state.inventory.player_resources.rupees_goal(), 0);
        logic.sprite_slot_mut(k).set_ai_state(0);
        logic.dark_world_hint_npc_idle(k);
        assert_eq!(
            logic.game_state.messaging.dialogue_message_index.value(),
            0xfe
        );
        assert_eq!(logic.sprite_slot_view(k).ai_state(), 0);

        logic.set_submodule(2);
        logic.dialogue_message_index_mut().set_value(0xc9);
        logic.fairy_check_if_touchable(k);
        assert_eq!(logic.sprite_slot_view(k).delay_aux4(), 40);
        logic.sprite_slot_mut(k).set_delay_aux4(0);
        logic.dialogue_message_index_mut().set_value(0xcb);
        logic.fairy_check_if_touchable(k);
        assert_eq!(logic.sprite_slot_view(k).delay_aux4(), 0);

        let mut buzzblob = fresh_state();
        buzzblob.buzzblob_select_new_direction(k);
        assert_eq!(buzzblob.sprite_slot_view(k).x_velocity(), 3);
        assert_eq!(buzzblob.sprite_slot_view(k).y_velocity(), 0);
        assert_eq!(buzzblob.sprite_slot_view(k).delay_main(), 48);

        let mut lumberjack = fresh_state();
        lumberjack
            .sprite_workspace_mut()
            .set_current_sprite_x(0x0100);
        lumberjack
            .sprite_workspace_mut()
            .set_current_sprite_y(0x0200);
        lumberjack.follower_link_state_mut().set_x(0x0100);
        lumberjack.follower_link_state_mut().set_y(0x0200);
        assert!(lumberjack.lumberjack_check_proximity(k, 0));
        lumberjack.follower_link_state_mut().set_x(0x0200);
        assert!(!lumberjack.lumberjack_check_proximity(k, 0));

        let mut blind_laser = fresh_state();
        blind_laser.garnish_slot_view_mut(29).set_garnish_type(1);
        blind_laser.sprite_set_x(k, 0x0456);
        blind_laser.sprite_set_y(k, 0x0789);
        blind_laser.sprite_slot_mut(k).set_graphics(6);
        blind_laser.blind_laser_spawn_trail_garnish(k);
        assert_eq!(blind_laser.garnish_slot_view(28).garnish_type(), 15);
        assert_eq!(
            blind_laser.game_state.sprites.garnish_runtime.active_type(),
            15
        );
        assert_eq!(blind_laser.garnish_slot_view(28).oam_flags(), 6);
        assert_eq!(blind_laser.garnish_slot_view(28).sprite(), k as u8);
        assert_eq!(blind_laser.garnish_slot_view(28).x_low(), 0x56);
        assert_eq!(blind_laser.garnish_slot_view(28).x_high(), 0x04);
        assert_eq!(blind_laser.garnish_slot_view(28).y_low(), 0x99);
        assert_eq!(blind_laser.garnish_slot_view(28).y_high(), 0x07);
        assert_eq!(blind_laser.garnish_slot_view(28).countdown(), 10);

        let mut runner_dust = fresh_state();
        runner_dust.sprite_slot_mut(k).set_die_action(14);
        runner_dust.running_boy_spawn_dust_garnish(k);
        assert_eq!(
            runner_dust.game_state.sprites.garnish_runtime.active_type(),
            0
        );
        runner_dust.sprite_slot_mut(k).set_die_action(15);
        runner_dust.sprite_set_x(k, 0x0100);
        runner_dust.sprite_set_y(k, 0x0200);
        runner_dust.garnish_slot_view_mut(29).set_garnish_type(1);
        runner_dust.running_boy_spawn_dust_garnish(k);
        assert_eq!(runner_dust.garnish_slot_view(28).garnish_type(), 20);
        assert_eq!(
            runner_dust.game_state.sprites.garnish_runtime.active_type(),
            20
        );
        assert_eq!(runner_dust.garnish_slot_view(28).x_low(), 0x04);
        assert_eq!(runner_dust.garnish_slot_view(28).x_high(), 0x01);
        assert_eq!(runner_dust.garnish_slot_view(28).y_low(), 0x1c);
        assert_eq!(runner_dust.garnish_slot_view(28).y_high(), 0x02);
        assert_eq!(runner_dust.garnish_slot_view(28).countdown(), 10);

        let mut cd = fresh_state();
        cd.sprite_slot_mut(k).set_subtype2(6);
        cd.sprite_cd_spawn_garnish(k);
        assert_eq!(cd.game_state.sprites.garnish_runtime.active_type(), 0);
        cd.sprite_slot_mut(k).set_subtype2(7);
        cd.garnish_slot_view_mut(29).set_garnish_type(1);
        cd.sprite_set_x(k, 0x0033);
        cd.sprite_set_y(k, 0x0044);
        cd.sprite_cd_spawn_garnish(k);
        assert_eq!(cd.sprite_slot_view(k).subtype2(), 8);
        assert_eq!(cd.game_state.system_signals.sound_effect_2() & 0x3f, 0x14);
        assert_eq!(cd.garnish_slot_view(28).garnish_type(), 0x0c);
        assert_eq!(cd.game_state.sprites.garnish_runtime.active_type(), 0x0c);
        assert_eq!(cd.garnish_slot_view(28).sprite(), k as u8);
        assert_eq!(cd.garnish_slot_view(28).x_low(), 0x33);
        assert_eq!(cd.garnish_slot_view(28).y_low(), 0x54);
        assert_eq!(cd.garnish_slot_view(28).countdown(), 127);

        let mut hint = fresh_state();
        hint.sprite_slot_mut(k).set_ai_state(2);
        hint.dark_world_hint_npc_restore_health(k);
        assert_eq!(
            hint.game_state.inventory.player_resources.heart_filler(),
            0xa0
        );
        assert_eq!(hint.sprite_slot_view(k).ai_state(), 0);

        let mut pipe = fresh_state();
        pipe.follower_link_state_mut().set_position_mode(7);
        pipe.player_state_mut().set_direction_lock(9);
        pipe.ancilla_slot_view_mut(3).set_ancilla_type(0x31);
        assert!(!pipe.pipe_validate_entry());
        assert_eq!(pipe.game_state.player.follower_link.position_mode(), 0);
        assert_eq!(pipe.game_state.player.follower_link.direction_lock(), 0);
        assert_eq!(pipe.ancilla_slot_view(3).ancilla_type(), 0);
        pipe.follower_link_state_mut().set_state_bits(0x80);
        assert!(pipe.pipe_validate_entry());
        pipe.follower_link_state_mut().clear_state_bits();
        pipe.follower_link_state_mut().set_auxiliary_state(2);
        assert!(pipe.pipe_validate_entry());

        let mut hobo_smoke = fresh_state();
        hobo_smoke.sprite_slot_mut(k).set_state(9);
        hobo_smoke.sprite_set_x(k, 0x0030);
        hobo_smoke.sprite_set_y(k, 0x0040);
        hobo_smoke.sprite_prep_hobo_spawn_smoke(k);
        assert_eq!(hobo_smoke.sprite_slot_view(15).sprite_type(), 0x2b);
        assert_eq!(hobo_smoke.sprite_get_x(15), 0x0030);
        assert_eq!(hobo_smoke.sprite_get_y(15), 0x0040);
        assert_eq!(hobo_smoke.sprite_slot_view(15).subtype2(), 0);
        assert_eq!(hobo_smoke.sprite_slot_view(15).ignore_projectile(), 0);

        let mut hobo_fire = fresh_state();
        hobo_fire.sprite_slot_mut(k).set_state(9);
        hobo_fire.sprite_slot_mut(15).set_oam_flags(0xff);
        hobo_fire.sprite_prep_hobo_spawn_fire(k);
        assert_eq!(hobo_fire.sprite_slot_view(15).sprite_type(), 0x2b);
        assert_eq!(hobo_fire.sprite_get_x(15), 0x0194);
        assert_eq!(hobo_fire.sprite_get_y(15), 0x003f);
        assert_eq!(hobo_fire.sprite_slot_view(15).subtype2(), 2);
        assert_eq!(hobo_fire.sprite_slot_view(15).ignore_projectile(), 2);
        assert_eq!(hobo_fire.sprite_slot_view(15).flags2(), 0);
        assert_eq!(hobo_fire.sprite_slot_view(15).oam_flags() & 0x0f, 0x03);

        let mut hobo_bubble = fresh_state();
        hobo_bubble.sprite_slot_mut(k).set_state(9);
        hobo_bubble.sprite_set_x(k, 0x0050);
        hobo_bubble.sprite_set_y(k, 0x0060);
        assert_eq!(hobo_bubble.hobo_spawn_bubble(k), 15);
        assert_eq!(hobo_bubble.sprite_slot_view(15).sprite_type(), 0x2b);
        assert_eq!(hobo_bubble.sprite_get_x(15), 0x0050);
        assert_eq!(hobo_bubble.sprite_get_y(15), 0x0060);
        assert_eq!(hobo_bubble.sprite_slot_view(15).subtype2(), 1);
        assert_eq!(hobo_bubble.sprite_slot_view(15).z_velocity(), 2);
        assert_eq!(hobo_bubble.sprite_slot_view(15).delay_main(), 96);
        assert_eq!(hobo_bubble.sprite_slot_view(15).delay_aux1(), 48);
        assert_eq!(hobo_bubble.sprite_slot_view(15).ignore_projectile(), 48);
        assert_eq!(hobo_bubble.sprite_slot_view(15).flags2(), 0);

        let mut hobo_smoke_active = fresh_state();
        hobo_smoke_active.sprite_slot_mut(k).set_state(9);
        hobo_smoke_active.sprite_set_x(k, 0x0070);
        hobo_smoke_active.sprite_set_y(k, 0x0080);
        hobo_smoke_active.hobo_spawn_smoke(k);
        assert_eq!(hobo_smoke_active.sprite_slot_view(15).sprite_type(), 0x2b);
        assert_eq!(hobo_smoke_active.sprite_get_x(15), 0x0070);
        assert_eq!(hobo_smoke_active.sprite_get_y(15), 0x007c);
        assert_eq!(hobo_smoke_active.sprite_slot_view(15).subtype2(), 3);
        assert_eq!(hobo_smoke_active.sprite_slot_view(15).z_velocity(), 7);
        assert_eq!(hobo_smoke_active.sprite_slot_view(15).delay_main(), 96);
        assert_eq!(
            hobo_smoke_active.sprite_slot_view(15).ignore_projectile(),
            96
        );
        assert_eq!(hobo_smoke_active.sprite_slot_view(15).flags2(), 0);

        let mut hobo = fresh_state();
        hobo.sprite_slot_mut(k).set_state(9);
        hobo.sprite_set_x(k, 0x0080);
        hobo.sprite_set_y(k, 0x0090);
        hobo.save_progress_mut().set_progress_indicator_3(1);
        hobo.sprite_prep_hobo(k);
        assert_eq!(hobo.sprite_slot_view(0).ai_state(), 3);
        assert_eq!(hobo.sprite_slot_view(0).ignore_projectile(), 1);
        assert_eq!(hobo.sprite_slot_view(15).state(), 9);
        assert_eq!(hobo.sprite_slot_view(1).state(), 0);
        assert_eq!(hobo.sprite_slot_view(15).sprite_type(), 0x2b);
        assert_eq!(hobo.sprite_slot_view(15).subtype2(), 2);
        assert_eq!(hobo.sprite_get_x(15), 0x0194);
        assert_eq!(hobo.sprite_get_y(15), 0x003f);

        let mut tree = fresh_state();
        tree.sprite_slot_mut(k).set_state(9);
        tree.sprite_set_x(k, 0x0120);
        tree.sprite_set_y(k, 0x0240);
        tree.sprite_prep_talking_tree(k);
        assert_eq!(tree.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(tree.sprite_get_x(k), 0x0118);
        assert_eq!(tree.sprite_slot_view(15).sprite_type(), 0x25);
        assert_eq!(tree.sprite_slot_view(15).head_direction(), 0);
        assert_eq!(tree.sprite_get_x(15), 0x0114);
        assert_eq!(tree.sprite_get_y(15), 0x0235);
        assert_eq!(tree.sprite_slot_view(15).subtype2(), 1);
        assert_eq!(tree.sprite_slot_view(14).sprite_type(), 0x25);
        assert_eq!(tree.sprite_slot_view(14).head_direction(), 1);
        assert_eq!(tree.sprite_get_x(14), 0x0126);
        assert_eq!(tree.sprite_get_y(14), 0x0235);
        assert_eq!(tree.sprite_slot_view(14).a(), 0x26);
        assert_eq!(tree.sprite_slot_view(14).b(), 0x01);
        assert_eq!(tree.sprite_slot_view(14).c(), 0x35);
        assert_eq!(tree.sprite_slot_view(14).e(), 0x02);
    }

    #[test]
    fn shopkeeper_and_antifairy_circle_prep_spawn_expected_helpers() {
        let k = 4;

        let mut shop = fresh_state();
        shop.sprite_slot_mut(k).set_state(9);
        shop.set_dungeon_room_index(0x0f);
        shop.sprite_set_x(k, 0x0200);
        shop.sprite_set_y(k, 0x0100);
        shop.sprite_prep_shopkeeper(k);
        assert_eq!(shop.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(shop.sprite_slot_view(k).flags2() & 2, 2);
        assert_eq!(shop.sprite_slot_view(k).oam_flags() & 12, 12);
        assert_eq!(shop.sprite_slot_view(k).flags3() & 16, 16);
        for (slot, what, x) in [
            (12, 7, 0x0200u16.wrapping_sub(44)),
            (11, 8, 0x0200u16.wrapping_add(8)),
            (10, 12, 0x0200u16.wrapping_add(60)),
        ] {
            assert_eq!(shop.sprite_slot_view(slot).state(), 9);
            assert_eq!(shop.sprite_slot_view(slot).sprite_type(), 0xbb);
            assert_eq!(shop.sprite_slot_view(slot).ignore_projectile(), what);
            assert_eq!(shop.sprite_slot_view(slot).subtype2(), what);
            assert_eq!(shop.sprite_get_x(slot), x);
            assert_eq!(shop.sprite_get_y(slot), 0x0127);
            assert_eq!(shop.sprite_slot_view(slot).flags2() & 4, 4);
        }

        let mut minigame = fresh_state();
        minigame.sprite_slot_mut(k).set_state(9);
        minigame.set_dungeon_room_index(0x06);
        minigame.sprite_prep_shopkeeper(k);
        assert_eq!(minigame.sprite_slot_view(k).subtype2(), 1);
        assert_eq!(minigame.sprite_slot_view(k).graphics(), 1);
        assert_eq!(minigame.ram[MINIGAME_CREDITS_PREP], 0xff);

        let mut terminate = fresh_state();
        terminate.ancilla_slot_view_mut(0).set_ancilla_type(0x22);
        terminate.ancilla_slot_view_mut(1).set_ancilla_type(0x21);
        terminate.ancilla_slot_view_mut(4).set_ancilla_type(0x22);
        terminate.ram[ANCILLA_AUX_TIMER] = 9;
        terminate.ram[ANCILLA_AUX_TIMER + 1] = 9;
        terminate.ram[ANCILLA_AUX_TIMER + 4] = 9;
        terminate.shop_keeper_rapid_terminate_receive_item();
        assert_eq!(terminate.ram[ANCILLA_AUX_TIMER], 1);
        assert_eq!(terminate.ram[ANCILLA_AUX_TIMER + 1], 9);
        assert_eq!(terminate.ram[ANCILLA_AUX_TIMER + 4], 1);

        let mut bat = fresh_state();
        bat.sprite_spawn_bat_crash_cutscene();
        assert_eq!(bat.sprite_slot_view(15).sprite_type(), 0x37);
        assert_eq!(bat.sprite_slot_view(15).y_velocity(), 0);
        assert_eq!(bat.sprite_slot_view(15).b(), 0);
        assert_eq!(bat.sprite_slot_view(15).direction(), 0);
        assert_eq!(bat.sprite_slot_view(15).floor(), 0);
        assert_eq!(bat.sprite_slot_view(15).subtype2(), 1);
        assert_eq!(bat.sprite_slot_view(15).flags2(), 1);
        assert_eq!(bat.sprite_slot_view(15).flags3(), 1);
        assert_eq!(bat.sprite_slot_view(15).oam_flags(), 1);
        assert_eq!(bat.sprite_get_x(15), 0x07cc);
        assert_eq!(bat.sprite_get_y(15), 0x0632);
        assert_eq!(bat.sprite_slot_view(15).deflection_bits(), 128);

        let mut circle = fresh_state();
        circle.sprite_slot_mut(k).set_state(9);
        circle.sprite_set_x(k, 0x0100);
        circle.sprite_set_y(k, 0x0200);
        circle.sprite_slot_mut(k).set_a(9);
        circle.sprite_slot_mut(k).set_b(9);
        circle.sprite_prep_antifairy_circle(k);
        assert_eq!(circle.sprite_get_x(k), 0x00f6);
        assert_eq!(circle.sprite_slot_view(k).y_velocity(), (-18i8) as u8);
        assert_eq!(circle.sprite_slot_view(k).x_velocity(), 0);
        assert_eq!(circle.sprite_slot_view(k).a(), 0);
        assert_eq!(circle.sprite_slot_view(k).b(), 0);
        assert_eq!(circle.game_state.scratch_counter.value(), 0xff);

        for (slot, x, y, xv, yv, a, b) in [
            (
                15,
                0x00f6u16.wrapping_add(10),
                0x0200u16.wrapping_add(10),
                (-18i8) as u8,
                0,
                0,
                1,
            ),
            (14, 0x00f6u16.wrapping_add(20), 0x0200, 0, 18, 1, 1),
            (
                13,
                0x00f6u16.wrapping_add(10),
                0x0200u16.wrapping_sub(10),
                18,
                0,
                1,
                0,
            ),
        ] {
            assert_eq!(circle.sprite_slot_view(slot).state(), 9);
            assert_eq!(circle.sprite_slot_view(slot).sprite_type(), 0x82);
            assert_eq!(circle.sprite_get_x(slot), x);
            assert_eq!(circle.sprite_get_y(slot), y);
            assert_eq!(circle.sprite_slot_view(slot).x_velocity(), xv);
            assert_eq!(circle.sprite_slot_view(slot).y_velocity(), yv);
            assert_eq!(circle.sprite_slot_view(slot).a(), a);
            assert_eq!(circle.sprite_slot_view(slot).b(), b);
        }
    }

    #[test]
    fn medallion_table_and_eyegore_prep_match_room_and_item_gates() {
        let k = 7;

        let mut bombos = fresh_state();
        bombos.set_overworld_screen(2);
        bombos.inventory_items_mut().set_bombos(1);
        bombos.sprite_slot_mut(k).set_x_low(0xf9);
        bombos.sprite_prep_medallion_table(k);
        assert_eq!(bombos.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(bombos.sprite_slot_view(k).x_low(), 1);
        assert_eq!(bombos.sprite_slot_view(k).graphics(), 4);
        assert_eq!(bombos.sprite_slot_view(k).ai_state(), 3);

        let mut ether_only_on_bombos_screen = fresh_state();
        ether_only_on_bombos_screen.set_overworld_screen(2);
        ether_only_on_bombos_screen
            .inventory_items_mut()
            .set_ether(1);
        ether_only_on_bombos_screen.sprite_prep_medallion_table(k);
        assert_eq!(
            ether_only_on_bombos_screen
                .sprite_slot_view(k)
                .ignore_projectile(),
            1
        );
        assert_eq!(
            ether_only_on_bombos_screen.sprite_slot_view(k).graphics(),
            0
        );
        assert_eq!(
            ether_only_on_bombos_screen.sprite_slot_view(k).ai_state(),
            0
        );

        let mut ether = fresh_state();
        ether.set_overworld_screen(3);
        ether.inventory_items_mut().set_ether(1);
        ether.sprite_slot_mut(k).set_x_low(0x20);
        ether.sprite_prep_medallion_table(k);
        assert_eq!(ether.sprite_slot_view(k).ignore_projectile(), 1);
        assert_eq!(ether.sprite_slot_view(k).x_low(), 0x20);
        assert_eq!(ether.sprite_slot_view(k).graphics(), 4);
        assert_eq!(ether.sprite_slot_view(k).ai_state(), 3);

        let mut eyegore = fresh_state();
        eyegore.dungeon_room_tracking_mut().set_room_index2(75);
        eyegore.sprite_slot_mut(k).set_sprite_type(0x83);
        eyegore.sprite_slot_mut(k).set_b(0xff);
        eyegore.sprite_slot_mut(k).set_deflection_bits(0xaa);
        eyegore.sprite_prep_eyegore(k);
        assert_eq!(eyegore.sprite_slot_view(k).b(), 0);
        assert_eq!(eyegore.sprite_slot_view(k).deflection_bits(), 0);

        let mut untouched = fresh_state();
        untouched.dungeon_room_tracking_mut().set_room_index2(74);
        untouched.sprite_slot_mut(k).set_b(4);
        untouched.sprite_slot_mut(k).set_deflection_bits(0xaa);
        untouched.sprite_prep_eyegore(k);
        assert_eq!(untouched.sprite_slot_view(k).b(), 4);
        assert_eq!(untouched.sprite_slot_view(k).deflection_bits(), 0xaa);
    }
}
