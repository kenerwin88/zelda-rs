//! Ported SpritePrep_* helpers from sprite_main.c.

use super::*;
use crate::types::{read_le_u16, sign16, sign8, write_le_u16};
use crate::zelda_rtl::sprite::{DrawMultipleData, SpriteSpawnInfo};

const IS_IN_DARK_WORLD_PREP: usize = 0x0fff;
const DUNG_FLOOR_MOVE_FLAGS_PREP: usize = 0x041a;
const ACTIVE_OVERLORD_INDEX_PREP: usize = 0x0fde;
const SPRITE_PREP_SHARED_COUNTER: usize = 0x0ff8;
const DIALOGUE_NUMBER_PREP: usize = 0x1cf2;
const LINK_RUPEES_IN_POND_PREP: usize = 0x0f36a;
const ITEM_DROP_LUCK_PREP: usize = 0x0cf9;
const LUCK_KILL_COUNTER_PREP: usize = 0x0cfa;
const NUM_SPRITES_KILLED_PREP: usize = 0x0cfb;
const SPRITE_DELAY_AUX3_PREP: usize = 0x0ee0;
const MINIGAME_CREDITS_PREP: usize = 0x04c4;
const FLAG_OVERWORLD_AREA_DID_CHANGE_PREP: usize = 0x0abf;
const ALT_SPRITE_STATE_PREP: usize = 0x1d00;
const ALT_SPRITE_TYPE_PREP: usize = 0x1d10;
const ALT_SPRITE_X_HI_PREP: usize = 0x1d30;
const ALT_SPRITE_Y_HI_PREP: usize = 0x1d50;
const SRAM_PROGRESS_INDICATOR_3_PREP: usize = 0x0f3c9;
const GARNISH_ACTIVE_PREP: usize = 0x0fb4;
const GARNISH_Y_LO_PREP: usize = 0x1f81e;
const GARNISH_X_LO_PREP: usize = 0x1f83c;
const GARNISH_Y_HI_PREP: usize = 0x1f85a;
const GARNISH_X_HI_PREP: usize = 0x1f878;
const GARNISH_Y_VEL_PREP: usize = 0x1f896;
const GARNISH_X_VEL_PREP: usize = 0x1f8b4;
const GARNISH_COUNTDOWN_PREP: usize = 0x1f90e;
const GARNISH_SPRITE_PREP: usize = 0x1f92c;
const GARNISH_FLOOR_PREP: usize = 0x1f968;
const GARNISH_OAM_FLAGS_PREP: usize = 0x1f9fe;
const SPRCOLL_X_BASE_PREP: usize = 0x0fbc;
const SPRCOLL_Y_BASE_PREP: usize = 0x0fbe;
const BEAMOS_X_LO_PREP: usize = 0x1fd80;
const BEAMOS_Y_LO_PREP: usize = 0x1fe80;
const BEAMOS_Y_HI_PREP: usize = 0x1ff00;
const MOLDORM_X_LO_PREP: usize = 0x1fc00;
const MOLDORM_X_HI_PREP: usize = 0x1fc80;
const MOLDORM_Y_LO_PREP: usize = 0x1fd00;
const MOLDORM_Y_HI_PREP: usize = 0x1fd80;
const CHAINCHOMP_X_HIST_PREP: usize = 0x1fc00;
const CHAINCHOMP_Y_HIST_PREP: usize = 0x1fd00;
const OVERLORD_X_LO_PREP: usize = 0x0b08;
const OVERLORD_X_HI_PREP: usize = 0x0b10;
const OVERLORD_Y_LO_PREP: usize = 0x0b18;
const OVERLORD_Y_HI_PREP: usize = 0x0b20;
const OVERLORD_GEN1_PREP: usize = 0x0b28;
const OVERLORD_GEN2_PREP: usize = 0x0b30;
const OVERLORD_GEN3_PREP: usize = 0x0b38;
const OVERLORD_FLOOR_PREP: usize = 0x0b40;
const SWAMOLA_X_LO_PREP: usize = 0x1fa5c;
const SWAMOLA_X_HI_PREP: usize = 0x1fb1c;
const SWAMOLA_Y_LO_PREP: usize = 0x1fbdc;
const SWAMOLA_Y_HI_PREP: usize = 0x1fc9c;
const K_FEATURES0_MISC_BUG_FIXES_PREP: u32 = 4096;

const K_WISH_POND_X: [u8; 8] = [0, 4, 8, 12, 16, 20, 24, 0];
const K_WISH_POND_Y: [u8; 8] = [0, 8, 16, 24, 32, 40, 4, 36];
const K_WISH_POND2_OAM_FLAGS: [u8; 76] = [
    5, 0xff, 5, 5, 5, 5, 5, 1, 2, 1, 1, 1, 2, 2, 2, 4, 4, 4, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1, 4, 4, 2,
    1, 6, 1, 2, 1, 2, 2, 1, 2, 2, 4, 1, 1, 4, 2, 1, 4, 2, 2, 4, 4, 4, 2, 1, 4, 1, 2, 2, 1, 2, 2, 1,
    1, 4, 4, 1, 2, 2, 4, 4, 4, 2, 5, 2, 1,
];
const K_RECEIVE_ITEM_TAB1_PREP: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
const K_WISH_POND_ITEM_OFFS: [u8; 32] = [
    0, 4, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 17, 20, 21, 22, 22, 23, 24, 25, 28, 30, 31, 32, 33,
    33, 37, 40, 42, 42, 42, 42,
];
const K_WISH_POND_ITEM_DATA: [u8; 50] = [
    0x3a, 0x3a, 0x3b, 0x3b, 0x0c, 0x2a, 0x0a, 0x27, 0x29, 0x0d, 0x07, 0x08, 0x0f, 0x10, 0x11, 0x12,
    0x09, 0x13, 0x14, 0x4a, 0x21, 0x1d, 0x15, 0x18, 0x19, 0x31, 0x1a, 0x1a, 0x1b, 0x1c, 0x4b, 0x1e,
    0x1f, 0x49, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x22, 0x23, 0x29, 0x16, 0x2b, 0x2c, 0x2d, 0x3d,
    0x3c, 0x48,
];

