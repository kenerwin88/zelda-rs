//! Shared transformation timer and effect guard on its original near frame.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO}};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinkTransformGuard { pc: u16, child: Option<u32>, child_return: Option<(u16, u16)>, complete: bool }
impl Default for LinkTransformGuard {
    fn default() -> Self { Self { pc: 0x82da, child: None, child_return: None, complete: false } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timers_cancel_ancilla_and_effect_frames_match_source() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
        let entries = [0x82da, 0x8305, 0x8309, 0x831f, 0x8327, 0x832c, 0x833d, 0x8351];
        let mut instructions = 0; let mut returns = 0; let mut children = std::collections::BTreeSet::new();
        for entry in entries {
            for case in 0..512usize {
                let mut machine = snes::Snes::new(); machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                r[0x3f5] = if case & 1 != 0 { case as u8 } else { 0 };
                r[0x3f6] = u8::from(case & 2 != 0); r[0x3f7] = u8::from(case & 4 != 0);
                r[0x5d] = [0, 23, 28, 30][case / 8 % 4]; r[0x2e2] = case as u8;
                r[0x308] = case as u8; r[0x309] = (case as u8).rotate_left(1);
                for i in 0..5 { r[0xc4a + i] = if i == case % 6 { [0, 0x30, 0x31, 0x24][case / 6 % 4] } else { 0 }; }
                for i in 0..16 { r[0x1e8+i] = (i as u8).wrapping_mul(17).wrapping_add(case as u8); }
                let mut cpu = NativeNmiContext { a: 0xb780, x: 224, y: 9, d: 0, db: 7, bank: 7,
                    pc: entry, sp: 0x1f2, p: 0x30 | (case as u8 & 0xc3) };
                machine.cpu.a = cpu.a; machine.cpu.x = cpu.x; machine.cpu.y = cpu.y; machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank; machine.cpu.pc = cpu.pc; machine.cpu.sp = cpu.sp;
                machine.cpu.e = false; machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(4937, snes::CpuRasterPosition::new(224, 1300),
                    snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                let mut source = snes::RomCpuTimingProbe::new(machine, timeline,
                    snes::SourcePpuReadState::snes9x_reset()).unwrap();
                let mut clock = source.timeline().clone();
                let mut work = LinkTransformGuard { pc: entry, ..Default::default() };
                for _ in 0..1000 {
                    if work.complete() || work.child().is_some() { break; }
                    super::super::native_work::compare_instruction(
                        &format!("Link transform entry={entry:04x} case={case} pc={:04x}", cpu.pc),
                        &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                        |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                    instructions += 1;
                }
                if let Some(child) = work.child() { children.insert(child); }
                else { assert!(work.complete()); returns += 1; }
            }
        }
        assert_eq!(children, [0x07f1a3, 0x079195, 0x09912c, 0x078028, 0x0ed6dd].into_iter().collect());
        assert!(returns > 0);
        eprintln!("4096 transformation fixtures, {instructions} Source instruction boundaries, {returns} physical near returns, all five effect frames");
    }
}
impl LinkTransformGuard {
    pub(crate) fn child(&self) -> Option<u32> { self.child }
    pub(crate) fn complete(&self) -> bool { self.complete }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        assert!(self.child.take().is_some());
        let (pc, sp) = self.child_return.take().unwrap();
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (7, pc, sp)); self.pc = pc;
    }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<u64> {
        if self.complete { return None; }
        assert!(self.child.is_none(), "transformation child still owns its frame");
        assert_eq!((cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38), (7, self.pc, 7, 0, 0x30));
        let mut w = Work { cpu, clock, bus, cycles: 0 }; w.fetch(); let pc = self.pc;
        self.pc = match pc {
            0x82da | 0x82e2 | 0x82f1 | 0x82fb => {
                w.drain(16); let v = w.read(match pc { 0x82da => 0x3f5, 0x82e2 => 0x3f7, _ => 0x308 + u32::from(pc == 0x82f1) }, 1); w.load(v, 1); pc + 3
            }
            0x82dd => { w.drain(16); let rhs = w.read(0x3f6, 1); w.load((w.cpu.a | rhs) & 255, 1); pc + 3 }
            0x82e0 | 0x82e5 | 0x82eb | 0x82ef | 0x82f6 | 0x8310 | 0x8314 | 0x831a | 0x8340 => {
                let take = match pc { 0x82e5 | 0x8314 => w.cpu.p & ZERO == 0,
                    0x831a | 0x8340 => w.cpu.p & NEGATIVE == 0, _ => w.cpu.p & ZERO != 0 };
                let target = match pc { 0x82e0 => 0x82d8, 0x82e5 => 0x833d, 0x82eb | 0x82ef => 0x835b,
                    0x82f6 => 0x82fb, 0x8310 => 0x8316, 0x8314 => 0x8319, 0x831a => 0x830b, _ => 0x8363 };
                if w.branch(take) { target } else { pc + 2 }
            }
            0x82e7 => { let v = w.dp_read(0x5d, 1); w.load(v, 1); pc + 2 }
            0x82e9 | 0x82ed | 0x830e | 0x8312 => { w.compare_immediate(match pc { 0x82e9 => 0x17, 0x82ed => 0x1c, 0x830e => 0x30, _ => 0x31 }, 1); pc + 2 }
            0x82f4 | 0x82fe => { w.mask(if pc == 0x82f4 { 2 } else { 0x80 }, 1); pc + 2 }
            0x82f8 | 0x8353 | 0x8356 | 0x835b | 0x835e => {
                w.drain(16); w.write(match pc { 0x82f8 => 0x308, 0x8353 => 0x37b, 0x8356 => 0x3f7, 0x835b => 0x3f5, _ => 0x3f6 }, 0, 1); pc + 3
            }
            0x8300 => { w.drain(6); w.push(w.cpu.a & 255, 1); pc + 1 }
            0x8305 => { w.drain(12); let v = w.pull(1); w.load(v, 1); pc + 1 }
            0x8306 | 0x832e | 0x8333 | 0x8336 | 0x8348 => {
                w.drain(16); w.write(match pc { 0x8306 => 0x308, 0x832e => 0x2e2, 0x8333 => 0x37b, 0x8336 => 0x3f7, _ => 0x2e0 }, w.cpu.a & 255, 1); pc + 3
            }
            0x8309 | 0x831f => { w.drain(8); if pc == 0x8309 { w.cpu.x = 4; } else { w.cpu.y = 4; } w.flags(4, 1); pc + 2 }
            0x830b => { w.drain(16); let v = w.read(0xc4a + u32::from(w.cpu.x), 1); w.load(v, 1); pc + 3 }
            0x8316 => { w.drain(22); w.write(0xc4a + u32::from(w.cpu.x), 0, 1); pc + 3 }
            0x8319 => { w.drain(6); w.cpu.x = w.cpu.x.wrapping_sub(1) & 255; w.flags(w.cpu.x, 1); pc + 1 }
            0x8321 | 0x8327 | 0x832c | 0x8331 | 0x8339 | 0x8342 | 0x8346 => {
                w.immediate(match pc { 0x8321 => 0x23, 0x8327 | 0x832c => 0x14, 0x8331 | 0x8346 => 1, 0x8339 => 0x0c, _ => 0x1c }, 1); pc + 2
            }
            0x833b | 0x8344 | 0x834b => { w.dp_store(match pc { 0x833b => 0x4b, 0x8344 => 0x5d, _ => 0x56 }, w.cpu.a & 255, 1); pc + 2 }
            0x833d => { w.drain(16); let v = w.read(0x2e2, 1).wrapping_sub(1) & 255; w.drain(6); w.flags(v, 1); w.write(0x2e2, v, 1); pc + 3 }
            0x8351 => { w.dp_store(0x4b, 0, 1); pc + 2 }
            0x8359 => { w.branch(true); 0x8363 }
            0x82d8 | 0x8361 | 0x8363 => { w.drain(6); if pc == 0x8363 { w.cpu.p |= CARRY; } else { w.cpu.p &= !CARRY; } pc + 1 }
            0x82d9 | 0x8362 | 0x8364 => { w.rts(); self.complete = true; w.cpu.pc }
            0x8301 | 0x831c | 0x8323 | 0x8329 | 0x834d => {
                let (target, far) = match pc { 0x8301 => (0x07f1a3, true), 0x831c => (0x079195, false),
                    0x8323 => (0x09912c, true), 0x8329 => (0x078028, false), _ => (0x0ed6dd, true) };
                let sp = w.cpu.sp; let ret = pc + if far { 4 } else { 3 };
                if far { w.operand_then_internal(3); w.push(7, 1); w.push(pc + 3, 2); }
                else { w.operand_then_internal(2); w.push(pc + 2, 2); }
                self.child = Some(target); self.child_return = Some((ret, sp)); self.pc = ret;
                w.cpu.bank = (target >> 16) as u8; w.cpu.pc = target as u16; return Some(w.cycles);
            }
            _ => unreachable!("transformation guard PC {pc:04x}"),
        };
        w.cpu.pc = self.pc; Some(w.cycles)
    }
}
