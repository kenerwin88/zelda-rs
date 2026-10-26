//! Player initialization, native stores, main dispatch, ground and bunny states, and camera transitions.

use super::*;
use crate::game_state::constants::{BIT9_OF_XCOORD, OAM_PRIORITY_VALUE, OAM_PRIORITY_VALUE_2, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER};

#[cfg(test)]
mod control_price_tests {
    use super::*;
    use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
        CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};

    #[test]
    fn default_handler_prices_match_executed_rom_paths() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for auxiliary in [0u8, 1] {
            for timer in [0u8, 1] {
                for poof in [0u8, 1] {
                    for handler in [0u8, 23, 28] {
                        let mut native = ZeldaState::new();
                        let mut image = native.wram_image().into_owned();
                        image[0x4d] = auxiliary;
                        image[0x3f5] = timer;
                        image[0x3f7] = poof;
                        image[0x5d] = handler;
                        image[0x2e2] = 1;
                        native.import_wram_image(&image);
                        native.sync_native_game_state_from_ram();
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        machine.ram.copy_from_slice(&native.wram_image());
                        machine.cpu.e = false;
                        machine.cpu.unpack_flags(0x30);
                        machine.cpu.k = 7;
                        machine.cpu.db = 7;
                        machine.cpu.pc = 0x8109;
                        machine.cpu.sp = 0x1fc;
                        let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                            CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                        let mut source = RomCpuTimingProbe::new(machine, clock,
                            SourcePpuReadState::snes9x_reset()).unwrap();
                        let mut measured = 0;
                        let mut ended = false;
                        for _ in 0..20000 {
                            let cpu = &source.snes().cpu;
                            let pc = (u32::from(cpu.k) << 16) | u32::from(cpu.pc);
                            if matches!(pc, 0x078130 | 0x0781a0 | 0x0783a1) {
                                ended = true; break;
                            }
                            let step = source.step().unwrap();
                            if (0x078109..=0x07812e).contains(&pc) {
                                measured += step.transactions.iter()
                                    .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                            }
                            if pc == 0x078128 { ended = true; break; }
                        }
                        assert!(ended);
                        let entry = native.link_default_entry_master();
                        let transformed = native.link_handle_bunny_transformation();
                        assert_eq!(entry + native.link_default_tail_master(transformed), measured,
                            "auxiliary={auxiliary} timer={timer} poof={poof} handler={handler}");
                    }
                }
            }
        }
    }

    #[test]
    fn auxiliary_handler_prices_match_executed_rom_paths() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for indoors in [0u8, 1] {
            for electric in [0u8, 1] {
                for cape in [0u8, 1] {
                    let mut native = ZeldaState::new();
                    let mut image = native.wram_image().into_owned();
                    image[0x1b] = indoors;
                    image[0x360] = electric;
                    image[0x55] = cape;
                    native.import_wram_image(&image);
                    native.sync_native_game_state_from_ram();
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    machine.ram.copy_from_slice(&native.wram_image());
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(0x30);
                    machine.cpu.k = 7;
                    machine.cpu.db = 7;
                    machine.cpu.pc = 0x8130;
                    machine.cpu.sp = 0x1fc;
                    let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                    let mut source = RomCpuTimingProbe::new(machine, clock,
                        SourcePpuReadState::snes9x_reset()).unwrap();
                    let mut measured = 0;
                    for _ in 0..20000 {
                        let cpu = &source.snes().cpu;
                        let pc = (u32::from(cpu.k) << 16) | u32::from(cpu.pc);
                        let step = source.step().unwrap();
                        if (0x078130..=0x07819f).contains(&pc) {
                            measured += step.transactions.iter()
                                .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                        }
                        if matches!(pc, 0x078190 | 0x07819d) { break; }
                    }
                    assert!(matches!(source.snes().cpu.pc, 0x86b5 | 0x996c));
                    assert_eq!(native.handle_link_from1d_self_master(), measured,
                        "indoors={indoors} electric={electric} cape={cape}");
                }
            }
        }
    }

    #[test]
    fn player_control_prices_match_executed_rom_paths() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let mut cases = vec![Vec::new()];
        for health in [0u8, 1, 2, 0xa8, 0xff] {
            for damage in [0u8, 1, 2, 0xff] {
                for handler in [0u8, 1, 4, 23] {
                    cases.push(vec![(0xf36d, health), (0x373, damage), (0x5d, handler)]);
                }
            }
        }
        for cape in [0u8, 1] {
            for disabled in [0u8, 1] {
                for ancilla in [0u8, 5] {
                    for action in [0u8, 1] {
                        for spin in [0u8, 1] {
                            for blink in [0u8, 1] {
                                cases.push(vec![(0x373, 1), (0xf36d, 16), (0x55, cape),
                                    (0x37b, disabled), (0xc4a, ancilla), (0x300, action),
                                    (0x3d, spin), (0x31f, blink)]);
                            }
                        }
                    }
                }
            }
        }
        for changes in cases {
            let mut native = ZeldaState::new();
            let mut image = native.wram_image().into_owned();
            for &(address, value) in &changes { image[address] = value; }
            native.import_wram_image(&image);
            native.sync_native_game_state_from_ram();
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&native.wram_image());
            machine.cpu.e = false;
            machine.cpu.unpack_flags(0x30);
            machine.cpu.k = 7;
            machine.cpu.db = 7;
            machine.cpu.pc = 0x807f;
            machine.cpu.sp = 0x1fc;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
            let mut source = RomCpuTimingProbe::new(machine, clock,
                SourcePpuReadState::snes9x_reset()).unwrap();
            let mut measured = 0;
            let mut dispatched = false;
            for _ in 0..1000 {
                let cpu = &source.snes().cpu;
                let pc = (u32::from(cpu.k) << 16) | u32::from(cpu.pc);
                let step = source.step().unwrap();
                if (0x07807f..=0x078106).contains(&pc) {
                    measured += step.transactions.iter()
                        .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                }
                if pc == 0x078106 { dispatched = true; break; }
            }
            assert!(dispatched);
            assert_eq!(native.link_control_self_master(), measured, "operands={changes:?}");
        }
    }
}

impl ZeldaState {
    pub(crate) fn native_player_animation_table_byte(address: u32) -> Option<u8> {
        let (table, base): (&[u8], u32) = match address {
            0x07e671..=0x07e674 => (&LINK_HANDLE_MOVING_ANIMATION_FULL_LONG_ENTRY_FACING_DIRECTION_BITS, 0x07e671),
            0x07e675..=0x07e684 => (&LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_POSE_ANIMATION_DELAYS, 0x07e675),
            0x07e685..=0x07e69c => (&LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_WALK_ANIMATION_DELAYS, 0x07e685),
            0x07e842..=0x07e879 => (&LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_DELAYS, 0x07e842),
            0x07e87a..=0x07e880 => (&LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_MIN_ANIM_STEPS, 0x07e87a),
            0x07e881..=0x07e887 => (&LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_THRESHOLDS, 0x07e881),
            _ => return None,
        };
        Some(table[(address - base) as usize])
    }

    pub(super) fn bit_sum4(value: u8) -> u8 {
        (value & 1) + ((value >> 1) & 1) + ((value >> 2) & 1) + ((value >> 3) & 1)
    }

