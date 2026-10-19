//! Link's physical wrapper, damage dispatch and ancilla movement suffix.
//! State handlers and out-of-line effects retain explicit near/far frames.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO}};
use snes::CpuMasterTimeline;
const STATES: [u16; 31] = [0x8109, 0x92d3, 0x86b5, 0xa804, 0x963b, 0x8872, 0x86b5,
    0x996c, 0xa50f, 0xa5f7, 0xa6d6, 0x894e, 0x8b74, 0x8dc6, 0x8e15, 0x8c69,
    0x8c69, 0x8f86, 0x915e, 0xab7c, 0xa9b1, 0x99ac, 0x9a5a, 0x83a1,
    0x8481, 0x8570, 0x85fb, 0x867b, 0x8365, 0xb416, 0xa804];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let i = address.checked_sub(0x078041)? as usize;
    (i < STATES.len() * 2).then(|| STATES[i / 2].to_le_bytes()[i & 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrapper_damage_state_table_and_ancilla_scans_match_source() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
        for a in 0x078041..0x07807f { assert_eq!(table_byte(a), Some(rom[(a - 0x040000) as usize])); }
        let mut boundaries = 0; let mut returns = 0;
        let mut states = std::collections::BTreeSet::new();
        let mut effects = std::collections::BTreeSet::new();
        let entries = [0x8000, 0x801c, 0x8028, 0x803d, 0x807f, 0x80c3, 0x80d5,
            0x80f7, 0x8102, 0x8689, 0x86a2];
        for entry in entries {
            for case in 0..512usize {
                let mut machine = snes::Snes::new(); machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                r[0x2e4] = if case & 1 != 0 { 0x80 } else { 0 };
                r[0x373] = [0, 1, 8, 255][case % 4]; r[0x37b] = if case & 4 != 0 { 1 } else { 0 };
                r[0x55] = if case & 8 != 0 { 1 } else { 0 };
                r[0x300] = if case & 16 != 0 { 1 } else { 0 }; r[0x3d] = if case & 32 != 0 { 1 } else { 0 };
                r[0x31f] = if case & 64 != 0 { 0 } else { 8 };
                r[0x5d] = (case % 31) as u8; r[0x1b] = u8::from(case & 128 != 0);
                r[0x3e9] = u8::from(case & 256 != 0);
                r[0xf36d] = [0, 8, 0xa8, 255][case % 4]; r[0] = [1, 8, 255, 0][case % 4];
                for i in 0..5 { r[0xc4a+i] = if i == case % 6 { [0, 5, 0x24, 0x2c][case / 6 % 4] } else { 0 }; }
                for i in 0..16 { r[0x1f0+i] = (i as u8).wrapping_mul(17).wrapping_add(case as u8); }
                r[0x20..0x22].copy_from_slice(&(case as u16).wrapping_mul(997).to_le_bytes());
                r[0x22..0x24].copy_from_slice(&(case as u16).wrapping_mul(71).to_le_bytes());
                let mut cpu = NativeNmiContext { a: 0xb726, x: 224, y: 9, d: 0,
                    db: if entry == 0x8000 { [0, 2, 7, 0x7e][case % 4] } else { 7 },
                    bank: 7, pc: entry, sp: 0x1f7, p: 0x30 | (case as u8 & 0xc3) };
                machine.cpu.a = cpu.a; machine.cpu.x = cpu.x; machine.cpu.y = cpu.y; machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank; machine.cpu.pc = cpu.pc; machine.cpu.sp = cpu.sp;
                machine.cpu.e = false; machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(4937,
                    snes::CpuRasterPosition::new(224, if case & 1 == 0 { 1300 } else { 500 }),
                    snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                let mut source = snes::RomCpuTimingProbe::new(machine, timeline,
                    snes::SourcePpuReadState::snes9x_reset()).unwrap();
                let mut clock = source.timeline().clone();
                let mut work = LinkMain { pc: entry, ..Default::default() };
                for _ in 0..1000 {
                    if work.complete() || work.child().is_some() { break; }
                    super::super::native_work::compare_instruction(
                        &format!("Link main entry={entry:04x} case={case} pc={:04x}", cpu.pc),
                        &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                        |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                    boundaries += 1;
                }
                match work.child() {
                    Some(LinkChild::State(pc)) => { states.insert(pc); }
                    Some(LinkChild::Auxiliary(pc)) => { effects.insert(pc); }
                    None => { assert!(work.complete()); returns += 1; }
                }
            }
        }
        assert_eq!(states, STATES.into_iter().collect());
        assert_eq!(effects, [0x07ae8f, 0x0dbb67, 0x08ed89, 0x08e7e6].into_iter().collect());
        assert!(returns > 0);
        eprintln!("5632 Link main fixtures, {boundaries} Source instruction boundaries, {returns} physical returns, all 31 state rows and four auxiliary frames");
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LinkChild { State(u16), Auxiliary(u32) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinkMain {
    pc: u16, near: Vec<(u16, u16)>, child: Option<LinkChild>,
    child_return: Option<(u8, u16, u16)>, complete: bool,
}
impl Default for LinkMain {
    fn default() -> Self { Self { pc: 0x8000, near: Vec::new(), child: None, child_return: None, complete: false } }
}
impl LinkMain {
    pub(crate) fn complete(&self) -> bool { self.complete }
    pub(crate) fn child(&self) -> Option<LinkChild> { self.child }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let child = self.child.take().expect("Link has no external child");
        let expected = self.child_return.take().expect("Link child requires its physical caller");
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), expected, "Link child must retire its real frame");
        if matches!(child, LinkChild::State(_)) { assert_eq!(self.near.pop(), Some((cpu.pc, cpu.sp))); }
        self.pc = cpu.pc;
    }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<u64> {
        if self.complete { return None; }
        assert!(self.child.is_none(), "Link child still owns the CPU");
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (7, self.pc, 0));
        let mut w = Work { cpu, clock, bus, cycles: 0 }; w.fetch();
        let pc = self.pc;
        self.pc = match pc {
            0x8000 | 0x8001 => { w.drain(6); w.push(u16::from(if pc == 0x8000 { w.cpu.db } else { 7 }), 1); pc + 1 }
            0x8002 | 0x801f => { w.drain(12); let v = w.pull(1); w.cpu.db = v as u8; w.flags(v, 1); pc + 1 }
            0x8003 => { w.width(0x20, true); pc + 2 }
            0x800f => { w.width(0x20, false); pc + 2 }
            0x8005 | 0x800a | 0x8084 | 0x80aa | 0x80d5 | 0x80db | 0x80e1 | 0x80fb | 0x8102 | 0x8689 => {
                let (a, width) = match pc { 0x8005 => (0x22, 2), 0x800a => (0x20, 2), 0x8084 => (0x55, 1),
                    0x80aa => (0x3d, 1), 0x80d5 => (0x1c, 1), 0x80db => (0x1d, 1), 0x80e1 => (0x10, 1), 0x8689 => (0x1b, 1), _ => (0x5d, 1) };
                let v = w.dp_read(a, width); w.load(v, width); pc + 2
            }
            0x8007 | 0x800c | 0x8036 | 0x80bb | 0x80e3 | 0x80f0 => {
                let (a, width) = match pc { 0x8007 => (0xfc2, 2), 0x800c => (0xfc4, 2), 0x8036 => (0xcf8, 1),
                    0x80bb | 0x80f0 => (0x31f, 1), _ => (0x10c, 1) };
                w.drain(16); w.write(a, w.cpu.a & if width == 1 { 255 } else { 65535 }, width); pc + 3
            }
            0x8011 | 0x8088 | 0x809b | 0x80ae | 0x80b1 => {
                w.drain(16); w.write(match pc { 0x8011 => 0xfc1, 0x8088 | 0x809b => 0x373, 0x80ae => 0xc4a, _ => 0x35f }, 0, 1); pc + 3
            }
            0x8014 | 0x807f | 0x8091 | 0x8096 | 0x809e | 0x80a5 | 0x80b4 | 0x868d => {
                let a = match pc { 0x8014 => 0x2e4, 0x807f | 0x8096 => 0x373, 0x8091 => 0x37b,
                    0x809e => 0xc4a, 0x80a5 => 0x300, 0x80b4 => 0x31f, _ => 0x3e9 };
                w.drain(16); let v = w.read(a, 1); w.load(v, 1); pc + 3
            }
            0x8017 | 0x8082 | 0x8086 | 0x8094 | 0x80a3 | 0x80a8 | 0x80ac | 0x80b7 | 0x80cf | 0x80d3 | 0x80fd | 0x868b | 0x8690 | 0x8699 | 0x86a0 | 0x86a9 | 0x86b2 => {
                let taken = match pc { 0x8017 | 0x8094 | 0x80a3 | 0x80a8 | 0x80b7 | 0x868b | 0x8699 | 0x86a9 => w.cpu.p & ZERO == 0,
                    0x80d3 => w.cpu.p & CARRY == 0, 0x86a0 | 0x86b2 => w.cpu.p & NEGATIVE == 0, _ => w.cpu.p & ZERO != 0 };
                let target = match pc { 0x8017 => 0x801c, 0x8082 | 0x8094 => 0x80fb, 0x8086 => 0x8091,
                    0x80a3 | 0x80a8 | 0x80ac => 0x80b4, 0x80b7 => 0x80be, 0x80cf => 0x80d5,
                    0x80d3 => 0x80f7, 0x80fd => 0x8102, 0x868b | 0x8690 => 0x86a2,
                    0x8699 => 0x869f, 0x86a0 => 0x8694, 0x86a9 => 0x86b1, _ => 0x86a4 };
                if w.branch(taken) { target } else { pc + 2 }
            }
            0x8019 | 0x801c | 0x8028 | 0x80c0 => {
                let target = match pc { 0x8019 => 0x807f, 0x801c => 0x8689, 0x8028 => 0x8036, _ => 0x8028 };
                let sp = w.cpu.sp; w.operand_then_internal(2); w.push(pc + 2, 2);
                self.near.push((pc + 3, sp)); target
            }
            0x802e | 0x8040 | 0x86b4 => {
                w.rts(); if let Some((ret, sp)) = self.near.pop() { assert_eq!((w.cpu.pc, w.cpu.sp), (ret, sp)); }
                else { self.complete = true; } w.cpu.pc
            }
            0x8020 => { w.rtl(); self.complete = true; w.cpu.pc }
            0x808b | 0x808d => { w.dp_store(if pc == 0x808b { 0x4d } else { 0x46 }, 0, 1); pc + 2 }
            0x808f | 0x86af => { w.branch(true); if pc == 0x808f { 0x80fb } else { 0x86b4 } }
            0x8099 | 0x80e8 | 0x80ec => { w.dp_store(match pc { 0x8099 => 0, 0x80e8 => 0x10, _ => 0x11 }, w.cpu.a & 255, 1); pc + 2 }
            0x80a1 | 0x80cd | 0x80d1 | 0x8697 | 0x86a7 => { w.compare_immediate(match pc { 0x80a1 => 5, 0x80cd => 0, 0x80d1 => 0xa8, 0x8697 => 0x24, _ => 0x2c }, 1); pc + 2 }
            0x80b9 | 0x80be | 0x80e6 | 0x80ea | 0x80ee => { w.immediate(match pc { 0x80b9 => 0x3a, 0x80be => 0x26, 0x80e6 => 0x12, 0x80ea => 1, _ => 0 }, 1); pc + 2 }
            0x80c3 => { w.drain(16); let v = w.read(0xcfc, 1).wrapping_add(1) & 255; w.drain(6); w.flags(v, 1); w.write(0xcfc, v, 1); pc + 3 }
            0x80c6 => { w.drain(24); let v = w.read(0x7ef36d, 1); w.load(v, 1); pc + 4 }
            0x80ca => { w.drain(6); w.cpu.p |= CARRY; pc + 1 }
            0x80cb => { let rhs = w.dp_read(0, 1); w.sbc(rhs, 1); pc + 2 }
            0x80d7 | 0x80dd | 0x80f3 | 0x80f7 => { w.drain(24); w.write(match pc { 0x80d7 => 0x7ec211, 0x80dd => 0x7ec212, 0x80f3 => 0x7ef372, _ => 0x7ef36d }, w.cpu.a & 255, 1); pc + 4 }
            0x8104 => { w.shift_left(1); pc + 1 }
            0x8105 => { w.drain(6); w.cpu.x = w.cpu.a & 255; w.flags(w.cpu.x, 1); pc + 1 }
            0x8106 => {
                w.operand_then_internal(2); let target = w.read(0x078041 + u32::from(w.cpu.x), 2);
                self.child = Some(LinkChild::State(target)); self.child_return = self.near.last().map(|(pc, sp)| (7, *pc, *sp)); target
            }
            0x8692 | 0x86a2 => { w.drain(8); w.cpu.x = 4; w.flags(4, 1); pc + 2 }
            0x8694 | 0x86a4 => { w.drain(16); let v = w.read(0xc4a + u32::from(w.cpu.x), 1); w.load(v, 1); pc + 3 }
            0x869f | 0x86b1 => { w.drain(6); w.cpu.x = w.cpu.x.wrapping_sub(1) & 255; w.flags(w.cpu.x, 1); pc + 1 }
            0x8039 | 0x80ff | 0x869b | 0x86ab => {
                let (target, far) = match pc { 0x8039 => (0x0dbb67, true), 0x80ff => (0x07ae8f, false), 0x869b => (0x08ed89, true), _ => (0x08e7e6, true) };
                let sp = w.cpu.sp; let ret = pc + if far { 4 } else { 3 };
                if far { w.operand_then_internal(3); w.push(7, 1); w.push(pc + 3, 2); }
                else { w.operand_then_internal(2); w.push(pc + 2, 2); }
                self.child = Some(LinkChild::Auxiliary(target)); self.child_return = Some((7, ret, sp));
                self.pc = ret; w.cpu.bank = (target >> 16) as u8; w.cpu.pc = target as u16; return Some(w.cycles);
            }
            0x803d => { w.drain(16); let rhs = w.read(0xcf8, 1); w.load((w.cpu.a | rhs) & 255, 1); pc + 3 }
            0x802b => { w.drain(16); w.write(0x12e, w.cpu.a & 255, 1); pc + 3 }
            _ => unreachable!("Link main PC {pc:04x}"),
        };
        w.cpu.pc = self.pc; Some(w.cycles)
    }
}
