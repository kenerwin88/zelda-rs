//! Map-scroll dispatch and area checks on the caller's actual CPU and clock.
//! Stripe builders receive their real near frames; they are separate owners.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MapScrollStripe {
    Horizontal,
    Vertical,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldMapScroll {
    pc: u16,
    complete: bool,
    child: Option<MapScrollStripe>,
    return_pc: u16,
    return_sp: u16,
}
impl Default for OverworldMapScroll {
    fn default() -> Self {
        Self {
            pc: 0xf273,
            complete: false,
            child: None,
            return_pc: 0,
            return_sp: 0,
        }
    }
}
pub(crate) fn dispatch_byte(address: u32) -> Option<u8> {
    const TABLE: [u16; 16] = [
        0xf2a2, 0xf2a5, 0xf2ac, 0xf2a2, 0xf2b3, 0xf2ba, 0xf2ba, 0xf2a2, 0xf2c8, 0xf2cf, 0xf2cf,
        0xf2a2, 0xf2a2, 0xf2a2, 0xf2a2, 0xf2a2,
    ];
    (0x02f253..0x02f273).contains(&address).then(|| {
        TABLE[((address - 0x02f253) / 2) as usize].to_le_bytes()
            [((address - 0x02f253) & 1) as usize]
    })
}
impl OverworldMapScroll {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child(&self) -> Option<MapScrollStripe> {
        self.child
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        assert!(self.child.is_some());
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp),
            (2, self.return_pc, self.return_sp)
        );
        self.child = None;
        self.pc = self.return_pc;
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete || self.child.is_some() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (2, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        w.fetch();
        self.pc = match pc {
            0xf273 | 0xf277 | 0xf281 | 0xf293 | 0xf299 | 0xf2bd | 0xf2d2 | 0xf2dd | 0xf311
            | 0xf345 | 0xf37f => {
                w.width(
                    0x30,
                    matches!(pc, 0xf273 | 0xf281 | 0xf2dd | 0xf311 | 0xf345 | 0xf37f),
                );
                pc + 2
            }
            0xf275 | 0xf2a2 => {
                w.dp_store(if pc == 0xf275 { 14 } else { 17 }, 0, m);
                pc + 2
            }
            0xf279 | 0xf29b | 0xf2bf | 0xf2d4 => {
                w.drain(16);
                let v = w.read(w.absolute(0x416), m);
                w.load(v, m);
                pc + 3
            }
            0xf27c => {
                w.shift_left(m);
                pc + 1
            }
            0xf27d => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, x);
                pc + 1
            }
            0xf27e => {
                w.operand_then_internal(1);
                w.drain(8);
                let target = w.read(0x02f253 + u32::from(w.cpu.x), 2);
                w.push(pc + 2, 2);
                target
            }
            0xf283 | 0xf2f1 | 0xf325 | 0xf363 | 0xf39d => {
                w.cpu.y = w.dp_read(14, x);
                w.flags(w.cpu.y, x);
                pc + 2
            }
            0xf285 | 0xf295 | 0xf2f3 | 0xf327 | 0xf365 | 0xf39f => {
                w.immediate(
                    match pc {
                        0xf285 => 65535,
                        0xf295 => 3,
                        0xf2f3 | 0xf327 => 128,
                        _ => 0x8040,
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xf288 | 0xf28b | 0xf2f6 | 0xf32a | 0xf368 | 0xf3a2 => {
                w.operand_then_internal(2);
                let a = if pc == 0xf28b { 0x1102 } else { 0x1100 };
                w.write(w.absolute(a + w.cpu.y), w.cpu.a, m);
                pc + 3
            }
            0xf28e => {
                w.drain(16);
                w.cpu.p |= CARRY;
                w.flags(w.cpu.y, 2);
                pc + 3
            }
            0xf291 | 0xf2e4 | 0xf2ef | 0xf318 | 0xf323 | 0xf34c | 0xf356 | 0xf361 | 0xf386
            | 0xf390 | 0xf39b => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let n = w.cpu.p & NEGATIVE != 0;
                let (take, target) = match pc {
                    0xf291 => (z, 0xf299),
                    0xf2e4 => (n, 0xf310),
                    0xf2ef => (!z, 0xf300),
                    0xf318 => (c, 0xf344),
                    0xf323 => (!z, 0xf334),
                    0xf34c => (!c, 0xf353),
                    0xf356 => (z, 0xf37e),
                    0xf361 => (!z, 0xf372),
                    0xf386 => (!c, 0xf38d),
                    0xf390 => (c, 0xf3b8),
                    _ => (!z, 0xf3ac),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0xf297 => {
                w.dp_store(23, w.cpu.a & 255, m);
                pc + 2
            }
            0xf29e | 0xf2c4 | 0xf2d9 | 0xf2a8 | 0xf2af | 0xf2b6 | 0xf2cb => {
                w.drain(16);
                w.write(
                    w.absolute(if pc == 0xf29e { 0x418 } else { 0x416 }),
                    if matches!(pc, 0xf2a8 | 0xf2af | 0xf2b6 | 0xf2cb) {
                        0
                    } else {
                        w.cpu.a & 255
                    },
                    m,
                );
                pc + 3
            }
            0xf2a5 | 0xf2ac | 0xf2b3 | 0xf2ba | 0xf2c8 | 0xf2cf | 0xf2fd | 0xf331 | 0xf36f
            | 0xf3a9 => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                match pc {
                    0xf2a5 => 0xf37f,
                    0xf2ac => 0xf345,
                    0xf2b3 | 0xf2ba => 0xf311,
                    0xf2c8 | 0xf2cf => 0xf2dd,
                    _ => {
                        self.return_pc = pc + 3;
                        self.return_sp = w.cpu.sp + 2;
                        self.child = Some(if matches!(pc, 0xf2fd | 0xf331) {
                            MapScrollStripe::Vertical
                        } else {
                            MapScrollStripe::Horizontal
                        });
                        if matches!(pc, 0xf2fd | 0xf331) {
                            0xf482
                        } else {
                            0xf3b9
                        }
                    }
                }
            }
            0xf2c2 | 0xf2d7 | 0xf2ec | 0xf320 | 0xf35e | 0xf398 | 0xf30b | 0xf33f | 0xf379
            | 0xf3b3 => {
                w.mask(
                    if matches!(pc, 0xf2c2 | 0xf2d7) {
                        3
                    } else if matches!(pc, 0xf2ec | 0xf320 | 0xf35e | 0xf398) {
                        255
                    } else {
                        31
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xf2df | 0xf300 | 0xf313 | 0xf334 | 0xf347 | 0xf381 | 0xf308 | 0xf33c | 0xf376
            | 0xf3b0 => {
                let v = w.dp_read(
                    if matches!(pc, 0xf308 | 0xf33c) {
                        0x88
                    } else if matches!(pc, 0xf376 | 0xf3b0) {
                        0x86
                    } else {
                        0x84
                    },
                    m,
                );
                w.load(v, m);
                pc + 2
            }
            0xf2e1 | 0xf315 | 0xf349 | 0xf353 | 0xf383 | 0xf38d => {
                w.compare_immediate(
                    match pc {
                        0xf315 => 0x1800,
                        0xf353 => 0,
                        0xf38d => 0x60,
                        _ => 128,
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xf2e6 | 0xf31a | 0xf358 | 0xf392 => {
                w.cpu.x = w.dp_read(0x8a, x);
                w.flags(w.cpu.x, x);
                pc + 2
            }
            0xf2e8 | 0xf31c | 0xf35a | 0xf394 => {
                w.drain(24);
                let v = w.read(0x02f88d + u32::from(w.cpu.x), m);
                w.load(v, m);
                pc + 4
            }
            0xf2f9 | 0xf2fa | 0xf32d | 0xf32e | 0xf36b | 0xf36c | 0xf3a5 | 0xf3a6 => {
                w.increment_y();
                pc + 1
            }
            0xf2fb | 0xf32f | 0xf36d | 0xf3a7 => {
                w.dp_store(14, w.cpu.y, x);
                pc + 2
            }
            0xf302 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xf303 | 0xf34e | 0xf388 => {
                w.drain(16);
                w.sbc(128, 2);
                pc + 3
            }
            0xf306 | 0xf33a | 0xf30e | 0xf342 | 0xf37c | 0xf3b6 => {
                w.dp_store(
                    if matches!(pc, 0xf306 | 0xf33a) {
                        0x84
                    } else if matches!(pc, 0xf30e | 0xf342) {
                        0x88
                    } else {
                        0x86
                    },
                    w.cpu.a,
                    m,
                );
                pc + 2
            }
            0xf30a | 0xf378 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), 2);
                pc + 1
            }
            0xf336 => {
                w.clear_carry();
                pc + 1
            }
            0xf337 => {
                w.drain(16);
                w.adc(128);
                pc + 3
            }
            0xf33e | 0xf3b2 => {
                w.increment_a(2);
                pc + 1
            }
            0xf351 | 0xf38b => {
                w.branch(true);
                if pc == 0xf351 {
                    0xf349
                } else {
                    0xf383
                }
            }
            0xf372 | 0xf374 | 0xf3ac | 0xf3ae => {
                w.dp_increment(0x84, 2, pc >= 0xf3ac);
                pc + 2
            }
            0xf2a1 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            0xf2a4 | 0xf2ab | 0xf2b2 | 0xf2b9 | 0xf2c7 | 0xf2ce | 0xf2dc | 0xf310 | 0xf344
            | 0xf37e | 0xf3b8 => {
                w.rts();
                w.cpu.pc
            }
            _ => unreachable!("map scroll PC {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn map_scroll_post_stripe_suffix_matches_source_nonempty_buffers() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        for (entry, area_return) in [
            (0xf300, 0xf2ca),
            (0xf334, 0xf2b5),
            (0xf372, 0xf2ae),
            (0xf3ac, 0xf2a7),
        ] {
            for length in [0u16, 2, 262, 266] {
                for db in [0u8, 2, 0x80, 0x82] {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    for (a, v) in [
                        (0x0e, length),
                        (0x84, 0x101u16),
                        (0x86, 31),
                        (0x88, 0),
                        (0x416, 9),
                    ] {
                        machine.ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
                    }
                    machine.ram[0x1100..0x1400].fill(0x5a);
                    machine.ram[0x1f5..0x1f7].copy_from_slice(&u16::to_le_bytes(area_return));
                    machine.ram[0x1f7..0x1f9].copy_from_slice(&[0x80, 0xf2]);
                    machine.ram[0x1f9..0x1fb].copy_from_slice(&[0xc0, 0xac]);
                    let mut cpu = NativeNmiContext {
                        a: 0x8001,
                        x: 12,
                        y: 266,
                        d: 0,
                        db,
                        bank: 2,
                        pc: entry,
                        sp: 0x1f4,
                        p: 0x43,
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.db = db;
                    machine.cpu.k = 2;
                    machine.cpu.pc = entry;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let clock = CpuMasterTimeline::at_raster(
                        4937,
                        snes::CpuRasterPosition::new(224, 1300),
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
                    let mut work = OverworldMapScroll {
                        pc: entry,
                        ..Default::default()
                    };
                    for _ in 0..100 {
                        if work.complete() {
                            break;
                        }
                        super::super::native_work::compare_instruction(
                            &format!("map suffix entry={entry:04x} length={length} db={db:02x} pc={:04x}",cpu.pc),
                            &mut cpu,&mut clock,&mut source,&mut ram,&rom,
                            |cpu,clock,bus| {work.step(cpu,clock,bus).unwrap();});
                        work = work.clone();
                        instructions += 1;
                    }
                    assert!(work.complete());
                    assert_eq!(ram, source.snes().ram);
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xacc1, 0x1fa));
                    assert_eq!(ram[0x17], if length == 0 { 0 } else { 3 });
                    assert_eq!(
                        &ram[0x1100 + length as usize..0x1104 + length as usize],
                        &[255; 4]
                    );
                }
            }
        }
        eprintln!("64 independently seeded post-stripe suffixes, {instructions} Source instruction boundaries; no Source child CPU or RAM imports");
    }
    #[test]
    fn map_scroll_matches_source_dispatch_area_checks_and_stripe_frames() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let small_maps = &rom[0x1788d..0x1788d + 128];
        for i in 0..32 {
            assert_eq!(dispatch_byte(0x02f253 + i), Some(rom[0x17253 + i as usize]));
        }
        let mut instructions = 0;
        let mut completed = 0;
        let mut children = [0, 0];
        for dir in [0, 1, 2, 4, 5, 6, 8, 9, 10, 15] {
            for pos in [0u16, 1, 0x5f, 0x60, 0x7f, 0x80, 0x17ff, 0xffff] {
                for screen in [0u16, 2] {
                    for db in [0u8, 2, 0x80, 0x82] {
                        let mut machine = snes::Snes::new();
                        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                        machine.ram[0x11] = 10;
                        machine.ram[0x416] = dir;
                        for (a, v) in [(0x84, pos), (0x86, 31), (0x88, 0), (0x8a, screen)] {
                            machine.ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
                        }
                        machine.ram[0x1100..0x1400].fill(0x5a);
                        machine.ram[0x1f9..0x1fb].copy_from_slice(&[0xc0, 0xac]);
                        machine.ram[0x1fb..0x1fd].copy_from_slice(&[0x7d, 0xa4]);
                        let mut cpu = NativeNmiContext {
                            a: 0x1100,
                            x: 3,
                            y: 7,
                            d: 0,
                            db,
                            bank: 2,
                            pc: 0xf273,
                            sp: 0x1f8,
                            p: 0x73,
                        };
                        machine.cpu.a = cpu.a;
                        machine.cpu.x = cpu.x;
                        machine.cpu.y = cpu.y;
                        machine.cpu.db = db;
                        machine.cpu.k = 2;
                        machine.cpu.pc = cpu.pc;
                        machine.cpu.sp = cpu.sp;
                        machine.cpu.e = false;
                        machine.cpu.unpack_flags(cpu.p);
                        let mut ram = machine.ram.clone();
                        let clock = CpuMasterTimeline::at_raster(
                            4937,
                            snes::CpuRasterPosition::new(224, 1300),
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
                        let mut work = OverworldMapScroll::default();
                        let mut stripes = None::<super::super::overworld_stripes::OverworldStripes>;
                        for _ in 0..10000 {
                            if work.complete() {
                                break;
                            }
                            if stripes.is_none() {
                                if let Some(child) = work.child() {
                                    children[usize::from(child == MapScrollStripe::Vertical)] += 1;
                                    assert_eq!(
                                        (cpu.pc, cpu.sp),
                                        (
                                            if child == MapScrollStripe::Vertical {
                                                0xf482
                                            } else {
                                                0xf3b9
                                            },
                                            0x1f2
                                        )
                                    );
                                    stripes = Some(
                                        super::super::overworld_stripes::OverworldStripes::new(
                                            child, &cpu,
                                        ),
                                    );
                                }
                            }
                            super::super::native_work::compare_instruction_with_devices(
                                &format!("map scroll dir={dir} pos={pos:04x} screen={screen} db={db:02x} pc={:04x}",cpu.pc),
                                &mut cpu,&mut clock,&mut source,&mut ram,&rom,None,
                                |_,access|match access {
                                    super::super::intro_clear::Access::Read(address,width) if address&0x8000!=0&&address<0x7e0000=>{
                                        let read=|a|dispatch_byte(a)
                                            .or_else(||super::super::overworld_stripes::table_byte(a,&rom[0x78000..0x80000]))
                                            .or_else(||a.checked_sub(0x02f88d).and_then(|offset|small_maps.get(offset as usize).copied()))
                                            .expect("only immutable map tables");
                                        Some(if width==1 {u16::from(read(address))} else {u16::from_le_bytes([read(address),read(address+1)])})
                                    },_=>None,
                                },|cpu,clock,bus| {
                                    if let Some(stripes)=&mut stripes {stripes.step(cpu,clock,bus).unwrap();}
                                    else {work.step(cpu,clock,bus).unwrap();}
                                });
                            if let Some(child) = &mut stripes {
                                *child = child.clone();
                                if child.complete() {
                                    work.return_from_child(&cpu);
                                    stripes = None;
                                }
                            }
                            work = work.clone();
                            instructions += 1;
                        }
                        assert_eq!(ram, source.snes().ram);
                        assert!(work.complete());
                        assert_eq!(work.child(), None);
                        completed += 1;
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xacc1, 0x1fa));
                    }
                }
            }
        }
        assert_eq!(completed, 640);
        assert_eq!(children, [40, 108]);
        eprintln!("640 native map-scroll callers, {instructions} Source instruction boundaries, {completed} complete returns, horizontal/vertical native stripe calls={children:?}; no behavioral ROM reads or Source child state imports");
    }
}
