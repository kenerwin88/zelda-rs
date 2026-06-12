use super::ram_byte;
use crate::game_state::constants::messaging::{
    MESSAGE_DMA_DST_ADDR, MESSAGE_DMA_TILE_BASE, MESSAGE_DMA_TILE_LIMIT, MESSAGE_DMA_TILE_SENTINEL,
};
use crate::game_state::constants::*;
use crate::game_state::RamVramUploadBufferMut;
use crate::types::{read_le_u16, write_le_u16};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    pub(crate) message_dma_destination_address: u16,
    pub(crate) message_dma_tile_base: u16,
    pub(crate) message_dma_tile_limit: u16,
    pub(crate) message_dma_tile_sentinel: u16,
    pub(crate) overworld_fixed_color_adjustment: u8,
    pub(crate) travel_bird_tile_offset: u8,
    pub(crate) animated_tile_data_source_address: u16,
    pub(crate) animated_tile_vram_destination_address: u16,
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
            message_dma_destination_address: read_le_u16(ram, MESSAGE_DMA_DST_ADDR),
            message_dma_tile_base: read_le_u16(ram, MESSAGE_DMA_TILE_BASE),
            message_dma_tile_limit: read_le_u16(ram, MESSAGE_DMA_TILE_LIMIT),
            message_dma_tile_sentinel: read_le_u16(ram, MESSAGE_DMA_TILE_SENTINEL),
            overworld_fixed_color_adjustment: ram_byte(ram, OVERWORLD_FIXED_COLOR_PLUSMINUS),
            travel_bird_tile_offset: ram_byte(ram, FLAG_TRAVEL_BIRD),
            animated_tile_data_source_address: read_le_u16(ram, ANIMATED_TILE_DATA_SRC),
            animated_tile_vram_destination_address: read_le_u16(ram, ANIMATED_TILE_VRAM_ADDR),
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

    pub(crate) fn vram_upload_buffer_remaining<'a>(&self, ram: &'a [u8]) -> &'a [u8] {
        &ram[self.vram_upload_buffer_base()..]
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
}

pub(crate) struct NativeVramUploadBufferBridgeMut<'a> {
    display: &'a mut DisplayState,
    ram_view: RamVramUploadBufferMut<'a>,
}

impl<'a> NativeVramUploadBufferBridgeMut<'a> {
    pub(crate) fn new(display: &'a mut DisplayState, ram: &'a mut [u8]) -> Self {
        *display = DisplayState::load_from_ram(ram);
        Self {
            display,
            ram_view: RamVramUploadBufferMut::new(ram),
        }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(self.display.vram_upload_cursor, self.ram_view.offset());
    }

    pub(crate) fn set_offset(&mut self, value: u16) {
        self.display.vram_upload_cursor = value;
        self.ram_view.set_offset(value);
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
        self.ram_view.write_buffer_byte(offset, value);
    }

    pub(crate) fn write_buffer_word(&mut self, offset: usize, value: u16) {
        self.ram_view.write_buffer_word(offset, value);
    }

    pub(crate) fn write_tilemap_word(&mut self, offset: usize, value: u16) {
        self.ram_view.write_tilemap_word(offset, value);
    }

    pub(crate) fn write_overworld_vram_word(&mut self, word_index: usize, value: u16) {
        self.ram_view.write_overworld_vram_word(word_index, value);
    }

    pub(crate) fn write_absolute_byte(&mut self, address: usize, value: u8) {
        self.ram_view.write_absolute_byte(address, value);
    }

    pub(crate) fn write_absolute_word(&mut self, address: usize, value: u16) {
        self.ram_view.write_absolute_word(address, value);
    }

    pub(crate) fn copy_buffer_bytes(&mut self, offset: usize, data: &[u8]) {
        self.ram_view.copy_buffer_bytes(offset, data);
    }

    pub(crate) fn terminate_buffer_at(&mut self, offset: usize) {
        self.ram_view.terminate_buffer_at(offset);
    }

    pub(crate) fn write_level_label_tiles(&mut self, left: &[u8; 14], right: &[u8; 14]) {
        self.ram_view.write_level_label_tiles(left, right);
    }

    pub(crate) fn write_map16_update_packet(
        &mut self,
        address: usize,
        vram_pos: u16,
        tiles: [u16; 4],
    ) {
        self.ram_view
            .write_map16_update_packet(address, vram_pos, tiles);
    }

    pub(crate) fn write_single_tile_stripe_packet(
        &mut self,
        address: usize,
        stripe: u16,
        tile: u16,
    ) {
        self.ram_view
            .write_single_tile_stripe_packet(address, stripe, tile);
    }

    pub(crate) fn write_tile_stripe_sentinel(&mut self, address: usize) {
        self.ram_view.write_tile_stripe_sentinel(address);
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
}
