//! Push-block loop, saved-tile drawing, pit publication, falling and motion.
//! Calls retain the real stack and source-width bus work until they return.

#[cfg(test)]
mod tests {
    use super::super::{intro_clear::Access, native_work::compare_instruction};
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn word_publications_preserve_source_byte_order_at_map_and_field_boundaries() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for pc in [0xd87f, 0xd812, 0xd8f9] {
            for h in [1280, 1300, 1340, 1358] {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                machine.ram.fill(0x5a);
                let mut cpu = NativeNmiContext {
                    a: 0xabcd,
                    x: if pc == 0xd812 { 0xaff } else { 0xfff },
                    y: 0,
                    d: 0,
                    db: 0,
                    bank: 1,
                    pc,
                    sp: 0x1000,
                    p: 0,
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(
                    91,
                    snes::CpuRasterPosition::new(224, h),
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
                let mut work = DungeonPushBlocks {
                    pc: 0x010000 | u32::from(pc),
                    ..Default::default()
                };
                compare_instruction(
                    &format!("word store {pc:04x} H{h}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                assert_eq!(ram, source.snes().ram);
            }
        }
    }
    #[test]
    fn push_blocks_retain_live_loop_tiles_motion_falling_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for (lo, hi) in [
            (0x01d7c0, 0x01d7c8),
            (0x07edf4, 0x07edf9),
            (0x07ee30, 0x07ee35),
            (0x07ef61, 0x07efb9),
        ] {
            for a in lo..hi {
                assert_eq!(
                    table_byte(a),
                    Some(rom[(a >> 16) as usize * 0x8000 + (a as usize & 0x7fff)]),
                    "table {a:06x}"
                );
            }
        }
        let mut visited = BTreeSet::new();
        let mut instructions = 0;
        let mut pending = 0;
        for mode in 0..9 {
            for case in 0..256usize {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
                }
                let r = &mut machine.ram;
                seed_case(r, case, mode);
                r[0x1fc] = 0xb5;
                r[0x1fd] = 0x87;
                r[0x1fe] = 2;
                let entry = if mode == 7 { 0x01d7fd } else { 0x01d81b };
                let mut cpu = NativeNmiContext {
                    a: 0xab01,
                    x: 3,
                    y: 0,
                    d: 0,
                    db: 0,
                    bank: 1,
                    pc: entry as u16,
                    sp: 0x1fb,
                    p: if mode == 7 { 0 } else { 0x30 },
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
                    100 + case as u64,
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
                let mut work = DungeonPushBlocks {
                    pc: entry,
                    ..Default::default()
                };
                for _ in 0..20000 {
                    if work.complete() || work.child().is_some() {
                        break;
                    }
                    visited.insert((u32::from(cpu.bank) << 16) | u32::from(cpu.pc));
                    let label = format!("push mode={mode} case={case} pc={:06x}", work.pc);
                    compare_instruction(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, |at, access| {
                                if let Access::Read(a, width) = access {
                                    if a & 0x408000 == 0x8000 {
                                        for byte in 0..width {
                                            let a = a + u32::from(byte);
                                            assert_eq!(
                                                table_byte(a),
                                                Some(
                                                    rom[(a >> 16) as usize * 0x8000
                                                        + (a as usize & 0x7fff)]
                                                ),
                                                "native table {a:06x}"
                                            );
                                        }
                                    }
                                }
                                bus(at, access)
                            })
                            .unwrap();
                        },
                    );
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                if let Some(PushBlockChild::RoomTransition(target)) = work.child() {
                    assert_eq!(mode, 6);
                    assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), target);
                    assert!(!work.complete());
                    pending += 1;
                } else {
                    assert!(
                        work.complete(),
                        "mode={mode} case={case} pc={:06x}",
                        work.pc
                    );
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (2, 0x87b6, 0x1fe, 0));
                    assert!(work.near_returns.is_empty() && work.far_returns.is_empty());
                    if mode == 8 {
                        assert_eq!(&ram[0x042c..0x042e], &[0, 0]);
                    }
                }
            }
        }
        for pc in [
            0x01d828, 0x01d96c, 0x01d95b, 0x01d960, 0x01d98e, 0x07ee35, 0x07ef42, 0x07efb9,
            0x07f094, 0x07ee17, 0x01d815, 0x01d827,
        ] {
            assert!(visited.contains(&pc), "uncovered push-block path {pc:06x}");
        }
        eprintln!("2304 push-block fixtures, {instructions} instruction boundaries, {} distinct PCs, {pending} explicit room-transition children",visited.len());
    }
}

