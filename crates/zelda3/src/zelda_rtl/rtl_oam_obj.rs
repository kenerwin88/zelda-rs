//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (oam_obj).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    /// Stage the next frame's OBJ scanout generations, recording the caller's
    /// source location so the display probe (`ZELDA3_DEBUG_DISPLAY_OAM_FRAME` /
    /// `ZELDA3_DEBUG_DISPLAY_OBJ_VRAM_FRAME`) can name which handler owns the
    /// decision. Every writer of `next_display_obj_scanout_generation` should go
    /// through here rather than assigning the field directly.
    #[track_caller]
    pub(super) fn set_next_display_obj_scanout(&mut self, value: Option<ObjScanoutGenerations>) {
        if value.is_some() {
            self.next_display_obj_scanout_provenance = Some(core::panic::Location::caller());
        }
        self.next_display_obj_scanout_generation = value;
    }

    /// Record the OAM consequence of an NMI that interrupted Sprite_Main.
    /// Interrupt_NMI either retains hardware OAM when its software latch is
    /// set, or NMI_DoUpdates copies the exact live $0800..$0a1f shadow.
    pub(super) fn stage_interrupted_sprite_main_oam_scanout(&mut self) {
        if !self.rom_startup_timing() {
            return;
        }
        if self.ram[crate::game_state::constants::NMI_BOOLEAN] != 0 {
            // Interrupt_NMI skips NMI_DoUpdates while the software latch is
            // already set. No $2104 DMA occurs, so the resident hardware OAM
            // table survives this vblank verbatim. Display capture versions
            // that table through resident_oam_dma, which advances only on a
            // real $2104 transfer.
            self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
                oam: OamScanoutSource::RetainResidentPpuOam,
                link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            }));
            return;
        }
        let oam = self
            .sprite_oam_shadow_buffer()
            .chunks_exact(2)
            .take(self.ppu.oam.len())
            .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
            .collect();
        self.next_display_obj_memory_generation =
            Some(DisplayObjGeneration::RetainCapturedOam { oam });
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeInterruptedSpriteMainShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
    }

    pub(super) fn stage_straight_interroom_fadeout_obj_source(&mut self) {
        if self.next_display_obj_scanout_generation.is_some() {
            // A resumed caller already staged the independently measured OAM
            // and Link-DMA generations for this capture.
            return;
        }
        let mut scanout = straight_interroom_fadeout_obj_scanout();
        // Link's raw VRAM scanout can be live while its semantic source still
        // belongs to the host-boundary operands consumed by the NMI. Preserve
        // the ordinary atomic fadeout snapshot explicit.
        scanout.link_obj_sources = GraphicsDmaGeneration::HostBoundaryBeforeMain;
        self.set_next_display_obj_scanout(Some(scanout));
    }

    /// Env-gated tracer for the presented Link OBJ CHR pipeline: under
    /// `ZELDA3_DEBUG_OBJ_PIPE=<frames>` each stage (NMI bracket, snapshot
    /// capture/publication, staging, promotion) logs a fingerprint of the OBJ
    /// page ($4000-$43ff) so the snapshot generation feeding a scanout can be
    /// followed end to end. `page` is the OBJ page slice itself.
    pub(super) fn debug_obj_pipe_enabled(&self) -> bool {
        nmi::debug_frame_selection_env_matches("ZELDA3_DEBUG_OBJ_PIPE", self.frame_ctr_dbg)
    }

    pub(crate) fn debug_obj_pipe(&self, stage: &str, page: &[u16]) {
        if !self.debug_obj_pipe_enabled() {
            return;
        }
        let sum: u64 = page.iter().map(|&word| u64::from(word)).sum();
        let sample = |offset: usize| page.get(offset).copied().unwrap_or(0);
        eprintln!(
            "objpipe host={} stage={stage} pack={:04x} sum={sum:08x} w4020={:04x} w4030={:04x} w40b0={:04x} w4120={:04x}",
            self.frame_ctr_dbg,
            read_le_u16(&self.ram, LINK_DMA_GRAPHICS_INDEX),
            sample(0x20),
            sample(0x30),
            sample(0xb0),
            sample(0x120),
        );
    }

    pub(super) fn link_obj_dma_phase_generations(&self) -> LinkObjDmaPhaseGenerations {
        let exit_frame = self.game_state.frame;
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or(exit_frame);
        let operands = link_obj_operands_across_main(
            entry_frame,
            exit_frame,
            rom_graphics_dma_plan(exit_frame.main_module, exit_frame.submodule).link_obj_operands,
        );
        let mut generations = link_obj_dma_generations_for_cpu_phase(
            self.game_execution_scheduler
                .resumed_call_stack_is_before_nmi(),
            operands,
        );
        if self
            .item_receipt_completion_live_link_dma_host
            .is_some_and(|host| host.wrapping_add(1) == self.frame_ctr_dbg)
        {
            // First NMI after a ground A-press receipt completion. C's chain:
            // LinkOam_Main assigns link_dma_graphics_index from the receipt's
            // new pose, and the prep (misc.c:399) turns it into
            // dma_source_addr_3/_4 — so the sources this NMI consumes were
            // authored by the iteration that just ran, not by the stale host
            // boundary captured before the receipt (f4587 CHR). Upload lane
            // only: the upload is visible from the FOLLOWING scanout, so the
            // presented generation keeps its cadence (f10039).
            generations.following_nmi = GraphicsDmaGeneration::LiveAfterMain;
        }
        generations
    }

    /// Link OBJ generation which had actually completed DMA when the current
    /// scanout was published. A resumed caller may already have authored the
    /// following source, but it is not presented until its NMI completes.
    pub(super) fn presented_link_obj_dma_generation(&self) -> GraphicsDmaGeneration {
        self.link_obj_dma_phase_generations().presented
    }

    pub(crate) fn reset_intro_sprite_oam_cursor(&mut self) {
        self.intro_scene_bridge_mut().set_sprite_oam_cursor(0x0800);
    }

    pub(crate) fn allocate_intro_sprite_oam_entries(&mut self, entry_count: usize) -> usize {
        self.intro_scene_bridge_mut()
            .allocate_oam_entries(entry_count)
    }

    pub(crate) fn sprite_oam_shadow_buffer(&self) -> &[u8] {
        self.game_state.display.sprite_oam_shadow_buffer(&self.ram)
    }

    pub(crate) fn reset_weather_vane_oam_offset(&mut self) {
        self.weather_vane_bridge_mut().reset_oam_offset();
    }

    pub(crate) fn advance_weather_vane_oam_offset(&mut self, value: u8) {
        self.weather_vane_bridge_mut().advance_oam_offset(value);
    }

    pub(crate) fn cached_sprite_slot(&self, slot: usize) -> CachedSpriteRead {
        self.game_state.sprites.cached_sprites.slot(slot)
    }

    pub(crate) fn cached_sprite_slot_mut(
        &mut self,
        slot: usize,
    ) -> NativeCachedSpriteBridgeMut<'_> {
        NativeCachedSpriteBridgeMut::new(
            &mut self.game_state.sprites.cached_sprites,
            &mut self.game_state.sprites.sprite_slots,
            &mut self.game_state.sprites.system,
            &mut self.ram,
            slot,
        )
    }

    /// Select the sole owner of a cached-sprite interruption boundary.
    ///
    /// The legacy shadow CPU can still provide a coarse Sprite_Main boundary,
    /// but once the continuous timing authority is live it exclusively owns
    /// fine-grained `UncacheAndExecuteSprite` statement progress. This prevents
    /// the same source interruption from being armed first by a reconstructed
    /// raster envelope and then again by the host-call receipt.
    pub(super) fn take_authoritative_cached_sprite_interruption(
        &mut self,
        legacy_shadow: Option<CachedSpriteCpuInterruption>,
    ) -> Option<(CachedSpriteCpuInterruption, Option<OriginalTimingBoundary>)> {
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            self.take_original_timing_cached_sprite_execution_progress()
                .map(|receipt| (receipt.progress.into(), Some(receipt.boundary)))
        } else {
            legacy_shadow.map(|boundary| (boundary, None))
        }
    }

    pub(super) fn stage_big_key_drop_waiting_obj_scanout(&mut self) {
        // PPU registers continue to be rewritten by each NMI, so this hold is
        // deliberately scoped to the OBJ DMA domain instead of retaining the
        // entire display snapshot.
        if let Some(oam) = self.retiring_or_last_presented_oam().map(<[u16]>::to_vec) {
            self.next_display_obj_memory_generation =
                Some(DisplayObjGeneration::RetainCapturedOam { oam });
        }
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            // No OAM DMA runs while the decompressor owns the main stack.
            // Keep the effective resident table verbatim; selecting a shadow
            // source here would overlay operands for a transfer that did not
            // occur and defeat the retained-memory receipt above.
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
    }

    pub(super) fn stage_dungeon_supertile_quadrant_upload_obj_scanout(&mut self) {
        if !self.rom_startup_timing() {
            return;
        }
        // The snapshot captures `resident_oam_dma`, which advances only when
        // a real $2104 transfer completes. Do not also freeze `ppu.oam` here:
        // this CPU phase can run after a later DMA in the same host, and an
        // unversioned copy would overwrite the scanout generation selected by
        // the closed-boundary receipt.
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        }));
    }

    pub(super) fn stage_room_72_supertile_scroll_obj_scanout(&mut self) {
        if !self.rom_startup_timing() || self.game_state.world.location.dungeon_room_index() != 0x72
        {
            return;
        }
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposePublishedShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
    }

    pub(super) fn stage_room_72_supertile_landing_obj_scanout(&mut self) {
        if !self.rom_startup_timing() || self.game_state.world.location.dungeon_room_index() != 0x72
        {
            return;
        }
        self.set_next_display_obj_scanout(Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmi,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        }));
    }

    pub(super) fn stage_atomic_item_graphics_return_obj_scanout(
        &mut self,
        continuation: ItemReceiptGraphicsContinuation,
    ) {
        let scanout = atomic_item_graphics_return_obj_scanout(continuation);
        let retained_display_memory = !matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x22, .. }
        );
        let live_animated_bg = !retained_display_memory
            || matches!(
                continuation,
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x24, .. }
            );
        if matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x22, .. }
        ) {
            self.enemy_drop_item_graphics_live_extended_oam_pending = true;
        }
        if (matches!(
            continuation,
            ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. }
        ) && scanout.link_obj == GraphicsDmaGeneration::HostBoundaryBeforeMain)
            || matches!(
                continuation,
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x24, .. }
            )
        {
            self.next_display_interrupted_item_receipt_obj_cache = true;
        }
        // RetainPublished deliberately keeps the already-visible snapshot
        // object. Annotate that retained generation with split Link pixel and
        // provenance ownership now, then carry the same split into the next
        // captured boundary as well.
        if let Some(snapshot) = self.display_snapshot.as_mut() {
            snapshot.enemy_drop_item_graphics_live_extended_oam = matches!(
                continuation,
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x22, .. }
            );
            snapshot.oam_scanout_source = match continuation {
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. } => {
                    OamScanoutSource::ComposePublishedShadowDma
                }
                ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. }
                | ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt { .. }
                | ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt { .. } => scanout.oam,
            };
            snapshot.vram_generation = if retained_display_memory {
                DisplayVramGeneration::RetainCapturedBeforeNmi
            } else {
                DisplayVramGeneration::ComposeLiveAfterNmi
            };
            snapshot.hud_vram_generation = if retained_display_memory {
                DisplayVramGeneration::RetainCapturedBeforeNmi
            } else {
                DisplayVramGeneration::ComposeLiveAfterNmi
            };
            snapshot.animated_bg_scanout_generation = if live_animated_bg {
                AnimatedBgScanoutGeneration::LiveAfterNmi
            } else {
                AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
            };
            snapshot.link_obj_scanout_generation = scanout.link_obj;
            snapshot.link_obj_source_generation = scanout.link_obj_sources;
        }
        if matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x24, .. }
        ) {
            // This receipt finishes after the current retained scanout has
            // already selected its OAM, but Main_PrepSpritesForNmi has prepared
            // the complete shadow consumed by the following vblank. Capture
            // that exact next OAM generation explicitly so an older retained
            // OBJ-memory generation cannot keep the packed size bits stale.
            let shadow = self.sprite_oam_shadow_buffer();
            let oam = shadow
                .chunks_exact(2)
                .take(self.ppu.oam.len())
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect();
            self.next_display_obj_memory_generation =
                Some(DisplayObjGeneration::RetainCapturedOam { oam });
        }
        self.publish_live_hud_vram_on_next_capture = true;
        self.next_display_vram_generation = if retained_display_memory {
            DisplayVramGeneration::RetainCapturedBeforeNmi
        } else {
            DisplayVramGeneration::ComposeLiveAfterNmi
        };
        self.next_display_animated_bg_scanout_generation = Some(if live_animated_bg {
            AnimatedBgScanoutGeneration::LiveAfterNmi
        } else {
            AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
        });
        self.set_next_display_obj_scanout(Some(scanout));
    }

    pub(super) fn compose_display_oam(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
    ) {
        let effective_obj_cache_generation = following.effective_obj_cache_generation();
        let last_published_oam = self.last_presented_oam.clone();
        let immutable_captured_oam = (plan.oam_scanout_source
            == OamScanoutSource::RetainImmutableCapturedPpu)
            .then(|| self.ppu.oam.clone());
        // The OBJ ledger promises a causal first-divergence receipt. Capture
        // competing publication generations automatically so that receipt is
        // self-contained instead of requiring a second replay with another
        // undocumented environment flag.
        let capture_publication_candidates =
            crate::debug_env::var_os("ZELDA3_CAPTURE_DISPLAY_CANDIDATES").is_some()
                || crate::debug_env::var_os("ZELDA3_CAPTURE_OBJ_STATE_LEDGER").is_some();
        let candidate_captured_oam = capture_publication_candidates.then(|| self.ppu.oam.clone());
        let candidate_captured_vram = capture_publication_candidates.then(|| self.ppu.vram.clone());
        let interrupted_obj_cache_base = matches!(
            effective_obj_cache_generation,
            DisplayObjCacheGeneration::CapturedBeforeNmi
                | DisplayObjCacheGeneration::EarlyLinkDmaFromHostOperands
        )
        .then(|| self.ppu.vram.clone());
        let live_extended_oam = following
            .enemy_drop_item_graphics_live_extended_oam
            .then(|| following.ppu.oam[256..].to_vec());
        let debug_oam_frame = nmi::debug_frame_selection_env_matches(
            "ZELDA3_DEBUG_DISPLAY_OAM_FRAME",
            self.frame_ctr_dbg,
        );
        let debug_captured_oam = debug_oam_frame.then(|| self.ppu.oam.clone());
        let debug_host_boundary_oam = debug_oam_frame
            .then(|| {
                self.pre_main_graphics_dma.as_ref().map(|graphics| {
                    graphics
                        .oam_shadow
                        .chunks_exact(2)
                        .map(|word| u16::from_le_bytes([word[0], word[1]]))
                        .collect::<Vec<_>>()
                })
            })
            .flatten();
        let debug_last_presented_oam = debug_oam_frame
            .then(|| self.last_presented_oam.clone())
            .flatten();
        let debug_following_shadow_oam = debug_oam_frame.then(|| {
            let byte_len = following.ppu.oam.len() * 2;
            following.ram[OAM_BUF..OAM_BUF + byte_len]
                .chunks_exact(2)
                .map(|word| u16::from_le_bytes([word[0], word[1]]))
                .collect::<Vec<_>>()
        });
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or_else(|| crate::game_state::FrameState::load_from_ram(&self.ram));
        let following_frame = crate::game_state::FrameState::load_from_ram(&following.ram);
        let following_room = crate::game_state::WorldLocationState::load_from_ram(&following.ram)
            .dungeon_room_index();
        let following_staircase_index =
            crate::game_state::DungeonState::load_from_ram(&following.ram)
                .stair_movement
                .staircase_index();
        let dungeon_gameplay_handoff_obj_cache_base =
            staircase_34_gameplay_handoff_decodes_early_host_link_obj_cache(
                entry_frame,
                following_frame,
                following_staircase_index,
                plan.link_obj_scanout_generation,
                plan.link_obj_source_generation,
            )
            .then(|| self.ppu.vram.clone());
        let room_82_staircase_30_gameplay_handoff_uses_live_obj_cache =
            room_82_staircase_30_gameplay_handoff_uses_live_obj_cache(
                entry_frame,
                following_frame,
                following_room,
                following_staircase_index,
                plan.link_obj_scanout_generation,
                plan.link_obj_source_generation,
            );
        let straight_interroom_post_sprite_graphics_obj_cache_base =
            straight_interroom_post_sprite_graphics_uses_host_link_obj_cache(
                following_frame,
                following_room,
                following_staircase_index,
            )
            .then(|| self.ppu.vram.clone());
        let resident_dungeon_landing_oam = (following_frame.main_module == 7
            && following_frame.submodule == 0x0f
            && following_frame.subsubmodule == 1)
            .then(|| self.ppu.oam.clone())
            .filter(|resident| resident[116 * 2].to_le_bytes()[1] != 0xf0);
        let debug_subtile_oam = crate::debug_env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some()
            && following_frame.main_module == 7
            && following_frame.submodule == 1
            && following_frame.subsubmodule == 6;
        let debug_entries = [12, 13, 92, 93, 102, 103, 107, 110, 111, 112, 113];
        let captured_entries = debug_subtile_oam
            .then(|| debug_entries.map(|entry| oam_entry_bytes(&self.ppu.oam, entry)));
        if !plan.retain_captured_oam {
            if let Some(resident_oam) = plan
                .oam_scanout_source
                .active_nmi_dma_is_presented()
                .then_some(self.resident_oam_dma.as_deref())
                .flatten()
                .filter(|oam| oam.len() == self.ppu.oam.len())
            {
                self.ppu.oam.clone_from_slice(resident_oam);
            } else {
                self.ppu.oam.clone_from(&following.ppu.oam);
            }
        }
        if let Some(oam) = following.obj_generation.retained_oam() {
            // Coarse full-memory retention supplies the baseline generation.
            // The more specific scanout-source provenance below may replace
            // or merge it, and an active-NMI event receipt is applied last.
            self.ppu.oam.clone_from_slice(oam);
        }
        let host_boundary_shadow_oam = (plan.oam_scanout_source
            == OamScanoutSource::ComposeHostBoundaryShadowDma)
            .then(|| {
                let graphics = self.pre_main_graphics_dma.as_ref()?;
                let mut oam = vec![0; self.ppu.oam.len()];
                publish_oam_shadow(&mut oam, &graphics.oam_shadow).then_some(oam)
            })
            .flatten();
        let published_shadow_oam = match plan.oam_scanout_source {
            OamScanoutSource::RetainCapturedBeforeNmi
            | OamScanoutSource::ComposePublishedShadowDma
            | OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink => {
                following.published_shadow_oam_dma.as_deref()
            }
            OamScanoutSource::ComposeHostBoundaryShadowDma => host_boundary_shadow_oam.as_deref(),
            OamScanoutSource::RetainPreviousPresented
            | OamScanoutSource::RetainImmutableCapturedPpu
            | OamScanoutSource::RetainResidentPpuOam
            | OamScanoutSource::ComposeObservedNmiReceipt
            | OamScanoutSource::ComposeLiveAfterNmi
            | OamScanoutSource::ComposeCompletedWorkAfterNmi
            | OamScanoutSource::ComposeLiveShadowAfterMain
            | OamScanoutSource::ComposeInterruptedSpriteMainShadowDma
            | OamScanoutSource::ComposeSpiralReturnPlayerShadowAfterMain
            | OamScanoutSource::ComposeLivePlayerOamAfterMain => None,
        };
        let main_iteration_completed = entry_frame.frame_counter != following_frame.frame_counter;
        let capture_waits_for_nmi = following.ram[crate::game_state::constants::NMI_BOOLEAN] != 0;
        let completed_dma_is_coherent = plan.oam_scanout_source
            == OamScanoutSource::RetainResidentPpuOam
            && capture_waits_for_nmi
            && following.link_obj_scanout_generation
                == GraphicsDmaGeneration::HostBoundaryBeforeMain;
        let active_oam_dma_defers_to_prior_provenance = following
            .effective_presented_dma
            .as_ref()
            .and_then(|receipt| receipt.completed_oam.as_ref())
            .is_some()
            && !following.oam_scanout_source.active_nmi_dma_is_presented();
        self.compose_oam_oam_scanout_source_retaincapturedbeforenmi(
            following,
            plan,
            active_oam_dma_defers_to_prior_provenance,
            completed_dma_is_coherent,
            entry_frame,
            following_frame,
            last_published_oam,
            main_iteration_completed,
            published_shadow_oam,
        );
        if plan.oam_scanout_source == OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink {
            // The rescue prompt interrupts Sprite_Main after unrelated sprites
            // have advanced but before Link's sorted body and equipment range
            // reaches hardware. Start from the live table and overlay only
            // those seven entries (including packed high bits) from the
            // pre-main host-boundary shadow.
            compose_host_boundary_link_oam(&mut self.ppu.oam, published_shadow_oam);
        }
        if plan.oam_scanout_source == OamScanoutSource::ComposeLivePlayerOamAfterMain {
            // LinkOam_Main owns entries 12..17 on this unsorted dungeon slice.
            // The remaining sprite table was already consumed from the entry
            // shadow and must not be advanced with Link's hold-item handoff.
            // Link's live coordinates were authored against the live camera,
            // while this scanout still owns the independently captured scroll
            // registers. Rebase the six player entries onto that display
            // generation instead of mixing camera generations.
            const PLAYER_OAM_WORDS: std::ops::Range<usize> = 24..36;
            let byte_start = OAM_BUF + PLAYER_OAM_WORDS.start * 2;
            let byte_end = OAM_BUF + PLAYER_OAM_WORDS.end * 2;
            if let Some(live_shadow) = following.ram.get(byte_start..byte_end) {
                for (word, bytes) in self.ppu.oam[PLAYER_OAM_WORDS]
                    .iter_mut()
                    .zip(live_shadow.chunks_exact(2))
                {
                    *word = u16::from_le_bytes([bytes[0], bytes[1]]);
                }
            }
            // The mixed OAM generation remains resident while the camera can
            // independently advance on later retained scanouts. Rebase against
            // the camera that authored this OAM DMA, not the scroll registers
            // that happen to be live for the current scanout.
            let (oam_camera_x, oam_camera_y) = following
                .dungeon_item_hold_entry_bg2_scroll
                .unwrap_or((self.ppu.bg_layer[1].h_scroll, self.ppu.bg_layer[1].v_scroll));
            let x_delta =
                read_le_u16(&following.ram, BG2_X_SCROLL).wrapping_sub(oam_camera_x) as u8;
            let y_delta =
                read_le_u16(&following.ram, BG2_Y_SCROLL).wrapping_sub(oam_camera_y) as u8;
            for entry in 12..18 {
                let [x, y] = self.ppu.oam[entry * 2].to_le_bytes();
                self.ppu.oam[entry * 2] =
                    u16::from_le_bytes([x.wrapping_add(x_delta), y.wrapping_add(y_delta)]);
            }
        }
        if plan.oam_scanout_source == OamScanoutSource::ComposeLiveShadowAfterMain {
            let byte_len = following.ppu.oam.len() * 2;
            for (word, bytes) in self
                .ppu
                .oam
                .iter_mut()
                .zip(following.ram[OAM_BUF..OAM_BUF + byte_len].chunks_exact(2))
            {
                *word = u16::from_le_bytes([bytes[0], bytes[1]]);
            }
        }
        if plan.oam_scanout_source == OamScanoutSource::ComposeCompletedWorkAfterNmi {
            if let Some(completed_oam) = following
                .completed_oam_dma_after_capture
                .as_deref()
                .filter(|oam| oam.len() == self.ppu.oam.len())
            {
                self.ppu.oam.clone_from_slice(completed_oam);
            }
        }
        if plan.oam_scanout_source == OamScanoutSource::ComposeSpiralReturnPlayerShadowAfterMain {
            // Link owns the contiguous sorted range observed at this return.
            // Copying the whole shadow would incorrectly advance unrelated
            // dungeon sprites that hardware has not DMAed yet.
            let player_oam = self
                .spiral_stair_return_player_oam_scanout
                .expect("spiral-return publication requires its captured player OAM");
            self.ppu.oam[102 * 2..112 * 2].copy_from_slice(&player_oam[..20]);
            for entry in 112..114 {
                // Sword/shield horizontal placement has already reached the
                // resident OAM generation; only the remaining bytes belong to
                // the completed player shadow at this split boundary.
                let source = (entry - 102) * 2;
                let [_, y] = player_oam[source].to_le_bytes();
                let [tile, attributes] = player_oam[source + 1].to_le_bytes();
                let x = self.ppu.oam[entry * 2].to_le_bytes()[0];
                self.ppu.oam[entry * 2] = u16::from_le_bytes([x, y]);
                self.ppu.oam[entry * 2 + 1] = u16::from_le_bytes([tile, attributes]);
            }
        }
        if main_iteration_completed
            && completed_dma_is_coherent
            && !active_oam_dma_defers_to_prior_provenance
        {
            // A completed OAM DMA is the final hardware event for this domain.
            // A retained scanout advances only when the ROM frame counter
            // proves that its main iteration completed. Resident-retain also
            // requires a pending captured NMI and Link OBJ provenance from the
            // same host boundary. A captured-before-NMI source never reaches
            // this fallback: its completed transfer is later than the event
            // named by the source. Once an explicit active-scanout receipt
            // exists, its OAM provenance wins: a later completed transfer must
            // not overwrite the shadow generation that actually retired for
            // this scanout.
            if let Some(completed_oam) = following
                .completed_oam_dma_after_capture
                .as_deref()
                .filter(|oam| oam.len() == self.ppu.oam.len())
            {
                self.ppu.oam.clone_from_slice(completed_oam);
            }
        }
        if plan.dungeon_state_13_phase == DungeonState13PublicationPhase::CallerReturn
            || plan.dungeon_faded_filter_phase == DungeonFadedFilterPublicationPhase::CallerReturn
        {
            // "Previously presented" is a render-boundary generation, not a
            // CPU-capture generation. Select it here, after all snapshot OAM
            // composition, so repeated frontend/diagnostic consumers agree.
            if let Some(oam) = self.last_presented_oam.as_ref() {
                self.ppu.oam.clone_from_slice(oam);
            }
        }
        if plan.publish_spiral_stair_return_equipment_handoff {
            // The return crosses the next main-loop prefix while sword/shield
            // graphics are changing. The completed OAM DMA owns the table,
            // except for the two equipment slots still retiring from Link's
            // return pass. Advance that retained equipment animation to the
            // scanout boundary without reviving the rest of the old player
            // generation.
            let returning_player_oam = self
                .spiral_stair_return_player_oam_scanout
                .expect("spiral-return publication requires its captured player OAM");
            for entry in 112..114 {
                let source = (entry - 102) * 2;
                let [x, y] = returning_player_oam[source].to_le_bytes();
                let [tile, mut attributes] = returning_player_oam[source + 1].to_le_bytes();
                if entry == 113 {
                    attributes ^= 0x40;
                }
                self.ppu.oam[entry * 2] = u16::from_le_bytes([x, y.wrapping_add(1)]);
                self.ppu.oam[entry * 2 + 1] = u16::from_le_bytes([tile, attributes]);
            }
        }
        if let Some(resident_oam) = resident_dungeon_landing_oam.as_deref() {
            // The landing main slice authors the following OAM shadow after
            // hardware has already published these entries. Keep the resident
            // Link-body size bit and four landing pieces from the completed
            // DMA generation, matching the v1.0.0 scanout boundary.
            compose_published_oam_entries(
                &mut self.ppu.oam,
                Some(resident_oam),
                [102, 116, 117, 118, 119],
            );
        }
        if let Some(vram) = following.obj_generation.retained_vram() {
            self.ppu.vram[0x4000..0x4400].copy_from_slice(vram);
        }
        let following_room = crate::game_state::WorldLocationState::load_from_ram(&following.ram)
            .dungeon_room_index();
        let room_71_item_graphics_return_live_boundary =
            room_71_item_graphics_return_crosses_completed_nmi(
                following_frame,
                following_room,
                plan.link_obj_scanout_generation,
                plan.oam_scanout_source,
            ) && self.ppu.oam[..256] == following.ppu.oam[..256];
        if room_71_item_graphics_return_live_boundary {
            // The final $10 item-graphics return crosses the packed OAM
            // size/X-bit DMA after the ordinary shadow generation was
            // selected. Snes9x's completed scanout confirms the hardware DMA
            // carried the late size/X rewrite; its only observable advance
            // over the published shadow is the 32-byte extended table, so the
            // low-table equality above is what distinguishes this completed
            // boundary from the preceding frames, where the live table is a
            // whole sprite generation ahead and the published shadow must
            // keep the scanout.
            self.ppu.oam[256..].copy_from_slice(&following.ppu.oam[256..]);
            // The same completed vblank consumed the queued BG1 stripe
            // upload (the receipt flash rewrites the painting tilemap), so
            // active scanout also owns the live BG1 tilemap image.
            let bg1_tilemap = usize::from(self.ppu.bg_layer[0].tilemap_adr);
            let bg1_tilemap_end = (bg1_tilemap + 0x1000).min(self.ppu.vram.len());
            self.ppu.vram[bg1_tilemap..bg1_tilemap_end]
                .copy_from_slice(&following.ppu.vram[bg1_tilemap..bg1_tilemap_end]);
        }
        if let Some(live_extended_oam) = live_extended_oam.as_deref() {
            // The enemy-drop $22 sheet returns through the NMI that publishes
            // the held-item size bit. Its retained low OAM table is already
            // correct, but the packed X/size table belongs to the completed
            // live DMA generation.
            self.ppu.oam[256..].copy_from_slice(live_extended_oam);
        }
        self.compose_oam_straight_interroom_fadeout_main_slice_publishes_host_oam(
            following,
            plan,
            entry_frame,
            following_frame,
            following_room,
        );
        if let Some(active_oam) = following
            .closed_oam_boundary_receipt
            .as_ref()
            .filter(|receipt| receipt.publication_host_frame == following.publication_host_frame)
            .filter(|_| !plan.oam_scanout_source.active_nmi_dma_is_presented())
            .map(|receipt| receipt.active_oam.as_slice())
        {
            // The live PPU and queued shadow now describe the next scanout.
            // Re-establish the exact resident generation captured when the
            // active boundary closed. An explicit active-NMI source has
            // already selected its completed transfer above and must not be
            // replaced by this older fallback generation.
            if active_oam.len() == self.ppu.oam.len() {
                self.ppu.oam.clone_from_slice(active_oam);
            }
        }
        if let Some(captured_entries) = captured_entries {
            let published = following.published_shadow_oam_dma.as_deref();
            eprintln!(
                "display_subtile_oam host={} phase={:02x}/{:02x}/{:02x} source={:?} link={:?}/{:?} retain={} captured={:02x?} following={:02x?} published={:02x?} composed={:02x?}",
                self.frame_ctr_dbg,
                following_frame.main_module,
                following_frame.submodule,
                following_frame.subsubmodule,
                plan.oam_scanout_source,
                plan.link_obj_scanout_generation,
                plan.link_obj_source_generation,
                plan.retain_captured_oam,
                captured_entries,
                debug_entries.map(|entry| oam_entry_bytes(&following.ppu.oam, entry)),
                published.map(|oam| debug_entries.map(|entry| oam_entry_bytes(oam, entry))),
                debug_entries.map(|entry| oam_entry_bytes(&self.ppu.oam, entry)),
            );
        }
        self.compose_oam_debug_captured_oam_as_deref(
            following,
            plan,
            debug_captured_oam,
            debug_following_shadow_oam,
            debug_host_boundary_oam,
            debug_last_presented_oam,
            following_frame,
            following_room,
        );
        if crate::debug_env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some()
            && following_room == 0x82
            && following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 3
        {
            const ROOM_82_TRANSITION_ENTRIES: [usize; 16] = [
                58, 59, 63, 66, 67, 68, 69, 102, 103, 104, 105, 107, 110, 111, 112, 113,
            ];
            let composed_entries =
                ROOM_82_TRANSITION_ENTRIES.map(|entry| oam_entry_bytes(&self.ppu.oam, entry));
            let following_entries =
                ROOM_82_TRANSITION_ENTRIES.map(|entry| oam_entry_bytes(&following.ppu.oam, entry));
            eprintln!(
                "display_room82_oam host={} entry={:02x}/{:02x}/{:02x} transition={} deferred={} source={:?} retain={} entries={ROOM_82_TRANSITION_ENTRIES:?} composed={composed_entries:02x?} following={following_entries:02x?}",
                self.frame_ctr_dbg,
                entry_frame.main_module,
                entry_frame.submodule,
                entry_frame.subsubmodule,
                self.screen_transition(),
                following.room_82_sprite_conversion_deferred_nmi,
                plan.oam_scanout_source,
                plan.retain_captured_oam,
            );
        }
        self.set_obj_vram_latch_traced(None);
        if following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 3
            && plan.link_obj_scanout_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain
        {
            // The state-3 entry scanout evaluates Link before the next pose's
            // upload becomes resident. Resolve the host-boundary DMA operands
            // after the transition OBJ bundle has been composed.
            if matches!(following_room, 0x70 | 0x71) {
                let live_sources = LinkDmaSources::load_from_ram(&following.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                for (destination, source) in [
                    (0x4020, LinkDmaSourceSlot::HeadTop),
                    (0x4120, LinkDmaSourceSlot::HeadBottom),
                ] {
                    let len = 0x40;
                    let source_address =
                        usize::from(live_sources.source(source).wrapping_sub(0x40));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                        (source_address >= 0x8000 && source_offset + len <= graphics.len())
                            .then_some(&graphics[source_offset..source_offset + len])
                    }) else {
                        if let Some(previous_obj_vram) = self.ppu.obj_previous_frame_vram.as_ref() {
                            self.ppu.vram[destination..destination_end]
                                .copy_from_slice(&previous_obj_vram[destination..destination_end]);
                        }
                        continue;
                    };
                    for (index, bytes) in source_bytes.chunks_exact(2).enumerate() {
                        self.ppu.vram[destination + index] =
                            u16::from_le_bytes([bytes[0], bytes[1]]);
                    }
                }
            } else if let Some(previous_obj_vram) = self.last_presented_obj_vram.as_deref() {
                self.ppu.vram[0x4000..0x4050].copy_from_slice(&previous_obj_vram[0x4000..0x4050]);
                self.ppu.vram[0x4100..0x4150].copy_from_slice(&previous_obj_vram[0x4100..0x4150]);
            }
        }
        let room_82_horizontal_deferred_nmi_publishes_entry_shadow =
            room_82_horizontal_deferred_nmi_publishes_entry_shadow_oam(
                entry_frame,
                following_frame,
                following_room,
                self.screen_transition(),
                following.room_82_sprite_conversion_deferred_nmi,
                plan.oam_scanout_source,
            );
        let room_82_horizontal_state3_followup =
            room_82_horizontal_state3_followup_publishes_entry_shadow_oam(
                entry_frame,
                following_frame,
                following_room,
                self.screen_transition(),
                following.room_82_sprite_conversion_deferred_nmi,
                plan.oam_scanout_source,
            );
        let retain_room_82_horizontal_sprite_conversion_return_oam =
            room_82_horizontal_sprite_conversion_return_retains_last_oam(
                entry_frame,
                following_frame,
                following_room,
                self.screen_transition(),
                plan.oam_scanout_source,
            );
        let room_82_horizontal_quadrant_filter_entry_publishes_host_boundary =
            room_82_horizontal_quadrant_filter_entry_publishes_host_boundary_oam(
                entry_frame,
                following_frame,
                following_room,
                self.screen_transition(),
                plan.oam_scanout_source,
            );
        let room_82_horizontal_first_scroll_publishes_host_boundary =
            room_82_horizontal_first_scroll_publishes_host_boundary_oam(
                entry_frame,
                following_frame,
                following_room,
                self.screen_transition(),
                plan.oam_scanout_source,
                plan.retain_captured_oam,
            );
        let entry_shadow = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.oam_shadow.as_slice());
        if debug_oam_frame && following_room == 0x82 {
            eprintln!(
                "display_room82_selection host={} entry={:02x}/{:02x}/{:02x} following={:02x}/{:02x}/{:02x} transition={} source={:?} retain={} deferred={} state3={} conversion_return={} filter_entry={} first_scroll={} host_shadow={}",
                self.frame_ctr_dbg,
                entry_frame.main_module,
                entry_frame.submodule,
                entry_frame.subsubmodule,
                following_frame.main_module,
                following_frame.submodule,
                following_frame.subsubmodule,
                self.screen_transition(),
                plan.oam_scanout_source,
                plan.retain_captured_oam,
                room_82_horizontal_deferred_nmi_publishes_entry_shadow,
                room_82_horizontal_state3_followup,
                retain_room_82_horizontal_sprite_conversion_return_oam,
                room_82_horizontal_quadrant_filter_entry_publishes_host_boundary,
                room_82_horizontal_first_scroll_publishes_host_boundary,
                entry_shadow.is_some(),
            );
        }
        // The lane takes `&mut self`; hand it a copy so the DMA-operand borrow ends here.
        let entry_shadow_owned = entry_shadow.map(<[u8]>::to_vec);
        self.compose_oam_room_82_horizontal_first_scroll_publishes_host_boundary(
            entry_shadow_owned.as_deref(),
            retain_room_82_horizontal_sprite_conversion_return_oam,
            room_82_horizontal_deferred_nmi_publishes_entry_shadow,
            room_82_horizontal_first_scroll_publishes_host_boundary,
            room_82_horizontal_quadrant_filter_entry_publishes_host_boundary,
            room_82_horizontal_state3_followup,
        );
        if following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 2
            && plan.link_obj_scanout_generation == GraphicsDmaGeneration::LiveAfterMain
        {
            // State 2 suppresses the ordinary Link DMA, so the early body,
            // head, and hand ranges remain on the image completed in state 1.
            // The live source words already describe the following upload and
            // would advance these ranges one scanout too early.
            if let Some(completed_obj_vram) = self
                .last_presented_obj_vram
                .as_deref()
                .map(|vram| &vram[0x4000..0x4400])
                .or_else(|| {
                    self.pre_main_graphics_dma
                        .as_ref()
                        .map(|graphics| &graphics.obj_vram[0x4000..0x4400])
                })
            {
                self.ppu.vram[0x4000..0x4050].copy_from_slice(&completed_obj_vram[0x000..0x050]);
                self.ppu.vram[0x4100..0x4150].copy_from_slice(&completed_obj_vram[0x100..0x150]);
            }

            if self.screen_transition() == 1 {
                // This transition direction retains the raw Link image while
                // Snes9x's decoded cache has already consumed the current head
                // operands. Direction 2 instead keeps the prior decoded head
                // generation alongside the retained raw image.
                let live_sources = LinkDmaSources::load_from_ram(&following.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                let mut obj_cache_vram = self.ppu.vram.clone();
                for (destination, source) in [
                    (0x4020, LinkDmaSourceSlot::HeadTop),
                    (0x4120, LinkDmaSourceSlot::HeadBottom),
                ] {
                    let len = 0x40;
                    let source_address = usize::from(live_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                        (source_address >= 0x8000 && source_offset + len <= graphics.len())
                            .then_some(&graphics[source_offset..source_offset + len])
                    }) else {
                        continue;
                    };
                    for (word, bytes) in obj_cache_vram[destination..destination_end]
                        .iter_mut()
                        .zip(source_bytes.chunks_exact(2))
                    {
                        *word = u16::from_le_bytes([bytes[0], bytes[1]]);
                    }
                }
                self.set_obj_vram_latch_traced(Some(obj_cache_vram));
            }
        }
        let dungeon_supertile_first_scroll_uses_live_obj_cache = following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 1
            && plan.oam_scanout_source == OamScanoutSource::RetainResidentPpuOam;
        let room_72_state8_scroll_after_first_tick_uses_live_obj_cache = following_room == 0x72
            && following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 8
            && following_frame.frame_counter != 0
            && self.screen_transition() == 1
            && plan.oam_scanout_source == OamScanoutSource::ComposePublishedShadowDma
            && plan.link_obj_scanout_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain
            && plan.link_obj_source_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain;
        let dungeon_subtile_shutter_handoff_uses_live_obj_cache = following_room == 0x72
            && following_frame.main_module == 7
            && following_frame.submodule == 5
            && plan.link_obj_scanout_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain
            && plan.link_obj_source_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain;
        let room_72_northward_subtile_shutter_retains_presented_obj_cache =
            dungeon_subtile_shutter_handoff_uses_live_obj_cache
                && room_72_northward_subtile_shutter_retains_presented_obj_cache(
                    following_frame,
                    following_room,
                    FollowerLinkState::load_from_ram(&following.ram).last_direction(),
                );
        let straight_interroom_fadeout_live_obj_cache =
            straight_interroom_fadeout_uses_live_decoded_obj_cache(
                following_frame,
                following_room,
                crate::game_state::DungeonState::load_from_ram(&following.ram)
                    .stair_movement
                    .staircase_index(),
            );
        let straight_interroom_post_sprite_graphics_live_obj_cache =
            straight_interroom_post_sprite_graphics_uses_host_link_obj_cache(
                following_frame,
                following_room,
                crate::game_state::DungeonState::load_from_ram(&following.ram)
                    .stair_movement
                    .staircase_index(),
            );
        let dungeon_dialogue_render_entry_host_link_obj_cache =
            dungeon_dialogue_render_entry_uses_host_link_obj_cache(entry_frame, following_frame);
        if plan.link_obj_source_generation == GraphicsDmaGeneration::LiveAfterMain
            && !dungeon_supertile_first_scroll_uses_live_obj_cache
            && (self.atomic_item_graphics_uses_partial_receipt_obj_cache()
                || following.interrupted_item_receipt_obj_cache)
        {
            // Only typed interrupted returns own this decoded-cache split. A
            // coarse host/live Link generation elsewhere must not inherit a
            // previous decoded head generation.
            let mut obj_cache_vram = self.ppu.vram.clone();
            let captured_sources = LinkDmaSources::load_from_ram(&self.ram);
            let link_graphics = self.asset_raw(57).map(Vec::from);
            for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                let source_address = usize::from(captured_sources.source(source));
                let source_offset = source_address.saturating_sub(0x8000);
                let destination_end = destination + len / 2;
                let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                    (source_address >= 0x8000 && source_offset + len <= graphics.len())
                        .then_some(&graphics[source_offset..source_offset + len])
                }) else {
                    obj_cache_vram[destination..destination_end]
                        .copy_from_slice(&following.ppu.vram[destination..destination_end]);
                    continue;
                };
                for (index, bytes) in source_bytes.chunks_exact(2).enumerate() {
                    obj_cache_vram[destination + index] = u16::from_le_bytes([bytes[0], bytes[1]]);
                }
            }
            if following.interrupted_item_receipt_obj_cache
                && plan.oam_scanout_source == OamScanoutSource::ComposeLiveAfterNmi
            {
                // The uncle-passage return completes the WRAM-backed shield
                // upload before scanout, then a later held NMI restores the
                // resident raw VRAM words. Snes9x therefore retains these four
                // newly decoded cache tiles even though its visible raw VRAM
                // still matches the preceding boundary.
                for (destination, source) in [
                    (0x4070, LinkDmaSourceSlot::ShieldUpper),
                    (0x4170, LinkDmaSourceSlot::ShieldLower),
                ] {
                    const LEN: usize = 0x40;
                    let source_address = usize::from(captured_sources.source(source));
                    let destination_end = destination + LEN / 2;
                    if let Some(source_bytes) = self.ram.get(source_address..source_address + LEN) {
                        for (word, bytes) in obj_cache_vram[destination..destination_end]
                            .iter_mut()
                            .zip(source_bytes.chunks_exact(2))
                        {
                            *word = u16::from_le_bytes([bytes[0], bytes[1]]);
                        }
                    }
                }
            }
            for range in [0x4240..0x4250, 0x4340..0x4350] {
                obj_cache_vram[range.clone()].copy_from_slice(&following.ppu.vram[range]);
            }
            // The two right-hand quadrants of the 16x16 receipt sprite were
            // not decoded by the interrupted transfer. Snes9x leaves those
            // cache entries transparent instead of exposing their resident
            // VRAM contents.
            obj_cache_vram[0x4250..0x4260].fill(0);
            obj_cache_vram[0x4350..0x4360].fill(0);
            self.set_obj_vram_latch_traced(Some(obj_cache_vram));
        } else {
            let dungeon_subtile_palette_filter_scanout = following_frame.main_module == 7
                && following_frame.submodule == 1
                && matches!(following_frame.subsubmodule, 6 | 7)
                && plan.oam_scanout_source == OamScanoutSource::RetainResidentPpuOam;
            let dungeon_subtile_palette_filter_uses_live_obj_cache =
                dungeon_subtile_palette_filter_scanout
                    && plan.link_obj_scanout_generation == GraphicsDmaGeneration::LiveAfterMain
                    && plan.link_obj_source_generation == GraphicsDmaGeneration::LiveAfterMain;
            let room_72_northward_palette_tail_uses_live_obj_cache =
                dungeon_subtile_palette_filter_uses_live_obj_cache
                    && room_72_northward_subtile_palette_tail_uses_live_obj_cache(
                        following_frame,
                        following_room,
                        FollowerLinkState::load_from_ram(&following.ram).last_direction(),
                    );
            let dungeon_subtile_palette_filter_holds_decoded_obj_cache =
                dungeon_subtile_palette_filter_scanout
                    && plan.link_obj_scanout_generation
                        == GraphicsDmaGeneration::HostBoundaryBeforeMain
                    && plan.link_obj_source_generation
                        == GraphicsDmaGeneration::HostBoundaryBeforeMain;
            let interrupted_dungeon_faded_filter_first_palette_pass =
                interrupted_dungeon_faded_filter_uses_host_link_obj_cache(
                    following_frame,
                    self.game_execution_scheduler.pre_main_caller_continuation(),
                ) || plan.dungeon_faded_filter_phase
                    == DungeonFadedFilterPublicationPhase::LandingPaletteCompletion;
            let blue_guard_workload_scanline = self
                .last_sprite_main_timing_workload
                .and_then(SpriteMainTimingWorkload::dungeon_map_backup_force_blank_output_scanline);
            let room_71_live_obj_cache = room_71_room_load_uses_live_obj_cache(
                entry_frame,
                following_frame,
                following_room,
                plan.link_obj_scanout_generation,
                plan.link_obj_source_generation,
                plan.oam_scanout_source,
                blue_guard_workload_scanline == Some(35),
            );
            let room_71_live_link_head_obj_cache =
                room_71_subtile_shutter_publishes_live_link_head_obj_cache(
                    entry_frame,
                    following_frame,
                    following_room,
                    read_le_u16(&following.ram, LINK_DMA_COUNTDOWN),
                );
            let room_71_obj_cache_publication = Room71ObjCachePublication::resolve(
                room_71_live_obj_cache,
                room_71_live_link_head_obj_cache,
            );
            if crate::debug_env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some()
                && following_room == 0x71
                && following_frame.main_module == 7
                && following_frame.submodule == 5
            {
                eprintln!(
                    "display_room71_obj_cache host={} entry={:02x}/{:02x}/{:02x} guard_scanline={blue_guard_workload_scanline:?} blank_from={:?} link={:?}/{:?} oam={:?} dma_source={:04x} dma_countdown={:04x} publication={room_71_obj_cache_publication:?}",
                    self.frame_ctr_dbg,
                    entry_frame.main_module,
                    entry_frame.submodule,
                    entry_frame.subsubmodule,
                    self.active_display_force_blank_event,
                    plan.link_obj_scanout_generation,
                    plan.link_obj_source_generation,
                    plan.oam_scanout_source,
                    read_le_u16(&following.ram, LINK_DMA_SOURCE_OFFSET),
                    read_le_u16(&following.ram, LINK_DMA_COUNTDOWN),
                );
            }
            if room_82_staircase_30_gameplay_handoff_uses_live_obj_cache {
                // Room $82's staircase-$30 handoff crosses a completed live OBJ
                // decode at the same host boundary where raw scanout retains its
                // entry generation. Keep the renderer cache on the post-main
                // generation without broadening the staircase-$34 early-DMA rule.
                self.set_obj_vram_latch_traced(Some(following.ppu.vram.clone()));
            } else if let Some(obj_cache_base) = dungeon_gameplay_handoff_obj_cache_base.as_deref()
            {
                // Gameplay's leading NMI has already decoded the early Link
                // batch from the host-boundary operands when main enters a
                // staircase-$34 transition submodule. Raw OBJ VRAM remains on its
                // independently retained generation for this scanout; this
                // is measured for both room-$72 spiral and room-$81 supertile
                // handoffs.
                let captured_sources = self
                    .pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.link_operands.sources)
                    .unwrap_or_else(|| LinkDmaSources::load_from_ram(&self.ram));
                self.set_obj_vram_latch_traced(Some(compose_early_link_obj_cache(
                    obj_cache_base,
                    captured_sources,
                    self.asset_raw(57),
                )));
            } else if straight_interroom_fadeout_live_obj_cache
                || straight_interroom_post_sprite_graphics_live_obj_cache
                || dungeon_dialogue_render_entry_host_link_obj_cache
            {
                // This scanout retains its raw OBJ page, but the NMI at the
                // host boundary has already invalidated and decoded the early
                // Link batch. Reconstruct that cache-only generation from the
                // operands captured for the completed DMA; following raw VRAM
                // intentionally still names the retained page here.
                let captured_sources = self
                    .pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.link_operands.sources)
                    .unwrap_or_else(|| LinkDmaSources::load_from_ram(&self.ram));
                let obj_cache_base = straight_interroom_post_sprite_graphics_obj_cache_base
                    .as_deref()
                    .unwrap_or(&self.ppu.vram);
                self.set_obj_vram_latch_traced(Some(compose_early_link_obj_cache(
                    obj_cache_base,
                    captured_sources,
                    self.asset_raw(57),
                )));
            } else if room_72_northward_palette_tail_uses_live_obj_cache {
                // By the state-7 palette tail, the northward transition has
                // crossed the Link DMA that Snes9x decodes for the active OBJ
                // list. The captured pre-main operands still name the prior
                // pose, while the following raw page is the decoded cache
                // generation that owns this scanout.
                self.set_obj_vram_latch_traced(Some(following.ppu.vram.clone()));
            } else if room_72_state8_scroll_after_first_tick_uses_live_obj_cache {
                // After the first resumed scroll tick, Snes9x decodes every
                // following Link DMA page before raw OBJ VRAM advances at the
                // display boundary. Carry only that renderer cache live.
                self.set_obj_vram_latch_traced(Some(following.ppu.vram.clone()));
            } else if dungeon_subtile_palette_filter_uses_live_obj_cache {
                // The palette loop can hold NMI_DoUpdates for a host frame.
                // Its captured source words still identify the decoded cache
                // generation already owned by this scanout, including the
                // generation made visible by the preceding boundary.
                // The ensuing NMI can consume live Link operands without
                // invalidating the tiles Snes9x already decoded for this
                // scanout. Keep the whole decoded Link batch on the captured
                // pre-main source generation; the next boundary owns the new
                // pose.
                let captured_sources = self
                    .pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.link_operands.sources)
                    .unwrap_or_else(|| LinkDmaSources::load_from_ram(&self.ram));
                let obj_cache_vram = compose_early_link_obj_cache(
                    &self.ppu.vram,
                    captured_sources,
                    self.asset_raw(57),
                );
                if crate::debug_env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some() {
                    eprintln!(
                        "display_subtile_cache host={} dma={} last={} head_source={:04x} raw_head={:04x} cache_head={:04x}",
                        self.frame_ctr_dbg,
                        self.link_obj_dma_completed_this_frame,
                        self.last_presented_obj_vram.is_some(),
                        captured_sources.source(LinkDmaSourceSlot::HeadTop),
                        self.ppu.vram[0x4020],
                        obj_cache_vram[0x4020],
                    );
                }
                self.set_obj_vram_latch_traced(Some(obj_cache_vram));
            } else if dungeon_subtile_palette_filter_holds_decoded_obj_cache {
                // A held NMI does not decode another Link batch. Raw OBJ VRAM
                // still belongs to its independently retained generation, so
                // carry forward the cache that the preceding scanout actually
                // presented instead of falling back to those raw words.
                if let Some(previous) = self.last_presented_obj_vram.as_deref() {
                    let mut obj_cache_vram = self.ppu.vram.clone();
                    obj_cache_vram[0x4000..0x4400].copy_from_slice(&previous[0x4000..0x4400]);
                    self.set_obj_vram_latch_traced(Some(obj_cache_vram));
                }
            } else if room_72_northward_subtile_shutter_retains_presented_obj_cache {
                // The northward palette tail already published the decoded Link
                // pose used by the active OBJ list. The following shutter slice
                // advances the raw/live DMA operands to the next pose, but
                // Snes9x keeps scanning the preceding decoded OBJ cache for one
                // more frame.
                if let Some(previous) = self.last_presented_obj_vram.as_deref() {
                    let mut obj_cache_vram = self.ppu.vram.clone();
                    obj_cache_vram[0x4000..0x4400].copy_from_slice(&previous[0x4000..0x4400]);
                    self.set_obj_vram_latch_traced(Some(obj_cache_vram));
                } else {
                    self.set_obj_vram_latch_traced(Some(following.ppu.vram.clone()));
                }
            } else if dungeon_subtile_shutter_handoff_uses_live_obj_cache {
                // Room $72's landing-to-shutter main slice retains the raw Link
                // page selected at host entry. Snes9x has nevertheless decoded
                // the post-main page for the active OBJ list, matching v1.0.0's
                // coherent live-VRAM render without advancing raw scanout.
                self.set_obj_vram_latch_traced(Some(following.ppu.vram.clone()));
            } else if dungeon_supertile_first_scroll_uses_live_obj_cache {
                // The first steady supertile-scroll scanout retains the raw
                // host-boundary Link tiles, but Snes9x's renderer cache has
                // already decoded the live NMI upload. Keep that cache-only
                // generation independent from the retained visible VRAM.
                let live_sources = LinkDmaSources::load_from_ram(&following.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                let mut obj_cache_vram = self.ppu.vram.clone();
                for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                    let source_address = usize::from(live_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                        (source_address >= 0x8000 && source_offset + len <= graphics.len())
                            .then_some(&graphics[source_offset..source_offset + len])
                    }) else {
                        obj_cache_vram[destination..destination_end]
                            .copy_from_slice(&following.ppu.vram[destination..destination_end]);
                        continue;
                    };
                    for (index, bytes) in source_bytes.chunks_exact(2).enumerate() {
                        obj_cache_vram[destination + index] =
                            u16::from_le_bytes([bytes[0], bytes[1]]);
                    }
                }
                self.set_obj_vram_latch_traced(Some(obj_cache_vram));
            } else if room_71_obj_cache_publication != Room71ObjCachePublication::RetainCurrent {
                // v1.0.0 rendered this handoff from live state. Preserve that
                // decoded OBJ generation while raw VRAM remains on the
                // independently selected host-boundary image.
                let mut obj_cache_vram = if room_71_obj_cache_publication
                    == Room71ObjCachePublication::UseCapturedWithLiveLinkHead
                {
                    self.ppu.vram.clone()
                } else {
                    following.ppu.vram.clone()
                };
                if room_71_obj_cache_publication
                    == Room71ObjCachePublication::UseCapturedWithLiveLinkHead
                {
                    // The captured display page already contains the live Link
                    // body and hand, but retains the preceding head pair. OBJ
                    // evaluation decodes the resident post-NMI head ranges
                    // (tiles $02/$03 and $12/$13) at this boundary.
                    let resident_obj_vram = &following.ppu.vram;
                    publish_live_link_head_obj_cache(&mut obj_cache_vram, resident_obj_vram);
                }
                self.set_obj_vram_latch_traced(Some(obj_cache_vram));
            } else if room_71_item_graphics_return_live_boundary {
                // The same completed vblank that advanced the packed OAM
                // size/X table also carried the 16x16 held-item CHR upload.
                // Snes9x's completed scanout decoded those receipt tiles, so
                // resolve the OBJ cache from live VRAM while raw scanout
                // retains the host-boundary image.
                self.set_obj_vram_latch_traced(Some(following.ppu.vram.clone()));
            } else if interrupted_dungeon_faded_filter_first_palette_pass {
                let captured_sources = self
                    .pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.link_operands.sources)
                    .unwrap_or_else(|| LinkDmaSources::load_from_ram(&self.ram));
                self.set_obj_vram_latch_traced(Some(compose_early_link_obj_cache(
                    &self.ppu.vram,
                    captured_sources,
                    self.asset_raw(57),
                )));
            }
        }
        if following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 3
            && !matches!(following_room, 0x70 | 0x71)
        {
            // The raw Link ranges remain on the prior completed DMA image, but
            // Snes9x has already decoded the post-NMI OBJ page used by this
            // scanout. Preserve that renderer-only generation independently.
            let mut obj_cache_vram =
                if following_room == 0x82 && following.room_82_sprite_conversion_deferred_nmi {
                    // Room $82 enters state $03 before its deferred vblank. The
                    // resident page is the decoded base for this scanout; only the
                    // completed early Link DMA batch advances independently below.
                    self.ppu.vram.clone()
                } else {
                    following.ppu.vram.clone()
                };
            let screen_transition = self.screen_transition();
            if matches!(screen_transition, 1 | 2) {
                let captured_sources = self
                    .pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.link_operands.sources)
                    .unwrap_or_else(|| LinkDmaSources::load_from_ram(&self.ram));
                let link_graphics = self.asset_raw(57).map(Vec::from);
                if screen_transition == 2 {
                    // Direction $02 reaches the deferred vblank after the
                    // complete early Link batch (body, head, and hands) has
                    // been decoded, while raw VRAM remains on the preceding
                    // image. Keep that cache generation renderer-only.
                    obj_cache_vram = compose_early_link_obj_cache(
                        &obj_cache_vram,
                        captured_sources,
                        link_graphics.as_deref(),
                    );
                } else {
                    // Direction $01 crosses only the two head transfers at
                    // this boundary.
                    for (destination, source) in [
                        (0x4020, LinkDmaSourceSlot::HeadTop),
                        (0x4120, LinkDmaSourceSlot::HeadBottom),
                    ] {
                        let len = 0x40;
                        let source_address = usize::from(captured_sources.source(source));
                        let source_offset = source_address.saturating_sub(0x8000);
                        let destination_end = destination + len / 2;
                        let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                            (source_address >= 0x8000 && source_offset + len <= graphics.len())
                                .then_some(&graphics[source_offset..source_offset + len])
                        }) else {
                            continue;
                        };
                        for (word, bytes) in obj_cache_vram[destination..destination_end]
                            .iter_mut()
                            .zip(source_bytes.chunks_exact(2))
                        {
                            *word = u16::from_le_bytes([bytes[0], bytes[1]]);
                        }
                    }
                }
            }
            self.set_obj_vram_latch_traced(Some(obj_cache_vram));
        }
        if plan.publish_live_spiral_stair_obj_cache {
            if plan.publish_live_spiral_stair_return_obj_vram {
                // The main-loop re-entry after the return boundary completes
                // the pending Link DMA before the next scanout. Unlike the
                // landing and motion boundaries, both the decoded cache and
                // the raw OBJ generation are visible here.
                self.ppu.vram[0x4000..0x4400].copy_from_slice(&following.ppu.vram[0x4000..0x4400]);
            }
            // The landing scanout publishes the completed OAM list while raw
            // OBJ VRAM still belongs to the preceding display generation.
            // Snes9x has already decoded the live early Link DMA operands for
            // that OAM list, so keep the cache generation independent.
            self.set_obj_vram_latch_traced(Some(compose_complete_link_obj_cache(
                &self.ppu.vram,
                LinkDmaSources::load_from_ram(&following.ram),
                self.asset_raw(57),
                &following.ram,
            )));
        }
        self.compose_oam_debug_display_obj_vram_frame(
            following,
            plan,
            following_frame,
            following_room,
        );
        if plan.dungeon_faded_filter_phase == DungeonFadedFilterPublicationPhase::CallerReturn {
            if let Some(previous) = self.last_presented_obj_vram.as_deref() {
                let mut obj_cache_vram = self.ppu.vram.clone();
                obj_cache_vram[0x4000..0x4400].copy_from_slice(&previous[0x4000..0x4400]);
                self.set_obj_vram_latch_traced(Some(obj_cache_vram));
            }
        }
        let interrupted_obj_cache = match effective_obj_cache_generation {
            DisplayObjCacheGeneration::FollowModuleCadence => None,
            DisplayObjCacheGeneration::CapturedBeforeNmi => interrupted_obj_cache_base,
            DisplayObjCacheGeneration::HostBoundaryBeforeMain => self
                .pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.obj_vram.clone()),
            DisplayObjCacheGeneration::EarlyLinkDmaFromHostOperands => self
                .pre_main_graphics_dma
                .as_ref()
                .zip(interrupted_obj_cache_base.as_deref())
                .map(|(graphics, base)| {
                    compose_early_link_obj_cache(
                        base,
                        graphics.link_operands.sources,
                        self.asset_raw(57),
                    )
                }),
        };
        if let Some(interrupted_obj_cache) = interrupted_obj_cache {
            // This typed interrupted-call receipt is later than module-level
            // cache fallbacks. It selects only the decoded OBJ page; raw VRAM
            // and OAM keep their independently recorded generations.
            self.set_obj_vram_latch_traced(Some(interrupted_obj_cache));
        }
        if let Some(explicit_obj_cache_vram) = following.explicit_obj_cache_vram.as_ref() {
            // This semantic receipt names the cache which was decoded before
            // sprite evaluation. A later raw-VRAM DMA in the same host interval
            // cannot retroactively invalidate that completed scanout.
            self.set_obj_vram_latch_traced(Some(explicit_obj_cache_vram.clone()));
        }
        if plan.oam_scanout_source.retains_previous_presented() {
            // Select this render-boundary generation last. The short iris
            // caller has already authored a future software shadow and can
            // also have a closed NMI receipt, but neither event occurred
            // before sprite evaluation for the field being composed.
            // A before/after receipt pair can compose twice within one host;
            // in that interval the retiring field is staged but has not yet
            // been promoted into `last_presented_oam`.
            if let Some(previous) = self.retiring_or_last_presented_oam().map(<[u16]>::to_vec) {
                self.ppu.oam.copy_from_slice(&previous);
            }
        }
        if let Some(immutable_captured_oam) = immutable_captured_oam.as_deref() {
            // Apply this event receipt after every module-level fallback. Its
            // defining hardware event is the absence of OAM DMA, so later
            // shadow, resident, and closed-boundary heuristics are inapplicable.
            self.ppu.oam.clone_from_slice(immutable_captured_oam);
        }
        self.compose_oam_capture_publication_candidates(
            following,
            plan,
            candidate_captured_oam,
            candidate_captured_vram,
            capture_publication_candidates,
            entry_frame,
            following_frame,
            following_room,
        );
        self.ppu.obj_previous_frame_vram = following.ppu.obj_previous_frame_vram.clone();
    }

    pub(super) fn retiring_or_last_presented_oam(&self) -> Option<&[u16]> {
        // Main-thread continuations can queue a retained generation before the
        // renderer promotes this host frame's composed scanout into history.
        // In that window, staged OAM is the generation currently retiring on
        // hardware; last-presented is the correct fallback before staging.
        self.staged_presented_oam
            .as_deref()
            .or(self.last_presented_oam.as_deref())
    }

    pub(super) fn compose_effective_presented_obj(&mut self, following: &DisplaySnapshot) {
        let captured_obj_name_bases = [self.ppu.obj_tile_adr1, self.ppu.obj_tile_adr2];
        let captured_obj_vram_latch = self.ppu.obj_vram_latch.clone();
        self.compose_effective_presented_obj_from_captured_scanout(
            following,
            captured_obj_name_bases,
            captured_obj_vram_latch.as_deref(),
        );
    }

    pub(super) fn compose_effective_presented_obj_from_captured_scanout(
        &mut self,
        following: &DisplaySnapshot,
        captured_obj_name_bases: [u16; 2],
        captured_obj_vram_latch: Option<&[u16]>,
    ) {
        if following.effective_obj_cache_generation()
            != DisplayObjCacheGeneration::FollowModuleCadence
            || following.explicit_obj_cache_vram.is_some()
        {
            // A typed interrupted-call receipt or an explicitly composed
            // publication cache names the generation at the later
            // sprite-evaluation event. A bulk NMI write receipt may still own
            // raw VRAM, but cannot retroactively replace that already-decoded
            // OBJ page. The bulk receipt remains the fallback only when no
            // cache owner was selected.
            return;
        }
        let Some(receipt) = following.effective_presented_dma.as_ref() else {
            return;
        };
        // OBSEL names word addresses in the SNES's 32K-word VRAM space. The
        // renderer masks the selected name base through $7fff before fetching,
        // so a hardware page can wrap from $7fff back to $0000.
        const OBJ_VRAM_WORD_MASK: usize = 0x7fff;
        let obj_name_base_1 = usize::from(captured_obj_name_bases[0]) & OBJ_VRAM_WORD_MASK;
        let obj_name_base_2 = usize::from(captured_obj_name_bases[1]) & OBJ_VRAM_WORD_MASK;
        let is_obj_cache_word = |index: usize| {
            let index = index & OBJ_VRAM_WORD_MASK;
            (index.wrapping_sub(obj_name_base_1) & OBJ_VRAM_WORD_MASK) < 0x1000
                || (index.wrapping_sub(obj_name_base_2) & OBJ_VRAM_WORD_MASK) < 0x1000
        };
        let obj_writes = receipt
            .vram_writes
            .iter()
            .copied()
            .filter(|&(index, _)| is_obj_cache_word(index))
            .collect::<Vec<_>>();
        if obj_writes.is_empty() {
            // An effective receipt can contain only coupled registers, CGRAM,
            // or an unrelated BG upload. None of those events invalidates an
            // OBJ tile, so do not synthesize a decoded cache from history.
            return;
        }
        if nmi::debug_frame_selection_env_matches("ZELDA3_DEBUG_OBJ_PIPE", self.frame_ctr_dbg) {
            eprintln!(
                "objpipe host={} stage=receipt_compose obj_writes={} total_writes={} last_presented_base={}",
                self.frame_ctr_dbg,
                obj_writes.len(),
                receipt.vram_writes.len(),
                self.last_presented_obj_vram.is_some(),
            );
        }
        // Snes9x invalidates decoded 4bpp tiles at the exact VRAM writes, then
        // lazily decodes them when OBJ evaluation needs them. Preserve the
        // complete prior cache generation and advance only the two hardware
        // OBJ name pages from this boundary's observed DMA writes.
        let mut obj_vram = captured_obj_vram_latch
            .filter(|vram| vram.len() == self.ppu.vram.len())
            .map(<[u16]>::to_vec)
            .or_else(|| {
                self.ppu
                    .obj_vram_latch
                    .as_ref()
                    .filter(|vram| vram.len() == self.ppu.vram.len())
                    .cloned()
            })
            .or_else(|| {
                self.last_presented_obj_vram
                    .as_ref()
                    .filter(|vram| vram.len() == self.ppu.vram.len())
                    .cloned()
            })
            .unwrap_or_else(|| self.ppu.vram.clone());
        for (index, value) in obj_writes {
            if let Some(word) = obj_vram.get_mut(index) {
                *word = value;
            }
        }
        // A recorded OBJ write owns the decoded cache generation even when its
        // value happens to equal raw VRAM. Collapsing that owner to None lets a
        // later sparse receipt fall back to an older presented cache.
        self.set_obj_vram_latch_traced(Some(obj_vram));
    }

    pub(super) fn record_completed_link_obj_dma_for_display_boundary(
        &mut self,
        sources: LinkDmaSources,
        source_generation: GraphicsDmaGeneration,
    ) {
        // `nmi_core_link_graphics_update` has already chosen between captured
        // and live operands. Preserve those exact C dma_source_addr_0..5 words
        // for an explicit leading-NMI receipt; trailing-NMI callers discard
        // this field because their transfer belongs to the next scanout.
        if let Some(writes) = self.active_effective_dma_writes.as_mut() {
            writes.completed_link_obj_dma = Some(CompletedLinkObjDma {
                sources,
                source_generation,
            });
        }
    }

    pub(super) fn record_completed_oam_dma_for_display_boundary(&mut self) {
        // This is an event receipt, not a prediction. nmi_do_updates_from has
        // already selected the hardware operand generation and copied that
        // exact payload into PPU OAM. Record what the DMA installed instead of
        // reconstructing it from frame-counter or dialogue state afterward.
        let completed = self.ppu.oam.clone();
        if let Some(writes) = self.active_effective_dma_writes.as_mut() {
            writes.completed_oam = Some(completed.clone());
        }
        if nmi::debug_frame_selection_env_matches(
            "ZELDA3_DEBUG_DISPLAY_OAM_FRAME",
            self.frame_ctr_dbg,
        ) {
            eprintln!(
                "oam_dma_receipt host={} active_snapshot={:?}",
                self.frame_ctr_dbg,
                self.display_snapshot.as_ref().map(|display| (
                    display.publication_host_frame,
                    display.accepts_nmi_dma_receipts,
                )),
            );
        }
        let retiring_oam = self.retiring_or_last_presented_oam().map(Vec::from);
        {
            let Some(display) = self
                .display_snapshot
                .as_mut()
                .filter(|display| display.publication_host_frame == self.frame_ctr_dbg)
            else {
                return;
            };
            if display.accepts_nmi_dma_receipts {
                display.completed_oam_dma_after_capture = Some(completed);
            } else if display.closed_oam_boundary_receipt.is_none() {
                let active_oam = display
                    .completed_oam_dma_after_capture
                    .as_deref()
                    .or(retiring_oam.as_deref())
                    .unwrap_or(&display.ppu.oam)
                    .to_vec();
                display.closed_oam_boundary_receipt = Some(ClosedOamBoundaryReceipt {
                    publication_host_frame: self.frame_ctr_dbg,
                    active_oam,
                });
            }
        }
    }

    pub(super) fn clear_oam_buffer(&mut self) {
        for i in 0..128 {
            self.oam_state_mut().hide_sprite_row(i);
        }
    }

    pub(super) fn set_oam_helper0_at(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut()
            .write_clipped_entry_with_extended(oam, x, y, charnum, flags, big);
    }

    pub(super) fn set_oam_helper1_at(
        &mut self,
        oam: usize,
        x: u16,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut()
            .write_entry_with_extended(oam, x, y, charnum, flags, big);
    }

    pub(super) fn set_oam_plain(
        &mut self,
        index: usize,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut()
            .write_indexed_entry_with_extended(index, x, y, charnum, flags, big);
    }

    pub(super) fn set_oam_helper0_index(
        &mut self,
        index: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut()
            .write_indexed_clipped_entry_with_extended(index, x, y, charnum, flags, big);
    }
}
