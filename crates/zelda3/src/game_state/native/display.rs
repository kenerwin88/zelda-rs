use super::ram_byte;
use crate::game_state::constants::messaging::{
    MESSAGE_DMA_DST_ADDR, MESSAGE_DMA_TILE_BASE, MESSAGE_DMA_TILE_LIMIT, MESSAGE_DMA_TILE_SENTINEL,
};
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LinkDmaSourceSlot {
    BodyTop,
    BodyBottom,
    HeadTop,
    HeadBottom,
    HandLeft,
    HandRight,
    SwordUpper,
    SwordLower,
    ShieldUpper,
    ShieldLower,
    AuxUpper,
    AuxLower,
    PushUpper,
    PushLower,
    AnimatedTileUpper,
    AnimatedTileLower,
    HeadPointerUpper,
    HeadPointerLower,
    BodyPointerUpper,
    BodyPointerLower,
    TravelBirdUpper,
    TravelBirdLower,
}

impl LinkDmaSourceSlot {
    fn index(self) -> usize {
        match self {
            Self::BodyTop => 0,
            Self::BodyBottom => 1,
            Self::HeadTop => 2,
            Self::HeadBottom => 3,
            Self::HandLeft => 4,
            Self::HandRight => 5,
            Self::SwordUpper => 6,
            Self::SwordLower => 7,
            Self::ShieldUpper => 8,
            Self::ShieldLower => 9,
            Self::AuxUpper => 10,
            Self::AuxLower => 11,
            Self::PushUpper => 12,
            Self::PushLower => 13,
            Self::AnimatedTileUpper => 14,
            Self::AnimatedTileLower => 15,
            Self::HeadPointerUpper => 16,
            Self::HeadPointerLower => 17,
            Self::BodyPointerUpper => 18,
            Self::BodyPointerLower => 19,
            Self::TravelBirdUpper => 20,
            Self::TravelBirdLower => 21,
        }
    }

    fn address(self) -> usize {
        match self {
            Self::BodyTop => DMA_SOURCE_ADDR_3,
            Self::BodyBottom => DMA_SOURCE_ADDR_0,
            Self::HeadTop => DMA_SOURCE_ADDR_4,
            Self::HeadBottom => DMA_SOURCE_ADDR_1,
            Self::HandLeft => DMA_SOURCE_ADDR_5,
            Self::HandRight => DMA_SOURCE_ADDR_2,
            Self::SwordUpper => DMA_SOURCE_ADDR_6,
            Self::SwordLower => DMA_SOURCE_ADDR_11,
            Self::ShieldUpper => DMA_SOURCE_ADDR_7,
            Self::ShieldLower => DMA_SOURCE_ADDR_12,
            Self::AuxUpper => DMA_SOURCE_ADDR_8,
            Self::AuxLower => DMA_SOURCE_ADDR_13,
            Self::PushUpper => DMA_SOURCE_ADDR_10,
            Self::PushLower => DMA_SOURCE_ADDR_15,
            Self::AnimatedTileUpper => DMA_SOURCE_ADDR_9,
            Self::AnimatedTileLower => DMA_SOURCE_ADDR_14,
            Self::HeadPointerUpper => DMA_SOURCE_ADDR_16,
            Self::HeadPointerLower => DMA_SOURCE_ADDR_18,
            Self::BodyPointerUpper => DMA_SOURCE_ADDR_17,
            Self::BodyPointerLower => DMA_SOURCE_ADDR_19,
            Self::TravelBirdUpper => DMA_SOURCE_ADDR_20,
            Self::TravelBirdLower => DMA_SOURCE_ADDR_21,
        }
    }
}

const LINK_DMA_SOURCE_SLOTS: [LinkDmaSourceSlot; 22] = [
    LinkDmaSourceSlot::BodyTop,
    LinkDmaSourceSlot::BodyBottom,
    LinkDmaSourceSlot::HeadTop,
    LinkDmaSourceSlot::HeadBottom,
    LinkDmaSourceSlot::HandLeft,
    LinkDmaSourceSlot::HandRight,
    LinkDmaSourceSlot::SwordUpper,
    LinkDmaSourceSlot::SwordLower,
    LinkDmaSourceSlot::ShieldUpper,
    LinkDmaSourceSlot::ShieldLower,
    LinkDmaSourceSlot::AuxUpper,
    LinkDmaSourceSlot::AuxLower,
    LinkDmaSourceSlot::PushUpper,
    LinkDmaSourceSlot::PushLower,
    LinkDmaSourceSlot::AnimatedTileUpper,
    LinkDmaSourceSlot::AnimatedTileLower,
    LinkDmaSourceSlot::HeadPointerUpper,
    LinkDmaSourceSlot::HeadPointerLower,
    LinkDmaSourceSlot::BodyPointerUpper,
    LinkDmaSourceSlot::BodyPointerLower,
    LinkDmaSourceSlot::TravelBirdUpper,
    LinkDmaSourceSlot::TravelBirdLower,
];

const HUD_INVENTORY_ORDER_CAPACITY: usize = 24;
const PALETTE_BANK_BYTES: usize = 512;
const PALETTE_VISIBLE_BYTES: usize = 256;
const VISIBLE_SUBPALETTE_CLEAR_START: usize = 32 * 2;
const VISIBLE_SUBPALETTE_CLEAR_LEN: usize = 192;
const SPRITE_SUBPALETTE_CLEAR_START: usize = 0x180;
const SPRITE_SUBPALETTE_CLEAR_LEN: usize = 0x80;
const HUD_TILEMAP_BYTES: usize = MOVING_WALL_REPLACEMENT_BUFFER - HUD_TILE_INDICES_BUFFER;
const SPOTLIGHT_HDMA_WORD_COUNT: usize = 0xf0;

pub(crate) struct GraphicsDecompressionScratch;

impl GraphicsDecompressionScratch {
    pub(crate) fn primary_buffer_offset() -> usize {
        PRIMARY_DECOMP_BUFFER_LOAD_GFX
    }

    pub(crate) fn secondary_buffer_offset() -> usize {
        SECONDARY_DECOMP_BUFFER_LOAD_GFX
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PaletteFilterState {
    countdown: u8,
    countdown_high: u8,
    darkening_or_lightening_screen: u8,
    darkening_or_lightening_screen_high: u8,
    color_window_selection: u8,
    color_math_control: u8,
    color_math_control_high: u8,
    fixed_color_red: u8,
    fixed_color_green: u8,
    fixed_color_blue: u8,
}

impl PaletteFilterState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            countdown: ram_byte(ram, PALETTE_FILTER_COUNTDOWN),
            countdown_high: ram_byte(ram, PALETTE_FILTER_COUNTDOWN + 1),
            darkening_or_lightening_screen: ram_byte(ram, DARKENING_OR_LIGHTENING_SCREEN),
            darkening_or_lightening_screen_high: ram_byte(ram, DARKENING_OR_LIGHTENING_SCREEN + 1),
            color_window_selection: ram_byte(ram, CGWSEL_COPY),
            color_math_control: ram_byte(ram, CGADSUB_COPY),
            color_math_control_high: ram_byte(ram, CGADSUB_COPY + 1),
            fixed_color_red: ram_byte(ram, COLDATA_COPY0),
            fixed_color_green: ram_byte(ram, COLDATA_COPY1),
            fixed_color_blue: ram_byte(ram, COLDATA_COPY2),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[PALETTE_FILTER_COUNTDOWN] = self.countdown;
        ram[PALETTE_FILTER_COUNTDOWN + 1] = self.countdown_high;
        ram[DARKENING_OR_LIGHTENING_SCREEN] = self.darkening_or_lightening_screen;
        ram[DARKENING_OR_LIGHTENING_SCREEN + 1] = self.darkening_or_lightening_screen_high;
        ram[CGWSEL_COPY] = self.color_window_selection;
        ram[CGADSUB_COPY] = self.color_math_control;
        ram[COLDATA_COPY0] = self.fixed_color_red;
        ram[COLDATA_COPY1] = self.fixed_color_green;
        ram[COLDATA_COPY2] = self.fixed_color_blue;
    }

    pub(crate) fn countdown(&self) -> u8 {
        self.countdown
    }

    pub(crate) fn countdown_word(&self) -> u16 {
        u16::from(self.countdown) | (u16::from(self.countdown_high) << 8)
    }

    pub(crate) fn darkening_or_lightening_screen(&self) -> u8 {
        self.darkening_or_lightening_screen
    }

    pub(crate) fn darkening_or_lightening_screen_word(&self) -> u16 {
        u16::from(self.darkening_or_lightening_screen)
            | (u16::from(self.darkening_or_lightening_screen_high) << 8)
    }

    pub(crate) fn color_window_selection(&self) -> u8 {
        self.color_window_selection
    }

    pub(crate) fn color_window_and_math_word(&self) -> u16 {
        u16::from(self.color_window_selection) | (u16::from(self.color_math_control) << 8)
    }

    pub(crate) fn color_math_control(&self) -> u8 {
        self.color_math_control
    }

    pub(crate) fn color_math_control_word(&self) -> u16 {
        u16::from(self.color_math_control) | (u16::from(self.color_math_control_high) << 8)
    }

    pub(crate) fn fixed_color_red(&self) -> u8 {
        self.fixed_color_red
    }

    pub(crate) fn fixed_color_green(&self) -> u8 {
        self.fixed_color_green
    }

    pub(crate) fn fixed_color_blue(&self) -> u8 {
        self.fixed_color_blue
    }