use super::{
    indoor_camera::IndoorCamera,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PushBlockChild {
    RoomTransition(u32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonPushBlocks {
    pc: u32,
    near_returns: Vec<u32>,
    far_returns: Vec<u32>,
    camera: Option<IndoorCamera>,
    child: Option<PushBlockChild>,
    complete: bool,
}
impl Default for DungeonPushBlocks {
    fn default() -> Self {
        Self {
            pc: 0x01d81b,
            near_returns: Vec::new(),
            far_returns: Vec::new(),
            camera: None,
            child: None,
            complete: false,
        }
    }
}
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    crate::ZeldaState::native_push_block_table_byte(address)
        .or_else(|| super::indoor_camera::table_byte(address))
}
impl DungeonPushBlocks {
    #[cfg(test)]
    pub(crate) fn pit_check_entry() -> Self {
        Self { pc: 0x01d7fd, ..Self::default() }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child(&self) -> Option<PushBlockChild> {
        self.child
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let camera = self
            .camera
            .as_mut()
            .expect("room transition has a camera owner");
        assert_eq!(
            self.child.take(),
            camera.child().map(PushBlockChild::RoomTransition)
        );
        camera.return_from_child(cpu);
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
        assert!(self.child.is_none(), "push-block child still owns the CPU");
        if let Some(camera) = self.camera.as_mut() {
            let cycles = camera.step(cpu, clock, bus);
            self.child = camera.child().map(PushBlockChild::RoomTransition);
            if camera.complete() {
                let target = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                assert_eq!(self.near_returns.pop(), Some(target));
                self.pc = target;
                self.camera = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 8),
            ((self.pc >> 16) as u8, self.pc as u16, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let xmask = if x == 1 { 255 } else { 65535 };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0x1d7c8 | 0x1d7d8 | 0x1d7f5 | 0x1d82b | 0x1d831 | 0x1d837 | 0x1d83d | 0x1d843
            | 0x1d84a | 0x1d851 | 0x1d858 | 0x1d85f | 0x1d875 | 0x1d8d4 | 0x1d8e5 | 0x1d913
            | 0x1d986 | 0x1d990 | 0x7eddb | 0x7ede0 | 0x7ede5 | 0x7edea | 0x7ee48 | 0x7eea6
            | 0x7eecb | 0x7eef0 | 0x7eef5 | 0x7eefa | 0x7eeff | 0x7ef48 | 0x7ef4d | 0x7ef53
            | 0x7efdc | 0x7efe1 | 0x7f006 | 0x7f00b => {
                let (operand, next) = match pc {
                    0x1d7c8 => (0x0500, 0x1d7cb),
                    0x1d7d8 => (0x0540, 0x1d7db),
                    0x1d7f5 => (0x0500, 0x1d7f8),
                    0x1d82b => (0x0560, 0x1d82e),
                    0x1d831 => (0x0580, 0x1d834),
                    0x1d837 => (0x05a0, 0x1d83a),
                    0x1d83d => (0x05c0, 0x1d840),
                    0x1d843 => (0x0540, 0x1d846),
                    0x1d84a => (0x0560, 0x1d84d),
                    0x1d851 => (0x0580, 0x1d854),
                    0x1d858 => (0x05a0, 0x1d85b),
                    0x1d85f => (0x05c0, 0x1d862),
                    0x1d875 => (0x0540, 0x1d878),
                    0x1d8d4 => (0x0540, 0x1d8d7),
                    0x1d8e5 => (0x0540, 0x1d8e8),
                    0x1d913 => (0x0540, 0x1d916),
                    0x1d986 => (0x0540, 0x1d989),
                    0x1d990 => (0x0540, 0x1d993),
                    0x7eddb => (0x05f0, 0x7edde),
                    0x7ede0 => (0x05ec, 0x7ede3),
                    0x7ede5 => (0x05e4, 0x7ede8),
                    0x7edea => (0x05e0, 0x7eded),
                    0x7ee48 => (0x05f8, 0x7ee4b),
                    0x7eea6 => (0x05f0, 0x7eea9),
                    0x7eecb => (0x05e4, 0x7eece),
                    0x7eef0 => (0x05e4, 0x7eef3),
                    0x7eef5 => (0x05e0, 0x7eef8),
                    0x7eefa => (0x05f0, 0x7eefd),
                    0x7eeff => (0x05ec, 0x7ef02),
                    0x7ef48 => (0x05f8, 0x7ef4b),
                    0x7ef4d => (0xef61, 0x7ef50),
                    0x7ef53 => (0xef65, 0x7ef56),
                    0x7efdc => (0xefa1, 0x7efdf),
                    0x7efe1 => (0xefb1, 0x7efe4),
                    0x7f006 => (0xef99, 0x7f009),
                    0x7f00b => (0xefa9, 0x7f00e),
                    _ => unreachable!(),
                };
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.y), m);
                w.load(value, m);
                next
            }
            0x1d7cb | 0x1d8f7 | 0x1d940 | 0x7ee26 | 0x7edc9 | 0x7ee58 | 0x7ee5c | 0x7ee86
            | 0x7f054 | 0x7f064 | 0x7f09b => {
                let (operand, next) = match pc {
                    0x1d7cb => (0x1d815, 0x1d7cd),
                    0x1d8f7 => (0x1d96c, 0x1d8f9),
                    0x1d940 => (0x1d945, 0x1d942),
                    0x7ee26 => (0x7ee2a, 0x7ee28),
                    0x7edc9 => (0x7edcd, 0x7edcb),
                    0x7ee58 => (0x7ee6b, 0x7ee5a),
                    0x7ee5c => (0x7ee65, 0x7ee5e),
                    0x7ee86 => (0x7eead, 0x7ee88),
                    0x7f054 => (0x7f058, 0x7f056),
                    0x7f064 => (0x7f07f, 0x7f066),
                    0x7f09b => (0x7f09e, 0x7f09d),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    operand
                } else {
                    next
                }
            }
            0x1d7cd | 0x1d7e5 | 0x1d7f8 | 0x1d802 | 0x1d8f4 | 0x1d94d | 0x7ee90 | 0x7eeb5
            | 0x7ef0a | 0x7ef2d | 0x7ef3b | 0x7f06d | 0x7f084 => {
                let (operand, next) = match pc {
                    0x1d7cd => (0x0001, 0x1d7d0),
                    0x1d7e5 => (0x0002, 0x1d7e8),
                    0x1d7f8 => (0x0003, 0x1d7fb),
                    0x1d802 => (0x0004, 0x1d805),
                    0x1d8f4 => (0x0020, 0x1d8f7),
                    0x1d94d => (0x0023, 0x1d950),
                    0x7ee90 => (0x0008, 0x7ee92),
                    0x7eeb5 => (0x0008, 0x7eeb7),
                    0x7ef0a => (0x0009, 0x7ef0c),
                    0x7ef2d => (0x0020, 0x7ef30),
                    0x7ef3b => (0x0020, 0x7ef3e),
                    0x7f06d => (0x0008, 0x7f070),
                    0x7f084 => (0xfff8, 0x7f087),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0x1d7d0 | 0x1d7e8 | 0x1d7fb | 0x1d805 | 0x1d823 | 0x1d8da | 0x1d950 | 0x7ee14
            | 0x7edbc | 0x7eed4 | 0x7f04a => {
                let (operand, next) = match pc {
                    0x1d7d0 => (0x1d7e5, 0x1d7d2),
                    0x1d7e8 => (0x1d802, 0x1d7ea),
                    0x1d7fb => (0x1d815, 0x1d7fd),
                    0x1d805 => (0x1d815, 0x1d807),
                    0x1d823 => (0x1d7c8, 0x1d825),
                    0x1d8da => (0x1d8e5, 0x1d8dc),
                    0x1d950 => (0x1d960, 0x1d952),
                    0x7ee14 => (0x7ee2d, 0x7ee16),
                    0x7edbc => (0x7edf2, 0x7edbe),
                    0x7eed4 => (0x7eeee, 0x7eed6),
                    0x7f04a => (0x7f05e, 0x7f04c),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO == 0) {
                    operand
                } else {
                    next
                }
            }
            0x1d7d2 | 0x1d7fd | 0x1d88d | 0x1d896 | 0x1d89f | 0x1d8a8 | 0x7edd8 | 0x7edef => {
                let (operand, next) = match pc {
                    0x1d7d2 => (0x1d828, 0x1d7d5),
                    0x1d7fd => (0x1d8d4, 0x1d800),
                    0x1d88d => (0x1d98e, 0x1d890),
                    0x1d896 => (0x1d98e, 0x1d899),
                    0x1d89f => (0x1d98e, 0x1d8a2),
                    0x1d8a8 => (0x1d98e, 0x1d8ab),
                    0x7edd8 => (0x7ee35, 0x7eddb),
                    0x7edef => (0x7efb9, 0x7edf2),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.push((next - 1) as u16, 2);
                self.near_returns.push(next);
                operand
            }
            0x1d7d5 | 0x1d80f | 0x1d828 | 0x1d887 | 0x1d8fc | 0x7ee09 => {
                let (operand, next) = match pc {
                    0x1d7d5 => (0x0474, 0x1d7d8),
                    0x1d80f => (0x042c, 0x1d812),
                    0x1d828 => (0x1000, 0x1d82b),
                    0x1d887 => (0x1000, 0x1d88a),
                    0x1d8fc => (0x1000, 0x1d8ff),
                    0x7ee09 => (0x02c3, 0x7ee0c),
                    _ => unreachable!(),
                };
                w.drain(16);
                let value = w.read(w.absolute(operand), x);
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x1d7db | 0x1d8c4 | 0x1d996 | 0x7ee7c | 0x7ef29 | 0x7ef37 | 0x7efe8 | 0x7eff0
            | 0x7eff8 | 0x7f000 | 0x7f012 | 0x7f01a | 0x7f078 | 0x7f08f | 0x7f0a0 => {
                let next = match pc {
                    0x1d7db => 0x1d7dc,
                    0x1d8c4 => 0x1d8c5,
                    0x1d996 => 0x1d997,
                    0x7ee7c => 0x7ee7d,
                    0x7ef29 => 0x7ef2a,
                    0x7ef37 => 0x7ef38,
                    0x7efe8 => 0x7efe9,
                    0x7eff0 => 0x7eff1,
                    0x7eff8 => 0x7eff9,
                    0x7f000 => 0x7f001,
                    0x7f012 => 0x7f013,
                    0x7f01a => 0x7f01b,
                    0x7f078 => 0x7f079,
                    0x7f08f => 0x7f090,
                    0x7f0a0 => 0x7f0a1,
                    _ => unreachable!(),
                };
                w.clear_carry();
                next
            }
            0x1d7dc => {
                let (operand, next) = (0x1d7c0, 0x1d7e0);
                w.drain(24);
                let value = w.read(operand + u32::from(w.cpu.x), m);
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0x1d7e0 | 0x1d963 | 0x7ee80 | 0x7ee9c | 0x7eea3 | 0x7eec1 | 0x7eec8 => {
                let (operand, next) = match pc {
                    0x1d7e0 => (0x0540, 0x1d7e3),
                    0x1d963 => (0x0500, 0x1d966),
                    0x7ee80 => (0x05f4, 0x7ee83),
                    0x7ee9c => (0x05f0, 0x7ee9f),
                    0x7eea3 => (0x05ec, 0x7eea6),
                    0x7eec1 => (0x05e4, 0x7eec4),
                    0x7eec8 => (0x05e0, 0x7eecb),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.write(w.absolute(operand) + u32::from(w.cpu.y), w.cpu.a & mask, m);
                next
            }
            0x1d7e3 | 0x1d800 | 0x1d80d | 0x1d95e | 0x7ee69 | 0x7eeab | 0x7f07d => {
                let (operand, next) = match pc {
                    0x1d7e3 => (0x1d80f, 0x1d7e5),
                    0x1d800 => (0x1d80f, 0x1d802),
                    0x1d80d => (0x1d815, 0x1d80f),
                    0x1d95e => (0x1d963, 0x1d960),
                    0x7ee69 => (0x7ee76, 0x7ee6b),
                    0x7eeab => (0x7eed0, 0x7eead),
                    0x7f07d => (0x7f094, 0x7f07f),
                    _ => unreachable!(),
                };
                if w.branch(true) {
                    operand
                } else {
                    next
                }
            }
            0x1d7ea | 0x1d807 | 0x1d825 | 0x1d8cb | 0x1d932 | 0x1d96c | 0x7edf9 | 0x7edb5
            | 0x7eeee | 0x7ef40 | 0x7ef5a | 0x7f094 | 0x7f0a5 => {
                let (operand, next) = match pc {
                    0x1d7ea => (0x0030, 0x1d7ec),
                    0x1d807 => (0x0030, 0x1d809),
                    0x1d825 => (0x0030, 0x1d827),
                    0x1d8cb => (0x0020, 0x1d8cd),
                    0x1d932 => (0x0020, 0x1d934),
                    0x1d96c => (0x0020, 0x1d96e),
                    0x7edf9 => (0x0030, 0x7edfb),
                    0x7edb5 => (0x0030, 0x7edb7),
                    0x7eeee => (0x0020, 0x7eef0),
                    0x7ef40 => (0x0020, 0x7ef42),
                    0x7ef5a => (0x0020, 0x7ef5c),
                    0x7f094 => (0x0020, 0x7f096),
                    0x7f0a5 => (0x0020, 0x7f0a7),
                    _ => unreachable!(),
                };
                w.width(operand as u8, false);
                next
            }
            0x1d7ec | 0x1d809 => {
                let (operand, next) = match pc {
                    0x1d7ec => (0x7edb5, 0x1d7f0),
                    0x1d809 => (0x7edf9, 0x1d80d),
                    _ => unreachable!(),
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((next - 1) as u16, 2);
                self.far_returns.push(next);
                operand
            }
            0x1d7f0 | 0x1d81b | 0x1d8d1 | 0x1d948 | 0x1d973 | 0x7ef22 | 0x7efc6 => {
                let (operand, next) = match pc {
                    0x1d7f0 => (0x0030, 0x1d7f2),
                    0x1d81b => (0x0030, 0x1d81d),
                    0x1d8d1 => (0x0030, 0x1d8d3),
                    0x1d948 => (0x0020, 0x1d94a),
                    0x1d973 => (0x0020, 0x1d975),
                    0x7ef22 => (0x0020, 0x7ef24),
                    0x7efc6 => (0x0020, 0x7efc8),
                    _ => unreachable!(),
                };
                w.width(operand as u8, true);
                next
            }
            0x1d7f2 | 0x1d81d | 0x1d975 => {
                let (operand, next) = match pc {
                    0x1d7f2 => (0x042c, 0x1d7f5),
                    0x1d81d => (0x042c, 0x1d820),
                    0x1d975 => (0x042c, 0x1d978),
                    _ => unreachable!(),
                };
                w.drain(16);
                let value = w.read(w.absolute(operand), x);
                w.cpu.y = value;
                w.flags(value, x);
                next
            }
            0x1d812 | 0x7eedf => {
                let (operand, next) = match pc {
                    0x1d812 => (0x0500, 0x1d815),
                    0x7eedf => (0x0500, 0x7eee2),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                let address = w.absolute(operand) + u32::from(w.cpu.x);
                let old = w.read(address, m);
                w.drain(6);
                let value = old.wrapping_add(1) & mask;
                w.flags(value, m);
                w.write_high_low(address, value, m);
                next
            }
            0x1d815 | 0x1d818 | 0x7ee06 => {
                let (operand, next) = match pc {
                    0x1d815 => (0x042c, 0x1d818),
                    0x1d818 => (0x042c, 0x1d81b),
                    0x7ee06 => (0x02c3, 0x7ee09),
                    _ => unreachable!(),
                };
                w.drain(16);
                let address = w.absolute(operand);
                let old = w.read(address, m);
                w.drain(6);
                let value = old.wrapping_add(1) & mask;
                w.flags(value, m);
                w.write_high_low(address, value, m);
                next
            }
            0x1d820 => {
                let (operand, next) = (0x0478, 0x1d823);
                w.drain(16);
                let value = w.read(w.absolute(operand), x);
                let left = w.cpu.y & xmask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & xmask, x);
                next
            }
            0x1d827 | 0x7ee2f | 0x7edf3 => {
                w.rtl();
                let target = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if let Some(expected) = self.far_returns.pop() {
                    assert_eq!(target, expected);
                } else {
                    assert!(self.near_returns.is_empty());
                    self.complete = true;
                }
                target
            }
            0x1d82e | 0x1d834 | 0x1d83a | 0x1d840 | 0x1d890 | 0x1d899 | 0x1d8a2 | 0x1d8ab
            | 0x1d8b1 | 0x1d8b4 | 0x1d8b7 | 0x1d8ba | 0x1d8c0 | 0x1d902 | 0x1d906 | 0x1d90c
            | 0x1d910 | 0x7ef44 | 0x7ef50 | 0x7ef56 => {
                let (operand, next) = match pc {
                    0x1d82e => (0x1006, 0x1d831),
                    0x1d834 => (0x100c, 0x1d837),
                    0x1d83a => (0x1012, 0x1d83d),
                    0x1d840 => (0x1018, 0x1d843),
                    0x1d890 => (0x1002, 0x1d893),
                    0x1d899 => (0x1008, 0x1d89c),
                    0x1d8a2 => (0x100e, 0x1d8a5),
                    0x1d8ab => (0x1014, 0x1d8ae),
                    0x1d8b1 => (0x1004, 0x1d8b4),
                    0x1d8b4 => (0x100a, 0x1d8b7),
                    0x1d8b7 => (0x1010, 0x1d8ba),
                    0x1d8ba => (0x1016, 0x1d8bd),
                    0x1d8c0 => (0x101a, 0x1d8c3),
                    0x1d902 => (0x1006, 0x1d905),
                    0x1d906 => (0x1012, 0x1d909),
                    0x1d90c => (0x100c, 0x1d90f),
                    0x1d910 => (0x1018, 0x1d913),
                    0x7ef44 => (0x0ea0, 0x7ef47),
                    0x7ef50 => (0x0f40, 0x7ef53),
                    0x7ef56 => (0x0f30, 0x7ef59),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.write(w.absolute(operand) + u32::from(w.cpu.x), w.cpu.a & mask, m);
                next
            }
            0x1d846 | 0x1d866 | 0x1d86e | 0x1d878 | 0x1d8d7 | 0x1d8e8 | 0x1d8f1 | 0x1d916
            | 0x1d97f | 0x1d993 | 0x1d99b | 0x1d9a7 | 0x1d9b1 | 0x7eea9 | 0x7eece | 0x7eee4
            | 0x7eeea | 0x7efca | 0x7f022 | 0x7f044 | 0x7f059 | 0x7f061 | 0x7f099 => {
                let (operand, next) = match pc {
                    0x1d846 => (0x3fff, 0x1d849),
                    0x1d866 => (0x03ff, 0x1d869),
                    0x1d86e => (0x00ff, 0x1d871),
                    0x1d878 => (0x3fff, 0x1d87b),
                    0x1d8d7 => (0x4000, 0x1d8da),
                    0x1d8e8 => (0x3fff, 0x1d8eb),
                    0x1d8f1 => (0x00ff, 0x1d8f4),
                    0x1d916 => (0x3fff, 0x1d919),
                    0x1d97f => (0x00ff, 0x1d982),
                    0x1d993 => (0x3fff, 0x1d996),
                    0x1d99b => (0x0040, 0x1d99e),
                    0x1d9a7 => (0x303f, 0x1d9aa),
                    0x1d9b1 => (0x0f80, 0x1d9b4),
                    0x7eea9 => (0x000f, 0x7eeab),
                    0x7eece => (0x000f, 0x7eed0),
                    0x7eee4 => (0x00fb, 0x7eee6),
                    0x7eeea => (0x00fb, 0x7eeec),
                    0x7efca => (0x000f, 0x7efcd),
                    0x7f022 => (0xfffb, 0x7f025),
                    0x7f044 => (0x00ff, 0x7f047),
                    0x7f059 => (0x00ff, 0x7f05c),
                    0x7f061 => (0x0002, 0x7f064),
                    0x7f099 => (0x0004, 0x7f09b),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(w.cpu.a & value & mask, m);
                next
            }
            0x1d849 | 0x1d869 | 0x1d87c | 0x1d8ec | 0x1d919 | 0x7ee4c | 0x7eed8 | 0x7eede
            | 0x7f050 => {
                let next = match pc {
                    0x1d849 => 0x1d84a,
                    0x1d869 => 0x1d86a,
                    0x1d87c => 0x1d87d,
                    0x1d8ec => 0x1d8ed,
                    0x1d919 => 0x1d91a,
                    0x7ee4c => 0x7ee4d,
                    0x7eed8 => 0x7eed9,
                    0x7eede => 0x7eedf,
                    0x7f050 => 0x7f051,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0x1d84d | 0x1d854 | 0x1d85b | 0x1d862 | 0x1d87f | 0x1d883 | 0x1d91d | 0x1d922
            | 0x1d929 | 0x1d92e | 0x1d982 | 0x1d989 => {
                let (operand, next) = match pc {
                    0x1d84d => (0x7e2000, 0x1d851),
                    0x1d854 => (0x7e2080, 0x1d858),
                    0x1d85b => (0x7e2002, 0x1d85f),
                    0x1d862 => (0x7e2082, 0x1d866),
                    0x1d87f => (0x7f2000, 0x1d883),
                    0x1d883 => (0x7f2040, 0x1d887),
                    0x1d91d => (0x7e2000, 0x1d921),
                    0x1d922 => (0x7e2002, 0x1d926),
                    0x1d929 => (0x7e2080, 0x1d92d),
                    0x1d92e => (0x7e2082, 0x1d932),
                    0x1d982 => (0x7ef940, 0x1d986),
                    0x1d989 => (0x7ef942, 0x1d98d),
                    _ => unreachable!(),
                };
                w.drain(24);
                w.write(operand + u32::from(w.cpu.x), w.cpu.a & mask, m);
                next
            }
            0x1d86a | 0x1d8ed => {
                let (operand, next) = match pc {
                    0x1d86a => (0x7efe00, 0x1d86e),
                    0x1d8ed => (0x7f2000, 0x1d8f1),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand + u32::from(w.cpu.x), m);
                w.load(value, m);
                next
            }
            0x1d871 | 0x1d873 | 0x1d8cf | 0x1d98e | 0x1d999 | 0x1d9a3 | 0x1d9ad | 0x7edde
            | 0x7ede3 | 0x7ede8 | 0x7eded | 0x7ee3c | 0x7ee3e | 0x7ee42 | 0x7ee46 | 0x7ee50
            | 0x7eee6 | 0x7eeec | 0x7eef3 | 0x7eef8 | 0x7eefd | 0x7ef02 | 0x7ef11 | 0x7ef16
            | 0x7ef1b | 0x7ef20 | 0x7efc0 | 0x7efc4 | 0x7efdf | 0x7efe4 | 0x7efec | 0x7eff4
            | 0x7effc | 0x7f004 | 0x7f009 | 0x7f00e | 0x7f016 | 0x7f01e | 0x7f025 | 0x7f028
            | 0x7f076 | 0x7f08d => {
                let (operand, next) = match pc {
                    0x1d871 => (0x0000, 0x1d873),
                    0x1d873 => (0x0001, 0x1d875),
                    0x1d8cf => (0x0014, 0x1d8d1),
                    0x1d98e => (0x000e, 0x1d990),
                    0x1d999 => (0x000e, 0x1d99b),
                    0x1d9a3 => (0x0000, 0x1d9a5),
                    0x1d9ad => (0x0000, 0x1d9af),
                    0x7edde => (0x0072, 0x7ede0),
                    0x7ede3 => (0x0073, 0x7ede5),
                    0x7ede8 => (0x0074, 0x7edea),
                    0x7eded => (0x0075, 0x7edef),
                    0x7ee3c => (0x000a, 0x7ee3e),
                    0x7ee3e => (0x000b, 0x7ee40),
                    0x7ee42 => (0x000c, 0x7ee44),
                    0x7ee46 => (0x000d, 0x7ee48),
                    0x7ee50 => (0x0000, 0x7ee52),
                    0x7eee6 => (0x0050, 0x7eee8),
                    0x7eeec => (0x0048, 0x7eeee),
                    0x7eef3 => (0x0000, 0x7eef5),
                    0x7eef8 => (0x0001, 0x7eefa),
                    0x7eefd => (0x0002, 0x7eeff),
                    0x7ef02 => (0x0003, 0x7ef04),
                    0x7ef11 => (0x0004, 0x7ef13),
                    0x7ef16 => (0x0005, 0x7ef18),
                    0x7ef1b => (0x0006, 0x7ef1d),
                    0x7ef20 => (0x0007, 0x7ef22),
                    0x7efc0 => (0x0040, 0x7efc2),
                    0x7efc4 => (0x0041, 0x7efc6),
                    0x7efdf => (0x000c, 0x7efe1),
                    0x7efe4 => (0x000e, 0x7efe6),
                    0x7efec => (0x0000, 0x7efee),
                    0x7eff4 => (0x0002, 0x7eff6),
                    0x7effc => (0x0004, 0x7effe),
                    0x7f004 => (0x0006, 0x7f006),
                    0x7f009 => (0x000c, 0x7f00b),
                    0x7f00e => (0x000e, 0x7f010),
                    0x7f016 => (0x0008, 0x7f018),
                    0x7f01e => (0x000a, 0x7f020),
                    0x7f025 => (0x0048, 0x7f027),
                    0x7f028 => (0x000e, 0x7f02a),
                    0x7f076 => (0x0000, 0x7f078),
                    0x7f08d => (0x0000, 0x7f08f),
                    _ => unreachable!(),
                };
                w.dp_store(operand, w.cpu.a & mask, m);
                next
            }
            0x1d87b | 0x1d8eb | 0x1d99e | 0x1d99f | 0x1d9a0 | 0x1d9a1 | 0x1d9aa | 0x1d9b4
            | 0x1d9b5 | 0x7ee4b | 0x7ee8c | 0x7ee8d | 0x7ee8e | 0x7ee8f | 0x7eeb1 | 0x7eeb2
            | 0x7eeb3 | 0x7eeb4 | 0x7eed7 | 0x7ef4b | 0x7efcf | 0x7f04f => {
                let next = match pc {
                    0x1d87b => 0x1d87c,
                    0x1d8eb => 0x1d8ec,
                    0x1d99e => 0x1d99f,
                    0x1d99f => 0x1d9a0,
                    0x1d9a0 => 0x1d9a1,
                    0x1d9a1 => 0x1d9a2,
                    0x1d9aa => 0x1d9ab,
                    0x1d9b4 => 0x1d9b5,
                    0x1d9b5 => 0x1d9b6,
                    0x7ee4b => 0x7ee4c,
                    0x7ee8c => 0x7ee8d,
                    0x7ee8d => 0x7ee8e,
                    0x7ee8e => 0x7ee8f,
                    0x7ee8f => 0x7ee90,
                    0x7eeb1 => 0x7eeb2,
                    0x7eeb2 => 0x7eeb3,
                    0x7eeb3 => 0x7eeb4,
                    0x7eeb4 => 0x7eeb5,
                    0x7eed7 => 0x7eed8,
                    0x7ef4b => 0x7ef4c,
                    0x7efcf => 0x7efd0,
                    0x7f04f => 0x7f050,
                    _ => unreachable!(),
                };
                w.drain(6);
                let value = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (value & 1) as u8;
                w.load(value >> 1, m);
                next
            }
            0x1d87d | 0x1d9a5 | 0x1d9af | 0x7edba | 0x7ee54 | 0x7ee8a | 0x7eeaf | 0x7eee2
            | 0x7eee8 | 0x7ef24 | 0x7ef32 | 0x7efbe | 0x7efc2 | 0x7efc8 | 0x7efd9 | 0x7f020
            | 0x7f02a | 0x7f034 | 0x7f042 | 0x7f066 | 0x7f07f => {
                let (operand, next) = match pc {
                    0x1d87d => (0x0000, 0x1d87f),
                    0x1d9a5 => (0x000e, 0x1d9a7),
                    0x1d9af => (0x000e, 0x1d9b1),
                    0x7edba => (0x0011, 0x7edbc),
                    0x7ee54 => (0x0000, 0x7ee56),
                    0x7ee8a => (0x0027, 0x7ee8c),
                    0x7eeaf => (0x0028, 0x7eeb1),
                    0x7eee2 => (0x0050, 0x7eee4),
                    0x7eee8 => (0x0048, 0x7eeea),
                    0x7ef24 => (0x0000, 0x7ef26),
                    0x7ef32 => (0x0002, 0x7ef34),
                    0x7efbe => (0x0021, 0x7efc0),
                    0x7efc2 => (0x0023, 0x7efc4),
                    0x7efc8 => (0x0067, 0x7efca),
                    0x7efd9 => (0x000e, 0x7efdb),
                    0x7f020 => (0x0048, 0x7f022),
                    0x7f02a => (0x0000, 0x7f02c),
                    0x7f034 => (0x0002, 0x7f036),
                    0x7f042 => (0x002f, 0x7f044),
                    0x7f066 => (0x0008, 0x7f068),
                    0x7f07f => (0x0008, 0x7f081),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.load(value, m);
                next
            }
            0x1d88a | 0x1d893 | 0x1d89c | 0x1d8a5 | 0x1d8ae | 0x1d8bd | 0x1d8cd | 0x1d8ff
            | 0x1d909 | 0x1d91a | 0x1d926 | 0x1d95b | 0x1d960 | 0x1d966 | 0x1d96e | 0x7edd0
            | 0x7ee40 | 0x7ee44 | 0x7ef42 => {
                let (operand, next) = match pc {
                    0x1d88a => (0x0000, 0x1d88d),
                    0x1d893 => (0x0080, 0x1d896),
                    0x1d89c => (0x0002, 0x1d89f),
                    0x1d8a5 => (0x0082, 0x1d8a8),
                    0x1d8ae => (0x0100, 0x1d8b1),
                    0x1d8bd => (0xffff, 0x1d8c0),
                    0x1d8cd => (0x0001, 0x1d8cf),
                    0x1d8ff => (0x0922, 0x1d902),
                    0x1d909 => (0x0932, 0x1d90c),
                    0x1d91a => (0x0922, 0x1d91d),
                    0x1d926 => (0x0932, 0x1d929),
                    0x1d95b => (0x0004, 0x1d95e),
                    0x1d960 => (0xffff, 0x1d963),
                    0x1d966 => (0x2727, 0x1d969),
                    0x1d96e => (0x0020, 0x1d970),
                    0x7edd0 => (0x0009, 0x7edd2),
                    0x7ee40 => (0x0003, 0x7ee42),
                    0x7ee44 => (0x0002, 0x7ee46),
                    0x7ef42 => (0x0008, 0x7ef44),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(value, m);
                next
            }
            0x1d8c3 | 0x7edcd | 0x7ee9f | 0x7eec4 | 0x7eed6 | 0x7f04e => {
                let next = match pc {
                    0x1d8c3 => 0x1d8c4,
                    0x7edcd => 0x7edce,
                    0x7ee9f => 0x7eea0,
                    0x7eec4 => 0x7eec5,
                    0x7eed6 => 0x7eed7,
                    0x7f04e => 0x7f04f,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.load(w.cpu.x, m);
                next
            }
            0x1d8c5 | 0x7ef2a | 0x7ef38 => {
                let (operand, next) = match pc {
                    0x1d8c5 => (0x0018, 0x1d8c8),
                    0x7ef2a => (0x0010, 0x7ef2d),
                    0x7ef38 => (0x0010, 0x7ef3b),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0x1d8c8 | 0x1d8e2 | 0x1d958 | 0x1d970 | 0x7ee0f | 0x7edd2 => {
                let (operand, next) = match pc {
                    0x1d8c8 => (0x1000, 0x1d8cb),
                    0x1d8e2 => (0x0641, 0x1d8e5),
                    0x1d958 => (0x0466, 0x1d95b),
                    0x1d970 => (0x012e, 0x1d973),
                    0x7ee0f => (0x02c4, 0x7ee12),
                    0x7edd2 => (0x02c4, 0x7edd5),
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(operand), w.cpu.a & mask, m);
                next
            }
            0x1d8d3 | 0x1d98d | 0x1d9b9 | 0x7ef60 | 0x7f0ab => {
                w.rts();
                let target = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                assert_eq!(self.near_returns.pop(), Some(target));
                target
            }
            0x1d8dc | 0x1d952 | 0x7ee39 => {
                let (operand, next) = match pc {
                    0x1d8dc => (0x0641, 0x1d8df),
                    0x1d952 => (0x0468, 0x1d955),
                    0x7ee39 => (0xee30, 0x7ee3c),
                    _ => unreachable!(),
                };
                w.drain(16);
                let value = w.read(w.absolute(operand), m);
                w.load(value, m);
                next
            }
            0x1d8df | 0x1d955 | 0x7ee60 | 0x7f072 | 0x7f089 => {
                let (operand, next) = match pc {
                    0x1d8df => (0x0001, 0x1d8e2),
                    0x1d955 => (0x0001, 0x1d958),
                    0x7ee60 => (0x00ff, 0x7ee62),
                    0x7f072 => (0xffff, 0x7f075),
                    0x7f089 => (0xffff, 0x7f08c),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load((w.cpu.a ^ value) & mask, m);
                next
            }
            0x1d8f9 | 0x7efdb => {
                let next = match pc {
                    0x1d8f9 => 0x1d8fa,
                    0x7efdb => 0x7efdc,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(w.cpu.a, m);
                next
            }
            0x1d8fa | 0x7edfe | 0x7ef47 | 0x7efb9 | 0x7f03e => {
                let next = match pc {
                    0x1d8fa => 0x1d8fb,
                    0x7edfe => 0x7edff,
                    0x7ef47 => 0x7ef48,
                    0x7efb9 => 0x7efba,
                    0x7f03e => 0x7f03f,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(w.cpu.y, x);
                next
            }
            0x1d8fb | 0x7ef04 | 0x7f03f => {
                let next = match pc {
                    0x1d8fb => 0x1d8fc,
                    0x7ef04 => 0x7ef05,
                    0x7f03f => 0x7f040,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(w.cpu.x, x);
                next
            }
            0x1d905 | 0x1d90f | 0x1d921 | 0x1d92d | 0x7ee62 | 0x7f075 | 0x7f08c => {
                let next = match pc {
                    0x1d905 => 0x1d906,
                    0x1d90f => 0x1d910,
                    0x1d921 => 0x1d922,
                    0x1d92d => 0x1d92e,
                    0x7ee62 => 0x7ee63,
                    0x7f075 => 0x7f076,
                    0x7f08c => 0x7f08d,
                    _ => unreachable!(),
                };
                w.increment_a(m);
                next
            }
            0x1d934 | 0x7edff | 0x7edbe | 0x7efba => {
                let (operand, next) = match pc {
                    0x1d934 => (0x0072, 0x1d936),
                    0x7edff => (0x000e, 0x7ee01),
                    0x7edbe => (0x0000, 0x7edc0),
                    0x7efba => (0x000e, 0x7efbc),
                    _ => unreachable!(),
                };
                w.dp_store(operand, w.cpu.y, x);
                next
            }
            0x1d936 | 0x1d942 | 0x7ee1d | 0x7ee28 | 0x7edc0 | 0x7edcb | 0x7ee52 | 0x7ee88
            | 0x7ee96 | 0x7eead | 0x7eebb | 0x7ef05 | 0x7f096 => {
                let (operand, next) = match pc {
                    0x1d936 => (0x0001, 0x1d939),
                    0x1d942 => (0x0000, 0x1d945),
                    0x7ee1d => (0x0001, 0x7ee1f),
                    0x7ee28 => (0x0000, 0x7ee2a),
                    0x7edc0 => (0x0001, 0x7edc2),
                    0x7edcb => (0x0000, 0x7edcd),
                    0x7ee52 => (0x0001, 0x7ee54),
                    0x7ee88 => (0x0000, 0x7ee8a),
                    0x7ee96 => (0x00ff, 0x7ee98),
                    0x7eead => (0x0000, 0x7eeaf),
                    0x7eebb => (0x00ff, 0x7eebd),
                    0x7ef05 => (0x000f, 0x7ef07),
                    0x7f096 => (0x0000, 0x7f098),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(x));
                let value = operand;
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x1d939 | 0x7ee0c | 0x7ee1f | 0x7edc2 | 0x7ee4d | 0x7eed9 | 0x7ef07 | 0x7ef0e
            | 0x7ef13 | 0x7ef18 | 0x7ef1d | 0x7f051 => {
                let (operand, next) = match pc {
                    0x1d939 => (0x05fc, 0x1d93c),
                    0x7ee0c => (0xedf4, 0x7ee0f),
                    0x7ee1f => (0x05fc, 0x7ee22),
                    0x7edc2 => (0x05fc, 0x7edc5),
                    0x7ee4d => (0xee31, 0x7ee50),
                    0x7eed9 => (0x05fc, 0x7eedc),
                    0x7ef07 => (0x0dd0, 0x7ef0a),
                    0x7ef0e => (0x0d10, 0x7ef11),
                    0x7ef13 => (0x0d30, 0x7ef16),
                    0x7ef18 => (0x0d00, 0x7ef1b),
                    0x7ef1d => (0x0d20, 0x7ef20),
                    0x7f051 => (0x05fc, 0x7f054),
                    _ => unreachable!(),
                };
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.x), m);
                w.load(value, m);
                next
            }
            0x1d93c | 0x7ee22 | 0x7edc5 | 0x7eedc => {
                let next = match pc {
                    0x1d93c => 0x1d93d,
                    0x7ee22 => 0x7ee23,
                    0x7edc5 => 0x7edc6,
                    0x7eedc => 0x7eedd,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & mask, m);
                next
            }
            0x1d93d | 0x7ee23 | 0x7edc6 | 0x7edce | 0x7ee78 | 0x7ee79 | 0x7ee7a | 0x7ee7b
            | 0x7eedd => {
                let next = match pc {
                    0x1d93d => 0x1d93e,
                    0x7ee23 => 0x7ee24,
                    0x7edc6 => 0x7edc7,
                    0x7edce => 0x7edcf,
                    0x7ee78 => 0x7ee79,
                    0x7ee79 => 0x7ee7a,
                    0x7ee7a => 0x7ee7b,
                    0x7ee7b => 0x7ee7c,
                    0x7eedd => 0x7eede,
                    _ => unreachable!(),
                };
                w.shift_left(m);
                next
            }
            0x1d93e | 0x7ee24 | 0x7edc7 | 0x7f02c | 0x7f030 | 0x7f036 | 0x7f03a => {
                let (operand, next) = match pc {
                    0x1d93e => (0x0072, 0x1d940),
                    0x7ee24 => (0x000e, 0x7ee26),
                    0x7edc7 => (0x0000, 0x7edc9),
                    0x7f02c => (0x0004, 0x7f02e),
                    0x7f030 => (0x0006, 0x7f032),
                    0x7f036 => (0x0004, 0x7f038),
                    0x7f03a => (0x0006, 0x7f03c),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0x1d945 | 0x7ee17 | 0x7ee2a => {
                let (operand, next) = match pc {
                    0x1d945 => (0x05fc, 0x1d948),
                    0x7ee17 => (0x0500, 0x7ee1a),
                    0x7ee2a => (0x05fc, 0x7ee2d),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.write(w.absolute(operand) + u32::from(w.cpu.x), 0, m);
                next
            }
            0x1d94a | 0x7ef5f | 0x7f05e => {
                let next = match pc {
                    0x1d94a => 0x1d94b,
                    0x7ef5f => 0x7ef60,
                    0x7f05e => 0x7f05f,
                    _ => unreachable!(),
                };
                w.drain(12);
                let value = w.pull(x);
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x1d94b | 0x7ee2d | 0x7ef59 | 0x7f05f | 0x7f0aa => {
                let next = match pc {
                    0x1d94b => 0x1d94c,
                    0x7ee2d => 0x7ee2e,
                    0x7ef59 => 0x7ef5a,
                    0x7f05f => 0x7f060,
                    0x7f0aa => 0x7f0ab,
                    _ => unreachable!(),
                };
                w.drain(12);
                let value = w.pull(x);
                w.cpu.y = value;
                w.flags(value, x);
                next
            }
            0x1d94c | 0x7f027 => {
                let next = match pc {
                    0x1d94c => 0x1d94d,
                    0x7f027 => 0x7f028,
                    _ => unreachable!(),
                };
                w.drain(12);
                let value = w.pull(m);
                w.load(value, m);
                next
            }
            0x1d969 => {
                let operand = 0x1d87f;
                w.drain(16);
                operand
            }
            0x1d978 => {
                let (operand, next) = (0x0520, 0x1d97b);
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.y), x);
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x1d97b => {
                let (operand, next) = (0x7ec000, 0x1d97f);
                w.drain(24);
                let value = w.read(operand, m);
                w.load(value, m);
                next
            }
            0x1d997 | 0x7f0a1 => {
                let (operand, next) = match pc {
                    0x1d997 => (0x000e, 0x1d999),
                    0x7f0a1 => (0x0000, 0x7f0a3),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0x1d9a2 | 0x1d9b8 => {
                let next = match pc {
                    0x1d9a2 => 0x1d9a3,
                    0x1d9b8 => 0x1d9b9,
                    _ => unreachable!(),
                };
                w.swap_bytes();
                next
            }
            0x1d9ab | 0x1d9b6 => {
                let (operand, next) = match pc {
                    0x1d9ab => (0x0000, 0x1d9ad),
                    0x1d9b6 => (0x0000, 0x1d9b8),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.load((w.cpu.a | value) & mask, m);
                next
            }
            0x7edfb | 0x7edb7 => {
                let next = match pc {
                    0x7edfb => 0x7edfc,
                    0x7edb7 => 0x7edb8,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                next
            }
            0x7edfc | 0x7edb8 => {
                let next = match pc {
                    0x7edfc => 0x7edfd,
                    0x7edb8 => 0x7edb9,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                next
            }
            0x7edfd | 0x7ee2e | 0x7edb9 | 0x7edf2 => {
                let next = match pc {
                    0x7edfd => 0x7edfe,
                    0x7ee2e => 0x7ee2f,
                    0x7edb9 => 0x7edba,
                    0x7edf2 => 0x7edf3,
                    _ => unreachable!(),
                };
                w.drain(12);
                let value = w.pull(1);
                w.cpu.db = value as u8;
                w.flags(value, 1);
                next
            }
            0x7ee01 => {
                let (operand, next) = (0x02c4, 0x7ee04);
                w.drain(16);
                let address = w.absolute(operand);
                let old = w.read(address, m);
                w.drain(6);
                let value = old.wrapping_sub(1) & mask;
                w.flags(value, m);
                w.write_high_low(address, value, m);
                next
            }
            0x7ee04 | 0x7ee74 | 0x7ef5d | 0x7efd4 => {
                let (operand, next) = match pc {
                    0x7ee04 => (0x7ee2d, 0x7ee06),
                    0x7ee74 => (0x7ee54, 0x7ee76),
                    0x7ef5d => (0x7ef07, 0x7ef5f),
                    0x7efd4 => (0x7efcf, 0x7efd6),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    operand
                } else {
                    next
                }
            }
            0x7ee12 | 0x7ee84 => {
                let (operand, next) = match pc {
                    0x7ee12 => (0x0004, 0x7ee14),
                    0x7ee84 => (0x0001, 0x7ee86),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(x));
                let value = operand;
                let left = w.cpu.x & xmask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & xmask, x);
                next
            }
            0x7ee16 | 0x7eed0 => {
                let next = match pc {
                    0x7ee16 => 0x7ee17,
                    0x7eed0 => 0x7eed1,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.x = w.cpu.y & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0x7ee1a | 0x7edd5 => {
                let (operand, next) = match pc {
                    0x7ee1a => (0x02c3, 0x7ee1d),
                    0x7edd5 => (0x02c3, 0x7edd8),
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(operand), 0, m);
                next
            }
            0x7edcf | 0x7ef4c => {
                let next = match pc {
                    0x7edcf => 0x7edd0,
                    0x7ef4c => 0x7ef4d,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.y = w.cpu.a & xmask;
                w.flags(w.cpu.y, x);
                next
            }
            0x7ee35 | 0x7ee37 | 0x7efbc => {
                let (operand, next) = match pc {
                    0x7ee35 => (0x0027, 0x7ee37),
                    0x7ee37 => (0x0028, 0x7ee39),
                    0x7efbc => (0x000f, 0x7efbe),
                    _ => unreachable!(),
                };
                w.dp_store(operand, 0, m);
                next
            }
            0x7ee56 | 0x7ee5a => {
                let (operand, next) = match pc {
                    0x7ee56 => (0x000c, 0x7ee58),
                    0x7ee5a => (0x000d, 0x7ee5c),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.load(w.cpu.a & value & mask, m);
                next
            }
            0x7ee5e | 0x7ee65 | 0x7ee76 | 0x7f09e => {
                let (operand, next) = match pc {
                    0x7ee5e => (0x000a, 0x7ee60),
                    0x7ee65 => (0x000a, 0x7ee67),
                    0x7ee76 => (0x0027, 0x7ee78),
                    0x7f09e => (0x0030, 0x7f0a0),
                    _ => unreachable!(),
                };
                w.operand_then_internal(1);
                let value = w.read(u32::from((operand as u16).wrapping_add(w.cpu.x)), m);
                w.load(value, m);
                next
            }
            0x7ee63 | 0x7ee67 | 0x7f0a3 => {
                let (operand, next) = match pc {
                    0x7ee63 => (0x000a, 0x7ee65),
                    0x7ee67 => (0x0027, 0x7ee69),
                    0x7f0a3 => (0x0030, 0x7f0a5),
                    _ => unreachable!(),
                };
                w.operand_then_internal(1);
                w.write(
                    u32::from((operand as u16).wrapping_add(w.cpu.x)),
                    w.cpu.a & mask,
                    m,
                );
                next
            }
            0x7ee6b | 0x7ee6d | 0x7ee6f | 0x7ee71 => {
                let (operand, next) = match pc {
                    0x7ee6b => (0x000c, 0x7ee6d),
                    0x7ee6d => (0x000c, 0x7ee6f),
                    0x7ee6f => (0x000d, 0x7ee71),
                    0x7ee71 => (0x000d, 0x7ee73),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.drain(6);
                w.cpu.p =
                    (w.cpu.p & !CARRY) | u8::from(value & (if m == 1 { 128 } else { 32768 }) != 0);
                let value = value.wrapping_shl(1) & mask;
                w.flags(value, m);
                w.write_high_low(u32::from(operand), value, m);
                next
            }
            0x7ee73 | 0x7ef5c => {
                let next = match pc {
                    0x7ee73 => 0x7ee74,
                    0x7ef5c => 0x7ef5d,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0x7ee7d | 0x7ee99 | 0x7eea0 | 0x7eebe | 0x7eec5 | 0x7efe9 | 0x7eff1 | 0x7eff9
            | 0x7f001 | 0x7f013 | 0x7f01b => {
                let (operand, next) = match pc {
                    0x7ee7d => (0x05f4, 0x7ee80),
                    0x7ee99 => (0x05f0, 0x7ee9c),
                    0x7eea0 => (0x05ec, 0x7eea3),
                    0x7eebe => (0x05e4, 0x7eec1),
                    0x7eec5 => (0x05e0, 0x7eec8),
                    0x7efe9 => (0xef71, 0x7efec),
                    0x7eff1 => (0xef79, 0x7eff4),
                    0x7eff9 => (0xef89, 0x7effc),
                    0x7f001 => (0xef91, 0x7f004),
                    0x7f013 => (0xef69, 0x7f016),
                    0x7f01b => (0xef81, 0x7f01e),
                    _ => unreachable!(),
                };
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.y), m);
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0x7ee83 => {
                let next = 0x7ee84;
                w.drain(6);
                w.push(u16::from(w.cpu.p), 1);
                next
            }
            0x7ee92 | 0x7eeb7 | 0x7ef0c | 0x7f02e | 0x7f032 | 0x7f038 | 0x7f06b | 0x7f087 => {
                let (operand, next) = match pc {
                    0x7ee92 => (0x7ee98, 0x7ee94),
                    0x7eeb7 => (0x7eebd, 0x7eeb9),
                    0x7ef0c => (0x7ef5a, 0x7ef0e),
                    0x7f02e => (0x7f034, 0x7f030),
                    0x7f032 => (0x7f03e, 0x7f034),
                    0x7f038 => (0x7f0a5, 0x7f03a),
                    0x7f06b => (0x7f0a5, 0x7f06d),
                    0x7f087 => (0x7f0a5, 0x7f089),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & CARRY == 0) {
                    operand
                } else {
                    next
                }
            }
            0x7ee94 | 0x7eeb9 => {
                let (operand, next) = match pc {
                    0x7ee94 => (0x00f0, 0x7ee96),
                    0x7eeb9 => (0x00f0, 0x7eebb),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load((w.cpu.a | value) & mask, m);
                next
            }
            0x7ee98 | 0x7eebd => {
                let next = match pc {
                    0x7ee98 => 0x7ee99,
                    0x7eebd => 0x7eebe,
                    _ => unreachable!(),
                };
                w.drain(12);
                w.cpu.p = w.pull(1) as u8;
                if w.cpu.p & 0x10 != 0 {
                    w.cpu.x &= 255;
                    w.cpu.y &= 255;
                }
                next
            }
            0x7eed1 | 0x7f047 => {
                let (operand, next) = match pc {
                    0x7eed1 => (0x05e8, 0x7eed4),
                    0x7f047 => (0x05f8, 0x7f04a),
                    _ => unreachable!(),
                };
                w.drain(16);
                if x == 2 || (operand & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(operand) + u32::from(w.cpu.x), m);
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0x7ef26 | 0x7ef34 | 0x7f068 | 0x7f081 => {
                let next = match pc {
                    0x7ef26 => 0x7ef27,
                    0x7ef34 => 0x7ef35,
                    0x7f068 => 0x7f069,
                    0x7f081 => 0x7f082,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.p |= CARRY;
                next
            }
            0x7ef27 | 0x7ef35 | 0x7f069 | 0x7f082 => {
                let (operand, next) = match pc {
                    0x7ef27 => (0x0004, 0x7ef29),
                    0x7ef35 => (0x0006, 0x7ef37),
                    0x7f069 => (0x000a, 0x7f06b),
                    0x7f082 => (0x000a, 0x7f084),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.sbc(value, m);
                next
            }
            0x7ef30 | 0x7ef3e | 0x7efd0 | 0x7f03c | 0x7f070 => {
                let (operand, next) = match pc {
                    0x7ef30 => (0x7ef5a, 0x7ef32),
                    0x7ef3e => (0x7ef5a, 0x7ef40),
                    0x7efd0 => (0x7efd9, 0x7efd2),
                    0x7f03c => (0x7f0a5, 0x7f03e),
                    0x7f070 => (0x7f0a5, 0x7f072),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & CARRY != 0) {
                    operand
                } else {
                    next
                }
            }
            0x7efcd | 0x7f04c | 0x7f056 => {
                let (operand, next) = match pc {
                    0x7efcd => (0x0006, 0x7efcf),
                    0x7f04c => (0x0001, 0x7f04e),
                    0x7f056 => (0x0004, 0x7f058),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(x));
                let value = operand;
                w.cpu.y = value;
                w.flags(value, x);
                next
            }
            0x7efd2 | 0x7efd3 => {
                let next = match pc {
                    0x7efd2 => 0x7efd3,
                    0x7efd3 => 0x7efd4,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & xmask;
                w.flags(w.cpu.y, x);
                next
            }
            0x7efd6 => {
                let operand = 0x7f0a5;
                w.operand_then_internal(2);
                operand
            }
            0x7efe6 | 0x7efee | 0x7eff6 | 0x7effe | 0x7f010 | 0x7f018 => {
                let (operand, next) = match pc {
                    0x7efe6 => (0x000c, 0x7efe8),
                    0x7efee => (0x000c, 0x7eff0),
                    0x7eff6 => (0x000e, 0x7eff8),
                    0x7effe => (0x000e, 0x7f000),
                    0x7f010 => (0x000c, 0x7f012),
                    0x7f018 => (0x000e, 0x7f01a),
                    _ => unreachable!(),
                };
                w.drain(8);
                let pointer = w.read(operand as u32, 2);
                let value = w.read(w.absolute(pointer), m);
                w.load(value, m);
                next
            }
            0x7f040 => {
                let (operand, next) = (0x000e, 0x7f042);
                let value = w.dp_read(operand, x);
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x7f058 | 0x7f060 | 0x7f098 => {
                let next = match pc {
                    0x7f058 => 0x7f059,
                    0x7f060 => 0x7f061,
                    0x7f098 => 0x7f099,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.load(w.cpu.y, m);
                next
            }
            0x7f05c => {
                let (operand, next) = (0x0048, 0x7f05e);
                let value = w.dp_read(operand, m);
                w.drain(6);
                w.cpu.p = (w.cpu.p & !ZERO) | if value & w.cpu.a & mask == 0 { ZERO } else { 0 };
                w.write_high_low(u32::from(operand), value | (w.cpu.a & mask), m);
                next
            }
            0x7f079 | 0x7f090 => {
                let (operand, next) = match pc {
                    0x7f079 => (0x000c, 0x7f07b),
                    0x7f090 => (0x000c, 0x7f092),
                    _ => unreachable!(),
                };
                w.drain(8);
                let pointer = w.read(operand as u32, 2);
                let value = w.read(w.absolute(pointer), m);
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0x7f07b | 0x7f092 => {
                let (operand, next) = match pc {
                    0x7f07b => (0x000c, 0x7f07d),
                    0x7f092 => (0x000c, 0x7f094),
                    _ => unreachable!(),
                };
                w.drain(8);
                let pointer = w.read(operand as u32, 2);
                w.write(w.absolute(pointer), w.cpu.a & mask, m);
                next
            }
            0x7f09d => {
                let next = 0x7f09e;
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0x7f0a7 => {
                let (operand, next) = (0x7e8f0, 0x7f0aa);
                w.operand_then_internal(2);
                w.push((next - 1) as u16, 2);
                self.near_returns.push(next);
                self.camera = Some(IndoorCamera::default());
                operand
            }
            _ => unreachable!("push-block continuation {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize, mode: usize) {
    fn word(r: &mut [u8], at: usize, value: u16) {
        r[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    r[0x11] = if mode == 6 { 0 } else { 15 };
    r[0x67] = (case % 16) as u8;
    r[0x2f] = (case % 4 * 2) as u8;
    r[0x02c3] = (case % 4) as u8;
    r[0x02c4] = [0, 1, 0x7f, 0x80, 0xff][case % 5];
    r[0x05fc] = 1;
    r[0x05fd] = 2;
    for index in 0..2 {
        let y = index * 2;
        word(r, 0x05f8 + y, (case % 4 * 2) as u16);
        word(r, 0x05e8 + y, (case % 16) as u16);
        r[0x05e4 + y] = (case as u8).wrapping_mul(11);
        r[0x05e0 + y] = 1;
        r[0x05f0 + y] = (case as u8).wrapping_mul(7);
        r[0x05ec + y] = 2;
    }
    // Exercise both overlapping and separated Link/block edges,
    // including the collision routine's live coordinate rewrite.
    let delta = (case / 16) as i16 - 12;
    let link_y = (u16::from(r[0x05f0]) | 0x200).wrapping_add_signed(delta);
    let link_x = (u16::from(r[0x05e4]) | 0x100).wrapping_add_signed(delta);
    word(r, 0x20, link_y);
    word(r, 0x22, link_x);
    for slot in 0..16 {
        r[0x0dd0 + slot] = if (slot + case) % 3 == 0 { 9 } else { 8 };
        r[0x0d10 + slot] = r[0x05e4].wrapping_add((slot * 2) as u8);
        r[0x0d30 + slot] = 1;
        r[0x0d00 + slot] = r[0x05f0].wrapping_add((slot * 2) as u8);
        r[0x0d20 + slot] = 2;
    }
    let index = if mode == 8 { 0xfe } else { 0 };
    let end = if mode == 0 || mode == 8 {
        0
    } else {
        2 * (case % 8 + 1)
    };
    word(r, 0x042c, index as u16);
    word(r, 0x0478, end as u16);
    word(r, 0x0474, (case % 4 * 2) as u16);
    word(r, 0x1000, (case % 8 * 24) as u16);
    for y in (0..16).step_by(2).chain(std::iter::once(0xfe)) {
        let record = match mode {
            1 => 0,
            2 => 1,
            3 | 6 => 2,
            4 | 8 => 4,
            5 => [3, 5, 0xffff, 0x8000][case % 4],
            7 => 3,
            _ => 0,
        };
        word(r, 0x0500 + y, record);
        let pos = ((case * 130 + y * 8) & 0x1ffe) as u16 | if case & 16 != 0 { 0x4000 } else { 0 };
        word(r, 0x0540 + y, pos);
        word(r, 0x0520 + y, (case % 64 * 4) as u16);
        let attr = [0x20, 0x23, 0x00, 0xff][case % 4];
        r[0x12000 + ((pos & 0x3fff) >> 1) as usize] = attr;
        for off in [0x0560, 0x0580, 0x05a0, 0x05c0] {
            word(r, off + y, ((case * 37 + off) & 0xffff) as u16);
        }
    }
    // The wrap case isolates the source's two byte-wide INCs after RTL.
    if mode == 8 {
        word(r, 0x05fe, 4);
    }
}
