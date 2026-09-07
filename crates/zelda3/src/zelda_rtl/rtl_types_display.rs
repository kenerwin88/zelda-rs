//! Types split out of `zelda_rtl.rs` by family (types_display).
//! Mechanical move: definitions are unchanged; items and struct fields
//! became `pub(crate)` so the parent module and its siblings see them.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompletedLinkObjDma {
    pub(crate) sources: LinkDmaSources,
    pub(crate) source_generation: GraphicsDmaGeneration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct NmiActiveDisplayBlanking {
    pub(crate) prefix_scanlines: u8,
    pub(crate) suffix_start_scanline: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ActiveDisplayBlankingScanout {
    pub(crate) suffix_start_scanline: Option<u8>,
    pub(crate) retain_prior_surface: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphicsDmaGeneration {
    HostBoundaryBeforeMain,
    LiveAfterMain,
}

impl GraphicsDmaGeneration {
    pub(crate) const fn resolve_live_override(self, publish_live_generation: bool) -> Self {
        if publish_live_generation {
            Self::LiveAfterMain
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OamScanoutSource {
    RetainCapturedBeforeNmi,
    /// Keep the immutable PPU OAM image that was present when display
    /// composition began. A held NMI can publish register writes without an
    /// OAM DMA; in that case no adjacent shadow or later boundary receipt is
    /// allowed to replace the hardware table already being scanned.
    RetainImmutableCapturedPpu,
    /// Publish the exact OAM operand recorded by a full NMI that followed an
    /// earlier held-latch NMI. The receipt, rather than a neighboring software
    /// shadow, decides which hardware generation owns this scanout.
    ComposeObservedNmiReceipt,
    /// Keep the exact OAM generation consumed by the preceding scanout.
    /// A caller-only CPU slice can return after sprite evaluation has already
    /// begun, so neither the newly captured PPU table nor the ensuing NMI DMA
    /// belongs to the active field.
    RetainPreviousPresented,
    /// Keep the OAM already resident in the captured PPU. Unlike the ordinary
    /// retained cadence, do not replace it with an adjacent published shadow:
    /// an interrupted native workload can leave the completed hardware DMA and
    /// the next software shadow on opposite sides of the host boundary.
    RetainResidentPpuOam,
    ComposePublishedShadowDma,
    /// Publish the complete OAM DMA operand captured at the host boundary.
    /// This is the source consumed by a leading NMI before the translated
    /// main-thread slice mutates the live software shadow.
    ComposeHostBoundaryShadowDma,
    ComposeLiveAfterNmi,
    /// Publish the current sprite table except for Link's sorted body and
    /// equipment entries, which still belong to the pre-main host boundary.
    ComposeLiveAfterNmiWithHostBoundaryLink,
    /// Publish the complete OAM DMA performed after a suspended CPU workload
    /// returned before vblank. This is intentionally explicit: ordinary
    /// module-level OAM deferral must not replace this measured generation.
    ComposeCompletedWorkAfterNmi,
    /// Publish the complete software OAM shadow authored after main. A
    /// translated caller can finish its OAM suffix before the display image is
    /// materialized, so neither the resident nor live PPU table represents the
    /// generation scanned by hardware.
    ComposeLiveShadowAfterMain,
    /// Publish the exact software OAM shadow present when vblank interrupted
    /// Sprite_Main. The ROM's NMI_DoUpdates copies $0800..$0a1f directly to
    /// PPU OAM, including partial ordinary- or cached-sprite authorship.
    ComposeInterruptedSpriteMainShadowDma,
    /// Publish only Link's sorted OAM range from the live software shadow.
    /// The spiral-stair return completes that player range before scanout,
    /// while the rest of the resident sprite table remains authoritative.
    ComposeSpiralReturnPlayerShadowAfterMain,
    /// Publish the OAM shadow authored by this main slice. Dungeon big-item
    /// pickup enters Link's hold-item handler before the boundary that starts
    /// the receipt sequence, so the live shadow wins even though the resident
    /// native PPU still owns the preceding DMA generation.
    ComposeLivePlayerOamAfterMain,
}

impl OamScanoutSource {
    pub(crate) const fn active_nmi_dma_is_presented(self) -> bool {
        matches!(
            self,
            Self::ComposeLiveAfterNmi | Self::ComposeCompletedWorkAfterNmi
        )
    }

    pub(crate) const fn retains_previous_presented(self) -> bool {
        matches!(self, Self::RetainPreviousPresented)
    }

    pub(crate) const fn blocks_post_composition_override(self) -> bool {
        matches!(
            self,
            Self::RetainPreviousPresented | Self::RetainImmutableCapturedPpu
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CgramScanoutGeneration {
    /// Prefer the pre-upload CGRAM latch when this boundary's NMI uploaded the
    /// main palette buffer. This is the ordinary main-loop cadence.
    #[default]
    BeforeNmiUpload,
    /// Keep the CGRAM used by the preceding scanout. A resumed palette-filter
    /// slice can author the captured PPU palette before the current field has
    /// consumed it, so even the captured boundary can be one generation ahead.
    RetainPreviousPresented,
    /// The suspended CPU workload completed its palette walk before vblank,
    /// so the ensuing NMI upload belongs to the scanout being published now.
    LiveAfterNmi,
}

impl From<GraphicsDmaGeneration> for OamScanoutSource {
    fn from(generation: GraphicsDmaGeneration) -> Self {
        match generation {
            GraphicsDmaGeneration::HostBoundaryBeforeMain => Self::RetainCapturedBeforeNmi,
            GraphicsDmaGeneration::LiveAfterMain => Self::ComposeLiveAfterNmi,
        }
    }
}

impl OamScanoutSource {
    pub(crate) const fn resolve_live_override(self, publish_live_after_nmi: bool) -> Self {
        if publish_live_after_nmi {
            Self::ComposeLiveAfterNmi
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ObjScanoutGenerations {
    pub(crate) oam: OamScanoutSource,
    pub(crate) link_obj: GraphicsDmaGeneration,
    pub(crate) link_obj_sources: GraphicsDmaGeneration,
}

impl ObjScanoutGenerations {
    pub(crate) const fn coherent(generation: GraphicsDmaGeneration) -> Self {
        Self {
            oam: match generation {
                GraphicsDmaGeneration::HostBoundaryBeforeMain => {
                    OamScanoutSource::RetainCapturedBeforeNmi
                }
                GraphicsDmaGeneration::LiveAfterMain => OamScanoutSource::ComposeLiveAfterNmi,
            },
            link_obj: generation,
            link_obj_sources: generation,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AnimatedBgScanoutGeneration {
    #[default]
    HostBoundaryBeforeNmi,
    LiveAfterNmi,
}

impl AnimatedBgScanoutGeneration {
    pub(crate) const fn resolve_live_override(self, publish_live_after_nmi: bool) -> Self {
        if publish_live_after_nmi {
            Self::LiveAfterNmi
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GraphicsDmaPlan {
    pub(crate) oam_operands: GraphicsDmaGeneration,
    pub(crate) oam_scanout: OamScanoutSource,
    pub(crate) link_obj_scanout: GraphicsDmaGeneration,
    pub(crate) link_obj_operands: GraphicsDmaGeneration,
    pub(crate) animated_bg_operands: GraphicsDmaGeneration,
    pub(crate) animated_bg_scanout: AnimatedBgScanoutGeneration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LinkObjDmaPhaseGenerations {
    pub(crate) presented: GraphicsDmaGeneration,
    pub(crate) following_nmi: GraphicsDmaGeneration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Room71ObjCachePublication {
    RetainCurrent,
    UseLive,
    UseCapturedWithLiveLinkHead,
}

impl Room71ObjCachePublication {
    pub(crate) const fn resolve(use_live: bool, publish_live_link_head: bool) -> Self {
        if publish_live_link_head {
            Self::UseCapturedWithLiveLinkHead
        } else if use_live {
            Self::UseLive
        } else {
            Self::RetainCurrent
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PreMainNmiScanoutGenerations {
    pub(crate) publication: DisplaySnapshotPublication,
    pub(crate) vram: DisplayVramGeneration,
    pub(crate) animated_bg: Option<AnimatedBgScanoutGeneration>,
    pub(crate) bg_scroll: DisplayBgScrollGeneration,
    pub(crate) obj: Option<ObjScanoutGenerations>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverworldSpriteReloadBg1Generation {
    RetainBeforePrepublishedRain,
    ComposeAtTransitionReturn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverworldSpriteReloadResumeScanout {
    ByReturnPhase(NmiPhase),
    CpuSliceEntry {
        scroll: BgScrollRegisterScanout,
        bg1_generation: OverworldSpriteReloadBg1Generation,
    },
}

impl OverworldSpriteReloadResumeScanout {
    pub(crate) const fn generations(self) -> PreMainNmiScanoutGenerations {
        match self {
            Self::ByReturnPhase(return_phase) => PreMainNmiScanoutGenerations {
                publication: match return_phase {
                    NmiPhase::BeforeNmi => DisplaySnapshotPublication::PublishCaptured,
                    NmiPhase::AfterNmi => DisplaySnapshotPublication::RetainPublished,
                },
                vram: DisplayVramGeneration::RetainCapturedBeforeNmi,
                animated_bg: None,
                bg_scroll: return_phase.return_bg_scroll_generation(),
                obj: None,
            },
            Self::CpuSliceEntry { scroll, .. } => PreMainNmiScanoutGenerations {
                publication: DisplaySnapshotPublication::PublishCaptured,
                vram: DisplayVramGeneration::ComposeLiveAfterNmi,
                // Animated-BG DMA is still owned by its ordinary NMI cadence.
                // The later 09/05 -> 09/06 boundary publishes the uploaded
                // tiles explicitly when that generation reaches scanout.
                animated_bg: None,
                bg_scroll: DisplayBgScrollGeneration::RetainCpuSliceEntry(scroll),
                obj: None,
            },
        }
    }

    pub(crate) fn complete_transition_return(self, returned: BgScrollRegisterScanout) -> Self {
        let Self::CpuSliceEntry {
            mut scroll,
            bg1_generation,
        } = self
        else {
            return self;
        };
        if bg1_generation == OverworldSpriteReloadBg1Generation::ComposeAtTransitionReturn {
            scroll.offsets[0] = returned.offsets[0];
        }
        scroll.offsets[1] = returned.offsets[1];
        Self::CpuSliceEntry {
            scroll,
            bg1_generation,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DialogueTextScanout {
    pub(crate) vram: Vec<u16>,
    pub(crate) glyph_runs: Vec<Bg3VwfGlyphRun>,
    pub(crate) glyph_run_dialogue_offsets: Vec<u16>,
    pub(crate) dialogue_msg_read_pos: u16,
    pub(crate) dialogue_message_id: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogueTextGeneration {
    PublishedDisplay,
    CurrentRenderBuffer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct DialogueScanoutOwnership(u8);

impl DialogueScanoutOwnership {
    pub(crate) const SNAPSHOT: Self = Self(0);
    pub(crate) const FROZEN_SCROLL_COPY: Self = Self(1);
    pub(crate) const COMPLETION_PENDING: Self = Self(2);
    pub(crate) const COMPLETED_SCROLL: Self = Self(3);
    pub(crate) const COMPLETION_STAGED_AFTER_FROZEN: Self = Self(4);
    pub(crate) const COMPLETION_STAGED_AFTER_SNAPSHOT: Self = Self(5);
    pub(crate) const RETIRED_TEXT_DMA: Self = Self(6);

    pub(crate) const fn is_snapshot(self) -> bool {
        self.0 == Self::SNAPSHOT.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BgScrollRegisterScanout {
    pub(crate) offsets: [[u16; 2]; 4],
}

impl BgScrollRegisterScanout {
    pub(crate) fn capture(ppu: &PpuState) -> Self {
        Self {
            offsets: std::array::from_fn(|index| {
                [ppu.bg_layer[index].h_scroll, ppu.bg_layer[index].v_scroll]
            }),
        }
    }

    pub(crate) fn publish_to(self, ppu: &mut PpuState) {
        for (layer, [h_scroll, v_scroll]) in ppu.bg_layer.iter_mut().zip(self.offsets) {
            layer.h_scroll = h_scroll;
            layer.v_scroll = v_scroll;
        }
    }

    pub(crate) fn after_nmi_writes(ppu: &PpuState, register_bytes: [[u8; 4]; 3]) -> Self {
        let mut scanout = Self::capture(ppu);
        let mut previous = ppu.scroll_prev;
        let mut previous2 = ppu.scroll_prev2;
        for (layer, [h_low, h_high, v_low, v_high]) in register_bytes.into_iter().enumerate() {
            for value in [h_low, h_high] {
                scanout.offsets[layer][0] = ((u16::from(value)) << 8)
                    | (u16::from(previous) & 0xf8)
                    | (u16::from(previous2) & 0x07);
                previous = value;
                previous2 = value;
            }
            for value in [v_low, v_high] {
                scanout.offsets[layer][1] = ((u16::from(value)) << 8) | u16::from(previous);
                previous = value;
            }
        }
        scanout
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayVramGeneration {
    #[default]
    ComposeLiveAfterNmi,
    RetainCapturedBeforeNmi,
}

impl DisplayVramGeneration {
    pub(crate) const fn resolve_for_scanout(
        self,
        retain_previous_nmi_display_memory: bool,
    ) -> Self {
        if retain_previous_nmi_display_memory || matches!(self, Self::RetainCapturedBeforeNmi) {
            Self::RetainCapturedBeforeNmi
        } else {
            Self::ComposeLiveAfterNmi
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplaySnapshotPublication {
    #[default]
    /// Publish this capture immediately and discard any staged generation.
    PublishCaptured,
    /// Stage this capture while publishing the generation staged previously.
    AdvanceStaged,
    /// Keep both the currently published and staged generations unchanged.
    RetainPublished,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DungeonLandingGoalDisplayHandoff {
    #[default]
    None,
    /// The final Module 7 circle has been staged. Its caller-return capture
    /// must retain the active mixed reset scanout in the same host interval.
    RetainCallerReturn,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayHdmaTableGeneration {
    #[default]
    Captured,
    /// The spotlight circle has been projected before HDMA starts, while OAM,
    /// VRAM, and the remaining display domains still own the older snapshot.
    SpotlightPublishedAheadOfSnapshot { active_table: Vec<u8> },
    /// The CPU is still projecting the new table while HDMA consumes it.
    /// Words that miss their scanline retain the pre-projection generation.
    AttractMapProjectionDuringScanout { before_projection: Vec<u8> },
    /// The spotlight circle builder crosses HDMA at scanline 221. The active
    /// scanout therefore owns the preceding table above that boundary and the
    /// newly projected table from that boundary downward.
    SpotlightProjectionDuringScanout {
        before_projection: [Vec<u8>; 2],
        after_projection: [Vec<u8>; 2],
        live_tail_start: usize,
    },
}

impl DisplayHdmaTableGeneration {
    pub(crate) fn compose_into(&self, ram: &mut [u8]) {
        match self {
            Self::Captured => {}
            Self::SpotlightPublishedAheadOfSnapshot { active_table } => {
                let byte_count = active_table.len().min(ZeldaState::HDMA_DYNAMIC_TABLE_LEN);
                ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + byte_count]
                    .copy_from_slice(&active_table[..byte_count]);
            }
            Self::AttractMapProjectionDuringScanout { before_projection } => {
                for scanline in 0..ATTRACT_MAP_PROJECTION_WORDS {
                    if !attract_map_projection_current_word_is_visible(scanline) {
                        let offset = scanline * 2;
                        ram[HDMA_TABLE_DYNAMIC + offset..HDMA_TABLE_DYNAMIC + offset + 2]
                            .copy_from_slice(&before_projection[offset..offset + 2]);
                    }
                }
            }
            Self::SpotlightProjectionDuringScanout {
                before_projection,
                after_projection,
                live_tail_start,
            } => {
                let byte_start = live_tail_start * 2;
                for ((table_base, before), after) in [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
                    .into_iter()
                    .zip(before_projection)
                    .zip(after_projection)
                {
                    let byte_count = before
                        .len()
                        .min(after.len())
                        .min(ZeldaState::HDMA_DYNAMIC_TABLE_LEN);
                    let split = byte_start.min(byte_count);
                    ram[table_base..table_base + split].copy_from_slice(&before[..split]);
                    ram[table_base + split..table_base + byte_count]
                        .copy_from_slice(&after[split..byte_count]);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayBgScrollGeneration {
    #[default]
    RetainCapturedBeforeNmi,
    /// A scheduled CPU continuation returned after NMI, so the active scanout
    /// owns the scroll registers from the start of that interrupted host slice
    /// rather than the snapshot captured after its caller suffix completed.
    RetainCpuSliceEntry(BgScrollRegisterScanout),
    ComposeLiveAfterNmi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DisplayedBgScrollSource {
    CapturedBeforeNmi,
    CpuSliceEntry(BgScrollRegisterScanout),
    LiveAfterNmi,
    LiveBg1AfterNmi,
}

impl DisplayedBgScrollSource {
    pub(crate) fn resolve(
        captured_generation: DisplayBgScrollGeneration,
        dungeon_exit_crosses_nmi_boundary: bool,
        publish_live_overworld_bad_weather_scroll: bool,
        attract_map_retains_display_memory: bool,
    ) -> Self {
        if captured_generation == DisplayBgScrollGeneration::ComposeLiveAfterNmi
            || dungeon_exit_crosses_nmi_boundary
            || attract_map_retains_display_memory
        {
            Self::LiveAfterNmi
        } else if publish_live_overworld_bad_weather_scroll {
            Self::LiveBg1AfterNmi
        } else if let DisplayBgScrollGeneration::RetainCpuSliceEntry(scroll) = captured_generation {
            Self::CpuSliceEntry(scroll)
        } else {
            Self::CapturedBeforeNmi
        }
    }

    pub(crate) fn compose_into(self, shown: &mut PpuState, live: &PpuState) {
        match self {
            Self::CapturedBeforeNmi => {}
            Self::CpuSliceEntry(scroll) => scroll.publish_to(shown),
            Self::LiveAfterNmi => {
                for (shown, live) in shown.bg_layer.iter_mut().zip(&live.bg_layer) {
                    shown.h_scroll = live.h_scroll;
                    shown.v_scroll = live.v_scroll;
                }
            }
            Self::LiveBg1AfterNmi => {
                shown.bg_layer[0].h_scroll = live.bg_layer[0].h_scroll;
                shown.bg_layer[0].v_scroll = live.bg_layer[0].v_scroll;
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayObjGeneration {
    #[default]
    FollowModuleCadence,
    RetainCapturedOam {
        oam: Vec<u16>,
    },
    RetainCapturedMemory {
        oam: Vec<u16>,
        vram: Vec<u16>,
    },
}

/// Decoded OBJ cache generation selected independently from raw OBJ VRAM.
/// Snes9x decodes CHR transfers as they complete, so a held NMI can retain
/// the host-boundary cache while the caller-return field observes the early
/// Link DMA batch sourced from that same boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayObjCacheGeneration {
    #[default]
    FollowModuleCadence,
    CapturedBeforeNmi,
    HostBoundaryBeforeMain,
    EarlyLinkDmaFromHostOperands,
}

impl DisplayObjGeneration {
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::FollowModuleCadence => "follow-module-cadence",
            Self::RetainCapturedOam { .. } => "retain-captured-oam",
            Self::RetainCapturedMemory { .. } => "retain-captured-memory",
        }
    }

    pub(crate) fn retained_oam(&self) -> Option<&[u16]> {
        match self {
            Self::FollowModuleCadence => None,
            Self::RetainCapturedOam { oam } | Self::RetainCapturedMemory { oam, .. } => Some(oam),
        }
    }

    pub(crate) fn retained_vram(&self) -> Option<&[u16]> {
        match self {
            Self::FollowModuleCadence | Self::RetainCapturedOam { .. } => None,
            Self::RetainCapturedMemory { vram, .. } => Some(vram),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DisplayPublicationSignals {
    pub(crate) retain_previous_nmi_display_memory: bool,
    pub(crate) module_oam_publication_is_deferred: bool,
    pub(crate) dungeon_exit_crosses_nmi_boundary: bool,
    pub(crate) publish_live_overworld_bad_weather_scroll: bool,
    pub(crate) attract_map_retains_display_memory: bool,
    pub(crate) world_map_fade_display: bool,
    pub(crate) world_map_mode7_brightness_is_early_published: bool,
    pub(crate) dungeon_item_hold_publishes_live_scroll: bool,
    pub(crate) spiral_stair_landing_publishes_live_display: bool,
    pub(crate) spiral_stair_motion_publishes_live_oam: bool,
    pub(crate) spiral_stair_return_publishes_live_shadow_oam: bool,
    pub(crate) overworld_sprite_reload_completion_retains_presented: bool,
    pub(crate) spiral_stair_return_publishes_live_registers: bool,
    pub(crate) spiral_stair_return_publishes_live_obj_cache: bool,
    pub(crate) dungeon_brightness_publishes_live_display: bool,
    pub(crate) dungeon_brightness_publishes_live_animated_bg: bool,
    pub(crate) dungeon_state_13_phase: DungeonState13PublicationPhase,
    pub(crate) dungeon_faded_filter_phase: DungeonFadedFilterPublicationPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DisplayPublicationPlan {
    pub(crate) vram_generation: DisplayVramGeneration,
    pub(crate) compose_live_cgram: bool,
    pub(crate) oam_scanout_source: OamScanoutSource,
    pub(crate) retain_captured_oam: bool,
    pub(crate) link_obj_scanout_generation: GraphicsDmaGeneration,
    pub(crate) link_obj_source_generation: GraphicsDmaGeneration,
    pub(crate) animated_bg_scanout_generation: AnimatedBgScanoutGeneration,
    pub(crate) bg_scroll_source: DisplayedBgScrollSource,
    pub(crate) world_map_fade_display: bool,
    pub(crate) world_map_mode7_brightness_is_early_published: bool,
    pub(crate) publish_live_screen_layers: bool,
    pub(crate) publish_post_main_hud_dma: bool,
    pub(crate) publish_live_nmi_copy_packets: bool,
    pub(crate) publish_live_dungeon_state_13_palette_and_registers: bool,
    pub(crate) dungeon_state_13_phase: DungeonState13PublicationPhase,
    pub(crate) dungeon_faded_filter_phase: DungeonFadedFilterPublicationPhase,
    pub(crate) publish_live_spiral_stair_obj_cache: bool,
    pub(crate) publish_live_spiral_stair_return_obj_vram: bool,
    pub(crate) publish_spiral_stair_return_equipment_handoff: bool,
}

impl DisplayPublicationPlan {
    pub(crate) fn resolve(snapshot: &DisplaySnapshot, signals: DisplayPublicationSignals) -> Self {
        let explicit_oam_generation = matches!(
            snapshot.oam_scanout_source,
            OamScanoutSource::RetainPreviousPresented
                | OamScanoutSource::RetainResidentPpuOam
                | OamScanoutSource::ComposePublishedShadowDma
                | OamScanoutSource::ComposeCompletedWorkAfterNmi
                | OamScanoutSource::ComposeLiveShadowAfterMain
                | OamScanoutSource::ComposeSpiralReturnPlayerShadowAfterMain
                | OamScanoutSource::ComposeLivePlayerOamAfterMain
        );
        let module_oam_scanout_source =
            if signals.module_oam_publication_is_deferred && !explicit_oam_generation {
                OamScanoutSource::RetainCapturedBeforeNmi
            } else {
                snapshot.oam_scanout_source
            };
        let oam_scanout_source = if matches!(
            signals.dungeon_state_13_phase,
            DungeonState13PublicationPhase::RecurringMain
                | DungeonState13PublicationPhase::AtomicCallerReturn
                | DungeonState13PublicationPhase::CallerReturn
        ) || signals.dungeon_faded_filter_phase
            == DungeonFadedFilterPublicationPhase::CallerReturn
        {
            OamScanoutSource::RetainCapturedBeforeNmi
        } else if signals.overworld_sprite_reload_completion_retains_presented {
            OamScanoutSource::RetainResidentPpuOam
        } else if signals.dungeon_exit_crosses_nmi_boundary {
            // The suspended spotlight caller completed its shadow before the
            // NMI. Use the exact completed DMA receipt; the snapshot's staged
            // host generation describes the pre-interrupt entry only.
            OamScanoutSource::ComposeCompletedWorkAfterNmi
        } else if signals.spiral_stair_return_publishes_live_shadow_oam {
            OamScanoutSource::ComposeSpiralReturnPlayerShadowAfterMain
        } else if signals.spiral_stair_return_publishes_live_obj_cache {
            OamScanoutSource::ComposePublishedShadowDma
        } else if signals.spiral_stair_landing_publishes_live_display
            || signals.spiral_stair_motion_publishes_live_oam
        {
            OamScanoutSource::ComposeCompletedWorkAfterNmi
        } else {
            module_oam_scanout_source
        };
        let link_obj_scanout_generation = if matches!(
            signals.dungeon_state_13_phase,
            DungeonState13PublicationPhase::AtomicCallerReturn
                | DungeonState13PublicationPhase::CallerReturn
        ) || signals.dungeon_faded_filter_phase
            == DungeonFadedFilterPublicationPhase::CallerReturn
        {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        } else if signals.spiral_stair_return_publishes_live_obj_cache {
            GraphicsDmaGeneration::LiveAfterMain
        } else {
            snapshot
                .link_obj_scanout_generation
                .resolve_live_override(signals.dungeon_exit_crosses_nmi_boundary)
        };
        let link_obj_source_generation = if matches!(
            signals.dungeon_state_13_phase,
            DungeonState13PublicationPhase::AtomicCallerReturn
                | DungeonState13PublicationPhase::CallerReturn
        ) || signals.dungeon_faded_filter_phase
            == DungeonFadedFilterPublicationPhase::CallerReturn
        {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        } else if signals.spiral_stair_return_publishes_live_obj_cache {
            GraphicsDmaGeneration::LiveAfterMain
        } else {
            snapshot
                .link_obj_source_generation
                .resolve_live_override(signals.dungeon_exit_crosses_nmi_boundary)
        };
        let atomic_item_graphics_return_publishes_live_vram = matches!(
            snapshot.link_obj_scanout_generation,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        ) && matches!(
            snapshot.link_obj_source_generation,
            GraphicsDmaGeneration::LiveAfterMain
        ) && snapshot.vram_generation
            == DisplayVramGeneration::ComposeLiveAfterNmi;
        Self {
            vram_generation: if atomic_item_graphics_return_publishes_live_vram
                || signals.dungeon_brightness_publishes_live_display
            {
                DisplayVramGeneration::ComposeLiveAfterNmi
            } else {
                snapshot
                    .vram_generation
                    .resolve_for_scanout(signals.retain_previous_nmi_display_memory)
            },
            compose_live_cgram: !signals.retain_previous_nmi_display_memory,
            oam_scanout_source,
            retain_captured_oam: retain_captured_oam_for_scanout(
                &snapshot.obj_generation,
                oam_scanout_source,
            ),
            link_obj_scanout_generation,
            link_obj_source_generation,
            animated_bg_scanout_generation: snapshot
                .animated_bg_scanout_generation
                .resolve_live_override(
                    signals.publish_live_overworld_bad_weather_scroll
                        || signals.dungeon_brightness_publishes_live_animated_bg,
                ),
            bg_scroll_source: if signals.dungeon_item_hold_publishes_live_scroll {
                DisplayedBgScrollSource::LiveAfterNmi
            } else {
                DisplayedBgScrollSource::resolve(
                    snapshot.bg_scroll_generation,
                    signals.dungeon_exit_crosses_nmi_boundary,
                    signals.publish_live_overworld_bad_weather_scroll,
                    signals.attract_map_retains_display_memory,
                )
            },
            world_map_fade_display: signals.world_map_fade_display,
            world_map_mode7_brightness_is_early_published: signals
                .world_map_mode7_brightness_is_early_published,
            publish_live_screen_layers: signals.spiral_stair_landing_publishes_live_display
                || signals.spiral_stair_return_publishes_live_registers
                || signals.dungeon_brightness_publishes_live_display,
            publish_post_main_hud_dma: signals.dungeon_brightness_publishes_live_display,
            publish_live_nmi_copy_packets: signals.dungeon_brightness_publishes_live_display,
            publish_live_dungeon_state_13_palette_and_registers: signals.dungeon_state_13_phase
                == DungeonState13PublicationPhase::PreMainQuadrantNmiEntry,
            dungeon_state_13_phase: signals.dungeon_state_13_phase,
            dungeon_faded_filter_phase: signals.dungeon_faded_filter_phase,
            publish_live_spiral_stair_obj_cache: signals
                .spiral_stair_landing_publishes_live_display
                || signals.spiral_stair_motion_publishes_live_oam
                || signals.spiral_stair_return_publishes_live_obj_cache,
            publish_live_spiral_stair_return_obj_vram: signals
                .spiral_stair_return_publishes_live_obj_cache,
            publish_spiral_stair_return_equipment_handoff: signals
                .spiral_stair_return_publishes_live_obj_cache,
        }
    }
}

/// Atomic transition surface for the dialogue scroll's CPU and scanout phases.
///
/// The referenced fields remain separate only because their positions are part
/// of the existing `ZeldaState` bincode layout. Production code must mutate
/// them through this machine so invalid execution/publication combinations
/// cannot be assembled one field at a time.
pub(crate) struct DialogueScrollMachineMut<'a> {
    pub(crate) continuation: &'a mut DialogueScrollContinuation,
    pub(crate) frozen_scanout: &'a mut Option<DialogueTextScanout>,
    pub(crate) publication: &'a mut DialogueScanoutOwnership,
    pub(crate) completion_scanout: &'a mut Option<DialogueTextScanout>,
    pub(crate) staged_completion: &'a mut Option<DialogueTextScanout>,
}

impl DialogueScrollMachineMut<'_> {
    pub(crate) fn phase(&self) -> DialogueScrollPhase {
        dialogue_scroll_phase(
            *self.continuation,
            *self.publication,
            self.frozen_scanout.is_some(),
            self.completion_scanout.is_some(),
            self.staged_completion.is_some(),
        )
    }

    pub(crate) fn begin_scroll(
        &mut self,
        frozen_scanout: DialogueTextScanout,
        completion_timing: DialogueScrollCompletionTiming,
    ) {
        let entry_phase = self.phase();
        assert!(
            matches!(
                entry_phase,
                DialogueScrollPhase::Idle
                    | DialogueScrollPhase::CompletedScroll
                    | DialogueScrollPhase::RetiredTextDma
            ),
            "dialogue scroll began from invalid phase {entry_phase:?}",
        );
        *self.continuation = DialogueScrollContinuation::begin(completion_timing);
        *self.frozen_scanout = Some(frozen_scanout);
        *self.publication = DialogueScanoutOwnership::FROZEN_SCROLL_COPY;
        *self.completion_scanout = None;
        debug_assert_eq!(
            self.phase(),
            DialogueScrollPhase::CopyingRemainingPixels { completion_timing }
        );
    }

    pub(crate) fn finish_remaining_pixels(
        &mut self,
        authoritative_completion_timing: Option<DialogueScrollCompletionTiming>,
    ) -> DialogueScrollCompletionTiming {
        let DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: planned_completion_timing,
        } = self.phase()
        else {
            panic!("dialogue scroll copy completed outside its copy phase");
        };
        let completion_timing =
            authoritative_completion_timing.unwrap_or(planned_completion_timing);
        self.continuation.finish_remaining_pixels(completion_timing);
        *self.publication = match completion_timing {
            DialogueScrollCompletionTiming::AfterReturnBoundary => {
                DialogueScanoutOwnership::FROZEN_SCROLL_COPY
            }
            DialogueScrollCompletionTiming::BeforeNextVblank => {
                DialogueScanoutOwnership::COMPLETION_PENDING
            }
        };
        completion_timing
    }

    pub(crate) fn finish_return(&mut self) {
        debug_assert_eq!(self.phase(), DialogueScrollPhase::ReturnOnly);
        self.continuation.finish_return();
        *self.publication = DialogueScanoutOwnership::SNAPSHOT;
        *self.frozen_scanout = None;
    }

    pub(crate) fn stage_completion_after_return(&mut self, completed_scanout: DialogueTextScanout) {
        debug_assert_eq!(self.phase(), DialogueScrollPhase::Idle);
        *self.staged_completion = Some(completed_scanout);
        *self.publication = DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_SNAPSHOT;
        debug_assert_eq!(
            self.phase(),
            DialogueScrollPhase::CompletionStagedAfterSnapshot
        );
    }

    pub(crate) fn stage_early_completion(&mut self, completed_scanout: DialogueTextScanout) {
        debug_assert_eq!(
            self.phase(),
            DialogueScrollPhase::CompletionPendingPublication
        );
        self.continuation.publish_early_completion();
        *self.staged_completion = Some(completed_scanout);
        *self.publication = DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_FROZEN;
        debug_assert_eq!(
            self.phase(),
            DialogueScrollPhase::CompletionStagedAfterFrozenScanout
        );
    }

    pub(crate) fn advance_display_boundary(
        &mut self,
        retiring_text_dma: Option<DialogueTextScanout>,
    ) {
        match self.phase() {
            DialogueScrollPhase::CompletionStagedAfterFrozenScanout
            | DialogueScrollPhase::CompletionStagedAfterSnapshot => {
                // A captured field is not proof that NMI_UploadBG3Text ran.
                // Both staging orders remain private until the exact BG3 DMA
                // receipt promotes them through the dedicated transition.
            }
            DialogueScrollPhase::CompletedScroll => {
                *self.completion_scanout = Some(
                    retiring_text_dma
                        .expect("completed dialogue boundary must sample the retiring text DMA"),
                );
                *self.publication = DialogueScanoutOwnership::RETIRED_TEXT_DMA;
                *self.frozen_scanout = None;
            }
            DialogueScrollPhase::RetiredTextDma => {
                *self.completion_scanout = None;
                *self.publication = DialogueScanoutOwnership::SNAPSHOT;
                *self.frozen_scanout = None;
            }
            DialogueScrollPhase::CopyingRemainingPixels { .. }
            | DialogueScrollPhase::ReturnOnly => {
                *self.publication = DialogueScanoutOwnership::FROZEN_SCROLL_COPY;
            }
            DialogueScrollPhase::CompletionPendingPublication => {
                *self.publication = DialogueScanoutOwnership::COMPLETION_PENDING;
            }
            DialogueScrollPhase::Idle => {
                *self.publication = DialogueScanoutOwnership::SNAPSHOT;
                *self.frozen_scanout = None;
            }
        }
    }

    pub(crate) fn complete_staged_text_dma(&mut self) {
        assert!(
            matches!(
                self.phase(),
                DialogueScrollPhase::CompletionStagedAfterFrozenScanout
                    | DialogueScrollPhase::CompletionStagedAfterSnapshot
            ),
            "BG3 text-DMA publication completed outside a staged dialogue phase",
        );
        *self.completion_scanout = self.staged_completion.take();
        *self.publication = DialogueScanoutOwnership::COMPLETED_SCROLL;
        *self.frozen_scanout = None;
        debug_assert_eq!(self.phase(), DialogueScrollPhase::CompletedScroll);
    }

    pub(crate) fn restore_transient_after_checkpoint(
        &mut self,
        completed_scanout: DialogueTextScanout,
    ) {
        if !self.continuation.is_idle()
            || self.completion_scanout.is_some()
            || self.staged_completion.is_some()
        {
            return;
        }
        match *self.publication {
            DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_FROZEN => {
                *self.staged_completion = Some(completed_scanout);
            }
            DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_SNAPSHOT => {
                *self.staged_completion = Some(completed_scanout);
                *self.frozen_scanout = None;
            }
            DialogueScanoutOwnership::COMPLETED_SCROLL => {
                *self.completion_scanout = Some(completed_scanout);
                *self.frozen_scanout = None;
            }
            DialogueScanoutOwnership::RETIRED_TEXT_DMA => {
                *self.completion_scanout = Some(completed_scanout);
                *self.frozen_scanout = None;
            }
            // Compatibility with checkpoints produced before staged
            // publication received its own serialized ownership value.
            DialogueScanoutOwnership::COMPLETION_PENDING => {
                *self.publication = DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_FROZEN;
                *self.staged_completion = Some(completed_scanout);
            }
            DialogueScanoutOwnership::FROZEN_SCROLL_COPY => {
                *self.publication = DialogueScanoutOwnership::SNAPSHOT;
                *self.frozen_scanout = None;
            }
            DialogueScanoutOwnership::SNAPSHOT => {}
            _ => {
                *self.publication = DialogueScanoutOwnership::SNAPSHOT;
                *self.frozen_scanout = None;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ColorMathRegisterScanout {
    pub(crate) windowsel: u32,
    pub(crate) clip_mode: u8,
    pub(crate) prevent_math_mode: u8,
    pub(crate) add_subscreen: bool,
    pub(crate) subtract_color: bool,
    pub(crate) half_color: bool,
    pub(crate) math_enabled: u8,
    pub(crate) fixed_color: [u8; 3],
    pub(crate) screen_enabled: [u8; 2],
    pub(crate) screen_windowed: [u8; 2],
}

impl ColorMathRegisterScanout {
    pub(crate) fn capture(ppu: &PpuState) -> Self {
        Self {
            windowsel: ppu.windowsel,
            clip_mode: ppu.clip_mode,
            prevent_math_mode: ppu.prevent_math_mode,
            add_subscreen: ppu.add_subscreen,
            subtract_color: ppu.subtract_color,
            half_color: ppu.half_color,
            math_enabled: ppu.math_enabled,
            fixed_color: [ppu.fixed_color_r, ppu.fixed_color_g, ppu.fixed_color_b],
            screen_enabled: ppu.screen_enabled,
            screen_windowed: ppu.screen_windowed,
        }
    }

    pub(crate) fn publish_to(self, ppu: &mut PpuState) {
        ppu.windowsel = self.windowsel;
        ppu.clip_mode = self.clip_mode;
        ppu.prevent_math_mode = self.prevent_math_mode;
        ppu.add_subscreen = self.add_subscreen;
        ppu.subtract_color = self.subtract_color;
        ppu.half_color = self.half_color;
        ppu.math_enabled = self.math_enabled;
        ppu.fixed_color_r = self.fixed_color[0];
        ppu.fixed_color_g = self.fixed_color[1];
        ppu.fixed_color_b = self.fixed_color[2];
        ppu.screen_enabled = self.screen_enabled;
        ppu.screen_windowed = self.screen_windowed;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InidispRegisterScanout {
    pub(crate) brightness: u8,
    pub(crate) forced_blank: bool,
}

impl InidispRegisterScanout {
    pub(crate) fn capture(ppu: &PpuState) -> Self {
        Self {
            brightness: ppu.brightness,
            forced_blank: ppu.forced_blank,
        }
    }

    pub(crate) fn publish_to(self, ppu: &mut PpuState) {
        ppu.brightness = self.brightness;
        ppu.forced_blank = self.forced_blank;
        // This receipt owns the value installed by C `WritePpuRegisters`, but
        // cannot rewrite scanlines which elapsed before that write completed.
        // Preserve the captured prefix; suffix metadata belongs to a later
        // active-display INIDISP transition and is superseded by this write.
        ppu.forced_blank_from_scanline = None;
        ppu.retain_active_display_history = false;
        ppu.scanout_brightness_override = None;
        ppu.refresh_brightness_cache();
    }
}

/// Coupled hardware generation installed by C `WritePpuRegisters`.
///
/// This is intentionally one receipt rather than independent predictions for
/// color math, scroll, brightness, mode, mosaic, Mode 7, and BG character
/// bases: the C NMI writes all of those registers in one ordered routine, and
/// a display boundary either observes that completed routine or it does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NmiPpuRegisterScanout {
    pub(crate) color_math: ColorMathRegisterScanout,
    pub(crate) bg_scroll: BgScrollRegisterScanout,
    pub(crate) inidisp: InidispRegisterScanout,
    pub(crate) mode: u8,
    pub(crate) mosaic_enabled: u8,
    pub(crate) mosaic_size: u8,
    pub(crate) mode7_matrix: [i16; 8],
    pub(crate) scroll_prev: u8,
    pub(crate) scroll_prev2: u8,
    pub(crate) m7_prev: u8,
    pub(crate) bg_tile_addresses: [u16; 4],
}

impl NmiPpuRegisterScanout {
    pub(crate) fn capture(ppu: &PpuState) -> Self {
        Self {
            color_math: ColorMathRegisterScanout::capture(ppu),
            bg_scroll: BgScrollRegisterScanout::capture(ppu),
            inidisp: InidispRegisterScanout::capture(ppu),
            mode: ppu.mode,
            mosaic_enabled: ppu.mosaic_enabled,
            mosaic_size: ppu.mosaic_size,
            mode7_matrix: ppu.m7_matrix,
            scroll_prev: ppu.scroll_prev,
            scroll_prev2: ppu.scroll_prev2,
            m7_prev: ppu.m7_prev,
            bg_tile_addresses: std::array::from_fn(|index| ppu.bg_layer[index].tile_adr),
        }
    }

    pub(crate) fn publish_to(self, ppu: &mut PpuState) {
        self.color_math.publish_to(ppu);
        self.bg_scroll.publish_to(ppu);
        self.inidisp.publish_to(ppu);
        ppu.mode = self.mode;
        ppu.mosaic_enabled = self.mosaic_enabled;
        ppu.mosaic_size = self.mosaic_size;
        ppu.m7_matrix = self.mode7_matrix;
        ppu.scroll_prev = self.scroll_prev;
        ppu.scroll_prev2 = self.scroll_prev2;
        ppu.m7_prev = self.m7_prev;
        for (layer, tile_address) in ppu.bg_layer.iter_mut().zip(self.bg_tile_addresses) {
            layer.tile_adr = tile_address;
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InterruptedDungeonSubmodulePublication {
    pub(crate) retiring_pre_spotlight_scanout: Option<LiveSpotlightScanout>,
    pub(crate) oam_scanout_source: OamScanoutSource,
    pub(crate) obj_cache_generation: DisplayObjCacheGeneration,
    pub(crate) ppu_registers: NmiPpuRegisterScanout,
    pub(crate) spotlight: Option<LiveSpotlightScanout>,
    pub(crate) provenance: &'static core::panic::Location<'static>,
}

#[derive(Debug, Clone)]
pub(crate) struct EffectiveDmaWriteSet {
    pub(crate) active_snapshot_accepts_receipt: bool,
    pub(crate) active_snapshot_epoch: Option<u64>,
    pub(crate) completed_ppu_registers_own_active_scanout: bool,
    pub(crate) vram_words: Vec<bool>,
    pub(crate) completed_oam: Option<Vec<u16>>,
    pub(crate) completed_link_obj_dma: Option<CompletedLinkObjDma>,
    pub(crate) completed_cgram: Option<Vec<u16>>,
    pub(crate) completed_ppu_registers: Option<NmiPpuRegisterScanout>,
    pub(crate) completed_dialogue_metadata: Option<PublishedDialogueMetadata>,
}

/// Linear proof that one just-completed NMI published the complete BG3 text
/// generation into one receptive display snapshot. It is deliberately neither
/// cloneable nor persisted; callers must consume it at the matching dialogue
/// staging boundary.
pub(crate) struct DialogueTextDmaPublicationToken {
    pub(crate) snapshot_epoch: u64,
    pub(crate) words: Vec<u16>,
    pub(crate) metadata: PublishedDialogueMetadata,
}

/// Sparse bulk-memory effects completed by an explicit leading NMI. The active
/// image is captured before the transfer so register and memory generations can
/// remain independent, but vblank VRAM writes complete before visible scanout
/// and therefore attach to that active image. This is independent of rooms and
/// modules; OAM, CGRAM, and PPU register latches join the receipt only when
/// those same-boundary hardware writes actually complete.
#[derive(Debug, Clone)]
pub(crate) struct EffectivePresentedDma {
    pub(crate) vram_writes: Vec<(usize, u16)>,
    /// Pattern data completed by an explicit leading NMI whose raw VRAM
    /// ownership remains at the captured host boundary.
    pub(crate) decoded_bg_vram_writes: Vec<(usize, u16)>,
    pub(crate) completed_oam: Option<Vec<u16>>,
    pub(crate) completed_link_obj_dma: Option<CompletedLinkObjDma>,
    pub(crate) completed_cgram: Option<Vec<u16>>,
    pub(crate) completed_ppu_registers: Option<NmiPpuRegisterScanout>,
    pub(crate) completed_dialogue_metadata: Option<PublishedDialogueMetadata>,
}

impl EffectivePresentedDma {
    pub(crate) fn animated_bg_only(decoded_only: bool, writes: Vec<(usize, u16)>) -> Self {
        Self {
            vram_writes: if !decoded_only {
                writes.clone()
            } else {
                Default::default()
            },
            decoded_bg_vram_writes: if decoded_only {
                writes
            } else {
                Default::default()
            },
            completed_oam: None,
            completed_link_obj_dma: None,
            completed_cgram: None,
            completed_ppu_registers: None,
            completed_dialogue_metadata: None,
        }
    }

    pub(crate) fn ppu_registers_only(registers: NmiPpuRegisterScanout) -> Self {
        Self {
            vram_writes: Vec::new(),
            decoded_bg_vram_writes: Vec::new(),
            completed_oam: None,
            completed_link_obj_dma: None,
            completed_cgram: None,
            completed_ppu_registers: Some(registers),
            completed_dialogue_metadata: None,
        }
    }

    pub(crate) fn from_write_set(writes: EffectiveDmaWriteSet, state: &ZeldaState) -> Self {
        Self {
            vram_writes: writes
                .vram_words
                .iter()
                .enumerate()
                .filter_map(|(index, &written)| written.then_some((index, state.ppu.vram[index])))
                .collect(),
            decoded_bg_vram_writes: Vec::new(),
            completed_oam: writes.completed_oam,
            completed_link_obj_dma: writes.completed_link_obj_dma,
            completed_cgram: writes.completed_cgram,
            completed_ppu_registers: writes.completed_ppu_registers,
            completed_dialogue_metadata: writes.completed_dialogue_metadata,
        }
    }

    pub(crate) fn merge_after(&mut self, later: Self) {
        fn merge_writes(current: &mut Vec<(usize, u16)>, later: Vec<(usize, u16)>) {
            for (index, value) in later {
                match current.binary_search_by_key(&index, |&(index, _)| index) {
                    Ok(position) => current[position].1 = value,
                    Err(position) => current.insert(position, (index, value)),
                }
            }
        }

        merge_writes(&mut self.vram_writes, later.vram_writes);
        merge_writes(
            &mut self.decoded_bg_vram_writes,
            later.decoded_bg_vram_writes,
        );
        if later.completed_oam.is_some() {
            self.completed_oam = later.completed_oam;
        }
        if later.completed_link_obj_dma.is_some() {
            self.completed_link_obj_dma = later.completed_link_obj_dma;
        }
        if later.completed_cgram.is_some() {
            self.completed_cgram = later.completed_cgram;
        }
        if later.completed_ppu_registers.is_some() {
            self.completed_ppu_registers = later.completed_ppu_registers;
        }
        if later.completed_dialogue_metadata.is_some() {
            self.completed_dialogue_metadata = later.completed_dialogue_metadata;
        }
    }

    pub(crate) fn take_vram_range(&mut self, range: std::ops::Range<usize>) -> Vec<(usize, u16)> {
        let (selected, retained): (Vec<_>, Vec<_>) = self
            .vram_writes
            .drain(..)
            .partition(|&(index, _)| range.contains(&index));
        self.vram_writes = retained;
        selected
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ClosedOamBoundaryReceipt {
    pub(crate) publication_host_frame: u32,
    pub(crate) active_oam: Vec<u16>,
}

#[derive(Debug, Clone)]
pub(crate) struct DisplaySnapshot {
    pub(crate) publication_epoch: u64,
    pub(crate) ram: Vec<u8>,
    pub(crate) ppu: PpuState,
    pub(crate) dma: DmaState,
    pub(crate) vram_chr_source: crate::chr_source::VramChrSourceTable,
    pub(crate) vram_chr_preview_source: crate::chr_source::VramChrSourceTable,
    pub(crate) hdma_table_generation: DisplayHdmaTableGeneration,
    pub(crate) vram_generation: DisplayVramGeneration,
    pub(crate) hud_vram_generation: DisplayVramGeneration,
    pub(crate) hud_vram_destination: usize,
    pub(crate) cgram_scanout_generation: CgramScanoutGeneration,
    /// Exact main-palette generation present when an interruptible filter
    /// entered. The atomic translation may finish the next generation before
    /// capture, while hardware uploaded this input generation at vblank.
    pub(crate) cgram_scanout_override: Option<Vec<u16>>,
    pub(crate) game_over_iris_goal_scanout_closed: bool,
    pub(crate) link_obj_scanout_generation: GraphicsDmaGeneration,
    pub(crate) link_obj_source_generation: GraphicsDmaGeneration,
    pub(crate) oam_scanout_source: OamScanoutSource,
    /// Exact hardware OAM image DMAed after this display boundary was captured.
    /// `None` means no OAM DMA completed before the boundary was published, so
    /// the PPU-resident OAM captured in `ppu` remains authoritative.
    pub(crate) completed_oam_dma_after_capture: Option<Vec<u16>>,
    /// Exact OAM generation used by the completed scanout returned from the
    /// timing authority's host call. Native composition still runs so its
    /// shadow/history owners advance normally, then this immutable surface
    /// receipt wins at the renderer boundary without mutating live PPU OAM.
    pub(crate) presented_oam_override: Option<Vec<u16>>,
    /// Exact HUD tilemap used by the completed source scanout. Apply this
    /// after native VRAM-generation composition so a later authored HUD
    /// buffer cannot leak backward across the NMI publication boundary.
    pub(crate) presented_hud_tilemap_override: Option<Vec<u16>>,
    /// Exact BG3 dialogue character generation used by the completed source
    /// scanout. Native scroll/retention composition still runs first; this
    /// receipt wins only at the final outgoing renderer boundary.
    pub(crate) presented_dialogue_text_override: Option<crate::PresentedDialogueText>,
    /// Exact BG scroll registers used by the completed source scanout. Native
    /// register composition still runs so the shadow owner advances normally;
    /// this immutable receipt wins only at the outgoing renderer boundary.
    pub(crate) presented_bg_scroll_override: Option<crate::PresentedBgScroll>,
    /// Exact Mode 7 transform used by the completed source scanout. Native
    /// PPU/HDMA composition still runs for shadow comparison; this immutable
    /// receipt wins only at the outgoing renderer boundary.
    pub(crate) presented_mode7_transform_override: Option<crate::PresentedMode7Transform>,
    /// Exact window boundaries used by the completed source scanout. Native
    /// HDMA composition still runs so its shadow owner advances normally;
    /// this immutable receipt wins only at the outgoing renderer boundary.
    pub(crate) presented_window_mask_override: Option<crate::PresentedWindowMask>,
    /// Uniform INIDISP state used by the completed source scanout. Native
    /// brightness/blanking composition still advances first; this immutable
    /// presentation receipt wins only at the outgoing surface boundary.
    pub(crate) presented_inidisp_override: Option<crate::PresentedInidisp>,
    /// Host-surface crop applied to the completed hardware scanout. This is a
    /// presentation-domain receipt, not a mutation of Zelda's live registers.
    pub(crate) presented_scanout_geometry_override: Option<crate::PresentedScanoutGeometry>,
    /// Exact decoded generation of Zelda's animated BG upload used by the
    /// completed source scanout. Raw VRAM may already contain the following
    /// NMI's words, so this remains an independent presentation domain.
    pub(crate) presented_animated_bg_tiles_override: Option<crate::PresentedAnimatedBgTiles>,
    /// Exact decoded address-bearing OBJ tiles used by the completed source
    /// scanout.
    /// This is a partial presentation overlay, not ownership of either full
    /// hardware name page; untouched tiles retain the normally composed OBJ
    /// cache generation.
    pub(crate) presented_obj_tiles_override: Option<crate::PresentedObjTiles>,
    /// Exact resident OAM image owned by this publication when an OAM DMA
    /// completed after its receipt window closed. The receipt is keyed to the
    /// publication epoch so retaining the snapshot cannot replay it later.
    pub(crate) closed_oam_boundary_receipt: Option<ClosedOamBoundaryReceipt>,
    /// Scroll registers written by the completed NMI after this immutable
    /// memory capture. Register mirrors latch for the immediately following
    /// scanout even though bulk display-memory receipts retire separately.
    /// Per-domain memory receipt completed after the preceding active scanout.
    /// It is consumed by this newly published hardware generation.
    pub(crate) effective_presented_dma: Option<EffectivePresentedDma>,
    /// This publication is owned by a held-latch NMI inside a long Module 7
    /// call. A later full-DMA catch-up NMI in the same host call advances live
    /// hardware only and must not attach its receipt to this returned image.
    pub(crate) interrupted_dungeon_submodule_nmi_owns_scanout: bool,
    /// A host snapshot owns exactly one following NMI boundary. Later NMIs
    /// may advance live PPU state but cannot replace this scanout's receipts.
    pub(crate) accepts_nmi_dma_receipts: bool,
    /// Host boundary currently publishing this snapshot. Retained/staged
    /// snapshots can outlive the host frame that originally captured them, so
    /// OAM completion is associated with the publication boundary, not object age.
    pub(crate) publication_host_frame: u32,
    /// Source location of the handler that staged this snapshot's OBJ scanout
    /// generations (`None` when the module-level plan was used). Diagnostic only;
    /// surfaced by the display OAM/OBJ-VRAM probes.
    pub(crate) obj_scanout_provenance: Option<&'static core::panic::Location<'static>>,
    pub(crate) dungeon_item_hold_entry_scanout: bool,
    pub(crate) dungeon_item_hold_entry_bg2_scroll: Option<(u16, u16)>,
    pub(crate) published_shadow_oam_dma: Option<Vec<u16>>,
    pub(crate) animated_bg_scanout_generation: AnimatedBgScanoutGeneration,
    /// Exact animated-tile memory and default renderer provenance resident at
    /// the hardware boundary owned by this snapshot.
    pub(crate) host_boundary_animated_bg_scanout: Option<AnimatedBgScanout>,
    pub(crate) bg_scroll_generation: DisplayBgScrollGeneration,
    pub(crate) spotlight_scanout_generation: SpotlightScanoutGeneration,
    pub(crate) obj_generation: DisplayObjGeneration,
    pub(crate) obj_cache_generation: DisplayObjCacheGeneration,
    /// Exact decoded OBJ page selected by an instruction-timed semantic
    /// continuation. Raw VRAM remains independently owned by completed DMA.
    pub(crate) explicit_obj_cache_vram: Option<Vec<u16>>,
    pub(crate) interrupted_item_receipt_obj_cache: bool,
    pub(crate) enemy_drop_item_graphics_live_extended_oam: bool,
    pub(crate) published_bg3_vwf_glyph_runs: Vec<Bg3VwfGlyphRun>,
    pub(crate) published_bg3_vwf_glyph_run_dialogue_offsets: Vec<u16>,
    pub(crate) published_dialogue_msg_read_pos: u16,
    pub(crate) published_dialogue_message_id: u16,
    pub(crate) intro_poly_upload_delay: u8,
    pub(crate) rom_reset_frame_delay: u8,
    pub(crate) intro_memory_darken_frame_delay: u8,
    pub(crate) nmi_poly_upload_deferred: u8,
    pub(crate) obj_vram_latch_generation: u64,
    pub(crate) room_82_sprite_conversion_deferred_nmi: bool,
    pub(crate) snes9x_poly_scheduler_counter: u8,
}

impl DisplaySnapshot {
    pub(crate) fn effective_obj_cache_generation(&self) -> DisplayObjCacheGeneration {
        if self.obj_cache_generation != DisplayObjCacheGeneration::FollowModuleCadence {
            return self.obj_cache_generation;
        }
        let frame = crate::game_state::FrameState::load_from_ram(&self.ram);
        let landing_wipe_interrupted_publication = frame.main_module == 7
            && frame.submodule == 0x0f
            && frame.subsubmodule == 1
            && matches!(
                self.oam_scanout_source,
                OamScanoutSource::RetainCapturedBeforeNmi
                    | OamScanoutSource::RetainImmutableCapturedPpu
                    | OamScanoutSource::RetainResidentPpuOam
            );
        if !self.interrupted_dungeon_submodule_nmi_owns_scanout
            && !landing_wipe_interrupted_publication
        {
            return DisplayObjCacheGeneration::FollowModuleCadence;
        }
        match self.oam_scanout_source {
            OamScanoutSource::RetainCapturedBeforeNmi => {
                DisplayObjCacheGeneration::CapturedBeforeNmi
            }
            OamScanoutSource::RetainImmutableCapturedPpu => {
                DisplayObjCacheGeneration::HostBoundaryBeforeMain
            }
            OamScanoutSource::RetainResidentPpuOam => {
                DisplayObjCacheGeneration::EarlyLinkDmaFromHostOperands
            }
            _ => DisplayObjCacheGeneration::FollowModuleCadence,
        }
    }

    /// Resolve the coupled spotlight controls/table generation before adding
    /// a scanout-local table projection. `ram` can already contain CPU writes
    /// from a later atomic call, while `spotlight_scanout_generation` owns the
    /// table actually selected for this staged hardware generation.
    pub(crate) fn effective_spotlight_hdma_tables(&self) -> [Vec<u8>; 2] {
        let mut ram = self.ram.clone();
        let mut dma = self.dma.clone();
        self.spotlight_scanout_generation
            .compose_hdma_into(&mut ram, &mut dma);
        self.hdma_table_generation.compose_into(&mut ram);
        spotlight_hdma_tables_from_ram(&ram)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublishedDialogueMetadata {
    pub(crate) glyph_runs: Vec<Bg3VwfGlyphRun>,
    pub(crate) glyph_run_dialogue_offsets: Vec<u16>,
    pub(crate) message_read_position: u16,
    pub(crate) message_id: u16,
}

impl PublishedDialogueMetadata {
    pub(crate) fn from_scanout(scanout: &DialogueTextScanout) -> Self {
        Self {
            glyph_runs: scanout.glyph_runs.clone(),
            glyph_run_dialogue_offsets: scanout.glyph_run_dialogue_offsets.clone(),
            message_read_position: scanout.dialogue_msg_read_pos,
            message_id: scanout.dialogue_message_id,
        }
    }

    pub(crate) fn from_snapshot(snapshot: &DisplaySnapshot) -> Self {
        Self {
            glyph_runs: snapshot.published_bg3_vwf_glyph_runs.clone(),
            glyph_run_dialogue_offsets: snapshot
                .published_bg3_vwf_glyph_run_dialogue_offsets
                .clone(),
            message_read_position: snapshot.published_dialogue_msg_read_pos,
            message_id: snapshot.published_dialogue_message_id,
        }
    }

    pub(crate) fn from_live_state(state: &ZeldaState) -> Self {
        Self {
            glyph_runs: state.published_bg3_vwf_glyph_runs.clone(),
            glyph_run_dialogue_offsets: state.published_bg3_vwf_glyph_run_dialogue_offsets.clone(),
            message_read_position: state.published_dialogue_msg_read_pos,
            message_id: state.published_dialogue_message_id,
        }
    }

    pub(crate) fn replace_in(self, state: &mut ZeldaState) -> Self {
        Self {
            glyph_runs: std::mem::replace(&mut state.published_bg3_vwf_glyph_runs, self.glyph_runs),
            glyph_run_dialogue_offsets: std::mem::replace(
                &mut state.published_bg3_vwf_glyph_run_dialogue_offsets,
                self.glyph_run_dialogue_offsets,
            ),
            message_read_position: std::mem::replace(
                &mut state.published_dialogue_msg_read_pos,
                self.message_read_position,
            ),
            message_id: std::mem::replace(
                &mut state.published_dialogue_message_id,
                self.message_id,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CaptureDisplayDiagnostics {
    pub(crate) attract_timeline: bool,
    pub(crate) frame_boundary: bool,
}

impl CaptureDisplayDiagnostics {
    pub(crate) fn from_env() -> Self {
        Self {
            attract_timeline: crate::debug_env::var_os("ZELDA3_DEBUG_ATTRACT_TIMELINE").is_some(),
            frame_boundary: crate::debug_env::var_os("ZELDA3_DEBUG_FRAME_BOUNDARY").is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DisplayDiagnostics {
    pub(crate) capture: CaptureDisplayDiagnostics,
    pub(crate) display_oam: bool,
    pub(crate) nmi_latch: bool,
    pub(crate) scroll_retain: bool,
}

/// Diagnostic-only alternatives available at a display publication boundary.
///
/// The parity probe requests these explicitly so one replay can evaluate all
/// existing generation hypotheses instead of rebuilding once per hypothesis.
#[derive(Clone, Debug)]
pub struct DebugDisplayPublicationCandidate {
    pub name: &'static str,
    pub oam: Option<Vec<u16>>,
    /// OBJ VRAM words for SNES byte addresses $8000-$87ff.
    pub obj_vram: Option<Vec<u16>>,
}

#[derive(Clone, Debug)]
pub struct DebugDisplayCgramCandidate {
    pub name: &'static str,
    pub cgram: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct DebugDisplayPublicationContext {
    pub render_host_frame: u32,
    pub publication_host_frame: u32,
    pub entry_frame: [u8; 4],
    pub following_frame: [u8; 4],
    pub dungeon_room: u8,
    pub staircase_index: u8,
    pub palette_filter_countdown: u16,
    pub nmi_update_latch: u8,
    pub oam_scanout_source: String,
    pub retain_captured_oam: bool,
    pub link_obj_scanout_generation: String,
    pub link_obj_source_generation: String,
    pub captured_to_host_oam_mismatches: usize,
    pub oam_dma_completed_after_active_scanout: bool,
}

impl DisplayDiagnostics {
    pub(crate) fn from_env() -> Self {
        Self {
            capture: CaptureDisplayDiagnostics::from_env(),
            display_oam: crate::debug_env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some(),
            nmi_latch: crate::debug_env::var_os("ZELDA3_DEBUG_NMI_LATCH").is_some(),
            scroll_retain: crate::debug_env::var_os("ZELDA3_DEBUG_SCROLL_RETAIN").is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PreMainAnimatedTileDma {
    pub(crate) source_address: usize,
    pub(crate) destination_address: usize,
    pub(crate) data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub(crate) struct AnimatedBgScanout {
    pub(crate) destination_address: usize,
    pub(crate) vram: Vec<u16>,
    pub(crate) logical_sources: crate::chr_source::VramChrSourceTable,
    pub(crate) preview_sources: crate::chr_source::VramChrSourceTable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RetainedVramRegion {
    pub(crate) destination: usize,
    pub(crate) words: Vec<u16>,
}

impl RetainedVramRegion {
    pub(crate) fn capture(vram: &[u16], destination: usize, word_count: usize) -> Option<Self> {
        let end = destination.checked_add(word_count)?;
        Some(Self {
            destination,
            words: vram.get(destination..end)?.to_vec(),
        })
    }

    pub(crate) fn publish_to(&self, vram: &mut [u16]) {
        let Some(end) = self.destination.checked_add(self.words.len()) else {
            return;
        };
        if let Some(destination) = vram.get_mut(self.destination..end) {
            destination.copy_from_slice(&self.words);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NmiCopyPacketScanout {
    pub(crate) words: Vec<(usize, u16)>,
}

impl NmiCopyPacketScanout {
    pub(crate) fn capture(vram: &[u16], packet_bytes: &[u8]) -> Self {
        let mut words = Vec::new();
        for packet in nmi::nmi_vram_copy_packets(packet_bytes) {
            let word_count = packet.data.len().div_ceil(2);
            let stride = match packet.direction {
                nmi::NmiVramCopyDirection::Horizontal => 1,
                nmi::NmiVramCopyDirection::Vertical => 32,
            };
            for index in 0..word_count {
                let address = packet.destination + index * stride;
                if let Some(&value) = vram.get(address) {
                    words.push((address, value));
                }
            }
        }
        Self { words }
    }

    pub(crate) fn publish_to(&self, vram: &mut [u16]) {
        for &(address, value) in &self.words {
            if let Some(word) = vram.get_mut(address) {
                *word = value;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PreMainGraphicsDma {
    pub(crate) entry_frame: crate::game_state::FrameState,
    pub(crate) entry_plan: GraphicsDmaPlan,
    pub(crate) entry_link_handler_state: u8,
    pub(crate) animated_tile: Option<PreMainAnimatedTileDma>,
    pub(crate) link_operands: PreMainLinkDmaOperands,
    /// Complete PPU VRAM resident at the host boundary before main. OBJ cache
    /// ownership is not limited to Link's first 64 tiles: both OBJ name pages
    /// can remain decoded across a later NMI/main generation.
    pub(crate) obj_vram: Vec<u16>,
    pub(crate) oam_shadow: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PreMainLinkDmaOperands {
    pub(crate) sources: LinkDmaSources,
    pub(crate) link_pack: u16,
    pub(crate) expanded_high_planes: [u8; LINK_DMA_EXPANDED_HIGH_PLANES_LEN],
}

impl PreMainLinkDmaOperands {
    pub(crate) fn capture(ram: &[u8]) -> Self {
        let mut expanded_high_planes = [0; LINK_DMA_EXPANDED_HIGH_PLANES_LEN];
        let end = LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_LEN;
        if let Some(bytes) = ram.get(LINK_DMA_EXPANDED_HIGH_PLANES_START..end) {
            expanded_high_planes.copy_from_slice(bytes);
        }
        Self {
            sources: LinkDmaSources::load_from_ram(ram),
            link_pack: read_le_u16(ram, LINK_DMA_GRAPHICS_INDEX) >> 1,
            expanded_high_planes,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bg3VwfGlyphRun {
    pub glyph_code: u16,
    pub origin_tile_number: u16,
    pub x: i16,
    pub y: i16,
    pub width: u8,
}
