//! Independent APU fixture for CPU owners which perform graphics and DMA work.
//! Only the initial CPU/WRAM/PPU inputs are shared with the source fixture.
use super::{
    intro_clear::Access,
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
};
use snes::{
    apu::{ApuHostPortProbe, ApuHostPortTiming, ApuState},
    Snes9xColdCpuExecutor,
};

pub(super) struct ApuFixture {
    timing: ApuHostPortTiming,
    pub(super) lines: usize,
    checked_lines: usize,
}

impl ApuFixture {
    pub(super) fn new() -> Self {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        Self {
            timing: ApuHostPortTiming::new(
                ApuHostPortProbe::from_snes9x_coroutine(apu).unwrap(),
                Default::default(),
            )
            .unwrap(),
            lines: 0,
            checked_lines: 0,
        }
    }

    pub(super) fn bus<F: FnMut(u64, Access) -> u16>(&mut self, access: F) -> ApuBus<'_, F> {
        ApuBus { apu: self, access }
    }

    /// Execute the fixture's CLC/XCE/LDA80/STA2100/BRA setup on matching
    /// timelines. APU state and acknowledgements are never copied from source.
    pub(super) fn warm_to(&mut self, source: &mut Snes9xColdCpuExecutor, wanted: u64) {
        let mut clock = source.machine().timeline().clone();
        let mut cpu = NativeNmiContext {
            a: 0,
            x: 0,
            y: 0,
            d: 0,
            db: 0,
            bank: 0,
            pc: 0x8000,
            sp: 0x1ff,
            p: 0x34,
        };
        while clock.clock_master_cycles() < wanted {
            let source_step = source.step().unwrap();
            assert!(source_step.accepted_interrupt.is_none());
            let mut w = Work {
                cpu: &mut cpu,
                clock: &mut clock,
                bus: self.bus(|_, _| 0),
                cycles: 0,
            };
            w.fetch();
            w.cpu.pc = match w.cpu.pc {
                0x8000 | 0x8001 => {
                    w.drain(6);
                    w.cpu.pc + 1
                }
                0x8002 => {
                    w.drain(8);
                    0x8004
                }
                0x8004 => {
                    w.drain(16);
                    w.write_io(0x2100, 0x80, 1);
                    0x8007
                }
                0x8007 => {
                    w.branch(true);
                    0x8007
                }
                _ => unreachable!(),
            };
            assert_eq!(clock, *source.machine().timeline(), "fixture warm-up clock");
        }
        self.assert_matches(source, "fixture warm-up");
        // Reset only coverage counters; retain the APU and both device clocks.
        self.lines = 0;
        self.checked_lines = 0;
    }

    pub(super) fn assert_matches(&mut self, source: &Snes9xColdCpuExecutor, label: &str) {
        let native = self.timing.machine();
        let expected = &source.machine().snes().apu;
        assert_eq!(
            (native.cycles, native.in_ports, native.out_ports),
            (expected.cycles, expected.in_ports, expected.out_ports),
            "{label}: APU clocks/ports after CPU{:02x}:{:04x}",
            source.machine().snes().cpu.k, source.machine().snes().cpu.pc
        );
        if self.lines == self.checked_lines {
            return;
        }
        let mut native = native.clone();
        let mut expected = expected.clone();
        // Diagnostic instruction-cost statistic only; every hardware field and
        // the SMP/DSP continuation remain in the comparison.
        native.spc.cycles_used = 0;
        expected.spc.cycles_used = 0;
        assert!(
            bincode::serialize(&native).unwrap() == bincode::serialize(&expected).unwrap(),
            "{label}: APU hardware at nativePC{:04x}/sourcePC{:04x}",
            native.spc.pc,
            expected.spc.pc
        );
        assert_eq!(
            native.capture_snes9x_apu_coroutine_checkpoint(),
            expected.capture_snes9x_apu_coroutine_checkpoint(),
            "{label}: APU continuation"
        );
        self.timing = self.timing.clone();
        self.checked_lines = self.lines;
    }
}

pub(super) struct ApuBus<'a, F> {
    apu: &'a mut ApuFixture,
    access: F,
}
impl<F: FnMut(u64, Access) -> u16> WorkBus for ApuBus<'_, F> {
    fn access(&mut self, at: u64, access: Access) -> u16 {
        (self.access)(at, access)
    }
    fn end_scanline(&mut self, at: u64) {
        self.apu.timing.end_scanline_at(at).unwrap();
        self.apu.lines += 1;
    }
}
