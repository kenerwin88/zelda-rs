//! Module07's common caller after its submodule returns. Saved scroll words
//! stay on the real stack while drawing children own the CPU.

#[cfg(test)]
mod tests {
    use super::super::native_work::compare_instruction;
    use super::*;
    #[test]
    fn common_caller_retains_source_guards_scroll_stack_and_owned_draw_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        for entry in [
            0x87af, 0x87b6, 0x87c1, 0x87c9, 0x87d1, 0x87da, 0x87e5, 0x883e, 0x8842, 0x8852, 0x8856,
            0x885a,
        ] {
            for case in 0..32usize {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                r[0x11] = if case & 1 != 0 { 15 } else { 0 };
                r[0x6c] = u8::from(case & 2 != 0);
                r[0x454] = u8::from(case & 4 != 0);
                r[0x428] = u8::from(case & 8 != 0);
                r[0x478] = 2;
                r[0x458] = (case & 1) as u8;
                r[0x1b] = 1;
                r[0x5fc] = 1;
                r[0x5fd] = 2;
                r[0x500] = [0, 1, 2, 4][case % 4];
                r[0x540] = 0x80;
                r[0x541] = 2;
                r[0x474] = (case % 4 * 2) as u8;
                r[0x2c4] = 1;
                r[0x2f] = (case % 4 * 2) as u8;
                r[0x4b] = 0;
                for (i, at) in [0xe0, 0xe2, 0xe6, 0xe8, 0x11a, 0x11c, 0x422, 0x424]
                    .into_iter()
                    .enumerate()
                {
                    let value = (case as u16)
                        .wrapping_mul(997)
                        .wrapping_add((i * 71) as u16);
                    r[at..at + 2].copy_from_slice(&value.to_le_bytes());
                }
                for i in 0..16 {
                    r[0x1f1 + i] = (i as u8).wrapping_mul(17).wrapping_add(case as u8);
                }
                let mut cpu = NativeNmiContext {
                    a: 0x1200,
                    x: 3,
                    y: 7,
                    d: 0,
                    db: 0,
                    bank: 2,
                    pc: entry,
                    sp: 0x1f0,
                    p: 0x30,
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.pc = cpu.pc;
                machine.cpu.k = cpu.bank;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(
                    170 + case as u64,
                    snes::CpuRasterPosition::new(224, 1300),
                    snes::CpuBusWorkload::default(),
                    snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                );
                let mut source = snes::RomCpuTimingProbe::new(
                    machine,
                    timeline,
                    snes::SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                let mut clock = source.timeline().clone();
                let mut work = DungeonCommon {
                    pc: entry,
                    ..Default::default()
                };
                for _ in 0..20000 {
                    if work.child().is_some() || work.complete() {
                        break;
                    }
                    compare_instruction(
                        &format!(
                            "common entry={entry:04x} case={case} pc={:02x}:{:04x}",
                            cpu.bank, cpu.pc
                        ),
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    instructions += 1;
                }
                assert!(
                    work.child().is_some() || work.complete(),
                    "common entry={entry:04x} case={case}"
                );
                assert_eq!(ram, source.snes().ram);
                if matches!(entry, 0x87e5 | 0x883e | 0x8842 | 0x8852 | 0x8856 | 0x885a) {
                    assert!(work.complete());
                    assert_eq!(work.child(), None);
                    assert_eq!(cpu.sp, if matches!(entry, 0x883e | 0x8842) { 0x1fb } else { 0x1f3 });
                }
            }
        }
        eprintln!("384 Module07 common-caller fixtures, {instructions} instruction boundaries");
    }
}

use super::{
    dungeon_push_blocks::{DungeonPushBlocks, PushBlockChild},
    lamp_cone::LampCone,
    push_block_drawing::PushBlockDrawing,
    floor_indicator::FloorIndicator,
    hud_update::HudUpdate,
    link_oam::LinkOam,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
    sprite_main::{SpriteChild, SpriteMain},
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    super::dungeon_push_blocks::table_byte(address)
        .or_else(|| super::sprite_main::table_byte(address))
        .or_else(|| super::link_oam::table_byte(address))
        .or_else(|| super::lamp_cone::table_byte(address))
        .or_else(|| super::push_block_drawing::table_byte(address))
        .or_else(|| super::floor_indicator::table_byte(address))
        .or_else(|| super::hud_update::table_byte(address))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommonChild {
    Call(u32),
    PushBlock(PushBlockChild),
    Sprites(SpriteChild),
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum OwnedChild {
    PushBlocks(DungeonPushBlocks),
    Sprites(SpriteMain),
    LinkOam(LinkOam),
    Lamp(LampCone),
    PushBlockDrawing(PushBlockDrawing),
    FloorIndicator(FloorIndicator),
    Hud(HudUpdate),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonCommon {
    pc: u16,
    owned: Option<OwnedChild>,
    child: Option<CommonChild>,
    return_sp: u16,
    tail: bool,
    complete: bool,
}
impl Default for DungeonCommon {
    fn default() -> Self {
        Self {
            pc: 0x87af,
            owned: None,
            child: None,
            return_sp: 0,
            tail: false,
            complete: false,
        }
    }
}
impl DungeonCommon {
    pub(crate) fn child(&self) -> Option<CommonChild> {
        self.child
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    /// The selected native child must retire its own return on the live stack.
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let child = self
            .child
            .take()
            .expect("common caller has a pending child");
        match self.owned.as_mut() {
            Some(OwnedChild::PushBlocks(work)) => {
                assert_eq!(child, CommonChild::PushBlock(work.child().unwrap()));
                work.return_from_child(cpu);
            }
            Some(OwnedChild::Sprites(work)) => {
                assert_eq!(child, CommonChild::Sprites(work.child().unwrap()));
                work.return_from_child(cpu);
            }
            Some(OwnedChild::LinkOam(_)) => unreachable!("LinkOam owns all its children"),
            Some(OwnedChild::Lamp(_)) => unreachable!("lamp cone has no external children"),
            Some(OwnedChild::PushBlockDrawing(_)) => unreachable!("push-block drawing owns its allocator"),
            Some(OwnedChild::FloorIndicator(_)) => unreachable!("floor indicator has no children"),
            Some(OwnedChild::Hud(_)) => unreachable!("HUD update owns its shared calls"),
            None => {
                assert!(matches!(child, CommonChild::Call(_)));
                assert_eq!(cpu.sp, self.return_sp);
                if self.tail {
                    self.complete = true;
                } else {
                    assert_eq!((cpu.bank, cpu.pc), (2, self.pc));
                }
            }
        }
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
        assert!(self.child.is_none(), "common child still owns the CPU");
        if let Some(owned) = self.owned.as_mut() {
            let (cycles, complete, child) = match owned {
                OwnedChild::PushBlocks(w) => {
                    let cycles = w.step(cpu, clock, bus);
                    (cycles, w.complete(), w.child().map(CommonChild::PushBlock))
                }
                OwnedChild::Sprites(w) => {
                    let cycles = w.step(cpu, clock, bus);
                    (cycles, w.complete(), w.child().map(CommonChild::Sprites))
                }
                OwnedChild::LinkOam(w) => {
                    let cycles = w.step(cpu, clock, bus);
                    (cycles, w.complete(), None)
                }
                OwnedChild::Lamp(w) => {
                    let cycles = w.step(cpu, clock, bus);
                    (cycles, w.complete(), None)
                }
                OwnedChild::PushBlockDrawing(w) => {
                    let cycles = w.step(cpu, clock, bus);
                    (cycles, w.complete(), None)
                }
                OwnedChild::FloorIndicator(w) => {
                    let cycles = w.step(cpu, clock, bus);
                    (cycles, w.complete(), None)
                }
                OwnedChild::Hud(w) => {
                    let cycles = w.step(cpu, clock, bus);
                    (cycles, w.complete(), None)
                }
            };
            if complete {
                self.owned = None;
                assert_eq!(cpu.sp, self.return_sp);
                if self.tail {
                    self.complete = true;
                } else {
                    assert_eq!((cpu.bank, cpu.pc), (2, self.pc));
                }
            }
            self.child = child;
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 8),
            (2, self.pc, 0, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        w.fetch();
        self.pc = match pc {
            0x87af => {
                w.drain(16);
                w.write(0x42c, 0, m);
                pc + 3
            }
            0x87b6 | 0x87c1 | 0x87c9 | 0x87da | 0x87e7 | 0x87f2 | 0x87fe | 0x880a => {
                let at = match pc {
                    0x87b6 | 0x87c1 | 0x87c9 => 0x11,
                    0x87da => 0x6c,
                    0x87e7 => 0xe2,
                    0x87f2 => 0xe8,
                    0x87fe => 0xe0,
                    _ => 0xe6,
                };
                let value = w.dp_read(at, m);
                w.load(value, m);
                pc + 2
            }
            0x87b8 | 0x87c3 | 0x87cb | 0x87dc | 0x87d4 | 0x881c => {
                let equal = matches!(pc, 0x87d4 | 0x881c);
                if w.branch((w.cpu.p & ZERO != 0) == equal) {
                    match pc {
                        0x87d4 => 0x87da,
                        0x881c => 0x8838,
                        _ => 0x87e1,
                    }
                } else {
                    pc + 2
                }
            }
            0x87b2 | 0x87ba | 0x87be | 0x87c5 | 0x87cd | 0x87d6 | 0x87de | 0x87e1 | 0x883a
            | 0x883e | 0x8852 | 0x8856 | 0x885a => {
                let target = match pc {
                    0x87b2 => {
                        self.owned = Some(OwnedChild::PushBlocks(Default::default()));
                        0x01d81b
                    }
                    0x87ba => 0x00e3fa,
                    0x87be => 0x02ba31,
                    0x87c5 => 0x01c2fd,
                    0x87cd => 0x01ce70,
                    0x87d6 => 0x01d6c1,
                    0x87de => 0x02885e,
                    0x87e1 => {
                        self.owned = Some(OwnedChild::Lamp(Default::default()));
                        0x00f567
                    }
                    0x883a => {
                        self.owned = Some(OwnedChild::PushBlockDrawing(Default::default()));
                        0x07f0ac
                    }
                    0x883e => {
                        self.owned = Some(OwnedChild::Sprites(Default::default()));
                        0x068328
                    }
                    0x8852 => {
                        self.owned = Some(OwnedChild::LinkOam(Default::default()));
                        0x0da18e
                    }
                    0x8856 => {
                        self.owned = Some(OwnedChild::Hud(Default::default()));
                        0x0ddb75
                    }
                    _ => {
                        self.owned = Some(OwnedChild::FloorIndicator(Default::default()));
                        0x0afd0c
                    }
                };
                self.return_sp = w.cpu.sp;
                let size = if matches!(pc, 0x87be | 0x87de) { 3 } else { 4 };
                self.tail = pc == 0x885a;
                if self.tail {
                    w.drain(24);
                    self.return_sp = self.return_sp.wrapping_add(3);
                } else {
                    w.operand_then_internal((size - 1) as u8);
                    if size == 4 {
                        w.push(u16::from(w.cpu.bank), 1);
                    }
                    w.push(pc + size - 1, 2);
                }
                w.cpu.bank = (target >> 16) as u8;
                w.cpu.pc = target as u16;
                if self.owned.is_none() {
                    self.child = Some(CommonChild::Call(target));
                }
                pc + size
            }
            0x87d1 | 0x8816 | 0x8820 | 0x882c => {
                w.drain(16);
                let value = w.read(
                    match pc {
                        0x87d1 => 0x454,
                        0x8816 => 0x428,
                        0x8820 => 0x422,
                        _ => 0x424,
                    },
                    m,
                );
                w.load(value, m);
                pc + 3
            }
            0x87e5 | 0x8838 | 0x8842 | 0x8850 => {
                w.width(
                    if pc == 0x87e5 { 0x21 } else { 0x20 },
                    matches!(pc, 0x87e5 | 0x8842),
                );
                pc + 2
            }
            0x87e9 | 0x87f4 | 0x8800 | 0x880c | 0x882b | 0x8837 => {
                w.drain(6);
                w.push(w.cpu.a, m);
                pc + 1
            }
            0x881e | 0x881f | 0x8844 | 0x8847 | 0x884a | 0x884d => {
                w.drain(12);
                let value = w.pull(m);
                w.load(value, m);
                pc + 1
            }
            0x87ea | 0x87f6 | 0x8802 | 0x880e => {
                w.drain(16);
                let value = w.read(
                    if matches!(pc, 0x87ea | 0x8802) {
                        0x11a
                    } else {
                        0x11c
                    },
                    m,
                );
                w.adc(value);
                pc + 3
            }
            0x8824 | 0x8830 => {
                let value = w.dp_read(if pc == 0x8824 { 0xe2 } else { 0xe8 }, m);
                w.adc(value);
                pc + 2
            }
            0x87f5 | 0x8801 | 0x880d | 0x8823 | 0x882f => {
                w.clear_carry();
                pc + 1
            }
            0x87ed | 0x87f9 | 0x8805 | 0x8811 | 0x8829 | 0x8835 | 0x8845 | 0x8848 | 0x884b
            | 0x884e => {
                let at = match pc {
                    0x87ed | 0x884e => 0xe2,
                    0x87f9 | 0x884b => 0xe8,
                    0x8805 | 0x8829 | 0x8848 => 0xe0,
                    _ => 0xe6,
                };
                w.dp_store(at, w.cpu.a, m);
                pc + 2
            }
            0x87ef | 0x87fb | 0x8807 | 0x8813 | 0x8826 | 0x8832 => {
                w.drain(16);
                w.write(
                    match pc {
                        0x87ef => 0x11e,
                        0x87fb => 0x122,
                        0x8807 | 0x8826 => 0x120,
                        _ => 0x124,
                    },
                    w.cpu.a,
                    m,
                );
                pc + 3
            }
            0x8819 => {
                w.mask(255, m);
                pc + 3
            }
            _ => unreachable!("Module07 common caller {pc:04x}"),
        };
        if self.owned.is_none() && self.child.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