    pub(crate) fn fixed_color_component(&self, index: usize) -> u8 {
        match index {
            0 => self.fixed_color_red,
            1 => self.fixed_color_green,
            2 => self.fixed_color_blue,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct HudInventoryOrderState {
    order: [u8; HUD_INVENTORY_ORDER_CAPACITY],
}

impl HudInventoryOrderState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut order = [0; HUD_INVENTORY_ORDER_CAPACITY];
        for (index, value) in order.iter_mut().enumerate() {
            *value = ram_byte(ram, HUD_INVENTORY_ORDER + index);
        }
        Self { order }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (index, value) in self.order.iter().copied().enumerate() {
            ram[HUD_INVENTORY_ORDER + index] = value;
        }
    }

    pub(crate) fn is_custom(&self) -> bool {
        self.order[0] != 0
    }

    pub(crate) fn item(&self, index: usize) -> u8 {
        self.order.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn initialize_default_order(&mut self, count: usize) {
        for index in 0..count.min(HUD_INVENTORY_ORDER_CAPACITY) {
            self.order[index] = index as u8 + 1;
        }
    }

    pub(crate) fn swap_items(&mut self, old_pos: usize, new_pos: usize) {
        if old_pos < HUD_INVENTORY_ORDER_CAPACITY && new_pos < HUD_INVENTORY_ORDER_CAPACITY {
            self.order.swap(old_pos, new_pos);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct HudRuntimeState {
    super_bomb_indicator_timer: u8,
    super_bomb_indicator_counter: u8,
    rupee_sfx_sound_delay: u8,
    heart_animation_active: u8,
    heart_refill_countdown: u8,
    heart_refill_animation_subpixel: u8,
    flashing_circle_timer: u8,
    previous_menu_joypad_h: u8,
    equipment_menu_exit_state: u8,
    bottle_menu_row: u8,
    dungeon_dark_with_lantern: u8,
    module_tick_counter: u8,
}

impl HudRuntimeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            super_bomb_indicator_timer: ram_byte(ram, SUPER_BOMB_INDICATOR_TIMER),
            super_bomb_indicator_counter: ram_byte(ram, SUPER_BOMB_INDICATOR_COUNTER),
            rupee_sfx_sound_delay: ram_byte(ram, RUPEE_SFX_SOUND_DELAY),
            heart_animation_active: ram_byte(ram, IS_DOING_HEART_ANIMATION),
            heart_refill_countdown: ram_byte(ram, HEART_REFILL_COUNTDOWN),
            heart_refill_animation_subpixel: ram_byte(ram, HEART_REFILL_ANIM_SUBPOS),
            flashing_circle_timer: ram_byte(ram, FLASHING_CIRCLE_TIMER),
            previous_menu_joypad_h: ram_byte(ram, MENU_PREV_JOYPAD_H),
            equipment_menu_exit_state: ram_byte(ram, EQUIPMENT_MENU_EXIT_STATE),
            bottle_menu_row: ram_byte(ram, BOTTLE_MENU_ROW),
            dungeon_dark_with_lantern: ram_byte(ram, HDR_DUNGEON_DARK_WITH_LANTERN),
            module_tick_counter: ram_byte(ram, HUD_MODULE_TICK_COUNTER),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[SUPER_BOMB_INDICATOR_TIMER] = self.super_bomb_indicator_timer;
        ram[SUPER_BOMB_INDICATOR_COUNTER] = self.super_bomb_indicator_counter;
        ram[RUPEE_SFX_SOUND_DELAY] = self.rupee_sfx_sound_delay;
        ram[IS_DOING_HEART_ANIMATION] = self.heart_animation_active;
        ram[HEART_REFILL_COUNTDOWN] = self.heart_refill_countdown;
        ram[HEART_REFILL_ANIM_SUBPOS] = self.heart_refill_animation_subpixel;
        ram[FLASHING_CIRCLE_TIMER] = self.flashing_circle_timer;
        ram[MENU_PREV_JOYPAD_H] = self.previous_menu_joypad_h;
        ram[EQUIPMENT_MENU_EXIT_STATE] = self.equipment_menu_exit_state;
        ram[BOTTLE_MENU_ROW] = self.bottle_menu_row;
        ram[HDR_DUNGEON_DARK_WITH_LANTERN] = self.dungeon_dark_with_lantern;
        ram[HUD_MODULE_TICK_COUNTER] = self.module_tick_counter;
    }

    pub(crate) fn super_bomb_indicator_timer(&self) -> u8 {
        self.super_bomb_indicator_timer
    }

    pub(crate) fn super_bomb_indicator_counter(&self) -> u8 {
        self.super_bomb_indicator_counter
    }

    pub(crate) fn rupee_sfx_sound_delay(&self) -> u8 {
        self.rupee_sfx_sound_delay
    }

    pub(crate) fn is_doing_heart_animation(&self) -> bool {
        self.heart_animation_active != 0
    }

    pub(crate) fn is_doing_heart_animation_raw(&self) -> u8 {
        self.heart_animation_active
    }

    pub(crate) fn heart_refill_countdown(&self) -> u8 {
        self.heart_refill_countdown
    }

    pub(crate) fn heart_refill_anim_subpos(&self) -> u8 {
        self.heart_refill_animation_subpixel
    }

    pub(crate) fn flashing_circle_timer(&self) -> u8 {
        self.flashing_circle_timer
    }

    pub(crate) fn prev_joypad_h(&self) -> u8 {
        self.previous_menu_joypad_h
    }

    pub(crate) fn equipment_menu_exit_state(&self) -> u8 {
        self.equipment_menu_exit_state
    }

    pub(crate) fn bottle_menu_row(&self) -> u8 {
        self.bottle_menu_row
    }

    pub(crate) fn dungeon_dark_with_lantern(&self) -> bool {
        self.dungeon_dark_with_lantern != 0
    }

    pub(crate) fn tick_counter(&self) -> u8 {
        self.module_tick_counter
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct HudTilemapState {
    floor_changed_timer: u16,
    tile_indices: Vec<u8>,
}

impl Default for HudTilemapState {
    fn default() -> Self {
        Self {
            floor_changed_timer: 0,
            tile_indices: vec![0; HUD_TILEMAP_BYTES],
        }
    }
}

impl HudTilemapState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut tile_indices = vec![0; HUD_TILEMAP_BYTES];
        let available = ram
            .len()
            .saturating_sub(HUD_TILE_INDICES_BUFFER)
            .min(HUD_TILEMAP_BYTES);
        tile_indices[..available]
            .copy_from_slice(&ram[HUD_TILE_INDICES_BUFFER..HUD_TILE_INDICES_BUFFER + available]);
        Self {
            floor_changed_timer: read_le_u16(ram, HUD_FLOOR_CHANGED_TIMER),
            tile_indices,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, HUD_FLOOR_CHANGED_TIMER, self.floor_changed_timer);
        ram[HUD_TILE_INDICES_BUFFER..HUD_TILE_INDICES_BUFFER + self.tile_indices.len()]
            .copy_from_slice(&self.tile_indices);
    }

    pub(crate) fn floor_changed_timer_low(&self) -> u8 {
        self.floor_changed_timer as u8
    }

    pub(crate) fn tile_word(&self, tile: usize) -> u16 {
        let offset = tile * 2;
        if offset + 1 >= self.tile_indices.len() {
            return 0;
        }
        u16::from(self.tile_indices[offset]) | (u16::from(self.tile_indices[offset + 1]) << 8)
    }

    fn set_floor_changed_timer(&mut self, value: u16) {
        self.floor_changed_timer = value;
    }

    fn clear_floor_changed_timer_low(&mut self) {
        self.floor_changed_timer &= 0xff00;
    }

    fn set_tile_word(&mut self, tile: usize, value: u16) {
        let offset = tile * 2;
        if offset + 1 < self.tile_indices.len() {
            self.tile_indices[offset] = value as u8;
            self.tile_indices[offset + 1] = (value >> 8) as u8;
        }
    }
}

pub(crate) struct HudStateRead<'a> {
    runtime: &'a HudRuntimeState,
    tilemap: &'a HudTilemapState,
}

impl<'a> HudStateRead<'a> {
    pub(crate) fn new(runtime: &'a HudRuntimeState, tilemap: &'a HudTilemapState) -> Self {
        Self { runtime, tilemap }
    }

    pub(crate) fn floor_changed_timer_low(&self) -> u8 {
        self.tilemap.floor_changed_timer_low()
    }

    pub(crate) fn super_bomb_indicator_timer(&self) -> u8 {
        self.runtime.super_bomb_indicator_timer()
    }

    pub(crate) fn super_bomb_indicator_counter(&self) -> u8 {
        self.runtime.super_bomb_indicator_counter()
    }

    pub(crate) fn rupee_sfx_sound_delay(&self) -> u8 {
        self.runtime.rupee_sfx_sound_delay()
    }

    pub(crate) fn is_doing_heart_animation(&self) -> bool {
        self.runtime.is_doing_heart_animation()
    }

    pub(crate) fn is_doing_heart_animation_raw(&self) -> u8 {
        self.runtime.is_doing_heart_animation_raw()
    }

    pub(crate) fn heart_refill_countdown(&self) -> u8 {
        self.runtime.heart_refill_countdown()
    }

    pub(crate) fn heart_refill_anim_subpos(&self) -> u8 {
        self.runtime.heart_refill_anim_subpos()
    }

    pub(crate) fn flashing_circle_timer(&self) -> u8 {
        self.runtime.flashing_circle_timer()
    }

    pub(crate) fn prev_joypad_h(&self) -> u8 {
        self.runtime.prev_joypad_h()
    }

    pub(crate) fn equipment_menu_exit_state(&self) -> u8 {
        self.runtime.equipment_menu_exit_state()
    }

    pub(crate) fn bottle_menu_row(&self) -> u8 {
        self.runtime.bottle_menu_row()
    }

    pub(crate) fn dungeon_dark_with_lantern(&self) -> bool {
        self.runtime.dungeon_dark_with_lantern()
    }

    pub(crate) fn tick_counter(&self) -> u8 {
        self.runtime.tick_counter()
    }

    pub(crate) fn tile_word(&self, tile: usize) -> u16 {
        self.tilemap.tile_word(tile)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TrinexxPaletteState {
    pub(crate) red_shell_delay: u8,
    pub(crate) blue_shell_delay: u8,
    pub(crate) red_shell_step: u8,
    pub(crate) blue_shell_step: u8,
}

impl TrinexxPaletteState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            red_shell_delay: ram_byte(ram, TRINEXX_RED_SHELL_PALETTE_DELAY),
            blue_shell_delay: ram_byte(ram, TRINEXX_BLUE_SHELL_PALETTE_DELAY),
            red_shell_step: ram_byte(ram, TRINEXX_RED_SHELL_PALETTE_STEP),
            blue_shell_step: ram_byte(ram, TRINEXX_BLUE_SHELL_PALETTE_STEP),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = self.red_shell_delay;
        ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = self.blue_shell_delay;
        ram[TRINEXX_RED_SHELL_PALETTE_STEP] = self.red_shell_step;
        ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = self.blue_shell_step;
    }

    pub(crate) fn decrement_red_shell_delay(&mut self) {
        self.red_shell_delay = self.red_shell_delay.wrapping_sub(1);
    }

    pub(crate) fn decrement_blue_shell_delay(&mut self) {
        self.blue_shell_delay = self.blue_shell_delay.wrapping_sub(1);
    }

    pub(crate) fn increment_red_shell_step(&mut self) -> u8 {
        self.red_shell_step = self.red_shell_step.wrapping_add(1);
        self.red_shell_step
    }

    pub(crate) fn increment_blue_shell_step(&mut self) -> u8 {
        self.blue_shell_step = self.blue_shell_step.wrapping_add(1);
        self.blue_shell_step
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct LinkDmaSources {
    sources: [u16; LINK_DMA_SOURCE_SLOTS.len()],
}

impl LinkDmaSources {
    fn load_from_ram(ram: &[u8]) -> Self {
        let mut sources = [0; LINK_DMA_SOURCE_SLOTS.len()];
        for slot in LINK_DMA_SOURCE_SLOTS {
            let address = slot.address();
            sources[slot.index()] = if address + 1 < ram.len() {
                read_le_u16(ram, address)
            } else {
                0
            };
        }
        Self { sources }
    }

    fn write_to_ram(&self, ram: &mut [u8]) {
        for slot in LINK_DMA_SOURCE_SLOTS {
            write_le_u16(ram, slot.address(), self.source(slot));
        }
    }

    fn source(&self, slot: LinkDmaSourceSlot) -> u16 {
        self.sources[slot.index()]
    }

    fn set_source(&mut self, slot: LinkDmaSourceSlot, value: u16) {
        self.sources[slot.index()] = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WaterHdmaWindowState {
    window_x: u16,
    window_y: u16,
    window_y_radius: u16,
    window_x_radius: u16,
    watergate_spotlight_y_upper: u16,
    watergate_pointer: u8,
    watergate_tilemap_pos_x2: u16,
}

impl WaterHdmaWindowState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            window_x: read_le_u16(ram, WATER_HDMA_WINDOW_X),
            window_y: read_le_u16(ram, WATER_HDMA_WINDOW_Y),
            window_y_radius: read_le_u16(ram, WATER_HDMA_WINDOW_Y_RADIUS),
            window_x_radius: read_le_u16(ram, WATER_HDMA_WINDOW_X_RADIUS),
            watergate_spotlight_y_upper: read_le_u16(ram, WATERGATE_SPOTLIGHT_Y_UPPER),
            watergate_pointer: ram_byte(ram, WATERGATE_POINTER),
            watergate_tilemap_pos_x2: read_le_u16(ram, WATERGATE_POS),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, WATER_HDMA_WINDOW_X, self.window_x);
        write_le_u16(ram, WATER_HDMA_WINDOW_Y, self.window_y);
        write_le_u16(ram, WATER_HDMA_WINDOW_Y_RADIUS, self.window_y_radius);
        write_le_u16(ram, WATER_HDMA_WINDOW_X_RADIUS, self.window_x_radius);
        write_le_u16(
            ram,
            WATERGATE_SPOTLIGHT_Y_UPPER,
            self.watergate_spotlight_y_upper,
        );
        ram[WATERGATE_POINTER] = self.watergate_pointer;
        write_le_u16(ram, WATERGATE_POS, self.watergate_tilemap_pos_x2);
    }

    pub(crate) fn window_x(&self) -> u16 {
        self.window_x
    }

    pub(crate) fn window_y(&self) -> u16 {
        self.window_y
    }

    pub(crate) fn window_y_radius(&self) -> u16 {
        self.window_y_radius
    }

    pub(crate) fn window_x_radius(&self) -> u16 {
        self.window_x_radius
    }

    pub(crate) fn watergate_spotlight_y_upper(&self) -> u16 {
        self.watergate_spotlight_y_upper
    }

    pub(crate) fn watergate_pointer(&self) -> u8 {
        self.watergate_pointer
    }

    pub(crate) fn watergate_tilemap_pos_x2(&self) -> u16 {
        self.watergate_tilemap_pos_x2
    }

    pub(crate) fn decrement_watergate_spotlight_y_upper(&mut self) -> u16 {
        self.watergate_spotlight_y_upper = self.watergate_spotlight_y_upper.wrapping_sub(1);
        self.watergate_spotlight_y_upper
    }

    pub(crate) fn set_watergate_pointer(&mut self, value: u8) {
        self.watergate_pointer = value;
    }

    pub(crate) fn increment_watergate_pointer(&mut self) -> u8 {
        self.watergate_pointer = self.watergate_pointer.wrapping_add(1);
        self.watergate_pointer
    }

    pub(crate) fn set_watergate_tilemap_pos_x2(&mut self, value: u16) {
        self.watergate_tilemap_pos_x2 = value;
    }

    pub(crate) fn set_window_x(&mut self, value: u16) {
        self.window_x = value;
    }

    pub(crate) fn set_window_y(&mut self, value: u16) {
        self.window_y = value;
    }

    pub(crate) fn set_window_x_radius(&mut self, value: u16) {
        self.window_x_radius = value;
    }

    pub(crate) fn set_window_y_radius_byte(&mut self, value: u8) {
        self.window_y_radius = (self.window_y_radius & 0xff00) | u16::from(value);
    }

    pub(crate) fn increment_window_y_radius_byte(&mut self) -> u8 {
        let next = (self.window_y_radius as u8).wrapping_add(1);
        self.set_window_y_radius_byte(next);
        next
    }

    pub(crate) fn set_watergate_spotlight_y_upper(&mut self, value: u16) {
        self.watergate_spotlight_y_upper = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldPaletteBackupState {
    main_indoors: u8,
    aux3_bg_palette_7: u8,
    main_indoors_copy: u8,
}

impl OverworldPaletteBackupState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            main_indoors: ram_byte(ram, OVERWORLD_PAL_MAIN_INDOORS_BACKUP),
            aux3_bg_palette_7: ram_byte(ram, OVERWORLD_PAL_AUX3_BP7_BACKUP),
            main_indoors_copy: ram_byte(ram, OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP] = self.main_indoors;
        ram[OVERWORLD_PAL_AUX3_BP7_BACKUP] = self.aux3_bg_palette_7;
        ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP] = self.main_indoors_copy;
    }

    pub(crate) fn main_indoors(&self) -> u8 {
        self.main_indoors
    }

    pub(crate) fn aux3_bg_palette_7(&self) -> u8 {
        self.aux3_bg_palette_7
    }

    pub(crate) fn main_indoors_copy(&self) -> u8 {
        self.main_indoors_copy
    }

    pub(crate) fn set_main_indoors(&mut self, value: u8) {
        self.main_indoors = value;
    }

    pub(crate) fn set_aux3_bg_palette_7(&mut self, value: u8) {
        self.aux3_bg_palette_7 = value;
    }

    pub(crate) fn set_main_indoors_copy(&mut self, value: u8) {
        self.main_indoors_copy = value;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PaletteBufferState {
    main: Vec<u8>,
    aux: Vec<u8>,
    overworld_backup: Vec<u8>,
    overworld_aux_or_main_offset: u16,
    sprite_palette_0_left: u8,
    sprite_palette_5_left: u8,
    sprite_palette_6_left: u8,
    main_palette_indoors: u8,
    hud_palette: u8,
    sprite_palette_6_right_indoors: u8,
    overworld_aux2_bg_palettes_5_to_7_high: u8,
    overworld_aux3_bg_palette_7_low: u8,
    overworld_palette_mode: u8,
}

impl Default for PaletteBufferState {
    fn default() -> Self {
        Self {
            main: vec![0; PALETTE_BANK_BYTES],
            aux: vec![0; PALETTE_BANK_BYTES],
            overworld_backup: vec![0; PALETTE_BANK_BYTES],
            overworld_aux_or_main_offset: 0,
            sprite_palette_0_left: 0,
            sprite_palette_5_left: 0,
            sprite_palette_6_left: 0,
            main_palette_indoors: 0,
            hud_palette: 0,
            sprite_palette_6_right_indoors: 0,
            overworld_aux2_bg_palettes_5_to_7_high: 0,
            overworld_aux3_bg_palette_7_low: 0,
            overworld_palette_mode: 0,
        }
    }
}

impl PaletteBufferState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            main: read_palette_bank(ram, MAIN_PALETTE_BUFFER),
            aux: read_palette_bank(ram, AUX_PALETTE_BUFFER),
            overworld_backup: read_palette_bank(ram, MAPBAK_PALETTE),
            overworld_aux_or_main_offset: read_le_u16(ram, OVERWORLD_PALETTE_AUX_OR_MAIN),
            sprite_palette_0_left: ram_byte(ram, PALETTE_SP0L),
            sprite_palette_5_left: ram_byte(ram, PALETTE_SP5L),
            sprite_palette_6_left: ram_byte(ram, PALETTE_SP6L),
            main_palette_indoors: ram_byte(ram, PALETTE_MAIN_INDOORS),
            hud_palette: ram_byte(ram, HUD_PALETTE),
            sprite_palette_6_right_indoors: ram_byte(ram, PALETTE_SP6R_INDOORS),
            overworld_aux2_bg_palettes_5_to_7_high: ram_byte(ram, OVERWORLD_PALETTE_AUX2_BP5TO7_HI),
            overworld_aux3_bg_palette_7_low: ram_byte(ram, OVERWORLD_PALETTE_AUX3_BP7_LO),
            overworld_palette_mode: ram_byte(ram, OVERWORLD_PALETTE_MODE),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_palette_bank(ram, MAIN_PALETTE_BUFFER, &self.main);
        write_palette_bank(ram, AUX_PALETTE_BUFFER, &self.aux);
        write_palette_bank(ram, MAPBAK_PALETTE, &self.overworld_backup);
        write_le_u16(
            ram,
            OVERWORLD_PALETTE_AUX_OR_MAIN,
            self.overworld_aux_or_main_offset,
        );
        ram[PALETTE_SP0L] = self.sprite_palette_0_left;
        ram[PALETTE_SP5L] = self.sprite_palette_5_left;
        ram[PALETTE_SP6L] = self.sprite_palette_6_left;
        ram[PALETTE_MAIN_INDOORS] = self.main_palette_indoors;
        ram[HUD_PALETTE] = self.hud_palette;
        ram[PALETTE_SP6R_INDOORS] = self.sprite_palette_6_right_indoors;
        ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = self.overworld_aux2_bg_palettes_5_to_7_high;
        ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = self.overworld_aux3_bg_palette_7_low;
        ram[OVERWORLD_PALETTE_MODE] = self.overworld_palette_mode;
    }

    pub(crate) fn main_color(&self, index: usize) -> u16 {
        read_palette_word(&self.main, index)
    }

    pub(crate) fn aux_color(&self, index: usize) -> u16 {
        read_palette_word(&self.aux, index)
    }

    pub(crate) fn aux_visible_slice(&self) -> &[u8] {
        &self.aux[..PALETTE_VISIBLE_BYTES]
    }

    pub(crate) fn main_full_slice(&self) -> &[u8] {
        &self.main[..PALETTE_BANK_BYTES]
    }

    pub(crate) fn aux_full_slice(&self) -> &[u8] {
        &self.aux[..PALETTE_BANK_BYTES]
    }

    pub(crate) fn overworld_palette_backup(&self) -> &[u8] {
        &self.overworld_backup[..PALETTE_BANK_BYTES]
    }

    pub(crate) fn overworld_aux_or_main_offset(&self) -> u16 {
        self.overworld_aux_or_main_offset
    }
}

fn read_palette_bank(ram: &[u8], base: usize) -> Vec<u8> {
    let mut bank = vec![0; PALETTE_BANK_BYTES];
    let available = ram.len().saturating_sub(base).min(PALETTE_BANK_BYTES);
    bank[..available].copy_from_slice(&ram[base..base + available]);
    bank
}

fn write_palette_bank(ram: &mut [u8], base: usize, bank: &[u8]) {
    let len = bank.len().min(PALETTE_BANK_BYTES);
    ram[base..base + len].copy_from_slice(&bank[..len]);
}

fn read_palette_word(bank: &[u8], index: usize) -> u16 {
    let offset = index * 2;
    if offset + 1 >= bank.len() {
        return 0;
    }
    u16::from(bank[offset]) | (u16::from(bank[offset + 1]) << 8)
}

fn write_palette_word(bank: &mut [u8], index: usize, value: u16) {
    let offset = index * 2;
    if offset + 1 < bank.len() {
        bank[offset] = value as u8;
        bank[offset + 1] = (value >> 8) as u8;
    }
}

pub(crate) struct NativePaletteBufferBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativePaletteBufferBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        display.palette_buffer = PaletteBufferState::load_from_ram(ram);
        Self { display, ram }
    }

    pub(crate) fn clear_aux_visible_subpalettes(&mut self) {
        self.display.palette_buffer.aux[VISIBLE_SUBPALETTE_CLEAR_START
            ..VISIBLE_SUBPALETTE_CLEAR_START + VISIBLE_SUBPALETTE_CLEAR_LEN]
            .fill(0);
        self.ram[AUX_PALETTE_BUFFER + VISIBLE_SUBPALETTE_CLEAR_START
            ..AUX_PALETTE_BUFFER + VISIBLE_SUBPALETTE_CLEAR_START + VISIBLE_SUBPALETTE_CLEAR_LEN]
            .fill(0);
    }

    pub(crate) fn clear_main_visible_subpalettes(&mut self) {
        self.display.palette_buffer.main[VISIBLE_SUBPALETTE_CLEAR_START
            ..VISIBLE_SUBPALETTE_CLEAR_START + VISIBLE_SUBPALETTE_CLEAR_LEN]
            .fill(0);
        self.ram[MAIN_PALETTE_BUFFER + VISIBLE_SUBPALETTE_CLEAR_START
            ..MAIN_PALETTE_BUFFER + VISIBLE_SUBPALETTE_CLEAR_START + VISIBLE_SUBPALETTE_CLEAR_LEN]
            .fill(0);
    }

    pub(crate) fn clear_aux_sprite_subpalettes(&mut self) {
        self.display.palette_buffer.aux[SPRITE_SUBPALETTE_CLEAR_START
            ..SPRITE_SUBPALETTE_CLEAR_START + SPRITE_SUBPALETTE_CLEAR_LEN]
            .fill(0);
        self.ram[AUX_PALETTE_BUFFER + SPRITE_SUBPALETTE_CLEAR_START
            ..AUX_PALETTE_BUFFER + SPRITE_SUBPALETTE_CLEAR_START + SPRITE_SUBPALETTE_CLEAR_LEN]
            .fill(0);
    }

    pub(crate) fn set_main_color(&mut self, index: usize, value: u16) {
        write_palette_word(&mut self.display.palette_buffer.main, index, value);
        write_le_u16(self.ram, MAIN_PALETTE_BUFFER + index * 2, value);
    }

    pub(crate) fn set_aux_color(&mut self, index: usize, value: u16) {
        write_palette_word(&mut self.display.palette_buffer.aux, index, value);
        write_le_u16(self.ram, AUX_PALETTE_BUFFER + index * 2, value);
    }

    pub(crate) fn set_overworld_aux_or_main_offset(&mut self, value: u16) {
        self.display.palette_buffer.overworld_aux_or_main_offset = value;
        write_le_u16(self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, value);
    }

    pub(crate) fn clear_overworld_aux_or_main_offset(&mut self) {
        self.set_overworld_aux_or_main_offset(0);
    }

    pub(crate) fn select_overworld_aux_palette_offset(&mut self) {
        self.set_overworld_aux_or_main_offset(0x0200);
    }

    pub(crate) fn keep_overworld_aux_or_main_low_byte(&mut self) {
        let value = self.display.palette_buffer.overworld_aux_or_main_offset & 0x00ff;
        self.set_overworld_aux_or_main_offset(value);
    }

