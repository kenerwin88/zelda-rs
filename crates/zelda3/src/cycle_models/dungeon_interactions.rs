//! Torch/door caller guards and tile probes on the retained dungeon CPU.
//! Active timer, door, switch and action bodies remain explicit boundaries.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    const OFFSETS: [u8; 24] = [
        7, 0, 24, 0, 8, 0, 8, 0, 0, 0, 0, 0, 255, 255, 17, 0, 2, 0, 2, 0, 128, 0, 128, 0,
    ];
    address
        .checked_sub(0x0099ea)
        .and_then(|i| OFFSETS.get(i as usize))
        .copied()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonInteractions {
    pc: u16,
    complete: bool,
}
impl Default for DungeonInteractions {
    fn default() -> Self {
        Self {
            pc: 0xce70,
            complete: false,
        }
    }
}
impl DungeonInteractions {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn resume_torch_return(&mut self, cpu: &NativeNmiContext) {
        assert_eq!(self.pending_body(), Some(0x01f4a6));
        assert_eq!((cpu.bank, cpu.pc, cpu.p & 0x30), (1, 0xce92, 0x30));
        self.pc = 0xce92;
    }
    pub(crate) fn pending_body(&self) -> Option<u32> {
        matches!(self.pc, 0xf4a6 | 0xceed | 0xcff1 | 0xd059)
            .then_some(0x010000 | u32::from(self.pc))
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
        assert!(self.pending_body().is_none());
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 8),
            (1, self.pc, 0, 0, 0)
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
        w.fetch();
        self.pc = match pc {
            0xce7b => {
                w.drain(8);
                w.cpu.x = 0;
                w.flags(0, x);
                0xce7d
            }
            0xce7d => {
                w.drain(16);
                if x == 2 {
                    w.drain(6);
                }
                let value = w.read(0x4f0 + u32::from(w.cpu.x), m);
                w.load(value, m);
                0xce80
            }
            0xce82 => {
                w.operand_then_internal(2);
                let address = 0x4f0 + u32::from(w.cpu.x);
                let old = w.read(address, m);
                w.drain(6);
                let value = old.wrapping_sub(1) & if m == 1 { 255 } else { 65535 };
                w.flags(value, m);
                w.write_high_low(address, value, m);
                0xce85
            }
            0xce87 => {
                w.drain(6);
                w.push(w.cpu.x, x);
                0xce88
            }
            0xce88 => {
                w.drain(6);
                w.load(w.cpu.x, m);
                0xce89
            }
            0xce89 => {
                w.drain(8);
                w.load(w.cpu.a | 0xc0, m);
                0xce8b
            }
            0xce8b => {
                w.drain(16);
                w.write(0x333, w.cpu.a & if m == 1 { 255 } else { 65535 }, m);
                0xce8e
            }
            0xce8e => {
                w.operand_then_internal(3);
                w.push(1, 1);
                w.push(0xce91, 2);
                0xf4a6
            }
            0xce92 => {
                w.drain(12);
                w.cpu.x = w.pull(x);
                w.flags(w.cpu.x, x);
                0xce93
            }
            0xce93 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & if x == 1 { 255 } else { 65535 };
                w.flags(w.cpu.x, x);
                0xce94
            }
            0xce94 => {
                w.compare_x_immediate(16, x);
                0xce96
            }
            0xce70 | 0xcea2 | 0xceaa | 0xceb7 | 0xd054 => {
                let address = match pc {
                    0xce70 => 0x1a,
                    0xcea2 => 0x2f,
                    0xceaa => 0x20,
                    0xceb7 => 0x22,
                    _ => 0x3a,
                };
                let value = w.dp_read(address, m);
                w.load(value, m);
                pc + 2
            }
            0xce72 | 0xcea4 | 0xceaf | 0xcebd | 0xced3 | 0xcee5 => {
                w.mask(
                    match pc {
                        0xce72 => 3,
                        0xcea4 => 255,
                        0xceaf | 0xcebd => 0x1f8,
                        _ => 0xf0,
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xce74 | 0xce79 | 0xce80 | 0xce85 | 0xce96 | 0xce9b | 0xcec7 | 0xced9 | 0xceeb
            | 0xcfef | 0xd057 => {
                let z = w.cpu.p & ZERO != 0;
                let (take, target) = match pc {
                    0xce74 | 0xce79 => (!z, 0xce98),
                    0xce80 => (z, 0xce93),
                    0xce85 => (!z, 0xce93),
                    0xce96 => (!z, 0xce7d),
                    0xce9b => (z, 0xcea0),
                    0xcec7 => (z, 0xcecc),
                    0xced9 => (z, 0xceed),
                    0xceeb => (!z, 0xcf44),
                    0xcfef => (w.cpu.p & 0x80 != 0, 0xd052),
                    _ => (w.cpu.p & CARRY == 0, 0xcfe9),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0xce76 | 0xce98 | 0xcfec => {
                w.drain(16);
                let value = w.read(
                    if pc == 0xce76 {
                        0x112
                    } else if pc == 0xce98 {
                        0x2e4
                    } else {
                        0x436
                    },
                    m,
                );
                w.load(value, m);
                pc + 3
            }
            0xce9d | 0xcf47 => {
                w.drain(16);
                0xcfea
            }
            0xcea0 | 0xcecc | 0xcfea | 0xd052 => {
                w.width(
                    if pc == 0xcea0 {
                        0x21
                    } else if pc == 0xcecc {
                        0x10
                    } else {
                        0x30
                    },
                    pc < 0xcfea,
                );
                pc + 2
            }
            0xcea7 | 0xceb5 => {
                w.dp_store(if pc == 0xcea7 { 8 } else { 0 }, w.cpu.a, m);
                pc + 2
            }
            0xcea9 | 0xcece | 0xcee0 => {
                w.drain(6);
                let value = if x == 1 { w.cpu.a & 255 } else { w.cpu.a };
                w.flags(value, x);
                if pc == 0xcea9 {
                    w.cpu.y = value;
                } else {
                    w.cpu.x = value;
                }
                pc + 1
            }
            0xceac | 0xceba | 0xcedd => {
                let base = match pc {
                    0xceac => 0x99ea,
                    0xceba => 0x99f2,
                    _ => 0x99fa,
                };
                w.drain(16);
                if x == 2 || (base & 255) + u32::from(w.cpu.y) > 255 {
                    w.drain(6);
                }
                let value = w.read(base + u32::from(w.cpu.y), m);
                w.adc(value);
                pc + 3
            }
            0xceb2..=0xceb4 | 0xd056 => {
                w.shift_left(m);
                pc + 1
            }
            0xcec0..=0xcec2 => {
                w.drain(6);
                let old = w.cpu.a;
                w.cpu.p = (w.cpu.p & !CARRY) | (old as u8 & 1);
                w.load(old >> 1, m);
                pc + 1
            }
            0xcec3 => {
                let value = w.dp_read(0, m);
                w.load(w.cpu.a | value, m);
                pc + 2
            }
            0xcec5 => {
                w.cpu.x = w.dp_read(0xee, x);
                w.flags(w.cpu.x, x);
                pc + 2
            }
            0xcec9 => {
                w.drain(16);
                w.load(w.cpu.a | 0x1000, 2);
                pc + 3
            }
            0xcecf | 0xcee1 => {
                w.drain(24);
                let value = w.read(0x7f2000 + u32::from(w.cpu.x), m);
                w.load(value, m);
                pc + 4
            }
            0xced6 | 0xcee8 => {
                w.compare_immediate(0xf0, m);
                pc + 3
            }
            0xcedb => {
                w.drain(6);
                w.load(w.cpu.x, m);
                pc + 1
            }
            0xceb9 | 0xcedc => {
                w.clear_carry();
                pc + 1
            }
            0xcf44 => {
                w.drain(16);
                w.write(0x4b8, 0, m);
                pc + 3
            }
            0xcfe9 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("dungeon interactions {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_interaction_guards_and_tile_probes_match_source() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x99ea..0x9a02 {
            assert_eq!(table_byte(address), Some(rom[(address & 0x7fff) as usize]));
        }
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..=255u8 {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            machine.ram[0x1a] = if case & 1 == 0 { 237 } else { 236 };
            machine.ram[0x2e4] = ((case >> 1) & 1) as u8;
            machine.ram[0x2f] = ((case >> 2) & 3) * 2;
            machine.ram[0xee] = (case >> 4) & 1;
            machine.ram[0x436] = if case & 32 != 0 { 255 } else { 0 };
            machine.ram[0x3a] = if case & 64 != 0 { 0x80 } else { 0 };
            machine.ram[0x20..0x24].copy_from_slice(&[0xfa, 0x21, 0x40, 9]);
            machine.ram[0x12000..0x14000].fill(if case & 128 != 0 { 0xf0 } else { 2 });
            machine.ram[0x4b8..0x4ba].copy_from_slice(&[0x77, 0x99]);
            machine.ram[0x1fa..0x1fd].copy_from_slice(&[0xd0, 0x87, 2]);
            let mut cpu = NativeNmiContext {
                a: 0x2100,
                x: 0,
                y: 6,
                d: 0,
                db: 0,
                bank: 1,
                pc: 0xce70,
                sp: 0x1f9,
                p: 0x32,
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
                17,
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
            let mut work = DungeonInteractions::default();
            for _ in 0..1000 {
                if work.complete() || work.pending_body().is_some() {
                    break;
                }
                visited.insert(cpu.pc);
                super::super::native_work::compare_instruction(
                    &format!("interactions case={case} pc={:04x}", cpu.pc),
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
            assert!(work.complete() || work.pending_body().is_some());
            assert_eq!(ram, source.snes().ram);
            if work.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87d1, 0x1fc));
            }
        }
        for pc in [0xce9d, 0xcec9, 0xcee1, 0xcf44, 0xcfe9, 0xd052] {
            assert!(visited.contains(&pc), "{pc:04x}");
        }
    }
}

#[cfg(test)]
mod torch_return_tests {
    use super::*;
    #[test]
    fn dungeon_interactions_resume_torch_returns_without_restarting_the_scan() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut owned = 0;
        let mut calls = 0;
        for case in 0..32usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            machine.ram[0x2e4] = 1;
            machine.ram[0x436] = 255;
            for i in 0..16 {
                machine.ram[0x4f0 + i] = if case & 1 != 0 || i == case / 2 { 1 } else { 0 };
            }
            machine.ram[0x1fa..0x1fd].copy_from_slice(&[0xd0, 0x87, 2]);
            let mut cpu = NativeNmiContext {
                a: 0x2100,
                x: 0,
                y: 6,
                d: 0,
                db: 0,
                bank: 1,
                pc: 0xce70,
                sp: 0x1f9,
                p: 0x32,
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
                17,
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
            let mut work = DungeonInteractions::default();
            while !work.complete() {
                if work.pending_body() == Some(0x01f4a6) {
                    let mut steps = 0;
                    while (source.snes().cpu.k, source.snes().cpu.pc) != (1, 0xce92) {
                        source.step().unwrap();
                        steps += 1;
                        assert!(steps < 30000);
                    }
                    let c = &source.snes().cpu;
                    cpu = NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags(),
                    };
                    clock = source.timeline().clone();
                    ram.copy_from_slice(&source.snes().ram);
                    work.resume_torch_return(&cpu);
                    calls += 1;
                }
                assert!(work.pending_body().is_none());
                let pc = cpu.pc;
                super::super::native_work::compare_instruction(
                    &format!("torch return case={case} pc={pc:04x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                owned += 1;
            }
            assert!(ram[0x4f0..0x500].iter().all(|b| *b == 0));
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87d1, 0x1fc));
        }
        assert_eq!(calls, 272);
        println!("native torch returns: 32 fixtures, {calls} semantic child returns, {owned} source instructions without scan replay");
    }
}
