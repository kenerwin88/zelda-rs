//! Normal movement's entry guards and outdoor coordinate cache. Calls which
//! still lack a native callee retain their original near or far frame.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, CARRY, ZERO}};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NormalBoundary { Call(u32), Tail(u32), Returned }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinkNormalPrefix { pc: u32, near: Vec<(u16, u16)>, boundary: Option<NormalBoundary>, child_return: Option<(u8, u16, u16)>, native_guard: bool, guard: Option<super::link_transform_guard::LinkTransformGuard> }
impl Default for LinkNormalPrefix {
    fn default() -> Self { Self { pc: 0x078109, near: Vec::new(), boundary: None, child_return: None, native_guard: false, guard: None } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_entry_outdoor_cache_and_recoil_frames_match_source() {
        compare_prefix(false);
    }
    #[test]
    fn normal_entry_and_transform_guard_keep_real_return_and_clock() {
        compare_prefix(true);
    }
    fn compare_prefix(native_guard: bool) {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
        let mut count = 0; let mut returned = 0;
        let mut calls = std::collections::BTreeSet::new();
        let mut tails = std::collections::BTreeSet::new();
        let entries = [0x078109, 0x07810c, 0x07811d, 0x078128, 0x078129, 0x07815c,
            0x07816d, 0x078170, 0x078189, 0x07818c, 0x078193, 0x0781a0];
        for entry in entries {
            for case in 0..256usize {
                let mut machine = snes::Snes::new(); machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                r[0x1b] = u8::from(case & 1 != 0); r[0xf5] = case as u8;
                r[0x5d] = [0, 17, 23, 30][case % 4]; r[0x4d] = u8::from(case & 2 != 0);
                r[0x360] = u8::from(case & 4 != 0); r[0x55] = u8::from(case & 8 != 0);
                r[0x37f] = case as u8;
                if native_guard {
                    r[0x3f5] = u8::from(case & 32 != 0);
                    r[0x3f6] = u8::from(case & 16 != 0);
                    r[0x3f7] = u8::from(case & 64 != 0);
                    r[0x2e2] = case as u8;
                }
                for a in [0xe2, 0xe8, 0x20, 0x22, 0xa6, 0xa9, 0x600, 0x604, 0x608, 0x60c,
                    0x610, 0x612, 0x614, 0x616, 0x618, 0x61c] {
                    r[a..a+2].copy_from_slice(&(case as u16).wrapping_mul(997).wrapping_add(a as u16).to_le_bytes());
                }
                for a in [0x2f, 0xee, 0x476, 0x6c, 0xa4, 0x3a, 0x50, 0x67] { r[a] = case as u8; }
                for i in 0..16 { r[0x1e8+i] = (i as u8).wrapping_mul(17).wrapping_add(case as u8); }
                let mut cpu = NativeNmiContext { a: 0xb702, x: 224, y: 9, d: 0, db: 7, bank: 7,
                    pc: entry as u16, sp: 0x1f4,
                    p: 0x30 | (case as u8 & 0xc2) | u8::from(case & 16 != 0) };
                machine.cpu.a = cpu.a; machine.cpu.x = cpu.x; machine.cpu.y = cpu.y; machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank; machine.cpu.pc = cpu.pc; machine.cpu.sp = cpu.sp;
                machine.cpu.e = false; machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(4937, snes::CpuRasterPosition::new(224, 1300),
                    snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                let mut source = snes::RomCpuTimingProbe::new(machine, timeline,
                    snes::SourcePpuReadState::snes9x_reset()).unwrap();
                let mut clock = source.timeline().clone();
                let mut work = LinkNormalPrefix { pc: entry, ..Default::default() };
                if native_guard { work = work.with_native_transform_guard(); }
                for _ in 0..1000 {
                    if work.boundary().is_some() { break; }
                    super::super::native_work::compare_instruction(
                        &format!("normal Link entry={entry:06x} case={case} pc={:02x}:{:04x}", cpu.bank, cpu.pc),
                        &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                        |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                    count += 1;
                }
                match work.boundary().unwrap() {
                    NormalBoundary::Call(pc) => { calls.insert(pc); }
                    NormalBoundary::Tail(pc) => { tails.insert(pc); }
                    NormalBoundary::Returned => returned += 1,
                }
            }
        }
        if native_guard {
            assert!(!calls.contains(&0x0782da));
            for pc in [0x07f1a3, 0x0ed6dd, 0x07983a, 0x07ae54, 0x079d84, 0x07802f, 0x07b5d6] {
                assert!(calls.contains(&pc), "uncovered composed guard call {pc:06x}");
            }
        } else {
            assert_eq!(calls, [0x0782da, 0x07983a, 0x07ae54, 0x079d84, 0x07802f, 0x07b5d6].into_iter().collect());
        }
        assert_eq!(tails, [0x0783a1, 0x07996c, 0x0786b5].into_iter().collect());
        assert!(returned > 0);
        eprintln!("3072 normal Link fixtures native_guard={native_guard}, {count} Source instruction boundaries, {returned} real near returns, {} call frames and three tail transfers", calls.len());
    }
}
impl LinkNormalPrefix {
    pub(crate) fn with_native_transform_guard(mut self) -> Self { self.native_guard = true; self }
    pub(crate) fn boundary(&self) -> Option<NormalBoundary> {
        if self.native_guard && self.boundary == Some(NormalBoundary::Call(0x0782da)) {
            self.guard.as_ref().and_then(|work| work.child()).map(NormalBoundary::Call)
        } else { self.boundary }
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        if let Some(guard) = self.guard.as_mut().filter(|work| work.child().is_some()) {
            guard.return_from_child(cpu); return;
        }
        assert!(matches!(self.boundary, Some(NormalBoundary::Call(_))));
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), self.child_return.take().unwrap());
        self.pc = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc); self.boundary = None;
    }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<u64> {
        if self.native_guard && self.boundary == Some(NormalBoundary::Call(0x0782da)) {
            let work = self.guard.get_or_insert_with(Default::default);
            let cycles = work.step(cpu, clock, bus);
            if work.complete() { self.guard = None; self.return_from_child(cpu); }
            return cycles;
        }
        if self.boundary.is_some() { return None; }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db), ((self.pc >> 16) as u8, self.pc as u16, 0, 7));
        let mut w = Work { cpu, clock, bus, cycles: 0 }; w.fetch(); let pc = self.pc;
        self.pc = match pc {
            0x078109 => { let sp = w.cpu.sp; w.operand_then_internal(2); w.push(0x810b, 2); self.near.push((0x810c, sp)); 0x07f514 }
            0x07f514 | 0x07810c | 0x07811f | 0x07812c | 0x078147 | 0x07815c | 0x078169 | 0x078181 | 0x078193 => {
                let a = match pc { 0x07f514 | 0x078193 => 0x1b, 0x07810c => 0xf5, 0x07811f => 0x5d,
                    0x07812c => 0x4d, 0x078147 => 0x3a, 0x07815c => 0x50, 0x078169 => 0x55, _ => 0x67 };
                let v = w.dp_read(a, 1); w.load(v, 1); pc + 2
            }
            0x07f516 | 0x078110 | 0x07811d | 0x078123 | 0x07812e | 0x078167 | 0x07816b | 0x078195 => {
                let take = match pc { 0x07f516 | 0x078123 | 0x078195 => w.cpu.p & ZERO == 0,
                    0x07811d => w.cpu.p & CARRY == 0, _ => w.cpu.p & ZERO != 0 };
                let target = match pc { 0x07f516 => 0x07f51c, 0x078110 => 0x07811a, 0x07811d => 0x078129,
                    0x078123 => 0x078128, 0x07812e => 0x0781a0, 0x078167 => 0x078193, 0x07816b => 0x078170, _ => 0x078197 };
                if w.branch(take) { target } else { pc + 2 }
            }
            0x07f518 => { w.operand_then_internal(3); w.push(7, 1); w.push(0xf51b, 2); 0x01ff28 }
            0x01ff28 => { w.width(0x20, true); pc + 2 }
            0x01ff94 => { w.width(0x20, false); pc + 2 }
            0x01ff2a | 0x01ff30 | 0x01ff36 | 0x01ff3c | 0x01ff88 | 0x01ff8e | 0x01ff96 | 0x01ff9c | 0x01ffa9 | 0x01ffaf => {
                let (a, width) = match pc { 0x01ff2a => (0xe2, 2), 0x01ff30 => (0xe8, 2), 0x01ff36 => (0x20, 2),
                    0x01ff3c => (0x22, 2), 0x01ff88 => (0xa6, 2), 0x01ff8e => (0xa9, 2),
                    0x01ff96 => (0x2f, 1), 0x01ff9c => (0xee, 1), 0x01ffa9 => (0x6c, 1), _ => (0xa4, 1) };
                let v = w.dp_read(a, width); w.load(v, width); pc + 2
            }
            0x01ff42 | 0x01ff49 | 0x01ff50 | 0x01ff57 | 0x01ff5e | 0x01ff65 | 0x01ff6c | 0x01ff73 | 0x01ff7a | 0x01ff81 | 0x01ffa2 | 0x078112 | 0x078164 => {
                let (a, width) = match pc { 0x01ff42 => (0x600, 2), 0x01ff49 => (0x604, 2), 0x01ff50 => (0x608, 2),
                    0x01ff57 => (0x60c, 2), 0x01ff5e => (0x610, 2), 0x01ff65 => (0x612, 2), 0x01ff6c => (0x614, 2),
                    0x01ff73 => (0x616, 2), 0x01ff7a => (0x618, 2), 0x01ff81 => (0x61c, 2),
                    0x01ffa2 => (0x476, 1), 0x078112 => (0x37f, 1), _ => (0x360, 1) };
                w.drain(16); let v = w.read(a, width); w.load(v, width); pc + 3
            }
            0x01ff2c | 0x01ff32 | 0x01ff38 | 0x01ff3e | 0x01ff45 | 0x01ff4c | 0x01ff53 | 0x01ff5a | 0x01ff61 | 0x01ff68 | 0x01ff6f | 0x01ff76 | 0x01ff7d | 0x01ff84 | 0x01ff8a | 0x01ff90 | 0x01ff98 | 0x01ff9e | 0x01ffa5 | 0x01ffab | 0x01ffb1 => {
                let i = [0x01ff2c, 0x01ff32, 0x01ff38, 0x01ff3e, 0x01ff45, 0x01ff4c, 0x01ff53, 0x01ff5a,
                    0x01ff61, 0x01ff68, 0x01ff6f, 0x01ff76, 0x01ff7d, 0x01ff84, 0x01ff8a, 0x01ff90,
                    0x01ff98, 0x01ff9e, 0x01ffa5, 0x01ffab, 0x01ffb1].iter().position(|a| *a == pc).unwrap();
                let (a, width) = if i < 16 { (0x7ec180 + i as u32 * 2, 2) } else { (0x7ec1a6 + (i - 16) as u32, 1) };
                w.drain(24); w.write(a, w.cpu.a & if width == 1 { 255 } else { 65535 }, width); pc + 4
            }
            0x01ffb5 => { w.rtl(); assert_eq!((w.cpu.bank, w.cpu.pc), (7, 0xf51c)); 0x07f51c }
            0x07f51c => { w.rts(); let (ret, sp) = self.near.pop().unwrap(); assert_eq!((w.cpu.pc, w.cpu.sp), (ret, sp)); 0x070000 | u32::from(ret) }
            0x07810e | 0x078149 | 0x07815e | 0x078183 => { w.mask(match pc { 0x07810e => 0x80, 0x078149 => 0xbf, 0x07815e => 0xfe, _ => 0xf0 }, 1); pc + 2 }
            0x078115 => { w.drain(8); w.load((w.cpu.a ^ 1) & 255, 1); pc + 2 }
            0x078117 | 0x078175 => { w.drain(16); w.write(if pc == 0x078117 { 0x37f } else { 0x37b }, w.cpu.a & 255, 1); pc + 3 }
            0x078121 => { w.compare_immediate(0x17, 1); pc + 2 }
            0x078125 | 0x078190 | 0x07819d => { w.operand_then_internal(2); let target = match pc { 0x078125 => 0x0783a1, 0x078190 => 0x07996c, _ => 0x0786b5 }; self.boundary = Some(NormalBoundary::Tail(target)); target }
            0x078128 => { w.rts(); self.boundary = Some(NormalBoundary::Returned); 0x070000 | u32::from(w.cpu.pc) }
            0x078129 | 0x078130 | 0x078133 | 0x078136 | 0x078139 | 0x07813c | 0x07813f | 0x078142 | 0x07814d | 0x078150 | 0x078153 | 0x078178 | 0x0781a8 => {
                let a = match pc { 0x078129 => 0x2ca, 0x078130 | 0x078178 => 0x300 + u32::from(pc == 0x078130),
                    0x078133 => 0x37a, 0x078136 => 0x20b, 0x078139 => 0x350, 0x07813c => 0x30d,
                    0x07813f => 0x30e, 0x078142 => 0x30a, 0x07814d => 0x308, 0x078150 => 0x309,
                    0x078153 => 0x376, _ => 0x2c6 };
                w.drain(16); w.write(a, 0, 1); pc + 3
            }
            0x078145 | 0x078156 | 0x078162 | 0x07817f | 0x078197 => { w.dp_store(match pc { 0x078145 => 0x3b, 0x078156 => 0x48, 0x078162 => 0x25, 0x07817f => 0x2e, _ => 0x6b }, 0, 1); pc + 2 }
            0x07814b | 0x078160 | 0x07817d | 0x078185 | 0x07818e | 0x07819b | 0x0781a2 | 0x0781a4 | 0x0781a6 => {
                w.dp_store(match pc { 0x07814b => 0x3a, 0x078160 => 0x50, 0x07817d => 0x3d, 0x078185 => 0x67,
                    0x07818e | 0x07819b => 0x5d, 0x0781a2 => 0x24, 0x0781a4 => 0x25, _ => 0x29 }, w.cpu.a & 255, 1); pc + 2
            }
            0x078173 | 0x07817b | 0x078187 | 0x07818c | 0x078199 | 0x0781a0 => {
                w.immediate(match pc { 0x078173 => 1, 0x07817b | 0x078199 => 2, 0x078187 => 0x2b, 0x07818c => 7, _ => 255 }, 1); pc + 2
            }
            0x07811a | 0x078158 | 0x07816d | 0x078170 | 0x078189 | 0x0781ab => {
                let (target, far) = match pc { 0x07811a => (0x0782da, false), 0x078158 => (0x07983a, true),
                    0x07816d => (0x07ae54, false), 0x078170 => (0x079d84, false), 0x078189 => (0x07802f, false), _ => (0x07b5d6, false) };
                let sp = w.cpu.sp; let ret = pc as u16 + if far { 4 } else { 3 };
                if far { w.operand_then_internal(3); w.push(7, 1); w.push(pc as u16 + 3, 2); }
                else { w.operand_then_internal(2); w.push(pc as u16 + 2, 2); }
                self.boundary = Some(NormalBoundary::Call(target)); self.child_return = Some((7, ret, sp)); target
            }
            _ => unreachable!("normal Link prefix PC {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8; w.cpu.pc = self.pc as u16; Some(w.cycles)
    }
}
