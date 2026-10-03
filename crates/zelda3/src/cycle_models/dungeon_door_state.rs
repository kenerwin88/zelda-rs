//! Shared door records and final-type decisions, including the caller's carry result.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonDoorState {
    pc: u16,
    complete: bool,
}
impl Default for DungeonDoorState {
    fn default() -> Self {
        Self {
            pc: 0xb0da,
            complete: false,
        }
    }
}
impl DungeonDoorState {
    pub(crate) fn complete(&self) -> bool {
        self.complete
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
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (1, self.pc, 0, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xb0da | 0xb0de | 0xb0e6 => {
                let base: u16 = match self.pc {
                    0xb0da => 0x19c0,
                    0xb0de => 0x19a0,
                    _ => 0x1980,
                };
                w.operand_then_internal(2);
                w.write(u32::from(base.wrapping_add(w.cpu.x)), w.cpu.a, 2);
                self.pc + 3
            }
            0xb0dd | 0xb0e1 | 0xb0e9 | 0xb162 => {
                w.drain(6);
                w.load(if self.pc == 0xb0dd { w.cpu.y } else { w.cpu.x }, 2);
                self.pc + 1
            }
            0xb0e2 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a as u8 & CARRY);
                w.load(w.cpu.a >> 1, 2);
                0xb0e3
            }
            0xb0e3 | 0xb163 => {
                w.swap_bytes();
                self.pc + 1
            }
            0xb0e4 => {
                let v = w.dp_read(4, 2);
                w.load(w.cpu.a | v, 2);
                0xb0e6
            }
            0xb0ea | 0xb0fe | 0xb11c | 0xb15b | 0xb169 | 0xb173 => {
                w.mask(
                    match self.pc {
                        0xb0ea => 15,
                        0xb169 => 3,
                        _ => 255,
                    },
                    2,
                );
                self.pc + 3
            }
            0xb0ed => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                0xb0ee
            }
            0xb0ee | 0xb148 | 0xb14d => {
                let v = if self.pc == 0xb148 { 0x32 } else { 8 };
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v), 2);
                self.pc + 3
            }
            0xb0f1 | 0xb0f9 | 0xb104 | 0xb109 | 0xb10e | 0xb122 | 0xb127 | 0xb12c | 0xb131
            | 0xb136 | 0xb14b | 0xb150 | 0xb157 | 0xb178 | 0xb17f => {
                let taken = match self.pc {
                    0xb0f1 => w.cpu.p & CARRY != 0,
                    0xb12c => w.cpu.p & CARRY == 0,
                    0xb109 | 0xb10e | 0xb157 | 0xb178 => w.cpu.p & ZERO == 0,
                    _ => w.cpu.p & ZERO != 0,
                };
                let target = match self.pc {
                    0xb104 => 0xb10b,
                    0xb109 => 0xb110,
                    0xb14b | 0xb150 | 0xb17f => 0xb18f,
                    0xb157 => 0xb18d,
                    0xb178 => 0xb181,
                    _ => 0xb141,
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xb0f3 | 0xb10b | 0xb138 | 0xb181 => {
                w.drain(16);
                let address = match self.pc {
                    0xb10b => 0x0468,
                    0xb138 => 0x0400,
                    _ => 0x068c,
                };
                let v = w.read(address, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xb0f6 | 0xb13b | 0xb184 => {
                w.operand_then_internal(2);
                let index = if self.pc == 0xb0f6 { w.cpu.y } else { w.cpu.x };
                let v = w.read(u32::from(0x98c0u16.wrapping_add(index)), 2);
                w.load(
                    if self.pc == 0xb0f6 {
                        w.cpu.a & v
                    } else {
                        w.cpu.a | v
                    },
                    2,
                );
                self.pc + 3
            }
            0xb0fb | 0xb113 | 0xb119 => {
                w.operand_then_internal(2);
                let base: u16 = if self.pc == 0xb113 { 0x9a02 } else { 0x1980 };
                let v = w.read(u32::from(base.wrapping_add(w.cpu.x)), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xb101 | 0xb106 | 0xb11f | 0xb124 | 0xb129 | 0xb12e | 0xb133 | 0xb154 => {
                w.compare_immediate(
                    match self.pc {
                        0xb101 | 0xb11f => 0x18,
                        0xb106 | 0xb124 => 0x44,
                        0xb129 | 0xb154 => 0x1a,
                        0xb12e => 0x40,
                        _ => 0x46,
                    },
                    2,
                );
                self.pc + 3
            }
            0xb110 => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                0xb111
            }
            0xb111 => {
                w.cpu.x = w.dp_read(4, 2);
                w.flags(w.cpu.x, 2);
                0xb113
            }
            0xb116 | 0xb15e => {
                w.dp_store(0x0a, w.cpu.a, 2);
                self.pc + 2
            }
            0xb118 => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                0xb119
            }
            0xb13e | 0xb164 | 0xb170 | 0xb187 => {
                w.drain(16);
                w.write(
                    match self.pc {
                        0xb13e => 0x0400,
                        0xb187 => 0x068c,
                        _ => 0x0436,
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 3
            }
            0xb141 => {
                w.cpu.y = w.dp_read(0x0a, 2);
                w.flags(w.cpu.y, 2);
                0xb143
            }
            0xb143 | 0xb144 | 0xb160 | 0xb161 => {
                w.drain(6);
                w.cpu.x = if self.pc <= 0xb144 {
                    w.cpu.x.wrapping_add(1)
                } else {
                    w.cpu.x.wrapping_sub(1)
                };
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xb145 => {
                w.drain(16);
                w.write(0x0460, w.cpu.x, 2);
                0xb148
            }
            0xb152 | 0xb159 | 0xb167 => {
                let v = w.dp_read(
                    match self.pc {
                        0xb152 => 4,
                        0xb159 => 0x2f,
                        _ => 0,
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0xb16c => {
                w.shift_left(2);
                0xb16d
            }
            0xb16d => {
                w.drain(16);
                let v = w.read(0x0436, 2);
                w.load(w.cpu.a | v, 2);
                0xb170
            }
            0xb176 | 0xb17d => {
                let v = w.dp_read(0x0a, 2);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a >= v);
                w.flags(w.cpu.a.wrapping_sub(v), 2);
                self.pc + 2
            }
            0xb17a => {
                w.drain(16);
                w.load(w.cpu.a ^ 2, 2);
                0xb17d
            }
            0xb18a => {
                w.drain(16);
                w.cpu.y = 0;
                w.flags(0, 2);
                0xb18d
            }
            0xb18d => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb18e
            }
            0xb18f => {
                w.clear_carry();
                0xb190
            }
            0xb18e | 0xb190 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("door state continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn dungeon_door_state_matches_source_records_remaps_invisible_direction_and_return_carry() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut state = crate::ZeldaState::new();
        state
            .set_assets(
                &std::fs::read(
                    std::env::var_os("ZELDA3_ASSETS").expect("door tests require native assets"),
                )
                .unwrap(),
            )
            .unwrap();
        let data = state.native_dungeon_object_data().unwrap();
        let mut visited = std::collections::BTreeSet::new();
        let mut cases = 0;
        let mut total = 0;
        let mut carry_results = [0usize; 2];
        for kind in (0..80u16).step_by(2) {
            for slot in 0..16u16 {
                for variant in 0..8 {
                    for direction in 0..4u16 {
                        let facings: &[u8] = if kind == 0x1a {
                            &[0, 2, 4, 6, 0xff]
                        } else {
                            &[2]
                        };
                        for &facing in facings {
                            let mut machine = Snes::new();
                            machine.cart.load(CartType::LoRom, &rom, 0x2000);
                            for (i, b) in machine.ram.iter_mut().enumerate() {
                                *b = (i.wrapping_mul(7) ^ (i >> 8)) as u8;
                            }
                            let x = slot * 2;
                            let mask = 0x8000u16 >> (slot & 7);
                            let opened = match variant & 3 {
                                0 => 0,
                                1 => mask,
                                2 => !mask,
                                _ => 0xffff,
                            };
                            for (a, value) in [
                                (4, kind),
                                (0x0a, kind),
                                (0, (kind << 8) | direction),
                                (0x0460, x),
                                (0x068c, opened),
                                (0x0400, if variant & 1 == 0 { 0 } else { 0x5aa5 }),
                                (0x0468, if variant & 4 == 0 { 0 } else { 0x0100 }),
                            ] {
                                machine.ram[a..a + 2].copy_from_slice(&value.to_le_bytes());
                            }
                            machine.ram[0x2f] = facing;
                            machine.ram[0x30] = if variant & 1 == 0 { 0 } else { 0xff };
                            machine.ram[0x1fc..0x1fe].copy_from_slice(&[0xec, 0xab]);
                            let mut cpu = NativeNmiContext {
                                a: direction,
                                x,
                                y: [
                                    0x0000, 0x07be, 0x0ffe, 0x1800, 0x1ffe, 0x2000, 0x3ffe, 0xffff,
                                ][variant],
                                d: 0,
                                db: 0,
                                bank: 1,
                                pc: 0xb0da,
                                sp: 0x1fb,
                                p: if variant & 1 == 0 { 0x41 } else { 0x84 },
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
                            let (v, h) = if variant & 1 == 0 {
                                (224, 1338)
                            } else {
                                (261, 1350)
                            };
                            let mut clock = CpuMasterTimeline::at_raster(
                                0,
                                CpuRasterPosition::new(v, h),
                                CpuBusWorkload::default(),
                                CpuFieldTiming::non_interlace(v == 261),
                            );
                            let mut source = RomCpuTimingProbe::new(
                                machine,
                                clock.clone(),
                                SourcePpuReadState::snes9x_reset(),
                            )
                            .unwrap();
                            clock.begin_synchronous_timeline().unwrap();
                            let mut work = DungeonDoorState::default();
                            let mut instructions = 0;
                            while !work.complete() {
                                visited.insert(cpu.pc);
                                let label=format!("door state type{kind:02x} slot{slot} direction{direction} facing{facing} variant{variant} pc{:04x}",cpu.pc);
                                super::super::native_work::compare_instruction_with_devices(
                                    &label,
                                    &mut cpu,
                                    &mut clock,
                                    &mut source,
                                    &mut ram,
                                    &rom,
                                    None,
                                    |_, access| match access {
                                        Access::Read(a, width)
                                            if a & 0x8000 != 0 && a < 0x7e0000 =>
                                        {
                                            let mut value = 0;
                                            for b in 0..width {
                                                value |= u16::from(
                                                    *data
                                                        .get(&(a + u32::from(b)))
                                                        .expect("bound native door data"),
                                                ) << (8 * b);
                                            }
                                            Some(value)
                                        }
                                        _ => None,
                                    },
                                    |cpu, clock, bus| {
                                        work.step(cpu, clock, bus).unwrap();
                                    },
                                );
                                assert_eq!(ram, source.snes().ram, "{label}");
                                work = work.clone();
                                instructions += 1;
                                assert!(instructions < 100);
                            }
                            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (1, 0xabed, 0x1fd));
                            assert_eq!(
                                work.step(&mut cpu, &mut clock, |_, _| panic!(
                                    "completed door state touched bus"
                                )),
                                None
                            );
                            carry_results[usize::from(cpu.p & CARRY != 0)] += 1;
                            total += instructions;
                            cases += 1;
                        }
                    }
                }
            }
        }
        for pc in [
            0xb0f3, 0xb10b, 0xb110, 0xb138, 0xb141, 0xb159, 0xb17a, 0xb181, 0xb18e, 0xb190,
        ] {
            assert!(visited.contains(&pc), "missing door state branch {pc:04x}");
        }
        assert!(carry_results.iter().all(|&n| n > 0));
        eprintln!("{cases} door state fixtures matched {total} instructions and {cases} native returns; carry clear/set {carry_results:?}");
    }
}
