//! Per-store publication for the retained properties caller. This never
//! imports a caller's whole RAM image or reconstructs its CPU clock.
use super::*;
use crate::game_state::constants::{NUM_MEMORIZED_TILES, OVERWORLD_AREA_INDEX};
use crate::cycle_models::pre_overworld_properties::PropertiesChild;
impl ZeldaState {
    pub(super) fn native_properties_data_byte(&self, address: u32,
        sprites: &std::collections::BTreeMap<u32, u8>,
        songs: &std::collections::BTreeMap<u32, u8>) -> Option<u8> {
        let address = address & 0x7fffff;
        crate::cycle_models::overworld_properties_caller::table_byte(address)
            .or_else(|| self.native_overworld_exit_table_byte(address))
            .or_else(|| crate::cycle_models::hud_update::table_byte(address))
            .or_else(|| Self::native_sprite_property_table_byte(address))
            .or_else(|| self.native_graphics_asset_byte(address))
            .or_else(|| load_gfx::native_animated_item_table_byte(address))
            .or_else(|| self.native_overworld_palette_byte(address))
            .or_else(|| Self::native_overworld_palette_selection_byte(address))
            .or_else(|| crate::cycle_models::load_file_player::data_byte(address))
            .or_else(|| sprites.get(&address).copied())
            .or_else(|| songs.get(&address).copied())
    }

