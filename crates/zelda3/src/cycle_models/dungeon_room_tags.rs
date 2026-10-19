//! Room-tag caller, staircase probe and two selected tag calls.
//! The buggy staircase ADC uses fixed source operands, never ROM opcodes.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RoomTagTail {
    Lamp,
    Returned,
    Staircase { head: u8, landing: u16, pos: u16 },
    Tag { index: u8, target: u16 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonRoomTags {
    pc: u32,
    tail: Option<RoomTagTail>,
    head: u8,
    landing: u16,
    pos: u16,
}
impl Default for DungeonRoomTags {
    fn default() -> Self {
        Self {
            pc: 0x0287c1,
            tail: None,
            head: 0,
            landing: 0,
            pos: 0,
        }
    }
}
const TAG_TARGETS: [u16; 64] = [
    0xc328, 0xc432, 0xc438, 0xc43e, 0xc444, 0xc44a, 0xc450, 0xc456, 0xc45c, 0xc461, 0xc4bf, 0xc432,
    0xc438, 0xc43e, 0xc444, 0xc44a, 0xc450, 0xc456, 0xc45c, 0xc461, 0xc4e7, 0xc508, 0xc541, 0xc599,
    0xca94, 0xcb1a, 0xcb49, 0xcbff, 0xc8d4, 0xc98b, 0xca17, 0xca17, 0xc67a, 0xcc00, 0xcc5b, 0xcc04,
    0xcc89, 0xc709, 0xc7a2, 0xc7cc, 0xc685, 0xc432, 0xc438, 0xc43e, 0xc444, 0xc44a, 0xc450, 0xc456,
    0xc45c, 0xc461, 0xc4bf, 0xc629, 0xcc08, 0xcc0c, 0xcc10, 0xcc14, 0xc74e, 0xcc18, 0xcc1c, 0xcc62,
    0xc7c2, 0xc767, 0xc8ae, 0xc4db,
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let i = address.checked_sub(0x01c27d)? as usize;
    (i < 128).then(|| TAG_TARGETS[i / 2].to_le_bytes()[i % 2])
}
impl DungeonRoomTags {
    pub(crate) fn tail(&self) -> Option<RoomTagTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
        assert_eq!((cpu.d, cpu.db, cpu.p & 8), (0, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0x0287c1 => {
                let v = w.dp_read(0x11, 1);
                w.load(v, 1);
                0x0287c3
            }
            0x0287c3 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    self.tail = Some(RoomTagTail::Lamp);
                    0x0287e1
                } else {
                    0x0287c5
                }
            }
            0x0287c5 => {
                w.operand_then_internal(3);
                w.push(2, 1);
                w.push(0x87c8, 2);
                0x01c2fd
            }
            0x01c2fd => {
                w.drain(16);
                let v = w.read(0x4c7, 1);
                w.load(v, 1);
                0x01c300
            }
            0x01c300 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x01c31b
                } else {
                    0x01c302
                }
            }
            0x01c302 | 0x01c326 => {
                w.width(0x30, false);
                pc + 2
            }
            0x01c304 => {
                w.operand_then_internal(2);
                w.push(0xc306, 2);
                0x01c329
            }
            0x01c307 => {
                w.dp_store(14, 0, 1);
                0x01c309
            }
            0x01c309 | 0x01c314 => {
                let v = w.dp_read(if pc == 0x01c309 { 0xae } else { 0xaf }, 1);
                w.load(v, 1);
                pc + 2
            }
            0x01c30b | 0x01c316 => {
                w.shift_left(1);
                pc + 1
            }
            0x01c30c | 0x01c317 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0x01c30d | 0x01c318 => {
                w.drain(8);
                w.drain(6);
                w.drain(8);
                let target = w.read(0x01c27d + u32::from(w.cpu.x), 2);
                w.push((pc + 2) as u16, 2);
                if target != 0xc328 {
                    self.tail = Some(RoomTagTail::Tag {
                        index: u8::from(pc == 0x01c318),
                        target,
                    });
                }
                0x010000 | u32::from(target)
            }
            0x01c310 => {
                w.immediate(1, 1);
                0x01c312
            }
            0x01c312 => {
                w.dp_store(14, w.cpu.a & 255, 1);
                0x01c314
            }
            0x01c31b => {
                w.drain(16);
                w.write(0x4c7, 0, 1);
                0x01c31e
            }
            0x01c31e => {
                w.rtl();
                assert_eq!((w.cpu.bank, w.cpu.pc, w.cpu.sp), (2, 0x87c9, 0x1fc));
                self.tail = Some(RoomTagTail::Returned);
                0x0287c9
            }
            0x01c325 => {
                w.drain(12);
                let v = w.pull(1);
                w.load(v, 1);
                0x01c326
            }
            0x01c328 => {
                w.rts();
                0x010000 | u32::from(w.cpu.pc)
            }
            0x01c329 => {
                w.width(0x20, true);
                0x01c32b
            }
            0x01c32b | 0x01c335 | 0x01c343 => {
                let v = w.dp_read(
                    match pc {
                        0x01c32b => 0x67,
                        0x01c335 => 0x20,
                        _ => 0x22,
                    },
                    2,
                );
                w.load(v, 2);
                pc + 2
            }
            0x01c32d | 0x01c33b | 0x01c345 => {
                w.mask(if pc == 0x01c32d { 12 } else { 0x01f8 }, 2);
                pc + 3
            }
            0x01c330 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x01c326
                } else {
                    0x01c332
                }
            }
            0x01c332 | 0x01c341 => {
                w.dp_store(if pc == 0x01c332 { 2 } else { 0 }, w.cpu.a, 2);
                pc + 2
            }
            0x01c334 | 0x01c386 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                if pc == 0x01c386 {
                    self.landing = w.cpu.y;
                }
                pc + 1
            }
            0x01c337 | 0x01c35f => {
                w.clear_carry();
                pc + 1
            }
            0x01c338 => {
                w.drain(16);
                let operand = match w.cpu.y {
                    4 => 8,
                    8 => 0,
                    12 => 0xffff,
                    _ => unreachable!(),
                };
                // The fixed operand owns the two slow-ROM read cycles.
                w.drain(16);
                w.adc(operand);
                0x01c33b
            }
            0x01c33e..=0x01c340 => {
                w.shift_left(2);
                pc + 1
            }
            0x01c348..=0x01c34a => {
                w.drain(6);
                let v = w.cpu.a;
                w.cpu.p = (w.cpu.p & !1) | (v as u8 & 1);
                w.load(v >> 1, 2);
                pc + 1
            }
            0x01c34b => {
                let v = w.dp_read(0, 2);
                w.load(w.cpu.a | v, 2);
                0x01c34d
            }
            0x01c34d => {
                w.cpu.x = w.dp_read(0xee, 1);
                w.flags(w.cpu.x, 1);
                0x01c34f
            }
            0x01c34f => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x01c354
                } else {
                    0x01c351
                }
            }
            0x01c351 => {
                w.drain(16);
                w.load(w.cpu.a | 0x1000, 2);
                0x01c354
            }
            0x01c354 => {
                w.width(0x10, true);
                0x01c356
            }
            0x01c356 | 0x01c363 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                if pc == 0x01c356 {
                    self.pos = w.cpu.x;
                }
                pc + 1
            }
            0x01c357 => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                0x01c358
            }
            0x01c358 => {
                w.cpu.y = w.dp_read(2, 2);
                w.flags(w.cpu.y, 2);
                0x01c35a
            }
            0x01c35a => {
                w.drain(16);
                w.cpu.p = (w.cpu.p & !1) | u8::from(w.cpu.y >= 4);
                w.flags(w.cpu.y.wrapping_sub(4), 2);
                0x01c35d
            }
            0x01c35d => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x01c364
                } else {
                    0x01c35f
                }
            }
            0x01c360 => {
                w.drain(16);
                w.adc(0x80);
                0x01c363
            }
            0x01c364 => {
                w.width(0x20, false);
                0x01c366
            }
            0x01c366 => {
                w.drain(24);
                let v = w.read(0x7f2000 + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                self.head = v as u8;
                0x01c36a
            }
            0x01c36a => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                0x01c36b
            }
            0x01c36b | 0x01c36f | 0x01c373 | 0x01c377 | 0x01c37b => {
                w.compare_immediate(
                    match pc {
                        0x01c36b => 0x26,
                        0x01c36f => 0x38,
                        0x01c373 => 0x39,
                        0x01c377 => 0x5e,
                        _ => 0x5f,
                    },
                    1,
                );
                pc + 2
            }
            0x01c36d | 0x01c371 | 0x01c375 | 0x01c379 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x01c37f
                } else {
                    pc + 2
                }
            }
            0x01c37d => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x01c326
                } else {
                    0x01c37f
                }
            }
            0x01c37f => {
                w.drain(6);
                w.push(w.cpu.a & 255, 1);
                0x01c380
            }
            0x01c380 => {
                w.dp_store(14, w.cpu.a & 255, 1);
                0x01c382
            }
            0x01c382 => {
                w.drain(24);
                let v = w.read(0x7f2040 + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                0x01c386
            }
            0x01c387 => {
                w.mask(0xf8, 1);
                0x01c389
            }
            0x01c389 => {
                w.compare_immediate(0x30, 1);
                0x01c38b
            }
            0x01c38b => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x01c325
                } else {
                    0x01c38d
                }
            }
            0x01c38d => {
                w.drain(16);
                let v = w.read(0x308, 1);
                w.load(v, 1);
                0x01c390
            }
            0x01c390 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    self.tail = Some(RoomTagTail::Staircase {
                        head: self.head,
                        landing: self.landing,
                        pos: self.pos,
                    });
                    0x01c39c
                } else {
                    0x01c392
                }
            }
            0x01c392 => {
                w.drain(12);
                let v = w.pull(1);
                w.load(v, 1);
                0x01c393
            }
            0x01c393 => {
                w.width(0x20, true);
                0x01c395
            }
            0x01c395 => {
                w.drain(16);
                let v = w.read(0xfc4, 2);
                w.load(v, 2);
                0x01c398
            }
            0x01c398 => {
                w.dp_store(0x20, w.cpu.a, 2);
                0x01c39a
            }
            0x01c39a => {
                w.branch(true);
                0x01c326
            }
            _ => unreachable!("room-tag source PC {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize, mode: usize) {
    fn word(r: &mut [u8], a: usize, v: u16) {
        r[a..a + 2].copy_from_slice(&v.to_le_bytes());
    }
    r[0x11] = if mode == 0 && case & 1 == 0 { 15 } else { 0 };
    r[0x4c7] = if mode == 0 { (case % 255 + 1) as u8 } else { 0 };
    let direction = if mode == 1 {
        0
    } else {
        [4u8, 8, 12, 0][case % 4]
    };
    r[0x67] = direction;
    r[0xee] = ((case / 4) % 2) as u8;
    let y = 0x1234u16.wrapping_add((case * 6) as u16);
    let x = 0x5678u16.wrapping_sub((case * 4) as u16);
    word(r, 0x20, y);
    word(r, 0x22, x);
    word(r, 0xfc4, 0xa100 + case as u16);
    r[0x308] = if mode == 4 { 128 } else { 0 };
    let operand = match direction {
        4 => 8,
        8 => 0,
        12 => 0xffff,
        _ => 0,
    };
    let pos = ((y.wrapping_add(operand) & 0x1f8) << 3)
        | ((x & 0x1f8) >> 3)
        | if r[0xee] != 0 { 0x1000 } else { 0 };
    let head = if mode == 2 {
        0x27
    } else {
        [0x26, 0x38, 0x39, 0x5e, 0x5f][case / 8 % 5]
    };
    r[0x12000 + usize::from(pos) + if direction == 4 { 128 } else { 0 }] = head;
    r[0x12000 + usize::from(pos) + 64] = if mode == 3 {
        0
    } else {
        0x30 + (case % 8) as u8
    };
    // Dispatcher cases reject the stair head before selecting their tag child.
    if mode == 6 || mode == 7 {
        r[0x12000 + usize::from(pos) + if direction == 4 { 128 } else { 0 }] = 0;
    }
    r[0xae] = if mode == 7 { (case % 64) as u8 } else { 0 };
    r[0xaf] = if mode == 6 { (case % 64) as u8 } else { 0 };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_room_tags_match_source_staircase_rejection_acceptance_and_selected_calls() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for i in 0..128 {
            assert_eq!(table_byte(0x01c27d + i), Some(rom[0xc27d + i as usize]));
        }
        for (a, v) in [(0x99ee, 8u16), (0x99f2, 0), (0x99f6, 0xffff)] {
            assert_eq!(
                u16::from_le_bytes([rom[a - 0x8000], rom[a - 0x8000 + 1]]),
                v
            );
        }
        let mut count = 0;
        let mut tails = [0; 4];
        for mode in 0..8 {
            for case in 0..256usize {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
                }
                seed_case(&mut machine.ram, case, mode);
                let mut cpu = NativeNmiContext {
                    a: 0x2110,
                    x: 0,
                    y: 6,
                    d: 0,
                    db: 0,
                    bank: 2,
                    pc: 0x87c1,
                    sp: 0x1fc,
                    p: 0x30 | (case as u8 & 0xc3),
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let clock = CpuMasterTimeline::at_raster(
                    4586,
                    snes::CpuRasterPosition::new(48, 896),
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
                let mut work = DungeonRoomTags::default();
                while work.tail().is_none() {
                    let label = format!("room tags mode={mode} case={case} pc={:06x}", work.pc);
                    if work.pc == 0x01c338 {
                        // This fixed operand has no runtime ROM bus getter. Other
                        // instructions compare every observable bus transaction.
                        source.step().unwrap();
                        work.step(&mut cpu, &mut clock, |_, _| -> u16 {
                            panic!("fixed staircase operand attempted a runtime read")
                        })
                        .unwrap();
                        let c = &source.snes().cpu;
                        assert_eq!(
                            cpu,
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
                        assert_eq!(clock, *source.timeline(), "{label}");
                    } else {
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
                    }
                    assert_eq!(ram, source.snes().ram, "{label}");
                    work = work.clone();
                    count += 1;
                }
                match work.tail().unwrap() {
                    RoomTagTail::Lamp => {
                        tails[0] += 1;
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87e1, 0x1fc));
                    }
                    RoomTagTail::Returned => {
                        tails[1] += 1;
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87c9, 0x1fc));
                    }
                    RoomTagTail::Staircase { head, landing, pos } => {
                        tails[2] += 1;
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (1, 0xc39c, 0x1f6));
                        assert_eq!(landing, cpu.y);
                        assert_eq!(pos, cpu.x);
                        assert_eq!(head, ram[0xe]);
                    }
                    RoomTagTail::Tag { index, target } => {
                        tails[3] += 1;
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (1, target, 0x1f7));
                        assert_eq!(ram[0xe], index);
                    }
                }
            }
        }
        assert!(tails.iter().all(|v| *v > 0));
        println!("native room tags: 2048 cases, {count} source instructions, tails={tails:?}, fixed staircase operand needs no runtime ROM getter");
    }
}
