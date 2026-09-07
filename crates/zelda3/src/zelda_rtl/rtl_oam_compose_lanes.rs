//! Blocks of `compose_display_oam` split out by topic (mechanically extracted;
//! bodies unchanged).
#![allow(clippy::too_many_arguments)]

use super::*;
use crate::game_state::FrameState;

impl ZeldaState {
    /// Extracted from `compose_display_oam` (mechanical move; body unchanged).
    pub(super) fn compose_oam_oam_scanout_source_retaincapturedbeforenmi(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
        active_oam_dma_defers_to_prior_provenance: bool,
        completed_dma_is_coherent: bool,
        entry_frame: FrameState,
        following_frame: FrameState,
        last_published_oam: Option<Vec<u16>>,
        main_iteration_completed: bool,
        published_shadow_oam: Option<&[u16]>,
    ) {
        if plan.oam_scanout_source == OamScanoutSource::RetainCapturedBeforeNmi
            && !game_over_menu_retains_resident_oam(
                entry_frame,
                following_frame,
                plan.oam_scanout_source,
            )
        {
            // A capture and the OAM DMA it precedes are distinct hardware
            // events. If no later DMA identifies the resident generation and
            // main completed, the queued shadow is the retiring transfer.
            // Otherwise retain the buffer that actually reached scanout.
            let published_oam = if (following.completed_oam_dma_after_capture.is_none()
                || !completed_dma_is_coherent
                || active_oam_dma_defers_to_prior_provenance)
                && main_iteration_completed
            {
                published_shadow_oam
            } else {
                last_published_oam.as_deref()
            };
            if let Some(published_oam) = published_oam.filter(|oam| oam.len() == self.ppu.oam.len())
            {
                self.ppu.oam.clone_from_slice(published_oam);
            }
        } else if matches!(
            plan.oam_scanout_source,
            OamScanoutSource::ComposePublishedShadowDma
                | OamScanoutSource::ComposeHostBoundaryShadowDma
        ) && !game_over_menu_retains_resident_oam(
            entry_frame,
            following_frame,
            plan.oam_scanout_source,
        ) {
            // The snapshot's PPU OAM includes main-thread shadow writes that
            // hardware has not necessarily DMAed yet. When scanout retains the
            // pre-NMI generation, use the shadow image published by the prior
            // completed OAM DMA rather than those newly authored coordinates.
            if let Some(published_shadow_oam) =
                published_shadow_oam.filter(|oam| oam.len() == self.ppu.oam.len())
            {
                self.ppu.oam.clone_from_slice(published_shadow_oam);
            }
        }
    }

