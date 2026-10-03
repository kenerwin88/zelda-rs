//! Native cycle models of original routines: the master-cycle cost the ROM
//! spends in a routine, computed from the same inputs the routine consumes,
//! without executing ROM code. Each model is priced instruction by
//! instruction from the routine's disassembly and verified against the
//! shadow CPU (`rom_cpu_timing`) over every input the cartridge holds, so a
//! ROM-less build can schedule exactly what a ROM-loaded build measures.
//! See docs/parity/romless-exact-play.md.

pub(crate) mod cucco;
pub(crate) mod decompress;
pub(crate) mod graphics_convert;
pub(crate) mod graphics_load;
pub(crate) mod main_wait;
pub(crate) mod nmi_stack;
pub(crate) mod nmi_updates;
pub(crate) mod nmi_stripes;
pub(crate) mod main_entry;
pub(crate) mod intro_entry;
pub(crate) mod intro_clear;
pub(crate) mod half_slot;
pub(crate) mod oam_pack;
pub(crate) mod sprite_dma;
pub(crate) mod sprite_kill;
pub(crate) mod sprite_prep;
pub(crate) mod sprite_shadow;
pub(crate) mod sprite_single;
pub(crate) mod startup_memory;
pub(crate) mod startup;
pub(crate) mod startup_bus;
pub(crate) mod text_buffer;
pub(crate) mod text_decode;
pub(crate) mod text_initialize;
pub(crate) mod text_pointers;
pub(crate) mod intro_palettes;
pub(crate) mod follower_graphics;
pub(crate) mod default_graphics;
pub(crate) mod native_tilemap_erase;
#[cfg(test)]
mod native_bus_tests;
pub(crate) mod animated_dungeon_graphics;
pub(crate) mod ppu_plane_conversion;
pub(crate) mod native_tileset_sheet;
pub(crate) mod common_sprite_graphics;
pub(crate) mod native_tilesets;
pub(crate) mod intro_darkening;
pub(crate) mod graphics_sources;
pub(crate) mod native_work;
pub(crate) mod item_graphics;
pub(crate) mod item_graphics_caller;
pub(crate) mod native_decompress;
pub(crate) mod tilesets;
pub(crate) mod tilemap_erase;
pub(crate) mod vwf;
pub(crate) mod vwf_drawing;

pub(crate) mod poly_interrupt;
pub(crate) mod poly_buffer_clear;
pub(crate) mod poly_shape;
pub(crate) mod poly_rotation;
pub(crate) mod poly_vertices;
pub(crate) mod poly_faces;
pub(crate) mod poly_raster;
pub(crate) mod poly_worker;

pub(crate) mod intro_poly_init;
pub(crate) mod triforce_animation;
pub(crate) mod intro_sprite_init;
pub(crate) mod intro_sprite_work;
pub(crate) mod intro_animation;
pub(crate) mod intro_palette_fade;
pub(crate) mod intro_sword;
pub(crate) mod intro_title;
pub(crate) mod file_select_init;
pub(crate) mod file_select_graphics;
pub(crate) mod file_select_erase;
pub(crate) mod file_select_palettes;
pub(crate) mod file_select_stripes;
pub(crate) mod file_select_draw;
pub(crate) mod file_select_main;
pub(crate) mod kill_file;
pub(crate) mod name_file_setup;
pub(crate) mod name_file_active;
pub(crate) mod load_file_common;
pub(crate) mod load_file;
pub(crate) mod load_file_hud;
pub(crate) mod load_file_player;
pub(crate) mod song_bank;
pub(crate) mod dungeon_entrance;
pub(crate) mod dungeon_header;
pub(crate) mod dungeon_floors;
pub(crate) mod dungeon_objects;
pub(crate) mod dungeon_moving_wall;
pub(crate) mod dungeon_water;
pub(crate) mod dungeon_saved_objects;
pub(crate) mod dungeon_room_decor;
pub(crate) mod dungeon_stairs;
pub(crate) mod dungeon_door_state;
pub(crate) mod dungeon_doors;
pub(crate) mod dungeon_room;
pub(crate) mod dungeon_upload;
pub(crate) mod dungeon_attributes;
pub(crate) mod dungeon_object_attributes;
pub(crate) mod dungeon_door_attributes;
pub(crate) mod dungeon_palettes;
pub(crate) mod dungeon_visited;
pub(crate) mod sprite_reset;
pub(crate) mod sprite_reset_data;
pub(crate) mod dungeon_data;
pub(crate) mod load_file_state;
pub(crate) mod gear_graphics;
pub(crate) mod enemy_damage;
pub(crate) mod sram_validate;
