//! Full tileset initializer with retained live selectors and nested graphics
//! calls. Selection tables are read by the caller, not copied into its state.
use super::{
    common_sprite_graphics::CommonSpriteGraphics,
    native_tileset_sheet::{NativeTilesetSheet, SheetEntry},
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Entry(u8),
    SpriteSelect { slot: u8, op: u8 },
    Sprites(u8),
    Theme(u8),
    MainSelect(u8),
    AuxSelect { slot: u8, op: u8 },
    Backgrounds(u8),
    Common,
    Sheet,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeTilesets {
    phase: Phase,
    return_phase: Option<Phase>,
    common: Option<CommonSpriteGraphics>,
    sheet: Option<NativeTilesetSheet>,
}
impl Default for NativeTilesets {
    fn default() -> Self {
        Self {
            phase: Phase::Entry(0),
            return_phase: None,
            common: None,
            sheet: None,
        }
    }
}
impl NativeTilesets {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::Common => self.common.as_ref().unwrap().routine(),
            Phase::Sheet => self.sheet.as_ref().unwrap().routine(),
            _ => 0xe19b,
        }
    }
    fn pc(phase: Phase) -> u16 {
        use Phase::*;
        match phase {
            Entry(i) => [
                0xe19b, 0xe19c, 0xe19d, 0xe19e, 0xe1a0, 0xe1a3, 0xe1a6, 0xe1a8, 0xe1ab, 0xe1ae,
                0xe1b0, 0xe1b3, 0xe1b6, 0xe1b7, 0xe1b8, 0xe1b9,
            ][usize::from(i)],
            SpriteSelect { slot, op } => {
                0xe1bb + u16::from(slot) * 15 + [0, 3, 5, 9, 13][usize::from(op)]
            }
            Sprites(i) => [
                0xe1f7, 0xe1f9, 0xe1fb, 0xe1fd, 0xe1ff, 0xe201, 0xe204, 0xe206, 0xe208, 0xe20b,
                0xe20d, 0xe20f, 0xe212, 0xe214, 0xe216,
            ][usize::from(i)],
            Theme(i) => [
                0xe219, 0xe21b, 0xe21e, 0xe221, 0xe224, 0xe227, 0xe228, 0xe229, 0xe22a, 0xe22b,
                0xe22e, 0xe231, 0xe232, 0xe233, 0xe234,
            ][usize::from(i)],
            MainSelect(i) => 0xe236 + u16::from(i / 2) * 5 + if i % 2 == 0 { 0 } else { 3 },
            AuxSelect { slot, op } => {
                0xe245 + u16::from(slot) * 14 + [0, 3, 5, 8, 12][usize::from(op)]
            }
            Backgrounds(i) => [
                0xe27d, 0xe280, 0xe282, 0xe284, 0xe286, 0xe288, 0xe28a, 0xe28d, 0xe28f, 0xe291,
                0xe294, 0xe296, 0xe298, 0xe29b, 0xe29d, 0xe29f, 0xe2a1, 0xe2a3, 0xe2a6, 0xe2a8,
                0xe2aa, 0xe2ac, 0xe2ae, 0xe2b1, 0xe2b3, 0xe2b5, 0xe2b7, 0xe2b9, 0xe2bc, 0xe2be,
                0xe2c0, 0xe2c2, 0xe2c4, 0xe2c7, 0xe2c9, 0xe2cb, 0xe2ce, 0xe2cf,
            ][usize::from(i)],
            _ => unreachable!(),
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        use Phase::*;
        match self.phase {
            Complete => return None,
            Common => {
                let child = self.common.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.common = None;
                    self.phase = self.return_phase.take().unwrap();
                    assert_eq!(cpu.pc, Self::pc(self.phase));
                }
                return cycles;
            }
            Sheet => {
                let child = self.sheet.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.sheet = None;
                    self.phase = self.return_phase.take().unwrap();
                    assert_eq!(cpu.pc, Self::pc(self.phase));
                }
                return cycles;
            }
            _ => {}
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (0, Self::pc(self.phase), 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut sheet_call = None;
        let next = match self.phase {
            Entry(i) => {
                match i {
                    0 | 1 => {
                        w.drain(6);
                        w.push(u16::from(if i == 0 { w.cpu.db } else { w.cpu.bank }), 1);
                    }
                    2 => {
                        w.drain(12);
                        w.cpu.db = w.pull(1) as u8;
                        w.flags(u16::from(w.cpu.db), 1);
                    }
                    3 | 6 => w.immediate(if i == 3 { 0x80 } else { 0x44 }, 1),
                    4 | 5 | 7 => {
                        w.drain(16);
                        w.write_io(
                            match i {
                                4 => 0x2115,
                                5 => 0x2116,
                                _ => 0x2117,
                            },
                            if i == 5 { 0 } else { w.cpu.a & 255 },
                            1,
                        );
                    }
                    8 => {
                        w.item_call();
                        self.common = Some(CommonSpriteGraphics::default());
                        w.cpu.pc = 0xe6b7;
                        self.return_phase = Some(Entry(9));
                        self.phase = Common;
                        return Some(w.cycles);
                    }
                    9 => w.width(0x30, true),
                    10 => {
                        w.drain(16);
                        let value = w.read(0xaa3, 2);
                        w.load(value, 2);
                    }
                    11 => w.mask(255, 2),
                    12 | 13 => w.shift_left(2),
                    14 => {
                        w.drain(6);
                        w.cpu.y = w.cpu.a;
                        w.flags(w.cpu.y, 2);
                    }
                    15 => w.width(0x20, false),
                    _ => unreachable!(),
                }
                if i == 15 {
                    SpriteSelect { slot: 0, op: 0 }
                } else {
                    Entry(i + 1)
                }
            }
            SpriteSelect { slot, op } => {
                let mut next = op + 1;
                match op {
                    0 => indexed_load(&mut w, 0xdb57 + u16::from(slot), true),
                    1 => {
                        if w.branch(w.cpu.p & ZERO != 0) {
                            next = 3;
                        }
                    }
                    2 => {
                        w.drain(24);
                        w.write(0x7ec2fc + u32::from(slot), w.cpu.a & 255, 1);
                    }
                    3 => {
                        w.drain(24);
                        let value = w.read(0x7ec2fc + u32::from(slot), 1);
                        w.load(value, 1);
                    }
                    4 => w.dp_store(9 - u16::from(slot), w.cpu.a & 255, 1),
                    _ => unreachable!(),
                }
                if op == 4 {
                    if slot == 3 {
                        Sprites(0)
                    } else {
                        SpriteSelect {
                            slot: slot + 1,
                            op: 0,
                        }
                    }
                } else {
                    SpriteSelect { slot, op: next }
                }
            }
            Sprites(i) => {
                match i {
                    0 => w.width(0x10, false),
                    1 | 6 | 9 | 12 => {
                        let slot = if i == 1 { 0 } else { (i - 3) / 3 };
                        load_y(&mut w, 9 - u16::from(slot));
                    }
                    2 => w.immediate(0x7e, 1),
                    3 => w.dp_store(2, w.cpu.a & 255, 1),
                    4 | 7 | 10 | 13 => w.item_x(0x78 + u16::from((i - 4) / 3) * 6, 1),
                    5 | 8 | 11 | 14 => sheet_call = Some(SheetEntry::Sprite),
                    _ => unreachable!(),
                }
                if i == 14 {
                    Theme(0)
                } else {
                    Sprites(i + 1)
                }
            }
            Theme(i) => {
                match i {
                    0 => w.width(0x30, true),
                    1 => w.immediate(0x2000, 2),
                    2 => {
                        w.drain(16);
                        w.write_io(0x2116, w.cpu.a, 2);
                    }
                    3 | 9 => {
                        w.drain(16);
                        let v = w.read(if i == 3 { 0xaa1 } else { 0xaa2 }, 2);
                        w.load(v, 2);
                    }
                    4 | 10 => w.mask(255, 2),
                    5 | 6 | 7 | 11 | 12 => w.shift_left(2),
                    8 => {
                        w.drain(6);
                        w.cpu.y = w.cpu.a;
                        w.flags(w.cpu.y, 2);
                    }
                    13 => w.tax_word(),
                    14 => w.width(0x20, false),
                    _ => unreachable!(),
                }
                if i == 14 {
                    MainSelect(0)
                } else {
                    Theme(i + 1)
                }
            }
            MainSelect(i) => {
                if i % 2 == 0 {
                    indexed_load(&mut w, 0xe073 + u16::from(i / 2), true);
                } else {
                    w.dp_store(13 - u16::from(i / 2), w.cpu.a & 255, 1);
                }
                if i == 5 {
                    AuxSelect { slot: 0, op: 0 }
                } else {
                    MainSelect(i + 1)
                }
            }
            AuxSelect { slot, op } => {
                let mut next = op + 1;
                match op {
                    0 => indexed_load(&mut w, 0xdd97 + u16::from(slot), false),
                    1 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = 3;
                        }
                    }
                    2 => indexed_load(&mut w, 0xe076 + u16::from(slot), true),
                    3 => {
                        w.drain(24);
                        w.write(0x7ec2f8 + u32::from(slot), w.cpu.a & 255, 1);
                    }
                    4 => w.dp_store(10 - u16::from(slot), w.cpu.a & 255, 1),
                    _ => unreachable!(),
                }
                if op == 4 {
                    if slot == 3 {
                        Backgrounds(0)
                    } else {
                        AuxSelect {
                            slot: slot + 1,
                            op: 0,
                        }
                    }
                } else {
                    AuxSelect { slot, op: next }
                }
            }
            Backgrounds(i) => {
                match i {
                    0 => indexed_load(&mut w, 0xe07a, true),
                    1 => w.dp_store(6, w.cpu.a & 255, 1),
                    2 => w.width(0x10, false),
                    3 => w.immediate(7, 1),
                    4 => w.dp_store(15, w.cpu.a & 255, 1),
                    5 | 8 | 11 => load_y(&mut w, 13 - u16::from((i - 5) / 3)),
                    6 | 9 | 12 | 35 => sheet_call = Some(SheetEntry::Background),
                    7 | 10 | 13 | 18 | 23 | 28 | 33 => w.dp_increment(15, 1, false),
                    14 | 19 | 24 | 29 => load_y(&mut w, 10 - u16::from((i - 14) / 5)),
                    15 | 20 | 25 | 30 => w.immediate(0x7e, 1),
                    16 | 21 | 26 | 31 => w.item_x(0x60 + u16::from((i - 16) / 5) * 6, 1),
                    17 | 22 | 27 | 32 => sheet_call = Some(SheetEntry::BackgroundDestination),
                    34 => load_y(&mut w, 6),
                    36 => {
                        w.drain(12);
                        w.cpu.db = w.pull(1) as u8;
                        w.flags(u16::from(w.cpu.db), 1);
                    }
                    37 => {
                        w.rtl();
                        self.phase = Complete;
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Backgrounds(i + 1)
            }
            _ => unreachable!(),
        };
        if let Some(entry) = sheet_call {
            let child = NativeTilesetSheet::new(entry);
            w.item_call();
            w.cpu.pc = child.routine() as u16;
            self.sheet = Some(child);
            self.return_phase = Some(next);
            self.phase = Sheet;
        } else {
            self.phase = next;
            w.cpu.pc = Self::pc(next);
        }
        Some(w.cycles)
    }
}
fn indexed_load<B: WorkBus>(w: &mut Work<'_, B>, base: u16, y: bool) {
    let index = if y { w.cpu.y } else { w.cpu.x };
    w.drain(16);
    if w.cpu.p & 0x10 == 0 || (base & 255) + index > 255 {
        w.drain(6);
    }
    let value = w.read(w.absolute(base) + u32::from(index), 1);
    w.load(value, 1);
}
fn load_y<B: WorkBus>(w: &mut Work<'_, B>, address: u16) {
    w.cpu.y = w.dp_read(address, 1);
    w.flags(w.cpu.y, 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn retained_tilesets_match_live_selections_nested_returns_and_ppu_publications() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for theme in 0..37 {
            for (module, aux, sprites, db, odd, v, h) in [
                (0, 0, 0, 12, false, 224, 1338),
                (1, 5, 11, 0, true, 261, 1350),
            ] {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 7)) as u8;
                }
                machine.ram[0x10] = module;
                machine.ram[0xaa1..0xaa5].copy_from_slice(&[theme, aux, sprites, theme % 12]);
                machine.ram[0xc2fc..0xc300].copy_from_slice(&[12, 82, 90, 95]);
                machine.ram[0x1fd..0x200].copy_from_slice(&[0x21, 0xc2, 12]);
                let mut work = NativeTilesets::default();
                let mut native = NativeNmiContext {
                    a: 0xab44,
                    x: 0x78,
                    y: 0xff,
                    d: 0,
                    db,
                    bank: 0,
                    pc: 0xe19b,
                    sp: 0x1fc,
                    p: 0xb0,
                };
                machine.cpu.a = native.a;
                machine.cpu.x = native.x;
                machine.cpu.y = native.y;
                machine.cpu.pc = native.pc;
                machine.cpu.db = db;
                machine.cpu.sp = native.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(native.p);
                let mut ram = machine.ram.clone();
                let mut ppu = machine.ppu.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(odd),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut count = 0;
                while !work.complete() {
                    let label=format!("tilesets theme={theme} module={module} aux={aux} sprites={sprites} phase={:?} pc={:04x}",work.phase,native.pc);
                    super::super::native_work::compare_instruction_with_ppu(
                        &label,
                        &mut native,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        Some(&mut ppu),
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    count += 1;
                    assert!(count < 1000000);
                }
                assert_eq!(
                    (native.bank, native.pc, native.sp, native.db),
                    (12, 0xc222, 0x1ff, db)
                );
                assert_eq!(ram, source.snes().ram);
                assert_eq!(ppu.vram, source.snes().ppu.vram);
            }
        }
    }
}
