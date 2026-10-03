//! PPU bus state retained by the native CPU from hardware reset. Presentation
//! OAM is not an input: byte accesses and processed timeline events own these
//! address flips, overflow flags and counter latches.
use snes::cpu_timeline::{CpuSynchronousBeamPosition, CpuSynchronousTimelineEvent, CpuSynchronousEventObserver};
use snes::CpuMasterTimeline;
use super::{intro_clear::Access, native_work::WorkBus};

/// Attach the reset-owned PPU to an existing native memory/device bus.
/// Every byte uses the last observed processed beam, before its access drains.
pub(crate) struct NativePpuBus<'a, B> {
    ppu: &'a mut Option<NativePpuState>,
    bus: B,
    beam: Option<CpuSynchronousBeamPosition>,
}

impl<'a, B> NativePpuBus<'a, B> {
    pub(crate) fn new(ppu: &'a mut Option<NativePpuState>, bus: B) -> Self {
        Self { ppu, bus, beam: None }
    }
}

impl<B: WorkBus> WorkBus for NativePpuBus<'_, B> {
    fn observe_clock(&mut self, clock: &CpuMasterTimeline) {
        if let Some(ppu) = self.ppu.as_mut() { ppu.synchronize(clock); }
        self.beam = clock.synchronous_beam_position();
        self.bus.observe_clock(clock);
    }

    fn access(&mut self, at: u64, access: Access) -> u16 {
        if let Access::Write(address, value, width) = access {
            if address & 0x400000 == 0 && matches!(address as u16, 0x2100..=0x213f | 0x4201) {
                assert_eq!(width, 1, "native PPU stores require byte boundaries");
                if let Some(ppu) = self.ppu.as_mut() {
                    ppu.write(address as u16, value as u8, self.beam.expect("PPU write observes its clock"));
                }
            }
        }
        let value = self.bus.access(at, access);
        if let Access::Read(address, 1) = access {
            if address & 0x400000 == 0 && (0x2134..=0x2136).contains(&(address as u16)) {
                if let Some(ppu) = self.ppu.as_mut() { ppu.note_multiplier_read(address as u16, value as u8); }
            }
        }
        value
    }

    fn end_scanline(&mut self, at: u64) { self.bus.end_scanline(at); }
    fn timeline_event(&mut self, event: CpuSynchronousTimelineEvent, at: u64) {
        self.bus.timeline_event(event, at);
    }
}

impl crate::ZeldaState {
    pub(crate) fn native_ppu_bus_write(&mut self, clock: &CpuMasterTimeline, address: u16, value: u8) {
        if let Some(ppu) = self.native_ppu.as_mut() {
            ppu.write_clocked(clock, address, value);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativePpuState {
    oam: snes::SourceOamPort,
    reads: snes::SourcePpuReadState,
    events: CpuSynchronousEventObserver,
}

impl NativePpuState {
    pub(crate) fn reset(clock: &CpuMasterTimeline) -> Self {
        Self {
            oam: snes::SourceOamPort::reset(),
            reads: snes::SourcePpuReadState::snes9x_reset(),
            events: clock.synchronous_event_observer(),
        }
    }

    pub(crate) fn synchronize(&mut self, clock: &CpuMasterTimeline) {
        let oam = &mut self.oam;
        clock.observe_processed_synchronous_events(&mut self.events,
            |event| Self::timeline_event(oam, event));
    }

    fn timeline_event(oam: &mut snes::SourceOamPort, event: CpuSynchronousTimelineEvent) {
        match event {
            CpuSynchronousTimelineEvent::RenderLine { scanline, odd_field } => {
                oam.render_line(scanline, odd_field);
            }
            CpuSynchronousTimelineEvent::HMax { completed_scanline, .. } => {
                // The audited Zelda CPU path uses NTSC non-interlaced fields.
                oam.enter_scanline((completed_scanline + 1) % 262);
            }
            CpuSynchronousTimelineEvent::Bus(_) => {}
        }
    }

    pub(crate) fn write_clocked(&mut self, clock: &CpuMasterTimeline, address: u16, value: u8) {
        self.synchronize(clock);
        self.write(address, value, clock.synchronous_beam_position().unwrap());
    }

    /// Other devices still own their registers. A handled read updates the
    /// PPU bus latch at this exact semantic access, including repeated reads.
    pub(crate) fn read(&mut self, address: u16, beam: CpuSynchronousBeamPosition) -> Option<u8> {
        match address {
            0x2138 => {
                let value = self.oam.read();
                self.reads.open_bus1 = value;
                Some(value)
            }
            0x213e => Some(self.reads.read_stat77(self.oam.stat77_flags())),
            _ => self.reads.read(address, beam),
        }
    }

    pub(crate) fn write(&mut self, address: u16, value: u8, beam: CpuSynchronousBeamPosition) {
        if (0x2100..=0x2133).contains(&address) {
            self.oam.set_v_counter(beam.scanline);
            self.oam.write(address as u8, value);
        } else if address == 0x4201 {
            self.reads.write_wrio(value, beam);
        }
    }

    /// The multiplier remains in the PPU arithmetic owner, but its reads
    /// drive the same PPU1 open bus used by OAM and STAT77.
    pub(crate) fn note_multiplier_read(&mut self, address: u16, value: u8) {
        assert!((0x2134..=0x2136).contains(&address));
        self.reads.open_bus1 = value;
    }

    #[cfg(test)]
    pub(crate) fn read_state(&self) -> snes::SourcePpuReadState {
        self.reads
    }

    #[cfg(test)]
    pub(crate) fn assert_matches_source(&mut self, clock: &CpuMasterTimeline, source: &snes::CpuSynchronousMachine) {
        assert_eq!(clock, source.timeline());
        self.synchronize(clock);
        assert_eq!(&self.oam, source.snes().source_oam_port().unwrap(), "native PPU bus history");
        assert_eq!(self.reads, source.source_ppu_reads(), "native PPU read latches");
    }
}
