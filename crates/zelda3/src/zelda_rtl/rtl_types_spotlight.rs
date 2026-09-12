//! Types split out of `zelda_rtl.rs` by family (types_spotlight).
//! Mechanical move: definitions are unchanged; items and struct fields
//! became `pub(crate)` so the parent module and its siblings see them.

use super::*;

/// The IrisSpotlight_ConfigureTable interruption classification shared by the
/// overworld and dungeon-exit spotlight CPU plans (`$00:F361..F3C4` table
/// build, `$00:F3B7..F3C4` copy, `$00:F377` return).
macro_rules! spotlight_table_interruption_methods {
    () => {
        pub(crate) const fn interrupted_during_table_build_or_copy(self) -> bool {
            (self.interrupted_pc >= 0x00_f361 && self.interrupted_pc <= 0x00_f3c4)
                || self.interrupted_return_address == 0x00_f377
        }

        pub(crate) const fn interrupted_during_table_copy(self) -> bool {
            self.interrupted_pc >= 0x00_f3b7 && self.interrupted_pc <= 0x00_f3c4
        }

        pub(crate) fn normalized_interruption_phase(mut self) -> Self {
            if self.interrupted_during_table_copy() {
                self.interrupted_pc = 0x00_f3b7;
                self.interrupted_return_address = 0;
            } else if self.interrupted_during_table_build_or_copy() {
                self.interrupted_pc = 0x00_f361;
                self.interrupted_return_address = 0;
            } else {
                self.interrupted_pc = 0;
                self.interrupted_return_address = 0;
            }
            self
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverworldSpotlightCpuPlan {
    pub(crate) interrupted_pc: u32,
    pub(crate) interrupted_return_address: u32,
    pub(crate) iterations_before_nmi: usize,
    pub(crate) nmis_before_module_exit: Option<u8>,
    pub(crate) active_window_words: [u16; SPOTLIGHT_VISIBLE_SCANLINES],
    pub(crate) following_window_words: [u16; SPOTLIGHT_VISIBLE_SCANLINES],
    pub(crate) next_entry_earliest: Option<CpuRasterPosition>,
    pub(crate) next_entry_latest: Option<CpuRasterPosition>,
}

impl OverworldSpotlightCpuPlan {
    pub(crate) fn current_boundary_key(
        self,
    ) -> (
        u32,
        u32,
        usize,
        Option<u8>,
        [u16; SPOTLIGHT_VISIBLE_SCANLINES],
        [u16; SPOTLIGHT_VISIBLE_SCANLINES],
    ) {
        (
            self.interrupted_pc,
            self.interrupted_return_address,
            self.iterations_before_nmi,
            self.nmis_before_module_exit,
            self.active_window_words,
            self.following_window_words,
        )
    }

    spotlight_table_interruption_methods!();

    pub(crate) const fn exits_module_before_next_nmi(self) -> bool {
        matches!(self.nmis_before_module_exit, Some(1))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DungeonExitSpotlightCpuPlan {
    pub(crate) interrupted_pc: u32,
    pub(crate) interrupted_return_address: u32,
    pub(crate) iterations_before_nmi: usize,
    pub(crate) link_position_integrated_before_first_nmi: bool,
    pub(crate) returned_to_main_wait_before_first_nmi: bool,
    pub(crate) main_loop_sprite_preparation_completed_before_second_nmi: bool,
    pub(crate) active_window_words: [u16; SPOTLIGHT_VISIBLE_SCANLINES],
    pub(crate) following_window_words: [u16; SPOTLIGHT_VISIBLE_SCANLINES],
    pub(crate) next_entry_earliest: Option<CpuRasterPosition>,
    pub(crate) next_entry_latest: Option<CpuRasterPosition>,
    pub(crate) successor_entry_earliest: Option<CpuRasterPosition>,
    pub(crate) successor_entry_latest: Option<CpuRasterPosition>,
}

impl DungeonExitSpotlightCpuPlan {
    pub(crate) fn current_boundary_key(
        self,
    ) -> (
        u32,
        u32,
        usize,
        bool,
        bool,
        bool,
        [u16; SPOTLIGHT_VISIBLE_SCANLINES],
        [u16; SPOTLIGHT_VISIBLE_SCANLINES],
    ) {
        (
            self.interrupted_pc,
            self.interrupted_return_address,
            self.iterations_before_nmi,
            self.link_position_integrated_before_first_nmi,
            self.returned_to_main_wait_before_first_nmi,
            self.main_loop_sprite_preparation_completed_before_second_nmi,
            self.active_window_words,
            self.following_window_words,
        )
    }

    spotlight_table_interruption_methods!();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DesertPrayerIrisCaller {
    InitializeCase2,
    PaletteFilterCase3,
    RecurringCase4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpotlightIterationPhase {
    /// The entry table is still being calculated when the next scanout starts.
    CloseEntryBeforeTablePublication,
    /// The entry table reaches HDMA before the next scanout starts.
    CloseEntryAfterTablePublication,
    /// HDMA consumes one complete table generation for the scanout.
    WholeTable,
    /// The circle calculation finishes early enough for HDMA to consume the
    /// completed table before the remaining display domains publish.
    WholeTableAfterTablePublication,
    /// The close stages a published prefix with newly authored final lines.
    MixedTailAfterReturn,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpotlightTableBuildContinuation {
    pub(crate) vertical_center: u16,
    pub(crate) upper_cursor: u16,
    pub(crate) lower_cursor: u16,
    pub(crate) completed: bool,
    pub(crate) pending_circle_input: Option<u8>,
    pub(crate) pending_lower_value: Option<u16>,
    pub(crate) pending_loop_completion_test: bool,
    pub(crate) pending_lower_cursor_decrement: bool,
    pub(crate) projection_tail_cleared: bool,
    pub(crate) projection_words_copied: u16,
    /// Exact source statement from which this native continuation was built.
    /// A following NMI may publish the same checkpoint again before the
    /// interrupted caller resumes; retaining it here lets that acceptance
    /// prove it still owns this continuation instead of treating the receipt
    /// as an unqualified no-op.
    pub(crate) source_progress: Option<SpotlightTableBuildProgress>,
}

impl SpotlightTableBuildContinuation {
    pub(crate) fn assert_recheckpointed_at(self, progress: SpotlightTableBuildProgress) {
        assert_eq!(
            self.source_progress,
            Some(progress),
            "an accepting NMI re-checkpointed a different spotlight table continuation",
        );
    }

    /// Prove that a same-host accepting NMI exposed this saved continuation
    /// again at the same or a later source statement.
    pub(crate) fn assert_recheckpoint_not_behind(self, progress: SpotlightTableBuildProgress) {
        let Some(source) = self.source_progress else {
            panic!("an accepting NMI cannot re-checkpoint an estimated spotlight continuation");
        };
        if source == progress {
            return;
        }
        if let (
            crate::SpotlightTableBuildCheckpoint::ProjectionCopy {
                copied_words: source_words,
            },
            crate::SpotlightTableBuildCheckpoint::ProjectionCopy {
                copied_words: accepted_words,
            },
        ) = (source.checkpoint, progress.checkpoint)
        {
            assert_eq!(
                source.completed_iterations, progress.completed_iterations,
                "an accepting NMI changed the completed spotlight row count during projection",
            );
            assert!(
                accepted_words >= source_words,
                "an accepting NMI moved a spotlight projection cursor backwards",
            );
            return;
        }
        let statement_order = |checkpoint| match checkpoint {
            crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization => 0,
            crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation { .. } => 1,
            crate::SpotlightTableBuildCheckpoint::AfterUpperTableWrite { .. } => 2,
            crate::SpotlightTableBuildCheckpoint::BeforeLowerTableWrite { .. } => 3,
            crate::SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest { .. } => 4,
            crate::SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement { .. } => 5,
            crate::SpotlightTableBuildCheckpoint::ProjectionCopy { .. } => 6,
        };
        assert!(
            progress.completed_iterations > source.completed_iterations
                || (progress.completed_iterations == source.completed_iterations
                    && statement_order(progress.checkpoint) > statement_order(source.checkpoint)),
            "an accepting NMI moved a spotlight continuation backwards: source={source:?} accepted={progress:?}",
        );
        match (source.checkpoint, progress.checkpoint) {
            (crate::SpotlightTableBuildCheckpoint::ProjectionCopy { .. }, non_projection)
                if !matches!(
                    non_projection,
                    crate::SpotlightTableBuildCheckpoint::ProjectionCopy { .. }
                ) =>
            {
                panic!(
                    "an accepting NMI moved a completed spotlight projection back into its row loop",
                )
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverworldSpotlightBuildPhase {
    Entry,
    Recurring,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpotlightDirection {
    Opening,
    Closing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpotlightFollowingFieldPublication {
    /// The active ROM field is already the retiring published generation, so
    /// the completion capture owns the immediately following field.
    WithCompletionCapture,
    /// An older exact receipt still occupies the retiring generation. The
    /// completion capture therefore owns this iteration's active field and
    /// its following field belongs to the next publication boundary.
    AfterCompletionCapture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpotlightFollowingFieldReceipt {
    pub(crate) words: [u16; SPOTLIGHT_VISIBLE_SCANLINES],
    pub(crate) publication: SpotlightFollowingFieldPublication,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpotlightIteration {
    pub(crate) direction: SpotlightDirection,
    pub(crate) phase: SpotlightIterationPhase,
    pub(crate) in_flight_publication: DisplaySnapshotPublication,
    pub(crate) completion_publication: DisplaySnapshotPublication,
    pub(crate) projects_tail_on_completion: bool,
    pub(crate) projection_uses_published_prefix: bool,
    pub(crate) rom_following_field_receipt: Option<SpotlightFollowingFieldReceipt>,
    pub(crate) main_loop_sprite_preparation_before_second_nmi: bool,
    pub(crate) completed_hdma_table_owns_active_scanout: bool,
}

impl SpotlightIteration {
    pub(crate) const fn opening() -> Self {
        Self {
            direction: SpotlightDirection::Opening,
            phase: SpotlightIterationPhase::WholeTable,
            in_flight_publication: DisplaySnapshotPublication::AdvanceStaged,
            completion_publication: DisplaySnapshotPublication::AdvanceStaged,
            projects_tail_on_completion: false,
            projection_uses_published_prefix: false,
            rom_following_field_receipt: None,
            main_loop_sprite_preparation_before_second_nmi: false,
            completed_hdma_table_owns_active_scanout: false,
        }
    }

    pub(crate) fn opening_from_rom_cpu_plan(plan: Option<OverworldSpotlightCpuPlan>) -> Self {
        let mut iteration = Self::opening();
        if plan.is_some_and(OverworldSpotlightCpuPlan::exits_module_before_next_nmi) {
            // On the opening goal return, the ROM reaches
            // IrisSpotlight_ResetTable and OpenSpotlight_Next2 in vblank
            // before the following active field. Only that completed HDMA
            // table owns the next scanout directly; VRAM, OAM, and the other
            // display domains retain their ordinary staged cadence.
            iteration.completed_hdma_table_owns_active_scanout = true;
        }
        iteration
    }

    pub(crate) const fn closing(phase: SpotlightIterationPhase) -> Self {
        Self {
            direction: SpotlightDirection::Closing,
            phase,
            in_flight_publication: DisplaySnapshotPublication::AdvanceStaged,
            completion_publication: phase.close_completion_publication(),
            projects_tail_on_completion: matches!(
                phase,
                SpotlightIterationPhase::WholeTable
                    | SpotlightIterationPhase::WholeTableAfterTablePublication
                    | SpotlightIterationPhase::MixedTailAfterReturn
            ),
            projection_uses_published_prefix: matches!(
                phase,
                SpotlightIterationPhase::MixedTailAfterReturn
            ),
            rom_following_field_receipt: None,
            main_loop_sprite_preparation_before_second_nmi: false,
            completed_hdma_table_owns_active_scanout: false,
        }
    }

    pub(crate) const fn with_rom_following_field_receipt(
        mut self,
        words: [u16; SPOTLIGHT_VISIBLE_SCANLINES],
        publication: SpotlightFollowingFieldPublication,
    ) -> Self {
        // The isolated CPU/HDMA run supplies exact channel-7 rows for this
        // iteration, so completion must not synthesize a geometry tail or
        // retain the preceding whole-table approximation.
        self.completion_publication = DisplaySnapshotPublication::PublishCaptured;
        self.projects_tail_on_completion = false;
        self.projection_uses_published_prefix = false;
        self.rom_following_field_receipt =
            Some(SpotlightFollowingFieldReceipt { words, publication });
        self
    }

    pub(crate) const fn with_rom_following_field_after_staged_active(
        mut self,
        words: [u16; SPOTLIGHT_VISIBLE_SCANLINES],
    ) -> Self {
        // The first ROM field was staged when the table-build continuation
        // began. Completion therefore retains that active field and queues
        // the measured following rows for the next publication boundary.
        self.projects_tail_on_completion = false;
        self.projection_uses_published_prefix = false;
        self.rom_following_field_receipt = Some(SpotlightFollowingFieldReceipt {
            words,
            publication: SpotlightFollowingFieldPublication::AfterCompletionCapture,
        });
        self
    }

    pub(crate) const fn after_rom_following_field_was_staged(mut self) -> Self {
        self.completion_publication = DisplaySnapshotPublication::PublishCaptured;
        self.rom_following_field_receipt = None;
        self.projects_tail_on_completion = false;
        self.projection_uses_published_prefix = false;
        self
    }

    pub(crate) const fn with_main_loop_sprite_preparation_before_second_nmi(mut self) -> Self {
        // ZeldaRunGameLoop calls NMI_PrepareSprites after Module_MainRouting.
        // When the isolated ROM run reaches that suffix between this
        // iteration's two vblank boundaries, keep the fact on the suspended
        // call itself instead of inferring it later from a module number.
        self.main_loop_sprite_preparation_before_second_nmi = true;
        self
    }

    /// The Game Over close begins after the active scanout has already latched
    /// the pre-iris controls. Retain that image while staging the first circle;
    /// later iterations use the ordinary one-generation-deferred pipeline.
    pub(crate) const fn game_over_closing(
        phase: SpotlightIterationPhase,
        enters_iris: bool,
    ) -> Self {
        Self {
            direction: SpotlightDirection::Closing,
            phase,
            in_flight_publication: if enters_iris {
                DisplaySnapshotPublication::RetainPublished
            } else {
                DisplaySnapshotPublication::AdvanceStaged
            },
            completion_publication: DisplaySnapshotPublication::AdvanceStaged,
            projects_tail_on_completion: !enters_iris
                && matches!(
                    phase,
                    SpotlightIterationPhase::WholeTable
                        | SpotlightIterationPhase::MixedTailAfterReturn
                ),
            projection_uses_published_prefix: !enters_iris,
            rom_following_field_receipt: None,
            main_loop_sprite_preparation_before_second_nmi: false,
            completed_hdma_table_owns_active_scanout: false,
        }
    }

    pub(crate) const fn after_game_over_build(self) -> Self {
        Self {
            projects_tail_on_completion: false,
            ..self
        }
    }

    pub(crate) const fn is_closing(self) -> bool {
        matches!(self.direction, SpotlightDirection::Closing)
    }

    pub(crate) const fn prepares_main_loop_sprites_before_second_nmi(self) -> bool {
        self.main_loop_sprite_preparation_before_second_nmi
    }

    pub(crate) const fn in_flight_publication(self) -> DisplaySnapshotPublication {
        self.in_flight_publication
    }

    pub(crate) const fn completion_publication(self) -> DisplaySnapshotPublication {
        self.completion_publication
    }

    pub(crate) const fn publishes_completed_hdma_table_to_active_scanout(self) -> bool {
        matches!(
            (self.direction, self.phase),
            (
                SpotlightDirection::Closing,
                SpotlightIterationPhase::WholeTable
                    | SpotlightIterationPhase::WholeTableAfterTablePublication
            )
        )
    }

    pub(crate) const fn completed_hdma_table_owns_active_scanout(self) -> bool {
        self.completed_hdma_table_owns_active_scanout
    }

    pub(crate) const fn projects_following_table_tail_on_completion(self) -> bool {
        self.projects_tail_on_completion
    }

    pub(crate) const fn projection_uses_published_prefix(self) -> bool {
        self.projection_uses_published_prefix
    }

    pub(crate) const fn rom_following_field_receipt(
        self,
    ) -> Option<SpotlightFollowingFieldReceipt> {
        self.rom_following_field_receipt
    }

    pub(crate) const fn game_over_build_completion_publication(self) -> DisplaySnapshotPublication {
        if matches!(
            self.phase,
            SpotlightIterationPhase::WholeTableAfterTablePublication
        ) {
            DisplaySnapshotPublication::PublishCaptured
        } else {
            DisplaySnapshotPublication::AdvanceStaged
        }
    }

    pub(crate) const fn game_over_build_needs_deferred_caller_return(self) -> bool {
        !matches!(
            self.phase,
            SpotlightIterationPhase::WholeTableAfterTablePublication
        )
    }
}

impl SpotlightIterationPhase {
    pub(crate) const fn is_close_entry(self) -> bool {
        matches!(
            self,
            Self::CloseEntryBeforeTablePublication | Self::CloseEntryAfterTablePublication
        )
    }

    pub(crate) const fn for_game_over_close_iteration(radius: u16) -> Self {
        // The Game Over palette/copy prefix pushes the first two recurring
        // circle builds across HDMA even at this shorter vertical center.
        // Once the input radius reaches $69, the smaller table finishes before
        // active scanout. Trace witnesses: $77->$70 and $70->$69 cross at
        // V=221; $69->$62 publishes the whole completed table.
        if radius > 0x69 {
            Self::MixedTailAfterReturn
        } else {
            Self::WholeTableAfterTablePublication
        }
    }

    pub(crate) const fn for_close_iteration(
        submodule: u8,
        radius: u16,
        vertical_center: u16,
    ) -> Self {
        if submodule == 0 {
            if spotlight_table_has_long_nmi_workload(vertical_center) {
                Self::CloseEntryBeforeTablePublication
            } else {
                Self::CloseEntryAfterTablePublication
            }
        } else if !spotlight_table_has_long_nmi_workload(vertical_center) {
            Self::WholeTableAfterTablePublication
        } else if radius != 0
            && spotlight_close_next_radius(radius) <= SPOTLIGHT_CLOSE_RADIUS_UPDATE_BEFORE_NMI_MAX
        {
            // Snes9x PC/V-counter traces show the next circle write reaching
            // HDMA at scanline 221 once the close has reached this CPU phase.
            Self::MixedTailAfterReturn
        } else {
            Self::WholeTable
        }
    }

    pub(crate) const fn close_completion_publication(self) -> DisplaySnapshotPublication {
        match self {
            Self::CloseEntryAfterTablePublication => DisplaySnapshotPublication::PublishCaptured,
            Self::WholeTable | Self::WholeTableAfterTablePublication => {
                DisplaySnapshotPublication::RetainPublished
            }
            Self::CloseEntryBeforeTablePublication | Self::MixedTailAfterReturn => {
                DisplaySnapshotPublication::AdvanceStaged
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LiveSpotlightScanout {
    pub(crate) windowsel: u32,
    pub(crate) screen_windowed: [u8; 2],
    pub(crate) hdma_enable_mask: u8,
    pub(crate) dma_channels: [DmaChannel; 2],
    pub(crate) hdma_tables: [Vec<u8>; 2],
    pub(crate) authoritative_rom_hdma_receipt: bool,
}

impl LiveSpotlightScanout {
    pub(crate) fn capture(state: &ZeldaState) -> Self {
        let windowsel = u32::from(state.ram[crate::game_state::constants::W12SEL_COPY])
            | (u32::from(state.ram[crate::game_state::constants::W34SEL_COPY]) << 8)
            | (u32::from(state.ram[crate::game_state::constants::WOBJSEL_COPY]) << 16);
        Self {
            windowsel,
            screen_windowed: [
                state.ram[crate::game_state::constants::TMW_COPY],
                state.ram[crate::game_state::constants::TSW_COPY],
            ],
            hdma_enable_mask: state.ram[crate::game_state::constants::HDMAEN_COPY],
            dma_channels: [state.dma.channel[6], state.dma.channel[7]],
            hdma_tables: spotlight_hdma_tables_from_ram(&state.ram),
            authoritative_rom_hdma_receipt: false,
        }
    }

    pub(crate) fn with_active_window_words(
        mut self,
        words: &[u16; SPOTLIGHT_VISIBLE_SCANLINES],
    ) -> Self {
        let table = &mut self.hdma_tables[0];
        for (scanline, word) in words.iter().copied().enumerate() {
            let offset = scanline * 2;
            table[offset..offset + 2].copy_from_slice(&word.to_le_bytes());
        }
        self
    }

    pub(crate) fn with_authoritative_rom_hdma_words(
        mut self,
        words: &[u16; SPOTLIGHT_VISIBLE_SCANLINES],
    ) -> Self {
        self = self.with_active_window_words(words);
        self.authoritative_rom_hdma_receipt = true;
        self
    }

    pub(crate) fn compose_hdma_into(&self, ram: &mut [u8], dma: &mut DmaState) {
        ram[crate::game_state::constants::HDMAEN_COPY] = self.hdma_enable_mask;
        dma.channel[6..8].copy_from_slice(&self.dma_channels);
        for (table_base, table) in [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
            .into_iter()
            .zip(&self.hdma_tables)
        {
            let byte_count = table.len().min(ZeldaState::HDMA_DYNAMIC_TABLE_LEN);
            ram[table_base..table_base + byte_count].copy_from_slice(&table[..byte_count]);
        }
    }

    pub(crate) fn compose_into(&self, ram: &mut [u8], ppu: &mut PpuState, dma: &mut DmaState) {
        ppu.windowsel = self.windowsel;
        ppu.screen_windowed = self.screen_windowed;
        self.compose_hdma_into(ram, dma);
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SpotlightScanoutGeneration {
    CapturedBeforeNmi,
    ComposeLiveAfterNmi(LiveSpotlightScanout),
}

impl SpotlightScanoutGeneration {
    pub(crate) fn compose_hdma_into(&self, ram: &mut [u8], dma: &mut DmaState) {
        if let Self::ComposeLiveAfterNmi(live) = self {
            live.compose_hdma_into(ram, dma);
        }
    }

    pub(crate) fn compose_into(&self, ram: &mut [u8], ppu: &mut PpuState, dma: &mut DmaState) {
        if let Self::ComposeLiveAfterNmi(live) = self {
            live.compose_into(ram, ppu, dma);
        }
    }
}
