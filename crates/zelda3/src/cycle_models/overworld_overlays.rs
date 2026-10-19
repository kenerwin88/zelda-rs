//! Retained overlay selection, cache, and restoration caller. The map loader
//! receives its real near frame; no NMI counts or reconstructed clock enter here.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, CARRY, ZERO}};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldOverlays {
    pc: u16,
    child: bool,
    child_return_sp: u16,
    complete: bool,
}
impl OverworldOverlays {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0xaf19 | 0xaf1e));
        Self { pc: entry, child: false, child_return_sp: 0, complete: false }
    }
    pub(crate) fn complete(&self) -> bool { self.complete }
    pub(crate) fn map_loader_pending(&self) -> bool { self.child }
    pub(crate) fn return_from_map_loader(&mut self, cpu: &NativeNmiContext) {
        assert!(self.child);
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xb094, self.child_return_sp));
        self.child = false;
        self.pc = 0xb094;
    }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline,
        bus: impl WorkBus) -> Option<u64> {
        if self.complete || self.child { return None; }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (2, self.pc, 0, 0));
        let mut w = Work { cpu, clock, bus, cycles: 0 };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let xw = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let a = w.cpu.a & if m == 1 { 255 } else { 65535 };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xaf19 | 0xaf97 | 0xb035 | 0xb039 | 0xb03d | 0xb050 | 0xb06a | 0xb079 => {
                w.immediate(match pc { 0xaf19 => 5, 0xaf97 | 0xb03d => 1,
                    0xb035 => 0x82, 0xb039 => 0x16, 0xb050 => 0x72, _ => 0x20 }, m);
                pc + 1 + u16::from(m)
            }
            0xaf1e | 0xb0a0 | 0xaf95 | 0xaf9c | 0xb033 | 0xb0cf => {
                w.drain(14);
                let mask = if matches!(pc, 0xb0a0 | 0xb0cf) { 0x20 } else { 0x30 };
                if matches!(pc, 0xaf1e | 0xb0a0) { w.cpu.p &= !mask; }
                else { w.cpu.p |= mask; if mask & 0x10 != 0 { w.cpu.x &= 255; w.cpu.y &= 255; } }
                pc + 2
            }
            0xaf20 | 0xaf26 | 0xaf2c | 0xaf32 | 0xaf58 | 0xaf62 | 0xafa8 | 0xafc6 |
            0xb011 | 0xb01e | 0xb094 | 0xb09a => {
                let at = match pc { 0xaf26 | 0xb011 | 0xb01e => 0x84, 0xaf2c => 0x88,
                    0xaf32 => 0x86, 0xaf62 => 0xa0, 0xb094 => 0x8c, 0xb09a => 0xe7, _ => 0x8a };
                let v = w.dp_read(at, m); w.load(v, m); pc + 2
            }
            0xaf38 | 0xaf3f | 0xaf46 => {
                w.drain(16); let at = match pc { 0xaf38 => 0x418, 0xaf3f => 0x410, _ => 0x416 };
                let v = w.read(w.absolute(at), m); w.load(v, m); pc + 3
            }
            0xaf6c | 0xafb2 | 0xafee | 0xaffc | 0xb044 | 0xb074 |
            0xb0a2 | 0xb0a8 | 0xb0ae | 0xb0b4 | 0xb0ba | 0xb0c1 | 0xb0c8 => {
                w.drain(24);
                let at = match pc { 0xaf6c | 0xafb2 => 0x7ef280 + u32::from(w.cpu.x),
                    0xafee => 0x7ef2f0, 0xaffc => 0x7ef3c5, 0xb044 => 0x7f5b00 + u32::from(w.cpu.x),
                    0xb074 | 0xb0a2 => 0x7ec213, 0xb0a8 => 0x7ec215, 0xb0ae => 0x7ec217,
                    0xb0b4 => 0x7ec219, 0xb0ba => 0x7ec21b, 0xb0c1 => 0x7ec21d, _ => 0x7ec21f };
                let v = w.read(at, m); w.load(v, m); pc + 4
            }
            0xaf22 | 0xaf28 | 0xaf2e | 0xaf34 | 0xaf3b | 0xaf42 | 0xaf49 => {
                w.drain(24); let at = match pc { 0xaf22 => 0xc213, 0xaf28 => 0xc215,
                    0xaf2e => 0xc217, 0xaf34 => 0xc219, 0xaf3b => 0xc21b, 0xaf42 => 0xc21d, _ => 0xc21f };
                w.write(0x7e0000 + at, a, m); pc + 4
            }
            0xaf1b | 0xaf99 | 0xb04c | 0xb0be | 0xb0c5 | 0xb0cc => {
                w.drain(16); let at = match pc { 0xb0be => 0x418, 0xb0c5 => 0x410, 0xb0cc => 0x416, _ => 0x12d };
                w.write(w.absolute(at), a, m); pc + 3
            }
            0xb01c | 0xb028 | 0xb037 | 0xb03b | 0xb03f | 0xb08f | 0xb09e | 0xb0a6 | 0xb0ac | 0xb0b2 | 0xb0b8 => {
                let at = match pc { 0xb01c | 0xb0b2 => 0x88, 0xb028 | 0xb0b8 => 0x86,
                    0xb037 => 0x99, 0xb03b => 0x1c, 0xb03f => 0x1d, 0xb08f => 0x9a,
                    0xb09e => 0xe7, 0xb0a6 => 0x8a, _ => 0x84 };
                w.dp_store(at, a, m); pc + 2
            }
            0xaf4d | 0xaf9e | 0xb08d => { w.dp_store(if pc == 0xaf4d { 0x8c } else { 0x1d }, 0, m); pc + 2 }
            0xaf4f | 0xaf52 | 0xb02a | 0xb02d | 0xb030 => {
                w.drain(16); let at = match pc { 0xaf4f => 0x622, 0xaf52 => 0x620,
                    0xb02a => 0x418, 0xb02d => 0x410, _ => 0x416 };
                w.write(w.absolute(at), 0, m); pc + 3
            }
            0xaf55 => { w.drain(8 * u32::from(xw)); w.cpu.y = 0x390; w.flags(w.cpu.y, xw); pc + 3 }
            0xaf5f | 0xaf69 | 0xaf70 | 0xaf7b | 0xaf83 | 0xafaf | 0xafb6 | 0xafbe | 0xafc3 | 0xafd7 | 0xaff9 | 0xb008 => {
                w.drain(8 * u32::from(xw)); w.cpu.x = match pc { 0xaf5f | 0xaf70 => 0x97,
                    0xaf69 | 0xafaf => 0x80, 0xaf7b => 0x94, 0xaf83 => 0x93, 0xafb6 => 0x9e,
                    0xafbe => 0x9d, 0xafc3 => 0x95, 0xafd7 => 0x9c, 0xaff9 => 0x96, _ => 0x9f };
                w.flags(w.cpu.x, xw); pc + 1 + u16::from(xw)
            }
            0xb00b | 0xb00d | 0xb00f => {
                w.dp_store(match pc { 0xb00b => 0x84, 0xb00d => 0x8a, _ => 0x8c },
                    if pc == 0xb00b { w.cpu.y } else { w.cpu.x }, xw); pc + 2
            }
            0xb042 | 0xb083 => { w.cpu.x = w.dp_read(if pc == 0xb042 { 0x8a } else { 0x11 }, xw); w.flags(w.cpu.x, xw); pc + 2 }
            0xaf5a | 0xaf64 | 0xaf7e | 0xaf86 | 0xaf8b | 0xaf90 | 0xafc8 | 0xafcd | 0xafd2 | 0xafda | 0xafdf | 0xafe4 | 0xafe9 | 0xb003 | 0xb096 |
            0xb052 | 0xb056 | 0xb05a | 0xb05e | 0xb062 | 0xb066 | 0xb06c | 0xb070 | 0xb07b | 0xb07f | 0xb085 | 0xb089 => {
                let ix = matches!(pc, 0xb052 | 0xb056 | 0xb05a | 0xb05e | 0xb062 | 0xb066 | 0xb06c | 0xb070 | 0xb07b | 0xb07f | 0xb085 | 0xb089);
                let width = if ix { xw } else { m }; w.drain(8 * u32::from(width));
                let rhs = match pc { 0xaf5a => 0x80, 0xaf64 => 0x180, 0xaf7e => 0x181, 0xaf86 => 0x189,
                    0xaf8b => 0x182, 0xaf90 => 0x183, 0xafc8 => 3, 0xafcd => 5, 0xafd2 => 7,
                    0xafda => 0x43, 0xafdf => 0x45, 0xafe4 => 0x47, 0xafe9 => 0x70, 0xb003 => 2,
                    0xb052 => 0x97, 0xb056 | 0xb096 => 0x94, 0xb05a => 0x93, 0xb05e => 0x9d,
                    0xb062 => 0x9e, 0xb066 => 0x9f, 0xb06c => 0x95, 0xb070 => 0x9c,
                    0xb07b => 0x5b, 0xb07f => 0x1b, 0xb085 => 0x23, _ => 0x2c };
                let lhs = if ix { w.cpu.x } else { w.cpu.a & if width == 1 { 255 } else { 65535 } };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(lhs >= rhs);
                w.flags(lhs.wrapping_sub(rhs) & if width == 1 { 255 } else { 65535 }, width);
                pc + 1 + u16::from(width)
            }
            0xaf73 | 0xafa3 | 0xafaa | 0xafb9 | 0xaff2 | 0xb000 | 0xb017 | 0xb024 | 0xb09c => {
                w.drain(8 * u32::from(m));
                let value = match pc { 0xafa3 => w.cpu.a & 0x3f, 0xb000 => w.cpu.a & 255,
                    0xb017 => w.cpu.a & 0xf80, 0xb024 => w.cpu.a & 0x3e,
                    0xb09c => w.cpu.a | 1, 0xaff2 => w.cpu.a & 0x20, _ => w.cpu.a & 0x40 };
                w.load(value, m); pc + 1 + u16::from(m)
            }
            0xaf5d | 0xaf67 | 0xaf76 | 0xaf81 | 0xaf89 | 0xaf8e | 0xaf93 | 0xafa6 | 0xafad | 0xafbc | 0xafcb | 0xafd0 | 0xafd5 | 0xafdd | 0xafe2 | 0xafe7 | 0xafec | 0xaff5 | 0xb006 |
            0xb054 | 0xb058 | 0xb05c | 0xb060 | 0xb064 | 0xb068 | 0xb06e | 0xb072 | 0xb07d | 0xb081 | 0xb087 | 0xb08b | 0xb098 => {
                let carry = matches!(pc, 0xaf5d | 0xb006);
                let nz = matches!(pc, 0xaf67 | 0xaf76 | 0xaf93 | 0xafa6 | 0xafad | 0xafbc | 0xafec | 0xaff5 | 0xb081 | 0xb098);
                let taken = if carry { (w.cpu.p & CARRY != 0) == (pc == 0xb006) }
                    else { (w.cpu.p & ZERO == 0) == nz };
                let target = match pc { 0xaf5d => 0xafa3, 0xaf67 => 0xaf7b, 0xaf76 | 0xaf93 => 0xaf9c,
                    0xaf81 | 0xaf89 => 0xaf78, 0xaf8e => 0xaf95, 0xafa6 => 0xafc3, 0xafad => 0xafbe,
                    0xafec => 0xaff9, 0xb081 => 0xb08d, 0xb098 => 0xb0a0,
                    0xb054 | 0xb058 | 0xb05c | 0xb060 | 0xb064 | 0xb068 | 0xb06e | 0xb072 | 0xb07d | 0xb087 | 0xb08b => 0xb08f,
                    _ => 0xb00b };
                if w.branch(taken) { target } else { pc + 2 }
            }
            0xaf78 => { w.drain(16); 0xb00b }
            0xafc1 | 0xaff7 => { w.branch(true); if pc == 0xafc1 { 0xb00b } else { 0xb008 } }
            0xb013 | 0xb020 => { w.drain(6); w.cpu.p |= CARRY; pc + 1 }
            0xb014 | 0xb021 => { w.drain(8 * u32::from(m)); w.sbc(if pc == 0xb014 { 0x400 } else { 0x10 }, m); pc + 3 }
            0xb01a | 0xb027 | 0xb048..=0xb04b => {
                w.drain(6); let mask = if m == 1 { 255 } else { 65535 };
                let value = w.cpu.a & mask;
                let left = pc == 0xb01a;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(if left { value & (if m == 1 { 0x80 } else { 0x8000 }) != 0 } else { value & 1 != 0 });
                w.load(if left { value.wrapping_mul(2) & mask } else { value >> 1 }, m); pc + 1
            }
            0xb01b => { w.drain(12); w.cpu.a = w.cpu.a.rotate_left(8); w.flags(w.cpu.a & 255, 1); pc + 1 }
            0xb041 => { w.drain(6); w.push(w.cpu.x, xw); pc + 1 }
            0xb04f => { w.drain(12); w.cpu.x = w.pull(xw); w.flags(w.cpu.x, xw); pc + 1 }
            0xb078 => { w.drain(6); w.cpu.x = w.cpu.a & if xw == 1 { 255 } else { 65535 }; w.flags(w.cpu.x, xw); pc + 1 }
            0xafa0 => { w.dp_increment(0x11, m, true); pc + 2 }
            0xb091 => {
                self.child_return_sp = w.cpu.sp;
                w.operand_then_internal(2); w.push(pc + 2, 2);
                self.child = true; 0xfd0d
            }
            0xafa2 | 0xb0d1 => { w.rts(); self.complete = true; w.cpu.pc }
            _ => unreachable!("overlay caller PC {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::native_work::compare_instruction;
    #[test]
    fn overlay_caller_instructions_and_map_loader_frame_match_source() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let pcs = [0xaf19, 0xaf1b, 0xaf1e, 0xaf20, 0xaf22, 0xaf26, 0xaf28, 0xaf2c, 0xaf2e, 0xaf32, 0xaf34, 0xaf38, 0xaf3b, 0xaf3f, 0xaf42, 0xaf46, 0xaf49, 0xaf4d, 0xaf4f, 0xaf52, 0xaf55, 0xaf58, 0xaf5a, 0xaf5d, 0xaf5f, 0xaf62, 0xaf64, 0xaf67, 0xaf69, 0xaf6c, 0xaf70, 0xaf73, 0xaf76, 0xaf78, 0xaf7b, 0xaf7e, 0xaf81, 0xaf83, 0xaf86, 0xaf89, 0xaf8b, 0xaf8e, 0xaf90, 0xaf93, 0xaf95, 0xaf97, 0xaf99, 0xaf9c, 0xaf9e, 0xafa0, 0xafa2, 0xafa3, 0xafa6, 0xafa8, 0xafaa, 0xafad, 0xafaf, 0xafb2, 0xafb6, 0xafb9, 0xafbc, 0xafbe, 0xafc1, 0xafc3, 0xafc6, 0xafc8, 0xafcb, 0xafcd, 0xafd0, 0xafd2, 0xafd5, 0xafd7, 0xafda, 0xafdd, 0xafdf, 0xafe2, 0xafe4, 0xafe7, 0xafe9, 0xafec, 0xafee, 0xaff2, 0xaff5, 0xaff7, 0xaff9, 0xaffc, 0xb000, 0xb003, 0xb006, 0xb008, 0xb00b, 0xb00d, 0xb00f, 0xb011, 0xb013, 0xb014, 0xb017, 0xb01a, 0xb01b, 0xb01c, 0xb01e, 0xb020, 0xb021, 0xb024, 0xb027, 0xb028, 0xb02a, 0xb02d, 0xb030, 0xb033, 0xb035, 0xb037, 0xb039, 0xb03b, 0xb03d, 0xb03f, 0xb041, 0xb042, 0xb044, 0xb048, 0xb049, 0xb04a, 0xb04b, 0xb04c, 0xb04f, 0xb050, 0xb052, 0xb054, 0xb056, 0xb058, 0xb05a, 0xb05c, 0xb05e, 0xb060, 0xb062, 0xb064, 0xb066, 0xb068, 0xb06a, 0xb06c, 0xb06e, 0xb070, 0xb072, 0xb074, 0xb078, 0xb079, 0xb07b, 0xb07d, 0xb07f, 0xb081, 0xb083, 0xb085, 0xb087, 0xb089, 0xb08b, 0xb08d, 0xb08f, 0xb091, 0xb094, 0xb096, 0xb098, 0xb09a, 0xb09c, 0xb09e, 0xb0a0, 0xb0a2, 0xb0a6, 0xb0a8, 0xb0ac, 0xb0ae, 0xb0b2, 0xb0b4, 0xb0b8, 0xb0ba, 0xb0be, 0xb0c1, 0xb0c5, 0xb0c8, 0xb0cc, 0xb0cf, 0xb0d1];
        let mut checked = 0;
        for pc in pcs {
            for case in 0..24usize {
                for position in [snes::CpuRasterPosition::new(100, 532),
                    snes::CpuRasterPosition::new(224, 1350), snes::CpuRasterPosition::new(261, 1350)] {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    for (i, v) in machine.ram.iter_mut().enumerate() {
                        *v = (i.wrapping_mul(37) ^ (i >> 8) ^ case.wrapping_mul(19)) as u8;
                    }
                    machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
                    let m8 = pc < 0xaf20 || (0xaf97..=0xafa2).contains(&pc)
                        || (0xb035..=0xb0a0).contains(&pc)
                        || (pc == 0xaf9c && case & 1 == 0);
                    let x8 = pc < 0xaf20 || (0xaf97..=0xafa2).contains(&pc) || pc >= 0xb035;
                    let mut cpu = NativeNmiContext { a: [0xb701, 0xff00, 0xffff, 0x390, 0x180, 0x70][case % 6],
                        x: [0, 1, 0x95, 255][case % 4], y: [0, 0x390, 255][case % 3],
                        d: 0, db: [0, 2, 0x80, 0x82][case % 4], bank: 2, pc, sp: 0x1fa,
                        p: u8::from(m8) * 0x20 | u8::from(x8) * 0x10 | [0, 1, 2, 0x43, 0x80, 0xc3][case % 6] };
                    if x8 { cpu.y &= 255; }
                    machine.cpu.a=cpu.a; machine.cpu.x=cpu.x; machine.cpu.y=cpu.y;
                    machine.cpu.dp=cpu.d; machine.cpu.db=cpu.db; machine.cpu.k=cpu.bank;
                    machine.cpu.pc=cpu.pc; machine.cpu.sp=cpu.sp; machine.cpu.e=false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let clock = CpuMasterTimeline::at_raster(case as u64, position,
                        snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                    let mut source = snes::RomCpuTimingProbe::new(machine, clock,
                        snes::SourcePpuReadState::snes9x_reset()).unwrap();
                    let mut clock = source.timeline().clone();
                    let mut work = OverworldOverlays::new(0xaf19); work.pc = pc;
                    compare_instruction(&format!("overlay caller pc={pc:04x} case={case}"),
                        &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                        |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                    assert_eq!(ram, source.snes().ram);
                    if pc == 0xb091 {
                        assert!(work.map_loader_pending());
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xfd0d, 0x1f8));
                        assert_eq!(&ram[0x1f9..0x1fb], &[0x93, 0xb0]);
                        assert_eq!(work.child_return_sp, 0x1fa);
                    }
                    checked += 1;
                }
            }
        }
        eprintln!("overlay caller original instruction boundaries={checked}");
    }
    #[test]
    fn overlay_selection_paths_retain_source_frames_until_the_map_loader() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let mut checked = 0;
        let mut skipped = 0;
        let mut handed_off = 0;
        let mut selections = std::collections::BTreeSet::new();
        for screen in 0..160u16 {
            for variant in 0..12usize {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                for (i, v) in machine.ram.iter_mut().enumerate() {
                    *v = (i.wrapping_mul(37) ^ (i >> 8) ^ variant.wrapping_mul(19)) as u8;
                }
                machine.ram[0x8a..0x8c].copy_from_slice(&screen.to_le_bytes());
                machine.ram[0xa0..0xa2].copy_from_slice(&[0x180u16, 0x181, 0x189, 0x182, 0x183, 0x18a][variant % 6].to_le_bytes());
                machine.ram[0xf300] = if variant < 6 { 0 } else { 0x40 };
                machine.ram[0xf2f0] = if variant & 1 == 0 { 0 } else { 0x20 };
                machine.ram[0xf3c5] = (variant % 3) as u8;
                machine.ram[0x11] = [1, 35, 44][variant % 3];
                machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
                let mut cpu = NativeNmiContext { a: 0xb701, x: 0x81, y: 0x42,
                    d: 0, db: [0, 2, 0x80, 0x82][variant % 4], bank: 2,
                    pc: if variant & 1 == 0 { 0xaf19 } else { 0xaf1e }, sp: 0x1fa,
                    p: 0x30 | [0, 1, 2, 0x43, 0x80, 0xc3][variant % 6] };
                machine.cpu.a=cpu.a; machine.cpu.x=cpu.x; machine.cpu.y=cpu.y;
                machine.cpu.dp=cpu.d; machine.cpu.db=cpu.db; machine.cpu.k=cpu.bank;
                machine.cpu.pc=cpu.pc; machine.cpu.sp=cpu.sp; machine.cpu.e=false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let clock = CpuMasterTimeline::at_raster(variant as u64,
                    [snes::CpuRasterPosition::new(100, 532), snes::CpuRasterPosition::new(224, 1350),
                     snes::CpuRasterPosition::new(261, 1350)][variant % 3],
                    snes::CpuBusWorkload::default(), snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                let mut source = snes::RomCpuTimingProbe::new(machine, clock,
                    snes::SourcePpuReadState::snes9x_reset()).unwrap();
                let mut clock = source.timeline().clone();
                let mut work = OverworldOverlays::new(cpu.pc);
                for _ in 0..256 {
                    if work.complete() || work.map_loader_pending() { break; }
                    compare_instruction(&format!("overlay selection screen={screen:02x} variant={variant} pc={:04x}",cpu.pc),
                        &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                        |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                    assert_eq!(ram, source.snes().ram);
                    checked += 1;
                }
                assert!(work.complete() || work.map_loader_pending());
                if work.complete() {
                    assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,0x83c6,0x1fc));
                    skipped += 1;
                } else {
                    assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,0xfd0d,0x1f8));
                    assert_eq!(&ram[0x1f9..0x1fb], &[0x93,0xb0]);
                    selections.insert(crate::types::read_le_u16(&ram,0x8c));
                    handed_off += 1;
                }
            }
        }
        assert_eq!(selections, [0x93,0x94,0x95,0x96,0x97,0x9c,0x9d,0x9e,0x9f].into_iter().collect());
        eprintln!("overlay selection Source cases=1920 instruction boundaries={checked} skipped={skipped} map_loader_handoffs={handed_off}");
    }

}
