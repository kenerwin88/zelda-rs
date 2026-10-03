//! Retained Triforce animation request and step dispatch. Each phase retires
//! one instruction; loaded operands, flags and stack belong to the caller.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

const STEP_ENTRIES: [u16; 6] = [0xc45b, 0xc47b, 0xc4ba, 0xc4d6, 0xc500, 0xc533];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let offset = address.checked_sub(0x0cc44f)? as usize;
    (offset < 12).then(|| STEP_ENTRIES[offset / 2].to_le_bytes()[offset % 2])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Request(u8),
    Dispatch(u8),
    Step { kind: u8, instruction: u8 },
    Rotate { kind: u8, instruction: u8 },
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TriforceAnimation {
    phase: Phase,
    selected: u8,
}
impl Default for TriforceAnimation {
    fn default() -> Self {
        Self {
            phase: Phase::Request(0),
            selected: 0,
        }
    }
}
impl TriforceAnimation {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    fn location(&self) -> (u8, u16) {
        match self.phase {
            Phase::Request(i) => (
                12,
                [
                    0xc435, 0xc437, 0xc43a, 0xc43d, 0xc43f, 0xc442, 0xc444, 0xc447,
                ][i as usize],
            ),
            Phase::Dispatch(i) => {
                if i < 2 {
                    (12, [0xc448, 0xc44b][i as usize])
                } else {
                    (
                        0,
                        [
                            0x8781, 0x8783, 0x8784, 0x8786, 0x8788, 0x878b, 0x878c, 0x878d, 0x878e,
                            0x8790, 0x8791, 0x8793, 0x8795, 0x8797, 0x8799,
                        ][i as usize - 2],
                    )
                }
            }
            Phase::Step {
                kind,
                instruction: i,
            } => (
                12,
                match kind {
                    0 => [0xc45b, 0xc45e, 0xc461, 0xc463, 0xc465][i as usize],
                    1 => [
                        0xc47b, 0xc47e, 0xc480, 0xc482, 0xc485, 0xc488, 0xc48a, 0xc48d, 0xc48e,
                        0xc490, 0xc4a5, 0xc4a8, 0xc4aa, 0xc4ac, 0xc4ae, 0xc4b0, 0xc4b2, 0xc4b4,
                        0xc4b6, 0xc4b9,
                    ][i as usize],
                    2 => [0xc4ba, 0xc4bd, 0xc4bf, 0xc4c2][i as usize],
                    3 => [
                        0xc4d6, 0xc4d9, 0xc4db, 0xc4dd, 0xc4e0, 0xc4e2, 0xc4e4, 0xc4e7, 0xc4e9,
                        0xc4ec,
                    ][i as usize],
                    4 => [
                        0xc500, 0xc503, 0xc506, 0xc509, 0xc50b, 0xc50e, 0xc510, 0xc513, 0xc515,
                        0xc518, 0xc51a, 0xc51c, 0xc51e, 0xc520, 0xc522, 0xc524, 0xc526, 0xc528,
                        0xc52a, 0xc52c, 0xc52e, 0xc530, 0xc532,
                    ][i as usize],
                    5 => 0xc533,
                    _ => unreachable!(),
                },
            ),
            Phase::Rotate {
                kind,
                instruction: i,
            } => (
                12,
                [0xc468, 0xc493, 0xc4c3, 0xc4ed][kind as usize]
                    + [0, 3, 4, 6, 9, 12, 13, 15, 18][i as usize],
            ),
            Phase::Complete => unreachable!(),
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc), self.location());
        assert_eq!(cpu.d, 0);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut next = match self.phase {
            Phase::Request(i) => Phase::Request(i + 1),
            Phase::Dispatch(i) => Phase::Dispatch(i + 1),
            Phase::Step { kind, instruction } => Phase::Step {
                kind,
                instruction: instruction + 1,
            },
            Phase::Rotate { kind, instruction } => Phase::Rotate {
                kind,
                instruction: instruction + 1,
            },
            Phase::Complete => unreachable!(),
        };
        match self.phase {
            Phase::Request(i) => match i {
                0 | 5 => w.immediate(1, 1),
                1 => {
                    let value = w.cpu.a;
                    store(&mut w, 0x12a, value);
                }
                2 => load(&mut w, 0x1f00),
                3 => {
                    if w.branch(w.cpu.p & ZERO == 0) {
                        next = Phase::Request(7);
                    }
                }
                4 => {
                    w.drain(16);
                    w.drain(6);
                    w.push(0xc441, 2);
                    next = Phase::Dispatch(0);
                }
                6 => {
                    let value = w.cpu.a;
                    store(&mut w, 0x1f00, value);
                }
                7 => {
                    w.rts();
                    next = Phase::Complete;
                }
                _ => unreachable!(),
            },
            Phase::Dispatch(i) => match i {
                0 => {
                    load(&mut w, 0x1e00);
                    self.selected = (w.cpu.a & 255) as u8;
                    assert!(self.selected < 6);
                }
                1 => {
                    w.drain(30);
                    w.push(u16::from(w.cpu.bank), 1);
                    w.push(0xc44e, 2);
                }
                2 => w.dp_store(3, w.cpu.y, 1),
                3 => {
                    w.drain(12);
                    w.cpu.y = w.pull(1);
                    w.flags(w.cpu.y, 1);
                }
                4 => w.dp_store(0, w.cpu.y, 1),
                5 => w.width(0x30, true),
                6 => w.mask(255, 2),
                7 => w.shift_left(2),
                8 => {
                    w.drain(6);
                    w.cpu.y = w.cpu.a;
                    w.flags(w.cpu.y, 2);
                }
                9 => {
                    w.drain(12);
                    let value = w.pull(2);
                    w.load(value, 2);
                }
                10 => w.dp_store(1, w.cpu.a, 2),
                11 => w.increment_y(),
                12 => {
                    let address = w.indirect_y_address(0);
                    let value = w.read(address, 2);
                    w.load(value, 2);
                }
                13 => w.dp_store(0, w.cpu.a, 2),
                14 => w.width(0x30, false),
                15 => {
                    w.cpu.y = w.dp_read(3, 1);
                    w.flags(w.cpu.y, 1);
                }
                16 => {
                    w.drain(16);
                    let pc = w.read(0, 2);
                    let bank = w.read(2, 1);
                    assert_eq!((bank, pc), (12, STEP_ENTRIES[self.selected as usize]));
                    next = Phase::Step {
                        kind: self.selected,
                        instruction: 0,
                    };
                }
                _ => unreachable!(),
            },
            Phase::Rotate {
                kind: _,
                instruction: i,
            } => match i {
                0 => load(&mut w, 0x1f05),
                1 | 5 => w.clear_carry(),
                2 | 6 => {
                    w.drain(8);
                    w.adc_byte(if i == 2 { 5 } else { 3 });
                }
                3 | 7 => {
                    let a = w.cpu.a;
                    store(&mut w, if i == 3 { 0x1f05 } else { 0x1f04 }, a);
                }
                4 => load(&mut w, 0x1f04),
                8 => {
                    w.rts();
                    next = Phase::Request(5);
                }
                _ => unreachable!(),
            },
            Phase::Step {
                kind,
                instruction: i,
            } => match kind {
                0 => match i {
                    0 => increment(&mut w, 0x1e01, true),
                    1 => load(&mut w, 0x1e01),
                    2 => w.compare_immediate(64, 1),
                    3 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Phase::Rotate {
                                kind: 0,
                                instruction: 0,
                            };
                        }
                    }
                    4 => {
                        increment(&mut w, 0x1e00, true);
                        next = Phase::Rotate {
                            kind: 0,
                            instruction: 0,
                        };
                    }
                    _ => unreachable!(),
                },
                1 => match i {
                    0 | 10 => load(&mut w, 0x1f02),
                    1 => w.compare_immediate(2, 1),
                    2 => {
                        if w.branch(w.cpu.p & CARRY != 0) {
                            next = Phase::Step {
                                kind: 1,
                                instruction: 8,
                            };
                        }
                    }
                    3 => store(&mut w, 0x1f02, 0),
                    4 => increment(&mut w, 0x1e00, true),
                    5 => w.immediate(64, 1),
                    6 => {
                        let a = w.cpu.a;
                        store(&mut w, 0x1e01, a);
                    }
                    7 | 19 => {
                        w.rts();
                        next = Phase::Request(5);
                    }
                    8 => {
                        w.drain(8);
                        w.sbc(2, 1);
                    }
                    9 => {
                        let a = w.cpu.a;
                        store(&mut w, 0x1f02, a);
                        next = Phase::Rotate {
                            kind: 1,
                            instruction: 0,
                        };
                    }
                    11 => w.compare_immediate(225, 1),
                    12 => {
                        if w.branch(w.cpu.p & CARRY != 0) {
                            next = Phase::Step {
                                kind: 1,
                                instruction: 15,
                            };
                        }
                    }
                    13 => {
                        w.drain(8);
                        w.cpu.x = 4;
                        w.flags(4, 1);
                    }
                    14 => w.dp_store(0x11, w.cpu.x, 1),
                    15 => w.compare_immediate(113, 1),
                    16 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Phase::Step {
                                kind: 1,
                                instruction: 19,
                            };
                        }
                    }
                    17 => w.immediate(1, 1),
                    18 => {
                        let a = w.cpu.a;
                        store(&mut w, 0x12c, a);
                    }
                    _ => unreachable!(),
                },
                2 => match i {
                    0 => increment(&mut w, 0x1e01, false),
                    1 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Phase::Rotate {
                                kind: 2,
                                instruction: 0,
                            };
                        }
                    }
                    2 => increment(&mut w, 0x1e00, true),
                    3 => {
                        w.rts();
                        next = Phase::Request(5);
                    }
                    _ => unreachable!(),
                },
                3 => match i {
                    0 => load(&mut w, 0x1f05),
                    1 => w.compare_immediate(250, 1),
                    2 | 5 => {
                        if w.branch(w.cpu.p & CARRY == 0) {
                            next = Phase::Rotate {
                                kind: 3,
                                instruction: 0,
                            };
                        }
                    }
                    3 => load(&mut w, 0x1f04),
                    4 => w.compare_immediate(252, 1),
                    6 => increment(&mut w, 0x1e00, true),
                    7 => w.immediate(32, 1),
                    8 => {
                        let a = w.cpu.a;
                        store(&mut w, 0x1e01, a);
                    }
                    9 => {
                        w.rts();
                        next = Phase::Request(5);
                    }
                    _ => unreachable!(),
                },
                4 => match i {
                    0 | 1 => store(&mut w, if i == 0 { 0x1f05 } else { 0x1f04 }, 0),
                    2 => increment(&mut w, 0x1e01, false),
                    3 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Phase::Step {
                                kind: 4,
                                instruction: 22,
                            };
                        }
                    }
                    4 => increment(&mut w, 0x1e00, true),
                    5 | 7 | 9 | 11 | 13 | 15 | 19 => w.immediate(
                        match i {
                            5 => 1,
                            7 | 19 => 3,
                            9 => 0x10,
                            11 => 5,
                            13 => 2,
                            15 => 0x31,
                            _ => unreachable!(),
                        },
                        1,
                    ),
                    6 | 8 => {
                        let a = w.cpu.a;
                        store(&mut w, if i == 6 { 0x1e15 } else { 0x1e1d }, a);
                    }
                    10 | 12 | 14 | 16 | 20 => w.dp_store(
                        match i {
                            10 => 0x1c,
                            12 => 0x1d,
                            14 => 0x99,
                            16 => 0x9a,
                            20 => 0x14,
                            _ => unreachable!(),
                        },
                        w.cpu.a & 255,
                        1,
                    ),
                    17 => w.dp_store(0xb0, 0, 1),
                    18 | 21 => w.dp_increment(if i == 18 { 0x15 } else { 0x11 }, 1, true),
                    22 => {
                        w.rts();
                        next = Phase::Request(5);
                    }
                    _ => unreachable!(),
                },
                5 => {
                    w.rts();
                    next = Phase::Request(5);
                }
                _ => unreachable!(),
            },
            Phase::Complete => unreachable!(),
        }
        // Step 1's rotation falls through to its size/music gates, unlike
        // the other three identical arithmetic blocks which end in RTS.
        if self.phase
            == (Phase::Rotate {
                kind: 1,
                instruction: 7,
            })
        {
            next = Phase::Step {
                kind: 1,
                instruction: 10,
            };
        }
        self.phase = next;
        if !self.complete() {
            (w.cpu.bank, w.cpu.pc) = self.location();
        }
        Some(w.cycles)
    }
}
fn load<B: WorkBus>(w: &mut Work<'_, B>, address: u16) {
    w.drain(16);
    let value = w.read(w.absolute(address), 1);
    w.load(value, 1);
}
fn store<B: WorkBus>(w: &mut Work<'_, B>, address: u16, value: u16) {
    w.drain(16);
    w.write(w.absolute(address), value & 255, 1);
}
fn increment<B: WorkBus>(w: &mut Work<'_, B>, address: u16, up: bool) {
    w.drain(16);
    let old = w.read(w.absolute(address), 1);
    let value = if up {
        old.wrapping_add(1)
    } else {
        old.wrapping_sub(1)
    } & 255;
    w.drain(6);
    w.flags(value, 1);
    w.write(w.absolute(address), value, 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn retained_triforce_request_and_all_steps_match_rom_bus_registers_and_clock() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x0cc44f..0x0cc45b {
            assert_eq!(table_byte(a), Some(rom[0x60000 + (a as usize & 0x7fff)]));
        }
        let mut instructions = 0;
        for kind in 0..6 {
            for seed in 0..=255u8 {
                compare_case(&rom, kind, seed, 0, &mut instructions);
            }
        }
        // Busy requests return before reading the step selector, including
        // selectors which could not be dispatched safely.
        for busy in [1, 0x80, 0xff] {
            compare_case(&rom, 0xff, 0xff, busy, &mut instructions);
        }
        eprintln!(
            "1539 Triforce cases, {instructions} independently checked instruction boundaries"
        );
    }
    fn compare_case(rom: &[u8], kind: u8, seed: u8, busy: u8, instructions: &mut usize) {
        let mut cpu = NativeNmiContext {
            a: 0xab00 | u16::from(seed),
            x: 0x52,
            y: 0x93,
            d: 0,
            db: if seed & 1 == 0 { 12 } else { 0 },
            bank: 12,
            pc: 0xc435,
            sp: 0x1fa,
            p: 0x30 | (seed & 0xc3),
        };
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 8) ^ usize::from(seed)) as u8;
        }
        machine.ram[0x1e00] = kind;
        machine.ram[0x1e01] = seed;
        machine.ram[0x1f00] = busy;
        machine.ram[0x1f02] = seed;
        machine.ram[0x1f04] = seed;
        machine.ram[0x1f05] = seed;
        machine.ram[0x1fb..0x1fd].copy_from_slice(&0xc40cu16.to_le_bytes());
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.dp = cpu.d;
        machine.cpu.db = cpu.db;
        machine.cpu.k = cpu.bank;
        machine.cpu.pc = cpu.pc;
        machine.cpu.sp = cpu.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(cpu.p);
        let mut ram = machine.ram.clone();
        let (v, h) = match seed % 3 {
            0 => (100, 510),
            1 => (224, 1340),
            _ => (261, 1320),
        };
        let mut clock = CpuMasterTimeline::at_raster(
            0,
            CpuRasterPosition::new(v, h),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source =
            RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                .unwrap();
        clock.begin_synchronous_timeline().unwrap();
        let mut work = TriforceAnimation::default();
        let mut count = 0;
        while !work.complete() {
            let label = format!("kind={kind} seed={seed} busy={busy} phase={:?}", work.phase);
            super::super::native_work::compare_instruction(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                rom,
                |cpu, clock, bus| {
                    work.step(cpu, clock, bus).unwrap();
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            // Retaining this value and CPU is sufficient to resume; no
            // completed stores or loaded operands are reconstructed.
            work = work.clone();
            count += 1;
            *instructions += 1;
            assert!(count < 100);
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (12, 0xc40d, 0x1fc));
        assert_eq!(ram[0x12a], 1);
        assert_eq!(ram[0x1f00], if busy == 0 { 1 } else { busy });
        assert!(work
            .step(&mut cpu, &mut clock, |_, _| panic!(
                "completed caller touched bus"
            ))
            .is_none());
    }
}
