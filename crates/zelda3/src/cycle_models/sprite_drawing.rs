//! Retained Uncle drawing and the shared multi-record, coordinate and shadow
//! routines. Calls and double returns consume the inherited stack; source data
//! is read at its live bus access, including wrapped WRAM drawing pointers.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    oam_allocate::OamAllocate,
};
use snes::CpuMasterTimeline;

// The byte-indexed equipment lookup can address beyond its nominal table.
// These immutable adjacent bytes are data for that lookup, never instructions
// decoded or executed by this owner. The entire indexed window is ROM-tested.
const DMA_INDEX_TAIL: [u8; 248] = [
    0x4b, 0xab, 0xa9, 0x18, 0x22, 0x84, 0xba, 0x0d, 0xc2, 0x20, 0xbd, 0xc0, 0x0d, 0x29, 0xff, 0x00,
    0x85, 0x00, 0xbd, 0xe0, 0x0d, 0x29, 0xff, 0x00, 0x85, 0x02, 0x0a, 0x65, 0x02, 0x65, 0x00, 0x0a,
    0x65, 0x00, 0x0a, 0x0a, 0x0a, 0x0a, 0x69, 0x03, 0xd2, 0x85, 0x08, 0xa9, 0x06, 0x00, 0x85, 0x06,
    0xe2, 0x30, 0xbd, 0xe0, 0x0d, 0x0a, 0x7d, 0xc0, 0x0d, 0xa8, 0xb9, 0x83, 0xd3, 0x8d, 0x07, 0x01,
    0xb9, 0x8a, 0xd3, 0x8d, 0x08, 0x01, 0x22, 0x70, 0xdf, 0x05, 0xbd, 0xe0, 0x0d, 0xf0, 0x08, 0xc9,
    0x03, 0xf0, 0x04, 0x22, 0x54, 0xdc, 0x06, 0xab, 0x6b, 0x04, 0x00, 0x00, 0x00, 0x27, 0x00, 0x00,
    0x00, 0x00, 0x00, 0xfb, 0xff, 0x0e, 0x00, 0x00, 0x02, 0xf8, 0xff, 0x06, 0x00, 0x0a, 0x04, 0x00,
    0x02, 0x08, 0x00, 0x06, 0x00, 0x0a, 0x44, 0x00, 0x02, 0xf8, 0xff, 0x0e, 0x00, 0x0a, 0x84, 0x00,
    0x02, 0x08, 0x00, 0x0e, 0x00, 0x0a, 0xc4, 0x00, 0x02, 0x00, 0x00, 0xfb, 0xff, 0x0e, 0x00, 0x00,
    0x02, 0x00, 0x00, 0xfb, 0xff, 0x0e, 0x00, 0x00, 0x02, 0xf8, 0xff, 0x06, 0x00, 0x0a, 0x04, 0x00,
    0x02, 0x08, 0x00, 0x06, 0x00, 0x0a, 0x44, 0x00, 0x02, 0xf8, 0xff, 0x0e, 0x00, 0x0a, 0x84, 0x00,
    0x02, 0x08, 0x00, 0x0e, 0x00, 0x0a, 0xc4, 0x00, 0x02, 0x00, 0x00, 0xfb, 0xff, 0x2e, 0x00, 0x00,
    0x02, 0x00, 0x00, 0xfb, 0xff, 0x2e, 0x00, 0x00, 0x02, 0xf8, 0xff, 0x07, 0x00, 0x0a, 0x04, 0x00,
    0x02, 0x08, 0x00, 0x07, 0x00, 0x0a, 0x44, 0x00, 0x02, 0xf8, 0xff, 0x0e, 0x00, 0x0a, 0x84, 0x00,
    0x02, 0x08, 0x00, 0x0e, 0x00, 0x0a, 0xc4, 0x00, 0x02, 0x8b, 0x4b, 0xab, 0xa9, 0x06, 0x85, 0x06,
    0x64, 0x07, 0xbd, 0xc0, 0x0d, 0x0a, 0x7d, 0xc0,
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    if (0x0dd392..0x0dd48a).contains(&address) {
        return Some(DMA_INDEX_TAIL[(address - 0x0dd392) as usize]);
    }
    crate::ZeldaState::native_uncle_draw_table_byte(address)
        .or_else(|| crate::ZeldaState::native_oam_table_byte(address))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uncle_drawing_and_shared_helpers_match_source_live_pointers_and_double_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x0dd203..0x0dd48a {
            assert_eq!(
                table_byte(a),
                Some(rom[(a >> 16) as usize * 0x8000 + (a as usize & 0x7fff)]),
                "table {a:06x}"
            );
        }
        let mut instructions = 0;
        let mut completed = 0;
        let mut killed = 0;
        for entry in [0x0dd391, 0x06e416, 0x06dc54, 0x06dc5c] {
            for slot in 0..16 {
                for case in 0..320 {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    for (i, byte) in machine.ram.iter_mut().enumerate() {
                        *byte = (i as u8).wrapping_mul(17).wrapping_add(case as u8);
                    }
                    let ram = &mut machine.ram;
                    let sx = [0u16, 0xffff, 0x100, 0x8000][case % 4];
                    let sy = [0u16, 0x80, 0xffff, 0x7fff][case / 4 % 4];
                    let px = sx.wrapping_add([0u16, 63, 239, 255, 256, 303, 304, 0xffbf][case % 8]);
                    let py = sy.wrapping_add(
                        [0u16, 239, 240, 255, 304, 0xffc0, 0xffbf, 128][case / 8 % 8],
                    );
                    ram[0xd10 + slot] = px as u8;
                    ram[0xd30 + slot] = (px >> 8) as u8;
                    ram[0xd00 + slot] = py as u8;
                    ram[0xd20 + slot] = (py >> 8) as u8;
                    for (a, value) in [
                        (0xe2, sx),
                        (0xe8, sy),
                        (0xfd8, px),
                        (0xfda, py),
                        (0x90, 0x840),
                        (0x92, 0xa30),
                        (0xfe2, [0, 0x1f0, 0x200, 0x201][case % 4]),
                        (0xfee, case as u16 & 7),
                    ] {
                        ram[a..a + 2].copy_from_slice(&value.to_le_bytes());
                    }
                    ram[0xde0 + slot] = [0, 1, 2, 3, 0xbd][case / 64];
                    ram[0xdc0 + slot] = (case / 32 % 2) as u8;
                    ram[0xdd0 + slot] = [8, 9, 10, 11][case / 8 % 4];
                    ram[0xf70 + slot] = [0, 16, 128, 255][case % 4];
                    ram[0xf60 + slot] = if case & 16 == 0 { 0 } else { 32 };
                    ram[0xcaa + slot] = if case & 32 == 0 { 0x80 } else { 0 };
                    ram[0xe40 + slot] = (case & 31) as u8;
                    ram[0xe60 + slot] = (case & 32) as u8;
                    ram[0xf00 + slot] = (case & 1) as u8;
                    ram[0x1fa1c + slot] = [0, 3, 11, 255][case % 4];
                    ram[0x1fa2c + slot] = [9, 10, 11, 255][case / 4 % 4];
                    ram[0x1fa3c + slot] = (case & 1) as u8;
                    ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 5]);
                    let mut cpu = NativeNmiContext {
                        a: 0xab00 | (case as u16 & 255),
                        x: slot as u16,
                        y: 0x51,
                        d: 0,
                        db: 5,
                        bank: (entry >> 16) as u8,
                        pc: entry as u16,
                        sp: 0x1fc,
                        p: 0x30 | (case as u8 & 0xc3),
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.db = cpu.db;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let clock = CpuMasterTimeline::at_raster(
                        [0, 1, 30002, 30003][case % 4],
                        snes::CpuRasterPosition::new(224, 1340),
                        snes::CpuBusWorkload::default(),
                        snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                    );
                    let mut source = snes::RomCpuTimingProbe::new(
                        machine,
                        clock,
                        snes::SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    let mut clock = source.timeline().clone();
                    let mut work = SpriteDrawing::new(entry);
                    for _ in 0..2000 {
                        if work.complete() || work.child().is_some() {
                            break;
                        }
                        let label = format!(
                            "entry={entry:06x} slot={slot} case={case} pc={:06x}",
                            work.pc
                        );
                        super::super::native_work::compare_instruction(
                            &label,
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        instructions += 1;
                    }
                    assert!(work.complete() || work.child().is_some());
                    assert_eq!(ram, source.snes().ram);
                    if work.complete() {
                        completed += 1;
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (5, 0x805a, 0x1ff));
                    } else {
                        killed += 1;
                        assert_eq!(work.child(), Some(DrawingChild::KillSelf));
                        assert_eq!((cpu.bank, cpu.pc), (9, 0xf1f8));
                    }
                }
            }
        }
        assert!(completed > 0 && killed > 0);
        eprintln!("20480 drawing fixtures, {instructions} instruction boundaries, completed={completed}, kill handoffs={killed}");
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrawingChild {
    KillSelf,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Call {
    returned: u32,
    far: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpriteDrawing {
    pc: u32,
    calls: Vec<Call>,
    allocator: Option<OamAllocate>,
    child: Option<DrawingChild>,
    complete: bool,
}
impl SpriteDrawing {
    pub(crate) fn new(pc: u32) -> Self {
        assert!(matches!(
            pc,
            0x0dd391 | 0x05df70 | 0x06e416 | 0x06dc54 | 0x06dc5c
        ));
        Self {
            pc,
            calls: Vec::new(),
            allocator: None,
            child: None,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child(&self) -> Option<DrawingChild> {
        self.child
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        assert!(self.child.take().is_some());
        self.accept_return(cpu, true);
    }
    fn accept_return(&mut self, cpu: &NativeNmiContext, far: bool) {
        let returned = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
        if let Some(call) = self.calls.pop() {
            assert_eq!(call, Call { returned, far });
        } else {
            self.complete = true;
        }
        self.pc = returned;
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert!(self.child.is_none(), "drawing child still owns the CPU");
        if let Some(allocator) = self.allocator.as_mut() {
            let cycles = allocator.step(cpu, clock, bus);
            if allocator.complete() {
                assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
                self.allocator = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 8),
            ((self.pc >> 16) as u8, self.pc as u16, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match pc {
            0x0dd391 | 0x0dd392 | 0x06dc54 | 0x06dc55 | 0x06dc5c | 0x06dc5d => {
                w.drain(6);
                w.push(
                    u16::from(if matches!(pc, 0x0dd391 | 0x06dc54 | 0x06dc5c) {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                pc + 1
            }
            0x0dd393 | 0x0dd3e9 | 0x06dc56 | 0x06dc5a | 0x06dc5e | 0x06dc62 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0x0dd394 | 0x0dd3bd | 0x05dfa3 | 0x06dc64 | 0x06dcb7 | 0x06dcd4 => {
                w.immediate(
                    match pc {
                        0x0dd394 => 24,
                        0x0dd3bd => 6,
                        0x05dfa3 => 0xf0,
                        0x06dc64 => 10,
                        0x06dcb7 => 0x38,
                        _ => 0x6c,
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x0dd396 => {
                w.operand_then_internal(3);
                w.push(13, 1);
                w.push(pc as u16 + 3, 2);
                w.cpu.pc = 0xba84;
                self.allocator = Some(OamAllocate::new(0xba84));
                pc + 4
            }
            0x0dd3d8 | 0x0dd3e5 | 0x05df70 | 0x05dfe9 | 0x05fa50 | 0x06e416 | 0x06e41a
            | 0x06e48d | 0x06dc57 | 0x06dc5f => {
                let (target, far) = match pc {
                    0x0dd3d8 => (0x05df70, true),
                    0x0dd3e5 => (0x06dc54, true),
                    0x05df70 => (0x05dfe9, false),
                    0x05dfe9 => (0x05fa50, false),
                    0x05fa50 => (0x06e416, true),
                    0x06e416 => (0x06e41a, false),
                    0x06e41a => (0x06e41e, false),
                    0x06e48d => (0x09f1f8, true),
                    0x06dc57 => (0x06dc64, false),
                    _ => (0x06dc66, false),
                };
                let width = if far { 3 } else { 2 };
                w.operand_then_internal(width);
                if far {
                    w.push(u16::from(w.cpu.bank), 1);
                }
                w.push(pc as u16 + u16::from(width), 2);
                self.calls.push(Call {
                    returned: pc + 1 + u32::from(width),
                    far,
                });
                if pc == 0x06e48d {
                    self.child = Some(DrawingChild::KillSelf);
                }
                target
            }
            0x0dd39a | 0x0dd3c2 | 0x05df7b | 0x05dfc9 | 0x05dfd6 | 0x05dfe1 | 0x06e421
            | 0x06e432 | 0x06e43d | 0x06e453 | 0x06e476 | 0x06e483 | 0x06dc87 | 0x06dc97 => {
                w.width(
                    if matches!(pc, 0x0dd3c2 | 0x05df7b | 0x05dfe1) {
                        0x30
                    } else {
                        0x20
                    },
                    matches!(
                        pc,
                        0x0dd39a | 0x05df7b | 0x05dfd6 | 0x06e421 | 0x06e43d | 0x06e476 | 0x06dc87
                    ),
                );
                pc + 2
            }
            0x0dd39c | 0x0dd3a4 | 0x0dd3c4 | 0x0dd3c8 | 0x0dd3cc | 0x0dd3d2 | 0x0dd3dc
            | 0x05dff3 | 0x06e436 | 0x06e457 | 0x06e45f | 0x06e462 | 0x06e488 | 0x06dc67
            | 0x06dc6c | 0x06dc73 | 0x06dc78 | 0x06dc9b | 0x06dca7 => {
                let (base, index) = match pc {
                    0x0dd39c | 0x0dd3c8 => (0xdc0, w.cpu.x),
                    0x0dd3a4 | 0x0dd3c4 | 0x0dd3dc => (0xde0, w.cpu.x),
                    0x0dd3cc => (0xd383, w.cpu.y),
                    0x0dd3d2 => (0xd38a, w.cpu.y),
                    0x05dff3 | 0x06dc78 => (0xdd0, w.cpu.x),
                    0x06e436 => (0xf70, w.cpu.x),
                    0x06e457 => (0xf60, w.cpu.x),
                    0x06e45f => (0xf50, w.cpu.x),
                    0x06e462 => (0xb89, w.cpu.x),
                    0x06e488 => (0xcaa, w.cpu.x),
                    0x06dc67 => (0xd00, w.cpu.x),
                    0x06dc6c => (0xd20, w.cpu.x),
                    0x06dc73 => (0xf00, w.cpu.x),
                    0x06dc9b => (0xe40, w.cpu.x),
                    _ => (0xe60, w.cpu.x),
                };
                w.drain(16);
                if x == 2 || (base & 255) + index > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(base) + u32::from(index), m);
                if matches!(pc, 0x0dd3c8 | 0x06dc67) {
                    w.adc_byte(v);
                } else {
                    w.load(
                        if pc == 0x06e462 {
                            (w.cpu.a ^ v) & mask
                        } else {
                            v
                        },
                        m,
                    );
                }
                pc + 3
            }
            0x0dd39f | 0x0dd3a7 | 0x05df8b | 0x05dfb7 | 0x06e45a | 0x06dc9e | 0x06dcaa
            | 0x06dcbd | 0x06dcdb | 0x06dcca | 0x06dce8 => {
                w.mask(
                    match pc {
                        0x0dd39f | 0x0dd3a7 => 255,
                        0x05df8b => 0x100,
                        0x05dfb7 => 0xf1ff,
                        0x06e45a | 0x06dcaa => 32,
                        0x06dc9e => 31,
                        0x06dcbd | 0x06dcdb => 48,
                        _ => 1,
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x0dd3a2 | 0x0dd3aa | 0x0dd3bb | 0x0dd3c0 | 0x05df8e | 0x06e429 | 0x06e439
            | 0x06e43b | 0x06e449 | 0x06e465 | 0x06e467 | 0x06e481 | 0x06dc6a | 0x06dc71
            | 0x06dc8e => {
                let a = match pc {
                    0x0dd3a2 | 0x06e429 => 0,
                    0x0dd3bb => 8,
                    0x0dd3c0 => 6,
                    0x05df8e => 12,
                    0x06e439 | 0x06e467 => 4,
                    0x06e43b | 0x06e465 => 5,
                    0x06dc71 => 3,
                    _ => 2,
                };
                w.dp_store(
                    a,
                    if matches!(pc, 0x06e43b | 0x06e467) {
                        0
                    } else {
                        w.cpu.a & mask
                    },
                    m,
                );
                pc + 2
            }
            0x0dd3ac | 0x0dd3b1 | 0x0dd3b4..=0x0dd3b7 | 0x0dd3c7 | 0x06dca0 | 0x06dca1 => {
                w.shift_left(m);
                pc + 1
            }
            0x0dd3ad | 0x0dd3af | 0x0dd3b2 | 0x05df86 | 0x05df95 | 0x05dfb3 | 0x05dfd0
            | 0x06e427 | 0x06e443 | 0x06e447 | 0x06e47c | 0x06e47f | 0x06dc8c => {
                let a = match pc {
                    0x0dd3ad | 0x05df95 => 2,
                    0x05dfb3 | 0x06e447 | 0x06e47f => 4,
                    0x05dfd0 => 13,
                    0x06e427 => 0xe2,
                    0x06e443 | 0x06e47c | 0x06dc8c => 0xe8,
                    _ => 0,
                };
                let v = w.dp_read(a, m);
                match pc {
                    0x05dfb3 => w.load((w.cpu.a ^ v) & mask, m),
                    0x05dfd0 => w.load((w.cpu.a | v) & mask, m),
                    0x06e427 | 0x06e443 | 0x06e447 | 0x06e47c | 0x06e47f | 0x06dc8c => w.sbc(v, m),
                    _ => w.adc(v),
                }
                pc + 2
            }
            0x0dd3b8 | 0x05df9b | 0x05dfc3 | 0x06e42c | 0x06e44d | 0x06dc6f | 0x06dc91 => {
                w.drain(8 * u32::from(m));
                match pc {
                    0x05dfc3 => w.sbc(0x800, m),
                    0x06dc6f => w.adc_byte(0),
                    _ => w.adc(match pc {
                        0x0dd3b8 => 0xd203,
                        0x06e42c | 0x06e44d => 64,
                        _ => 16,
                    }),
                }
                pc + 1 + u32::from(m)
            }
            0x0dd3cb | 0x05dfc8 | 0x06dca2 | 0x06dcc7 | 0x06dce5 => {
                w.drain(6);
                let v = w.cpu.a & if x == 1 { 255 } else { 65535 };
                if pc == 0x05dfc8 {
                    w.cpu.x = v;
                } else {
                    w.cpu.y = v;
                }
                w.flags(v, x);
                pc + 1
            }
            0x0dd3cf | 0x0dd3d5 | 0x05dfed | 0x05dff0 | 0x05e006 | 0x06e46b | 0x06e470 => {
                let a = match pc {
                    0x0dd3cf => 0x107,
                    0x0dd3d5 => 0x108,
                    0x05dfed | 0x05e006 => 0xcfe,
                    0x05dff0 => 0xcff,
                    0x06e46b => 0xfa8,
                    _ => 0xfa9,
                };
                w.drain(16);
                w.write(
                    w.absolute(a),
                    if matches!(pc, 0x05dfed | 0x05dff0) {
                        0
                    } else {
                        w.cpu.a & mask
                    },
                    m,
                );
                pc + 3
            }
            0x0dd3df | 0x0dd3e3 | 0x05df73 | 0x05df78 | 0x05dfa1 | 0x05dfb5 | 0x05dfdf
            | 0x05dff8 | 0x05e000 | 0x05fa54 | 0x06e434 | 0x06e455 | 0x06e45c | 0x06e48b
            | 0x06e494 | 0x06dc76 | 0x06dc7d | 0x06dc85 | 0x06dc99 | 0x06dcac => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let (take, target) = match pc {
                    0x0dd3df | 0x0dd3e3 => (z, 0x0dd3e9),
                    0x05df73 => (true, 0x05df78),
                    0x05df78 => (c, 0x05dfe4),
                    0x05dfa1 => (!c, 0x05dfa9),
                    0x05dfb5 => (!c, 0x05dfbd),
                    0x05dfdf => (!z, 0x05df83),
                    0x05dff8 => (!z, 0x05dffe),
                    0x05e000 => (!z, 0x05e009),
                    0x05fa54 => (!c, 0x05fa58),
                    0x06e434 => (c, 0x06e476),
                    0x06e455 => (!c, 0x06e45e),
                    0x06e45c => (z, 0x06e485),
                    0x06e48b => (w.cpu.p & NEGATIVE != 0, 0x06e491),
                    0x06e494 => (true, 0x06e45f),
                    0x06dc76 => (!z, 0x06dcce),
                    0x06dc7d => (!z, 0x06dc87),
                    0x06dc85 => (z, 0x06dcce),
                    0x06dc99 => (c, 0x06dcee),
                    _ => (z, 0x06dccf),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0x0dd3e1 | 0x05df9e | 0x05dfae | 0x05dff6 | 0x05dffe | 0x06e42f | 0x06e450
            | 0x06dc7b | 0x06dc83 | 0x06dc94 => {
                w.compare_immediate(
                    match pc {
                        0x0dd3e1 | 0x06dc83 => 3,
                        0x05df9e | 0x06dc94 => 0x100,
                        0x05dfae => 1,
                        0x05dff6 | 0x06dc7b => 10,
                        0x05dffe => 11,
                        _ => 0x170,
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x05df7a | 0x05dfc0 | 0x05dfec | 0x06e445 => {
                w.drain(6);
                let (v, width) = match pc {
                    0x05dfec => (u16::from(w.cpu.p), 1),
                    0x06e445 => (w.cpu.a & mask, m),
                    _ => (w.cpu.x, x),
                };
                w.push(v, width);
                pc + 1
            }
            0x05dfd5 | 0x05dfe3 | 0x05e009 | 0x06e44b | 0x05fa56 | 0x05fa57 | 0x06e491
            | 0x06e492 => {
                w.drain(12);
                let v = w.pull(if matches!(pc, 0x05dfd5 | 0x05dfe3) {
                    x
                } else {
                    m
                });
                if matches!(pc, 0x05dfd5 | 0x05dfe3) {
                    w.cpu.x = v;
                    w.flags(v, x);
                } else if pc == 0x05e009 {
                    w.cpu.p = v as u8;
                    if w.cpu.p & 0x10 != 0 {
                        w.cpu.x &= 255;
                        w.cpu.y &= 255;
                    }
                } else {
                    w.load(v, m);
                }
                // Two byte pulls discard this helper's near return. The later
                // RTS consumes the next caller's frame, exactly as the source.
                if matches!(pc, 0x05fa57 | 0x06e492) {
                    assert!(!self.calls.pop().expect("double-return frame").far);
                }
                pc + 1
            }
            0x05df7d | 0x06e473 => {
                w.drain(8 * u32::from(x));
                w.cpu.y = 0;
                w.flags(0, x);
                pc + 1 + u32::from(x)
            }
            0x05df80 | 0x05dfab | 0x06e423 | 0x06e43f | 0x06e478 => {
                w.drain(16);
                let v = w.read(
                    w.absolute(match pc {
                        0x05df80 => 0x90,
                        0x05dfab => 0xcfe,
                        0x06e423 => 0xfd8,
                        _ => 0xfda,
                    }),
                    if pc == 0x05df80 { x } else { m },
                );
                if pc == 0x05df80 {
                    w.cpu.x = v;
                    w.flags(v, x);
                } else {
                    w.load(v, m);
                }
                pc + 3
            }
            0x05df83 | 0x05df92 | 0x05dfb1 | 0x05dfce | 0x06dcaf => {
                w.drain(8);
                let pointer = w.read(if pc == 0x06dcaf { 0x90 } else { 8 }, 2);
                if x == 2 || (pointer & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(pointer) + u32::from(w.cpu.y), m);
                w.load(v, m);
                pc + 2
            }
            0x05df85 | 0x05df94 | 0x05df9a | 0x06e42b | 0x06e44c | 0x06e45e | 0x06dc66
            | 0x06dc90 => {
                w.clear_carry();
                pc + 1
            }
            0x05dfc2 | 0x06e426 | 0x06e442 | 0x06e446 | 0x06e47b | 0x06e47e | 0x06e493
            | 0x06dc8b => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0x05df88 | 0x05df97 | 0x05dfa6 | 0x05dfbd | 0x05dfd2 | 0x06e41e => {
                let a = match pc {
                    0x05df88 => 0,
                    0x05df97 | 0x05dfa6 => 1,
                    0x05dfbd => 2,
                    0x05dfd2 => 0xa20,
                    _ => 0xf00,
                };
                w.operand_then_internal(2);
                w.write(
                    w.absolute(a) + u32::from(w.cpu.x),
                    if pc == 0x06e41e { 0 } else { w.cpu.a & mask },
                    m,
                );
                pc + 3
            }
            0x05df90
            | 0x05df91
            | 0x05dfa9
            | 0x05dfaa
            | 0x05dfcb..=0x05dfcd
            | 0x05dfd8..=0x05dfdc
            | 0x06dcae
            | 0x06dcb6
            | 0x06dcc1
            | 0x06dcd1
            | 0x06dcd6
            | 0x06dcdf => {
                w.drain(6);
                let v = if (0x05dfd9..=0x05dfdc).contains(&pc) {
                    &mut w.cpu.x
                } else {
                    &mut w.cpu.y
                };
                *v = v.wrapping_add(1) & if x == 1 { 255 } else { 65535 };
                let value = *v;
                w.flags(value, x);
                pc + 1
            }
            0x05dfba | 0x06dcbf | 0x06dcdd | 0x06dcea => {
                w.drain(8 * u32::from(m));
                w.load(
                    (w.cpu.a
                        | match pc {
                            0x05dfba => 0x400,
                            0x06dcea => 2,
                            _ => 8,
                        })
                        & mask,
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x05dfc1 | 0x06dcc4 | 0x06dce2 => {
                w.drain(6);
                w.load(if pc == 0x05dfc1 { w.cpu.x } else { w.cpu.y }, m);
                pc + 1
            }
            0x05dfc6 | 0x05dfc7 | 0x06dcc5 | 0x06dcc6 | 0x06dce3 | 0x06dce4 => {
                w.drain(6);
                let v = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, m);
                pc + 1
            }
            0x05dfdd => {
                w.dp_increment(6, m, false);
                pc + 2
            }
            0x05dffa | 0x05e002 | 0x06dc7f => {
                w.drain(24);
                let a = match pc {
                    0x05dffa => 0x7ffa2c,
                    0x05e002 => 0x7ffa3c,
                    _ => 0x7ffa1c,
                };
                let v = w.read(a + u32::from(w.cpu.x), m);
                w.load(v, m);
                pc + 4
            }
            0x06e469 | 0x06e46e | 0x06dc89 | 0x06dca3 | 0x06dcb1 | 0x06dcbb | 0x06dcc8
            | 0x06dccf | 0x06dcd9 | 0x06dce6 => {
                let a = match pc {
                    0x06e469 | 0x06dca3 => 0,
                    0x06dcbb | 0x06dcd9 => 5,
                    0x06dcc8 | 0x06dce6 => 1,
                    _ => 2,
                };
                let v = w.dp_read(a, m);
                w.load(v, m);
                pc + 2
            }
            0x06e485 => {
                w.operand_then_internal(2);
                let a = w.absolute(0xf00) + u32::from(w.cpu.x);
                let v = w.read(a, m).wrapping_add(1) & mask;
                w.drain(6);
                w.flags(v, m);
                w.write(a, v, m);
                pc + 3
            }
            0x06dca5 | 0x06dcb4 | 0x06dcb9 | 0x06dcc2 | 0x06dccc | 0x06dcd2 | 0x06dcd7
            | 0x06dce0 | 0x06dcec => {
                w.drain(8);
                let pointer = w.read(
                    if matches!(pc, 0x06dccc | 0x06dcec) {
                        0x92
                    } else {
                        0x90
                    },
                    2,
                );
                w.drain(6);
                w.write(w.absolute(pointer) + u32::from(w.cpu.y), w.cpu.a & mask, m);
                pc + 2
            }
            0x06dcb3 => {
                w.increment_a(m);
                pc + 1
            }
            0x0dd3ea | 0x05dfe4 | 0x05e00a | 0x05fa58 | 0x06e419 | 0x06e41d | 0x06e475
            | 0x06dc5b | 0x06dc63 | 0x06dcce | 0x06dcee => {
                let far = matches!(pc, 0x0dd3ea | 0x05dfe4 | 0x06e419 | 0x06dc5b | 0x06dc63);
                if far {
                    w.rtl();
                } else {
                    w.rts();
                }
                self.accept_return(w.cpu, far);
                self.pc
            }
            _ => unreachable!("sprite drawing {pc:06x}"),
        };
        if self.allocator.is_none() {
            w.cpu.bank = (self.pc >> 16) as u8;
            w.cpu.pc = self.pc as u16;
        }
        Some(w.cycles)
    }
}