    /// Extracted from `compose_display_oam` (mechanical move; body unchanged).
    pub(super) fn compose_oam_straight_interroom_fadeout_main_slice_publishes_host_oam(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
        entry_frame: FrameState,
        following_frame: FrameState,
        following_room: u8,
    ) {
        if straight_interroom_fadeout_main_slice_publishes_host_oam(
            entry_frame,
            following_frame,
            following_room,
            crate::game_state::DungeonState::load_from_ram(&following.ram)
                .stair_movement
                .staircase_index(),
            plan.oam_scanout_source,
        ) {
            // A completed palette-loop main slice crosses the leading OAM DMA.
            // The unchanged-frame caller slices between those completions must
            // continue retaining captured OAM, so the frame-counter edge is the
            // semantic publication discriminator rather than palette parity.
            if let Some(host_boundary) = self.pre_main_graphics_dma.as_ref() {
                publish_oam_shadow(&mut self.ppu.oam, &host_boundary.oam_shadow);
            }
        } else if straight_interroom_quadrant_pipeline_publishes_host_oam(
            entry_frame,
            following_frame,
            following_room,
            crate::game_state::DungeonState::load_from_ram(&following.ram)
                .stair_movement
                .staircase_index(),
        ) {
            // Each post-NMI CPU state authors a new shadow after hardware has
            // already selected the host-boundary OAM DMA for this scanout.
            if let Some(host_boundary) = self.pre_main_graphics_dma.as_ref() {
                publish_oam_shadow(&mut self.ppu.oam, &host_boundary.oam_shadow);
            }
        } else if straight_interroom_palette_caller_retains_presented_display(
            entry_frame,
            following_frame,
            following_room,
            crate::game_state::DungeonState::load_from_ram(&following.ram)
                .stair_movement
                .staircase_index(),
        ) {
            // The unchanged-frame palette caller has no completed OAM DMA;
            // retain the same display generation as its CGRAM publication.
            if let Some(oam) = self.last_presented_oam.as_ref() {
                self.ppu.oam.copy_from_slice(oam);
            }
        } else if straight_interroom_fadeout_no_nmi_caller_retains_presented_display(
            entry_frame,
            following_frame,
            following_room,
            crate::game_state::DungeonState::load_from_ram(&following.ram)
                .stair_movement
                .staircase_index(),
            following.ram[crate::game_state::constants::NMI_BOOLEAN],
            plan.link_obj_scanout_generation,
        ) {
            // The caller-only slice has no OAM DMA edge either. Keep the exact
            // sprite generation published by the preceding completed slice.
            if let Some(oam) = self.last_presented_oam.as_ref() {
                self.ppu.oam.copy_from_slice(oam);
            }
        }
    }

    /// Extracted from `compose_display_oam` (mechanical move; body unchanged).
    pub(super) fn compose_oam_debug_captured_oam_as_deref(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
        debug_captured_oam: Option<Vec<u16>>,
        debug_following_shadow_oam: Option<Vec<u16>>,
        debug_host_boundary_oam: Option<Vec<u16>>,
        debug_last_presented_oam: Option<Vec<u16>>,
        following_frame: FrameState,
        following_room: u8,
    ) {
        if let Some(captured) = debug_captured_oam.as_deref() {
            let host_boundary = debug_host_boundary_oam.as_deref();
            let published = following.published_shadow_oam_dma.as_deref();
            let last_presented = debug_last_presented_oam.as_deref();
            let following_shadow = debug_following_shadow_oam.as_deref();
            let entries = (0..128)
                .filter(|&entry| {
                    let composed = oam_entry_bytes(&self.ppu.oam, entry);
                    [host_boundary, published, last_presented, following_shadow]
                        .into_iter()
                        .flatten()
                        .any(|oam| oam_entry_bytes(oam, entry) != composed)
                        || oam_entry_bytes(captured, entry) != composed
                        || oam_entry_bytes(&following.ppu.oam, entry) != composed
                })
                .take(32)
                .map(|entry| {
                    (
                        entry,
                        oam_entry_bytes(captured, entry),
                        host_boundary.map(|oam| oam_entry_bytes(oam, entry)),
                        published.map(|oam| oam_entry_bytes(oam, entry)),
                        last_presented.map(|oam| oam_entry_bytes(oam, entry)),
                        oam_entry_bytes(&following.ppu.oam, entry),
                        following_shadow.map(|oam| oam_entry_bytes(oam, entry)),
                        oam_entry_bytes(&self.ppu.oam, entry),
                    )
                })
                .collect::<Vec<_>>();
            let extended = (256..self.ppu.oam.len())
                .filter_map(|word| {
                    let composed = self.ppu.oam[word];
                    let values = (
                        captured[word],
                        host_boundary.map(|oam| oam[word]),
                        published.map(|oam| oam[word]),
                        last_presented.map(|oam| oam[word]),
                        following.ppu.oam[word],
                        composed,
                    );
                    (values.0 != composed
                        || values.1.is_some_and(|value| value != composed)
                        || values.2.is_some_and(|value| value != composed)
                        || values.3.is_some_and(|value| value != composed)
                        || values.4 != composed)
                        .then_some((word, values))
                })
                .collect::<Vec<_>>();
            let focus_entries =
                [12, 13, 14, 15, 16, 17, 102, 103, 104, 105, 110, 111].map(|entry| {
                    (
                        entry,
                        oam_entry_bytes(captured, entry),
                        debug_host_boundary_oam
                            .as_deref()
                            .map(|oam| oam_entry_bytes(oam, entry)),
                        published.map(|oam| oam_entry_bytes(oam, entry)),
                        last_presented.map(|oam| oam_entry_bytes(oam, entry)),
                        oam_entry_bytes(&following.ppu.oam, entry),
                        following_shadow.map(|oam| oam_entry_bytes(oam, entry)),
                        oam_entry_bytes(&self.ppu.oam, entry),
                    )
                });
            eprintln!(
                "display_oam_frame host={} room={:04x} phase={:02x}/{:02x}/{:02x} source={:?} staged_by={:?} retain={} obj={} sorting={:02x}->{:02x} offset={:04x}->{:04x} entries=(entry,captured,host,published,last,following_ppu,following_shadow,composed){entries:02x?} focus={focus_entries:02x?} extended=(word,captured,host,published,last,following,composed){extended:02x?}",
                self.frame_ctr_dbg,
                following_room,
                following_frame.main_module,
                following_frame.submodule,
                following_frame.subsubmodule,
                plan.oam_scanout_source,
                following.obj_scanout_provenance,
                plan.retain_captured_oam,
                following.obj_generation.name(),
                self.ram[crate::game_state::constants::SORT_SPRITES_SETTING],
                following.ram[crate::game_state::constants::SORT_SPRITES_SETTING],
                read_le_u16(
                    &self.ram,
                    crate::game_state::constants::SORT_SPRITES_OFFSET_INTO_OAM_BUFFER,
                ),
                read_le_u16(
                    &following.ram,
                    crate::game_state::constants::SORT_SPRITES_OFFSET_INTO_OAM_BUFFER,
                ),
            );
        }
    }

