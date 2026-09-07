//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (vram).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    pub(crate) fn set_overworld_hole_tilemap_pos(&mut self, value: u16) {
        self.world_transient_mut()
            .set_overworld_hole_tilemap_pos(value);
    }

    pub(crate) fn decrement_milestone_item_gfx_swap_countdown(&mut self) {
        self.world_transient_mut()
            .decrement_milestone_item_gfx_swap_countdown();
    }

    pub(crate) fn attract_vram_destination_high_is_clear(&self) -> bool {
        self.game_state
            .display
            .attract_vram_destination_high_is_clear()
    }

    pub(crate) fn attract_vram_destination_page_offset(&self) -> u8 {
        self.game_state
            .display
            .attract_vram_destination_page_offset()
    }

    pub(crate) fn attract_vram_destination_address(&self) -> u16 {
        self.game_state.display.attract_vram_destination_address
    }

    pub(crate) fn set_attract_vram_destination_address(&mut self, value: u16) {
        self.attract_vram_destination_bridge_mut()
            .set_address(value);
    }

    pub(crate) fn clear_attract_vram_destination_address(&mut self) {
        self.attract_vram_destination_bridge_mut().clear_address();
    }

    pub(crate) fn set_attract_vram_destination_page_offset(&mut self, value: u8) {
        self.attract_vram_destination_bridge_mut()
            .set_page_offset(value);
    }

    pub(crate) fn decrement_attract_vram_destination_page_offset(&mut self) {
        self.attract_vram_destination_bridge_mut()
            .decrement_page_offset();
    }

    pub(crate) fn decrement_attract_vram_destination_address(&mut self) -> u16 {
        self.attract_vram_destination_bridge_mut()
            .decrement_address()
    }

    pub(crate) fn set_bg_vram_load_mode(&mut self, value: u8) {
        self.display_core_mut().set_bg_vram_load_mode(value);
    }

    pub(crate) fn queue_tilemap_update(&mut self, destination_page: u8, source_offset: u16) {
        self.display_core_mut()
            .queue_tilemap_update(destination_page, source_offset);
    }

    pub(crate) fn clear_pending_tilemap_update_destination(&mut self) {
        self.display_core_mut()
            .clear_pending_tilemap_update_destination();
    }

    pub(crate) fn clear_bg_vram_load_mode(&mut self) {
        self.display_core_mut().clear_bg_vram_load_mode();
    }

    pub(crate) fn set_chr_halfslot_request(&mut self, value: u8) {
        self.display_core_mut().set_chr_halfslot_request(value);
    }

    pub(crate) fn clear_chr_halfslot_request(&mut self) {
        self.display_core_mut().clear_chr_halfslot_request();
    }

    pub(crate) fn increment_chr_halfslot_request(&mut self) -> u8 {
        self.display_core_mut().increment_chr_halfslot_request()
    }

    pub(crate) fn reset_incremental_vram_upload_counter(&mut self) {
        self.display_core_mut()
            .reset_incremental_vram_upload_counter();
    }

    pub(crate) fn increment_vram_upload_counter(&mut self) -> u8 {
        self.display_core_mut().increment_vram_upload_counter()
    }

    pub(crate) fn set_animated_tile_vram_destination_address(&mut self, value: u16) {
        self.display_core_mut()
            .set_animated_tile_vram_destination_address(value);
    }

    pub(crate) fn copy_tilemap_upload_stripe_bytes(&mut self, bytes: &[u8]) {
        let start = crate::game_state::constants::nmi::TILEMAP_UPLOAD_BUFFER;
        let len = bytes.len().min(self.ram.len().saturating_sub(start));
        self.ram[start..start + len].copy_from_slice(&bytes[..len]);
        let cursor = self
            .game_state
            .display
            .apply_tilemap_upload_prefix_to_vram_cursor(&bytes[..len]);
        write_le_u16(
            &mut self.ram,
            crate::game_state::constants::nmi::VRAM_UPLOAD_OFFSET,
            cursor,
        );
        debug_assert_eq!(
            self.game_state.display.vram_upload_cursor,
            read_le_u16(
                &self.ram,
                crate::game_state::constants::nmi::VRAM_UPLOAD_OFFSET
            )
        );
    }

    pub(crate) fn copy_overworld_sprite_graphics_range(
        &mut self,
        dst: usize,
        data: &[u8],
        src: usize,
        len: usize,
    ) {
        self.overworld_config_table_mut()
            .copy_sprite_graphics_range(dst, data, src, len);
    }

    pub(super) fn debug_assert_hud_tilemap_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.display.hud_tilemap,
            HudTilemapState::load_from_ram(&self.ram)
        );
    }

    pub(crate) fn graphics_primary_decompression_buffer(&self, len: usize) -> Vec<u8> {
        GraphicsDecompressionScratch::primary_buffer(&self.ram, len)
    }

    pub(crate) fn graphics_combined_decompression_buffers(&self) -> Vec<u8> {
        GraphicsDecompressionScratch::combined_buffers(&self.ram)
    }

    pub(crate) fn copy_decompressed_graphics_to(&mut self, dst: usize, data: &[u8]) -> usize {
        GraphicsDecompressionScratch::copy_to_buffer(&mut self.ram, dst, data)
    }

    #[track_caller]
    pub(crate) fn write_expanded_graphics_tile_row(
        &mut self,
        dst: usize,
        low_plane: u8,
        high_plane: u8,
        upper_plane: u8,
        composite_plane: u8,
    ) {
        crate::types::ww_check(dst, 2, "expand_gfx_tile_row[lo/hi]", low_plane as u32);
        crate::types::ww_check(
            dst + 0x10,
            2,
            "expand_gfx_tile_row[up/comp]",
            upper_plane as u32,
        );
        self.ram[dst] = low_plane;
        self.ram[dst + 1] = high_plane;
        self.ram[dst + 0x10] = upper_plane;
        self.ram[dst + 0x11] = composite_plane;
    }

    pub(crate) fn graphics_sprite_decompression_buffer_tail(&self) -> Vec<u8> {
        GraphicsDecompressionScratch::sprite_buffer_tail(&self.ram)
    }

    pub(crate) fn write_decoded_overworld_map32_to_bg2_tilemap(&mut self, dst: usize, idx: usize) {
        OverworldMap16DecodeScratch::write_decoded_map32_to_bg2_tilemap(&mut self.ram, dst, idx);
        // The decode wrote the BG2 tilemap as raw RAM, bypassing the live
        // dungeon tilemap cache. Mirror the four written words back so overworld
        // readers and the frame-end projection stay coherent with RAM.
        self.game_state
            .dungeon
            .room_tilemaps
            .mirror_decoded_map32_from_ram(&self.ram, dst);
    }

    pub(crate) fn vram_upload_buffer_word(&self, offset: usize) -> u16 {
        self.game_state
            .display
            .vram_upload_buffer_word(&self.ram, offset)
    }

    pub(crate) fn vram_upload_tilemap_word(&self, offset: usize) -> u16 {
        self.game_state
            .display
            .vram_upload_tilemap_word(&self.ram, offset)
    }

    pub(crate) fn vram_upload_buffer_byte(&self, offset: usize) -> u8 {
        self.game_state
            .display
            .vram_upload_buffer_byte(&self.ram, offset)
    }

    pub(crate) fn vram_upload_buffer_remaining(&self) -> &[u8] {
        self.game_state
            .display
            .vram_upload_buffer_remaining(&self.ram)
    }

    pub(crate) fn vram_upload_buffer_remaining_len(&self) -> usize {
        self.ram
            .len()
            .saturating_sub(self.game_state.display.vram_upload_buffer_base())
    }

    pub(crate) fn tilemap_upload_stripe_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .tilemap_upload_stripe_buffer(&self.ram)
    }

    pub(crate) fn pending_tilemap_update_source_data(&self) -> &[u8] {
        self.game_state
            .display
            .pending_tilemap_update_source_data(&self.ram)
    }

    pub(crate) fn arbitrary_tilemap_destination(&self, slot: usize) -> u16 {
        self.game_state
            .display
            .arbitrary_tilemap_destination(&self.ram, slot)
    }

    pub(crate) fn bg1_wall_top_tilemap_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .bg1_wall_top_tilemap_buffer(&self.ram)
    }

    pub(crate) fn bg1_wall_bottom_tilemap_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .bg1_wall_bottom_tilemap_buffer(&self.ram)
    }

    pub(crate) fn vram_dma_source_bytes(&self, source_addr: usize, len: usize) -> &[u8] {
        self.game_state
            .display
            .vram_dma_source_bytes(&self.ram, source_addr, len)
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_buffer_byte(&mut self, offset: usize, value: u8) {
        self.vram_upload_mut().write_buffer_byte(offset, value);
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_buffer_word(&mut self, offset: usize, value: u16) {
        self.vram_upload_mut().write_buffer_word(offset, value);
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_tilemap_word(&mut self, offset: usize, value: u16) {
        self.vram_upload_mut().write_tilemap_word(offset, value);
    }

    pub(crate) fn write_overworld_vram_word(&mut self, word_index: usize, value: u16) {
        self.vram_upload_mut()
            .write_overworld_vram_word(word_index, value);
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_absolute_byte(&mut self, address: usize, value: u8) {
        self.vram_upload_mut().write_absolute_byte(address, value);
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_absolute_word(&mut self, address: usize, value: u16) {
        self.vram_upload_mut().write_absolute_word(address, value);
    }

    pub(crate) fn copy_vram_upload_buffer_bytes(&mut self, offset: usize, data: &[u8]) {
        self.vram_upload_mut().copy_buffer_bytes(offset, data);
    }

    pub(crate) fn terminate_vram_upload_buffer_at(&mut self, offset: usize) {
        self.vram_upload_mut().terminate_buffer_at(offset);
    }

    pub(crate) fn write_vram_upload_level_label_tiles(
        &mut self,
        left: &[u8; 14],
        right: &[u8; 14],
    ) {
        self.vram_upload_mut().write_level_label_tiles(left, right);
    }

    pub(crate) fn write_vram_upload_map16_update_packet(
        &mut self,
        address: usize,
        vram_pos: u16,
        tiles: [u16; 4],
    ) {
        self.vram_upload_mut()
            .write_map16_update_packet(address, vram_pos, tiles);
    }

    pub(crate) fn write_vram_upload_single_tile_stripe_packet(
        &mut self,
        address: usize,
        stripe: u16,
        tile: u16,
    ) {
        self.vram_upload_mut()
            .write_single_tile_stripe_packet(address, stripe, tile);
    }

    pub(crate) fn write_vram_upload_tile_stripe_sentinel(&mut self, address: usize) {
        self.vram_upload_mut().write_tile_stripe_sentinel(address);
    }

    pub(crate) fn set_vram_upload_cursor(&mut self, value: u16) {
        self.vram_upload_mut().set_offset(value);
    }

    pub(crate) fn clear_vram_upload_cursor(&mut self) {
        self.vram_upload_mut().clear_offset();
    }

    pub(crate) fn advance_vram_upload_cursor_by(&mut self, value: u16) -> u16 {
        self.vram_upload_mut().advance_offset_by(value)
    }

    /// Publish the CPU/OAM generation authored after an atomic item-graphics
    /// caller returns.  The source-authoritative terminal chest path and the
    /// legacy measured completion share this postlude, but own their interrupt
    /// and common suffix at different dispatch layers.
    pub(super) fn complete_atomic_item_graphics_return_postlude(
        &mut self,
        continuation: ItemReceiptGraphicsContinuation,
    ) {
        let mut law = vec![0u16; self.ppu.oam.len()];
        if publish_oam_shadow(&mut law, self.sprite_oam_shadow_buffer()) {
            if nmi::debug_frame_selection_env_matches(
                "ZELDA3_DEBUG_OAM_LAW_EVENTS",
                self.frame_ctr_dbg,
            ) {
                eprintln!(
                    "oam_law_transfer host={} w204={:04x} fc={:02x} link_y={:04x} bg2v={:04x} (item-completion)",
                    self.frame_ctr_dbg,
                    law[204],
                    self.game_state.frame.frame_counter,
                    read_le_u16(&self.ram, crate::game_state::constants::LINK_Y_COORD),
                    read_le_u16(&self.ram, crate::game_state::constants::BG2_Y_SCROLL),
                );
            }
            self.oam_law_pending = Some(law);
        }
        self.stage_atomic_item_graphics_return_obj_scanout(continuation);
    }

    pub(super) fn compose_display_vram(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
        retained_full_tilemap_vram: Option<&RetainedVramRegion>,
    ) {
        let debug_vram_frame = crate::debug_env::var("ZELDA3_DEBUG_DISPLAY_VRAM_FRAME")
            .ok()
            .and_then(|frame| frame.parse::<u32>().ok())
            .is_some_and(|frame| frame == self.frame_ctr_dbg);
        // The polygon worker publishes through its NMI handshake at the start
        // of the frame. Preserve that completed pre-NMI buffer rather than a
        // job that may have finished later in the current CPU slice.
        let presented_poly = self.selected_intro_poly_display_buffer();
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or_else(|| crate::game_state::FrameState::load_from_ram(&self.ram));
        let following_frame = crate::game_state::FrameState::load_from_ram(&following.ram);
        let host_boundary_link_obj_vram = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.obj_vram[0x4000..0x4400].to_vec())
            .unwrap_or_else(|| self.ppu.vram[0x4000..0x4400].to_vec());
        let entry_link_obj_vram = (matches!(
            plan.link_obj_scanout_generation,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        ) && matches!(
            plan.link_obj_source_generation,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        ))
        .then(|| host_boundary_link_obj_vram.clone());
        let following_room = crate::game_state::WorldLocationState::load_from_ram(&following.ram)
            .dungeon_room_index();
        let completed_item_return_publishes_live_tilemap = (plan.oam_scanout_source
            == OamScanoutSource::ComposeLivePlayerOamAfterMain
            && plan.link_obj_scanout_generation == GraphicsDmaGeneration::LiveAfterMain
            && plan.link_obj_source_generation == GraphicsDmaGeneration::LiveAfterMain
            && following.hud_vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi)
            || (following.interrupted_item_receipt_obj_cache
                && plan.oam_scanout_source == OamScanoutSource::ComposePublishedShadowDma);
        if debug_vram_frame {
            eprintln!(
                "display_vram_candidates host={} room={:04x} entry={:02x}/{:02x}/{:02x} following={:02x}/{:02x}/{:02x} vram={:?} hud={:?}@{:04x} link={:?}/{:?} animated={:?} oam={:?} item_return_cache={} item_return_tilemap={} captured_0798={:04x} following_0798={:04x} captured_089d={:04x} following_089d={:04x} captured_0c00={:04x} following_0c00={:04x} captured_1193={:04x} following_1193={:04x} captured_3b00={:04x} following_3b00={:04x} captured_hud={:04x}/{:04x}/{:04x}/{:04x} following_hud={:04x}/{:04x}/{:04x}/{:04x} captured_60c3={:04x} following_60c3={:04x}",
                self.frame_ctr_dbg,
                following_room,
                entry_frame.main_module,
                entry_frame.submodule,
                entry_frame.subsubmodule,
                following_frame.main_module,
                following_frame.submodule,
                following_frame.subsubmodule,
                plan.vram_generation,
                following.hud_vram_generation,
                following.hud_vram_destination,
                plan.link_obj_scanout_generation,
                plan.link_obj_source_generation,
                plan.animated_bg_scanout_generation,
                plan.oam_scanout_source,
                following.interrupted_item_receipt_obj_cache,
                completed_item_return_publishes_live_tilemap,
                self.ppu.vram[0x0798],
                following.ppu.vram[0x0798],
                self.ppu.vram[0x089d],
                following.ppu.vram[0x089d],
                self.ppu.vram[0x0c00],
                following.ppu.vram[0x0c00],
                self.ppu.vram[0x1193],
                following.ppu.vram[0x1193],
                self.ppu.vram[0x3b00],
                following.ppu.vram[0x3b00],
                self.ppu.vram[0x60b9],
                self.ppu.vram[0x60ba],
                self.ppu.vram[0x60d9],
                self.ppu.vram[0x60da],
                following.ppu.vram[0x60b9],
                following.ppu.vram[0x60ba],
                following.ppu.vram[0x60d9],
                following.ppu.vram[0x60da],
                self.ppu.vram[0x60c3],
                following.ppu.vram[0x60c3],
            );
        }
        let retained_doorway_link_obj_vram = dungeon_transition_retains_presented_link_vram(
            entry_frame,
            following_frame,
            following_room,
        )
        .then(|| {
            [
                self.ppu.vram[0x4000..0x4050].to_vec(),
                self.ppu.vram[0x4100..0x4150].to_vec(),
            ]
        });
        let retained_hud_vram = matches!(
            following.hud_vram_generation,
            DisplayVramGeneration::RetainCapturedBeforeNmi
        )
        .then(|| {
            RetainedVramRegion::capture(
                &self.ppu.vram,
                following.hud_vram_destination,
                HUD_TILEMAP_NMI_WORDS,
            )
        })
        .flatten();
        let retained_nmi_copy_packet_vram = (self.ram[NMI_COPY_PACKETS_FLAG] != 0
            && !plan.publish_live_nmi_copy_packets)
            .then(|| {
                NmiCopyPacketScanout::capture(
                    &self.ppu.vram,
                    &self.ram[crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF..],
                )
            });
        // Animated-BG DMA configures the next active frame. Retain the
        // host-boundary VRAM generation at whichever destination the current
        // tileset selected ($3b00 indoors, $3c00 outdoors). The resumed
        // bad-weather tail is the measured exception that publishes the live
        // post-NMI generation.
        let previous_animated_bg_vram = (plan.animated_bg_scanout_generation
            == AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi)
            .then(|| {
                following
                    .host_boundary_animated_bg_scanout
                    .as_ref()
                    .filter(|scanout| {
                        scanout.destination_address + scanout.vram.len() <= self.ppu.vram.len()
                    })
                    .map(|scanout| (scanout.destination_address, scanout.vram.clone()))
            })
            .flatten();
        if plan.vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi {
            self.ppu.vram.clone_from(&following.ppu.vram);
            if let Some((destination, animated_bg_vram)) = previous_animated_bg_vram.as_ref() {
                self.ppu.vram[*destination..*destination + animated_bg_vram.len()]
                    .copy_from_slice(animated_bg_vram);
            }
            if let Some(entry_link_obj_vram) = entry_link_obj_vram {
                self.ppu.vram[0x4000..0x4400].copy_from_slice(&entry_link_obj_vram);
            }
            if let Some([upper, lower]) = retained_doorway_link_obj_vram {
                self.ppu.vram[0x4000..0x4050].copy_from_slice(&upper);
                self.ppu.vram[0x4100..0x4150].copy_from_slice(&lower);
            }
            if let Some(retained_hud_vram) = retained_hud_vram.as_ref() {
                retained_hud_vram.publish_to(&mut self.ppu.vram);
            }
            if let Some(scanout) = retained_nmi_copy_packet_vram.as_ref() {
                scanout.publish_to(&mut self.ppu.vram);
            }
            self.ppu.vram[0x5800..0x5c00].copy_from_slice(&presented_poly);
        } else if matches!(
            plan.link_obj_scanout_generation,
            GraphicsDmaGeneration::LiveAfterMain
        ) {
            if matches!(
                plan.oam_scanout_source,
                OamScanoutSource::ComposePublishedShadowDma
                    | OamScanoutSource::ComposeLivePlayerOamAfterMain
            ) {
                self.ppu.vram[0x4000..0x4400].copy_from_slice(&following.ppu.vram[0x4000..0x4400]);
            } else if !self.atomic_item_graphics_holds_following_nmi() {
                // A long NMI can retain the pre-upload BG/tilemap image while its
                // OBJ DMA still completes before the sprites are scanned. Keep
                // this domain independent: Snes9x's completed tile cache can own
                // the early Link body/head/hand batch while the rest of visible
                // VRAM, including later Link OBJ uploads, remains captured.
                for range in [0x4000..0x4050, 0x4100..0x4150] {
                    self.ppu.vram[range.clone()].copy_from_slice(&following.ppu.vram[range]);
                }
            } else {
                // Resolve that DMA from the scanout snapshot's own source words.
                // The following coarse host slice can already have latched a long
                // item-graphics call and skipped NMI_DoUpdates, leaving its PPU
                // bytes one upload behind even though the snapshot owns the next
                // hardware generation (oracle frame 4582).
                let captured_sources = LinkDmaSources::load_from_ram(&self.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                    let source_address = usize::from(captured_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    if let Some(link_graphics) = link_graphics.as_deref().filter(|graphics| {
                        source_address >= 0x8000 && source_offset + len <= graphics.len()
                    }) {
                        self.copy_asset_bytes_to_vram(
                            destination,
                            link_graphics,
                            source_address,
                            len,
                        );
                    } else {
                        self.ppu.vram[destination..destination_end]
                            .copy_from_slice(&following.ppu.vram[destination..destination_end]);
                    }
                }
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi {
            if let Some((destination, animated_bg_vram)) = previous_animated_bg_vram.as_ref() {
                self.ppu.vram[*destination..*destination + animated_bg_vram.len()]
                    .copy_from_slice(animated_bg_vram);
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && plan.animated_bg_scanout_generation == AnimatedBgScanoutGeneration::LiveAfterNmi
        {
            // A pending stripe can retain the general VRAM generation while
            // the leading-NMI animated-tile DMA independently completes for
            // this scanout. Compose that $400-byte domain explicitly; merely
            // selecting LiveAfterNmi is otherwise a no-op in the retained
            // whole-VRAM branch above.
            let destination = read_le_u16(&following.ram, ANIMATED_TILE_VRAM_ADDR) as usize;
            const ANIMATED_BG_NMI_WORDS: usize = 0x200;
            if destination != 0 {
                if let (Some(live), Some(presented)) = (
                    following
                        .ppu
                        .vram
                        .get(destination..destination + ANIMATED_BG_NMI_WORDS),
                    self.ppu
                        .vram
                        .get_mut(destination..destination + ANIMATED_BG_NMI_WORDS),
                ) {
                    presented.copy_from_slice(live);
                }
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && following.hud_vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi
            && following.hud_vram_destination != 0
        {
            // Item-receipt return retains the room/tile animation generation,
            // but the HUD packet has completed and is independently visible.
            // Publish just that packet's destination instead of promoting the
            // entire post-NMI VRAM image.
            let destination = following.hud_vram_destination;
            if let (Some(live), Some(presented)) = (
                following
                    .ppu
                    .vram
                    .get(destination..destination + HUD_TILEMAP_NMI_WORDS),
                self.ppu
                    .vram
                    .get_mut(destination..destination + HUD_TILEMAP_NMI_WORDS),
            ) {
                presented.copy_from_slice(live);
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && completed_item_return_publishes_live_tilemap
        {
            // The item-receipt return crosses the tilemap DMA before the
            // caller's held-item OAM is scanned. BG1/BG2 map words therefore
            // belong to the live post-NMI image even though their CHR and the
            // rest of general VRAM retain the host-boundary generation.
            self.ppu.vram[0x0000..0x2000].copy_from_slice(&following.ppu.vram[0x0000..0x2000]);
        }
        if let Some(retained_full_tilemap_vram) = retained_full_tilemap_vram {
            retained_full_tilemap_vram.publish_to(&mut self.ppu.vram);
        }
        if plan.publish_post_main_hud_dma && following.hud_vram_destination != 0 {
            // Entering Module07_0A crosses the trailing vblank after Link has
            // spent lamp magic and the common dungeon suffix has rebuilt the
            // HUD. The oracle therefore scans the newly-authored HUD buffer,
            // even though the resident live PPU still contains the leading-NMI
            // upload. Compose that independently-published DMA domain.
            let destination = following.hud_vram_destination;
            if let Some(presented) = self
                .ppu
                .vram
                .get_mut(destination..destination + HUD_TILEMAP_NMI_WORDS)
            {
                for (index, word) in presented.iter_mut().enumerate() {
                    *word = read_le_u16(&following.ram, HUD_TILE_INDICES_BUFFER + index * 2);
                }
            }
        }
        if debug_vram_frame {
            eprintln!(
                "display_vram_selected host={} selected_0798={:04x} selected_089d={:04x} selected_0c00={:04x} selected_1193={:04x} selected_3b00={:04x} selected_hud={:04x}/{:04x}/{:04x}/{:04x} selected_60c3={:04x} retained_copy_packet={}",
                self.frame_ctr_dbg,
                self.ppu.vram[0x0798],
                self.ppu.vram[0x089d],
                self.ppu.vram[0x0c00],
                self.ppu.vram[0x1193],
                self.ppu.vram[0x3b00],
                self.ppu.vram[0x60b9],
                self.ppu.vram[0x60ba],
                self.ppu.vram[0x60d9],
                self.ppu.vram[0x60da],
                self.ppu.vram[0x60c3],
                retained_nmi_copy_packet_vram.is_some(),
            );
        }
    }

    pub(super) fn compose_display_chr_sources(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
    ) {
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or_else(|| crate::game_state::FrameState::load_from_ram(&self.ram));
        let following_frame = crate::game_state::FrameState::load_from_ram(&following.ram);
        let following_room = crate::game_state::WorldLocationState::load_from_ram(&following.ram)
            .dungeon_room_index();
        let independently_published_link_sources = dungeon_transition_retains_presented_link_vram(
            entry_frame,
            following_frame,
            following_room,
        )
        .then(|| {
            (
                self.pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.link_operands)
                    .unwrap_or_else(|| PreMainLinkDmaOperands::capture(&self.ram)),
                self.vram_chr_source.clone(),
                self.vram_chr_preview_source.clone(),
            )
        });
        let dungeon_supertile_state3_retains_presented_link_sources = following_frame.main_module
            == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 3;
        let dungeon_supertile_state8_publishes_completed_link_sources = following_frame.main_module
            == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 8;
        let retained_link_obj_sources = (matches!(
            plan.link_obj_source_generation,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        )
            || dungeon_supertile_state3_retains_presented_link_sources)
            .then(|| {
                if dungeon_supertile_state8_publishes_completed_link_sources {
                    // The held state-8 NMI writes the host-boundary Link bytes
                    // before main advances the animation. Keep those visible
                    // bytes, but publish the logical identity recorded by that
                    // completed NMI rather than the older snapshot metadata.
                    (
                        following.vram_chr_source.clone(),
                        following.vram_chr_preview_source.clone(),
                    )
                } else {
                    (
                        self.vram_chr_source.clone(),
                        self.vram_chr_preview_source.clone(),
                    )
                }
            });
        if plan.vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi {
            self.vram_chr_source.clone_from(&following.vram_chr_source);
            self.vram_chr_preview_source
                .clone_from(&following.vram_chr_preview_source);
            if let Some((logical, preview)) = retained_link_obj_sources.as_ref() {
                self.vram_chr_source
                    .copy_word_range_from(logical, 0x4000..0x4400);
                self.vram_chr_preview_source
                    .copy_word_range_from(preview, 0x4000..0x4400);
            }
        } else if matches!(
            plan.link_obj_source_generation,
            GraphicsDmaGeneration::LiveAfterMain
        ) {
            if matches!(
                plan.oam_scanout_source,
                OamScanoutSource::ComposePublishedShadowDma
                    | OamScanoutSource::ComposeLivePlayerOamAfterMain
            ) {
                self.vram_chr_source
                    .copy_word_range_from(&following.vram_chr_source, 0x4000..0x4400);
                self.vram_chr_preview_source
                    .copy_word_range_from(&following.vram_chr_preview_source, 0x4000..0x4400);
            } else if !self.atomic_item_graphics_holds_following_nmi()
                && !dungeon_supertile_state3_retains_presented_link_sources
            {
                for range in [0x4000..0x4050, 0x4100..0x4150] {
                    self.vram_chr_source
                        .copy_word_range_from(&following.vram_chr_source, range.clone());
                    self.vram_chr_preview_source
                        .copy_word_range_from(&following.vram_chr_preview_source, range);
                }
            } else if !dungeon_supertile_state3_retains_presented_link_sources {
                let captured_sources = LinkDmaSources::load_from_ram(&self.ram);
                let link_graphics_len = self.asset_raw(57).map(<[u8]>::len);
                let link_pack = read_le_u16(
                    &self.ram,
                    crate::game_state::constants::LINK_DMA_GRAPHICS_INDEX,
                ) >> 1;
                for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                    let source_address = usize::from(captured_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    if source_address >= 0x8000
                        && link_graphics_len
                            .is_some_and(|asset_len| source_offset + len <= asset_len)
                    {
                        let tile_count = (len / 2).div_ceil(16);
                        let base_offset = (source_offset >> 5) as u16;
                        self.vram_chr_source.record_tiles_from(
                            destination,
                            tile_count,
                            crate::chr_source::CHR_KIND_LINK,
                            link_pack,
                            base_offset,
                        );
                        self.vram_chr_preview_source.record_tiles_from(
                            destination,
                            tile_count,
                            crate::chr_source::CHR_KIND_LINK,
                            link_pack,
                            base_offset,
                        );
                    } else {
                        let range = destination..destination + len / 2;
                        self.vram_chr_source
                            .copy_word_range_from(&following.vram_chr_source, range.clone());
                        self.vram_chr_preview_source
                            .copy_word_range_from(&following.vram_chr_preview_source, range);
                    }
                }
            }
        }
        if plan.animated_bg_scanout_generation == AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
        {
            if let Some(scanout) = following.host_boundary_animated_bg_scanout.as_ref() {
                let range = scanout.destination_address
                    ..scanout
                        .destination_address
                        .saturating_add(scanout.vram.len());
                self.vram_chr_source
                    .copy_word_range_from(&scanout.logical_sources, range.clone());
                self.vram_chr_preview_source
                    .copy_word_range_from(&scanout.preview_sources, range);
            }
        }
        if dungeon_supertile_state3_retains_presented_link_sources {
            if let Some(logical) = self.last_presented_vram_chr_source.as_ref() {
                self.vram_chr_source
                    .copy_word_range_from(logical, 0x4000..0x4400);
            }
            if let Some(preview) = self.last_presented_vram_chr_preview_source.as_ref() {
                self.vram_chr_preview_source
                    .copy_word_range_from(preview, 0x4000..0x4400);
            }
        }
        if let Some((operands, retained_logical, retained_preview)) =
            independently_published_link_sources.as_ref()
        {
            let link_graphics_len = self.asset_raw(57).map(<[u8]>::len);
            for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                let source_address = usize::from(operands.sources.source(source));
                let source_offset = source_address.saturating_sub(0x8000);
                let range = destination..destination + len / 2;
                if source_address >= 0x8000
                    && link_graphics_len.is_some_and(|asset_len| source_offset + len <= asset_len)
                {
                    let tile_count = (len / 2).div_ceil(16);
                    let base_offset = (source_offset >> 5) as u16;
                    self.vram_chr_source.record_tiles_from(
                        destination,
                        tile_count,
                        crate::chr_source::CHR_KIND_LINK,
                        operands.link_pack,
                        base_offset,
                    );
                    self.vram_chr_preview_source.record_tiles_from(
                        destination,
                        tile_count,
                        crate::chr_source::CHR_KIND_LINK,
                        operands.link_pack,
                        base_offset,
                    );
                } else {
                    self.vram_chr_source
                        .copy_word_range_from(retained_logical, range.clone());
                    self.vram_chr_preview_source
                        .copy_word_range_from(retained_preview, range);
                }
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && plan.animated_bg_scanout_generation == AnimatedBgScanoutGeneration::LiveAfterNmi
        {
            let destination = read_le_u16(&following.ram, ANIMATED_TILE_VRAM_ADDR) as usize;
            if destination != 0 {
                let range = destination..destination + 0x200;
                self.vram_chr_source
                    .copy_word_range_from(&following.vram_chr_source, range.clone());
                self.vram_chr_preview_source
                    .copy_word_range_from(&following.vram_chr_preview_source, range);
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && following.hud_vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi
            && following.hud_vram_destination != 0
        {
            // HUD tile bytes can already match the resident PPU image while
            // their modern CHR ownership was authored by the just-completed
            // item-receipt return. Keep the raw retained scanout, but publish
            // the matching BG3 source identities with it; otherwise the modern
            // renderer treats an exact HUD icon as transparent.
            let bg3_chr_start = usize::from(following.ppu.bg_layer[2].tile_adr);
            let bg3_chr_end = bg3_chr_start.saturating_add(0x1000).min(0x8000);
            if bg3_chr_start < bg3_chr_end {
                let range = bg3_chr_start..bg3_chr_end;
                self.vram_chr_source
                    .copy_word_range_from(&following.vram_chr_source, range.clone());
                self.vram_chr_preview_source
                    .copy_word_range_from(&following.vram_chr_preview_source, range);
            }
        }
    }

    pub(super) fn mark_effective_dma_vram_word(&mut self, index: usize) {
        if let Some(writes) = self.active_effective_dma_writes.as_mut() {
            if let Some(written) = writes.vram_words.get_mut(index) {
                *written = true;
            }
        }
    }

    pub(super) fn mark_effective_dma_vram_range(&mut self, range: std::ops::Range<usize>) {
        if let Some(writes) = self.active_effective_dma_writes.as_mut() {
            let end = range.end.min(writes.vram_words.len());
            if range.start < end {
                writes.vram_words[range.start..end].fill(true);
            }
        }
    }

    pub(super) fn compose_effective_presented_vram(&mut self, following: &DisplaySnapshot) {
        let Some(receipt) = following.effective_presented_dma.as_ref() else {
            return;
        };

        // These are observed writes from the explicit leading NMI attached to
        // this immutable scanout, not a prediction from live module state. A
        // coarse retained-memory baseline can still own every untouched word,
        // while each DMA destination advances to the value installed before
        // visible scanout begins.
        for &(index, value) in &receipt.vram_writes {
            if let Some(word) = self.ppu.vram.get_mut(index) {
                *word = value;
            }
        }
    }

    pub(super) fn compose_decoded_bg_chr_cache(&mut self) {
        let destination = read_le_u16(&self.ram, ANIMATED_TILE_VRAM_ADDR) as usize;
        const ANIMATED_BG_NMI_WORDS: usize = 0x200;
        let Some(decoded) = self
            .ppu
            .bg_vram_latch
            .as_deref()
            .and_then(|cache| {
                cache.get(destination..destination.saturating_add(ANIMATED_BG_NMI_WORDS))
            })
            .map(Vec::from)
        else {
            return;
        };
        let Some(presented) = self
            .ppu
            .vram
            .get(destination..destination.saturating_add(ANIMATED_BG_NMI_WORDS))
        else {
            self.ppu.bg_vram_latch = None;
            return;
        };
        if presented == decoded.as_slice() {
            self.ppu.bg_vram_latch = None;
            return;
        }
        let mut cache = self.ppu.vram.clone();
        cache[destination..destination + ANIMATED_BG_NMI_WORDS].copy_from_slice(&decoded);
        self.ppu.bg_vram_latch = Some(cache);
    }

    pub(super) fn compose_effective_presented_bg_chr_cache(&mut self, following: &DisplaySnapshot) {
        let Some(receipt) = following.effective_presented_dma.as_ref() else {
            return;
        };
        if receipt.decoded_bg_vram_writes.is_empty() {
            return;
        }

        // Only an explicit leading-NMI receipt can refine renderer pattern
        // data independently. Ordinary trailing DMA is resident state for a
        // later capture and never enters this path.
        let mut decoded = self
            .ppu
            .bg_vram_latch
            .take()
            .unwrap_or_else(|| self.ppu.vram.clone());
        for &(index, value) in &receipt.decoded_bg_vram_writes {
            if let Some(word) = decoded.get_mut(index) {
                *word = value;
            }
        }
        self.ppu.bg_vram_latch = (decoded != self.ppu.vram).then_some(decoded);
    }

    pub fn vram(&self) -> &[u16] {
        &self.ppu.vram
    }

    /// Read-only access to the per-VRAM-slot logical CHR source table
    /// (animation-modeled asset renderer M1 bookkeeping).
    pub fn vram_chr_source(&self) -> &crate::chr_source::VramChrSourceTable {
        &self.vram_chr_source
    }

    /// Read-only access to the raw per-VRAM-slot CHR source table used by
    /// authoring/preview tooling. This table preserves sprite pack/tile identity
    /// even when the render source table is content-hashed for correctness.
    pub fn vram_chr_preview_source(&self) -> &crate::chr_source::VramChrSourceTable {
        &self.vram_chr_preview_source
    }

    pub fn vram_mut(&mut self) -> &mut [u16] {
        &mut self.ppu.vram
    }

    pub fn apply_link_graphics(&mut self, file: &[u8]) -> bool {
        if file.len() < 27 || &file[0..4] != b"ZSPR" {
            return false;
        }

        let Ok(pixel_offs) = read_le_u32(file, 9).map(|v| v as usize) else {
            return false;
        };
        let pixel_length = read_le_u16(file, 13) as usize;
        let Ok(palette_offs) = read_le_u32(file, 15).map(|v| v as usize) else {
            return false;
        };
        let palette_length = read_le_u16(file, 19) as usize;
        let pixel_end = match pixel_offs.checked_add(pixel_length) {
            Some(end) => end,
            None => return false,
        };
        let palette_end = match palette_offs.checked_add(palette_length) {
            Some(end) => end,
            None => return false,
        };
        if pixel_end > file.len() || palette_end > file.len() || pixel_length != 0x7000 {
            return false;
        }

        let Some(assets) = self.assets.as_mut() else {
            return false;
        };
        if assets.asset(57).map(|asset| asset.len()) != Some(0x7000)
            || assets.asset(81).map(|asset| asset.len()) != Some(150)
        {
            return false;
        }

        let Some(link_graphics) = assets.asset_mut(57) else {
            return false;
        };
        link_graphics.copy_from_slice(&file[pixel_offs..pixel_offs + 0x7000]);

        if palette_length >= 120 {
            let Some(armor_and_gloves) = assets.asset_mut(81) else {
                return false;
            };
            armor_and_gloves[..120].copy_from_slice(&file[palette_offs..palette_offs + 120]);
        }
        if palette_length >= 124 {
            self.gloves_color = [
                read_word_from_slice(file, palette_offs + 120),
                read_word_from_slice(file, palette_offs + 122),
            ];
        }

        true
    }

    pub(super) fn emu_synchronize_whole_state(&mut self) {
        if let Some(sync_all) = self.emu_syncall {
            sync_all(self);
        }
    }
}
