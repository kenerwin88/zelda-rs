//! Module10's post-table caller through LinkOam and the real main-wait return.
//! The configure-table RTL has already consumed its own far frame. The module
//! dispatch's near frame and ZeldaRunGameLoop's far frame stay on the live bus.
use super::{
    link_oam::LinkOam,
    player_animation::PlayerAnimation,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
    oam_pack::CallerPacking,
    spotlight::Spotlight,
    sprite_dma::{MainSpriteTail, SourceWords},
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    if (0x02997a..=0x029981).contains(&address) {
        return Some([8, 4, 2, 1, 202, 153, 25, 154][(address - 0x02997a) as usize]);
    }
    if (0x029ad3..=0x029ad6).contains(&address) {
        return Some([0xe6, 0x9a, 0x19, 0x9a][(address - 0x029ad3) as usize]);
    }
    super::link_oam::table_byte(address)
        .or_else(||super::overworld_sprite_proximity::table_byte(address))
        .or_else(|| super::sprite_main::table_byte(address))
        .or_else(|| crate::zelda_rtl::ZeldaState::native_follower_table_byte(address))
        .or_else(|| super::player_animation::table_byte(address))
        .or_else(|| crate::zelda_rtl::link_dma_data_byte(address))
        .or_else(|| crate::zelda_rtl::ZeldaState::native_spotlight_table_byte(address))
        .or_else(|| crate::zelda_rtl::ZeldaState::native_hdma_descriptor_byte(address))
}

/// Module0F keeps its own dispatch frame around the shared table builder.
/// Initialization and outdoor movement remain real child calls until their
/// owners return; the indoor closing path reaches the physical main wait.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ClosingSpotlight {
    entry_graphics_field: Option<u64>,
    sprites: Option<super::dungeon_sprite_prefix::DungeonSpritePrefix>,
    prefix: Option<u32>,
    initialization: Option<ClosingInitialization>,
    table: Option<Spotlight>,
    caller: SpotlightCallerReturn,
    child: Option<u32>,
}
impl ClosingSpotlight {
    pub(crate) fn entry_graphics_field(&self) -> Option<u64> { self.entry_graphics_field }
    pub(crate) fn sprite_preparation_master_cycles(&self) -> u64 {
        self.caller.sprite_preparation_master_cycles()
    }
    pub(crate) fn after_sprite_return() -> Self {
        Self { entry_graphics_field: None, sprites: None, prefix: Some(0x029986), initialization: None, table: None,
            caller: SpotlightCallerReturn::closing(), child: None }
    }
    pub(crate) fn from_module_entry() -> Self {
        Self { sprites: Some(super::dungeon_sprite_prefix::DungeonSpritePrefix::closing()),
            ..Self::after_sprite_return() }
    }
    pub(crate) fn sprite_tail(&self) -> Option<super::dungeon_sprite_prefix::SpritePrefixTail> {
        self.sprites.as_ref().and_then(|work| work.tail())
    }
    pub(crate) fn child(&self) -> Option<u32> {
        self.initialization.as_ref().and_then(|work| work.child).or(self.child).or(self.caller.child)
    }
    pub(crate) fn complete(&self) -> bool { self.caller.complete() }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        if let Some(child) = self.child.take() {
            assert_eq!(child, 0x0299ca);
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x998d, 0x1fc));
            self.prefix = None;
            self.caller.pc = 0x02998d;
        } else {
            assert_eq!(self.caller.child.take(), Some(0x07e245));
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x99a4, 0x1fc));
            self.caller.pc = 0x0299a4;
        }
    }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<(u64, u32)> {
        if self.complete() { return None; }
        if let Some(sprites) = &mut self.sprites {
            assert!(sprites.tail().is_none(), "closing sprite child {:?} retains its call frame at {cpu:?}", sprites.tail());
            let routine = sprites.routine();
            let cycles = sprites.step(cpu, clock, bus).unwrap();
            if sprites.tail() == Some(super::dungeon_sprite_prefix::SpritePrefixTail::Returned) {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x9986, 0x1fc));
                self.sprites = None;
            }
            return Some((cycles, routine));
        }
        assert!(self.child().is_none(), "closing child retains its physical call frame");
        if let Some(initialization) = &mut self.initialization {
            let result = initialization.step(cpu, clock, bus);
            if initialization.complete {
                self.entry_graphics_field = Some(clock.field_index());
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x998d, 0x1fc));
                self.initialization = None;
                self.prefix = None;
                self.caller.pc = 0x02998d;
            }
            return result;
        }
        if let Some(table) = &mut self.table {
            let cycles = table.step(cpu, clock, bus).unwrap();
            if table.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x9a1d, 0x1fa));
                self.table = None;
            }
            return Some((cycles, 0x00f312));
        }
        if let Some(pc) = self.prefix {
            assert_eq!((cpu.bank, u32::from(cpu.pc), cpu.sp, cpu.db, cpu.d, cpu.p & 0x38),
                (2, pc & 65535, if pc == 0x029a19 { 0x1fa } else { 0x1fc }, 0, 0, 0x30));
            let mut w = Work { cpu, clock, bus, cycles: 0 };
            w.fetch();
            self.prefix = match pc {
                0x029986 => { let value = w.dp_read(0x11, 1); w.load(value, 1); Some(pc + 2) }
                0x029988 => { w.shift_left(1); Some(pc + 1) }
                0x029989 => { w.drain(6); w.cpu.x = w.cpu.a & 255; w.flags(w.cpu.x, 1); Some(pc + 1) }
                0x02998a => {
                    w.drain(8); w.drain(6); w.drain(8);
                    let target = w.read(0x02997e + u32::from(w.cpu.x), 2);
                    w.push(0x998c, 2);
                    match target {
                        0x9a19 => Some(0x029a19),
                        0x99ca => { self.initialization = Some(ClosingInitialization::default()); Some(0x0299ca) }
                        _ => unreachable!("closing dispatch target {target:04x}"),
                    }
                }
                0x029a19 => {
                    w.operand_then_internal(3); w.push(2, 1); w.push(0x9a1c, 2);
                    w.cpu.bank = 0; w.cpu.pc = 0xf312;
                    self.table = Some(Spotlight::new(0xf312)); None
                }
                _ => unreachable!("closing prefix {pc:06x}"),
            };
            if let Some(next) = self.prefix { w.cpu.pc = next as u16; }
            return Some((w.cycles, 0x029982));
        }
        self.caller.step(cpu, clock, bus)
    }
}

