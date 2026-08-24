//! Generic S-CPU master-cycle timeline and hardware bus events.
//!
//! The absolute clock records physical elapsed master cycles. Non-interlaced
//! odd fields therefore contain a 1,360-cycle scanline 240 and are four master
//! cycles shorter than long fields; no synthetic time is added at that edge.

pub const MASTER_CYCLES_PER_SCANLINE: u32 = 1_364;
pub const NTSC_SCANLINES_PER_FIELD: u32 = 262;
pub const NMI_SCANLINE: u32 = 225;
pub const HDMA_INIT_CYCLE: u32 = 20;
// Despite its name, pinned Snes9x's `M1SNES = { 1, 3, 2 }`: `_5A22 == 2`
// selects the v2 WRAM refresh schedule. Reset starts at H=538, then each new
// scanline toggles H=534/H=538 except entry to odd non-interlace V=240.
pub const SNES9X_WRAM_REFRESH_V2_EARLY_CYCLE: u32 = 534;
pub const SNES9X_WRAM_REFRESH_V2_LATE_CYCLE: u32 = 538;
pub const WRAM_REFRESH_STALL_MASTER_CYCLES: u32 = 40;
pub const HDMA_START_CYCLE: u32 = 1_106;
pub const SHORT_SCANLINE_END_CYCLE: u32 = 1_360;
pub const NTSC_FIELD_MASTER_CYCLES: u64 =
    NTSC_SCANLINES_PER_FIELD as u64 * MASTER_CYCLES_PER_SCANLINE as u64;
pub const NTSC_SHORT_FIELD_MASTER_CYCLES: u64 = NTSC_FIELD_MASTER_CYCLES - 4;
// Pinned Snes9x 1.63 schedules an enabled VBlank NMI for 12 master cycles
// after VBlank publication. CPU instructions remain atomic, so actual
// acceptance can occur later than this earliest boundary.
pub const SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES: u64 = 12;
// Pinned Snes9x 1.63 retimes an NMI which remains pending when general DMA
// completes to `CPU.Cycles + Timings.NMIDMADelay` (dma.cpp). The pinned timing
// model sets NMIDMADelay to 24 master cycles (memmap.cpp).
pub const SNES9X_NMI_GENERAL_DMA_DELAY_MASTER_CYCLES: u64 = 24;

/// A 65816 position within an NTSC field, expressed in S-CPU master cycles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuRasterPosition {
    scanline: u16,
    master_cycle: u16,
}

impl CpuRasterPosition {
    pub const fn new(scanline: u16, master_cycle: u16) -> Self {
        Self {
            scanline,
            master_cycle,
        }
    }

    const fn nominal_field_master_cycles(self) -> u64 {
        self.scanline as u64 * MASTER_CYCLES_PER_SCANLINE as u64 + self.master_cycle as u64
    }

    pub const fn coordinates(self) -> (u16, u16) {
        (self.scanline, self.master_cycle)
    }
}

/// Bus work which consumes CPU time on the master-cycle timeline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CpuBusWorkload {
    hdma_stall_master_cycles: u16,
    dynamic_hdma: bool,
}

impl CpuBusWorkload {
    pub const fn with_hdma_stall(hdma_stall_master_cycles: u16) -> Self {
        Self {
            hdma_stall_master_cycles,
            dynamic_hdma: false,
        }
    }

    pub const fn with_dynamic_hdma() -> Self {
        Self {
            hdma_stall_master_cycles: 0,
            dynamic_hdma: true,
        }
    }

    pub const fn hdma_stall_master_cycles(self) -> u16 {
        self.hdma_stall_master_cycles
    }

