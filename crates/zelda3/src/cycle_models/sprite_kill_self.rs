//! Sprite_KillSelf keeps the drawing caller's far frame on the native bus.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, CARRY, ZERO}};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    const MASKS: [u8; 8] = [0x7f, 0xbf, 0xdf, 0xef, 0xf7, 0xfb, 0xfd, 0xfe];
    address.checked_sub(0x09f24b).and_then(|i| MASKS.get(i as usize)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sprite_kill_self_matches_source_indoor_returns_and_load_block_updates() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        for address in 0x09f24b..=0x09f252 {
            let offset = ((address >> 16) & 127) * 0x8000 + (address & 0x7fff);
            assert_eq!(table_byte(address), Some(rom[offset as usize]));
        }
        let mut instructions = 0;
        for field in 0..2 {
            for indoors in 0..2 {
                for slot in 0..16usize {
                    for deflect in [0u8, 0x40, 0x80, 0xc0] {
                        for block in [0u16, 7, 0x1234, 0xfffe, 0xffff] {
                            let mut machine = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom)
                                .unwrap().machine().snes().clone();
                            machine.ram.fill(0xa5);
                            machine.ram[0x1b] = indoors;
                            machine.ram[0xcaa + slot] = deflect;
                            machine.ram[0xbc0 + slot * 2..0xbc2 + slot * 2].copy_from_slice(&block.to_le_bytes());
                            machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 6]);
                            let mut cpu = NativeNmiContext { a: 0x1200, x: slot as u16, y: 7,
                                d: 0, db: 5, bank: 9, pc: 0xf1f8, sp: 0x1fc, p: 0x30 };
                            machine.cpu.a = cpu.a; machine.cpu.x = cpu.x; machine.cpu.y = cpu.y;
                            machine.cpu.dp = cpu.d; machine.cpu.db = cpu.db; machine.cpu.k = cpu.bank;
                            machine.cpu.pc = cpu.pc; machine.cpu.sp = cpu.sp; machine.cpu.e = false;
                            machine.cpu.unpack_flags(cpu.p);
                            let mut ram = machine.ram.clone();
                            let clock = CpuMasterTimeline::at_raster(field,
                                snes::CpuRasterPosition::new(224, 1340), snes::CpuBusWorkload::default(),
                                snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                            let mut source = snes::RomCpuTimingProbe::new(machine, clock,
                                snes::SourcePpuReadState::snes9x_reset()).unwrap();
                            let mut clock = source.timeline().clone();
                            let mut work = SpriteKillSelf::default();
                            for _ in 0..64 {
                                if work.complete() { break; }
                                super::super::native_work::compare_instruction("Sprite_KillSelf",
                                    &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                                    |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                                instructions += 1;
                            }
                            assert!(work.complete());
                            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (6, 0x805a, 0x1ff));
                            assert_eq!(ram, source.snes().ram);
                        }
                    }
                }
            }
        }
        eprintln!("1280 Sprite_KillSelf source fixtures, {instructions} instruction boundaries");
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpriteKillSelf { complete: bool }
impl SpriteKillSelf {
    pub(crate) fn complete(&self) -> bool { self.complete }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<u64> {
        if self.complete { return None; }
        assert_eq!((cpu.bank, cpu.d, cpu.p & 0x18), (9, 0, 0x10));
        let mut w = Work { cpu, clock, bus, cycles: 0 };
        let pc = w.cpu.pc;
        let width = if w.cpu.p & 0x20 == 0 { 2 } else { 1 };
        w.fetch();
        w.cpu.pc = match pc {
            0xf1f8 | 0xf20b => {
                w.drain(16);
                let (base, index) = if pc == 0xf1f8 { (0xcaa, w.cpu.x) } else { (0xbc0, w.cpu.y) };
                if (base & 255) + index > 255 { w.drain(6); }
                let value = w.read(w.absolute(base) + u32::from(index), width);
                w.load(value, width);
                pc + 3
            }
            0xf1fb | 0xf229 => { w.mask(if pc == 0xf1fb { 0x40 } else { 7 }, 1); pc + 2 }
            0xf1fd | 0xf201 | 0xf220 | 0xf237 => {
                let take = if pc == 0xf220 { w.cpu.p & CARRY != 0 } else { w.cpu.p & ZERO == 0 };
                let target = match pc { 0xf1fd => 0xf203, 0xf201 => 0xf24a, 0xf220 => 0xf235, _ => 0xf245 };
                if w.branch(take) { target } else { pc + 2 }
            }
            0xf1ff | 0xf235 | 0xf227 => {
                let value = w.dp_read(if pc == 0xf227 { 0 } else { 0x1b }, 1);
                w.load(value, 1); pc + 2
            }
            0xf203 | 0xf23e | 0xf241 | 0xf247 => {
                w.operand_then_internal(2);
                let (base, index) = match pc { 0xf203 => (0xdd0, w.cpu.x),
                    0xf241 => (0xbc1, w.cpu.y), 0xf247 => (0xbc0, w.cpu.x), _ => (0xbc0, w.cpu.y) };
                w.write(w.absolute(base) + u32::from(index), if pc == 0xf203 { 0 } else { w.cpu.a & 255 }, 1);
                pc + 3
            }
            0xf206 | 0xf239 => { w.drain(6); w.load(w.cpu.x, 1); pc + 1 }
            0xf207 | 0xf23a => { w.shift_left(1); pc + 1 }
            0xf208 | 0xf23b | 0xf22b => {
                w.drain(6);
                let value = w.cpu.a & 255;
                if pc == 0xf22b { w.cpu.x = value; } else { w.cpu.y = value; }
                w.flags(value, 1); pc + 1
            }
            0xf209 | 0xf21e => { w.width(0x20, pc == 0xf209); pc + 2 }
            0xf20e | 0xf21b | 0xf224 => {
                let value = if width == 1 { w.cpu.a & 255 } else { w.cpu.a };
                w.dp_store(match pc { 0xf20e => 0, 0xf21b => 1, _ => 3 }, value, width); pc + 2
            }
            0xf210 => { w.compare_immediate(0xffff, 2); pc + 3 }
            0xf213 | 0xf226 => {
                w.drain(6); w.push(if pc == 0xf213 { u16::from(w.cpu.p) } else { w.cpu.x }, 1); pc + 1
            }
            0xf214..=0xf216 => {
                w.drain(6);
                let value = w.cpu.a;
                w.cpu.p = (w.cpu.p & !CARRY) | (value & 1) as u8;
                w.load(value >> 1, 2); pc + 1
            }
            0xf217 => { w.clear_carry(); pc + 1 }
            0xf218 => { w.drain(16); w.adc(0xef80); pc + 3 }
            0xf21d | 0xf234 => {
                w.drain(12); let value = w.pull(1);
                if pc == 0xf21d { w.cpu.p = value as u8; } else { w.cpu.x = value; w.flags(value, 1); }
                pc + 1
            }
            0xf222 | 0xf23c | 0xf245 => { w.immediate(if pc == 0xf222 { 0x7f } else { 0xff }, 1); pc + 2 }
            0xf22c => { let value = w.indirect_read(1, 1); w.load(value, 1); pc + 2 }
            0xf22e => {
                w.drain(24); let value = w.read(0x09f24b + u32::from(w.cpu.x), 1);
                w.load(w.cpu.a & value, 1); pc + 4
            }
            0xf232 => {
                w.drain(8); let pointer = w.read(1, 2); let bank = w.read(3, 1);
                w.write((u32::from(bank) << 16) | u32::from(pointer), w.cpu.a & 255, 1); pc + 2
            }
            0xf244 | 0xf24a => { w.rtl(); self.complete = true; w.cpu.pc }
            _ => unreachable!("Sprite_KillSelf {pc:04x}"),
        };
        Some(w.cycles)
    }
}
