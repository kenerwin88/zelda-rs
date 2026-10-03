//! PPU bus state retained by the native CPU from hardware reset. Presentation
//! OAM is not an input: byte accesses and processed timeline events own these
//! address flips, overflow flags and counter latches.
use snes::cpu_timeline::{CpuSynchronousBeamPosition, CpuSynchronousTimelineEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativePpuState {
    oam: snes::SourceOamPort,
    reads: snes::SourcePpuReadState,
}

impl NativePpuState {
    pub(crate) fn reset() -> Self {
        Self {
            oam: snes::SourceOamPort::reset(),
            reads: snes::SourcePpuReadState::snes9x_reset(),
        }
    }

    pub(crate) fn timeline_event(&mut self, event: CpuSynchronousTimelineEvent) {
        match event {
            CpuSynchronousTimelineEvent::RenderLine { scanline, odd_field } => {
                self.oam.render_line(scanline, odd_field);
            }
            CpuSynchronousTimelineEvent::HMax { completed_scanline, .. } => {
                // The audited Zelda CPU path uses NTSC non-interlaced fields.
                self.oam.enter_scanline((completed_scanline + 1) % 262);
            }
            CpuSynchronousTimelineEvent::Bus(_) => {}
        }
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
}
