//! Poly NMI publication and live interrupt-stack exchange. Each step retires
//! one native instruction; the caller owns hardware IRQ register writes and
//! resumes the thread selected by the actual RTI frame.
use super::{
    intro_clear::Access,
    native_work::{BeamBus, Work, WorkBus},
    nmi_stack::NativeNmiContext,
};
use snes::{cpu_timeline::CpuSynchronousTimelineEvent, CpuMasterTimeline, DmaState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Gate(u8),
    CallUpload,
    Upload(u8),
    IrqSetup(u8),
    Registers(u8),
    IrqPrologue(u8),
    MainIrq(u16),
    AcknowledgeIrq,
    Restore(u8),
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PolyInterrupt {
    phase: Phase,
    irq_return: bool,
    swap_stack: bool,
    vertical_timer: u16,
    irq_deadline: Option<u64>,
}

/// Live hardware programming to hand to the subsequent IRQ deadline owner.
/// This does not claim that the worker or IRQ acceptance is implemented.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PolyIrqProgramming {
    pub(crate) vertical_timer: u16,
    pub(crate) nmitimen: u8,
    pub(crate) enabled_at: u64,
    pub(crate) irq_deadline: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PolyWait {
    #[default]
    LoadRequest,
    TestRequest,
    LoadUpload,
    TestUpload,
    Ready,
}

impl PolyWait {
    pub(crate) fn ready(self) -> bool {
        self == Self::Ready
    }
    fn pc(self) -> u16 {
        match self {
            Self::LoadRequest => 0xf81d,
            Self::TestRequest => 0xf81f,
            Self::LoadUpload => 0xf821,
            Self::TestUpload => 0xf823,
            Self::Ready => 0xf825,
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> u64 {
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x30),
            (9, self.pc(), 0x1f00, 9, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        *self = match *self {
            Self::LoadRequest | Self::LoadUpload => {
                let value = w.dp_read(
                    if *self == Self::LoadRequest {
                        0x1f00
                    } else {
                        0x1f0c
                    },
                    1,
                );
                w.load(value, 1);
                if *self == Self::LoadRequest {
                    Self::TestRequest
                } else {
                    Self::TestUpload
                }
            }
            Self::TestRequest => {
                if w.branch(w.cpu.p & super::nmi_stack::ZERO != 0) {
                    Self::LoadRequest
                } else {
                    Self::LoadUpload
                }
            }
            Self::TestUpload => {
                if w.branch(w.cpu.p & super::nmi_stack::ZERO == 0) {
                    Self::LoadRequest
                } else {
                    Self::Ready
                }
            }
            Self::Ready => panic!("poly render worker needs its retained owner"),
        };
        w.cpu.pc = self.pc();
        w.cycles
    }
}

// The threaded handler intentionally omits the non-threaded mode/mosaic
// stores. This is its source-ordered list of live operands, not a snapshot.
const REGISTERS: &[(u16, u16)] = &[
    (0x96, 0x2123),
    (0x97, 0x2124),
    (0x98, 0x2125),
    (0x99, 0x2130),
    (0x9a, 0x2131),
    (0x9c, 0x2132),
    (0x9d, 0x2132),
    (0x9e, 0x2132),
    (0x1c, 0x212c),
    (0x1d, 0x212d),
    (0x1e, 0x212e),
    (0x1f, 0x212f),
    (0x120, 0x210d),
    (0x121, 0x210d),
    (0x124, 0x210e),
    (0x125, 0x210e),
    (0x11e, 0x210f),
    (0x11f, 0x210f),
    (0x122, 0x2110),
    (0x123, 0x2110),
    (0xe4, 0x2111),
    (0xe5, 0x2111),
    (0xea, 0x2112),
    (0xeb, 0x2112),
    (0x13, 0x2100),
    (0x9b, 0x420c),
];

impl PolyInterrupt {
    pub(crate) fn nmi() -> Self {
        Self {
            phase: Phase::Gate(0),
            irq_return: false,
            swap_stack: true,
            vertical_timer: 0,
            irq_deadline: None,
        }
    }
    /// The IRQ prologue has already saved the running thread's registers.
    pub(crate) fn irq_return() -> Self {
        Self {
            phase: Phase::AcknowledgeIrq,
            irq_return: true,
            swap_stack: true,
            vertical_timer: 0,
            irq_deadline: None,
        }
    }
    pub(crate) fn irq() -> Self {
        Self {
            phase: Phase::IrqPrologue(0),
            ..Self::irq_return()
        }
    }
    pub(crate) fn irq_deadline(&self) -> Option<u64> {
        self.irq_deadline
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    fn pc(&self) -> u16 {
        match self.phase {
            Phase::Gate(i) => [0x8144, 0x8147, 0x8149][usize::from(i)],
            Phase::CallUpload => 0x822d,
            Phase::Upload(i) => [
                0x9347, 0x934a, 0x934c, 0x934e, 0x9351, 0x9353, 0x9356, 0x9359, 0x935c, 0x935f,
                0x9362, 0x9365, 0x9367, 0x936a, 0x936d, 0x9370, 0x9372, 0x9374, 0x9377, 0x937a,
            ][usize::from(i)],
            Phase::IrqSetup(i) => [0x8230, 0x8232, 0x8235, 0x8238, 0x823a][usize::from(i)],
            Phase::Registers(i) => {
                let pair = usize::from(i / 2);
                0x823d
                    + REGISTERS[..pair]
                        .iter()
                        .map(|(a, _)| if *a < 256 { 5 } else { 6 })
                        .sum::<u16>()
                    + if i & 1 == 0 {
                        0
                    } else if REGISTERS[pair].0 < 256 {
                        2
                    } else {
                        3
                    }
            }
            Phase::AcknowledgeIrq => 0x831f,
            Phase::MainIrq(pc) => pc,
            Phase::IrqPrologue(i) => [
                0x82d8, 0x82d9, 0x82db, 0x82dc, 0x82dd, 0x82de, 0x82df, 0x82e0, 0x82e1, 0x82e2,
                0x82e4, 0x82e7,
            ][usize::from(i)],
            Phase::Restore(i) => {
                let base = if !self.swap_stack {
                    0x830e
                } else if self.irq_return {
                    0x8322
                } else {
                    0x82c7
                };
                base + [0, 2, 3, 4, 7, 8, 11, 12, 13, 14, 15, 16][usize::from(i)]
            }
            Phase::Complete => unreachable!(),
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl WorkBus,
    ) -> Option<u64> {
        self.step_on_bus(cpu, clock, dma, bus)
    }
    pub(crate) fn step_with_beam(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl FnMut(u64, Access, Option<snes::cpu_timeline::CpuSynchronousBeamPosition>) -> u16,
    ) -> Option<u64> {
        self.step_on_bus(cpu, clock, dma, BeamBus::new(bus))
    }
    pub(crate) fn step_on_bus(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl WorkBus,
    ) -> Option<u64> {
        use Phase::*;
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc), (0, self.pc()));
        if !matches!(self.phase, Restore(7..=11) | IrqPrologue(0..=8)) {
            assert_eq!(cpu.db, 0);
        }
        // PLB/PLD in the restore tail replace the handler's data context.
        // No direct-page operands follow those pulls.
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.phase = match self.phase {
            Gate(i) => {
                match i {
                    0 => {
                        w.drain(16);
                        let v = w.read(0x12a, 1);
                        w.load(v, 1);
                    }
                    1 => {
                        assert_ne!(w.cpu.a & 255, 0, "poly NMI requires an active thread");
                        w.branch(false);
                    }
                    2 => w.drain(16),
                    _ => unreachable!(),
                }
                if i == 2 {
                    CallUpload
                } else {
                    Gate(i + 1)
                }
            }
            CallUpload => {
                w.drain(16);
                w.drain(6);
                w.push(0x822f, 2);
                Upload(0)
            }
            Upload(i) => {
                let mut next = Upload(i + 1);
                match i {
                    0 => {
                        w.drain(16);
                        let v = w.read(0x1f0c, 1);
                        w.load(v, 1);
                    }
                    1 => {
                        if w.branch(w.cpu.a & 255 == 0) {
                            next = Upload(19);
                        }
                    }
                    2 => w.immediate(0x80, 1),
                    3 => {
                        let value = w.cpu.a & 255;
                        store_io(&mut w, dma, 0x2115, value, 1);
                    }
                    4 => w.width(0x20, true),
                    5 => w.immediate(0x5800, 2),
                    6 | 8 | 10 | 14 => {
                        let address = match i {
                            6 => 0x2116,
                            8 => 0x4300,
                            10 => 0x4302,
                            _ => 0x4305,
                        };
                        let value = w.cpu.a;
                        store_io(&mut w, dma, address, value, 2);
                    }
                    7 => w.immediate(0x1801, 2),
                    9 => w.immediate(0xe800, 2),
                    11 => {
                        w.drain(8);
                        w.cpu.x = 0x7e;
                        w.flags(0x7e, 1);
                    }
                    12 => {
                        let value = w.cpu.x;
                        store_io(&mut w, dma, 0x4304, value, 1);
                    }
                    13 => w.immediate(0x800, 2),
                    15 => w.width(0x20, false),
                    16 => w.immediate(1, 1),
                    17 => {
                        w.drain(16);
                        w.bus.access(
                            w.clock.clock_master_cycles(),
                            Access::Write(0x420b, w.cpu.a & 255, 1),
                        );
                        let mut events = Vec::new();
                        let bus = &mut w.bus;
                        dma.run_synchronous_forward_ppu_with_clock(
                            w.clock,
                            w.cpu.a as u8,
                            |clock, _, source, register| {
                                bus.observe_clock(clock);
                                let at = clock.clock_master_cycles();
                                let value = bus.access(at, Access::Read(source, 1));
                                bus.access(
                                    at,
                                    Access::Write(0x2100 + u32::from(register), value, 1),
                                );
                            },
                            |event, _| events.push(event),
                        );
                        // DMA and refresh belong to hardware accounting;
                        // the CPU instruction owns its six-clock I/O write.
                        w.cycles += 6;
                        for event in events {
                            if let CpuSynchronousTimelineEvent::HMax {
                                completed_field_index,
                                completed_scanline: 224,
                                event_timestamp,
                                ..
                            } = event
                            {
                                w.bus.access(
                                    event_timestamp.master_cycles(),
                                    Access::Vblank {
                                        field: completed_field_index,
                                        at: event_timestamp.master_cycles(),
                                    },
                                );
                            }
                        }
                    }
                    18 => {
                        w.drain(16);
                        w.write(0x1f0c, 0, 1);
                    }
                    19 => {
                        w.rts();
                        assert_eq!(w.cpu.pc, 0x8230);
                        next = IrqSetup(0);
                    }
                    _ => unreachable!(),
                }
                next
            }
            IrqSetup(i) => {
                match i {
                    0 => {
                        let v = w.dp_read(0xff, 1);
                        w.load(v, 1);
                    }
                    1 => {
                        let value = w.cpu.a & 255;
                        self.vertical_timer = value;
                        store_io(&mut w, dma, 0x4209, value, 1);
                    }
                    4 => {
                        w.drain(16);
                        self.irq_deadline = snes::cpu_timeline::vertical_irq_deadline(
                            w.clock,
                            self.vertical_timer,
                            true,
                        )
                        .map(|at| at.master_cycles());
                        w.write_io(0x4200, w.cpu.a & 255, 1);
                    }
                    2 => store_io(&mut w, dma, 0x420a, 0, 1),
                    3 => w.immediate(0xa1, 1),
                    _ => unreachable!(),
                }
                if i == 4 {
                    Registers(0)
                } else {
                    IrqSetup(i + 1)
                }
            }
            Registers(i) => {
                let (source, target) = REGISTERS[usize::from(i / 2)];
                if i & 1 == 0 {
                    w.drain(if source < 256 { 8 } else { 16 });
                    let value = w.read(u32::from(source), 1);
                    w.load(value, 1);
                } else {
                    let value = w.cpu.a & 255;
                    store_io(&mut w, dma, target, value, 1);
                }
                if usize::from(i + 1) == REGISTERS.len() * 2 {
                    Restore(0)
                } else {
                    Registers(i + 1)
                }
            }
            AcknowledgeIrq => {
                w.drain(16);
                let value = w.read_io(0x4211, 1);
                w.load(value, 1);
                Restore(0)
            }
            IrqPrologue(i) => {
                match i {
                    0 => {
                        w.drain(6);
                        w.cpu.p |= 4;
                    }
                    1 => w.width(0x30, true),
                    2..=7 => {
                        w.drain(6);
                        let (value, width) = match i {
                            2 => (w.cpu.a, 2),
                            3 => (w.cpu.x, 2),
                            4 => (w.cpu.y, 2),
                            5 => (w.cpu.d, 2),
                            6 => (u16::from(w.cpu.db), 1),
                            _ => (u16::from(w.cpu.bank), 1),
                        };
                        w.push(value, width);
                    }
                    8 => {
                        w.drain(12);
                        w.cpu.db = w.pull(1) as u8;
                        w.flags(u16::from(w.cpu.db), 1);
                    }
                    9 => w.width(0x30, false),
                    10 => {
                        w.drain(16);
                        let value = w.read(0x12a, 1);
                        w.load(value, 1);
                    }
                    11 => {
                        self.swap_stack = w.branch(w.cpu.a & 255 != 0);
                    }
                    _ => unreachable!(),
                }
                if i == 11 {
                    if self.swap_stack {
                        AcknowledgeIrq
                    } else {
                        MainIrq(0x82e9)
                    }
                } else {
                    IrqPrologue(i + 1)
                }
            }
            MainIrq(pc) => match pc {
                0x82e9 => {
                    w.drain(16);
                    let value = w.read_io(0x4211, 1);
                    w.load(value, 1);
                    MainIrq(0x82ec)
                }
                0x82ec => MainIrq(if w.branch(w.cpu.p & super::nmi_stack::NEGATIVE == 0) {
                    0x8317
                } else {
                    0x82ee
                }),
                0x82ee => {
                    w.drain(16);
                    let value = w.read(0x128, 1);
                    w.load(value, 1);
                    MainIrq(0x82f1)
                }
                0x82f1 => MainIrq(if w.branch(w.cpu.p & super::nmi_stack::ZERO != 0) {
                    0x8317
                } else {
                    0x82f3
                }),
                0x82f3 => {
                    w.drain(16);
                    let value = w.read_io(0x4212, 1);
                    w.cpu.p = (w.cpu.p & !0xc2)
                        | (value as u8 & 0xc0)
                        | if w.cpu.a & value & 255 == 0 {
                            super::nmi_stack::ZERO
                        } else {
                            0
                        };
                    MainIrq(0x82f6)
                }
                0x82f6 => MainIrq(if w.branch(w.cpu.p & 0x40 == 0) {
                    0x82f3
                } else {
                    0x82f8
                }),
                0x82f8 | 0x82fe | 0x830a => {
                    w.drain(16);
                    let value = w.read(
                        match pc {
                            0x82f8 => 0x630,
                            0x82fe => 0x631,
                            _ => 0x128,
                        },
                        1,
                    );
                    w.load(value, 1);
                    MainIrq(pc + 3)
                }
                0x82fb | 0x8301 | 0x8304 | 0x8307 | 0x8314 => {
                    w.drain(16);
                    let zero = matches!(pc, 0x8304 | 0x8307);
                    w.write_io(
                        if pc == 0x8314 {
                            0x4200
                        } else if zero {
                            0x2112
                        } else {
                            0x2111
                        },
                        if zero { 0 } else { w.cpu.a & 255 },
                        1,
                    );
                    MainIrq(pc + 3)
                }
                0x830d => MainIrq(if w.branch(w.cpu.p & super::nmi_stack::NEGATIVE == 0) {
                    0x8317
                } else {
                    0x830f
                }),
                0x830f => {
                    w.drain(16);
                    w.write(0x128, 0, 1);
                    MainIrq(0x8312)
                }
                0x8312 => {
                    w.immediate(0x81, 1);
                    MainIrq(0x8314)
                }
                0x8317 => {
                    w.width(0x30, true);
                    Restore(6)
                }
                _ => unreachable!(),
            },
            Restore(i) => {
                match i {
                    0 => w.width(0x30, true),
                    1 => {
                        w.drain(6);
                        w.load(w.cpu.sp, 2);
                    }
                    2 => w.tax_word(),
                    3 => {
                        w.drain(16);
                        let value = w.read(0x1f0a, 2);
                        w.load(value, 2);
                    }
                    4 => {
                        w.drain(6);
                        w.cpu.sp = w.cpu.a;
                    }
                    5 => {
                        w.drain(16);
                        w.write(0x1f0a, w.cpu.x, 2);
                    }
                    6..=10 => {
                        w.drain(12);
                        let width = if i == 6 { 1 } else { 2 };
                        let value = w.pull(width);
                        w.flags(value, width);
                        match i {
                            6 => w.cpu.db = value as u8,
                            7 => w.cpu.d = value,
                            8 => w.cpu.y = value,
                            9 => w.cpu.x = value,
                            _ => w.cpu.a = value,
                        }
                    }
                    11 => {
                        w.drain(12);
                        w.cpu.p = w.pull(1) as u8;
                        if w.cpu.p & 0x10 != 0 {
                            w.cpu.x &= 255;
                            w.cpu.y &= 255;
                        }
                        w.cpu.pc = w.pull(2);
                        w.cpu.bank = w.pull(1) as u8;
                    }
                    _ => unreachable!(),
                }
                if i == 11 {
                    Complete
                } else {
                    Restore(i + 1)
                }
            }
            Complete => unreachable!(),
        };
        if !self.complete() {
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
}

fn store_io<B: WorkBus>(
    w: &mut Work<'_, B>,
    dma: &mut DmaState,
    address: u16,
    value: u16,
    width: u8,
) {
    w.drain(16);
    if (0x4300..0x4380).contains(&address) {
        for byte in 0..width {
            dma.write_register(address + u16::from(byte), (value >> (8 * byte)) as u8);
        }
    }
    w.write_io(address, value, width);
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};

    #[test]
    fn poly_interrupt_matches_live_stack_dma_registers_and_clock() {
        let mut rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .expect("poly interrupt differential test requires ROM");
        // Test-only reset prelude preserves the handler's already-enabled
        // NMI state. Enabling NMI for the first time inside vblank would
        // create a new register-edge interrupt, unlike this real caller.
        // A fixture vector acknowledges earlier NMIs while waiting for the
        // chosen raster. The actual handler bodies remain untouched.
        rom[..14].copy_from_slice(&[
            0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0xa9, 0x81, 0x8d, 0, 0x42, 0x80, 0xfe,
        ]);
        rom[0x20..0x24].copy_from_slice(&[0xad, 0x10, 0x42, 0x40]);
        rom[0x7fea..0x7fec].copy_from_slice(&0x8020u16.to_le_bytes());
        for irq in [false, true] {
            for pending in [0, 1, 0x80] {
                for (status, wanted) in [
                    (0, 100 * 1364 + 510),
                    (0x30, 225 * 1364 + 1338),
                    (0xff, 357366 + 261 * 1364 + 1300),
                ] {
                    compare_case(&rom, irq, pending, status, wanted, None);
                }
            }
        }
        // Begin after TIMEUP acknowledgement with the ordinary handler's
        // saved stack. Cover both retained and one-shot scroll requests,
        // including reads that wait for HBlank and cross an HMax event.
        for flag in [0, 1, 0x7f, 0x80, 0xff] {
            for cycle in [510, 1074, 1328] {
                compare_case(&rom, true, 0, 0x30, 100 * 1364 + cycle, Some(flag));
            }
        }
    }

    fn compare_case(
        rom: &[u8],
        irq: bool,
        pending: u8,
        status: u8,
        wanted: u64,
        main_flag: Option<u8>,
    ) {
        let mut source = Snes9xColdCpuExecutor::from_lorom_reset(rom).unwrap();
        while source.machine().timestamp().master_cycles() < wanted {
            source.step().unwrap();
        }
        let mut ram = source.machine().snes().ram.clone();
        for (i, b) in ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 7) ^ usize::from(status)) as u8;
        }
        ram[0x12a] = 1;
        ram[0x1f0c] = pending;
        ram[0x9b] = 0;
        ram[0x13] = 0x80;
        ram[0xff] = 255;
        let saved = if irq { 0x1f2usize } else { 0x1f31 };
        ram[0x1f0a..0x1f0c].copy_from_slice(&(saved as u16).to_le_bytes());
        ram[saved + 1..saved + 14].copy_from_slice(&[
            9, 0, 0x1f, 0x76, 0x98, 0x54, 0x32, 0xdc, 0xfe, status, 0x1d, 0xf8, 9,
        ]);
        let mut native = NativeNmiContext {
            a: 0xab00,
            x: 0x93,
            y: 0x87,
            d: if irq { 0x1f00 } else { 0 },
            db: 0,
            bank: 0,
            pc: if irq { 0x831f } else { 0x8144 },
            sp: if irq { 0x1f31 } else { 0x1f2 },
            p: 0x34,
        };
        if let Some(flag) = main_flag {
            ram[0x12a] = 0;
            ram[0x128] = flag;
            native.pc = 0x82ee;
            native.d = 0;
            native.sp = saved as u16;
        }
        let mut registers = source.machine().snes().cpu.clone();
        registers.a = native.a;
        registers.x = native.x;
        registers.y = native.y;
        registers.dp = native.d;
        registers.db = 0;
        registers.k = 0;
        registers.pc = native.pc;
        registers.sp = native.sp;
        registers.e = false;
        registers.unpack_flags(native.p);
        let sram = source.machine().snes().cart.ram.clone();
        source
            .rebase_private_timing_state_at(source.machine().timestamp(), &registers, &ram, &sram)
            .unwrap();
        let mut clock = source.machine().timeline().clone();
        let mut ppu = source.machine().snes().ppu.clone();
        let mut dma = source.machine().snes().dma.clone();
        let mut work = if irq {
            PolyInterrupt::irq_return()
        } else {
            PolyInterrupt::nmi()
        };
        if main_flag.is_some() {
            work.phase = Phase::MainIrq(0x82ee);
            work.swap_stack = false;
        }
        while !work.complete() {
            let label = format!(
                "irq={irq} pending={pending} p={status:02x} wanted={wanted} main_flag={main_flag:?} phase={:?}",
                work.phase
            );
            let operand = u32::from(native.pc) + 1;
            let mut first = true;
            let expected: Vec<_> = source
                .step()
                .unwrap()
                .accesses
                .into_iter()
                .filter_map(|a| {
                    let (access, value) = match a.kind {
                        Kind::Read { value, width } => {
                            let skip = first && a.address == operand;
                            first = false;
                            if skip {
                                return None;
                            }
                            (Access::Read(a.address, width), value)
                        }
                        Kind::Write { value, width } => (Access::Write(a.address, value, width), 0),
                        _ => return None,
                    };
                    Some((a.timestamp.master_cycles(), access, value))
                })
                .collect();
            let is_dma = expected
                .iter()
                .any(|(_, a, _)| matches!(a, Access::Write(0x420b, _, _)));
            let mut actual = Vec::new();
            work.step_with_beam(&mut native, &mut clock, &mut dma, |at, access, beam| {
                let value = match access {
                    Access::Read(address, width) => {
                        if address == 0x4211 {
                            // No IRQ is pending in this fixture. TIMEUP's
                            // low seven bits retain the last operand-fetch
                            // bus value, the high byte of LDA $4211.
                            u16::from(rom[0x0321] & 0x7f)
                        } else if address == 0x4212 {
                            let beam = beam.unwrap();
                            2 | u16::from((225..228).contains(&beam.scanline))
                                | (u16::from(beam.cycles < 4 || beam.cycles >= 1096) << 6)
                                | (u16::from(beam.scanline >= 225) << 7)
                        } else {
                            let offset = if address >= 0x7e0000 {
                                address as usize - 0x7e0000
                            } else {
                                address as usize
                            };
                            if width == 1 {
                                u16::from(ram[offset])
                            } else {
                                u16::from_le_bytes([ram[offset], ram[offset + 1]])
                            }
                        }
                    }
                    Access::Write(address, value, width) => {
                        if (0x2100..0x2140).contains(&address) {
                            assert_eq!(width, 1);
                            ppu.write(address as u8, value as u8);
                        } else if address < 0x2000 {
                            ram[address as usize..address as usize + usize::from(width)]
                                .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                        } else {
                            assert!(matches!(address,0x4200|0x4209..=0x420c|0x4300..=0x4306));
                        }
                        0
                    }
                    Access::Vblank { .. } => return 0,
                };
                if !is_dma || matches!(access, Access::Write(0x420b, _, _)) {
                    actual.push((at, access, value));
                }
                value
            })
            .unwrap();
            assert_eq!(actual, expected, "{label}");
            let c = &source.machine().snes().cpu;
            assert_eq!(
                native,
                NativeNmiContext {
                    a: c.a,
                    x: c.x,
                    y: c.y,
                    d: c.dp,
                    db: c.db,
                    bank: c.k,
                    pc: c.pc,
                    sp: c.sp,
                    p: c.pack_flags()
                },
                "{label}"
            );
            assert_eq!(clock, *source.machine().timeline(), "{label}");
            assert_eq!(ram, source.machine().snes().ram, "{label}");
            assert_eq!(ppu.vram, source.machine().snes().ppu.vram, "{label}");
            for (native, source) in ppu
                .bg_layer
                .iter()
                .zip(&source.machine().snes().ppu.bg_layer)
            {
                assert_eq!(
                    (native.h_scroll, native.v_scroll),
                    (source.h_scroll, source.v_scroll),
                    "{label}"
                );
            }
            assert_eq!(
                (ppu.scroll_prev, ppu.scroll_prev2),
                (
                    source.machine().snes().ppu.scroll_prev,
                    source.machine().snes().ppu.scroll_prev2
                ),
                "{label}"
            );
            assert_eq!(
                bincode::serialize(&dma.channel).unwrap(),
                bincode::serialize(&source.machine().snes().dma.channel).unwrap(),
                "{label}"
            );
            // Continuation carries no borrowed stack/register image.
            work = work.clone();
        }
        assert_eq!(
            (native.bank, native.pc, native.sp),
            (9, 0xf81d, saved as u16 + 13)
        );
        assert_eq!(
            u16::from_le_bytes([ram[0x1f0a], ram[0x1f0b]]),
            if main_flag.is_some() { saved as u16 } else if irq { 0x1f31 } else { 0x1f2 }
        );
    }
}
