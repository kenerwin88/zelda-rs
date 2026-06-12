use super::*;

pub(crate) struct HudRawStateView<'a> {
    ram: &'a [u8],
}

impl<'a> HudRawStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn floor_changed_timer_low(&self) -> u8 {
        byte(self.ram, HUD_FLOOR_CHANGED_TIMER)
    }

    pub(crate) fn tile_word(&self, tile: usize) -> u16 {
        word(self.ram, HUD_TILE_INDICES_BUFFER + tile * 2)
    }
}

pub(crate) struct HudRawStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> HudRawStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_floor_changed_timer(&mut self, value: u16) {
        write_le_u16(self.ram, HUD_FLOOR_CHANGED_TIMER, value);
    }

    pub(crate) fn set_tile_word(&mut self, tile: usize, value: u16) {
        write_le_u16(self.ram, HUD_TILE_INDICES_BUFFER + tile * 2, value);
    }

    pub(crate) fn clear_floor_changed_timer_low(&mut self) {
        self.ram[HUD_FLOOR_CHANGED_TIMER] = 0;
    }
}

pub(crate) struct GraphicsScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> GraphicsScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_aux_bg_subset_pack(&mut self, index: usize, value: u8) {
        self.ram[AUX_BG_SUBSET_0 + index] = value;
    }

    pub(crate) fn primary_decomp_buffer_offset() -> usize {
        PRIMARY_DECOMP_BUFFER_LOAD_GFX
    }

    pub(crate) fn secondary_decomp_buffer_offset() -> usize {
        SECONDARY_DECOMP_BUFFER_LOAD_GFX
    }

    pub(crate) fn primary_decomp_buffer(&self, len: usize) -> Vec<u8> {
        self.ram[PRIMARY_DECOMP_BUFFER_LOAD_GFX..PRIMARY_DECOMP_BUFFER_LOAD_GFX + len].to_vec()
    }

    pub(crate) fn combined_decomp_buffers(&self) -> Vec<u8> {
        self.primary_decomp_buffer(0x0c00)
    }

    pub(crate) fn copy_to_primary_decomp_buffer(&mut self, data: &[u8]) {
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
            let x = word(self.ram, base + 0x1880);
            let a = word(self.ram, base + 0x1c80);
            let b = word(self.ram, base + 0x1e80);
            let c = word(self.ram, base + 0x1a80);
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
            let color = word(self.ram, PEG_TILE_GFX_BUFFER + (first >> 1) * 2 + i * 2);
            write_le_u16(self.ram, MESSAGING_BUF_LOAD_GFX + i * 2, color);
        }
        for i in 0..64 {
            let color = word(self.ram, PEG_TILE_GFX_BUFFER + (second >> 1) * 2 + i * 2);
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
        word(self.ram, AGAHNIM_PAL_SETTING + index * 2)
    }

    pub(crate) fn set_agahnim_palette_word(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, AGAHNIM_PAL_SETTING + index * 2, value);
    }

    pub(crate) fn sprite_decomp_buffer_tail(&self) -> Vec<u8> {
        self.ram[SPRITE_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec()
    }

    pub(crate) fn staged_bg_and_sprite_decomp_buffers(&self) -> Vec<u8> {
        self.ram[BG_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec()
    }
}

pub(crate) struct SpotlightHdmaView<'a> {
    ram: &'a [u8],
}
impl<'a> SpotlightHdmaView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn y_lower(&self) -> u16 {
        word(self.ram, SPOTLIGHT_Y_LOWER)
    }
    pub(crate) fn y_upper(&self) -> u16 {
        word(self.ram, SPOTLIGHT_Y_UPPER)
    }
    pub(crate) fn window_x_center(&self) -> u16 {
        word(self.ram, SPOTLIGHT_WINDOW_X_CENTER)
    }
    pub(crate) fn window_state(&self) -> u16 {
        word(self.ram, SPOTLIGHT_WINDOW_STATE)
    }
    pub(crate) fn window_state_byte(&self) -> u8 {
        byte(self.ram, SPOTLIGHT_WINDOW_STATE)
    }
    pub(crate) fn window_radius(&self) -> u16 {
        word(self.ram, SPOTLIGHT_WINDOW_RADIUS)
    }
    pub(crate) fn window_radius_byte(&self) -> u8 {
        byte(self.ram, SPOTLIGHT_WINDOW_RADIUS)
    }
    pub(crate) fn window_y_buffer(&self) -> u16 {
        word(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER)
    }
    pub(crate) fn window_y_buffer_byte(&self) -> u8 {
        byte(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER)
    }
    pub(crate) fn hdma_table_dynamic_entry(&self, index: usize) -> u16 {
        word(self.ram, HDMA_TABLE_DYNAMIC + index * 2)
    }
}
pub(crate) struct SpotlightHdmaViewMut<'a> {
    ram: &'a mut [u8],
}
impl<'a> SpotlightHdmaViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn set_y_lower(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_Y_LOWER, v);
    }
    pub(crate) fn set_y_upper(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_Y_UPPER, v);
    }
    pub(crate) fn set_window_x_center(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_X_CENTER, v);
    }
    pub(crate) fn set_window_state(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_STATE, v);
    }
    pub(crate) fn set_window_radius(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_RADIUS, v);
    }
    pub(crate) fn set_window_y_buffer(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER, v);
    }
    pub(crate) fn decrement_window_y_buffer(&mut self) -> u16 {
        let v = word(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER).wrapping_sub(1);
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER, v);
        v
    }

    pub(crate) fn set_window_radius_byte(&mut self, v: u8) {
        self.ram[SPOTLIGHT_WINDOW_RADIUS] = v;
    }

    pub(crate) fn set_window_state_byte(&mut self, v: u8) {
        self.ram[SPOTLIGHT_WINDOW_STATE] = v;
    }

    pub(crate) fn set_window_y_buffer_byte(&mut self, v: u8) {
        self.ram[SPOTLIGHT_WINDOW_Y_BUFFER] = v;
    }

    pub(crate) fn increment_window_y_buffer_byte(&mut self) {
        self.ram[SPOTLIGHT_WINDOW_Y_BUFFER] = self.ram[SPOTLIGHT_WINDOW_Y_BUFFER].wrapping_add(1);
    }

    pub(crate) fn shr_window_radius_byte(&mut self, shift: u8) {
        self.ram[SPOTLIGHT_WINDOW_RADIUS] >>= shift;
    }

    pub(crate) fn add_window_radius_byte(&mut self, v: u8) {
        self.ram[SPOTLIGHT_WINDOW_RADIUS] = self.ram[SPOTLIGHT_WINDOW_RADIUS].wrapping_add(v);
    }

    pub(crate) fn hdma_table_dynamic_entry(&self, index: usize) -> u16 {
        word(self.ram, HDMA_TABLE_DYNAMIC + index * 2)
    }

    pub(crate) fn set_hdma_table_dynamic_entry(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, HDMA_TABLE_DYNAMIC + index * 2, value);
    }

    pub(crate) fn clear_hdma_table_dynamic(&mut self, count: usize) {
        for i in 0..count {
            write_le_u16(self.ram, HDMA_TABLE_DYNAMIC + i * 2, 0);
        }
    }

    pub(crate) fn clear_hdma_table_dynamic_range(&mut self, start: usize, count: usize) {
        for i in start..start + count {
            write_le_u16(self.ram, HDMA_TABLE_DYNAMIC + i * 2, 0);
        }
    }
}
