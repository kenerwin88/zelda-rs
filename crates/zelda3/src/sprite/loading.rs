//! Sprite slot initialization, overworld proximity loading, dungeon loading, and room caching.

use super::*;

impl ZeldaState {

    // void Sprite_InitializeSlots() {  // 89afd6
    //   for (int k = 15; k >= 0; k--) {
    //     uint8 st = sprite_state[k], ty = sprite_type[k];
    //     if (st != 0) {
    //       if (st == 10) {
    //         if (ty != 0xec && ty != 0xd2) {
    //           link_picking_throw_state = 0;
    //           link_state_bits = 0;
    //           sprite_state[k] = 0;
    //         }
    //       } else {
    //         if (ty != 0x6c && sprite_room[k] != BYTE(overworld_area_index))
    //           sprite_state[k] = 0;
    //       }
    //     }
    //   }
    //   for (int k = 7; k >= 0; k--) {
    //     if (overlord_type[k] && overlord_spawned_in_area[k] != BYTE(overworld_area_index))
    //       overlord_type[k] = 0;
    //   }
    // }
    pub(in crate::zelda_rtl) fn sprite_initialize_slots(&mut self) {
        let area = self.game_state.world.region.overworld_area_low();
        for k in (0..=15usize).rev() {
            let st = self.sprite_slot_view(k).state();
            let ty = self.sprite_slot_view(k).sprite_type();
            if st == 0 {
                continue;
            }
            if st == 10 {
                if ty != 0xec && ty != 0xd2 {
                    self.follower_link_state_mut().clear_picking_throw_state();
                    self.follower_link_state_mut().clear_state_bits();
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                }
            } else if ty != 0x6c && self.sprite_slot_view(k).room() != area {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        }
        for k in (0..=7usize).rev() {
            if self.overlord_slot_view(k).overlord_type() != 0
                && self.overlord_slot_view(k).spawned_area() != area
            {
                self.overlord_slot_view_mut(k).clear();
            }
        }
    }

    // void Sprite_InitializeMirrorPortal() {  // 89af89
    //   for (int k = 15; k >= 0; k--) {
    //     if (sprite_state[k] && sprite_type[k] == 0x6c)
    //       sprite_state[k] = 0;
    //   }
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(0xff, 0x6c, &info);
    //   if (j < 0)
    //     j = 0;
    //   Sprite_SetX(j, bird_travel_x_hi[15] << 8 | bird_travel_x_lo[15]);
    //   Sprite_SetY(j, (bird_travel_y_hi[15] << 8 | bird_travel_y_lo[15]) + 8);
    //   sprite_floor[j] = 0;
    //   sprite_ignore_projectile[j] = 1;
    // }
    pub(in crate::zelda_rtl) fn sprite_initialize_mirror_portal(&mut self) {
        self.sprite_remove_mirror_portals();
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0xff, 0x6c, &mut info).max(0) as usize;
        self.sprite_finish_mirror_portal(j);
    }

    fn sprite_remove_mirror_portals(&mut self) {
        for k in (0..=15usize).rev() {
            if self.sprite_slot_view(k).state() != 0
                && self.sprite_slot_view(k).sprite_type() == 0x6c
            {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        }
    }

    pub(in crate::zelda_rtl) fn sprite_begin_mirror_portal_reset(&mut self, slot: u8, completed_stores: u8) {
        self.sprite_remove_mirror_portals();
        let selected = (0..16)
            .rev()
            .find(|&k| self.sprite_slot_view(k).state() == 0)
            .expect("source portal spawn requires a free slot");
        assert_eq!(selected, usize::from(slot));
        self.sprite_spawn_dynamically_selected_prefix(
            0xff,
            0x6c,
            &mut SpriteSpawnInfo::default(),
            selected,
            crate::SpriteDynamicSpawnProgress::ResetProperties { completed_stores },
        );
    }

    pub(in crate::zelda_rtl) fn sprite_resume_mirror_portal_reset(&mut self, slot: u8, completed_stores: u8) {
        self.sprite_spawn_dynamically_selected_from(
            0xff,
            &mut SpriteSpawnInfo::default(),
            usize::from(slot),
            crate::SpriteDynamicSpawnProgress::ResetProperties { completed_stores },
        );
        self.sprite_finish_mirror_portal(usize::from(slot));
    }

    pub(in crate::zelda_rtl) fn sprite_begin_mirror_portal_load_properties(
        &mut self,
        slot: u8,
        completed_stores: u8,
    ) {
        self.sprite_remove_mirror_portals();
        let selected = (0..16)
            .rev()
            .find(|&k| self.sprite_slot_view(k).state() == 0)
            .expect("source portal spawn requires a free slot");
        assert_eq!(selected, usize::from(slot));
        self.sprite_spawn_dynamically_selected_prefix(
            0xff,
            0x6c,
            &mut SpriteSpawnInfo::default(),
            selected,
            crate::SpriteDynamicSpawnProgress::LoadProperties { completed_stores },
        );
    }

    pub(in crate::zelda_rtl) fn sprite_resume_mirror_portal_load_properties(
        &mut self,
        slot: u8,
        completed_stores: u8,
    ) {
        self.sprite_spawn_dynamically_selected_from(
            0xff,
            &mut SpriteSpawnInfo::default(),
            usize::from(slot),
            crate::SpriteDynamicSpawnProgress::LoadProperties { completed_stores },
        );
        self.sprite_finish_mirror_portal(usize::from(slot));
    }

    fn sprite_finish_mirror_portal(&mut self, ju: usize) {
        let bird = self
            .game_state
            .world
            .overworld
            .bird_travel_destinations
            .destination(15);
        let x = bird.x;
        let y = bird.y.wrapping_add(8);
        self.sprite_set_x(ju, x);
        self.sprite_set_y(ju, y);
        let value = 0;
        self.sprite_slot_view_mut(ju).set_floor(value);
        let value = 1;
        self.sprite_slot_view_mut(ju).set_ignore_projectile(value);
    }

    // void Sprite_ResetAll() {  // 89c44e
    //   Sprite_DisableAll();
    //   Sprite_ResetAll_noDisable();
    // }
    pub(in crate::zelda_rtl) fn sprite_reset_all(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x09_c44e);
        if self.lag_free { crate::cycle_ledger::charge(62); } // $09:C44E JSL DisableAll.
        self.sprite_disable_all();
        // $09:C452 falls through in C44E's call frame, rather than calling
        // the separately callable no-disable entry below.
        self.sprite_reset_all_no_disable_fixed_prefix();
        self.sprite_reset_all_no_disable_after_fixed_prefix();
    }