    pub(crate) fn clear_main_full(&mut self) {
        self.display.palette_buffer.main.fill(0);
        self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + PALETTE_BANK_BYTES].fill(0);
    }

    pub(crate) fn copy_aux_visible_from(&mut self, palette: &[u8]) {
        self.copy_aux_range_from(0, PALETTE_VISIBLE_BYTES, palette);
    }

    pub(crate) fn copy_aux_full_from(&mut self, palette: &[u8]) {
        self.copy_aux_range_from(0, PALETTE_BANK_BYTES, palette);
    }

    pub(crate) fn backup_overworld_palette_from(&mut self, palette: &[u8]) {
        let len = palette.len().min(PALETTE_BANK_BYTES);
        self.display.palette_buffer.overworld_backup[..len].copy_from_slice(&palette[..len]);
        self.ram[MAPBAK_PALETTE..MAPBAK_PALETTE + len].copy_from_slice(&palette[..len]);
    }

    pub(crate) fn copy_main_full_from(&mut self, palette: &[u8]) {
        self.copy_main_range_from(0, PALETTE_BANK_BYTES, palette);
    }

    pub(crate) fn copy_main_palette_bytes(&mut self, src: &[u8], len: usize) {
        self.copy_main_range_from(0, len.min(PALETTE_BANK_BYTES), src);
    }

    pub(crate) fn set_sp0l(&mut self, value: u8) {
        self.display.palette_buffer.sprite_palette_0_left = value;
        self.ram[PALETTE_SP0L] = value;
    }

    pub(crate) fn set_sp5l(&mut self, value: u8) {
        self.display.palette_buffer.sprite_palette_5_left = value;
        self.ram[PALETTE_SP5L] = value;
    }

    pub(crate) fn set_sp6l(&mut self, value: u8) {
        self.display.palette_buffer.sprite_palette_6_left = value;
        self.ram[PALETTE_SP6L] = value;
    }

    pub(crate) fn set_palette_main_indoors(&mut self, value: u8) {
        self.display.palette_buffer.main_palette_indoors = value;
        self.ram[PALETTE_MAIN_INDOORS] = value;
    }

    pub(crate) fn set_hud_palette(&mut self, value: u8) {
        self.display.palette_buffer.hud_palette = value;
        self.ram[HUD_PALETTE] = value;
    }

    pub(crate) fn set_sp6r_indoors(&mut self, value: u8) {
        self.display.palette_buffer.sprite_palette_6_right_indoors = value;
        self.ram[PALETTE_SP6R_INDOORS] = value;
    }

    pub(crate) fn set_overworld_palette_aux2_hi(&mut self, value: u8) {
        self.display
            .palette_buffer
            .overworld_aux2_bg_palettes_5_to_7_high = value;
        self.ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = value;
    }

    pub(crate) fn set_overworld_palette_aux3_lo(&mut self, value: u8) {
        self.display.palette_buffer.overworld_aux3_bg_palette_7_low = value;
        self.ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = value;
    }

    pub(crate) fn set_bg_tile_animation_countdown(&mut self, value: u16) {
        self.display.bg_tile_animation_countdown = value;
        write_le_u16(self.ram, BG_TILE_ANIMATION_COUNTDOWN, value);
    }

    pub(crate) fn set_overworld_palette_mode(&mut self, value: u8) {
        self.display.palette_buffer.overworld_palette_mode = value;
        self.ram[OVERWORLD_PALETTE_MODE] = value;
    }

    fn copy_aux_range_from(&mut self, start: usize, len: usize, src: &[u8]) {
        let len = len.min(src.len()).min(PALETTE_BANK_BYTES - start);
        self.display.palette_buffer.aux[start..start + len].copy_from_slice(&src[..len]);
        self.ram[AUX_PALETTE_BUFFER + start..AUX_PALETTE_BUFFER + start + len]
            .copy_from_slice(&src[..len]);
    }

    fn copy_main_range_from(&mut self, start: usize, len: usize, src: &[u8]) {
        let len = len.min(src.len()).min(PALETTE_BANK_BYTES - start);
        self.display.palette_buffer.main[start..start + len].copy_from_slice(&src[..len]);
        self.ram[MAIN_PALETTE_BUFFER + start..MAIN_PALETTE_BUFFER + start + len]
            .copy_from_slice(&src[..len]);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpotlightHdmaState {
    y_lower: u16,
    y_upper: u16,
    window_x_center: u16,
    window_state: u16,
    window_radius: u16,
    window_y_buffer: u16,
    dynamic_table: Vec<u16>,
}

impl Default for SpotlightHdmaState {
    fn default() -> Self {
        Self {
            y_lower: 0,
            y_upper: 0,
            window_x_center: 0,
            window_state: 0,
            window_radius: 0,
            window_y_buffer: 0,
            dynamic_table: vec![0; SPOTLIGHT_HDMA_WORD_COUNT],
        }
    }
}

impl SpotlightHdmaState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut dynamic_table = vec![0; SPOTLIGHT_HDMA_WORD_COUNT];
        for (index, value) in dynamic_table.iter_mut().enumerate() {
            *value = read_le_u16(ram, HDMA_TABLE_DYNAMIC + index * 2);
        }

        Self {
            y_lower: read_le_u16(ram, SPOTLIGHT_Y_LOWER),
            y_upper: read_le_u16(ram, SPOTLIGHT_Y_UPPER),
            window_x_center: read_le_u16(ram, SPOTLIGHT_WINDOW_X_CENTER),
            window_state: read_le_u16(ram, SPOTLIGHT_WINDOW_STATE),
            window_radius: read_le_u16(ram, SPOTLIGHT_WINDOW_RADIUS),
            window_y_buffer: read_le_u16(ram, SPOTLIGHT_WINDOW_Y_BUFFER),
            dynamic_table,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, SPOTLIGHT_Y_LOWER, self.y_lower);
        write_le_u16(ram, SPOTLIGHT_Y_UPPER, self.y_upper);
        write_le_u16(ram, SPOTLIGHT_WINDOW_X_CENTER, self.window_x_center);
        write_le_u16(ram, SPOTLIGHT_WINDOW_STATE, self.window_state);
        write_le_u16(ram, SPOTLIGHT_WINDOW_RADIUS, self.window_radius);
        write_le_u16(ram, SPOTLIGHT_WINDOW_Y_BUFFER, self.window_y_buffer);

        for index in 0..SPOTLIGHT_HDMA_WORD_COUNT {
            let value = self.dynamic_table.get(index).copied().unwrap_or(0);
            write_le_u16(ram, HDMA_TABLE_DYNAMIC + index * 2, value);
        }
    }

    pub(crate) fn y_lower(&self) -> u16 {
        self.y_lower
    }

    pub(crate) fn y_upper(&self) -> u16 {
        self.y_upper
    }

    pub(crate) fn window_x_center(&self) -> u16 {
        self.window_x_center
    }

    pub(crate) fn window_state(&self) -> u16 {
        self.window_state
    }

    pub(crate) fn window_state_byte(&self) -> u8 {
        self.window_state as u8
    }

    pub(crate) fn window_radius(&self) -> u16 {
        self.window_radius
    }

    pub(crate) fn window_radius_byte(&self) -> u8 {
        self.window_radius as u8
    }

    pub(crate) fn window_y_buffer(&self) -> u16 {
        self.window_y_buffer
    }

    pub(crate) fn window_y_buffer_byte(&self) -> u8 {
        self.window_y_buffer as u8
    }

    pub(crate) fn hdma_table_dynamic_entry(&self, index: usize) -> u16 {
        self.dynamic_table.get(index).copied().unwrap_or(0)
    }

    fn set_low_byte(word: &mut u16, value: u8) {
        *word = (*word & 0xff00) | u16::from(value);
    }

    pub(crate) fn set_y_lower(&mut self, value: u16) {
        self.y_lower = value;
    }

    pub(crate) fn set_y_upper(&mut self, value: u16) {
        self.y_upper = value;
    }

    pub(crate) fn set_window_x_center(&mut self, value: u16) {
        self.window_x_center = value;
    }

    pub(crate) fn set_window_state(&mut self, value: u16) {
        self.window_state = value;
    }

    pub(crate) fn set_window_radius(&mut self, value: u16) {
        self.window_radius = value;
    }

    pub(crate) fn set_window_y_buffer(&mut self, value: u16) {
        self.window_y_buffer = value;
    }

    pub(crate) fn decrement_window_y_buffer(&mut self) -> u16 {
        self.window_y_buffer = self.window_y_buffer.wrapping_sub(1);
        self.window_y_buffer
    }

    pub(crate) fn set_window_radius_byte(&mut self, value: u8) {
        Self::set_low_byte(&mut self.window_radius, value);
    }

    pub(crate) fn set_window_state_byte(&mut self, value: u8) {
        Self::set_low_byte(&mut self.window_state, value);
    }

    pub(crate) fn set_window_y_buffer_byte(&mut self, value: u8) {
        Self::set_low_byte(&mut self.window_y_buffer, value);
    }

    pub(crate) fn increment_window_y_buffer_byte(&mut self) {
        let next = self.window_y_buffer_byte().wrapping_add(1);
        Self::set_low_byte(&mut self.window_y_buffer, next);
    }

    pub(crate) fn shr_window_radius_byte(&mut self, shift: u8) {
        let next = self.window_radius_byte() >> shift;
        Self::set_low_byte(&mut self.window_radius, next);
    }

    pub(crate) fn add_window_radius_byte(&mut self, value: u8) {
        let next = self.window_radius_byte().wrapping_add(value);
        Self::set_low_byte(&mut self.window_radius, next);
    }

    pub(crate) fn set_hdma_table_dynamic_entry(&mut self, index: usize, value: u16) {
        if let Some(entry) = self.dynamic_table.get_mut(index) {
            *entry = value;
        }
    }

    pub(crate) fn clear_hdma_table_dynamic(&mut self, count: usize) {
        self.clear_hdma_table_dynamic_range(0, count);
    }

    pub(crate) fn clear_hdma_table_dynamic_range(&mut self, start: usize, count: usize) {
        let end = start.saturating_add(count).min(self.dynamic_table.len());
        for entry in &mut self.dynamic_table[start.min(end)..end] {
            *entry = 0;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PpuScrollCopyState {
    bg1_h_copy: u16,
    bg1_v_copy: u16,
    bg2_h_copy: u16,
    bg2_v_copy: u16,
    bg1_h_copy2: u16,
    bg1_v_copy2: u16,
    bg2_h_copy2: u16,
    bg2_v_copy2: u16,
    bg3_h_copy2: u16,
    bg3_v_copy2: u16,
    bg2_h_copy2_cached: u16,
    bg2_v_copy2_cached: u16,
    map_backup_bg1_h_copy2: u16,
    map_backup_bg2_h_copy2: u16,
    map_backup_bg1_v_copy2: u16,
    map_backup_bg2_v_copy2: u16,
    special_exit_bg2_h_copy2: u16,
    special_exit_bg2_v_copy2: u16,
    exit_bg2_h_copy2: u16,
    exit_bg2_v_copy2: u16,
    mode7_center_x: u16,
    mode7_center_y: u16,
    bg1_h_subpixel: u16,
    bg1_v_subpixel: u16,
    camera_y_scroll_cached: u16,
    camera_x_scroll_cached: u16,
    mapbak_tm: u16,
    mapbak_ts: u8,
    mapbak_main_tile_theme_index: u8,
    mapbak_sprite_graphics_index: u8,
    mapbak_aux_tile_theme_index: u8,
    mapbak_bg1_x_offset: u16,
    mapbak_bg1_y_offset: u16,
    mapbak_cgwsel: u16,
    mapbak_hdmaen: u8,
    mapbak_palette: Vec<u8>,
}

const MAPBAK_PALETTE_BYTES: usize = 0x200;

impl Default for PpuScrollCopyState {
    fn default() -> Self {
        Self {
            bg1_h_copy: 0,
            bg1_v_copy: 0,
            bg2_h_copy: 0,
            bg2_v_copy: 0,
            bg1_h_copy2: 0,
            bg1_v_copy2: 0,
            bg2_h_copy2: 0,
            bg2_v_copy2: 0,
            bg3_h_copy2: 0,
            bg3_v_copy2: 0,
            bg2_h_copy2_cached: 0,
            bg2_v_copy2_cached: 0,
            map_backup_bg1_h_copy2: 0,
            map_backup_bg2_h_copy2: 0,
            map_backup_bg1_v_copy2: 0,
            map_backup_bg2_v_copy2: 0,
            special_exit_bg2_h_copy2: 0,
            special_exit_bg2_v_copy2: 0,
            exit_bg2_h_copy2: 0,
            exit_bg2_v_copy2: 0,
            mode7_center_x: 0,
            mode7_center_y: 0,
            bg1_h_subpixel: 0,
            bg1_v_subpixel: 0,
            camera_y_scroll_cached: 0,
            camera_x_scroll_cached: 0,
            mapbak_tm: 0,
            mapbak_ts: 0,
            mapbak_main_tile_theme_index: 0,
            mapbak_sprite_graphics_index: 0,
            mapbak_aux_tile_theme_index: 0,
            mapbak_bg1_x_offset: 0,
            mapbak_bg1_y_offset: 0,
            mapbak_cgwsel: 0,
            mapbak_hdmaen: 0,
            mapbak_palette: vec![0; MAPBAK_PALETTE_BYTES],
        }
    }
}

impl PpuScrollCopyState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mapbak_palette = ram[MAPBAK_PALETTE..MAPBAK_PALETTE + MAPBAK_PALETTE_BYTES].to_vec();
        Self {
            bg1_h_copy: read_le_u16(ram, BG1_H_SCROLL_COPY),
            bg1_v_copy: read_le_u16(ram, BG1_V_SCROLL_COPY),
            bg2_h_copy: read_le_u16(ram, BG2_H_SCROLL_COPY),
            bg2_v_copy: read_le_u16(ram, BG2_V_SCROLL_COPY),
            bg1_h_copy2: read_le_u16(ram, BG1_X_SCROLL),
            bg1_v_copy2: read_le_u16(ram, BG1_Y_SCROLL),
            bg2_h_copy2: read_le_u16(ram, BG2_X_SCROLL),
            bg2_v_copy2: read_le_u16(ram, BG2_Y_SCROLL),
            bg3_h_copy2: read_le_u16(ram, BG3_H_SCROLL_COPY2),
            bg3_v_copy2: read_le_u16(ram, BG3_V_SCROLL_COPY2),
            bg2_h_copy2_cached: read_le_u16(ram, BG2_H_SCROLL_COPY2_CACHED),
            bg2_v_copy2_cached: read_le_u16(ram, BG2_V_SCROLL_COPY2_CACHED),
            map_backup_bg1_h_copy2: read_le_u16(ram, MAP_BACKUP_BG1_H_SCROLL_COPY2),
            map_backup_bg2_h_copy2: read_le_u16(ram, MAP_BACKUP_BG2_H_SCROLL_COPY2),
            map_backup_bg1_v_copy2: read_le_u16(ram, MAP_BACKUP_BG1_V_SCROLL_COPY2),
            map_backup_bg2_v_copy2: read_le_u16(ram, MAP_BACKUP_BG2_V_SCROLL_COPY2),
            special_exit_bg2_h_copy2: read_le_u16(ram, BG2_H_SCROLL_COPY2_SPECIAL_EXIT),
            special_exit_bg2_v_copy2: read_le_u16(ram, BG2_V_SCROLL_COPY2_SPECIAL_EXIT),
            exit_bg2_h_copy2: read_le_u16(ram, BG2_H_SCROLL_COPY2_EXIT),
            exit_bg2_v_copy2: read_le_u16(ram, BG2_V_SCROLL_COPY2_EXIT),
            mode7_center_x: read_le_u16(ram, MODE7_CENTER_X_COPY),
            mode7_center_y: read_le_u16(ram, MODE7_CENTER_Y_COPY),
            bg1_h_subpixel: read_le_u16(ram, BG1_H_SCROLL_SUBPIXEL),
            bg1_v_subpixel: read_le_u16(ram, BG1_V_SCROLL_SUBPIXEL),
            camera_y_scroll_cached: read_le_u16(ram, CAMERA_Y_COORD_SCROLL_LOW_CACHED),
            camera_x_scroll_cached: read_le_u16(ram, CAMERA_X_COORD_SCROLL_LOW_CACHED),
            mapbak_tm: read_le_u16(ram, MAPBAK_TM),
            mapbak_ts: ram_byte(ram, MAPBAK_TS),
            mapbak_main_tile_theme_index: ram_byte(ram, MAPBAK_MAIN_TILE_THEME_INDEX),
            mapbak_sprite_graphics_index: ram_byte(ram, MAPBAK_SPRITE_GRAPHICS_INDEX),
            mapbak_aux_tile_theme_index: ram_byte(ram, MAPBAK_AUX_TILE_THEME_INDEX),
            mapbak_bg1_x_offset: read_le_u16(ram, MAPBAK_BG1_X_OFFSET),
            mapbak_bg1_y_offset: read_le_u16(ram, MAPBAK_BG1_Y_OFFSET),
            mapbak_cgwsel: read_le_u16(ram, MAPBAK_CGWSEL),
            mapbak_hdmaen: ram_byte(ram, MAPBAK_HDMAEN),
            mapbak_palette,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, BG1_H_SCROLL_COPY, self.bg1_h_copy);
        write_le_u16(ram, BG1_V_SCROLL_COPY, self.bg1_v_copy);
        write_le_u16(ram, BG2_H_SCROLL_COPY, self.bg2_h_copy);
        write_le_u16(ram, BG2_V_SCROLL_COPY, self.bg2_v_copy);
        write_le_u16(ram, BG1_X_SCROLL, self.bg1_h_copy2);
        write_le_u16(ram, BG1_Y_SCROLL, self.bg1_v_copy2);
        write_le_u16(ram, BG2_X_SCROLL, self.bg2_h_copy2);
        write_le_u16(ram, BG2_Y_SCROLL, self.bg2_v_copy2);
        write_le_u16(ram, BG3_H_SCROLL_COPY2, self.bg3_h_copy2);
        write_le_u16(ram, BG3_V_SCROLL_COPY2, self.bg3_v_copy2);
        write_le_u16(ram, BG2_H_SCROLL_COPY2_CACHED, self.bg2_h_copy2_cached);
        write_le_u16(ram, BG2_V_SCROLL_COPY2_CACHED, self.bg2_v_copy2_cached);
        write_le_u16(
            ram,
            MAP_BACKUP_BG1_H_SCROLL_COPY2,
            self.map_backup_bg1_h_copy2,
        );
        write_le_u16(
            ram,
            MAP_BACKUP_BG2_H_SCROLL_COPY2,
            self.map_backup_bg2_h_copy2,
        );
        write_le_u16(
            ram,
            MAP_BACKUP_BG1_V_SCROLL_COPY2,
            self.map_backup_bg1_v_copy2,
        );
        write_le_u16(
            ram,
            MAP_BACKUP_BG2_V_SCROLL_COPY2,
            self.map_backup_bg2_v_copy2,
        );
        write_le_u16(
            ram,
            BG2_H_SCROLL_COPY2_SPECIAL_EXIT,
            self.special_exit_bg2_h_copy2,
        );
        write_le_u16(
            ram,
            BG2_V_SCROLL_COPY2_SPECIAL_EXIT,
            self.special_exit_bg2_v_copy2,
        );
        write_le_u16(ram, BG2_H_SCROLL_COPY2_EXIT, self.exit_bg2_h_copy2);
        write_le_u16(ram, BG2_V_SCROLL_COPY2_EXIT, self.exit_bg2_v_copy2);
        write_le_u16(ram, MODE7_CENTER_X_COPY, self.mode7_center_x);
        write_le_u16(ram, MODE7_CENTER_Y_COPY, self.mode7_center_y);
        write_le_u16(ram, BG1_H_SCROLL_SUBPIXEL, self.bg1_h_subpixel);
        write_le_u16(ram, BG1_V_SCROLL_SUBPIXEL, self.bg1_v_subpixel);
        write_le_u16(
            ram,
            CAMERA_Y_COORD_SCROLL_LOW_CACHED,
            self.camera_y_scroll_cached,
        );
        write_le_u16(
            ram,
            CAMERA_X_COORD_SCROLL_LOW_CACHED,
            self.camera_x_scroll_cached,
        );
        write_le_u16(ram, MAPBAK_TM, self.mapbak_tm);
        ram[MAPBAK_TS] = self.mapbak_ts;
        ram[MAPBAK_MAIN_TILE_THEME_INDEX] = self.mapbak_main_tile_theme_index;
        ram[MAPBAK_SPRITE_GRAPHICS_INDEX] = self.mapbak_sprite_graphics_index;
        ram[MAPBAK_AUX_TILE_THEME_INDEX] = self.mapbak_aux_tile_theme_index;
        write_le_u16(ram, MAPBAK_BG1_X_OFFSET, self.mapbak_bg1_x_offset);
        write_le_u16(ram, MAPBAK_BG1_Y_OFFSET, self.mapbak_bg1_y_offset);
        write_le_u16(ram, MAPBAK_CGWSEL, self.mapbak_cgwsel);
        ram[MAPBAK_HDMAEN] = self.mapbak_hdmaen;
        ram[MAPBAK_PALETTE..MAPBAK_PALETTE + MAPBAK_PALETTE_BYTES].fill(0);
        let len = self.mapbak_palette.len().min(MAPBAK_PALETTE_BYTES);
        ram[MAPBAK_PALETTE..MAPBAK_PALETTE + len].copy_from_slice(&self.mapbak_palette[..len]);
    }

    pub(crate) fn bg2_h_copy2_offset() -> usize {
        BG2_X_SCROLL
    }

    pub(crate) fn bg1_h_high(&self) -> u8 {
        (self.bg1_h_copy >> 8) as u8
    }

    pub(crate) fn bg1_h_copy(&self) -> u16 {
        self.bg1_h_copy
    }

    pub(crate) fn bg1_h_copy_low(&self) -> u8 {
        self.bg1_h_copy as u8
    }

    pub(crate) fn bg1_v_high(&self) -> u8 {
        (self.bg1_v_copy >> 8) as u8
    }

    pub(crate) fn bg1_v_copy(&self) -> u16 {
        self.bg1_v_copy
    }

    pub(crate) fn bg1_v_copy_low(&self) -> u8 {
        self.bg1_v_copy as u8
    }

    pub(crate) fn bg2_h_high(&self) -> u8 {
        (self.bg2_h_copy >> 8) as u8
    }

    pub(crate) fn bg2_h_copy(&self) -> u16 {
        self.bg2_h_copy
    }

    pub(crate) fn bg2_h_copy_low(&self) -> u8 {
        self.bg2_h_copy as u8
    }

    pub(crate) fn bg2_v_high(&self) -> u8 {
        (self.bg2_v_copy >> 8) as u8
    }

    pub(crate) fn bg2_v_copy(&self) -> u16 {
        self.bg2_v_copy
    }

    pub(crate) fn bg2_v_copy_low(&self) -> u8 {
        self.bg2_v_copy as u8
    }

    pub(crate) fn bg1_h_copy2(&self) -> u16 {
        self.bg1_h_copy2
    }

    pub(crate) fn bg1_v_copy2(&self) -> u16 {
        self.bg1_v_copy2
    }

    pub(crate) fn bg2_h_copy2(&self) -> u16 {
        self.bg2_h_copy2
    }

    pub(crate) fn bg2_v_copy2(&self) -> u16 {
        self.bg2_v_copy2
    }

    pub(crate) fn bg2_copy2_for_axis(&self, vertical: bool) -> u16 {
        if vertical {
            self.bg2_v_copy2()
        } else {
            self.bg2_h_copy2()
        }
    }

    pub(crate) fn bg3_h_high(&self) -> u8 {
        (self.bg3_h_copy2 >> 8) as u8
    }

    pub(crate) fn bg3_h_copy2(&self) -> u16 {
        self.bg3_h_copy2
    }

    pub(crate) fn bg3_h_copy2_low(&self) -> u8 {
        self.bg3_h_copy2 as u8
    }

    pub(crate) fn bg3_v_high(&self) -> u8 {
        (self.bg3_v_copy2 >> 8) as u8
    }

    pub(crate) fn bg3_v_copy2(&self) -> u16 {
        self.bg3_v_copy2
    }

    pub(crate) fn bg3_v_copy2_low(&self) -> u8 {
        self.bg3_v_copy2 as u8
    }

    pub(crate) fn bg2_h_copy2_cached(&self) -> u16 {
        self.bg2_h_copy2_cached
    }

    pub(crate) fn bg2_v_copy2_cached(&self) -> u16 {
        self.bg2_v_copy2_cached
    }

    pub(crate) fn map_backup_bg1_h_copy2(&self) -> u16 {
        self.map_backup_bg1_h_copy2
    }

    pub(crate) fn mapbak_tm(&self) -> u8 {
        self.mapbak_tm as u8
    }

    pub(crate) fn map_backup_bg2_h_copy2(&self) -> u16 {
        self.map_backup_bg2_h_copy2
    }

    pub(crate) fn map_backup_bg1_v_copy2(&self) -> u16 {
        self.map_backup_bg1_v_copy2
    }

    pub(crate) fn map_backup_bg2_v_copy2(&self) -> u16 {
        self.map_backup_bg2_v_copy2
    }

    pub(crate) fn special_exit_bg2_h_copy2(&self) -> u16 {
        self.special_exit_bg2_h_copy2
    }

    pub(crate) fn special_exit_bg2_v_copy2(&self) -> u16 {
        self.special_exit_bg2_v_copy2
    }

    pub(crate) fn exit_bg2_h_copy2(&self) -> u16 {
        self.exit_bg2_h_copy2
    }

    pub(crate) fn exit_bg2_v_copy2(&self) -> u16 {
        self.exit_bg2_v_copy2
    }

    pub(crate) fn mode7_center_x_high(&self) -> u8 {
        (self.mode7_center_x >> 8) as u8
    }

    pub(crate) fn mode7_center_x(&self) -> u16 {
        self.mode7_center_x
    }

    pub(crate) fn mode7_center_y_high(&self) -> u8 {
        (self.mode7_center_y >> 8) as u8
    }

    pub(crate) fn mode7_center_y(&self) -> u16 {
        self.mode7_center_y
    }

    pub(crate) fn bg1_h_subpixel(&self) -> u16 {
        self.bg1_h_subpixel
    }

    pub(crate) fn bg1_v_subpixel(&self) -> u16 {
        self.bg1_v_subpixel
    }

    pub(crate) fn mapbak_main_tile_theme_index(&self) -> u8 {
        self.mapbak_main_tile_theme_index
    }

    pub(crate) fn mapbak_sprite_graphics_index(&self) -> u8 {
        self.mapbak_sprite_graphics_index
    }

    pub(crate) fn mapbak_aux_tile_theme_index(&self) -> u8 {
        self.mapbak_aux_tile_theme_index
    }

    pub(crate) fn mapbak_tm_word(&self) -> u16 {
        self.mapbak_tm
    }

    pub(crate) fn mapbak_ts(&self) -> u8 {
        self.mapbak_ts
    }

    pub(crate) fn mapbak_bg1_x_offset(&self) -> u16 {
        self.mapbak_bg1_x_offset
    }

    pub(crate) fn mapbak_bg1_y_offset(&self) -> u16 {
        self.mapbak_bg1_y_offset
    }

    pub(crate) fn mapbak_cgwsel(&self) -> u8 {
        self.mapbak_cgwsel as u8
    }

    pub(crate) fn mapbak_cgwsel_word(&self) -> u16 {
        self.mapbak_cgwsel
    }

    pub(crate) fn mapbak_hdmaen(&self) -> u8 {
        self.mapbak_hdmaen
    }

    pub(crate) fn mapbak_palette_slice(&self) -> &[u8] {
        &self.mapbak_palette
    }

    fn set_low_byte(word: &mut u16, value: u8) {
        *word = (*word & 0xff00) | u16::from(value);
    }

    fn set_high_byte(word: &mut u16, value: u8) {
        *word = (*word & 0x00ff) | (u16::from(value) << 8);
    }

    fn add_low_byte(word: &mut u16, value: u8) {
        Self::set_low_byte(word, (*word as u8).wrapping_add(value));
    }

    fn subtract_low_byte(word: &mut u16, value: u8) {
        Self::set_low_byte(word, (*word as u8).wrapping_sub(value));
    }

    pub(crate) fn set_mapbak_tm(&mut self, value: u8) {
        Self::set_low_byte(&mut self.mapbak_tm, value);
    }

    pub(crate) fn set_mapbak_ts(&mut self, value: u8) {
        self.mapbak_ts = value;
    }

    pub(crate) fn set_mapbak_tm_word(&mut self, value: u16) {
        self.mapbak_tm = value;
    }

    pub(crate) fn set_bg1_h_high(&mut self, value: u8) {
        Self::set_high_byte(&mut self.bg1_h_copy2, value);
    }

    pub(crate) fn set_bg1_h_copy(&mut self, value: u16) {
        self.bg1_h_copy = value;
    }

    pub(crate) fn set_bg1_v_copy(&mut self, value: u16) {
        self.bg1_v_copy = value;
    }

    pub(crate) fn set_bg2_h_copy(&mut self, value: u16) {
        self.bg2_h_copy = value;
    }

    pub(crate) fn set_bg2_v_copy(&mut self, value: u16) {
        self.bg2_v_copy = value;
    }

    pub(crate) fn set_bg1_h_copy_low(&mut self, value: u8) {
        Self::set_low_byte(&mut self.bg1_h_copy, value);
    }

    pub(crate) fn set_bg1_v_copy_low(&mut self, value: u8) {
        Self::set_low_byte(&mut self.bg1_v_copy, value);
    }

    pub(crate) fn set_bg2_h_copy_low(&mut self, value: u8) {
        Self::set_low_byte(&mut self.bg2_h_copy, value);
    }

    pub(crate) fn set_bg2_v_copy_low(&mut self, value: u8) {
        Self::set_low_byte(&mut self.bg2_v_copy, value);
    }

    pub(crate) fn set_bg1_h_copy2(&mut self, value: u16) {
        self.bg1_h_copy2 = value;
    }

    pub(crate) fn set_bg1_v_copy2(&mut self, value: u16) {
        self.bg1_v_copy2 = value;
    }

    pub(crate) fn set_bg2_h_copy2(&mut self, value: u16) {
        self.bg2_h_copy2 = value;
    }

    pub(crate) fn set_bg2_v_copy2(&mut self, value: u16) {
        self.bg2_v_copy2 = value;
    }

    pub(crate) fn set_bg3_h_copy2(&mut self, value: u16) {
        self.bg3_h_copy2 = value;
    }

    pub(crate) fn set_bg3_v_copy2(&mut self, value: u16) {
        self.bg3_v_copy2 = value;
    }

    pub(crate) fn set_bg3_v_copy2_low(&mut self, value: u8) {
        Self::set_low_byte(&mut self.bg3_v_copy2, value);
    }

    pub(crate) fn set_mode7_center_x(&mut self, value: u16) {
        self.mode7_center_x = value;
    }

    pub(crate) fn set_mode7_center_y(&mut self, value: u16) {
        self.mode7_center_y = value;
    }

    pub(crate) fn set_mode7_center(&mut self, x: u16, y: u16) {
        self.set_mode7_center_x(x);
        self.set_mode7_center_y(y);
    }

    pub(crate) fn set_bg1_h_live_and_copy(&mut self, value: u16) {
        self.set_bg1_h_copy2(value);
        self.set_bg1_h_copy(value);
    }

    pub(crate) fn set_bg1_v_live_and_copy(&mut self, value: u16) {
        self.set_bg1_v_copy2(value);
        self.set_bg1_v_copy(value);
    }

    pub(crate) fn set_bg2_h_live_and_copy(&mut self, value: u16) {
        self.set_bg2_h_copy2(value);
        self.set_bg2_h_copy(value);
    }

    pub(crate) fn set_bg2_v_live_and_copy(&mut self, value: u16) {
        self.set_bg2_v_copy2(value);
        self.set_bg2_v_copy(value);
    }

    pub(crate) fn set_bg1_bg2_h_live_and_copy(&mut self, value: u16) {
        self.set_bg2_h_live_and_copy(value);
        self.set_bg1_h_live_and_copy(value);
    }

    pub(crate) fn set_bg1_bg2_v_live_and_copy(&mut self, value: u16) {
        self.set_bg2_v_live_and_copy(value);
        self.set_bg1_v_live_and_copy(value);
    }

    pub(crate) fn set_bg1_bg2_live_and_copy(
        &mut self,
        bg2_h: u16,
        bg2_v: u16,
        bg1_h: u16,
        bg1_v: u16,
    ) {
        self.set_bg2_h_live_and_copy(bg2_h);
        self.set_bg2_v_live_and_copy(bg2_v);
        self.set_bg1_h_live_and_copy(bg1_h);
        self.set_bg1_v_live_and_copy(bg1_v);
    }

    pub(crate) fn set_bg2_h_copy2_cached(&mut self, value: u16) {
        self.bg2_h_copy2_cached = value;
    }

    pub(crate) fn set_bg2_v_copy2_cached(&mut self, value: u16) {
        self.bg2_v_copy2_cached = value;
    }

    pub(crate) fn cache_bg2_live_scroll(&mut self) {
        self.bg2_h_copy2_cached = self.bg2_h_copy2;
        self.bg2_v_copy2_cached = self.bg2_v_copy2;
    }

    pub(crate) fn cache_camera_scroll(&mut self, camera_y: u16, camera_x: u16) {
        self.camera_y_scroll_cached = camera_y;
        self.camera_x_scroll_cached = camera_x;
    }

    pub(crate) fn save_special_exit_bg2_live_scroll(&mut self) {
        self.special_exit_bg2_h_copy2 = self.bg2_h_copy2;
        self.special_exit_bg2_v_copy2 = self.bg2_v_copy2;
    }

    pub(crate) fn save_exit_bg2_live_scroll(&mut self) {
        self.exit_bg2_h_copy2 = self.bg2_h_copy2;
        self.exit_bg2_v_copy2 = self.bg2_v_copy2;
    }

    pub(crate) fn restore_special_exit_bg2_scroll_to_all_layers(&mut self) {
        self.set_all_layer_h_scrolls(self.special_exit_bg2_h_copy2);
        self.set_all_layer_v_scrolls(self.special_exit_bg2_v_copy2);
    }

    pub(crate) fn restore_exit_bg2_scroll_to_all_layers(&mut self) {
        self.set_all_layer_h_scrolls(self.exit_bg2_h_copy2);
        self.set_all_layer_v_scrolls(self.exit_bg2_v_copy2);
    }

    pub(crate) fn set_all_layer_h_scrolls(&mut self, value: u16) {
        self.set_bg2_h_copy2(value);
        self.set_bg2_h_copy(value);
        self.set_bg1_h_copy2(value);
        self.set_bg1_h_copy(value);
    }

    pub(crate) fn set_all_layer_v_scrolls(&mut self, value: u16) {
        self.set_bg2_v_copy2(value);
        self.set_bg2_v_copy(value);
        self.set_bg1_v_copy2(value);
        self.set_bg1_v_copy(value);
    }

    pub(crate) fn set_map_backup_scrolls(
        &mut self,
        bg1_h: u16,
        bg2_h: u16,
        bg1_v: u16,
        bg2_v: u16,
    ) {
        self.map_backup_bg1_h_copy2 = bg1_h;
        self.map_backup_bg2_h_copy2 = bg2_h;
        self.map_backup_bg1_v_copy2 = bg1_v;
        self.map_backup_bg2_v_copy2 = bg2_v;
    }

    pub(crate) fn clear_bg3_h_copy2(&mut self) {
        self.set_bg3_h_copy2(0);
    }

    pub(crate) fn clear_bg3_v_copy2(&mut self) {
        self.set_bg3_v_copy2(0);
    }

    pub(crate) fn add_bg1_h_copy_low(&mut self, value: u8) {
        Self::add_low_byte(&mut self.bg1_h_copy, value);
    }

    pub(crate) fn add_bg1_v_copy_low(&mut self, value: u8) {
        Self::add_low_byte(&mut self.bg1_v_copy, value);
    }

    pub(crate) fn add_bg2_v_copy_low(&mut self, value: u8) {
        Self::add_low_byte(&mut self.bg2_v_copy, value);
    }

    pub(crate) fn subtract_bg2_h_copy_low(&mut self, value: u8) {
        Self::subtract_low_byte(&mut self.bg2_h_copy, value);
    }

    pub(crate) fn add_bg2_h_copy2_signed(&mut self, value: i8) {
        self.bg2_h_copy2 = self.bg2_h_copy2.wrapping_add(value as i16 as u16);
    }

    pub(crate) fn add_bg2_v_copy2_signed(&mut self, value: i8) {
        self.bg2_v_copy2 = self.bg2_v_copy2.wrapping_add(value as i16 as u16);
    }

    pub(crate) fn add_bg3_v_copy2_signed(&mut self, value: i8) {
        self.bg3_v_copy2 = self.bg3_v_copy2.wrapping_add(value as i16 as u16);
    }

    fn add_subpixel_scroll(subpixel: &mut u16, scroll: &mut u16, value: u32) {
        let current = u32::from(*subpixel) | (u32::from(*scroll) << 16);
        let next = current.wrapping_add(value);
        *subpixel = next as u16;
        *scroll = (next >> 16) as u16;
    }

    fn subtract_subpixel_scroll(subpixel: &mut u16, scroll: &mut u16, value: u32) {
        let current = u32::from(*subpixel) | (u32::from(*scroll) << 16);
        let next = current.wrapping_sub(value);
        *subpixel = next as u16;
        *scroll = (next >> 16) as u16;
    }

    pub(crate) fn clear_bg1_scroll_subpixels(&mut self) {
        self.bg1_h_subpixel = 0;
        self.bg1_v_subpixel = 0;
    }

    pub(crate) fn add_bg1_h_live_subpixel(&mut self, subpixel: u16, scroll: u16) {
        Self::add_subpixel_scroll(
            &mut self.bg1_h_subpixel,
            &mut self.bg1_h_copy2,
            u32::from(subpixel) | (u32::from(scroll) << 16),
        );
    }

    pub(crate) fn add_bg1_v_live_subpixel(&mut self, subpixel: u16, scroll: u16) {
        Self::add_subpixel_scroll(
            &mut self.bg1_v_subpixel,
            &mut self.bg1_v_copy2,
            u32::from(subpixel) | (u32::from(scroll) << 16),
        );
    }

    pub(crate) fn subtract_bg1_v_live_subpixel(&mut self, value: u32) {
        Self::subtract_subpixel_scroll(&mut self.bg1_v_subpixel, &mut self.bg1_v_copy2, value);
    }

    pub(crate) fn add_bg1_h_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
        self.add_bg1_h_live_subpixel(subpixel, scroll);
    }

    pub(crate) fn add_bg1_v_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
        self.add_bg1_v_live_subpixel(subpixel, scroll);
    }

    pub(crate) fn subtract_bg1_v_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
        Self::subtract_subpixel_scroll(
            &mut self.bg1_v_subpixel,
            &mut self.bg1_v_copy2,
            u32::from(subpixel) | (u32::from(scroll) << 16),
        );
    }

    pub(crate) fn set_bg1_h_subpixel(&mut self, value: u16) {
        self.bg1_h_subpixel = value;
    }

    pub(crate) fn set_bg1_v_subpixel(&mut self, value: u16) {
        self.bg1_v_subpixel = value;
    }

    pub(crate) fn step_bg2_h_copy2_toward_cached(&mut self) {
        if self.bg2_h_copy2 != self.bg2_h_copy2_cached {
            self.bg2_h_copy2 = if self.bg2_h_copy2 < self.bg2_h_copy2_cached {
                self.bg2_h_copy2.wrapping_add(1)
            } else {
                self.bg2_h_copy2.wrapping_sub(1)
            };
        }
    }

    pub(crate) fn step_bg2_v_copy2_toward_cached(&mut self) {
        if self.bg2_v_copy2 != self.bg2_v_copy2_cached {
            self.bg2_v_copy2 = if self.bg2_v_copy2 < self.bg2_v_copy2_cached {
                self.bg2_v_copy2.wrapping_add(1)
            } else {
                self.bg2_v_copy2.wrapping_sub(1)
            };
        }
    }

    pub(crate) fn add_bg2_h_copy2(&mut self, value: u16) {
        self.bg2_h_copy2 = self.bg2_h_copy2.wrapping_add(value);
    }

    pub(crate) fn add_bg2_v_copy2(&mut self, value: u16) {
        self.bg2_v_copy2 = self.bg2_v_copy2.wrapping_add(value);
    }

    pub(crate) fn add_bg2_copy2_for_axis_signed(&mut self, vertical: bool, value: i16) {
        if vertical {
            self.bg2_v_copy2 = self.bg2_v_copy2.wrapping_add_signed(value);
        } else {
            self.bg2_h_copy2 = self.bg2_h_copy2.wrapping_add_signed(value);
        }
    }

    pub(crate) fn copy_bg1_live_to_ppu_copy(&mut self) {
        self.bg1_h_copy = self.bg1_h_copy2;
        self.bg1_v_copy = self.bg1_v_copy2;
    }

    pub(crate) fn copy_bg2_live_to_ppu_copy(&mut self) {
        self.bg2_h_copy = self.bg2_h_copy2;
        self.bg2_v_copy = self.bg2_v_copy2;
    }

    pub(crate) fn copy_live_to_ppu_copy(&mut self) {
        self.copy_bg1_live_to_ppu_copy();
        self.copy_bg2_live_to_ppu_copy();
    }

    pub(crate) fn copy_bg2_live_to_bg1_live(&mut self) {
        self.bg1_h_copy2 = self.bg2_h_copy2;
        self.bg1_v_copy2 = self.bg2_v_copy2;
    }

    pub(crate) fn copy_bg2_h_live_to_bg1_h_live(&mut self) {
        self.bg1_h_copy2 = self.bg2_h_copy2;
    }

    pub(crate) fn copy_bg2_v_live_to_bg1_v_live(&mut self) {
        self.bg1_v_copy2 = self.bg2_v_copy2;
    }

    pub(crate) fn set_mapbak_main_tile_theme_index(&mut self, value: u8) {
        self.mapbak_main_tile_theme_index = value;
    }

    pub(crate) fn set_mapbak_sprite_graphics_index(&mut self, value: u8) {
        self.mapbak_sprite_graphics_index = value;
    }

    pub(crate) fn set_mapbak_aux_tile_theme_index(&mut self, value: u8) {
        self.mapbak_aux_tile_theme_index = value;
    }

    pub(crate) fn set_mapbak_bg1_x_offset(&mut self, value: u16) {
        self.mapbak_bg1_x_offset = value;
    }

    pub(crate) fn set_mapbak_bg1_y_offset(&mut self, value: u16) {
        self.mapbak_bg1_y_offset = value;
    }

    pub(crate) fn set_mapbak_cgwsel(&mut self, value: u8) {
        Self::set_low_byte(&mut self.mapbak_cgwsel, value);
    }

    pub(crate) fn set_mapbak_cgwsel_word(&mut self, value: u16) {
        self.mapbak_cgwsel = value;
    }

    pub(crate) fn set_mapbak_hdmaen(&mut self, value: u8) {
        self.mapbak_hdmaen = value;
    }

    pub(crate) fn copy_mapbak_palette_from(&mut self, palette: &[u8]) {
        let len = palette.len().min(MAPBAK_PALETTE_BYTES);
        self.mapbak_palette.resize(MAPBAK_PALETTE_BYTES, 0);
        self.mapbak_palette[..len].copy_from_slice(&palette[..len]);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DisplayState {
    pub(crate) screen_brightness: u8,
    pub(crate) nmi_update_latch: u8,
    pub(crate) core_update_disable_flag: u8,
    pub(crate) pending_nmi_subroutine: u8,
    pub(crate) bg_vram_load_mode: u8,
    pub(crate) pending_tilemap_update_destination_page: u8,
    pub(crate) pending_tilemap_update_source_offset: u16,
    pub(crate) bg_mode: u8,
    pub(crate) main_screen_layers: u8,
    pub(crate) sub_screen_layers: u8,
    pub(crate) bg12_window_selection: u8,
    pub(crate) bg34_window_selection: u8,
    pub(crate) object_color_window_selection: u8,
    pub(crate) main_screen_window_layers: u8,
    pub(crate) sub_screen_window_layers: u8,
    pub(crate) nmi_copy_packets_request: u8,
    pub(crate) pending_polyhedral_update: u8,
    pub(crate) chr_halfslot_request: u8,
    pub(crate) nmi_thread_active: bool,
    pub(crate) nmi_thread_stack_pointer: u16,
    pub(crate) irq_control_flag: u8,
    pub(crate) vertical_irq_trigger: u8,
    pub(crate) sprite_dma_head_pointer: u8,
    pub(crate) sprite_dma_body_pointer: u8,
    pub(crate) hdma_enable_mask: u8,
    pub(crate) mosaic_copy: u8,
    pub(crate) mosaic_level: u8,
    pub(crate) mosaic_target_level: u8,
    pub(crate) mosaic_direction: u8,
    pub(crate) nmi_load_target_address: u16,
    pub(crate) vram_upload_cursor: u16,
    pub(crate) incremental_vram_upload_counter: u8,
    pub(crate) link_dma_sources: LinkDmaSources,
    pub(crate) bg_tile_animation_countdown: u16,
    pub(crate) message_dma_destination_address: u16,
    pub(crate) message_dma_tile_base: u16,
    pub(crate) message_dma_tile_limit: u16,
    pub(crate) message_dma_tile_sentinel: u16,
    pub(crate) overworld_fixed_color_adjustment: u8,
    pub(crate) travel_bird_tile_offset: u8,
    pub(crate) star_tile_restore_phase: u8,
    pub(crate) animated_tile_data_source_address: u16,
    pub(crate) animated_tile_vram_destination_address: u16,
    pub(crate) attract_vram_destination_address: u16,
    pub(crate) palette_buffer: PaletteBufferState,
    pub(crate) palette_filter: PaletteFilterState,
    pub(crate) trinexx_palette: TrinexxPaletteState,
    pub(crate) hud_runtime: HudRuntimeState,
    pub(crate) hud_tilemap: HudTilemapState,
    pub(crate) hud_inventory_order: HudInventoryOrderState,
    pub(crate) water_hdma_window: WaterHdmaWindowState,
    pub(crate) overworld_palette_backup: OverworldPaletteBackupState,
    pub(crate) ppu_scroll_copy: PpuScrollCopyState,
    pub(crate) spotlight_hdma: SpotlightHdmaState,
}

impl DisplayState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            screen_brightness: ram_byte(ram, INIDISP_COPY),
            nmi_update_latch: ram_byte(ram, NMI_BOOLEAN),
            core_update_disable_flag: ram_byte(ram, NMI_DISABLE_CORE_UPDATES),
            pending_nmi_subroutine: ram_byte(ram, NMI_SUBROUTINE_INDEX),
            bg_vram_load_mode: ram_byte(ram, NMI_LOAD_BG_FROM_VRAM),
            pending_tilemap_update_destination_page: ram_byte(ram, NMI_UPDATE_TILEMAP_DST),
            pending_tilemap_update_source_offset: read_le_u16(ram, NMI_UPDATE_TILEMAP_SRC),
            bg_mode: ram_byte(ram, BGMODE_COPY),
            main_screen_layers: ram_byte(ram, TM_COPY),
            sub_screen_layers: ram_byte(ram, TS_COPY),
            bg12_window_selection: ram_byte(ram, W12SEL_COPY),
            bg34_window_selection: ram_byte(ram, W34SEL_COPY),
            object_color_window_selection: ram_byte(ram, WOBJSEL_COPY),
            main_screen_window_layers: ram_byte(ram, TMW_COPY),
            sub_screen_window_layers: ram_byte(ram, TSW_COPY),
            nmi_copy_packets_request: ram_byte(ram, NMI_COPY_PACKETS_FLAG),
            pending_polyhedral_update: ram_byte(ram, NMI_FLAG_UPDATE_POLYHEDRAL),
            chr_halfslot_request: ram_byte(ram, LOAD_CHR_HALFSLOT_EVEN_ODD),
            nmi_thread_active: ram_byte(ram, NMI_THREAD_ACTIVE) != 0,
            nmi_thread_stack_pointer: read_le_u16(ram, POLY_THREAD_STACK),
            irq_control_flag: ram_byte(ram, IRQ_FLAG),
            vertical_irq_trigger: ram_byte(ram, VIRQ_TRIGGER),
            sprite_dma_head_pointer: ram_byte(ram, DMA_HEAD_POINTER),
            sprite_dma_body_pointer: ram_byte(ram, DMA_BODY_POINTER),
            hdma_enable_mask: ram_byte(ram, HDMAEN_COPY),
            mosaic_copy: ram_byte(ram, MOSAIC_COPY),
            mosaic_level: ram_byte(ram, MOSAIC_LEVEL),
            mosaic_target_level: ram_byte(ram, MOSAIC_TARGET_LEVEL),
            mosaic_direction: ram_byte(ram, MOSAIC_INC_OR_DEC),
            nmi_load_target_address: read_le_u16(ram, NMI_LOAD_TARGET_ADDR),
            vram_upload_cursor: read_le_u16(ram, VRAM_UPLOAD_OFFSET),
            incremental_vram_upload_counter: ram_byte(ram, INCREMENTAL_COUNTER_FOR_VRAM),
            link_dma_sources: LinkDmaSources::load_from_ram(ram),
            bg_tile_animation_countdown: read_le_u16(ram, BG_TILE_ANIMATION_COUNTDOWN),
            message_dma_destination_address: read_le_u16(ram, MESSAGE_DMA_DST_ADDR),
            message_dma_tile_base: read_le_u16(ram, MESSAGE_DMA_TILE_BASE),
            message_dma_tile_limit: read_le_u16(ram, MESSAGE_DMA_TILE_LIMIT),
            message_dma_tile_sentinel: read_le_u16(ram, MESSAGE_DMA_TILE_SENTINEL),
            overworld_fixed_color_adjustment: ram_byte(ram, OVERWORLD_FIXED_COLOR_PLUSMINUS),
            travel_bird_tile_offset: ram_byte(ram, FLAG_TRAVEL_BIRD),
            star_tile_restore_phase: ram_byte(ram, STAR_TILE_RESTORE_PHASE),
            animated_tile_data_source_address: read_le_u16(ram, ANIMATED_TILE_DATA_SRC),
            animated_tile_vram_destination_address: read_le_u16(ram, ANIMATED_TILE_VRAM_ADDR),
            attract_vram_destination_address: read_le_u16(ram, ATTRACT_VRAM_DST),
            palette_buffer: PaletteBufferState::load_from_ram(ram),
            palette_filter: PaletteFilterState::load_from_ram(ram),
            trinexx_palette: TrinexxPaletteState::load_from_ram(ram),
            hud_runtime: HudRuntimeState::load_from_ram(ram),
            hud_tilemap: HudTilemapState::load_from_ram(ram),
            hud_inventory_order: HudInventoryOrderState::load_from_ram(ram),
            water_hdma_window: WaterHdmaWindowState::load_from_ram(ram),
            overworld_palette_backup: OverworldPaletteBackupState::load_from_ram(ram),
            ppu_scroll_copy: PpuScrollCopyState::load_from_ram(ram),
            spotlight_hdma: SpotlightHdmaState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[INIDISP_COPY] = self.screen_brightness;
        ram[NMI_BOOLEAN] = self.nmi_update_latch;
        ram[NMI_DISABLE_CORE_UPDATES] = self.core_update_disable_flag;
        ram[NMI_SUBROUTINE_INDEX] = self.pending_nmi_subroutine;
        ram[NMI_LOAD_BG_FROM_VRAM] = self.bg_vram_load_mode;
        ram[NMI_UPDATE_TILEMAP_DST] = self.pending_tilemap_update_destination_page;
        write_le_u16(
            ram,
            NMI_UPDATE_TILEMAP_SRC,
            self.pending_tilemap_update_source_offset,
        );
        ram[BGMODE_COPY] = self.bg_mode;
        ram[TM_COPY] = self.main_screen_layers;
        ram[TS_COPY] = self.sub_screen_layers;
        ram[W12SEL_COPY] = self.bg12_window_selection;
        ram[W34SEL_COPY] = self.bg34_window_selection;
        ram[WOBJSEL_COPY] = self.object_color_window_selection;
        ram[TMW_COPY] = self.main_screen_window_layers;
        ram[TSW_COPY] = self.sub_screen_window_layers;
        ram[NMI_COPY_PACKETS_FLAG] = self.nmi_copy_packets_request;
        ram[NMI_FLAG_UPDATE_POLYHEDRAL] = self.pending_polyhedral_update;
        ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = self.chr_halfslot_request;
        ram[NMI_THREAD_ACTIVE] = u8::from(self.nmi_thread_active);
        write_le_u16(ram, POLY_THREAD_STACK, self.nmi_thread_stack_pointer);
        ram[IRQ_FLAG] = self.irq_control_flag;
        ram[VIRQ_TRIGGER] = self.vertical_irq_trigger;
        ram[DMA_HEAD_POINTER] = self.sprite_dma_head_pointer;
        ram[DMA_BODY_POINTER] = self.sprite_dma_body_pointer;
        ram[HDMAEN_COPY] = self.hdma_enable_mask;
        ram[MOSAIC_COPY] = self.mosaic_copy;
        ram[MOSAIC_LEVEL] = self.mosaic_level;
        ram[MOSAIC_TARGET_LEVEL] = self.mosaic_target_level;
        ram[MOSAIC_INC_OR_DEC] = self.mosaic_direction;
        write_le_u16(ram, NMI_LOAD_TARGET_ADDR, self.nmi_load_target_address);
        write_le_u16(ram, VRAM_UPLOAD_OFFSET, self.vram_upload_cursor);
        ram[INCREMENTAL_COUNTER_FOR_VRAM] = self.incremental_vram_upload_counter;
        self.link_dma_sources.write_to_ram(ram);
        write_le_u16(
            ram,
            BG_TILE_ANIMATION_COUNTDOWN,
            self.bg_tile_animation_countdown,
        );
        write_le_u16(
            ram,
            MESSAGE_DMA_DST_ADDR,
            self.message_dma_destination_address,
        );
        write_le_u16(ram, MESSAGE_DMA_TILE_BASE, self.message_dma_tile_base);
        write_le_u16(ram, MESSAGE_DMA_TILE_LIMIT, self.message_dma_tile_limit);
        write_le_u16(
            ram,
            MESSAGE_DMA_TILE_SENTINEL,
            self.message_dma_tile_sentinel,
        );
        ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = self.overworld_fixed_color_adjustment;
        ram[FLAG_TRAVEL_BIRD] = self.travel_bird_tile_offset;
        ram[STAR_TILE_RESTORE_PHASE] = self.star_tile_restore_phase;
        write_le_u16(
            ram,
            ANIMATED_TILE_DATA_SRC,
            self.animated_tile_data_source_address,
        );
        write_le_u16(
            ram,
            ANIMATED_TILE_VRAM_ADDR,
            self.animated_tile_vram_destination_address,
        );
        write_le_u16(ram, ATTRACT_VRAM_DST, self.attract_vram_destination_address);
        self.palette_buffer.write_to_ram(ram);
        self.palette_filter.write_to_ram(ram);
        self.trinexx_palette.write_to_ram(ram);
        self.hud_runtime.write_to_ram(ram);
        self.hud_tilemap.write_to_ram(ram);
        self.hud_inventory_order.write_to_ram(ram);
        self.water_hdma_window.write_to_ram(ram);
        self.overworld_palette_backup.write_to_ram(ram);
        self.ppu_scroll_copy.write_to_ram(ram);
        self.spotlight_hdma.write_to_ram(ram);
    }

    pub(crate) fn nmi_update_is_latched(&self) -> bool {
        self.nmi_update_latch != 0
    }

    pub(crate) fn core_updates_are_disabled(&self) -> bool {
        self.core_update_disable_flag != 0
    }

    pub(crate) fn has_bg_vram_load(&self) -> bool {
        self.bg_vram_load_mode != 0
    }

    pub(crate) fn has_pending_tilemap_update(&self) -> bool {
        self.pending_tilemap_update_destination_page != 0
    }

    pub(crate) fn pending_tilemap_update_vram_destination(&self) -> usize {
        usize::from(self.pending_tilemap_update_destination_page) * 256
    }

    pub(crate) fn pending_tilemap_update_source_address(&self) -> usize {
        crate::game_state::constants::nmi::BG_CHAR_BUFFER
            + usize::from(self.pending_tilemap_update_source_offset)
    }

    pub(crate) fn pending_tilemap_update_source_data<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[self.pending_tilemap_update_source_address().min(ram.len())..]
    }

    pub(crate) fn layer_masks_word(&self) -> u16 {
        u16::from(self.main_screen_layers) | (u16::from(self.sub_screen_layers) << 8)
    }

    pub(crate) fn has_nmi_copy_packets_request(&self) -> bool {
        self.nmi_copy_packets_request != 0
    }

    pub(crate) fn has_pending_polyhedral_update(&self) -> bool {
        self.pending_polyhedral_update != 0
    }

    pub(crate) fn has_chr_halfslot_request(&self) -> bool {
        self.chr_halfslot_request != 0
    }

    pub(crate) fn nmi_thread_uses_poly_stack(&self) -> bool {
        self.nmi_thread_active && self.nmi_thread_stack_pointer != 0x1f31
    }

    pub(crate) fn has_irq_control_flag(&self) -> bool {
        self.irq_control_flag != 0
    }

    pub(crate) fn irq_control_has_vcounter_marker(&self) -> bool {
        self.irq_control_flag & 0x80 != 0
    }

    pub(crate) fn is_hdma_channel_enabled(&self, channel: usize) -> bool {
        self.hdma_enable_mask & (1 << channel) != 0
    }

    pub(crate) fn sprite_oam_shadow_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[OAM_BUF..]
    }

    pub(crate) fn mosaic_target_level_word(&self) -> u16 {
        u16::from(self.mosaic_target_level)
    }

    pub(crate) fn nmi_load_target_page(&self) -> u8 {
        self.nmi_load_target_address as u8
    }

    pub(crate) fn vram_upload_cursor_usize(&self) -> usize {
        usize::from(self.vram_upload_cursor)
    }

    pub(crate) fn vram_upload_buffer_base(&self) -> usize {
        VRAM_UPLOAD_DATA
    }

    pub(crate) fn vram_upload_buffer_address(&self, offset: usize) -> usize {
        self.vram_upload_buffer_base() + offset
    }

    pub(crate) fn vram_upload_buffer_word(&self, ram: &[u8], offset: usize) -> u16 {
        let address = self.vram_upload_buffer_address(offset);
        if address + 1 < ram.len() {
            read_le_u16(ram, address)
        } else {
            0
        }
    }

    pub(crate) fn vram_upload_tilemap_word(&self, ram: &[u8], offset: usize) -> u16 {
        let address = VRAM_UPLOAD_OFFSET + offset;
        if address + 1 < ram.len() {
            read_le_u16(ram, address)
        } else {
            0
        }
    }

    pub(crate) fn vram_upload_buffer_byte(&self, ram: &[u8], offset: usize) -> u8 {
        ram.get(self.vram_upload_buffer_address(offset))
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn link_dma_source(&self, slot: LinkDmaSourceSlot) -> u16 {
        self.link_dma_sources.source(slot)
    }

    pub(crate) fn vram_upload_buffer_remaining<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[self.vram_upload_buffer_base()..]
    }

    pub(crate) fn nmi_vram_packet_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF..]
    }

    pub(crate) fn overworld_tile_attribute_word(&self, ram: &[u8], index: usize) -> u16 {
        let address = crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + index * 2;
        if address + 1 < ram.len() {
            read_le_u16(ram, address)
        } else {
            0
        }
    }

    pub(crate) fn overworld_tile_upload_word(&self, ram: &[u8], index: usize) -> u16 {
        let address = crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + index * 2;
        if address + 1 < ram.len() {
            read_le_u16(ram, address)
        } else {
            0
        }
    }

    pub(crate) fn tilemap_upload_stripe_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::TILEMAP_UPLOAD_BUFFER..]
    }

    pub(crate) fn secondary_stripe_upload_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::STRIPE_BUFFER_021B..]
    }

    pub(crate) fn background_character_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER..]
    }

    pub(crate) fn background_character_secondary_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER_1..]
    }

    pub(crate) fn background_character_half_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::BG_CHAR_HALF_BUFFER..]
    }

    pub(crate) fn bg1_wall_top_tilemap_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::BG1_WALL_TOP_BUFFER..]
    }

    pub(crate) fn bg1_wall_bottom_tilemap_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::BG1_WALL_BOTTOM_BUFFER..]
    }

    pub(crate) fn game_over_text_tile_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_BUFFER..]
    }

    pub(crate) fn game_over_text_tail_tile_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_TAIL_BUFFER..]
    }

    pub(crate) fn polyhedral_tile_buffer<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[POLYHEDRAL_BUFFER..]
    }

    pub(crate) fn arbitrary_tilemap_destination(&self, ram: &[u8], slot: usize) -> u16 {
        read_le_u16(
            ram,
            crate::game_state::constants::nmi::ARBITRARY_TILEMAP_DST_BUFFER + slot * 2,
        )
    }

    pub(crate) fn dungeon_bg2_attribute_table<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[DUNGEON_BG2_ATTR_TABLE..]
    }

    pub(crate) fn dungeon_bg1_attribute_table<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[DUNGEON_BG1_ATTR_TABLE..]
    }

    pub(crate) fn vram_dma_source_bytes<'a>(
        &self,
        ram: &'a [u8],
        source_addr: usize,
        len: usize,
    ) -> &'a [u8] {
        let start = source_addr.min(ram.len());
        let end = source_addr.saturating_add(len).min(ram.len());
        &ram[start..end]
    }

    pub(crate) fn current_vram_upload_data_address(&self) -> usize {
        self.vram_upload_buffer_address(self.vram_upload_cursor_usize())
    }

    pub(crate) fn incremental_vram_upload_counter_usize(&self) -> usize {
        usize::from(self.incremental_vram_upload_counter)
    }

    pub(crate) fn message_dma_destination_address_usize(&self) -> usize {
        usize::from(self.message_dma_destination_address)
    }

    pub(crate) fn message_dma_tile_indices<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[HUD_TILE_INDICES_BUFFER..]
    }

    pub(crate) fn has_travel_bird_tile_upload(&self) -> bool {
        self.travel_bird_tile_offset != 0
    }

    pub(crate) fn star_tile_restore_source_offsets(&self) -> (usize, usize) {
        if self.star_tile_restore_phase != 0 {
            (32, 0)
        } else {
            (0, 32)
        }
    }

    pub(crate) fn animated_tile_data_source_usize(&self) -> usize {
        usize::from(self.animated_tile_data_source_address)
    }

    pub(crate) fn animated_tile_data<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[self.animated_tile_data_source_usize().min(ram.len())..]
    }

    pub(crate) fn has_animated_tile_data_source(&self) -> bool {
        self.animated_tile_data_source_address != 0
    }

    pub(crate) fn animated_tile_vram_destination_usize(&self) -> usize {
        usize::from(self.animated_tile_vram_destination_address)
    }

    pub(crate) fn attract_vram_destination_high_is_clear(&self) -> bool {
        self.attract_vram_destination_address < 0x0100
    }

    pub(crate) fn attract_vram_destination_page_offset(&self) -> u8 {
        self.attract_vram_destination_address as u8
    }
}

pub(crate) struct NativeAttractVramDestinationBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativeAttractVramDestinationBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        *display = DisplayState::load_from_ram(ram);
        Self { display, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.display.attract_vram_destination_address,
            read_le_u16(self.ram, ATTRACT_VRAM_DST)
        );
    }

    pub(crate) fn set_address(&mut self, value: u16) {
        self.display.attract_vram_destination_address = value;
        write_le_u16(self.ram, ATTRACT_VRAM_DST, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_address(&mut self) {
        self.set_address(0);
    }

    pub(crate) fn set_page_offset(&mut self, value: u8) {
        self.display.attract_vram_destination_address =
            (self.display.attract_vram_destination_address & 0xff00) | u16::from(value);
        self.ram[ATTRACT_VRAM_DST] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_page_offset(&mut self) {
        let next = self
            .display
            .attract_vram_destination_page_offset()
            .wrapping_sub(1);
        self.set_page_offset(next);
    }

    pub(crate) fn decrement_address(&mut self) -> u16 {
        let next = self
            .display
            .attract_vram_destination_address
            .wrapping_sub(1);
        self.set_address(next);
        next
    }
}

pub(crate) struct NativePaletteFilterBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativePaletteFilterBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        display.palette_filter = PaletteFilterState::load_from_ram(ram);
        Self { display, ram }
    }

    fn sync(&mut self) {
        self.display.palette_filter.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.display.palette_filter,
            PaletteFilterState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.display.palette_filter.countdown = value;
        self.sync();
    }

    pub(crate) fn increment_countdown(&mut self) {
        self.display.palette_filter.countdown =
            self.display.palette_filter.countdown.wrapping_add(1);
        self.sync();
    }

    pub(crate) fn decrement_countdown(&mut self) {
        self.display.palette_filter.countdown =
            self.display.palette_filter.countdown.wrapping_sub(1);
        self.sync();
    }

    pub(crate) fn set_countdown_word(&mut self, value: u16) {
        self.display.palette_filter.countdown = value as u8;
        self.display.palette_filter.countdown_high = (value >> 8) as u8;
        self.sync();
    }

    pub(crate) fn set_darkening_or_lightening_screen(&mut self, value: u8) {
        self.display.palette_filter.darkening_or_lightening_screen = value;
        self.sync();
    }

    pub(crate) fn xor_darkening_or_lightening_screen(&mut self, value: u8) {
        self.display.palette_filter.darkening_or_lightening_screen ^= value;
        self.sync();
    }

    pub(crate) fn set_darkening_or_lightening_screen_word(&mut self, value: u16) {
        self.display.palette_filter.darkening_or_lightening_screen = value as u8;
        self.display
            .palette_filter
            .darkening_or_lightening_screen_high = (value >> 8) as u8;
        self.sync();
    }

    pub(crate) fn set_color_window_selection(&mut self, value: u8) {
        self.display.palette_filter.color_window_selection = value;
        self.sync();
    }

    pub(crate) fn set_color_window_and_math_word(&mut self, value: u16) {
        self.display.palette_filter.color_window_selection = value as u8;
        self.display.palette_filter.color_math_control = (value >> 8) as u8;
        self.sync();
    }

    pub(crate) fn set_color_math_control(&mut self, value: u8) {
        self.display.palette_filter.color_math_control = value;
        self.sync();
    }

    pub(crate) fn set_fixed_color_red(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_red = value;
        self.sync();
    }

    pub(crate) fn or_fixed_color_red(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_red |= value;
        self.sync();
    }

    pub(crate) fn subtract_fixed_color_red(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_red = self
            .display
            .palette_filter
            .fixed_color_red
            .wrapping_sub(value);
        self.sync();
    }

    pub(crate) fn set_fixed_color_green(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_green = value;
        self.sync();
    }

    pub(crate) fn or_fixed_color_green(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_green |= value;
        self.sync();
    }

    pub(crate) fn subtract_fixed_color_green(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_green = self
            .display
            .palette_filter
            .fixed_color_green
            .wrapping_sub(value);
        self.sync();
    }

    pub(crate) fn set_fixed_color_blue(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_blue = value;
        self.sync();
    }

    pub(crate) fn or_fixed_color_blue(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_blue |= value;
        self.sync();
    }

    pub(crate) fn subtract_fixed_color_blue(&mut self, value: u8) {
        self.display.palette_filter.fixed_color_blue = self
            .display
            .palette_filter
            .fixed_color_blue
            .wrapping_sub(value);
        self.sync();
    }

    pub(crate) fn set_fixed_color_component(&mut self, index: usize, value: u8) {
        match index {
            0 => self.display.palette_filter.fixed_color_red = value,
            1 => self.display.palette_filter.fixed_color_green = value,
            2 => self.display.palette_filter.fixed_color_blue = value,
            _ => return,
        }
        self.sync();
    }

    pub(crate) fn or_fixed_color_component(&mut self, index: usize, value: u8) {
        match index {
            0 => self.display.palette_filter.fixed_color_red |= value,
            1 => self.display.palette_filter.fixed_color_green |= value,
            2 => self.display.palette_filter.fixed_color_blue |= value,
            _ => return,
        }
        self.sync();
    }
}

pub(crate) struct NativeHudInventoryOrderBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativeHudInventoryOrderBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        *display = DisplayState::load_from_ram(ram);
        Self { display, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.display.hud_inventory_order,
            HudInventoryOrderState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn initialize_default_order(&mut self, count: usize) {
        self.display
            .hud_inventory_order
            .initialize_default_order(count);
        for index in 0..count.min(HUD_INVENTORY_ORDER_CAPACITY) {
            self.ram[HUD_INVENTORY_ORDER + index] = index as u8 + 1;
        }
        self.debug_assert_matches_ram();
    }

    pub(crate) fn swap_items(&mut self, old_pos: usize, new_pos: usize) {
        self.display
            .hud_inventory_order
            .swap_items(old_pos, new_pos);
        if old_pos < HUD_INVENTORY_ORDER_CAPACITY && new_pos < HUD_INVENTORY_ORDER_CAPACITY {
            self.ram
                .swap(HUD_INVENTORY_ORDER + old_pos, HUD_INVENTORY_ORDER + new_pos);
        }
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeHudStateBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativeHudStateBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        display.hud_runtime = HudRuntimeState::load_from_ram(ram);
        display.hud_tilemap = HudTilemapState::load_from_ram(ram);
        Self { display, ram }
    }

    fn sync_runtime(&mut self) {
        self.display.hud_runtime.write_to_ram(self.ram);
        self.debug_assert_runtime_matches_ram();
    }

    fn debug_assert_runtime_matches_ram(&self) {
        debug_assert_eq!(
            self.display.hud_runtime,
            HudRuntimeState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_floor_changed_timer(&mut self, value: u16) {
        self.display.hud_tilemap.set_floor_changed_timer(value);
        write_le_u16(self.ram, HUD_FLOOR_CHANGED_TIMER, value);
    }

    pub(crate) fn set_super_bomb_indicator_timer(&mut self, value: u8) {
        self.display.hud_runtime.super_bomb_indicator_timer = value;
        self.sync_runtime();
    }

    pub(crate) fn set_super_bomb_indicator_counter(&mut self, value: u8) {
        self.display.hud_runtime.super_bomb_indicator_counter = value;
        self.sync_runtime();
    }

    pub(crate) fn set_rupee_sfx_sound_delay(&mut self, value: u8) {
        self.display.hud_runtime.rupee_sfx_sound_delay = value;
        self.sync_runtime();
    }

    pub(crate) fn set_is_doing_heart_animation(&mut self, value: u8) {
        self.display.hud_runtime.heart_animation_active = value;
        self.sync_runtime();
    }

    pub(crate) fn set_tile_word(&mut self, tile: usize, value: u16) {
        let offset = tile * 2;
        if offset + 1 < self.display.hud_tilemap.tile_indices.len() {
            self.display.hud_tilemap.set_tile_word(tile, value);
            write_le_u16(self.ram, HUD_TILE_INDICES_BUFFER + offset, value);
        }
    }

    pub(crate) fn clear_is_doing_heart_animation(&mut self) {
        self.set_is_doing_heart_animation(0);
    }

    pub(crate) fn set_heart_refill_countdown(&mut self, value: u8) {
        self.display.hud_runtime.heart_refill_countdown = value;
        self.sync_runtime();
    }

    pub(crate) fn set_heart_refill_anim_subpos(&mut self, value: u8) {
        self.display.hud_runtime.heart_refill_animation_subpixel = value;
        self.sync_runtime();
    }

    pub(crate) fn set_flashing_circle_timer(&mut self, value: u8) {
        self.display.hud_runtime.flashing_circle_timer = value;
        self.sync_runtime();
    }

    pub(crate) fn set_prev_joypad_h(&mut self, value: u8) {
        self.display.hud_runtime.previous_menu_joypad_h = value;
        self.sync_runtime();
    }

    pub(crate) fn clear_prev_joypad_h(&mut self) {
        self.set_prev_joypad_h(0);
    }

    pub(crate) fn set_equipment_menu_exit_state(&mut self, value: u8) {
        self.display.hud_runtime.equipment_menu_exit_state = value;
        self.sync_runtime();
    }

    pub(crate) fn set_bottle_menu_row(&mut self, value: u8) {
        self.display.hud_runtime.bottle_menu_row = value;
        self.sync_runtime();
    }

    pub(crate) fn decrement_bottle_menu_row(&mut self) -> u8 {
        let row = self.display.hud_runtime.bottle_menu_row.wrapping_sub(1);
        self.set_bottle_menu_row(row);
        row
    }

    pub(crate) fn set_dungeon_dark_with_lantern(&mut self) {
        self.display.hud_runtime.dungeon_dark_with_lantern = 1;
        self.sync_runtime();
    }

    pub(crate) fn set_tick_counter(&mut self, value: u8) {
        self.display.hud_runtime.module_tick_counter = value;
        self.sync_runtime();
    }

    pub(crate) fn clear_floor_changed_timer_low(&mut self) {
        self.display.hud_tilemap.clear_floor_changed_timer_low();
        self.ram[HUD_FLOOR_CHANGED_TIMER] = 0;
    }
}

pub(crate) struct NativeTrinexxPaletteBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativeTrinexxPaletteBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        *display = DisplayState::load_from_ram(ram);
        Self { display, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.display.trinexx_palette,
            TrinexxPaletteState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_red_shell_delay(&mut self, value: u8) {
        self.display.trinexx_palette.red_shell_delay = value;
        self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_blue_shell_delay(&mut self, value: u8) {
        self.display.trinexx_palette.blue_shell_delay = value;
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_red_shell_step(&mut self, value: u8) {
        self.display.trinexx_palette.red_shell_step = value;
        self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_blue_shell_step(&mut self, value: u8) {
        self.display.trinexx_palette.blue_shell_step = value;
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_red_shell_delay(&mut self) {
        self.display.trinexx_palette.decrement_red_shell_delay();
        self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] =
            self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY].wrapping_sub(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_blue_shell_delay(&mut self) {
        self.display.trinexx_palette.decrement_blue_shell_delay();
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] =
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY].wrapping_sub(1);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_red_shell_step(&mut self) -> u8 {
        let value = self.display.trinexx_palette.increment_red_shell_step();
        self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] =
            self.ram[TRINEXX_RED_SHELL_PALETTE_STEP].wrapping_add(1);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn increment_blue_shell_step(&mut self) -> u8 {
        let value = self.display.trinexx_palette.increment_blue_shell_step();
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] =
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP].wrapping_add(1);
        self.debug_assert_matches_ram();
        value
    }
}

pub(crate) struct NativeWaterHdmaWindowBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativeWaterHdmaWindowBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        *display = DisplayState::load_from_ram(ram);
        Self { display, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.display.water_hdma_window,
            WaterHdmaWindowState::load_from_ram(self.ram)
        );
        debug_assert_eq!(
            self.display.spotlight_hdma,
            SpotlightHdmaState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn decrement_watergate_spotlight_y_upper(&mut self) -> u16 {
        let value = self
            .display
            .water_hdma_window
            .decrement_watergate_spotlight_y_upper();
        write_le_u16(self.ram, WATERGATE_SPOTLIGHT_Y_UPPER, value);
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_watergate_pointer(&mut self, value: u8) {
        self.display.water_hdma_window.set_watergate_pointer(value);
        self.ram[WATERGATE_POINTER] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_watergate_pointer(&mut self) -> u8 {
        let value = self.display.water_hdma_window.increment_watergate_pointer();
        self.ram[WATERGATE_POINTER] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_watergate_tilemap_pos_x2(&mut self, value: u16) {
        self.display
            .water_hdma_window
            .set_watergate_tilemap_pos_x2(value);
        write_le_u16(self.ram, WATERGATE_POS, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_window_x(&mut self, value: u16) {
        self.display.water_hdma_window.set_window_x(value);
        write_le_u16(self.ram, WATER_HDMA_WINDOW_X, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_window_y(&mut self, value: u16) {
        self.display.water_hdma_window.set_window_y(value);
        write_le_u16(self.ram, WATER_HDMA_WINDOW_Y, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_window_x_radius(&mut self, value: u16) {
        self.display.water_hdma_window.set_window_x_radius(value);
        write_le_u16(self.ram, WATER_HDMA_WINDOW_X_RADIUS, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_window_y_radius_byte(&mut self, value: u8) {
        self.display
            .water_hdma_window
            .set_window_y_radius_byte(value);
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn increment_window_y_radius_byte(&mut self) -> u8 {
        let value = self
            .display
            .water_hdma_window
            .increment_window_y_radius_byte();
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS] = value;
        self.debug_assert_matches_ram();
        value
    }

    pub(crate) fn set_watergate_spotlight_y_upper(&mut self, value: u16) {
        self.display
            .water_hdma_window
            .set_watergate_spotlight_y_upper(value);
        write_le_u16(self.ram, WATERGATE_SPOTLIGHT_Y_UPPER, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn copy_watergate_spotlight_to_spotlight_upper(&mut self) {
        let value = read_le_u16(self.ram, WATERGATE_SPOTLIGHT_Y_UPPER);
        self.display.spotlight_hdma.set_y_upper(value);
        write_le_u16(self.ram, SPOTLIGHT_Y_UPPER, value);
    }

    pub(crate) fn advance_watergate_window_y_radius(&mut self) -> u8 {
        self.copy_watergate_spotlight_to_spotlight_upper();
        self.display.spotlight_hdma.increment_window_y_buffer_byte();
        self.ram[SPOTLIGHT_WINDOW_Y_BUFFER] = self.display.spotlight_hdma.window_y_buffer_byte();
        let x_radius_minus_margin = self.ram[WATER_HDMA_WINDOW_X_RADIUS].wrapping_sub(8);
        let value = self.ram[SPOTLIGHT_WINDOW_Y_BUFFER].wrapping_add(x_radius_minus_margin);
        self.display
            .water_hdma_window
            .set_window_y_radius_byte(value);
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS] = value;
        self.debug_assert_matches_ram();
        value
    }
}

pub(crate) struct NativeVramUploadBufferBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativeVramUploadBufferBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        *display = DisplayState::load_from_ram(ram);
        Self { display, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.display.vram_upload_cursor,
            read_le_u16(self.ram, VRAM_UPLOAD_OFFSET)
        );
    }

    pub(crate) fn set_offset(&mut self, value: u16) {
        self.display.vram_upload_cursor = value;
        write_le_u16(self.ram, VRAM_UPLOAD_OFFSET, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn clear_offset(&mut self) {
        self.set_offset(0);
    }

    pub(crate) fn advance_offset_by(&mut self, value: u16) -> u16 {
        let next = self.display.vram_upload_cursor.wrapping_add(value);
        self.set_offset(next);
        next
    }

    pub(crate) fn write_buffer_byte(&mut self, offset: usize, value: u8) {
        self.ram[VRAM_UPLOAD_DATA + offset] = value;
    }

    pub(crate) fn write_buffer_word(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, VRAM_UPLOAD_DATA + offset, value);
    }

    pub(crate) fn write_tilemap_word(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, VRAM_UPLOAD_OFFSET + offset, value);
    }

    pub(crate) fn write_overworld_vram_word(&mut self, word_index: usize, value: u16) {
        write_le_u16(self.ram, UVRAM_DATA + word_index * 2, value);
    }

    pub(crate) fn write_absolute_byte(&mut self, address: usize, value: u8) {
        self.ram[address] = value;
    }

    pub(crate) fn write_absolute_word(&mut self, address: usize, value: u16) {
        write_le_u16(self.ram, address, value);
    }

    pub(crate) fn copy_buffer_bytes(&mut self, offset: usize, data: &[u8]) {
        self.ram[VRAM_UPLOAD_DATA + offset..VRAM_UPLOAD_DATA + offset + data.len()]
            .copy_from_slice(data);
    }

    pub(crate) fn terminate_buffer_at(&mut self, offset: usize) {
        self.ram[VRAM_UPLOAD_DATA + offset] = 0xff;
    }

    pub(crate) fn write_level_label_tiles(&mut self, left: &[u8; 14], right: &[u8; 14]) {
        self.ram[VRAM_UPLOAD_DATA + 32] = 0xff;
        for i in (0..14).rev() {
            self.ram[VRAM_UPLOAD_DATA + i] = left[i];
            self.ram[VRAM_UPLOAD_DATA + i + 16] = right[i];
        }
    }

    pub(crate) fn write_map16_update_packet(
        &mut self,
        address: usize,
        vram_pos: u16,
        tiles: [u16; 4],
    ) {
        write_le_u16(self.ram, address, vram_pos.swap_bytes());
        write_le_u16(self.ram, address + 2, 0x0300);
        write_le_u16(self.ram, address + 4, tiles[0]);
        write_le_u16(self.ram, address + 6, tiles[1]);
        write_le_u16(
            self.ram,
            address + 8,
            vram_pos.wrapping_add(0x20).swap_bytes(),
        );
        write_le_u16(self.ram, address + 10, 0x0300);
        write_le_u16(self.ram, address + 12, tiles[2]);
        write_le_u16(self.ram, address + 14, tiles[3]);
        write_le_u16(self.ram, address + 16, 0xffff);
    }

    pub(crate) fn write_single_tile_stripe_packet(
        &mut self,
        address: usize,
        stripe: u16,
        tile: u16,
    ) {
        write_le_u16(self.ram, address, stripe);
        write_le_u16(self.ram, address + 2, 0x0100);
        write_le_u16(self.ram, address + 4, tile);
    }

    pub(crate) fn write_tile_stripe_sentinel(&mut self, address: usize) {
        write_le_u16(self.ram, address, 0xffff);
    }
}

pub(crate) struct NativeGraphicsScratchBridgeMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> NativeGraphicsScratchBridgeMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_aux_bg_subset_pack(&mut self, index: usize, value: u8) {
        self.ram[AUX_BG_SUBSET_0 + index] = value;
    }

    pub(crate) fn primary_decompression_buffer(&self, len: usize) -> Vec<u8> {
        self.ram[PRIMARY_DECOMP_BUFFER_LOAD_GFX..PRIMARY_DECOMP_BUFFER_LOAD_GFX + len].to_vec()
    }

    pub(crate) fn combined_decompression_buffers(&self) -> Vec<u8> {
        self.primary_decompression_buffer(0x0c00)
    }

    pub(crate) fn copy_to_primary_decompression_buffer(&mut self, data: &[u8]) {
        let len = data.len().min(
            self.ram
                .len()
                .saturating_sub(PRIMARY_DECOMP_BUFFER_LOAD_GFX),
        );
        self.ram[PRIMARY_DECOMP_BUFFER_LOAD_GFX..PRIMARY_DECOMP_BUFFER_LOAD_GFX + len]
            .copy_from_slice(&data[..len]);
    }

    pub(crate) fn copy_decompressed_graphics_to(&mut self, dst: usize, data: &[u8]) -> usize {
        let len = data.len().min(self.ram.len().saturating_sub(dst));
        self.ram[dst..dst + len].copy_from_slice(&data[..len]);
        len
    }

    pub(crate) fn rotate_animated_dungeon_tile_planes(&mut self) {
        for i in 0..256 {
            let base = 0x9000 + i * 2;
            let x = read_le_u16(self.ram, base + 0x1880);
            let a = read_le_u16(self.ram, base + 0x1c80);
            let b = read_le_u16(self.ram, base + 0x1e80);
            let c = read_le_u16(self.ram, base + 0x1a80);
            write_le_u16(self.ram, base + 0x1880, a);
            write_le_u16(self.ram, base + 0x1c80, b);
            write_le_u16(self.ram, base + 0x1e80, c);
            write_le_u16(self.ram, base + 0x1a80, x);
        }
    }

    pub(crate) fn write_expanded_tile_row(
        &mut self,
        dst: usize,
        low_plane: u8,
        high_plane: u8,
        upper_plane: u8,
        composite_plane: u8,
    ) {
        self.ram[dst] = low_plane;
        self.ram[dst + 1] = high_plane;
        self.ram[dst + 0x10] = upper_plane;
        self.ram[dst + 0x11] = composite_plane;
    }

    pub(crate) fn set_dungeon_line_pointer_row0(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_LINE_PTRS_ROW0 + index * 2, value);
    }

    pub(crate) fn copy_message_rows(&mut self, dst: usize, src0: usize, src1: usize, len: usize) {
        for i in 0..len {
            self.ram[MESSAGING_RENDER_BUFFER + dst + i] = self.ram[src0 + i];
            self.ram[MESSAGING_RENDER_BUFFER + dst + len + i] = self.ram[src1 + i];
        }
    }

    pub(crate) fn copy_peg_tile_graphics_to_message_buffer(&mut self, first: usize, second: usize) {
        for i in 0..64 {
            let color = read_le_u16(self.ram, PEG_TILE_GFX_BUFFER + (first >> 1) * 2 + i * 2);
            write_le_u16(self.ram, MESSAGING_BUF_LOAD_GFX + i * 2, color);
        }
        for i in 0..64 {
            let color = read_le_u16(self.ram, PEG_TILE_GFX_BUFFER + (second >> 1) * 2 + i * 2);
            write_le_u16(self.ram, MESSAGING_BUF_LOAD_GFX + (64 + i) * 2, color);
        }
    }

    pub(crate) fn copy_dynamic_hdma_table_to_reserved(&mut self, count: usize) {
        let bytes = count * 2;
        let src = self.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + bytes].to_vec();
        self.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + bytes].copy_from_slice(&src);
    }

    pub(crate) fn clear_agahnim_palette_settings(&mut self, len: usize) {
        self.ram[AGAHNIM_PAL_SETTING..AGAHNIM_PAL_SETTING + len].fill(0);
    }

    pub(crate) fn agahnim_palette_word(&self, index: usize) -> u16 {
        read_le_u16(self.ram, AGAHNIM_PAL_SETTING + index * 2)
    }

    pub(crate) fn set_agahnim_palette_word(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, AGAHNIM_PAL_SETTING + index * 2, value);
    }

    pub(crate) fn sprite_decompression_buffer_tail(&self) -> Vec<u8> {
        self.ram[SPRITE_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec()
    }

    pub(crate) fn staged_bg_and_sprite_decompression_buffers(&self) -> Vec<u8> {
        self.ram[BG_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec()
    }
}

pub(crate) struct NativeOverworldPaletteBackupBridgeMut<'a> {
    backup: &'a mut OverworldPaletteBackupState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldPaletteBackupBridgeMut<'a> {
    pub(crate) fn new(backup: &'a mut OverworldPaletteBackupState, ram: &'a mut [u8]) -> Self {
        *backup = OverworldPaletteBackupState::load_from_ram(ram);
        Self { backup, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.backup,
            OverworldPaletteBackupState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_main_indoors_backup(&mut self, value: u8) {
        self.backup.set_main_indoors(value);
        self.ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_aux3_bg_palette_7_backup(&mut self, value: u8) {
        self.backup.set_aux3_bg_palette_7(value);
        self.ram[OVERWORLD_PAL_AUX3_BP7_BACKUP] = value;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_main_indoors_copy_backup(&mut self, value: u8) {
        self.backup.set_main_indoors_copy(value);
        self.ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP] = value;
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeSpotlightHdmaBridgeMut<'a> {
    state: &'a mut SpotlightHdmaState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpotlightHdmaBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpotlightHdmaState, ram: &'a mut [u8]) -> Self {
        *state = SpotlightHdmaState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, SpotlightHdmaState::load_from_ram(self.ram));
    }

    pub(crate) fn set_y_lower(&mut self, value: u16) {
        self.state.set_y_lower(value);
        self.sync();
    }

    pub(crate) fn set_y_upper(&mut self, value: u16) {
        self.state.set_y_upper(value);
        self.sync();
    }

    pub(crate) fn set_window_x_center(&mut self, value: u16) {
        self.state.set_window_x_center(value);
        self.sync();
    }

    pub(crate) fn set_window_state(&mut self, value: u16) {
        self.state.set_window_state(value);
        self.sync();
    }

    pub(crate) fn set_window_radius(&mut self, value: u16) {
        self.state.set_window_radius(value);
        self.sync();
    }

    pub(crate) fn set_window_y_buffer(&mut self, value: u16) {
        self.state.set_window_y_buffer(value);
        self.sync();
    }

    pub(crate) fn decrement_window_y_buffer(&mut self) -> u16 {
        let value = self.state.decrement_window_y_buffer();
        self.sync();
        value
    }

    pub(crate) fn set_window_radius_byte(&mut self, value: u8) {
        self.state.set_window_radius_byte(value);
        self.sync();
    }

    pub(crate) fn set_window_state_byte(&mut self, value: u8) {
        self.state.set_window_state_byte(value);
        self.sync();
    }

    pub(crate) fn set_window_y_buffer_byte(&mut self, value: u8) {
        self.state.set_window_y_buffer_byte(value);
        self.sync();
    }

    pub(crate) fn increment_window_y_buffer_byte(&mut self) {
        self.state.increment_window_y_buffer_byte();
        self.sync();
    }

    pub(crate) fn shr_window_radius_byte(&mut self, shift: u8) {
        self.state.shr_window_radius_byte(shift);
        self.sync();
    }

    pub(crate) fn add_window_radius_byte(&mut self, value: u8) {
        self.state.add_window_radius_byte(value);
        self.sync();
    }

    pub(crate) fn hdma_table_dynamic_entry(&self, index: usize) -> u16 {
        self.state.hdma_table_dynamic_entry(index)
    }

    pub(crate) fn set_hdma_table_dynamic_entry(&mut self, index: usize, value: u16) {
        self.state.set_hdma_table_dynamic_entry(index, value);
        self.sync();
    }

    pub(crate) fn clear_hdma_table_dynamic(&mut self, count: usize) {
        self.state.clear_hdma_table_dynamic(count);
        self.sync();
    }

    pub(crate) fn clear_hdma_table_dynamic_range(&mut self, start: usize, count: usize) {
        self.state.clear_hdma_table_dynamic_range(start, count);
        self.sync();
    }
}

macro_rules! ppu_scroll_bridge_methods {
    ($(fn $name:ident($($arg:ident: $ty:ty),*);)*) => {
        $(
            pub(crate) fn $name(&mut self, $($arg: $ty),*) {
                self.state.$name($($arg),*);
                self.sync();
            }
        )*
    };
}

pub(crate) struct NativePpuScrollCopyBridgeMut<'a> {
    state: &'a mut PpuScrollCopyState,
    ram: &'a mut [u8],
}

impl<'a> NativePpuScrollCopyBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut PpuScrollCopyState, ram: &'a mut [u8]) -> Self {
        *state = PpuScrollCopyState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, PpuScrollCopyState::load_from_ram(self.ram));
    }

    ppu_scroll_bridge_methods! {
        fn set_mapbak_tm(value: u8);
        fn set_mapbak_ts(value: u8);
        fn set_mapbak_tm_word(value: u16);
        fn set_bg1_h_high(value: u8);
        fn set_bg1_h_copy(value: u16);
        fn set_bg1_v_copy(value: u16);
        fn set_bg2_h_copy(value: u16);
        fn set_bg2_v_copy(value: u16);
        fn set_bg1_h_copy_low(value: u8);
        fn set_bg1_v_copy_low(value: u8);
        fn set_bg2_h_copy_low(value: u8);
        fn set_bg2_v_copy_low(value: u8);
        fn set_bg1_h_copy2(value: u16);
        fn set_bg1_v_copy2(value: u16);
        fn set_bg2_h_copy2(value: u16);
        fn set_bg2_v_copy2(value: u16);
        fn set_bg3_h_copy2(value: u16);
        fn set_bg3_v_copy2(value: u16);
        fn set_bg3_v_copy2_low(value: u8);
        fn set_mode7_center_x(value: u16);
        fn set_mode7_center_y(value: u16);
        fn set_mode7_center(x: u16, y: u16);
        fn set_bg1_h_live_and_copy(value: u16);
        fn set_bg1_v_live_and_copy(value: u16);
        fn set_bg2_h_live_and_copy(value: u16);
        fn set_bg2_v_live_and_copy(value: u16);
        fn set_bg1_bg2_h_live_and_copy(value: u16);
        fn set_bg1_bg2_v_live_and_copy(value: u16);
        fn set_bg1_bg2_live_and_copy(bg2_h: u16, bg2_v: u16, bg1_h: u16, bg1_v: u16);
        fn set_bg2_h_copy2_cached(value: u16);
        fn set_bg2_v_copy2_cached(value: u16);
        fn cache_bg2_live_scroll();
        fn save_special_exit_bg2_live_scroll();
        fn save_exit_bg2_live_scroll();
        fn restore_special_exit_bg2_scroll_to_all_layers();
        fn restore_exit_bg2_scroll_to_all_layers();
        fn set_all_layer_h_scrolls(value: u16);
        fn set_all_layer_v_scrolls(value: u16);
        fn set_map_backup_scrolls(bg1_h: u16, bg2_h: u16, bg1_v: u16, bg2_v: u16);
        fn clear_bg3_h_copy2();
        fn clear_bg3_v_copy2();
        fn add_bg1_h_copy_low(value: u8);
        fn add_bg1_v_copy_low(value: u8);
        fn add_bg2_v_copy_low(value: u8);
        fn subtract_bg2_h_copy_low(value: u8);
        fn add_bg2_h_copy2_signed(value: i8);
        fn add_bg2_v_copy2_signed(value: i8);
        fn add_bg3_v_copy2_signed(value: i8);
        fn clear_bg1_scroll_subpixels();
        fn add_bg1_h_live_subpixel(subpixel: u16, scroll: u16);
        fn add_bg1_v_live_subpixel(subpixel: u16, scroll: u16);
        fn subtract_bg1_v_live_subpixel(value: u32);
        fn add_bg1_h_copy2_subpixel(subpixel: u16, scroll: u16);
        fn add_bg1_v_copy2_subpixel(subpixel: u16, scroll: u16);
        fn subtract_bg1_v_copy2_subpixel(subpixel: u16, scroll: u16);
        fn set_bg1_h_subpixel(value: u16);
        fn set_bg1_v_subpixel(value: u16);
        fn step_bg2_h_copy2_toward_cached();
        fn step_bg2_v_copy2_toward_cached();
        fn add_bg2_h_copy2(value: u16);
        fn add_bg2_v_copy2(value: u16);
        fn add_bg2_copy2_for_axis_signed(vertical: bool, value: i16);
        fn copy_bg1_live_to_ppu_copy();
        fn copy_bg2_live_to_ppu_copy();
        fn copy_live_to_ppu_copy();
        fn copy_bg2_live_to_bg1_live();
        fn copy_bg2_h_live_to_bg1_h_live();
        fn copy_bg2_v_live_to_bg1_v_live();
        fn set_mapbak_main_tile_theme_index(value: u8);
        fn set_mapbak_sprite_graphics_index(value: u8);
        fn set_mapbak_aux_tile_theme_index(value: u8);
        fn set_mapbak_bg1_x_offset(value: u16);
        fn set_mapbak_bg1_y_offset(value: u16);
        fn set_mapbak_cgwsel(value: u8);
        fn set_mapbak_cgwsel_word(value: u16);
        fn set_mapbak_hdmaen(value: u8);
        fn copy_mapbak_palette_from(palette: &[u8]);
    }

    pub(crate) fn cache_camera_scroll(&mut self) {
        let camera_y = read_le_u16(self.ram, CAMERA_Y_COORD_SCROLL_LOW);
        let camera_x = read_le_u16(self.ram, CAMERA_X_COORD_SCROLL_LOW);
        self.state.cache_camera_scroll(camera_y, camera_x);
        self.sync();
    }
}

pub(crate) struct NativeDisplayStateBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram: &'a mut [u8],
}

impl<'a> NativeDisplayStateBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        *display = DisplayState::load_from_ram(ram);
        Self { display, ram }
    }

    fn debug_assert_screen_brightness_matches_ram(&self) {
        debug_assert_eq!(
            self.display.screen_brightness,
            ram_byte(self.ram, INIDISP_COPY)
        );
    }

    fn debug_assert_nmi_update_latch_matches_ram(&self) {
        debug_assert_eq!(
            self.display.nmi_update_latch,
            ram_byte(self.ram, NMI_BOOLEAN)
        );
    }

    fn debug_assert_core_update_disable_flag_matches_ram(&self) {
        debug_assert_eq!(
            self.display.core_update_disable_flag,
            ram_byte(self.ram, NMI_DISABLE_CORE_UPDATES)
        );
    }

    fn debug_assert_pending_nmi_subroutine_matches_ram(&self) {
        debug_assert_eq!(
            self.display.pending_nmi_subroutine,
            ram_byte(self.ram, NMI_SUBROUTINE_INDEX)
        );
    }

    fn debug_assert_bg_vram_load_mode_matches_ram(&self) {
        debug_assert_eq!(
            self.display.bg_vram_load_mode,
            ram_byte(self.ram, NMI_LOAD_BG_FROM_VRAM)
        );
    }

    fn debug_assert_pending_tilemap_update_matches_ram(&self) {
        debug_assert_eq!(
            self.display.pending_tilemap_update_destination_page,
            ram_byte(self.ram, NMI_UPDATE_TILEMAP_DST)
        );
        debug_assert_eq!(
            self.display.pending_tilemap_update_source_offset,
            read_le_u16(self.ram, NMI_UPDATE_TILEMAP_SRC)
        );
    }

    fn debug_assert_bg_mode_matches_ram(&self) {
        debug_assert_eq!(self.display.bg_mode, ram_byte(self.ram, BGMODE_COPY));
    }

    fn debug_assert_screen_layer_masks_match_ram(&self) {
        debug_assert_eq!(self.display.main_screen_layers, ram_byte(self.ram, TM_COPY));
        debug_assert_eq!(self.display.sub_screen_layers, ram_byte(self.ram, TS_COPY));
    }

    fn debug_assert_window_layer_masks_match_ram(&self) {
        debug_assert_eq!(
            self.display.bg12_window_selection,
            ram_byte(self.ram, W12SEL_COPY)
        );
        debug_assert_eq!(
            self.display.bg34_window_selection,
            ram_byte(self.ram, W34SEL_COPY)
        );
        debug_assert_eq!(
            self.display.object_color_window_selection,
            ram_byte(self.ram, WOBJSEL_COPY)
        );
        debug_assert_eq!(
            self.display.main_screen_window_layers,
            ram_byte(self.ram, TMW_COPY)
        );
        debug_assert_eq!(
            self.display.sub_screen_window_layers,
            ram_byte(self.ram, TSW_COPY)
        );
    }

    fn debug_assert_nmi_copy_packets_request_matches_ram(&self) {
        debug_assert_eq!(
            self.display.nmi_copy_packets_request,
            ram_byte(self.ram, NMI_COPY_PACKETS_FLAG)
        );
    }

    fn debug_assert_pending_polyhedral_update_matches_ram(&self) {
        debug_assert_eq!(
            self.display.pending_polyhedral_update,
            ram_byte(self.ram, NMI_FLAG_UPDATE_POLYHEDRAL)
        );
    }

    fn debug_assert_chr_halfslot_request_matches_ram(&self) {
        debug_assert_eq!(
            self.display.chr_halfslot_request,
            ram_byte(self.ram, LOAD_CHR_HALFSLOT_EVEN_ODD)
        );
    }

    fn debug_assert_nmi_thread_control_matches_ram(&self) {
        debug_assert_eq!(
            self.display.nmi_thread_active,
            ram_byte(self.ram, NMI_THREAD_ACTIVE) != 0
        );
        debug_assert_eq!(
            self.display.nmi_thread_stack_pointer,
            read_le_u16(self.ram, POLY_THREAD_STACK)
        );
    }

    fn debug_assert_irq_control_matches_ram(&self) {
        debug_assert_eq!(self.display.irq_control_flag, ram_byte(self.ram, IRQ_FLAG));
        debug_assert_eq!(
            self.display.vertical_irq_trigger,
            ram_byte(self.ram, VIRQ_TRIGGER)
        );
    }

    fn debug_assert_sprite_dma_pointers_match_ram(&self) {
        debug_assert_eq!(
            self.display.sprite_dma_head_pointer,
            ram_byte(self.ram, DMA_HEAD_POINTER)
        );
        debug_assert_eq!(
            self.display.sprite_dma_body_pointer,
            ram_byte(self.ram, DMA_BODY_POINTER)
        );
    }

    fn debug_assert_hdma_enable_mask_matches_ram(&self) {
        debug_assert_eq!(
            self.display.hdma_enable_mask,
            ram_byte(self.ram, HDMAEN_COPY)
        );
    }

    fn debug_assert_mosaic_control_matches_ram(&self) {
        debug_assert_eq!(self.display.mosaic_copy, ram_byte(self.ram, MOSAIC_COPY));
        debug_assert_eq!(self.display.mosaic_level, ram_byte(self.ram, MOSAIC_LEVEL));
        debug_assert_eq!(
            self.display.mosaic_target_level,
            ram_byte(self.ram, MOSAIC_TARGET_LEVEL)
        );
        debug_assert_eq!(
            self.display.mosaic_direction,
            ram_byte(self.ram, MOSAIC_INC_OR_DEC)
        );
    }

    fn debug_assert_nmi_load_target_address_matches_ram(&self) {
        debug_assert_eq!(
            self.display.nmi_load_target_address,
            read_le_u16(self.ram, NMI_LOAD_TARGET_ADDR)
        );
    }

    fn debug_assert_incremental_vram_upload_counter_matches_ram(&self) {
        debug_assert_eq!(
            self.display.incremental_vram_upload_counter,
            ram_byte(self.ram, INCREMENTAL_COUNTER_FOR_VRAM)
        );
    }

    fn debug_assert_link_dma_sources_match_ram(&self) {
        for slot in LINK_DMA_SOURCE_SLOTS {
            debug_assert_eq!(
                self.display.link_dma_sources.source(slot),
                read_le_u16(self.ram, slot.address())
            );
        }
    }

    fn debug_assert_bg_tile_animation_countdown_matches_ram(&self) {
        debug_assert_eq!(
            self.display.bg_tile_animation_countdown,
            read_le_u16(self.ram, BG_TILE_ANIMATION_COUNTDOWN)
        );
    }

    fn debug_assert_message_dma_destination_address_matches_ram(&self) {
        debug_assert_eq!(
            self.display.message_dma_destination_address,
            read_le_u16(self.ram, MESSAGE_DMA_DST_ADDR)
        );
    }

    fn debug_assert_message_dma_tile_range_matches_ram(&self) {
        debug_assert_eq!(
            self.display.message_dma_tile_base,
            read_le_u16(self.ram, MESSAGE_DMA_TILE_BASE)
        );
        debug_assert_eq!(
            self.display.message_dma_tile_limit,
            read_le_u16(self.ram, MESSAGE_DMA_TILE_LIMIT)
        );
        debug_assert_eq!(
            self.display.message_dma_tile_sentinel,
            read_le_u16(self.ram, MESSAGE_DMA_TILE_SENTINEL)
        );
    }

    fn debug_assert_overworld_fixed_color_adjustment_matches_ram(&self) {
        debug_assert_eq!(
            self.display.overworld_fixed_color_adjustment,
            ram_byte(self.ram, OVERWORLD_FIXED_COLOR_PLUSMINUS)
        );
    }

    fn debug_assert_travel_bird_tile_offset_matches_ram(&self) {
        debug_assert_eq!(
            self.display.travel_bird_tile_offset,
            ram_byte(self.ram, FLAG_TRAVEL_BIRD)
        );
    }

    fn debug_assert_star_tile_restore_phase_matches_ram(&self) {
        debug_assert_eq!(
            self.display.star_tile_restore_phase,
            ram_byte(self.ram, STAR_TILE_RESTORE_PHASE)
        );
    }

    fn debug_assert_animated_tile_upload_metadata_matches_ram(&self) {
        debug_assert_eq!(
            self.display.animated_tile_data_source_address,
            read_le_u16(self.ram, ANIMATED_TILE_DATA_SRC)
        );
        debug_assert_eq!(
            self.display.animated_tile_vram_destination_address,
            read_le_u16(self.ram, ANIMATED_TILE_VRAM_ADDR)
        );
    }

    pub(crate) fn set_screen_brightness(&mut self, value: u8) {
        self.display.screen_brightness = value;
        self.ram[INIDISP_COPY] = value;
        self.debug_assert_screen_brightness_matches_ram();
    }

    pub(crate) fn increment_screen_brightness(&mut self) -> u8 {
        let value = self.display.screen_brightness.wrapping_add(1);
        self.set_screen_brightness(value);
        value
    }

    pub(crate) fn decrement_screen_brightness(&mut self) -> u8 {
        let value = self.display.screen_brightness.wrapping_sub(1);
        self.set_screen_brightness(value);
        value
    }

    pub(crate) fn set_nmi_update_latch(&mut self, value: u8) {
        self.display.nmi_update_latch = value;
        self.ram[NMI_BOOLEAN] = value;
        self.debug_assert_nmi_update_latch_matches_ram();
    }

    pub(crate) fn latch_nmi_update(&mut self) {
        self.set_nmi_update_latch(1);
    }

    pub(crate) fn clear_nmi_update_latch(&mut self) {
        self.set_nmi_update_latch(0);
    }

    pub(crate) fn set_core_update_disable_flag(&mut self, value: u8) {
        self.display.core_update_disable_flag = value;
        self.ram[NMI_DISABLE_CORE_UPDATES] = value;
        self.debug_assert_core_update_disable_flag_matches_ram();
    }

    pub(crate) fn set_core_update_disable_flag_word(&mut self, value: u16) {
        self.display.core_update_disable_flag = value as u8;
        write_le_u16(self.ram, NMI_DISABLE_CORE_UPDATES, value);
        self.debug_assert_core_update_disable_flag_matches_ram();
    }

    pub(crate) fn clear_core_update_disable_flag(&mut self) {
        self.set_core_update_disable_flag(0);
    }

    pub(crate) fn increment_core_update_disable_flag(&mut self) -> u8 {
        let value = self.display.core_update_disable_flag.wrapping_add(1);
        self.set_core_update_disable_flag(value);
        value
    }

    pub(crate) fn set_pending_nmi_subroutine(&mut self, value: u8) {
        self.display.pending_nmi_subroutine = value;
        self.ram[NMI_SUBROUTINE_INDEX] = value;
        self.debug_assert_pending_nmi_subroutine_matches_ram();
    }

    pub(crate) fn clear_pending_nmi_subroutine(&mut self) {
        self.set_pending_nmi_subroutine(0);
    }

    pub(crate) fn take_pending_nmi_subroutine(&mut self) -> u8 {
        let value = self.display.pending_nmi_subroutine;
        self.clear_pending_nmi_subroutine();
        value
    }

    pub(crate) fn set_bg_vram_load_mode(&mut self, value: u8) {
        self.display.bg_vram_load_mode = value;
        self.ram[NMI_LOAD_BG_FROM_VRAM] = value;
        self.debug_assert_bg_vram_load_mode_matches_ram();
    }

    pub(crate) fn clear_bg_vram_load_mode(&mut self) {
        self.set_bg_vram_load_mode(0);
    }

    pub(crate) fn queue_tilemap_update(&mut self, destination_page: u8, source_offset: u16) {
        self.display.pending_tilemap_update_destination_page = destination_page;
        self.display.pending_tilemap_update_source_offset = source_offset;
        self.ram[NMI_UPDATE_TILEMAP_DST] = destination_page;
        write_le_u16(self.ram, NMI_UPDATE_TILEMAP_SRC, source_offset);
        self.debug_assert_pending_tilemap_update_matches_ram();
    }

    pub(crate) fn clear_pending_tilemap_update_destination(&mut self) {
        self.display.pending_tilemap_update_destination_page = 0;
        self.ram[NMI_UPDATE_TILEMAP_DST] = 0;
        self.debug_assert_pending_tilemap_update_matches_ram();
    }

    pub(crate) fn set_bg_mode(&mut self, value: u8) {
        self.display.bg_mode = value;
        self.ram[BGMODE_COPY] = value;
        self.debug_assert_bg_mode_matches_ram();
    }

    pub(crate) fn set_main_screen_layers(&mut self, value: u8) {
        self.display.main_screen_layers = value;
        self.ram[TM_COPY] = value;
        self.debug_assert_screen_layer_masks_match_ram();
    }

    pub(crate) fn and_main_screen_layers(&mut self, value: u8) {
        let layers = self.display.main_screen_layers & value;
        self.set_main_screen_layers(layers);
    }

    pub(crate) fn or_main_screen_layers(&mut self, value: u8) {
        let layers = self.display.main_screen_layers | value;
        self.set_main_screen_layers(layers);
    }

    pub(crate) fn set_sub_screen_layers(&mut self, value: u8) {
        self.display.sub_screen_layers = value;
        self.ram[TS_COPY] = value;
        self.debug_assert_screen_layer_masks_match_ram();
    }

    pub(crate) fn clear_sub_screen_layers_word(&mut self) {
        self.display.sub_screen_layers = 0;
        self.display.main_screen_window_layers = 0;
        write_le_u16(self.ram, TS_COPY, 0);
        self.debug_assert_screen_layer_masks_match_ram();
        self.debug_assert_window_layer_masks_match_ram();
    }

    pub(crate) fn and_sub_screen_layers(&mut self, value: u8) {
        let layers = self.display.sub_screen_layers & value;
        self.set_sub_screen_layers(layers);
    }

    pub(crate) fn or_sub_screen_layers(&mut self, value: u8) {
        let layers = self.display.sub_screen_layers | value;
        self.set_sub_screen_layers(layers);
    }

    pub(crate) fn set_layer_masks_word(&mut self, value: u16) {
        self.display.main_screen_layers = value as u8;
        self.display.sub_screen_layers = (value >> 8) as u8;
        write_le_u16(self.ram, TM_COPY, value);
        self.debug_assert_screen_layer_masks_match_ram();
    }

    pub(crate) fn set_bg12_window_selection(&mut self, value: u8) {
        self.display.bg12_window_selection = value;
        self.ram[W12SEL_COPY] = value;
        self.debug_assert_window_layer_masks_match_ram();
    }

    pub(crate) fn set_bg34_window_selection(&mut self, value: u8) {
        self.display.bg34_window_selection = value;
        self.ram[W34SEL_COPY] = value;
        self.debug_assert_window_layer_masks_match_ram();
    }

    pub(crate) fn set_object_color_window_selection(&mut self, value: u8) {
        self.display.object_color_window_selection = value;
        self.ram[WOBJSEL_COPY] = value;
        self.debug_assert_window_layer_masks_match_ram();
    }

    pub(crate) fn set_main_screen_window_layers(&mut self, value: u8) {
        self.display.main_screen_window_layers = value;
        self.ram[TMW_COPY] = value;
        self.debug_assert_window_layer_masks_match_ram();
    }

    pub(crate) fn set_sub_screen_window_layers(&mut self, value: u8) {
        self.display.sub_screen_window_layers = value;
        self.ram[TSW_COPY] = value;
        self.debug_assert_window_layer_masks_match_ram();
    }

    pub(crate) fn set_window_layer_masks(
        &mut self,
        bg12_window_selection: u8,
        bg34_window_selection: u8,
        object_color_window_selection: u8,
        main_screen_window_layers: u8,
        sub_screen_window_layers: u8,
    ) {
        self.display.bg12_window_selection = bg12_window_selection;
        self.display.bg34_window_selection = bg34_window_selection;
        self.display.object_color_window_selection = object_color_window_selection;
        self.display.main_screen_window_layers = main_screen_window_layers;
        self.display.sub_screen_window_layers = sub_screen_window_layers;
        self.ram[W12SEL_COPY] = bg12_window_selection;
        self.ram[W34SEL_COPY] = bg34_window_selection;
        self.ram[WOBJSEL_COPY] = object_color_window_selection;
        self.ram[TMW_COPY] = main_screen_window_layers;
        self.ram[TSW_COPY] = sub_screen_window_layers;
        self.debug_assert_window_layer_masks_match_ram();
    }

    pub(crate) fn clear_window_layer_masks(&mut self) {
        self.set_window_layer_masks(0, 0, 0, 0, 0);
    }

    pub(crate) fn clear_window_main_sub_masks(&mut self) {
        self.display.main_screen_window_layers = 0;
        self.display.sub_screen_window_layers = 0;
        write_le_u16(self.ram, TMW_COPY, 0);
        self.debug_assert_window_layer_masks_match_ram();
    }

    pub(crate) fn set_nmi_copy_packets_request(&mut self, value: u8) {
        self.display.nmi_copy_packets_request = value;
        self.ram[NMI_COPY_PACKETS_FLAG] = value;
        self.debug_assert_nmi_copy_packets_request_matches_ram();
    }

    pub(crate) fn request_nmi_copy_packets(&mut self) {
        self.set_nmi_copy_packets_request(1);
    }

    pub(crate) fn clear_nmi_copy_packets_request(&mut self) {
        self.set_nmi_copy_packets_request(0);
    }

    pub(crate) fn set_pending_polyhedral_update(&mut self, value: u8) {
        self.display.pending_polyhedral_update = value;
        self.ram[NMI_FLAG_UPDATE_POLYHEDRAL] = value;
        self.debug_assert_pending_polyhedral_update_matches_ram();
    }

    pub(crate) fn request_polyhedral_nmi_update(&mut self) {
        self.set_pending_polyhedral_update(0xff);
    }

    pub(crate) fn clear_pending_polyhedral_update(&mut self) {
        self.set_pending_polyhedral_update(0);
    }

    pub(crate) fn set_chr_halfslot_request(&mut self, value: u8) {
        self.display.chr_halfslot_request = value;
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = value;
        self.debug_assert_chr_halfslot_request_matches_ram();
    }

    pub(crate) fn clear_chr_halfslot_request(&mut self) {
        self.set_chr_halfslot_request(0);
    }

    pub(crate) fn increment_chr_halfslot_request(&mut self) -> u8 {
        let value = self.display.chr_halfslot_request.wrapping_add(1);
        self.set_chr_halfslot_request(value);
        value
    }

    pub(crate) fn set_nmi_thread_active(&mut self, active: bool) {
        self.display.nmi_thread_active = active;
        self.ram[NMI_THREAD_ACTIVE] = u8::from(active);
        self.debug_assert_nmi_thread_control_matches_ram();
    }

    pub(crate) fn activate_nmi_thread(&mut self) {
        self.set_nmi_thread_active(true);
    }

    pub(crate) fn deactivate_nmi_thread(&mut self) {
        self.set_nmi_thread_active(false);
    }

    pub(crate) fn set_nmi_thread_stack_pointer(&mut self, value: u16) {
        self.display.nmi_thread_stack_pointer = value;
        write_le_u16(self.ram, POLY_THREAD_STACK, value);
        self.debug_assert_nmi_thread_control_matches_ram();
    }

    pub(crate) fn set_irq_control_flag(&mut self, value: u8) {
        self.display.irq_control_flag = value;
        self.ram[IRQ_FLAG] = value;
        self.debug_assert_irq_control_matches_ram();
    }

    pub(crate) fn clear_irq_control_flag(&mut self) {
        self.set_irq_control_flag(0);
    }

    pub(crate) fn set_vertical_irq_trigger(&mut self, value: u8) {
        self.display.vertical_irq_trigger = value;
        self.ram[VIRQ_TRIGGER] = value;
        self.debug_assert_irq_control_matches_ram();
    }

    pub(crate) fn set_sprite_dma_head_pointer(&mut self, value: u8) {
        self.display.sprite_dma_head_pointer = value;
        self.ram[DMA_HEAD_POINTER] = value;
        self.debug_assert_sprite_dma_pointers_match_ram();
    }

    pub(crate) fn set_sprite_dma_body_pointer(&mut self, value: u8) {
        self.display.sprite_dma_body_pointer = value;
        self.ram[DMA_BODY_POINTER] = value;
        self.debug_assert_sprite_dma_pointers_match_ram();
    }

    pub(crate) fn set_hdma_enable_mask(&mut self, value: u8) {
        self.display.hdma_enable_mask = value;
        self.ram[HDMAEN_COPY] = value;
        self.debug_assert_hdma_enable_mask_matches_ram();
    }

    pub(crate) fn clear_hdma_enable_mask(&mut self) {
        self.set_hdma_enable_mask(0);
    }

    pub(crate) fn set_mosaic_copy(&mut self, value: u8) {
        self.display.mosaic_copy = value;
        self.ram[MOSAIC_COPY] = value;
        self.debug_assert_mosaic_control_matches_ram();
    }

    pub(crate) fn set_mosaic_copy_from_level_or(&mut self, mask: u8) {
        self.set_mosaic_copy(self.display.mosaic_level | mask);
    }

    pub(crate) fn set_mosaic_level(&mut self, value: u8) {
        self.display.mosaic_level = value;
        self.ram[MOSAIC_LEVEL] = value;
        self.debug_assert_mosaic_control_matches_ram();
    }

    pub(crate) fn clear_mosaic_level(&mut self) {
        self.set_mosaic_level(0);
    }

    pub(crate) fn clear_mosaic_level_word(&mut self) {
        self.display.mosaic_level = 0;
        write_le_u16(self.ram, MOSAIC_LEVEL, 0);
        self.debug_assert_mosaic_control_matches_ram();
    }

    pub(crate) fn increment_mosaic_level_by(&mut self, value: u8) -> u8 {
        let level = self.display.mosaic_level.wrapping_add(value);
        self.set_mosaic_level(level);
        level
    }

    pub(crate) fn decrement_mosaic_level_by(&mut self, value: u8) -> u8 {
        let level = self.display.mosaic_level.wrapping_sub(value);
        self.set_mosaic_level(level);
        level
    }

    pub(crate) fn set_mosaic_target_level(&mut self, value: u8) {
        self.display.mosaic_target_level = value;
        self.ram[MOSAIC_TARGET_LEVEL] = value;
        self.debug_assert_mosaic_control_matches_ram();
    }

    pub(crate) fn set_mosaic_target_level_word(&mut self, value: u16) {
        self.display.mosaic_target_level = value as u8;
        write_le_u16(self.ram, MOSAIC_TARGET_LEVEL, value);
        self.debug_assert_mosaic_control_matches_ram();
    }

    pub(crate) fn clear_mosaic_target_level(&mut self) {
        self.set_mosaic_target_level(0);
    }

    pub(crate) fn clear_mosaic_target_level_word(&mut self) {
        self.display.mosaic_target_level = 0;
        write_le_u16(self.ram, MOSAIC_TARGET_LEVEL, 0);
        self.debug_assert_mosaic_control_matches_ram();
    }

    pub(crate) fn set_mosaic_direction(&mut self, value: u8) {
        self.display.mosaic_direction = value;
        self.ram[MOSAIC_INC_OR_DEC] = value;
        self.debug_assert_mosaic_control_matches_ram();
    }

    pub(crate) fn clear_mosaic_direction(&mut self) {
        self.set_mosaic_direction(0);
    }

    pub(crate) fn set_nmi_load_target_page(&mut self, value: u8) {
        self.display.nmi_load_target_address =
            (self.display.nmi_load_target_address & 0xff00) | u16::from(value);
        self.ram[NMI_LOAD_TARGET_ADDR] = value;
        self.debug_assert_nmi_load_target_address_matches_ram();
    }

    pub(crate) fn set_nmi_load_target_address(&mut self, value: u16) {
        self.display.nmi_load_target_address = value;
        write_le_u16(self.ram, NMI_LOAD_TARGET_ADDR, value);
        self.debug_assert_nmi_load_target_address_matches_ram();
    }

    pub(crate) fn reset_incremental_vram_upload_counter(&mut self) {
        self.display.incremental_vram_upload_counter = 0;
        self.ram[INCREMENTAL_COUNTER_FOR_VRAM] = 0;
        self.debug_assert_incremental_vram_upload_counter_matches_ram();
    }

    pub(crate) fn increment_vram_upload_counter(&mut self) -> u8 {
        let value = self.display.incremental_vram_upload_counter.wrapping_add(1);
        self.display.incremental_vram_upload_counter = value;
        self.ram[INCREMENTAL_COUNTER_FOR_VRAM] = value;
        self.debug_assert_incremental_vram_upload_counter_matches_ram();
        value
    }

    fn set_link_dma_source(&mut self, slot: LinkDmaSourceSlot, value: u16) {
        self.display.link_dma_sources.set_source(slot, value);
        write_le_u16(self.ram, slot.address(), value);
        self.debug_assert_link_dma_sources_match_ram();
    }

    pub(crate) fn set_link_body_dma_sources(&mut self, top: u16, bottom: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::BodyTop, top);
        self.set_link_dma_source(LinkDmaSourceSlot::BodyBottom, bottom);
    }

    pub(crate) fn set_link_head_dma_sources(&mut self, top: u16, bottom: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::HeadTop, top);
        self.set_link_dma_source(LinkDmaSourceSlot::HeadBottom, bottom);
    }

    pub(crate) fn set_link_hand_dma_sources(&mut self, left: u16, right: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::HandLeft, left);
        self.set_link_dma_source(LinkDmaSourceSlot::HandRight, right);
    }

    pub(crate) fn set_link_sword_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::SwordUpper, upper);
        self.set_link_dma_source(LinkDmaSourceSlot::SwordLower, lower);
    }

    pub(crate) fn set_link_shield_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::ShieldUpper, upper);
        self.set_link_dma_source(LinkDmaSourceSlot::ShieldLower, lower);
    }

    pub(crate) fn set_link_aux_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::AuxUpper, upper);
        self.set_link_dma_source(LinkDmaSourceSlot::AuxLower, lower);
    }

    pub(crate) fn set_link_push_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::PushUpper, upper);
        self.set_link_dma_source(LinkDmaSourceSlot::PushLower, lower);
    }

    pub(crate) fn set_link_animated_tile_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::AnimatedTileUpper, upper);
        self.set_link_dma_source(LinkDmaSourceSlot::AnimatedTileLower, lower);
    }

    pub(crate) fn set_link_head_pointer_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::HeadPointerUpper, upper);
        self.set_link_dma_source(LinkDmaSourceSlot::HeadPointerLower, lower);
    }

    pub(crate) fn set_link_body_pointer_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::BodyPointerUpper, upper);
        self.set_link_dma_source(LinkDmaSourceSlot::BodyPointerLower, lower);
    }

    pub(crate) fn set_travel_bird_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(LinkDmaSourceSlot::TravelBirdUpper, upper);
        self.set_link_dma_source(LinkDmaSourceSlot::TravelBirdLower, lower);
    }

    pub(crate) fn reset_bg_tile_animation_countdown(&mut self, value: u16) {
        self.display.bg_tile_animation_countdown = value;
        write_le_u16(self.ram, BG_TILE_ANIMATION_COUNTDOWN, value);
        self.debug_assert_bg_tile_animation_countdown_matches_ram();
    }

    pub(crate) fn set_message_dma_destination_address(&mut self, value: u16) {
        self.display.message_dma_destination_address = value;
        write_le_u16(self.ram, MESSAGE_DMA_DST_ADDR, value);
        self.debug_assert_message_dma_destination_address_matches_ram();
    }

    pub(crate) fn set_message_dma_tile_base(&mut self, value: u16) {
        self.display.message_dma_tile_base = value;
        write_le_u16(self.ram, MESSAGE_DMA_TILE_BASE, value);
        self.debug_assert_message_dma_tile_range_matches_ram();
    }

    pub(crate) fn set_message_dma_tile_limit(&mut self, value: u16) {
        self.display.message_dma_tile_limit = value;
        write_le_u16(self.ram, MESSAGE_DMA_TILE_LIMIT, value);
        self.debug_assert_message_dma_tile_range_matches_ram();
    }

    pub(crate) fn set_message_dma_tile_sentinel(&mut self, value: u16) {
        self.display.message_dma_tile_sentinel = value;
        write_le_u16(self.ram, MESSAGE_DMA_TILE_SENTINEL, value);
        self.debug_assert_message_dma_tile_range_matches_ram();
    }

    pub(crate) fn set_overworld_fixed_color_adjustment(&mut self, value: u8) {
        self.display.overworld_fixed_color_adjustment = value;
        self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = value;
        self.debug_assert_overworld_fixed_color_adjustment_matches_ram();
    }

    pub(crate) fn set_travel_bird_tile_offset(&mut self, value: u8) {
        self.display.travel_bird_tile_offset = value;
        self.ram[FLAG_TRAVEL_BIRD] = value;
        self.debug_assert_travel_bird_tile_offset_matches_ram();
    }

    pub(crate) fn clear_star_tile_restore_phase(&mut self) {
        self.display.star_tile_restore_phase = 0;
        self.ram[STAR_TILE_RESTORE_PHASE] = 0;
        self.debug_assert_star_tile_restore_phase_matches_ram();
    }

    pub(crate) fn set_animated_tile_data_source_address(&mut self, value: u16) {
        self.display.animated_tile_data_source_address = value;
        write_le_u16(self.ram, ANIMATED_TILE_DATA_SRC, value);
        self.debug_assert_animated_tile_upload_metadata_matches_ram();
    }

    pub(crate) fn set_animated_tile_vram_destination_address(&mut self, value: u16) {
        self.display.animated_tile_vram_destination_address = value;
        write_le_u16(self.ram, ANIMATED_TILE_VRAM_ADDR, value);
        self.debug_assert_animated_tile_upload_metadata_matches_ram();
    }

    pub(crate) fn set_overworld_tile_attribute_word(&mut self, index: usize, value: u16) {
        let address = crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + index * 2;
        write_le_u16(self.ram, address, value);
        debug_assert_eq!(
            self.display.overworld_tile_attribute_word(self.ram, index),
            value
        );
    }

    pub(crate) fn set_overworld_tile_upload_word(&mut self, index: usize, value: u16) {
        let address = crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + index * 2;
        write_le_u16(self.ram, address, value);
        debug_assert_eq!(
            self.display.overworld_tile_upload_word(self.ram, index),
            value
        );
    }

    pub(crate) fn terminate_overworld_tile_upload_words(&mut self, index: usize) {
        self.set_overworld_tile_upload_word(index, 0xffff);
    }

    pub(crate) fn copy_tilemap_upload_stripe_bytes(&mut self, bytes: &[u8]) {
        let start = crate::game_state::constants::nmi::TILEMAP_UPLOAD_BUFFER;
        let len = bytes.len().min(self.ram.len().saturating_sub(start));
        self.ram[start..start + len].copy_from_slice(&bytes[..len]);

        // This legacy upload buffer starts at the same word used as the VRAM
        // upload cursor, so keep the native metadata synchronized after writes.
        self.display.vram_upload_cursor = read_le_u16(self.ram, VRAM_UPLOAD_OFFSET);
        debug_assert_eq!(
            self.display.vram_upload_cursor,
            read_le_u16(self.ram, VRAM_UPLOAD_OFFSET)
        );
    }
}