/// Module0F's initialization keeps the near dispatch frame around two
/// existing native children. Outdoor map saving retains its actual far frame.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ClosingInitialization {
    pc: u16,
    floor: Option<super::floor_indicator::FloorIndicator>,
    configure: Option<Spotlight>,
    child: Option<u32>,
    complete: bool,
}
impl Default for ClosingInitialization {
    fn default() -> Self {
        Self { pc: 0x99ca, floor: None, configure: None, child: None, complete: false }
    }
}
impl ClosingInitialization {
    fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline,
        bus: impl WorkBus) -> Option<(u64, u32)> {
        if self.complete { return None; }
        if let Some(floor) = &mut self.floor {
            let cycles = floor.step(cpu, clock, bus).unwrap();
            if floor.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x9a10, 0x1fa));
                self.floor = None;
            }
            return Some((cycles, 0x0afd0c));
        }
        if let Some(configure) = &mut self.configure {
            let cycles = configure.step(cpu, clock, bus).unwrap();
            if configure.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x9a16, 0x1fa));
                self.configure = None;
            }
            return Some((cycles, 0x00f28b));
        }
        assert!(self.child.is_none());
        assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.d), (2, self.pc, 0x1fa, 0, 0));
        let pc = self.pc;
        let mut w = Work { cpu, clock, bus, cycles: 0 };
        w.fetch();
        self.pc = match pc {
            0x99ca | 0x99cd | 0x9a09 => {
                let address = match pc { 0x99ca => 0x12a, 0x99cd => 0x1f0c, _ => 0x4a0 };
                w.drain(16); w.write(address, 0, 1); pc + 3
            }
            0x99d0 => { let value = w.dp_read(0x1b, 1); w.load(value, 1); pc + 2 }
            0x99d2 | 0x99eb | 0x99f3 | 0x99f7 | 0x99fe => {
                let taken = if pc == 0x99f3 { w.cpu.p & CARRY == 0 } else { w.cpu.p & ZERO == 0 };
                let target = match pc { 0x99d2 => 0x99e2, 0x99eb => 0x99f5,
                    0x99f3 => 0x9a09, 0x99f7 => 0x9a04, _ => 0x9a06 };
                if w.branch(taken) { target } else { pc + 2 }
            }
            0x99e2 | 0x99f9 => {
                w.drain(16);
                let value = w.read(if pc == 0x99e2 {0x10e} else {0x130}, 1);
                w.cpu.x = value; w.flags(value, 1); pc + 3
            }
            0x99e5 | 0x99ed => {
                w.drain(24);
                let address = if pc == 0x99e5 { 0x02d82e + u32::from(w.cpu.x) } else { 0x7ef3c5 };
                let value = w.read(address, 1); w.load(value, 1); pc + 4
            }
            0x99e9 | 0x99f1 | 0x99f5 => {
                w.compare_immediate(match pc {0x99e9 => 3, 0x99f1 => 2, _ => 0xf2}, 1); pc + 2
            }
            0x99fc => { w.compare_x_immediate(12, 1); pc + 2 }
            0x9a00 | 0x9a04 => { w.immediate(if pc == 0x9a00 {7} else {0xf1}, 1); pc + 2 }
            0x9a02 => { w.drain(8); w.drain(6); 0x9a06 }
            0x9a06 => { w.drain(16); w.write(0x12c, w.cpu.a & 255, 1); pc + 3 }
            0x99d4 | 0x9a0c | 0x9a12 => {
                w.operand_then_internal(3); w.push(2, 1); w.push(pc + 3, 2);
                let (bank, target) = match pc {
                    0x99d4 => { self.child = Some(0x0ffd3c); (15, 0xfd3c) }
                    0x9a0c => { self.floor = Some(Default::default()); (10, 0xfd0c) }
                    _ => { self.configure = Some(Spotlight::new(0xf28b)); (0, 0xf28b) }
                };
                w.cpu.bank = bank; w.cpu.pc = target;
                self.pc = pc + 4;
                return Some((w.cycles, 0x0299ca));
            }
            0x9a10 | 0x9a16 => { w.dp_increment(if pc == 0x9a10 {0x16} else {0x11}, 1, true); pc + 2 }
            0x9a18 => { w.rts(); self.complete = true; w.cpu.pc }
            _ => unreachable!("closing initialization {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some((w.cycles, 0x0299ca))
    }
}

