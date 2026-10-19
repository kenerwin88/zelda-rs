//! Ordinary overworld control keeps menu decisions and movement callees on
//! the inherited near frame. Unowned callees remain explicit physical calls.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, ZERO}};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerChild { SelectMenu, EntranceAnimation, LinkMain, SuperBomb, Chr, Camera,
    UseEntrance, PaletteAnimation, Transitions, SpecialWorldExit }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldPlayer { pc: u16, child: Option<PlayerChild>, return_sp: u16, complete: bool }
impl Default for OverworldPlayer {
    fn default() -> Self { Self { pc: 0xa53c, child: None, return_sp: 0, complete: false } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_and_link_wrapper_keep_inherited_clock_and_real_return() {
        use super::super::link_main::LinkMain;
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
        let mut instructions = 0; let mut returned_links = 0; let mut link_children = 0;
        for case in 0..512usize {
            let mut machine = snes::Snes::new(); machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            let r = &mut machine.ram;
            r[0x10] = 9; r[0x4b4] = 255;
            r[0x2e4] = u8::from(case & 1 != 0);
            r[0x5d] = (case % 31) as u8;
            r[0x373] = if case & 2 != 0 { 8 } else { 0 };
            r[0x55] = u8::from(case & 4 != 0);
            r[0x1b] = u8::from(case & 8 != 0);
            r[0x3e9] = u8::from(case & 16 != 0);
            for i in 0..5 { r[0xc4a + i] = if i == case % 6 { [0, 0x24, 0x2c][case / 6 % 3] } else { 0 }; }
            let mut cpu = NativeNmiContext { a: 0xb702, x: 224, y: 9, d: 0, db: 0, bank: 2, pc: 0xa53c, sp: 0x1fa, p: 0x30 };
            machine.cpu.a = cpu.a; machine.cpu.x = cpu.x; machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank; machine.cpu.pc = cpu.pc; machine.cpu.sp = cpu.sp;
            machine.cpu.e = false; machine.cpu.unpack_flags(cpu.p);
            let mut ram = machine.ram.clone();
            let timeline = CpuMasterTimeline::at_raster(4937, snes::CpuRasterPosition::new(224, 1300),
                snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
            let mut source = snes::RomCpuTimingProbe::new(machine, timeline, snes::SourcePpuReadState::snes9x_reset()).unwrap();
            let mut clock = source.timeline().clone(); let mut parent = OverworldPlayer::default();
            let mut link: Option<LinkMain> = None;
            for _ in 0..1000 {
                if parent.child() == Some(PlayerChild::LinkMain) && link.is_none() {
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (7, 0x8000, 0x1f7));
                    link = Some(Default::default());
                }
                if link.as_ref().is_some_and(|work| work.child().is_some()) { link_children += 1; break; }
                if link.is_none() && (parent.child().is_some() || parent.complete()) { break; }
                super::super::native_work::compare_instruction(
                    &format!("player/Link case={case} pc={:02x}:{:04x}", cpu.bank, cpu.pc),
                    &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                    |cpu, clock, bus| {
                        if let Some(work) = &mut link { work.step(cpu, clock, bus).unwrap(); }
                        else { parent.step(cpu, clock, bus).unwrap(); }
                    });
                instructions += 1;
                if link.as_ref().is_some_and(LinkMain::complete) {
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (2, 0xa5a6, 0x1fa, 0));
                    parent.return_from_child(&cpu); link = None; returned_links += 1;
                }
            }
        }
        assert!(returned_links > 0 && link_children > 0);
        eprintln!("512 player/Link compositions, {instructions} Source instruction boundaries, {returned_links} real Link returns, {link_children} explicit Link children");
    }
    #[test]
    fn control_menu_area_and_all_callee_frames_match_source() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
        let mut instructions = 0;
        let mut returns = 0;
        let mut children = std::collections::BTreeSet::new();
        let entries = [0xa53c, 0xa58e, 0xa5a6, 0xa5b1, 0xa5d3, 0xa5e9,
            0xa58a, 0xa59c, 0xa5a2, 0xa5ad, 0xa5cc, 0xa5d0, 0xa5d9, 0xa5dd, 0xa5e1, 0xa5e6];
        for entry in entries {
            for case in 0..256usize {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                r[0x112] = if case & 1 != 0 { 0x80 } else { 0 };
                r[0x2e4] = if case & 2 != 0 { 2 } else { 0 };
                r[0xffc] = if case & 4 != 0 { 4 } else { 0 };
                r[0x4c6] = if case & 8 != 0 { 8 } else { 0 };
                r[0xf4] = case as u8;
                r[0xf6] = (case as u8).rotate_left(2);
                r[0xf0] = (case as u8).rotate_left(1);
                r[0x10] = if case & 1 != 0 { 11 } else { 9 };
                r[0x4b4] = if case & 1 != 0 { 255 } else { case as u8 };
                r[0x1ce8] = case as u8;
                r[0x20..0x22].copy_from_slice(&(case as u16 * 997).to_le_bytes());
                r[0x22..0x24].copy_from_slice(&(case as u16 * 71).to_le_bytes());
                for i in 0..16 { r[0x1f0+i] = (i as u8).wrapping_mul(17).wrapping_add(case as u8); }
                let mut cpu = NativeNmiContext { a: 0xb702, x: 224, y: 9, d: 0, db: 0, bank: 2,
                    pc: entry, sp: 0x1fa, p: 0x30 | (case as u8 & 0xc3) };
                machine.cpu.a = cpu.a; machine.cpu.x = cpu.x; machine.cpu.y = cpu.y;
                machine.cpu.pc = cpu.pc; machine.cpu.k = cpu.bank; machine.cpu.sp = cpu.sp;
                machine.cpu.e = false; machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(4937,
                    snes::CpuRasterPosition::new(224, if case & 1 == 0 { 1300 } else { 500 }),
                    snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                let mut source = snes::RomCpuTimingProbe::new(machine, timeline,
                    snes::SourcePpuReadState::snes9x_reset()).unwrap();
                let mut clock = source.timeline().clone();
                let mut work = OverworldPlayer { pc: entry, ..Default::default() };
                for _ in 0..100 {
                    if work.complete() || work.child().is_some() { break; }
                    super::super::native_work::compare_instruction(
                        &format!("overworld player entry={entry:04x} case={case} pc={:04x}", cpu.pc),
                        &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                        |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                    instructions += 1;
                }
                if let Some(child) = work.child() { children.insert(format!("{child:?}")); }
                else { assert!(work.complete()); returns += 1; }
            }
        }
        assert_eq!(children.len(), 10);
        assert!(returns > 0);
        eprintln!("4096 overworld control fixtures, {instructions} Source instruction boundaries, {returns} physical near returns, all ten callee frames");
    }
}
impl OverworldPlayer {
    pub(crate) fn complete(&self) -> bool { self.complete }
    pub(crate) fn child(&self) -> Option<PlayerChild> { self.child }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        assert!(self.child.is_some(), "overworld player has no active child");
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, self.pc, self.return_sp),
            "overworld player child must retire its actual return");
        self.child = None;
    }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<u64> {
        if self.complete { return None; }
        assert!(self.child.is_none(), "overworld player child still owns the CPU");
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db), (2, self.pc, 0, 0));
        let mut w = Work { cpu, clock, bus, cycles: 0 }; w.fetch();
        let pc = self.pc;
        self.pc = match pc {
            0xa53c | 0xa577 | 0xa597 | 0xa5a6 => {
                let a = match pc { 0xa53c => 0x112, 0xa577 => 0x1ce8, 0xa597 => 0x4c6, _ => 0x4b4 };
                w.drain(16); let v = w.read(a, 1); w.load(v, 1); pc + 3
            }
            0xa53f | 0xa542 | 0xa545 | 0xa5c3 => {
                let width = if pc == 0xa5c3 { 2 } else { 1 };
                let a = match pc { 0xa53f => 0x2e4, 0xa542 => 0xffc, 0xa545 => 0x4c6, _ => 0x700 };
                w.drain(16); let v = w.read(a, width);
                w.load((w.cpu.a | v) & if width == 1 { 255 } else { 65535 }, width); pc + 3
            }
            0xa54d | 0xa55a | 0xa567 | 0xa571 | 0xa587 | 0xa5b3 | 0xa5be | 0xa5d3 => {
                let a = match pc { 0xa54d => 0xf4, 0xa55a => 0xf6, 0xa571 => 0xf0, 0xa5b3 => 0x20, 0xa5be => 0x22, _ => 0x10 };
                let width = if matches!(pc, 0xa5b3 | 0xa5be) { 2 } else { 1 };
                let v = w.dp_read(a, width); w.load(v, width); pc + 2
            }
            0xa54f | 0xa55c | 0xa573 | 0xa5b5 | 0xa5c0 => {
                let (v, width) = match pc { 0xa54f => (0x10, 1), 0xa55c => (0x40, 1), 0xa573 => (0x20, 1), _ => (0x1e00, 2) };
                w.mask(v, width); pc + 1 + u16::from(width)
            }
            0xa548 | 0xa551 | 0xa55e | 0xa575 | 0xa59a | 0xa5ab | 0xa5d7 => {
                let target = match pc { 0xa548 => 0xa54d, 0xa551 => 0xa55a, 0xa55e => 0xa571,
                    0xa575 => 0xa597, 0xa59a => 0xa5a0, 0xa5ab => 0xa5b1, _ => 0xa5e6 };
                if w.branch(w.cpu.p & ZERO != 0) { target } else { pc + 2 }
            }
            0xa54a => { w.drain(16); 0xa597 }
            0xa558 | 0xa595 | 0xa5e4 => { w.branch(true); match pc { 0xa558 | 0xa595 => 0xa565, _ => 0xa5e9 } }
            0xa553 | 0xa560 => { w.drain(16); w.write(0x200, 0, 1); pc + 3 }
            0xa556 | 0xa563 | 0xa56c | 0xa57f | 0xa593 => {
                let (v, width) = match pc { 0xa556 => (1, 1), 0xa563 => (7, 1), 0xa56c => (14, 1), 0xa593 => (11, 1), _ => (0x186, 2) };
                w.immediate(v, width); pc + 1 + u16::from(width)
            }
            0xa565 | 0xa56e | 0xa58f => { w.dp_store(if pc == 0xa565 { 0x11 } else { 0x10 }, w.cpu.a & 255, 1); pc + 2 }
            0xa569 | 0xa57a | 0xa582 | 0xa5bb | 0xa5c7 => {
                let (a, width) = match pc { 0xa569 => (0x10c, 1), 0xa57a => (0x1cf4, 1), 0xa582 => (0x1cf0, 2), _ => (0x700, 2) };
                w.drain(16); w.write(a, w.cpu.a & if width == 1 { 255 } else { 65535 }, width); pc + 3
            }
            0xa57d | 0xa5b1 => { w.width(0x20, true); pc + 2 }
            0xa585 | 0xa5ca | 0xa5e9 => { w.width(0x20, false); pc + 2 }
            0xa5a0 => { w.width(0x30, false); pc + 2 }
            0xa589 => { w.drain(6); w.push(w.cpu.a & 255, 1); pc + 1 }
            0xa58e => { w.drain(12); let v = w.pull(1); w.load(v, 1); pc + 1 }
            0xa591 => { w.dp_store(0xb0, 0, 1); pc + 2 }
            0xa5a9 | 0xa5d5 => { w.compare_immediate(if pc == 0xa5a9 { 255 } else { 11 }, 1); pc + 2 }
            0xa5b8..=0xa5ba => { w.shift_left(2); pc + 1 }
            0xa5c6 => { w.drain(12); let v = w.cpu.a.rotate_left(8); w.cpu.a = v; w.flags(v & 255, 1); pc + 1 }
            0xa58a | 0xa59c | 0xa5a2 | 0xa5ad | 0xa5cc | 0xa5d9 | 0xa5dd | 0xa5d0 | 0xa5e1 | 0xa5e6 => {
                let (target, child, far) = match pc {
                    0xa58a => (0x0ffdaa, PlayerChild::SelectMenu, true),
                    0xa59c => (0x1bcac4, PlayerChild::EntranceAnimation, true),
                    0xa5a2 => (0x078000, PlayerChild::LinkMain, true),
                    0xa5ad => (0x0afda8, PlayerChild::SuperBomb, true),
                    0xa5cc => (0x00e3fa, PlayerChild::Chr, true),
                    0xa5d0 => (0x02bb90, PlayerChild::Camera, false),
                    0xa5d9 => (0x1bbbf4, PlayerChild::UseEntrance, true),
                    0xa5dd => (0x0ef582, PlayerChild::PaletteAnimation, true),
                    0xa5e1 => (0x02a9c4, PlayerChild::Transitions, false),
                    _ => (0x02ab7b, PlayerChild::SpecialWorldExit, false),
                };
                self.return_sp = w.cpu.sp;
                if far { w.operand_then_internal(3); w.push(2, 1); w.push(pc + 3, 2); self.pc = pc + 4; }
                else { w.drain(8); w.drain(6); w.drain(8); w.push(pc + 2, 2); self.pc = pc + 3; }
                self.child = Some(child); w.cpu.bank = (target >> 16) as u8; w.cpu.pc = target as u16;
                return Some(w.cycles);
            }
            0xa570 | 0xa5eb => { w.rts(); self.complete = true; w.cpu.pc }
            _ => unreachable!("overworld player PC {pc:04x}"),
        };
        w.cpu.pc = self.pc; Some(w.cycles)
    }
}
