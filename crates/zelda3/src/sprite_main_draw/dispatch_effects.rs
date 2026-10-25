//! Active-sprite dispatch and shared shadow, projectile, and dust effects.

use super::*;
use crate::types::sign8;

impl ZeldaState {

    // -----------------------------------------------------------------------
    // void SpriteDraw_BigShadow(int k, int anim) {  // 9dd1a8 — sprite_main.c:17651
    //   cur_sprite_y += sprite_z[k];
    //   oam_cur_ptr += 16;
    //   oam_ext_cur_ptr += 4;
    //   Sprite_DrawMultiple(k, &kLargeShadow_Dmd[anim * 3], 3, NULL);
    //   Sprite_Get16BitCoords(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_big_shadow(&mut self, k: usize, anim: i32) {
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let z = self.sprite_slot_view(k).z() as u16;
        self.sprite_workspace_mut()
            .set_current_sprite_y(cur_y.wrapping_add(z));
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(16));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(4));
        let base = (anim as usize) * 3;
        self.sprite_draw_multiple(k, &LARGE_SHADOW_DRAW_FRAMES[base..base + 3]);
        self.sprite_get16_bit_coords(k);
    }

    // void SpriteActive_Main(int k) {  // 869271
    pub(in crate::zelda_rtl) fn sprite_active_main(&mut self, k: usize) {
        if (0x41..=0x70).contains(&self.sprite_slot_view(k).sprite_type()) {
            // $06:9271 RTS-dispatches through BFEA. Restore the two real
            // call frames; these 844 clocks were already charged together
            // by charge_sprite_active_main_dispatch, not missing work.
            crate::cycle_ledger::charge(260 + 62 + 42);
            let _long = crate::cycle_ledger::routine(0x05_b5c3);
            crate::cycle_ledger::charge(190);
            let _main = crate::cycle_ledger::routine(0x05_b5d3);
            crate::cycle_ledger::charge(290);
            return self.sprite_active_main_dispatch(k);
        }
        self.charge_sprite_active_main_dispatch(k);
        // Types $79-$BC and $BD-$D7 dispatch to the bank-6 bounces
        // $06:BFEF/BFF4 (JSL ... RTS, 104, charged into the open scope) into
        // SpriteActive3_MainLong $1E:8B11 / SpriteActive4_MainLong $1D:C21A
        // (PHB PHK PLB JSR ... PLB RTL, 190). Their SpriteActive3/4_Main
        // ($1E:8B19 / $1D:C222) is a second RTS dispatch (290) whose handler
        // runs inside its frame, so the handler charges into that scope.
        let (long, main) = match self.sprite_slot_view(k).sprite_type() {
            0x79..=0xbc => (0x1e_8b11, 0x1e_8b19),
            0xbd..=0xd7 => (0x1d_c21a, 0x1d_c222),
            _ => return self.sprite_active_main_dispatch(k),
        };
        crate::cycle_ledger::charge(104);
        let _long = crate::cycle_ledger::routine(long);
        crate::cycle_ledger::charge(190);
        let _main = crate::cycle_ledger::routine(main);
        crate::cycle_ledger::charge(290);
        self.sprite_active_main_dispatch(k);
    }

    pub(in crate::zelda_rtl) fn charge_sprite_active_main_dispatch(&self, k: usize) {
        // Cycle ledger: SpriteActive_Main $06:9271 (entry m8 x8) as reached
        // by Sprite_ExecuteSingle's JMP: an RTS dispatch (LDA $e20,x REP #$30
        // AND #$00ff ASL TAY LDA $9283,y DEC PHA SEP #$30 RTS, 260) that runs
        // inside the frame that jumped, so it charges into the open scope
        // (no scope of its own; `sprite_active_main_jsr` is the JSR form).
        // Types $41-$70 dispatch to the bank-6 bounce
        // SpriteModule_Active_bounce $06:BFEA (JSL $05:B5C3 62 ... RTS 42)
        // into SpriteModule_Active $05:B5C3 (PHB PHK PLB JSR $05:B5D3 PLB
        // RTL, 190), whose SpriteActive2_Main $05:B5D3 is a second RTS
        // dispatch (LDA $e20,x SEC SBC #$41 REP #$30 AND ASL TAY LDA $b5e8,y
        // DEC PHA SEP RTS, 290); all of it charges into the open scope. The
        // other trampolines at $06:BFEF/BFF4/BFFE/C003/C008/D04A are not
        // charged here.
        crate::cycle_ledger::charge(260);
        if (0x41..=0x70).contains(&self.sprite_slot_view(k).sprite_type()) {
            crate::cycle_ledger::charge(62 + 190 + 290 + 42);
        }
    }

    /// `SpriteActive_Main` reached by `JSR $06:9271` (SpriteDeath_MainEx,
    /// SpriteModule_Carried/Fall2, SpriteStunned_Main_Func1,
    /// SpritePrep_TrooperAndArcherSoldier and the $06:8526 wrapper): a JSR
    /// target, so the scope encloses the dispatch and the handler.
    pub(in crate::zelda_rtl) fn sprite_active_main_jsr(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x06_9271);
        self.sprite_active_main(k);
    }

    /// `SpriteActive_Main_` $06:8526, the long-call wrapper (PHB PHK PLB JSR
    /// $06:9271 PLB RTL, 190) used from other banks (SpriteModule_Burn's
    /// bounce, SpriteExplode_SpawnEA, the credits sprite draws).
    pub(in crate::zelda_rtl) fn sprite_active_main_long(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x06_8526);
        crate::cycle_ledger::charge(190);
        self.sprite_active_main_jsr(k);
    }

    fn sprite_active_main_dispatch(&mut self, k: usize) {
        // A handler that charges nothing is not annotated yet: the ledger
        // prefix it ran in is incomplete (see `cycle_ledger::probe_annotation`).
        let _probe = crate::cycle_ledger::probe_annotation();
        match self.sprite_slot_view(k).sprite_type() {
            0x00 => self.sprite_raven(k),
            0x01 => self.sprite_01_vulture_bounce(k),
            0x02 => self.sprite_02_stalfos_head(k),
            0x03 => {}
            0x04 => self.sprite_pull_switch_bounce(k),
            0x05 => self.sprite_pull_switch_bounce(k),
            0x06 => self.sprite_pull_switch_bounce(k),
            0x07 => self.sprite_pull_switch_bounce(k),
            0x08 => self.sprite_08_octorok(k),
            0x09 => self.sprite_09_giant_moldorm(k),
            0x0a => self.sprite_08_octorok(k),
            0x0b => self.sprite_0_b_cucco(k),
            0x0c => self.sprite_0_c_octorok_stone(k),
            0x0d => self.sprite_0_d_buzzblob(k),
            0x0e => self.sprite_0_e_snapdragon(k),
            0x0f => self.sprite_0_f_octoballoon(k),
            0x10 => self.sprite_10_octoballoon_baby(k),
            0x11 => self.sprite_11_hinox(k),
            0x12 => self.sprite_12_moblin(k),
            0x13 => self.sprite_13_mini_helmasaur(k),
            0x14 => self.sprite_14_thieves_town_grate(k),
            0x15 => self.sprite_15_antifairy(k),
            0x16 => self.sprite_16_elder_bounce(k),
            0x17 => self.sprite_17_hoarder(k),
            0x18 => self.sprite_18_mini_moldorm(k),
            0x19 => self.sprite_19_poe(k),
            0x1a => self.sprite_1_a_smithy(k),
            0x1b => self.sprite_1_b_arrow(k),
            0x1c => self.sprite_1_c_statue(k),
            0x1d => self.sprite_1_d_flute_quest(k),
            0x1e => self.sprite_1_e_crystal_switch(k),
            0x1f => self.sprite_1_f_sick_kid(k),
            0x20 => self.sprite_20_sluggula(k),
            0x21 => self.sprite_21_water_switch(k),
            0x22 => self.sprite_22_ropa(k),
            0x23 => self.sprite_23_red_bari(k),
            0x24 => self.sprite_23_red_bari(k),
            0x25 => self.sprite_25_talking_tree(k),
            0x26 => self.sprite_26_hardhat_beetle(k),
            0x27 => self.sprite_27_deadrock(k),
            0x28 => self.sprite_28_dark_world_hint_npc(k),
            0x29 => self.sprite_human_multi_1(k),
            0x2a => self.sprite_sweeping_lady(k),
            0x2b => self.sprite_2_b_hobo(k),
            0x2c => self.sprite_lumberjacks(k),
            0x2d => self.sprite_2_d_telepathic_tile(k),
            0x2e => self.sprite_2_e_flute_kid(k),
            0x2f => self.sprite_maze_game_lady(k),
            0x30 => self.sprite_maze_game_guy(k),
            0x31 => self.sprite_fortune_teller(k),
            0x32 => self.sprite_quarrel_bros(k),
            0x33 => self.sprite_33_rupee_pull(k),
            0x34 => self.sprite_young_snitch_lady(k),
            0x35 => self.sprite_inn_keeper(k),
            0x36 => self.sprite_witch(k),
            0x37 => self.sprite_37_waterfall(k),
            0x38 => self.sprite_38_eye_statue(k),
            0x39 => self.sprite_39_locksmith(k),
            0x3a => self.sprite_3_a_magic_bat(k),
            0x3b => self.sprite_dash_item(k),
            0x3c => self.sprite_trough_boy(k),
            0x3d => self.sprite_old_snitch_lady(k),
            0x3e => self.sprite_17_hoarder(k),
            0x3f => self.sprite_tutorial_guard_or_barrier(k),
            0x40 => self.sprite_tutorial_guard_or_barrier(k),
            0x41 => self.sprite_41_blue_guard(k),
            0x42 => self.sprite_41_blue_guard(k),
            0x43 => self.sprite_41_blue_guard(k),
            0x44 => self.sprite_44_bluesain_bolt(k),
            0x45 => self.sprite_45_hog_spear_man(k),
            0x46 => self.sprite_46_blue_archer(k),
            0x47 => self.sprite_47_green_bush_guard(k),
            0x48 => self.sprite_48_red_javelin_guard(k),
            0x49 => self.sprite_49_red_bush_guard(k),
            0x4a => self.sprite_4_a_bomb_guard(k),
            0x4b => self.sprite_4_b_green_knife_guard(k),
            0x4c => self.sprite_4_c_geldman(k),
            0x4d => self.sprite_4_d_toppo(k),
            0x4e => self.sprite_4_e_popo(k),
            0x4f => self.sprite_4_e_popo(k),
            0x50 => self.sprite_50_cannonball(k),
            0x51 => self.sprite_51_armos_statue(k),
            0x52 => self.sprite_52_king_zora(k),
            0x53 => self.sprite_53_armos_knight(k),
            0x54 => self.sprite_54_lanmolas(k),
            0x55 => self.sprite_55_zora(k),
            0x56 => self.sprite_56_walking_zora(k),
            0x57 => self.sprite_57_desert_statue(k),
            0x58 => self.sprite_58_crab(k),
            0x59 => self.sprite_59_lost_woods_bird(k),
            0x5a => self.sprite_5_a_lost_woods_squirrel(k),
            0x5b => self.sprite_5_b_spark_clockwise(k),
            0x5c => self.sprite_5_b_spark_clockwise(k),
            0x5d => self.sprite_5_d_roller_vertical_down_first(k),
            0x5e => self.sprite_5_d_roller_vertical_down_first(k),
            0x5f => self.sprite_5_d_roller_vertical_down_first(k),
            0x60 => self.sprite_5_d_roller_vertical_down_first(k),
            0x61 => self.sprite_61_beamos(k),
            0x62 => self.sprite_62_master_sword(k),
            0x63 => self.sprite_63_debirando_pit(k),
            0x64 => self.sprite_64_debirando(k),
            0x65 => self.sprite_65_archery_game(k),
            0x66 => self.sprite_66_wall_cannon_vertical_left(k),
            0x67 => self.sprite_66_wall_cannon_vertical_left(k),
            0x68 => self.sprite_66_wall_cannon_vertical_left(k),
            0x69 => self.sprite_66_wall_cannon_vertical_left(k),
            0x6a => self.sprite_6_a_ball_n_chain(k),
            0x6b => self.sprite_6_b_cannon_trooper(k),
            0x6c => self.sprite_6_c_mirror_portal(k),
            0x6d => self.sprite_6_d_rat(k),
            0x6e => self.sprite_6_e_rope(k),
            0x6f => self.sprite_6_f_keese(k),
            0x70 => self.sprite_70_king_helmasaur_fireball(k),
            0x71 => self.sprite_71_leever(k),
            0x72 => self.sprite_72_fairy_pond(k),
            0x73 => self.sprite_73_uncle_and_priest(k),
            0x74 => self.sprite_running_man(k),
            0x75 => self.sprite_bottle_vendor(k),
            0x76 => self.sprite_76_zelda(k),
            0x77 => self.sprite_15_antifairy(k),
            0x78 => self.sprite_78_mrs_sahasrahla(k),
            0x79 => self.sprite_79_bee(k),
            0x7a => self.sprite_7_a_agahnim(k),
            0x7b => self.sprite_7_b_agahnim_balls(k),
            0x7c => self.sprite_7_c_green_stalfos(k),
            0x7d => self.sprite_7_d_big_spike(k),
            0x7e => self.sprite_7_e_firebar_clockwise(k),
            0x7f => self.sprite_7_e_firebar_clockwise(k),
            0x80 => self.sprite_80_firesnake(k),
            0x81 => self.sprite_81_hover(k),
            0x82 => self.sprite_82_antifairy_circle(k),
            0x83 => self.sprite_83_green_eyegore(k),
            0x84 => self.sprite_83_green_eyegore(k),
            0x85 => self.sprite_85_yellow_stalfos(k),
            0x86 => self.sprite_86_kodongo(k),
            0x87 => self.sprite_87_kodongo_fire(k),
            0x88 => self.sprite_88_mothula(k),
            0x89 => self.sprite_89_mothula_beam(k),
            0x8a => self.sprite_8_a_spike_block(k),
            0x8b => self.sprite_8_b_gibdo(k),
            0x8c => self.sprite_8_c_arrghus(k),
            0x8d => self.sprite_8_d_arrghi(k),
            0x8e => self.sprite_8_e_terrorpin(k),
            0x8f => self.sprite_8_f_blob(k),
            0x90 => self.sprite_90_wallmaster(k),
            0x91 => self.sprite_91_stalfos_knight(k),
            0x92 => self.sprite_92_helmasaur_king(k),
            0x93 => self.sprite_93_bumper(k),
            0x94 => self.sprite_94_pirogusu(k),
            0x95 => self.sprite_95_laser_eye_left(k),
            0x96 => self.sprite_95_laser_eye_left(k),
            0x97 => self.sprite_95_laser_eye_left(k),
            0x98 => self.sprite_95_laser_eye_left(k),
            0x99 => self.sprite_99_pengator(k),
            0x9a => self.sprite_9_a_kyameron(k),
            0x9b => self.sprite_9_b_wizzrobe(k),
            0x9c => self.sprite_9_c_zoro(k),
            0x9d => self.sprite_9_c_zoro(k),
            0x9e => self.sprite_9_e_haunted_grove_ostritch(k),
            0x9f => self.sprite_9_f_haunted_grove_rabbit(k),
            0xa0 => self.sprite_a0_haunted_grove_bird(k),
            0xa1 => self.sprite_a1_freezor(k),
            0xa2 => self.sprite_a2_kholdstare(k),
            0xa3 => self.sprite_a3_kholdstare_shell(k),
            0xa4 => self.sprite_a4_falling_ice(k),
            0xa5 => self.sprite_zazak_main(k),
            0xa6 => self.sprite_zazak_main(k),
            0xa7 => self.sprite_a7_stalfos(k),
            0xa8 => self.sprite_a8_green_zirro(k),
            0xa9 => self.sprite_a8_green_zirro(k),
            0xaa => self.sprite_aa_pikit(k),
            0xab => self.sprite_ab_crystal_maiden(k),
            0xac => self.sprite_ac_apple(k),
            0xad => self.sprite_ad_old_man(k),
            0xae => self.sprite_ae_pipe_down(k),
            0xaf => self.sprite_ae_pipe_down(k),
            0xb0 => self.sprite_ae_pipe_down(k),
            0xb1 => self.sprite_ae_pipe_down(k),
            0xb2 => self.sprite_b2_player_bee(k),
            0xb3 => self.sprite_b3_pedestal_plaque(k),
            0xb4 => self.sprite_b4_purple_chest(k),
            0xb5 => self.sprite_b5_bomb_shop(k),
            0xb6 => self.sprite_b6_kiki(k),
            0xb7 => self.sprite_b7_blind_maiden(k),
            0xb8 => self.sprite_b8_dialogue_tester(k),
            0xb9 => self.sprite_b9_bully_and_pink_ball(k),
            0xba => self.sprite_ba_whirlpool(k),
            0xbb => self.sprite_bb_shopkeeper(k),
            0xbc => self.sprite_bc_drunkard(k),
            0xbd => self.sprite_bd_vitreous(k),
            0xbe => self.sprite_be_vitreous_eye(k),
            0xbf => self.sprite_bf_lightning(k),
            0xc0 => self.sprite_c0_catfish(k),
            0xc1 => self.sprite_c1_cutscene_agahnim(k),
            0xc2 => self.sprite_c2_boulder(k),
            0xc3 => self.sprite_c3_gibo(k),
            0xc4 => self.sprite_c4_thief(k),
            0xc5 => self.sprite_c5_medusa(k),
            0xc6 => self.sprite_c6_4_way_shooter(k),
            0xc7 => self.sprite_c7_pokey(k),
            0xc8 => self.sprite_c8_big_fairy(k),
            0xc9 => self.sprite_c9_tektite(k),
            0xca => self.sprite_ca_chain_chomp(k),
            0xcb => self.sprite_cb_trinexx_rock_head(k),
            0xcc => self.sprite_cc(k),
            0xcd => self.sprite_cd(k),
            0xce => self.sprite_ce_blind(k),
            0xcf => self.sprite_cf_swamola(k),
            0xd0 => self.sprite_d0_lynel(k),
            0xd1 => self.sprite_d1_bunny_beam(k),
            0xd2 => self.sprite_d2_flopping_fish(k),
            0xd3 => self.sprite_d3_stal(k),
            0xd4 => self.sprite_d4_landmine(k),
            0xd5 => self.sprite_d5_dig_game_guy(k),
            0xd6 => self.sprite_d6_ganon(k),
            0xd7 => self.sprite_d6_ganon(k),
            0xd8 => self.sprite_d8_heart(k),
            0xd9 => self.sprite_d9_green_rupee(k),
            0xda => self.sprite_d9_green_rupee(k),
            0xdb => self.sprite_d9_green_rupee(k),
            0xdc => self.sprite_d9_green_rupee(k),
            0xdd => self.sprite_d9_green_rupee(k),
            0xde => self.sprite_d9_green_rupee(k),
            0xdf => self.sprite_d9_green_rupee(k),
            0xe0 => self.sprite_d9_green_rupee(k),
            0xe1 => self.sprite_d9_green_rupee(k),
            0xe2 => self.sprite_d9_green_rupee(k),
            0xe3 => self.sprite_e3_fairy(k),
            0xe4 => self.sprite_e4_small_key(k),
            0xe5 => self.sprite_e4_small_key(k),
            0xe6 => self.sprite_d9_green_rupee(k),
            0xe7 => self.sprite_e7_mushroom(k),
            0xe8 => self.sprite_e8_fake_sword(k),
            0xe9 => self.sprite_e9_potion_shop(k),
            0xea => self.sprite_heart_container(k),
            0xeb => self.sprite_heart_piece(k),
            0xec => self.sprite_ec_thrown_item(k),
            0xed => self.sprite_ed_somaria_platform(k),
            0xee => self.sprite_ee_movable_mantle(k),
            0xef => self.sprite_ed_somaria_platform(k),
            0xf0 => self.sprite_ed_somaria_platform(k),
            0xf1 => self.sprite_ed_somaria_platform(k),
            0xf2 => self.sprite_f2_medallion_tablet(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void DustCloud_Draw(int k) {  // 8dd120
    //   Four cloud tiles selected by sprite_graphics.
    // }
    pub(super) fn dust_cloud_draw(&mut self, k: usize) {
        let value = 0x14;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let base = (self.sprite_slot_view(k).graphics() as usize) * 4;
        self.sprite_draw_multiple(k, &DUST_CLOUD_DRAW_FRAMES[base..base + 4]);
    }

    // -----------------------------------------------------------------------
    // void Sprite_DustCloud(int k) {  // 85f2b2
    pub(super) fn sprite_dust_cloud(&mut self, k: usize) {
        self.dust_cloud_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() != 0 {
            return;
        }
        let value = 5;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let a = self.sprite_slot_view(k).a() as usize;
        if !sign8(DUST_CLOUD_GRAPHICS[a]) {
            let value = DUST_CLOUD_GRAPHICS[a];
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_slot_view_mut(k).add_a(1);
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_DrawLargeShadow2(int k) {  // 9dd1af
    //   Clamp z-derived animation and delegate to SpriteDraw_BigShadow.
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_large_shadow2(&mut self, k: usize) {
        let z = (self.sprite_slot_view(k).z() >> 3).min(4);
        self.sprite_draw_big_shadow(k, z as i32);
    }

    // -----------------------------------------------------------------------
    // void Flame_Draw(int k) {  // 9ec35c
    //   Two flame tiles selected by sprite_graphics.
    // }
    pub(in crate::zelda_rtl) fn flame_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 2;
        self.sprite_draw_multiple(k, &FLAME_DRAW_FRAMES[base..base + 2]);
    }

    // -----------------------------------------------------------------------
    // void SeekerEnergyBall_Draw(int k) {  // 9edc3e
    //   Four small orb tiles toggled by subtype animation.
    // }
    pub(super) fn seeker_energy_ball_draw(&mut self, k: usize) {
        let base = (((self.sprite_slot_view(k).subtype2() >> 2) & 1) as usize) * 4;
        self.sprite_draw_multiple(k, &ENERGY_BALL_DRAW_FRAMES[base..base + 4]);
    }

    // -----------------------------------------------------------------------
    // void Wizzbeam_Draw(int k) {  // 8dbe68
    //   Two small tiles selected by sprite_D.
    // }
    pub(in crate::zelda_rtl) fn wizzbeam_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).direction() as usize) * 2;
        self.sprite_draw_multiple(k, &WIZZBEAM_DRAW_FRAMES[base..base + 2]);
    }

    // -----------------------------------------------------------------------
    // void Trident_Draw(int k) {  // 9d9c1c
    pub(in crate::zelda_rtl) fn trident_draw(&mut self, k: usize) {
        let g = self.sprite_slot_view(k).g();
        if g == 0 {
            return;
        }
        let j = if g == 9 {
            3
        } else if g >= 9 {
            4
        } else {
            usize::from(self.sprite_slot_view(k).direction())
        };
        let cur_x = self
            .game_state
            .sprites
            .workspace
            .current_sprite_x()
            .wrapping_add_signed(i16::from(TRIDENT_DRAW_X_OFFSETS[j]));
        let cur_y = self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_add_signed(i16::from(TRIDENT_DRAW_Y_OFFSETS[j]));
        self.sprite_workspace_mut().set_current_sprite_x(cur_x);
        self.sprite_workspace_mut().set_current_sprite_y(cur_y);
        let bak = self.sprite_slot_view(k).object_priority();
        self.sprite_slot_view_mut(k).and_object_priority(!0x0f);
        let base = usize::from(g.wrapping_sub(1)) * 5;
        self.sprite_draw_multiple(k, &TRIDENT_DRAW_FRAMES[base..base + 5]);
        let value = bak;
        self.sprite_slot_view_mut(k).set_object_priority(value);
        self.sprite_get16_bit_coords(k);
    }
}