    /// Extracted from `compose_display_oam` (mechanical move; body unchanged).
    pub(super) fn compose_oam_room_82_horizontal_first_scroll_publishes_host_boundary(
        &mut self,
        entry_shadow: Option<&[u8]>,
        retain_room_82_horizontal_sprite_conversion_return_oam: bool,
        room_82_horizontal_deferred_nmi_publishes_entry_shadow: bool,
        room_82_horizontal_first_scroll_publishes_host_boundary: bool,
        room_82_horizontal_quadrant_filter_entry_publishes_host_boundary: bool,
        room_82_horizontal_state3_followup: bool,
    ) {
        if room_82_horizontal_first_scroll_publishes_host_boundary {
            // State 8 starts after the held filtering return has completed its
            // host-boundary OAM work. The queued capture still describes the
            // earlier filtering slice, before Link's diagonal movement.
            if let Some(entry_shadow) = entry_shadow {
                publish_oam_shadow(&mut self.ppu.oam, entry_shadow);
            }
        } else if room_82_horizontal_quadrant_filter_entry_publishes_host_boundary {
            // The quadrant build completes its OAM work at the host boundary
            // before state 6 starts filtering. The generic published shadow
            // still describes Link's preceding horizontal position.
            if let Some(entry_shadow) = entry_shadow {
                publish_oam_shadow(&mut self.ppu.oam, entry_shadow);
            }
        } else if room_82_horizontal_deferred_nmi_publishes_entry_shadow {
            // Direction $02 reaches the deferred NMI with the complete sorted
            // table staged at host entry.
            if let Some(entry_shadow) = entry_shadow {
                publish_oam_shadow(&mut self.ppu.oam, entry_shadow);
            }
        } else if room_82_horizontal_state3_followup {
            // The first follow-up still has Link active in the entry shadow.
            // Once main offscreens that shadow, hardware retains the preceding
            // sorted table instead of publishing the retired copy.
            let entry_shadow_keeps_link = entry_shadow
                .and_then(|shadow| shadow.get(102 * 4 + 1))
                .is_some_and(|&y| y != 0xf0);
            if entry_shadow_keeps_link {
                publish_oam_shadow(&mut self.ppu.oam, entry_shadow.unwrap());
            } else if let Some(last_presented) = self
                .last_presented_oam
                .as_deref()
                .filter(|oam| oam.len() == self.ppu.oam.len())
            {
                self.ppu.oam.clone_from_slice(last_presented);
            }
        } else if retain_room_82_horizontal_sprite_conversion_return_oam {
            // The caller-resume state has already retired Link from every
            // software shadow, while the PPU continues scanning the sorted
            // table published by the deferred conversion workload.
            if let Some(last_presented) = self
                .last_presented_oam
                .as_deref()
                .filter(|oam| oam.len() == self.ppu.oam.len())
            {
                self.ppu.oam.clone_from_slice(last_presented);
            }
        }
    }