/// Opening's dispatch, table and physical caller return on one native clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OpeningSpotlight {
    sprites: Option<super::dungeon_sprite_prefix::DungeonSpritePrefix>,
    prefix: Option<u32>,
    table: Option<Spotlight>,
    entry_table: bool,
    caller: SpotlightCallerReturn,
}
impl Default for OpeningSpotlight {
    fn default() -> Self {
        Self {
            sprites: None,
            prefix: Some(0x029a19),
            table: None,
            entry_table: false,
            caller: Default::default(),
        }
    }
}
impl OpeningSpotlight {
    pub(crate) fn from_module_entry()->Self {
        Self {sprites:Some(super::dungeon_sprite_prefix::DungeonSpritePrefix::opening()),..Self::from_sprite_return()}
    }
    pub(crate) fn sprite_preparation_master_cycles(&self)->u64 {
        self.caller.sprite_preparation_master_cycles()
    }
    /// Sprite_Main has returned through the module's actual far call. Only
    /// ZeldaRunGameLoop's far frame remains; dispatch builds the near frame.
    pub(crate) fn from_sprite_return() -> Self {
        Self {
            prefix: Some(0x029adb),
            ..Default::default()
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.caller.complete()
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<(u64, u32)> {
        if let Some(sprites)=&mut self.sprites {
            assert!(sprites.tail().is_none(),"opening sprite child {:?} retains its call frame at {cpu:?}",sprites.tail());
            let routine=sprites.routine();let cycles=sprites.step(cpu,clock,bus).unwrap();
            if sprites.tail()==Some(super::dungeon_sprite_prefix::SpritePrefixTail::Returned) {
                assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,0x9adb,0x1fc));self.sprites=None;
            }
            return Some((cycles,routine));
        }
        if let Some(pc) = self.prefix {
            assert_eq!(
                (cpu.bank, u32::from(cpu.pc), cpu.d, cpu.db, cpu.p & 0x38),
                (2, pc & 65535, 0, 0, 0x30)
            );
            let mut w = Work {
                cpu,
                clock,
                bus,
                cycles: 0,
            };
            w.fetch();
            self.prefix = match pc {
                0x029adb => {
                    let value = w.dp_read(0x11, 1);
                    w.load(value, 1);
                    Some(pc + 2)
                }
                0x029add => {
                    w.shift_left(1);
                    Some(pc + 1)
                }
                0x029ade => {
                    w.drain(6);
                    w.cpu.x = w.cpu.a & 255;
                    w.flags(w.cpu.x, 1);
                    Some(pc + 1)
                }
                0x029adf => {
                    assert_eq!(w.cpu.sp, 0x1fc);
                    w.drain(8);
                    w.drain(6);
                    w.drain(8);
                    let target = w.read(0x029ad3 + u32::from(w.cpu.x), 2);
                    w.push(0x9ae1, 2);
                    Some(0x020000 | u32::from(target))
                }
                0x029a19 | 0x029ae6 => {
                    assert_eq!(w.cpu.sp, 0x1fa);
                    self.entry_table = pc == 0x029ae6;
                    w.operand_then_internal(3);
                    w.push(2, 1);
                    w.push((pc + 3) as u16, 2);
                    w.cpu.bank = 0;
                    w.cpu.pc = if self.entry_table { 0xf295 } else { 0xf312 };
                    self.table = Some(Spotlight::new(w.cpu.pc));
                    None
                }
                0x029aea => {
                    w.dp_increment(0x11, 1, true);
                    Some(pc + 2)
                }
                0x029aec => {
                    w.rts();
                    assert_eq!((w.cpu.bank, w.cpu.pc, w.cpu.sp), (2, 0x9ae2, 0x1fc));
                    self.caller.pc = 0x029ae2;
                    None
                }
                _ => unreachable!("opening caller prefix {pc:06x}"),
            };
            if let Some(next) = self.prefix {
                w.cpu.pc = next as u16;
            }
            return Some((w.cycles, if pc < 0x029ad7 { 0x029a19 } else { 0x029ad7 }));
        }
        if let Some(table) = &mut self.table {
            let cycles = table.step(cpu, clock, bus).unwrap();
            if table.complete() {
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp),
                    (2, if self.entry_table { 0x9aea } else { 0x9a1d }, 0x1fa)
                );
                self.table = None;
                if self.entry_table {
                    self.prefix = Some(0x029aea);
                }
            }
            return Some((cycles, if self.entry_table { 0x00f295 } else { 0x00f312 }));
        }
        self.caller.step(cpu, clock, bus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycle_models::{intro_clear::Access, native_work::compare_instruction_with_devices};
    #[test]
    fn closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let mut instructions = 0;
        for case in 0..3 {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            let r = &mut machine.ram;
            r[0x10] = 15;
            r[0x11] = u8::from(case != 0);
            r[0x8a] = if case == 1 { 15 } else { 0 };
            r[0x1fb..0x200].copy_from_slice(&[0x8c, 0x99, 0x59, 0x80, 0]);
            let mut work = ClosingSpotlight::after_sprite_return();
            if case != 0 { work.prefix = None; }
            let mut cpu = NativeNmiContext {
                a: 0xab01, x: 7, y: 3, d: 0, db: 0, bank: 2,
                pc: if case == 0 { 0x9986 } else { 0x9a1d },
                sp: if case == 0 { 0x1fc } else { 0x1fa }, p: 0x30,
            };
            machine.cpu.a = cpu.a; machine.cpu.x = cpu.x; machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank; machine.cpu.pc = cpu.pc; machine.cpu.sp = cpu.sp;
            machine.cpu.e = false; machine.cpu.unpack_flags(cpu.p);
            let clock = CpuMasterTimeline::at_raster(37582,
                snes::CpuRasterPosition::new(191, 1300), snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN);
            let mut source = snes::RomCpuTimingProbe::new(machine, clock,
                snes::SourcePpuReadState::snes9x_reset()).unwrap();
            let mut clock = source.timeline().clone();
            let mut ram = source.snes().ram.clone();
            while work.child().is_none() {
                super::super::native_work::compare_instruction(
                    &format!("closing child case={case} pc={:04x}", cpu.pc),
                    &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                    |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                work = work.clone();
                instructions += 1;
            }
            if case == 0 {
                assert_eq!(work.child(), Some(0x0299ca));
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x99ca, 0x1fa));
                assert_eq!(&ram[0x1fb..0x1fd], &[0x8c, 0x99]);
            } else {
                assert_eq!(work.child(), Some(0x07e245));
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (7, 0xe245, 0x1f9));
                assert_eq!(&ram[0x1fa..0x1fd], &[0xa3, 0x99, 2]);
                assert_eq!(ram[0x351], u8::from(case == 1));
                assert_eq!(ram[0x5e], 6);
            }
            assert!(!work.complete());
        }
        eprintln!("3 closing child fixtures, {instructions} Source instruction boundaries; real near and far frames retained");
    }
    #[test]
    fn spotlight_caller_retains_goal_branches_nested_frames_and_main_wait_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut tables = 0;
        let mut visited = std::collections::BTreeSet::new();
        for module in [6, 9, 16, 0x1b] {
            for submodule in [0, 1, 255] {
                for case in 0..128usize {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    let r = &mut machine.ram;
                    r[0x10] = module;
                    r[0x11] = submodule;
                    r[0x1b] = 0;
                    r[0xa0..0xa2]
                        .copy_from_slice(&[0x20u16, 0x120, 0x21, 0xffff][case % 4].to_le_bytes());
                    r[0x8a..0x8c].copy_from_slice(
                        &[3u16, 5, 7, 0x43, 0x45, 0x47, 0, 0xffff][case / 4 % 8].to_le_bytes(),
                    );
                    r[0x696] = u8::from(case & 32 != 0);
                    r[0x698..0x69a].copy_from_slice(
                        &[0u16, 1, 0x100, 0x101, 0x7f01, 0x8000, 0x8001, 0xff00][case / 16 % 8]
                            .to_le_bytes(),
                    );
                    r[0x50] = [0, 1, 0xfe, 0xff][case % 4];
                    r[0x3b] = 0xff;
                    r[0x308] = 1 << (case % 8);
                    r[0x309] = (case / 8 % 4) as u8;
                    r[0x30a] = (case / 4 % 3) as u8;
                    r[0x376] = 0xff;
                    r[0x3ef] = 0xff;
                    r[0x96..0x99].fill(0xff);
                    r[0x1e] = 0xff;
                    r[0x1f] = 0xff;
                    r[0x12a] = 0xff;
                    r[0x1f0c] = 0xff;
                    r[0xc148..0xc14a]
                        .copy_from_slice(&(case as u16).wrapping_mul(997).to_le_bytes());
                    r[0x20..0x24].copy_from_slice(&[0x70, 1, 0x40, 2]);
                    r[0x2f] = (case / 32 % 4 * 2) as u8;
                    r[0x2e] = (case / 4 % 6) as u8;
                    r[0x67] = [0, 4, 8, 12][case % 4];
                    r[0xf359] = (case / 8 % 5) as u8;
                    r[0xf35a] = (case / 16 % 4) as u8;
                    r[0x1fb..0x200].copy_from_slice(&[0xe1, 0x9a, 0x59, 0x80, 0]);
                    for (i, b) in r[0xa20..0xaa0].iter_mut().enumerate() {
                        *b = (i as u8).wrapping_mul(37) ^ case as u8;
                    }
                    let mut cpu = NativeNmiContext {
                        a: 0x1200,
                        x: 3,
                        y: 7,
                        d: 0,
                        db: 0,
                        bank: 2,
                        pc: 0x9a1d,
                        sp: 0x1fa,
                        p: 0x30 | [0, 1, 0x41, 0xc3][case % 4],
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ppu = machine.ppu.clone();
                    let mut ram = machine.ram.clone();
                    let clock = CpuMasterTimeline::at_raster(
                        4921 + case as u64,
                        snes::CpuRasterPosition::new(
                            [20, 224, 261, 170][case % 4],
                            [0, 1300, 1350, 468][case % 4],
                        ),
                        snes::CpuBusWorkload::default(),
                        snes::CpuFieldTiming::non_interlace(case & 1 == 0),
                    );
                    let mut source = snes::RomCpuTimingProbe::new(
                        machine,
                        clock,
                        snes::SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    let mut clock = source.timeline().clone();
                    let mut work = SpotlightCallerReturn::default();
                    for _ in 0..20000 {
                        if work.complete() {
                            break;
                        }
                        let pc = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                        visited.insert(pc);
                        compare_instruction_with_devices(&format!("spotlight caller module={module} submodule={submodule} case={case} pc={pc:06x}"),
                            &mut cpu,&mut clock,&mut source,&mut ram,&rom,Some(&mut ppu),
                            |_,access|match access {
                                Access::Read(address,width) if address&0x8000!=0 && address<0x7e0000=> {
                                    let mut read=|a| {tables+=1;table_byte(a).unwrap_or_else(||panic!("caller read escaped immutable tables {a:06x} module={module} submodule={submodule} case={case} pc={pc:06x}"))};
                                    Some(if width==1 {u16::from(read(address))} else {u16::from_le_bytes([read(address),read(address+1)])})
                                }
                                Access::Write(address,_,_) if address&65535==0x420c=>Some(0),
                                _=>None,
                            },|cpu,clock,bus| {work.step(cpu,clock,bus).unwrap();});
                        assert!(
                            ram == source.snes().ram,
                            "module={module} submodule={submodule} case={case} pc={pc:06x} RAM"
                        );
                        work = work.clone();
                        instructions += 1;
                    }
                    assert!(work.complete());
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (0, 0x8034, 0x1ff, 0));
                    assert!(work.sprite_preparation_master_cycles() > 0);
                    assert_eq!(ram, source.snes().ram);
                }
            }
        }
        // All 95 post-table caller/helper instructions from the disassembly,
        // plus the drawing and main caller handoffs.
        for pc in [
            0x00893d, 0x00893f, 0x008942, 0x008944, 0x008947, 0x008949, 0x029a18, 0x029a1d,
            0x029a20, 0x029a23, 0x029a25, 0x029a27, 0x029a29, 0x029a2b, 0x029a2d, 0x029a2f,
            0x029a33, 0x029a35, 0x029a37, 0x029a39, 0x029a3b, 0x029a3d, 0x029a41, 0x029a45,
            0x029a47, 0x029a49, 0x029a4b, 0x029a4d, 0x029a4f, 0x029a51, 0x029a53, 0x029a55,
            0x029a57, 0x029a59, 0x029a5b, 0x029a5d, 0x029a5f, 0x029a61, 0x029a64, 0x029a67,
            0x029a6a, 0x029a6c, 0x029a6f, 0x029a71, 0x029a73, 0x029a74, 0x029a76, 0x029a78,
            0x029a7b, 0x029a7d, 0x029a80, 0x029a83, 0x029a86, 0x029a88, 0x029a8a, 0x029a8c,
            0x029a8e, 0x029a91, 0x029a93, 0x029a95, 0x029a97, 0x029a99, 0x029a9b, 0x029a9e,
            0x029aa0, 0x029aa3, 0x029aa6, 0x029aa8, 0x029aab, 0x029aad, 0x029ab0, 0x029ab2,
            0x029ab5, 0x029ab7, 0x029aba, 0x029abd, 0x029ac0, 0x029ac2, 0x029ac5, 0x029ac7,
            0x029aca, 0x029acc, 0x029ace, 0x029ad0, 0x029ad2, 0x029ae2, 0x07b107, 0x07b10a,
            0x07b10c, 0x07b10f, 0x07b112, 0x07b115, 0x07b117, 0x07b119, 0x07b11b, 0x0da18e,
            0x00805a, 0x00805f,
        ] {
            assert!(visited.contains(&pc), "unvisited caller path {pc:06x}");
        }
        eprintln!("1536 complete native spotlight caller returns, {instructions} Source instruction boundaries, {tables} immutable table bytes, {} distinct PCs; no behavioral ROM reads or Source child state imports",visited.len());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpotlightCallerReturn {
    pc: u32,
    link: Option<LinkOam>,
    animation: Option<PlayerAnimation>,
    closing: bool,
    child: Option<u32>,
    caller_returned: bool,
    packing: CallerPacking,
    sources: SourceWords,
    tail: MainSpriteTail,
    source_master_cycles: u64,
}
impl Default for SpotlightCallerReturn {
    fn default() -> Self {
        Self {
            pc: 0x029a1d,
            link: None,
            animation: None,
            closing: false,
            child: None,
            caller_returned: false,
            packing: Default::default(),
            sources: Default::default(),
            tail: Default::default(),
            source_master_cycles: 0,
        }
    }
}
impl SpotlightCallerReturn {
    pub(crate) fn closing() -> Self { Self { closing: true, ..Default::default() } }
    pub(crate) fn complete(&self) -> bool {
        self.tail.complete()
    }
    pub(crate) fn sprite_preparation_master_cycles(&self) -> u64 {
        self.packing.body_master_cycles()
            + self.source_master_cycles
            + self.tail.body_master_cycles()
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<(u64, u32)> {
        if self.complete() {
            return None;
        }
        assert!(self.child.is_none(), "spotlight child has not returned");
        if let Some(animation) = &mut self.animation {
            let cycles = animation.step(cpu, clock, bus).unwrap();
            if animation.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x99c6, 0x1fc));
                self.animation = None;
                self.pc = 0x0299c6;
            }
            return Some((cycles, 0x07e6a6));
        }
        if let Some(link) = &mut self.link {
            let cycles = link.step(cpu, clock, bus).unwrap();
            if link.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (0, 0x805a, 0x1ff));
                self.link = None;
                self.caller_returned = true;
            }
            return Some((cycles, 0x0da18e));
        }
        if self.caller_returned {
            if !self.packing.complete() {
                let routine = if self.packing.pending_call() {
                    0x008034
                } else {
                    0x0085fc
                };
                return self
                    .packing
                    .step_with_bus(cpu, clock, bus)
                    .map(|cycles| (cycles, routine));
            }
            if !self.sources.is_complete() {
                let cycles = self.sources.step_clocked(cpu, clock, bus).unwrap();
                self.source_master_cycles += cycles;
                return Some((cycles, 0x0085fc));
            }
            return self.tail.step(cpu, clock, bus);
        }
        assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
        assert_eq!((cpu.d, cpu.p & 8), (0, 0));
        let pc = self.pc;
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match pc {
            0x02998d | 0x029991 | 0x0299a8 | 0x0299ac => {
                let address = match pc { 0x029991 => 0x8a, 0x0299a8 => 0x2f, _ => 0x1b };
                let value = w.dp_read(address, 1); w.load(value, 1); pc + 2
            }
            0x02998f | 0x029995 | 0x0299ae | 0x0299b7 => {
                let target = match pc { 0x02998f => 0x0299a8, 0x029995 => 0x02999c, _ => 0x0299ba };
                if w.branch(w.cpu.p & ZERO == 0) { target } else { pc + 2 }
            }
            0x029993 | 0x0299b5 => {
                w.compare_immediate(if pc == 0x029993 { 15 } else { 0x43 }, 1); pc + 2
            }
            0x029997 | 0x02999c => { w.immediate(if pc == 0x029997 { 1 } else { 6 }, 1); pc + 2 }
            0x029999 => { w.drain(16); w.write(w.absolute(0x351), w.cpu.a & 255, 1); pc + 3 }
            0x02999e | 0x0299be | 0x0299c0 => {
                w.dp_store(match pc { 0x02999e => 0x5e, 0x0299be => 0x26, _ => 0x67 }, w.cpu.a & 255, 1); pc + 2
            }
            0x0299a0 => {
                w.operand_then_internal(3); w.push(2, 1); w.push(0x99a3, 2);
                self.child = Some(0x07e245); 0x07e245
            }
            0x0299a4 | 0x0299a6 => { w.dp_store(if pc == 0x0299a4 { 0x31 } else { 0x30 }, 0, 1); pc + 2 }
            0x0299aa => {
                w.drain(6); let value = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (value as u8 & 1);
                w.load(value >> 1, 1); pc + 1
            }
            0x0299ab => { w.drain(6); w.cpu.x = w.cpu.a & 255; w.flags(w.cpu.x, 1); pc + 1 }
            0x0299b0 => { w.drain(8); w.cpu.x = 0; w.flags(0, 1); pc + 2 }
            0x0299b2 => { w.drain(16); let value = w.read(w.absolute(0x10e), 1); w.load(value, 1); pc + 3 }
            0x0299b9 => { w.drain(6); w.cpu.x = w.cpu.x.wrapping_add(1) & 255; w.flags(w.cpu.x, 1); pc + 1 }
            0x0299ba => { w.drain(24); let value = w.read(0x02997a + u32::from(w.cpu.x), 1); w.load(value, 1); pc + 4 }
            0x0299c2 => {
                w.operand_then_internal(3); w.push(2, 1); w.push(0x99c5, 2);
                self.animation = Some(PlayerAnimation::default()); 0x07e6a6
            }
            0x0299c6 => { w.drain(24); self.link = Some(LinkOam::default()); 0x0da18e }
            0x029a1d | 0x029a20 | 0x029a83 | 0x029a9b | 0x008944 | 0x07b107 | 0x07b10c
            | 0x07b10f | 0x07b112 => {
                let address = match pc {
                    0x029a1d => 0x12a,
                    0x029a20 => 0x1f0c,
                    0x029a83 => 0x690,
                    0x029a9b => 0x3ef,
                    0x008944 => 0x420c,
                    0x07b107 => 0x30a,
                    0x07b10c => 0x308,
                    0x07b10f => 0x309,
                    _ => 0x376,
                };
                w.drain(16);
                if pc == 0x008944 {
                    w.write_io(address, 0, m);
                } else {
                    w.write(w.absolute(address), 0, m);
                }
                pc + 3
            }
            0x029a23 | 0x029a27 | 0x029a37 | 0x029a45 | 0x029a4b | 0x029a4f | 0x029aa6
            | 0x07b115 => {
                let address = match pc {
                    0x029a23 => 0x11,
                    0x029a4b => 0xa1,
                    0x029a4f => 0xa0,
                    0x029aa6 => 0x8a,
                    0x07b115 => 0x50,
                    _ => 0x10,
                };
                let value = w.dp_read(address, m);
                w.load(value, m);
                pc + 2
            }
            0x029a29 | 0x029a39 | 0x029a47 | 0x029a51 | 0x029aa8 | 0x029aad | 0x029ab2
            | 0x029abd | 0x029ac2 | 0x029ac7 => {
                let value = match pc {
                    0x029a29 => 6,
                    0x029a39 | 0x029a47 => 9,
                    0x029a51 => 0x20,
                    0x029aa8 => 3,
                    0x029aad => 5,
                    0x029ab2 => 7,
                    0x029abd => 0x43,
                    0x029ac2 => 0x45,
                    _ => 0x47,
                };
                w.drain(8 * u32::from(m));
                let a = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= value);
                w.flags(a.wrapping_sub(value) & mask, m);
                pc + 1 + u32::from(m)
            }
            0x029a25 | 0x029a2b | 0x029a3b | 0x029a49 | 0x029a4d | 0x029a53 | 0x029a59
            | 0x029a6a | 0x029a6f | 0x029a74 | 0x029aab | 0x029ab0 | 0x029ab5 | 0x029ac0
            | 0x029ac5 | 0x029aca => {
                let z = w.cpu.p & ZERO != 0;
                let (taken, target) = match pc {
                    0x029a25 => (!z, 0x029a18),
                    0x029a2b => (!z, 0x029a37),
                    0x029a3b => (z, 0x029a45),
                    0x029a49 => (!z, 0x029a91),
                    0x029a4d => (!z, 0x029a55),
                    0x029a53 => (z, 0x029a5f),
                    0x029a59 => (!z, 0x029a5d),
                    0x029a6a | 0x029a6f => (z, 0x029a91),
                    0x029a74 => (w.cpu.p & CARRY == 0, 0x029a78),
                    0x029aca => (!z, 0x029ad0),
                    _ => (z, 0x029acc),
                };
                if w.branch(taken) {
                    target
                } else {
                    pc + 2
                }
            }
            0x029a2d => {
                w.width(0x20, true);
                pc + 2
            }
            0x029a35 => {
                w.width(0x20, false);
                pc + 2
            }
            0x029a9e => {
                w.width(0x30, true);
                pc + 2
            }
            0x029ad0 => {
                w.width(0x30, false);
                pc + 2
            }
            0x029a2f => {
                w.drain(24);
                let value = w.read(0x7ec148, m);
                w.load(value, m);
                pc + 4
            }
            0x029a33 | 0x029a5d | 0x029a88 | 0x07b119 | 0x008942 => {
                let address = match pc {
                    0x029a33 => 0x20,
                    0x029a5d | 0x029a88 => 0x11,
                    0x07b119 => 0x50,
                    _ => 0x13,
                };
                w.dp_store(address, w.cpu.a & mask, m);
                pc + 2
            }
            0x029a3d | 0x029a41 => {
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                if pc == 0x029a3d {
                    0x00893d
                } else {
                    0x07b107
                }
            }
            0x029a55 | 0x029a5b | 0x029a5f | 0x029a86 | 0x029a8c | 0x00893d => {
                let value = match pc {
                    0x029a55 => 10,
                    0x029a5b => 11,
                    0x029a5f => 16,
                    0x029a86 => 9,
                    0x029a8c => 21,
                    _ => 128,
                };
                w.immediate(value, m);
                pc + 1 + u32::from(m)
            }
            0x029a57 => {
                let value = w.dp_read(0x2f, x);
                w.cpu.x = value;
                w.flags(value, x);
                pc + 2
            }
            0x029a61 | 0x029a7d | 0x029a8e => {
                let address = match pc {
                    0x029a61 => 0x69a,
                    0x029a7d => 0x699,
                    _ => 0x12f,
                };
                w.drain(16);
                w.write(w.absolute(address), w.cpu.a & mask, m);
                pc + 3
            }
            0x029a64 | 0x029a6c | 0x029a78 => {
                let address = if pc == 0x029a64 { 0x696 } else { 0x699 };
                w.drain(16);
                let value = w.read(w.absolute(address), m);
                w.load(value, m);
                pc + 3
            }
            0x029a67 => {
                w.drain(16);
                let value = w.read(w.absolute(0x698), m);
                w.load((w.cpu.a | value) & mask, m);
                pc + 3
            }
            0x029a71 | 0x029a76 | 0x029aa0 | 0x029aa3 | 0x029ab7 | 0x029aba => {
                let value = match pc {
                    0x029a71 => 0,
                    0x029a76 => 24,
                    0x029aa0 => 0x4c26,
                    0x029aa3 => 0x8c4c,
                    0x029ab7 => 0x4a26,
                    _ => 0x874a,
                };
                w.drain(8 * u32::from(x));
                if matches!(pc, 0x029aa3 | 0x029aba) {
                    w.cpu.y = value;
                } else {
                    w.cpu.x = value;
                }
                w.flags(value, x);
                pc + 1 + u32::from(x)
            }
            0x029a73 => {
                w.drain(6);
                let a = w.cpu.a & mask;
                w.cpu.p =
                    (w.cpu.p & !CARRY) | u8::from(a & (if m == 1 { 128 } else { 32768 }) != 0);
                w.load(a << 1 & mask, m);
                pc + 1
            }
            0x029a7b | 0x07b117 => {
                w.drain(8 * u32::from(m));
                w.load(w.cpu.a & (if pc == 0x029a7b { 127 } else { 254 }), m);
                pc + 1 + u32::from(m)
            }
            0x029a80 => {
                w.drain(16);
                w.write(w.absolute(0x692), w.cpu.x, x);
                pc + 3
            }
            0x029a8a | 0x029a91 | 0x029a93 | 0x029a95 | 0x029a97 | 0x029a99 | 0x008947
            | 0x07b10a => {
                let address = match pc {
                    0x029a8a => 0xb0,
                    0x029a91 => 0x96,
                    0x029a93 => 0x97,
                    0x029a95 => 0x98,
                    0x029a97 => 0x1e,
                    0x029a99 => 0x1f,
                    0x008947 => 0x9b,
                    _ => 0x3b,
                };
                w.dp_store(address, 0, m);
                pc + 2
            }
            0x029acc | 0x029ace => {
                w.dp_store(
                    if pc == 0x029acc { 0x9c } else { 0x9d },
                    if pc == 0x029acc { w.cpu.x } else { w.cpu.y },
                    x,
                );
                pc + 2
            }
            0x029a18 | 0x029ad2 => {
                w.rts();
                let returned = if self.closing { 0x998d } else { 0x9ae2 };
                assert_eq!((w.cpu.bank, w.cpu.pc, w.cpu.sp), (2, returned, 0x1fc));
                0x020000 | u32::from(returned)
            }
            0x029ae2 => {
                w.drain(24);
                self.link = Some(LinkOam::default());
                0x0da18e
            }
            0x00893f => {
                w.drain(16);
                w.write_io(0x2100, w.cpu.a & mask, m);
                pc + 3
            }
            0x008949 | 0x07b11b => {
                w.rtl();
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            _ => unreachable!("spotlight caller PC {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some((
            w.cycles,
            match pc >> 16 {
                0 => 0x00893d,
                7 => 0x07b107,
                _ if (0x029a37..=0x029ad2).contains(&pc) => 0x029a37,
                _ if pc == 0x029ae2 => 0x029ad7,
                _ => 0x029a19,
            },
        ))
    }
}
