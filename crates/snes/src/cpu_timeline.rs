//! Generic S-CPU master-cycle timeline and hardware bus events.
//!
//! This is a behavior-neutral extraction of the timing model originally owned
//! by Zelda's game-execution scheduler. In particular, the odd-field short
//! scanline remains represented as a four-master-cycle timeline advance. A
//! later physical-clock refactor can change that representation independently
//! of Zelda's semantic raster deadlines.

pub const MASTER_CYCLES_PER_SCANLINE: u32 = 1_364;
pub const NTSC_SCANLINES_PER_FIELD: u32 = 262;
pub const NMI_SCANLINE: u32 = 225;
pub const HDMA_INIT_CYCLE: u32 = 20;
// The pinned Snes9x core selects M1SNES (`_5A22 == 1`), whose cpu.cpp reset
// path uses SNES_WRAM_REFRESH_HC_v1 rather than the M2-only v2 position.
pub const WRAM_REFRESH_CYCLE: u32 = 530;
pub const WRAM_REFRESH_STALL_MASTER_CYCLES: u32 = 40;
pub const HDMA_START_CYCLE: u32 = 1_106;
pub const SHORT_SCANLINE_END_CYCLE: u32 = 1_360;
pub const SHORT_SCANLINE_MISSING_MASTER_CYCLES: u32 = 4;
pub const NTSC_FIELD_MASTER_CYCLES: u64 =
    NTSC_SCANLINES_PER_FIELD as u64 * MASTER_CYCLES_PER_SCANLINE as u64;
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

    pub const fn unwrapped_master_cycles(self) -> u64 {
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

    const fn field_is_odd(self, field_index: u64) -> bool {
        self.odd_field ^ (field_index & 1 != 0)
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuMasterTimeline {
    clock_master_cycles: u64,
    bus: CpuBusWorkload,
    field_timing: CpuFieldTiming,
    processed_timeline_event: Option<(u64, CpuTimelineEvent)>,
}

impl CpuMasterTimeline {
    pub const fn new(
        clock_master_cycles: u64,
        bus: CpuBusWorkload,
        field_timing: CpuFieldTiming,
    ) -> Self {
        Self {
            clock_master_cycles,
            bus,
            field_timing,
            processed_timeline_event: None,
        }
    }

    pub const fn clock_master_cycles(self) -> u64 {
        self.clock_master_cycles
    }

    pub const fn bus_workload(self) -> CpuBusWorkload {
        self.bus
    }

    /// Advance interruptible work no farther than an absolute caller-owned
    /// deadline, retaining any work which did not execute.
    pub fn advance_interruptible_until(
        &mut self,
        deadline_master_cycles: u64,
        mut work_master_cycles: u32,
    ) -> CpuTimelineDeadlineAdvance {
        debug_assert!(self.clock_master_cycles < deadline_master_cycles);
        while work_master_cycles != 0 {
            let (work_until_event, event) = self.next_timeline_event();
            let event_stall = event.map_or(0, |event| self.fixed_event_advance(event));
            let master_cycles_until_deadline = deadline_master_cycles - self.clock_master_cycles;
            if master_cycles_until_deadline <= u64::from(work_until_event) {
                if u64::from(work_master_cycles) < master_cycles_until_deadline {
                    self.clock_master_cycles += u64::from(work_master_cycles);
                    return CpuTimelineDeadlineAdvance::Complete;
                }
                self.clock_master_cycles = deadline_master_cycles;
                return CpuTimelineDeadlineAdvance::ReachedDeadline {
                    remaining_work_master_cycles: work_master_cycles
                        - u32::try_from(master_cycles_until_deadline)
                            .expect("timeline deadline distance exceeded remaining CPU work"),
                };
            }
            if work_master_cycles <= work_until_event {
                self.clock_master_cycles += u64::from(work_master_cycles);
                return CpuTimelineDeadlineAdvance::Complete;
            }

            self.clock_master_cycles += u64::from(work_until_event);
            work_master_cycles -= work_until_event;
            if let Some(event) = event {
                self.processed_timeline_event = Some((self.clock_master_cycles, event));
            }
            if self.clock_master_cycles + u64::from(event_stall) >= deadline_master_cycles {
                self.clock_master_cycles = deadline_master_cycles;
                return CpuTimelineDeadlineAdvance::ReachedDeadline {
                    remaining_work_master_cycles: work_master_cycles,
                };
            }
            self.clock_master_cycles += u64::from(event_stall);
        }
        CpuTimelineDeadlineAdvance::Complete
    }

    pub fn advance_work_unbounded(&mut self, work_master_cycles: u32) {
        let hdma_stall_master_cycles = self.bus.hdma_stall_master_cycles;
        self.advance_work_unbounded_with(work_master_cycles, |event, _| match event {
            CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => WRAM_REFRESH_STALL_MASTER_CYCLES,
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart) => u32::from(hdma_stall_master_cycles),
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit) => 0,
            CpuTimelineEvent::ShortScanline => SHORT_SCANLINE_MISSING_MASTER_CYCLES,
        });
    }

    pub fn advance_work_unbounded_with(
        &mut self,
        mut work_master_cycles: u32,
        mut event_advance: impl FnMut(CpuTimelineEvent, u16) -> u32,
    ) {
        while work_master_cycles != 0 {
            let (work_until_event, event) = self.next_timeline_event();
            if work_until_event < work_master_cycles {
                self.clock_master_cycles += u64::from(work_until_event);
                work_master_cycles -= work_until_event;
                if let Some(event) = event {
                    let field_cycle = self.clock_master_cycles % NTSC_FIELD_MASTER_CYCLES;
                    let scanline = (field_cycle / u64::from(MASTER_CYCLES_PER_SCANLINE)) as u16;
                    self.processed_timeline_event = Some((self.clock_master_cycles, event));
                    self.clock_master_cycles += u64::from(event_advance(event, scanline));
                }
            } else {
                self.clock_master_cycles += u64::from(work_master_cycles);
                work_master_cycles = 0;
            }
        }
    }

    pub fn raster_position(self) -> CpuRasterPosition {
        let field_cycle = self.clock_master_cycles % NTSC_FIELD_MASTER_CYCLES;
        CpuRasterPosition::new(
            (field_cycle / u64::from(MASTER_CYCLES_PER_SCANLINE)) as u16,
            (field_cycle % u64::from(MASTER_CYCLES_PER_SCANLINE)) as u16,
        )
    }

    fn next_timeline_event(self) -> (u32, Option<CpuTimelineEvent>) {
        let field_index = self.clock_master_cycles / NTSC_FIELD_MASTER_CYCLES;
        let field_cycle = self.clock_master_cycles % NTSC_FIELD_MASTER_CYCLES;
        let scanline = field_cycle / u64::from(MASTER_CYCLES_PER_SCANLINE);
        let cycle = (field_cycle % u64::from(MASTER_CYCLES_PER_SCANLINE)) as u32;
        let mut next_event_cycle = MASTER_CYCLES_PER_SCANLINE;
        let mut next_event = None;

        for (event_cycle, event, enabled) in [
            (
                HDMA_INIT_CYCLE,
                CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit),
                scanline == 0 && self.bus.dynamic_hdma,
            ),
            (
                WRAM_REFRESH_CYCLE,
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

    fn fixed_event_advance(&self, event: CpuTimelineEvent) -> u32 {
        match event {
            CpuTimelineEvent::Bus(CpuBusEvent::WramRefresh) => WRAM_REFRESH_STALL_MASTER_CYCLES,
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaStart) => {
                u32::from(self.bus.hdma_stall_master_cycles)
            }
            CpuTimelineEvent::Bus(CpuBusEvent::HdmaInit) => 0,
            CpuTimelineEvent::ShortScanline => SHORT_SCANLINE_MISSING_MASTER_CYCLES,
        }
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
        CpuMasterTimeline::new(
            field * NTSC_FIELD_MASTER_CYCLES + raster.unwrapped_master_cycles(),
            bus,
            timing,
        )
    }

    #[test]
    fn exact_event_endpoint_defers_stall_until_following_work() {
        for field in [0, LONG_TIMELINE_FIELD] {
            let mut timeline = at_raster(
                field,
                CpuRasterPosition::new(100, 524),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            timeline.advance_work_unbounded(6);
            assert_eq!(timeline.raster_position(), CpuRasterPosition::new(100, 530));
            timeline.advance_work_unbounded(6);
            assert_eq!(timeline.raster_position(), CpuRasterPosition::new(100, 576));
        }
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
                CpuTimelineEvent::ShortScanline => SHORT_SCANLINE_MISSING_MASTER_CYCLES,
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
    fn odd_short_scanline_keeps_extracted_nominal_skip_behavior() {
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

        even.advance_work_unbounded(20);
        odd.advance_work_unbounded(20);
        assert_eq!(even.raster_position(), CpuRasterPosition::new(241, 6));
        assert_eq!(odd.raster_position(), CpuRasterPosition::new(241, 10));
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
}