    pub(in crate::zelda_rtl) fn dungeon_handle_layer_change(&mut self) {
        self.follower_link_state_mut().mark_lower_level_mirror();
        if self
            .game_state
            .dungeon
            .stair_movement
            .kind_of_in_room_staircase()
            == 0
        {
            self.increment_dungeon_room_index_by(16);
        }
        if self
            .game_state
            .dungeon
            .stair_movement
            .kind_of_in_room_staircase()
            != 2
        {
            self.follower_link_state_mut().mark_lower_level();
        }
        self.follower_link_state_mut()
            .clear_about_to_jump_off_ledge();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(in crate::zelda_rtl) fn check_ability_to_swim(&mut self) {
        if !self.game_state.player.follower_link.is_bunny_mirror()
            && self.game_state.inventory.items.has_flippers()
        {
            return;
        }
        if self.game_state.inventory.items.has_moon_pearl() {
            self.follower_link_state_mut().clear_bunny_mirror();
        }
        self.follower_link_state_mut().set_visibility_status(0x0c);
        let submodule = if self.game_state.world.location.is_indoors() {
            20
        } else {
            42
        };
        self.set_submodule(submodule);
    }

    /// Retained animation and LinkOam publish only their live store, including
    /// byte writes to words shared with the tile and OAM owners.
    pub(crate) fn native_player_drawing_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = address as usize;
        if matches!(address, 0x31f | 0x355 | 0x356) {
            assert_eq!(width, 1);
            match address {
                0x31f => self.follower_link_state_mut().set_blink_countdown(value as u8),
                0x355 => self.follower_link_state_mut().set_secondary_water_grass_timer(value as u8),
                0x356 => self.follower_link_state_mut().set_primary_water_grass_timer(value as u8),
                _ => unreachable!(),
            }
            return true;
        }
        if matches!(address, 0x20..=0x21 | 0x26 | 0x2d..=0x2f | 0x44..=0x45 | 0x102..=0x105 | 0x107..=0x109 | 0x323 | 0x346..=0x347) {
            for byte in 0..usize::from(width) {
                let value_byte = (value >> (byte * 8)) as u8;
                if address + byte != 0x10a {
                    assert!(self.game_state.player.follower_link.write_drawing_byte(address + byte, value_byte), "drawing store address={address:05x} value={value:04x} width={width} byte={byte}");
                }
            }
            for byte in 0..usize::from(width) { self.write_wram_byte(address + byte, value.to_le_bytes()[byte]); }
            return true;
        }
        let (base, old) = match address {
            0x64..=0x65 => (0x64, read_le_u16(&self.ram, OAM_PRIORITY_VALUE)),
            0x72..=0x77 => (address & !1, read_le_u16(&self.ram, address & !1)),
            0x90..=0x93 => (address & !1, read_le_u16(&self.ram, address & !1)),
            0x100..=0x101 => (0x100, read_le_u16(&self.ram, LINK_DMA_GRAPHICS_INDEX)),
            0x352..=0x353 => (0x352, read_le_u16(&self.ram, SORT_SPRITES_OFFSET_INTO_OAM_BUFFER)),
            0x35d..=0x35e => (0x35d, read_le_u16(&self.ram, OAM_PRIORITY_VALUE_2)),
            0x3fa..=0x3fb => (0x3fa, read_le_u16(&self.ram, BIT9_OF_XCOORD)),
            0x354 => {assert_eq!(width, 1);self.oam_state_mut().set_player_oam_computed_value(value as u8);return true;}
            _ => return false,
        };
        assert!(address + usize::from(width) <= base + 2);
        let word = if width == 2 { value } else {
            let shift = (address-base)*8;
            (old & !(255 << shift)) | ((value & 255) << shift)
        };
        match base {
            0x64 => self.oam_state_mut().set_priority_word(word),
            0x72 => self.tile_detect_position_mut().set_interaction_scratch_y(word),
            0x74 => self.tile_detect_position_mut().set_interaction_scratch_x(word),
            0x76 => self.tile_detect_position_mut().set_interacting_tile(word),
            0x90 => self.oam_state_mut().set_current_pointer(word),
            0x92 => self.oam_state_mut().set_current_extended_pointer(word),
            0x100 => self.follower_link_state_mut().set_link_dma_graphics_index_word(word),
            0x352 => self.oam_state_mut().set_sort_sprites_offset(word),
            0x35d => self.oam_state_mut().set_priority_value_2(word),
            0x3fa => self.follower_link_state_mut().set_bit9_of_xcoord_word(word),
            _ => unreachable!(),
        }
        true
    }

    pub(crate) fn native_load_player_initialization_store(&mut self, address: usize, value: u16, width: u8) -> bool {
        match address {
            0x25 | 0x26 | 0x2f | 0x3a..=0x3d | 0x46 | 0x48 | 0x4d | 0x50 | 0x55 | 0x5e | 0x67 | 0x79 |
            0x20b | 0x2cb | 0x2da | 0x2e1 | 0x2e2 | 0x2f5 | 0x300 | 0x301 | 0x308 | 0x309 | 0x30a |
            0x30d | 0x30e | 0x31f | 0x326 | 0x328 | 0x32a | 0x34f | 0x350 | 0x360 | 0x376 | 0x37a | 0x37b => {
                self.follower_link_state_mut().write_initialization_store(address, value, width);
            }
            0x32b | 0x32d => {assert_eq!(width,2);self.swim_acceleration_mut().set_mode(address-0x32b,value);}
            0x32f | 0x331 => {assert_eq!(width,2);self.swim_acceleration_mut().set_speed_active_flag(address-0x32f,value);}
            0x334 | 0x336 => {assert_eq!(width,2);self.swim_acceleration_mut().set_max_speed(address-0x334,value);}
            0x33c | 0x33e => {assert_eq!(width,2);self.swim_acceleration_mut().set_acceleration(address-0x33c,value);}
            _ => return false,
        }
        true
    }

