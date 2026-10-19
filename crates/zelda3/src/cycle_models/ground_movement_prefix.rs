//! Ground movement setup through the first collision call, with live stack and bus work.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroundMovementTail {
    Recoil,
    Swimming,
    Collision { clear_vel_after: bool },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GroundMovementPrefix {
    pc: u16,
    tail: Option<GroundMovementTail>,
    clear_vel_after: bool,
}
impl Default for GroundMovementPrefix {
    fn default() -> Self {
        Self {
            pc: 0x81e2,
            tail: None,
            clear_vel_after: false,
        }
    }
}
impl GroundMovementPrefix {
    pub(crate) fn tail(&self) -> Option<GroundMovementTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        let mode = if (0x9875..=0x9893).contains(&self.pc) {
            0x10
        } else {
            0x30
        };
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (7, self.pc, 0, mode)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x81e2 | 0x81e6 | 0x8200 | 0x8206 | 0x8243 | 0x8249 | 0x824f | 0x8272 | 0x827f
            | 0x828f => {
                let value = w.dp_read(
                    match self.pc {
                        0x81e2 => 0x46,
                        0x81e6 => 0x6b,
                        0x8200 => 0x3a,
                        0x8206 => 0x50,
                        0x8243 => 0x3c,
                        0x8249 => 0x3a,
                        0x824f => 0x3a,
                        0x8272 => 0x49,
                        0x827f => 0xf0,
                        0x828f => 0x48,
                        _ => unreachable!(),
                    },
                    1,
                );
                w.load(value, 1);
                self.pc + 2
            }
            0x820f | 0x8218 | 0x821d | 0x8224 | 0x822b | 0x8232 | 0x8239 | 0x823e | 0x8257
            | 0x8278 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match self.pc {
                        0x820f => 0x377,
                        0x8218 => 0x2e1,
                        0x821d => 0x376,
                        0x8224 => 0x308,
                        0x822b => 0x308,
                        0x8232 => 0x309,
                        0x8239 => 0x301,
                        0x823e => 0x37a,
                        0x8257 => 0x34a,
                        0x8278 => 0x376,
                        _ => unreachable!(),
                    }),
                    1,
                );
                w.load(value, 1);
                self.pc + 3
            }
            0x81ea | 0x81f5 | 0x8214 | 0x828d | 0x82a7 | 0x82a9 | 0x82ab | 0x82ad => {
                w.dp_store(
                    match self.pc {
                        0x81ea => 0x6b,
                        0x81f5 => 0x3b,
                        0x8214 => 0x67,
                        0x828d => 0x2e,
                        0x82a7 => 0x2a,
                        0x82a9 => 0x2b,
                        0x82ab => 0x6b,
                        0x82ad => 0x48,
                        _ => unreachable!(),
                    },
                    0,
                    1,
                );
                if self.pc == 0x8214 {
                    self.clear_vel_after = true;
                }
                self.pc + 2
            }
            0x81ec | 0x81ef | 0x81f2 | 0x81f7 | 0x81fa | 0x81fd => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0x81ec => 0x30d,
                        0x81ef => 0x30e,
                        0x81f2 => 0x30a,
                        0x81f7 => 0x309,
                        0x81fa => 0x308,
                        0x81fd => 0x376,
                        _ => unreachable!(),
                    }),
                    0,
                    1,
                );
                self.pc + 3
            }
            0x820a | 0x8285 | 0x8287 | 0x8289 | 0x828b | 0x8293 | 0x82a1 | 0x82b9 => {
                w.dp_store(
                    match self.pc {
                        0x820a => 0x50,
                        0x8285 => 0x30,
                        0x8287 => 0x31,
                        0x8289 => 0x67,
                        0x828b => 0x26,
                        0x8293 => 0x48,
                        0x82a1 => 0x67,
                        0x82b9 => 0x26,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0x825e | 0x8261 | 0x8266 | 0x8269 => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0x825e => 0x335,
                        0x8261 => 0x337,
                        0x8266 => 0x334,
                        0x8269 => 0x336,
                        _ => unreachable!(),
                    }),
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 3
            }
            0x8202 | 0x8208 | 0x8220 | 0x8227 | 0x822e | 0x8235 | 0x824b | 0x8251 | 0x8274
            | 0x827b | 0x8281 | 0x8291 => {
                w.mask(
                    match self.pc {
                        0x8202 => 0x80,
                        0x8208 => 0xfe,
                        0x8220 => 0xfd,
                        0x8227 => 0x7f,
                        0x822e => 0x80,
                        0x8235 => 0x1,
                        0x824b => 0x20,
                        0x8251 => 0x80,
                        0x8274 => 0xf,
                        0x827b => 0x2,
                        0x8281 => 0xf,
                        0x8291 => 0xf0,
                        _ => unreachable!(),
                    },
                    1,
                );
                self.pc + 2
            }
            0x81e4 | 0x81e8 | 0x8204 | 0x8212 | 0x821b | 0x8222 | 0x8229 | 0x8230 | 0x8237
            | 0x823c | 0x8241 | 0x8247 | 0x824d | 0x8253 | 0x825a | 0x8276 | 0x827d | 0x8283
            | 0x82a5 => {
                let (taken, target) = match self.pc {
                    0x81e4 => (w.cpu.p & ZERO != 0, 0x820f),
                    0x81e8 => (w.cpu.p & ZERO != 0, 0x81ec),
                    0x8204 => (w.cpu.p & ZERO == 0, 0x820c),
                    0x8212 => (w.cpu.p & ZERO != 0, 0x8218),
                    0x821b => (w.cpu.p & ZERO == 0, 0x8255),
                    0x8222 => (w.cpu.p & ZERO == 0, 0x8255),
                    0x8229 => (w.cpu.p & ZERO == 0, 0x8255),
                    0x8230 => (w.cpu.p & ZERO != 0, 0x8239),
                    0x8237 => (w.cpu.p & ZERO == 0, 0x8255),
                    0x823c => (w.cpu.p & ZERO == 0, 0x8255),
                    0x8241 => (w.cpu.p & ZERO == 0, 0x8255),
                    0x8247 => (w.cpu.p & CARRY != 0, 0x8257),
                    0x824d => (w.cpu.p & ZERO == 0, 0x8257),
                    0x8253 => (w.cpu.p & ZERO != 0, 0x8257),
                    0x825a => (w.cpu.p & ZERO != 0, 0x826f),
                    0x8276 => (w.cpu.p & ZERO == 0, 0x82a1),
                    0x827d => (w.cpu.p & ZERO == 0, 0x82bb),
                    0x8283 => (w.cpu.p & ZERO == 0, 0x82a1),
                    0x82a5 => (w.cpu.p & ZERO != 0, 0x82b9),
                    _ => unreachable!(),
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x8216 | 0x8255 | 0x829f => {
                w.branch(true);
                if self.pc == 0x8216 {
                    0x8255
                } else {
                    0x82bb
                }
            }
            0x820c | 0x826c => {
                w.operand_then_internal(2);
                if self.pc == 0x820c {
                    self.tail = Some(GroundMovementTail::Recoil);
                    0x8711
                } else {
                    self.tail = Some(GroundMovementTail::Swimming);
                    0x9715
                }
            }
            0x8245 => {
                w.compare_immediate(9, 1);
                0x8247
            }
            0x825c | 0x8264 => {
                w.drain(8);
                w.load(if self.pc == 0x825c { 1 } else { 0x80 }, 1);
                self.pc + 2
            }
            0x826f | 0x82bb => {
                w.item_call();
                if self.pc == 0x826f {
                    0x9873
                } else {
                    self.tail = Some(GroundMovementTail::Collision {
                        clear_vel_after: self.clear_vel_after,
                    });
                    0xb64f
                }
            }
            0x8295 | 0x829a | 0x82af | 0x82b4 => {
                w.item_x(
                    if matches!(self.pc, 0x8295 | 0x82af) {
                        0x20
                    } else {
                        0x13
                    },
                    1,
                );
                self.pc + 2
            }
            0x8297 | 0x829c | 0x82b1 | 0x82b6 => {
                w.drain(16);
                w.write(
                    w.absolute(if matches!(self.pc, 0x8297 | 0x82b1) {
                        0x371
                    } else {
                        0x375
                    }),
                    w.cpu.x,
                    1,
                );
                self.pc + 3
            }
            0x82a3 => {
                let right = w.dp_read(0x26, 1);
                let left = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | if left >= right { CARRY } else { 0 };
                w.flags(left.wrapping_sub(right) & 255, 1);
                0x82a5
            }
            0x9873 | 0x9893 => {
                w.width(0x20, self.pc == 0x9873);
                self.pc + 2
            }
            0x9875 | 0x9878 | 0x987b | 0x987e | 0x9881 | 0x9884 | 0x9887 | 0x988a | 0x988d
            | 0x9890 => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0x9875 => 0x32f,
                        0x9878 => 0x331,
                        0x987b => 0x326,
                        0x987e => 0x328,
                        0x9881 => 0x32b,
                        0x9884 => 0x32d,
                        0x9887 => 0x33c,
                        0x988a => 0x33e,
                        0x988d => 0x334,
                        0x9890 => 0x336,
                        _ => unreachable!(),
                    }),
                    0,
                    2,
                );
                self.pc + 3
            }
            0x9895 => {
                w.rts();
                assert_eq!(w.cpu.pc, 0x8272);
                w.cpu.pc
            }
            _ => panic!("ground movement prefix PC {:04x}", self.pc),
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
    fn ground_movement_prefix_matches_source_guards_directions_and_reset() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut tails = [false; 4];
        for timer in 0..=255u8 {
            for variant in 0..24u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7)) as u8;
                }
                for a in [
                    0x46, 0x377, 0x2e1, 0x376, 0x308, 0x309, 0x301, 0x37a, 0x3c, 0x3a, 0x34a, 0x49,
                ] {
                    r[a] = 0;
                }
                r[0x26] = timer.rotate_left(2);
                r[0xf0] = timer;
                r[0x6b] = timer;
                r[0x48] = timer;
                r[0x50] = timer;
                match variant {
                    0..=2 => {
                        r[0x46] = 1;
                        r[0x3a] = if variant == 2 { 0x80 } else { 0 };
                        r[0x6b] = if variant == 0 { 0 } else { timer };
                    }
                    3 => r[0x377] = 1,
                    4 => r[0x2e1] = 1,
                    5 => r[0x376] = timer,
                    6 => r[0x308] = timer,
                    7 => {
                        r[0x308] = 0x80;
                        r[0x309] = timer;
                    }
                    8 => r[0x301] = 1,
                    9 => r[0x37a] = 1,
                    10 => {
                        r[0x3c] = timer;
                        r[0x3a] = 0x80;
                    }
                    11 => r[0x3a] = timer,
                    12 => r[0x34a] = 1,
                    13 => r[0x49] = timer,
                    14 => {
                        r[0x376] = 2;
                        r[0x49] = timer;
                    }
                    15 => {
                        r[0x26] = timer & 15;
                    }
                    16 => r[0xf0] = 0,
                    17 => {
                        r[0x49] = timer;
                        r[0x26] = timer & 15;
                    }
                    18..=23 => {
                        r[0x3c] = 9;
                        r[0x3a] = 0x80;
                        r[0x34a] = u8::from(variant & 1 != 0);
                        r[0x49] = timer;
                    }
                    _ => unreachable!(),
                }
                let pc = 0x81e2;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: 8,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1f4,
                    p: 0x30 | (timer & 0xc3),
                };
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
                let mut clock = CpuMasterTimeline::at_raster(
                    4586,
                    CpuRasterPosition::new(
                        if variant & 1 == 0 { 22 } else { 261 },
                        if variant & 1 == 0 { 26 } else { 1350 },
                    ),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(variant & 1 != 0),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = GroundMovementPrefix::default();
                while work.tail().is_none() {
                    let label =
                        format!("movement timer={timer} variant={variant} pc={:04x}", cpu.pc);
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, _| None,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                let (index, end, sp) = match work.tail().unwrap() {
                    GroundMovementTail::Recoil => (0, 0x8711, 0x1f4),
                    GroundMovementTail::Swimming => (1, 0x9715, 0x1f4),
                    GroundMovementTail::Collision {
                        clear_vel_after: false,
                    } => (2, 0xb64f, 0x1f2),
                    GroundMovementTail::Collision {
                        clear_vel_after: true,
                    } => (3, 0xb64f, 0x1f2),
                };
                tails[index] = true;
                assert_eq!((cpu.pc, cpu.sp), (end, sp));
            }
        }
        assert!(tails.iter().all(|seen| *seen));
        println!("ground movement prefix: 6144 cases, {instructions} source instructions, all four tail boundaries");
    }
}
