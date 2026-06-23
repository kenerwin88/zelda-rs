// Methods ported from zelda3/src/ending.c and included inside ZeldaState.

use super::sprite::{DrawMultipleData, PrepOamCoordsRet};
use super::*;
use crate::types::sign8;
use crate::zelda_rtl::misc::DUNG_ANIMATED_TILES;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

mod ending_shared;
use ending_shared::*;

impl ZeldaState {
    fn apply_dung_pal_info(&mut self, idx: u8) {
        let (_, _, pal2, pal3) = DUNG_PAL_INFOS_ENDING[idx as usize];
        self.set_sp5l(pal2);
        self.set_sp6l(pal3);
    }

    fn CallForDuckIndoors(&mut self) {
        self.call_for_duck_indoors();
    }

    fn Sprite_SpawnBatCrashCutscene(&mut self) {
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

    fn sprite_get_16_bit_coords_ending(&mut self, k: usize) {
        self.sprite_get16_bit_coords(k);
    }

    fn sprite_active_main_ending(&mut self, k: usize) {
        self.sprite_active_main(k);
    }

    fn ending_asset_u16(&self, asset: usize, index: usize) -> u16 {
        let data = self
            .asset_raw(asset)
            .unwrap_or_else(|| panic!("missing ending asset {asset}"));
        read_word_from_slice(data, index * 2)
    }

    fn set_oam_helper0_addr(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        let y = if y.wrapping_add(0x10) < 0x100 {
            y as u8
        } else {
            0xf0
        };
        self.oam_state_mut()
            .write_entry(oam, x as u8, y, charnum, flags);
        let value = big | ((x >> 8) as u8 & 1);
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);
    }

    pub(super) fn Intro_SetupScreen(&mut self) {
        self.intro_setup_screen();
    }

    pub(super) fn Intro_LoadTextPointersAndPalettes(&mut self) {
        self.intro_load_text_pointers_and_palettes();
    }