    pub(super) fn native_overworld_properties_store(
        &mut self,
        child: Option<PropertiesChild>,
        address: u32,
        value: u16,
        width: u8,
    ) -> Result<(), usize> {
        let address = if address & 0x400000 == 0 && address & 65535 < 0x2000 {
            address & 65535
        } else {
            address
        };
        let offset = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else {
            address as usize
        };
        assert!(matches!(width, 1 | 2));
        // PPU registers are hardware, not the identically numbered WRAM
        // offsets. The production device bus retires their clock separately.
        if address < 0x400000 && (0x2100..=0x213f).contains(&(address & 65535)) {
            assert_eq!(width, 1);
            self.ppu.write(address as u8, value as u8);
            return Ok(());
        }
        self.ram[offset..offset + usize::from(width)]
            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        if (0xc300..0xc700).contains(&offset) {
            assert_eq!(width, 2);
            assert_eq!(offset & 1, 0);
            let index = (offset - if offset < 0xc500 { 0xc300 } else { 0xc500 }) / 2;
            if child == Some(PropertiesChild::FixedColorAndScroll) {
                if offset < 0xc500 {
                    self.set_aux_color_constant(index, value);
                } else {
                    self.set_main_color_constant(index, value);
                }
            } else {
                if offset < 0xc500 {
                    self.set_aux_color_asset(index, value);
                } else {
                    self.set_main_color_asset(index, value);
                }
            }
        } else if self.native_load_hud_store(address, value, width) {
        } else if child == Some(PropertiesChild::DungeonExit)
            && offset == 0x40c {
            self.save_progress_mut().set_palace_index_x2(value as u8);
            // The original word store also retires the unmodeled companion byte.
        } else if child == Some(PropertiesChild::DungeonExit)
            && (width == 1 || offset & 1 == 0)
            && self.native_dungeon_entrance_store(address, value, width)
        {
        } else if child == Some(PropertiesChild::Follower)
            && self.native_follower_initialization_store(address, value, width)
        {
        } else if width == 1 && self.native_load_player_initialization_store(offset, value, width) {
        } else if matches!(
            child,
            Some(PropertiesChild::SpriteReload | PropertiesChild::MirrorPortal)
        ) && self.native_sprite_main_store(offset as u32, value, width)
        {
        } else if child == Some(PropertiesChild::SpriteReload)
            && width == 1
            && value == 0
            && self.native_sprite_disable_store(address, value, width)
        {
        } else if child == Some(PropertiesChild::SpriteReload)
            && width == 1
            && self.native_sprite_reset_suffix_store(address, value, width)
        {
        } else if matches!(offset, 0x6000..=0x8fff | 0xa680..=0xb27f | 0xbd40..=0xbdbf | 0x14000..=0x14bff)
        {
            // Canonical graphics staging RAM. These buffers have no duplicate
            // scalar owner; native decompression/conversion reads them live.
        } else {
            for byte in 0..usize::from(width) {
                let at = offset + byte;
                let v = (value >> (byte * 8)) as u8;
                if matches!(at, 0..=0xf | 0x1d0..=0x1ff) {
                    continue;
                }
                if (child == Some(PropertiesChild::Tilesets) && (0xbd..=0xce).contains(&at))
                    || (child == Some(PropertiesChild::AnimatedTiles) && (0xc8..=0xce).contains(&at))
                    || (child == Some(PropertiesChild::PlayerGraphics) && (0xbd..=0xce).contains(&at))
                {
                    // The graphics routines keep their live decode/conversion
                    // scratch in RAM. Other modules reuse these addresses.
                    continue;
                }
                if matches!((child, at),
                    (Some(PropertiesChild::PaletteCache), 0xc00c)
                    | (Some(PropertiesChild::DungeonExit), 0xc018))
                {
                    // Companion bytes of original word stores to the byte
                    // fields mosaic_target_level and fixed_color_plusminus.
                    continue;
                }
                if self.native_load_player_initialization_store(at, u16::from(v), 1) {
                    continue;
                }
                if self.game_state.world.write_entrance_byte(at, v)
                    | self.game_state.world.write_properties_byte(at, v)
                    | self
                        .game_state
                        .display
                        .ppu_scroll_copy
                        .write_entrance_byte(at, v)
                    | self.game_state.dungeon.write_entrance_byte(at, v)
                    | self.game_state.dungeon.write_header_byte(at, v)
                    | self.game_state.display.write_load_parent_byte(at, v)
                    | self.game_state.display.write_properties_byte(at, v)
                    | self.game_state.dungeon.write_load_parent_byte(at, v)
                    | self
                        .game_state
                        .player
                        .follower_link
                        .write_load_parent_byte(at, v)
                    | self.game_state.player.follower_link.write_properties_byte(at, v)
                    | self.game_state.sprites.write_record_byte(at, v)
                    | self.game_state.sprites.write_disable_byte(at, v)
                    | self.game_state.sprites.garnish_runtime.write_collision_byte(at, v)
                    | self
                        .game_state
                        .player
                        .follower_link
                        .write_entrance_byte(at, v)
                    | self
                        .game_state
                        .player
                        .follower_link
                        .write_interactive_cleanup_byte(at, v)
                    | self
                        .game_state
                        .player
                        .follower_link
                        .write_sprite_disable_byte(at, v)
                    | self.native_load_file_parent_store(at as u32, u16::from(v), 1)
                {
                    continue;
                }
                match at {
                    0x2f2 => self.game_state.sprites.follower_runtime.set_event_flags(v),
                    0x35f => self.game_state.minigame.set_flag_boomerang_in_place(v),
                    0x3fc => self.game_state.minigame.set_is_archer_or_shovel_game(v),
                    0x4ac..=0x4ad => self.game_state.memorized_tiles.set_count(read_le_u16(&self.ram, NUM_MEMORIZED_TILES)),
                    0xaa3 => self.game_state.sprites.system.set_graphics_index(v),
                    0x10 => self.set_main_module(v),
                    0x11 => self.set_submodule(v),
                    0x15 => {
                        self.game_state.system_signals.set_cgram_update_flag(v);
                    }
                    0x16 => self.system_signals_mut().set_hud_update_flag(v),
                    0x30 => self.follower_link_state_mut().set_y_velocity(v),
                    0x20..=0x23 => {
                        let base = at & !1;
                        let old = read_le_u16(&self.ram, base);
                        let shift = (at & 1) * 8;
                        let word = (old & !(255 << shift)) | (u16::from(v) << shift);
                        if base == 0x20 {
                            self.follower_link_state_mut().set_y(word);
                        } else {
                            self.follower_link_state_mut().set_x(word);
                        }
                    }
                    0x10c => self.set_saved_module_for_menu(v),
                    0x132 => self.set_queued_music_control(v),
                    0x136 if child == Some(PropertiesChild::SongBankTail) => {
                        assert_eq!(v, 0);
                        self.game_state.system_signals.select_overworld_song_bank();
                    }
                    0xc2f8..=0xc2fb => self
                        .world_palette_theme_mut()
                        .set_aux_bg_subset(at - 0xc2f8, v),
                    0xc2fc..=0xc2ff => self
                        .sprite_workspace_mut()
                        .set_graphics_subset(at - 0xc2fc, v),
                    0x3f4 => {
                        assert_eq!(v, 0);
                        self.dungeon_environment_mut()
                            .clear_somaria_block_bg_check_flag();
                    }
                    0x40a..=0x40b => {
                        let old = read_le_u16(&self.ram, OVERWORLD_AREA_INDEX);
                        let shift = (at & 1) * 8;
                        self.set_overworld_area_index_word(
                            (old & !(255 << shift)) | (u16::from(v) << shift),
                        );
                    }
                    _ => return Err(at),
                }
            }
        }
        self.ram[offset..offset + usize::from(width)]
            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        Ok(())
    }
}