    pub const fn dynamic_hdma(self) -> bool {
        self.dynamic_hdma
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuBusEvent {
    WramRefresh,
    HdmaInit,
    HdmaStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuTimelineEvent {
    Bus(CpuBusEvent),
    ShortScanline,
}

/// One physical-clock event observed by the opt-in synchronous CPU bus.
///
/// This is separate from [`CpuTimelineEvent`] because the legacy scheduler
/// deliberately treats HMax as a coordinate rollover rather than a semantic
/// callback. Synchronous device access must observe that rollover so the APU
/// reference clock can be updated before the next bus semantic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuSynchronousTimelineEvent {
    Bus(CpuBusEvent),
    HMax {
        completed_field_index: u64,
        completed_scanline: u16,
        line_master_cycles: u16,
        /// Exact physical HMax position. The callback's separate timestamp is
        /// the fully charged transaction endpoint and may be later because
        /// pinned Snes9x drains events only after an atomic AddCycles charge.
        event_timestamp: CpuMasterTimestamp,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CpuSynchronousTimelineStartError {
    #[error("CPU timeline was already advanced by the legacy scheduler")]
    LegacyTimelineAlreadyClaimed,
    #[error("CPU timeline checkpoint at {timestamp:?} has ambiguous {event:?} ownership")]
    AmbiguousEventState {
        event: CpuBusEvent,
        timestamp: CpuMasterTimestamp,
    },
}

/// A single physical S-CPU master-clock timestamp observed before bus
/// semantics execute.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuMasterTimestamp(u64);

impl CpuMasterTimestamp {
    pub const fn new(master_cycles: u64) -> Self {
        Self(master_cycles)
    }

    pub const fn master_cycles(self) -> u64 {
        self.0
    }
}

/// Video-field state which changes the CPU's available master-cycle budget.
/// In non-interlace mode, scanline 240 of each odd field is one dot shorter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuFieldTiming {
    odd_field: bool,
    interlace: bool,
}

impl CpuFieldTiming {
    pub const NON_INTERLACE_EVEN: Self = Self {
        odd_field: false,
        interlace: false,
    };

    pub const fn non_interlace(odd_field: bool) -> Self {
        Self {
            odd_field,
            interlace: false,
        }
    }

    pub const fn field_is_odd(self, field_index: u64) -> bool {
        self.odd_field ^ (field_index & 1 != 0)
    }

    pub const fn field_master_cycles(self, field_index: u64) -> u64 {
        if !self.interlace && self.field_is_odd(field_index) {
            NTSC_SHORT_FIELD_MASTER_CYCLES
        } else {
            NTSC_FIELD_MASTER_CYCLES
        }
    }

    /// Physical master-clock timestamp of the selected field's H=0,V=0.
    pub const fn field_start_master_cycles(self, field_index: u64) -> u64 {
        if self.interlace {
            return field_index * NTSC_FIELD_MASTER_CYCLES;
        }
        let complete_pairs = field_index / 2;
        let mut clock =
            complete_pairs * (NTSC_FIELD_MASTER_CYCLES + NTSC_SHORT_FIELD_MASTER_CYCLES);
        if field_index & 1 != 0 {
            clock += self.field_master_cycles(field_index - 1);
        }
        clock
    }

    /// Physical timestamp for a raster coordinate in the selected field.
    pub const fn master_cycles_at(self, field_index: u64, raster: CpuRasterPosition) -> u64 {
        let mut field_cycle = raster.nominal_field_master_cycles();
        if !self.interlace
            && self.field_is_odd(field_index)
            && (raster.scanline > 240
                || (raster.scanline == 240
                    && raster.master_cycle as u32 >= SHORT_SCANLINE_END_CYCLE))
        {
            field_cycle -= 4;
        }
        self.field_start_master_cycles(field_index) + field_cycle
    }

    const fn field_and_cycle_at(self, clock_master_cycles: u64) -> (u64, u64) {
        if self.interlace {
            return (
                clock_master_cycles / NTSC_FIELD_MASTER_CYCLES,
                clock_master_cycles % NTSC_FIELD_MASTER_CYCLES,
            );
        }

        let pair_master_cycles = NTSC_FIELD_MASTER_CYCLES + NTSC_SHORT_FIELD_MASTER_CYCLES;
        let complete_pairs = clock_master_cycles / pair_master_cycles;
        let within_pair = clock_master_cycles % pair_master_cycles;
        let first_field = complete_pairs * 2;
        let first_field_master_cycles = self.field_master_cycles(first_field);
        if within_pair < first_field_master_cycles {
            (first_field, within_pair)
        } else {
            (first_field + 1, within_pair - first_field_master_cycles)
        }
    }

    const fn raster_at(self, field_index: u64, physical_field_cycle: u64) -> CpuRasterPosition {
        let short_line_end =
            240 * MASTER_CYCLES_PER_SCANLINE as u64 + SHORT_SCANLINE_END_CYCLE as u64;
        let nominal_field_cycle = if !self.interlace
            && self.field_is_odd(field_index)
            && physical_field_cycle >= short_line_end
        {
            physical_field_cycle + 4
        } else {
            physical_field_cycle
        };
        CpuRasterPosition::new(
            (nominal_field_cycle / MASTER_CYCLES_PER_SCANLINE as u64) as u16,
            (nominal_field_cycle % MASTER_CYCLES_PER_SCANLINE as u64) as u16,
        )
    }
}

/// Pinned Snes9x M1SNES (`_5A22 == 2`) WRAM-refresh position for one line.
///
/// This preserves the carried v2 toggle phase from reset's initial H=538.
/// `CpuFieldTiming` makes the interlace premise explicit; its current public
/// constructors model non-interlace timing, including the skipped toggle when
/// entering V=240 of an odd field.
pub const fn snes9x_wram_refresh_cycle(
    field_index: u64,
    scanline: u16,
    field_timing: CpuFieldTiming,
) -> u32 {
    let odd_fields_before = if field_timing.field_is_odd(0) {
        field_index / 2 + field_index % 2
    } else {
        field_index / 2
    };
    let skipped_current_v240_toggle =
        !field_timing.interlace && field_timing.field_is_odd(field_index) && scanline >= 240;
    let skipped_toggles = odd_fields_before + skipped_current_v240_toggle as u64;
    if (scanline as u64 + skipped_toggles) & 1 == 0 {
        SNES9X_WRAM_REFRESH_V2_LATE_CYCLE
    } else {
        SNES9X_WRAM_REFRESH_V2_EARLY_CYCLE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuTimelineDeadlineAdvance {
    Complete,
    ReachedDeadline { remaining_work_master_cycles: u32 },
}

/// Generic hardware timeline used by higher-level schedulers.
///
/// The timeline owns the absolute clock, field parity, bus workload, and event
/// de-duplication. Callers retain ownership of semantic deadlines such as
/// Zelda's display-publication and caller-continuation boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuMasterTimeline {
    clock_master_cycles: u64,
    bus: CpuBusWorkload,
    field_timing: CpuFieldTiming,
    wram_refresh_cycle: u16,
    processed_timeline_event: Option<(u64, CpuTimelineEvent)>,
    mode: CpuTimelineMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CpuTimelineMode {
    Unclaimed,
    Legacy,
    Synchronous(CpuSynchronousEventCursor),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CpuSynchronousEventCursor {
    master_cycles: u64,
    field_index: u64,
    scanline: u16,
    cycle_in_scanline: u16,
    event: CpuSynchronousTimelineEvent,
}

impl CpuMasterTimeline {
    pub const fn new(
        clock_master_cycles: u64,
        bus: CpuBusWorkload,
        field_timing: CpuFieldTiming,
    ) -> Self {
        let (field_index, physical_field_cycle) =
            field_timing.field_and_cycle_at(clock_master_cycles);
        let raster = field_timing.raster_at(field_index, physical_field_cycle);
        Self {
            clock_master_cycles,
            bus,
            field_timing,
            wram_refresh_cycle: snes9x_wram_refresh_cycle(
                field_index,
                raster.scanline,
                field_timing,
            ) as u16,
            processed_timeline_event: None,
            mode: CpuTimelineMode::Unclaimed,
        }
    }

    pub const fn at_raster(
        field_index: u64,
        raster: CpuRasterPosition,
        bus: CpuBusWorkload,
        field_timing: CpuFieldTiming,
    ) -> Self {
        Self::new(
            field_timing.master_cycles_at(field_index, raster),
            bus,
            field_timing,
        )
    }

    pub const fn clock_master_cycles(&self) -> u64 {
        self.clock_master_cycles
    }

    pub const fn timestamp(&self) -> CpuMasterTimestamp {
        CpuMasterTimestamp::new(self.clock_master_cycles)
    }

    pub const fn bus_workload(&self) -> CpuBusWorkload {
        self.bus
    }

    pub const fn wram_refresh_cycle(&self) -> u32 {
        self.wram_refresh_cycle as u32
    }

    pub const fn field_index(&self) -> u64 {
        self.field_timing
            .field_and_cycle_at(self.clock_master_cycles)
            .0
    }

    pub const fn master_cycles_at_raster(
        &self,
        field_index: u64,
        raster: CpuRasterPosition,
    ) -> u64 {
        self.field_timing.master_cycles_at(field_index, raster)
    }

    /// Advance interruptible work no farther than an absolute caller-owned
    /// deadline, retaining any work which did not execute.
    pub fn advance_interruptible_until(
        &mut self,
        deadline_master_cycles: u64,
        mut work_master_cycles: u32,
    ) -> CpuTimelineDeadlineAdvance {
        self.claim_legacy_timeline();
        debug_assert!(self.clock_master_cycles < deadline_master_cycles);
        while work_master_cycles != 0 {
            let (work_until_event, event) = self.next_timeline_event();
            let event_stall = event.map_or(0, |event| self.fixed_event_advance(event));
            let master_cycles_until_deadline = deadline_master_cycles - self.clock_master_cycles;
            if master_cycles_until_deadline <= u64::from(work_until_event) {
                if u64::from(work_master_cycles) < master_cycles_until_deadline {
                    self.advance_physical_clock(u64::from(work_master_cycles));
                    return CpuTimelineDeadlineAdvance::Complete;
                }
                self.set_physical_clock(deadline_master_cycles);
                return CpuTimelineDeadlineAdvance::ReachedDeadline {
                    remaining_work_master_cycles: work_master_cycles
                        - u32::try_from(master_cycles_until_deadline)
                            .expect("timeline deadline distance exceeded remaining CPU work"),
                };
            }
            if work_master_cycles <= work_until_event {
                self.advance_physical_clock(u64::from(work_master_cycles));
                return CpuTimelineDeadlineAdvance::Complete;
            }

            self.advance_physical_clock(u64::from(work_until_event));
            work_master_cycles -= work_until_event;
            if let Some(event) = event {
                self.processed_timeline_event = Some((self.clock_master_cycles, event));
            }
            if self.clock_master_cycles + u64::from(event_stall) >= deadline_master_cycles {
                self.set_physical_clock(deadline_master_cycles);
                return CpuTimelineDeadlineAdvance::ReachedDeadline {
                    remaining_work_master_cycles: work_master_cycles,
                };
            }
            self.advance_physical_clock(u64::from(event_stall));
        }
        CpuTimelineDeadlineAdvance::Complete
    }

    pub fn advance_work_unbounded(&mut self, work_master_cycles: u32) {
        let hdma_stall_master_cycles = self.bus.hdma_stall_master_cycles;
        self.advance_work_unbounded_with(work_master_cycles, |event, _| match event {
            CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => WRAM_REFRESH_STALL_MASTER_CYCLES,
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart) => u32::from(hdma_stall_master_cycles),
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit) => 0,
            CpuTimelineEvent::ShortScanline => 0,
        });
    }

    pub fn advance_work_unbounded_with(
        &mut self,
        mut work_master_cycles: u32,
        mut event_advance: impl FnMut(CpuTimelineEvent, u16) -> u32,
    ) {
        self.claim_legacy_timeline();
        while work_master_cycles != 0 {
            let (work_until_event, event) = self.next_timeline_event();
            if work_until_event < work_master_cycles {
                self.advance_physical_clock(u64::from(work_until_event));
                work_master_cycles -= work_until_event;
                if let Some(event) = event {
                    let scanline = match event {
                        CpuTimelineEvent::ShortScanline => 240,
                        CpuTimelineEvent::Bus(_) => self.raster_position().scanline,
                    };
                    self.processed_timeline_event = Some((self.clock_master_cycles, event));
                    self.advance_physical_clock(u64::from(event_advance(event, scanline)));
                }
            } else {
                self.advance_physical_clock(u64::from(work_master_cycles));
                work_master_cycles = 0;
            }
        }
    }

    /// Claim a newly created timeline for source-ordered synchronous work.
    /// Checkpoints in a fixed event's consumed-clock window are rejected: a
    /// bare raster there cannot prove whether the event is pending or drained.
    pub fn begin_synchronous_timeline(&mut self) -> Result<(), CpuSynchronousTimelineStartError> {
        match self.mode {
            CpuTimelineMode::Synchronous(_) => return Ok(()),
            CpuTimelineMode::Legacy => {
                return Err(CpuSynchronousTimelineStartError::LegacyTimelineAlreadyClaimed)
            }
            CpuTimelineMode::Unclaimed => {}
        }

        let raster = self.raster_position();
        for (cycle, event, enabled) in
            self.synchronous_bus_events(self.field_index(), raster.scanline)
        {
            if !enabled {
                continue;
            }
            let fixed_stall = self.fixed_event_advance(CpuTimelineEvent::Bus(event));
            let raster_cycle = u32::from(raster.master_cycle);
            let ambiguous = if fixed_stall == 0 {
                raster_cycle == cycle
            } else {
                (cycle..cycle + fixed_stall).contains(&raster_cycle)
            };
            if ambiguous {
                return Err(CpuSynchronousTimelineStartError::AmbiguousEventState {
                    event,
                    timestamp: self.timestamp(),
                });
            }
        }
        let line_start_master_cycles = self.clock_master_cycles - u64::from(raster.master_cycle);
        let cursor = self.synchronous_cursor_on_line_after(
            self.field_index(),
            raster.scanline,
            line_start_master_cycles,
            Some(u32::from(raster.master_cycle)),
        );
        self.mode = CpuTimelineMode::Synchronous(cursor);
        Ok(())
    }

    /// Charge one complete Snes9x `AddCycles` transaction after its semantic,
    /// then drain every due event using `Cycles >= NextEvent`. The callback
    /// observes the transaction's access-end clock, including overshoot, and
    /// returns clocks added by that event handler. A callback error retains the
    /// due event and charged transaction; resume it with a zero-cycle call.
    pub fn advance_synchronous_after_semantics_with<E>(
        &mut self,
        work_master_cycles: u32,
        mut observe_event: impl FnMut(CpuSynchronousTimelineEvent, CpuMasterTimestamp) -> Result<u32, E>,
    ) -> Result<(), E> {
        let CpuTimelineMode::Synchronous(mut cursor) = self.mode else {
            panic!("synchronous CPU work requires begin_synchronous_timeline")
        };

        // Snes9x getset/AddCycles performs the complete transaction charge
        // before it drains every now-due event with `Cycles >= NextEvent`.
        self.advance_physical_clock_preserving_refresh(u64::from(work_master_cycles));
        while self.clock_master_cycles >= cursor.master_cycles {
            let event = cursor.event;
            let next_cursor = self.synchronous_cursor_after(cursor);
            // Keep the current event owned until its fallible handler succeeds.
            // Clocks returned by the handler are intentionally not drained
            // recursively; the outer loop observes them after it returns.
            let handler_master_cycles = observe_event(event, self.timestamp())?;
            self.mode = CpuTimelineMode::Synchronous(next_cursor);
            if matches!(event, CpuSynchronousTimelineEvent::HMax { .. }) {
                self.wram_refresh_cycle = snes9x_wram_refresh_cycle(
                    next_cursor.field_index,
                    next_cursor.scanline,
                    self.field_timing,
                ) as u16;
            }
            let fixed_master_cycles = match event {
                CpuSynchronousTimelineEvent::Bus(event) => {
                    self.fixed_event_advance(CpuTimelineEvent::Bus(event))
                }
                CpuSynchronousTimelineEvent::HMax { .. } => 0,
            };
            self.advance_physical_clock_preserving_refresh(
                u64::from(fixed_master_cycles) + u64::from(handler_master_cycles),
            );
            cursor = next_cursor;
        }
        Ok(())
    }

    /// Pinned Snes9x's direct `PCBase` opcode fetch increments `CPU.Cycles`
    /// without invoking `AddCycles`; a due event is intentionally left owned
    /// by the synchronous cursor until the next source transaction drains it.
    pub(crate) fn advance_synchronous_pcbase_opcode_fetch(&mut self, memory_speed: u8) {
        assert!(
            matches!(self.mode, CpuTimelineMode::Synchronous(_)),
            "PCBase fetch requires a claimed synchronous timeline"
        );
        self.advance_physical_clock_preserving_refresh(u64::from(memory_speed));
    }

    pub fn raster_position(&self) -> CpuRasterPosition {
        let (field_index, physical_field_cycle) = self
            .field_timing
            .field_and_cycle_at(self.clock_master_cycles);
        self.field_timing
            .raster_at(field_index, physical_field_cycle)
    }

    fn next_timeline_event(&self) -> (u32, Option<CpuTimelineEvent>) {
        let (field_index, _) = self
            .field_timing
            .field_and_cycle_at(self.clock_master_cycles);
        let raster = self.raster_position();
        let scanline = u64::from(raster.scanline);
        let cycle = u32::from(raster.master_cycle);
        let mut next_event_cycle = MASTER_CYCLES_PER_SCANLINE;
        let mut next_event = None;

        for (event_cycle, event, enabled) in [
            (
                HDMA_INIT_CYCLE,
                CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit),
                scanline == 0 && self.bus.dynamic_hdma,
            ),
            (
                u32::from(self.wram_refresh_cycle),
                CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh),
                true,
            ),
            (
                HDMA_START_CYCLE,
                CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart),
                scanline < u64::from(NMI_SCANLINE)
                    && (self.bus.dynamic_hdma || self.bus.hdma_stall_master_cycles != 0),
            ),
            (
                SHORT_SCANLINE_END_CYCLE,
                CpuTimelineEvent::ShortScanline,
                scanline == 240
                    && !self.field_timing.interlace
                    && self.field_timing.field_is_odd(field_index),
            ),
        ] {
            let already_processed =
                self.processed_timeline_event == Some((self.clock_master_cycles, event));
            if enabled
                && !already_processed
                && cycle <= event_cycle
                && event_cycle < next_event_cycle
            {
                next_event_cycle = event_cycle;
                next_event = Some(event);
            }
        }
        (next_event_cycle - cycle, next_event)
    }

    fn claim_legacy_timeline(&mut self) {
        match self.mode {
            CpuTimelineMode::Unclaimed | CpuTimelineMode::Legacy => {
                self.mode = CpuTimelineMode::Legacy
            }
            CpuTimelineMode::Synchronous(_) => {
                panic!("synchronous and legacy CPU timeline APIs cannot be mixed")
            }
        }
    }

    fn synchronous_bus_events(
        &self,
        field_index: u64,
        scanline: u16,
    ) -> [(u32, CpuBusEvent, bool); 3] {
        [
            (
                HDMA_INIT_CYCLE,
                CpuBusEvent::HdmaInit,
                scanline == 0 && self.bus.dynamic_hdma,
            ),
            (
                snes9x_wram_refresh_cycle(field_index, scanline, self.field_timing),
                CpuBusEvent::WramRefresh,
                true,
            ),
            (
                HDMA_START_CYCLE,
                CpuBusEvent::HdmaStart,
                u32::from(scanline) < NMI_SCANLINE
                    && (self.bus.dynamic_hdma || self.bus.hdma_stall_master_cycles != 0),
            ),
        ]
    }

    fn synchronous_cursor_on_line_after(
        &self,
        field_index: u64,
        scanline: u16,
        line_start_master_cycles: u64,
        after_cycle: Option<u32>,
    ) -> CpuSynchronousEventCursor {
        for (cycle, event, enabled) in self.synchronous_bus_events(field_index, scanline) {
            if enabled && after_cycle.is_none_or(|after| cycle > after) {
                return CpuSynchronousEventCursor {
                    master_cycles: line_start_master_cycles + u64::from(cycle),
                    field_index,
                    scanline,
                    cycle_in_scanline: cycle as u16,
                    event: CpuSynchronousTimelineEvent::Bus(event),
                };
            }
        }

        let short_scanline = scanline == 240
            && !self.field_timing.interlace
            && self.field_timing.field_is_odd(field_index);
        let hmax = if short_scanline {
            SHORT_SCANLINE_END_CYCLE
        } else {
            MASTER_CYCLES_PER_SCANLINE
        };
        CpuSynchronousEventCursor {
            master_cycles: line_start_master_cycles + u64::from(hmax),
            field_index,
            scanline,
            cycle_in_scanline: hmax as u16,
            event: CpuSynchronousTimelineEvent::HMax {
                completed_field_index: field_index,
                completed_scanline: scanline,
                line_master_cycles: hmax as u16,
                event_timestamp: CpuMasterTimestamp::new(
                    line_start_master_cycles + u64::from(hmax),
                ),
            },
        }
    }

    fn synchronous_cursor_after(
        &self,
        cursor: CpuSynchronousEventCursor,
    ) -> CpuSynchronousEventCursor {
        match cursor.event {
            CpuSynchronousTimelineEvent::Bus(_) => self.synchronous_cursor_on_line_after(
                cursor.field_index,
                cursor.scanline,
                cursor.master_cycles - u64::from(cursor.cycle_in_scanline),
                Some(u32::from(cursor.cycle_in_scanline)),
            ),
            CpuSynchronousTimelineEvent::HMax { .. } => {
                let (field_index, scanline) =
                    if u32::from(cursor.scanline) + 1 == NTSC_SCANLINES_PER_FIELD {
                        (cursor.field_index + 1, 0)
                    } else {
                        (cursor.field_index, cursor.scanline + 1)
                    };
                self.synchronous_cursor_on_line_after(
                    field_index,
                    scanline,
                    cursor.master_cycles,
                    None,
                )
            }
        }
    }

    fn fixed_event_advance(&self, event: CpuTimelineEvent) -> u32 {
        match event {
            CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => WRAM_REFRESH_STALL_MASTER_CYCLES,
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart) => {
                u32::from(self.bus.hdma_stall_master_cycles)
            }
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit) => 0,
            CpuTimelineEvent::ShortScanline => 0,
        }
    }

    fn advance_physical_clock(&mut self, master_cycles: u64) {
        self.set_physical_clock(self.clock_master_cycles + master_cycles);
    }

    fn advance_physical_clock_preserving_refresh(&mut self, master_cycles: u64) {
        self.clock_master_cycles += master_cycles;
    }

    fn set_physical_clock(&mut self, clock_master_cycles: u64) {
        self.clock_master_cycles = clock_master_cycles;
        let (field_index, physical_field_cycle) = self
            .field_timing
            .field_and_cycle_at(self.clock_master_cycles);
        let raster = self
            .field_timing
            .raster_at(field_index, physical_field_cycle);
        self.wram_refresh_cycle =
            snes9x_wram_refresh_cycle(field_index, raster.scanline, self.field_timing) as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LONG_TIMELINE_FIELD: u64 = 24_001;

    fn at_raster(
        field: u64,
        raster: CpuRasterPosition,
        bus: CpuBusWorkload,
        timing: CpuFieldTiming,
    ) -> CpuMasterTimeline {
        CpuMasterTimeline::at_raster(field, raster, bus, timing)
    }

    #[test]
    fn exact_event_endpoint_defers_stall_until_following_work() {
        for field in [0, LONG_TIMELINE_FIELD] {
            let mut timeline = at_raster(
                field,
                CpuRasterPosition::new(100, 532),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            timeline.advance_work_unbounded(6);
            assert_eq!(timeline.raster_position(), CpuRasterPosition::new(100, 538));
            timeline.advance_work_unbounded(6);
            assert_eq!(timeline.raster_position(), CpuRasterPosition::new(100, 584));
        }
    }

    #[test]
    fn snes9x_v2_refresh_phase_is_carried_across_scanlines_and_fields() {
        let timing = CpuFieldTiming::NON_INTERLACE_EVEN;

        // cpu.cpp resets the selected v2 model at 538; cpuexec.cpp toggles
        // 534/538 on each ordinary new scanline.
        assert_eq!(snes9x_wram_refresh_cycle(0, 0, timing), 538);
        assert_eq!(snes9x_wram_refresh_cycle(0, 1, timing), 534);
        assert_eq!(snes9x_wram_refresh_cycle(0, 2, timing), 538);

        // Entering V=240 of an odd non-interlace field is the one place where
        // Snes9x skips the toggle, so V239 and V240 share H=534.
        assert_eq!(snes9x_wram_refresh_cycle(1, 239, timing), 534);
        assert_eq!(snes9x_wram_refresh_cycle(1, 240, timing), 534);
        assert_eq!(snes9x_wram_refresh_cycle(1, 241, timing), 538);

        // The skipped toggle is carried into the following field; it is not a
        // function of scanline parity alone.
        assert_eq!(snes9x_wram_refresh_cycle(2, 0, timing), 534);
        assert_eq!(snes9x_wram_refresh_cycle(2, 1, timing), 538);
        assert_eq!(snes9x_wram_refresh_cycle(4, 0, timing), 538);
    }

    #[test]
    fn pinned_cold_fixture_receipt_selects_m1_and_v2_refresh_at_reset() {
        let reset = crate::test_bootstrap_fixture::records()
            .into_iter()
            .find(|record| record["kind"] == "reset-state")
            .expect("cold fixture must include its source reset receipt");
        assert_eq!(reset["cpu_model_identity"].as_u64(), Some(1));
        assert_eq!(reset["cpu_model_5a22"].as_u64(), Some(2));
        assert_eq!(reset["wram_refresh_position"].as_u64(), Some(538));
        assert_eq!(
            snes9x_wram_refresh_cycle(0, 0, CpuFieldTiming::NON_INTERLACE_EVEN),
            538
        );
    }

    #[test]
    fn timeline_retains_v2_refresh_phase_after_more_than_twenty_four_thousand_fields() {
        let timing = CpuFieldTiming::NON_INTERLACE_EVEN;
        let field = LONG_TIMELINE_FIELD;
        let mut timeline = at_raster(
            field,
            CpuRasterPosition::new(239, 1_350),
            CpuBusWorkload::default(),
            timing,
        );
        assert_eq!(timeline.wram_refresh_cycle(), 534);

        timeline.advance_work_unbounded(20);
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(240, 6));
        assert_eq!(timeline.wram_refresh_cycle(), 534);

        let mut at_end_of_short_line = at_raster(
            field,
            CpuRasterPosition::new(240, 1_350),
            CpuBusWorkload::default(),
            timing,
        );
        at_end_of_short_line.advance_work_unbounded(10);
        assert_eq!(
            at_end_of_short_line.raster_position(),
            CpuRasterPosition::new(241, 0)
        );
        assert_eq!(at_end_of_short_line.wram_refresh_cycle(), 538);
    }

    #[test]
    fn synchronous_exact_refresh_endpoint_applies_stall_before_next_timestamp() {
        let mut timeline = at_raster(
            0,
            CpuRasterPosition::new(100, 532),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline.begin_synchronous_timeline().unwrap();
        let refresh_timestamp = timeline.clock_master_cycles() + 6;
        let mut events = Vec::new();
        timeline
            .advance_synchronous_after_semantics_with(6, |event, timestamp| {
                events.push((event, timestamp));
                Ok::<u32, ()>(0)
            })
            .unwrap();
        assert_eq!(
            events,
            [(
                CpuSynchronousTimelineEvent::Bus(CpuBusEvent::WramRefresh),
                CpuMasterTimestamp::new(refresh_timestamp),
            )]
        );
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(100, 578));
        assert_eq!(timeline.clock_master_cycles(), refresh_timestamp + 40);
    }

    #[test]
    fn synchronous_refresh_observes_access_overshoot_before_adding_stall() {
        let mut timeline = at_raster(
            0,
            CpuRasterPosition::new(100, 534),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline.begin_synchronous_timeline().unwrap();
        let observation = timeline.clock_master_cycles() + 8;
        let mut events = Vec::new();
        timeline
            .advance_synchronous_after_semantics_with(8, |event, timestamp| {
                events.push((event, timestamp));
                Ok::<u32, ()>(0)
            })
            .unwrap();
        assert_eq!(
            events,
            [(
                CpuSynchronousTimelineEvent::Bus(CpuBusEvent::WramRefresh),
                CpuMasterTimestamp::new(observation),
            )]
        );
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(100, 582));
    }

    #[test]
    fn synchronous_hmax_emits_normal_and_odd_short_line_context_at_access_end() {
        let cases = [
            (
                CpuFieldTiming::NON_INTERLACE_EVEN,
                CpuRasterPosition::new(7, 1_358),
                8,
                CpuRasterPosition::new(8, 2),
                7,
                MASTER_CYCLES_PER_SCANLINE as u16,
            ),
            (
                CpuFieldTiming::non_interlace(true),
                CpuRasterPosition::new(240, 1_354),
                6,
                CpuRasterPosition::new(241, 0),
                240,
                SHORT_SCANLINE_END_CYCLE as u16,
            ),
        ];
        for (timing, entry, work, expected_raster, completed_scanline, line_master_cycles) in cases
        {
            let mut timeline = at_raster(0, entry, CpuBusWorkload::default(), timing);
            timeline.begin_synchronous_timeline().unwrap();
            let expected_timestamp = timeline.clock_master_cycles() + u64::from(work);
            let expected_hmax_timestamp = timing.master_cycles_at(0, entry)
                - u64::from(entry.coordinates().1)
                + u64::from(line_master_cycles);
            let mut events = Vec::new();
            timeline
                .advance_synchronous_after_semantics_with(work, |event, timestamp| {
                    events.push((event, timestamp));
                    Ok::<u32, ()>(0)
                })
                .unwrap();
            assert_eq!(
                events,
                [(
                    CpuSynchronousTimelineEvent::HMax {
                        completed_field_index: 0,
                        completed_scanline,
                        line_master_cycles,
                        event_timestamp: CpuMasterTimestamp::new(expected_hmax_timestamp),
                    },
                    CpuMasterTimestamp::new(expected_timestamp),
                )]
            );
            assert_eq!(timeline.raster_position(), expected_raster);
            assert_eq!(timeline.clock_master_cycles(), expected_timestamp);
        }
    }

    #[test]
    fn synchronous_handler_clocks_are_added_before_nested_events_drain() {
        let mut timeline = at_raster(
            0,
            CpuRasterPosition::new(100, 534),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline.begin_synchronous_timeline().unwrap();
        let mut events = Vec::new();
        timeline
            .advance_synchronous_after_semantics_with(8, |event, timestamp| {
                events.push((event, timestamp));
                Ok::<u32, ()>(match event {
                    CpuSynchronousTimelineEvent::Bus(CpuBusEvent::WramRefresh) => 900,
                    _ => 0,
                })
            })
            .unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            (
                CpuSynchronousTimelineEvent::Bus(CpuBusEvent::WramRefresh),
                CpuMasterTimestamp::new(
                    CpuFieldTiming::NON_INTERLACE_EVEN
                        .master_cycles_at(0, CpuRasterPosition::new(100, 542),),
                ),
            )
        );
        assert_eq!(
            events[1],
            (
                CpuSynchronousTimelineEvent::HMax {
                    completed_field_index: 0,
                    completed_scanline: 100,
                    line_master_cycles: MASTER_CYCLES_PER_SCANLINE as u16,
                    event_timestamp: CpuMasterTimestamp::new(
                        CpuFieldTiming::NON_INTERLACE_EVEN
                            .master_cycles_at(0, CpuRasterPosition::new(101, 0),),
                    ),
                },
                CpuMasterTimestamp::new(
                    CpuFieldTiming::NON_INTERLACE_EVEN
                        .master_cycles_at(0, CpuRasterPosition::new(101, 118),),
                ),
            )
        );
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(101, 118));
    }

    #[test]
    fn synchronous_start_rejects_ambiguous_event_and_legacy_ownership() {
        let mut exact_refresh = at_raster(
            0,
            CpuRasterPosition::new(100, SNES9X_WRAM_REFRESH_V2_LATE_CYCLE as u16),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            exact_refresh.begin_synchronous_timeline(),
            Err(CpuSynchronousTimelineStartError::AmbiguousEventState {
                event: CpuBusEvent::WramRefresh,
                timestamp: exact_refresh.timestamp(),
            })
        );

        let mut inside_refresh = at_raster(
            0,
            CpuRasterPosition::new(100, 542),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        assert_eq!(
            inside_refresh.begin_synchronous_timeline(),
            Err(CpuSynchronousTimelineStartError::AmbiguousEventState {
                event: CpuBusEvent::WramRefresh,
                timestamp: inside_refresh.timestamp(),
            })
        );

        let mut legacy = at_raster(
            0,
            CpuRasterPosition::new(100, 100),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        legacy.advance_work_unbounded(1);
        assert_eq!(
            legacy.begin_synchronous_timeline(),
            Err(CpuSynchronousTimelineStartError::LegacyTimelineAlreadyClaimed)
        );
    }

    #[test]
    fn dynamic_event_sequence_preserves_refresh_and_hdma_order() {
        let mut timeline = at_raster(
            LONG_TIMELINE_FIELD,
            CpuRasterPosition::new(100, 524),
            CpuBusWorkload::with_dynamic_hdma(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut events = Vec::new();
        timeline.advance_work_unbounded_with(600, |event, scanline| {
            events.push((event, scanline));
            match event {
                CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => WRAM_REFRESH_STALL_MASTER_CYCLES,
                CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart) => 42,
                CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit) => 0,
                CpuTimelineEvent::ShortScanline => 0,
            }
        });
        assert_eq!(
            events,
            [
                (CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh), 100),
                (CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart), 100),
            ],
        );
        assert_eq!(
            timeline.raster_position(),
            CpuRasterPosition::new(100, 1_206)
        );
    }

    #[test]
    fn odd_short_scanline_advances_raster_without_phantom_master_cycles() {
        let entry = CpuRasterPosition::new(240, 1_350);
        let mut even = at_raster(
            LONG_TIMELINE_FIELD - 1,
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut odd = at_raster(
            LONG_TIMELINE_FIELD,
            entry,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );

        let even_clock = even.clock_master_cycles();
        let odd_clock = odd.clock_master_cycles();
        even.advance_work_unbounded(20);
        odd.advance_work_unbounded(20);
        assert_eq!(even.clock_master_cycles() - even_clock, 20);
        assert_eq!(odd.clock_master_cycles() - odd_clock, 20);
        assert_eq!(even.raster_position(), CpuRasterPosition::new(241, 6));
        assert_eq!(odd.raster_position(), CpuRasterPosition::new(241, 10));
    }

    #[test]
    fn odd_h1350_reaches_next_scanline_in_ten_physical_cycles() {
        let mut odd = at_raster(
            LONG_TIMELINE_FIELD,
            CpuRasterPosition::new(240, 1_350),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let start = odd.clock_master_cycles();
        odd.advance_work_unbounded(10);
        assert_eq!(odd.clock_master_cycles(), start + 10);
        assert_eq!(odd.raster_position(), CpuRasterPosition::new(241, 0));
    }

    #[test]
    fn physical_clock_total_excludes_every_completed_short_line() {
        let timing = CpuFieldTiming::NON_INTERLACE_EVEN;
        let completed_short_fields = LONG_TIMELINE_FIELD / 2;
        let expected = LONG_TIMELINE_FIELD * NTSC_FIELD_MASTER_CYCLES - completed_short_fields * 4;
        assert_eq!(
            timing.field_start_master_cycles(LONG_TIMELINE_FIELD),
            expected,
        );
        let timeline = at_raster(
            LONG_TIMELINE_FIELD,
            CpuRasterPosition::new(0, 0),
            CpuBusWorkload::default(),
            timing,
        );
        assert_eq!(timeline.clock_master_cycles(), expected);
        assert_eq!(timeline.field_index(), LONG_TIMELINE_FIELD);
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(0, 0));
    }

    #[test]
    fn initially_odd_timing_starts_with_the_short_field() {
        let timing = CpuFieldTiming::non_interlace(true);
        assert_eq!(
            timing.field_master_cycles(0),
            NTSC_SHORT_FIELD_MASTER_CYCLES
        );
        assert_eq!(timing.field_master_cycles(1), NTSC_FIELD_MASTER_CYCLES);
        assert_eq!(
            timing.field_start_master_cycles(1),
            NTSC_SHORT_FIELD_MASTER_CYCLES,
        );

        let field_one = at_raster(
            1,
            CpuRasterPosition::new(0, 0),
            CpuBusWorkload::default(),
            timing,
        );
        assert_eq!(
            field_one.clock_master_cycles(),
            NTSC_SHORT_FIELD_MASTER_CYCLES
        );
        assert_eq!(field_one.field_index(), 1);
        assert_eq!(field_one.raster_position(), CpuRasterPosition::new(0, 0));
    }

    #[test]
    fn deadline_crossing_after_short_line_uses_physical_distance() {
        let timing = CpuFieldTiming::NON_INTERLACE_EVEN;
        let field = LONG_TIMELINE_FIELD;
        let mut timeline = at_raster(
            field,
            CpuRasterPosition::new(240, 1_350),
            CpuBusWorkload::default(),
            timing,
        );
        let start = timeline.clock_master_cycles();
        let deadline = timing.master_cycles_at(field, CpuRasterPosition::new(241, 4));
        assert_eq!(
            timeline.advance_interruptible_until(deadline, 20),
            CpuTimelineDeadlineAdvance::ReachedDeadline {
                remaining_work_master_cycles: 6,
            },
        );
        assert_eq!(timeline.clock_master_cycles(), start + 14);
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(241, 4));
    }

    #[test]
    fn deadline_advance_retains_unexecuted_work() {
        let mut timeline = at_raster(
            0,
            CpuRasterPosition::new(224, 1_300),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let deadline = u64::from(NMI_SCANLINE * MASTER_CYCLES_PER_SCANLINE);
        assert_eq!(
            timeline.advance_interruptible_until(deadline, 100),
            CpuTimelineDeadlineAdvance::ReachedDeadline {
                remaining_work_master_cycles: 36,
            },
        );
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(225, 0));
    }

    #[test]
    fn non_draining_pcbase_crossing_retains_refresh_phase_until_hmax_drain() {
        let mut timeline = at_raster(
            0,
            CpuRasterPosition::new(0, 1_358),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline.begin_synchronous_timeline().unwrap();
        assert_eq!(timeline.wram_refresh_cycle(), 538);

        timeline.advance_synchronous_pcbase_opcode_fetch(8);
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(1, 2));
        assert_eq!(timeline.wram_refresh_cycle(), 538);

        let mut observed = Vec::new();
        timeline
            .advance_synchronous_after_semantics_with(8, |event, timestamp| {
                observed.push((event, timestamp));
                Ok::<_, ()>(0)
            })
            .unwrap();
        assert_eq!(timeline.raster_position(), CpuRasterPosition::new(1, 10));
        assert_eq!(timeline.wram_refresh_cycle(), 534);
        assert_eq!(observed.len(), 1);
        assert!(matches!(
            observed[0].0,
            CpuSynchronousTimelineEvent::HMax {
                completed_field_index: 0,
                completed_scanline: 0,
                line_master_cycles: 1_364,
                ..
            }
        ));
        assert_eq!(observed[0].1.master_cycles(), 1_374);
    }
}
