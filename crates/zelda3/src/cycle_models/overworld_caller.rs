//! Module09's physical dispatch and scroll/sprite/HUD/rain return path.
//! Callees retain their real frames; a child must return before this caller resumes.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, CARRY, ZERO}};
use snes::CpuMasterTimeline;

const SUBMODULES: [u16; 48] = [
    0xa53c, 0xab88, 0xabbc, 0xabc6, 0xabed, 0xac27, 0xabda, 0xac3a,
    0xc242, 0xad4a, 0xac8f, 0xacc2, 0xad6c, 0xae5e, 0xaf19, 0xab88,
    0xabbc, 0xabc6, 0xabed, 0xac27, 0xabda, 0xac3a, 0xb0d2, 0xae5e,
    0xb1c8, 0xb1df, 0xab88, 0xabbc, 0xb150, 0xaece, 0xaeea, 0xc2a4,
    0xaf1e, 0xed25, 0xb1bb, 0xb1fa, 0xae5e, 0xaf0b, 0xab88, 0xabbc,
    0xed59, 0xb0d2, 0xb528, 0xb40a, 0xb1fa, 0xb40e, 0xb40f, 0xb521,
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    match address {
        0x02a40d..=0x02a46c => {
            let i = (address - 0x02a40d) as usize;
            Some(SUBMODULES[i / 2].to_le_bytes()[i & 1])
        }
        0x02a46d..=0x02a474 => Some([1, 0, 1, 0, 0, 17, 0, 17][(address - 0x02a46d) as usize]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dispatch_scroll_stack_and_rain_return_match_source() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
        for address in 0x02a40d..=0x02a474 {
            assert_eq!(table_byte(address), Some(rom[(address - 0x018000) as usize]));
        }
        let mut boundaries = 0;
        let mut completed = 0;
        let mut children = [0; 4];
        for entry in [0xa475, 0xa47e, 0xa4b5, 0xa4c9, 0xa4cd] {
            for case in 0..1024usize {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                r[0x11] = (case % 48) as u8;
                r[0x1a] = case as u8;
                r[0x8a] = [0x70, 0x1b, 3, 5][case / 256];
                r[0x1ab] = 0;
                r[0xf3c5] = [0, 1, 2, 3][case / 256];
                r[0xf2f0] = if case & 256 != 0 { 0x20 } else { 0 };
                r[0x494] = case as u8;
                for (i, a) in [0xe0, 0xe2, 0xe6, 0xe8, 0x11a, 0x11c].into_iter().enumerate() {
                    let value = (case as u16).wrapping_mul(997).wrapping_add(i as u16 * 73);
                    r[a..a + 2].copy_from_slice(&value.to_le_bytes());
                }
                for i in 0..32 { r[0x1e0 + i] = (i as u8).wrapping_mul(17).wrapping_add(case as u8); }
                let mut cpu = NativeNmiContext { a: 0xb702, x: 224, y: 9, d: 0, db: 0, bank: 2,
                    pc: entry, sp: if entry == 0xa4b5 { 0x1f4 } else { 0x1fc },
                    p: 0x30 | (case as u8 & 0xc3) };
                machine.cpu.a = cpu.a; machine.cpu.x = cpu.x; machine.cpu.y = cpu.y;
                machine.cpu.pc = cpu.pc; machine.cpu.k = cpu.bank; machine.cpu.sp = cpu.sp;
                machine.cpu.e = false; machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(4921,
                    snes::CpuRasterPosition::new(224, if case & 1 == 0 { 1300 } else { 500 }),
                    snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                let mut source = snes::RomCpuTimingProbe::new(machine, timeline,
                    snes::SourcePpuReadState::snes9x_reset()).unwrap();
                let mut clock = source.timeline().clone();
                let mut work = OverworldCaller { pc: entry, ..Default::default() };
                for _ in 0..100 {
                    if work.complete() || work.child().is_some() { break; }
                    super::super::native_work::compare_instruction(
                        &format!("overworld entry={entry:04x} case={case} pc={:04x}", cpu.pc),
                        &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                        |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                    boundaries += 1;
                }
                if let Some(child) = work.child() {
                    children[match child { OverworldChild::Submodule(_) => 0, OverworldChild::Sprites => 1,
                        OverworldChild::LinkOam => 2, OverworldChild::Hud => 3 }] += 1;
                } else { assert!(work.complete()); completed += 1; }
            }
        }
        assert!(children.iter().all(|n| *n == 1024));
        assert_eq!(completed, 1024);
        eprintln!("5120 overworld caller fixtures, {boundaries} Source instruction boundaries, {completed} physical long returns, child handoffs={children:?}");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverworldChild { Submodule(u16), Sprites, LinkOam, Hud }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldCaller { pc: u16, child: Option<OverworldChild>, child_sp: u16, complete: bool }
impl Default for OverworldCaller {
    fn default() -> Self { Self { pc: 0xa475, child: None, child_sp: 0, complete: false } }
}
impl OverworldCaller {
    pub(crate) fn complete(&self) -> bool { self.complete }
    pub(crate) fn child(&self) -> Option<OverworldChild> { self.child }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let child = self.child.expect("overworld caller has no child");
        let sp = self.child_sp.wrapping_add(if matches!(child, OverworldChild::Submodule(_)) { 2 } else { 3 });
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, self.pc, sp), "overworld child must retire its physical return");
        self.child = None;
    }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<u64> {
        if self.complete { return None; }
        assert!(self.child.is_none(), "overworld child still owns the CPU");
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db), (2, self.pc, 0, 0));
        let mut w = Work { cpu, clock, bus, cycles: 0 }; w.fetch();
        let pc = self.pc;
        self.pc = match pc {
            0xa475 => { w.width(0x30, false); pc + 2 }
            0xa477 | 0xa480 | 0xa48b | 0xa497 | 0xa4a3 | 0xa4cd | 0xa4e3 | 0xa50a | 0xa51a | 0xa523 => {
                let a = match pc { 0xa477 => 0x11, 0xa480 => 0xe2, 0xa48b => 0xe8, 0xa497 => 0xe0, 0xa4a3 => 0xe6, 0xa4cd => 0x8a, 0xa51a => 0xe1, 0xa523 => 0xe7, _ => 0x1a };
                let width = if matches!(pc, 0xa480 | 0xa48b | 0xa497 | 0xa4a3) { 2 } else { 1 };
                let value = w.dp_read(a, width); w.load(value, width); pc + 2
            }
            0xa479 => { w.shift_left(1); pc + 1 }
            0xa47a | 0xa519 => { w.drain(6); w.cpu.x = w.cpu.a & 255; w.flags(w.cpu.x, 1); pc + 1 }
            0xa47b => {
                w.drain(8); w.drain(6); w.drain(8);
                let target = w.read(0x02a40d + u32::from(w.cpu.x), 2);
                w.push(0xa47d, 2); self.child = Some(OverworldChild::Submodule(target)); self.child_sp = w.cpu.sp;
                w.cpu.pc = target; self.pc = 0xa47e; return Some(w.cycles);
            }
            0xa47e => { w.width(0x21, true); pc + 2 }
            0xa4af | 0xa4c3 => { w.width(0x20, false); pc + 2 }
            0xa4b5 => { w.width(0x20, true); pc + 2 }
            0xa482 | 0xa48d | 0xa499 | 0xa4a5 => { w.drain(6); w.push(w.cpu.a, 2); pc + 1 }
            0xa483 | 0xa48f | 0xa49b | 0xa4a7 => {
                w.drain(16); let rhs = w.read(if matches!(pc, 0xa483 | 0xa49b) { 0x11a } else { 0x11c }, 2); w.adc(rhs); pc + 3
            }
            0xa486 | 0xa492 | 0xa49e | 0xa4aa | 0xa4b8 | 0xa4bb | 0xa4be | 0xa4c1 | 0xa508 | 0xa521 | 0xa52a => {
                let a = match pc { 0xa486 | 0xa4c1 => 0xe2, 0xa492 | 0xa4be => 0xe8, 0xa49e | 0xa4bb => 0xe0, 0xa4aa | 0xa4b8 => 0xe6, 0xa508 => 0x9a, 0xa521 => 0xe1, _ => 0xe7 };
                let width = if pc < 0xa508 { 2 } else { 1 };
                w.dp_store(a, w.cpu.a & if width == 1 { 255 } else { 65535 }, width); pc + 2
            }
            0xa488 | 0xa494 | 0xa4a0 | 0xa4ac | 0xa503 | 0xa516 => {
                w.drain(16); let a = match pc { 0xa488 => 0x11e, 0xa494 => 0x122, 0xa4a0 => 0x120, 0xa4ac => 0x124, 0xa503 => 0x12e, _ => 0x494 };
                let width = if pc < 0xa503 { 2 } else { 1 };
                let value = if pc == 0xa503 { w.cpu.x } else { w.cpu.a };
                w.write(a, value & if width == 1 { 255 } else { 65535 }, width); pc + 3
            }
            0xa48e | 0xa49a | 0xa4a6 | 0xa51c | 0xa525 => { w.clear_carry(); pc + 1 }
            0xa4b1 | 0xa4c5 | 0xa4c9 => {
                let (target, child) = match pc { 0xa4b1 => (0x068328, OverworldChild::Sprites), 0xa4c5 => (0x0da18e, OverworldChild::LinkOam), _ => (0x0ddb75, OverworldChild::Hud) };
                w.operand_then_internal(3); w.push(2, 1); w.push(pc + 3, 2);
                self.child = Some(child); self.child_sp = w.cpu.sp; self.pc = pc + 4;
                w.cpu.bank = (target >> 16) as u8; w.cpu.pc = target as u16; return Some(w.cycles);
            }
            0xa4b7 | 0xa4ba | 0xa4bd | 0xa4c0 => { w.drain(12); let value = w.pull(2); w.load(value, 2); pc + 1 }
            0xa4cf | 0xa4d7 | 0xa4e5 | 0xa4e9 | 0xa4ed | 0xa4f1 | 0xa4f5 | 0xa4f9 => {
                let v = match pc { 0xa4cf => 0x70, 0xa4d7 => 2, 0xa4e5 => 3, 0xa4e9 => 5, 0xa4ed => 0x24, 0xa4f1 => 0x2c, 0xa4f5 => 0x58, _ => 0x5a };
                w.compare_immediate(v, 1); pc + 2
            }
            0xa4d1 | 0xa4d9 | 0xa4e1 | 0xa4e7 | 0xa4eb | 0xa4ef | 0xa4f3 | 0xa4f7 | 0xa4fb | 0xa50e => {
                let taken = match pc { 0xa4d9 => w.cpu.p & CARRY != 0, 0xa4e1 | 0xa4fb | 0xa50e => w.cpu.p & ZERO == 0, _ => w.cpu.p & ZERO != 0 };
                let target = match pc { 0xa4d1 => 0xa4db, 0xa4d9 | 0xa4e1 | 0xa50e => 0xa52c, 0xa4e7 | 0xa4f7 => 0xa506, 0xa4eb | 0xa4f3 => 0xa4fd, 0xa4ef => 0xa501, _ => 0xa50a };
                if w.branch(taken) { target } else { pc + 2 }
            }
            0xa4d3 | 0xa4db => { w.drain(24); let value = w.read(if pc == 0xa4d3 { 0x7ef3c5 } else { 0x7ef2f0 }, 1); w.load(value, 1); pc + 4 }
            0xa4df | 0xa50c | 0xa514 => { w.mask(if pc == 0xa4df { 0x20 } else { 3 }, 1); pc + 2 }
            0xa4fd | 0xa506 => { w.immediate(if pc == 0xa4fd { 0x72 } else { 0x32 }, 1); pc + 2 }
            0xa4ff => { w.branch(true); 0xa508 }
            0xa501 => { w.drain(8); w.cpu.x = 0x36; w.flags(0x36, 1); pc + 2 }
            0xa510 => { w.drain(16); let v = w.read(0x494, 1); w.load(v, 1); pc + 3 }
            0xa513 => { w.increment_a(1); pc + 1 }
            0xa51d | 0xa526 => { w.drain(24); let rhs = w.read(if pc == 0xa51d { 0x02a46d } else { 0x02a471 } + u32::from(w.cpu.x), 1); w.adc_byte(rhs); pc + 4 }
            0xa52c => { w.rtl(); self.complete = true; w.cpu.pc }
            _ => unreachable!("overworld caller PC {pc:04x}"),
        };
        w.cpu.pc = self.pc; Some(w.cycles)
    }
}