    pub(super) fn credits_load_scene_overworld_prep_gfx(&mut self) {
        self.enable_force_blank();
        self.erase_tile_maps_normal();
        self.set_color_window_selection(0x82);
        let k = (self.game_state.frame.submodule >> 1) as usize;
        self.set_dungeon_room(ENDING_SCENE_ENTRANCES[k]);
        if k != 6 && k != 15 {
            self.LoadOverworldFromDungeon();
        } else {
            self.Overworld_EnterSpecialArea();
        }
        self.set_music_control(0);
        self.set_ambient_sound_effect(0);
        let t = self.game_state.world.location.overworld_screen_index() & !0x40;
        self.DecompressAnimatedOverworldTiles(if t == 3 || t == 5 || t == 7 {
            0x58
        } else {
            0x5a
        });
        let k = (self.game_state.frame.submodule >> 1) as usize;
        self.sprite_system_mut()
            .set_graphics_index(ENDING_SPRITE_PACKS[k]);
        let sprpal = ENDING_SPRITE_PALETTES[k];
        self.set_hud_palette(1);
        self.initialize_tilesets();
        self.OverworldLoadScreensPaletteSet();
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(self.game_state.world.location.overworld_screen_index()),
            sprpal,
        );
        self.palette_load_hud();
        if self.game_state.frame.submodule == 0 {
            self.TransferFontToVRAM();
        }
        self.overworld_load_palettes_inner();
        self.Overworld_SetFixedColAndScroll();
        if self.game_state.world.location.overworld_screen_index() >= 128 {
            self.Palette_SetOwBgColor();
        }
        self.set_bg_mode(9);
        self.increment_subsubmodule();
    }

    pub(super) fn credits_load_scene_overworld_overlay(&mut self) {
        self.Overworld_LoadOverlays2();
        self.set_music_control(0);
        self.set_ambient_sound_effect(0);
        self.decrement_submodule();
        self.increment_subsubmodule();
    }

    pub(super) fn credits_load_scene_overworld_load_map(&mut self) {
        self.Overworld_LoadAndBuildScreen();
        self.credits_prep_and_load_sprites();
        self.ending_scratch_mut().clear_primary_word();
        self.set_subsubmodule(0);
    }

    pub(super) fn credits_operate_scrolling_and_tile_map(&mut self) {
        self.credits_handle_camera_scroll_control();
        if self.has_screen_transition_direction_bits() {
            self.OverworldHandleMapScroll();
        }
    }

    pub(super) fn credits_load_cool_background(&mut self) {
        self.world_palette_theme_mut().set_main_tile_theme_index(33);
        self.world_palette_theme_mut().set_aux_tile_theme_index(59);
        self.sprite_system_mut().set_graphics_index(45);
        self.initialize_tilesets();
        self.set_overworld_screen(0x5b);
        self.Overworld_LoadPalettes(
            self.GetOverworldBgPalette(self.game_state.world.location.overworld_screen_index()),
            0x13,
        );
        self.set_overworld_palette_aux2_hi(3);
        self.palette_load_ow_bg2();
        self.overworld_copy_palettes_to_cache();
        self.Overworld_LoadOverlays2();
        self.set_bg1_y_low(0);
        self.set_bg1_x_low(0);
        self.decrement_submodule();
    }

    pub(super) fn credits_load_scene_dungeon(&mut self) {
        self.enable_force_blank();
        self.erase_tile_maps_normal();
        let i = (self.game_state.frame.submodule >> 1) as usize;
        self.set_which_entrance(ENDING_SCENE_ENTRANCES[i]);
        self.Dungeon_LoadEntrance();
        self.dungeon_torch_mut().clear_lit_torches();
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        self.Dungeon_LoadAndDrawRoom();
        self.decompress_animated_dungeon_tiles(
            DUNG_ANIMATED_TILES
                [self.game_state.world.palette_theme.main_tile_theme_index() as usize]
                as usize,
        );
        self.sprite_system_mut()
            .set_graphics_index(ENDING_SPRITE_PACKS[i]);
        self.apply_dung_pal_info(ENDING_SPRITE_PALETTES[i] & 0x3f);
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(10);
        self.initialize_tilesets();
        self.set_sp6r_indoors(10);
        self.Dungeon_LoadPalettes();
        self.set_bg_mode(9);
        self.ending_scratch_mut().clear_primary_word();
        self.set_screen_brightness(0);
        self.increment_submodule();
        self.credits_prep_and_load_sprites();
    }

    pub(super) fn module18_ganon_emerges(&mut self) {
        let hofs2 = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let vofs2 = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        let hofs1 = self.game_state.display.ppu_scroll_copy.bg1_h_copy2();
        let vofs1 = self.game_state.display.ppu_scroll_copy.bg1_v_copy2();
        let bg1_x_offset = self.game_state.world.scroll.bg1_x_offset();
        let bg1_y_offset = self.game_state.world.scroll.bg1_y_offset();
        self.set_bg1_bg2_live_and_copy(
            hofs2.wrapping_add(bg1_x_offset),
            vofs2.wrapping_add(bg1_y_offset),
            hofs1.wrapping_add(bg1_x_offset),
            vofs1.wrapping_add(bg1_y_offset),
        );
        self.sprite_main();
        self.set_bg1_y(vofs1);
        self.set_bg1_x(hofs1);
        self.set_bg2_y(vofs2);
        self.set_bg2_x(hofs2);
        match self.overworld_map_state() {
            0 => {
                self.dungeon_handle_layer_effect();
                self.CallForDuckIndoors();
                self.SaveDungeonKeys();
                self.increment_overworld_map_state();
                self.follower_link_state_mut().increment_immobilized_flag();
            }
            1 => {
                self.dungeon_handle_layer_effect();
                if self.game_state.frame.submodule == 10 {
                    self.set_overworld_screen(91);
                    self.set_indoor_flag(0);
                    self.set_main_module(24);
                    self.set_submodule(0);
                    self.set_overworld_map_state(2);
                }
            }
            2 => {
                self.dungeon_handle_layer_effect();
                self.decrement_screen_brightness();
                if self.game_state.display.screen_brightness == 0 {
                    self.enable_force_blank();
                    self.increment_overworld_map_state();
                    self.hud_rebuild_indoor();
                    self.follower_link_state_mut().clear_movement_velocity();
                }
            }
            3 => {
                self.set_birdtravel_status(8);
                self.clear_bird_travel_stop_status(1);
                self.FluteMenu_LoadSelectedScreen();
                self.LoadOWMusicIfNeeded();
                self.set_music_control(9);
            }
            4 => {
                self.Overworld_LoadOverlayAndMap();
                self.set_subsubmodule(0);
            }
            5 => {
                self.increment_screen_brightness();
                if self.game_state.display.screen_brightness == 15 {
                    self.dungeon_savegame_state_mut()
                        .clear_savegame_state_bits();
                    self.clear_modal_pause_flag();
                    self.Sprite_SpawnBatCrashCutscene();
                    self.follower_link_state_mut().set_facing(2);
                    self.set_saved_module_for_menu(9);
                    self.set_indoor_flag(0);
                    self.increment_overworld_map_state();
                    self.set_subsubmodule(128);
                    self.save_progress_mut().set_palace_index_x2(255);
                }
            }
            6 => {}
            7 => {
                self.decrement_subsubmodule();
                if self.game_state.frame.subsubmodule == 0 {
                    self.increment_overworld_map_state();
                }
            }
            8 => self.BirdTravel_Finish_Doit(),
            _ => {}
        }
        self.link_oam_main();
    }

    pub(super) fn module19_triforce_room(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => {
                self.link_reset_properties_a();
                self.follower_link_state_mut()
                    .set_last_direction_moved_towards(0);
                self.set_music_control(0xf1);
                self.reset_transition_props_and_advance_reset_interface();
            }
            1 => {
                self.conditional_mosaic_control();
                self.apply_palette_filter_bounce();
            }
            2 => {
                self.enable_force_blank();
                self.load_credits_songs();
                self.set_dungeon_room(0x189);
                self.erase_tile_maps_normal();
                self.Palette_RevertTranslucencySwap();
                self.Overworld_EnterSpecialArea();
                self.Overworld_LoadOverlays2();
                self.increment_subsubmodule();
                self.set_main_module(25);
                self.set_submodule(0);
            }
            3 => {
                self.world_palette_theme_mut().set_main_tile_theme_index(36);
                self.sprite_system_mut().set_graphics_index(125);
                self.world_palette_theme_mut().set_aux_tile_theme_index(81);
                self.initialize_tilesets();
                self.Overworld_LoadAreaPalettesEx(4);
                self.Overworld_LoadPalettes(14, 0);
                self.SpecialOverworld_CopyPalettesToCache();
                self.increment_subsubmodule();
            }
            4 => {
                let bak0 = self.game_state.frame.subsubmodule;
                self.Module08_02_LoadAndAdvance();
                self.set_subsubmodule(bak0.wrapping_add(1));
                self.set_screen_brightness(15);
                self.set_countdown(31);
                self.clear_mosaic_target_level();
                self.set_bg1_h_high(1);
                self.set_color_window_selection(2);
                self.set_color_math_control(50);
                self.set_mosaic_level(240);
                {
                    let mut player = self.follower_link_state_mut();
                    player.set_y_low(236);
                    player.set_x_low(120);
                    player.set_lower_level_state(2);
                }
                self.set_music_control(32);
                self.set_main_module(25);
                self.set_submodule(0);
            }
            5 => {
                self.follower_link_state_mut()
                    .set_direction_and_last_direction(8);
                self.follower_link_state_mut().set_facing(0);
                if self.game_state.player.follower_link.y_low() < 192 {
                    self.follower_link_state_mut()
                        .set_direction_and_last_direction(0);
                    self.follower_link_state_mut().clear_animation_step();
                    self.increment_subsubmodule();
                }
            }
            6 => {
                if self.game_state.display.palette_filter.countdown() & 1 == 0
                    && self.game_state.display.mosaic_level != 0
                {
                    self.decrement_mosaic_level_by(0x10);
                }
                self.set_bg_mode(9);
                self.set_mosaic_copy_from_level_or(7);
                self.apply_palette_filter_bounce();
            }
            7 => {
                self.triforce_room_prep_gfx_slot_for_poly();
                self.dialogue_message_index_mut().set_value(0x173);
                self.main_show_text_message();
                self.RenderText();
                self.ending_scratch_mut().set_primary_low(0x80);
                self.set_main_module(25);
                self.increment_subsubmodule();
            }
            8 | 10 => {
                self.advance_polyhedral();
                if self.game_state.frame.subsubmodule == 11 {
                    self.set_music_control(33);
                    self.set_main_module(25);
                    self.follower_link_state_mut()
                        .set_direction_and_last_direction(0);
                    self.increment_submodule();
                }
            }
            9 => {
                self.advance_polyhedral();
                self.RenderText();
                if self.game_state.frame.submodule == 0 {
                    self.set_overworld_map_state(0);
                    self.set_main_module(25);
                    self.increment_subsubmodule();
                }
            }
            11 => {
                self.advance_polyhedral();
                self.triforce_room_link_approach_triforce();
                if self.game_state.frame.subsubmodule == 12 {
                    self.follower_link_state_mut()
                        .set_direction_and_last_direction(0);
                }
            }
            12 => {
                self.advance_polyhedral();
                if self.ending_scratch_mut().decrement_primary_low() == 0 {
                    self.Palette_AnimGetMasterSword2();
                    self.increment_submodule();
                }
            }
            13 => {
                self.advance_polyhedral();
                self.PaletteFilter_BlindingWhiteTriforce();
                if self
                    .game_state
                    .display
                    .palette_filter
                    .darkening_or_lightening_screen()
                    == 255
                {
                    self.increment_subsubmodule();
                }
            }
            14 => {
                self.decrement_screen_brightness();
                if self.game_state.display.screen_brightness == 0 {
                    self.set_main_module(26);
                    self.set_submodule(0);
                    self.set_subsubmodule(0);
                    self.set_irq_control_flag(0xff);
                    self.deactivate_nmi_thread();
                    self.clear_pending_polyhedral_update();
                    self.save_progress_mut().set_dark_world_state(0);
                }
            }
            _ => {}
        }
        self.copy_live_to_ppu_copy();
        if self.game_state.frame.subsubmodule < 7 || self.game_state.frame.subsubmodule >= 11 {
            self.link_handle_velocity();
            self.link_handle_moving_animation_full_long_entry();
        }
        self.link_oam_main();
    }

    pub(super) fn Intro_InitializeBackgroundSettings(&mut self) {
        self.intro_initialize_background_settings();
    }

    pub(super) fn Polyhedral_InitializeThread(&mut self) {
        self.polyhedral_initialize_thread();
    }

    pub(super) fn Module00_Intro(&mut self) {
        self.module00_intro();
    }

    pub(super) fn Intro_Init(&mut self) {
        self.intro_init();
    }

    pub(super) fn Intro_Init_Continue(&mut self) {
        self.intro_init_continue();
    }

    pub(super) fn intro_clear1kb_blocks_of_wram(&mut self) {
        let i = self.game_state.dungeon.scratch_word.primary_word();
        let r18 = self.game_state.dungeon.scratch_word.secondary_word();
        let i = self.clear_intro_wram_block_columns(i, r18);
        self.ending_scratch_mut().set_primary_word(i);
        self.ending_scratch_mut()
            .set_secondary_word(i.wrapping_sub(0x400));
    }

    pub(super) fn Intro_InitializeMemory_darken(&mut self) {
        self.intro_initialize_memory_darken();
    }

    pub(super) fn IntroZeldaFadein(&mut self) {
        self.intro_zelda_fadein();
    }

    pub(super) fn Intro_FadeInBg(&mut self) {
        self.intro_fade_in_bg();
    }

    pub(super) fn Intro_SwordComingDown(&mut self) {
        self.intro_sword_coming_down();
    }

    pub(super) fn Intro_WaitPlayer(&mut self) {
        self.intro_wait_player();
    }

    pub(super) fn FadeMusicAndResetSRAMMirror(&mut self) {
        self.fade_music_and_reset_sram_mirror();
    }

    pub(super) fn Intro_InitializeTriforcePolyThread(&mut self) {
        self.intro_initialize_triforce_poly_thread();
    }

    pub(super) fn Intro_InitGfx_Helper(&mut self) {
        self.intro_init_gfx_helper();
    }

    pub(super) fn LoadTriforceSpritePalette(&mut self) {
        self.load_triforce_sprite_palette();
    }

    pub(super) fn Intro_HandleAllTriforceAnimations(&mut self) {
        self.intro_handle_all_triforce_animations();
    }

    pub(super) fn Scene_AnimateEverySprite(&mut self) {
        self.scene_animate_every_sprite();
    }

    pub(super) fn Intro_AnimateTriforce(&mut self) {
        self.intro_animate_triforce();
    }

    pub(super) fn Intro_RunStep(&mut self) {
        self.intro_run_step();
    }

    pub(super) fn Intro_AnimOneObj(&mut self, k: usize) {
        self.intro_anim_one_obj(k);
    }

    pub(super) fn Intro_SpriteType_A_0(&mut self, k: usize) {
        self.intro_sprite_type_a_0(k);
    }

    pub(super) fn Intro_SpriteType_B_0(&mut self, k: usize) {
        self.intro_sprite_type_b_0(k);
    }

    pub(super) fn AnimateSceneSprite_DrawTriangle(&mut self, k: usize) {
        self.animate_scene_sprite_draw_triangle(k);
    }

    pub(super) fn intro_copy_sprite_type4_to_oam(&mut self, k: usize) {
        self.animate_scene_sprite_add_objects_to_oam_buffer(
            k,
            if k == 2 {
                &INTRO_COPY_SPRITE_TYPE4_TO_OAM_RIGHT_ENTRIES
            } else {
                &INTRO_COPY_SPRITE_TYPE4_TO_OAM_LEFT_ENTRIES
            },
        );
    }

    pub(super) fn exit_0_cca90(&mut self, _k: usize) {}

    pub(super) fn InitializeSceneSprite_Copyright(&mut self, k: usize) {
        self.initialize_scene_sprite_copyright(k);
    }

    pub(super) fn AnimateSceneSprite_Copyright(&mut self, k: usize) {
        self.animate_scene_sprite_copyright(k);
    }

    pub(super) fn InitializeSceneSprite_Sparkle(&mut self, k: usize) {
        self.initialize_scene_sprite_sparkle(k);
    }

    pub(super) fn AnimateSceneSprite_Sparkle(&mut self, k: usize) {
        self.animate_scene_sprite_sparkle(k);
    }

    #[rustfmt::skip]
    pub(super) fn animate_scene_sprite_add_objects_to_oam_buffer(&mut self, k: usize, entries: &[IntroSpriteEnt]) {
        self.animate_scene_sprite_add_objects_to_oam_buffer_with_offset(k, entries, 0, 0);
    }

    #[rustfmt::skip]
    fn animate_scene_sprite_add_objects_to_oam_buffer_with_offset(&mut self, k: usize, entries: &[IntroSpriteEnt], x_delta: i16, y_delta: i16) {
        let x = self.intro_actor(k).x();
        let y = self.intro_actor(k).y();
        let mut oam = self.allocate_intro_sprite_oam_entries(entries.len());
        for &(x_off, y_off, charnum, flags, ext) in entries {
            let obj_x = x.wrapping_add((x_off as i16).wrapping_add(x_delta) as u16);
            let obj_y = y.wrapping_add((y_off as i16).wrapping_add(y_delta) as u16);
            self.set_oam_helper0_at(oam, obj_x, obj_y, charnum, flags, ext);
            oam += 4;
        }
    }

    pub(super) fn AnimateSceneSprite_MoveTriangle(&mut self, k: usize) {
        self.animate_scene_sprite_move_triangle(k);
    }

    pub(super) fn triforce_room_prep_gfx_slot_for_poly(&mut self) {
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(8);
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.intro_actor_mut(0).set_init_phase(1);
        self.intro_actor_mut(1).set_init_phase(1);
        self.intro_actor_mut(2).set_init_phase(1);
        self.intro_actor_mut(0).set_subtype(4);
        self.intro_actor_mut(1).set_subtype(5);
        self.intro_actor_mut(2).set_subtype(6);
        self.set_screen_brightness(15);
        self.increment_submodule();
    }

    pub(super) fn credits_initialize_polyhedral(&mut self) {
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(8);
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.poly_runtime_mut().clear_config1();
        for k in 0..3 {
            self.intro_actor_mut(k).set_init_phase(1);
            self.intro_actor_mut(k).set_subtype(7);
        }
        self.set_screen_brightness(15);
        self.increment_submodule();
    }

    pub(super) fn advance_polyhedral(&mut self) {
        self.triforce_room_handle_poly();
        self.scene_animate_every_sprite();
    }

    pub(super) fn triforce_room_handle_poly(&mut self) {
        self.activate_nmi_thread();
        self.pause_intro_triangle_motion();
        if self.game_state.ending.attract_scene.intro_did_run_step() != 0 {
            return;
        }
        match self.game_state.ending.attract_scene.intro_step_index() {
            0 => {
                self.poly_runtime_mut().subtract_config1(2);
                if self.game_state.poly.runtime.config1() < 2 {
                    self.poly_runtime_mut().clear_config1();
                    self.attract_scene_mut().increment_intro_step_index();
                    self.increment_subsubmodule();
                }
                if self.game_state.frame.subsubmodule >= 10 {
                    self.attract_scene_mut().increment_intro_step_index();
                    self.intro_actor_mut(1).set_y_velocity(5);
                }
                self.poly_runtime_mut().add_angle_b(2);
                self.poly_runtime_mut().add_angle_a(1);
            }
            1 => {
                if self.game_state.frame.subsubmodule >= 10 {
                    self.attract_scene_mut().increment_intro_step_index();
                    self.intro_actor_mut(1).set_y_velocity(5);
                }
                self.poly_runtime_mut().add_angle_b(2);
                self.poly_runtime_mut().add_angle_a(1);
            }
            2 => {
                self.start_triforce_countdown(0x1c0);
                if self.game_state.poly.runtime.config1() < 128 {
                    self.poly_runtime_mut().increment_config1();
                } else if (self.game_state.poly.runtime.angle_b().wrapping_sub(10) & 0x7f) >= 92
                    && self.game_state.poly.runtime.angle_a().wrapping_sub(11) >= 220
                {
                    self.poly_runtime_mut().clear_angles();
                    self.increment_subsubmodule();
                    self.attract_scene_mut().increment_intro_step_index();
                    self.set_sound_effect_1(44);
                    self.set_main_color(0xd7, 0x7fff);
                    self.increment_cgram_update_flag();
                    self.attract_scene_mut().set_intro_step_timer(6);
                    break_triforce_handle_poly(self);
                    return;
                }
                self.poly_runtime_mut().add_angle_b(5);
                self.poly_runtime_mut().add_angle_a(3);
            }
            3 => {
                self.attract_scene_mut().decrement_intro_step_timer();
                if self.game_state.ending.attract_scene.intro_step_timer() == 0 {
                    self.set_main_color(0xd7, POLYHEDRAL_PALETTE[7]);
                    self.increment_cgram_update_flag();
                    self.attract_scene_mut().increment_intro_step_index();
                }
            }
            _ => {}
        }
        self.attract_scene_mut().mark_intro_did_run_step();
        self.resume_intro_triangle_motion();
        self.attract_scene_mut().increment_intro_frame_counter();
    }

    pub(super) fn credits_animate_the_triangles(&mut self) {
        self.attract_scene_mut().increment_intro_frame_counter();
        self.activate_nmi_thread();
        if self.game_state.ending.attract_scene.intro_did_run_step() == 0 {
            self.poly_runtime_mut().add_angle_b(3);
            self.poly_runtime_mut().add_angle_a(1);
            self.attract_scene_mut().mark_intro_did_run_step();
        }
        self.scene_animate_every_sprite();
    }

    pub(super) fn initialize_scene_sprite_triforce_room_triangle(&mut self, k: usize) {
        self.write_intro_x(
            k,
            INITIALIZE_SCENE_SPRITE_TRIFORCE_ROOM_TRIANGLE_X_OFFSETS[k],
        );
        self.write_intro_y(
            k,
            INITIALIZE_SCENE_SPRITE_TRIFORCE_ROOM_TRIANGLE_Y_OFFSETS[k],
        );
        self.intro_actor_mut(k)
            .set_x_velocity(INITIALIZE_SCENE_SPRITE_TRIFORCE_ROOM_TRIANGLE_X_VELOCITIES[k] as u8);
        self.intro_actor_mut(k)
            .set_y_velocity(INITIALIZE_SCENE_SPRITE_TRIFORCE_ROOM_TRIANGLE_Y_VELOCITIES[k] as u8);
        self.intro_actor_mut(k).increment_init_phase();
    }

    pub(super) fn intro_sprite_type_b_456(&mut self, k: usize) {
        self.intro_copy_sprite_type4_to_oam(k);
        if self
            .game_state
            .ending
            .intro_scene
            .triangle_motion_is_paused()
        {
            return;
        }
        self.animate_scene_sprite_move_triangle(k);
        match self.game_state.ending.attract_scene.intro_step_index() {
            0 => {
                if self.game_state.ending.attract_scene.intro_frame_counter() & 7 == 0 {
                    self.intro_actor_mut(k).add_x_velocity(
                        ANIMATE_SCENE_SPRITE_CREDITS_TRIANGLE_X_ACCELERATION[k] as u8,
                    );
                }
                if self.game_state.ending.attract_scene.intro_frame_counter() & 3 == 0 {
                    self.intro_actor_mut(k)
                        .add_y_velocity(INTRO_SPRITE_TYPE_B_456_Y_ACCELERATION[k] as u8);
                }
            }
            1 => {
                self.intro_actor_mut(k).set_x_velocity(0);
                self.intro_actor_mut(k).set_y_velocity(0);
            }
            2 => {
                if self.game_state.ending.attract_scene.intro_frame_counter() & 3 == 0 {
                    self.animate_triforce_room_triangle_handle_contracting(k);
                }
                if ANIMATE_TRIFORCE_ROOM_TRIANGLE_HANDLE_CONTRACTING_FINAL_X[k]
                    == self.intro_actor(k).x_low()
                {
                    self.intro_actor_mut(k).set_x_velocity(0);
                }
                if ANIMATE_TRIFORCE_ROOM_TRIANGLE_HANDLE_CONTRACTING_FINAL_Y[k]
                    == self.intro_actor(k).y_low()
                {
                    self.intro_actor_mut(k).set_y_velocity(0);
                }
            }
            3 | 4 => {
                let ctr = self.game_state.ending.intro_scene.triforce_countdown;
                if ctr == 0 {
                    self.intro_actor_mut(k)
                        .set_y_low(INTRO_SPRITE_TYPE_B_456_FINAL_Y2[k]);
                } else {
                    self.decrement_triforce_countdown();
                }
            }
            _ => {}
        }
    }

    pub(super) fn animate_triforce_room_triangle_handle_contracting(&mut self, k: usize) {
        let xv = self.intro_actor(k).x_velocity().wrapping_add(
            if self.intro_actor(k).x_low()
                <= ANIMATE_TRIFORCE_ROOM_TRIANGLE_HANDLE_CONTRACTING_FINAL_X[k]
            {
                1
            } else {
                0xff
            },
        );
        self.intro_actor_mut(k).set_x_velocity(match xv {
            0x11 => 0x10,
            0xef => 0xf0,
            _ => xv,
        });
        let yv = self.intro_actor(k).y_velocity().wrapping_add(
            if self.intro_actor(k).y_low()
                <= ANIMATE_TRIFORCE_ROOM_TRIANGLE_HANDLE_CONTRACTING_FINAL_Y[k]
            {
                1
            } else {
                0xff
            },
        );
        self.intro_actor_mut(k).set_y_velocity(match yv {
            0x11 => 0x10,
            0xef => 0xf0,
            _ => yv,
        });
    }

    pub(super) fn initialize_scene_sprite_credits_triangle(&mut self, k: usize) {
        self.intro_actor_mut(k).set_x(i16::from(
            INITIALIZE_SCENE_SPRITE_CREDITS_TRIANGLE_X_OFFSETS[k],
        ));
        self.intro_actor_mut(k).set_y(i16::from(
            INITIALIZE_SCENE_SPRITE_CREDITS_TRIANGLE_Y_OFFSETS[k],
        ));
        self.intro_actor_mut(k).increment_init_phase();
    }

    pub(super) fn animate_scene_sprite_credits_triangle(&mut self, k: usize) {
        self.load_triforce_sprite_palette();
        self.intro_copy_sprite_type4_to_oam(k);
        self.animate_scene_sprite_move_triangle(k);
        if self.game_state.frame.submodule != 36 {
            self.intro_actor_mut(k).set_state(0);
            return;
        }
        if self.intro_actor(k).state() != 80 {
            self.intro_actor_mut(k).increment_state();
            self.intro_actor_mut(k)
                .add_x_velocity(ANIMATE_SCENE_SPRITE_CREDITS_TRIANGLE_X_ACCELERATION[k] as u8);
            self.intro_actor_mut(k)
                .add_y_velocity(ANIMATE_SCENE_SPRITE_CREDITS_TRIANGLE_Y_ACCELERATION[k] as u8);
        }
    }

    pub(super) fn Intro_DisplayLogo(&mut self) {
        self.intro_display_logo();
    }

    pub(super) fn Intro_SetupSwordAndIntroFlash(&mut self) {
        self.intro_setup_sword_and_intro_flash();
    }

    pub(super) fn Intro_PeriodicSwordAndIntroFlash(&mut self) {
        self.intro_periodic_sword_and_intro_flash();
    }

    pub(super) fn module1_a_credits(&mut self) {
        self.oam_state_mut().init_credits_region_base();
        match self.game_state.frame.submodule {
            0 | 4 | 6 | 8 | 10 | 12 | 14 | 16 | 18 | 24 | 26 | 28 | 30 => {
                self.credits_load_next_scene_overworld()
            }
            2 | 20 | 22 => self.credits_load_next_scene_dungeon(),
            1 | 5 | 7 | 9 | 11 | 13 | 15 | 17 | 19 | 25 | 27 | 29 | 31 => {
                self.credits_scroll_scene_overworld()
            }
            3 | 21 | 23 => self.credits_scroll_scene_dungeon(),
            32 => self.end_sequence_32(),
            33 => self.credits_brighten_triangles(),
            34 => self.credits_fade_color_and_begin_animating(),
            35 => self.credits_stop_credits_scroll(),
            36 => self.credits_fade_and_disperse_triangles(),
            37 => self.credits_fade_in_the_end(),
            38 => self.credits_hang_forever(),
            _ => {}
        }
    }

    pub(super) fn credits_load_next_scene_overworld(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.credits_load_scene_overworld_prep_gfx(),
            1 => self.credits_load_scene_overworld_overlay(),
            2 => self.credits_load_scene_overworld_load_map(),
            _ => {}
        }
        self.credits_add_ending_sequence_text();
    }

    pub(super) fn credits_load_next_scene_dungeon(&mut self) {
        self.credits_load_scene_dungeon();
        self.credits_add_ending_sequence_text();
    }

    pub(super) fn credits_prep_and_load_sprites(&mut self) {
        for k in (0..16).rev() {
            self.sprite_prep_reset_properties(k);
            self.sprite_slot_view_mut(k).set_state(0);
            self.sprite_slot_view_mut(k).set_flags5(0);
            self.sprite_slot_view_mut(k).set_deflection_bits(0);
        }
        let scene = (self.game_state.frame.submodule >> 1) as usize;
        match scene {
            2 => {
                self.sprite_slot_view_mut(6).set_y_velocity((-16i8) as u8);
                self.init_ending_sprites_overworld(scene);
            }
            3 => {
                self.sprite_slot_view_mut(5).set_a(22);
                self.sprite_slot_view_mut(0).set_y_velocity((-16i8) as u8);
                self.sprite_slot_view_mut(1).set_y_velocity(16);
                self.sprite_slot_view_mut(1).set_head_direction(1);
                for j in (0..=2).rev() {
                    self.sprite_slot_view_mut(2 + j).set_sprite_type(0x57);
                    self.sprite_slot_view_mut(2 + j).set_oam_flags(0x31);
                }
                self.init_ending_sprites_overworld(scene);
            }
            6 => {
                self.sprite_slot_view_mut(0).set_delay_main(255);
                self.sprite_slot_view_mut(1).set_delay_main(255);
                self.sprite_slot_view_mut(2).set_delay_main(255);
                self.init_ending_sprites_overworld(scene);
            }
            7 => {
                self.sprite_slot_view_mut(1).set_delay_main(255);
                self.init_ending_sprites_overworld(scene);
            }
            9 => {
                for j in (0..=4).rev() {
                    self.sprite_slot_view_mut(j).set_delay_main((j * 19) as u8);
                    self.sprite_slot_view_mut(j).set_state(0);
                }
                self.sprite_slot_view_mut(5).set_sprite_type(0x2e);
                for j in (0..=1).rev() {
                    self.sprite_slot_view_mut(7 + j).set_sprite_type(0x9f);
                    self.sprite_slot_view_mut(9 + j).set_sprite_type(0xa0);
                    self.sprite_slot_view_mut(7 + j).set_flags2(1);
                    self.sprite_slot_view_mut(9 + j).set_flags2(2);
                    self.sprite_slot_view_mut(7 + j).set_flags3(0x10);
                    self.sprite_slot_view_mut(9 + j).set_flags3(0x10);
                }
                self.init_ending_sprites_overworld(scene);
            }
            10 => {
                self.sprite_slot_view_mut(1).set_delay_main(0x10);
                self.sprite_slot_view_mut(2).set_delay_main(0x20);
                self.sprite_slot_view_mut(3).set_oam_flags(8);
                self.sprite_slot_view_mut(4).set_oam_flags(8);
                self.init_ending_sprites_dungeon(scene);
            }
            11 => {
                self.sprite_slot_view_mut(4).set_oam_flags(0x79);
                self.sprite_slot_view_mut(5).set_oam_flags(0x39);
                self.sprite_slot_view_mut(1).set_direction(1);
                self.sprite_slot_view_mut(1).set_a(4);
                self.init_ending_sprites_dungeon(scene);
            }
            12 => {
                for j in (0..=1).rev() {
                    self.sprite_slot_view_mut(j + 3).set_oam_flags(0x39);
                    self.sprite_slot_view_mut(j + 3).set_sprite_type(0x0b);
                    self.sprite_slot_view_mut(j + 3).set_flags3(0x10);
                    self.sprite_slot_view_mut(j + 3).set_flags2(1);
                }
                self.sprite_slot_view_mut(5).set_sprite_type(0x2a);
                self.sprite_slot_view_mut(6).set_sprite_type(0x79);
                self.sprite_slot_view_mut(6).set_ai_state(1);
                self.sprite_slot_view_mut(6).set_z(5);
                self.init_ending_sprites_overworld(scene);
            }
            14 => {
                self.sprite_slot_view_mut(5).set_y_velocity((-16i8) as u8);
                self.sprite_slot_view_mut(6).set_y_velocity(16);
                self.sprite_slot_view_mut(6).set_head_direction(1);
                self.sprite_slot_view_mut(0).set_a(8);
                for j in (0..=3).rev() {
                    self.sprite_slot_view_mut(1 + j).set_y_velocity(4);
                }
                self.init_ending_sprites_overworld(scene);
            }
            15 => {
                self.sprite_slot_view_mut(4).set_c(2);
                self.sprite_slot_view_mut(5).set_y_velocity(8);
                self.sprite_slot_view_mut(1).set_delay_main(0x13);
                self.sprite_slot_view_mut(4).set_delay_main(0x40);
                self.init_ending_sprites_overworld(scene);
            }
            0 | 4 | 5 | 8 | 13 => self.init_ending_sprites_overworld(scene),
            1 => self.init_ending_sprites_dungeon(scene),
            _ => {}
        }
    }

    fn init_ending_sprites_overworld(&mut self, scene: usize) {
        let idx = ENDING_SCENE_SPRITE_RANGES[scene];
        let num = ENDING_SCENE_SPRITE_RANGES[scene + 1] - idx;
        let area = self.game_state.world.region.overworld_area_index();
        let base_x = area.wrapping_shl(9) & 0x0f00;
        let base_y = area.wrapping_shl(6) & 0x0e00;
        for k in (0..num).rev() {
            self.garnish_state_mut().set_sprcoll_x_size(0xffff);
            self.garnish_state_mut().set_sprcoll_y_size(0xffff);
            let x = base_x.wrapping_add(ENDING_SPRITE_X_OFFSETS[idx + k]);
            let y = base_y.wrapping_add(ENDING_SPRITE_Y_OFFSETS[idx + k]);
            self.sprite_slot_view_mut(k).set_x_low(x as u8);
            self.sprite_slot_view_mut(k).set_x_high((x >> 8) as u8);
            self.sprite_slot_view_mut(k).set_y_low(y as u8);
            self.sprite_slot_view_mut(k).set_y_high((y >> 8) as u8);
        }
    }

    fn init_ending_sprites_dungeon(&mut self, scene: usize) {
        let idx = ENDING_SCENE_SPRITE_RANGES[scene];
        let num = ENDING_SCENE_SPRITE_RANGES[scene + 1] - idx;
        let room = self.game_state.dungeon.room_tracking.room_index2_word();
        self.sprite_workspace_mut()
            .set_room_origin_y_high(((room >> 3) as u8) & 0xfe);
        self.sprite_workspace_mut()
            .set_room_origin_x_high(((room & 15) << 1) as u8);
        for k in (0..num).rev() {
            self.garnish_state_mut().set_sprcoll_x_size(0xffff);
            self.garnish_state_mut().set_sprcoll_y_size(0xffff);
            let x = ((self.game_state.sprites.workspace.room_origin_x_high() as u16) << 8)
                .wrapping_add(ENDING_SPRITE_X_OFFSETS[idx + k]);
            let y = ((self.game_state.sprites.workspace.room_origin_y_high() as u16) << 8)
                .wrapping_add(ENDING_SPRITE_Y_OFFSETS[idx + k]);
            self.sprite_slot_view_mut(k).set_x_low(x as u8);
            self.sprite_slot_view_mut(k).set_x_high((x >> 8) as u8);
            self.sprite_slot_view_mut(k).set_y_low(y as u8);
            self.sprite_slot_view_mut(k).set_y_high((y >> 8) as u8);
        }
    }

    pub(super) fn credits_scroll_scene_overworld(&mut self) {
        for k in (0..16).rev() {
            if self.sprite_slot_view(k).delay_main() != 0 {
                let delay = self.sprite_slot_view(k).delay_main().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_main(delay);
            }
        }
        let i = (self.game_state.frame.submodule >> 1) as usize;
        self.follower_link_state_mut().clear_movement_velocity();
        let r16 = self.game_state.dungeon.scratch_word.primary_word();
        if r16 >= 0x40 && r16 & 1 == 0 {
            if self.game_state.display.ppu_scroll_copy.bg2_v_copy2()
                != ENDING_SCENE_SCROLL_TARGET_Y[i]
            {
                self.follower_link_state_mut()
                    .set_y_velocity(ENDING_SCENE_SCROLL_Y_VELOCITIES[i] as u8);
            }
            if self.game_state.display.ppu_scroll_copy.bg2_h_copy2()
                != ENDING_SCENE_SCROLL_TARGET_X[i]
            {
                self.follower_link_state_mut()
                    .set_x_velocity(ENDING_SCENE_SCROLL_X_VELOCITIES[i] as u8);
            }
        }
        self.credits_operate_scrolling_and_tile_map();
        self.credits_handle_scene_fade();
    }

    pub(super) fn credits_scroll_scene_dungeon(&mut self) {
        for k in (0..16).rev() {
            if self.sprite_slot_view(k).delay_main() != 0 {
                let delay = self.sprite_slot_view(k).delay_main().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_main(delay);
            }
        }
        let i = (self.game_state.frame.submodule >> 1) as usize;
        let r16 = self.game_state.dungeon.scratch_word.primary_word();
        if r16 >= 0x40 && r16 & 1 == 0 {
            if self.game_state.display.ppu_scroll_copy.bg2_v_copy2()
                != ENDING_SCENE_SCROLL_TARGET_Y[i]
            {
                self.add_bg2_v_copy2_signed(ENDING_SCENE_SCROLL_Y_VELOCITIES[i]);
            }
            if self.game_state.display.ppu_scroll_copy.bg2_h_copy2()
                != ENDING_SCENE_SCROLL_TARGET_X[i]
            {
                self.add_bg2_h_copy2_signed(ENDING_SCENE_SCROLL_X_VELOCITIES[i]);
            }
        }
        self.credits_handle_scene_fade();
    }

    pub(super) fn credits_handle_scene_fade(&mut self) {
        let i = (self.game_state.frame.submodule >> 1) as usize;
        let r16 = self.game_state.dungeon.scratch_word.primary_word();
        match i {
            0 => {
                for k in (8..=11).rev() {
                    self.sprite_slot_view_mut(k)
                        .set_oam_flags(CREDITS_HANDLE_SCENE_FADE_CASE0_OAM_FLAGS[k]);
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE0_SPRITE_GFX[k],
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE0_SPRITE_CHARS[k],
                    );
                }
                for k in (2..=7).rev() {
                    let oam_flags = CREDITS_HANDLE_SCENE_FADE_CASE0_OAM_FLAGS[k]
                        | ((self.game_state.frame.frame_counter << 2) & 0x40);
                    self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE0_SPRITE_GFX[k],
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE0_SPRITE_CHARS[k],
                    );
                }
                for k in (0..=1).rev() {
                    self.sprite_slot_view_mut(k)
                        .set_oam_flags(CREDITS_HANDLE_SCENE_FADE_CASE0_OAM_FLAGS[k]);
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE0_SPRITE_GFX[k],
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE0_SPRITE_CHARS[k],
                    );
                }
            }
            1 => {
                self.credits_sprite_draw_single(0, 3, 12);
                self.credits_sprite_draw_draw_shadow(0);
                let k = 1;
                self.sprite_slot_view_mut(k).set_sprite_type(0x73);
                self.sprite_slot_view_mut(k).set_oam_flags(0x27);
                self.sprite_slot_view_mut(k).set_e(2);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 16);
            }
            2 => {
                let bird_frame_idx = ((self.game_state.frame.frame_counter >> 2) & 1) as usize;
                self.set_flag_travel_bird(
                    CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE2_BIRD_FLAG_FRAMES[bird_frame_idx],
                );
                let mut k = 6usize;
                let j = ((self.sprite_slot_view(k).x_velocity() >> 7) & 1) as usize;
                let oam_flags = self.sprite_slot_view(k).x_velocity().wrapping_add(
                    CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE2_BIRD_OAM_VELOCITY_OFFSETS[j] as u8,
                ) >> 1
                    & 0x40
                    | 0x32;
                self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                self.credits_sprite_draw_single(k, 2, 0x24);
                self.credits_sprite_draw_circling_birds(k);
                k -= 1;
                self.sprite_slot_view_mut(k).set_oam_flags(0x31);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = self.sprite_slot_view(k).a() as usize;
                    let a = self.sprite_slot_view(k).a() ^ 1;
                    self.sprite_slot_view_mut(k).set_a(a);
                    self.sprite_slot_view_mut(k)
                        .set_delay_main(CREDITS_HANDLE_SCENE_FADE_CASE2_DELAYS[j]);
                    let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1) & 3;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
                self.credits_sprite_draw_single(k, 2, 0x26);
                k -= 1;
                loop {
                    if self.game_state.frame.frame_counter & 15 == 0 {
                        let graphics = self.sprite_slot_view(k).graphics() ^ 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    }
                    self.sprite_slot_view_mut(k).set_oam_flags(0x31);
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE2_SPRITE_GFX[k],
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE2_SPRITE_CHARS[k],
                    );
                    self.end_sequence_draw_shadow2(k);
                    if k == 0 {
                        break;
                    }
                    k -= 1;
                }
            }
            3 => {
                let mut k = 0usize;
                while k < 5 {
                    if k < 2 {
                        self.sprite_slot_view_mut(k).set_sprite_type(1);
                        self.sprite_slot_view_mut(k).set_oam_flags(0x0b);
                        self.credits_sprite_draw_set_shadow_prop(k, 2);
                        self.sprite_slot_view_mut(k).set_z(48);
                        let j = ((self.game_state.frame.frame_counter.wrapping_add(if k != 0 {
                            0x5f
                        } else {
                            0x7d
                        })) >> 2
                            & 3) as usize;
                        self.sprite_slot_view_mut(k)
                            .set_graphics(CREDITS_HANDLE_SCENE_FADE_CASE3_GRAPHICS[j]);
                        self.credits_sprite_draw_circling_birds(k);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 12);
                    } else {
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 16);
                    }
                    k += 1;
                }
                self.credits_sprite_draw_single(k, 2, 0x38);
                self.ending_func2(k, 0x30);
                k += 1;
                self.credits_sprite_draw_single(k, 3, 0x3a);
            }
            4 => {
                let mut k = 2usize;
                self.sprite_slot_view_mut(k).set_oam_flags(0x35);
                self.credits_sprite_draw_single(k, 1, 0x3c);
                k -= 1;
                loop {
                    let oam_flags =
                        self.sprite_slot_view(k).x_velocity().wrapping_sub(1) >> 1 & 0x40 ^ 0x71;
                    self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                    let graphics = self.game_state.frame.frame_counter >> 3 & 1;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                    if r16 >= CREDITS_HANDLE_SCENE_FADE_CASE4_COUNTERS[k]
                        && self.sprite_slot_view(k).delay_main() == 0
                    {
                        let a = CREDITS_HANDLE_SCENE_FADE_CASE4_DELAY_VELOCITIES
                            [self.sprite_slot_view(k).a() as usize];
                        self.sprite_slot_view_mut(k).set_delay_main(a & 0xf8);
                        self.sprite_slot_view_mut(k).set_y_velocity(
                            CREDITS_HANDLE_SCENE_FADE_CASE4_XY_VELOCITIES[((a & 7) + 2) as usize]
                                as u8,
                        );
                        self.sprite_slot_view_mut(k).set_x_velocity(
                            CREDITS_HANDLE_SCENE_FADE_CASE4_XY_VELOCITIES[(a & 7) as usize] as u8,
                        );
                        self.sprite_slot_view_mut(k).increment_a();
                    }
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE4_SPRITE_GFX[k],
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE4_SPRITE_CHARS[k],
                    );
                    self.end_sequence_draw_shadow2(k);
                    self.sprite_move_xy(k);
                    if k == 0 {
                        break;
                    }
                    k -= 1;
                }
            }
            5 => {
                if r16 == 0x200 {
                    self.set_sound_effect_1(1);
                } else if r16 == 0x208 {
                    self.set_sound_effect_1(0x2c);
                }
                if r16.wrapping_sub(0x208) < 0x30 {
                    self.credits_sprite_draw_add_sparkle(2, 10, r16.wrapping_sub(0x208) as u8);
                }
                let mut k = 3usize;
                if r16 >= 0x200 {
                    self.sprite_slot_view_mut(k).set_graphics(1);
                }
                self.sprite_slot_view_mut(k).set_oam_flags(0x31);
                self.credits_sprite_draw_single(k, 4, 8);
                self.end_sequence_draw_shadow2(k);
                let j = self.sprite_slot_view(k).graphics() as usize;
                k -= 1;
                self.sprite_slot_view_mut(k).set_graphics(j as u8);
                {
                    self.follower_link_state_mut()
                        .set_sword_dma_graphics_index(0);
                    self.follower_link_state_mut()
                        .set_shield_dma_graphics_index(
                            CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE5_SHIELD_DMA_GFX[j],
                        );
                }
                self.sprite_slot_view_mut(k).set_oam_flags(0x30);
                self.follower_link_state_mut()
                    .set_link_dma_graphics_index_word(
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE5_LINK_DMA_GFX[j],
                    );
                self.credits_sprite_draw_single(
                    k,
                    5,
                    CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE5_SPRITE_CHARS[j],
                );
                self.end_sequence_draw_shadow2(k);
            }
            6 => {
                let idx = ENDING_SCENE_SPRITE_RANGES[i];
                let num = ENDING_SCENE_SPRITE_RANGES[i + 1] - idx;
                for k in (0..num).rev() {
                    self.sprite_system_mut().set_cur_object_index(k as u8);
                    self.sprite_slot_view_mut(k)
                        .set_sprite_type(CREDITS_HANDLE_SCENE_FADE_SPRITE_TYPES[k]);
                    self.oam_allocate_from_region_a(CREDITS_HANDLE_SCENE_FADE_OAM_SIZES[k]);
                    self.sprite_slot_view_mut(k)
                        .set_ai_state(CREDITS_HANDLE_SCENE_FADE_STATES[k]);
                    let j = if r16 >= 0x26f { k + 3 } else { k };
                    if r16 == 0x26f {
                        self.set_sound_effect_2(0x21);
                    }
                    self.sprite_slot_view_mut(k)
                        .set_graphics(CREDITS_HANDLE_SCENE_FADE_GRAPHICS[j]);
                    self.sprite_slot_view_mut(k).set_oam_flags(0x33);
                    self.sprite_get_16_bit_coords_ending(k);
                    self.sprite_active_main_ending(k);
                }
            }
            7 => {
                let mut k = 1usize;
                self.credits_sprite_draw_set_shadow_prop(k, 2);
                self.sprite_slot_view_mut(k).set_sprite_type(0xe9);
                self.oam_allocate_from_region_a(0x0c);
                self.sprite_slot_view_mut(k).set_oam_flags(0x37);
                self.sprite_get_16_bit_coords_ending(k);
                if self.game_state.frame.frame_counter & 15 == 0 {
                    let graphics = self.sprite_slot_view(k).graphics() ^ 1;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
                self.sprite_active_main_ending(k);
                if r16 >= 0x180 {
                    self.sprite_slot_view_mut(k).set_y_velocity(4);
                    if self.sprite_slot_view(k).y_low() != 0x7c {
                        self.sprite_move_xy(k);
                    }
                }
                k -= 1;
                self.sprite_slot_view_mut(k).set_sprite_type(0x36);
                self.oam_allocate_from_region_a(0x18);
                self.sprite_slot_view_mut(k).set_oam_flags(0x39);
                self.sprite_get_16_bit_coords_ending(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).set_delay_main(4);
                    let graphics = self.sprite_slot_view(k).graphics().wrapping_add(
                        CREDITS_HANDLE_SCENE_FADE_GRAPHICS_STEPS[((r16 >> 9) & 1) as usize] as u8,
                    ) & 7;
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
                self.sprite_active_main_ending(k);
            }
            8 => {
                let k = 0usize;
                self.sprite_slot_view_mut(k).set_sprite_type(0x2c);
                self.oam_allocate_from_region_a(0x2c);
                self.sprite_slot_view_mut(k).set_oam_flags(0x3b);
                self.sprite_get_16_bit_coords_ending(k);
                let graphics = if r16 < 0x1c0 {
                    ((r16 >> 5) & 1) as u8
                } else {
                    2
                };
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                self.sprite_active_main_ending(k);
            }
            9 => {
                let mut k = 0usize;
                while k < 5 {
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        self.sprite_slot_view_mut(k).set_delay_main(96);
                        self.sprite_slot_view_mut(k).set_state(96);
                        self.sprite_slot_view_mut(k).set_x_velocity(0);
                        self.sprite_slot_view_mut(k).set_x_low(238);
                        self.sprite_slot_view_mut(k).set_x_high(4);
                        self.sprite_slot_view_mut(k).set_y_low(24);
                        self.sprite_slot_view_mut(k).set_y_high(11);
                    }
                    if self.sprite_slot_view(k).state() != 0 {
                        self.sprite_slot_view_mut(k).set_y_velocity((-8i8) as u8);
                        self.sprite_move_xy(k);
                        if self.game_state.frame.frame_counter & 1 == 0 {
                            let delta = if ((self.game_state.frame.frame_counter >> 5) ^ k as u8)
                                & 1
                                != 0
                            {
                                -1i8
                            } else {
                                1i8
                            };
                            self.sprite_slot_view_mut(k).add_x_velocity(delta as u8);
                        }
                        self.credits_sprite_draw_single(k, 1, 0x10);
                    }
                    k += 1;
                }
                loop {
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        let a = self.sprite_slot_view(k).a() as usize;
                        let delay = if k == 5 {
                            CREDITS_HANDLE_SCENE_FADE_DELAYS1[a]
                        } else {
                            CREDITS_HANDLE_SCENE_FADE_DELAYS2[a]
                        };
                        self.sprite_slot_view_mut(k).set_delay_main(delay);
                        let next_a = self.sprite_slot_view(k).a().wrapping_add(1) & 3;
                        self.sprite_slot_view_mut(k).set_a(next_a);
                        let graphics = self.sprite_slot_view(k).graphics() ^ 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    }
                    if k == 5 {
                        self.sprite_slot_view_mut(k).set_oam_flags(0x31);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 0x10);
                        k += 1;
                    } else {
                        self.credits_sprite_draw_single(k, 2, 0x12);
                        k += 1;
                        break;
                    }
                }
                while k != 11 {
                    self.sprite_slot_view_mut(k)
                        .set_oam_flags(CREDITS_HANDLE_SCENE_FADE_OAM_FLAGS[k - 7]);
                    self.sprite_slot_view_mut(k)
                        .set_direction(CREDITS_HANDLE_SCENE_FADE_DIRECTIONS[k - 7]);
                    self.credits_sprite_draw_activate_and_run_sprite(
                        k,
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE8_RUN_SPRITE_CHARS[k - 7],
                    );
                    k += 1;
                }
            }
            10 => {
                let k = 5usize;
                self.sprite_get_16_bit_coords_ending(k);
                if self.sprite_slot_view(k).pause() == 0 {
                    let xb = CREDITS_HANDLE_SCENE_FADE_WISH_POND_X_OFFSETS
                        [(self.get_random_number() & 7) as usize]
                        .wrapping_add(self.game_state.sprites.workspace.current_sprite_x() as u8);
                    let yb = CREDITS_HANDLE_SCENE_FADE_WISH_POND_Y_OFFSETS
                        [(self.get_random_number() & 7) as usize]
                        .wrapping_add(self.game_state.sprites.workspace.current_sprite_y() as u8);
                    self.credits_sprite_draw_add_sparkle(3, xb, yb);
                }
                for k in 3..5 {
                    if self.sprite_slot_view(k).delay_aux1() != 0 {
                        let delay = self.sprite_slot_view(k).delay_aux1().wrapping_sub(1);
                        self.sprite_slot_view_mut(k).set_delay_aux1(delay);
                    }
                    self.sprite_slot_view_mut(k).set_sprite_type(0xe3);
                    self.credits_sprite_draw_set_shadow_prop(k, 1);
                    self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                }
                self.sprite_slot_view_mut(k).set_sprite_type(0x72);
                self.sprite_slot_view_mut(k).set_oam_flags(0x3b);
                self.sprite_slot_view_mut(k).set_state(9);
                self.sprite_slot_view_mut(k).set_b(9);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 0x30);
            }
            11 => {
                if r16 >= 0x170 {
                    for k in 4..6 {
                        self.credits_sprite_draw_single(k, 1, 0x3e);
                    }
                    let k = 0usize;
                    self.sprite_slot_view_mut(k).set_oam_flags(0x39);
                    if r16 < 0x1c0 {
                        self.sprite_slot_view_mut(k).set_graphics(2);
                    } else if self.sprite_slot_view(k).delay_main() == 0 {
                        self.sprite_slot_view_mut(k).set_delay_main(0x20);
                        let graphics = (self.sprite_slot_view(k).graphics() ^ 1) & 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    }
                    self.credits_sprite_draw_single(k, 4, 6);
                } else {
                    for k in 0..2 {
                        self.sprite_slot_view_mut(k).set_sprite_type(0x1a);
                        self.sprite_slot_view_mut(k).set_oam_flags(0x39);
                        self.credits_sprite_draw_set_shadow_prop(k, 2);
                        let bak0 = self.game_state.frame.main_module;
                        self.credits_sprite_draw_activate_and_run_sprite(k, 0x0c);
                        self.set_main_module(bak0);
                        if self.sprite_slot_view(k).b() == 15 && self.sprite_slot_view(k).a() == 4 {
                            self.sprite_slot_view_mut(k + 2).set_delay_main(15);
                        }
                        let j = self.sprite_slot_view(k + 2).delay_main();
                        if j != 0 {
                            self.sprite_slot_view_mut(k + 2).set_oam_flags(2);
                            self.sprite_slot_view_mut(k + 2)
                                .set_graphics(CREDITS_HANDLE_SCENE_FADE_GRAPHICS_2[j as usize]);
                            self.credits_sprite_draw_single(k + 2, 2, 0x36);
                        }
                    }
                }
            }
            12 => {
                let mut k = 6usize;
                let graphics = self.game_state.frame.frame_counter & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                if self.sprite_slot_view(k).graphics() == 0 {
                    let x_delta = if sign8(self.sprite_slot_view(k).x_low().wrapping_sub(0x80)) {
                        1
                    } else {
                        0xff
                    };
                    let y_delta = if sign8(self.sprite_slot_view(k).y_low().wrapping_sub(0xb0)) {
                        1
                    } else {
                        0xff
                    };
                    self.sprite_slot_view_mut(k).add_x_velocity(x_delta);
                    self.sprite_slot_view_mut(k).add_y_velocity(y_delta);
                    self.sprite_move_xy(k);
                }
                let oam_flags = (self.sprite_slot_view(k).x_velocity() >> 1 & 0x40) ^ 0x7e;
                self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                self.sprite_slot_view_mut(k).set_flags2(1);
                self.sprite_slot_view_mut(k).set_flags3(0x30);
                self.sprite_slot_view_mut(k).set_z(16);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 8);
                k -= 1;
                self.sprite_slot_view_mut(k).set_oam_flags(0x37);
                self.credits_sprite_draw_set_shadow_prop(k, 2);
                self.credits_sprite_draw_activate_and_run_sprite(k, 12);
                k -= 1;
                self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                k -= 1;
                self.credits_sprite_draw_activate_and_run_sprite(k, 8);
                k -= 1;
                loop {
                    self.credits_sprite_draw_single(
                        k,
                        CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE12_SPRITE_GFX[k],
                        (k * 2) as u8,
                    );
                    if k == 0 {
                        self.ending_func2(k, 0x30);
                    } else if k & !1 != 0 {
                        let graphics = self.game_state.frame.frame_counter >> 3 & 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                    } else {
                        let j = (self.game_state.frame.frame_counter & 0x1f) as usize;
                        if j < 0x0f {
                            self.sprite_slot_view_mut(k)
                                .set_z(CREDITS_HANDLE_SCENE_FADE_Z_OFFSETS[j]);
                        }
                        self.sprite_slot_view_mut(k)
                            .set_graphics(if j < 0x0f { 1 } else { 0 });
                        self.credits_sprite_draw_draw_shadow(k);
                    }
                    if k == 0 {
                        break;
                    }
                    k -= 1;
                }
            }
            13 => {
                let k = 0usize;
                if r16 == 0x200 {
                    self.sprite_slot_view_mut(k).set_x_velocity((-4i8) as u8);
                }
                let graphics = self.game_state.frame.frame_counter >> 4 & 1;
                self.sprite_slot_view_mut(k).set_graphics(graphics);
                if self.sprite_slot_view(k).x_low() == 56 {
                    self.sprite_slot_view_mut(k).set_x_velocity(0);
                    let graphics = self.sprite_slot_view(k).graphics().wrapping_add(2);
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                }
                self.credits_sprite_draw_single(k, 3, 0x34);
                self.sprite_move_xy(k);
            }
            14 => {
                let mut k = 6usize;
                while k != 0 {
                    if k >= 5 {
                        self.sprite_slot_view_mut(k).set_sprite_type(0);
                        self.credits_sprite_draw_set_shadow_prop(k, 1);
                        let graphics =
                            (self.game_state.frame.frame_counter.wrapping_add(0x4a) & 8) >> 3;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                        self.sprite_slot_view_mut(k).set_z(32);
                        self.credits_sprite_draw_circling_birds(k);
                        let oam_flags = (self.sprite_slot_view(k).x_velocity() >> 1 & 0x40) ^ 0x0f;
                        self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 8);
                    } else {
                        self.sprite_slot_view_mut(k).set_sprite_type(0x0d);
                        if k == 1 {
                            self.sprite_slot_view_mut(k).set_head_direction(0x0d);
                        }
                        self.credits_sprite_draw_set_shadow_prop(k, 3);
                        self.sprite_slot_view_mut(k).set_oam_flags(0x2b);
                        let mut a = self.sprite_slot_view(k).delay_main();
                        if a == 0 {
                            a = 0xc0;
                            self.sprite_slot_view_mut(k).set_delay_main(a);
                        }
                        a >>= 1;
                        if a == 0 {
                            self.sprite_slot_view_mut(k).set_y_velocity(0);
                            self.sprite_slot_view_mut(k).set_x_velocity(0);
                        } else if a < CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE14_TIMING_THRESHOLDS[k]
                            && self.game_state.frame.frame_counter & 3 == 0
                            && self.sprite_slot_view(k).y_velocity() != 0
                        {
                            let mut v = self.sprite_slot_view(k).y_velocity().wrapping_sub(1);
                            self.sprite_slot_view_mut(k).set_y_velocity(v);
                            v = v.wrapping_sub(4);
                            if k < 3 {
                                v = (0u8).wrapping_sub(v);
                            }
                            self.sprite_slot_view_mut(k).set_x_velocity(v);
                        }
                        self.sprite_move_xy(k);
                        let graphics = CREDITS_HANDLE_SCENE_FADE_CREDITS_CASE14_BIRD_GFX_FRAMES
                            [((self.game_state.frame.frame_counter >> 3) & 3) as usize];
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                        self.credits_sprite_draw_preexisting_sprite_draw(k, 16);
                    }
                    k -= 1;
                }
                self.credits_sprite_draw_single(k, 3, 0x18);
                self.ending_func2(k, 0x20);
            }
            15 => {
                let sparkle_table = self
                    .asset_raw(73)
                    .unwrap_or_else(|| panic!("missing ending asset 73"));
                let sparkle_idx = sparkle_table[self.game_state.frame.frame_counter as usize] & 3;
                self.credits_sprite_draw_add_sparkle(
                    2,
                    CREDITS_HANDLE_SCENE_FADE_X_OFFSETS[sparkle_idx as usize],
                    CREDITS_HANDLE_SCENE_FADE_Y_OFFSETS[sparkle_idx as usize],
                );
                let mut k = 2usize;
                self.sprite_slot_view_mut(k).set_sprite_type(0x62);
                self.sprite_slot_view_mut(k).set_oam_flags(0x39);
                self.credits_sprite_draw_preexisting_sprite_draw(k, 0x18);
                let mut j = 1u8;
                loop {
                    k += 1;
                    if self.sprite_slot_view(k).delay_aux1() != 0 {
                        let delay = self.sprite_slot_view(k).delay_aux1().wrapping_sub(1);
                        self.sprite_slot_view_mut(k).set_delay_aux1(delay);
                    }
                    let oam_flags = (self.sprite_slot_view(k).x_velocity() >> 1 & 0x40)
                        ^ CREDITS_HANDLE_SCENE_FADE_OAM_FLAGS_2[j as usize];
                    self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        self.sprite_slot_view_mut(k).set_delay_main(128);
                        self.sprite_slot_view_mut(k).set_a(0);
                    }
                    if self.sprite_slot_view(k).a() == 0 {
                        let graphics = (self.game_state.frame.frame_counter >> 2 & 1) + 2;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                        self.credits_sprite_draw_move_squirrel(k);
                    } else if self.sprite_slot_view(k).delay_aux1() == 0 {
                        if self.sprite_slot_view(k).b() == 8 {
                            self.sprite_slot_view_mut(k).set_b(0);
                        }
                        let b = self.sprite_slot_view(k).b() & 7;
                        self.sprite_slot_view_mut(k)
                            .set_delay_aux1(CREDITS_HANDLE_SCENE_FADE_DELAYS[b as usize]);
                        let graphics = (self.sprite_slot_view(k).graphics() & 1) ^ 1;
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                        let b = self.sprite_slot_view(k).b().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_b(b);
                    }
                    self.credits_sprite_draw_single(k, 1, 20);
                    self.end_sequence_draw_shadow2(k);
                    if j == 0 {
                        break;
                    }
                    j -= 1;
                }
                self.credits_sprite_draw_walk_link_away_from_pedestal(k + 1);
            }
            _ => {}
        }

        let k = (self.game_state.frame.submodule >> 1) as usize;
        let r16 = self.game_state.dungeon.scratch_word.primary_word();
        if r16 >= CREDITS_HANDLE_SCENE_FADE_CREDITS_SCENE_FADE_SCROLL_LIMITS[k] {
            if r16 & 1 == 0 {
                self.decrement_screen_brightness();
                if self.game_state.display.screen_brightness == 0 {
                    self.increment_submodule();
                } else {
                    self.ending_scratch_mut()
                        .set_primary_word(r16.wrapping_add(1));
                }
            } else {
                self.ending_scratch_mut()
                    .set_primary_word(r16.wrapping_add(1));
            }
        } else {
            if r16 & 1 == 0 && self.game_state.display.screen_brightness != 15 {
                self.increment_screen_brightness();
            }
            self.ending_scratch_mut()
                .set_primary_word(r16.wrapping_add(1));
        }
        self.copy_live_to_ppu_copy();
    }

    pub(super) fn credits_sprite_draw_draw_shadow(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_oam_flags(0x30);
        self.credits_sprite_draw_set_shadow_prop(k, 0);
        self.oam_allocate_from_region_a(4);
        let mut info = self.ending_coords;
        self.sprite_draw_shadow_custom(k, &mut info, 10);
        self.ending_coords = info;
    }

    pub(super) fn end_sequence_draw_shadow2(&mut self, k: usize) {
        self.credits_sprite_draw_set_shadow_prop(k, 0);
        self.oam_allocate_from_region_a(4);
        let mut info = self.ending_coords;
        self.sprite_draw_shadow_custom(k, &mut info, 10);
        self.ending_coords = info;
    }

    pub(super) fn ending_func2(&mut self, k: usize, ain: u8) {
        self.sprite_slot_view_mut(k).set_oam_flags(ain);
        self.end_sequence_draw_shadow2(k);
        let mut j = self.sprite_slot_view(k).a();
        if self.sprite_slot_view(k).delay_main() == 0 {
            j = j.wrapping_add(1);
            if j == 8 {
                j = 6;
            } else if j == 22 {
                j = 21;
            } else if j == 28 {
                j = 27;
            }
            self.sprite_slot_view_mut(k).set_a(j);
            self.sprite_slot_view_mut(k)
                .set_delay_main(ENDING_FUNC2_DELAYS[j.wrapping_sub(1) as usize]);
        }
        let a = ENDING_FUNC2_ENDING_SPRITE_ANIMATION_STEPS[j as usize];
        let graphics = if a == -1 {
            self.game_state.frame.frame_counter >> 3 & 1
        } else {
            a as u8
        };
        self.sprite_slot_view_mut(k).set_graphics(graphics);
        if (j < 5 || (10..15).contains(&j)) && self.game_state.frame.frame_counter & 1 == 0 {
            let y_low = self.sprite_slot_view(k).y_low().wrapping_add(1);
            self.sprite_slot_view_mut(k).set_y_low(y_low);
        }
    }

    pub(super) fn credits_sprite_draw_activate_and_run_sprite(&mut self, k: usize, a: u8) {
        self.sprite_system_mut().set_cur_object_index(k as u8);
        self.oam_allocate_from_region_a(a);
        self.sprite_get_16_bit_coords_ending(k);
        let bak0 = self.game_state.frame.submodule;
        self.set_submodule(0);
        self.sprite_slot_view_mut(k).set_state(9);
        self.sprite_active_main_ending(k);
        self.set_submodule(bak0);
    }

    pub(super) fn credits_sprite_draw_preexisting_sprite_draw(&mut self, k: usize, a: u8) {
        self.oam_allocate_from_region_a(a);
        self.sprite_system_mut().set_cur_object_index(k as u8);
        self.sprite_get_16_bit_coords_ending(k);
        self.sprite_active_main_ending(k);
    }

    pub(super) fn credits_sprite_draw_single(&mut self, k: usize, a: u8, j: u8) {
        self.oam_allocate_from_region_a(a.wrapping_mul(4));
        self.sprite_get_16_bit_coords_ending(k);
        let entries = END_SEQUENCE_DRAW_FRAME_SETS[(j >> 1) as usize];
        let start = a as usize * self.sprite_slot_view(k).graphics() as usize;
        let dmd: Vec<DrawMultipleData> = entries[start..start + a as usize]
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect();
        let mut info = PrepOamCoordsRet::default();
        self.sprite_draw_multiple(k, &dmd, Some(&mut info));
        self.ending_coords = info;
    }

    pub(super) fn credits_sprite_draw_set_shadow_prop(&mut self, k: usize, a: u8) {
        self.sprite_slot_view_mut(k).set_flags2(a);
        self.sprite_slot_view_mut(k).set_flags3(16);
    }

    pub(super) fn credits_sprite_draw_add_sparkle(&mut self, j_count: usize, xb: u8, yb: u8) {
        self.sprite_slot_view_mut(0).set_c(j_count as u8);
        for k in 0..j_count {
            let mut j = self.sprite_slot_view(k).graphics();
            if self.sprite_slot_view(k).delay_main() == 0 {
                j = j.wrapping_add(1);
                if j >= 6 {
                    self.sprite_slot_view_mut(k).set_x_low(xb);
                    self.sprite_slot_view_mut(k).set_y_low(yb);
                    j = 0;
                }
                self.sprite_slot_view_mut(k).set_graphics(j);
                self.sprite_slot_view_mut(k)
                    .set_delay_main(CREDITS_SPRITE_DRAW_ADD_SPARKLE_DELAYS[j as usize]);
            }
            if j != 0 {
                self.credits_sprite_draw_single(k, 1, 0x1c);
            }
        }
    }

    pub(super) fn credits_sprite_draw_walk_link_away_from_pedestal(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1) & 7;
            self.sprite_slot_view_mut(k).set_graphics(graphics);
            self.sprite_slot_view_mut(k).set_delay_main(4);
        }
        let dma = CREDITS_SPRITE_DRAW_WALK_LINK_AWAY_FROM_PEDESTAL_DMA_SOURCES
            [self.sprite_slot_view(k).graphics() as usize];
        self.follower_link_state_mut()
            .set_link_dma_graphics_index_word(dma);
        self.sprite_slot_view_mut(k).set_oam_flags(32);
        self.credits_sprite_draw_single(k, 2, 26);
        self.end_sequence_draw_shadow2(k);
        self.sprite_move_xy(k);
    }

    pub(super) fn credits_sprite_draw_move_squirrel(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() < 64 {
            let c = self.sprite_slot_view(k).c().wrapping_add(1) & 3;
            self.sprite_slot_view_mut(k).set_c(c);
            self.sprite_slot_view_mut(k).increment_a();
        } else {
            let j = self.sprite_slot_view(k).c() as usize;
            self.sprite_slot_view_mut(k)
                .set_x_velocity(CREDITS_SPRITE_DRAW_MOVE_SQUIRREL_X_VELOCITIES[j] as u8);
            self.sprite_slot_view_mut(k)
                .set_y_velocity(CREDITS_SPRITE_DRAW_MOVE_SQUIRREL_Y_VELOCITIES[j] as u8);
            self.sprite_move_xy(k);
        }
    }

    pub(super) fn credits_sprite_draw_circling_birds(&mut self, k: usize) {
        let j = self.sprite_slot_view(k).direction() & 1;
        let x_velocity = self
            .sprite_slot_view(k)
            .x_velocity()
            .wrapping_add(if j != 0 { 0xff } else { 1 });
        self.sprite_slot_view_mut(k).set_x_velocity(x_velocity);
        if self.sprite_slot_view(k).x_velocity()
            == CREDITS_SPRITE_DRAW_CIRCLING_BIRDS_TARGET_X_OFFSETS[j as usize] as u8
        {
            self.sprite_slot_view_mut(k).increment_direction();
        }
        if self.game_state.frame.frame_counter & 1 == 0 {
            let j = self.sprite_slot_view(k).head_direction() & 1;
            let y_velocity = self
                .sprite_slot_view(k)
                .y_velocity()
                .wrapping_add(if j != 0 { 0xff } else { 1 });
            self.sprite_slot_view_mut(k).set_y_velocity(y_velocity);
            if self.sprite_slot_view(k).y_velocity()
                == CREDITS_SPRITE_DRAW_CIRCLING_BIRDS_TARGET_Y_OFFSETS[j as usize] as u8
            {
                let head_direction = self.sprite_slot_view(k).head_direction().wrapping_add(1);
                self.sprite_slot_view_mut(k)
                    .set_head_direction(head_direction);
            }
        }
        self.sprite_move_xy(k);
    }

    pub(super) fn credits_handle_camera_scroll_control(&mut self) {
        if self.game_state.player.follower_link.y_velocity() != 0 {
            let y_vel = self.game_state.player.follower_link.y_velocity_signed();
            self.add_bg2_v_copy2_signed(y_vel);
            let which_axis = if y_vel < 0 { 0 } else { 1 };
            let other_axis = if y_vel < 0 { 1 } else { 0 };
            let mut value = self
                .game_state
                .world
                .camera_boundaries
                .overworld_scroll_counter_for_axis(which_axis)
                .wrapping_add(y_vel.unsigned_abs() as u16);
            if (value as i16).wrapping_sub(0x10) >= 0 {
                value = value.wrapping_sub(0x10);
                let bits =
                    self.screen_transition_direction_bits_word() | if y_vel < 0 { 8 } else { 4 };
                self.set_screen_transition_direction_bits_word(bits);
            }
            self.set_overworld_scroll_counter_for_axis(which_axis, value);
            self.set_overworld_scroll_counter_for_axis(other_axis, 0u16.wrapping_sub(value));
            let mut r4 = y_vel as i16 as u16;
            self.set_overworld_vertical_scroll_delta(r4);
            let oi = self.game_state.world.region.overlay_index();
            if oi != 0x97 && oi != 0x9d {
                let subp;
                if oi == 0xb5 || oi == 0xbe {
                    subp = (r4 & 3) << 14;
                    r4 = ((r4 as i16) >> 2) as u16;
                } else {
                    subp = (r4 & 1) << 15;
                    r4 = ((r4 as i16) >> 1) as u16;
                }
                self.add_bg1_v_copy2_subpixel(subp, r4);
            }
        }
        if self.game_state.player.follower_link.x_velocity() != 0 {
            let x_vel = self.game_state.player.follower_link.x_velocity_signed();
            self.add_bg2_h_copy2_signed(x_vel);
            let which_axis = if x_vel < 0 { 2 } else { 3 };
            let other_axis = if x_vel < 0 { 3 } else { 2 };
            let mut value = self
                .game_state
                .world
                .camera_boundaries
                .overworld_scroll_counter_for_axis(which_axis)
                .wrapping_add(x_vel.unsigned_abs() as u16);
            if (value as i16).wrapping_sub(0x10) >= 0 {
                value = value.wrapping_sub(0x10);
                let bits =
                    self.screen_transition_direction_bits_word() | if x_vel < 0 { 2 } else { 1 };
                self.set_screen_transition_direction_bits_word(bits);
            }
            self.set_overworld_scroll_counter_for_axis(which_axis, value);
            self.set_overworld_scroll_counter_for_axis(other_axis, 0u16.wrapping_sub(value));
            let mut r4 = x_vel as i16 as u16;
            self.set_overworld_horizontal_scroll_delta(r4);
            let oi = self.game_state.world.region.overlay_index();
            if oi != 0x97 && oi != 0x9d && r4 != 0 {
                let subp;
                if oi == 0x95 || oi == 0x9e {
                    subp = (r4 & 3) << 14;
                    r4 = ((r4 as i16) >> 2) as u16;
                } else {
                    subp = (r4 & 1) << 15;
                    r4 = ((r4 as i16) >> 1) as u16;
                }
                self.add_bg1_h_copy2_subpixel(subp, r4);
            }
        }
        if self.game_state.world.region.overlay_index() == 0x9c {
            self.subtract_bg1_v_copy2_subpixel(0x2000, 0);
            let bg1v = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg1_v_copy2()
                .wrapping_add(self.overworld_vertical_scroll_delta());
            self.set_bg1_y(bg1v);
            self.copy_bg2_h_live_to_bg1_h_live();
        } else if self.game_state.world.region.overlay_index() == 0x97
            || self.game_state.world.region.overlay_index() == 0x9d
        {
            self.add_bg1_v_copy2_subpixel(0x2000, 0);
            self.add_bg1_h_copy2_subpixel(0x2000, 0);
        }
        if self.game_state.world.location.dungeon_room() == 0x181 {
            let bg2v = self.game_state.display.ppu_scroll_copy.bg2_v_copy2() | 0x100;
            self.set_bg1_y(bg2v);
            self.copy_bg2_h_live_to_bg1_h_live();
        }
    }

    pub(super) fn end_sequence_32(&mut self) {
        self.enable_force_blank();
        self.erase_tile_maps_triforce();
        self.transfer_font_to_vram();
        self.credits_load_cool_background();
        self.credits_initialize_polyhedral();
        self.set_screen_brightness(128);
        self.select_overworld_aux_palette_offset();
        self.set_hud_palette(1);
        self.palette_load_hud();
        self.increment_cgram_update_flag();
        self.save_progress_mut().set_death_count_for_palace(4, 0);
        let palace_13 = self
            .game_state
            .inventory
            .save_progress
            .death_count_for_palace(13)
            .wrapping_add(
                self.game_state
                    .inventory
                    .save_progress
                    .pending_death_save_counter(),
            );
        self.save_progress_mut()
            .set_death_count_for_palace(13, palace_13);
        let mut sum = palace_13;
        for i in (0..=12).rev() {
            sum = sum.wrapping_add(
                self.game_state
                    .inventory
                    .save_progress
                    .death_count_for_palace(i),
            );
        }
        self.save_progress_mut().set_total_death_save_counter(sum);
        self.save_progress_mut().clear_pending_death_save_counter();
        let health = END_SEQUENCE_32_HEALTH_AFTER_DEATH
            [(self.game_state.inventory.player_resources.health_capacity() >> 3) as usize];
        self.player_resources_mut().set_current_health(health);
        self.save_progress_mut().set_dark_world_state(0x40);
        self.SaveGameFile();
        self.set_aux_color(38, 0);
        self.set_main_color(38, 0);
        self.set_aux_color(0, 0);
        self.set_main_color(0, 0);
        self.set_main_screen_layers(0x16);
        self.set_sub_screen_layers(0);
        self.ending_scratch_mut().set_primary_word(0x6800);
        self.ending_scratch_mut().set_secondary_word(0);
        self.clear_ending_palace_death_count_digit_step();
        self.set_bg2_y((-0x48i16) as u16);
        self.set_bg2_x(0x90);
        self.set_bg3_v_copy2(0);
        self.set_bg3_h_copy2(0);
        self.credits_add_next_attribution();
        self.set_music_control(0x22);
        self.set_color_window_selection(0);
        self.set_color_math_control(162);
        self.zelda_ppu_write(0x2108, 0x13);
        self.set_fixed_color_red(0x3f);
        self.set_fixed_color_green(0x5f);
        self.set_fixed_color_blue(0x9f);
        self.set_subsubmodule(64);
        self.set_screen_brightness(0);
        self.hdma_setup(0, 0xebd53, 0x42, 0, BG2HOFS as u8, 0);
        self.set_hdma_enable_mask(0x80);
        self.copy_live_to_ppu_copy();
    }

    pub(super) fn credits_fade_out_fixed_col(&mut self) {
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule == 0 {
            self.set_subsubmodule(16);
            if self.game_state.display.palette_filter.fixed_color_red() != 32 {
                self.subtract_fixed_color_red(1);
            } else if self.game_state.display.palette_filter.fixed_color_green() != 64 {
                self.subtract_fixed_color_green(1);
            } else if self.game_state.display.palette_filter.fixed_color_blue() != 128 {
                self.subtract_fixed_color_blue(1);
            }
        }
    }

    pub(super) fn credits_fade_color_and_begin_animating(&mut self) {
        self.credits_fade_out_fixed_col();
        self.set_core_update_disable_flag(1);
        self.credits_animate_the_triangles();
        if self.game_state.frame.frame_counter & 3 == 0 {
            let bg2 = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(1);
            self.set_bg2_x(bg2);
            if bg2 == 0x0c00 {
                self.zelda_ppu_write(0x2108, 0x13);
            }
            let a1 = bg2 >> 1;
            let a0 = a1.wrapping_add(bg2);
            self.room_bounds_mut()
                .set_packed_bounds(a0, a0 >> 1, a1, a1 >> 1);
            if self.game_state.display.ppu_scroll_copy.bg3_v_copy2() == 3288 {
                self.ending_scratch_mut().set_primary_word(0x80);
                self.increment_submodule();
            } else {
                self.add_bg3_v_copy2_signed(1);
                let bg3v = self.game_state.display.ppu_scroll_copy.bg3_v_copy2();
                if bg3v & 7 == 0 {
                    self.ending_scratch_mut().set_secondary_word(bg3v >> 3);
                    self.credits_add_next_attribution();
                }
            }
        }
        self.copy_live_to_ppu_copy();
    }

    pub(super) fn credits_add_next_attribution(&mut self) {
        let mut dst = self.game_state.display.current_vram_upload_data_address();
        let mut r16 = self.game_state.dungeon.scratch_word.primary_word();

        self.write_vram_upload_absolute_word(dst, r16.swap_bytes());
        self.write_vram_upload_absolute_word(dst + 2, 0x3e40);
        let blank_tile = self.ending_asset_u16(76, 159);
        self.write_vram_upload_absolute_word(dst + 4, blank_tile);
        dst += 6;

        let r18 = self.game_state.dungeon.scratch_word.secondary_word() as usize;
        if r18 < 394 {
            let text_off = self.ending_asset_u16(75, r18) as usize;
            let text = self
                .asset_raw(74)
                .unwrap_or_else(|| panic!("missing ending asset 74"))
                .to_vec();
            if text[text_off] != 0xff {
                let addr_delta = text[text_off] as u16;
                let n = text[text_off + 1];
                self.write_vram_upload_absolute_word(
                    dst,
                    r16.wrapping_add(addr_delta).swap_bytes(),
                );
                self.write_vram_upload_absolute_word(dst + 2, (n as u16).swap_bytes());
                dst += 4;
                let count = ((n.wrapping_add(1)) >> 1) as usize;
                for q in 0..count {
                    let ch = text[text_off + 2 + q] as usize;
                    let tile = self.ending_asset_u16(76, ch);
                    self.write_vram_upload_absolute_word(dst, tile);
                    dst += 2;
                }
            }

            let credits = &self.game_state.ending.credits;
            let which_idx = credits.palace_death_count_index();
            if credits.should_write_digit_for_scroll_y(
                (r18 as u16).wrapping_mul(2),
                CREDITS_ADD_NEXT_ATTRIBUTION_DIGITS_SCROLL_Y[which_idx],
            ) {
                let t = CREDITS_ADD_NEXT_ATTRIBUTION_DIGIT_CHARS[credits.digit_tile_base_index()];
                self.set_ending_death_count_digit_tile_base(t);
                self.write_vram_upload_absolute_word(dst, r16.wrapping_add(0x19).swap_bytes());
                self.write_vram_upload_absolute_word(dst + 2, 0x0500);
                let palace = CREDITS_ADD_NEXT_ATTRIBUTION_ATTRIBUTION_PALACE_ORDER[which_idx];
                // C reads the death count from RAM directly (DEATHS_PER_PALACE). The
                // native save_progress.dungeon_info is a 0x500-byte snapshot of the save
                // region that is NOT bulk-projected each frame, so it can lag a death
                // increment — reading it here showed a digit one off (frame 1070894+,
                // final credits). Read RAM live to match C.
                let mut deaths = crate::types::read_le_u16(
                    &self.ram,
                    crate::game_state::constants::DEATHS_PER_PALACE + palace * 2,
                );
                if deaths >= 1000 {
                    deaths = 999;
                }
                self.write_vram_upload_absolute_word(dst + 8, t.wrapping_add(deaths % 10));
                deaths /= 10;
                self.write_vram_upload_absolute_word(dst + 6, t.wrapping_add(deaths % 10));
                deaths /= 10;
                self.write_vram_upload_absolute_word(dst + 4, t.wrapping_add(deaths));
                dst += 10;
                self.advance_ending_palace_death_count_digit_step();
            }
        }

        r16 = r16.wrapping_add(0x20);
        if r16 & 0x3ff == 0 {
            r16 = (r16 & 0x6800) ^ 0x800;
        }
        self.ending_scratch_mut().set_primary_word(r16);
        let upload_base = self.game_state.display.vram_upload_buffer_base();
        self.set_vram_upload_cursor((dst - upload_base) as u16);
        self.write_vram_upload_absolute_byte(dst, 0xff);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn credits_add_ending_sequence_text(&mut self) {
        let mut dst = self.game_state.display.vram_upload_buffer_base();
        self.write_vram_upload_absolute_word(dst, 0x0060);
        self.write_vram_upload_absolute_word(dst + 2, 0xfe47);
        let blank_tile = self.ending_asset_u16(76, 159);
        self.write_vram_upload_absolute_word(dst + 4, blank_tile);
        dst += 6;

        let scene = (self.game_state.frame.submodule >> 1) as usize;
        let mut curo = self.ending_asset_u16(77, scene) as usize;
        let endo = self.ending_asset_u16(77, scene + 1) as usize;
        let data = self
            .asset_raw(78)
            .unwrap_or_else(|| panic!("missing ending asset 78"))
            .to_vec();
        while curo != endo {
            let a = u16::from_le_bytes([data[curo], data[curo + 1]]);
            let b = u16::from_le_bytes([data[curo + 2], data[curo + 3]]);
            self.write_vram_upload_absolute_word(dst, a);
            self.write_vram_upload_absolute_word(dst + 2, b);
            let m = ((b >> 9) & 0x7f) as usize;
            dst += 4;
            curo += 4;
            for _ in 0..=m {
                let ch = data[curo] as usize;
                let tile = self.ending_asset_u16(76, ch);
                self.write_vram_upload_absolute_word(dst, tile);
                dst += 2;
                curo += 1;
            }
        }
        let upload_base = self.game_state.display.vram_upload_buffer_base();
        self.set_vram_upload_cursor((dst - upload_base) as u16);
        self.write_vram_upload_absolute_byte(dst, 0xff);
        self.set_bg_vram_load_mode(1);
    }

    pub(super) fn credits_brighten_triangles(&mut self) {
        if self.game_state.frame.frame_counter & 15 == 0 {
            self.increment_screen_brightness();
            if self.game_state.display.screen_brightness == 15 {
                self.increment_submodule();
            }
        }
        self.credits_animate_the_triangles();
    }

    pub(super) fn credits_stop_credits_scroll(&mut self) {
        if self.ending_scratch_mut().decrement_primary_low() == 0 {
            self.set_darkening_or_lightening_screen_word(0);
            self.set_countdown_word(0);
            self.set_mosaic_target_level_word(0x1f);
            self.increment_submodule();
            self.ending_scratch_mut().set_primary_word(0x00c0);
            self.ending_scratch_mut().set_secondary_word(0);
        }
        self.credits_animate_the_triangles();
    }

    pub(super) fn credits_fade_and_disperse_triangles(&mut self) {
        self.ending_scratch_mut().decrement_primary_low();
        if self.game_state.dungeon.scratch_word.secondary_low() == 0 {
            self.apply_palette_filter_bounce();
            if self.game_state.display.palette_filter.countdown() != 0 {
                self.credits_animate_the_triangles();
                return;
            }
            self.ending_scratch_mut().increment_secondary_low();
        }
        if self.game_state.dungeon.scratch_word.primary_low() != 0 {
            self.credits_animate_the_triangles();
            return;
        }
        self.increment_submodule();
        self.PaletteFilter_WishPonds_Inner();
    }

    pub(super) fn credits_fade_in_the_end(&mut self) {
        if self.game_state.frame.frame_counter & 7 == 0 {
            self.PaletteFilter_SP5F();
            if self.game_state.display.palette_filter.countdown() == 0 {
                self.increment_submodule();
            }
        }
        self.credits_hang_forever();
    }

    pub(super) fn credits_hang_forever(&mut self) {
        self.set_oam_plain(0, 0xa0, 0xb8, 0x00, 0x3b, 2);
        self.set_oam_plain(1, 0xb0, 0xb8, 0x02, 0x3b, 2);
        self.set_oam_plain(2, 0xc0, 0xb8, 0x04, 0x3b, 2);
        self.set_oam_plain(3, 0xd0, 0xb8, 0x06, 0x3b, 2);
    }

    pub(super) fn crystal_cutscene_initialize_polyhedral(&mut self) {
        self.poly_runtime_mut().set_config1(156);
        self.poly_runtime_mut().set_color_mode(1);
        self.activate_nmi_thread();
        self.attract_scene_mut().mark_intro_did_run_step();
        self.poly_runtime_mut().set_base_x(32);
        self.poly_runtime_mut().set_base_y(32);
        self.poly_runtime_mut().set_shape_depth_bias_low(32);
        self.poly_runtime_mut().set_model(0);
        self.poly_runtime_mut().set_angle_a(16);
        self.set_sub_screen_layers(0);
        self.set_main_screen_layers(0x16);
    }
}