    // void Sprite_DisableAll() {  // 89c22f
    //   ...see sprite.c...
    // }
    /// Sprite_Main's caller stores publish through the same owners as its
    /// ordinary entry, without running any later caller or child work early.
    pub(crate) fn native_sprite_main_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        match address {
            0xae8 => {
                assert_eq!(width, 1);
                self.set_sprite_dma_head_pointer(value as u8);
                return true;
            }
            0xaea => {
                assert_eq!(width, 1);
                self.set_sprite_dma_body_pointer(value as u8);
                return true;
            }
            0xfb7 => {
                assert_eq!(width,1);
                self.game_state.sprites.system.set_ranged_based_toggler(value as u8);
                self.ram[SPR_RANGED_BASED_TOGGLER]=value as u8;return true;
            }
            0xfb5 => {assert_eq!(width,1);self.temp_counter_mut().set(value as u8);return true;}
            0x223 => {
                assert_eq!((width, value), (1, 0));
                self.clear_tile_interaction_shared_flag();
                return true;
            }
            0x1cd8 => {
                assert_eq!(width, 1);
                self.messaging_state_mut().set_module(value as u8);
                return true;
            }
            0x1cf0 | 0x1cf1 => {
                assert!(width == 1 || (address == 0x1cf0 && width == 2));
                let old = self.game_state.messaging.dialogue_message_index.value();
                let shift = (address & 1) * 8;
                self.dialogue_message_index_mut().set_value(if width == 2 { value }
                    else { (old & !(255 << shift)) | ((value & 255) << shift) });
                return true;
            }
            0x10c => {
                assert_eq!(width, 1);
                self.set_saved_module_for_menu(value as u8);
                return true;
            }
            _ => {}
        }
        if matches!(address, 0x20..=0x23 | 0x40..=0x41 | 0x46 | 0x4d | 0x5e | 0x68..=0x69 | 0xb7c..=0xb7f | 0xfc2..=0xfc5) {
            assert!(width == 1 || (width == 2 && matches!(address,0x20|0x22|0xb7c|0xb7e|0xfc2|0xfc4)));
            for byte in 0..usize::from(width) {
                let address = address as usize + byte;
                let value = (value >> (byte * 8)) as u8;
                assert!(self.game_state.player.follower_link.write_sprite_player_byte(address, value));
                self.write_wram_byte(address, value);
            }
            return true;
        }
        if (0xcfe..0xd00).contains(&address) {
            assert!(address + u32::from(width) <= 0xd00);
            for byte in 0..usize::from(width) {
                let address = address as usize + byte;
                let value = (value >> (byte * 8)) as u8;
                self.game_state.sprites.workspace.write_draw_priority_byte(address == 0xcff, value);
                self.write_wram_byte(address, value);
            }
            return true;
        }
        if (0xfa8..0xfac).contains(&address) {
            assert!(address + u32::from(width) <= 0xfac);
            for byte in 0..usize::from(width) {
                let address = address as usize + byte;
                let value = (value >> (byte * 8)) as u8;
                self.game_state.sprites.draw_hitbox_work.write_instruction_byte(address, value);
                self.write_wram_byte(address, value);
            }
            return true;
        }
        if self.game_state.sprites.sprite_slots.write_instruction_store(&mut self.ram, address as usize, value, width) {
            return true;
        }
        if (0xfd8..0xfdc).contains(&address) {
            assert!(address + u32::from(width) <= 0xfdc);
            for byte in 0..usize::from(width) {
                let address = address as usize + byte;
                let shift = (address & 1) * 8;
                let old = read_le_u16(&self.ram, address & !1);
                let word = (old & !(255 << shift)) | (((value >> (byte * 8)) & 255) << shift);
                if address < 0xfda { self.sprite_workspace_mut().set_current_sprite_x(word); }
                else { self.sprite_workspace_mut().set_current_sprite_y(word); }
            }
            return true;
        }
        if (0xfe0..0xff8).contains(&address) {
            assert_eq!((width, address & 1), (2, 0));
            if address < 0xfec {
                self.oam_state_mut().set_region_base_word((address as usize - 0xfe0) / 2, value);
            } else {
                self.oam_state_mut().set_region_alloc_counter((address as usize - 0xfec) / 2, value);
            }
            return true;
        }
        if !matches!(address, 0x47 | 0x314 | 0x377 | 0x379 | 0x69e | 0x69f | 0xb7b | 0xb9a | 0xfa0 | 0xfab | 0xfb2 | 0xfc6 | 0xfdc | 0xffa | 0xfff) {
            return false;
        }
        assert_eq!(width, 1);
        let value = value as u8;
        match address {
            0xffa => self.sprite_system_mut().set_alt_sprites_flag(value),
            0xfff => self.set_dark_world_region_index(value),
            0xfb2 => self.sprite_workspace_mut().set_pickup_slot_cache(value),
            0xfab => self.hitbox_scratch_offset_mut().set_x_high_offset(value),
            0xfa0 => self.sprite_system_mut().set_cur_object_index(value),
            0xfc6 => self.sprite_system_mut().set_chr_halfslot_state(value),
            0xfdc => self.sprite_system_mut().set_alert_flag(value),
            0x47 => self.sprite_battle_mut().set_damaging_enemies_timer(value),
            0x314 => self.follower_link_state_mut().set_sprite_pickup_flag(value),
            0x377 => self.follower_link_state_mut().set_pull_action_state(value),
            0x379 => { assert_eq!(value, 0); self.follower_link_state_mut().clear_player_pose_draw_counter(); }
            0xb7b => { assert_eq!(value, 0); self.follower_link_state_mut().clear_prevent_movement(); }
            0xb9a => { assert_eq!(value, 0); self.archery_game_mut().clear_out_of_arrows(); }
            0x69e => { assert_eq!(value, 0); self.clear_overworld_vertical_scroll_delta_low(); }
            0x69f => self.set_overworld_horizontal_scroll_delta_low(value),
            _ => unreachable!(),
        }
        true
    }

    /// Record loading keeps its live pointer, coordinate temporaries and slot
    /// stores through nested sprite and overlord children. Publish the retired
    /// bytes directly to their owners without refreshing unrelated slot state.
    pub(crate) fn native_sprite_record_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        if address == 0 {
            assert_eq!(width, 2);
            for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
                assert!(self.game_state.sprites.write_record_byte(index, byte));
            }
        } else {
            if (0x100..=0x1ff).contains(&address) { return false; }
            assert_eq!(width, 1);
            let byte = value as u8;
            if !self.game_state.sprites.write_record_byte(address, byte) {
                match address {
                    // The record-stream cursor at $04 has no separate scalar
                    // owner; its native continuation reads the live RAM byte.
                    4 => {}
                    0xfb3 => self.game_state.oam.set_sprite_sorting_setting(byte),
                    0xfb5 => self.game_state.scratch_counter.set(byte),
                    0xb00..=0xb47 => {
                        let mut slot = self.overlord_slot_view_mut((address - 0xb00) & 7);
                        match (address - 0xb00) / 8 {
                            0 => slot.set_overlord_type(byte),
                            1 => slot.set_x_low(byte),
                            2 => slot.set_x_high(byte),
                            3 => slot.set_y_low(byte),
                            4 => slot.set_y_high(byte),
                            5 => slot.set_gen1(byte),
                            6 => slot.set_gen2(byte),
                            7 => slot.set_gen3(byte),
                            8 => slot.set_floor(byte),
                            _ => unreachable!(),
                        }
                    }
                    0xcca..=0xcd1 => self.overlord_slot_view_mut(address - 0xcca).set_spawned_area(byte),
                    _ => return false,
                }
            }
        }
        for byte in 0..usize::from(width) { self.write_wram_byte(address + byte, value.to_le_bytes()[byte]); }
        true
    }

    /// The reset caller publishes collision bounds and history before loading
    /// new records. Its evicted-room store shares the presence-bank views.
    pub(crate) fn native_sprite_reset_parent_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        match address {
            0 => {
                assert_eq!(width, 2);
                for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
                    self.game_state.sprites.workspace.write_zero_page_byte(index, byte);
                }
            }
            0x308 | 0x309 => {
                assert_eq!(width, 1);
                self.game_state.player.follower_link.write_sprite_reset_parent_byte(address, value as u8);
            }
            0xfb8 | 0xfba | 0xb80..=0xb86 | 0x1df80..=0x1ef7e => {
                assert_eq!(width, 2);
                assert_eq!(address & 1, 0);
                match address {
                    0xfb8 => self.game_state.sprites.garnish_runtime.set_sprcoll_x_size(value),
                    0xfba => self.game_state.sprites.garnish_runtime.set_sprcoll_y_size(value),
                    _ => for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
                        if address < 0x1df80 {
                            self.game_state.dungeon.room_tracking.write_history_byte(address + index - 0xb80, byte);
                        } else {
                            assert!(self.game_state.sprites.write_reset_suffix_byte(address + index, byte));
                        }
                    },
                }
            }
            _ => return false,
        }
        for byte in 0..usize::from(width) { self.write_wram_byte(address + byte, value.to_le_bytes()[byte]); }
        true
    }

    /// Outgoing cache stores retain both cached slot aliases and system flags
    /// while the continuation walks the live sprite fields in source order.
    pub(crate) fn native_sprite_cache_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        assert_eq!(width, 1);
        if !self.game_state.sprites.write_cache_byte(address, value as u8) { return false; }
        self.write_wram_byte(address, value as u8);
        true
    }

    /// Reset suffix stores follow the completed disable child. Keep the long
    /// descending clears and byte-wise history reset visible to native owners.
    pub(crate) fn native_sprite_reset_suffix_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        if !matches!(address, 0x2f0 | 0x4b4 | 0xb6a | 0xb80..=0xb87 | 0xfb3 | 0xfc6 |
            0xfdc | 0xfdd | 0xffd | 0x1df80..=0x1f17f) { return false; }
        assert_eq!(width, 1);
        let byte = value as u8;
        if !self.game_state.sprites.write_reset_suffix_byte(address, byte) {
            match address {
                0x2f0 => self.game_state.messaging.runtime.message_or_sprite_state_cache = byte,
                0x4b4 => self.game_state.display.hud_runtime.set_super_bomb_indicator_timer(byte),
                0xb80..=0xb87 => self.game_state.dungeon.room_tracking.write_history_byte(address - 0xb80, byte),
                0xfb3 => self.game_state.oam.set_sprite_sorting_setting(byte),
                _ => unreachable!("unowned sprite-reset suffix byte {address:05x}"),
            }
        }
        self.write_wram_byte(address, byte);
        true
    }

    /// Retire one Sprite_DisableAll store through its native actor owner. The
    /// continuation retains the descending slot cursors and conditional guards.
    pub(crate) fn native_sprite_disable_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        if !matches!(address, 0x2ec | 0x3fc | 0xb00..=0xb07 | 0xb6a | 0xb88 | 0xb99 |
            0xb9b | 0xb9e | 0xc4a..=0xc53 | 0xcf4 | 0xdd0..=0xddf | 0xfb4 | 0xfc6 |
            0xff8 | 0xff9 | 0xffb..=0xffd | 0x1f800..=0x1f81d) { return false; }
        assert_eq!((width, value), (1, 0));
        if !self.game_state.sprites.write_disable_byte(address, value as u8)
            && !self.game_state.player.follower_link.write_sprite_disable_byte(address, value as u8) {
            match address {
                0xb00..=0xb07 => self.overlord_slot_view_mut(address - 0xb00).clear(),
                0x1f800..=0x1f81d => self.garnish_slot_view_mut(address - 0x1f800).set_garnish_type(0),
                0x3fc => self.game_state.minigame.clear_is_archer_or_shovel_game(),
                0xb88 => self.game_state.archery_game.clear_hit_counter(),
                0xb99 => self.game_state.archery_game.set_arrows_left(0),
                0xb9b => self.game_state.sprite_battle.clear_item_drop_counter(),
                0xb9e => self.game_state.dungeon.room_effects.clear_trap_trigger_latch(),
                0xcf4 => self.game_state.dungeon.room_effects.set_activate_bomb_trap_overlord(0),
                0xff9 => self.game_state.ending.attract_scene.intro_palette_flash_count = 0,
                _ => unreachable!("unowned sprite-disable byte {address:05x}"),
            }
        }
        self.write_wram_byte(address, value as u8);
        true
    }

    pub(in crate::zelda_rtl) fn sprite_disable_all(&mut self) {
        self.apply_sprite_disable_actions_through(
            None,
            DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
        );
        self.sprite_disable_all_after_limit_instance();
    }

    /// Publish only Sprite_DisableAll's early live-slot state clears while a
    /// later overworld reload generation remains deferred. The surrounding
    /// reset side effects already ran in the provisional loader body.
    pub(in crate::zelda_rtl) fn sprite_disable_live_slots_before_deferred_overworld_reload(&mut self) {
        for k in (0..=15usize).rev() {
            if self.sprite_slot_view(k).state() != 0
                && (self.game_state.world.location.is_indoors()
                    || self.sprite_slot_view(k).sprite_type() != 0x6c)
            {
                self.sprite_slot_view_mut(k).set_state(0);
            }
        }
    }

    pub(in crate::zelda_rtl) fn apply_sprite_disable_actions_through(
        &mut self,
        completed: Option<DungeonSpriteDisableCpuProgress>,
        target: DungeonSpriteDisableCpuProgress,
    ) {
        let start = completed
            .map(DungeonSpriteDisableCpuProgress::source_ordinal)
            .map_or(0, |ordinal| ordinal + 1);
        let target = target.source_ordinal();
        assert!(
            start <= target,
            "Sprite_DisableAll semantic progress moved backwards",
        );
        for ordinal in start..=target {
            match ordinal {
                0..=15 => {
                    let k = usize::from(15 - ordinal);
                    if self.sprite_slot_view(k).state() != 0
                        && (self.game_state.world.location.is_indoors()
                            || self.sprite_slot_view(k).sprite_type() != 0x6c)
                    {
                        self.sprite_slot_view_mut(k).set_state(0);
                    }
                }
                16..=25 => {
                    let k = usize::from(9 - (ordinal - 16));
                    self.ancilla_slot_view_mut(k).clear();
                }
                26 => self.follower_link_state_mut().clear_ancilla_pickup_flag(),
                27 => self.sprite_system_mut().set_limit_instance(0),
                _ => unreachable!("Sprite_DisableAll action ordinal is bounded"),
            }
        }
    }

    pub(in crate::zelda_rtl) fn complete_sprite_disable_after_states(&mut self, slot: u8) {
        self.apply_sprite_disable_actions_through(
            Some(DungeonSpriteDisableCpuProgress::SpriteStatesThrough { slot }),
            DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
        );
        self.sprite_disable_all_after_limit_instance();
    }

    fn sprite_disable_all_after_limit_instance(&mut self) {
        self.sprite_disable_all_before_garnish_clear();
        self.complete_sprite_disable_garnish_clear(30);
    }

    pub(in crate::zelda_rtl) fn sprite_disable_all_before_garnish_clear(&mut self) {
        self.sprite_battle_mut().clear_item_drop_counter();
        self.archery_game_mut().clear_hit_counter();
        self.archery_game_mut().set_arrows_left(0);
        self.garnish_state_mut().clear_active_type();
        self.dungeon_room_effects_mut().clear_trap_trigger_latch();
        self.dungeon_room_effects_mut()
            .set_activate_bomb_trap_overlord(0);
        self.attract_scene_mut().clear_intro_palette_flash_count();
        self.sprite_workspace_mut().set_reset_scratch_a(0);
        self.sprite_workspace_mut().set_reset_scratch_b(0);
        self.follower_link_state_mut().clear_menu_block();
        self.garnish_state_mut().clear_boulder_trap_count();
        self.sprite_system_mut().set_chr_halfslot_state(0);
        self.minigame_state_mut().clear_is_archer_or_shovel_game();
        for k in (0..8).rev() {
            self.overlord_slot_view_mut(k).clear();
        }
    }

    pub(in crate::zelda_rtl) fn complete_sprite_disable_garnish_clear(&mut self, remaining: u8) {
        assert!(remaining <= 30);
        for k in (0..usize::from(remaining)).rev() {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
        }
    }

    // void Sprite_ResetAll_noDisable() {  // 89c452
    //   haunted_grove_flute_event_latch = 0;
    //   sprite_alert_flag = 0;
    //   overworld_boulder_trap_count = 0;
    //   MESSAGE_OR_SPRITE_STATE_CACHE = 0;
    //   sprite_chr_halfslot_state = 0;
    //   sprite_limit_instance = 0;
    //   sort_sprites_setting = 0;
    //   if (follower_indicator != 13)
    //     super_bomb_indicator_unk2 = 0xfe;
    //   memset(sprite_where_in_room, 0, 0x1000);
    //   memset(overworld_sprite_was_loaded, 0, 0x200);
    //   memset(dungeon_room_history, 0xff, 8);
    // }
    pub(in crate::zelda_rtl) fn sprite_reset_all_no_disable(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::profile_routine(0x09_c452, true));
        self.sprite_reset_all_no_disable_fixed_prefix();
        self.sprite_reset_all_no_disable_after_fixed_prefix();
    }

    /// Complete the fixed stores before Sprite_ResetAll_noDisable's first
    /// large descending clear. This is the exact source boundary at $09:C47B.
    pub(in crate::zelda_rtl) fn sprite_reset_all_no_disable_fixed_prefix(&mut self) {
        if self.lag_free {
            // $09:C452..C479: seven clears, follower test/branch, M8/X8
            // then REP #$10, LDX #$0FFF, LDA #0 before the bulk clear.
            crate::cycle_ledger::charge(296 + if self.game_state.sprites.follower_runtime.indicator() == 13 { 6 } else { 48 } + 62);
        }
        self.garnish_state_mut()
            .clear_haunted_grove_flute_event_latch();
        self.sprite_system_mut().set_alert_flag(0);
        self.garnish_state_mut().clear_boulder_trap_count();
        self.messaging_state_mut()
            .clear_message_or_sprite_state_cache();
        self.sprite_system_mut().set_chr_halfslot_state(0);
        self.sprite_system_mut().set_limit_instance(0);
        self.oam_state_mut().clear_sprite_sorting_setting();
        if self.game_state.sprites.follower_runtime.indicator() != 13 {
            self.set_super_bomb_indicator_timer(0xfe);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_reset_all_no_disable_after_fixed_prefix(&mut self) {
        if self.lag_free {
            // $09:C47B..C480 M8/X16: 4096 STA-long,X/DEX/BPL passes.
            crate::cycle_ledger::charge(4096 * 70 + 4095 * 6);
        }
        self.sprite_workspace_mut().clear_where_in_room();
        self.sprite_reset_all_after_room_mask_clear();
    }

    pub(super) fn sprite_reset_all_after_room_mask_clear(&mut self) {
        if self.lag_free {
            // $09:C482..C498: 512 loaded-mask clears, SEP #$10,
            // eight room-history stores and RTL; actual taken loop branches.
            crate::cycle_ledger::charge(24 + 512 * 70 + 511 * 6 + 54
                + 8 * 68 + 7 * 6 + 44);
        }
        self.clear_all_overworld_sprite_loaded_masks();
        self.dungeon_room_tracking_mut().reset_room_history();
    }

    pub(in crate::zelda_rtl) fn sprite_reload_all_overworld(&mut self) -> OverworldSpriteReloadWorkload {
        self.sprite_disable_all();
        self.sprite_overworld_reload_all_just_load()
    }

    pub(in crate::zelda_rtl) fn sprite_begin_reload_all_overworld(&mut self) {
        self.sprite_disable_all();
        self.sprite_reset_all_no_disable();
    }

    pub(in crate::zelda_rtl) fn sprite_overworld_reload_all_just_load(
        &mut self,
    ) -> OverworldSpriteReloadWorkload {
        if self.lag_free {
            crate::cycle_ledger::charge(298); // $09:C49D..C4AB, calls/DBR/RTL only.
        }
        let work = self.begin_overworld_sprite_reload_just_load();
        self.complete_overworld_sprite_reload_scan(work)
    }

    pub(in crate::zelda_rtl) fn begin_overworld_sprite_reload_just_load(
        &mut self,
    ) -> OverworldSpriteReloadScanWork {
        let sprite_records = self.begin_overworld_sprite_reload_presence_just_load();
        self.begin_overworld_sprite_reload_scan_from_presence(sprite_records)
    }

    pub(in crate::zelda_rtl) fn begin_overworld_sprite_reload_presence_just_load(&mut self) -> usize {
        self.sprite_reset_all_no_disable();
        self.overworld_load_sprites()
    }

    pub(in crate::zelda_rtl) fn sprite_finish_reload_all_overworld(&mut self) -> OverworldSpriteReloadWorkload {
        let work = self.begin_overworld_sprite_reload_scan();
        self.complete_overworld_sprite_reload_scan(work)
    }

    pub(in crate::zelda_rtl) fn complete_overworld_sprite_reload_scan(
        &mut self,
        mut work: OverworldSpriteReloadScanWork,
    ) -> OverworldSpriteReloadWorkload {
        let total_cells = work.total_cells();
        self.advance_overworld_sprite_reload_scan_through_cell(&mut work, total_cells);
        self.finish_overworld_sprite_reload_scan(work)
    }

    pub(in crate::zelda_rtl) fn begin_overworld_sprite_reload_scan(&mut self) -> OverworldSpriteReloadScanWork {
        let sprite_records = self.overworld_load_sprites();
        self.begin_overworld_sprite_reload_scan_from_presence(sprite_records)
    }

    pub(in crate::zelda_rtl) fn begin_overworld_sprite_reload_scan_from_presence(
        &mut self,
        sprite_records: usize,
    ) -> OverworldSpriteReloadScanWork {
        let scan = self.begin_overworld_proximity_cell_scan();
        OverworldSpriteReloadScanWork {
            sprite_records,
            scan,
        }
    }

    pub(in crate::zelda_rtl) fn advance_overworld_sprite_reload_scan_through_cell(
        &mut self,
        work: &mut OverworldSpriteReloadScanWork,
        ordinal: u16,
    ) {
        self.advance_overworld_proximity_scan_through_cell(&mut work.scan, ordinal);
    }

    pub(in crate::zelda_rtl) fn finish_overworld_sprite_reload_scan(
        &mut self,
        work: OverworldSpriteReloadScanWork,
    ) -> OverworldSpriteReloadWorkload {
        let in_bounds_proximity_checks = self.finish_overworld_proximity_cell_scan(work.scan);
        OverworldSpriteReloadWorkload {
            sprite_records: work.sprite_records,
            in_bounds_proximity_checks,
        }
    }

    pub(in crate::zelda_rtl) fn overworld_load_sprites(&mut self) -> usize {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x09_c4ac));
        if self.lag_free {
            // $09:C4AC..C50D: collision bounds and progress-dependent pointer;
            // C635+area never crosses a page (area <= $9F). C55B..C55D return.
            let pointer = match self.game_state.inventory.save_progress.progress_indicator() {
                3 => 6 + 122,
                2 => 32 + 6 + 100,
                _ => 32 + 122,
            };
            crate::cycle_ledger::charge(672 + pointer + 48 + 78 + 6 + 64);
        }
        let area = self.game_state.world.region.overworld_area();
        let area_lo = self.game_state.world.region.overworld_area_low() as usize;
        self.garnish_state_mut().set_sprcoll_x_base((area & 7) << 9);
        self.garnish_state_mut()
            .set_sprcoll_y_base((((area & 0x3f) >> 2) & 0x0e) << 8);
        let size = u16::from(OVERWORLD_AREA_SPRCOLL_SIZES[area_lo]) << 8;
        self.garnish_state_mut().set_sprcoll_x_size(size);
        self.garnish_state_mut().set_sprcoll_y_size(size);

        let base = match self.game_state.inventory.save_progress.progress_indicator() {
            3 => 2,
            2 => 1,
            _ => 0,
        };
        let Some(offsets) = self.asset_raw(159).map(Vec::from) else {
            return 0;
        };
        let Some(sprites) = self.asset_raw(160).map(Vec::from) else {
            return 0;
        };
        let offs_idx = (area as usize + base * 144) * 2;
        if offs_idx + 1 >= offsets.len() {
            return 0;
        }
        let mut src = read_word_from_slice(&offsets, offs_idx) as usize;
        let mut sprite_records = 0usize;
        while src < sprites.len() && sprites[src] != 0xff {
            if src + 2 >= sprites.len() {
                break;
            }
            sprite_records += 1;
            if self.lag_free {
                // $09:C510..C51E; ordinary records take BNE and execute the
                // coordinate/presence stores $C528..C559; traps take C520..26.
                crate::cycle_ledger::charge(78 + 134
                    + if sprites[src + 2] == 0xf4 { 110 } else { 6 + 754 });
            }
            if sprites[src + 2] == 0xf4 {
                self.garnish_state_mut().increment_boulder_trap_count();
                src += 3;
                continue;
            }

            let r2 = (sprites[src] >> 4) << 2;
            let r6 = (sprites[src + 1] >> 4).wrapping_add(r2);
            let r5 = (sprites[src + 1] & 0x0f) | (sprites[src] << 4);
            let idx = usize::from(r5) | (usize::from(r6) << 8);
            let value = sprites[src + 2].wrapping_add(1);
            self.set_overworld_sprite_presence_marker(idx, value);
            src += 3;
        }
        sprite_records
    }

    pub(in crate::zelda_rtl) fn sprite_activate_all_proxima(&mut self) -> usize {
        let mut scan = self.begin_overworld_proximity_cell_scan();
        let total_cells = scan.total_cells();
        self.advance_overworld_proximity_scan_through_cell(&mut scan, total_cells);
        self.finish_overworld_proximity_cell_scan(scan)
    }

    pub(in crate::zelda_rtl) fn begin_overworld_proximity_cell_scan(&mut self) -> OverworldProximityScanWork {
        if self.lag_free {
            crate::cycle_ledger::charge(210); // $09:C55E..C56D: save scroll, setup.
        }
        let saved_bg2_h = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let saved_scroll_delta = self.overworld_horizontal_scroll_delta_low();
        self.set_overworld_horizontal_scroll_delta_low(0xff);
        let extension: u16 = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_EXTEND_SCREEN64_SPRITE)
        {
            0x40
        } else {
            0
        };
        self.set_bg2_x(saved_bg2_h.wrapping_sub(extension));
        let y_start = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_sub(0x30);
        OverworldProximityScanWork {
            saved_bg2_h,
            saved_scroll_delta,
            x_extension: extension,
            // Sprite_ActivateWhenProximal sees the temporary $069F=$FF and
            // computes its first X from the current BG2 and screen extension.
            x: self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_sub(0x10)
                .wrapping_sub(extension),
            y: y_start,
            y_start,
            rows_in_column: 0,
            columns_remaining: (22 + (extension >> 3)) as u8,
            cells_completed: 0,
            total_cells: (22 + (extension >> 3)) * 22,
            in_bounds_proximity_checks: 0,
        }
    }

    pub(in crate::zelda_rtl) fn advance_overworld_proximity_cell_scan(
        &mut self,
        scan: &mut OverworldProximityScanWork,
    ) -> bool {
        if scan.columns_remaining == 0 {
            return false;
        }
        if self.lag_free {
            // $09:C5E3..C5F3 one row's call/control, with 21 taken BPLs.
            crate::cycle_ledger::charge(222 + if scan.rows_in_column == 21 { 0 } else { 6 });
            if scan.rows_in_column == 0 {
                // $09:C5BB..C5E1: nonzero negative scroll; each column enters
                // Sprite_ActivateWhenProximal anew. C56F..C582 outer control.
                crate::cycle_ledger::charge(64 + 16 + 14 + 332 + 42 + 268
                    + if scan.columns_remaining == 1 { 0 } else { 6 });
            }
        }
        scan.in_bounds_proximity_checks +=
            u16::from(self.sprite_overworld_proximity_motivated_load(scan.x, scan.y));
        scan.cells_completed += 1;
        scan.rows_in_column += 1;
        if scan.rows_in_column == 22 {
            scan.rows_in_column = 0;
            scan.columns_remaining -= 1;
            let bg2_h = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(16);
            self.set_bg2_x(bg2_h);
            // The original caller recomputes these locals on every column.
            // A sprite activation may mutate translated state in between.
            scan.x = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_sub(0x10)
                .wrapping_sub(scan.x_extension);
            scan.y_start = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_sub(0x30);
            scan.y = scan.y_start;
        } else {
            scan.y = scan.y.wrapping_add(16);
        }
        true
    }

    /// Bring the translated scan to a source `CellReturned` ordinal. A later
    /// caller can consume the exact CPU event without guessing an NMI budget
    /// or replaying cells already completed in an earlier host.
    pub(in crate::zelda_rtl) fn advance_overworld_proximity_scan_through_cell(
        &mut self,
        scan: &mut OverworldProximityScanWork,
        ordinal: u16,
    ) {
        assert!(ordinal >= scan.cells_completed && ordinal <= scan.total_cells);
        while scan.cells_completed < ordinal {
            assert!(self.advance_overworld_proximity_cell_scan(scan));
        }
    }

    pub(in crate::zelda_rtl) fn finish_overworld_proximity_cell_scan(
        &mut self,
        scan: OverworldProximityScanWork,
    ) -> usize {
        assert_eq!(
            scan.columns_remaining, 0,
            "overworld proximity scan returned before its last cell"
        );
        assert_eq!(scan.cells_completed, scan.total_cells);
        if self.lag_free {
            crate::cycle_ledger::charge(206); // $09:C584..C58E restore/RTS.
        }
        self.set_overworld_horizontal_scroll_delta_low(scan.saved_scroll_delta);
        self.set_bg2_x(scan.saved_bg2_h);
        usize::from(scan.in_bounds_proximity_checks)
    }

    pub(in crate::zelda_rtl) fn sprite_proximity_activation(&mut self) {
        // $06:833B JSL $09:C58F: wrapper self work; both scan bodies are children.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x09c58f));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x09c58f);
        if self.game_state.frame.submodule != 0 {
            b.block(0x09c596);
            self.sprite_activate_when_proximal();
            b.block(0x09c599);
            self.sprite_activate_when_proximal_big();
            b.block(0x09c59c);
        } else {
            b.taken(0x09c594); b.block(0x09c59e);
            if self.game_state.sprites.system.ranged_based_toggler() & 1 == 0 {
                b.block(0x09c5a5);
                self.sprite_activate_when_proximal();
            } else { b.taken(0x09c5a3); }
            b.block(0x09c5a8);
            if self.game_state.sprites.system.ranged_based_toggler() & 1 != 0 {
                b.block(0x09c5af);
                self.sprite_activate_when_proximal_big();
            } else { b.taken(0x09c5ad); }
            b.block(0x09c5b2);
            self.sprite_system_mut().increment_ranged_based_toggler();
        }
    }

    pub(in crate::zelda_rtl) fn sprite_activate_when_proximal(&mut self) -> usize {
        let _scope = crate::cycle_ledger::routine(0x09_c5bb);
        if self.lag_free {
            // $09:C5BB..C5C0: LDY, scroll load and BEQ (M8/X8).
            crate::cycle_ledger::charge(64);
        }
        if self.overworld_horizontal_scroll_delta_low() == 0 {
            if self.lag_free { crate::cycle_ledger::charge(6 + 42); } // taken BEQ, $C5F5 RTS
            return 0;
        }
        if self.lag_free {
            // $09:C5C2..C5E1: BPL/optional INY and coordinate/loop setup.
            // Y=0/1 cannot cross either $C5B7/$C5B9 table's page.
            crate::cycle_ledger::charge(16 + if sign8(self.overworld_horizontal_scroll_delta_low()) { 14 } else { 6 } + 332);
        }
        let xt: u16 = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_EXTEND_SCREEN64_SPRITE)
        {
            0x40
        } else {
            0
        };
        let x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_add(if sign8(self.overworld_horizontal_scroll_delta_low()) {
                0u16.wrapping_sub(0x10).wrapping_sub(xt)
            } else {
                0x110u16.wrapping_add(xt)
            });
        let mut y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_sub(0x30);
        let mut in_bounds_proximity_checks = 0usize;
        for row in (0..=21).rev() {
            if self.lag_free {
                // $09:C5E3..C5F3: JSR only, Y increment, DEX/BPL.
                // $C6F5 and its children retain their existing prices.
                crate::cycle_ledger::charge(222 + if row == 0 { 0 } else { 6 });
            }
            in_bounds_proximity_checks +=
                usize::from(self.sprite_overworld_proximity_motivated_load(x, y));
            y = y.wrapping_add(16);
        }
        if self.lag_free { crate::cycle_ledger::charge(42); } // $09:C5F5 RTS
        in_bounds_proximity_checks
    }

    pub(in crate::zelda_rtl) fn sprite_activate_when_proximal_big(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x09_c5fa));
        if self.lag_free { crate::cycle_ledger::charge(64); } // $09:C5FA..C5FF LDY/LDA/BEQ.
        if self.overworld_vertical_scroll_delta_low() == 0 {
            if self.lag_free { crate::cycle_ledger::charge(6 + 42); } // Taken BEQ; $C634 RTS.
            return;
        }
        if self.lag_free {
            // $09:C601..C620 BPL/optional INY, coordinates and LDX #21.
            crate::cycle_ledger::charge(16 + if sign8(self.overworld_vertical_scroll_delta_low()) { 14 } else { 6 } + 332);
        }
        let xt: u16 = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_EXTEND_SCREEN64_SPRITE)
        {
            0x40
        } else {
            0
        };
        let mut x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_sub(0x30)
            .wrapping_sub(xt);
        let y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(if sign8(self.overworld_vertical_scroll_delta_low()) {
                0u16.wrapping_sub(0x10)
            } else {
                0x110
            });
        for column in (0..=(21 + (xt >> 3))).rev() {
            if self.lag_free {
                // $09:C622..C632 JSR only, X increment, DEX/BPL.
                crate::cycle_ledger::charge(222 + if column == 0 { 0 } else { 6 });
            }
            self.sprite_overworld_proximity_motivated_load(x, y);
            x = x.wrapping_add(16);
        }
        if self.lag_free { crate::cycle_ledger::charge(42); } // $09:C634 RTS.
    }

    pub(in crate::zelda_rtl) fn sprite_overworld_proximity_motivated_load(&mut self, x: u16, y: u16) -> bool {
        let _scope = crate::cycle_ledger::routine(0x09_c6f5);
        let sprcoll_x_base = self.game_state.sprites.garnish_runtime.sprcoll_x_word();
        let sprcoll_y_base = self.game_state.sprites.garnish_runtime.sprcoll_y_word();
        let xt = x.wrapping_sub(sprcoll_x_base);
        let yt = y.wrapping_sub(sprcoll_y_base);
        if self.lag_free {
            // $09:C6F5..C730: X bounds; conditional Y bounds; block-index/call
            // only in bounds. Proxima child's body remains independently owned.
            let suffix = if xt >= self.game_state.sprites.garnish_runtime.sprcoll_x_size() { 6 }
                else { 194 + if yt >= self.game_state.sprites.garnish_runtime.sprcoll_y_size() { 6 } else { 406 } };
            crate::cycle_ledger::charge(164 + suffix + 64);
        }
        if xt >= self.game_state.sprites.garnish_runtime.sprcoll_x_size()
            || yt >= self.game_state.sprites.garnish_runtime.sprcoll_y_size()
        {
            return false;
        }

        let r1 = (((yt >> 8) * 4) | (xt >> 8)) as u8;
        let r0 = ((y & 0x00f0) | ((x >> 4) & 0x000f)) as u8;
        self.overworld_load_proxima_sprite_if_alive((u16::from(r1) << 8) | u16::from(r0));
        true
    }

    pub(in crate::zelda_rtl) fn overworld_load_proxima_sprite_if_alive(&mut self, blk: u16) {
        let _scope = crate::cycle_ledger::routine(0x09_c739);
        // $09:C739..C74B, M8/X8; the shared C76F RTS belongs here.
        if self.lag_free { crate::cycle_ledger::charge(250 + 42); }
        let sprite_to_spawn = self
            .game_state
            .sprites
            .overworld_sprite_presence
            .marker(blk as usize);
        if sprite_to_spawn == 0 {
            if self.lag_free { crate::cycle_ledger::charge(6); }
            return;
        }

        let loadedmask = 0x80u8 >> (blk & 7);
        // $09:C74D..C76A, no C731,Y page crossing for Y=0..7.
        if self.lag_free { crate::cycle_ledger::charge(378); }
        if self
            .game_state
            .sprites
            .overworld_sprite_loaded
            .is_loaded(blk, loadedmask)
        {
            if self.lag_free { crate::cycle_ledger::charge(6); }
            return;
        }

        if self.lag_free { crate::cycle_ledger::charge(46); } // $09:C76C JSR only.
        let _child = crate::cycle_ledger::routine(0x09_c770);
        if sprite_to_spawn >= 0xf4 {
            let k = self.alloc_overlord();
            if k < 0 {
                return;
            }
            let k = k as usize;
            self.set_overworld_sprite_loaded_mask(blk, loadedmask);
            self.overlord_slot_view_mut(k).set_sprite_block_pos(blk);
            self.overlord_slot_view_mut(k)
                .set_overlord_type(sprite_to_spawn.wrapping_sub(0xf3));
            let x_low = ((blk << 4) & 0x00f0) as u8
                + if self.overlord_slot_view(k).overlord_type() == 1 {
                    8
                } else {
                    0
                };
            let x_high = (((blk >> 8) & 3) as u8).wrapping_add(
                (self.game_state.sprites.garnish_runtime.sprcoll_x_word() >> 8) as u8,
            );
            self.overlord_slot_view_mut(k)
                .set_x(u16::from(x_low) | (u16::from(x_high) << 8));
            let y_low = (blk & 0x00f0) as u8;
            let y_high = ((blk >> 10) as u8).wrapping_add(
                (self.game_state.sprites.garnish_runtime.sprcoll_y_word() >> 8) as u8,
            );
            self.overlord_slot_view_mut(k)
                .set_y(u16::from(y_low) | (u16::from(y_high) << 8));
            self.overlord_slot_view_mut(k).set_floor(0);
            let area = self.game_state.world.region.overworld_area_low();
            self.overlord_slot_view_mut(k).set_spawned_area(area);
            self.overlord_slot_view_mut(k).set_gen2(0);
            self.overlord_slot_view_mut(k).set_gen1(0);
            self.overlord_slot_view_mut(k).set_gen3(0);
        } else {
            let k = self.overworld_alloc_sprite(sprite_to_spawn);
            if k < 0 {
                return;
            }
            let k = k as usize;
            self.set_overworld_sprite_loaded_mask(blk, loadedmask);
            self.sprite_slot_view_mut(k).set_n_word(blk);
            let value = sprite_to_spawn.wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_sprite_type(value);
            let value = 8;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = ((blk << 4) & 0x00f0) as u8;
            self.sprite_slot_view_mut(k).set_x_low(value);
            let value = (blk & 0x00f0) as u8;
            self.sprite_slot_view_mut(k).set_y_low(value);
            let value = (((blk >> 8) & 3) as u8).wrapping_add(
                (self.game_state.sprites.garnish_runtime.sprcoll_x_word() >> 8) as u8,
            );
            self.sprite_slot_view_mut(k).set_x_high(value);
            let value = ((blk >> 10) as u8).wrapping_add(
                (self.game_state.sprites.garnish_runtime.sprcoll_y_word() >> 8) as u8,
            );
            self.sprite_slot_view_mut(k).set_y_high(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_floor(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_subtype(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_die_action(value);
            if self.pending_overworld_sprite_reload_slots.is_none() {
                if let Some(activations) = self.pending_overworld_sprite_activations.as_mut() {
                    activations.push_back((k as u8, self.game_state.sprites.sprite_slots.clone()));
                }
            }
        }
    }

    /// Bind the packed overworld sprite lists to their physical data addresses.
    /// The reload worker consumes these records without cartridge instructions.
    pub(crate) fn native_overworld_sprite_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        use crate::cycle_models::overworld_sprite_reload_data::{EARLY, RESCUE, LATE, EMPTY_LIST};
        let offsets = self.asset_raw(159)?;
        let records = self.asset_raw(160)?;
        let mut data = std::collections::BTreeMap::new();
        let mut put = |address, value| {
            if let Some(old) = data.insert(address, value) {
                assert_eq!(old, value, "overlapping overworld sprite data {address:06x}");
            }
        };
        for (stage, (base, pointers)) in [(0x09c881, &EARLY), (0x09c901, &RESCUE), (0x09ca21, &LATE)]
            .into_iter().enumerate()
        {
            for (screen, pointer) in pointers.iter().enumerate() {
                for (byte, value) in pointer.to_le_bytes().into_iter().enumerate() {
                    put(base + (screen * 2 + byte) as u32, value);
                }
                // The initial-world physical table has only 64 entries. Its
                // remaining reads alias the following rescue-world table;
                // packed initial-world slots 64..143 are intentionally empty.
                let packed_index = if stage == 0 && screen >= 64 {
                    144 + screen - 64
                } else {
                    stage * 144 + screen
                };
                let start = usize::from(read_word_from_slice(offsets, packed_index * 2));
                let mut end = start;
                while *records.get(end)? != 0xff { end += 3; }
                // Secondary cells of large areas have no packed list, while
                // their physical pointer aliases the primary area's list.
                // Bind that shared list from its nonempty packed owner.
                if end == start && *pointer != EMPTY_LIST { continue; }
                for (byte, &value) in records.get(start..=end)?.iter().enumerate() {
                    put(0x090000 + u32::from(*pointer) + byte as u32, value);
                }
            }
        }
        for (i, &value) in OVERWORLD_AREA_SPRCOLL_SIZES.iter().enumerate() {
            put(0x09c635 + i as u32, value);
        }
        for (i, value) in [0x10, 0xf0, 0x01, 0xff].into_iter().enumerate() {
            put(0x09c5b7 + i as u32, value);
        }
        for bit in 0..8 { put(0x09c731 + bit, 0x80 >> bit); }
        for pointers in [&EARLY, &RESCUE, &LATE] {
            for pointer in pointers {
                assert!(data.contains_key(&(0x090000 + u32::from(*pointer))),
                    "unbound overworld sprite list {pointer:04x}");
            }
        }
        Some(data)
    }

    /// Bind existing native records and properties at their physical data
    /// addresses. The reset owner reads these bytes without ROM execution.
    pub(crate) fn native_dungeon_sprite_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        let records = self.asset_raw(58)?;
        let offsets = self.asset_raw(59)?;
        let mut data = std::collections::BTreeMap::new();
        let mut put = |address, value| {
            if let Some(old) = data.insert(address, value) {
                assert_eq!(old, value, "overlapping native sprite records at {address:06x}");
            }
        };
        for (room, &pointer) in crate::cycle_models::sprite_reset_data::ROOM_POINTERS.iter().enumerate() {
            for (i, value) in pointer.to_le_bytes().into_iter().enumerate() {
                put(0x09d62e + (room * 2 + i) as u32, value);
            }
            let start = usize::from(read_word_from_slice(offsets, room * 2));
            let mut end = start + 1;
            while *records.get(end)? != 0xff { end += 3; }
            for (i, &value) in records.get(start..=end)?.iter().enumerate() {
                put(0x090000 + u32::from(pointer) + i as u32, value);
            }
        }
        for slot in 0..16 {
            for (i, value) in (1u16 << slot).to_le_bytes().into_iter().enumerate() {
                put(0x09c2d5 + (slot * 2 + i) as u32, value);
            }
        }
        for ty in 0..SPRITE_INIT_TABLE_LEN {
            put(0x0db725 + ty as u32, sprite_init_value(SPRITE_INIT_DEFL_BITS_TABLE, ty as u8));
        }
        Some(data)
    }

    pub(in crate::zelda_rtl) fn dungeon_reset_sprites(&mut self) {
        self.dungeon_reset_sprites_before_room_load();
        self.dungeon_load_sprites();
    }

    pub(in crate::zelda_rtl) fn dungeon_reset_sprites_through_cpu_progress(
        &mut self,
        progress: DungeonResetSpritesCpuProgress,
    ) {
        match progress {
            DungeonResetSpritesCpuProgress::Cache { slot, field } => {
                self.dungeon_cache_trans_sprites_through_cpu_progress(slot, field);
            }
            DungeonResetSpritesCpuProgress::Disable(progress) => {
                if self.game_state.world.location.is_indoors() {
                    self.dungeon_cache_trans_sprites();
                }
                self.follower_link_state_mut().clear_picking_throw_state();
                self.follower_link_state_mut().clear_state_bits();
                self.apply_sprite_disable_actions_through(None, progress);
            }
            DungeonResetSpritesCpuProgress::SpritesDisabled => {
                self.dungeon_reset_sprites_through_sprite_disable_all();
            }
            DungeonResetSpritesCpuProgress::GarnishTypesThrough { slot } => {
                assert!(slot < 30);
                if self.game_state.world.location.is_indoors() {
                    self.dungeon_cache_trans_sprites();
                }
                self.follower_link_state_mut().clear_picking_throw_state();
                self.follower_link_state_mut().clear_state_bits();
                self.apply_sprite_disable_actions_through(
                    None, DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
                );
                self.sprite_disable_all_before_garnish_clear();
                for k in (usize::from(slot)..30).rev() {
                    self.garnish_slot_view_mut(k).set_garnish_type(0);
                }
            }
            DungeonResetSpritesCpuProgress::CollisionXSizeSet => {
                self.dungeon_reset_sprites_through_sprite_disable_all();
                self.garnish_state_mut().set_sprcoll_x_size(0xffff);
            }
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted => {
                self.dungeon_reset_sprites_through_sprite_disable_all();
                self.dungeon_reset_sprites_set_collision_sizes();
            }
            DungeonResetSpritesCpuProgress::Load(progress) => {
                self.dungeon_reset_sprites_before_room_load();
                self.dungeon_load_sprites_through_cpu_progress(progress);
            }
            DungeonResetSpritesCpuProgress::LoadStarted => {
                self.dungeon_reset_sprites_before_room_load();
                self.dungeon_load_sprites_prefix();
            }
            DungeonResetSpritesCpuProgress::LoadBeforeOrigin => {
                self.dungeon_reset_sprites_before_room_load();
            }
        }
    }

    pub(in crate::zelda_rtl) fn dungeon_resume_reset_sprites_after_cpu_progress(
        &mut self,
        progress: DungeonResetSpritesCpuProgress,
    ) {
        match progress {
            DungeonResetSpritesCpuProgress::Cache { slot, field } => {
                self.dungeon_resume_cache_trans_sprites_after_cpu_progress(slot, field);
                self.dungeon_reset_sprites_after_cache_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::Disable(progress) => {
                if progress != DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared {
                    self.apply_sprite_disable_actions_through(
                        Some(progress),
                        DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
                    );
                }
                self.sprite_disable_all_after_limit_instance();
                self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::SpritesDisabled => {
                self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::GarnishTypesThrough { slot } => {
                assert!(slot < 30);
                self.complete_sprite_disable_garnish_clear(slot);
                self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::CollisionXSizeSet => {
                self.garnish_state_mut().set_sprcoll_y_size(0xffff);
                self.dungeon_reset_sprites_after_collision_sizes_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted => {
                self.dungeon_reset_sprites_after_collision_sizes_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::Load(progress) => {
                self.dungeon_resume_load_sprites_after_cpu_progress(progress);
            }
            DungeonResetSpritesCpuProgress::LoadStarted => {
                if let Some((sprites, start)) = self.dungeon_sprite_records() {
                    self.dungeon_load_sprite_records(&sprites, start);
                }
            }
            DungeonResetSpritesCpuProgress::LoadBeforeOrigin => self.dungeon_load_sprites(),
        }
    }

    /// Advance an already-applied source prefix to a later receipt from the
    /// next timing-authority host call. This is deliberately narrow: the only
    /// cross-host refinements are within `Sprite_DisableAll`,
    /// `Dungeon_CacheTransSprites`, and `Dungeon_LoadSprites`.
    pub(in crate::zelda_rtl) fn dungeon_advance_reset_sprites_cpu_progress(
        &mut self,
        completed: DungeonResetSpritesCpuProgress,
        target: DungeonResetSpritesCpuProgress,
    ) -> bool {
        match (completed, target) {
            (
                DungeonResetSpritesCpuProgress::LoadBeforeOrigin,
                DungeonResetSpritesCpuProgress::LoadBeforeOrigin,
            )
            | (
                DungeonResetSpritesCpuProgress::LoadStarted,
                DungeonResetSpritesCpuProgress::LoadStarted,
            ) => true,
            (
                DungeonResetSpritesCpuProgress::Disable(completed),
                DungeonResetSpritesCpuProgress::Disable(target),
            ) if completed.source_ordinal() <= target.source_ordinal() => {
                if completed != target {
                    self.apply_sprite_disable_actions_through(Some(completed), target);
                }
                true
            }
            (
                DungeonResetSpritesCpuProgress::Cache {
                    slot: completed_slot,
                    field: completed_field,
                },
                DungeonResetSpritesCpuProgress::Cache {
                    slot: target_slot,
                    field: target_field,
                },
            ) if target_slot < completed_slot
                || (target_slot == completed_slot && completed_field <= target_field) =>
            {
                // The C caching loop walks slots 15..0 with the fields of
                // each slot in source order; apply only the delta between
                // the two checkpoints (route host 133571 refined slot 15's
                // `I` to `IgnoreProjectile` at its accepting NMI).
                assert!(target_field <= self.dungeon_cache_last_field(usize::from(target_slot)));
                if target_slot == completed_slot {
                    if completed_field != target_field {
                        self.dungeon_cache_trans_sprite_fields(
                            usize::from(target_slot),
                            Some(completed_field),
                            target_field,
                        );
                    }
                } else {
                    let completed_slot = usize::from(completed_slot);
                    self.dungeon_cache_trans_sprite_fields(
                        completed_slot,
                        Some(completed_field),
                        self.dungeon_cache_last_field(completed_slot),
                    );
                    for k in (usize::from(target_slot) + 1..completed_slot).rev() {
                        self.dungeon_cache_trans_sprite_fields(
                            k,
                            None,
                            self.dungeon_cache_last_field(k),
                        );
                    }
                    self.dungeon_cache_trans_sprite_fields(
                        usize::from(target_slot),
                        None,
                        target_field,
                    );
                }
                true
            }
            (
                DungeonResetSpritesCpuProgress::SpritesDisabled,
                DungeonResetSpritesCpuProgress::CollisionXSizeSet,
            ) => {
                self.garnish_state_mut().set_sprcoll_x_size(0xffff);
                true
            }
            (
                DungeonResetSpritesCpuProgress::SpritesDisabled,
                DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
            ) => {
                self.dungeon_reset_sprites_set_collision_sizes();
                true
            }
            (
                DungeonResetSpritesCpuProgress::CollisionXSizeSet,
                DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
            ) => {
                self.garnish_state_mut().set_sprcoll_y_size(0xffff);
                true
            }
            (
                DungeonResetSpritesCpuProgress::Load(completed),
                DungeonResetSpritesCpuProgress::Load(target),
            ) => self.dungeon_advance_load_sprites_cpu_progress(completed, target),
            _ => false,
        }
    }

    fn dungeon_advance_load_sprites_cpu_progress(
        &mut self,
        completed: DungeonLoadSpritesCpuProgress,
        target: DungeonLoadSpritesCpuProgress,
    ) -> bool {
        if target.normal_load_ordinal < completed.normal_load_ordinal
            || (target.normal_load_ordinal == completed.normal_load_ordinal
                && target.checkpoint < completed.checkpoint)
        {
            return false;
        }
        if target == completed {
            return true;
        }
        let Some((sprites, start)) = self.dungeon_sprite_records() else {
            return false;
        };
        let mut k = 0usize;
        let mut src = start + 1;
        let mut normal_load_ordinal = 0u16;
        let mut completed_reached = false;
        while src + 2 < sprites.len() && sprites[src] != 0xff {
            let y = sprites[src];
            let x = sprites[src + 1];
            let sprite_type = sprites[src + 2];
            let loads_normal_slot =
                self.dungeon_sprite_record_loads_normal_slot(k, y, x, sprite_type);
            if !completed_reached {
                if loads_normal_slot && normal_load_ordinal == completed.normal_load_ordinal {
                    if k as u8 != completed.slot {
                        return false;
                    }
                    if target.normal_load_ordinal == completed.normal_load_ordinal {
                        if target.slot != completed.slot {
                            return false;
                        }
                        self.dungeon_load_single_sprite_fields(
                            k,
                            y,
                            x,
                            sprite_type,
                            Some(completed.checkpoint),
                            target.checkpoint,
                        );
                        return true;
                    }
                    self.dungeon_load_single_sprite_fields(
                        k,
                        y,
                        x,
                        sprite_type,
                        Some(completed.checkpoint),
                        DungeonSpriteLoadCheckpoint::Complete,
                    );
                    completed_reached = true;
                    k += 1;
                    normal_load_ordinal = normal_load_ordinal.saturating_add(1);
                } else {
                    if Self::dungeon_sprite_record_advances_slot(y, x, sprite_type) {
                        k += 1;
                    }
                    if loads_normal_slot {
                        normal_load_ordinal = normal_load_ordinal.saturating_add(1);
                    }
                }
            } else if loads_normal_slot && normal_load_ordinal == target.normal_load_ordinal {
                if k as u8 != target.slot {
                    return false;
                }
                self.dungeon_load_single_sprite_fields(
                    k,
                    y,
                    x,
                    sprite_type,
                    None,
                    target.checkpoint,
                );
                return true;
            } else {
                k = (self.dungeon_load_single_sprite(k, y, x, sprite_type) + 1) as usize;
                if loads_normal_slot {
                    normal_load_ordinal = normal_load_ordinal.saturating_add(1);
                }
            }
            src += 3;
        }
        false
    }

    pub(in crate::zelda_rtl) fn dungeon_reset_sprites_before_room_load(&mut self) {
        self.dungeon_reset_sprites_through_sprite_disable_all();
        self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
    }

    fn dungeon_reset_sprites_through_sprite_disable_all(&mut self) {
        if self.game_state.world.location.is_indoors() {
            self.dungeon_cache_trans_sprites();
        }
        self.dungeon_reset_sprites_after_cache_through_sprite_disable_all();
    }

    fn dungeon_reset_sprites_after_cache_through_sprite_disable_all(&mut self) {
        {
            self.follower_link_state_mut().clear_picking_throw_state();
            self.follower_link_state_mut().clear_state_bits();
        }
        self.sprite_disable_all();
    }

    fn dungeon_reset_sprites_after_sprite_disable_before_room_load(&mut self) {
        self.dungeon_reset_sprites_set_collision_sizes();
        self.dungeon_reset_sprites_after_collision_sizes_before_room_load();
    }

    fn dungeon_reset_sprites_set_collision_sizes(&mut self) {
        self.garnish_state_mut().set_sprcoll_x_size(0xffff);
        self.garnish_state_mut().set_sprcoll_y_size(0xffff);
    }

    fn dungeon_reset_sprites_after_collision_sizes_before_room_load(&mut self) {
        let room = self.game_state.dungeon.room_tracking.room_index2_word();
        let seen =
            (0..4).any(|i| self.game_state.dungeon.room_tracking.room_history_entry(i) == room);
        if !seen {
            let dropped = self.game_state.dungeon.room_tracking.room_history_entry(3);
            for i in (1..4).rev() {
                let prev = self
                    .game_state
                    .dungeon
                    .room_tracking
                    .room_history_entry(i - 1);
                self.dungeon_room_tracking_mut()
                    .set_room_history_entry(i, prev);
            }
            self.dungeon_room_tracking_mut()
                .set_room_history_entry(0, room);
            if dropped != 0xffff {
                self.set_sprite_where_in_room_mask(dropped, 0);
            }
        }
    }

    fn dungeon_reset_sprites_after_cache_before_room_load(&mut self) {
        self.dungeon_reset_sprites_after_cache_through_sprite_disable_all();
        self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
    }

    pub(in crate::zelda_rtl) fn dungeon_load_sprites(&mut self) {
        if let Some((sprites, start)) = self.dungeon_load_sprites_prefix() {
            self.dungeon_load_sprite_records(&sprites, start);
        }
    }

    fn dungeon_load_sprites_prefix(&mut self) -> Option<(Vec<u8>, usize)> {
        let (sprites, start) = self.dungeon_sprite_records()?;
        let room = self.game_state.dungeon.room_tracking.room_index2_word() as usize;

        self.sprite_workspace_mut()
            .set_room_origin_y_high(((room >> 3) & 0xfe) as u8);
        self.sprite_workspace_mut()
            .set_room_origin_x_high(((room & 0x0f) << 1) as u8);
        self.oam_state_mut()
            .set_sprite_sorting_setting(sprites[start]);
        Some((sprites, start))
    }

    fn dungeon_load_sprite_records(&mut self, sprites: &[u8], start: usize) {
        let mut k = 0isize;
        let mut src = start + 1;
        while src < sprites.len() && sprites[src] != 0xff {
            if src + 2 >= sprites.len() {
                break;
            }
            k = self.dungeon_load_single_sprite(
                k as usize,
                sprites[src],
                sprites[src + 1],
                sprites[src + 2],
            );
            k += 1;
            src += 3;
        }
    }

    fn dungeon_sprite_records(&self) -> Option<(Vec<u8>, usize)> {
        let sprites = self.asset_raw(58).map(Vec::from)?;
        let offsets = self.asset_raw(59).map(Vec::from)?;
        let room = self.game_state.dungeon.room_tracking.room_index2_word() as usize;
        let start = read_word_from_slice(&offsets, room * 2) as usize;
        (start < sprites.len()).then_some((sprites, start))
    }

    /// Resolve the semantic load ordinal from native room records and the
    /// native death mask. Markers, overlords and suppressed records do not
    /// publish a normal sprite; a CPU slot is not a normal-load ordinal.
    pub(in crate::zelda_rtl) fn dungeon_load_progress_for_slot(
        &self,
        slot: u8,
        checkpoint: DungeonSpriteLoadCheckpoint,
    ) -> DungeonLoadSpritesCpuProgress {
        assert!(slot < 16);
        let (sprites, start) = self.dungeon_sprite_records()
            .expect("interrupted room sprite load requires native room records");
        let mut k = 0usize;
        let mut normal_load_ordinal = 0u16;
        let mut src = start + 1;
        while src + 2 < sprites.len() && sprites[src] != 0xff {
            let (y, x, sprite_type) = (sprites[src], sprites[src + 1], sprites[src + 2]);
            if self.dungeon_sprite_record_loads_normal_slot(k, y, x, sprite_type) {
                if k == usize::from(slot) {
                    return DungeonLoadSpritesCpuProgress { normal_load_ordinal, slot, checkpoint };
                }
                normal_load_ordinal += 1;
            }
            if Self::dungeon_sprite_record_advances_slot(y, x, sprite_type) {
                k += 1;
            }
            src += 3;
        }
        panic!("interrupted sprite slot {slot} is not a native normal room load");
    }

    fn dungeon_load_sprites_through_cpu_progress(
        &mut self,
        progress: DungeonLoadSpritesCpuProgress,
    ) {
        let Some((sprites, start)) = self.dungeon_sprite_records() else {
            return;
        };
        let room = self.game_state.dungeon.room_tracking.room_index2_word() as usize;
        self.sprite_workspace_mut()
            .set_room_origin_y_high(((room >> 3) & 0xfe) as u8);
        self.sprite_workspace_mut()
            .set_room_origin_x_high(((room & 0x0f) << 1) as u8);
        self.oam_state_mut()
            .set_sprite_sorting_setting(sprites[start]);

        let mut k = 0isize;
        let mut src = start + 1;
        let mut normal_load_ordinal = 0u16;
        while src + 2 < sprites.len() && sprites[src] != 0xff {
            let y = sprites[src];
            let x = sprites[src + 1];
            let sprite_type = sprites[src + 2];
            let loads_normal_slot =
                self.dungeon_sprite_record_loads_normal_slot(k as usize, y, x, sprite_type);
            if loads_normal_slot && normal_load_ordinal == progress.normal_load_ordinal {
                assert_eq!(
                    k as u8, progress.slot,
                    "translated room sprite slot disagrees with the ROM checkpoint",
                );
                self.dungeon_load_single_sprite_fields(
                    k as usize,
                    y,
                    x,
                    sprite_type,
                    None,
                    progress.checkpoint,
                );
                return;
            }
            k = self.dungeon_load_single_sprite(k as usize, y, x, sprite_type);
            k += 1;
            if loads_normal_slot {
                normal_load_ordinal = normal_load_ordinal.saturating_add(1);
            }
            src += 3;
        }
        panic!(
            "ROM room-load checkpoint normal-load ordinal {} is outside the room sprite list",
            progress.normal_load_ordinal,
        );
    }

    fn dungeon_resume_load_sprites_after_cpu_progress(
        &mut self,
        progress: DungeonLoadSpritesCpuProgress,
    ) {
        let Some((sprites, start)) = self.dungeon_sprite_records() else {
            return;
        };
        let mut k = 0usize;
        let mut src = start + 1;
        let mut normal_load_ordinal = 0u16;
        let mut target_reached = false;
        while src + 2 < sprites.len() && sprites[src] != 0xff {
            let y = sprites[src];
            let x = sprites[src + 1];
            let sprite_type = sprites[src + 2];
            let loads_normal_slot =
                self.dungeon_sprite_record_loads_normal_slot(k, y, x, sprite_type);
            if target_reached {
                k = (self.dungeon_load_single_sprite(k, y, x, sprite_type) + 1) as usize;
            } else if loads_normal_slot && normal_load_ordinal == progress.normal_load_ordinal {
                assert_eq!(
                    k as u8, progress.slot,
                    "translated room sprite slot disagrees with the ROM checkpoint",
                );
                self.dungeon_load_single_sprite_fields(
                    k,
                    y,
                    x,
                    sprite_type,
                    Some(progress.checkpoint),
                    DungeonSpriteLoadCheckpoint::Complete,
                );
                k += 1;
                normal_load_ordinal = normal_load_ordinal.saturating_add(1);
                target_reached = true;
            } else {
                if Self::dungeon_sprite_record_advances_slot(y, x, sprite_type) {
                    k += 1;
                }
                if loads_normal_slot {
                    normal_load_ordinal = normal_load_ordinal.saturating_add(1);
                }
            }
            src += 3;
        }
        assert!(
            target_reached,
            "ROM room-load checkpoint normal-load ordinal {} is outside the room sprite list",
            progress.normal_load_ordinal,
        );
    }

    fn dungeon_sprite_record_advances_slot(y: u8, x: u8, sprite_type: u8) -> bool {
        if sprite_type == 0xe4 && (y == 0xfe || y == 0xfd) {
            return false;
        }
        if sprite_type != 0xe4 && x >= 0xe0 {
            return false;
        }
        true
    }

    fn dungeon_sprite_record_loads_normal_slot(
        &self,
        k: usize,
        y: u8,
        x: u8,
        sprite_type: u8,
    ) -> bool {
        Self::dungeon_sprite_record_advances_slot(y, x, sprite_type)
            && (sprite_init_value(SPRITE_INIT_DEFL_BITS_TABLE, sprite_type) & 1 != 0
                || self.sprite_where_in_room_mask(
                    self.game_state.dungeon.room_tracking.room_index2_word(),
                ) & (1 << k)
                    == 0)
    }

    pub(in crate::zelda_rtl) fn dungeon_load_single_sprite(
        &mut self,
        k: usize,
        y: u8,
        x: u8,
        sprite_type: u8,
    ) -> isize {
        if sprite_type == 0xe4 {
            if y == 0xfe || y == 0xfd {
                if k != 0 {
                    let value = if y == 0xfe { 1 } else { 2 };
                    self.sprite_slot_view_mut(k - 1).set_die_action(value);
                }
                return k as isize - 1;
            }
        } else if x >= 0xe0 {
            self.dungeon_load_single_overlord(&[y, x, sprite_type]);
            return k as isize - 1;
        }

        if sprite_init_value(SPRITE_INIT_DEFL_BITS_TABLE, sprite_type) & 1 == 0
            && self
                .sprite_where_in_room_mask(self.game_state.dungeon.room_tracking.room_index2_word())
                & (1 << k)
                != 0
        {
            return k as isize;
        }

        self.dungeon_load_single_sprite_fields(
            k,
            y,
            x,
            sprite_type,
            None,
            DungeonSpriteLoadCheckpoint::Complete,
        );
        k as isize
    }

    fn dungeon_load_single_sprite_fields(
        &mut self,
        k: usize,
        y: u8,
        x: u8,
        sprite_type: u8,
        completed: Option<DungeonSpriteLoadCheckpoint>,
        through: DungeonSpriteLoadCheckpoint,
    ) {
        let should_apply = |checkpoint| {
            completed.is_none_or(|completed| checkpoint > completed) && checkpoint <= through
        };
        if should_apply(DungeonSpriteLoadCheckpoint::State) {
            self.sprite_slot_view_mut(k).set_state(8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::TempY) {
            self.temp_counter_mut().set(y);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::Floor) {
            self.sprite_slot_view_mut(k).set_floor(y >> 7);
        }
        let y_coord = (((y as u16) << 4) & 0x01ff)
            + ((self.game_state.sprites.workspace.room_origin_y_high() as u16) << 8);
        if should_apply(DungeonSpriteLoadCheckpoint::YLow) {
            self.sprite_slot_view_mut(k).set_y_low(y_coord as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::YHigh) {
            self.sprite_slot_view_mut(k)
                .set_y_high((y_coord >> 8) as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::SharedX) {
            self.sprite_workspace_mut().set_shared_scratch_a(x);
        }
        let x_coord = (((x as u16) << 4) & 0x01ff)
            + ((self.game_state.sprites.workspace.room_origin_x_high() as u16) << 8);
        if should_apply(DungeonSpriteLoadCheckpoint::XLow) {
            self.sprite_slot_view_mut(k).set_x_low(x_coord as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::XHigh) {
            self.sprite_slot_view_mut(k)
                .set_x_high((x_coord >> 8) as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::Type) {
            self.sprite_slot_view_mut(k).set_sprite_type(sprite_type);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::SubtypeClear) {
            self.sprite_slot_view_mut(k).set_subtype(0);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::TempSubtype) {
            let counter = (self.game_state.scratch_counter.value() & 0x60) >> 2;
            self.temp_counter_mut().set(counter);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::SubtypeFinal) {
            let subtype = self.game_state.scratch_counter.value() | (x >> 5);
            self.sprite_slot_view_mut(k).set_subtype(subtype);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::SpawnIndex) {
            self.sprite_slot_view_mut(k).set_n(k as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::Complete) {
            self.sprite_slot_view_mut(k).set_die_action(0);
        }
    }

    // void Dungeon_LoadSingleOverlord(const uint8 *src) {  // 89c3e8
    //   int k = AllocOverlord();
    //   if (k < 0)
    //     return;
    //   uint8 y = src[0], x = src[1], type = src[2];
    //   overlord_type[k] = type;
    //   overlord_floor[k] = (y >> 7);
    //   int t = ((y << 4) & 0x1ff) + (SPRITE_ROOM_ORIGIN_Y_HI << 8);
    //   overlord_y_lo[k] = t;
    //   overlord_y_hi[k] = t >> 8;
    //   t = ((x << 4) & 0x1ff) + (SPRITE_ROOM_ORIGIN_X_HI << 8);
    //   overlord_x_lo[k] = t;
    //   overlord_x_hi[k] = t >> 8;
    //   overlord_spawned_in_area[k] = overworld_area_index;
    //   overlord_gen2[k] = 0;
    //   overlord_gen1[k] = 0;
    //   overlord_gen3[k] = 0;
    //   if (overlord_type[k] == 10 || overlord_type[k] == 11) {
    //     overlord_gen2[k] = 160;
    //   } else if (overlord_type[k] == 3) {
    //     overlord_gen2[k] = 255;
    //     overlord_x_lo[k] -= 8;
    //   }
    // }
    pub(in crate::zelda_rtl) fn dungeon_load_single_overlord(&mut self, src: &[u8]) {
        let k = self.alloc_overlord();
        if k < 0 || src.len() < 3 {
            return;
        }
        let k = k as usize;
        let y = src[0];
        let x = src[1];
        let type_ = src[2];
        self.overlord_slot_view_mut(k).set_overlord_type(type_);
        self.overlord_slot_view_mut(k).set_floor(y >> 7);
        let mut t = (((y as u16) << 4) & 0x01ff)
            + ((self.game_state.sprites.workspace.room_origin_y_high() as u16) << 8);
        self.overlord_slot_view_mut(k).set_y(t);
        t = (((x as u16) << 4) & 0x01ff)
            + ((self.game_state.sprites.workspace.room_origin_x_high() as u16) << 8);
        self.overlord_slot_view_mut(k).set_x(t);
        let area = self.game_state.world.region.overworld_area_low();
        self.overlord_slot_view_mut(k).set_spawned_area(area);
        self.overlord_slot_view_mut(k).set_gen2(0);
        self.overlord_slot_view_mut(k).set_gen1(0);
        self.overlord_slot_view_mut(k).set_gen3(0);
        if self.overlord_slot_view(k).overlord_type() == 10
            || self.overlord_slot_view(k).overlord_type() == 11
        {
            self.overlord_slot_view_mut(k).set_gen2(160);
        } else if self.overlord_slot_view(k).overlord_type() == 3 {
            self.overlord_slot_view_mut(k).set_gen2(255);
            self.overlord_slot_view_mut(k).subtract_x_low(8);
        }
    }

    // void ExecuteCachedSprites() {  // 9de9da
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn execute_cached_sprites(&mut self) {
        self.execute_cached_sprites_from_native_slot(None,None,None);
    }

    pub(super) fn execute_cached_sprites_after_native_entry(
        &mut self,
        slot: u8,
        backup: &[u8; 24],
        state: u8,
        timer: crate::cycle_models::sprite_timers::SpriteTimersTail,
    ) {
        self.execute_cached_sprites_from_native_slot(
            Some(slot),
            Some(backup),
            Some((state, timer)),
        );
    }

    fn execute_cached_sprites_from_native_slot(
        &mut self,
        native_slot: Option<u8>,
        native_backup: Option<&[u8; 24]>,
        native_timer: Option<(u8, crate::cycle_models::sprite_timers::SpriteTimersTail)>,
    ) {
        // Cycle ledger: ExecuteCachedSprites $1D:E9DA (m8 x8). The ROM tests
        // the four exit conditions in order; each is a block plus a taken BEQ
        // into $1D:E9FC STZ $0ffa : RTL (76).
        let _scope = crate::cycle_ledger::routine(0x1d_e9da);
        if native_slot.is_none() {
            crate::cycle_ledger::charge(if self.game_state.world.location.is_outdoors() {
                40 + 6 + 76 // $1D:E9DA-E9DC LDA $1b : BEQ taken
            } else if self.game_state.frame.submodule == 0 {
                40 + 40 + 6 + 76 // + $1D:E9DE-E9E0 LDA $11 : BEQ taken
            } else if self.game_state.frame.submodule == 14 {
                40 + 40 + 32 + 6 + 76 // + $1D:E9E2-E9E4 CMP #$0e : BEQ taken
            } else if self.game_state.sprites.system.alt_sprites_flag() == 0 {
                40 + 40 + 32 + 48 + 6 + 76 // + $1D:E9E6-E9E9 LDA $0ffa : BEQ taken
            } else {
                40 + 40 + 32 + 48 + 16 // all four fall through + $1D:E9EB LDX #$0f
            });
            if self.game_state.world.location.is_outdoors()
                || self.game_state.frame.submodule == 0
                || self.game_state.frame.submodule == 14
                || self.game_state.sprites.system.alt_sprites_flag() == 0
            {
                self.sprite_system_mut().clear_alt_sprites_flag();
                return;
            }
        }
        let interruption = self.take_dungeon_cached_sprite_cpu_interruption();
        if crate::zelda_rtl::debug_cached_sprite_cpu_for_host(self.frame_ctr_dbg) {
            eprintln!(
                "cached_sprite_native host={} state={:02x}/{:02x}/{:02x} interruption={interruption:?}",
                self.frame_ctr_dbg,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
            );
        }
        for i in (0..native_slot.map_or(16, |slot| usize::from(slot) + 1)).rev() {
            self.sprite_system_mut().set_cur_object_index(i as u8);
            let loaded_backup = native_backup.filter(|_| native_slot == Some(i as u8));
            let loaded_timer = native_timer.filter(|_| native_slot == Some(i as u8));
            if let Some((state, tail)) = loaded_timer {
                assert!(self.native_sprite_timer_handoff.is_none());
                self.native_sprite_timer_handoff = Some((i as u8, state, tail));
            }
            // $1D:E9ED-E9F3 STX $0fa0, LDA $1d00,x, BEQ (80): an active cached
            // slot falls into $1D:E9F5 JSR UncacheAndExecuteSprite (46), an
            // inactive one takes the BEQ (+6). The $1D:E9F8-E9F9 DEX : BPL
            // tail (30, +6 taken) follows the slot below.
            if native_slot != Some(i as u8) {
                if self.cached_sprite_slot(i).is_active() {
                    crate::cycle_ledger::charge(80 + 46);
                } else {
                    crate::cycle_ledger::charge(80 + 6);
                }
            }
            if self.cached_sprite_slot(i).is_active()
                || interruption.is_some_and(|(boundary, _)|
                    matches!(boundary, CachedSpriteCpuInterruption::AfterSlot { slot } if usize::from(slot) == i))
            {
                if interruption.is_some_and(|(boundary, _)| usize::from(boundary.slot()) == i) {
                    // The boundary can follow an inactive cached slot. Keep
                    // this slot's DEX and every lower slot for the resumed loop.
                    let mut live_slot_backup = [0; 24];
                    let (mut boundary, authority_boundary) = interruption.unwrap();
                    match boundary {
                        CachedSpriteCpuInterruption::AfterSlot { .. } => {
                            if self.cached_sprite_slot(i).is_active() {
                                self.uncache_and_execute_sprite_with_loaded_backup(i,loaded_backup,loaded_timer);
                            }
                        }
                        CachedSpriteCpuInterruption::Loading { copied_fields, .. } => {
                            if let Some(backup)=loaded_backup {
                                live_slot_backup=*backup;
                                // Native work already retired all 24 field copies.
                                boundary=CachedSpriteCpuInterruption::Loading {slot:i as u8,copied_fields:24};
                            } else {
                                self.cached_sprite_slot_mut(i).load_cached_fields_into_live_before_nmi(&mut live_slot_backup,usize::from(copied_fields));
                            }
                        }
                        CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                            continuation: None,
                            ..
                        } => {
                            self.load_cached_sprite_unless_native(i,&mut live_slot_backup,loaded_backup);
                            assert_eq!(
                                self.sprite_slot_view(i).state(),
                                9,
                                "cached Antfairy body checkpoint requires an active sprite",
                            );
                            // The ROM is inside UncacheAndExecuteSprite $1D:EA00
                            // (its $1D:EA00-EB01 load block ran, 3,160), the
                            // Sprite_ExecuteSingle_ $06:84DA wrapper (190) and
                            // Sprite_ExecuteSingle when the NMI lands; the
                            // restore half runs in the continuation.
                            let _uncache = crate::cycle_ledger::routine(0x1d_ea00);
                            crate::cycle_ledger::charge(if loaded_backup.is_some() {48}else{3_160});
                            let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                            crate::cycle_ledger::charge(if loaded_timer.is_some() {72}else{190});
                            let _execute_single = self.sprite_execute_single_lane_scope(i, true);
                            self.sprite_timers_and_oam(i);
                            let continuation = self.antfairy_draw_continuation(i);
                            self.sprite_slot_view_mut(i).add_subtype2(1);
                            boundary = CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                                slot: i as u8,
                                continuation: Some(continuation),
                            };
                        }
                        CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                            continuation: Some(_),
                            ..
                        } => unreachable!(
                            "cached Antfairy boundary was already bound before entering its native call site"
                        ),
                        CachedSpriteCpuInterruption::ExecutingGreenKnifeGuardBeforeRecruitOamPrep { .. } => {
                            self.load_cached_sprite_unless_native(i,&mut live_slot_backup,loaded_backup);
                            assert_eq!(self.sprite_slot_view(i).state(), 9);
                            assert_eq!(self.sprite_slot_view(i).sprite_type(), 0x4b);
                            let _uncache = crate::cycle_ledger::routine(0x1d_ea00);
                            crate::cycle_ledger::charge(if loaded_backup.is_some() {48}else{3_160});
                            let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                            crate::cycle_ledger::charge(if loaded_timer.is_some() {72}else{190});
                            let _execute_single = self.sprite_execute_single_lane_scope(i, true);
                            self.sprite_timers_and_oam(i);
                            self.charge_sprite_active_main_dispatch(i);
                            self.green_knife_guard_before_recruit_oam_prep(i);
                        }
                        CachedSpriteCpuInterruption::ExecutingGuardBeforeShadow {
                            continuation: None, ..
                        } => {
                            self.load_cached_sprite_unless_native(i,&mut live_slot_backup,loaded_backup);
                            assert_eq!(self.sprite_slot_view(i).state(), 9);
                            assert!((0x41..=0x44).contains(&self.sprite_slot_view(i).sprite_type()));
                            let _uncache = crate::cycle_ledger::routine(0x1d_ea00);
                            crate::cycle_ledger::charge(if loaded_backup.is_some() {48}else{3_160});
                            let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                            crate::cycle_ledger::charge(if loaded_timer.is_some() {72}else{190});
                            let _execute_single = self.sprite_execute_single_lane_scope(i, true);
                            self.sprite_timers_and_oam(i);
                            self.charge_sprite_active_main_dispatch(i);
                            let continuation = self.guard_animation_until_checkpoint(
                                i, crate::GuardAnimationCheckpoint::ShadowOamPending);
                            boundary = CachedSpriteCpuInterruption::ExecutingGuardBeforeShadow {
                                slot: i as u8, continuation: Some(continuation),
                            };
                        }
                        CachedSpriteCpuInterruption::ExecutingGuardBeforeShadow {
                            continuation: Some(_), ..
                        } => unreachable!("cached guard shadow boundary was already bound"),
                        CachedSpriteCpuInterruption::ExecutingAfterCoordinates {
                            state: None, ..
                        }
                        | CachedSpriteCpuInterruption::ExecutingAfterOamAllocation {
                            state: None, ..
                        } => {
                            self.load_cached_sprite_unless_native(i,&mut live_slot_backup,loaded_backup);
                            let state = self.sprite_slot_view(i).state();
                            assert_ne!(state, 0);
                            let _uncache = crate::cycle_ledger::routine(0x1d_ea00);
                            crate::cycle_ledger::charge(if loaded_backup.is_some() {48}else{3_160});
                            let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                            crate::cycle_ledger::charge(if loaded_timer.is_some() {72}else{190});
                            let _execute_single = self.sprite_execute_single_lane_scope(i, false);
                            boundary = if matches!(boundary,
                                CachedSpriteCpuInterruption::ExecutingAfterCoordinates { .. }) {
                                self.sprite_timers_and_oam_through_coordinates(i);
                                CachedSpriteCpuInterruption::ExecutingAfterCoordinates {
                                    slot: i as u8, state: Some(state),
                                }
                            } else {
                                self.sprite_timers_and_oam_through_oam_allocation(i);
                                CachedSpriteCpuInterruption::ExecutingAfterOamAllocation {
                                    slot: i as u8, state: Some(state),
                                }
                            };
                        }
                        CachedSpriteCpuInterruption::ExecutingAfterCoordinates {
                            state: Some(_), ..
                        }
                        | CachedSpriteCpuInterruption::ExecutingAfterOamAllocation {
                            state: Some(_), ..
                        } => unreachable!("cached OAM boundary was already bound"),
                        CachedSpriteCpuInterruption::Restoring { live_fields, .. } => {
                            self.load_cached_sprite_unless_native(i,&mut live_slot_backup,loaded_backup);
                            {
                                // $06:84DA Sprite_ExecuteSingle_ long-call wrapper
                                // (PHB PHK PLB JSR PLB RTL, 190).
                                let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                                crate::cycle_ledger::charge(if loaded_timer.is_some() {72}else{190});
                                self.sprite_execute_single(i);
                            }
                            if self.sprite_slot_view(i).pause() != 0 {
                                self.cached_sprite_slot_mut(i).clear_state();
                            }
                            self.cached_sprite_slot_mut(i)
                                .restore_live_suffix_from_backup_before_nmi(
                                    &live_slot_backup,
                                    usize::from(live_fields),
                                );
                        }
                    }
                    let dungeon = self
                        .active_dungeon_sprite_main_return
                        .take()
                        .expect("cached-sprite interruption must suspend a Module 7 sprite loop");
                    self.stage_interrupted_sprite_main_oam_scanout();
                    let continuation = GameWorkContinuation::FinishDungeonCachedSpriteMain {
                        boundary,
                        live_slot_backup,
                        dungeon,
                    };
                    match authority_boundary {
                        Some(OriginalTimingBoundary::NmiAccepted) => self
                            .game_execution_scheduler
                            .schedule_after_current_trailing_nmi(continuation),
                        Some(OriginalTimingBoundary::HostReturn) => {
                            self.game_execution_scheduler.schedule_work(continuation, 1);
                        }
                        None => {
                            if !self
                                .game_execution_scheduler
                                .schedule_after_pending_pre_main_nmi(continuation)
                            {
                                self.game_execution_scheduler.schedule_work(continuation, 1);
                            }
                        }
                    }
                    return;
                }
                if self.cached_sprite_slot(i).is_active() {
                    self.uncache_and_execute_sprite_with_loaded_backup(i,loaded_backup,loaded_timer);
                }
            }
            // $1D:E9F8-E9F9 DEX : BPL (30), taken (+6) for every slot but 0.
            crate::cycle_ledger::charge(if i > 0 { 30 + 6 } else { 30 });
        }
        // $1D:E9FB RTL (44).
        crate::cycle_ledger::charge(44);
    }

    pub(in crate::zelda_rtl) fn complete_cached_sprite_main_after_interrupted_slot(
        &mut self,
        boundary: CachedSpriteCpuInterruption,
        live_slot_backup: &[u8; 24],
    ) {
        // Cycle ledger: the remainder of a suspended ExecuteCachedSprites
        // $1D:E9DA body (the interrupted slot's UncacheAndExecuteSprite blocks
        // are not modelled across the boundary; see `execute_cached_sprites`).
        let _scope = crate::cycle_ledger::routine(0x1d_e9da);
        let interrupted_slot = usize::from(boundary.slot());
        let mut live_slot_backup = *live_slot_backup;
        match boundary {
            CachedSpriteCpuInterruption::AfterSlot { .. } => {}
            CachedSpriteCpuInterruption::Loading { copied_fields, .. } => {
                self.cached_sprite_slot_mut(interrupted_slot)
                    .complete_cached_dynamic_fields_into_live_after_nmi(
                        &mut live_slot_backup,
                        usize::from(copied_fields),
                    );
                {
                    // $06:84DA Sprite_ExecuteSingle_ long-call wrapper (190).
                    let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                    crate::cycle_ledger::charge(190);
                    self.sprite_execute_single(interrupted_slot);
                }
                if self.sprite_slot_view(interrupted_slot).pause() != 0 {
                    self.cached_sprite_slot_mut(interrupted_slot).clear_state();
                }
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_from_backup(&live_slot_backup);
            }
            CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                continuation: Some(continuation),
                ..
            } => {
                // The ROM resumes inside Sprite_ExecuteSingle under the
                // Sprite_ExecuteSingle_ $06:84DA wrapper and
                // UncacheAndExecuteSprite $1D:EA00 (the load block and the
                // entry blocks were charged when the lane suspended).
                let _uncache = crate::cycle_ledger::routine(0x1d_ea00);
                {
                    let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                    let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                    self.complete_antfairy_after_subtype2_increment(interrupted_slot, continuation);
                }
                if self.sprite_slot_view(interrupted_slot).pause() != 0 {
                    // $1D:EB03 STZ $1d00,x (38).
                    crate::cycle_ledger::charge(38);
                    self.cached_sprite_slot_mut(interrupted_slot).clear_state();
                } else {
                    // BEQ taken.
                    crate::cycle_ledger::charge(6);
                }
                // $1D:EB06-EB67: 24 PLA/STA restores + RTS (1,628).
                crate::cycle_ledger::charge(1_628);
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_from_backup(&live_slot_backup);
            }
            CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                continuation: None,
                ..
            } => unreachable!(
                "cached Antfairy subtype boundary did not bind to a native draw caller"
            ),
            CachedSpriteCpuInterruption::ExecutingGreenKnifeGuardBeforeRecruitOamPrep { .. } => {
                let _uncache = crate::cycle_ledger::routine(0x1d_ea00);
                let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                self.green_knife_guard_from_recruit_draw(interrupted_slot);
                if self.sprite_slot_view(interrupted_slot).pause() != 0 {
                    crate::cycle_ledger::charge(38);
                    self.cached_sprite_slot_mut(interrupted_slot).clear_state();
                } else {
                    crate::cycle_ledger::charge(6);
                }
                crate::cycle_ledger::charge(1_628);
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_from_backup(&live_slot_backup);
            }
            CachedSpriteCpuInterruption::ExecutingGuardBeforeShadow {
                continuation: Some(continuation), ..
            } => {
                let _uncache = crate::cycle_ledger::routine(0x1d_ea00);
                let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                self.complete_guard_animation_at_checkpoint(interrupted_slot, continuation);
                if self.sprite_slot_view(interrupted_slot).pause() != 0 {
                    crate::cycle_ledger::charge(38);
                    self.cached_sprite_slot_mut(interrupted_slot).clear_state();
                } else {
                    crate::cycle_ledger::charge(6);
                }
                crate::cycle_ledger::charge(1_628);
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_from_backup(&live_slot_backup);
            }
            CachedSpriteCpuInterruption::ExecutingGuardBeforeShadow {
                continuation: None, ..
            } => unreachable!("cached guard shadow boundary did not bind its native draw"),
            CachedSpriteCpuInterruption::ExecutingAfterCoordinates {
                state: Some(state), ..
            }
            | CachedSpriteCpuInterruption::ExecutingAfterOamAllocation {
                state: Some(state), ..
            } => {
                let _uncache = crate::cycle_ledger::routine(0x1d_ea00);
                let _wrapper = crate::cycle_ledger::routine(0x06_84da);
                let _execute_single = crate::cycle_ledger::routine(0x06_84e2);
                if matches!(boundary, CachedSpriteCpuInterruption::ExecutingAfterCoordinates { .. }) {
                    self.sprite_timers_and_oam_after_coordinates_through_oam_allocation(interrupted_slot);
                }
                self.sprite_timers_and_oam_after_oam_allocation(interrupted_slot);
                self.sprite_execute_single_after_timers(interrupted_slot, state);
                if self.sprite_slot_view(interrupted_slot).pause() != 0 {
                    crate::cycle_ledger::charge(38);
                    self.cached_sprite_slot_mut(interrupted_slot).clear_state();
                } else {
                    crate::cycle_ledger::charge(6);
                }
                crate::cycle_ledger::charge(1_628);
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_from_backup(&live_slot_backup);
            }
            CachedSpriteCpuInterruption::ExecutingAfterCoordinates {
                state: None, ..
            }
            | CachedSpriteCpuInterruption::ExecutingAfterOamAllocation {
                state: None, ..
            } => unreachable!("cached OAM boundary did not bind its dispatch state"),
            CachedSpriteCpuInterruption::Restoring { live_fields, .. } => {
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_prefix_from_backup_after_nmi(
                        &live_slot_backup,
                        usize::from(live_fields),
                    );
            }
        }

        // The interrupted slot's $1D:E9F8-E9F9 DEX : BPL (30, +6 taken).
        crate::cycle_ledger::charge(if interrupted_slot > 0 { 30 + 6 } else { 30 });
        for i in (0..interrupted_slot).rev() {
            self.sprite_system_mut().set_cur_object_index(i as u8);
            // $1D:E9ED-E9F3 (80) + JSR (46) or BEQ taken (+6); see
            // `execute_cached_sprites`.
            if self.cached_sprite_slot(i).is_active() {
                crate::cycle_ledger::charge(80 + 46);
                self.uncache_and_execute_sprite(i);
            } else {
                crate::cycle_ledger::charge(80 + 6);
            }
            // $1D:E9F8-E9F9 DEX : BPL (30), taken (+6) for every slot but 0.
            crate::cycle_ledger::charge(if i > 0 { 30 + 6 } else { 30 });
        }
        // $1D:E9FB RTL (44). The Sprite_Main suffix after the cached-sprite
        // call was already charged by `complete_sprite_main_after_all_slots`
        // in the host that suspended here, so the replicated CHR half-slot
        // publication below charges nothing.
        crate::cycle_ledger::charge(44);
        if self.game_state.display.has_chr_halfslot_request() {
            let chr_halfslot_request = self.game_state.display.chr_halfslot_request;
            self.sprite_system_mut()
                .set_chr_halfslot_state(chr_halfslot_request);
        }
    }

    // void UncacheAndExecuteSprite(int k) {  // 9dea00
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn uncache_and_execute_sprite(&mut self, k: usize) {
        self.uncache_and_execute_sprite_with_loaded_backup(k,None,None);
    }

    fn load_cached_sprite_unless_native(
        &mut self,
        k: usize,
        backup: &mut [u8; 24],
        native: Option<&[u8; 24]>,
    ) {
        if let Some(native) = native {
            *backup = *native;
        } else {
            self.cached_sprite_slot_mut(k).load_cached_into_live(backup);
        }
    }

    fn uncache_and_execute_sprite_with_loaded_backup(
        &mut self,
        k: usize,
        native: Option<&[u8; 24]>,
        timer: Option<(u8, crate::cycle_models::sprite_timers::SpriteTimersTail)>,
    ) {
        // Cycle ledger: UncacheAndExecuteSprite $1D:EA00 (m8 x8, X <= 15 so
        // no abs,x page crossing). $1D:EA00-EB01: 24 backup pushes, 24 cached
        // field loads into the live slot, JSL Sprite_ExecuteSingle_ (62),
        // LDA $f00,x, BEQ not taken: 3,160.
        let _scope = crate::cycle_ledger::routine(0x1d_ea00);
        crate::cycle_ledger::charge(if native.is_some() { 48 } else { 3_160 });
        let mut bak = [0u8; 24];
        self.load_cached_sprite_unless_native(k, &mut bak, native);
        {
            // $06:84DA Sprite_ExecuteSingle_ long-call wrapper (PHB PHK PLB
            // JSR Sprite_ExecuteSingle PLB RTL, 190).
            let _wrapper = crate::cycle_ledger::routine(0x06_84da);
            if let Some((state, tail)) = timer {
                crate::cycle_ledger::charge(72); // Retained PLB and RTL of long wrapper.
                self.sprite_execute_single_after_native_timers(k, state, tail);
            } else {
                crate::cycle_ledger::charge(190);
                self.sprite_execute_single(k);
            }
        }
        if self.sprite_slot_view(k).pause() != 0 {
            // $1D:EB03 STZ $1d00,x (38).
            crate::cycle_ledger::charge(38);
            self.cached_sprite_slot_mut(k).clear_state();
        } else {
            // BEQ taken.
            crate::cycle_ledger::charge(6);
        }
        // $1D:EB06-EB67: 24 PLA/STA restores + RTS (1,628).
        crate::cycle_ledger::charge(1_628);
        self.cached_sprite_slot_mut(k)
            .restore_live_from_backup(&bak);
    }

    // void Dungeon_CacheTransSprites() {  // 89c176
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn dungeon_cache_trans_sprites(&mut self) {
        if self.game_state.world.location.is_outdoors() {
            return;
        }
        let value = self.game_state.world.location.indoor_flag();
        self.sprite_system_mut().set_alt_sprites_flag(value);
        for k in (0..16usize).rev() {
            self.dungeon_cache_trans_sprite_fields(k, None, self.dungeon_cache_last_field(k));
        }
    }

    fn dungeon_cache_trans_sprites_through_cpu_progress(
        &mut self,
        target_slot: u8,
        through: CachedSpriteCacheField,
    ) {
        assert!(self.game_state.world.location.is_indoors());
        let indoor_flag = self.game_state.world.location.indoor_flag();
        self.sprite_system_mut().set_alt_sprites_flag(indoor_flag);
        for k in (usize::from(target_slot)..16).rev() {
            if k == usize::from(target_slot) {
                assert!(through <= self.dungeon_cache_last_field(k));
                self.dungeon_cache_trans_sprite_fields(k, None, through);
                return;
            }
            self.dungeon_cache_trans_sprite_fields(k, None, self.dungeon_cache_last_field(k));
        }
        unreachable!("cache checkpoint slot is outside the 16-slot C loop");
    }

    fn dungeon_resume_cache_trans_sprites_after_cpu_progress(
        &mut self,
        target_slot: u8,
        completed: CachedSpriteCacheField,
    ) {
        assert!(self.game_state.world.location.is_indoors());
        let target_slot = usize::from(target_slot);
        assert!(completed <= self.dungeon_cache_last_field(target_slot));
        self.dungeon_cache_trans_sprite_fields(
            target_slot,
            Some(completed),
            self.dungeon_cache_last_field(target_slot),
        );
        for k in (0..target_slot).rev() {
            self.dungeon_cache_trans_sprite_fields(k, None, self.dungeon_cache_last_field(k));
        }
    }

    fn dungeon_cache_last_field(&self, k: usize) -> CachedSpriteCacheField {
        let slot = self.sprite_slot_view(k);
        if slot.pause() != 0 || matches!(slot.state(), 4 | 10) {
            CachedSpriteCacheField::YHigh
        } else {
            CachedSpriteCacheField::IgnoreProjectile
        }
    }

    fn dungeon_cache_trans_sprite_fields(
        &mut self,
        k: usize,
        completed: Option<CachedSpriteCacheField>,
        through: CachedSpriteCacheField,
    ) {
        for field in CachedSpriteCacheField::C_SOURCE_ORDER {
            if completed.is_none_or(|completed| field > completed) && field <= through {
                self.cached_sprite_slot_mut(k).cache_field_from_live(field);
            }
        }
    }
}
