//! Shared indoor camera/door continuation after player movement. Quadrant
//! changes delegate their final room/SRAM publication to the existing owner.

#[cfg(test)]
mod tests {
    use super::super::{intro_clear::Access, native_work::compare_instruction};
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn indoor_camera_matches_source_quadrant_publications_and_door_transition_boundaries() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut visited = BTreeSet::new();
        let mut transitions = BTreeSet::new();
        let mut instructions = 0;
        let mut completed = 0;
        for mode in 0..5 {
            for case in 0..1024usize {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i as u8).wrapping_mul(17).wrapping_add(case as u8);
                }
                let r = &mut machine.ram;
                r[0x1b] = u8::from(mode != 0);
                r[0x6c] = if mode < 2 { 0 } else { mode - 1 };
                r[0x26] = (case % 16) as u8;
                let lows = [0, 3, 7, 0x10, 0xe4, 0xeb, 0xfc, 0xff];
                r[0x20] = lows[case / 16 % 8];
                r[0x21] = (case / 4) as u8;
                r[0x22] = lows[case / 128 % 8];
                r[0x23] = (case / 8) as u8;
                r[0x40] = r[0x21].wrapping_add([0, 1, 255, 2][case % 4]);
                r[0x41] = r[0x23].wrapping_add([0, 1, 255, 2][case / 4 % 4]);
                r[0xa6] = ((case / 8 % 2) * 2) as u8;
                r[0xa7] = ((case / 16 % 2) * 2) as u8;
                r[0xa9] = (case % 2) as u8;
                r[0xaa] = (case / 2 % 2 * 2) as u8;
                r[0xa0] = (case % 256) as u8;
                r[0xa1] = 0;
                r[0x1fd] = 0xa9;
                r[0x1fe] = 0xf0;
                let mut cpu = NativeNmiContext {
                    a: 0xabcd,
                    x: 13,
                    y: 27,
                    d: 0,
                    db: if case & 1 != 0 { 7 } else { 0 },
                    bank: 7,
                    pc: 0xe8f0,
                    sp: 0x1fc,
                    p: 0x30 | (case as u8 & 1),
                };
                let original_db = cpu.db;
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(
                    400 + case as u64,
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
                let mut work = IndoorCamera::default();
                for _ in 0..2000 {
                    if work.complete() || work.child().is_some() {
                        break;
                    }
                    visited.insert((u32::from(cpu.bank) << 16) | u32::from(cpu.pc));
                    compare_instruction(
                        &format!(
                            "indoor camera mode={mode} case={case} pc={:02x}:{:04x}",
                            cpu.bank, cpu.pc
                        ),
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, |at, access| {
                                if let Access::Read(a, width) = access {
                                    if a & 0x408000 == 0x8000 {
                                        for b in 0..width {
                                            let a = a + u32::from(b);
                                            assert_eq!(
                                                table_byte(a),
                                                Some(
                                                    rom[(a >> 16) as usize * 0x8000
                                                        + (a as usize & 0x7fff)]
                                                ),
                                                "camera table {a:06x}"
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
                if let Some(target) = work.child() {
                    assert!(!work.complete());
                    assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), target);
                    transitions.insert(target);
                } else {
                    assert!(work.complete());
                    assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                        (7, 0xf0aa, 0x1fe, original_db)
                    );
                    assert!(work.near_returns.is_empty() && work.far_returns.is_empty());
                    completed += 1;
                }
            }
        }
        assert_eq!(
            transitions,
            BTreeSet::from([0x02b62e, 0x02b6cd, 0x02b76e, 0x02b81c])
        );
        for pc in [
            0x07e900, 0x07e8fc, 0x07e8f8, 0x07e981, 0x07e9d3, 0x02b8bd, 0x02b8f9, 0x02b909,
            0x02b919, 0x02b968, 0x02b99a, 0x02ba27, 0x02b8cb, 0x02b8f2,
        ] {
            assert!(visited.contains(&pc), "uncovered camera path {pc:06x}");
        }
        eprintln!("5120 indoor-camera fixtures, {instructions} instruction boundaries, {completed} full returns, {} distinct PCs, four retained room-transition children",visited.len());
    }
}

use super::{
    dungeon_visited::VisitedFlags,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IndoorCamera {
    pc: u32,
    near_returns: Vec<u32>,
    far_returns: Vec<u32>,
    visited: Option<VisitedFlags>,
    child: Option<u32>,
    complete: bool,
}
impl Default for IndoorCamera {
    fn default() -> Self {
        Self {
            pc: 0x07e8f0,
            near_returns: Vec::new(),
            far_returns: Vec::new(),
            visited: None,
            child: None,
            complete: false,
        }
    }
}
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    crate::ZeldaState::native_dungeon_visited_flag_byte(address)
}
impl IndoorCamera {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    /// A real far call into a room transition; that native child owns its RTL.
    pub(crate) fn child(&self) -> Option<u32> {
        self.child
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        self.child.take().expect("room transition must be pending");
        let target = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
        assert_eq!(self.far_returns.pop(), Some(target));
        self.pc = target;
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
        assert!(self.child.is_none(), "room transition still owns the CPU");
        if self.pc == 0x02b8cb && self.visited.is_none() {
            self.visited = Some(VisitedFlags::default());
        }
        if let Some(visited) = self.visited.as_mut() {
            let cycles = visited.step(cpu, clock, bus);
            if visited.complete() {
                let target = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                assert_eq!(self.far_returns.pop(), Some(target));
                self.pc = target;
                self.visited = None;
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
            0x7e8f0 | 0x7e8f4 | 0x7e905 | 0x7e917 | 0x7e926 | 0x7e931 | 0x7e93b | 0x7e944
            | 0x7e956 | 0x7e965 | 0x7e970 | 0x7e97a | 0x7e983 | 0x7e9ab | 0x7e9d6 | 0x7e9dd
            | 0x7e9e4 | 0x7e9f4 | 0x2b8bd | 0x2b8f9 | 0x2b909 | 0x2b919 => {
                let (operand, next) = match pc {
                    0x7e8f0 => (0x001b, 0x7e8f2),
                    0x7e8f4 => (0x006c, 0x7e8f6),
                    0x7e905 => (0x0026, 0x7e907),
                    0x7e917 => (0x0020, 0x7e919),
                    0x7e926 => (0x0001, 0x7e928),
                    0x7e931 => (0x0020, 0x7e933),
                    0x7e93b => (0x0001, 0x7e93d),
                    0x7e944 => (0x0026, 0x7e946),
                    0x7e956 => (0x0022, 0x7e958),
                    0x7e965 => (0x0001, 0x7e967),
                    0x7e970 => (0x0022, 0x7e972),
                    0x7e97a => (0x0001, 0x7e97c),
                    0x7e983 => (0x0069, 0x7e985),
                    0x7e9ab => (0x0068, 0x7e9ad),
                    0x7e9d6 => (0x0021, 0x7e9d8),
                    0x7e9dd => (0x0023, 0x7e9df),
                    0x7e9e4 => (0x0069, 0x7e9e6),
                    0x7e9f4 => (0x0068, 0x7e9f6),
                    0x2b8bd => (0x00a9, 0x2b8bf),
                    0x2b8f9 => (0x00a9, 0x2b8fb),
                    0x2b909 => (0x00aa, 0x2b90b),
                    0x2b919 => (0x00aa, 0x2b91b),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.load(value, m);
                next
            }
            0x7e8f2 | 0x7e8f6 | 0x7e909 | 0x7e913 | 0x7e948 | 0x7e952 | 0x7e985 | 0x7e9ad
            | 0x7e9e6 | 0x7e9f6 => {
                let (operand, next) = match pc {
                    0x7e8f2 => (0x7e900, 0x7e8f4),
                    0x7e8f6 => (0x7e8fc, 0x7e8f8),
                    0x7e909 => (0x7e942, 0x7e90b),
                    0x7e913 => (0x7e92f, 0x7e915),
                    0x7e948 => (0x7e981, 0x7e94a),
                    0x7e952 => (0x7e96e, 0x7e954),
                    0x7e985 => (0x7e9ab, 0x7e987),
                    0x7e9ad => (0x7e9d2, 0x7e9af),
                    0x7e9e6 => (0x7e9f4, 0x7e9e8),
                    0x7e9f6 => (0x7ea04, 0x7e9f8),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    operand
                } else {
                    next
                }
            }
            0x7e8f8 => {
                let operand = 0x7e901;
                w.drain(24);
                operand
            }
            0x7e8fc | 0x7e9ea | 0x7e9f0 | 0x7e9fa | 0x7ea00 => {
                let (operand, next) = match pc {
                    0x7e8fc => (0x7e9d3, 0x7e900),
                    0x7e9ea => (0x2b8bd, 0x7e9ee),
                    0x7e9f0 => (0x2b8f9, 0x7e9f4),
                    0x7e9fa => (0x2b919, 0x7e9fe),
                    0x7ea00 => (0x2b909, 0x7ea04),
                    _ => unreachable!(),
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((next - 1) as u16, 2);
                self.far_returns.push(next);
                operand
            }
            0x7e900 | 0x7e999 | 0x7e9aa | 0x7e9c1 | 0x7e9d2 | 0x2b980 | 0x2b9b2 | 0x2ba30 => {
                w.rts();
                let target = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
                if let Some(expected) = self.near_returns.pop() {
                    assert_eq!(target, expected);
                } else {
                    assert!(self.far_returns.is_empty());
                    self.complete = true;
                }
                target
            }
            0x7e901 | 0x7e903 => {
                let (operand, next) = match pc {
                    0x7e901 => (0x0068, 0x7e903),
                    0x7e903 => (0x0069, 0x7e905),
                    _ => unreachable!(),
                };
                w.dp_store(operand, 0, m);
                next
            }
            0x7e907 | 0x7e911 | 0x7e91f | 0x7e946 | 0x7e950 | 0x7e95e => {
                let (operand, next) = match pc {
                    0x7e907 => (0x000c, 0x7e909),
                    0x7e911 => (0x0004, 0x7e913),
                    0x7e91f => (0x00fc, 0x7e922),
                    0x7e946 => (0x0003, 0x7e948),
                    0x7e950 => (0x0001, 0x7e952),
                    0x7e95e => (0x00fc, 0x7e961),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(w.cpu.a & value & mask, m);
                next
            }
            0x7e90b | 0x7e94a => {
                let (operand, next) = match pc {
                    0x7e90b => (0x006c, 0x7e90d),
                    0x7e94a => (0x006c, 0x7e94c),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, x);
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x7e90d | 0x7e94c => {
                let (operand, next) = match pc {
                    0x7e90d => (0x0001, 0x7e90f),
                    0x7e94c => (0x0002, 0x7e94e),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(x));
                let value = operand;
                let left = w.cpu.x & xmask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & xmask, x);
                next
            }
            0x7e90f | 0x7e922 | 0x7e94e | 0x7e961 => {
                let (operand, next) = match pc {
                    0x7e90f => (0x7e942, 0x7e911),
                    0x7e922 => (0x7e942, 0x7e924),
                    0x7e94e => (0x7e981, 0x7e950),
                    0x7e961 => (0x7e981, 0x7e963),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO == 0) {
                    operand
                } else {
                    next
                }
            }
            0x7e915 | 0x7e92f | 0x7e954 | 0x7e96e | 0x2b968 | 0x2b99a => {
                let (operand, next) = match pc {
                    0x7e915 => (0x0020, 0x7e917),
                    0x7e92f => (0x0020, 0x7e931),
                    0x7e954 => (0x0020, 0x7e956),
                    0x7e96e => (0x0020, 0x7e970),
                    0x2b968 => (0x0020, 0x2b96a),
                    0x2b99a => (0x0020, 0x2b99c),
                    _ => unreachable!(),
                };
                w.width(operand as u8, true);
                next
            }
            0x7e919 | 0x7e958 | 0x2b96d | 0x2b977 => {
                let next = match pc {
                    0x7e919 => 0x7e91a,
                    0x7e958 => 0x7e959,
                    0x2b96d => 0x2b96e,
                    0x2b977 => 0x2b978,
                    _ => unreachable!(),
                };
                w.clear_carry();
                next
            }
            0x7e91a | 0x7e959 | 0x2b96e | 0x2b978 => {
                let (operand, next) = match pc {
                    0x7e91a => (0x001c, 0x7e91d),
                    0x7e959 => (0x0015, 0x7e95c),
                    0x2b96e => (0x0100, 0x2b971),
                    0x2b978 => (0x0100, 0x2b97b),
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
            0x7e91d | 0x7e92b | 0x7e937 | 0x7e940 | 0x7e95c | 0x7e96a | 0x7e976 | 0x7e97f
            | 0x7e9db | 0x7e9e2 | 0x2b8c1 | 0x2b8fd | 0x2b90d | 0x2b91d | 0x2ba2e => {
                let (operand, next) = match pc {
                    0x7e91d => (0x0000, 0x7e91f),
                    0x7e92b => (0x0068, 0x7e92d),
                    0x7e937 => (0x0000, 0x7e939),
                    0x7e940 => (0x0068, 0x7e942),
                    0x7e95c => (0x0000, 0x7e95e),
                    0x7e96a => (0x0069, 0x7e96c),
                    0x7e976 => (0x0000, 0x7e978),
                    0x7e97f => (0x0069, 0x7e981),
                    0x7e9db => (0x0068, 0x7e9dd),
                    0x7e9e2 => (0x0069, 0x7e9e4),
                    0x2b8c1 => (0x00a9, 0x2b8c3),
                    0x2b8fd => (0x00a9, 0x2b8ff),
                    0x2b90d => (0x00aa, 0x2b90f),
                    0x2b91d => (0x00aa, 0x2b91f),
                    0x2ba2e => (0x00a8, 0x2ba30),
                    _ => unreachable!(),
                };
                w.dp_store(operand, w.cpu.a & mask, m);
                next
            }
            0x7e924 | 0x7e939 | 0x7e942 | 0x7e963 | 0x7e978 | 0x7e981 | 0x2b97e | 0x2b9b0 => {
                let (operand, next) = match pc {
                    0x7e924 => (0x0020, 0x7e926),
                    0x7e939 => (0x0020, 0x7e93b),
                    0x7e942 => (0x0020, 0x7e944),
                    0x7e963 => (0x0020, 0x7e965),
                    0x7e978 => (0x0020, 0x7e97a),
                    0x7e981 => (0x0020, 0x7e983),
                    0x2b97e => (0x0020, 0x2b980),
                    0x2b9b0 => (0x0020, 0x2b9b2),
                    _ => unreachable!(),
                };
                w.width(operand as u8, false);
                next
            }
            0x7e928 | 0x7e933 | 0x7e93d | 0x7e967 | 0x7e972 | 0x7e97c | 0x7e9d8 | 0x7e9df
            | 0x2b99f | 0x2b9a9 => {
                let next = match pc {
                    0x7e928 => 0x7e929,
                    0x7e933 => 0x7e934,
                    0x7e93d => 0x7e93e,
                    0x7e967 => 0x7e968,
                    0x7e972 => 0x7e973,
                    0x7e97c => 0x7e97d,
                    0x7e9d8 => 0x7e9d9,
                    0x7e9df => 0x7e9e0,
                    0x2b99f => 0x2b9a0,
                    0x2b9a9 => 0x2b9aa,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.p |= CARRY;
                next
            }
            0x7e929 | 0x7e93e | 0x7e968 | 0x7e97d | 0x7e9d9 | 0x7e9e0 => {
                let (operand, next) = match pc {
                    0x7e929 => (0x0040, 0x7e92b),
                    0x7e93e => (0x0040, 0x7e940),
                    0x7e968 => (0x0041, 0x7e96a),
                    0x7e97d => (0x0041, 0x7e97f),
                    0x7e9d9 => (0x0040, 0x7e9db),
                    0x7e9e0 => (0x0041, 0x7e9e2),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.sbc(value, m);
                next
            }
            0x7e92d | 0x7e96c | 0x7e9ee | 0x2b907 | 0x2b917 | 0x2b927 => {
                let (operand, next) = match pc {
                    0x7e92d => (0x7e942, 0x7e92f),
                    0x7e96c => (0x7e981, 0x7e96e),
                    0x7e9ee => (0x7e9f4, 0x7e9f0),
                    0x2b907 => (0x2b8cb, 0x2b909),
                    0x2b917 => (0x2b8cb, 0x2b919),
                    0x2b927 => (0x2b8cb, 0x2b929),
                    _ => unreachable!(),
                };
                if w.branch(true) {
                    operand
                } else {
                    next
                }
            }
            0x7e934 | 0x7e973 | 0x2b9a0 | 0x2b9aa => {
                let (operand, next) = match pc {
                    0x7e934 => (0x0012, 0x7e937),
                    0x7e973 => (0x0008, 0x7e976),
                    0x2b9a0 => (0x0100, 0x2b9a3),
                    0x2b9aa => (0x0100, 0x2b9ad),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.sbc(value, m);
                next
            }
            0x7e987 | 0x7e9e8 => {
                let (operand, next) = match pc {
                    0x7e987 => (0x7e99a, 0x7e989),
                    0x7e9e8 => (0x7e9f0, 0x7e9ea),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & NEGATIVE != 0) {
                    operand
                } else {
                    next
                }
            }
            0x7e989 | 0x7e98c | 0x7e98f | 0x7e992 | 0x7e99a | 0x7e99d | 0x7e9a0 | 0x7e9a3
            | 0x7e9b1 | 0x7e9b4 | 0x7e9b7 | 0x7e9ba | 0x7e9c2 | 0x7e9c5 | 0x7e9c8 | 0x7e9cb => {
                let (operand, next) = match pc {
                    0x7e989 => (0x030b, 0x7e98c),
                    0x7e98c => (0x0308, 0x7e98f),
                    0x7e98f => (0x0309, 0x7e992),
                    0x7e992 => (0x0376, 0x7e995),
                    0x7e99a => (0x030b, 0x7e99d),
                    0x7e99d => (0x0308, 0x7e9a0),
                    0x7e9a0 => (0x0309, 0x7e9a3),
                    0x7e9a3 => (0x0376, 0x7e9a6),
                    0x7e9b1 => (0x030b, 0x7e9b4),
                    0x7e9b4 => (0x0308, 0x7e9b7),
                    0x7e9b7 => (0x0309, 0x7e9ba),
                    0x7e9ba => (0x0376, 0x7e9bd),
                    0x7e9c2 => (0x030b, 0x7e9c5),
                    0x7e9c5 => (0x0308, 0x7e9c8),
                    0x7e9c8 => (0x0309, 0x7e9cb),
                    0x7e9cb => (0x0376, 0x7e9ce),
                    _ => unreachable!(),
                };
                w.drain(16);
                w.write(w.absolute(operand), 0, m);
                next
            }
            0x7e995 | 0x7e9a6 | 0x7e9bd | 0x7e9ce => {
                let (operand, next) = match pc {
                    0x7e995 => (0x2b62e, 0x7e999),
                    0x7e9a6 => (0x2b6cd, 0x7e9aa),
                    0x7e9bd => (0x2b81c, 0x7e9c1),
                    0x7e9ce => (0x2b76e, 0x7e9d2),
                    _ => unreachable!(),
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push((next - 1) as u16, 2);
                self.far_returns.push(next);
                self.child = Some(operand);
                operand
            }
            0x7e9af | 0x7e9f8 => {
                let (operand, next) = match pc {
                    0x7e9af => (0x7e9c2, 0x7e9b1),
                    0x7e9f8 => (0x7ea00, 0x7e9fa),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    operand
                } else {
                    next
                }
            }
            0x7e9d3 => {
                let next = 0x7e9d4;
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                next
            }
            0x7e9d4 => {
                let next = 0x7e9d5;
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                next
            }
            0x7e9d5 | 0x7e9fe | 0x7ea04 => {
                let next = match pc {
                    0x7e9d5 => 0x7e9d6,
                    0x7e9fe => 0x7e9ff,
                    0x7ea04 => 0x7ea05,
                    _ => unreachable!(),
                };
                w.drain(12);
                let value = w.pull(1);
                w.cpu.db = value as u8;
                w.flags(value, 1);
                next
            }
            0x7e9ff | 0x7ea05 => {
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
            0x2b8bf | 0x2b8fb | 0x2b90b | 0x2b91b => {
                let (operand, next) = match pc {
                    0x2b8bf => (0x0001, 0x2b8c1),
                    0x2b8fb => (0x0001, 0x2b8fd),
                    0x2b90b => (0x0002, 0x2b90d),
                    0x2b91b => (0x0002, 0x2b91d),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load((w.cpu.a ^ value) & mask, m);
                next
            }
            0x2b8c3 | 0x2b8c8 | 0x2b8ff | 0x2b904 | 0x2b90f | 0x2b914 | 0x2b91f | 0x2b924 => {
                let (operand, next) = match pc {
                    0x2b8c3 => (0x2ba27, 0x2b8c6),
                    0x2b8c8 => (0x2b968, 0x2b8cb),
                    0x2b8ff => (0x2ba27, 0x2b902),
                    0x2b904 => (0x2b99a, 0x2b907),
                    0x2b90f => (0x2ba27, 0x2b912),
                    0x2b914 => (0x2b968, 0x2b917),
                    0x2b91f => (0x2ba27, 0x2b922),
                    0x2b924 => (0x2b99a, 0x2b927),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.push((next - 1) as u16, 2);
                self.near_returns.push(next);
                operand
            }
            0x2b8c6 | 0x2b902 | 0x2b912 | 0x2b922 => {
                let (operand, next) = match pc {
                    0x2b8c6 => (0x0008, 0x2b8c8),
                    0x2b902 => (0x0008, 0x2b904),
                    0x2b912 => (0x0000, 0x2b914),
                    0x2b922 => (0x0000, 0x2b924),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(x));
                let value = operand;
                w.cpu.x = value;
                w.flags(value, x);
                next
            }
            0x2b96a | 0x2b974 | 0x2b99c | 0x2b9a6 => {
                let (operand, next) = match pc {
                    0x2b96a => (0x0600, 0x2b96d),
                    0x2b974 => (0x0604, 0x2b977),
                    0x2b99c => (0x0600, 0x2b99f),
                    0x2b9a6 => (0x0604, 0x2b9a9),
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
            0x2b971 | 0x2b97b | 0x2b9a3 | 0x2b9ad => {
                let (operand, next) = match pc {
                    0x2b971 => (0x0600, 0x2b974),
                    0x2b97b => (0x0604, 0x2b97e),
                    0x2b9a3 => (0x0600, 0x2b9a6),
                    0x2b9ad => (0x0604, 0x2b9b0),
                    _ => unreachable!(),
                };
                w.operand_then_internal(2);
                w.write(w.absolute(operand) + u32::from(w.cpu.x), w.cpu.a & mask, m);
                next
            }
            0x2ba27 => {
                let (operand, next) = (0x040e, 0x2ba2a);
                w.drain(16);
                let value = w.read(w.absolute(operand), m);
                w.load(value, m);
                next
            }
            0x2ba2a | 0x2ba2c => {
                let (operand, next) = match pc {
                    0x2ba2a => (0x00aa, 0x2ba2c),
                    0x2ba2c => (0x00a9, 0x2ba2e),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.load((w.cpu.a | value) & mask, m);
                next
            }
            _ => unreachable!("indoor camera continuation {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}