    /// New-game placement and its blanket child publish their player, scratch,
    /// and ancilla stores in order, preserving the child coordinate handoff.
    pub(crate) fn native_opening_player_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        match address {
            0 | 2 => {
                assert_eq!(width, 2);
                for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
                    self.game_state.sprites.workspace.write_zero_page_byte(address + index, byte);
                }
            }
            0x20 | 0x22 => {
                assert_eq!(width, 2);
                for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
                    self.game_state.player.follower_link.write_opening_byte(address + index, byte);
                }
            }
            0x5d | 0x374 | 0x37c | 0x37d => {
                assert_eq!(width, 1);
                self.game_state.player.follower_link.write_opening_byte(address, value as u8);
            }
            0x280 | 0x3ca | 0xbfa | 0xc04 | 0xc0e | 0xc18 | 0xc4a | 0xc7c | 0xc90 => {
                // AncillaAdd_Blanket selects slot zero, then its coordinate
                // child retires the four bytes from the retained DP words.
                assert_eq!(width, 1);
                self.game_state.sprites.ancilla_slots.set_shared_byte(address, value as u8);
            }
            _ => return false,
        }
        for byte in 0..usize::from(width) { self.write_wram_byte(address + byte, value.to_le_bytes()[byte]); }
        true
    }

    /// Room-entry caching publishes its reset bytes and camera/player snapshots
    /// in source order, retaining partial caches across interruption boundaries.
    pub(crate) fn native_room_entry_cache_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        if matches!(address, 0xc180..=0xc19e) {
            assert_eq!(width, 2);
            assert_eq!(address & 1, 0);
            let world = &mut self.game_state.world;
            match address {
                0xc180 | 0xc182 => self.game_state.display.ppu_scroll_copy.write_room_entry_cache_word(address, value),
                0xc184 | 0xc186 | 0xc19e => {
                    for (i, byte) in value.to_le_bytes().into_iter().enumerate() {
                        assert!(self.game_state.player.follower_link.write_room_entry_cache_byte(address + i, byte));
                    }
                }
                0xc188 => world.transient.cached_room_bounds_y_start = value,
                0xc18a => world.transient.cached_room_bounds_y_end = value,
                0xc18c => world.transient.cached_room_bounds_x_start = value,
                0xc18e => world.transient.cached_room_bounds_x_end = value,
                0xc190..=0xc196 => world.camera_boundaries.cached_scroll_targets[(address - 0xc190) / 2] = value,
                0xc198 => world.camera_boundaries.cached_camera_y_low = value,
                0xc19a => world.camera_boundaries.cached_camera_x_low = value,
                0xc19c => {
                    world.transient.cached_quadrant_fullsize_x = value as u8;
                    world.transient.cached_quadrant_fullsize_y = (value >> 8) as u8;
                }
                _ => unreachable!(),
            }
        } else {
            if !matches!(address, 0x11 | 0xb0 | 0x200 | 0x418 | 0x641 | 0x642 | 0xc1a6..=0xc1aa) {
                return false;
            }
            assert_eq!(width, 1);
            let byte = value as u8;
            match address {
                0x11 => self.game_state.frame.set_submodule(byte),
                0xb0 => self.game_state.frame.set_subsubmodule(byte),
                0x200 => self.game_state.world.overworld.map_ui.set_map_state(byte),
                0x418 => self.game_state.world.overworld.transition.set_screen_transition(byte),
                0x641 | 0x642 | 0xc1aa => {
                    assert!(self.game_state.dungeon.write_room_entry_cache_byte(address, byte));
                }
                0xc1a6..=0xc1a8 => {
                    assert!(self.game_state.player.follower_link.write_room_entry_cache_byte(address, byte));
                }
                0xc1a9 => self.game_state.world.transient.set_standing_in_doorway_cached(byte),
                _ => unreachable!(),
            }
        }
        for byte in 0..usize::from(width) { self.write_wram_byte(address + byte, value.to_le_bytes()[byte]); }
        true
    }

    /// Interactive cleanup retains its slot loop and pickup/recoil decisions.
    /// Publish each retired store without adopting other components from RAM.
    pub(crate) fn native_interactive_cleanup_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let address = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        assert_eq!(width, 1);
        let byte = value as u8;
        if self.game_state.player.follower_link.write_interactive_cleanup_byte(address, byte) {
            // The native player component owns these individual action bytes.
        } else {
            match address {
                0xc4a..=0xc4f => self.game_state.sprites.ancilla_slots.set_type(address - 0xc4a, byte),
                0x2f2 => {
                    assert_eq!(byte, 0);
                    self.game_state.sprites.follower_runtime.clear_event_flags();
                }
                0x35f => {
                    assert_eq!(byte, 0);
                    self.game_state.minigame.clear_flag_boomerang_in_place();
                }
                0x3fc => self.game_state.minigame.set_is_archer_or_shovel_game(byte),
                0x646 => {
                    assert_eq!(byte, 0);
                    self.game_state.dungeon.environment.clear_somaria_block_switch_counter();
                }
                _ => return false,
            }
        }
        self.ram[address] = byte;
        true
    }

    pub(in crate::zelda_rtl) fn link_initialize(&mut self) {
        self.follower_link_state_mut()
            .initialize_link_action_state();
        self.link_reset_swimming_state();
        self.follower_link_state_mut().finish_initialization();

        if self
            .game_state
            .enhanced_features
            .has(LINK_INITIALIZE_FEATURES0_MISC_BUG_FIXES)
        {
            self.follower_link_state_mut()
                .clear_misc_bugfix_movement_state();
            self.set_bg1_y_offset(0);
            self.set_bg1_x_offset(0);

            if !self.game_state.inventory.items.has_moon_pearl()
                && self.game_state.inventory.save_progress.dark_world_state() != 0
            {
                self.follower_link_state_mut().become_bunny_handler();
                self.load_gear_palettes_bunny();
            }
        }
    }

    pub(in crate::zelda_rtl) fn link_reset_properties_a(&mut self) {
        self.follower_link_state_mut()
            .reset_movement_and_transformation_state();
        // C's Link_ResetProperties_A also zeroes is_archer_or_shovel_game,
        // tagalong_event_flags, and BYTE(tiledetect_tile_type); route those through
        // the native states that own them so no model goes stale.
        self.minigame_state_mut().clear_is_archer_or_shovel_game();
        self.follower_state_mut().clear_event_flags();
        self.tile_detect_position_mut().clear_tile_type_low();
        self.link_reset_swimming_state();
        self.link_reset_properties_b();
    }

    pub(super) fn link_reset_properties_b(&mut self) {
        self.follower_link_state_mut()
            .reset_platform_and_pit_state();
        self.link_reset_properties_c();
    }

    pub(in crate::zelda_rtl) fn link_reset_properties_c(&mut self) {
        if self
            .game_state
            .enhanced_features
            .has(LINK_RESET_PROPERTIES_C_FEATURES0_MISC_BUG_FIXES)
        {
            self.clear_custom_spell_animation();
        }

        self.follower_link_state_mut().reset_action_state();
    }

    pub(in crate::zelda_rtl) fn link_tuck_into_bed(&mut self) {
        self.follower_link_state_mut().set_y(0x215a);
        self.follower_link_state_mut().set_x(0x0940);
        self.follower_link_state_mut().setup_bed_pose();
        self.ancilla_add_blanket(0x20);
    }

    pub(in crate::zelda_rtl) fn link_reset_swimming_state_marked(&mut self) {
        // $07:8158, $07:83E1 and $07:9747 JSL $983A.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07983a));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x07983a);
        self.link_reset_swimming_state();
        b.block(0x079849);
    }

    pub(in crate::zelda_rtl) fn link_reset_swimming_state(&mut self) {
        self.follower_link_state_mut().reset_swim_stroke_state();
        self.reset_all_acceleration();
    }

    pub(in crate::zelda_rtl) fn link_reset_state_after_damaging_pit(&mut self) {
        self.link_reset_swimming_state();
        let has_moon_pearl = self.game_state.inventory.items.has_moon_pearl();
        self.follower_link_state_mut().reset_after_damaging_pit(has_moon_pearl);
    }

    pub(super) fn link_state_bunny_recache(&mut self) {
        let has_moon_pearl = self.game_state.inventory.items.has_moon_pearl();
        self.follower_link_state_mut()
            .recache_bunny_state(has_moon_pearl);
        self.link_reset_swimming_state();
        self.follower_link_state_mut().set_handler_state(2);
        if self.game_state.inventory.items.has_moon_pearl() {
            self.follower_link_state_mut().clear_handler_state();
            self.load_actual_gear_palettes();
        }
    }

    pub(in crate::zelda_rtl) fn link_set_to_deep_water(&mut self) {
        self.follower_link_state_mut().enter_deep_water();
        self.link_reset_swimming_state();
    }

    pub(super) fn link_splash_upon_landing(&mut self) {
        if self.game_state.player.follower_link.is_bunny_mirror() {
            if self.game_state.player.follower_link.is_in_deep_water() {
                self.ancilla_add_splash(21, 0);
                self.link_state_bunny_recache();
                return;
            }
            let has_moon_pearl = self.game_state.inventory.items.has_moon_pearl();
            self.follower_link_state_mut().land_after_splash(has_moon_pearl);
        } else if self.game_state.player.follower_link.is_in_deep_water() {
            if self.game_state.player.follower_link.handler_state()
                != LINK_SPLASH_UPON_LANDING_PLAYER_HANDLER_STATE_RECOIL_OTHER
            {
                self.ancilla_add_splash(21, 0);
            }
            self.link_force_unequip_cape_quietly();
            let has_moon_pearl = self.game_state.inventory.items.has_moon_pearl();
            self.follower_link_state_mut().land_after_splash(has_moon_pearl);
        } else {
            let has_moon_pearl = self.game_state.inventory.items.has_moon_pearl();
            self.follower_link_state_mut().land_after_splash(has_moon_pearl);
        }
    }

    pub(in crate::zelda_rtl) fn cache_camera_properties(&mut self) {
        if self.lag_free {
            // $01:FF28..FFB5: exact fixed camera/property cache and RTL.
            crate::cycle_ledger::charge_routine(0x01_ff28, 1776);
        }
        let bg2_x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg2_y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.cache_bg2_live_scroll_from(bg2_x, bg2_y);
        self.follower_link_state_mut().cache_current_position();
        let y_start = self.game_state.world.room_bounds.y_bound(0);
        let y_end = self.game_state.world.room_bounds.y_bound(2);
        let x_start = self.game_state.world.room_bounds.x_bound(0);
        let x_end = self.game_state.world.room_bounds.x_bound(2);
        self.set_cached_room_bounds(y_start, y_end, x_start, x_end);
        self.cache_scroll_targets();
        self.cache_camera_scroll();
        self.cache_quadrant_fullsize_state();
        self.follower_link_state_mut().cache_current_quadrants();
        self.follower_link_state_mut().cache_facing();
        self.follower_link_state_mut().cache_lower_level_states();
        let doorway_state = self.game_state.player.follower_link.doorway_state();
        self.set_standing_in_doorway_cached(doorway_state);
        self.dungeon_stair_movement_mut().cache_current_floor();
    }

    pub(in crate::zelda_rtl) fn link_main(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x07_8000);
        crate::cycle_ledger::charge(340); // Bank/width setup, saved XY, pause and immobilization gate.
        self.follower_link_state_mut()
            .cache_previous_position_from_current_xy_order();
        self.clear_modal_pause_flag();
        if !self.game_state.player.follower_link.is_immobilized() {
            crate::cycle_ledger::charge(46); // JSR control handler.
            self.link_control_handler();
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(46); // JSR shared ancilla scan.
        self.handle_somaria_and_graves();
        crate::cycle_ledger::charge(72); // PLB : RTL after the scan returns.
    }

    pub(super) fn link_control_handler(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x07_807f);
        if self.lag_free { crate::cycle_ledger::charge(self.link_control_self_master()); }
        if self.game_state.player.follower_link.given_damage() != 0 {
            if self.game_state.player.follower_link.is_cape_active() {
                self.follower_link_state_mut().clear_given_damage();
                self.follower_link_state_mut().clear_auxiliary_state();
                self.follower_link_state_mut().set_incapacitated_timer(0);
            } else if self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer()
                == 0
            {
                let dmg = self.game_state.player.follower_link.given_damage();
                self.follower_link_state_mut().clear_given_damage();
                if self.ancilla_slot_view(0).ancilla_type() == 5
                    && self.game_state.player.follower_link.action_handler_timer() == 0
                    && self
                        .game_state
                        .player
                        .follower_link
                        .spin_attack_delay_timer()
                        != 0
                {
                    self.ancilla_slot_view_mut(0).set_ancilla_type(0);
                    self.minigame_state_mut().clear_flag_boomerang_in_place();
                }
                if self.game_state.player.follower_link.blink_countdown() == 0 {
                    self.follower_link_state_mut().set_blink_countdown(58);
                }
                self.ancilla_sfx2_near(38);
                self.sprite_battle_mut().increment_times_hurt_by_sprites();
                let new_dmg = self
                    .game_state
                    .inventory
                    .player_resources
                    .current_health()
                    .wrapping_sub(dmg);
                let new_dmg = if new_dmg == 0 || new_dmg >= 0xa8 {
                    let main_layers = self.game_state.display.main_screen_layers;
                    let sub_layers = self.game_state.display.sub_screen_layers;
                    self.set_mapbak_tm(main_layers);
                    self.set_mapbak_ts(sub_layers);
                    let main_module = self.game_state.frame.main_module;
                    self.set_saved_module_for_menu(main_module);
                    self.set_main_module(18);
                    self.set_submodule(1);
                    self.follower_link_state_mut().clear_blink_countdown();
                    self.player_resources_mut().set_heart_filler(0);
                    0
                } else {
                    new_dmg
                };
                self.player_resources_mut().set_current_health(new_dmg);
            }
        }
        if self.game_state.player.follower_link.handler_state() != 0 {
            self.player_check_handle_cape_stuff();
        }
        match self.game_state.player.follower_link.handler_state() {
            0 => self.link_state_default(),
            1 => self.link_state_pits(),
            2 | 6 => self.link_state_recoil(),
            3 | 30 => self.link_state_spin_attack(),
            4 => self.player_handler_04_swimming(),
            5 => self.link_state_on_ice(),
            7 => self.link_state_zapped(),
            8 => self.link_state_using_ether(),
            9 => self.link_state_using_bombos(),
            10 => self.link_state_using_quake(),
            11 => self.link_hop_hopping_south_ow(),
            12 => self.link_state_hopping_horizontally_ow(),
            13 => self.link_state_hopping_diagonally_up_ow(),
            14 => self.link_state_hopping_diagonally_down_ow(),
            15 | 16 => self.link_state_0_f(),
            17 => self.link_state_dashing(),
            18 => self.link_state_exiting_dash(),
            19 => self.link_state_hookshotting(),
            20 => self.link_state_crossing_worlds(),
            21 => self.player_handler_15_hold_item(),
            22 => self.link_state_sleeping(),
            23 => self.player_handler_17_bunny(),
            24 => self.link_state_holding_big_rock(),
            25 => self.link_state_receiving_ether(),
            26 => self.link_state_receiving_bombos(),
            27 => self.link_state_reading_desert_tablet(),
            28 => self.link_state_temporary_bunny(),
            29 => self.link_state_tree_pull(),
            _ => {}
        }
    }

    /// $07:807F..8106: damage/cape/health branches and handler dispatch.
    /// The SFX call instruction is included; its body and the selected
    /// player's tail handler retain their own owners.
    fn link_control_self_master(&self) -> u64 {
        let p = &self.game_state.player.follower_link;
        let mut clocks = 48;
        if p.given_damage() == 0 { clocks += 6; }
        else {
            clocks += 40;
            if p.is_cape_active() { clocks += 102; }
            else {
                clocks += 6 + 48;
                if p.sprite_damage_disable_timer() != 0 { clocks += 6; }
                else {
                    clocks += 152;
                    if self.ancilla_slot_view(0).ancilla_type() != 5 { clocks += 6; }
                    else {
                        clocks += 48;
                        if p.action_handler_timer() != 0 { clocks += 6; }
                        else { clocks += 40 + if p.spin_attack_delay_timer() == 0 { 6 } else { 64 }; }
                    }
                    clocks += 48 + if p.blink_countdown() != 0 { 6 } else { 48 };
                    let health = self.game_state.inventory.player_resources.current_health()
                        .wrapping_sub(p.given_damage());
                    clocks += 218 + if health == 0 { 6 + 352 } else {
                        32 + if health < 0xa8 { 6 } else { 352 }
                    } + 40;
                }
            }
        }
        clocks + 40 + if p.handler_state() == 0 { 6 } else { 46 } + 98
    }

    pub(in crate::zelda_rtl) fn link_state_default(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_8109));
        if self.lag_free {
            // $07:8109..811D: camera/bunny call instructions and control.
            // The unported F5 debug-cheat toggle ($8112..8117) stays unpriced.
            crate::cycle_ledger::charge(self.link_default_entry_master());
        }
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.lag_free {
                // $07:811F..8128: tail to Bunny or RTS; body excluded.
                crate::cycle_ledger::charge(self.link_default_tail_master(true));
            }
            if self.game_state.player.follower_link.handler_state() == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        if self.lag_free {
            // $07:8129..812E, including the taken BCC from $811D.
            crate::cycle_ledger::charge(self.link_default_tail_master(false));
        }
        self.follower_link_state_mut().set_pit_correction_timer(0);
        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.handle_link_from1_d();
        } else {
            self.player_handler_00_ground_3();
        }
    }

    fn link_default_entry_master(&self) -> u64 {
        // $07:8109..811D, excluding the unported debug toggle at $8112.
        102 + if self.ram[0xf5] & 0x80 == 0 { 6 } else { 0 } + 62
    }

    fn link_default_tail_master(&self, transformed: bool) -> u64 {
        // $07:811F..812E, selected after BunnyTransformation returns.
        let p = &self.game_state.player.follower_link;
        if transformed {
            56 + if p.handler_state() == 23 { 30 } else { 6 + 42 }
        } else {
            6 + 72 + if p.has_auxiliary_state() { 0 } else { 6 }
        }
    }

    // $07:8130..819F: scratch/property clearing, electrical/cape branches,
    // and call/BRL instructions only. Swimming reset, cape/sword reset, SFX
    // and selected recoil/zap bodies retain separate ownership.
    fn handle_link_from1d_self_master(&self) -> u64 {
        let p = &self.game_state.player.follower_link;
        let mut clocks = 630;
        if p.electrocute_on_touch() == 0 {
            clocks += 6 + 40 + if self.game_state.world.location.is_indoors() { 6 } else { 0 } + 94;
        } else {
            clocks += 40 + if p.is_cape_active() { 46 } else { 6 } + 386;
        }
        clocks
    }

    pub(super) fn handle_link_from1_d(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_8130));
        if self.lag_free { crate::cycle_ledger::charge(self.handle_link_from1d_self_master()); }
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_position_mode();
        self.follower_link_state_mut().clear_action_scratch_state();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut()
            .clear_state_item_and_grab_flags();
        self.follower_link_state_mut().clear_defense_flags();
        self.link_reset_swimming_state_marked();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_z_high();
        if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
            if self.game_state.player.follower_link.is_cape_active() {
                self.link_force_unequip_cape_quietly();
            }
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(2);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_direction_flags(0x0f);
            self.ancilla_sfx3_near(43);
            self.follower_link_state_mut().set_handler_state(7);
            self.link_state_zapped();
        } else {
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            self.follower_link_state_mut().set_handler_state(2);
            self.link_state_recoil();
        }
    }

    pub(super) fn link_state_0_f(&mut self) {
        // LinkState_0F is an assert-only unreachable state in the C port.
        panic!("LinkState_0F reached");
    }

    pub(super) fn player_handler_15_hold_item(&mut self) {}

    pub(super) fn link_handle_bunny_transformation(&mut self) -> bool {
        if self.game_state.player.follower_link.temp_bunny_timer() == 0 {
            return false;
        }

        if !self.game_state.player.follower_link.needs_transform_poof() {
            if matches!(
                self.game_state.player.follower_link.handler_state(),
                23 | 28
            ) {
                self.follower_link_state_mut().clear_temp_bunny_timer();
                return false;
            }
            if self
                .game_state
                .player
                .follower_link
                .picking_throw_state_has(2)
            {
                self.follower_link_state_mut().clear_state_bits();
            }
            let preserved_lift_bit = self.game_state.player.follower_link.state_bits() & 0x80;
            self.link_reset_properties_a();
            self.follower_link_state_mut()
                .set_state_bits(preserved_lift_bit);

            for i in 0..5 {
                if matches!(self.ancilla_slot_view(i).ancilla_type(), 0x30 | 0x31) {
                    self.ancilla_slot_view_mut(i).set_ancilla_type(0);
                }
            }
            self.link_cancel_dash();
            self.ancilla_add_cape_poof(0x23, 4);
            self.ancilla_sfx2_near(0x14);
            self.follower_link_state_mut().set_cape_transform_timer(20);
            self.follower_link_state_mut().start_bunny_transform_poof();
        }

        if (self.follower_link_state_mut().tick_cape_transform_timer() as i8).is_negative() {
            self.follower_link_state_mut().set_handler_state(28);
            self.follower_link_state_mut().finish_bunny_transform_poof();
            self.load_gear_palettes_bunny();
        }
        true
    }

    pub(in crate::zelda_rtl) fn link_state_temporary_bunny(&mut self) {
        let timer = self.game_state.player.follower_link.temp_bunny_timer();
        if timer == 0 {
            self.ancilla_add_cape_poof(0x23, 4);
            self.ancilla_sfx2_near(0x15);
            self.follower_link_state_mut().set_cape_transform_timer(32);
            self.follower_link_state_mut().clear_handler_state();
            self.link_reset_properties_c();
            self.follower_link_state_mut().clear_bunny_transform_flags();
            self.load_actual_gear_palettes();
            self.link_state_default();
        } else {
            self.follower_link_state_mut().decrement_temp_bunny_timer();
            self.player_handler_17_bunny();
        }
    }

    pub(in crate::zelda_rtl) fn player_handler_17_bunny(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.follower_link_state_mut().set_pit_correction_timer(0);
        if !self.game_state.player.follower_link.is_in_deep_water() {
            if !self.game_state.player.follower_link.has_auxiliary_state() {
                self.link_temp_bunny_func2();
                return;
            }
            if self.game_state.inventory.items.has_moon_pearl() {
                self.follower_link_state_mut().clear_bunny_mirror();
            }
        }
        self.link_state_bunny_recache();
    }

    pub(super) fn link_temp_bunny_func2(&mut self) {
        if self.game_state.player.follower_link.incapacitated_timer() != 0 {
            self.link_handle_recoil_and_timer(false);
            return;
        }
        self.follower_link_state_mut().set_z(0xffff);
        self.follower_link_state_mut().set_actual_z_velocity(0xff);
        self.follower_link_state_mut().set_recoil_timer(0);
        if self.game_state.player.follower_link.flag_moving() != 0 {
            self.swim_acceleration_mut().set_max_speed(0, 0x0180);
            self.swim_acceleration_mut().set_max_speed(2, 0x0180);
            self.link_handle_swim_movements();
            return;
        }

        self.reset_all_acceleration();
        self.link_handle_y_item_marked();
        let mut dir = (self
            .game_state
            .player
            .follower_link
            .force_move_any_direction() as u8)
            & 0x0f;
        if dir == 0 {
            dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        }
        if dir == 0 {
            self.follower_link_state_mut()
                .clear_movement_velocity_and_direction();
            self.follower_link_state_mut().set_last_direction(0);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().and_defense_flags(!9);
            self.follower_link_state_mut().reset_push_fatigue_timer();
            self.follower_link_state_mut().reset_jump_ledge_timer();
        } else {
            self.follower_link_state_mut().set_direction(dir);
            if dir != self.game_state.player.follower_link.last_direction() {
                self.follower_link_state_mut().set_last_direction(dir);
                self.follower_link_state_mut().clear_movement_subpixels();
                self.follower_link_state_mut()
                    .clear_moving_against_diag_tile();
                self.follower_link_state_mut().clear_defense_flags();
                self.follower_link_state_mut().reset_push_fatigue_timer();
                self.follower_link_state_mut().reset_jump_ledge_timer();
            }
        }
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_state_holding_big_rock(&mut self) {
        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().clear_action_scratch_state();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut().set_z_low(0);
            if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
                self.link_reset_sword_and_item_usage();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(2);
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.follower_link_state_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.follower_link_state_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        self.follower_link_state_mut().set_z(0xffff);
        self.follower_link_state_mut().set_actual_z_velocity(0xff);
        self.follower_link_state_mut().set_recoil_timer(0);
        if self.game_state.player.follower_link.incapacitated_timer() != 0 {
            self.follower_link_state_mut()
                .clear_lift_throw_scratch_state();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }

        self.link_handle_a_press_marked();
        let dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        if dir == 0 {
            self.follower_link_state_mut()
                .clear_movement_velocity_and_direction();
            self.follower_link_state_mut().set_last_direction(0);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().and_defense_flags(!9);
            self.follower_link_state_mut().reset_push_fatigue_timer();
            self.follower_link_state_mut().reset_jump_ledge_timer();
        } else {
            self.follower_link_state_mut().set_direction(dir);
            if dir != self.game_state.player.follower_link.last_direction() {
                self.follower_link_state_mut().set_last_direction(dir);
                self.follower_link_state_mut().clear_movement_subpixels();
                self.follower_link_state_mut()
                    .clear_moving_against_diag_tile();
                self.follower_link_state_mut().clear_defense_flags();
                self.follower_link_state_mut().reset_push_fatigue_timer();
                self.follower_link_state_mut().reset_jump_ledge_timer();
            }
        }
        self.link_handle_moving_animation_full_long_entry();
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(in crate::zelda_rtl) fn handle_somaria_and_graves(&mut self) {
        if let Some(tail)=self.early_player_ancilla_tail.take() {self.finish_native_player_ancilla_scan(tail);return;}

        let _scope = crate::cycle_ledger::routine(0x07_8689);
        crate::cycle_ledger::charge(40); // Indoor gate.
        if self.game_state.world.location.is_outdoors() {
            crate::cycle_ledger::charge(48); // Grave latch gate.
            if self.game_state.player.follower_link.hookshot_grave_latch() {
                crate::cycle_ledger::charge(16); // LDX #4.
                self.finish_grave_scan_below(5);
            } else {
                crate::cycle_ledger::charge(6);
            }
        } else {
            crate::cycle_ledger::charge(6);
        }
        self.finish_somaria_scan();
    }

    fn finish_grave_scan_below(&mut self, slot: usize) {
        for i in (0..slot).rev() {
            crate::cycle_ledger::charge(64);
            if self.ancilla_slot_view(i).ancilla_type() == 0x24 {
                crate::cycle_ledger::charge(62);
                self.gravestone_move(i);
            } else {
                crate::cycle_ledger::charge(6);
            }
            crate::cycle_ledger::charge(30 + if i != 0 { 6 } else { 0 });
        }
    }

    fn finish_somaria_scan(&mut self) {
        crate::cycle_ledger::charge(16); // Somaria scan starts independently at slot 4.
        for i in (0..5).rev() {
            crate::cycle_ledger::charge(64);
            if self.ancilla_slot_view(i).ancilla_type() == 0x2c {
                crate::cycle_ledger::charge(62);
                self.somaria_block_handle_player_interaction(i);
                crate::cycle_ledger::charge(22 + 42); // BRA : RTS after first match.
                return;
            }
            crate::cycle_ledger::charge(6 + 30 + if i != 0 { 6 } else { 0 });
        }
        crate::cycle_ledger::charge(42);

    }

    pub(super) fn finish_native_player_ancilla_scan(&mut self, tail: crate::cycle_models::player_ancilla_return::PlayerAncillaTail) {
        use crate::cycle_models::player_ancilla_return::PlayerAncillaTail;
        match tail {
            PlayerAncillaTail::Returned => {},
            PlayerAncillaTail::Grave(slot) => {
                self.gravestone_move(slot as usize);
                crate::cycle_ledger::charge(30 + if slot!=0 {6} else {0});
                self.finish_grave_scan_below(slot as usize);
                self.finish_somaria_scan();
            }
            PlayerAncillaTail::Somaria(slot) => {
                self.somaria_block_handle_player_interaction(slot as usize);
                crate::cycle_ledger::charge(22 + 42);
            }
        }
    }

    pub(super) fn link_state_on_ice(&mut self) {
        // LinkState_OnIce is an assert-only unreachable state in the C port.
        panic!("LinkState_OnIce reached");
    }

    pub(super) fn link_state_receiving_ether(&mut self) {
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_given_damage();
        let frames = self
            .follower_link_state_mut()
            .decrement_button_b_frames_word();
        if (frames as i16).is_negative() {
            self.follower_link_state_mut().set_button_b_frames_word(0);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
        } else if frames == 0xbf {
            self.follower_link_state_mut().force_hold_sword_up();
        } else if frames == 160 {
            let x = self.game_state.player.follower_link.x();
            let y = self.game_state.player.follower_link.y();
            self.follower_link_state_mut().set_x(0x06b0);
            self.follower_link_state_mut().set_y(0x0037);
            self.ancilla_add_ether_spell(0x18, 0);
            self.follower_link_state_mut().set_x(x);
            self.follower_link_state_mut().set_y(y);
        } else if frames == 0 {
            self.ancilla_add_falling_prize(0x29, 0, 4);
            self.follower_link_state_mut().immobilize();
            self.follower_link_state_mut().clear_menu_block();
        }
    }

    pub(super) fn link_state_receiving_bombos(&mut self) {
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_given_damage();
        let frames = self
            .follower_link_state_mut()
            .decrement_button_b_frames_word();
        if (frames as i16).is_negative() {
            self.follower_link_state_mut().set_button_b_frames_word(0);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
        } else if frames == 223 {
            self.follower_link_state_mut().force_hold_sword_up();
        } else if frames == 160 {
            let x = self.game_state.player.follower_link.x();
            let y = self.game_state.player.follower_link.y();
            self.follower_link_state_mut().set_x(0x0378);
            self.follower_link_state_mut().set_y(0x0eb0);
            self.ancilla_add_bombos_spell(0x19, 0);
            self.follower_link_state_mut().set_x(x);
            self.follower_link_state_mut().set_y(y);
        } else if frames == 0 {
            self.ancilla_add_falling_prize(0x29, 5, 4);
            self.follower_link_state_mut().immobilize();
        }
    }

    pub(in crate::zelda_rtl) fn ether_tablet_start_cutscene(&mut self) {
        self.follower_link_state_mut()
            .set_button_b_frames_word(0x00c0);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().set_handler_state(25);
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.follower_link_state_mut().set_menu_block_flag(1);
    }

    pub(in crate::zelda_rtl) fn bombos_tablet_start_cutscene(&mut self) {
        self.follower_link_state_mut()
            .set_button_b_frames_word(0x00e0);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().set_handler_state(26);
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.set_custom_spell_animation_active();
    }

    pub(super) fn link_state_reading_desert_tablet(&mut self) {
        let button_b_frames = self
            .game_state
            .player
            .follower_link
            .button_b_frames()
            .wrapping_sub(1);
        self.follower_link_state_mut()
            .set_button_b_frames(button_b_frames);
        if self.game_state.player.follower_link.button_b_frames() == 0 {
            self.follower_link_state_mut().clear_handler_state();
            self.link_perform_desert_prayer();
        }
    }

    pub(super) fn player_handler_00_ground_3(&mut self) {
        // $07:812E BRL $81A0: full ground body; measured suffix entries stay unmarked.
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x0781a0);
        self.apply_links_movement_to_camera_called = false;
        self.follower_link_state_mut().set_z(0xffff);
        self.follower_link_state_mut().set_actual_z_velocity(0xff);
        self.follower_link_state_mut().set_recoil_timer(0);

        let tossed = self.link_handle_toss();
        b.block(0x0781ae);
        if !tossed {
            b.block(0x0781b0);
            self.ground_apress_defers_atomic_item_receipt = true;
            self.link_handle_a_press_marked();
            self.ground_apress_defers_atomic_item_receipt = false;
            if matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                        ground_apress_tail: Some(_),
                        ..
                    },
                })
            ) {
                // The chest receipt scheduled by this A-press blocks the ROM
                // iteration inside its decompression; the remainder of this
                // handler runs at the receipt's completion slice with the
                // joypad latch it holds now.
                return;
            }
            self.player_handler_00_ground_3_after_a_press_body(true, true);
        } else {
            b.taken(0x0781ae);
            self.player_handler_00_ground_3_after_a_press_body(false, true);
        }
    }

    /// Everything `HandleLink_From1D` runs after its A-press dispatch. Split
    /// out so a chest receipt's completion slice can resume it after the
    /// decompression the ROM blocks on (see the ground_apress_tail receipt).
    pub(in crate::zelda_rtl) fn player_handler_00_ground_3_after_a_press(&mut self, ran_a_press: bool) {
        self.player_handler_00_ground_3_after_a_press_body(ran_a_press, false);
    }

    fn player_handler_00_ground_3_after_a_press_body(&mut self, ran_a_press: bool, marked: bool) {
        let b = crate::cycle_ledger::blocks(marked && self.lag_free);
        let mut clear_vel_after = false;
        let native_y_tail = self.early_ground_y_item_tail.take();
        let native_sword_cape_tail = self.early_ground_sword_cape_tail.take();
        if let Some(tail) = self.early_ground_movement_tail.take() {
            assert!(ran_a_press && native_y_tail.is_some());
            assert_eq!(native_sword_cape_tail, Some(crate::cycle_models::ground_sword_cape_return::GroundSwordCapeTail::Movement));
            match tail {
                crate::cycle_models::ground_movement_prefix::GroundMovementTail::Recoil => self.link_handle_recoil_and_timer(false),
                crate::cycle_models::ground_movement_prefix::GroundMovementTail::Swimming => self.link_handle_swim_movements(),
                crate::cycle_models::ground_movement_prefix::GroundMovementTail::Collision { clear_vel_after } => {
                    if let Some(tail) = self.early_diagonal_collision_tail.take() {
                        self.finish_native_diagonal_collision(tail);
                        self.finish_ground_movement_after_diagonal(clear_vel_after);
                    } else {
                        self.finish_ground_movement_tail(clear_vel_after);
                    }
                },
            }
            return;
        }
        use crate::cycle_models::ground_sword_cape_return::GroundSwordCapeTail;
        if let Some(tail) = native_sword_cape_tail {
            assert!(native_y_tail.is_some());
            if tail == GroundSwordCapeTail::ClearMovement {
                self.finish_ground_movement_camera_tail();
                return;
            }
        }
        if native_y_tail.is_some() {
            assert!(ran_a_press, "native Y-item continuation requires its A-press caller");
        }
        if ran_a_press {
            b.block(0x0781b3);
            if self.game_state.player.follower_link.has_action_state()
                || self.game_state.player.follower_link.has_grabbing_wall_state() { b.taken(0x0781b9); }
            else {
                b.block(0x0781bb);
                if self.game_state.player.follower_link.has_pull_action_state() { b.taken(0x0781be); }
                else {
                    b.block(0x0781c0);
                    if self.game_state.player.follower_link.handler_state() == 17 { b.taken(0x0781c4); }
                }
            }
        }
        let run_y_item = match native_y_tail {
            Some(crate::cycle_models::ground_y_item_prefix::GroundYItemTail::SkipYItem) => false,
            Some(_) => true,
            None => ran_a_press
                && !self.game_state.player.follower_link.has_action_state()
                && !self.game_state.player.follower_link.has_grabbing_wall_state()
                && !self.game_state.player.follower_link.has_pull_action_state()
                && self.game_state.player.follower_link.handler_state() != 17,
        };
        if run_y_item {
            b.block(0x0781c6);
            use crate::cycle_models::{ground_y_item_prefix::GroundYItemTail, y_item_dispatch::YItemChild};
            match native_y_tail {
                Some(GroundYItemTail::AfterYItem) => {}
                Some(GroundYItemTail::YItemChild(YItemChild::Item(item))) => self.link_dispatch_y_item(item),
                Some(GroundYItemTail::YItemChild(YItemChild::ForceUnequipCape)) => {
                    self.link_force_unequip_cape();
                    self.link_select_and_dispatch_y_item(self.game_state.player.follower_link.current_item_y());
                }
                Some(GroundYItemTail::LampMagic) => self.link_item_lamp_after_native_magic_entry(),
                Some(GroundYItemTail::SkipYItem) => unreachable!(),
                None => self.link_handle_y_item_marked(),
            }
            if self
                .game_state
                .enhanced_features
                .has(PLAYER_HANDLER_00_GROUND_3_FEATURES0_MISC_BUG_FIXES)
                && ((self.game_state.frame.main_module == 14
                    && self.game_state.frame.submodule != 2)
                    || matches!(self.game_state.player.follower_link.handler_state(), 8..=10))
            {
                self.finish_ground_movement_clear_vel_tail();
                return;
            }
            b.block(0x0781c9);
            if self.game_state.inventory.save_progress.progress_indicator() == 0 { b.taken(0x0781cd); }
            if self.game_state.inventory.save_progress.progress_indicator() != 0
                && !matches!(native_sword_cape_tail, Some(GroundSwordCapeTail::Movement | GroundSwordCapeTail::ForceUnequipCape)) {
                b.block(0x0781cf);
                match native_sword_cape_tail {
                    Some(GroundSwordCapeTail::CheckSwordSwing | GroundSwordCapeTail::SwordSwingActive) => self.link_check_for_sword_swing(),
                    Some(GroundSwordCapeTail::SwordControls) => self.handle_sword_controls(),
                    // $07:81CF JSR $9AC2; no resumed prefix owns this body.
                    None => self.link_handle_sword_cooldown_marked(),
                    _ => unreachable!(),
                }
                b.block(0x0781d2);
                if self.game_state.player.follower_link.handler_state() == 3 {
                    b.block(0x0781d8);
                    b.block(0x0782d2);
                    b.block(0x0782d8);
                    self.finish_ground_movement_clear_vel_tail();
                    return;
                }
                b.taken(0x0781d6);
            }
        }

        b.block(0x0781df);
        let _ = &mut clear_vel_after;
        match native_sword_cape_tail {
            Some(GroundSwordCapeTail::Movement) => {}
            Some(GroundSwordCapeTail::ForceUnequipCape) => self.link_force_unequip_cape(),
            Some(GroundSwordCapeTail::ClearMovement) => unreachable!(),
            _ => self.link_handle_cape_passive_lift_check(),
        }
        b.block(0x0781e2);
        if self.game_state.player.follower_link.incapacitated_timer() != 0 {
            b.block(0x0781e6);
            if self.game_state.player.follower_link.moving_against_diag_tile() == 0 { b.taken(0x0781e8); }
            else { b.block(0x0781ea); }
            b.block(0x0781ec);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            self.follower_link_state_mut()
                .clear_lift_throw_scratch_state();
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                b.block(0x078206);
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
            else { b.taken(0x078204); }
            b.block(0x07820c);
            self.link_handle_recoil_and_timer(false);
            return;
        }

        b.taken(0x0781e4); b.block(0x07820f);
        if self.game_state.player.follower_link.has_pull_action_state() {
            b.block(0x078214);
            self.follower_link_state_mut().set_direction(0);
            clear_vel_after = true;
        } else {
            b.taken(0x078212); b.block(0x078218);
            let p = &self.game_state.player.follower_link;
            if p.is_transforming() { b.taken(0x07821b); }
            else {
                b.block(0x07821d);
                if p.grabbing_wall() & !2 != 0 { b.taken(0x078222); }
                else {
                    b.block(0x078224);
                    if p.state_bits() & 0x7f != 0 { b.taken(0x078229); }
                    else {
                        b.block(0x07822b);
                        let mut skip = false;
                        if p.state_bits() & 0x80 == 0 { b.taken(0x078230); }
                        else {
                            b.block(0x078232);
                            if p.picking_throw_state() & 1 != 0 { b.taken(0x078237); skip = true; }
                        }
                        if !skip {
                            b.block(0x078239);
                            if p.item_in_hand() != 0 { b.taken(0x07823c); }
                            else {
                                b.block(0x07823e);
                                if p.position_mode() != 0 { b.taken(0x078241); }
                                else {
                                    b.block(0x078243);
                                    if p.button_b_frames() >= 9 { b.taken(0x078247); }
                                    else {
                                        b.block(0x078249);
                                        if p.button_mask_b_y() & 0x20 != 0 { b.taken(0x07824d); }
                                        else {
                                            b.block(0x07824f);
                                            if p.button_mask_b_y() & 0x80 == 0 { b.taken(0x078253); }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if !self.game_state.player.follower_link.has_pull_action_state()
            && !self.game_state.player.follower_link.is_transforming()
            && self
                .game_state
                .player
                .follower_link
                .is_ready_to_start_ground_movement()
            && (self.game_state.player.follower_link.button_b_frames() >= 9
                || (self.game_state.player.follower_link.button_mask_b_y() & 0x20) != 0
                || (self.game_state.player.follower_link.button_mask_b_y() & 0x80) == 0)
        {
            b.block(0x078257);
            if self.game_state.player.follower_link.flag_moving() != 0 {
                b.block(0x07825c);
                self.swim_acceleration_mut().set_max_speed(0, 0x0180);
                self.swim_acceleration_mut().set_max_speed(2, 0x0180);
                self.link_handle_swim_movements();
                return;
            }

            b.taken(0x07825a); b.block(0x07826f);
            self.reset_all_acceleration();
            b.block(0x078272);
            let mut dir = (self
                .game_state
                .player
                .follower_link
                .force_move_any_direction() as u8)
                & 0x0f;
            if dir == 0 {
                b.block(0x078278);
                if self.game_state.player.follower_link.grabbing_wall_has(2) {
                    b.taken(0x07827d);
                    self.finish_ground_movement_tail_marked(clear_vel_after, marked);
                    return;
                }
                b.block(0x07827f);
                dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                if dir != 0 { b.taken(0x078283); }
            }
            else { b.taken(0x078276); }
            if dir == 0 {
                b.block(0x078285);
                self.follower_link_state_mut()
                    .clear_movement_velocity_and_direction();
                self.follower_link_state_mut().set_last_direction(0);
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().and_defense_flags(!0x0f);
                self.follower_link_state_mut().reset_push_fatigue_timer();
                self.follower_link_state_mut().reset_jump_ledge_timer();
            } else {
                b.block(0x0782a1);
                self.follower_link_state_mut().set_direction(dir);
                if dir != self.game_state.player.follower_link.last_direction() {
                    b.block(0x0782a7);
                    self.follower_link_state_mut().set_last_direction(dir);
                    self.follower_link_state_mut().clear_movement_subpixels();
                    self.follower_link_state_mut()
                        .clear_moving_against_diag_tile();
                    self.follower_link_state_mut().clear_defense_flags();
                    self.follower_link_state_mut().reset_push_fatigue_timer();
                    self.follower_link_state_mut().reset_jump_ledge_timer();
                } else { b.taken(0x0782a5); }
                b.block(0x0782b9);
            }
        } else { b.block(0x078255); }

        self.finish_ground_movement_tail_marked(clear_vel_after, marked);
    }

    pub(in crate::zelda_rtl) fn handle_indoor_camera_and_doors(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e8f0));
        if self.lag_free {
            // $07:E8F0..E900: tail JML or camera JSL; child bodies excluded.
            let clocks = if self.game_state.world.location.is_outdoors() { 40 + 6 + 42 }
                else if self.game_state.player.follower_link.doorway_state() != 0 { 40 + 40 + 32 }
                else { 40 + 40 + 6 + 62 + 42 };
            crate::cycle_ledger::charge(clocks);
        }
        if self.game_state.world.location.is_outdoors() {
            return;
        }
        if self.game_state.player.follower_link.doorway_state() != 0 {
            self.handle_door_transitions();
        } else {
            self.apply_links_movement_to_camera();
        }
    }

    pub(in crate::zelda_rtl) fn cache_camera_properties_if_outdoors(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_f514));
        if self.lag_free {
            // $07:F514..F51C: branch, optional JSL and RTS; FF28 excluded.
            crate::cycle_ledger::charge(40 + if self.game_state.world.location.is_outdoors() { 62 } else { 6 } + 42);
        }
        if self.game_state.world.location.is_outdoors() {
            self.cache_camera_properties();
        }
    }

    pub(in crate::zelda_rtl) fn handle_door_transitions(&mut self) {
        self.follower_link_state_mut().clear_page_movement_deltas();

        if self
            .game_state
            .enhanced_features
            .has(HANDLE_DOOR_TRANSITIONS_FEATURES0_MISC_BUG_FIXES)
            && !(self.game_state.frame.main_module == 7 && self.game_state.frame.submodule == 0)
        {
            return;
        }

        if self.game_state.player.follower_link.last_direction() & 0x0c != 0
            && self.game_state.player.follower_link.doorway_state() == 1
        {
            if self.game_state.player.follower_link.last_direction() & 4 != 0 {
                let t = self.game_state.player.follower_link.y().wrapping_add(28);
                if t & 0x00fc == 0 {
                    self.follower_link_state_mut()
                        .set_y_page_movement_delta_from_high_position((t >> 8) as u8);
                }
            } else {
                let t = self.game_state.player.follower_link.y().wrapping_sub(18);
                self.follower_link_state_mut()
                    .set_y_page_movement_delta_from_high_position((t >> 8) as u8);
            }
        }

        if self.game_state.player.follower_link.last_direction() & 3 != 0
            && self.game_state.player.follower_link.doorway_state() == 2
        {
            if self.game_state.player.follower_link.last_direction() & 1 != 0 {
                let t = self.game_state.player.follower_link.x().wrapping_add(21);
                if t & 0x00fc == 0 {
                    self.follower_link_state_mut()
                        .set_x_page_movement_delta_from_high_position((t >> 8) as u8);
                }
            } else {
                let t = self.game_state.player.follower_link.x().wrapping_sub(8);
                self.follower_link_state_mut()
                    .set_x_page_movement_delta_from_high_position((t >> 8) as u8);
            }
        }

        if self.game_state.player.follower_link.x_page_movement_delta() != 0 {
            self.follower_link_state_mut().set_y_button_action_timer(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            if self
                .game_state
                .player
                .follower_link
                .x_page_movement_delta_signed()
                .is_negative()
            {
                self.dung_start_inter_room_trans_left_plus();
            } else {
                self.handle_edge_transition_movement_east_right_by8();
            }
        } else if self.game_state.player.follower_link.y_page_movement_delta() != 0 {
            self.follower_link_state_mut().set_y_button_action_timer(0);
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            if self
                .game_state
                .player
                .follower_link
                .y_page_movement_delta_signed()
                .is_negative()
            {
                self.dungeon_start_inter_room_trans_up();
            } else {
                self.handle_edge_transition_movement_south_down_by16();
            }
        }
    }

    pub(in crate::zelda_rtl) fn apply_links_movement_to_camera(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_e9d3));
        if self.lag_free {
            // $07:E9D3..E9E2: bank setup and page-delta stores only.
            crate::cycle_ledger::charge(244);
        }
        self.apply_links_movement_to_camera_called = true;
        let (y_delta, x_delta) = {
            (
                self.game_state
                    .player
                    .follower_link
                    .y_high()
                    .wrapping_sub(self.game_state.player.follower_link.safe_return_y_high()),
                self.game_state
                    .player
                    .follower_link
                    .x_high()
                    .wrapping_sub(self.game_state.player.follower_link.safe_return_x_high()),
            )
        };
        self.follower_link_state_mut()
            .set_page_movement_deltas(y_delta, x_delta);

        self.finish_camera_after_native_page_deltas();
    }

    pub(in crate::zelda_rtl) fn finish_camera_after_native_page_deltas(&mut self) {
        if self.lag_free {
            // $07:E9E4..EA05: X/Y branch/call instructions and bank return.
            let p = &self.game_state.player.follower_link;
            let x = p.x_page_movement_delta_signed();
            let y = p.y_page_movement_delta_signed();
            let clocks = 40 + if x == 0 { 6 } else if x < 0 { 16 + 6 + 62 } else { 16 + 84 }
                + 40 + if y == 0 { 6 + 72 } else if y < 0 { 16 + 134 } else { 16 + 6 + 62 + 72 };
            crate::cycle_ledger::charge_in_routine(0x07_e9d3, clocks);
        }
        self.apply_links_movement_to_camera_called = true;
        if self.game_state.player.follower_link.x_page_movement_delta() != 0 {
            if self
                .game_state
                .player
                .follower_link
                .x_page_movement_delta_signed()
                .is_negative()
            {
                self.adjust_quadrant_and_camera_left();
            } else {
                self.adjust_quadrant_and_camera_right();
            }
        }
        if self.game_state.player.follower_link.y_page_movement_delta() != 0 {
            if self
                .game_state
                .player
                .follower_link
                .y_page_movement_delta_signed()
                .is_negative()
            {
                self.adjust_quadrant_and_camera_up();
            } else {
                self.adjust_quadrant_and_camera_down();
            }
        }
    }
}
