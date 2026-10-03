//! Shared iris work. Every circle store, counter poll, projection word and
//! return remains on the caller's live CPU clock, including an interrupted
//! divider/multiplier sequence. The bus owns the hardware read state.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    sprite_reset::SpriteReset,
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Spotlight {
    pc: u16,
    calls: Vec<u16>,
    reset: Option<SpriteReset>,
    complete: bool,
}
impl Spotlight {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0xf28b | 0xf295 | 0xf312 | 0xf427));
        Self {
            pc: entry,
            calls: Vec::new(),
            reset: None,
            complete: false,
        }
    }
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
        if let Some(reset) = self.reset.as_mut() {
            let cycles = reset.step(cpu, clock, bus);
            if reset.complete() {
                self.reset = None;
                assert_eq!((cpu.bank, cpu.pc), (0, self.pc));
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (0, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let xm = if x == 1 { 255 } else { 65535 };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xf28b | 0xf295 => {
                w.width(0x10, true);
                pc + 2
            }
            0xf2ec => {
                w.width(0x10, false);
                pc + 2
            }
            0xf315 | 0xf427 | 0xf502 => {
                w.width(0x30, true);
                pc + 2
            }
            0xf3d7 | 0xf4e6 => {
                w.width(0x20, false);
                pc + 2
            }
            0xf3eb | 0xf423 | 0xf448 | 0xf4cc => {
                w.width(0x30, false);
                pc + 2
            }
            0xf4e0 => {
                w.width(0x20, true);
                pc + 2
            }
            0xf28d | 0xf290 | 0xf297 | 0xf29a | 0xf2a6 | 0xf2af | 0xf3b4 | 0xf429 => {
                let v = match pc {
                    0xf290 => 126,
                    0xf297 => 2,
                    0xf2a6 => 0x2641,
                    0xf2af => 0xf2fb,
                    0xf429 => 62,
                    _ => 0,
                };
                w.drain(8 * u32::from(x));
                if matches!(pc, 0xf28d | 0xf297) {
                    w.cpu.y = v
                } else {
                    w.cpu.x = v
                };
                w.flags(v, x);
                pc + 1 + u16::from(x)
            }
            0xf29d | 0xf2a0 => {
                w.drain(16);
                w.write(
                    w.absolute(if pc == 0xf29d { 0x67e } else { 0x67c }),
                    if pc == 0xf29d { w.cpu.y } else { w.cpu.x },
                    x,
                );
                pc + 3
            }
            0xf2a3 | 0xf2a9 | 0xf2ac | 0xf2b2 | 0xf2b5 | 0xf2ba | 0xf2bd | 0xf2c2 | 0xf2c5
            | 0xf3e2 | 0xf4ce | 0xf4d1 | 0xf4d7 | 0xf4ee | 0xf4f4 => {
                let address = match pc {
                    0xf2a3 => 0x420c,
                    0xf2a9 => 0x4360,
                    0xf2ac => 0x4370,
                    0xf2b2 => 0x4362,
                    0xf2b5 => 0x4372,
                    0xf2ba => 0x4364,
                    0xf2bd => 0x4374,
                    0xf2c2 => 0x4367,
                    0xf2c5 => 0x4377,
                    0xf3e2 => 0x2100,
                    0xf4ce => 0x4205,
                    0xf4d1 => 0x4204,
                    0xf4d7 => 0x4206,
                    0xf4ee => 0x4202,
                    _ => 0x4203,
                };
                let (value, width) = match pc {
                    0xf2a3 | 0xf4d1 => (0, m),
                    0xf2a9 | 0xf2ac | 0xf2b2 | 0xf2b5 => (w.cpu.x, x),
                    0xf4ee => (w.cpu.y, x),
                    _ => (w.cpu.a & mask, m),
                };
                assert_eq!(w.cpu.db, 0);
                w.drain(16);
                w.write_io(address, value, width);
                pc + 3
            }
            0xf2b8 | 0xf2c0 | 0xf2c8 | 0xf2cc | 0xf2d0 | 0xf2e0 | 0xf2e4 | 0xf2e8 | 0xf2f2
            | 0xf2f6 | 0xf34e | 0xf361 | 0xf3de | 0xf42c | 0xf521 | 0xf52d | 0xf538 => {
                let v = match pc {
                    0xf2c8 | 0xf2d0 => 0x33,
                    0xf2cc => 3,
                    0xf2e0 => 0x20,
                    0xf2e4 => 0x40,
                    0xf2e8 | 0xf2f2 | 0xf3de => 0x80,
                    0xf2f6 => 15,
                    0xf34e => 224,
                    0xf361 | 0xf521 | 0xf52d | 0xf538 => 255,
                    0xf42c => 0xff00,
                    _ => 0,
                };
                w.immediate(v, m);
                pc + 1 + u16::from(m)
            }
            0xf2ca | 0xf2ce | 0xf2d2 | 0xf2d6 | 0xf2da | 0xf2e2 | 0xf2e6 | 0xf2ea | 0xf2f4
            | 0xf2f8 | 0xf320 | 0xf347 | 0xf351 | 0xf358 | 0xf35f | 0xf364 | 0xf3e0 | 0xf419
            | 0xf500 | 0xf510 | 0xf524 | 0xf53b => {
                let a = match pc {
                    0xf2ca => 0x96,
                    0xf2ce => 0x97,
                    0xf2d2 => 0x98,
                    0xf2d6 => 0x1e,
                    0xf2da => 0x1f,
                    0xf2e2 => 0x9c,
                    0xf2e6 => 0x9d,
                    0xf2ea => 0x9e,
                    0xf2f4 => 0x9b,
                    0xf2f8 | 0xf3e0 => 0x13,
                    0xf320 => 0xe,
                    0xf347 | 0xf351 => 6,
                    0xf358 => 10,
                    0xf35f => 4,
                    0xf364 | 0xf53b => 8,
                    0xf419 => 0x10,
                    0xf510 => 2,
                    _ => 0,
                };
                w.dp_store(a, w.cpu.a & mask, m);
                pc + 2
            }
            0xf2d4 | 0xf2d8 | 0xf2dc | 0xf317 | 0xf329 | 0xf332 | 0xf344 | 0xf353 | 0xf35a
            | 0xf366 | 0xf378 | 0xf381 | 0xf387 | 0xf390 | 0xf396 | 0xf3f1 | 0xf3fb | 0xf506
            | 0xf50a | 0xf526 => {
                let a = match pc {
                    0xf2d4 => 0x1c,
                    0xf2d8 => 0x1d,
                    0xf2dc | 0xf3fb => 0x1b,
                    0xf317 => 0x20,
                    0xf332 => 0x22,
                    0xf329 | 0xf344 | 0xf35a | 0xf396 => 14,
                    0xf353 | 0xf366 | 0xf387 => 6,
                    0xf378 => 4,
                    0xf381 | 0xf390 => 8,
                    0xf3f1 => 0x10,
                    0xf506 => 10,
                    0xf50a => 0,
                    _ => 2,
                };
                let v = w.dp_read(a, m);
                w.load(v, m);
                pc + 2
            }
            0xf293 | 0xf2de | 0xf34c | 0xf36b | 0xf370 | 0xf37e | 0xf38d | 0xf39a | 0xf3b2
            | 0xf3c3 | 0xf3d5 | 0xf3dc | 0xf3e5 | 0xf3f5 | 0xf3f9 | 0xf3fd | 0xf411 | 0xf41d
            | 0xf446 | 0xf508 | 0xf51a | 0xf51f | 0xf52b | 0xf536 => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let n = w.cpu.p & NEGATIVE != 0;
                let (take, to) = match pc {
                    0xf293 => (true, 0xf29d),
                    0xf2de => (!z, 0xf2ec),
                    0xf34c => (c, 0xf353),
                    0xf36b => (c, 0xf378),
                    0xf370 => (z, 0xf375),
                    0xf37e => (c, 0xf387),
                    0xf38d => (c, 0xf396),
                    0xf39a => (z, 0xf3a3),
                    0xf3b2 => (!c, 0xf3a3),
                    0xf3c3 => (!c, 0xf3b7),
                    0xf3d5 => (!z, 0xf423),
                    0xf3dc => (!z, 0xf3e7),
                    0xf3e5 => (true, 0xf3eb),
                    0xf3f5 => (z, 0xf3fb),
                    0xf3f9 => (!z, 0xf416),
                    0xf3fd => (!z, 0xf40c),
                    0xf411 => (z, 0xf416),
                    0xf41d => (!z, 0xf423),
                    0xf446 => (!n, 0xf42f),
                    0xf508 => (z, 0xf53d),
                    0xf51a => (n, 0xf526),
                    0xf51f => (z, 0xf524),
                    0xf52b => (z, 0xf530),
                    _ => (!z, 0xf53b),
                };
                if w.branch(take) {
                    to
                } else {
                    pc + 2
                }
            }
            0xf2ee | 0xf3e7 | 0xf41f => {
                let target = match pc {
                    0xf2ee => 0xf312,
                    0xf3e7 => 0xf427,
                    _ => 0xc44e,
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(pc + 3, 2);
                if pc == 0xf41f {
                    self.reset = Some(SpriteReset::from_entry(0x09c44e));
                    w.cpu.bank = 9;
                    w.cpu.pc = target;
                    pc + 4
                } else {
                    self.calls.push(pc + 4);
                    target
                }
            }
            0xf2fa | 0xf426 | 0xf44a => {
                w.rtl();
                if let Some(returned) = self.calls.pop() {
                    assert_eq!((w.cpu.bank, w.cpu.pc), (0, returned));
                } else {
                    self.complete = true;
                }
                w.cpu.pc
            }
            0xf312 | 0xf313 => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0xf312 { w.cpu.db } else { w.cpu.bank }),
                    1,
                );
                pc + 1
            }
            0xf314 | 0xf425 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xf319 | 0xf322 | 0xf334 | 0xf355 | 0xf35c | 0xf515 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xf31c | 0xf32b | 0xf337 | 0xf3cb | 0xf50c => {
                w.clear_carry();
                pc + 1
            }
            0xf31a | 0xf335 | 0xf356 | 0xf35d | 0xf516 => {
                let v = w.dp_read(
                    match pc {
                        0xf31a => 0xe8,
                        0xf335 => 0xe2,
                        0xf356 => 14,
                        0xf35d => 10,
                        _ => 0,
                    },
                    m,
                );
                w.sbc(v, m);
                pc + 2
            }
            0xf31d | 0xf338 => {
                w.drain(16);
                w.adc(if pc == 0xf31d { 12 } else { 8 });
                pc + 3
            }
            0xf323 | 0xf32c | 0xf50d => {
                w.drain(16);
                let v = w.read(w.absolute(if pc == 0xf50d { 0x670 } else { 0x67c }), m);
                if pc == 0xf323 {
                    w.sbc(v, m)
                } else {
                    w.adc(v)
                };
                pc + 3
            }
            0xf326 | 0xf32f | 0xf33b | 0xf341 | 0xf3cf | 0xf409 | 0xf413 => {
                let a = match pc {
                    0xf326 => 0x674,
                    0xf32f => 0x676,
                    0xf33b => 0x670,
                    0xf341 => 0x67a,
                    0xf3cf => 0x67c,
                    0xf409 => 0x12d,
                    _ => 0x12c,
                };
                w.drain(16);
                w.write(w.absolute(a), w.cpu.a & mask, m);
                pc + 3
            }
            0xf33e | 0xf36d | 0xf3c8 | 0xf3d9 | 0xf40c | 0xf416 | 0xf4d4 | 0xf4f1 | 0xf512 => {
                let a = match pc {
                    0xf36d => 0x67a,
                    0xf3d9 => 0x67e,
                    0xf40c => 0x132,
                    0xf416 => 0x10c,
                    0xf512 => 0x670,
                    _ => 0x67c,
                };
                w.drain(16);
                let v = w.read(w.absolute(a), m);
                w.load(v, m);
                pc + 3
            }
            0xf346 | 0xf37a | 0xf389 => {
                w.shift_left(m);
                pc + 1
            }
            0xf349 | 0xf37b | 0xf38a | 0xf3af | 0xf3f3 | 0xf3f7 | 0xf40f | 0xf41b | 0xf533 => {
                let v = match pc {
                    0xf349 => 224,
                    0xf37b | 0xf38a => 448,
                    0xf3af => 192,
                    0xf3f3 => 7,
                    0xf3f7 => 16,
                    0xf40f => 255,
                    0xf41b => 6,
                    _ => 65535,
                };
                w.compare_immediate(v, m);
                pc + 1 + u16::from(m)
            }
            0xf368 | 0xf398 | 0xf3d2 => {
                let v = if pc == 0xf398 {
                    w.dp_read(4, m)
                } else {
                    w.drain(16);
                    if pc == 0xf3d2 && (x == 2 || 0x0a + w.cpu.x > 255) {
                        w.drain(6);
                    }
                    w.read(
                        w.absolute(if pc == 0xf368 {
                            0x676
                        } else {
                            0xf30a + w.cpu.x
                        }),
                        m,
                    )
                };
                let a = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & mask, m);
                pc + if pc == 0xf398 { 2 } else { 3 }
            }
            0xf372 => {
                w.drain(16);
                let a = w.absolute(0x67a);
                let v = w.read(a, m).wrapping_sub(1) & mask;
                w.drain(6);
                w.flags(v, m);
                w.write(a, v, m);
                pc + 3
            }
            0xf375 => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                0xf4cc
            }
            0xf380 | 0xf38f | 0xf4e8 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & xm;
                w.flags(w.cpu.x, x);
                pc + 1
            }
            0xf383 | 0xf392 => {
                w.drain(24);
                w.write(0x7f7000 + u32::from(w.cpu.x), w.cpu.a & mask, m);
                pc + 4
            }
            0xf39c | 0xf39e => {
                w.dp_increment(if pc == 0xf39c { 4 } else { 6 }, m, pc == 0xf39c);
                pc + 2
            }
            0xf3a0 => {
                w.drain(16);
                0xf361
            }
            0xf3a3 | 0xf3a6 | 0xf3a9 | 0xf4e2 | 0xf4fd => {
                let a = match pc {
                    0xf3a3 => 0x2137,
                    0xf3a6 => 0x213f,
                    0xf3a9 => 0x213d,
                    0xf4e2 => 0x4214,
                    _ => 0x4217,
                };
                assert_eq!(w.cpu.db, 0);
                w.drain(16);
                let v = w.read_io(a, m);
                w.load(v, m);
                pc + 3
            }
            0xf3ac => {
                w.mask(255, m);
                pc + 3
            }
            0xf3b7 | 0xf401 => {
                w.drain(24);
                let v = w.read(
                    if pc == 0xf3b7 { 0x7f7000 } else { 0x7f5b00 } + u32::from(w.cpu.x),
                    m,
                );
                w.load(v, m);
                pc + 4
            }
            0xf3bb | 0xf42f | 0xf432 | 0xf435 | 0xf438 | 0xf43b | 0xf43e | 0xf441 => {
                let base = if pc == 0xf3bb {
                    0x1b00
                } else {
                    0x1b00 + ((pc - 0xf42f) / 3) * 0x40
                };
                w.operand_then_internal(2);
                w.write(w.absolute(base) + u32::from(w.cpu.x), w.cpu.a & mask, m);
                pc + 3
            }
            0xf3be | 0xf3bf => {
                w.increment_x();
                pc + 1
            }
            0xf3c0 => {
                w.compare_x_immediate(448, x);
                pc + 3
            }
            0xf3c5 | 0xf3ff => {
                let v = if pc == 0xf3ff {
                    w.dp_read(0x8a, x)
                } else {
                    w.drain(16);
                    w.read(w.absolute(0x67e), x)
                };
                w.cpu.x = v;
                w.flags(v, x);
                pc + if pc == 0xf3ff { 2 } else { 3 }
            }
            0xf3cc => {
                w.operand_then_internal(2);
                let v = w.read(w.absolute(0xf302) + u32::from(w.cpu.x), m);
                w.adc(v);
                pc + 3
            }
            0xf3ed | 0xf3ef | 0xf4f9 | 0xf4fb | 0xf518 => {
                w.dp_store(
                    match pc {
                        0xf3ed => 0xb0,
                        0xf3ef => 0x11,
                        0xf4f9 => 1,
                        0xf4fb => 11,
                        _ => 0,
                    },
                    0,
                    m,
                );
                pc + 2
            }
            0xf405..=0xf408 | 0xf4e5 => {
                w.drain(6);
                let v = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, m);
                pc + 1
            }
            0xf444 | 0xf445 => {
                w.decrement_x();
                pc + 1
            }
            0xf4da..=0xf4df | 0xf4f7 | 0xf4f8 => {
                w.drain(6);
                pc + 1
            }
            0xf4e9 => {
                w.drain(16);
                if x == 2 || 0x4b + w.cpu.x > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xf44b) + u32::from(w.cpu.x), x);
                w.cpu.y = v;
                w.flags(v, x);
                pc + 3
            }
            0xf4ec => {
                w.dp_store(10, w.cpu.y, x);
                pc + 2
            }
            0xf504 => {
                let v = w.dp_read(0, m);
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | ((v >> 15) as u8);
                let v = v << 1;
                w.flags(v, m);
                w.write(0, v, m);
                pc + 2
            }
            0xf51c | 0xf528 => {
                w.drain(16);
                w.cpu.p = (w.cpu.p & !ZERO) | if w.cpu.a & 0xff00 == 0 { ZERO } else { 0 };
                pc + 3
            }
            0xf530 => {
                w.swap_bytes();
                pc + 1
            }
            0xf531 => {
                let v = w.dp_read(0, m);
                w.load((w.cpu.a | v) & mask, m);
                pc + 2
            }
            0xf53d => {
                w.rts();
                w.cpu.pc
            }
            _ => unreachable!("spotlight continuation {pc:04x}"),
        };
        if self.reset.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        intro_clear::Access,
        native_work::{compare_instruction_with_devices, BeamBus},
    };
    use super::*;
    use snes::apu::{ApuHostPortProbe, ApuHostPortTiming, ApuState};
    fn apu(at: u64) -> ApuHostPortTiming {
        let mut state = ApuState::new();
        state.reset_snes9x_coroutine();
        ApuHostPortTiming::new(
            ApuHostPortProbe::from_snes9x_coroutine(state).unwrap(),
            snes::Snes9xApuClockCheckpoint::new(at, 0, 0).unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn native_spotlight_bus_registers_and_table_progress_match_source() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for range in [0xf2fb..0xf312, 0xf44b..0xf4cc] {
            for a in range {
                assert_eq!(
                    crate::ZeldaState::native_spotlight_table_byte(a),
                    Some(rom[a as usize & 0x7fff])
                );
            }
        }
        let mut instructions = 0;
        let mut waits = 0;
        let mut copies = 0;
        let mut resets = 0;
        for field in [0, 1, 30_002, 30_003] {
            for case in 0..40usize {
                let entry = match case {
                    0 => 0xf295,
                    1 => 0xf28b,
                    2 => 0xf427,
                    _ => 0xf312,
                };
                // Device-capable isolated source fixture. This is not a cold
                // route claim: caller registers/RAM and beam are test inputs.
                let mut machine = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom)
                    .unwrap()
                    .machine()
                    .snes()
                    .clone();
                machine.ram.fill(0);
                machine.write(0x2100, 0x80);
                machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 2]);
                for (a, v) in [
                    (0x20, if case % 4 == 0 { 24u16 } else { 80 }),
                    (
                        0x22,
                        if case % 4 == 1 {
                            0xfff0
                        } else if case % 4 == 2 {
                            280
                        } else {
                            64
                        },
                    ),
                    (0x67c, ((case % 19) * 7) as u16),
                    (0x67e, if case < 22 { 0 } else { 2 }),
                    (0x132, if case % 2 == 0 { 0xff } else { 0x11 }),
                    (0x10c, if case % 5 == 0 { 6 } else { 7 }),
                ] {
                    machine.ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
                }
                machine.ram[0x10] = if case % 3 == 0 { 16 } else { 7 };
                machine.ram[0x1b] = (case & 1) as u8;
                machine.ram[0x1c] = 0x17;
                machine.ram[0x1d] = 2;
                machine.ram[0x8a] = 0x22;
                machine.ram[0x15b22] = 0x31;
                let mut cpu = NativeNmiContext {
                    a: 0xab01,
                    x: 0xe0,
                    y: 7,
                    d: 0,
                    db: 0,
                    bank: 0,
                    pc: entry,
                    sp: 0x1fc,
                    p: 0x31,
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
                let mut devices = snes::Snes::new();
                devices.write(0x2100, 0x80);
                // Both fixture ports begin with zeroed OAM bytes. The
                // renderer's ordinary reset uses hidden-sprite sentinels.
                devices.ppu.oam.fill(0);
                let mut reads = snes::SourcePpuReadState::snes9x_reset();
                let beam = std::cell::Cell::new(None);
                let timeline = CpuMasterTimeline::at_raster(
                    field,
                    snes::CpuRasterPosition::new(
                        if case % 2 == 0 { 224 } else { 0 },
                        if case % 2 == 0 { 1320 } else { 520 },
                    ),
                    snes::CpuBusWorkload::default(),
                    snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                );
                let mut source = snes::RomCpuTimingProbe::new(machine, timeline, reads).unwrap();
                let mut clock = source.timeline().clone();
                let mut ports = apu(clock.clock_master_cycles());
                source
                    .attach_apu_port_owner(apu(clock.clock_master_cycles()))
                    .unwrap();
                let mut work = Spotlight::new(entry);
                for _ in 0..200_000 {
                    if work.complete() {
                        break;
                    }
                    waits += usize::from(cpu.pc == 0xf3a3);
                    copies += usize::from(cpu.pc == 0xf3bb);
                    resets += usize::from(cpu.pc == 0xf42f);
                    let label = format!(
                        "field={field} case={case} pc={:02x}:{:04x}",
                        cpu.bank, cpu.pc
                    );
                    compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |at, access| match access {
                            Access::Read(a, width) if (0x2100..0x4400).contains(&a) => {
                                assert_eq!(width, 1);
                                let v = if a == 0x2140 {
                                    ports.read_cpu_port_at(at, 0).unwrap()
                                } else if let Some(v) = reads.read(a as u16, beam.get().unwrap()) {
                                    v
                                } else {
                                    let v = devices.read(a);
                                    if matches!(a, 0x2138 | 0x213e) {
                                        reads.open_bus1 = v;
                                    }
                                    v
                                };
                                Some(u16::from(v))
                            }
                            Access::Write(a, v, width) if (0x2100..0x4400).contains(&a) => {
                                assert_eq!(width, 1);
                                devices.write(a, v as u8);
                                Some(0)
                            }
                            _ => None,
                        },
                        |cpu, clock, bus| {
                            work.step(
                                cpu,
                                clock,
                                BeamBus::new(|at, access, current| {
                                    beam.set(current);
                                    bus(at, access)
                                }),
                            )
                            .unwrap();
                        },
                    );
                    instructions += 1;
                }
                assert!(work.complete(), "{field}/{case} stopped {:04x}", cpu.pc);
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x805a, 0x1ff));
                assert_eq!(ram, source.snes().ram, "{field}/{case}");
                assert_eq!(
                    reads,
                    *source.ppu_reads(),
                    "PPU counter state {field}/{case}"
                );
                assert_eq!(devices.ppu.forced_blank, source.snes().ppu.forced_blank);
                assert_eq!(
                    bincode::serialize(&devices.dma).unwrap(),
                    bincode::serialize(&source.snes().dma).unwrap()
                );
            }
        }
        assert!(waits > 160 && copies > 0 && resets > 0);
        eprintln!("160 spotlight fixtures, {instructions} instruction boundaries, {waits} counter polls, {copies} projection words, {resets} reset iterations");
    }
}