const K_SPRITE_INIT_BUMP_DAMAGE_PREP: [u8; 243] = [
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
        self.ram[SPRITE_STATE + k] = self.ram[SPRITE_STATE + k].wrapping_add(1);
        match self.ram[SPRITE_TYPE + k] {
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
        const TAB0: [u8; 8] = [0, 2, 1, 3, 6, 4, 5, 7];

        let subtype = self.ram[SPRITE_SUBTYPE + k];
        if subtype != 0 {
            if (subtype & 7) >= 5 {
                let j = usize::from(if (subtype & 7) != 5 { 4 } else { 0 } + ((subtype >> 3) & 3));
                self.ram[SPRITE_B + k] = TAB0[j];
                self.ram[SPRITE_FLAGS + k] = (self.ram[SPRITE_FLAGS + k] & 0x0f) | 0x50;
                self.sprite_prep_trooper_and_archer_soldier(k);
                return;
            }
            self.ram[SPRITE_D + k] = ((subtype & 7).wrapping_sub(1)) ^ 1;
        }
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            self.ram[SPRITE_FLAGS5 + k] &= !0x80;
            return;
        }
        self.ram[SPRITE_AI_STATE + k] = 1;
        self.ram[SPRITE_DELAY_MAIN + k] = 112;
        let dir = self.sprite_direction_to_face_link(k, None);
        self.ram[SPRITE_D + k] = dir;
        self.ram[SPRITE_HEAD_DIR + k] = dir;
        self.sprite_prep_trooper_and_archer_soldier(k);
    }

    // void SpritePrep_TrooperAndArcherSoldier(int k) {  // 869001
    pub(super) fn sprite_prep_trooper_and_archer_soldier(&mut self, k: usize) {
        let bak0 = self.frame_control_view().submodule();
        self.frame_control_view_mut().set_submodule(0);
        self.ram[SPRITE_DEFL_BITS + k] = (self.ram[SPRITE_DEFL_BITS + k] >> 1) | 0x80;
        self.sprite_active_main(k);
        self.sprite_active_main(k);
        self.ram[SPRITE_DEFL_BITS + k] = self.ram[SPRITE_DEFL_BITS + k].wrapping_shl(1);
        self.frame_control_view_mut().set_submodule(bak0);
    }

    pub(super) fn sprite_prep_mantle(&mut self, k: usize) {
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(3);
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
    }

    pub(super) fn sprite_prep_switch(&mut self, k: usize) {
        let room = self.ram[DUNGEON_ROOM_INDEX2];
        if room == 0xce || room == 4 || room == 0x3f {
            self.ram[SPRITE_OAM_FLAGS + k] = 0x0d;
        }
    }

    pub(super) fn sprite_prep_switch_facing_up(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_do_nothing_a(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_rat(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [0, 5];
        const HEALTH: [u8; 2] = [2, 8];
        let j = self.ram[IS_IN_DARK_WORLD_PREP] as usize;
        self.ram[SPRITE_BUMP_DAMAGE + k] = BUMP_DAMAGE[j];
        self.ram[SPRITE_HEALTH + k] = HEALTH[j];
    }

    pub(super) fn sprite_prep_keese(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [0x80, 0x85];
        const HEALTH: [u8; 2] = [1, 4];
        const FLAGS5: [u8; 2] = [0, 7];
        let j = self.ram[IS_IN_DARK_WORLD_PREP] as usize;
        self.ram[SPRITE_BUMP_DAMAGE + k] = BUMP_DAMAGE[j];
        self.ram[SPRITE_HEALTH + k] = HEALTH[j];
        self.ram[SPRITE_FLAGS5 + k] = FLAGS5[j];
    }

    pub(super) fn sprite_prep_rope(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [1, 5];
        const HEALTH: [u8; 2] = [4, 8];
        const FLAGS5: [u8; 2] = [1, 7];
        let j = self.ram[IS_IN_DARK_WORLD_PREP] as usize;
        self.ram[SPRITE_BUMP_DAMAGE + k] = BUMP_DAMAGE[j];
        self.ram[SPRITE_HEALTH + k] = HEALTH[j];
        self.ram[SPRITE_FLAGS5 + k] = FLAGS5[j];
    }

    pub(super) fn sprite_prep_babasu(&mut self, k: usize) {
        self.sprite_prep_move_down_8px(k);
        self.sprite_prep_zoro(k);
    }

    pub(super) fn sprite_prep_pokey(&mut self, k: usize) {
        const INIT_XVEL: [i8; 4] = [16, -16, 16, -16];
        const INIT_YVEL: [i8; 4] = [16, 16, -16, -16];
        self.ram[SPRITE_A + k] = 3;
        self.ram[SPRITE_B + k] = 8;
        let j = (self.get_random_number() & 3) as usize;
        self.ram[SPRITE_X_VEL + k] = INIT_XVEL[j] as u8;
        self.ram[SPRITE_Y_VEL + k] = INIT_YVEL[j] as u8;
    }

    pub(super) fn sprite_prep_gibo(&mut self, k: usize) {
        self.ram[SPRITE_Z + k] = 16;
        self.ram[SPRITE_G + k] = 8;
    }

    pub(super) fn sprite_prep_octoballoon(&mut self, k: usize) {
        const DELAY: [u8; 4] = [192, 208, 224, 240];
        self.ram[SPRITE_DELAY_MAIN + k] = DELAY[k & 3];
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
        self.ram[SPRITE_DELAY_MAIN + k] = 128;
        self.ram[SPRITE_ROOM + k] = 2;
        self.ram[MUSIC_CONTROL] = 0x1e;
    }

    pub(super) fn sprite_prep_mini_vitreous(&mut self, k: usize) {
        self.sprite_return_if_boss_finished(k);
    }

    pub(super) fn sprite_prep_agahnims_barrier(&mut self, k: usize) {
        if self.ram[SAVE_OW_EVENT_INFO + self.ram[OVERWORLD_SCREEN_INDEX] as usize] & 0x40 != 0 {
            self.ram[SPRITE_GRAPHICS + k] = 4;
        }
        self.sprite_prep_move_down_8px_right8px(k);
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(12);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_catfish(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right8px(k);
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(12);
        self.sprite_prep_ignore_projectiles(k);
    }

    pub(super) fn sprite_prep_cutscene_agahnim(&mut self, k: usize) {
        if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x4000 != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        } else {
            self.cutscene_agahnim_spawn_zelda_on_altar(k);
            self.ram[SPRITE_IGNORE_PROJECTILE + k] =
                self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        }
    }

    pub(super) fn cutscene_agahnim_spawn_zelda_on_altar(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(6);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc1, &mut info);
        let j = j as usize;
        self.ram[SPRITE_A + j] = 1;
        self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
        self.sprite_set_spawned_coordinates(j, &info);
        self.ram[SPRITE_Y_LO + j] = (info.r2_y as u8).wrapping_add(40);
        self.ram[SPRITE_FLAGS2 + j] = 0;
        self.ram[SPRITE_OAM_FLAGS + j] = 12;
    }

    pub(super) fn sprite_prep_vitreous(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.sprite_prep_move_down_8px_right8px(k);
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(16);
        self.vitreous_spawn_smaller_eyes(k);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_raven(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [0x81, 0x88];
        const HEALTH: [u8; 2] = [4, 8];
        const FLAGS5: [u8; 2] = [6, 2];
        let j = self.ram[IS_IN_DARK_WORLD_PREP] as usize;
        self.ram[SPRITE_BUMP_DAMAGE + k] = BUMP_DAMAGE[j];
        self.ram[SPRITE_HEALTH + k] = HEALTH[j];
        self.ram[SPRITE_FLAGS5 + k] = FLAGS5[j];
        self.sprite_prep_vulture(k);
    }

    pub(super) fn sprite_prep_vulture(&mut self, k: usize) {
        self.ram[SPRITE_Z + k] = 0;
        self.ram[SPRITE_A + k] = (self.ram[SPRITE_X_LO + k] & 16) >> 4;
        self.ram[SPRITE_SUBTYPE + k] = 254;
    }

    pub(super) fn sprite_prep_poe(&mut self, k: usize) {
        self.ram[SPRITE_Z + k] = 12;
        self.ram[SPRITE_SUBTYPE + k] = 254;
    }

    pub(super) fn sprite_prep_do_nothing_c(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_blind_maiden(&mut self, k: usize) {
        if read_le_u16(&self.ram, SAVE_DUNG_INFO + 0xac * 2) & 0x0800 == 0 {
            self.ram[SPRITE_IGNORE_PROJECTILE + k] =
                self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
            if self.ram[FOLLOWER_INDICATOR] != 6 {
                self.ram[FOLLOWER_INDICATOR] = 6;
                self.ram[FOLLOWER_DROPPED] = 0;
                self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 0;
                self.load_follower_graphics();
                self.follower_initialize();
                self.ram[FOLLOWER_INDICATOR] = 0;
                return;
            }
        }
        self.ram[SPRITE_STATE + k] = 0;
    }

    pub(super) fn sprite_prep_snitches(&mut self, k: usize) {
        self.ram[SPRITE_D + k] = 2;
        self.ram[SPRITE_HEAD_DIR + k] = 2;
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.ram[SPRITE_A + k] = self.ram[SPRITE_X_LO + k];
        self.ram[SPRITE_B + k] = self.ram[SPRITE_X_HI + k];
        self.ram[SPRITE_X_VEL + k] = (-9i8) as u8;
    }

    pub(super) fn sprite_prep_running_man(&mut self, k: usize) {
        self.ram[SPRITE_HEAD_DIR + k] = 2;
        self.ram[SPRITE_D + k] = 2;
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_arrow_game_bounce(&mut self, k: usize) {
        const X: [u8; 8] = [0, 0x40, 0x80, 0xc0, 0x30, 0x60, 0x90, 0xc0];
        const Y: [u8; 8] = [0, 0x4f, 0x4f, 0x4f, 0x5a, 0x5a, 0x5a, 0x5a];
        const A: [u8; 8] = [0, 1, 1, 1, 2, 2, 2, 2];
        const XVEL: [i8; 2] = [-8, 12];
        const FLAGS4: [u8; 2] = [0x1c, 0x15];

        self.ram[ARCHERY_GAME_HIT_COUNTER] = 0;
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(9);
        for i in (1..=7).rev() {
            self.ram[SPRITE_TYPE + i] = 0x65;
            self.ram[SPRITE_STATE + i] = 9;
            self.sprite_prep_load_properties(i);
            self.ram[SPRITE_X_HI + i] = self.ram[LINK_X_COORD + 1];
            self.ram[SPRITE_X_LO + i] = X[i];
            self.ram[SPRITE_Y_HI + i] = self.ram[LINK_Y_COORD + 1];
            self.ram[SPRITE_Y_LO + i] = Y[i];
            self.ram[SPRITE_A + i] = A[i];
            let j = (A[i] - 1) as usize;
            self.ram[SPRITE_GRAPHICS + i] = j as u8;
            self.ram[SPRITE_X_VEL + i] = XVEL[j] as u8;
            self.ram[SPRITE_FLAGS4 + i] = FLAGS4[j];
            self.ram[SPRITE_OAM_FLAGS + i] = 13;
            self.ram[SPRITE_FLOOR + i] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            self.ram[SPRITE_SUBTYPE2 + i] = self.get_random_number();
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.ram[SPRITE_SUBTYPE + k] = self.ram[LINK_NUM_ARROWS];
    }

    pub(super) fn sprite_prep_mushroom(&mut self, k: usize) {
        if self.ram[LINK_ITEM_MUSHROOM] >= 2 {
            self.ram[SPRITE_STATE + k] = 0;
        } else {
            self.ram[SPRITE_GRAPHICS + k] = 0;
            self.ram[SPRITE_OAM_FLAGS + k] |= 8;
            self.ram[SPRITE_IGNORE_PROJECTILE + k] =
                self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        }
    }

    pub(super) fn sprite_prep_potion_shop(&mut self, k: usize) {
        self.magic_shop_assistant_spawn_powder(k);
        self.magic_shop_assistant_spawn_green_cauldron(k);
        self.magic_shop_assistant_spawn_blue_cauldron(k);
        self.magic_shop_assistant_spawn_red_cauldron(k);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn magic_shop_assistant_spawn_powder(&mut self, k: usize) {
        if self.ram[FLAG_OVERWORLD_AREA_DID_CHANGE_PREP] == 0 || self.ram[LINK_ITEM_MUSHROOM] == 2 {
            return;
        }
        if read_le_u16(&self.ram, SAVE_DUNG_INFO + 0x109 * 2) & 0x80 != 0 {
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
        self.ram[SPRITE_SUBTYPE2 + j] = subtype;
        self.sprite_set_x(j, info.r0_x.wrapping_add(x_off as u16));
        self.sprite_set_y(j, info.r2_y.wrapping_add(y_off as u16));
        self.ram[SPRITE_FLAGS4 + j] = 3;
        self.ram[SPRITE_DEFL_BITS + j] |= 0x20;
    }

    pub(super) fn sprite_prep_mini_moldorm_bounce(&mut self, k: usize) {
        let mut j = 32 * k;
        for _ in 0..32 {
            self.ram[MOLDORM_X_LO_PREP + j] = self.ram[SPRITE_X_LO + k];
            self.ram[MOLDORM_X_HI_PREP + j] = self.ram[SPRITE_X_HI + k];
            self.ram[MOLDORM_Y_LO_PREP + j] = self.ram[SPRITE_Y_LO + k];
            self.ram[MOLDORM_Y_HI_PREP + j] = self.ram[SPRITE_Y_HI + k];
            j += 1;
        }
    }

    pub(super) fn sprite_prep_bomber(&mut self, k: usize) {
        self.ram[SPRITE_Z + k] = 16;
        self.ram[SPRITE_SUBTYPE + k] = 254;
    }

    pub(super) fn sprite_prep_bomb_shoppe(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xb5, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_x(j, info.r0_x.wrapping_sub(24));
            self.sprite_set_y(j, info.r2_y.wrapping_sub(24));
            self.ram[SPRITE_SUBTYPE2 + j] = 1;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
        }

        if self.ram[LINK_HAS_CRYSTALS] & 5 == 5
            && self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 32 != 0
        {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xb5, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, info.r0_x.wrapping_sub(56));
                self.sprite_set_y(j, info.r2_y.wrapping_sub(24));
                self.ram[SPRITE_SUBTYPE2 + j] = 2;
                self.ram[SPRITE_IGNORE_PROJECTILE + j] = 2;
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
            self.ram[SPRITE_SUBTYPE2 + j] = 3;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 3;
            self.ram[SPRITE_Z + j] = 4;
            self.ram[SPRITE_Z_VEL + j] = (-12i8) as u8;
            self.ram[SPRITE_DELAY_MAIN + j] = 23;
            self.ram[SPRITE_FLAGS3 + j] &= !0x11u8;
        }
    }

    // void ArcheryGameGuy_ShowMsg(int k, int msg) {  // 8582bf
    //   dialogue_message_index = msg;
    //   Sprite_ShowMessageMinimal();
    //   sprite_delay_main[k] = 0;
    // }
    pub(super) fn archery_game_guy_show_msg(&mut self, k: usize, msg: i32) {
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, msg as u16);
        self.sprite_show_message_minimal_c();
        self.ram[SPRITE_DELAY_MAIN + k] = 0;
    }

    pub(super) fn sprite_65_archery_game(&mut self, k: usize) {
        self.ram[LINK_NUM_ARROWS] = self.ram[SPRITE_SUBTYPE + k];
        if self.ram[SPRITE_A + k] == 0 {
            self.archery_game_host(k);
        } else {
            self.sprite_good_or_bad_archery_target(k);
        }
    }

    pub(super) fn archery_game_host(&mut self, k: usize) {
        if self.ram[ARCHERY_GAME_ARROWS_LEFT] == 0 {
            self.ram[ARCHERY_GAME_OUT_OF_ARROWS] =
                self.ram[ARCHERY_GAME_OUT_OF_ARROWS].wrapping_add(1);
        }
        self.archery_game_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.ram[SPRITE_FLAGS4 + k] = 0;
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.ram[LINK_SPEED_SETTING] = 0;
            self.link_cancel_dash();
        }
        if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] & 7 == 0 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x11);
            }
            self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_DELAY_MAIN + k] & 4) >> 2;
        } else {
            const GFX: [u8; 4] = [3, 4, 3, 2];
            let idx = if self.ram[SPRITE_AI_STATE + k] != 0 {
                ((self.ram[FRAME_COUNTER] >> 5) & 3) as usize
            } else {
                0
            };
            self.ram[SPRITE_GRAPHICS + k] = GFX[idx];
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_FLAGS4 + k] = 10;
                if self.sprite_check_damage_to_link_same_layer(k)
                    && self.ram[FILTERED_JOYPAD_L] & 0x80 != 0
                {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.archery_game_guy_show_msg(k, 0x85);
                }
            }
            1 | 3 => {
                if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0
                    && read_le_u16(&self.ram, LINK_RUPEES_GOAL) >= 20
                {
                    self.ram[SPRITE_HEAD_DIR + k] = 0;
                    self.ram[ARCHERY_GAME_HIT_COUNTER] = 0;
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.archery_game_guy_show_msg(k, 0x86);
                } else {
                    self.ram[SPRITE_AI_STATE + k] = 0;
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

        if self.ram[SPRITE_HEAD_DIR + k] == 0 {
            self.ram[ARCHERY_GAME_ARROWS_LEFT] = 5;
            self.sprite_initialize_secondary_item_minigame(2);
            self.ram[SPRITE_DELAY_AUX1 + k] = 39;
            let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL);
            write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees.wrapping_sub(20));
            self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_HEAD_DIR + k].wrapping_add(1);
        }

        self.oam_allocate_from_region_a(0x34);
        let Some((info_x, info_y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let count = if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
            NUM_SPR[(self.ram[SPRITE_DELAY_AUX1 + k] >> 3) as usize]
        } else {
            self.ram[ARCHERY_GAME_ARROWS_LEFT]
        };
        let mut i = (count as i32) * 2 + 7;
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
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

        let ancillas_active = (0..=4).any(|i| self.ram[ANCILLA_TYPE + i] != 0);
        if self.ram[ARCHERY_GAME_ARROWS_LEFT]
            | self.ram[SPRITE_DELAY_AUX4 + k]
            | u8::from(ancillas_active)
            != 0
        {
            return;
        }
        self.ram[SPRITE_FLAGS4 + k] = 0x0a;
        if self.sprite_check_damage_to_link_same_layer(k) && self.ram[FILTERED_JOYPAD_L] & 0x80 != 0
        {
            self.archery_game_guy_show_msg(k, 0x88);
            self.ram[SPRITE_AI_STATE + k] = 3;
        }
    }

    pub(super) fn sprite_good_or_bad_archery_target(&mut self, k: usize) {
        const CASH_PRIZE: [u8; 10] = [4, 8, 16, 32, 64, 99, 99, 99, 99, 99];
        if self.ram[SPRITE_A + k] == 1 {
            if self.ram[SPRITE_G + k] >= 5 {
                self.ram[SPRITE_B + k] = 6;
            }
            self.ram[SPRITE_FLAGS2 + k] &= !0x1f;
            let j = if self.ram[SPRITE_DELAY_AUX2 + k] != 0 {
                self.ram[SPRITE_DELAY_AUX2 + k]
            } else {
                self.ram[SPRITE_SUBTYPE2 + k] >> 3
            };
            self.ram[SPRITE_OAM_FLAGS + k] =
                (self.ram[SPRITE_OAM_FLAGS + k] & !0x40) | ((j & 4) << 4);
            self.ram[CUR_SPRITE_Y] = self.ram[CUR_SPRITE_Y].wrapping_sub(3);
            self.sprite_draw_single_large(k);
            if self.ram[SPRITE_DELAY_AUX2 + k] != 0 {
                if self.ram[SPRITE_DELAY_AUX2 + k] == 96
                    && self.frame_control_view().submodule() == 0
                {
                    self.ram[SPRITE_DELAY_MAIN] = 112;
                    let prize = CASH_PRIZE[self.ram[SPRITE_B + k].wrapping_sub(1) as usize] as u16;
                    let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL).wrapping_add(prize);
                    write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees);
                }
                self.ram[SPRITE_FLAGS2 + k] |= 5;
                self.archery_game_draw_prize(k);
            }
        } else {
            self.ram[SPRITE_FLAGS2 + k] &= !0x1f;
            self.ram[CUR_SPRITE_Y] = self.ram[CUR_SPRITE_Y].wrapping_add(3);
            self.sprite_draw_single_large(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        if self.ram[SPRITE_DELAY_AUX3_PREP + k] == 1 {
            self.ram[SOUND_EFFECT_1] = 0x3c;
        }
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        self.sprite_move_x(k);
        if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
            self.ram[SPRITE_IGNORE_PROJECTILE + k] = self.ram[SPRITE_DELAY_MAIN + k];
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                if self.sprite_check_tile_collision(k) != 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 16;
                    self.ram[SPRITE_DELAY_AUX2 + k] = 0;
                }
            } else if self.ram[SPRITE_DELAY_MAIN + k] == 1 {
                const TARGET_X: [u8; 2] = [(-24i8) as u8, 8];
                self.ram[SPRITE_X_LO + k] = TARGET_X[self.ram[SPRITE_GRAPHICS + k] as usize];
                self.ram[SPRITE_X_HI + k] = self.ram[LINK_X_COORD + 1];
                self.ram[SPRITE_DELAY_AUX1 + k] = 32;
                self.ram[SPRITE_G + k] = 0;
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
        self.ram[oam] = x;
        self.ram[oam + 1] = y;
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        self.ram[BYTEWISE_EXTENDED_OAM + ((oam - OAM_BUF) / 4)] = big;
    }

    pub(super) fn sprite_prep_bully_and_victim(&mut self, k: usize) {
        self.spawn_bully(k);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn spawn_bully(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xb9, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.ram[SPRITE_SUBTYPE2 + j] = 2;
            self.ram[SPRITE_HEAD_DIR + j] = k as u8;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
        }
    }

    pub(super) fn ball_guy_play_bounce_noise(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
    }

    pub(super) fn garnish_alloc_force(&mut self) -> i32 {
        (0..30)
            .rev()
            .find(|&k| self.ram[GARNISH_TYPE + k] == 0)
            .unwrap_or(0) as i32
    }

    pub(super) fn garnish_alloc(&mut self) -> i32 {
        (0..30)
            .rev()
            .find(|&k| self.ram[GARNISH_TYPE + k] == 0)
            .map_or(-1, |k| k as i32)
    }

    pub(super) fn garnish_alloc_low(&mut self) -> i32 {
        (0..15)
            .rev()
            .find(|&k| self.ram[GARNISH_TYPE + k] == 0)
            .map_or(-1, |k| k as i32)
    }

    pub(super) fn garnish_alloc_limit(&mut self, k: usize) -> i32 {
        (0..=k)
            .rev()
            .find(|&k| self.ram[GARNISH_TYPE + k] == 0)
            .map_or(-1, |k| k as i32)
    }

    pub(super) fn garnish_alloc_overwrite_old_low(&mut self) -> i32 {
        if let Some(k) = (0..15).rev().find(|&k| self.ram[GARNISH_TYPE + k] == 0) {
            return k as i32;
        }
        self.ram[SPRITE_PREP_SHARED_COUNTER] = self.ram[SPRITE_PREP_SHARED_COUNTER].wrapping_sub(1);
        if sign8(self.ram[SPRITE_PREP_SHARED_COUNTER]) {
            self.ram[SPRITE_PREP_SHARED_COUNTER] = 14;
        }
        self.ram[SPRITE_PREP_SHARED_COUNTER] as i32
    }

    pub(super) fn garnish_alloc_overwrite_old(&mut self) -> i32 {
        if let Some(k) = (0..30).rev().find(|&k| self.ram[GARNISH_TYPE + k] == 0) {
            return k as i32;
        }
        self.ram[SPRITE_PREP_SHARED_COUNTER] = self.ram[SPRITE_PREP_SHARED_COUNTER].wrapping_sub(1);
        if sign8(self.ram[SPRITE_PREP_SHARED_COUNTER]) {
            self.ram[SPRITE_PREP_SHARED_COUNTER] = 29;
        }
        self.ram[SPRITE_PREP_SHARED_COUNTER] as i32
    }

    pub(super) fn garnish_set_x(&mut self, k: usize, x: u16) {
        self.ram[GARNISH_X_LO_PREP + k] = x as u8;
        self.ram[GARNISH_X_HI_PREP + k] = (x >> 8) as u8;
    }

    pub(super) fn garnish_set_y(&mut self, k: usize, y: u16) {
        self.ram[GARNISH_Y_LO_PREP + k] = y as u8;
        self.ram[GARNISH_Y_HI_PREP + k] = (y >> 8) as u8;
    }

    // void Sprite_SpawnSparkleGarnish(int k) {  // 858008
    pub(super) fn sprite_spawn_sparkle_garnish(&mut self, k: usize) {
        const COORD: [i8; 4] = [-4, 0, 4, 8];
        if (self.ram[FRAME_COUNTER] & 3) != 0 {
            return;
        }
        let j = self.garnish_alloc_force() as usize;
        self.ram[GARNISH_TYPE + j] = 0x12;
        self.ram[GARNISH_ACTIVE_PREP] = 0x12;
        let x = self
            .sprite_get_x(k)
            .wrapping_add(COORD[usize::from(self.get_random_number() & 3)] as i16 as u16);
        let y = self
            .sprite_get_y(k)
            .wrapping_add(COORD[usize::from(self.get_random_number() & 3)] as i16 as u16);
        self.garnish_set_x(j, x);
        self.garnish_set_y(j, y);
        self.ram[GARNISH_SPRITE_PREP + j] = k as u8;
        self.ram[GARNISH_COUNTDOWN_PREP + j] = 15;
    }

    // void Sprite_SpawnDummyDeathAnimation(int k) {  // 89ae7e
    pub(super) fn sprite_spawn_dummy_death_animation(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x0b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.ram[SPRITE_STATE + j] = 6;
            self.ram[SPRITE_DELAY_MAIN + j] = 15;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x14);
            self.ram[SPRITE_FLOOR + j] = 2;
        }
    }

    // void Sprite_MagicBat_SpawnLightning(int k) {  // 89aea8
    pub(super) fn sprite_magic_bat_spawn_lightning(&mut self, k: usize) {
        const XVEL: [i8; 4] = [-8, -4, 4, 8];
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
                        .wrapping_sub(u16::from(self.ram[SPRITE_Z + k])),
                );
                self.ram[SPRITE_Z + j] = 0;
                self.ram[SPRITE_Y_VEL + j] = 24;
                self.ram[SPRITE_HEAD_DIR + j] = 24;
                self.ram[SPRITE_IGNORE_PROJECTILE + j] = 24;
                self.ram[SPRITE_FLAGS2 + j] = 0x80;
                self.ram[SPRITE_FLAGS3 + j] = 3;
                self.ram[SPRITE_OAM_FLAGS + j] = 3;
                self.ram[SPRITE_DELAY_MAIN + j] = 32;
                self.ram[SPRITE_GRAPHICS + j] = 2;
                let i = usize::from(self.ram[SPRITE_G + k]);
                self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
                self.ram[SPRITE_SUBTYPE2 + j] = ST2[i];
                self.ram[SPRITE_FLOOR + j] = 2;
                self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
            }
        }
    }

    pub(super) fn garnish_spawn_pyramid_debris(&mut self, x: i8, y: i8, xvel: i8, yvel: i8) {
        let k = self.garnish_alloc_force() as usize;
        self.ram[SOUND_EFFECT_2] = 3;
        self.ram[SOUND_EFFECT_1] = 31;
        self.ram[SOUND_EFFECT_AMBIENT] = 5;
        self.ram[GARNISH_TYPE + k] = 19;
        self.ram[GARNISH_ACTIVE_PREP] = 19;
        self.ram[GARNISH_X_LO_PREP + k] = 232u8.wrapping_add_signed(x);
        self.ram[GARNISH_Y_LO_PREP + k] = 96u8.wrapping_add_signed(y);
        self.ram[GARNISH_X_VEL_PREP + k] = xvel as u8;
        self.ram[GARNISH_Y_VEL_PREP + k] = yvel as u8;
        self.ram[GARNISH_COUNTDOWN_PREP + k] = (self.get_random_number() & 31).wrapping_add(48);
    }

    pub(super) fn kholdstare_spawn_puff_cloud_garnish(&mut self, k: usize) {
        const XY: [i8; 8] = [-8, -6, -4, -2, 0, 2, 4, 6];
        if (k as u8 ^ self.ram[FRAME_COUNTER]) & 3 != 0 {
            return;
        }
        let j = self.garnish_alloc_low();
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.ram[GARNISH_TYPE + j] = 7;
        self.ram[GARNISH_ACTIVE_PREP] = 7;
        self.ram[GARNISH_COUNTDOWN_PREP + j] = 31;
        let x = read_le_u16(&self.ram, CUR_SPRITE_X)
            .wrapping_add_signed(i16::from(XY[(self.get_random_number() & 7) as usize]));
        let y = read_le_u16(&self.ram, CUR_SPRITE_Y)
            .wrapping_add_signed(i16::from(XY[(self.get_random_number() & 7) as usize]) + 16);
        self.garnish_set_x(j, x);
        self.garnish_set_y(j, y);
        self.ram[GARNISH_FLOOR_PREP + j] = 0;
    }

    pub(super) fn garnish_flame_trail(&mut self, k: usize, is_low: bool) -> i32 {
        let j = if is_low {
            self.garnish_alloc_overwrite_old_low()
        } else {
            self.garnish_alloc_overwrite_old()
        };
        let j_usize = j as usize;
        self.ram[GARNISH_TYPE + j_usize] = 0x10;
        self.ram[GARNISH_ACTIVE_PREP] = 0x10;
        self.ram[GARNISH_SPRITE_PREP + j_usize] = k as u8;
        self.garnish_set_x(j_usize, self.sprite_get_x(k));
        self.garnish_set_y(j_usize, self.sprite_get_y(k).wrapping_add(16));
        self.ram[GARNISH_COUNTDOWN_PREP + j_usize] = 127;
        j
    }

    pub(super) fn fire_bat_animate(&mut self, k: usize) {
        const GFX: [u8; 4] = [4, 5, 6, 5];

        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        let i = ((self.ram[SPRITE_SUBTYPE2 + k] >> 2) & 3) as usize;
        self.ram[SPRITE_GRAPHICS + k] = GFX[i];
    }

    pub(super) fn fire_bat_move(&mut self, k: usize) {
        self.fire_bat_animate(k);
        self.sprite_move_xy(k);

        if self.ram[SPRITE_SUBTYPE2 + k] & 7 != 0 {
            return;
        }

        let j = self.garnish_flame_trail(k, true) as usize;
        self.ram[GARNISH_COUNTDOWN_PREP + j] = if self.ram[SPRITE_ANIM_CLOCK + k] == 5 {
            0x2f
        } else {
            0x4f
        };
    }

    pub(super) fn fireball_spawn_trail_garnish(&mut self, k: usize) {
        if (k as u8 ^ self.ram[FRAME_COUNTER]) & 3 != 0 {
            return;
        }
        let j = self.garnish_alloc() as usize;
        self.ram[GARNISH_TYPE + j] = 8;
        self.ram[GARNISH_ACTIVE_PREP] = 8;
        self.ram[GARNISH_COUNTDOWN_PREP + j] = 11;
        let x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let y = read_le_u16(&self.ram, CUR_SPRITE_Y).wrapping_add(16);
        self.garnish_set_x(j, x);
        self.garnish_set_y(j, y);
        self.ram[GARNISH_SPRITE_PREP + j] = k as u8;
    }

    pub(super) fn firesnake_spawn_fireball(&mut self, j: usize) {
        if ((j as u8) ^ self.ram[FRAME_COUNTER]) & 7 != 0 {
            return;
        }

        let k = self.garnish_alloc();
        if k < 0 {
            return;
        }

        let k = k as usize;
        self.ram[GARNISH_TYPE + k] = 1;
        self.ram[GARNISH_ACTIVE_PREP] = 1;
        self.ram[GARNISH_X_LO_PREP + k] = self.ram[SPRITE_X_LO + j];
        self.ram[GARNISH_X_HI_PREP + k] = self.ram[SPRITE_X_HI + j];
        self.garnish_set_y(k, self.sprite_get_y(j).wrapping_add(16));
        self.ram[GARNISH_COUNTDOWN_PREP + k] = 32;
        self.ram[GARNISH_SPRITE_PREP + k] = j as u8;
        self.ram[GARNISH_FLOOR_PREP + k] = self.ram[SPRITE_FLOOR + j];
    }

    pub(super) fn catfish_spawn_plop(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xec, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.ram[SPRITE_STATE + j] = 3;
            self.ram[SPRITE_DELAY_MAIN + j] = 15;
            self.ram[SPRITE_AI_STATE + j] = 0;
            self.ram[SPRITE_FLAGS2 + j] = 3;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
        }
    }

    pub(super) fn catfish_regurgitate_medallion(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc0, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.ram[SPRITE_X_VEL + j] = 24;
            self.ram[SPRITE_Z_VEL + j] = 48;
            self.ram[SPRITE_A + j] = 17;
            self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
            self.ram[SPRITE_FLAGS2 + j] = 0x83;
            self.ram[SPRITE_FLAGS3 + j] = 0x58;
            self.ram[SPRITE_OAM_FLAGS + j] = 0x58 & 0x0f;
            self.DecodeAnimatedSpriteTile_variable(0x1c);
        }
    }

    pub(super) fn sprite_spawn_water_splash(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc0, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.ram[SPRITE_A + j_usize] = 0x80;
            self.ram[SPRITE_FLAGS2 + j_usize] = 2;
            self.ram[SPRITE_IGNORE_PROJECTILE + j_usize] = 2;
            self.ram[SPRITE_OAM_FLAGS + j_usize] = 4;
            self.ram[SPRITE_DELAY_MAIN + j_usize] = 31;
        }
        j
    }

    pub(super) fn sprite_spawn_small_splash(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0xec, &mut info, 14);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.ram[SOUND_EFFECT_1] = 0;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
            self.ram[SPRITE_STATE + j_usize] = 3;
            self.ram[SPRITE_DELAY_MAIN + j_usize] = 15;
            self.ram[SPRITE_AI_STATE + j_usize] = 0;
            self.ram[SPRITE_FLAGS2 + j_usize] = 3;
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
            self.ram[SPRITE_SUBTYPE2 + j_usize] = 1;
        }
        j
    }

    pub(super) fn sprite_spawn_superficial_bomb_blast(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.ram[SPRITE_STATE + j_usize] = 6;
            self.ram[SPRITE_DELAY_AUX1 + j_usize] = 31;
            self.ram[SPRITE_C + j_usize] = 3;
            self.ram[SPRITE_FLAGS2 + j_usize] = 3;
            self.ram[SPRITE_OAM_FLAGS + j_usize] = 4;
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
            self.ram[SPRITE_TYPE + j_usize] = 0x4a;
            self.ram[SPRITE_C + j_usize] = 1;
            self.ram[SPRITE_DELAY_AUX1 + j_usize] = 255;
            self.ram[SPRITE_FLAGS3 + j_usize] = 0x18;
            self.ram[SPRITE_OAM_FLAGS + j_usize] = 8;
            self.ram[SPRITE_HEALTH + j_usize] = 0;
            self.ram[SPRITE_DELAY_AUX1 + j_usize] = 80;
            self.ram[SPRITE_X_VEL + j_usize] = 24;
            self.ram[SPRITE_Z_VEL + j_usize] = 48;
        }
        j
    }

    pub(super) fn spawn_boss_poof(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xce, &mut info);
        let j_usize = j as usize;
        self.sprite_set_x(j_usize, info.r0_x.wrapping_add(16));
        self.sprite_set_y(j_usize, info.r2_y.wrapping_add(40));
        self.ram[SPRITE_GRAPHICS + j_usize] = 0x0f;
        self.ram[SPRITE_A + j_usize] = 1;
        self.ram[SPRITE_DELAY_MAIN + j_usize] = 47;
        self.ram[SPRITE_FLAGS2 + j_usize] = 9;
        self.ram[SPRITE_IGNORE_PROJECTILE + j_usize] = 9;
        self.ram[SOUND_EFFECT_1] = 12;
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
        self.ram[SPRITE_FLAGS3 + j_usize] = (self.ram[SPRITE_FLAGS3 + j_usize] & 0xfe) | 0x40;
        self.ram[SPRITE_OAM_FLAGS + j_usize] = 6;
        self.ram[SPRITE_FLAGS4 + j_usize] = 0x54;
        self.ram[SPRITE_E + j_usize] = 0x54;
        self.ram[SPRITE_FLAGS2 + j_usize] = 0x20;
        self.sprite_apply_speed_towards_link(j_usize, 0x20);
        self.ram[SPRITE_DELAY_MAIN + j_usize] = 20;
        self.ram[SPRITE_DELAY_AUX1 + j_usize] = 16;
        self.ram[SPRITE_FLAGS5 + j_usize] = 0;
        self.ram[SPRITE_DEFL_BITS + j_usize] = 0x48;
        j
    }

    pub(super) fn sprite_spawn_fire_phlegm(&mut self, k: usize) -> i32 {
        const X: [i8; 4] = [16, -8, 4, 4];
        const Y: [i8; 4] = [-2, -2, 8, -20];
        const XVEL: [i8; 4] = [48, -48, 0, 0];
        const YVEL: [i8; 4] = [0, 0, 48, -48];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xa5, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_sfx_queue_sfx3_with_pan(k, 5);
            self.sprite_set_spawned_coordinates(j_usize, &info);
            let i = self.ram[SPRITE_D + k] as usize;
            self.sprite_set_x(j_usize, info.r0_x.wrapping_add_signed(i16::from(X[i])));
            self.sprite_set_y(j_usize, info.r2_y.wrapping_add_signed(i16::from(Y[i])));
            self.ram[SPRITE_X_VEL + j_usize] = XVEL[i] as u8;
            self.ram[SPRITE_Y_VEL + j_usize] = YVEL[i] as u8;
            self.ram[SPRITE_FLAGS3 + j_usize] |= 0x40;
            self.ram[SPRITE_DEFL_BITS + j_usize] = 0x40;
            self.ram[SPRITE_FLAGS2 + j_usize] = 0x21;
            self.ram[SPRITE_B + j_usize] = 0x21;
            self.ram[SPRITE_OAM_FLAGS + j_usize] = 2;
            self.ram[SPRITE_FLAGS4 + j_usize] = 0x14;
            self.ram[SPRITE_IGNORE_PROJECTILE + j_usize] = 20;
            self.ram[SPRITE_BUMP_DAMAGE + j_usize] = 37;
            if self.ram[LINK_SHIELD_TYPE] >= 3 {
                self.ram[SPRITE_FLAGS5 + j_usize] = 0x20;
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
        self.ram[SPRITE_GRAPHICS + j] = 2;
        self.ram[SPRITE_Z_VEL + j] = self.ram[SPRITE_Z_VEL + k];
        self.ram[SPRITE_SUBTYPE2 + j] = 1;
        self.ram[SPRITE_AI_STATE + j] = 2;
        self.ram[SPRITE_DELAY_MAIN + j] = 8;
        self.sprite_set_spawned_coordinates(j, &info);
        j as i32
    }

    pub(super) fn sprite_spawn_poof_garnish(&mut self, j: usize) {
        let k = self.garnish_alloc_force() as usize;
        self.ram[GARNISH_TYPE + k] = 10;
        self.ram[GARNISH_ACTIVE_PREP] = 10;
        self.ram[GARNISH_X_LO_PREP + k] = self.ram[SPRITE_X_LO + j];
        self.ram[GARNISH_X_HI_PREP + k] = self.ram[SPRITE_X_HI + j];
        let y = self.sprite_get_y(j).wrapping_add(16);
        self.ram[GARNISH_Y_LO_PREP + k] = y as u8;
        self.ram[GARNISH_Y_HI_PREP + k] = (y >> 8) as u8;
        self.ram[GARNISH_SPRITE_PREP + k] = self.ram[SPRITE_FLOOR + j];
        self.ram[GARNISH_COUNTDOWN_PREP + k] = 15;
    }

    pub(super) fn octorok_fire_loogie(&mut self, k: usize) {
        const X: [i8; 4] = [12, -12, 0, 0];
        const Y: [i8; 4] = [4, 4, 12, -12];
        const XVEL: [i8; 4] = [44, -44, 0, 0];
        const YVEL: [i8; 4] = [0, 0, 44, -44];

        let mut info = SpriteSpawnInfo::default();
        self.sprite_sfx_queue_sfx2_with_pan(k, 7);
        let j = self.sprite_spawn_dynamically(k, 0x0c, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.ram[SPRITE_D + k] as usize;
            self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(X[i])));
            self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(Y[i])));
            self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
            self.ram[SPRITE_Y_VEL + j] = YVEL[i] as u8;
        }
    }

    pub(super) fn moblin_materialize_spear(&mut self, k: usize) {
        const X: [i8; 4] = [11, -2, -3, 11];
        const Y: [i8; 4] = [-3, -3, 3, -11];
        const XVEL: [i8; 4] = [32, -32, 0, 0];
        const YVEL: [i8; 4] = [0, 0, 32, -32];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x1b, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.ram[SPRITE_D + k] as usize;
            self.ram[SPRITE_A + j] = 3;
            self.ram[SPRITE_D + j] = i as u8;
            self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(X[i])));
            self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(Y[i])));
            self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
            self.ram[SPRITE_Y_VEL + j] = YVEL[i] as u8;
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
        let i = match self.ram[SPRITE_TYPE + k] {
            0x3d => 0,
            0x35 => 1,
            _ => 2,
        };
        let x_base = read_le_u16(&self.ram, SPRCOLL_X_BASE_PREP) & 0xff00;
        let y_base = read_le_u16(&self.ram, SPRCOLL_Y_BASE_PREP) & 0xff00;
        self.sprite_set_x(j, X[i].wrapping_add(x_base));
        self.sprite_set_y(j, Y[i].wrapping_add(y_base));
        self.ram[SPRITE_FLOOR + j] = 0;
        self.ram[SPRITE_HEALTH + j] = 4;
        self.ram[SPRITE_DEFL_BITS + j] = 0x80;
        self.ram[SPRITE_FLAGS5 + j] = 0x90;
        self.ram[SPRITE_OAM_FLAGS + j] = 0x0b;
    }

    pub(super) fn ancilla_terminate_sparkle_objects(&mut self) {
        for i in (0..=4).rev() {
            let t = self.ram[ANCILLA_TYPE + i];
            if matches!(t, 0x2a | 0x2b | 0x30 | 0x31 | 0x18 | 0x19 | 0x0c) {
                self.ram[ANCILLA_TYPE + i] = 0;
            }
        }
    }

    pub(super) fn kodongo_set_direction(&mut self, k: usize) {
        const XVEL: [i8; 4] = [16, -16, 0, 0];
        const YVEL: [i8; 4] = [0, 0, 16, -16];

        let j = self.ram[SPRITE_D + k] as usize;
        self.ram[SPRITE_X_VEL + k] = XVEL[j] as u8;
        self.ram[SPRITE_Y_VEL + k] = YVEL[j] as u8;
    }

    pub(super) fn kodongo_spawn_fire(&mut self, k: usize) {
        const X: [i8; 4] = [8, -8, 0, 0];
        const Y: [i8; 4] = [0, 0, 8, -8];
        const XVEL: [i8; 4] = [24, -24, 0, 0];
        const YVEL: [i8; 4] = [0, 0, 24, -24];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0x87, &mut info, 13);
        if j >= 0 {
            let j = j as usize;
            let i = self.ram[SPRITE_D + k] as usize;
            self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(X[i])));
            self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(Y[i])));
            self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
            self.ram[SPRITE_Y_VEL + j] = YVEL[i] as u8;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
        }
    }

    pub(super) fn create_six_blue_balls(&mut self, k: usize) {
        const XVEL: [i8; 6] = [0, 24, 24, 0, -24, -24];
        const YVEL: [i8; 6] = [-32, -16, 16, 32, 16, -16];

        self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
        self.ram[TMP_COUNTER] = 5;
        loop {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x55, &mut info);
            if j >= 0 {
                let j = j as usize;
                let i = self.ram[TMP_COUNTER] as usize;
                self.sprite_set_x(j, info.r0_x.wrapping_add(4));
                self.sprite_set_y(j, info.r2_y.wrapping_add(4));
                self.ram[SPRITE_FLAGS3 + j] = (self.ram[SPRITE_FLAGS3 + j] & !1) | 0x40;
                self.ram[SPRITE_OAM_FLAGS + j] = 4;
                self.ram[SPRITE_DELAY_AUX1 + j] = 4;
                self.ram[SPRITE_FLAGS4 + j] = 20;
                self.ram[SPRITE_C + j] = 20;
                self.ram[SPRITE_E + j] = 20;
                self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
                self.ram[SPRITE_Y_VEL + j] = YVEL[i] as u8;
            }

            self.ram[TMP_COUNTER] = self.ram[TMP_COUNTER].wrapping_sub(1);
            if sign8(self.ram[TMP_COUNTER]) {
                break;
            }
        }
        self.ram[TMP_COUNTER] = 0;
    }

    pub(super) fn lanmola_spawn_shrapnel(&mut self, k: usize) {
        const YVEL: [i8; 8] = [28, -28, 28, -28, 0, 36, 0, -36];
        const XVEL: [i8; 8] = [-28, -28, 28, 28, -36, 0, 36, 0];

        self.ram[TMP_COUNTER] = if self.ram[SPRITE_STATE + 0]
            .wrapping_add(self.ram[SPRITE_STATE + 1])
            .wrapping_add(self.ram[SPRITE_STATE + 2])
            < 10
        {
            7
        } else {
            3
        };

        loop {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xc2, &mut info);
            if j >= 0 {
                let j = j as usize;
                let i = self.ram[TMP_COUNTER] as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.ram[SPRITE_X_LO + j] = (info.r0_x as u8).wrapping_add(4);
                self.ram[SPRITE_Y_LO + j] = (info.r2_y as u8).wrapping_add(4);
                self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
                self.ram[SPRITE_BUMP_DAMAGE + j] = 1;
                self.ram[SPRITE_FLAGS4 + j] = 1;
                self.ram[SPRITE_Z + j] = 0;
                self.ram[SPRITE_FLAGS2 + j] = 0x20;
                self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
                self.ram[SPRITE_Y_VEL + j] = YVEL[i] as u8;
                self.ram[SPRITE_GRAPHICS + j] = self.get_random_number() & 1;
            }

            self.ram[TMP_COUNTER] = self.ram[TMP_COUNTER].wrapping_sub(1);
            if sign8(self.ram[TMP_COUNTER]) {
                break;
            }
        }
    }

    pub(super) fn octoballoon_form_babby(&mut self, k: usize) {
        const XVEL: [i8; 6] = [16, 11, -11, -16, -11, 11];
        const YVEL: [i8; 6] = [0, 11, 11, 0, -11, -11];

        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
        for i in (0..=5).rev() {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x10, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
                self.ram[SPRITE_Y_VEL + j] = YVEL[i] as u8;
                self.ram[SPRITE_Z_VEL + j] = 48;
                self.ram[SPRITE_SUBTYPE2 + j] = 255;
            }
        }
    }

    pub(super) fn pink_ball_handle_message(&mut self, k: usize) {
        if self.ram[SPRITE_DELAY_AUX4 + k] != 0 {
            return;
        }
        let msg = if self.ram[LINK_ITEM_MOON_PEARL] & 1 != 0 {
            0x15c
        } else {
            0x15b
        };
        if self.sprite_show_message_on_contact(k, msg) & 0x100 != 0 {
            self.ram[SPRITE_X_VEL + k] ^= 255;
            self.ram[SPRITE_Y_VEL + k] ^= 255;
            if self.ram[SPRITE_E + k] != 0 {
                self.ball_guy_play_bounce_noise(k);
            }
            self.ram[SPRITE_DELAY_AUX4 + k] = 64;
        }
    }

    pub(super) fn bully_handle_message(&mut self, k: usize) {
        if self.ram[SPRITE_DELAY_AUX4 + k] != 0 {
            return;
        }
        let msg = if self.ram[LINK_ITEM_MOON_PEARL] & 1 != 0 {
            0x15e
        } else {
            0x15d
        };
        if self.sprite_show_message_on_contact(k, msg) & 0x100 != 0 {
            self.ram[SPRITE_X_VEL + k] ^= 255;
            self.ram[SPRITE_Y_VEL + k] ^= 255;
            self.ram[SPRITE_DELAY_AUX4 + k] = 64;
        }
    }

    pub(super) fn rupee_pull_spawn_prize(&mut self, k: usize) {
        const XVEL: [i8; 4] = [-18, -12, 12, 18];
        const YVEL: [i8; 4] = [16, 24, 24, 16];
        const TYPE: [u8; 3] = [0xd9, 0xda, 0xdb];

        if self.ram[NUM_SPRITES_KILLED_PREP] != 0 {
            self.ram[SPRITE_SHARED_SCRATCH_A] = if self.ram[NUM_SPRITES_KILLED_PREP] < 4 {
                0
            } else if self.ram[NUMBER_OF_TIMES_HURT_BY_SPRITES] != 0 {
                1
            } else {
                2
            };
            self.ram[TMP_COUNTER] = 3;
            loop {
                let mut info = SpriteSpawnInfo::default();
                let what = TYPE[self.ram[SPRITE_SHARED_SCRATCH_A] as usize];
                let j = self.sprite_spawn_dynamically(k, what, &mut info);
                if j < 0 {
                    break;
                }

                let j = j as usize;
                let i = self.ram[TMP_COUNTER] as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
                self.ram[SPRITE_Y_VEL + j] = YVEL[i] as u8;
                self.ram[SPRITE_STUNNED + j] = 255;
                self.ram[SPRITE_DELAY_AUX4 + j] = 32;
                self.ram[SPRITE_DELAY_AUX3_PREP + j] = 32;
                self.ram[SPRITE_Z_VEL + j] = 32;

                self.ram[TMP_COUNTER] = self.ram[TMP_COUNTER].wrapping_sub(1);
                if sign8(self.ram[TMP_COUNTER]) {
                    break;
                }
            }
        }
        self.ram[NUM_SPRITES_KILLED_PREP] = 0;
        self.ram[NUMBER_OF_TIMES_HURT_BY_SPRITES] = 0;
    }

    pub(super) fn sluggula_drop_bomb(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0x4a, &mut info, 11);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.ram[SPRITE_TYPE + j_usize] = 0x4a;
            self.ram[SPRITE_C + j_usize] = 1;
            self.ram[SPRITE_DELAY_AUX1 + j_usize] = 255;
            self.ram[SPRITE_FLAGS3 + j_usize] = 0x18;
            self.ram[SPRITE_OAM_FLAGS + j_usize] = 8;
            self.ram[SPRITE_HEALTH + j_usize] = 0;
        }
    }

    pub(super) fn talking_tree_spawn_bomb(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.ram[SPRITE_TYPE + j_usize] = 0x4a;
            self.ram[SPRITE_C + j_usize] = 1;
            self.ram[SPRITE_DELAY_AUX1 + j_usize] = 255;
            self.ram[SPRITE_FLAGS3 + j_usize] = 0x18;
            self.ram[SPRITE_OAM_FLAGS + j_usize] = 8;
            self.ram[SPRITE_HEALTH + j_usize] = 0;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.ram[SPRITE_DELAY_AUX1 + j_usize] = 64;
            self.ram[SPRITE_Y_VEL + j_usize] = 24;
            self.ram[SPRITE_Z_VEL + j_usize] = 18;
        }
    }

    pub(super) fn pirogusu_spawn_splash(&mut self, k: usize) {
        const TAB0: [u8; 4] = [3, 4, 5, 4];
        if (k as u8 ^ self.ram[FRAME_COUNTER]) & 3 != 0 {
            return;
        }
        let x = TAB0[(self.get_random_number() & 3) as usize];
        let y = TAB0[(self.get_random_number() & 3) as usize];
        let j = self.garnish_alloc_low();
        if j >= 0 {
            let j_usize = j as usize;
            self.ram[GARNISH_TYPE + j_usize] = 11;
            self.ram[GARNISH_ACTIVE_PREP] = 11;
            self.garnish_set_x(j_usize, self.sprite_get_x(k).wrapping_add(u16::from(x)));
            self.garnish_set_y(
                j_usize,
                self.sprite_get_y(k)
                    .wrapping_add(u16::from(y))
                    .wrapping_add(16),
            );
            self.ram[GARNISH_COUNTDOWN_PREP + j_usize] = 15;
        }
    }

    pub(super) fn lightning_spawn_garnish(&mut self, k: usize) {
        let j = self.garnish_alloc_overwrite_old() as usize;
        self.ram[GARNISH_TYPE + j] = 9;
        self.ram[GARNISH_ACTIVE_PREP] = 9;
        self.ram[GARNISH_SPRITE_PREP + j] = self.ram[SPRITE_A + k];
        self.ram[GARNISH_X_LO_PREP + j] = self.ram[SPRITE_X_LO + k];
        self.ram[GARNISH_X_HI_PREP + j] = self.ram[SPRITE_X_HI + k];
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(16));
        self.ram[GARNISH_COUNTDOWN_PREP + j] = 32;
    }

    pub(super) fn laser_beam_build_up_garnish(&mut self, k: usize) {
        let j = self.garnish_alloc_overwrite_old() as usize;
        self.ram[GARNISH_TYPE + j] = 4;
        self.ram[GARNISH_ACTIVE_PREP] = 4;
        self.garnish_set_x(j, self.sprite_get_x(k));
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(16));
        self.ram[GARNISH_COUNTDOWN_PREP + j] = 16;
        self.ram[GARNISH_OAM_FLAGS_PREP + j] = self.ram[SPRITE_GRAPHICS + k];
        self.ram[GARNISH_SPRITE_PREP + j] = k as u8;
        self.ram[GARNISH_FLOOR_PREP + j] = self.ram[SPRITE_FLOOR + k];
    }

    pub(super) fn laser_eye_fire_beam(&mut self, k: usize) {
        const SPAWN_XY: [i8; 6] = [12, -4, 4, 4, 12, -4];
        const SPAWN_XYVEL: [i8; 6] = [112, -112, 0, 0, 112, -112];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x95, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = self.ram[SPRITE_D + k] as usize;
            self.ram[SPRITE_GRAPHICS + j] = (i as u8 & 2) >> 1;
            self.sprite_set_x(j, info.r0_x.wrapping_add_signed(i16::from(SPAWN_XY[i])));
            self.sprite_set_y(j, info.r2_y.wrapping_add_signed(i16::from(SPAWN_XY[i + 2])));
            self.ram[SPRITE_X_VEL + j] = SPAWN_XYVEL[i] as u8;
            self.ram[SPRITE_Y_VEL + j] = SPAWN_XYVEL[i + 2] as u8;
            self.ram[SPRITE_FLAGS2 + j] = 0x20;
            self.ram[SPRITE_A + j] = 0x20;
            self.ram[SPRITE_OAM_FLAGS + j] = 5;
            self.ram[SPRITE_DEFL_BITS + j] = 0x48;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 0x48;
            self.ram[SPRITE_DELAY_MAIN + j] = 5;
            if self.ram[LINK_SHIELD_TYPE] == 3 {
                self.ram[SPRITE_FLAGS5 + j] = 32;
            }
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
        }
    }

    pub(super) fn get_position_relative_to_the_great_overlord_ganon(&mut self, k: usize) {
        const X: [i8; 2] = [20, -18];
        const Y: [i8; 2] = [-20, -20];

        let j = self.ram[SPRITE_D + 0] as usize;
        let x = u16::from(self.ram[OVERLORD_X_HI_PREP + k])
            | (u16::from(self.ram[OVERLORD_Y_HI_PREP + k]) << 8);
        let y = u16::from(self.ram[OVERLORD_GEN2_PREP + k])
            | (u16::from(self.ram[OVERLORD_FLOOR_PREP + k]) << 8);
        self.sprite_set_x(k, x.wrapping_add_signed(i16::from(X[j])));
        self.sprite_set_y(k, y.wrapping_add_signed(i16::from(Y[j])));
    }

    pub(super) fn sasha_idle(&mut self, k: usize) {
        if self.ram[LINK_WHICH_PENDANTS] & 4 == 0 {
            if self.sprite_show_solicited_message(k, 0x32) & 0x100 != 0 {
                self.ram[SPRITE_AI_STATE + k] = 1;
            }
        } else if self.ram[LINK_ITEM_BOOTS] == 0 {
            let m = if self.ram[SAVEGAME_MAP_ICONS_INDICATOR] >= 3 {
                0x38
            } else {
                0x39
            };
            if self.sprite_show_solicited_message(k, m) & 0x100 != 0 {
                self.ram[SPRITE_AI_STATE + k] = 2;
            }
        } else if self.ram[LINK_ITEM_ICE_ROD] == 0 {
            self.sprite_show_solicited_message(k, 0x37);
        } else if self.ram[LINK_WHICH_PENDANTS] & 7 != 7 {
            self.sprite_show_solicited_message(k, 0x34);
        } else if self.ram[LINK_SWORD_TYPE] < 2 {
            self.sprite_show_solicited_message(k, 0x30);
        } else {
            self.sprite_show_solicited_message(k, 0x31);
        }
        self.ram[SPRITE_GRAPHICS + k] = self.ram[FRAME_COUNTER] >> 5 & 1;
    }

    pub(super) fn old_man_revert_to_sprite(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xad, &mut info);
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.ram[SPRITE_D + j] = self.ram[TAGALONG_LAYERBITS + k] & 3;
        self.ram[SPRITE_HEAD_DIR + j] = self.ram[TAGALONG_LAYERBITS + k] & 3;
        let y =
            u16::from(self.ram[TAGALONG_Y_LO + k]) | (u16::from(self.ram[TAGALONG_Y_HI + k]) << 8);
        let x =
            u16::from(self.ram[TAGALONG_X_LO + k]) | (u16::from(self.ram[TAGALONG_X_HI + k]) << 8);
        self.sprite_set_y(j, y.wrapping_add(2));
        self.sprite_set_x(j, x.wrapping_add(2));
        self.ram[SPRITE_FLOOR + j] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
        self.ram[SPRITE_SUBTYPE2 + j] = 1;
        self.old_man_enable_cutscene();
        self.ram[FOLLOWER_INDICATOR] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
    }

    pub(super) fn old_man_enable_cutscene(&mut self) {
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
    }

    pub(super) fn sprite_ad_old_man(&mut self, k: usize) {
        const OLD_MOUNTAIN_MAN_MSGS: [u16; 3] = [0x9e, 0x9f, 0xa0];

        self.old_mountain_man_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.ram[SPRITE_SUBTYPE2 + k] {
            0 => match self.ram[SPRITE_AI_STATE + k] {
                0 => {
                    self.sprite_track_body_to_head(k);
                    let dir = self.sprite_direction_to_face_link(k, None) ^ 3;
                    self.ram[SPRITE_HEAD_DIR + k] = dir;
                    let j = self.sprite_show_message_on_contact(k, 0x9c);
                    if j & 0x100 != 0 {
                        self.ram[SPRITE_D + k] = j as u8;
                        self.ram[SPRITE_HEAD_DIR + k] = j as u8;
                        self.ram[SPRITE_AI_STATE + k] = 1;
                    }
                }
                1 => {
                    self.ram[FOLLOWER_INDICATOR] = 4;
                    self.sprite_become_follower(k);
                    self.ram[WHICH_STARTING_POINT] = 5;
                    self.ram[SPRITE_STATE + k] = 0;
                    self.cache_camera_properties();
                }
                _ => {}
            },
            1 => {
                self.sprite_move_xy(k);
                match self.ram[SPRITE_AI_STATE + k] {
                    0 => {
                        self.ram[SPRITE_AI_STATE + k] =
                            self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        self.ram[ITEM_RECEIPT_METHOD] = 0;
                        self.link_receive_item(0x1a, 0);
                        self.ram[WHICH_STARTING_POINT] = 1;
                        self.old_man_enable_cutscene();
                        self.ram[SPRITE_DELAY_MAIN + k] = 48;
                        self.ram[SPRITE_X_VEL + k] = 8;
                        self.ram[SPRITE_Y_VEL + k] = 4;
                        self.ram[SPRITE_D + k] = 3;
                        self.ram[SPRITE_HEAD_DIR + k] = 3;
                    }
                    1 => {
                        self.old_man_enable_cutscene();
                        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                            self.ram[SPRITE_AI_STATE + k] =
                                self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        }
                        self.ram[SPRITE_GRAPHICS + k] =
                            ((k as u8) ^ self.ram[FRAME_COUNTER]) >> 3 & 1;
                    }
                    2 => {
                        self.ram[SPRITE_HEAD_DIR + k] = 0;
                        self.ram[SPRITE_D + k] = 0;
                        let j = self.ram[ACTIVE_OVERLORD_INDEX_PREP] as usize;
                        let x = u16::from(self.ram[OVERLORD_X_LO_PREP + j])
                            | (u16::from(self.ram[OVERLORD_X_HI_PREP + j]) << 8);
                        let y = u16::from(self.ram[OVERLORD_Y_LO_PREP + j])
                            | (u16::from(self.ram[OVERLORD_Y_HI_PREP + j]) << 8);
                        if y >= self.sprite_get_y(k) {
                            self.ram[SPRITE_AI_STATE + k] =
                                self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                            self.ram[SPRITE_Y_VEL + k] = 0;
                            self.ram[SPRITE_X_VEL + k] = 0;
                        } else {
                            let pt = self.sprite_project_speed_towards_location(k, x, y, 8);
                            self.ram[SPRITE_Y_VEL + k] = pt.y;
                            self.ram[SPRITE_X_VEL + k] = pt.x;
                            self.ram[SPRITE_GRAPHICS + k] =
                                ((k as u8) ^ self.ram[FRAME_COUNTER]) >> 3 & 1;
                            self.old_man_enable_cutscene();
                        }
                    }
                    3 => {
                        self.ram[SPRITE_STATE + k] = 0;
                        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
                        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
                    }
                    _ => {}
                }
            }
            2 => {
                self.sprite_behave_as_barrier(k);
                if self.ram[SPRITE_AI_STATE + k] != 0 {
                    self.ram[LINK_HEARTS_FILLER] = 160;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
                let j = if self.ram[SRAM_PROGRESS_INDICATOR] >= 3 {
                    2
                } else {
                    self.ram[LINK_ITEM_MOON_PEARL] as usize
                };
                if self.sprite_show_solicited_message(k, OLD_MOUNTAIN_MAN_MSGS[j]) & 0x100 != 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
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

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.sprite_show_solicited_message(k, 0x107);
                let bak = self.ram[SPRITE_X_LO + k];
                self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_sub(16);
                self.sprite_get_16bit_coords_for_prep(k);
                self.ram[SPRITE_X_VEL + k] = 1;
                self.ram[SPRITE_Y_VEL + k] = 1;
                if self.sprite_check_tile_collision(k) == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    if self.ram[FOLLOWER_INDICATOR] != 0 {
                        self.ram[SPRITE_AI_STATE + k] = 5;
                    }
                }
                self.ram[SPRITE_X_LO + k] = bak;
            }
            1 => {
                self.ram[FOLLOWER_INDICATOR] = 9;
                self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 0;
                self.load_follower_graphics();
                self.follower_initialize();
                write_le_u16(&mut self.ram, SHARED_MESSAGE_TIMER, 0x40);
                self.ram[SPRITE_STATE + k] = 0;
            }
            2 => {
                if self.sprite_check_if_link_is_busy() {
                    return;
                }
                let j = if self.ram[FOLLOWER_DROPPED] != 0 {
                    self.sprite_show_solicited_message(k, 0x109)
                } else {
                    self.sprite_show_message_on_contact(k, 0x109)
                };
                if j & 0x100 != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                }
            }
            3 => {
                if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 {
                    if self.ram[FOLLOWER_DROPPED] != 0 {
                        self.sprite_show_message_unconditional(0x10c);
                        self.ram[SPRITE_AI_STATE + k] = 2;
                    } else {
                        self.ram[ITEM_RECEIPT_METHOD] = 0;
                        self.link_receive_item(0x16, 0);
                        self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] |= 0x10;
                        self.ram[SPRITE_AI_STATE + k] = 4;
                        self.ram[FOLLOWER_INDICATOR] = 0;
                    }
                } else {
                    self.sprite_show_message_unconditional(0x10a);
                    self.ram[SPRITE_AI_STATE + k] = 2;
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
        if self.ram[SPRITE_HEAD_DIR + k] != 0 {
            self.sprite_mad_batter_bolt(k);
            return;
        }

        if self.ram[SPRITE_AI_STATE + k] != 0 {
            self.sprite_draw_single_large(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_move_z(k);

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[LINK_MAGIC_CONSUMPTION] >= 2
                    || !self.sprite_check_damage_to_link_same_layer(k)
                {
                    return;
                }
                for i in (0..=4).rev() {
                    if self.ram[ANCILLA_TYPE + i] == 0x1a {
                        self.sprite_spawn_superficial_bomb_blast(k);
                        self.sprite_sfx_queue_sfx1_with_pan(k, 0x0d);
                        self.ram[SPRITE_AI_STATE + k] =
                            self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        self.ram[SPRITE_A + k] = 20;
                        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                        self.ram[SPRITE_OAM_FLAGS + k] |= 32;
                        return;
                    }
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_sub(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = self.ram[SPRITE_A + k];
                    if self.ram[SPRITE_DELAY_MAIN + k] != 1 {
                        const RISING_UP_X_ACCEL: [i8; 2] = [-8, 7];
                        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_DELAY_MAIN + k] >> 2;
                        let idx = (self.ram[SPRITE_A + k] & 1) as usize;
                        self.ram[SPRITE_X_VEL + k] =
                            self.ram[SPRITE_X_VEL + k].wrapping_add(RISING_UP_X_ACCEL[idx] as u8);
                        self.ram[SPRITE_GRAPHICS + k] ^= 1;
                    } else {
                        self.sprite_show_message_unconditional(0x110);
                        self.ram[SPRITE_AI_STATE + k] =
                            self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        self.ram[SPRITE_GRAPHICS + k] = 0;
                        self.ram[SPRITE_Z_VEL + k] = 0;
                        self.ram[SPRITE_X_VEL + k] = 0;
                        self.ram[SPRITE_DELAY_MAIN + k] = 255;
                    }
                }
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_AUX1 + k] = 64;
                }
                const OAM_FLAGS: [u8; 8] = [0x0a, 4, 2, 4, 2, 0x0a, 4, 2];
                let idx = ((self.ram[SPRITE_DELAY_MAIN + k] >> 1) & 7) as usize;
                self.ram[SPRITE_OAM_FLAGS + k] =
                    (self.ram[SPRITE_OAM_FLAGS + k] & !0x0e) | OAM_FLAGS[idx];
                if self.ram[SPRITE_DELAY_MAIN + k] == 240 {
                    self.sprite_magic_bat_spawn_lightning(k);
                }
            }
            3 => {
                if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                    self.sprite_show_message_unconditional(0x111);
                    self.Palette_Restore_BG_And_HUD();
                    self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                        self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[LINK_MAGIC_CONSUMPTION] = 1;
                    self.hud_refresh_icon();
                } else if self.ram[SPRITE_DELAY_AUX1 + k] == 0x10 {
                    self.ram[INTRO_TIMES_PAL_FLASH] = 0x10;
                }
            }
            4 => {
                self.sprite_spawn_dummy_death_animation(k);
                self.ram[SPRITE_STATE + k] = 0;
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
            }
            _ => {}
        }
    }

    pub(super) fn sprite_72_fairy_pond(&mut self, k: usize) {
        if self.ram[SPRITE_A + k] != 0 {
            self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_sub(1);
            if self.ram[SPRITE_C + k] == 0 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_C + k] >> 3;
            self.oam_allocate_from_region_c(4);
            self.sprite_draw_single_small(k);
            return;
        }
        if self.ram[SPRITE_B + k] != 0 {
            self.faerie_queen_draw(k);
            self.ram[SPRITE_GRAPHICS + k] = self.ram[FRAME_COUNTER] >> 4 & 1;
            if self.ram[FRAME_COUNTER] & 15 != 0 {
                return;
            }
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x72, &mut info);
            if j >= 0 {
                let j = j as usize;
                let xoff = K_WISH_POND_X[(self.get_random_number() & 7) as usize] as u16;
                let yoff = K_WISH_POND_Y[(self.get_random_number() & 7) as usize] as u16;
                self.sprite_set_x(j, info.r0_x.wrapping_add(xoff));
                self.sprite_set_y(j, info.r2_y.wrapping_add(yoff));
                self.ram[SPRITE_C + j] = 31;
                self.ram[SPRITE_A + j] = 31;
                self.ram[SPRITE_FLAGS2 + j] = 0;
                self.ram[SPRITE_FLAGS3 + j] = 0x48;
                self.ram[SPRITE_OAM_FLAGS + j] = 0x48 & 0x0f;
                self.ram[SPRITE_B + j] = 1;
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
        if self.world_state_view().dungeon_room() as u8 != 21 {
            self.sprite_wish_pond3(k);
        } else {
            self.sprite_happiness_pond(k);
        }
    }

    pub(super) fn sprite_wish_pond3(&mut self, k: usize) {
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 || self.sprite_check_if_link_is_busy() {
                    return;
                }
                if self.sprite_show_message_on_contact(k, 0x14a) & 0x100 != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.link_reset_properties_a();
                    self.ram[LINK_DIRECTION_FACING] = 0;
                    self.ram[SPRITE_HEAD_DIR + k] = 0;
                }
            }
            1 => {
                if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 {
                    self.sprite_show_message_unconditional(0x8a);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                } else {
                    self.sprite_show_message_unconditional(0x14b);
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 255;
                }
            }
            2 => {
                self.ram[SPRITE_AI_STATE + k] = 3;
                let j = self.ram[CHOICE_IN_MULTISELECT_BOX] as usize;
                self.ram[SPRITE_C + k] = j as u8;
                let item = self.ram[LINK_ITEM_BOW + j];
                self.ram[LINK_ITEM_BOW + j] = 0;
                let item_idx = if j == 3 || j == 32 { 1 } else { item };
                let data_idx = K_WISH_POND_ITEM_OFFS[j]
                    .wrapping_add(item_idx)
                    .wrapping_sub(1) as usize;
                let t = K_WISH_POND_ITEM_DATA[data_idx];
                self.ancilla_add_tossed_pond_item(0x28, t, 4);
                self.hud_refresh_icon();
                self.ram[SPRITE_GRAPHICS + k] = t;
                self.ram[SPRITE_D + k] = item;
                self.ram[SPRITE_DELAY_MAIN + k] = 255;
            }
            3 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0x72, &mut info);
                    if j >= 0 {
                        let j = j as usize;
                        self.sprite_set_x(j, info.r0_x);
                        self.sprite_set_y(j, info.r2_y.wrapping_sub(80));
                        self.ram[MUSIC_CONTROL] = 0x1b;
                        self.ram[LAST_MUSIC_CONTROL] = 0;
                        self.ram[SPRITE_B + j] = 1;
                        self.Palette_AssertTranslucencySwap();
                        self.PaletteFilter_WishPonds();
                        self.ram[SPRITE_E + k] = j as u8;
                        self.ram[SPRITE_AI_STATE + k] = 4;
                        self.ram[SPRITE_DELAY_MAIN + k] = 255;
                    }
                }
            }
            4 => {
                if self.ram[FRAME_COUNTER] & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.ram[PALETTE_FILTER_COUNTDOWN] == 0 {
                        self.sprite_show_message_unconditional(0x8b);
                        self.Palette_RevertTranslucencySwap();
                        self.ram[TS_COPY] = 0;
                        self.ram[CGADSUB_COPY] = 0x20;
                        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                            self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                        self.ram[SPRITE_AI_STATE + k] = 5;
                    }
                }
            }
            5 => {
                self.ram[SPRITE_AI_STATE + k] = if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 {
                    6
                } else {
                    11
                };
            }
            6 => {
                self.ram[SPRITE_AI_STATE + k] = 7;
                if self.ram[SAVEGAME_IS_DARKWORLD] == 0 {
                    match self.ram[SPRITE_GRAPHICS + k] {
                        12 => {
                            self.ram[SPRITE_GRAPHICS + k] = 42;
                            self.ram[SPRITE_HEAD_DIR + k] = 1;
                        }
                        4 => {
                            self.ram[SPRITE_GRAPHICS + k] = 5;
                            self.ram[SPRITE_HEAD_DIR + k] = 2;
                        }
                        22 => {
                            self.ram[SPRITE_GRAPHICS + k] = 44;
                            self.ram[SPRITE_HEAD_DIR + k] = 3;
                        }
                        _ => {
                            self.sprite_show_message_unconditional(0x14d);
                            return;
                        }
                    }
                } else {
                    match self.ram[SPRITE_GRAPHICS + k] {
                        58 => {
                            self.ram[SPRITE_GRAPHICS + k] = 59;
                            self.ram[SPRITE_HEAD_DIR + k] = 4;
                            self.sprite_show_message_unconditional(0x14f);
                            return;
                        }
                        2 => {
                            self.ram[SPRITE_GRAPHICS + k] = 3;
                            self.ram[SPRITE_HEAD_DIR + k] = 5;
                        }
                        22 => {
                            self.ram[SPRITE_GRAPHICS + k] = 44;
                            self.ram[SPRITE_HEAD_DIR + k] = 3;
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
                if self.ram[SPRITE_C + k] == 3 {
                    let idx = self.ram[SPRITE_C + k] as usize;
                    self.ram[LINK_ITEM_BOW + idx] = self.ram[SPRITE_D + k];
                }
                self.Palette_AssertTranslucencySwap();
                self.ram[TS_COPY] = 2;
                self.ram[CGADSUB_COPY] = 0x30;
                self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                    self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                self.ram[SPRITE_AI_STATE + k] = 8;
            }
            8 => {
                if self.ram[FRAME_COUNTER] & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.ram[PALETTE_FILTER_COUNTDOWN] == 30 {
                        let j = self.ram[SPRITE_E + k] as usize;
                        self.ram[SPRITE_STATE + j] = 0;
                    } else if self.ram[PALETTE_FILTER_COUNTDOWN] == 0 {
                        self.ram[SPRITE_AI_STATE + k] = 9;
                    }
                }
            }
            9 => {
                self.PaletteFilter_RestoreSP5F();
                self.Palette_RevertTranslucencySwap();
                self.ram[ITEM_RECEIPT_METHOD] = 2;
                self.link_receive_item(self.ram[SPRITE_GRAPHICS + k], 0);
                self.ram[SPRITE_AI_STATE + k] = 10;
            }
            10 => {
                const MSGS: [u16; 5] = [0x8f, 0x90, 0x92, 0x91, 0x93];
                let head = self.ram[SPRITE_HEAD_DIR + k];
                if head != 0 {
                    self.sprite_show_message_unconditional(MSGS[head.wrapping_sub(1) as usize]);
                }
                self.ram[SPRITE_AI_STATE + k] = 0;
                self.ram[SPRITE_DELAY_MAIN + k] = 255;
            }
            11 => {
                self.sprite_show_message_unconditional(0x8d);
                self.ram[SPRITE_AI_STATE + k] = 12;
            }
            12 => {
                self.ram[SPRITE_AI_STATE + k] = if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 {
                    13
                } else {
                    6
                };
            }
            13 => {
                self.sprite_show_message_unconditional(0x8e);
                self.ram[SPRITE_AI_STATE + k] = 7;
            }
            _ => {}
        }
    }

    pub(super) fn sprite_happiness_pond(&mut self, k: usize) {
        const COST: [u8; 4] = [5, 20, 25, 50];
        const COST_HEX: [u8; 4] = [5, 0x20, 0x25, 0x50];
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 || self.sprite_check_if_link_is_busy() {
                    return;
                }
                if self.sprite_show_message_on_contact(k, 0x89) & 0x100 != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.link_reset_properties_a();
                    self.ancilla_terminate_sparkle_objects();
                    self.ram[LINK_DIRECTION_FACING] = 0;
                }
            }
            1 => {
                if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 {
                    let i = u8::from(
                        (self.ram[LINK_BOMB_UPGRADES] | self.ram[LINK_ARROW_UPGRADES]) != 0,
                    );
                    self.ram[SPRITE_GRAPHICS + k] = i * 2;
                    let cost_index = (i * 2) as usize;
                    write_le_u16(
                        &mut self.ram,
                        DIALOGUE_NUMBER_PREP,
                        u16::from(COST_HEX[cost_index])
                            | (u16::from(COST_HEX[cost_index + 1]) << 8),
                    );
                    self.sprite_show_message_unconditional(0x14e);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                } else {
                    self.happiness_pond_show_later(k);
                }
            }
            2 => {
                let i =
                    self.ram[SPRITE_GRAPHICS + k].wrapping_add(self.ram[CHOICE_IN_MULTISELECT_BOX]);
                self.ram[DIALOGUE_NUMBER_PREP + 1] = COST_HEX[i as usize];
                if read_le_u16(&self.ram, LINK_RUPEES_GOAL) < COST[i as usize] as u16 {
                    self.happiness_pond_show_later(k);
                } else {
                    self.ram[SPRITE_D + k] = COST[i as usize];
                    self.ram[SPRITE_HEAD_DIR + k] = i;
                    self.ram[SPRITE_AI_STATE + k] = 3;
                }
            }
            3 => {
                self.ram[SPRITE_DELAY_MAIN + k] = 80;
                let i = self.ram[SPRITE_D + k];
                let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL).wrapping_sub(i as u16);
                write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees);
                self.ram[LINK_RUPEES_IN_POND_PREP] =
                    self.ram[LINK_RUPEES_IN_POND_PREP].wrapping_add(i);
                self.add_happiness_pond_rupees(self.ram[SPRITE_HEAD_DIR + k]);
                if self.ram[LINK_RUPEES_IN_POND_PREP] >= 100 {
                    self.ram[LINK_RUPEES_IN_POND_PREP] =
                        self.ram[LINK_RUPEES_IN_POND_PREP].wrapping_sub(100);
                    self.ram[SPRITE_AI_STATE + k] = 5;
                    return;
                }
                let pond = self.ram[LINK_RUPEES_IN_POND_PREP];
                self.ram[DIALOGUE_NUMBER_PREP] = (pond / 10) * 16 + (pond % 10);
                self.ram[SPRITE_AI_STATE + k] = 4;
            }
            4 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.sprite_show_message_unconditional(0x94);
                    self.ram[SPRITE_AI_STATE + k] = 13;
                }
            }
            5 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0x72, &mut info);
                    assert!(
                        j >= 0,
                        "Sprite_HappinessPond expected Sprite_SpawnDynamically to succeed"
                    );
                    let j = j as usize;
                    self.sprite_set_x(j, info.r0_x);
                    self.sprite_set_y(j, info.r2_y.wrapping_sub(80));
                    self.ram[MUSIC_CONTROL] = 0x1b;
                    self.ram[LAST_MUSIC_CONTROL] = 0;
                    self.ram[SPRITE_B + j] = 1;
                    self.Palette_AssertTranslucencySwap();
                    self.PaletteFilter_WishPonds();
                    self.ram[SPRITE_E + k] = j as u8;
                    self.ram[SPRITE_AI_STATE + k] = 6;
                    self.ram[SPRITE_DELAY_MAIN + k] = 255;
                }
            }
            6 => {
                if self.ram[FRAME_COUNTER] & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.ram[PALETTE_FILTER_COUNTDOWN] == 0 {
                        self.sprite_show_message_unconditional(0x95);
                        self.Palette_RevertTranslucencySwap();
                        self.ram[TS_COPY] = 0;
                        self.ram[CGADSUB_COPY] = 0x20;
                        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                            self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                        self.ram[SPRITE_AI_STATE + k] = 7;
                    }
                }
            }
            7 => {
                self.ram[SPRITE_AI_STATE + k] = if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 {
                    8
                } else {
                    12
                };
            }
            8 => {
                const MAX_BOMBS_HEX: [u8; 8] = [0x10, 0x15, 0x20, 0x25, 0x30, 0x35, 0x40, 0x50];
                let i = self.ram[LINK_BOMB_UPGRADES].wrapping_add(1);
                if i != 8 {
                    self.ram[LINK_BOMB_UPGRADES] = i;
                    self.ram[LINK_BOMB_FILLER] = MAX_BOMBS_HEX[i as usize];
                    self.ram[DIALOGUE_NUMBER_PREP] = self.ram[LINK_BOMB_FILLER];
                    self.sprite_show_message_unconditional(0x96);
                } else {
                    let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL).wrapping_add(100);
                    write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees);
                    self.sprite_show_message_unconditional(0x98);
                }
                self.ram[SPRITE_AI_STATE + k] = 9;
            }
            9 => {
                self.Palette_AssertTranslucencySwap();
                self.ram[TS_COPY] = 2;
                self.ram[CGADSUB_COPY] = 0x30;
                self.ram[FLAG_UPDATE_CGRAM_IN_NMI] =
                    self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
                self.ram[SPRITE_AI_STATE + k] = 10;
            }
            10 => {
                if self.ram[FRAME_COUNTER] & 7 == 0 {
                    self.PaletteFilter_SP5F();
                    if self.ram[PALETTE_FILTER_COUNTDOWN] == 30 {
                        let j = self.ram[SPRITE_E + k] as usize;
                        self.ram[SPRITE_STATE + j] = 0;
                    } else if self.ram[PALETTE_FILTER_COUNTDOWN] == 0 {
                        self.ram[SPRITE_AI_STATE + k] = 11;
                    }
                }
            }
            11 => {
                self.PaletteFilter_RestoreSP5F();
                self.Palette_RevertTranslucencySwap();
                self.ram[SPRITE_AI_STATE + k] = 0;
                self.ram[SPRITE_DELAY_MAIN + k] = 255;
            }
            12 => {
                const MAX_ARROWS_HEX: [u8; 8] = [0x30, 0x35, 0x40, 0x45, 0x50, 0x55, 0x60, 0x70];
                let i = self.ram[LINK_ARROW_UPGRADES].wrapping_add(1);
                if i != 8 {
                    self.ram[LINK_ARROW_UPGRADES] = i;
                    self.ram[LINK_ARROW_FILLER] = MAX_ARROWS_HEX[i as usize];
                    self.ram[DIALOGUE_NUMBER_PREP] = self.ram[LINK_ARROW_FILLER];
                    self.sprite_show_message_unconditional(0x97);
                } else {
                    let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL).wrapping_add(100);
                    write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees);
                    self.sprite_show_message_unconditional(0x98);
                }
                self.ram[SPRITE_AI_STATE + k] = 9;
            }
            13 => {
                self.sprite_show_message_unconditional(0x154);
                self.ram[SPRITE_AI_STATE + k] = 14;
            }
            14 => {
                const LUCK_MSG: [u16; 4] = [0x150, 0x151, 0x152, 0x153];
                const LUCK: [u8; 4] = [1, 0, 0, 2];
                let i = (self.get_random_number() & 3) as usize;
                self.ram[ITEM_DROP_LUCK_PREP] = LUCK[i];
                self.ram[LUCK_KILL_COUNTER_PREP] = 0;
                self.sprite_show_message_unconditional(LUCK_MSG[i]);
                self.ram[SPRITE_AI_STATE + k] = 0;
                self.ram[SPRITE_DELAY_MAIN + k] = 255;
            }
            _ => {}
        }
    }

    fn happiness_pond_show_later(&mut self, k: usize) {
        self.sprite_show_message_unconditional(0x14c);
        self.ram[SPRITE_AI_STATE + k] = 0;
        self.ram[SPRITE_DELAY_MAIN + k] = 255;
    }

    pub(super) fn wish_pond2_draw(&mut self, k: usize) {
        const DMD: [DrawMultipleData; 8] = [
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
        if self.world_state_view().dungeon_room() as u8 == 21 {
            return;
        }
        let t = self.ram[SPRITE_AI_STATE + k];
        if !matches!(t, 5 | 6 | 11 | 12) {
            return;
        }
        let g = self.ram[SPRITE_GRAPHICS + k] as usize;
        let mut f = K_WISH_POND2_OAM_FLAGS[g];
        if f == 0xff {
            f = 5;
        }
        self.ram[SPRITE_OAM_FLAGS + k] = (f & 7) * 2;
        let start = ((K_RECEIVE_ITEM_TAB1_PREP[g] >> 1) * 4) as usize;
        self.sprite_draw_multiple(k, &DMD[start..start + 4], None);
    }

    fn sprite_get_16bit_coords_for_prep(&mut self, k: usize) {
        let x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        write_le_u16(&mut self.ram, CUR_SPRITE_X, x);
        write_le_u16(&mut self.ram, CUR_SPRITE_Y, y);
    }

    pub(super) fn pink_ball_handle_deceleration(&mut self, k: usize) {
        if self.ram[SPRITE_X_VEL + k] != 0 {
            self.ram[SPRITE_X_VEL + k] =
                self.ram[SPRITE_X_VEL + k].wrapping_add(if sign8(self.ram[SPRITE_X_VEL + k]) {
                    2
                } else {
                    0u8.wrapping_sub(2)
                });
        }
        if self.ram[SPRITE_Y_VEL + k] != 0 {
            self.ram[SPRITE_Y_VEL + k] =
                self.ram[SPRITE_Y_VEL + k].wrapping_add(if sign8(self.ram[SPRITE_Y_VEL + k]) {
                    2
                } else {
                    0u8.wrapping_sub(2)
                });
        }
    }

    pub(super) fn pink_ball_distress(&mut self, k: usize) {
        let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        self.sprite_draw_distress_custom(x, y, self.ram[FRAME_COUNTER]);
    }

    pub(super) fn spawn_apple(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xac, &mut info);
        if j < 0 {
            return;
        }

        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        self.ram[SPRITE_AI_STATE + j] = 1;
        self.ram[SPRITE_A + j] = 255;
        self.ram[SPRITE_Z + j] = 8;
        self.ram[SPRITE_Z_VEL + j] = 22;
        let x = (info.r0_x & !0xff).wrapping_add(u16::from(self.get_random_number()));
        let y = (info.r2_y & !0xff).wrapping_add(u16::from(self.get_random_number()));
        let pt = self.sprite_project_speed_towards_location(k, x, y, 10);
        self.ram[SPRITE_X_VEL + j] = pt.x;
        self.ram[SPRITE_Y_VEL + j] = pt.y;
    }

    pub(super) fn sprite_transmute_to_bomb(&mut self, k: usize) {
        self.ram[SPRITE_TYPE + k] = 0x4a;
        self.ram[SPRITE_C + k] = 1;
        self.ram[SPRITE_DELAY_AUX1 + k] = 255;
        self.ram[SPRITE_FLAGS3 + k] = 0x18;
        self.ram[SPRITE_OAM_FLAGS + k] = 8;
        self.ram[SPRITE_HEALTH + k] = 0;
    }

    pub(super) fn beamos_fire_laser(&mut self, k: usize) {
        if self.ram[SPRITE_LIMIT_INSTANCE] >= 4 {
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
            info.r0_x
                .wrapping_add_signed(i16::from(self.ram[DUNGMAP_VAR7] as i8)),
        );
        self.sprite_set_y(
            j,
            info.r2_y
                .wrapping_add_signed(i16::from(self.ram[DUNGMAP_VAR7 + 1] as i8)),
        );
        self.sprite_apply_speed_towards_link(j, 0x20);
        self.ram[SPRITE_FLAGS2 + j] = 0x3f;
        self.ram[SPRITE_FLAGS4 + j] = 0x54;
        self.ram[SPRITE_C + j] = 1;
        self.ram[SPRITE_DEFL_BITS + j] = 0x48;
        self.ram[SPRITE_OAM_FLAGS + j] = 3;
        self.ram[SPRITE_BUMP_DAMAGE + j] = 4;
        self.ram[SPRITE_DELAY_AUX1 + j] = 12;
        let t = self.ram[SPRITE_LIMIT_INSTANCE] as usize;
        self.ram[SPRITE_GRAPHICS + j] = t as u8;
        self.ram[SPRITE_LIMIT_INSTANCE] = self.ram[SPRITE_LIMIT_INSTANCE].wrapping_add(1);

        for i in 0..32 {
            let o = t * 32 + i;
            self.ram[BEAMOS_X_LO_PREP + o] = self.ram[SPRITE_X_LO + j];
            self.ram[BEAMOS_X_HI + o] = self.ram[SPRITE_X_HI + j];
            self.ram[BEAMOS_Y_LO_PREP + o] = self.ram[SPRITE_Y_LO + j];
            self.ram[BEAMOS_Y_HI_PREP + o] = self.ram[SPRITE_Y_HI + j];
        }
    }

    pub(super) fn octoballoon_find(&mut self) -> bool {
        (0..16)
            .rev()
            .any(|i| self.ram[SPRITE_STATE + i] != 0 && self.ram[SPRITE_TYPE + i] == 0x10)
    }

    pub(super) fn potion_cauldron_go_beep(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x3c);
    }

    pub(super) fn potion_cauldron_check_bottles(&mut self) -> bool {
        (self.ram[LINK_BOTTLE_INFO]
            | self.ram[LINK_BOTTLE_INFO + 1]
            | self.ram[LINK_BOTTLE_INFO + 2]
            | self.ram[LINK_BOTTLE_INFO + 3])
            >= 2
    }

    pub(super) fn dark_world_hint_npc_handle_payment(&mut self) -> bool {
        let rupees_goal = read_le_u16(&self.ram, LINK_RUPEES_GOAL);
        if rupees_goal < 20 {
            return false;
        }
        write_le_u16(
            &mut self.ram,
            LINK_RUPEES_GOAL,
            rupees_goal.wrapping_sub(20),
        );
        true
    }

    pub(super) fn dark_world_hint_npc_idle(&mut self, k: usize) {
        if self.sprite_show_solicited_message(k, 0xfe) & 0x100 != 0 {
            self.ram[SPRITE_AI_STATE + k] = 1;
        }
    }

    pub(super) fn fairy_check_if_touchable(&mut self, k: usize) {
        let msg = read_le_u16(&self.ram, DIALOGUE_MESSAGE_INDEX);
        if self.frame_control_view().submodule() == 2 && (msg == 0xc9 || msg == 0xca) {
            self.ram[SPRITE_DELAY_AUX4 + k] = 40;
        }
    }

    pub(super) fn buzzblob_select_new_direction(&mut self, k: usize) {
        const XVEL: [i8; 8] = [3, 2, -2, -3, -2, 2, 0, 0];
        const YVEL: [i8; 8] = [0, 2, 2, 0, -2, -2, 0, 0];
        const DELAY: [u8; 8] = [48, 48, 48, 48, 48, 48, 64, 64];
        let j = (self.get_random_number() & 7) as usize;
        self.ram[SPRITE_X_VEL + k] = XVEL[j] as u8;
        self.ram[SPRITE_Y_VEL + k] = YVEL[j] as u8;
        self.ram[SPRITE_DELAY_MAIN + k] = DELAY[j];
    }

    pub(super) fn lumberjack_check_proximity(&mut self, _k: usize, j: usize) -> bool {
        const X: [u16; 2] = [48, 52];
        const Y: [u16; 2] = [19, 20];
        const W: [u16; 2] = [98, 106];
        const H: [u16; 2] = [37, 40];
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        cur_x.wrapping_sub(link_x).wrapping_add(X[j]) < W[j]
            && cur_y.wrapping_sub(link_y).wrapping_add(Y[j]) < H[j]
    }

    pub(super) fn blind_laser_spawn_trail_garnish(&mut self, j: usize) {
        let k = self.garnish_alloc_overwrite_old() as usize;
        self.ram[GARNISH_TYPE + k] = 15;
        self.ram[GARNISH_ACTIVE_PREP] = 15;
        self.ram[GARNISH_OAM_FLAGS_PREP + k] = self.ram[SPRITE_GRAPHICS + j];
        self.ram[GARNISH_SPRITE_PREP + k] = j as u8;
        self.ram[GARNISH_X_LO_PREP + k] = self.ram[SPRITE_X_LO + j];
        self.ram[GARNISH_X_HI_PREP + k] = self.ram[SPRITE_X_HI + j];
        self.garnish_set_y(k, self.sprite_get_y(j).wrapping_add(16));
        self.ram[GARNISH_COUNTDOWN_PREP + k] = 10;
    }

    pub(super) fn running_boy_spawn_dust_garnish(&mut self, k: usize) {
        self.ram[SPRITE_DIE_ACTION + k] = self.ram[SPRITE_DIE_ACTION + k].wrapping_add(1);
        if self.ram[SPRITE_DIE_ACTION + k] & 0x0f != 0 {
            return;
        }
        let j = self.garnish_alloc_force() as usize;
        self.ram[GARNISH_TYPE + j] = 20;
        self.ram[GARNISH_ACTIVE_PREP] = 20;
        self.garnish_set_x(j, self.sprite_get_x(k).wrapping_add(4));
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(28));
        self.ram[GARNISH_COUNTDOWN_PREP + j] = 10;
    }

    pub(super) fn sprite_cd_spawn_garnish(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        if self.ram[SPRITE_SUBTYPE2 + k] & 7 != 0 {
            return;
        }
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x14);
        let j = self.garnish_alloc_overwrite_old() as usize;
        self.ram[GARNISH_TYPE + j] = 0x0c;
        self.ram[GARNISH_ACTIVE_PREP] = 0x0c;
        self.ram[GARNISH_SPRITE_PREP + j] = k as u8;
        self.garnish_set_x(j, self.sprite_get_x(k));
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(16));
        self.ram[GARNISH_COUNTDOWN_PREP + j] = 127;
    }

    pub(super) fn dark_world_hint_npc_restore_health(&mut self, k: usize) {
        self.ram[LINK_HEARTS_FILLER] = 0xa0;
        self.ram[SPRITE_AI_STATE + k] = 0;
    }

    pub(super) fn pipe_validate_entry(&mut self) -> bool {
        for k in (0..=4).rev() {
            if self.ram[ANCILLA_TYPE + k] == 0x31 {
                self.ram[LINK_POSITION_MODE] = 0;
                self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
                self.ram[ANCILLA_TYPE + k] = 0;
                break;
            }
        }
        (self.ram[LINK_STATE_BITS] & 0x80) | self.ram[LINK_AUXILIARY_STATE] != 0
    }

    pub(super) fn sprite_prep_zoro(&mut self, k: usize) {
        self.ram[SPRITE_D + k] = self.ram[SPRITE_TYPE + k].wrapping_sub(0x9c) << 1;
        self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k].wrapping_sub(1);
    }

    pub(super) fn sprite_prep_popo(&mut self, k: usize) {
        self.ram[SPRITE_B + k] = 7;
    }

    pub(super) fn sprite_prep_popo2(&mut self, k: usize) {
        self.ram[SPRITE_B + k] = 15;
    }

    pub(super) fn sprite_prep_statue(&mut self, k: usize) {
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(7);
    }

    pub(super) fn sprite_prep_bari(&mut self, k: usize) {
        self.ram[SPRITE_Z + k] = 6;
        if self.ram[DUNGEON_ROOM_INDEX2] == 206 {
            self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_sub(1);
        }
        self.ram[SPRITE_DELAY_AUX1 + k] = (self.get_random_number() & 63).wrapping_add(128);
    }

    pub(super) fn sprite_prep_green_stalfos(&mut self, k: usize) {
        self.ram[SPRITE_Z + k] = 9;
    }

    pub(super) fn sprite_prep_water_lever(&mut self, k: usize) {
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(5);
    }

    pub(super) fn sprite_prep_fire_debirando(&mut self, k: usize) {
        self.ram[SPRITE_TYPE + k] = 0x63;
        self.sprite_prep_load_properties(k);
        self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_sub(1);
        self.sprite_prep_debirando_pit(k);
    }

    pub(super) fn sprite_prep_debirando_pit(&mut self, k: usize) {
        const DEBIRANDO_OAM_FLAGS: [u8; 2] = [6, 8];

        self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
        self.ram[SPRITE_DELAY_MAIN + k] = 0;
        self.ram[SPRITE_GRAPHICS + k] = 6;
        self.sprite_prep_ignore_projectiles(k);

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x64, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.ram[SPRITE_DELAY_MAIN + j] = 96;
            self.ram[SPRITE_HEAD_DIR + k] = j as u8;
            self.ram[SPRITE_G + j] = self.ram[SPRITE_G + k];
            self.ram[SPRITE_OAM_FLAGS + j] = DEBIRANDO_OAM_FLAGS[self.ram[SPRITE_G + j] as usize];
        }
    }

    pub(super) fn sprite_prep_weak_guard(&mut self, k: usize) {
        let dir = self.get_random_number() & 3;
        self.ram[SPRITE_D + k] = dir;
        self.ram[SPRITE_HEAD_DIR + k] = dir;
        self.ram[SPRITE_DELAY_MAIN + k] = 16;
    }

    pub(super) fn sprite_prep_laser_eye_bounce(&mut self, k: usize) {
        let t = self.ram[SPRITE_TYPE + k];
        self.ram[SPRITE_D + k] = t.wrapping_sub(0x95);
        if t >= 0x97 {
            self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
            self.ram[SPRITE_HEAD_DIR + k] = (self.ram[SPRITE_X_LO + k] & 16) ^ 16;
            if self.ram[SPRITE_HEAD_DIR + k] == 0 {
                self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k]
                    .wrapping_add(if (t & 1) != 0 { (-8i8) as u8 } else { 8 });
            }
        } else {
            self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_Y_LO + k] & 16;
            if self.ram[SPRITE_HEAD_DIR + k] == 0 {
                self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k]
                    .wrapping_add(if (t & 1) != 0 { (-8i8) as u8 } else { 8 });
            }
        }
    }

    pub(super) fn sprite_prep_wall_cannon(&mut self, k: usize) {
        self.ram[SPRITE_D + k] = self.ram[SPRITE_TYPE + k].wrapping_sub(0x66);
        self.ram[SPRITE_A + k] = self.ram[SPRITE_D + k] & 2;
    }

    pub(super) fn sprite_prep_purple_chest(&mut self, k: usize) {
        if self.ram[FOLLOWER_INDICATOR] != 12
            && self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 16 == 0
            && self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 32 != 0
        {
            self.ram[SPRITE_IGNORE_PROJECTILE + k] =
                self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        } else {
            self.ram[SPRITE_STATE + k] = 0;
        }
    }

    pub(super) fn sprite_prep_smithy(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        if self.ram[SAVEGAME_IS_DARKWORLD] & 64 != 0 {
            if self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 32 != 0
                || self.ram[FOLLOWER_INDICATOR] != 0
            {
                self.ram[SPRITE_STATE + k] = 0;
            } else {
                self.ram[SPRITE_SUBTYPE2 + k] = 2;
            }
            return;
        }

        self.sprite_prep_smithy_spawn_dumb_barrier_sprite(k);
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(2);
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(3);
        if self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 32 == 0 {
            return;
        }

        let j = self.sprite_prep_smithy_spawn_dwarf_pal(k);
        self.sprite_prep_smithy_spawn_dumb_barrier_sprite(j as usize);
        self.ram[SPRITE_E + j as usize] = k as u8;
        self.ram[SPRITE_E + k] = j as u8;

        if self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 0x80 != 0 {
            self.ram[SPRITE_AI_STATE + k] = 5;
            self.ram[SPRITE_AI_STATE + j as usize] = 5;
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
        self.ram[SPRITE_X_LO + j] = self.ram[SPRITE_X_LO + j].wrapping_add(0x2c);
        self.ram[SPRITE_D + j] = 1;
        self.ram[SPRITE_A + j] = 4;
        self.ram[SPRITE_IGNORE_PROJECTILE + j] = 4;
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
        self.ram[SPRITE_SUBTYPE2 + j] = 1;
        self.ram[SPRITE_FLAGS4 + j] = 0;
        self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
    }

    pub(super) fn sprite_prep_ignore_projectiles(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_haunted_grove_animal(&mut self, k: usize) {
        self.ram[SPRITE_D + k] = self.sprite_is_right_of_link(k).a;
        self.sprite_prep_haunted_grove_ostritch(k);
    }

    pub(super) fn sprite_prep_haunted_grove_ostritch(&mut self, k: usize) {
        if self.ram[LINK_ITEM_FLUTE] >= 2 {
            self.ram[SPRITE_STATE + k] = 0;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_whirlpool(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.ram[SPRITE_A + k] = 1;
    }

    pub(super) fn sprite_prep_bonk_item(&mut self, k: usize) {
        const DASH_ITEM_MASK: [u16; 2] = [0x4000, 0x2000];
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            self.ram[SPRITE_GRAPHICS + k] = 2;
            return;
        }

        self.ram[SPRITE_FLOOR + k] = 2;
        if self.world_state_view().dungeon_room() == 0x0107 {
            if self.ram[LINK_ITEM_BOOK] != 0 {
                self.ram[SPRITE_STATE + k] = 0;
            } else {
                self.DecodeAnimatedSpriteTile_variable(0x0e);
            }
        } else {
            let j = self.ram[ITEM_DROP_COUNTER];
            self.ram[ITEM_DROP_COUNTER] = self.ram[ITEM_DROP_COUNTER].wrapping_add(1);
            self.ram[SPRITE_DIE_ACTION + k] = j;
            if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & DASH_ITEM_MASK[j as usize] != 0 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k].wrapping_add(1);
            self.ram[SPRITE_OAM_FLAGS + k] = 8;
            self.ram[SPRITE_FLAGS3 + k] |= 0x20;
        }
    }

    pub(super) fn sprite_prep_digging_game_guy_bounce(&mut self, k: usize) {
        if self.player_state_view().y() < self.sprite_get_y(k) {
            self.ram[SPRITE_AI_STATE + k] = 5;
            self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_sub(9);
            self.ram[SPRITE_GRAPHICS + k] = 1;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
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
        self.ram[SPRITE_X_VEL + k] = 0;

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_Y_LO + k].wrapping_add(7) < self.ram[LINK_Y_COORD]
                    && self.sprite_direction_to_face_link(k, None) == 2
                {
                    if self.ram[FOLLOWER_INDICATOR] == 0 {
                        if self.sprite_show_solicited_message(k, 0x187) & 0x100 != 0 {
                            self.ram[SPRITE_AI_STATE + k] =
                                self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        }
                    } else {
                        self.sprite_show_solicited_message(k, 0x18c);
                    }
                }
            }
            1 => {
                let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL);
                if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 && rupees >= 80 {
                    write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees.wrapping_sub(80));
                    self.sprite_show_message_unconditional(0x188);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_GRAPHICS + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 80;
                    self.ram[BEAMOS_X_HI] = 0;
                    self.ram[BEAMOS_X_HI + 1] = 0;
                    self.ram[SPRITE_DELAY_AUX1 + k] = 5;
                    self.sprite_initialize_secondary_item_minigame(1);
                    self.ram[MUSIC_CONTROL] = 14;
                } else {
                    self.sprite_show_message_unconditional(0x189);
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_GRAPHICS + k] = 1;
                } else if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                    self.ram[SPRITE_GRAPHICS + k] ^= 3;
                    if self.ram[SPRITE_GRAPHICS + k] & 1 != 0 {
                        self.ram[SPRITE_X_VEL + k] = (-16i8) as u8;
                    }
                    self.ram[SPRITE_DELAY_AUX1 + k] = 5;
                }
            }
            3 => {
                self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                self.ram[SUPER_BOMB_INDICATOR_COUNTER] = 0;
                self.ram[SUPER_BOMB_INDICATOR_TIMER] = 30;
            }
            4 => {
                if (self.ram[SUPER_BOMB_INDICATOR_TIMER] as i8) > 0
                    || self.ram[LINK_POSITION_MODE] & 1 != 0
                {
                    return;
                }
                self.ram[MUSIC_CONTROL] = 9;
                self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                self.ram[IS_ARCHER_OR_SHOVEL_GAME] = 0;
                write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x18a);
                self.sprite_show_message_minimal_c();
                self.ram[SUPER_BOMB_INDICATOR_TIMER] = 254;
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
        self.ram[IS_ARCHER_OR_SHOVEL_GAME] = what;
        self.link_reset_properties_c();
        for k in (0..=4).rev() {
            match self.ram[ANCILLA_TYPE + k] {
                0x30 | 0x31 => self.ram[ANCILLA_TYPE + k] = 0,
                5 => {
                    self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
                    self.ram[ANCILLA_TYPE + k] = 0;
                }
                _ => {}
            }
        }
    }

    pub(super) fn sprite_prep_thieves_town_grate(&mut self, k: usize) {
        if self.ram[SAVE_OW_EVENT_INFO + 0x58] & 0x20 != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(8));
    }

    pub(super) fn sprite_prep_rupee_pull(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(8));
    }

    pub(super) fn sprite_prep_shopkeeper(&mut self, k: usize) {
        const SHOP_KEEPER_WHERE: [u8; 13] = [
            0x0f, 0x10, 0x00, 0x06, 0x18, 0x12, 0x1e, 0xff, 0x1f, 0x23, 0x24, 0x25, 0x27,
        ];

        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.ram[SPRITE_FLAGS2 + k] |= 2;
        self.ram[SPRITE_OAM_FLAGS + k] |= 12;
        self.ram[SPRITE_FLAGS3 + k] |= 16;

        let room = self.ram[DUNGEON_ROOM_INDEX];
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
                self.ram[SPRITE_SUBTYPE2 + k] = 4;
                self.ram[MINIGAME_CREDITS_PREP] = 0xff;
            }
            3 => {
                self.ram[SPRITE_SUBTYPE2 + k] = 1;
                self.ram[SPRITE_GRAPHICS + k] = 1;
                self.ram[MINIGAME_CREDITS_PREP] = 0xff;
            }
            4 => {
                self.ram[SPRITE_SUBTYPE2 + k] = 3;
                self.ram[MINIGAME_CREDITS_PREP] = 0xff;
            }
            5 | 7 | 8 => {
                self.shop_keeper_spawn_shop_item(k, 0, 7);
                self.shop_keeper_spawn_shop_item(k, 1, 10);
                self.shop_keeper_spawn_shop_item(k, 2, 12);
            }
            6 | 9 | 12 => self.ram[SPRITE_SUBTYPE2 + k] = 2,
            10 => self.ram[SPRITE_SUBTYPE2 + k] = 5,
            11 => self.ram[SPRITE_SUBTYPE2 + k] = 6,
            _ => unreachable!(),
        }
    }

    pub(super) fn shop_keeper_spawn_shop_item(&mut self, k: usize, pos: usize, what: u8) {
        const SHOP_KEEPER_ITEM_X: [i16; 3] = [-44, 8, 60];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0xbb, &mut info, 12);
        assert!(j >= 0);
        let j = j as usize;
        self.ram[SPRITE_IGNORE_PROJECTILE + j] = what;
        self.ram[SPRITE_SUBTYPE2 + j] = what;
        self.sprite_set_x(j, info.r0_x.wrapping_add(SHOP_KEEPER_ITEM_X[pos] as u16));
        self.sprite_set_y(j, info.r2_y.wrapping_add(0x27));
        self.ram[SPRITE_FLAGS2 + j] |= 4;
    }

    pub(super) fn shop_keeper_rapid_terminate_receive_item(&mut self) {
        for i in (0..=4).rev() {
            if self.ram[ANCILLA_TYPE + i] == 0x22 {
                self.ram[ANCILLA_AUX_TIMER + i] = 1;
            }
        }
    }

    pub(super) fn sprite_spawn_bat_crash_cutscene(&mut self) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0x37, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.ram[SPRITE_Y_VEL + j] = 0;
            self.ram[SPRITE_B + j] = 0;
            self.ram[SPRITE_D + j] = 0;
            self.ram[SPRITE_FLOOR + j] = 0;
            self.ram[SPRITE_SUBTYPE2 + j] = 1;
            self.ram[SPRITE_FLAGS2 + j] = 1;
            self.ram[SPRITE_FLAGS3 + j] = 1;
            self.ram[SPRITE_OAM_FLAGS + j] = 1;
            self.ram[SPRITE_X_LO + j] = 204;
            self.ram[SPRITE_X_HI + j] = 7;
            self.ram[SPRITE_Y_LO + j] = 50;
            self.ram[SPRITE_Y_HI + j] = 6;
            self.ram[SPRITE_DEFL_BITS + j] = 128;
        }
    }

    pub(super) fn sprite_prep_storyteller(&mut self, k: usize) {
        const ROOMS: [u8; 5] = [0x0e, 0x0e, 0x12, 0x1a, 0x14];
        let mut r = ROOMS
            .iter()
            .position(|&room| room == self.ram[DUNGEON_ROOM_INDEX])
            .map_or(0xff, |idx| idx as u8);
        if r == 0 && self.ram[SPRITE_X_HI + k] & 1 != 0 {
            r = 1;
        }
        self.ram[SPRITE_SUBTYPE2 + k] = r;
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_adults(&mut self, k: usize) {
        const HUMAN_MULTI_TYPES: [u8; 3] = [3, 0xe1, 0x19];
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.ram[SPRITE_SUBTYPE2 + k] = HUMAN_MULTI_TYPES
            .iter()
            .position(|&room| room == self.ram[DUNGEON_ROOM_INDEX])
            .map_or(0xff, |idx| idx as u8);
    }

    pub(super) fn sprite_prep_sage(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        if self.ram[DUNGEON_ROOM_INDEX] == 10 {
            self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
            self.ram[SPRITE_OAM_FLAGS + k] = 11;
        }
    }

    pub(super) fn sprite_prep_kiki(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        if self.ram[SAVE_OW_EVENT_INFO + self.ram[OVERWORLD_SCREEN_INDEX] as usize] & 0x20 != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
    }

    pub(super) fn sprite_prep_locksmith(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        if self.ram[FOLLOWER_INDICATOR] == 9 {
            self.ram[SPRITE_STATE + k] = 0;
            return;
        }
        if self.ram[FOLLOWER_INDICATOR] == 12 {
            self.ram[SPRITE_AI_STATE + k] = 2;
        }
        if self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 0x10 != 0 {
            self.ram[SPRITE_AI_STATE + k] = 4;
        }
    }

    pub(super) fn sprite_prep_sick_kid(&mut self, k: usize) {
        if self.ram[LINK_ITEM_BUG_NET] != 0 {
            self.ram[SPRITE_AI_STATE + k] = 3;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_tektite(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [9, 7];
        const HEALTH: [u8; 2] = [8, 12];
        const BUMP_DAMAGE: [u8; 2] = [3, 5];
        let j = ((self.ram[SPRITE_X_LO + k] >> 4) & 1) as usize;
        self.ram[SPRITE_A + k] = j as u8;
        self.ram[SPRITE_OAM_FLAGS + k] = OAM_FLAGS[j];
        self.ram[SPRITE_HEALTH + k] = HEALTH[j];
        self.ram[SPRITE_BUMP_DAMAGE + k] = BUMP_DAMAGE[j];
        self.sprite_apply_speed_towards_link(k, 16);
        self.ram[SPRITE_Z_VEL + k] = 32;
        self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_chainchomp_bounce(&mut self, k: usize) {
        let mut i = k * 8;
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        for _ in (0..=5).rev() {
            write_le_u16(&mut self.ram, CHAINCHOMP_X_HIST_PREP + i * 2, cur_x);
            write_le_u16(&mut self.ram, CHAINCHOMP_Y_HIST_PREP + i * 2, cur_y);
            i += 1;
        }
        self.ram[SPRITE_A + k] = self.ram[SPRITE_X_LO + k];
        self.ram[SPRITE_B + k] = self.ram[SPRITE_X_HI + k];
        self.ram[SPRITE_C + k] = self.ram[SPRITE_Y_LO + k];
        self.ram[SPRITE_G + k] = self.ram[SPRITE_Y_HI + k];
    }

    pub(super) fn chain_chomp_move_chain(&mut self, k: usize) {
        const MULS: [u8; 6] = [205, 154, 102, 51, 8, 0xbd];

        let x = u16::from(self.ram[SPRITE_A + k]) | (u16::from(self.ram[SPRITE_B + k]) << 8);
        let y = u16::from(self.ram[SPRITE_C + k]) | (u16::from(self.ram[SPRITE_G + k]) << 8);
        let mut pos = k * 8;
        let x2 = read_le_u16(&self.ram, CHAINCHOMP_X_HIST_PREP + pos * 2).wrapping_sub(x);
        let y2 = read_le_u16(&self.ram, CHAINCHOMP_Y_HIST_PREP + pos * 2).wrapping_sub(y);
        pos += 1;

        for _ in (0..=5).rev() {
            let mul = MULS[(pos & 7) - 1];
            let x3 = x.wrapping_add_signed(chain_chomp_one_mult_prep(x2 as u8, mul) as i16);
            let y3 = y.wrapping_add_signed(chain_chomp_one_mult_prep(y2 as u8, mul) as i16);

            let x_addr = CHAINCHOMP_X_HIST_PREP + pos * 2;
            let old_x = read_le_u16(&self.ram, x_addr);
            let dx = old_x.wrapping_sub(x3);
            if dx != 0 {
                let new_x = if sign16(dx) {
                    old_x.wrapping_add(1)
                } else {
                    old_x.wrapping_sub(1)
                };
                write_le_u16(&mut self.ram, x_addr, new_x);
            }

            let y_addr = CHAINCHOMP_Y_HIST_PREP + pos * 2;
            let old_y = read_le_u16(&self.ram, y_addr);
            let dy = old_y.wrapping_sub(y3);
            if dy != 0 {
                let new_y = if sign16(dy) {
                    old_y.wrapping_add(1)
                } else {
                    old_y.wrapping_sub(1)
                };
                write_le_u16(&mut self.ram, y_addr, new_y);
            }

            pos += 1;
        }
    }

    pub(super) fn chain_chomp_handle_leash(&mut self, k: usize) {
        let mut pos = k * 8;
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        write_le_u16(&mut self.ram, CHAINCHOMP_X_HIST_PREP + pos * 2, cur_x);
        write_le_u16(&mut self.ram, CHAINCHOMP_Y_HIST_PREP + pos * 2, cur_y);

        for _ in 0..6 {
            let x_addr = CHAINCHOMP_X_HIST_PREP + pos * 2;
            let next_x_addr = CHAINCHOMP_X_HIST_PREP + (pos + 1) * 2;
            let x = read_le_u16(&self.ram, x_addr);
            let next_x = read_le_u16(&self.ram, next_x_addr);
            let dx = x.wrapping_sub(next_x);
            if !sign16(dx.wrapping_sub(8)) {
                write_le_u16(&mut self.ram, next_x_addr, x.wrapping_sub(8));
            } else if sign16(dx.wrapping_add(8)) {
                write_le_u16(&mut self.ram, next_x_addr, x.wrapping_add(8));
            }

            let y_addr = CHAINCHOMP_Y_HIST_PREP + pos * 2;
            let next_y_addr = CHAINCHOMP_Y_HIST_PREP + (pos + 1) * 2;
            let y = read_le_u16(&self.ram, y_addr);
            let next_y = read_le_u16(&self.ram, next_y_addr);
            let dy = y.wrapping_sub(next_y);
            if !sign16(dy.wrapping_sub(8)) {
                write_le_u16(&mut self.ram, next_y_addr, y.wrapping_sub(8));
            } else if sign16(dy.wrapping_add(8)) {
                write_le_u16(&mut self.ram, next_y_addr, y.wrapping_add(8));
            }

            pos += 1;
        }
    }

    pub(super) fn sprite_prep_big_fairy(&mut self, k: usize) {
        self.ram[SPRITE_Z + k] = 24;
        self.sprite_prep_move_down_8px_right8px(k);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_mrs_sahasrahla(&mut self, k: usize) {
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(8);
        self.sprite_prep_magic_bat(k);
    }

    pub(super) fn sprite_prep_magic_bat(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_fortune_teller(&mut self, k: usize) {
        self.sprite_prep_incr_xy_low8(k);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_fairy_pond(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [10, 2];
        let j = ((self.ram[SPRITE_X_LO + k] >> 4) & 1) as usize;
        self.ram[SPRITE_A + k] = j as u8;
        self.ram[SPRITE_OAM_FLAGS + k] = OAM_FLAGS[j];
    }

    pub(super) fn sprite_prep_hobo(&mut self, k: usize) {
        for _ in 1..=15 {
            self.sprite_prep_hobo_spawn_smoke(k);
        }
        for i in (1..=15).rev() {
            if self.ram[SPRITE_TYPE + i] == 0x2b {
                self.ram[SPRITE_STATE + i] = 0;
            }
        }
        self.sprite_prep_hobo_spawn_fire(k);
        if self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 1 != 0 {
            self.ram[SPRITE_AI_STATE] = 3;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE] = 1;
    }

    pub(super) fn sprite_prep_hobo_spawn_smoke(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            self.ram[SPRITE_SUBTYPE2 + j] = 0;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 0;
        }
    }

    pub(super) fn sprite_prep_hobo_spawn_fire(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_x(j, 0x0194);
            self.sprite_set_y(j, 0x003f);
            self.ram[SPRITE_SUBTYPE2 + j] = 2;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 2;
            self.ram[SPRITE_FLAGS2 + j] = 0;
            self.ram[SPRITE_OAM_FLAGS + j] = (self.ram[SPRITE_OAM_FLAGS + j] & !0x0e) | 2;
        }
    }

    pub(super) fn hobo_spawn_bubble(&mut self, k: usize) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x2b, &mut info);
        if j >= 0 {
            let j_usize = j as usize;
            self.sprite_set_spawned_coordinates(j_usize, &info);
            self.ram[SPRITE_SUBTYPE2 + j_usize] = 1;
            self.ram[SPRITE_Z_VEL + j_usize] = 2;
            self.ram[SPRITE_DELAY_MAIN + j_usize] = 96;
            self.ram[SPRITE_DELAY_AUX1 + j_usize] = 48;
            self.ram[SPRITE_IGNORE_PROJECTILE + j_usize] = 48;
            self.ram[SPRITE_FLAGS2 + j_usize] = 0;
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
            self.ram[SPRITE_SUBTYPE2 + j] = 3;
            self.ram[SPRITE_Z_VEL + j] = 7;
            self.ram[SPRITE_DELAY_MAIN + j] = 96;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 96;
            self.ram[SPRITE_FLAGS2 + j] = 0;
        }
    }

    pub(super) fn sprite_prep_master_sword(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(6);
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(6);
    }

    pub(super) fn sprite_prep_roller_horizontal_right_first(&mut self, k: usize) {
        self.ram[SPRITE_AI_STATE + k] = (!self.ram[SPRITE_X_LO + k] & 16) >> 4;
        if self.ram[SPRITE_AI_STATE + k] != 0 {
            self.ram[SPRITE_FLAGS4 + k] = self.ram[SPRITE_FLAGS4 + k].wrapping_add(1);
        }
        self.ram[SPRITE_D + k] = 0;
    }

    pub(super) fn sprite_prep_roller_left_right(&mut self, k: usize) {
        self.ram[SPRITE_AI_STATE + k] = (!self.ram[SPRITE_X_LO + k] & 16) >> 4;
        if self.ram[SPRITE_AI_STATE + k] != 0 {
            self.ram[SPRITE_FLAGS4 + k] = self.ram[SPRITE_FLAGS4 + k].wrapping_add(1);
        }
        self.ram[SPRITE_D + k] = 1;
    }

    pub(super) fn sprite_prep_roller_vertical_down_first(&mut self, k: usize) {
        self.ram[SPRITE_AI_STATE + k] = (self.ram[SPRITE_Y_LO + k] & 16) >> 4;
        if self.ram[SPRITE_AI_STATE + k] != 0 {
            self.ram[SPRITE_FLAGS4 + k] = self.ram[SPRITE_FLAGS4 + k].wrapping_add(1);
        }
        self.ram[SPRITE_D + k] = 2;
    }

    pub(super) fn sprite_prep_roller_up_down(&mut self, k: usize) {
        self.ram[SPRITE_AI_STATE + k] = (self.ram[SPRITE_Y_LO + k] & 16) >> 4;
        if self.ram[SPRITE_AI_STATE + k] != 0 {
            self.ram[SPRITE_FLAGS4 + k] = self.ram[SPRITE_FLAGS4 + k].wrapping_add(1);
        }
        self.ram[SPRITE_D + k] = 3;
    }

    pub(super) fn sprite_prep_kodongo(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(4);
        self.sprite_set_y(k, self.sprite_get_y(k).wrapping_sub(5));
        self.ram[SPRITE_SUBTYPE + k] = self.ram[SPRITE_SUBTYPE + k].wrapping_sub(1);
    }

    pub(super) fn sprite_prep_spark(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE + k] = self.ram[SPRITE_SUBTYPE + k].wrapping_sub(1);
    }

    pub(super) fn sprite_prep_lost_woods_bird(&mut self, k: usize) {
        self.ram[SPRITE_Z_VEL + k] = (self.get_random_number() & 0x1f).wrapping_sub(0x10);
        self.ram[SPRITE_Z + k] = 64;
        self.sprite_prep_lost_woods_squirrel(k);
    }

    pub(super) fn sprite_prep_lost_woods_squirrel(&mut self, k: usize) {
        self.ram[SPRITE_X_VEL + k] = if self.sprite_is_right_of_link(k).a != 0 {
            (-16i8) as u8
        } else {
            16
        };
        let y_vel = if sign8(self.ram[OVERWORLD_SCROLL_DELTA]) {
            4
        } else {
            (-4i8) as u8
        };
        self.ram[SPRITE_Y_VEL + k] = y_vel;
        self.ram[SPRITE_IGNORE_PROJECTILE + k] = y_vel;
    }

    pub(super) fn sprite_prep_antifairy(&mut self, k: usize) {
        const XVEL: [i8; 2] = [16, -16];
        self.ram[SPRITE_X_VEL + k] = XVEL[((self.ram[SPRITE_X_LO + k] >> 4) & 1) as usize] as u8;
        self.ram[SPRITE_Y_VEL + k] = (-16i8) as u8;
    }

    pub(super) fn sprite_prep_antifairy_circle(&mut self, k: usize) {
        const X: [i16; 3] = [10, 20, 10];
        const Y: [i16; 3] = [-10, 0, 10];
        const XVEL: [i8; 3] = [18, 0, -18];
        const YVEL: [i8; 3] = [0, 18, 0];
        const A: [u8; 3] = [1, 1, 0];
        const B: [u8; 3] = [0, 1, 1];

        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_sub(10));
        self.ram[SPRITE_Y_VEL + k] = (-18i8) as u8;
        self.ram[SPRITE_X_VEL + k] = 0;
        self.ram[SPRITE_A + k] = 0;
        self.ram[SPRITE_B + k] = 0;
        self.ram[TMP_COUNTER] = 2;
        loop {
            let i = self.ram[TMP_COUNTER] as usize;
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x82, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, info.r0_x.wrapping_add(X[i] as u16));
                self.sprite_set_y(j, info.r2_y.wrapping_add(Y[i] as u16));
                self.ram[SPRITE_X_VEL + j] = XVEL[i] as u8;
                self.ram[SPRITE_Y_VEL + j] = YVEL[i] as u8;
                self.ram[SPRITE_A + j] = A[i];
                self.ram[SPRITE_B + j] = B[i];
            }
            self.ram[TMP_COUNTER] = self.ram[TMP_COUNTER].wrapping_sub(1);
            if sign8(self.ram[TMP_COUNTER]) {
                break;
            }
        }
    }

    pub(super) fn sprite_prep_king_zora(&mut self, k: usize) {
        if self.ram[LINK_ITEM_FLIPPERS] != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        } else {
            self.ram[SPRITE_IGNORE_PROJECTILE + k] =
                self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        }
    }

    pub(super) fn sprite_prep_do_nothing_d(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_octorok(&mut self, k: usize) {
        const BUMP_DAMAGE: [u8; 2] = [3, 5];
        const HEALTH: [u8; 2] = [2, 4];
        let j = self.ram[IS_IN_DARK_WORLD_PREP] as usize;
        self.ram[SPRITE_HEALTH + k] = HEALTH[j];
        self.ram[SPRITE_BUMP_DAMAGE + k] = BUMP_DAMAGE[j];
        self.ram[SPRITE_DELAY_MAIN + k] = self.get_random_number() & 127;
    }

    pub(super) fn sprite_prep_swimming_zora(&mut self, k: usize) {
        self.ram[SPRITE_DELAY_MAIN + k] = 64;
        self.sprite_prep_geldman(k);
    }

    pub(super) fn sprite_prep_geldman(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_kyameron(&mut self, k: usize) {
        self.ram[SPRITE_A + k] = self.ram[SPRITE_X_LO + k];
        self.ram[SPRITE_B + k] = self.ram[SPRITE_X_HI + k];
        self.ram[SPRITE_C + k] = self.ram[SPRITE_Y_LO + k];
        self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_Y_HI + k];
    }

    pub(super) fn sprite_prep_walking_zora(&mut self, k: usize) {
        self.ram[SPRITE_DELAY_MAIN + k] = 96;
    }

    pub(super) fn sprite_prep_talking_tree(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
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
            self.ram[SPRITE_HEAD_DIR + j] = dir as u8;
            let x = info.r0_x.wrapping_add(TALKING_TREE_SPAWN_X[dir] as u16);
            let y = info.r2_y.wrapping_sub(11);
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);
            self.ram[SPRITE_A + j] = x as u8;
            self.ram[SPRITE_B + j] = (x >> 8) as u8;
            self.ram[SPRITE_C + j] = y as u8;
            self.ram[SPRITE_E + j] = (y >> 8) as u8;
            self.ram[SPRITE_SUBTYPE2 + j] = 1;
        }
    }

    pub(super) fn sprite_prep_swamola(&mut self, k: usize) {
        self.sprite_prep_swamola_initialize_segments(k);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_swamola_initialize_segments(&mut self, k: usize) {
        const BUGGY_SWAMOLA_LOOKUP: [usize; 6] = [0x1c, 0xa9, 0x03, 0x9d, 0x90, 0x0d];
        let mut j = if self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_MISC_BUG_FIXES_PREP != 0
        {
            k * 32
        } else {
            BUGGY_SWAMOLA_LOOKUP[k]
        };
        for _ in 0..32 {
            self.ram[SWAMOLA_X_LO_PREP + j] = self.ram[SPRITE_X_LO + k];
            self.ram[SWAMOLA_X_HI_PREP + j] = self.ram[SPRITE_X_HI + k];
            self.ram[SWAMOLA_Y_LO_PREP + j] = self.ram[SPRITE_Y_LO + k];
            self.ram[SWAMOLA_Y_HI_PREP + j] = self.ram[SPRITE_Y_HI + k];
            j += 1;
        }
    }

    pub(super) fn sprite_prep_flute_kid(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.ram[SPRITE_SUBTYPE2 + k] = (self.ram[SAVEGAME_IS_DARKWORLD] >> 6) & 1;
        if self.ram[SPRITE_SUBTYPE2 + k] != 0 {
            if self.ram[SRAM_PROGRESS_INDICATOR_3_PREP] & 8 != 0 || self.ram[LINK_ITEM_FLUTE] > 2 {
                self.ram[SPRITE_GRAPHICS + k] = 3;
                self.ram[SPRITE_AI_STATE + k] = 5;
            } else if self.ram[LINK_ITEM_FLUTE] == 2 {
                self.ram[SPRITE_GRAPHICS + k] = 1;
            }
            self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
            self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(8);
        } else if self.ram[LINK_ITEM_FLUTE] >= 2 {
            self.ram[SPRITE_STATE + k] = 0;
        } else {
            self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(7);
        }
    }

    pub(super) fn sprite_prep_move_down_8px(&mut self, k: usize) {
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(8);
    }

    pub(super) fn sprite_prep_zazakku(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_pedestal_plaque(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        if self.ram[OVERWORLD_SCREEN_INDEX] == 48 {
            self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(7);
        }
    }

    pub(super) fn sprite_prep_stalfos(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE + k] = self.ram[SPRITE_X_LO + k] & 16;
        if self.ram[SPRITE_SUBTYPE + k] != 0 {
            self.ram[SPRITE_OAM_FLAGS + k] = 7;
        }
    }

    pub(super) fn sprite_prep_moldorm(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.sprite_initialized_segmented(k);
    }

    pub(super) fn sprite_prep_lanmolas(&mut self, k: usize) {
        const INIT_DELAY: [u8; 3] = [128, 192, 255];
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_DELAY_MAIN + k] = INIT_DELAY[k];
        self.ram[SPRITE_Z + k] = 0xff;
        for i in 0..64 {
            self.ram[BEAMOS_X_HI + k * 0x40 + i] = 0xff;
        }
        self.ram[GARNISH_Y_LO_PREP + k] = 7;
    }

    pub(super) fn sprite_prep_bumper(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_move_down_8px_right8px(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(8);
    }

    pub(super) fn sprite_prep_hardhat_beetle(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [6, 8];
        const HEALTH: [u8; 2] = [32, 6];
        const A: [u8; 2] = [16, 12];
        const STATE: [u8; 2] = [1, 3];
        const FLAGS5: [u8; 2] = [2, 6];
        const BUMP_DAMAGE: [u8; 2] = [5, 3];
        let j = usize::from((self.ram[SPRITE_X_LO + k] & 0x10) != 0);
        self.ram[SPRITE_OAM_FLAGS + k] = OAM_FLAGS[j];
        self.ram[SPRITE_HEALTH + k] = HEALTH[j];
        self.ram[SPRITE_A + k] = A[j];
        self.ram[SPRITE_AI_STATE + k] = STATE[j];
        self.ram[SPRITE_FLAGS5 + k] = FLAGS5[j];
        self.ram[SPRITE_BUMP_DAMAGE + k] = BUMP_DAMAGE[j];
    }

    pub(super) fn sprite_prep_mini_helmasaur(&mut self, k: usize) {
        self.ram[SPRITE_A + k] = 16;
        self.ram[SPRITE_AI_STATE + k] = 1;
    }

    pub(super) fn sprite_prep_fairy(&mut self, k: usize) {
        self.ram[SPRITE_A + k] = self.get_random_number() & 1;
        self.ram[SPRITE_D + k] = self.ram[SPRITE_A + k] ^ 1;
        self.sprite_prep_absorbable(k);
    }

    pub(super) fn sprite_prep_falling_ice(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_armos_knight(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_DELAY_MAIN + k] = 255;
        self.ram[SPRITE_PREP_SHARED_COUNTER] = self.ram[SPRITE_PREP_SHARED_COUNTER].wrapping_add(1);
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_desert_statue(&mut self, k: usize) {
        self.ram[SPRITE_A + k] = self.ram[SPRITE_LIMIT_INSTANCE];
        self.ram[SPRITE_LIMIT_INSTANCE] = self.ram[SPRITE_LIMIT_INSTANCE].wrapping_add(1);
        self.sprite_prep_move_down_8px_right8px(k);
        self.ram[SPRITE_D + k] = if self.ram[SPRITE_X_LO + k] < 0x30 {
            1
        } else if self.ram[SPRITE_X_LO + k] < 0xe0 {
            3
        } else {
            2
        };
    }

    pub(super) fn sprite_prep_big_spike(&mut self, k: usize) {
        self.sprite_prep_move_down_8px_right8px(k);
        self.sprite_prep_kyameron(k);
    }

    pub(super) fn sprite_prep_crystal_switch(&mut self, k: usize) {
        const CRYSTAL_SWITCH_PAL: [u8; 2] = [2, 4];
        self.ram[SPRITE_OAM_FLAGS + k] |=
            CRYSTAL_SWITCH_PAL[(self.ram[ORANGE_BLUE_BARRIER_STATE] & 1) as usize];
    }

    pub(super) fn sprite_prep_kholdstare_shell(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_DELAY_AUX1 + k] = 192;
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_kholdstare(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_AI_STATE + k] = 3;
        self.sprite_prep_ignore_projectiles(k);
        self.sprite_prep_move_down_8px_right8px(k);
    }

    pub(super) fn sprite_prep_agahnim(&mut self, k: usize) {
        const OAM_FLAGS: [u8; 2] = [11, 7];
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_GRAPHICS + k] = 0;
        self.ram[SPRITE_D + k] = 3;
        self.sprite_prep_move_down_8px_right8px(k);
        self.ram[SPRITE_OAM_FLAGS + k] = OAM_FLAGS[self.ram[IS_IN_DARK_WORLD_PREP] as usize];
    }

    pub(super) fn sprite_prep_trinexx(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.trinexx_components_initialize(k);
        for i in (0..=15).rev() {
            self.ram[ALT_SPRITE_STATE_PREP + i] = 0;
        }
    }

    pub(super) fn trinexx_components_initialize(&mut self, k: usize) {
        match self.ram[SPRITE_TYPE + k] {
            0xcb => {
                self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
                self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(16);
                self.trinexx_cache_position(k);
                self.ram[OVERLORD_X_LO + 2] = 0;
                self.ram[OVERLORD_X_LO + 3] = 0;
                self.ram[OVERLORD_X_LO + 5] = 0;
                self.ram[OVERLORD_X_LO + 7] = 0;
                self.ram[OVERLORD_X_HI_PREP] = 0;
                self.ram[OVERLORD_X_LO + 6] = 255;
                self.trinexx_restore_xy(k);
            }
            0xcc => {
                self.ram[SPRITE_GRAPHICS + k] = 3;
                self.ram[SPRITE_DELAY_MAIN + k] = 128;
                self.trinexx_initialize_alt_sprites(k);
            }
            0xcd => {
                self.ram[SPRITE_DELAY_MAIN + k] = 255;
                self.trinexx_initialize_alt_sprites(k);
            }
            _ => {}
        }
    }

    fn trinexx_initialize_alt_sprites(&mut self, k: usize) {
        for j in (0..=0x1a).rev() {
            self.ram[ALT_SPRITE_TYPE_PREP + j] = 0x40;
            self.ram[ALT_SPRITE_X_HI_PREP + j] = 0;
            self.ram[ALT_SPRITE_Y_HI_PREP + j] = 0;
        }
        self.ram[SPRITE_SUBTYPE2 + k] = 1;
        self.trinexx_cache_position(k);
    }

    pub(super) fn sprite_prep_helmasaur_king(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.helmasaur_king_initialize(k);
        for i in 0..16 {
            self.ram[ALT_SPRITE_STATE_PREP + i] = 0;
        }
    }

    pub(super) fn sprite_prep_absorbable(&mut self, k: usize) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            self.ram[SPRITE_E + k] = self.ram[SPRITE_E + k].wrapping_add(1);
            self.ram[SPRITE_IGNORE_PROJECTILE + k] =
                self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        }
    }

    pub(super) fn sprite_prep_overworld_bonk_item(&mut self, k: usize) {
        self.ram[SPRITE_E + k] = self.ram[SPRITE_E + k].wrapping_add(1);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_shield_pickup(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_nice_bee(&mut self, k: usize) {
        let or_bottle = self.ram[LINK_BOTTLE_INFO]
            | self.ram[LINK_BOTTLE_INFO + 1]
            | self.ram[LINK_BOTTLE_INFO + 2]
            | self.ram[LINK_BOTTLE_INFO + 3];
        if or_bottle & 8 != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
        self.ram[SPRITE_E + k] = self.ram[SPRITE_E + k].wrapping_add(1);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_do_nothing_g(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_fire_bar(&mut self, k: usize) {
        self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_add(1);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
    }

    pub(super) fn sprite_prep_spike(&mut self, k: usize) {
        self.ram[SPRITE_X_VEL + k] = 32;
        self.ram[SPRITE_Y_VEL + k] = (-16i8) as u8;
        self.sprite_move_y(k);
        self.ram[SPRITE_Y_VEL + k] = 0;
    }

    pub(super) fn sprite_prep_rock_stal(&mut self, k: usize) {
        self.ram[SPRITE_Y_VEL + k] = (-16i8) as u8;
        self.sprite_move_y(k);
        self.ram[SPRITE_Y_VEL + k] = 0;
    }

    pub(super) fn sprite_prep_blob(&mut self, k: usize) {
        self.ram[SPRITE_GRAPHICS + k] = 4;
        self.sprite_prep_ignore_projectiles(k);
    }

    pub(super) fn sprite_prep_arrghus(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_Z + k] = 24;
    }

    pub(super) fn sprite_prep_arrghi(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_SUBTYPE2 + k] = self.get_random_number();
        if k == 13 {
            self.ram[OVERLORD_X_LO_PREP + 2] = 0;
            self.ram[OVERLORD_X_LO_PREP + 3] = 0;
            self.arrghus_handle_puffs(0);
        }
        self.ram[SPRITE_X_LO + k] = self.ram[OVERLORD_X_LO_PREP + k + 7];
        self.ram[SPRITE_X_HI + k] = self.ram[OVERLORD_Y_LO_PREP + k + 7];
        self.ram[SPRITE_Y_LO + k] = self.ram[OVERLORD_GEN1_PREP + k + 7];
        self.ram[SPRITE_Y_HI + k] = self.ram[OVERLORD_GEN3_PREP + k + 7];
    }

    pub(super) fn arrghus_handle_puffs(&mut self, k: usize) {
        const TAB0: [u16; 13] = [
            0, 0x40, 0x80, 0xc0, 0x100, 0x140, 0x180, 0x1c0, 0, 0x66, 0xcc, 0x132, 0x198,
        ];
        const TAB1: [u16; 13] = [0, 0, 0, 0, 0, 0, 0, 0, 0x1ff, 0x1ff, 0x1ff, 0x1ff, 0x1ff];
        const TAB2: [u8; 13] = [
            0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c,
        ];
        const TAB3: [i8; 52] = [
            0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3, -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5,
            -4, -3, -2, -1, 0, -1, -2, -3, -4, -5, -6, -6, -5, -4, -3, -2, -1, 0, -1, -2, -3, -4,
            -5, -6, -6, -5, -4, -3, -2, -1,
        ];

        let base = read_le_u16(&self.ram, OVERLORD_X_LO_PREP)
            .wrapping_add(self.ram[OVERLORD_X_LO_PREP + 4] as u16);
        write_le_u16(&mut self.ram, OVERLORD_X_LO_PREP, base);

        if self.ram[FRAME_COUNTER] & 3 == 0 {
            self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
            if self.ram[SPRITE_A + k] == 13 {
                self.ram[SPRITE_A + k] = 0;
            }
        }
        if self.ram[FRAME_COUNTER] & 7 == 0 {
            self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_add(1);
            if self.ram[SPRITE_B + k] == 13 {
                self.ram[SPRITE_B + k] = 0;
            }
        }

        let sprite_x = self.sprite_get_x(k) as i32;
        let sprite_y = self.sprite_get_y(k) as i32;
        for i in 0..13 {
            let r0 = base.wrapping_add(TAB0[i]) ^ TAB1[i];
            let r14 = self.ram[OVERLORD_X_LO_PREP + 2].wrapping_add(TAB2[i]);
            let sin_arg = r14.wrapping_add_signed(TAB3[self.ram[SPRITE_A + k] as usize + i]);
            let cos_arg = r14.wrapping_add_signed(TAB3[self.ram[SPRITE_B + k] as usize + i]);
            let sin_val = super::sprite_main_draw::arrgi_sin(r0, sin_arg) as i32;
            let cos_val = super::sprite_main_draw::arrgi_sin(r0.wrapping_add(0x80), cos_arg) as i32;

            let tx = sprite_x + sin_val;
            self.ram[OVERLORD_X_HI_PREP + i] = tx as u8;
            self.ram[OVERLORD_Y_HI_PREP + i] = (tx >> 8) as u8;

            let ty = sprite_y + cos_val - 0x10;
            self.ram[OVERLORD_GEN2_PREP + i] = ty as u8;
            self.ram[OVERLORD_FLOOR_PREP + i] = (ty >> 8) as u8;
        }
        self.ram[TMP_COUNTER] = 13;
    }

    pub(super) fn sprite_prep_mothula(&mut self, k: usize) {
        if self.sprite_return_if_boss_finished(k) {
            return;
        }
        self.ram[SPRITE_DELAY_MAIN + k] = 80;
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.ram[SPRITE_GRAPHICS + k] = 2;
        self.ram[DUNG_FLOOR_MOVE_FLAGS_PREP] = self.ram[DUNG_FLOOR_MOVE_FLAGS_PREP].wrapping_add(1);
        self.ram[SPRITE_C + k] = 112;
    }

    pub(super) fn sprite_prep_do_nothing_h(&mut self, _k: usize) {}

    pub(super) fn heart_upgrade_check_if_already_obtained(&mut self, k: usize) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            let screen = self.ram[OVERWORLD_SCREEN_INDEX] as usize;
            if (screen == 0x3b && self.ram[SAVE_OW_EVENT_INFO + 0x3b] & 0x20 == 0)
                || self.ram[SAVE_OW_EVENT_INFO + screen] & 0x40 != 0
            {
                self.ram[SPRITE_STATE + k] = 0;
            }
        } else {
            let j = self.ram[SPRITE_X_HI + k] & 1;
            let mask = if j != 0 { 0x2000 } else { 0x4000 };
            if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & mask != 0 {
                self.ram[SPRITE_STATE + k] = 0;
            }
        }
    }

    pub(super) fn heart_upgrade_set_obtained_flag(&mut self, k: usize) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            let screen = self.ram[OVERWORLD_SCREEN_INDEX] as usize;
            self.ram[SAVE_OW_EVENT_INFO + screen] |= 0x40;
        } else {
            let mask = if self.ram[SPRITE_X_HI + k] & 1 != 0 {
                0x2000
            } else {
                0x4000
            };
            let bits = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) | mask;
            write_le_u16(&mut self.ram, DUNG_SAVEGAME_STATE_BITS, bits);
        }
    }

    pub(super) fn sprite_prep_heart_container(&mut self, k: usize) {
        self.heart_upgrade_check_if_already_obtained(k);
    }

    pub(super) fn sprite_prep_heart_piece(&mut self, k: usize) {
        self.heart_upgrade_check_if_already_obtained(k);
    }

    pub(super) fn sprite_prep_small_key(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE + k] = 255;
        let j = self.ram[ITEM_DROP_COUNTER];
        self.ram[ITEM_DROP_COUNTER] = self.ram[ITEM_DROP_COUNTER].wrapping_add(1);
        self.ram[SPRITE_DIE_ACTION + k] = j;
    }

    pub(super) fn sprite_prep_key_set_item_drop(&mut self, k: usize) {
        self.ram[SPRITE_DIE_ACTION + k] = self.ram[ITEM_DROP_COUNTER];
        self.ram[ITEM_DROP_COUNTER] = self.ram[ITEM_DROP_COUNTER].wrapping_add(1);
    }

    pub(super) fn sprite_prep_big_key(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
        self.ram[SPRITE_SUBTYPE + k] = 0xff;
        self.sprite_prep_big_key_load_graphics(k);
    }

    pub(super) fn sprite_prep_big_key_load_graphics(&mut self, k: usize) {
        self.DecodeAnimatedSpriteTile_variable(0x22);
        self.sprite_prep_key_set_item_drop(k);
    }

    pub(super) fn sprite_prep_incr_xy_low8(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_add(8);
    }

    pub(super) fn sprite_prep_fake_sword(&mut self, _k: usize) {}

    pub(super) fn sprite_prep_old_man_bounce(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        if self.ram[DUNGEON_ROOM_INDEX] == 0xe4 {
            self.ram[SPRITE_SUBTYPE2 + k] = 2;
            return;
        }
        if self.ram[FOLLOWER_INDICATOR] == 0 {
            if self.ram[LINK_ITEM_MIRROR] == 2 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            self.ram[FOLLOWER_INDICATOR] = 4;
            self.load_follower_graphics();
            self.ram[FOLLOWER_INDICATOR] = 0;
        } else {
            self.ram[SPRITE_STATE + k] = 0;
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
        if self.ram[LINK_SWORD_TYPE] >= 2 {
            self.ram[SPRITE_STATE + k] = 0;
            return;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        let dir = self.sprite_direction_to_face_link(k, None) ^ 3;
        self.ram[SPRITE_D + k] = dir;
        self.ram[SPRITE_HEAD_DIR + k] = dir;

        let follower = self.ram[FOLLOWER_INDICATOR];
        self.ram[FOLLOWER_INDICATOR] = 1;
        self.load_follower_graphics();
        self.ram[FOLLOWER_INDICATOR] = follower;

        if self.ram[DUNGEON_ROOM_INDEX] == 0x12 {
            self.ram[SPRITE_SUBTYPE2 + k] = 2;
            if self.ram[SRAM_PROGRESS_FLAGS] & 4 == 0 {
                self.ram[SPRITE_STATE + k] = 0;
            } else {
                let x = self.sprite_get_x(k).wrapping_add(6);
                let y = self.sprite_get_y(k).wrapping_add(15);
                self.sprite_set_x(k, x);
                self.sprite_set_y(k, y);
                self.ram[SPRITE_FLAGS4 + k] = 3;
            }
        } else {
            self.ram[SPRITE_SUBTYPE2 + k] = 0;
            if self.ram[FOLLOWER_INDICATOR] == 1 || self.ram[SRAM_PROGRESS_FLAGS] & 4 != 0 {
                self.ram[SPRITE_STATE + k] = 0;
            }
        }
    }

    pub(super) fn sprite_prep_medallion_table(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        if self.ram[OVERWORLD_SCREEN_INDEX] != 3 {
            self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
            if self.ram[LINK_ITEM_BOMBOS] != 0 {
                self.ram[SPRITE_GRAPHICS + k] = 4;
                self.ram[SPRITE_AI_STATE + k] = 3;
            }
        } else if self.ram[LINK_ITEM_ETHER] != 0 {
            self.ram[SPRITE_GRAPHICS + k] = 4;
            self.ram[SPRITE_AI_STATE + k] = 3;
        }
    }

    pub(super) fn sprite_prep_eyegore(&mut self, k: usize) {
        let room = self.ram[DUNGEON_ROOM_INDEX2];
        if room == 12 || room == 27 || room == 75 || room == 107 {
            self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_add(1);
            if self.ram[SPRITE_TYPE + k] == 0x83 {
                self.ram[SPRITE_DEFL_BITS + k] = 0;
            }
        }
    }

    fn sprite_return_if_boss_finished(&mut self, k: usize) -> bool {
        if read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 != 0 {
            self.ram[SPRITE_STATE + k] = 0;
            return true;
        }
        for j in (0..16).rev() {
            if K_SPRITE_INIT_BUMP_DAMAGE_PREP[self.ram[SPRITE_TYPE + j] as usize] & 0x10 == 0 {
                self.ram[SPRITE_STATE + j] = 0;
            }
        }
        false
    }

    pub(super) fn sprite_initialized_segmented(&mut self, k: usize) {
        for i in 0..128 {
            self.ram[MOLDORM_X_LO_PREP + i] = self.ram[SPRITE_X_LO + k];
            self.ram[MOLDORM_X_HI_PREP + i] = self.ram[SPRITE_X_HI + k];
            self.ram[MOLDORM_Y_LO_PREP + i] = self.ram[SPRITE_Y_LO + k];
            self.ram[MOLDORM_Y_HI_PREP + i] = self.ram[SPRITE_Y_HI + k];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> Box<ZeldaState> {
        Box::new(ZeldaState::new())
    }

    #[test]
    fn simple_sprite_prep_offsets_and_flags_match_c() {
        let mut s = fresh_state();
        let k = 2;
        s.ram[SPRITE_X_LO + k] = 0xf9;
        s.ram[SPRITE_Y_LO + k] = 0xfb;
        s.sprite_prep_mantle(k);
        assert_eq!(s.ram[SPRITE_X_LO + k], 1);
        assert_eq!(s.ram[SPRITE_Y_LO + k], 0xfe);

        s.sprite_prep_move_down_8px_right8px(k);
        assert_eq!(s.ram[SPRITE_X_LO + k], 9);
        assert_eq!(s.ram[SPRITE_Y_LO + k], 6);

        s.ram[SPRITE_IGNORE_PROJECTILE + k] = 0xff;
        s.sprite_prep_ignore_projectiles(k);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 0);
    }

    #[test]
    fn dark_world_enemy_prep_uses_second_property_row() {
        let mut s = fresh_state();
        let k = 3;
        s.ram[IS_IN_DARK_WORLD_PREP] = 1;
        s.sprite_prep_keese(k);
        assert_eq!(s.ram[SPRITE_BUMP_DAMAGE + k], 0x85);
        assert_eq!(s.ram[SPRITE_HEALTH + k], 4);
        assert_eq!(s.ram[SPRITE_FLAGS5 + k], 7);

        s.sprite_prep_rope(k);
        assert_eq!(s.ram[SPRITE_BUMP_DAMAGE + k], 5);
        assert_eq!(s.ram[SPRITE_HEALTH + k], 8);
        assert_eq!(s.ram[SPRITE_FLAGS5 + k], 7);
    }

    #[test]
    fn position_snapshot_prep_copies_low_high_coords() {
        let mut s = fresh_state();
        let k = 4;
        s.ram[SPRITE_X_LO + k] = 0x12;
        s.ram[SPRITE_X_HI + k] = 0x01;
        s.ram[SPRITE_Y_LO + k] = 0x34;
        s.ram[SPRITE_Y_HI + k] = 0x02;
        s.sprite_prep_kyameron(k);
        assert_eq!(s.ram[SPRITE_A + k], 0x12);
        assert_eq!(s.ram[SPRITE_B + k], 0x01);
        assert_eq!(s.ram[SPRITE_C + k], 0x34);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 0x02);
    }

    #[test]
    fn key_prep_consumes_item_drop_counter() {
        let mut s = fresh_state();
        let k = 5;
        s.ram[ITEM_DROP_COUNTER] = 0x7e;
        s.sprite_prep_small_key(k);
        assert_eq!(s.ram[SPRITE_SUBTYPE + k], 0xff);
        assert_eq!(s.ram[SPRITE_DIE_ACTION + k], 0x7e);
        assert_eq!(s.ram[ITEM_DROP_COUNTER], 0x7f);

        s.sprite_prep_key_set_item_drop(k);
        assert_eq!(s.ram[SPRITE_DIE_ACTION + k], 0x7f);
        assert_eq!(s.ram[ITEM_DROP_COUNTER], 0x80);
    }

    #[test]
    fn flute_kid_prep_handles_light_and_dark_world_branches() {
        let mut light = fresh_state();
        let k = 6;
        light.ram[LINK_ITEM_FLUTE] = 2;
        light.ram[SPRITE_STATE + k] = 9;
        light.sprite_prep_flute_kid(k);
        assert_eq!(light.ram[SPRITE_STATE + k], 0);

        let mut dark = fresh_state();
        dark.ram[SAVEGAME_IS_DARKWORLD] = 0x40;
        dark.ram[SRAM_PROGRESS_INDICATOR_3_PREP] = 8;
        dark.ram[SPRITE_X_LO + k] = 10;
        dark.ram[SPRITE_Y_LO + k] = 20;
        dark.sprite_prep_flute_kid(k);
        assert_eq!(dark.ram[SPRITE_SUBTYPE2 + k], 1);
        assert_eq!(dark.ram[SPRITE_GRAPHICS + k], 3);
        assert_eq!(dark.ram[SPRITE_AI_STATE + k], 5);
        assert_eq!(dark.ram[SPRITE_X_LO + k], 18);
        assert_eq!(dark.ram[SPRITE_Y_LO + k], 12);
    }

    #[test]
    fn return_if_boss_finished_clears_non_boss_sprites_or_self_when_finished() {
        let mut s = fresh_state();
        for k in 0..16 {
            s.ram[SPRITE_STATE + k] = 9;
            s.ram[SPRITE_TYPE + k] = 0;
        }
        s.ram[SPRITE_TYPE + 3] = 9; // bump damage 0x13 keeps state.
        assert!(!s.sprite_return_if_boss_finished(2));
        assert_eq!(s.ram[SPRITE_STATE + 0], 0);
        assert_eq!(s.ram[SPRITE_STATE + 3], 9);

        let mut finished = fresh_state();
        finished.ram[SPRITE_STATE + 2] = 9;
        write_le_u16(&mut finished.ram, DUNG_SAVEGAME_STATE_BITS, 0x8000);
        assert!(finished.sprite_return_if_boss_finished(2));
        assert_eq!(finished.ram[SPRITE_STATE + 2], 0);
    }

    #[test]
    fn room_lookup_prep_sets_subtype_and_ignore_projectile() {
        let mut s = fresh_state();
        let k = 7;
        s.ram[DUNGEON_ROOM_INDEX] = 0x12;
        s.sprite_prep_storyteller(k);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + k], 2);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);

        s.ram[DUNGEON_ROOM_INDEX] = 0x03;
        s.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
        s.sprite_prep_adults(k);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + k], 0);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
    }

    #[test]
    fn rupee_pull_and_grate_shift_x_16_bit_left() {
        let mut s = fresh_state();
        let k = 8;
        s.sprite_set_x(k, 0x0104);
        s.sprite_prep_rupee_pull(k);
        assert_eq!(s.sprite_get_x(k), 0x00fc);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);

        s.sprite_set_x(k, 0x0004);
        s.ram[SAVE_OW_EVENT_INFO + 0x58] = 0x20;
        s.ram[SPRITE_STATE + k] = 9;
        s.sprite_prep_thieves_town_grate(k);
        assert_eq!(s.sprite_get_x(k), 0xfffc);
        assert_eq!(s.ram[SPRITE_STATE + k], 0);
    }

    #[test]
    fn laser_eye_prep_matches_orientation_branches() {
        let mut s = fresh_state();
        let k = 9;
        s.ram[SPRITE_TYPE + k] = 0x96;
        s.ram[SPRITE_X_LO + k] = 0x20;
        s.ram[SPRITE_Y_LO + k] = 0;
        s.sprite_prep_laser_eye_bounce(k);
        assert_eq!(s.ram[SPRITE_D + k], 1);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 0);
        assert_eq!(s.ram[SPRITE_X_LO + k], 0x28);

        s.ram[SPRITE_TYPE + k] = 0x97;
        s.ram[SPRITE_X_LO + k] = 0x08;
        s.ram[SPRITE_Y_LO + k] = 0x20;
        s.sprite_prep_laser_eye_bounce(k);
        assert_eq!(s.ram[SPRITE_D + k], 2);
        assert_eq!(s.ram[SPRITE_X_LO + k], 0x10);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 0);
        assert_eq!(s.ram[SPRITE_Y_LO + k], 0x18);

        let mut beam = fresh_state();
        beam.ram[SPRITE_STATE + k] = 9;
        beam.sprite_set_x(k, 0x0200);
        beam.sprite_set_y(k, 0x0100);
        beam.ram[SPRITE_D + k] = 0;
        beam.ram[LINK_SHIELD_TYPE] = 3;
        beam.laser_eye_fire_beam(k);
        assert_eq!(beam.ram[SPRITE_TYPE + 15], 0x95);
        assert_eq!(beam.ram[SPRITE_GRAPHICS + 15], 0);
        assert_eq!(beam.sprite_get_x(15), 0x020c);
        assert_eq!(beam.sprite_get_y(15), 0x0104);
        assert_eq!(beam.ram[SPRITE_X_VEL + 15], 112);
        assert_eq!(beam.ram[SPRITE_Y_VEL + 15], 0);
        assert_eq!(beam.ram[SPRITE_FLAGS2 + 15], 0x20);
        assert_eq!(beam.ram[SPRITE_A + 15], 0x20);
        assert_eq!(beam.ram[SPRITE_OAM_FLAGS + 15], 5);
        assert_eq!(beam.ram[SPRITE_DEFL_BITS + 15], 0x48);
        assert_eq!(beam.ram[SPRITE_IGNORE_PROJECTILE + 15], 0x48);
        assert_eq!(beam.ram[SPRITE_DELAY_MAIN + 15], 5);
        assert_eq!(beam.ram[SPRITE_FLAGS5 + 15], 32);
        assert_eq!(beam.ram[SOUND_EFFECT_2] & 0x3f, 0x19);

        let mut ganon_pos = fresh_state();
        ganon_pos.ram[SPRITE_D] = 1;
        ganon_pos.ram[OVERLORD_X_HI_PREP + k] = 0x80;
        ganon_pos.ram[OVERLORD_Y_HI_PREP + k] = 0x02;
        ganon_pos.ram[OVERLORD_GEN2_PREP + k] = 0x40;
        ganon_pos.ram[OVERLORD_FLOOR_PREP + k] = 0x03;
        ganon_pos.get_position_relative_to_the_great_overlord_ganon(k);
        assert_eq!(ganon_pos.sprite_get_x(k), 0x026e);
        assert_eq!(ganon_pos.sprite_get_y(k), 0x032c);

        let mut beamos = fresh_state();
        beamos.ram[SPRITE_STATE + k] = 9;
        beamos.sprite_set_x(k, 0x0100);
        beamos.sprite_set_y(k, 0x0200);
        beamos.ram[DUNGMAP_VAR7] = 4;
        beamos.ram[DUNGMAP_VAR7 + 1] = (-4i8) as u8;
        beamos.ram[SPRITE_LIMIT_INSTANCE] = 2;
        write_le_u16(&mut beamos.ram, LINK_X_COORD, 0x0124);
        write_le_u16(&mut beamos.ram, LINK_Y_COORD, 0x01f4);
        beamos.beamos_fire_laser(k);
        assert_eq!(beamos.ram[SPRITE_TYPE + 15], 0x61);
        assert_eq!(beamos.sprite_get_x(15), 0x0104);
        assert_eq!(beamos.sprite_get_y(15), 0x01fc);
        assert_eq!(beamos.ram[SPRITE_X_VEL + 15], 0x20);
        assert_eq!(beamos.ram[SPRITE_Y_VEL + 15], 0);
        assert_eq!(beamos.ram[SPRITE_FLAGS2 + 15], 0x3f);
        assert_eq!(beamos.ram[SPRITE_FLAGS4 + 15], 0x54);
        assert_eq!(beamos.ram[SPRITE_C + 15], 1);
        assert_eq!(beamos.ram[SPRITE_DEFL_BITS + 15], 0x48);
        assert_eq!(beamos.ram[SPRITE_OAM_FLAGS + 15], 3);
        assert_eq!(beamos.ram[SPRITE_BUMP_DAMAGE + 15], 4);
        assert_eq!(beamos.ram[SPRITE_DELAY_AUX1 + 15], 12);
        assert_eq!(beamos.ram[SPRITE_GRAPHICS + 15], 2);
        assert_eq!(beamos.ram[SPRITE_LIMIT_INSTANCE], 3);
        assert_eq!(beamos.ram[SOUND_EFFECT_2] & 0x3f, 0x19);
        let history = 2 * 32;
        assert_eq!(beamos.ram[BEAMOS_X_LO_PREP + history], 0x04);
        assert_eq!(beamos.ram[BEAMOS_X_HI + history], 0x01);
        assert_eq!(beamos.ram[BEAMOS_Y_LO_PREP + history], 0xfc);
        assert_eq!(beamos.ram[BEAMOS_Y_HI_PREP + history], 0x01);
        assert_eq!(beamos.ram[BEAMOS_X_LO_PREP + history + 31], 0x04);

        let mut beamos_limited = fresh_state();
        beamos_limited.ram[SPRITE_LIMIT_INSTANCE] = 4;
        beamos_limited.beamos_fire_laser(k);
        assert_eq!(beamos_limited.ram[SPRITE_LIMIT_INSTANCE], 4);
        assert_eq!(beamos_limited.ram[SPRITE_TYPE + 15], 0);
    }

    #[test]
    fn boss_gated_prep_sets_expected_state_when_unfinished() {
        let mut s = fresh_state();
        let k = 10;
        s.ram[SPRITE_X_LO + k] = 0x20;
        s.ram[SPRITE_Y_LO + k] = 0x30;
        s.ram[IS_IN_DARK_WORLD_PREP] = 1;
        s.sprite_prep_agahnim(k);
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 0);
        assert_eq!(s.ram[SPRITE_D + k], 3);
        assert_eq!(s.ram[SPRITE_OAM_FLAGS + k], 7);
        assert_eq!(s.ram[SPRITE_X_LO + k], 0x28);
        assert_eq!(s.ram[SPRITE_Y_LO + k], 0x38);

        s.sprite_prep_kholdstare(k);
        assert_eq!(s.ram[SPRITE_AI_STATE + k], 3);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
    }

    #[test]
    fn armos_desert_and_big_spike_prep_update_state() {
        let mut s = fresh_state();
        let k = 11;
        s.ram[SPRITE_X_LO + k] = 0x2f;
        s.ram[SPRITE_Y_LO + k] = 0x40;
        s.ram[SPRITE_LIMIT_INSTANCE] = 5;
        s.sprite_prep_desert_statue(k);
        assert_eq!(s.ram[SPRITE_A + k], 5);
        assert_eq!(s.ram[SPRITE_LIMIT_INSTANCE], 6);
        assert_eq!(s.ram[SPRITE_D + k], 3); // after +8, x is now 0x37.

        s.sprite_prep_armos_knight(k);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 255);
        assert_eq!(s.ram[SPRITE_PREP_SHARED_COUNTER], 1);

        s.ram[SPRITE_X_LO + k] = 0x10;
        s.ram[SPRITE_X_HI + k] = 1;
        s.ram[SPRITE_Y_LO + k] = 0x20;
        s.ram[SPRITE_Y_HI + k] = 2;
        s.sprite_prep_big_spike(k);
        assert_eq!(s.ram[SPRITE_A + k], 0x18);
        assert_eq!(s.ram[SPRITE_B + k], 1);
        assert_eq!(s.ram[SPRITE_C + k], 0x28);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 2);
    }

    #[test]
    fn barrier_catfish_and_mini_vitreous_prep_match_simple_branches() {
        let mut s = fresh_state();
        let k = 12;
        s.ram[OVERWORLD_SCREEN_INDEX] = 5;
        s.ram[SAVE_OW_EVENT_INFO + 5] = 0x40;
        s.ram[SPRITE_X_LO + k] = 0x10;
        s.ram[SPRITE_Y_LO + k] = 0x20;
        s.sprite_prep_agahnims_barrier(k);
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 4);
        assert_eq!(s.ram[SPRITE_X_LO + k], 0x18);
        assert_eq!(s.ram[SPRITE_Y_LO + k], 0x1c);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);

        s.ram[SPRITE_X_LO + k] = 0x30;
        s.ram[SPRITE_Y_LO + k] = 0x40;
        s.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
        s.sprite_prep_catfish(k);
        assert_eq!(s.ram[SPRITE_X_LO + k], 0x38);
        assert_eq!(s.ram[SPRITE_Y_LO + k], 0x3c);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);

        s.ram[SPRITE_STATE + k] = 9;
        write_le_u16(&mut s.ram, DUNG_SAVEGAME_STATE_BITS, 0x8000);
        s.sprite_prep_mini_vitreous(k);
        assert_eq!(s.ram[SPRITE_STATE + k], 0);

        let mut cutscene = fresh_state();
        cutscene.ram[SPRITE_STATE + k] = 9;
        cutscene.sprite_set_x(k, 0x0100);
        cutscene.sprite_set_y(k, 0x0200);
        cutscene.sprite_prep_cutscene_agahnim(k);
        assert_eq!(cutscene.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(cutscene.sprite_get_x(k), 0x0108);
        assert_eq!(cutscene.sprite_get_y(k), 0x0206);
        assert_eq!(cutscene.ram[SPRITE_TYPE + 15], 0xc1);
        assert_eq!(cutscene.ram[SPRITE_A + 15], 1);
        assert_eq!(cutscene.ram[SPRITE_IGNORE_PROJECTILE + 15], 1);
        assert_eq!(cutscene.sprite_get_x(15), 0x0108);
        assert_eq!(cutscene.ram[SPRITE_Y_HI + 15], 0x02);
        assert_eq!(cutscene.ram[SPRITE_Y_LO + 15], 0x2e);
        assert_eq!(cutscene.ram[SPRITE_FLAGS2 + 15], 0);
        assert_eq!(cutscene.ram[SPRITE_OAM_FLAGS + 15], 12);

        let mut cutscene_done = fresh_state();
        cutscene_done.ram[SPRITE_STATE + k] = 9;
        write_le_u16(&mut cutscene_done.ram, DUNG_SAVEGAME_STATE_BITS, 0x4000);
        cutscene_done.sprite_prep_cutscene_agahnim(k);
        assert_eq!(cutscene_done.ram[SPRITE_STATE + k], 0);
        assert_eq!(cutscene_done.ram[SPRITE_STATE + 15], 0);
    }

    #[test]
    fn ganon_helmasaur_and_trinexx_prep_call_existing_initializers() {
        let mut ganon = fresh_state();
        let k = 13;
        ganon.ram[SPRITE_D + k] = 1;
        ganon.sprite_prep_ganon(k);
        assert_eq!(ganon.ram[SPRITE_DELAY_MAIN + k], 128);
        assert_eq!(ganon.ram[SPRITE_ROOM + k], 2);
        assert_eq!(ganon.ram[MUSIC_CONTROL], 0x1e);

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
        trinexx_body.ram[SPRITE_TYPE + k] = 0xcb;
        trinexx_body.ram[SPRITE_X_LO + k] = 0x20;
        trinexx_body.ram[SPRITE_X_HI + k] = 1;
        trinexx_body.ram[SPRITE_Y_LO + k] = 0x30;
        trinexx_body.ram[SPRITE_Y_HI + k] = 2;
        trinexx_body.ram[ALT_SPRITE_STATE_PREP + 3] = 0xaa;
        trinexx_body.sprite_prep_trinexx(k);
        assert_eq!(trinexx_body.ram[SPRITE_A + k], 0x28);
        assert_eq!(trinexx_body.ram[SPRITE_B + k], 1);
        assert_eq!(trinexx_body.ram[SPRITE_C + k], 0x40);
        assert_eq!(trinexx_body.ram[SPRITE_G + k], 2);
        assert_eq!(trinexx_body.sprite_get_x(k), 0x0128);
        assert_eq!(trinexx_body.sprite_get_y(k), 0x024c);
        assert_eq!(trinexx_body.ram[OVERLORD_X_LO + 2], 0);
        assert_eq!(trinexx_body.ram[OVERLORD_X_LO + 6], 255);
        assert_eq!(trinexx_body.ram[OVERLORD_X_HI_PREP], 0);
        assert_eq!(trinexx_body.ram[ALT_SPRITE_STATE_PREP + 3], 0);

        let mut trinexx_head = fresh_state();
        trinexx_head.ram[SPRITE_TYPE + k] = 0xcc;
        trinexx_head.ram[SPRITE_X_LO + k] = 0x44;
        trinexx_head.ram[SPRITE_X_HI + k] = 3;
        trinexx_head.ram[SPRITE_Y_LO + k] = 0x55;
        trinexx_head.ram[SPRITE_Y_HI + k] = 4;
        trinexx_head.ram[ALT_SPRITE_TYPE_PREP + 0x1a] = 0;
        trinexx_head.ram[ALT_SPRITE_X_HI_PREP + 0x1a] = 0xff;
        trinexx_head.ram[ALT_SPRITE_Y_HI_PREP + 0x1a] = 0xff;
        trinexx_head.sprite_prep_trinexx(k);
        assert_eq!(trinexx_head.ram[SPRITE_GRAPHICS + k], 3);
        assert_eq!(trinexx_head.ram[SPRITE_DELAY_MAIN + k], 128);
        assert_eq!(trinexx_head.ram[SPRITE_SUBTYPE2 + k], 1);
        assert_eq!(trinexx_head.ram[ALT_SPRITE_TYPE_PREP + 0x1a], 0x40);
        assert_eq!(trinexx_head.ram[ALT_SPRITE_X_HI_PREP + 0x1a], 0);
        assert_eq!(trinexx_head.ram[ALT_SPRITE_Y_HI_PREP + 0x1a], 0);
        assert_eq!(trinexx_head.ram[SPRITE_A + k], 0x44);
        assert_eq!(trinexx_head.ram[SPRITE_G + k], 4);
    }

    #[test]
    fn moldorm_and_chainchomp_history_buffers_are_seeded_from_sprite_position() {
        let mut s = fresh_state();
        let k = 2;
        s.ram[SPRITE_X_LO + k] = 0x44;
        s.ram[SPRITE_X_HI + k] = 0x01;
        s.ram[SPRITE_Y_LO + k] = 0x55;
        s.ram[SPRITE_Y_HI + k] = 0x02;
        s.sprite_prep_mini_moldorm_bounce(k);
        let base = 32 * k;
        assert_eq!(s.ram[MOLDORM_X_LO_PREP + base], 0x44);
        assert_eq!(s.ram[MOLDORM_X_HI_PREP + base + 31], 0x01);
        assert_eq!(s.ram[MOLDORM_Y_LO_PREP + base + 15], 0x55);
        assert_eq!(s.ram[MOLDORM_Y_HI_PREP + base + 31], 0x02);

        write_le_u16(&mut s.ram, CUR_SPRITE_X, 0x1234);
        write_le_u16(&mut s.ram, CUR_SPRITE_Y, 0x5678);
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
        assert_eq!(s.ram[SPRITE_A + k], 0x44);
        assert_eq!(s.ram[SPRITE_G + k], 0x02);

        let mut leash = fresh_state();
        write_le_u16(&mut leash.ram, CUR_SPRITE_X, 0x0100);
        write_le_u16(&mut leash.ram, CUR_SPRITE_Y, 0x0200);
        write_le_u16(
            &mut leash.ram,
            CHAINCHOMP_X_HIST_PREP + (hist + 1) * 2,
            0x0120,
        );
        write_le_u16(
            &mut leash.ram,
            CHAINCHOMP_Y_HIST_PREP + (hist + 1) * 2,
            0x01e0,
        );
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
        moving_chain.ram[SPRITE_A + k] = 0x00;
        moving_chain.ram[SPRITE_B + k] = 0x01;
        moving_chain.ram[SPRITE_C + k] = 0x00;
        moving_chain.ram[SPRITE_G + k] = 0x02;
        write_le_u16(
            &mut moving_chain.ram,
            CHAINCHOMP_X_HIST_PREP + hist * 2,
            0x0110,
        );
        write_le_u16(
            &mut moving_chain.ram,
            CHAINCHOMP_Y_HIST_PREP + hist * 2,
            0x0220,
        );
        write_le_u16(
            &mut moving_chain.ram,
            CHAINCHOMP_X_HIST_PREP + (hist + 1) * 2,
            0x0100,
        );
        write_le_u16(
            &mut moving_chain.ram,
            CHAINCHOMP_Y_HIST_PREP + (hist + 1) * 2,
            0x0230,
        );
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
        assert_eq!(outdoor.ram[SPRITE_GRAPHICS + k], 2);

        let mut indoor = fresh_state();
        indoor.ram[PLAYER_IS_INDOORS] = 1;
        indoor.ram[ITEM_DROP_COUNTER] = 1;
        indoor.ram[SPRITE_GRAPHICS + k] = 4;
        write_le_u16(&mut indoor.ram, DUNG_SAVEGAME_STATE_BITS, 0x2000);
        indoor.ram[SPRITE_STATE + k] = 9;
        indoor.sprite_prep_bonk_item(k);
        assert_eq!(indoor.ram[SPRITE_FLOOR + k], 2);
        assert_eq!(indoor.ram[SPRITE_DIE_ACTION + k], 1);
        assert_eq!(indoor.ram[SPRITE_STATE + k], 0);
        assert_eq!(indoor.ram[SPRITE_GRAPHICS + k], 5);
        assert_eq!(indoor.ram[SPRITE_OAM_FLAGS + k], 8);
        assert_eq!(indoor.ram[SPRITE_FLAGS3 + k] & 0x20, 0x20);

        let mut key = fresh_state();
        key.ram[SPRITE_X_LO + k] = 0x20;
        key.ram[ITEM_DROP_COUNTER] = 7;
        key.sprite_prep_big_key(k);
        assert_eq!(key.ram[SPRITE_X_LO + k], 0x28);
        assert_eq!(key.ram[SPRITE_SUBTYPE + k], 0xff);
        assert_eq!(key.ram[SPRITE_DIE_ACTION + k], 7);
        assert_eq!(key.ram[ITEM_DROP_COUNTER], 8);

        let mut chest = fresh_state();
        chest.ram[SRAM_PROGRESS_INDICATOR_3_PREP] = 32;
        chest.sprite_prep_purple_chest(k);
        assert_eq!(chest.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        chest.ram[FOLLOWER_INDICATOR] = 12;
        chest.ram[SPRITE_STATE + k] = 9;
        chest.sprite_prep_purple_chest(k);
        assert_eq!(chest.ram[SPRITE_STATE + k], 0);
    }

    #[test]
    fn smithy_prep_matches_world_and_progress_gates() {
        let k = 6;

        let mut dark_waiting = fresh_state();
        dark_waiting.ram[SAVEGAME_IS_DARKWORLD] = 0x40;
        dark_waiting.ram[SPRITE_STATE + k] = 9;
        dark_waiting.sprite_prep_smithy(k);
        assert_eq!(dark_waiting.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(dark_waiting.ram[SPRITE_SUBTYPE2 + k], 2);
        assert_eq!(dark_waiting.ram[SPRITE_STATE + k], 9);

        let mut dark_done = fresh_state();
        dark_done.ram[SAVEGAME_IS_DARKWORLD] = 0x40;
        dark_done.ram[SRAM_PROGRESS_INDICATOR_3_PREP] = 32;
        dark_done.ram[SPRITE_STATE + k] = 9;
        dark_done.sprite_prep_smithy(k);
        assert_eq!(dark_done.ram[SPRITE_STATE + k], 0);

        let mut light_alone = fresh_state();
        light_alone.ram[SPRITE_STATE + k] = 9;
        light_alone.sprite_set_x(k, 0x0100);
        light_alone.sprite_set_y(k, 0x0200);
        light_alone.sprite_prep_smithy(k);
        assert_eq!(light_alone.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(light_alone.sprite_get_x(k), 0x0102);
        assert_eq!(light_alone.sprite_get_y(k), 0x02fd);
        assert_eq!(light_alone.ram[SPRITE_TYPE + 15], 0x31);
        assert_eq!(light_alone.sprite_get_x(15), 0x0100);
        assert_eq!(light_alone.sprite_get_y(15), 0x0200);
        assert_eq!(light_alone.ram[SPRITE_SUBTYPE2 + 15], 1);
        assert_eq!(light_alone.ram[SPRITE_IGNORE_PROJECTILE + 15], 1);

        let mut light_reunited = fresh_state();
        light_reunited.ram[SPRITE_STATE + k] = 9;
        light_reunited.ram[SRAM_PROGRESS_INDICATOR_3_PREP] = 0xa0;
        light_reunited.sprite_set_x(k, 0x0100);
        light_reunited.sprite_set_y(k, 0x0200);
        light_reunited.sprite_prep_smithy(k);
        assert_eq!(light_reunited.ram[SPRITE_TYPE + 15], 0x31);
        assert_eq!(light_reunited.ram[SPRITE_TYPE + 14], 0x1a);
        assert_eq!(light_reunited.sprite_get_x(14), 0x012e);
        assert_eq!(light_reunited.sprite_get_y(14), 0x02fd);
        assert_eq!(light_reunited.ram[SPRITE_D + 14], 1);
        assert_eq!(light_reunited.ram[SPRITE_A + 14], 4);
        assert_eq!(light_reunited.ram[SPRITE_IGNORE_PROJECTILE + 14], 4);
        assert_eq!(light_reunited.ram[SPRITE_TYPE + 13], 0x31);
        assert_eq!(light_reunited.sprite_get_x(13), 0x012e);
        assert_eq!(light_reunited.sprite_get_y(13), 0x02fd);
        assert_eq!(light_reunited.ram[SPRITE_E + 14], k as u8);
        assert_eq!(light_reunited.ram[SPRITE_E + k], 14);
        assert_eq!(light_reunited.ram[SPRITE_AI_STATE + k], 5);
        assert_eq!(light_reunited.ram[SPRITE_AI_STATE + 14], 5);
    }

    #[test]
    fn lanmolas_moldorm_and_tektite_prep_initialize_state() {
        let mut s = fresh_state();
        let k = 1;
        s.ram[SPRITE_X_LO + k] = 0x66;
        s.ram[SPRITE_X_HI + k] = 0x03;
        s.ram[SPRITE_Y_LO + k] = 0x77;
        s.ram[SPRITE_Y_HI + k] = 0x04;
        s.sprite_prep_moldorm(k);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(s.ram[MOLDORM_X_LO_PREP], 0x66);
        assert_eq!(s.ram[MOLDORM_Y_HI_PREP + 127], 0x04);

        let mut lanmolas = fresh_state();
        let k = 2;
        lanmolas.sprite_prep_lanmolas(k);
        assert_eq!(lanmolas.ram[SPRITE_DELAY_MAIN + k], 255);
        assert_eq!(lanmolas.ram[SPRITE_Z + k], 0xff);
        assert_eq!(lanmolas.ram[BEAMOS_X_HI + k * 0x40], 0xff);
        assert_eq!(lanmolas.ram[BEAMOS_X_HI + k * 0x40 + 63], 0xff);
        assert_eq!(lanmolas.ram[GARNISH_Y_LO_PREP + k], 7);

        let mut shrapnel = fresh_state();
        shrapnel.ram[SPRITE_STATE + k] = 9;
        shrapnel.sprite_set_x(k, 0x0120);
        shrapnel.sprite_set_y(k, 0x0340);
        shrapnel.lanmola_spawn_shrapnel(k);
        assert_eq!(shrapnel.ram[TMP_COUNTER], 0xff);
        assert_eq!(shrapnel.ram[SPRITE_TYPE + 15], 0xc2);
        assert_eq!(shrapnel.sprite_get_x(15), 0x0124);
        assert_eq!(shrapnel.sprite_get_y(15), 0x0344);
        assert_eq!(shrapnel.ram[SPRITE_IGNORE_PROJECTILE + 15], 1);
        assert_eq!(shrapnel.ram[SPRITE_BUMP_DAMAGE + 15], 1);
        assert_eq!(shrapnel.ram[SPRITE_FLAGS4 + 15], 1);
        assert_eq!(shrapnel.ram[SPRITE_Z + 15], 0);
        assert_eq!(shrapnel.ram[SPRITE_FLAGS2 + 15], 0x20);
        assert_eq!(shrapnel.ram[SPRITE_X_VEL + 15], 0);
        assert_eq!(shrapnel.ram[SPRITE_Y_VEL + 15], (-36i8) as u8);
        assert_eq!(shrapnel.ram[SPRITE_GRAPHICS + 15], 0);
        assert_eq!(shrapnel.ram[SPRITE_TYPE + 8], 0xc2);
        assert_eq!(shrapnel.ram[SPRITE_X_VEL + 8], (-28i8) as u8);
        assert_eq!(shrapnel.ram[SPRITE_Y_VEL + 8], 28);

        let mut short_shrapnel = fresh_state();
        short_shrapnel.ram[SPRITE_STATE + 0] = 9;
        short_shrapnel.ram[SPRITE_STATE + 1] = 9;
        short_shrapnel.ram[SPRITE_STATE + 2] = 9;
        short_shrapnel.ram[SPRITE_STATE + k] = 9;
        short_shrapnel.sprite_set_x(k, 0x0050);
        short_shrapnel.sprite_set_y(k, 0x0060);
        short_shrapnel.lanmola_spawn_shrapnel(k);
        assert_eq!(short_shrapnel.ram[SPRITE_TYPE + 15], 0xc2);
        assert_eq!(short_shrapnel.ram[SPRITE_X_VEL + 15], 28);
        assert_eq!(short_shrapnel.ram[SPRITE_Y_VEL + 15], (-28i8) as u8);
        assert_eq!(short_shrapnel.ram[SPRITE_TYPE + 12], 0xc2);
        assert_eq!(short_shrapnel.ram[SPRITE_TYPE + 11], 0);

        let mut tektite = fresh_state();
        let k = 4;
        tektite.ram[SPRITE_X_LO + k] = 0x10;
        tektite.sprite_prep_tektite(k);
        assert_eq!(tektite.ram[SPRITE_A + k], 1);
        assert_eq!(tektite.ram[SPRITE_OAM_FLAGS + k], 7);
        assert_eq!(tektite.ram[SPRITE_HEALTH + k], 12);
        assert_eq!(tektite.ram[SPRITE_BUMP_DAMAGE + k], 5);
        assert_eq!(tektite.ram[SPRITE_Z_VEL + k], 32);
        assert_eq!(tektite.ram[SPRITE_AI_STATE + k], 1);
    }

    #[test]
    fn snitch_running_man_and_mushroom_prep_match_simple_gates() {
        let k = 6;

        let mut snitch = fresh_state();
        snitch.ram[SPRITE_X_LO + k] = 0x34;
        snitch.ram[SPRITE_X_HI + k] = 0x12;
        snitch.sprite_prep_snitches(k);
        assert_eq!(snitch.ram[SPRITE_D + k], 2);
        assert_eq!(snitch.ram[SPRITE_HEAD_DIR + k], 2);
        assert_eq!(snitch.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(snitch.ram[SPRITE_A + k], 0x34);
        assert_eq!(snitch.ram[SPRITE_B + k], 0x12);
        assert_eq!(snitch.ram[SPRITE_X_VEL + k], (-9i8) as u8);

        let mut bounce = fresh_state();
        bounce.ram[SPRITE_X_LO + k] = 0x55;
        bounce.sprite_prep_snitch_bounce_2(k);
        assert_eq!(bounce.ram[SPRITE_A + k], 0x55);
        bounce.sprite_prep_snitch_bounce_3(k);
        assert_eq!(bounce.ram[SPRITE_IGNORE_PROJECTILE + k], 2);

        let mut runner = fresh_state();
        runner.sprite_prep_running_man(k);
        assert_eq!(runner.ram[SPRITE_D + k], 2);
        assert_eq!(runner.ram[SPRITE_HEAD_DIR + k], 2);
        assert_eq!(runner.ram[SPRITE_IGNORE_PROJECTILE + k], 1);

        let mut mushroom = fresh_state();
        mushroom.ram[LINK_ITEM_MUSHROOM] = 1;
        mushroom.ram[SPRITE_GRAPHICS + k] = 7;
        mushroom.sprite_prep_mushroom(k);
        assert_eq!(mushroom.ram[SPRITE_GRAPHICS + k], 0);
        assert_eq!(mushroom.ram[SPRITE_OAM_FLAGS + k] & 8, 8);
        assert_eq!(mushroom.ram[SPRITE_IGNORE_PROJECTILE + k], 1);

        mushroom.ram[LINK_ITEM_MUSHROOM] = 2;
        mushroom.ram[SPRITE_STATE + k] = 9;
        mushroom.sprite_prep_mushroom(k);
        assert_eq!(mushroom.ram[SPRITE_STATE + k], 0);
    }

    #[test]
    fn potion_shop_prep_spawns_powder_and_cauldrons_with_barrier_flags() {
        let k = 4;
        let mut s = fresh_state();
        s.ram[SPRITE_STATE + k] = 9;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        s.ram[FLAG_OVERWORLD_AREA_DID_CHANGE_PREP] = 1;
        s.ram[LINK_ITEM_MUSHROOM] = 1;
        write_le_u16(&mut s.ram, SAVE_DUNG_INFO + 0x109 * 2, 0x80);

        s.sprite_prep_potion_shop(k);

        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
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
            assert_eq!(s.ram[SPRITE_STATE + slot], 9);
            assert_eq!(s.ram[SPRITE_TYPE + slot], 0xe9);
            assert_eq!(s.ram[SPRITE_SUBTYPE2 + slot], subtype);
            assert_eq!(s.sprite_get_x(slot), x);
            assert_eq!(s.sprite_get_y(slot), y);
            assert_eq!(s.ram[SPRITE_FLAGS4 + slot], 3);
            assert_eq!(s.ram[SPRITE_DEFL_BITS + slot] & 0x20, 0x20);
        }

        let mut skipped_powder = fresh_state();
        skipped_powder.ram[SPRITE_STATE + k] = 9;
        skipped_powder.ram[FLAG_OVERWORLD_AREA_DID_CHANGE_PREP] = 0;
        skipped_powder.ram[LINK_ITEM_MUSHROOM] = 1;
        write_le_u16(&mut skipped_powder.ram, SAVE_DUNG_INFO + 0x109 * 2, 0x80);
        skipped_powder.sprite_prep_potion_shop(k);
        assert_eq!(skipped_powder.ram[SPRITE_SUBTYPE2 + 15], 2);
        assert_eq!(skipped_powder.ram[SPRITE_SUBTYPE2 + 14], 3);
        assert_eq!(skipped_powder.ram[SPRITE_SUBTYPE2 + 13], 4);
        assert_eq!(skipped_powder.ram[SPRITE_STATE + 12], 0);
    }

    #[test]
    fn arrow_game_prep_seeds_archery_sprites_from_link_state() {
        let k = 0;
        let mut s = fresh_state();
        s.ram[SPRITE_Y_LO + k] = 0x30;
        s.ram[ARCHERY_GAME_HIT_COUNTER] = 0xaa;
        s.ram[LINK_X_COORD + 1] = 0x12;
        s.ram[LINK_Y_COORD + 1] = 0x34;
        s.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        s.ram[LINK_NUM_ARROWS] = 17;

        s.sprite_prep_arrow_game_bounce(k);

        assert_eq!(s.ram[ARCHERY_GAME_HIT_COUNTER], 0);
        assert_eq!(s.ram[SPRITE_Y_LO + k], 0x27);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(s.ram[SPRITE_SUBTYPE + k], 17);

        assert_eq!(s.ram[SPRITE_TYPE + 1], 0x65);
        assert_eq!(s.ram[SPRITE_STATE + 1], 9);
        assert_eq!(s.ram[SPRITE_X_HI + 1], 0x12);
        assert_eq!(s.ram[SPRITE_X_LO + 1], 0x40);
        assert_eq!(s.ram[SPRITE_Y_HI + 1], 0x34);
        assert_eq!(s.ram[SPRITE_Y_LO + 1], 0x4f);
        assert_eq!(s.ram[SPRITE_A + 1], 1);
        assert_eq!(s.ram[SPRITE_GRAPHICS + 1], 0);
        assert_eq!(s.ram[SPRITE_X_VEL + 1], (-8i8) as u8);
        assert_eq!(s.ram[SPRITE_FLAGS4 + 1], 0x1c);
        assert_eq!(s.ram[SPRITE_OAM_FLAGS + 1], 13);
        assert_eq!(s.ram[SPRITE_FLOOR + 1], 1);

        assert_eq!(s.ram[SPRITE_X_LO + 7], 0xc0);
        assert_eq!(s.ram[SPRITE_Y_LO + 7], 0x5a);
        assert_eq!(s.ram[SPRITE_A + 7], 2);
        assert_eq!(s.ram[SPRITE_GRAPHICS + 7], 1);
        assert_eq!(s.ram[SPRITE_X_VEL + 7], 12);
        assert_eq!(s.ram[SPRITE_FLAGS4 + 7], 0x15);
    }

    #[test]
    fn heart_upgrade_prep_clears_already_obtained_entries() {
        let k = 4;

        let mut overworld = fresh_state();
        overworld.ram[SPRITE_STATE + k] = 9;
        overworld.ram[OVERWORLD_SCREEN_INDEX] = 0x22;
        overworld.ram[SAVE_OW_EVENT_INFO + 0x22] = 0x40;
        overworld.sprite_prep_heart_container(k);
        assert_eq!(overworld.ram[SPRITE_STATE + k], 0);
        overworld.ram[SAVE_OW_EVENT_INFO + 0x22] = 0x10;
        overworld.heart_upgrade_set_obtained_flag(k);
        assert_eq!(overworld.ram[SAVE_OW_EVENT_INFO + 0x22], 0x50);

        let mut lumberjack = fresh_state();
        lumberjack.ram[SPRITE_STATE + k] = 9;
        lumberjack.ram[OVERWORLD_SCREEN_INDEX] = 0x3b;
        lumberjack.ram[SAVE_OW_EVENT_INFO + 0x3b] = 0;
        lumberjack.sprite_prep_heart_piece(k);
        assert_eq!(lumberjack.ram[SPRITE_STATE + k], 0);

        let mut dungeon = fresh_state();
        dungeon.ram[PLAYER_IS_INDOORS] = 1;
        dungeon.ram[SPRITE_STATE + k] = 9;
        dungeon.ram[SPRITE_X_HI + k] = 0;
        write_le_u16(&mut dungeon.ram, DUNG_SAVEGAME_STATE_BITS, 0x4000);
        dungeon.heart_upgrade_check_if_already_obtained(k);
        assert_eq!(dungeon.ram[SPRITE_STATE + k], 0);
        write_le_u16(&mut dungeon.ram, DUNG_SAVEGAME_STATE_BITS, 0x0001);
        dungeon.heart_upgrade_set_obtained_flag(k);
        assert_eq!(read_le_u16(&dungeon.ram, DUNG_SAVEGAME_STATE_BITS), 0x4001);

        dungeon.ram[SPRITE_X_HI + k] = 1;
        write_le_u16(&mut dungeon.ram, DUNG_SAVEGAME_STATE_BITS, 0x0002);
        dungeon.heart_upgrade_set_obtained_flag(k);
        assert_eq!(read_le_u16(&dungeon.ram, DUNG_SAVEGAME_STATE_BITS), 0x2002);

        let mut untouched = fresh_state();
        untouched.ram[SPRITE_STATE + k] = 9;
        untouched.ram[OVERWORLD_SCREEN_INDEX] = 0x11;
        untouched.heart_upgrade_check_if_already_obtained(k);
        assert_eq!(untouched.ram[SPRITE_STATE + k], 9);
    }

    #[test]
    fn swamola_prep_initializes_segment_history_and_position_snapshot() {
        let k = 2;
        let mut buggy = fresh_state();
        buggy.ram[SPRITE_X_LO + k] = 0x44;
        buggy.ram[SPRITE_X_HI + k] = 0x01;
        buggy.ram[SPRITE_Y_LO + k] = 0x88;
        buggy.ram[SPRITE_Y_HI + k] = 0x02;
        buggy.sprite_prep_swamola(k);
        let buggy_start = 0x03;
        assert_eq!(buggy.ram[SWAMOLA_X_LO_PREP + buggy_start], 0x44);
        assert_eq!(buggy.ram[SWAMOLA_X_HI_PREP + buggy_start + 31], 0x01);
        assert_eq!(buggy.ram[SWAMOLA_Y_LO_PREP + buggy_start], 0x88);
        assert_eq!(buggy.ram[SWAMOLA_Y_HI_PREP + buggy_start + 31], 0x02);
        assert_eq!(buggy.ram[SPRITE_A + k], 0x44);
        assert_eq!(buggy.ram[SPRITE_B + k], 0x01);
        assert_eq!(buggy.ram[SPRITE_C + k], 0x88);
        assert_eq!(buggy.ram[SPRITE_HEAD_DIR + k], 0x02);

        let mut fixed = fresh_state();
        fixed.write_u32_ram(ENHANCED_FEATURES0, K_FEATURES0_MISC_BUG_FIXES_PREP);
        fixed.ram[SPRITE_X_LO + k] = 0x77;
        fixed.ram[SPRITE_X_HI + k] = 0x03;
        fixed.ram[SPRITE_Y_LO + k] = 0x99;
        fixed.ram[SPRITE_Y_HI + k] = 0x04;
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
        maiden.ram[SPRITE_STATE + k] = 9;
        maiden.ram[FOLLOWER_INDICATOR] = 0;
        maiden.ram[FOLLOWER_DROPPED] = 0x80;
        maiden.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 7;
        maiden.sprite_prep_blind_maiden(k);
        assert_eq!(maiden.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(maiden.ram[FOLLOWER_INDICATOR], 0);
        assert_eq!(maiden.ram[FOLLOWER_DROPPED], 0);
        assert_eq!(maiden.ram[TAGALONG_APPEARANCE_NONE_FLAG], 0);
        assert_eq!(maiden.ram[SPRITE_STATE + k], 9);

        let mut maiden_finished = fresh_state();
        maiden_finished.ram[SPRITE_STATE + k] = 9;
        write_le_u16(&mut maiden_finished.ram, SAVE_DUNG_INFO + 0xac * 2, 0x0800);
        maiden_finished.sprite_prep_blind_maiden(k);
        assert_eq!(maiden_finished.ram[SPRITE_STATE + k], 0);

        let mut old_man_room = fresh_state();
        old_man_room.ram[DUNGEON_ROOM_INDEX] = 0xe4;
        old_man_room.sprite_prep_old_man_bounce(k);
        assert_eq!(old_man_room.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(old_man_room.ram[SPRITE_SUBTYPE2 + k], 2);

        let mut old_man_mirror = fresh_state();
        old_man_mirror.ram[LINK_ITEM_MIRROR] = 2;
        old_man_mirror.ram[SPRITE_STATE + k] = 9;
        old_man_mirror.sprite_prep_old_man_bounce(k);
        assert_eq!(old_man_mirror.ram[SPRITE_STATE + k], 0);
        assert_eq!(old_man_mirror.ram[FOLLOWER_INDICATOR], 0);

        let mut old_man_followed = fresh_state();
        old_man_followed.ram[FOLLOWER_INDICATOR] = 1;
        old_man_followed.ram[SPRITE_STATE + k] = 9;
        old_man_followed.sprite_prep_old_man_bounce(k);
        assert_eq!(old_man_followed.ram[SPRITE_STATE + k], 0);
        assert_eq!(old_man_followed.ram[FOLLOWER_INDICATOR], 1);
    }

    #[test]
    fn zelda_bounce_prep_matches_sword_room_and_progress_gates() {
        let k = 6;

        let mut has_sword = fresh_state();
        has_sword.ram[LINK_SWORD_TYPE] = 2;
        has_sword.ram[SPRITE_STATE + k] = 9;
        has_sword.sprite_prep_zelda_bounce(k);
        assert_eq!(has_sword.ram[SPRITE_STATE + k], 0);

        let mut cell = fresh_state();
        cell.ram[SPRITE_STATE + k] = 9;
        cell.ram[DUNGEON_ROOM_INDEX] = 0x12;
        cell.ram[SRAM_PROGRESS_FLAGS] = 4;
        cell.ram[FOLLOWER_INDICATOR] = 7;
        cell.sprite_set_x(k, 0x0100);
        cell.sprite_set_y(k, 0x0200);
        write_le_u16(&mut cell.ram, LINK_X_COORD, 0x0180);
        write_le_u16(&mut cell.ram, LINK_Y_COORD, 0x0200);
        cell.sprite_prep_zelda_bounce(k);
        assert_eq!(cell.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(cell.ram[SPRITE_D + k], 3);
        assert_eq!(cell.ram[SPRITE_HEAD_DIR + k], 3);
        assert_eq!(cell.ram[FOLLOWER_INDICATOR], 7);
        assert_eq!(cell.ram[SPRITE_SUBTYPE2 + k], 2);
        assert_eq!(cell.sprite_get_x(k), 0x0106);
        assert_eq!(cell.sprite_get_y(k), 0x020f);
        assert_eq!(cell.ram[SPRITE_FLAGS4 + k], 3);
        assert_eq!(cell.ram[SPRITE_STATE + k], 9);

        let mut not_rescued = fresh_state();
        not_rescued.ram[SPRITE_STATE + k] = 9;
        not_rescued.ram[DUNGEON_ROOM_INDEX] = 0x12;
        not_rescued.ram[SRAM_PROGRESS_FLAGS] = 0;
        not_rescued.sprite_prep_zelda_bounce(k);
        assert_eq!(not_rescued.ram[SPRITE_STATE + k], 0);

        let mut follower_present = fresh_state();
        follower_present.ram[SPRITE_STATE + k] = 9;
        follower_present.ram[DUNGEON_ROOM_INDEX] = 0x20;
        follower_present.ram[FOLLOWER_INDICATOR] = 1;
        follower_present.sprite_prep_zelda_bounce(k);
        assert_eq!(follower_present.ram[SPRITE_SUBTYPE2 + k], 0);
        assert_eq!(follower_present.ram[SPRITE_STATE + k], 0);
    }

    #[test]
    fn bomb_shoppe_prep_spawns_visible_bombs_and_big_bomb_when_unlocked() {
        let k = 2;
        let mut s = fresh_state();
        s.ram[SPRITE_STATE + k] = 9;
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0230);
        s.ram[LINK_HAS_CRYSTALS] = 5;
        s.ram[SRAM_PROGRESS_INDICATOR_3_PREP] = 32;

        s.sprite_prep_bomb_shoppe(k);

        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(s.ram[SPRITE_STATE + 15], 9);
        assert_eq!(s.ram[SPRITE_TYPE + 15], 0xb5);
        assert_eq!(s.sprite_get_x(15), 0x0120u16.wrapping_sub(24));
        assert_eq!(s.sprite_get_y(15), 0x0230u16.wrapping_sub(24));
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + 15], 1);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + 15], 1);
        assert_eq!(s.ram[SPRITE_STATE + 14], 9);
        assert_eq!(s.ram[SPRITE_TYPE + 14], 0xb5);
        assert_eq!(s.sprite_get_x(14), 0x0120u16.wrapping_sub(56));
        assert_eq!(s.sprite_get_y(14), 0x0230u16.wrapping_sub(24));
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + 14], 2);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + 14], 2);

        let mut locked = fresh_state();
        locked.ram[SPRITE_STATE + k] = 9;
        locked.sprite_set_x(k, 0x0040);
        locked.sprite_set_y(k, 0x0050);
        locked.ram[LINK_HAS_CRYSTALS] = 4;
        locked.ram[SRAM_PROGRESS_INDICATOR_3_PREP] = 32;
        locked.sprite_prep_bomb_shoppe(k);
        assert_eq!(locked.ram[SPRITE_STATE + 15], 9);
        assert_eq!(locked.ram[SPRITE_STATE + 14], 0);
    }

    #[test]
    fn bomb_shop_clerk_exhalation_spawns_huff_with_exact_state() {
        let k = 2;
        let mut s = fresh_state();
        s.ram[SPRITE_STATE + k] = 9;
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0230);
        s.ram[SPRITE_Z + k] = 9;
        s.ram[SPRITE_FLAGS3 + 15] = 0xff;

        s.bomb_shop_clerk_exhalation(k);

        assert_eq!(s.ram[SPRITE_STATE + 15], 9);
        assert_eq!(s.ram[SPRITE_TYPE + 15], 0xb5);
        assert_eq!(s.sprite_get_x(15), 0x0124);
        assert_eq!(s.sprite_get_y(15), 0x0240);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + 15], 3);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + 15], 3);
        assert_eq!(s.ram[SPRITE_Z + 15], 4);
        assert_eq!(s.ram[SPRITE_Z_VEL + 15], (-12i8) as u8);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + 15], 23);
        assert_eq!(s.ram[SPRITE_FLAGS3 + 15] & 0x11, 0);
    }

    #[test]
    fn bomb_shop_clerk_exhalation_noops_when_no_spawn_slot_exists() {
        let k = 2;
        let mut s = fresh_state();
        for slot in 0..16 {
            s.ram[SPRITE_STATE + slot] = 9;
            s.ram[SPRITE_TYPE + slot] = 0xa0 + slot as u8;
        }
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        let before = s.ram[SPRITE_TYPE + 15];

        s.bomb_shop_clerk_exhalation(k);

        assert_eq!(s.ram[SPRITE_TYPE + 15], before);
        assert_eq!(s.sprite_get_x(15), 0);
        assert_eq!(s.sprite_get_y(15), 0);
    }

    #[test]
    fn archery_game_guy_show_msg_sets_message_module_and_clears_delay() {
        let k = 4;
        let mut s = fresh_state();
        s.ram[TILE_INTERACTION_SHARED_FLAG] = 7;
        s.ram[MESSAGING_MODULE] = 9;
        s.ram[SUBMODULE_INDEX] = 1;
        s.ram[MAIN_MODULE_INDEX] = 3;
        s.ram[SAVED_MODULE_FOR_MENU] = 0;
        s.ram[SPRITE_DELAY_MAIN + k] = 88;

        s.archery_game_guy_show_msg(k, 0x86);

        assert_eq!(read_le_u16(&s.ram, DIALOGUE_MESSAGE_INDEX), 0x86);
        assert_eq!(s.ram[TILE_INTERACTION_SHARED_FLAG], 0);
        assert_eq!(s.ram[MESSAGING_MODULE], 0);
        assert_eq!(s.ram[SUBMODULE_INDEX], 2);
        assert_eq!(s.ram[SAVED_MODULE_FOR_MENU], 3);
        assert_eq!(s.ram[MAIN_MODULE_INDEX], 14);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + k], 0);
    }

    #[test]
    fn debirando_prep_spawns_pit_pair_and_fire_variant_reloads_properties() {
        let k = 3;
        let mut pit = fresh_state();
        pit.ram[SPRITE_STATE + k] = 9;
        pit.ram[SPRITE_G + k] = 0;
        pit.ram[SPRITE_DELAY_MAIN + k] = 7;
        pit.ram[SPRITE_GRAPHICS + k] = 2;
        pit.ram[SPRITE_X_LO + k] = 0x70;
        pit.ram[SPRITE_Y_LO + k] = 0x80;

        pit.sprite_prep_debirando_pit(k);

        assert_eq!(pit.ram[SPRITE_G + k], 1);
        assert_eq!(pit.ram[SPRITE_DELAY_MAIN + k], 0);
        assert_eq!(pit.ram[SPRITE_GRAPHICS + k], 6);
        assert_eq!(pit.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(pit.ram[SPRITE_HEAD_DIR + k], 15);
        assert_eq!(pit.ram[SPRITE_STATE + 15], 9);
        assert_eq!(pit.ram[SPRITE_TYPE + 15], 0x64);
        assert_eq!(pit.ram[SPRITE_DELAY_MAIN + 15], 96);
        assert_eq!(pit.ram[SPRITE_G + 15], 1);
        assert_eq!(pit.ram[SPRITE_OAM_FLAGS + 15], 8);
        assert_eq!(pit.sprite_get_x(15), pit.sprite_get_x(k));
        assert_eq!(pit.sprite_get_y(15), pit.sprite_get_y(k));

        let mut fire = fresh_state();
        fire.ram[SPRITE_STATE + k] = 9;
        fire.ram[SPRITE_TYPE + k] = 0x64;
        fire.ram[SPRITE_G + k] = 7;
        fire.ram[SPRITE_DELAY_MAIN + k] = 9;
        fire.ram[SPRITE_X_LO + k] = 0x44;
        fire.ram[SPRITE_Y_LO + k] = 0x55;
        fire.sprite_prep_fire_debirando(k);
        assert_eq!(fire.ram[SPRITE_TYPE + k], 0x63);
        assert_eq!(fire.ram[SPRITE_G + k], 0);
        assert_eq!(fire.ram[SPRITE_GRAPHICS + k], 6);
        assert_eq!(fire.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(fire.ram[SPRITE_TYPE + 15], 0x64);
        assert_eq!(fire.ram[SPRITE_G + 15], 0);
        assert_eq!(fire.ram[SPRITE_OAM_FLAGS + 15], 6);
    }

    #[test]
    fn bully_hobo_and_talking_tree_prep_spawn_helper_sprites() {
        let k = 4;

        let mut bully = fresh_state();
        bully.ram[SPRITE_STATE + k] = 9;
        bully.sprite_set_x(k, 0x0110);
        bully.sprite_set_y(k, 0x0220);
        bully.sprite_prep_bully_and_victim(k);
        assert_eq!(bully.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(bully.ram[SPRITE_STATE + 15], 9);
        assert_eq!(bully.ram[SPRITE_TYPE + 15], 0xb9);
        assert_eq!(bully.sprite_get_x(15), 0x0110);
        assert_eq!(bully.sprite_get_y(15), 0x0220);
        assert_eq!(bully.ram[SPRITE_SUBTYPE2 + 15], 2);
        assert_eq!(bully.ram[SPRITE_HEAD_DIR + 15], k as u8);
        assert_eq!(bully.ram[SPRITE_IGNORE_PROJECTILE + 15], 1);
        bully.ball_guy_play_bounce_noise(k);
        assert_eq!(bully.ram[SOUND_EFFECT_2] & 0x3f, 0x32);

        let mut garnish = fresh_state();
        garnish.ram[GARNISH_TYPE + 29] = 1;
        garnish.ram[GARNISH_TYPE + 14] = 1;
        assert_eq!(garnish.garnish_alloc_force(), 28);
        assert_eq!(garnish.garnish_alloc(), 28);
        assert_eq!(garnish.garnish_alloc_low(), 13);
        assert_eq!(garnish.garnish_alloc_limit(12), 12);

        garnish.ram[GARNISH_TYPE..GARNISH_TYPE + 30].fill(1);
        assert_eq!(garnish.garnish_alloc_force(), 0);
        assert_eq!(garnish.garnish_alloc(), -1);
        assert_eq!(garnish.garnish_alloc_low(), -1);
        assert_eq!(garnish.garnish_alloc_limit(12), -1);
        assert_eq!(garnish.garnish_alloc_overwrite_old_low(), 14);
        assert_eq!(garnish.garnish_alloc_overwrite_old(), 13);

        let mut coords = fresh_state();
        coords.garnish_set_x(3, 0x1234);
        coords.garnish_set_y(3, 0xabcd);
        assert_eq!(coords.ram[GARNISH_X_LO_PREP + 3], 0x34);
        assert_eq!(coords.ram[GARNISH_X_HI_PREP + 3], 0x12);
        assert_eq!(coords.ram[GARNISH_Y_LO_PREP + 3], 0xcd);
        assert_eq!(coords.ram[GARNISH_Y_HI_PREP + 3], 0xab);

        let mut debris = fresh_state();
        debris.ram[GARNISH_TYPE + 29] = 1;
        debris.garnish_spawn_pyramid_debris(-4, 5, -7, 9);
        assert_eq!(debris.ram[SOUND_EFFECT_2], 3);
        assert_eq!(debris.ram[SOUND_EFFECT_1], 31);
        assert_eq!(debris.ram[SOUND_EFFECT_AMBIENT], 5);
        assert_eq!(debris.ram[GARNISH_TYPE + 28], 19);
        assert_eq!(debris.ram[GARNISH_ACTIVE_PREP], 19);
        assert_eq!(debris.ram[GARNISH_X_LO_PREP + 28], 228);
        assert_eq!(debris.ram[GARNISH_Y_LO_PREP + 28], 101);
        assert_eq!(debris.ram[GARNISH_X_VEL_PREP + 28], (-7i8) as u8);
        assert_eq!(debris.ram[GARNISH_Y_VEL_PREP + 28], 9);
        assert_eq!(debris.ram[GARNISH_COUNTDOWN_PREP + 28], 72);

        let mut puff = fresh_state();
        let puff_owner = 6;
        puff.ram[FRAME_COUNTER] = 2;
        puff.ram[GARNISH_TYPE + 14] = 1;
        write_le_u16(&mut puff.ram, CUR_SPRITE_X, 0x0200);
        write_le_u16(&mut puff.ram, CUR_SPRITE_Y, 0x0300);
        puff.kholdstare_spawn_puff_cloud_garnish(puff_owner);
        assert_eq!(puff.ram[GARNISH_TYPE + 13], 7);
        assert_eq!(puff.ram[GARNISH_ACTIVE_PREP], 7);
        assert_eq!(puff.ram[GARNISH_COUNTDOWN_PREP + 13], 31);
        assert_eq!(puff.ram[GARNISH_X_LO_PREP + 13], 0xfa);
        assert_eq!(puff.ram[GARNISH_X_HI_PREP + 13], 0x01);
        assert_eq!(puff.ram[GARNISH_Y_LO_PREP + 13], 0x12);
        assert_eq!(puff.ram[GARNISH_Y_HI_PREP + 13], 0x03);
        assert_eq!(puff.ram[GARNISH_FLOOR_PREP + 13], 0);

        let mut flame = fresh_state();
        flame.ram[GARNISH_TYPE + 29] = 1;
        flame.sprite_set_x(k, 0x0456);
        flame.sprite_set_y(k, 0x0789);
        assert_eq!(flame.garnish_flame_trail(k, false), 28);
        assert_eq!(flame.ram[GARNISH_TYPE + 28], 0x10);
        assert_eq!(flame.ram[GARNISH_ACTIVE_PREP], 0x10);
        assert_eq!(flame.ram[GARNISH_SPRITE_PREP + 28], k as u8);
        assert_eq!(flame.ram[GARNISH_X_LO_PREP + 28], 0x56);
        assert_eq!(flame.ram[GARNISH_X_HI_PREP + 28], 0x04);
        assert_eq!(flame.ram[GARNISH_Y_LO_PREP + 28], 0x99);
        assert_eq!(flame.ram[GARNISH_Y_HI_PREP + 28], 0x07);
        assert_eq!(flame.ram[GARNISH_COUNTDOWN_PREP + 28], 127);

        let mut low_flame = fresh_state();
        low_flame.ram[GARNISH_TYPE + 14] = 1;
        low_flame.sprite_set_x(k, 0x0012);
        low_flame.sprite_set_y(k, 0x00f8);
        assert_eq!(low_flame.garnish_flame_trail(k, true), 13);
        assert_eq!(low_flame.ram[GARNISH_TYPE + 13], 0x10);
        assert_eq!(low_flame.ram[GARNISH_Y_LO_PREP + 13], 0x08);
        assert_eq!(low_flame.ram[GARNISH_Y_HI_PREP + 13], 0x01);

        let mut fire_bat = fresh_state();
        fire_bat.ram[SPRITE_SUBTYPE2 + k] = 3;
        fire_bat.fire_bat_animate(k);
        assert_eq!(fire_bat.ram[SPRITE_SUBTYPE2 + k], 4);
        assert_eq!(fire_bat.ram[SPRITE_GRAPHICS + k], 5);

        let mut moving_fire_bat = fresh_state();
        moving_fire_bat.ram[GARNISH_TYPE + 14] = 1;
        moving_fire_bat.ram[SPRITE_SUBTYPE2 + k] = 7;
        moving_fire_bat.ram[SPRITE_ANIM_CLOCK + k] = 5;
        moving_fire_bat.sprite_set_x(k, 0x0124);
        moving_fire_bat.sprite_set_y(k, 0x0340);
        moving_fire_bat.fire_bat_move(k);
        assert_eq!(moving_fire_bat.ram[SPRITE_SUBTYPE2 + k], 8);
        assert_eq!(moving_fire_bat.ram[SPRITE_GRAPHICS + k], 6);
        assert_eq!(moving_fire_bat.ram[GARNISH_TYPE + 13], 0x10);
        assert_eq!(moving_fire_bat.ram[GARNISH_ACTIVE_PREP], 0x10);
        assert_eq!(moving_fire_bat.ram[GARNISH_SPRITE_PREP + 13], k as u8);
        assert_eq!(moving_fire_bat.ram[GARNISH_X_LO_PREP + 13], 0x24);
        assert_eq!(moving_fire_bat.ram[GARNISH_X_HI_PREP + 13], 0x01);
        assert_eq!(moving_fire_bat.ram[GARNISH_Y_LO_PREP + 13], 0x50);
        assert_eq!(moving_fire_bat.ram[GARNISH_Y_HI_PREP + 13], 0x03);
        assert_eq!(moving_fire_bat.ram[GARNISH_COUNTDOWN_PREP + 13], 0x2f);

        let mut skipped_fire_bat = fresh_state();
        skipped_fire_bat.ram[SPRITE_SUBTYPE2 + k] = 0;
        skipped_fire_bat.fire_bat_move(k);
        assert_eq!(skipped_fire_bat.ram[SPRITE_SUBTYPE2 + k], 1);
        assert_eq!(skipped_fire_bat.ram[GARNISH_ACTIVE_PREP], 0);

        let mut fireball = fresh_state();
        fireball.ram[FRAME_COUNTER] = 0;
        fireball.ram[GARNISH_TYPE + 29] = 1;
        write_le_u16(&mut fireball.ram, CUR_SPRITE_X, 0x0123);
        write_le_u16(&mut fireball.ram, CUR_SPRITE_Y, 0x02f5);
        fireball.fireball_spawn_trail_garnish(k);
        assert_eq!(fireball.ram[GARNISH_TYPE + 28], 8);
        assert_eq!(fireball.ram[GARNISH_ACTIVE_PREP], 8);
        assert_eq!(fireball.ram[GARNISH_COUNTDOWN_PREP + 28], 11);
        assert_eq!(fireball.ram[GARNISH_X_LO_PREP + 28], 0x23);
        assert_eq!(fireball.ram[GARNISH_X_HI_PREP + 28], 0x01);
        assert_eq!(fireball.ram[GARNISH_Y_LO_PREP + 28], 0x05);
        assert_eq!(fireball.ram[GARNISH_Y_HI_PREP + 28], 0x03);
        assert_eq!(fireball.ram[GARNISH_SPRITE_PREP + 28], k as u8);

        let mut skipped_fireball = fresh_state();
        skipped_fireball.ram[FRAME_COUNTER] = 1;
        skipped_fireball.fireball_spawn_trail_garnish(k);
        assert_eq!(skipped_fireball.ram[GARNISH_ACTIVE_PREP], 0);

        let mut firesnake = fresh_state();
        firesnake.ram[FRAME_COUNTER] = k as u8;
        firesnake.ram[GARNISH_TYPE + 29] = 1;
        firesnake.sprite_set_x(k, 0x0167);
        firesnake.sprite_set_y(k, 0x02f0);
        firesnake.ram[SPRITE_FLOOR + k] = 2;
        firesnake.firesnake_spawn_fireball(k);
        assert_eq!(firesnake.ram[GARNISH_TYPE + 28], 1);
        assert_eq!(firesnake.ram[GARNISH_ACTIVE_PREP], 1);
        assert_eq!(firesnake.ram[GARNISH_X_LO_PREP + 28], 0x67);
        assert_eq!(firesnake.ram[GARNISH_X_HI_PREP + 28], 0x01);
        assert_eq!(firesnake.ram[GARNISH_Y_LO_PREP + 28], 0x00);
        assert_eq!(firesnake.ram[GARNISH_Y_HI_PREP + 28], 0x03);
        assert_eq!(firesnake.ram[GARNISH_COUNTDOWN_PREP + 28], 32);
        assert_eq!(firesnake.ram[GARNISH_SPRITE_PREP + 28], k as u8);
        assert_eq!(firesnake.ram[GARNISH_FLOOR_PREP + 28], 2);

        let mut skipped_firesnake = fresh_state();
        skipped_firesnake.ram[FRAME_COUNTER] = (k as u8) ^ 1;
        skipped_firesnake.firesnake_spawn_fireball(k);
        assert_eq!(skipped_firesnake.ram[GARNISH_ACTIVE_PREP], 0);

        let mut plop = fresh_state();
        plop.ram[SPRITE_STATE + k] = 9;
        plop.sprite_set_x(k, 0x0100);
        plop.sprite_set_y(k, 0x0200);
        plop.catfish_spawn_plop(k);
        assert_eq!(plop.ram[SPRITE_TYPE + 15], 0xec);
        assert_eq!(plop.sprite_get_x(15), 0x0100);
        assert_eq!(plop.sprite_get_y(15), 0x0200);
        assert_eq!(plop.ram[SPRITE_STATE + 15], 3);
        assert_eq!(plop.ram[SPRITE_DELAY_MAIN + 15], 15);
        assert_eq!(plop.ram[SPRITE_AI_STATE + 15], 0);
        assert_eq!(plop.ram[SPRITE_FLAGS2 + 15], 3);
        assert_eq!(plop.ram[SOUND_EFFECT_1] & 0x3f, 0x28);

        let mut medallion = fresh_state();
        medallion.ram[SPRITE_STATE + k] = 9;
        medallion.sprite_set_x(k, 0x0100);
        medallion.sprite_set_y(k, 0x0200);
        medallion.catfish_regurgitate_medallion(k);
        assert_eq!(medallion.ram[SPRITE_TYPE + 15], 0xc0);
        assert_eq!(medallion.sprite_get_x(15), 0x0100);
        assert_eq!(medallion.sprite_get_y(15), 0x0200);
        assert_eq!(medallion.ram[SPRITE_X_VEL + 15], 24);
        assert_eq!(medallion.ram[SPRITE_Z_VEL + 15], 48);
        assert_eq!(medallion.ram[SPRITE_A + 15], 17);
        assert_eq!(medallion.ram[SOUND_EFFECT_1] & 0x3f, 0x20);
        assert_eq!(medallion.ram[SPRITE_FLAGS2 + 15], 0x83);
        assert_eq!(medallion.ram[SPRITE_FLAGS3 + 15], 0x58);
        assert_eq!(medallion.ram[SPRITE_OAM_FLAGS + 15], 8);

        let mut splash = fresh_state();
        splash.ram[SPRITE_STATE + k] = 9;
        splash.sprite_set_x(k, 0x0030);
        splash.sprite_set_y(k, 0x0040);
        assert_eq!(splash.sprite_spawn_water_splash(k), 15);
        assert_eq!(splash.ram[SPRITE_TYPE + 15], 0xc0);
        assert_eq!(splash.sprite_get_x(15), 0x0030);
        assert_eq!(splash.sprite_get_y(15), 0x0040);
        assert_eq!(splash.ram[SPRITE_A + 15], 0x80);
        assert_eq!(splash.ram[SPRITE_FLAGS2 + 15], 2);
        assert_eq!(splash.ram[SPRITE_IGNORE_PROJECTILE + 15], 2);
        assert_eq!(splash.ram[SPRITE_OAM_FLAGS + 15], 4);
        assert_eq!(splash.ram[SPRITE_DELAY_MAIN + 15], 31);

        let mut small_splash = fresh_state();
        small_splash.ram[SPRITE_STATE + k] = 9;
        small_splash.sprite_set_x(k, 0x0060);
        small_splash.sprite_set_y(k, 0x0070);
        small_splash.ram[SOUND_EFFECT_1] = 0xff;
        assert_eq!(small_splash.sprite_spawn_small_splash(k), 14);
        assert_eq!(small_splash.ram[SPRITE_TYPE + 14], 0xec);
        assert_eq!(small_splash.sprite_get_x(14), 0x0060);
        assert_eq!(small_splash.sprite_get_y(14), 0x0070);
        assert_eq!(small_splash.ram[SOUND_EFFECT_1] & 0x3f, 0x28);
        assert_eq!(small_splash.ram[SPRITE_STATE + 14], 3);
        assert_eq!(small_splash.ram[SPRITE_DELAY_MAIN + 14], 15);
        assert_eq!(small_splash.ram[SPRITE_AI_STATE + 14], 0);
        assert_eq!(small_splash.ram[SPRITE_FLAGS2 + 14], 3);

        let mut dust = fresh_state();
        dust.ram[SPRITE_STATE + k] = 9;
        dust.sprite_set_x(k, 0x0100);
        dust.sprite_set_y(k, 0x0200);
        assert_eq!(dust.sprite_spawn_dust_cloud(k), 15);
        assert_eq!(dust.ram[SPRITE_TYPE + 15], 0xf2);
        assert_eq!(dust.sprite_get_x(15), 0x00fc);
        assert_eq!(dust.sprite_get_y(15), 0x0208);
        assert_eq!(dust.ram[SPRITE_SUBTYPE2 + 15], 1);

        let mut blast = fresh_state();
        blast.ram[SPRITE_STATE + k] = 9;
        blast.sprite_set_x(k, 0x0018);
        blast.sprite_set_y(k, 0x0028);
        assert_eq!(blast.sprite_spawn_superficial_bomb_blast(k), 15);
        assert_eq!(blast.ram[SPRITE_TYPE + 15], 0x4a);
        assert_eq!(blast.sprite_get_x(15), 0x0018);
        assert_eq!(blast.sprite_get_y(15), 0x0028);
        assert_eq!(blast.ram[SPRITE_STATE + 15], 6);
        assert_eq!(blast.ram[SPRITE_DELAY_AUX1 + 15], 31);
        assert_eq!(blast.ram[SPRITE_C + 15], 3);
        assert_eq!(blast.ram[SPRITE_FLAGS2 + 15], 3);
        assert_eq!(blast.ram[SPRITE_OAM_FLAGS + 15], 4);
        assert_eq!(blast.ram[SOUND_EFFECT_1] & 0x3f, 0x15);

        let mut bomb = fresh_state();
        bomb.ram[SPRITE_STATE + k] = 9;
        bomb.sprite_set_x(k, 0x0044);
        bomb.sprite_set_y(k, 0x0055);
        assert_eq!(bomb.sprite_spawn_bomb(k), 15);
        assert_eq!(bomb.ram[SPRITE_TYPE + 15], 0x4a);
        assert_eq!(bomb.sprite_get_x(15), 0x0044);
        assert_eq!(bomb.sprite_get_y(15), 0x0055);
        assert_eq!(bomb.ram[SPRITE_C + 15], 1);
        assert_eq!(bomb.ram[SPRITE_DELAY_AUX1 + 15], 80);
        assert_eq!(bomb.ram[SPRITE_FLAGS3 + 15], 0x18);
        assert_eq!(bomb.ram[SPRITE_OAM_FLAGS + 15], 8);
        assert_eq!(bomb.ram[SPRITE_HEALTH + 15], 0);
        assert_eq!(bomb.ram[SPRITE_X_VEL + 15], 24);
        assert_eq!(bomb.ram[SPRITE_Z_VEL + 15], 48);

        let mut poof = fresh_state();
        poof.ram[SPRITE_STATE + k] = 9;
        poof.sprite_set_x(k, 0x0100);
        poof.sprite_set_y(k, 0x0200);
        assert_eq!(poof.spawn_boss_poof(k), 15);
        assert_eq!(poof.ram[SPRITE_TYPE + 15], 0xce);
        assert_eq!(poof.sprite_get_x(15), 0x0110);
        assert_eq!(poof.sprite_get_y(15), 0x0228);
        assert_eq!(poof.ram[SPRITE_GRAPHICS + 15], 0x0f);
        assert_eq!(poof.ram[SPRITE_A + 15], 1);
        assert_eq!(poof.ram[SPRITE_DELAY_MAIN + 15], 47);
        assert_eq!(poof.ram[SPRITE_FLAGS2 + 15], 9);
        assert_eq!(poof.ram[SPRITE_IGNORE_PROJECTILE + 15], 9);
        assert_eq!(poof.ram[SOUND_EFFECT_1], 12);

        let mut fireball = fresh_state();
        fireball.ram[SPRITE_STATE + k] = 9;
        fireball.sprite_set_x(k, 0x0100);
        fireball.sprite_set_y(k, 0x0200);
        fireball.ram[SPRITE_Z + k] = 16;
        write_le_u16(&mut fireball.ram, LINK_X_COORD, 0x0124);
        write_le_u16(&mut fireball.ram, LINK_Y_COORD, 0x01ec);
        assert_eq!(fireball.sprite_spawn_fireball(k), 13);
        assert_eq!(fireball.ram[SPRITE_TYPE + 13], 0x55);
        assert_eq!(fireball.sprite_get_x(13), 0x0104);
        assert_eq!(fireball.sprite_get_y(13), 0x01f4);
        assert_eq!(fireball.ram[SPRITE_FLAGS3 + 13], 0x42);
        assert_eq!(fireball.ram[SPRITE_OAM_FLAGS + 13], 6);
        assert_eq!(fireball.ram[SPRITE_FLAGS4 + 13], 0x54);
        assert_eq!(fireball.ram[SPRITE_E + 13], 0x54);
        assert_eq!(fireball.ram[SPRITE_FLAGS2 + 13], 0x20);
        assert_eq!(fireball.ram[SPRITE_X_VEL + 13], 0x20);
        assert_eq!(fireball.ram[SPRITE_Y_VEL + 13], 0);
        assert_eq!(fireball.ram[SPRITE_DELAY_MAIN + 13], 20);
        assert_eq!(fireball.ram[SPRITE_DELAY_AUX1 + 13], 16);
        assert_eq!(fireball.ram[SPRITE_FLAGS5 + 13], 0);
        assert_eq!(fireball.ram[SPRITE_DEFL_BITS + 13], 0x48);
        assert_eq!(fireball.ram[SOUND_EFFECT_2] & 0x3f, 0x19);

        let mut phlegm = fresh_state();
        phlegm.ram[SPRITE_STATE + k] = 9;
        phlegm.sprite_set_x(k, 0x0040);
        phlegm.sprite_set_y(k, 0x0060);
        phlegm.ram[SPRITE_Z + k] = 7;
        phlegm.ram[SPRITE_D + k] = 1;
        phlegm.ram[LINK_SHIELD_TYPE] = 3;
        assert_eq!(phlegm.sprite_spawn_fire_phlegm(k), 15);
        assert_eq!(phlegm.ram[SPRITE_TYPE + 15], 0xa5);
        assert_eq!(phlegm.sprite_get_x(15), 0x0038);
        assert_eq!(phlegm.sprite_get_y(15), 0x005e);
        assert_eq!(phlegm.ram[SPRITE_X_VEL + 15], (-48i8) as u8);
        assert_eq!(phlegm.ram[SPRITE_Y_VEL + 15], 0);
        assert_eq!(phlegm.ram[SPRITE_FLAGS3 + 15] & 0x40, 0x40);
        assert_eq!(phlegm.ram[SPRITE_DEFL_BITS + 15], 0x40);
        assert_eq!(phlegm.ram[SPRITE_FLAGS2 + 15], 0x21);
        assert_eq!(phlegm.ram[SPRITE_B + 15], 0x21);
        assert_eq!(phlegm.ram[SPRITE_OAM_FLAGS + 15], 2);
        assert_eq!(phlegm.ram[SPRITE_FLAGS4 + 15], 0x14);
        assert_eq!(phlegm.ram[SPRITE_IGNORE_PROJECTILE + 15], 20);
        assert_eq!(phlegm.ram[SPRITE_BUMP_DAMAGE + 15], 37);
        assert_eq!(phlegm.ram[SPRITE_FLAGS5 + 15], 0x20);
        assert_eq!(phlegm.ram[SOUND_EFFECT_2] & 0x3f, 5);

        let mut leaves = fresh_state();
        leaves.ram[SPRITE_STATE + k] = 9;
        leaves.sprite_set_x(k, 0x0120);
        leaves.sprite_set_y(k, 0x0340);
        leaves.ram[SPRITE_Z_VEL + k] = 0x24;
        assert_eq!(leaves.lumberjack_tree_spawn_leaves(k), 15);
        assert_eq!(leaves.ram[SPRITE_TYPE + 15], 0x3b);
        assert_eq!(leaves.sprite_get_x(15), 0x0120);
        assert_eq!(leaves.sprite_get_y(15), 0x0340);
        assert_eq!(leaves.ram[SPRITE_GRAPHICS + 15], 2);
        assert_eq!(leaves.ram[SPRITE_Z_VEL + 15], 0x24);
        assert_eq!(leaves.ram[SPRITE_SUBTYPE2 + 15], 1);
        assert_eq!(leaves.ram[SPRITE_AI_STATE + 15], 2);
        assert_eq!(leaves.ram[SPRITE_DELAY_MAIN + 15], 8);

        let mut garnish_poof = fresh_state();
        garnish_poof.sprite_set_x(k, 0x0234);
        garnish_poof.sprite_set_y(k, 0x0456);
        garnish_poof.ram[SPRITE_FLOOR + k] = 2;
        garnish_poof.sprite_spawn_poof_garnish(k);
        assert_eq!(garnish_poof.ram[GARNISH_TYPE + 29], 10);
        assert_eq!(garnish_poof.ram[GARNISH_ACTIVE_PREP], 10);
        assert_eq!(garnish_poof.ram[GARNISH_X_LO_PREP + 29], 0x34);
        assert_eq!(garnish_poof.ram[GARNISH_X_HI_PREP + 29], 0x02);
        assert_eq!(garnish_poof.ram[GARNISH_Y_LO_PREP + 29], 0x66);
        assert_eq!(garnish_poof.ram[GARNISH_Y_HI_PREP + 29], 0x04);
        assert_eq!(garnish_poof.ram[GARNISH_SPRITE_PREP + 29], 2);
        assert_eq!(garnish_poof.ram[GARNISH_COUNTDOWN_PREP + 29], 15);

        let mut octorok = fresh_state();
        octorok.ram[SPRITE_STATE + k] = 9;
        octorok.sprite_set_x(k, 0x0100);
        octorok.sprite_set_y(k, 0x0200);
        octorok.ram[SPRITE_D + k] = 0;
        octorok.octorok_fire_loogie(k);
        assert_eq!(octorok.ram[SPRITE_TYPE + 15], 0x0c);
        assert_eq!(octorok.sprite_get_x(15), 0x010c);
        assert_eq!(octorok.sprite_get_y(15), 0x0204);
        assert_eq!(octorok.ram[SPRITE_X_VEL + 15], 44);
        assert_eq!(octorok.ram[SPRITE_Y_VEL + 15], 0);
        assert_eq!(octorok.ram[SOUND_EFFECT_1] & 0x3f, 7);

        let mut moblin = fresh_state();
        moblin.ram[SPRITE_STATE + k] = 9;
        moblin.sprite_set_x(k, 0x0200);
        moblin.sprite_set_y(k, 0x0100);
        moblin.ram[SPRITE_D + k] = 3;
        moblin.moblin_materialize_spear(k);
        assert_eq!(moblin.ram[SPRITE_TYPE + 15], 0x1b);
        assert_eq!(moblin.ram[SPRITE_A + 15], 3);
        assert_eq!(moblin.ram[SPRITE_D + 15], 3);
        assert_eq!(moblin.sprite_get_x(15), 0x020b);
        assert_eq!(moblin.sprite_get_y(15), 0x00f5);
        assert_eq!(moblin.ram[SPRITE_X_VEL + 15], 0);
        assert_eq!(moblin.ram[SPRITE_Y_VEL + 15], (-32i8) as u8);

        let mut snitch = fresh_state();
        snitch.ram[SPRITE_STATE + k] = 9;
        snitch.ram[SPRITE_TYPE + k] = 0x35;
        write_le_u16(&mut snitch.ram, SPRCOLL_X_BASE_PREP, 0x1200);
        write_le_u16(&mut snitch.ram, SPRCOLL_Y_BASE_PREP, 0x3400);
        snitch.snitch_spawn_guard(k);
        assert_eq!(snitch.ram[SPRITE_TYPE], 0x45);
        assert_eq!(snitch.ram[SPRITE_STATE], 9);
        assert_eq!(snitch.sprite_get_x(0), 0x1540);
        assert_eq!(snitch.sprite_get_y(0), 0x37b0);
        assert_eq!(snitch.ram[SPRITE_FLOOR], 0);
        assert_eq!(snitch.ram[SPRITE_HEALTH], 4);
        assert_eq!(snitch.ram[SPRITE_DEFL_BITS], 0x80);
        assert_eq!(snitch.ram[SPRITE_FLAGS5], 0x90);
        assert_eq!(snitch.ram[SPRITE_OAM_FLAGS], 0x0b);

        let mut sparkle = fresh_state();
        for (idx, ty) in [0x2a, 0x21, 0x30, 0x19, 0x0c].into_iter().enumerate() {
            sparkle.ram[ANCILLA_TYPE + idx] = ty;
        }
        sparkle.ancilla_terminate_sparkle_objects();
        assert_eq!(sparkle.ram[ANCILLA_TYPE], 0);
        assert_eq!(sparkle.ram[ANCILLA_TYPE + 1], 0x21);
        assert_eq!(sparkle.ram[ANCILLA_TYPE + 2], 0);
        assert_eq!(sparkle.ram[ANCILLA_TYPE + 3], 0);
        assert_eq!(sparkle.ram[ANCILLA_TYPE + 4], 0);

        let mut kodongo = fresh_state();
        kodongo.ram[SPRITE_D + k] = 2;
        kodongo.kodongo_set_direction(k);
        assert_eq!(kodongo.ram[SPRITE_X_VEL + k], 0);
        assert_eq!(kodongo.ram[SPRITE_Y_VEL + k], 16);

        let mut kodongo_fire = fresh_state();
        kodongo_fire.ram[SPRITE_STATE + k] = 9;
        kodongo_fire.sprite_set_x(k, 0x0300);
        kodongo_fire.sprite_set_y(k, 0x0040);
        kodongo_fire.ram[SPRITE_D + k] = 1;
        kodongo_fire.kodongo_spawn_fire(k);
        assert_eq!(kodongo_fire.ram[SPRITE_TYPE + 13], 0x87);
        assert_eq!(kodongo_fire.sprite_get_x(13), 0x02f8);
        assert_eq!(kodongo_fire.sprite_get_y(13), 0x0040);
        assert_eq!(kodongo_fire.ram[SPRITE_X_VEL + 13], (-24i8) as u8);
        assert_eq!(kodongo_fire.ram[SPRITE_Y_VEL + 13], 0);
        assert_eq!(kodongo_fire.ram[SPRITE_IGNORE_PROJECTILE + 13], 1);

        let mut blue_balls = fresh_state();
        blue_balls.ram[SPRITE_STATE + k] = 9;
        blue_balls.sprite_set_x(k, 0x0120);
        blue_balls.sprite_set_y(k, 0x0340);
        blue_balls.create_six_blue_balls(k);
        assert_eq!(blue_balls.ram[SOUND_EFFECT_2] & 0x3f, 0x36);
        assert_eq!(blue_balls.ram[TMP_COUNTER], 0);
        assert_eq!(blue_balls.ram[SPRITE_TYPE + 15], 0x55);
        assert_eq!(blue_balls.sprite_get_x(15), 0x0124);
        assert_eq!(blue_balls.sprite_get_y(15), 0x0344);
        assert_eq!(blue_balls.ram[SPRITE_FLAGS3 + 15], 0x42);
        assert_eq!(blue_balls.ram[SPRITE_OAM_FLAGS + 15], 4);
        assert_eq!(blue_balls.ram[SPRITE_DELAY_AUX1 + 15], 4);
        assert_eq!(blue_balls.ram[SPRITE_FLAGS4 + 15], 20);
        assert_eq!(blue_balls.ram[SPRITE_C + 15], 20);
        assert_eq!(blue_balls.ram[SPRITE_E + 15], 20);
        assert_eq!(blue_balls.ram[SPRITE_X_VEL + 15], (-24i8) as u8);
        assert_eq!(blue_balls.ram[SPRITE_Y_VEL + 15], (-16i8) as u8);
        assert_eq!(blue_balls.ram[SPRITE_TYPE + 10], 0x55);
        assert_eq!(blue_balls.ram[SPRITE_X_VEL + 10], 0);
        assert_eq!(blue_balls.ram[SPRITE_Y_VEL + 10], (-32i8) as u8);

        let mut octoballoon = fresh_state();
        octoballoon.ram[SPRITE_STATE + k] = 9;
        octoballoon.sprite_set_x(k, 0x0110);
        octoballoon.sprite_set_y(k, 0x0220);
        octoballoon.octoballoon_form_babby(k);
        assert_eq!(octoballoon.ram[SOUND_EFFECT_1] & 0x3f, 0x0c);
        assert_eq!(octoballoon.ram[SPRITE_TYPE + 15], 0x10);
        assert_eq!(octoballoon.sprite_get_x(15), 0x0110);
        assert_eq!(octoballoon.sprite_get_y(15), 0x0220);
        assert_eq!(octoballoon.ram[SPRITE_X_VEL + 15], 11);
        assert_eq!(octoballoon.ram[SPRITE_Y_VEL + 15], (-11i8) as u8);
        assert_eq!(octoballoon.ram[SPRITE_Z_VEL + 15], 48);
        assert_eq!(octoballoon.ram[SPRITE_SUBTYPE2 + 15], 255);
        assert_eq!(octoballoon.ram[SPRITE_TYPE + 10], 0x10);
        assert_eq!(octoballoon.ram[SPRITE_X_VEL + 10], 16);
        assert_eq!(octoballoon.ram[SPRITE_Y_VEL + 10], 0);

        let mut bully = fresh_state();
        bully.ram[SPRITE_STATE + k] = 9;
        bully.sprite_set_x(k, 0x0440);
        bully.sprite_set_y(k, 0x0550);
        bully.ball_guy_play_bounce_noise(k);
        assert_eq!(bully.ram[SOUND_EFFECT_2] & 0x3f, 0x32);
        bully.spawn_bully(k);
        assert_eq!(bully.ram[SPRITE_TYPE + 15], 0xb9);
        assert_eq!(bully.sprite_get_x(15), 0x0440);
        assert_eq!(bully.sprite_get_y(15), 0x0550);
        assert_eq!(bully.ram[SPRITE_SUBTYPE2 + 15], 2);
        assert_eq!(bully.ram[SPRITE_HEAD_DIR + 15], k as u8);
        assert_eq!(bully.ram[SPRITE_IGNORE_PROJECTILE + 15], 1);

        let mut rupees = fresh_state();
        rupees.ram[SPRITE_STATE + k] = 9;
        rupees.sprite_set_x(k, 0x0180);
        rupees.sprite_set_y(k, 0x0280);
        rupees.ram[NUM_SPRITES_KILLED_PREP] = 4;
        rupees.ram[NUMBER_OF_TIMES_HURT_BY_SPRITES] = 0;
        rupees.rupee_pull_spawn_prize(k);
        assert_eq!(rupees.ram[SPRITE_SHARED_SCRATCH_A], 2);
        assert_eq!(rupees.ram[TMP_COUNTER], 0xff);
        assert_eq!(rupees.ram[NUM_SPRITES_KILLED_PREP], 0);
        assert_eq!(rupees.ram[NUMBER_OF_TIMES_HURT_BY_SPRITES], 0);
        assert_eq!(rupees.ram[SPRITE_TYPE + 15], 0xdb);
        assert_eq!(rupees.sprite_get_x(15), 0x0180);
        assert_eq!(rupees.sprite_get_y(15), 0x0280);
        assert_eq!(rupees.ram[SPRITE_X_VEL + 15], 18);
        assert_eq!(rupees.ram[SPRITE_Y_VEL + 15], 16);
        assert_eq!(rupees.ram[SPRITE_STUNNED + 15], 255);
        assert_eq!(rupees.ram[SPRITE_DELAY_AUX4 + 15], 32);
        assert_eq!(rupees.ram[SPRITE_DELAY_AUX3_PREP + 15], 32);
        assert_eq!(rupees.ram[SPRITE_Z_VEL + 15], 32);
        assert_eq!(rupees.ram[SPRITE_TYPE + 12], 0xdb);
        assert_eq!(rupees.ram[SPRITE_X_VEL + 12], (-18i8) as u8);
        assert_eq!(rupees.ram[SPRITE_Y_VEL + 12], 16);

        let mut pink = fresh_state();
        pink.ram[SPRITE_X_VEL + k] = 10;
        pink.ram[SPRITE_Y_VEL + k] = (-10i8) as u8;
        pink.pink_ball_handle_deceleration(k);
        assert_eq!(pink.ram[SPRITE_X_VEL + k], 8);
        assert_eq!(pink.ram[SPRITE_Y_VEL + k], (-8i8) as u8);
        write_le_u16(&mut pink.ram, OAM_CUR_PTR, 0x0800);
        pink.sprite_set_x(k, 0x0100);
        pink.sprite_set_y(k, 0x0120);
        pink.ram[FRAME_COUNTER] = 0x18;
        pink.pink_ball_distress(k);
        assert_eq!(pink.ram[SPRITE_PAUSE + k], 0);

        let mut pink_msg = fresh_state();
        pink_msg.ram[SPRITE_D + k] = 3;
        pink_msg.ram[SPRITE_X_VEL + k] = 0x12;
        pink_msg.ram[SPRITE_Y_VEL + k] = 0x34;
        pink_msg.pink_ball_handle_message(k);
        assert_eq!(read_le_u16(&pink_msg.ram, DIALOGUE_MESSAGE_INDEX), 0x15b);
        assert_eq!(pink_msg.ram[SPRITE_X_VEL + k], 0xed);
        assert_eq!(pink_msg.ram[SPRITE_Y_VEL + k], 0xcb);
        assert_eq!(pink_msg.ram[SPRITE_DELAY_AUX4 + k], 64);
        pink_msg.ram[SPRITE_DELAY_AUX4 + k] = 0;
        pink_msg.ram[LINK_ITEM_MOON_PEARL] = 1;
        pink_msg.pink_ball_handle_message(k);
        assert_eq!(read_le_u16(&pink_msg.ram, DIALOGUE_MESSAGE_INDEX), 0x15c);

        let mut bully_msg = fresh_state();
        bully_msg.ram[SPRITE_D + k] = 2;
        bully_msg.ram[SPRITE_X_VEL + k] = 0x12;
        bully_msg.ram[SPRITE_Y_VEL + k] = 0x34;
        bully_msg.bully_handle_message(k);
        assert_eq!(read_le_u16(&bully_msg.ram, DIALOGUE_MESSAGE_INDEX), 0x15d);
        assert_eq!(bully_msg.ram[SPRITE_X_VEL + k], 0xed);
        assert_eq!(bully_msg.ram[SPRITE_Y_VEL + k], 0xcb);
        assert_eq!(bully_msg.ram[SPRITE_DELAY_AUX4 + k], 64);
        bully_msg.ram[SPRITE_DELAY_AUX4 + k] = 0;
        bully_msg.ram[LINK_ITEM_MOON_PEARL] = 1;
        bully_msg.bully_handle_message(k);
        assert_eq!(read_le_u16(&bully_msg.ram, DIALOGUE_MESSAGE_INDEX), 0x15e);

        let mut sasha = fresh_state();
        sasha.ram[SPRITE_STATE + k] = 9;
        sasha.ram[FRAME_COUNTER] = 0x20;
        sasha.sasha_idle(k);
        assert_eq!(read_le_u16(&sasha.ram, DIALOGUE_MESSAGE_INDEX), 0x32);
        assert_eq!(sasha.ram[SPRITE_GRAPHICS + k], 1);
        sasha.ram[LINK_WHICH_PENDANTS] = 4;
        sasha.ram[SAVEGAME_MAP_ICONS_INDICATOR] = 3;
        sasha.sasha_idle(k);
        assert_eq!(read_le_u16(&sasha.ram, DIALOGUE_MESSAGE_INDEX), 0x38);
        sasha.ram[LINK_ITEM_BOOTS] = 1;
        sasha.sasha_idle(k);
        assert_eq!(read_le_u16(&sasha.ram, DIALOGUE_MESSAGE_INDEX), 0x37);
        sasha.ram[LINK_ITEM_ICE_ROD] = 1;
        sasha.sasha_idle(k);
        assert_eq!(read_le_u16(&sasha.ram, DIALOGUE_MESSAGE_INDEX), 0x34);
        sasha.ram[LINK_WHICH_PENDANTS] = 7;
        sasha.sasha_idle(k);
        assert_eq!(read_le_u16(&sasha.ram, DIALOGUE_MESSAGE_INDEX), 0x30);
        sasha.ram[LINK_SWORD_TYPE] = 2;
        sasha.sasha_idle(k);
        assert_eq!(read_le_u16(&sasha.ram, DIALOGUE_MESSAGE_INDEX), 0x31);

        let mut old_man = fresh_state();
        let t = 2;
        old_man.ram[TAGALONG_LAYERBITS + t] = 2;
        old_man.ram[TAGALONG_Y_LO + t] = 0x40;
        old_man.ram[TAGALONG_Y_HI + t] = 0x03;
        old_man.ram[TAGALONG_X_LO + t] = 0x20;
        old_man.ram[TAGALONG_X_HI + t] = 0x04;
        old_man.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        old_man.ram[FOLLOWER_INDICATOR] = 6;
        old_man.ram[LINK_SPEED_SETTING] = 9;
        old_man.old_man_revert_to_sprite(t);
        assert_eq!(old_man.ram[SPRITE_TYPE + 15], 0xad);
        assert_eq!(old_man.ram[SPRITE_D + 15], 2);
        assert_eq!(old_man.ram[SPRITE_HEAD_DIR + 15], 2);
        assert_eq!(old_man.sprite_get_y(15), 0x0342);
        assert_eq!(old_man.sprite_get_x(15), 0x0422);
        assert_eq!(old_man.ram[SPRITE_FLOOR + 15], 1);
        assert_eq!(old_man.ram[SPRITE_IGNORE_PROJECTILE + 15], 1);
        assert_eq!(old_man.ram[SPRITE_SUBTYPE2 + 15], 1);
        assert_eq!(old_man.ram[FLAG_IS_LINK_IMMOBILIZED], 1);
        assert_eq!(old_man.ram[LINK_DISABLE_SPRITE_DAMAGE], 1);
        assert_eq!(old_man.ram[FOLLOWER_INDICATOR], 0);
        assert_eq!(old_man.ram[LINK_SPEED_SETTING], 0);

        let mut apple = fresh_state();
        apple.ram[SPRITE_STATE + k] = 9;
        apple.sprite_set_x(k, 0x0200);
        apple.sprite_set_y(k, 0x0300);
        apple.ram[FRAME_COUNTER] = 0;
        apple.ram[0x0fa1] = 0;
        apple.spawn_apple(k);
        assert_eq!(apple.ram[SPRITE_TYPE + 15], 0xac);
        assert_eq!(apple.sprite_get_x(15), 0x0200);
        assert_eq!(apple.sprite_get_y(15), 0x0300);
        assert_eq!(apple.ram[SPRITE_AI_STATE + 15], 1);
        assert_eq!(apple.ram[SPRITE_A + 15], 255);
        assert_eq!(apple.ram[SPRITE_Z + 15], 8);
        assert_eq!(apple.ram[SPRITE_Z_VEL + 15], 22);
        assert_eq!(apple.ram[SPRITE_X_VEL + 15], 10);
        assert_eq!(apple.ram[SPRITE_Y_VEL + 15], 3);

        let mut transmute = fresh_state();
        transmute.ram[SPRITE_TYPE + k] = 0xd8;
        transmute.ram[SPRITE_HEALTH + k] = 7;
        transmute.sprite_transmute_to_bomb(k);
        assert_eq!(transmute.ram[SPRITE_TYPE + k], 0x4a);
        assert_eq!(transmute.ram[SPRITE_C + k], 1);
        assert_eq!(transmute.ram[SPRITE_DELAY_AUX1 + k], 255);
        assert_eq!(transmute.ram[SPRITE_FLAGS3 + k], 0x18);
        assert_eq!(transmute.ram[SPRITE_OAM_FLAGS + k], 8);
        assert_eq!(transmute.ram[SPRITE_HEALTH + k], 0);

        let mut sluggula = fresh_state();
        sluggula.ram[SPRITE_STATE + k] = 9;
        sluggula.sprite_set_x(k, 0x0120);
        sluggula.sprite_set_y(k, 0x0340);
        sluggula.sluggula_drop_bomb(k);
        assert_eq!(sluggula.ram[SPRITE_TYPE + 11], 0x4a);
        assert_eq!(sluggula.sprite_get_x(11), 0x0120);
        assert_eq!(sluggula.sprite_get_y(11), 0x0340);
        assert_eq!(sluggula.ram[SPRITE_C + 11], 1);
        assert_eq!(sluggula.ram[SPRITE_DELAY_AUX1 + 11], 255);
        assert_eq!(sluggula.ram[SPRITE_FLAGS3 + 11], 0x18);
        assert_eq!(sluggula.ram[SPRITE_OAM_FLAGS + 11], 8);
        assert_eq!(sluggula.ram[SPRITE_HEALTH + 11], 0);

        let mut tree_bomb = fresh_state();
        tree_bomb.ram[SPRITE_STATE + k] = 9;
        tree_bomb.sprite_set_x(k, 0x0048);
        tree_bomb.sprite_set_y(k, 0x0058);
        tree_bomb.talking_tree_spawn_bomb(k);
        assert_eq!(tree_bomb.ram[SPRITE_TYPE + 15], 0x4a);
        assert_eq!(tree_bomb.sprite_get_x(15), 0x0048);
        assert_eq!(tree_bomb.sprite_get_y(15), 0x0058);
        assert_eq!(tree_bomb.ram[SPRITE_C + 15], 1);
        assert_eq!(tree_bomb.ram[SPRITE_DELAY_AUX1 + 15], 64);
        assert_eq!(tree_bomb.ram[SPRITE_FLAGS3 + 15], 0x18);
        assert_eq!(tree_bomb.ram[SPRITE_OAM_FLAGS + 15], 8);
        assert_eq!(tree_bomb.ram[SPRITE_HEALTH + 15], 0);
        assert_eq!(tree_bomb.ram[SPRITE_Y_VEL + 15], 24);
        assert_eq!(tree_bomb.ram[SPRITE_Z_VEL + 15], 18);

        let mut tree_eye = fresh_state();
        tree_eye.ram[SPRITE_STATE + k] = 9;
        tree_eye.sprite_set_x(k, 0x0200);
        tree_eye.sprite_set_y(k, 0x0300);
        tree_eye.sprite_prep_talking_tree_spawn_eyeball(k, 1);
        assert_eq!(tree_eye.ram[SPRITE_TYPE + 15], 0x25);
        assert_eq!(tree_eye.ram[SPRITE_HEAD_DIR + 15], 1);
        assert_eq!(tree_eye.sprite_get_x(15), 0x020e);
        assert_eq!(tree_eye.sprite_get_y(15), 0x02f5);
        assert_eq!(tree_eye.ram[SPRITE_A + 15], 0x0e);
        assert_eq!(tree_eye.ram[SPRITE_B + 15], 0x02);
        assert_eq!(tree_eye.ram[SPRITE_C + 15], 0xf5);
        assert_eq!(tree_eye.ram[SPRITE_E + 15], 0x02);
        assert_eq!(tree_eye.ram[SPRITE_SUBTYPE2 + 15], 1);

        let mut pirogusu = fresh_state();
        pirogusu.ram[FRAME_COUNTER] = k as u8;
        pirogusu.ram[GARNISH_TYPE + 14] = 1;
        pirogusu.sprite_set_x(k, 0x0110);
        pirogusu.sprite_set_y(k, 0x0220);
        pirogusu.pirogusu_spawn_splash(k);
        assert_eq!(pirogusu.ram[GARNISH_TYPE + 13], 11);
        assert_eq!(pirogusu.ram[GARNISH_ACTIVE_PREP], 11);
        assert_eq!(pirogusu.ram[GARNISH_X_LO_PREP + 13], 0x15);
        assert_eq!(pirogusu.ram[GARNISH_X_HI_PREP + 13], 0x01);
        assert_eq!(pirogusu.ram[GARNISH_Y_LO_PREP + 13], 0x34);
        assert_eq!(pirogusu.ram[GARNISH_Y_HI_PREP + 13], 0x02);
        assert_eq!(pirogusu.ram[GARNISH_COUNTDOWN_PREP + 13], 15);

        let mut lightning = fresh_state();
        lightning.ram[GARNISH_TYPE + 29] = 1;
        lightning.sprite_set_x(k, 0x0123);
        lightning.sprite_set_y(k, 0x02f4);
        lightning.ram[SPRITE_A + k] = 7;
        lightning.lightning_spawn_garnish(k);
        assert_eq!(lightning.ram[GARNISH_TYPE + 28], 9);
        assert_eq!(lightning.ram[GARNISH_ACTIVE_PREP], 9);
        assert_eq!(lightning.ram[GARNISH_SPRITE_PREP + 28], 7);
        assert_eq!(lightning.ram[GARNISH_X_LO_PREP + 28], 0x23);
        assert_eq!(lightning.ram[GARNISH_X_HI_PREP + 28], 0x01);
        assert_eq!(lightning.ram[GARNISH_Y_LO_PREP + 28], 0x04);
        assert_eq!(lightning.ram[GARNISH_Y_HI_PREP + 28], 0x03);
        assert_eq!(lightning.ram[GARNISH_COUNTDOWN_PREP + 28], 32);

        let mut laser = fresh_state();
        laser.ram[GARNISH_TYPE + 29] = 1;
        laser.sprite_set_x(k, 0x0034);
        laser.sprite_set_y(k, 0x00f0);
        laser.ram[SPRITE_GRAPHICS + k] = 5;
        laser.ram[SPRITE_FLOOR + k] = 2;
        laser.laser_beam_build_up_garnish(k);
        assert_eq!(laser.ram[GARNISH_TYPE + 28], 4);
        assert_eq!(laser.ram[GARNISH_ACTIVE_PREP], 4);
        assert_eq!(laser.ram[GARNISH_X_LO_PREP + 28], 0x34);
        assert_eq!(laser.ram[GARNISH_X_HI_PREP + 28], 0x00);
        assert_eq!(laser.ram[GARNISH_Y_LO_PREP + 28], 0x00);
        assert_eq!(laser.ram[GARNISH_Y_HI_PREP + 28], 0x01);
        assert_eq!(laser.ram[GARNISH_COUNTDOWN_PREP + 28], 16);
        assert_eq!(laser.ram[GARNISH_OAM_FLAGS_PREP + 28], 5);
        assert_eq!(laser.ram[GARNISH_SPRITE_PREP + 28], k as u8);
        assert_eq!(laser.ram[GARNISH_FLOOR_PREP + 28], 2);

        let mut logic = fresh_state();
        assert!(!logic.octoballoon_find());
        logic.ram[SPRITE_STATE + 10] = 9;
        logic.ram[SPRITE_TYPE + 10] = 0x10;
        assert!(logic.octoballoon_find());

        assert!(!logic.potion_cauldron_check_bottles());
        logic.ram[LINK_BOTTLE_INFO + 2] = 2;
        assert!(logic.potion_cauldron_check_bottles());
        logic.potion_cauldron_go_beep(k);
        assert_eq!(logic.ram[SOUND_EFFECT_1] & 0x3f, 0x3c);

        write_le_u16(&mut logic.ram, LINK_RUPEES_GOAL, 19);
        assert!(!logic.dark_world_hint_npc_handle_payment());
        assert_eq!(read_le_u16(&logic.ram, LINK_RUPEES_GOAL), 19);
        write_le_u16(&mut logic.ram, LINK_RUPEES_GOAL, 20);
        assert!(logic.dark_world_hint_npc_handle_payment());
        assert_eq!(read_le_u16(&logic.ram, LINK_RUPEES_GOAL), 0);
        logic.ram[SPRITE_AI_STATE + k] = 0;
        logic.dark_world_hint_npc_idle(k);
        assert_eq!(read_le_u16(&logic.ram, DIALOGUE_MESSAGE_INDEX), 0xfe);
        assert_eq!(logic.ram[SPRITE_AI_STATE + k], 0);

        logic.ram[SUBMODULE_INDEX] = 2;
        write_le_u16(&mut logic.ram, DIALOGUE_MESSAGE_INDEX, 0xc9);
        logic.fairy_check_if_touchable(k);
        assert_eq!(logic.ram[SPRITE_DELAY_AUX4 + k], 40);
        logic.ram[SPRITE_DELAY_AUX4 + k] = 0;
        write_le_u16(&mut logic.ram, DIALOGUE_MESSAGE_INDEX, 0xcb);
        logic.fairy_check_if_touchable(k);
        assert_eq!(logic.ram[SPRITE_DELAY_AUX4 + k], 0);

        let mut buzzblob = fresh_state();
        buzzblob.buzzblob_select_new_direction(k);
        assert_eq!(buzzblob.ram[SPRITE_X_VEL + k], 3);
        assert_eq!(buzzblob.ram[SPRITE_Y_VEL + k], 0);
        assert_eq!(buzzblob.ram[SPRITE_DELAY_MAIN + k], 48);

        let mut lumberjack = fresh_state();
        write_le_u16(&mut lumberjack.ram, CUR_SPRITE_X, 0x0100);
        write_le_u16(&mut lumberjack.ram, CUR_SPRITE_Y, 0x0200);
        write_le_u16(&mut lumberjack.ram, LINK_X_COORD, 0x0100);
        write_le_u16(&mut lumberjack.ram, LINK_Y_COORD, 0x0200);
        assert!(lumberjack.lumberjack_check_proximity(k, 0));
        write_le_u16(&mut lumberjack.ram, LINK_X_COORD, 0x0200);
        assert!(!lumberjack.lumberjack_check_proximity(k, 0));

        let mut blind_laser = fresh_state();
        blind_laser.ram[GARNISH_TYPE + 29] = 1;
        blind_laser.sprite_set_x(k, 0x0456);
        blind_laser.sprite_set_y(k, 0x0789);
        blind_laser.ram[SPRITE_GRAPHICS + k] = 6;
        blind_laser.blind_laser_spawn_trail_garnish(k);
        assert_eq!(blind_laser.ram[GARNISH_TYPE + 28], 15);
        assert_eq!(blind_laser.ram[GARNISH_ACTIVE_PREP], 15);
        assert_eq!(blind_laser.ram[GARNISH_OAM_FLAGS_PREP + 28], 6);
        assert_eq!(blind_laser.ram[GARNISH_SPRITE_PREP + 28], k as u8);
        assert_eq!(blind_laser.ram[GARNISH_X_LO_PREP + 28], 0x56);
        assert_eq!(blind_laser.ram[GARNISH_X_HI_PREP + 28], 0x04);
        assert_eq!(blind_laser.ram[GARNISH_Y_LO_PREP + 28], 0x99);
        assert_eq!(blind_laser.ram[GARNISH_Y_HI_PREP + 28], 0x07);
        assert_eq!(blind_laser.ram[GARNISH_COUNTDOWN_PREP + 28], 10);

        let mut runner_dust = fresh_state();
        runner_dust.ram[SPRITE_DIE_ACTION + k] = 14;
        runner_dust.running_boy_spawn_dust_garnish(k);
        assert_eq!(runner_dust.ram[GARNISH_ACTIVE_PREP], 0);
        runner_dust.ram[SPRITE_DIE_ACTION + k] = 15;
        runner_dust.sprite_set_x(k, 0x0100);
        runner_dust.sprite_set_y(k, 0x0200);
        runner_dust.ram[GARNISH_TYPE + 29] = 1;
        runner_dust.running_boy_spawn_dust_garnish(k);
        assert_eq!(runner_dust.ram[GARNISH_TYPE + 28], 20);
        assert_eq!(runner_dust.ram[GARNISH_ACTIVE_PREP], 20);
        assert_eq!(runner_dust.ram[GARNISH_X_LO_PREP + 28], 0x04);
        assert_eq!(runner_dust.ram[GARNISH_X_HI_PREP + 28], 0x01);
        assert_eq!(runner_dust.ram[GARNISH_Y_LO_PREP + 28], 0x1c);
        assert_eq!(runner_dust.ram[GARNISH_Y_HI_PREP + 28], 0x02);
        assert_eq!(runner_dust.ram[GARNISH_COUNTDOWN_PREP + 28], 10);

        let mut cd = fresh_state();
        cd.ram[SPRITE_SUBTYPE2 + k] = 6;
        cd.sprite_cd_spawn_garnish(k);
        assert_eq!(cd.ram[GARNISH_ACTIVE_PREP], 0);
        cd.ram[SPRITE_SUBTYPE2 + k] = 7;
        cd.ram[GARNISH_TYPE + 29] = 1;
        cd.sprite_set_x(k, 0x0033);
        cd.sprite_set_y(k, 0x0044);
        cd.sprite_cd_spawn_garnish(k);
        assert_eq!(cd.ram[SPRITE_SUBTYPE2 + k], 8);
        assert_eq!(cd.ram[SOUND_EFFECT_2] & 0x3f, 0x14);
        assert_eq!(cd.ram[GARNISH_TYPE + 28], 0x0c);
        assert_eq!(cd.ram[GARNISH_ACTIVE_PREP], 0x0c);
        assert_eq!(cd.ram[GARNISH_SPRITE_PREP + 28], k as u8);
        assert_eq!(cd.ram[GARNISH_X_LO_PREP + 28], 0x33);
        assert_eq!(cd.ram[GARNISH_Y_LO_PREP + 28], 0x54);
        assert_eq!(cd.ram[GARNISH_COUNTDOWN_PREP + 28], 127);

        let mut hint = fresh_state();
        hint.ram[SPRITE_AI_STATE + k] = 2;
        hint.dark_world_hint_npc_restore_health(k);
        assert_eq!(hint.ram[LINK_HEARTS_FILLER], 0xa0);
        assert_eq!(hint.ram[SPRITE_AI_STATE + k], 0);

        let mut pipe = fresh_state();
        pipe.ram[LINK_POSITION_MODE] = 7;
        pipe.ram[LINK_CANT_CHANGE_DIRECTION] = 9;
        pipe.ram[ANCILLA_TYPE + 3] = 0x31;
        assert!(!pipe.pipe_validate_entry());
        assert_eq!(pipe.ram[LINK_POSITION_MODE], 0);
        assert_eq!(pipe.ram[LINK_CANT_CHANGE_DIRECTION], 0);
        assert_eq!(pipe.ram[ANCILLA_TYPE + 3], 0);
        pipe.ram[LINK_STATE_BITS] = 0x80;
        assert!(pipe.pipe_validate_entry());
        pipe.ram[LINK_STATE_BITS] = 0;
        pipe.ram[LINK_AUXILIARY_STATE] = 2;
        assert!(pipe.pipe_validate_entry());

        let mut hobo_smoke = fresh_state();
        hobo_smoke.ram[SPRITE_STATE + k] = 9;
        hobo_smoke.sprite_set_x(k, 0x0030);
        hobo_smoke.sprite_set_y(k, 0x0040);
        hobo_smoke.sprite_prep_hobo_spawn_smoke(k);
        assert_eq!(hobo_smoke.ram[SPRITE_TYPE + 15], 0x2b);
        assert_eq!(hobo_smoke.sprite_get_x(15), 0x0030);
        assert_eq!(hobo_smoke.sprite_get_y(15), 0x0040);
        assert_eq!(hobo_smoke.ram[SPRITE_SUBTYPE2 + 15], 0);
        assert_eq!(hobo_smoke.ram[SPRITE_IGNORE_PROJECTILE + 15], 0);

        let mut hobo_fire = fresh_state();
        hobo_fire.ram[SPRITE_STATE + k] = 9;
        hobo_fire.ram[SPRITE_OAM_FLAGS + 15] = 0xff;
        hobo_fire.sprite_prep_hobo_spawn_fire(k);
        assert_eq!(hobo_fire.ram[SPRITE_TYPE + 15], 0x2b);
        assert_eq!(hobo_fire.sprite_get_x(15), 0x0194);
        assert_eq!(hobo_fire.sprite_get_y(15), 0x003f);
        assert_eq!(hobo_fire.ram[SPRITE_SUBTYPE2 + 15], 2);
        assert_eq!(hobo_fire.ram[SPRITE_IGNORE_PROJECTILE + 15], 2);
        assert_eq!(hobo_fire.ram[SPRITE_FLAGS2 + 15], 0);
        assert_eq!(hobo_fire.ram[SPRITE_OAM_FLAGS + 15] & 0x0f, 0x03);

        let mut hobo_bubble = fresh_state();
        hobo_bubble.ram[SPRITE_STATE + k] = 9;
        hobo_bubble.sprite_set_x(k, 0x0050);
        hobo_bubble.sprite_set_y(k, 0x0060);
        assert_eq!(hobo_bubble.hobo_spawn_bubble(k), 15);
        assert_eq!(hobo_bubble.ram[SPRITE_TYPE + 15], 0x2b);
        assert_eq!(hobo_bubble.sprite_get_x(15), 0x0050);
        assert_eq!(hobo_bubble.sprite_get_y(15), 0x0060);
        assert_eq!(hobo_bubble.ram[SPRITE_SUBTYPE2 + 15], 1);
        assert_eq!(hobo_bubble.ram[SPRITE_Z_VEL + 15], 2);
        assert_eq!(hobo_bubble.ram[SPRITE_DELAY_MAIN + 15], 96);
        assert_eq!(hobo_bubble.ram[SPRITE_DELAY_AUX1 + 15], 48);
        assert_eq!(hobo_bubble.ram[SPRITE_IGNORE_PROJECTILE + 15], 48);
        assert_eq!(hobo_bubble.ram[SPRITE_FLAGS2 + 15], 0);

        let mut hobo_smoke_active = fresh_state();
        hobo_smoke_active.ram[SPRITE_STATE + k] = 9;
        hobo_smoke_active.sprite_set_x(k, 0x0070);
        hobo_smoke_active.sprite_set_y(k, 0x0080);
        hobo_smoke_active.hobo_spawn_smoke(k);
        assert_eq!(hobo_smoke_active.ram[SPRITE_TYPE + 15], 0x2b);
        assert_eq!(hobo_smoke_active.sprite_get_x(15), 0x0070);
        assert_eq!(hobo_smoke_active.sprite_get_y(15), 0x007c);
        assert_eq!(hobo_smoke_active.ram[SPRITE_SUBTYPE2 + 15], 3);
        assert_eq!(hobo_smoke_active.ram[SPRITE_Z_VEL + 15], 7);
        assert_eq!(hobo_smoke_active.ram[SPRITE_DELAY_MAIN + 15], 96);
        assert_eq!(hobo_smoke_active.ram[SPRITE_IGNORE_PROJECTILE + 15], 96);
        assert_eq!(hobo_smoke_active.ram[SPRITE_FLAGS2 + 15], 0);

        let mut hobo = fresh_state();
        hobo.ram[SPRITE_STATE + k] = 9;
        hobo.sprite_set_x(k, 0x0080);
        hobo.sprite_set_y(k, 0x0090);
        hobo.ram[SRAM_PROGRESS_INDICATOR_3_PREP] = 1;
        hobo.sprite_prep_hobo(k);
        assert_eq!(hobo.ram[SPRITE_AI_STATE], 3);
        assert_eq!(hobo.ram[SPRITE_IGNORE_PROJECTILE], 1);
        assert_eq!(hobo.ram[SPRITE_STATE + 15], 9);
        assert_eq!(hobo.ram[SPRITE_STATE + 1], 0);
        assert_eq!(hobo.ram[SPRITE_TYPE + 15], 0x2b);
        assert_eq!(hobo.ram[SPRITE_SUBTYPE2 + 15], 2);
        assert_eq!(hobo.sprite_get_x(15), 0x0194);
        assert_eq!(hobo.sprite_get_y(15), 0x003f);

        let mut tree = fresh_state();
        tree.ram[SPRITE_STATE + k] = 9;
        tree.sprite_set_x(k, 0x0120);
        tree.sprite_set_y(k, 0x0240);
        tree.sprite_prep_talking_tree(k);
        assert_eq!(tree.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(tree.sprite_get_x(k), 0x0118);
        assert_eq!(tree.ram[SPRITE_TYPE + 15], 0x25);
        assert_eq!(tree.ram[SPRITE_HEAD_DIR + 15], 0);
        assert_eq!(tree.sprite_get_x(15), 0x0114);
        assert_eq!(tree.sprite_get_y(15), 0x0235);
        assert_eq!(tree.ram[SPRITE_SUBTYPE2 + 15], 1);
        assert_eq!(tree.ram[SPRITE_TYPE + 14], 0x25);
        assert_eq!(tree.ram[SPRITE_HEAD_DIR + 14], 1);
        assert_eq!(tree.sprite_get_x(14), 0x0126);
        assert_eq!(tree.sprite_get_y(14), 0x0235);
        assert_eq!(tree.ram[SPRITE_A + 14], 0x26);
        assert_eq!(tree.ram[SPRITE_B + 14], 0x01);
        assert_eq!(tree.ram[SPRITE_C + 14], 0x35);
        assert_eq!(tree.ram[SPRITE_E + 14], 0x02);
    }

    #[test]
    fn shopkeeper_and_antifairy_circle_prep_spawn_expected_helpers() {
        let k = 4;

        let mut shop = fresh_state();
        shop.ram[SPRITE_STATE + k] = 9;
        shop.ram[DUNGEON_ROOM_INDEX] = 0x0f;
        shop.sprite_set_x(k, 0x0200);
        shop.sprite_set_y(k, 0x0100);
        shop.sprite_prep_shopkeeper(k);
        assert_eq!(shop.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(shop.ram[SPRITE_FLAGS2 + k] & 2, 2);
        assert_eq!(shop.ram[SPRITE_OAM_FLAGS + k] & 12, 12);
        assert_eq!(shop.ram[SPRITE_FLAGS3 + k] & 16, 16);
        for (slot, what, x) in [
            (12, 7, 0x0200u16.wrapping_sub(44)),
            (11, 8, 0x0200u16.wrapping_add(8)),
            (10, 12, 0x0200u16.wrapping_add(60)),
        ] {
            assert_eq!(shop.ram[SPRITE_STATE + slot], 9);
            assert_eq!(shop.ram[SPRITE_TYPE + slot], 0xbb);
            assert_eq!(shop.ram[SPRITE_IGNORE_PROJECTILE + slot], what);
            assert_eq!(shop.ram[SPRITE_SUBTYPE2 + slot], what);
            assert_eq!(shop.sprite_get_x(slot), x);
            assert_eq!(shop.sprite_get_y(slot), 0x0127);
            assert_eq!(shop.ram[SPRITE_FLAGS2 + slot] & 4, 4);
        }

        let mut minigame = fresh_state();
        minigame.ram[SPRITE_STATE + k] = 9;
        minigame.ram[DUNGEON_ROOM_INDEX] = 0x06;
        minigame.sprite_prep_shopkeeper(k);
        assert_eq!(minigame.ram[SPRITE_SUBTYPE2 + k], 1);
        assert_eq!(minigame.ram[SPRITE_GRAPHICS + k], 1);
        assert_eq!(minigame.ram[MINIGAME_CREDITS_PREP], 0xff);

        let mut terminate = fresh_state();
        terminate.ram[ANCILLA_TYPE] = 0x22;
        terminate.ram[ANCILLA_TYPE + 1] = 0x21;
        terminate.ram[ANCILLA_TYPE + 4] = 0x22;
        terminate.ram[ANCILLA_AUX_TIMER] = 9;
        terminate.ram[ANCILLA_AUX_TIMER + 1] = 9;
        terminate.ram[ANCILLA_AUX_TIMER + 4] = 9;
        terminate.shop_keeper_rapid_terminate_receive_item();
        assert_eq!(terminate.ram[ANCILLA_AUX_TIMER], 1);
        assert_eq!(terminate.ram[ANCILLA_AUX_TIMER + 1], 9);
        assert_eq!(terminate.ram[ANCILLA_AUX_TIMER + 4], 1);

        let mut bat = fresh_state();
        bat.sprite_spawn_bat_crash_cutscene();
        assert_eq!(bat.ram[SPRITE_TYPE + 15], 0x37);
        assert_eq!(bat.ram[SPRITE_Y_VEL + 15], 0);
        assert_eq!(bat.ram[SPRITE_B + 15], 0);
        assert_eq!(bat.ram[SPRITE_D + 15], 0);
        assert_eq!(bat.ram[SPRITE_FLOOR + 15], 0);
        assert_eq!(bat.ram[SPRITE_SUBTYPE2 + 15], 1);
        assert_eq!(bat.ram[SPRITE_FLAGS2 + 15], 1);
        assert_eq!(bat.ram[SPRITE_FLAGS3 + 15], 1);
        assert_eq!(bat.ram[SPRITE_OAM_FLAGS + 15], 1);
        assert_eq!(bat.sprite_get_x(15), 0x07cc);
        assert_eq!(bat.sprite_get_y(15), 0x0632);
        assert_eq!(bat.ram[SPRITE_DEFL_BITS + 15], 128);

        let mut circle = fresh_state();
        circle.ram[SPRITE_STATE + k] = 9;
        circle.sprite_set_x(k, 0x0100);
        circle.sprite_set_y(k, 0x0200);
        circle.ram[SPRITE_A + k] = 9;
        circle.ram[SPRITE_B + k] = 9;
        circle.sprite_prep_antifairy_circle(k);
        assert_eq!(circle.sprite_get_x(k), 0x00f6);
        assert_eq!(circle.ram[SPRITE_Y_VEL + k], (-18i8) as u8);
        assert_eq!(circle.ram[SPRITE_X_VEL + k], 0);
        assert_eq!(circle.ram[SPRITE_A + k], 0);
        assert_eq!(circle.ram[SPRITE_B + k], 0);
        assert_eq!(circle.ram[TMP_COUNTER], 0xff);

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
            assert_eq!(circle.ram[SPRITE_STATE + slot], 9);
            assert_eq!(circle.ram[SPRITE_TYPE + slot], 0x82);
            assert_eq!(circle.sprite_get_x(slot), x);
            assert_eq!(circle.sprite_get_y(slot), y);
            assert_eq!(circle.ram[SPRITE_X_VEL + slot], xv);
            assert_eq!(circle.ram[SPRITE_Y_VEL + slot], yv);
            assert_eq!(circle.ram[SPRITE_A + slot], a);
            assert_eq!(circle.ram[SPRITE_B + slot], b);
        }
    }

    #[test]
    fn arrghi_prep_copies_overlord_positions_and_updates_puff_ring() {
        let k = 12;
        let mut plain = fresh_state();
        plain.ram[OVERLORD_X_LO_PREP + k + 7] = 0x21;
        plain.ram[OVERLORD_Y_LO_PREP + k + 7] = 0x02;
        plain.ram[OVERLORD_GEN1_PREP + k + 7] = 0x43;
        plain.ram[OVERLORD_GEN3_PREP + k + 7] = 0x04;
        plain.sprite_prep_arrghi(k);
        assert_eq!(plain.ram[SPRITE_X_LO + k], 0x21);
        assert_eq!(plain.ram[SPRITE_X_HI + k], 0x02);
        assert_eq!(plain.ram[SPRITE_Y_LO + k], 0x43);
        assert_eq!(plain.ram[SPRITE_Y_HI + k], 0x04);

        let mut puffs = fresh_state();
        let k = 13;
        puffs.sprite_set_x(0, 0x0100);
        puffs.sprite_set_y(0, 0x0200);
        puffs.ram[OVERLORD_X_LO_PREP] = 0;
        puffs.ram[OVERLORD_X_LO_PREP + 1] = 0;
        puffs.ram[OVERLORD_X_LO_PREP + 2] = 0xaa;
        puffs.ram[OVERLORD_X_LO_PREP + 3] = 0xbb;
        puffs.ram[OVERLORD_X_LO_PREP + 4] = 0;
        puffs.ram[OVERLORD_X_LO_PREP + k + 7] = 0x56;
        puffs.ram[OVERLORD_Y_LO_PREP + k + 7] = 0x07;
        puffs.ram[OVERLORD_GEN1_PREP + k + 7] = 0x78;
        puffs.ram[OVERLORD_GEN3_PREP + k + 7] = 0x09;
        puffs.ram[FRAME_COUNTER] = 0;
        puffs.sprite_prep_arrghi(k);
        assert_eq!(puffs.ram[OVERLORD_X_LO_PREP + 2], 0);
        assert_eq!(puffs.ram[OVERLORD_X_LO_PREP + 3], 0);
        assert_eq!(puffs.ram[SPRITE_A], 1);
        assert_eq!(puffs.ram[SPRITE_B], 1);
        assert_eq!(puffs.ram[TMP_COUNTER], 13);
        assert_eq!(puffs.ram[OVERLORD_X_HI_PREP], 0);
        assert_eq!(puffs.ram[OVERLORD_Y_HI_PREP], 1);
        assert_eq!(puffs.ram[OVERLORD_GEN2_PREP], 3);
        assert_eq!(puffs.ram[OVERLORD_FLOOR_PREP], 2);
        assert_eq!(
            puffs.ram[SPRITE_X_LO + k],
            puffs.ram[OVERLORD_X_LO_PREP + k + 7]
        );
        assert_eq!(
            puffs.ram[SPRITE_X_HI + k],
            puffs.ram[OVERLORD_Y_LO_PREP + k + 7]
        );
        assert_eq!(
            puffs.ram[SPRITE_Y_LO + k],
            puffs.ram[OVERLORD_GEN1_PREP + k + 7]
        );
        assert_eq!(
            puffs.ram[SPRITE_Y_HI + k],
            puffs.ram[OVERLORD_GEN3_PREP + k + 7]
        );
    }

    #[test]
    fn medallion_table_and_eyegore_prep_match_room_and_item_gates() {
        let k = 7;

        let mut bombos = fresh_state();
        bombos.ram[OVERWORLD_SCREEN_INDEX] = 2;
        bombos.ram[LINK_ITEM_BOMBOS] = 1;
        bombos.ram[SPRITE_X_LO + k] = 0xf9;
        bombos.sprite_prep_medallion_table(k);
        assert_eq!(bombos.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(bombos.ram[SPRITE_X_LO + k], 1);
        assert_eq!(bombos.ram[SPRITE_GRAPHICS + k], 4);
        assert_eq!(bombos.ram[SPRITE_AI_STATE + k], 3);

        let mut ether_only_on_bombos_screen = fresh_state();
        ether_only_on_bombos_screen.ram[OVERWORLD_SCREEN_INDEX] = 2;
        ether_only_on_bombos_screen.ram[LINK_ITEM_ETHER] = 1;
        ether_only_on_bombos_screen.sprite_prep_medallion_table(k);
        assert_eq!(
            ether_only_on_bombos_screen.ram[SPRITE_IGNORE_PROJECTILE + k],
            1
        );
        assert_eq!(ether_only_on_bombos_screen.ram[SPRITE_GRAPHICS + k], 0);
        assert_eq!(ether_only_on_bombos_screen.ram[SPRITE_AI_STATE + k], 0);

        let mut ether = fresh_state();
        ether.ram[OVERWORLD_SCREEN_INDEX] = 3;
        ether.ram[LINK_ITEM_ETHER] = 1;
        ether.ram[SPRITE_X_LO + k] = 0x20;
        ether.sprite_prep_medallion_table(k);
        assert_eq!(ether.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(ether.ram[SPRITE_X_LO + k], 0x20);
        assert_eq!(ether.ram[SPRITE_GRAPHICS + k], 4);
        assert_eq!(ether.ram[SPRITE_AI_STATE + k], 3);

        let mut eyegore = fresh_state();
        eyegore.ram[DUNGEON_ROOM_INDEX2] = 75;
        eyegore.ram[SPRITE_TYPE + k] = 0x83;
        eyegore.ram[SPRITE_B + k] = 0xff;
        eyegore.ram[SPRITE_DEFL_BITS + k] = 0xaa;
        eyegore.sprite_prep_eyegore(k);
        assert_eq!(eyegore.ram[SPRITE_B + k], 0);
        assert_eq!(eyegore.ram[SPRITE_DEFL_BITS + k], 0);

        let mut untouched = fresh_state();
        untouched.ram[DUNGEON_ROOM_INDEX2] = 74;
        untouched.ram[SPRITE_B + k] = 4;
        untouched.ram[SPRITE_DEFL_BITS + k] = 0xaa;
        untouched.sprite_prep_eyegore(k);
        assert_eq!(untouched.ram[SPRITE_B + k], 4);
        assert_eq!(untouched.ram[SPRITE_DEFL_BITS + k], 0xaa);
    }
}