    /// Extracted from `compose_display_oam` (mechanical move; body unchanged).
    pub(super) fn compose_oam_debug_display_obj_vram_frame(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
        following_frame: FrameState,
        following_room: u8,
    ) {
        if crate::debug_env::var("ZELDA3_DEBUG_DISPLAY_OBJ_VRAM_FRAME")
            .ok()
            .and_then(|frame| frame.parse::<u32>().ok())
            .is_some_and(|frame| frame == self.frame_ctr_dbg)
        {
            const LINK_TILE_02_WORDS: std::ops::Range<usize> = 0x4020..0x4030;
            const LINK_TILE_03_WORDS: std::ops::Range<usize> = 0x4030..0x4040;
            const TILE_20_WORDS: std::ops::Range<usize> = 0x4200..0x4210;
            let selected = self.ppu.obj_vram_latch.as_deref().unwrap_or(&self.ppu.vram);
            let last_presented = self
                .last_presented_obj_vram
                .as_deref()
                .map(|vram| &vram[TILE_20_WORDS.clone()]);
            let previous_frame = self
                .ppu
                .obj_previous_frame_vram
                .as_deref()
                .map(|vram| &vram[TILE_20_WORDS.clone()]);
            eprintln!(
                "display_obj_vram_link_tile host={} tile=02 visible={:04x?} following={:04x?} last_presented={:04x?} previous_frame={:04x?} selected={:04x?} source={:?} following_source={:?} last_source={:?}",
                self.frame_ctr_dbg,
                &self.ppu.vram[LINK_TILE_02_WORDS.clone()],
                &following.ppu.vram[LINK_TILE_02_WORDS.clone()],
                self.last_presented_obj_vram
                    .as_deref()
                    .map(|vram| &vram[LINK_TILE_02_WORDS.clone()]),
                self.ppu
                    .obj_previous_frame_vram
                    .as_deref()
                    .map(|vram| &vram[LINK_TILE_02_WORDS.clone()]),
                &selected[LINK_TILE_02_WORDS],
                self.vram_chr_source.get(0x402),
                following.vram_chr_source.get(0x402),
                self.last_presented_vram_chr_source
                    .as_ref()
                    .map(|source| source.get(0x402)),
            );
            eprintln!(
                "display_obj_vram_link_tile host={} tile=03 visible={:04x?} following={:04x?} last_presented={:04x?} previous_frame={:04x?} selected={:04x?}",
                self.frame_ctr_dbg,
                &self.ppu.vram[LINK_TILE_03_WORDS.clone()],
                &following.ppu.vram[LINK_TILE_03_WORDS.clone()],
                self.last_presented_obj_vram
                    .as_deref()
                    .map(|vram| &vram[LINK_TILE_03_WORDS.clone()]),
                self.ppu
                    .obj_previous_frame_vram
                    .as_deref()
                    .map(|vram| &vram[LINK_TILE_03_WORDS.clone()]),
                &selected[LINK_TILE_03_WORDS],
            );
            eprintln!(
                "display_obj_vram_frame host={} room={:04x} phase={:02x}/{:02x}/{:02x} frame_counter={:02x} y_step={:02x} sources={:?} source={:?} retain={} link={:?}/{:?} latch={} visible={:04x?} following={:04x?} last_presented={:04x?} previous_frame={:04x?} selected={:04x?}",
                self.frame_ctr_dbg,
                following_room,
                following_frame.main_module,
                following_frame.submodule,
                following_frame.subsubmodule,
                following_frame.frame_counter,
                FollowerLinkState::load_from_ram(&following.ram).y_button_action_step(),
                LinkDmaSources::load_from_ram(&following.ram),
                plan.oam_scanout_source,
                plan.retain_captured_oam,
                plan.link_obj_scanout_generation,
                plan.link_obj_source_generation,
                self.ppu.obj_vram_latch.is_some(),
                &self.ppu.vram[TILE_20_WORDS.clone()],
                &following.ppu.vram[TILE_20_WORDS.clone()],
                last_presented,
                previous_frame,
                &selected[TILE_20_WORDS],
            );
        }
    }