fn break_triforce_handle_poly(state: &mut ZeldaState) {
    state.attract_scene_mut().mark_intro_did_run_step();
    state.resume_intro_triangle_motion();
    state.attract_scene_mut().increment_intro_frame_counter();
}

impl ZeldaState {
    pub(super) fn fade_music_and_reset_sram_mirror(&mut self) {
        self.set_irq_control_flag(0xff);
        self.set_main_screen_layers(0x15);
        self.set_sub_screen_layers(0);
        self.set_indoor_flag(0);
        self.set_music_control(0xf1);
        self.set_backdrop_color_black();
        self.follower_link_state_mut()
            .clear_link_state_block_for_ending();
        self.save_progress_mut().clear_dungeon_info();
        self.set_main_module(1);
        self.set_restart_check_flag(1);
        self.set_submodule(0);
    }

    pub(super) fn load_triforce_sprite_palette(&mut self) {
        for (i, color) in POLYHEDRAL_PALETTE.iter().enumerate() {
            self.set_main_color(0xd0 + i, *color);
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn module00_intro(&mut self) {
        let skip_at = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_SKIP_INTRO_ON_KEYPRESS)
        {
            4
        } else {
            8
        };
        if self.game_state.frame.submodule >= skip_at
            && (((self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
                | self.game_state.player.follower_link.filtered_joypad_h())
                & 0xd0)
                != 0
        {
            self.fade_music_and_reset_sram_mirror();
            return;
        }

        match self.game_state.frame.submodule {
            0 => self.intro_init(),
            1 => self.intro_init_continue(),
            2 | 10 => self.intro_initialize_triforce_poly_thread(),
            3 | 4 | 9 | 11 => self.intro_handle_all_triforce_animations(),
            5 => self.intro_zelda_fadein(),
            6 => self.intro_sword_coming_down(),
            7 => self.intro_fade_in_bg(),
            8 => self.intro_wait_player(),
            _ => {}
        }
    }

    pub(super) fn intro_zelda_fadein(&mut self) {
        self.intro_handle_all_triforce_animations();
        if self.game_state.frame.frame_counter & 1 == 0 {
            return;
        }
        self.palette_fade_intro_one_step();
        if self.game_state.display.palette_filter.countdown() == 0 {
            self.set_subsubmodule(42);
            self.increment_submodule();
            self.intro_setup_sword_and_intro_flash();
        } else if self.game_state.display.palette_filter.countdown() == 13 {
            self.set_main_screen_layers(0x15);
            self.set_sub_screen_layers(0);
        }
    }

    pub(super) fn intro_setup_sword_and_intro_flash(&mut self) {
        self.intro_sword_mut().reset_sword_state();
        self.intro_periodic_sword_and_intro_flash();
    }

    pub(super) fn intro_sword_coming_down(&mut self) {
        self.intro_handle_all_triforce_animations();
        self.attract_scene_mut().clear_intro_did_run_step();
        self.deactivate_nmi_thread();
        self.intro_periodic_sword_and_intro_flash();
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule == 0 {
            self.increment_submodule();
            self.set_color_window_selection(2);
            self.set_color_math_control(0x22);
            self.set_countdown_word(31);
            self.set_sub_screen_layers(2);
        }
    }

    pub(super) fn intro_fade_in_bg(&mut self) {
        self.intro_periodic_sword_and_intro_flash();
        self.intro_handle_all_triforce_animations();
        if self.game_state.display.palette_filter.countdown() != 0 {
            if self.game_state.frame.frame_counter & 1 != 0 {
                self.palette_fade_intro2();
            }
        } else if (((self.game_state.player.follower_link.filtered_joypad_l() & 0xc0)
            | self.game_state.player.follower_link.filtered_joypad_h())
            & 0xd0)
            != 0
        {
            self.fade_music_and_reset_sram_mirror();
        } else {
            self.decrement_subsubmodule();
            if self.game_state.frame.subsubmodule == 0 {
                self.increment_submodule();
            }
        }
    }

    pub(super) fn intro_wait_player(&mut self) {
        self.intro_handle_all_triforce_animations();
        self.attract_scene_mut().clear_intro_did_run_step();
        self.deactivate_nmi_thread();
        self.intro_periodic_sword_and_intro_flash();
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule == 0 {
            self.increment_submodule();
            self.set_main_module(20);
            self.set_submodule(0);
            self.follower_link_state_mut().set_x(0);
        }
    }

    pub(super) fn intro_periodic_sword_and_intro_flash(&mut self) {
        if self.game_state.intro_sword.sparkle_timer() != 0 {
            self.intro_sword_mut().decrement_sparkle_timer();
        }
        self.set_backdrop_color_black();
        if self
            .game_state
            .ending
            .attract_scene
            .intro_palette_flash_count()
            != 0
        {
            if self
                .game_state
                .ending
                .attract_scene
                .intro_palette_flash_count()
                & 3
                != 0
            {
                let flash = if self
                    .game_state
                    .enhanced_features
                    .has(FEATURE_DIM_ENDING_FLASHES)
                {
                    0x05
                } else {
                    0x1f
                };
                let flash_component = self.game_state.intro_sword.flash_rgb_channel();
                self.or_fixed_color_component(flash_component, flash);
                self.intro_sword_mut().cycle_flash_rgb_channel();
            }
            self.attract_scene_mut()
                .decrement_intro_palette_flash_count();
        }

        let sword_y = self.game_state.intro_sword.ypos();
        for j in (0..10).rev() {
            let y = sword_y.wrapping_add(INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_Y_OFFSETS[j]);
            let visible_y = if y & 0xff00 != 0 { 0xf8u8 } else { y as u8 }.wrapping_sub(8);
            self.set_oam_plain(
                0x52 + j,
                INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_X_OFFSETS[j],
                visible_y,
                INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_CHARS[j],
                0x21,
                2,
            );
        }

        if sword_y != 30 {
            if sword_y == 0xffbe {
                self.set_sound_effect_1(1);
            } else if sword_y == 14 {
                self.intro_sword_mut().set_flash_rgb_channel_word(0);
                self.attract_scene_mut().set_intro_palette_flash_count(0x20);
                self.set_sound_effect_1(0x2c);
            }
            self.intro_sword_mut().set_ypos(sword_y.wrapping_add(16));
        }

        match self.game_state.intro_sword.anim_phase() {
            0 => {
                if self
                    .game_state
                    .ending
                    .attract_scene
                    .intro_palette_flash_count()
                    == 0
                    && self.game_state.intro_sword.ypos() == 30
                {
                    self.intro_sword_mut().advance_anim_step();
                }
            }
            1 => {
                if self.game_state.intro_sword.sparkle_timer() == 0 {
                    if self
                        .intro_sword_mut()
                        .decrement_sparkle_step_check_negative()
                    {
                        self.intro_sword_mut().set_sparkle_step(0);
                        self.intro_sword_mut().set_sparkle_timer(2);
                        self.intro_sword_mut().advance_anim_step();
                        return;
                    }
                    let timer = INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_INTRO_SWORD_SPARKLE_TIMERS
                        [self.game_state.intro_sword.sparkle_step() as usize];
                    self.intro_sword_mut().set_sparkle_timer(timer);
                }
                self.set_oam_plain(
                    0x50,
                    0x44,
                    0x43,
                    INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_SPARKLE_CHARS
                        [self.game_state.intro_sword.sparkle_step() as usize],
                    0x25,
                    0,
                );
            }
            2 => {
                let k = self.game_state.intro_sword.sparkle_step() as usize;
                if k >= 7 {
                    return;
                }
                let y_base = self.game_state.intro_sword.sparkle_y_offset().min(0x4f);
                let y = y_base
                    .wrapping_add(self.game_state.intro_sword.ypos() as u8)
                    .wrapping_add(0x31);
                self.set_oam_plain(
                    0x50,
                    0x42,
                    y,
                    INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_SPARKLE_CHARS_2[k],
                    0x23,
                    0,
                );
                self.set_oam_plain(
                    0x51,
                    0x42,
                    y.wrapping_add(8),
                    INTRO_PERIODIC_SWORD_AND_INTRO_FLASH_SPARKLE_CHARS_2[k + 1],
                    0x23,
                    0,
                );
                if self.game_state.intro_sword.sparkle_timer() == 0 {
                    self.intro_sword_mut().advance_sparkle_y_offset();
                    if matches!(
                        self.game_state.intro_sword.sparkle_y_offset(),
                        0x04 | 0x48 | 0x4c | 0x58
                    ) {
                        let step = self.game_state.intro_sword.sparkle_step().wrapping_add(2);
                        self.intro_sword_mut().set_sparkle_step(step);
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn intro_init(&mut self) {
        self.intro_setup_screen();
        self.set_screen_brightness(15);
        self.set_subsubmodule(0);
        self.intro_startup_delay = 0;
        self.increment_cgram_update_flag();
        self.increment_submodule();
        self.set_sound_effect_2(10);
        self.intro_init_continue();
    }

    pub(super) fn intro_setup_screen(&mut self) {
        self.set_core_update_disable_flag(0x80);
        self.enable_force_blank();
        self.set_main_screen_layers(16);
        self.set_sub_screen_layers(0);
        self.intro_initialize_background_settings();
        self.set_color_window_selection(0x20);
        self.set_chr_halfslot_request(20);
        self.graphics_load_chr_half_slot();
        self.clear_chr_halfslot_request();
        self.LoadOWMusicIfNeeded();

        for i in 0..17 {
            self.set_main_color(144 + i, 0x7fff);
            self.ppu.vram[0x27f0 + i] = 0;
        }

        self.ending_scratch_mut().set_primary_word(0x1ffe);
        self.ending_scratch_mut().set_secondary_word(0x1bfe);
    }

    pub(super) fn intro_initialize_background_settings(&mut self) {
        self.set_bg_mode(9);
        self.set_mosaic_copy(0);
        self.zelda_ppu_write(0x2107, 0x13);
        self.zelda_ppu_write(0x2108, 0x03);
        self.zelda_ppu_write(0x2109, 0x63);
        self.set_color_math_control(32);
        self.set_fixed_color_red(32);
        self.set_fixed_color_green(64);
        self.set_fixed_color_blue(128);
    }

    pub(super) fn intro_init_continue(&mut self) {
        self.intro_display_logo();
        let t = self.game_state.frame.subsubmodule;
        self.increment_subsubmodule();
        match t {
            0..=7 => self.intro_clear1kb_blocks_of_wram(),
            8 => self.intro_load_text_pointers_and_palettes(),
            9 => self.load_item_gfx_into_wram_4bpp_buffer(),
            10 => self.load_follower_graphics(),
            _ => {
                self.decrement_screen_brightness();
                if self.game_state.display.screen_brightness == 0 {
                    if self.rom_startup_timing() {
                        self.enable_force_blank();
                        let delay = configured_intro_memory_darken_frame_delay();
                        if delay != 0 {
                            self.intro_memory_darken_frame_delay = delay;
                            return;
                        }
                        self.intro_initialize_memory_darken_finish();
                        return;
                    }
                    self.intro_initialize_memory_darken();
                }
            }
        }
    }

    pub(super) fn intro_load_text_pointers_and_palettes(&mut self) {
        self.Text_GenerateMessagePointers();
        self.overworld_load_all_palettes();
    }

    pub(super) fn intro_initialize_memory_darken(&mut self) {
        self.enable_force_blank();
        self.intro_initialize_memory_darken_finish();
    }

    pub(super) fn intro_initialize_memory_darken_finish(&mut self) {
        self.erase_tile_maps_normal();
        self.world_palette_theme_mut().set_main_tile_theme_index(35);
        self.sprite_system_mut().set_graphics_index(125);
        self.world_palette_theme_mut().set_aux_tile_theme_index(81);
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(8);
        self.load_default_graphics();
        self.initialize_tilesets();
        self.decompress_animated_dungeon_tiles(0x5d);
        self.set_bg_tile_animation_countdown(2);
        self.set_overworld_screen(0);
        self.set_palette_main_indoors(0);
        self.set_overworld_palette_aux3_lo(0);
        self.ending_scratch_mut().set_primary_word(0);
        self.ending_scratch_mut().set_secondary_word(0);
        self.set_darkening_or_lightening_screen_word(2);
        self.set_countdown_word(31);
        self.clear_mosaic_target_level();
        self.increment_submodule();
    }

    pub(super) fn intro_initialize_triforce_poly_thread(&mut self) {
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(8);
        self.load_common_sprites();
        self.intro_init_gfx_helper();
        self.intro_actor_mut(0).set_init_phase(1);
        self.intro_actor_mut(1).set_init_phase(1);
        self.intro_actor_mut(2).set_init_phase(1);
        self.intro_actor_mut(0).set_subtype(0);
        self.intro_actor_mut(1).set_subtype(0);
        self.intro_actor_mut(2).set_subtype(0);
        self.intro_actor_mut(4).set_init_phase(1);
        self.intro_actor_mut(4).set_subtype(2);
        self.set_screen_brightness(15);
        self.increment_submodule();
    }

    pub(super) fn intro_init_gfx_helper(&mut self) {
        self.polyhedral_initialize_thread();
        self.load_triforce_sprite_palette();
        self.set_vertical_irq_trigger(0x90);
        self.poly_runtime_mut().set_config1(0xff);
        self.poly_runtime_mut().set_base_x(32);
        self.poly_runtime_mut().set_base_y(32);
        self.poly_runtime_mut().set_shape_depth_bias_low(32);
        self.poly_runtime_mut().set_angle_a(0xa0);
        self.poly_runtime_mut().set_angle_b(0x60);
        self.poly_runtime_mut().set_color_mode(1);
        self.poly_runtime_mut().set_model(1);
        self.activate_nmi_thread();
        self.attract_scene_mut().mark_intro_did_run_step();
        if self.rom_startup_timing() {
            self.intro_poly_upload_delay = configured_intro_thread_start_delay();
            self.intro_sprite_animation_start_delay =
                configured_intro_sprite_animation_start_delay();
        }
        self.attract_scene_mut().clear_intro_step_state_block();
        // clear_intro_step_state_block fills ram[INTRO_STEP_INDEX..+0x70] (0x1e00-0x1e6f) to 0,
        // which includes the intro actor slots (IS_INITED/SUBTYPE/STATE/X/Y at 0x1e10-0x1e4d), but
        // only resyncs the attract-scene native (step index/timer/frame counter). The IntroActorState
        // native still holds the previous scene's actors and would re-stamp the just-cleared slots,
        // leaving NEW a scene out of phase with the old clone at the intro reset (f585167). Resync it.
        self.game_state.ending.intro_actors =
            crate::game_state::IntroActorState::load_from_ram(&self.ram);
        if self.rom_startup_timing() {
            for _ in 0..configured_intro_poly_bootstrap_steps() {
                self.intro_run_step();
            }
        }
    }

    pub(super) fn polyhedral_initialize_thread(&mut self) {
        self.clear_poly_thread_work_area();
        self.set_nmi_thread_stack_pointer(0x1f31);
        self.write_poly_thread_init_bytes();
    }

    pub(super) fn intro_handle_all_triforce_animations(&mut self) {
        if self.rom_startup_timing() && self.intro_sprite_animation_start_delay != 0 {
            self.intro_sprite_animation_start_delay =
                self.intro_sprite_animation_start_delay.saturating_sub(1);
            self.intro_animate_triforce();
            return;
        }
        self.attract_scene_mut().increment_intro_frame_counter();
        self.intro_animate_triforce();
        self.scene_animate_every_sprite();
    }

    pub(super) fn intro_animate_triforce(&mut self) {
        self.activate_nmi_thread();
        if self.rom_startup_timing() && self.intro_memory_darken_frame_delay == 0 {
            if self.intro_poly_upload_delay != 0 {
                self.intro_poly_upload_delay = self.intro_poly_upload_delay.saturating_sub(1);
                self.attract_scene_mut().mark_intro_did_run_step();
                return;
            }
            if self.bsnes_hold_intro_step_this_frame {
                self.attract_scene_mut().mark_intro_did_run_step();
                return;
            }
            self.intro_run_step();
            self.attract_scene_mut().mark_intro_did_run_step();
            return;
        }
        if self.game_state.ending.attract_scene.intro_did_run_step() == 0 {
            self.intro_run_step();
            self.attract_scene_mut().mark_intro_did_run_step();
        }
    }

    pub(super) fn intro_run_step(&mut self) {
        match self.game_state.ending.attract_scene.intro_step_index() {
            0 => {
                self.attract_scene_mut().increment_intro_step_timer();
                if self.game_state.ending.attract_scene.intro_step_timer() == 64 {
                    self.attract_scene_mut().increment_intro_step_index();
                }
                self.poly_runtime_mut().add_angle_b(5);
                self.poly_runtime_mut().add_angle_a(3);
            }
            1 => {
                if self.game_state.poly.runtime.config1() < 2 {
                    self.poly_runtime_mut().clear_config1();
                    self.attract_scene_mut().increment_intro_step_index();
                    self.attract_scene_mut().set_intro_step_timer(64);
                    return;
                }
                self.poly_runtime_mut().subtract_config1(2);
                self.poly_runtime_mut().add_angle_b(5);
                self.poly_runtime_mut().add_angle_a(3);
                if self.game_state.poly.runtime.config1() < 225 {
                    self.set_submodule(4);
                }
                if self.game_state.poly.runtime.config1() == 113 {
                    self.set_music_control(1);
                }
            }
            2 => {
                self.attract_scene_mut().decrement_intro_step_timer();
                if self.game_state.ending.attract_scene.intro_step_timer() == 0 {
                    self.attract_scene_mut().increment_intro_step_index();
                } else {
                    self.poly_runtime_mut().add_angle_b(5);
                    self.poly_runtime_mut().add_angle_a(3);
                }
            }
            3 => {
                if self.game_state.poly.runtime.angle_b() >= 250
                    && self.game_state.poly.runtime.angle_a() >= 252
                {
                    self.attract_scene_mut().increment_intro_step_index();
                    self.attract_scene_mut().set_intro_step_timer(32);
                } else {
                    self.poly_runtime_mut().add_angle_b(5);
                    self.poly_runtime_mut().add_angle_a(3);
                }
            }
            4 => {
                self.poly_runtime_mut().set_angle_b(0);
                self.poly_runtime_mut().set_angle_a(0);
                self.attract_scene_mut().decrement_intro_step_timer();
                if self.game_state.ending.attract_scene.intro_step_timer() == 0 {
                    self.attract_scene_mut().increment_intro_step_index();
                    self.intro_actor_mut(5).set_init_phase(1);
                    self.intro_actor_mut(5).set_subtype(3);
                    self.set_main_screen_layers(0x10);
                    self.set_sub_screen_layers(5);
                    self.set_color_window_selection(2);
                    self.set_color_math_control(0x31);
                    self.set_subsubmodule(0);
                    self.increment_cgram_update_flag();
                    self.set_bg_vram_load_mode(3);
                    self.increment_submodule();
                }
            }
            _ => {}
        }
    }

    pub(super) fn scene_animate_every_sprite(&mut self) {
        self.reset_intro_sprite_oam_cursor();
        for k in (0..8).rev() {
            self.intro_anim_one_obj(k);
        }
    }

    pub(super) fn intro_anim_one_obj(&mut self, k: usize) {
        match self.intro_actor(k).init_phase() {
            1 => match self.intro_actor(k).subtype() {
                0 => self.intro_sprite_type_a_0(k),
                1 => self.exit_0_cca90(k),
                2 => self.initialize_scene_sprite_copyright(k),
                3 => self.initialize_scene_sprite_sparkle(k),
                4 | 5 | 6 => self.initialize_scene_sprite_triforce_room_triangle(k),
                7 => self.initialize_scene_sprite_credits_triangle(k),
                _ => {}
            },
            2 => match self.intro_actor(k).subtype() {
                0 => self.intro_sprite_type_b_0(k),
                1 => self.exit_0_cca90(k),
                2 => self.animate_scene_sprite_copyright(k),
                3 => self.animate_scene_sprite_sparkle(k),
                4 | 5 | 6 => self.intro_sprite_type_b_456(k),
                7 => self.animate_scene_sprite_credits_triangle(k),
                _ => {}
            },
            _ => {}
        }
    }

    pub(super) fn intro_sprite_type_a_0(&mut self, k: usize) {
        self.write_intro_x(k, INTRO_SPRITE_TYPE_A_0_X_OFFSETS[k]);
        self.write_intro_y(k, INTRO_SPRITE_TYPE_A_0_Y_OFFSETS[k]);
        self.intro_actor_mut(k)
            .set_x_velocity(INTRO_SPRITE_TYPE_A_0_X_VELOCITIES[k] as u8);
        self.intro_actor_mut(k)
            .set_y_velocity(INTRO_SPRITE_TYPE_A_0_Y_VELOCITIES[k] as u8);
        self.intro_actor_mut(k).increment_init_phase();
    }

    pub(super) fn initialize_scene_sprite_copyright(&mut self, k: usize) {
        self.write_intro_x(k, 76);
        self.write_intro_y(k, 184);
        self.intro_actor_mut(k).increment_init_phase();
    }

    pub(super) fn intro_sprite_type_b_0(&mut self, k: usize) {
        self.animate_scene_sprite_draw_triangle(k);
        self.animate_scene_sprite_move_triangle(k);
        if self.game_state.ending.attract_scene.intro_step_index() != 5 {
            if self.game_state.ending.attract_scene.intro_frame_counter() & 31 == 0 {
                self.intro_actor_mut(k)
                    .add_x_velocity(INTRO_SPRITE_TYPE_A_0_X_VELOCITIES[k] as u8);
                self.intro_actor_mut(k)
                    .add_y_velocity(INTRO_SPRITE_TYPE_A_0_Y_VELOCITIES[k] as u8);
            }
            if self.intro_actor(k).x_low() == INTRO_SPRITE_TYPE_B_0_X_LIMITS[k] {
                self.intro_actor_mut(k).set_x_velocity(0);
            }
            if self.intro_actor(k).y_low() == INTRO_SPRITE_TYPE_B_0_Y_LIMITS[k] {
                self.intro_actor_mut(k).set_y_velocity(0);
            }
        } else {
            self.intro_actor_mut(k).set_x_velocity(0);
            self.intro_actor_mut(k).set_y_velocity(0);
        }
    }

    pub(super) fn animate_scene_sprite_copyright(&mut self, k: usize) {
        self.animate_scene_sprite_add_objects_to_oam_buffer(
            k,
            &ANIMATE_SCENE_SPRITE_COPYRIGHT_SPRITE_ENTRIES,
        );
    }

    pub(super) fn initialize_scene_sprite_sparkle(&mut self, k: usize) {
        let j = (self.game_state.ending.attract_scene.intro_frame_counter() >> 5 & 3) as usize;
        self.intro_actor_mut(k)
            .set_x(i16::from(INITIALIZE_SCENE_SPRITE_SPARKLE_X_OFFSETS[j]));
        self.intro_actor_mut(k)
            .set_y(i16::from(INITIALIZE_SCENE_SPRITE_SPARKLE_Y_OFFSETS[j]));
        self.intro_actor_mut(k).increment_init_phase();
    }

    pub(super) fn animate_scene_sprite_sparkle(&mut self, k: usize) {
        let state = self.intro_actor(k).state();
        if state < 4 {
            self.animate_scene_sprite_add_objects_to_oam_buffer(
                k,
                &ANIMATE_SCENE_SPRITE_SPARKLE_SPRITE_ENTRIES[state as usize..state as usize + 1],
            );
        }

        let next_state = ANIMATE_SCENE_SPRITE_SPARKLE_STATES
            [(self.game_state.ending.attract_scene.intro_frame_counter() >> 2 & 7) as usize];
        self.intro_actor_mut(k).set_state(next_state);
        let j = (self.game_state.ending.attract_scene.intro_frame_counter() >> 5 & 3) as usize;
        self.intro_actor_mut(k)
            .set_x_low(INITIALIZE_SCENE_SPRITE_SPARKLE_X_OFFSETS[j]);
        self.intro_actor_mut(k)
            .set_y_low(INITIALIZE_SCENE_SPRITE_SPARKLE_Y_OFFSETS[j]);
    }

    pub(super) fn animate_scene_sprite_draw_triangle(&mut self, k: usize) {
        self.animate_scene_sprite_add_objects_to_oam_buffer(
            k,
            if k == 2 {
                &ANIMATE_SCENE_SPRITE_DRAW_TRIANGLE_RIGHT_ENTRIES
            } else {
                &ANIMATE_SCENE_SPRITE_DRAW_TRIANGLE_LEFT_ENTRIES
            },
        );
    }

    pub(super) fn animate_scene_sprite_move_triangle(&mut self, k: usize) {
        if self.intro_actor(k).x_velocity() != 0 {
            self.intro_actor_mut(k).move_x();
        }
        if self.intro_actor(k).y_velocity() != 0 {
            self.intro_actor_mut(k).move_y();
        }
    }

    pub(super) fn intro_display_logo(&mut self) {
        for i in 0..4 {
            self.set_oam_plain(
                i,
                INTRO_DISPLAY_LOGO_INTRO_LOGO_X[i],
                0x68,
                INTRO_DISPLAY_LOGO_INTRO_LOGO_TILE[i],
                0x32,
                2,
            );
        }
    }
}
