use crate::cpu_timeline::CpuSynchronousBeamPosition;

/// Explicit pinned NTSC PPU read-register state for a development CPU probe.
/// CPU OpenBus is a separate owner. Supplying this state asserts provenance;
/// the probe never infers it from rendered pixels or WRAM snapshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourcePpuReadState {
    pub wrio: u8,
    pub open_bus1: u8,
    pub open_bus2: u8,
    pub h_latched: u16,
    pub v_latched: u16,
    pub h_read_high: bool,
    pub v_read_high: bool,
    /// Memory.FillRAM[$213f] bit6, cleared by reading STAT78.
    pub counter_latched: bool,
}

impl SourcePpuReadState {
    /// ppu.cpp:S9xSoftResetPPU: WRIO/RDIO=$ff, zero read buses/counters/flips.
    /// This is only a reset seed, not the state of an arbitrary later caller.
    pub const fn snes9x_reset() -> Self {
        Self {
            wrio: 0xff,
            open_bus1: 0,
            open_bus2: 0,
            h_latched: 0,
            v_latched: 0,
            h_read_high: false,
            v_read_high: false,
            counter_latched: false,
        }
    }

    fn latch(&mut self, beam: CpuSynchronousBeamPosition) {
        // ppu.cpp:S9xLatchCounters: dots322 and326 last six clocks on
        // 1364-clock lines. The second comparison uses the adjusted count.
        let mut hc = beam.cycles;
        if beam.line_master_cycles == 1364 {
            if hc >= 1292 {
                hc -= 2;
            }
            if hc >= 1308 {
                hc -= 2;
            }
        }
        self.h_latched = u16::try_from(hc / 4).expect("source beam counter fits in u16");
        self.v_latched = beam.scanline;
        self.counter_latched = true;
    }

    pub(super) fn write_wrio(&mut self, value: u8, beam: CpuSynchronousBeamPosition) {
        // S9xSetCPU($4201) force-latches on the high-to-low edge, then
        // publishes the complete WRIO byte to both $4201 and RDIO($4213).
        if value & 0x80 == 0 && self.wrio & 0x80 != 0 {
            self.latch(beam);
        }
        self.wrio = value;
    }

    pub(super) fn read(&mut self, address: u16, beam: CpuSynchronousBeamPosition) -> Option<u8> {
        match address {
            0x2137 => {
                if self.wrio & 0x80 != 0 {
                    self.latch(beam);
                }
                Some(self.open_bus1)
            }
            0x213c | 0x213d => {
                let (counter, high) = if address == 0x213c {
                    (self.h_latched, &mut self.h_read_high)
                } else {
                    (self.v_latched, &mut self.v_read_high)
                };
                let value = if *high {
                    (self.open_bus2 & 0xfe) | ((counter >> 8) as u8 & 1)
                } else {
                    counter as u8
                };
                *high = !*high;
                self.open_bus2 = value;
                Some(value)
            }
            0x213f => {
                // Pinned M1SNES={1,3,2}: the NTSC 5C78 model reports3.
                let value = (self.open_bus2 & 0x20)
                    | (u8::from(beam.odd_field) << 7)
                    | (u8::from(self.counter_latched) << 6)
                    | 3;
                self.counter_latched = false;
                self.h_read_high = false;
                self.v_read_high = false;
                self.open_bus2 = value;
                Some(value)
            }
            0x4213 => Some(self.wrio),
            _ => None,
        }
    }
}