    /// Extracted from `compose_display_oam` (mechanical move; body unchanged).
    pub(super) fn compose_oam_capture_publication_candidates(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
        candidate_captured_oam: Option<Vec<u16>>,
        candidate_captured_vram: Option<Vec<u16>>,
        capture_publication_candidates: bool,
        entry_frame: FrameState,
        following_frame: FrameState,
        following_room: u8,
    ) {
        if capture_publication_candidates {
            let obj_page = |vram: &[u16]| vram.get(0x4000..0x4400).map(|page| page.to_vec());
            let mut candidates = vec![DebugDisplayPublicationCandidate {
                name: "selected",
                oam: Some(self.ppu.oam.clone()),
                obj_vram: obj_page(self.ppu.obj_vram_latch.as_deref().unwrap_or(&self.ppu.vram)),
            }];
            if let (Some(oam), Some(vram)) = (
                candidate_captured_oam.as_ref(),
                candidate_captured_vram.as_ref(),
            ) {
                candidates.push(DebugDisplayPublicationCandidate {
                    name: "captured_before_nmi",
                    oam: Some(oam.clone()),
                    obj_vram: obj_page(vram),
                });
            }
            candidates.push(DebugDisplayPublicationCandidate {
                name: "live_after_main",
                oam: Some(following.ppu.oam.clone()),
                obj_vram: obj_page(&following.ppu.vram),
            });
            if let Some(graphics) = self.pre_main_graphics_dma.as_ref() {
                let mut host_oam = vec![0; self.ppu.oam.len()];
                let host_oam =
                    publish_oam_shadow(&mut host_oam, &graphics.oam_shadow).then_some(host_oam);
                candidates.push(DebugDisplayPublicationCandidate {
                    name: "host_boundary_before_main",
                    oam: host_oam.clone(),
                    obj_vram: obj_page(&graphics.obj_vram),
                });
                if let Some(captured_vram) = candidate_captured_vram.as_deref() {
                    candidates.push(DebugDisplayPublicationCandidate {
                        name: "early_link_dma_from_host_operands",
                        oam: host_oam,
                        obj_vram: obj_page(&compose_early_link_obj_cache(
                            captured_vram,
                            graphics.link_operands.sources,
                            self.asset_raw(57),
                        )),
                    });
                }
            }
            if let Some(published) = following.published_shadow_oam_dma.as_ref() {
                candidates.push(DebugDisplayPublicationCandidate {
                    name: "published_shadow_dma",
                    oam: Some(published.clone()),
                    obj_vram: None,
                });
            }
            if let Some(completed) = following.completed_oam_dma_after_capture.as_ref() {
                candidates.push(DebugDisplayPublicationCandidate {
                    name: "completed_oam_dma_after_capture",
                    oam: Some(completed.clone()),
                    obj_vram: None,
                });
            }
            if let Some(resident) = self.resident_oam_dma.as_ref() {
                candidates.push(DebugDisplayPublicationCandidate {
                    name: "resident_oam_dma",
                    oam: Some(resident.clone()),
                    obj_vram: None,
                });
            }
            if self.last_presented_oam.is_some() || self.last_presented_obj_vram.is_some() {
                candidates.push(DebugDisplayPublicationCandidate {
                    name: "last_presented",
                    oam: self.last_presented_oam.clone(),
                    obj_vram: self.last_presented_obj_vram.as_deref().and_then(obj_page),
                });
            }
            if let Some(captured_vram) = candidate_captured_vram.as_deref() {
                if let Some(snapshot_sources) = following
                    .effective_presented_dma
                    .as_ref()
                    .and_then(|receipt| receipt.completed_link_obj_dma)
                    .filter(|completed| {
                        completed.source_generation == GraphicsDmaGeneration::LiveAfterMain
                    })
                    .map(|completed| completed.sources)
                {
                    candidates.push(DebugDisplayPublicationCandidate {
                        name: "early_link_dma_from_completed_receipt",
                        oam: Some(following.ppu.oam.clone()),
                        obj_vram: obj_page(&compose_early_link_obj_cache(
                            captured_vram,
                            snapshot_sources,
                            self.asset_raw(57),
                        )),
                    });
                }
                candidates.push(DebugDisplayPublicationCandidate {
                    name: "early_link_dma_from_live_operands",
                    oam: Some(following.ppu.oam.clone()),
                    obj_vram: obj_page(&compose_early_link_obj_cache(
                        captured_vram,
                        LinkDmaSources::load_from_ram(&following.ram),
                        self.asset_raw(57),
                    )),
                });
            }
            let captured_to_host_oam_mismatches = candidate_captured_oam
                .as_deref()
                .zip(self.pre_main_graphics_dma.as_ref())
                .map(|(captured, graphics)| {
                    captured
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .zip(&graphics.oam_shadow)
                        .filter(|(captured, host)| captured != *host)
                        .count()
                })
                .unwrap_or(0);
            self.debug_display_publication_context = Some(DebugDisplayPublicationContext {
                render_host_frame: self.frame_ctr_dbg,
                publication_host_frame: following.publication_host_frame,
                entry_frame: [
                    entry_frame.main_module,
                    entry_frame.submodule,
                    entry_frame.subsubmodule,
                    entry_frame.frame_counter,
                ],
                following_frame: [
                    following_frame.main_module,
                    following_frame.submodule,
                    following_frame.subsubmodule,
                    following_frame.frame_counter,
                ],
                dungeon_room: following_room,
                staircase_index: crate::game_state::DungeonState::load_from_ram(&following.ram)
                    .stair_movement
                    .staircase_index(),
                palette_filter_countdown: read_le_u16(
                    &following.ram,
                    crate::game_state::constants::PALETTE_FILTER_COUNTDOWN,
                ),
                nmi_update_latch: following.ram[crate::game_state::constants::NMI_BOOLEAN],
                oam_scanout_source: format!("{:?}", plan.oam_scanout_source),
                retain_captured_oam: plan.retain_captured_oam,
                link_obj_scanout_generation: format!("{:?}", plan.link_obj_scanout_generation),
                link_obj_source_generation: format!("{:?}", plan.link_obj_source_generation),
                captured_to_host_oam_mismatches,
                oam_dma_completed_after_active_scanout: following
                    .closed_oam_boundary_receipt
                    .as_ref()
                    .is_some_and(|receipt| {
                        receipt.publication_host_frame == following.publication_host_frame
                    }),
            });
            self.debug_display_publication_candidates = candidates;
        } else {
            self.debug_display_publication_candidates.clear();
            self.debug_display_publication_context = None;
        }
    }
}
