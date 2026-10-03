//! Sprite_Main's shared caller, region reset, and child dispatch loops.
//! Unimplemented active bodies are explicit handoffs with their real stack
//! frames still live. They cannot be skipped or completed by this owner.
use super::{
    ancilla_main::AncillaMain,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    sprite_slot::{SlotChild, SpriteSlot},
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    super::sprite_slot::table_byte(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sprite_main_callers_match_source_and_retain_active_children() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x0683c7..0x0683d3 {
            assert_eq!(
                table_byte(a),
                Some(rom[(a >> 16) as usize * 0x8000 + (a as usize & 0x7fff)])
            );
        }
        let mut instructions = 0;
        let mut completed = 0;
        let mut children = [0usize; 12];
        for entry in [
            0x068328, 0x0684e2, 0x09b06e, 0x09b07f, 0x099f91, 0x088242, 0x09b773, 0x1de9da,
        ] {
            for case in 0..32usize {
                for field in [0, 1, 30002, 30003] {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    let ram = &mut machine.ram;
                    ram[0x1b] = 1;
                    ram[0x11] = [0, 15, 14, 1][case & 3];
                    ram[0xf3ca] = (case & 1) as u8;
                    ram[0x314] = (case * 7) as u8;
                    ram[0x47] = [0, 1, 0x80, 255][case & 3];
                    ram[0xfdc] = [0, 1, 255][case % 3];
                    ram[0xaaa] = (case * 13) as u8;
                    ram[0xfb4] = (case & 1) as u8;
                    ram[0xffa] = (case & 1) as u8;
                    ram[0xbc0..0xbe0].fill(0x47);
                    match case {
                        16 => ram[0xf3cc] = 1,
                        17 => ram[0xfac] = 1,
                        18 => ram[0xc4a + 3] = 0x21,
                        19 => {
                            ram[0x11] = 0;
                            ram[0xb00 + 4] = 1;
                        }
                        20 => {
                            ram[0x11] = 15;
                            ram[0xffa] = 1;
                            ram[0x1d00 + 3] = 1;
                        }
                        21 => ram[0xdd0 + 3] = 9,
                        22 => {
                            ram[0xfb4] = 1;
                            ram[0x1f800 + 20] = 1;
                        }
                        23 => {
                            ram[0x1b] = 0;
                            ram[0x11] = 0;
                            ram[0xffd] = 1;
                        }
                        24..=26 => {
                            ram[0x11] = 0;
                            ram[0xff9] = (case - 23) as u8;
                        }
                        27 => {
                            ram[0xfb4] = 1;
                            ram[0x1f800 + 3] = 1;
                        }
                        28 => ram[0xfc1] = 1,
                        _ => {}
                    }
                    let near = entry == 0x0684e2;
                    let mut cpu = NativeNmiContext {
                        a: 0xab17,
                        x: 3,
                        y: 7,
                        d: 0,
                        db: if near { 6 } else { 0 },
                        bank: (entry >> 16) as u8,
                        pc: entry as u16,
                        sp: if near { 0x1fd } else { 0x1fc },
                        p: 0x30 | ((case as u8) & 0xc3),
                    };
                    if near {
                        ram[0x1fe..0x200].copy_from_slice(&[0x59, 0x80]);
                    } else {
                        ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 2]);
                    }
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.db = cpu.db;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let timeline = CpuMasterTimeline::at_raster(
                        field,
                        snes::CpuRasterPosition::new(224, if case & 1 == 0 { 1340 } else { 500 }),
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
                    let mut work = SpriteMain::from_entry(entry);
                    for _ in 0..2000 {
                        if work.complete() || work.child().is_some() {
                            break;
                        }
                        let label = format!(
                            "entry={entry:06x} case={case} field={field} pc={:06x}",
                            work.pc
                        );
                        super::super::native_work::compare_instruction(
                            &label,
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
                    assert!(work.complete() || work.child().is_some());
                    assert_eq!(ram, source.snes().ram, "{entry:06x}/{case}/{field}");
                    if let Some(child) = work.child() {
                        let kind = match child {
                            SpriteChild::Proximity => 0,
                            SpriteChild::Garnish => 1,
                            SpriteChild::Follower => 2,
                            SpriteChild::AncillaSpark => 3,
                            SpriteChild::Ancilla => 4,
                            SpriteChild::Overlord => 5,
                            SpriteChild::OverworldSpawner => 6,
                            SpriteChild::Sprite(_) => 7,
                            SpriteChild::CachedSprite => 8,
                            SpriteChild::PaletteRestore => 9,
                            SpriteChild::PaletteAdd => 10,
                            SpriteChild::PaletteSubtract => 11,
                        };
                        children[kind] += 1;
                        assert!(!work.complete());
                    } else {
                        completed += 1;
                        assert_eq!(
                            (cpu.bank, cpu.pc, cpu.sp),
                            (if near { 6 } else { 2 }, 0x805a, 0x1ff)
                        );
                    }
                }
            }
        }
        assert!(
            children.iter().all(|&n| n > 0),
            "missing child branches {children:?}"
        );
        eprintln!("1024 sprite caller fixtures, {instructions} instruction boundaries, {completed} complete returns, child handoffs={children:?}");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpriteChild {
    Proximity,
    Garnish,
    Follower,
    AncillaSpark,
    Ancilla,
    Overlord,
    OverworldSpawner,
    Sprite(SlotChild),
    CachedSprite,
    PaletteRestore,
    PaletteAdd,
    PaletteSubtract,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Call {
    returned: u32,
    far: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpriteMain {
    pc: u32,
    calls: Vec<Call>,
    ancilla: Option<AncillaMain>,
    slot: Option<SpriteSlot>,
    child: Option<SpriteChild>,
    complete: bool,
}
impl Default for SpriteMain {
    fn default() -> Self {
        Self::from_entry(0x068328)
    }
}
impl SpriteMain {
    fn from_entry(pc: u32) -> Self {
        assert!(matches!(
            pc,
            0x068328 | 0x0684e2 | 0x09b06e | 0x09b07f | 0x099f91 | 0x088242 | 0x09b773 | 0x1de9da
        ));
        Self {
            pc,
            calls: Vec::new(),
            ancilla: None,
            slot: None,
            child: None,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child(&self) -> Option<SpriteChild> {
        self.child
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let child = self.child.take().expect("sprite child must be pending");
        if let Some(slot) = self.slot.as_mut() {
            assert_eq!(
                child,
                SpriteChild::Sprite(slot.child().expect("slot body must be pending"))
            );
            slot.return_from_child(cpu);
            if slot.complete() {
                self.slot = None;
                self.finish_call(cpu);
            }
            return;
        }
        if let Some(ancilla) = self.ancilla.take() {
            assert!(
                ancilla.child().is_some(),
                "ancilla must reach its body handoff"
            );
        }
        self.finish_call(cpu);
    }
    fn finish_call(&mut self, cpu: &NativeNmiContext) {
        let returned = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
        if let Some(call) = self.calls.pop() {
            assert_eq!(
                returned, call.returned,
                "sprite child must execute its own return"
            );
        } else {
            self.complete = true;
        }
        self.pc = returned;
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
        assert!(self.child.is_none(), "sprite child still owns the CPU");
        if let Some(slot) = self.slot.as_mut() {
            let cycles = slot.step(cpu, clock, bus);
            if slot.complete() {
                self.slot = None;
                self.finish_call(cpu);
            } else if let Some(child) = slot.child() {
                self.child = Some(SpriteChild::Sprite(child));
            }
            return cycles;
        }
        if let Some(ancilla) = self.ancilla.as_mut() {
            let cycles = ancilla.step(cpu, clock, bus);
            if ancilla.complete() {
                self.ancilla = None;
                let returned = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                assert_eq!(
                    self.calls.pop(),
                    Some(Call {
                        returned,
                        far: false
                    })
                );
                self.pc = returned;
            } else if ancilla.child().is_some() {
                self.child = Some(SpriteChild::Ancilla);
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x18),
            ((self.pc >> 16) as u8, self.pc as u16, 0, 0x10)
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
            0x068328 | 0x06834e | 0x068377 | 0x068510 | 0x09b77e | 0x09b714 | 0x09b71d
            | 0x1de9da | 0x1de9de => {
                let a = match pc {
                    0x068328 | 0x068510 | 0x09b714 | 0x1de9da => 0x1b,
                    0x068377 => 0x47,
                    _ => 0x11,
                };
                let v = w.dp_read(a, 1);
                w.load(v, 1);
                pc + 2
            }
            0x06832a | 0x068348 | 0x068350 | 0x06837b | 0x06837f | 0x06838f | 0x0683a8
            | 0x0683bc | 0x0683e1 | 0x0684e5 | 0x068512 | 0x09b075 | 0x09b082 | 0x09b094
            | 0x09b09d | 0x09b0bd | 0x099fc8 | 0x088f8c | 0x088333 | 0x088339 | 0x09b783
            | 0x09b78a | 0x09b790 | 0x09b716 | 0x09b71b | 0x09b722 | 0x1de9b9 | 0x1de9bd
            | 0x1de9c2 | 0x1de9cb | 0x1de9d1 | 0x1de9dc | 0x1de9e0 | 0x1de9e4 | 0x1de9e9
            | 0x1de9f3 | 0x1de9f9 => {
                let z = w.cpu.p & ZERO != 0;
                let n = w.cpu.p & NEGATIVE != 0;
                let c = w.cpu.p & CARRY != 0;
                let (take, target) = match pc {
                    0x06832a => (!z, 0x06833f),
                    0x068348 => (z, 0x06834b),
                    0x068350 => (!z, 0x06835e),
                    0x06837b => (z, 0x068381),
                    0x06837f => (true, 0x068383),
                    0x06838f => (z, 0x068394),
                    0x0683a8 => (!n, 0x0683a1),
                    0x0683bc => (z, 0x0683c1),
                    0x0683e1 => (!c, 0x0683d7),
                    0x0684e5 => (z, 0x068510),
                    0x068512 => (!z, 0x068520),
                    0x09b075 => (z, 0x09b07e),
                    0x09b082 => (z, 0x09b08b),
                    0x09b094 => (!z, 0x09b08e),
                    0x09b09d => (!n, 0x09b099),
                    0x09b0bd => (z, 0x09b123),
                    0x099fc8 => (!z, 0x099fcb),
                    0x088f8c => (z, 0x088f81),
                    0x088333 => (z, 0x088338),
                    0x088339 => (!n, 0x08832d),
                    0x09b783 => (!z, 0x09b792),
                    0x09b78a => (z, 0x09b78f),
                    0x09b790 => (!n, 0x09b787),
                    0x09b716 | 0x09b722 => (!z, 0x09b772),
                    0x09b71b => (z, 0x09b772),
                    0x1de9b9 | 0x1de9cb => (z, if pc == 0x1de9b9 { 0x1de9d9 } else { 0x1de9d3 }),
                    0x1de9bd => (!z, 0x1de9d9),
                    0x1de9c2 => (!z, 0x1de9c9),
                    0x1de9d1 => (true, 0x1de9d7),
                    0x1de9dc | 0x1de9e0 | 0x1de9e4 | 0x1de9e9 => (z, 0x1de9fc),
                    0x1de9f3 => (z, 0x1de9f8),
                    0x1de9f9 => (!n, 0x1de9ed),
                    _ => unreachable!(),
                };
                let next = if w.branch(take) { target } else { pc + 2 };
                self.child = match next {
                    0x09b0bf => Some(SpriteChild::Garnish),
                    0x099fcb => Some(SpriteChild::Follower),
                    0x088f8e => Some(SpriteChild::AncillaSpark),
                    0x09b724 => Some(SpriteChild::OverworldSpawner),
                    _ => None,
                };
                if next == 0x0684e7 {
                    self.slot = Some(SpriteSlot::default());
                }
                next
            }
            0x06832c | 0x06832f | 0x068332 | 0x068335 | 0x068338 | 0x068352 | 0x068355
            | 0x068358 | 0x06835b | 0x06836f | 0x068383 | 0x068386 | 0x068389 | 0x06839c
            | 0x0683ae | 0x0683b1 | 0x1de9fc => {
                let a = match pc {
                    0x06832c..=0x068338 => 0xc7c + (pc as u16 - 0x832c) / 3,
                    0x068352..=0x06835b => 0xb7c + (pc as u16 - 0x8352) / 3,
                    0x06836f => 0x314,
                    0x068383 => 0x379,
                    0x068386 => 0x377,
                    0x068389 => 0xb7b,
                    0x06839c => 0xb9a,
                    0x0683ae => 0x69f,
                    0x0683b1 => 0x69e,
                    _ => 0xffa,
                };
                w.drain(16);
                w.write(w.absolute(a), 0, 1);
                pc + 3
            }
            0x06833f | 0x09b077 | 0x09b084 | 0x099f91 | 0x088242 | 0x09b773 | 0x068340
            | 0x09b078 | 0x09b085 | 0x099f92 | 0x088243 | 0x09b774 => {
                let phk = matches!(
                    pc,
                    0x068340 | 0x09b078 | 0x09b085 | 0x099f92 | 0x088243 | 0x09b774
                );
                w.drain(6);
                w.push(u16::from(if phk { w.cpu.bank } else { w.cpu.db }), 1);
                pc + 1
            }
            0x068341 | 0x0683b4 | 0x09b079 | 0x09b07d | 0x09b086 | 0x09b08a | 0x099f93
            | 0x099f97 | 0x088244 | 0x08824b | 0x09b775 | 0x09b77c => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0x068342 | 0x0683d3 | 0x06839f | 0x09b08c | 0x09b097 | 0x08832b | 0x09b785
            | 0x1de9eb => {
                let value = match pc {
                    0x06839f | 0x1de9eb => 15,
                    0x09b08c => 29,
                    0x09b097 => 14,
                    0x08832b => 9,
                    0x09b785 => 7,
                    _ => 0,
                };
                w.drain(8);
                if matches!(pc, 0x068342 | 0x0683d3) {
                    w.cpu.y = value;
                } else {
                    w.cpu.x = value;
                }
                w.flags(value, 1);
                pc + 2
            }
            0x068344 | 0x099fc4 => {
                w.drain(24);
                let v = w.read(if pc == 0x068344 { 0x7ef3ca } else { 0x7ef3cc }, 1);
                w.load(v, 1);
                pc + 4
            }
            0x06834a | 0x0683dd | 0x0683de => {
                w.drain(6);
                w.cpu.y = (w.cpu.y + 1) & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0x06834b | 0x06836c | 0x068374 | 0x0683a1 | 0x0683be | 0x09b0b6 | 0x08832d
            | 0x1de9ed => {
                let a = match pc {
                    0x06834b => 0xfff,
                    0x06836c => 0xfb2,
                    0x068374 => 0xfab,
                    0x0683be => 0xfc6,
                    _ => 0xfa0,
                };
                let v = if pc == 0x06834b {
                    w.cpu.y
                } else if matches!(pc, 0x0683a1 | 0x09b0b6 | 0x08832d | 0x1de9ed) {
                    w.cpu.x
                } else {
                    w.cpu.a & 255
                };
                w.drain(16);
                w.write(w.absolute(a), v, 1);
                pc + 3
            }
            0x068369 | 0x06838c | 0x0683b9 | 0x09b072 | 0x09b07f | 0x088f89 | 0x09b718
            | 0x1de9b6 | 0x1de9e6 => {
                let a = match pc {
                    0x068369 => 0x314,
                    0x06838c => 0xfdc,
                    0x0683b9 => 0xaaa,
                    0x09b072 | 0x09b07f => 0xfb4,
                    0x088f89 => 0xfac,
                    0x09b718 => 0xffd,
                    0x1de9b6 => 0xff9,
                    _ => 0xffa,
                };
                w.drain(16);
                let v = w.read(w.absolute(a), 1);
                w.load(v, 1);
                pc + 3
            }
            0x068372 | 0x068517 | 0x068520 => {
                w.immediate(if pc == 0x068372 { 128 } else { 255 }, 1);
                pc + 2
            }
            0x068379 | 0x1de9c9 => {
                w.mask(if pc == 0x068379 { 127 } else { 1 }, 1);
                pc + 2
            }
            0x06837d => {
                w.dp_increment(0x47, 1, false);
                pc + 2
            }
            0x068381 => {
                w.dp_store(0x47, 0, 1);
                pc + 2
            }
            0x1de9d7 => {
                w.dp_increment(0x15, 1, true);
                pc + 2
            }
            0x068391 | 0x1de9bf => {
                let a = w.absolute(if pc == 0x068391 { 0xfdc } else { 0xff9 });
                w.drain(16);
                let v = w.read(a, 1).wrapping_sub(1) & 255;
                w.drain(6);
                w.flags(v, 1);
                w.write(a, v, 1);
                pc + 3
            }
            0x0683a7 | 0x09b091 | 0x09b09c | 0x088338 | 0x09b78f | 0x1de9f8 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0x0683d5 | 0x0683e3 => {
                w.width(0x20, pc == 0x0683d5);
                pc + 2
            }
            0x0683d7 | 0x0684e2 | 0x088330 | 0x09b787 | 0x1de9f0 => {
                let a = match pc {
                    0x0683d7 => 0x83c7,
                    0x0684e2 => 0xdd0,
                    0x088330 => 0xc4a,
                    0x09b787 => 0xb00,
                    _ => 0x1d00,
                };
                let index = if pc == 0x0683d7 { w.cpu.y } else { w.cpu.x };
                w.drain(16);
                if (a & 255) + index > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(index), m);
                w.load(v, m);
                pc + 3
            }
            0x0683da | 0x068519 | 0x06851c | 0x068522 => {
                let (base, index) = match pc {
                    0x0683da => (0xfe0, w.cpu.y),
                    0x068519 => (0xbc0, w.cpu.y),
                    0x06851c => (0xbc1, w.cpu.y),
                    _ => (0xbc0, w.cpu.x),
                };
                w.operand_then_internal(2);
                w.write(
                    w.absolute(base) + u32::from(index),
                    w.cpu.a & if m == 1 { 255 } else { 65535 },
                    m,
                );
                pc + 3
            }
            0x0683df | 0x09b092 => {
                w.drain(8);
                let (v, rhs) = if pc == 0x0683df {
                    (w.cpu.y, 11)
                } else {
                    (w.cpu.x, 14)
                };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(v >= rhs);
                w.flags(v.wrapping_sub(rhs) & 255, 1);
                pc + 2
            }
            0x068514 => {
                w.drain(6);
                w.load(w.cpu.x, 1);
                pc + 1
            }
            0x068515 => {
                w.shift_left(1);
                pc + 1
            }
            0x068516 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0x09b0b9 => {
                w.drain(24);
                let v = w.read(0x7ff800 + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                pc + 4
            }
            0x09b780 | 0x09b71f => {
                w.drain(16);
                let v = w.read(w.absolute(0xfc1), 1);
                w.load((w.cpu.a | v) & 255, 1);
                pc + 3
            }
            0x1de9bb => {
                w.cpu.y = w.dp_read(0x11, 1);
                w.flags(w.cpu.y, 1);
                pc + 2
            }
            0x1de9e2 => {
                w.compare_immediate(14, 1);
                pc + 2
            }
            0x06833b | 0x06835e | 0x068361 | 0x068365 | 0x068394 | 0x068398 | 0x0683a4
            | 0x0683aa | 0x0683b5 | 0x09b06e | 0x09b07a | 0x09b087 | 0x09b08e | 0x09b099
            | 0x099f94 | 0x088245 | 0x088248 | 0x088335 | 0x09b776 | 0x09b779 | 0x09b78c
            | 0x1de9c4 | 0x1de9cd | 0x1de9d3 | 0x1de9f5 => {
                let (target, far, child) = match pc {
                    0x06833b => (0x09c58f, true, Some(SpriteChild::Proximity)),
                    0x06835e => (0x0683d3, false, None),
                    0x068361 => (0x09b06e, true, None),
                    0x068365 => (0x099f91, true, None),
                    0x068394 => (0x088242, true, None),
                    0x068398 => (0x09b773, true, None),
                    0x0683a4 => (0x0684e2, false, None),
                    0x0683aa => (0x09b07f, true, None),
                    0x0683b5 => (0x1de9da, true, None),
                    0x09b06e => (0x1de9b6, true, None),
                    0x09b07a => (0x09b08c, false, None),
                    0x09b087 => (0x09b097, false, None),
                    0x09b08e | 0x09b099 => (0x09b0b6, false, None),
                    0x099f94 => (0x099fc4, false, None),
                    0x088245 => (0x088f89, false, None),
                    0x088248 => (0x08832b, false, None),
                    0x088335 => (0x08833c, false, Some(SpriteChild::Ancilla)),
                    0x09b776 => (0x09b77e, false, None),
                    0x09b779 => (0x09b714, false, None),
                    0x09b78c => (0x09b793, false, Some(SpriteChild::Overlord)),
                    0x1de9f5 => (0x1dea00, false, Some(SpriteChild::CachedSprite)),
                    0x1de9c4 => (0x0ed8fb, true, Some(SpriteChild::PaletteRestore)),
                    0x1de9cd => (0x0ed757, true, Some(SpriteChild::PaletteAdd)),
                    0x1de9d3 => (0x0ed83a, true, Some(SpriteChild::PaletteSubtract)),
                    _ => unreachable!(),
                };
                let width = if far { 3 } else { 2 };
                w.operand_then_internal(width);
                if far {
                    w.push(u16::from(w.cpu.bank), 1);
                }
                w.push(pc as u16 + u16::from(width), 2);
                self.calls.push(Call {
                    returned: pc + 1 + u32::from(width),
                    far,
                });
                if child == Some(SpriteChild::Ancilla) {
                    self.ancilla = Some(AncillaMain::default());
                } else {
                    self.child = child;
                }
                target
            }
            0x0683c1 | 0x09b07e | 0x09b08b | 0x099f98 | 0x08824c | 0x09b77d | 0x1de9c8
            | 0x1de9d9 | 0x1de9fb | 0x1de9ff | 0x0683e5 | 0x06851f | 0x068525 | 0x09b096
            | 0x09b09f | 0x09b123 | 0x099fca | 0x088f81 | 0x08833b | 0x09b792 | 0x09b772 => {
                let far = matches!(
                    pc,
                    0x0683c1
                        | 0x09b07e
                        | 0x09b08b
                        | 0x099f98
                        | 0x08824c
                        | 0x09b77d
                        | 0x1de9c8
                        | 0x1de9d9
                        | 0x1de9fb
                        | 0x1de9ff
                );
                if far {
                    w.rtl();
                } else {
                    w.rts();
                }
                let returned = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if let Some(call) = self.calls.pop() {
                    assert_eq!(call, Call { returned, far });
                } else {
                    self.complete = true;
                }
                returned
            }
            _ => unreachable!("sprite main continuation {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}
