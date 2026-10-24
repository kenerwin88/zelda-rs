//! PreOverworld_LoadProperties retires its own instructions and hands each
//! callee the live CPU, stack and hardware clock. LoadOWMusicIfNeeded is a
//! tail call: its RTS consumes the properties caller's original near frame.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PropertiesChild {
    BunnyStatus,
    DungeonExit,
    SpecialExit,
    SongList,
    HudRefill,
    AnimatedTiles,
    Tilesets,
    ScreenPalette,
    OverworldPalettes,
    BackgroundColor,
    PaletteCache,
    FixedColorAndScroll,
    Follower,
    PlayerGraphics,
    SpriteReload,
    MirrorPortal,
    ResetPlayer,
    BunnyPalettes,
    SongBankTail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Call {
    child: PropertiesChild,
    returned: u16,
    sp: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreOverworldProperties {
    pc: u16,
    call: Option<Call>,
    caller_return_pc: u16,
    caller_sp: u16,
    complete: bool,
}

impl PreOverworldProperties {
    /// The enclosing native JSR supplies its successor and pre-call stack.
    pub(crate) fn new(caller_return_pc: u16, caller_sp: u16) -> Self {
        Self {
            pc: 0x83c7,
            call: None,
            caller_return_pc,
            caller_sp,
            complete: false,
        }
    }
    pub(crate) fn child(&self) -> Option<PropertiesChild> {
        self.call.map(|call| call.child)
    }
    #[cfg(test)]
    #[cfg(feature = "rom-timing")]
    pub(crate) fn at_instruction_for_test(pc: u16) -> Self {
        Self { pc, ..Self::new(0x83c6, 0x1fc) }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let call = self
            .call
            .expect("properties caller has no outstanding call");
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.sp),
            (2, call.returned, call.sp),
            "properties callee must consume its real stack frame"
        );
        self.call = None;
        self.pc = call.returned;
        self.complete = call.child == PropertiesChild::SongBankTail;
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete || self.call.is_some() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (2, self.pc, 0, 0x30)
        );
        if self.pc == 0x83c7 {
            assert_eq!(
                cpu.sp,
                self.caller_sp.wrapping_sub(2),
                "properties near frame"
            );
        }
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0x83c7 | 0x83e4 | 0x84bf | 0x84cf | 0x84d5 | 0x84ff | 0x8521 | 0x8528 | 0x8530
            | 0x8534 => {
                w.immediate(
                    match pc {
                        0x83c7 => 0x82,
                        0x83e4 => 0xff,
                        0x84cf => 0x1e,
                        0x84d5 | 0x8530 => 9,
                        0x8521 => 1,
                        0x8528 => 0x17,
                        _ => 0,
                    },
                    1,
                );
                pc + 2
            }
            0x83ee | 0x8410 => {
                w.drain(8);
                w.cpu.y = if pc == 0x83ee { 0x58 } else { 0x5a };
                w.flags(w.cpu.y, 1);
                pc + 2
            }
            0x83f0 | 0x8400 | 0x8418 | 0x8422 | 0x843e | 0x8448 | 0x8452 | 0x845b | 0x8465
            | 0x846d | 0x8481 | 0x8489 | 0x84e8 | 0x84f2 => {
                w.drain(8);
                w.cpu.x = match pc {
                    0x8400 | 0x8481 => 9,
                    0x8418 => 7,
                    0x843e | 0x84e8 => 5,
                    0x8452 => 0xf3,
                    0x8465 => 3,
                    0x846d => 0x0d,
                    0x8489 => 4,
                    0x84f2 => 1,
                    _ => 2,
                };
                w.flags(w.cpu.x, 1);
                pc + 2
            }
            0x83d2 | 0x83f2 | 0x8402 | 0x8412 | 0x8424 | 0x8432 | 0x8438 | 0x844a | 0x846f
            | 0x84ad | 0x84c9 | 0x84de => {
                let address = match pc {
                    0x83d2 | 0x84ad => 0x10,
                    0x8424 | 0x8432 | 0x844a => 0xa0,
                    _ => 0x8a,
                };
                let value = w.dp_read(address, 1);
                w.load(value, 1);
                pc + 2
            }
            0x841a | 0x8440 | 0x845d | 0x8467 | 0x8483 | 0x849b | 0x84a1 | 0x84ea | 0x84f7
            | 0x8515 | 0x851b => {
                w.drain(24);
                let address = match pc {
                    0x841a | 0x845d | 0x84ea => 0x7ef3c5,
                    0x8440 => 0x7ef300,
                    0x8467 | 0x851b => 0x7ef3ca,
                    0x8483 | 0x8515 => 0x7ef357,
                    0x849b => 0x7efd40 + u32::from(w.cpu.x),
                    0x84a1 => 0x00fd1c + u32::from(w.cpu.x),
                    0x84f7 => 0x7ef3cc,
                    _ => unreachable!(),
                };
                let value = w.read(address, 1);
                w.load(value, 1);
                pc + 4
            }
            0x8454 => {
                w.drain(16);
                let value = w.read(w.absolute(0x0132), 1);
                w.load(value, 1);
                pc + 3
            }
            0x8499 => {
                let value = w.dp_read(0x8a, 1);
                w.cpu.x = value;
                w.flags(value, 1);
                pc + 2
            }
            0x83d4 | 0x83f4 | 0x83f8 | 0x83fc | 0x8404 | 0x8408 | 0x840c | 0x8414 | 0x841e
            | 0x8426 | 0x842a | 0x842e | 0x8434 | 0x843a | 0x844e | 0x8457 | 0x8461 | 0x8471
            | 0x8475 | 0x8479 | 0x847d | 0x84af | 0x84ee | 0x84fb => {
                w.compare_immediate(
                    match pc {
                        0x83d4 | 0x84af => 8,
                        0x83f4 | 0x841e => 3,
                        0x83f8 => 5,
                        0x83fc => 7,
                        0x8404 | 0x8475 => 0x43,
                        0x8408 | 0x8479 => 0x45,
                        0x840c | 0x847d => 0x47,
                        0x8414 | 0x8471 => 0x40,
                        0x8426 => 0xe3,
                        0x842a | 0x843a => 0x18,
                        0x842e => 0x2f,
                        0x8434 => 0x1f,
                        0x844e => 0xe1,
                        0x8457 => 0xf2,
                        0x8461 | 0x84ee => 2,
                        0x84fb => 6,
                        _ => unreachable!(),
                    },
                    1,
                );
                pc + 2
            }
            0x8444 | 0x84cb | 0x84e0 => {
                w.mask(if pc == 0x84cb { 0x3f } else { 0x40 }, 1);
                pc + 2
            }
            0x83d6 | 0x8436 | 0x8487 | 0x84b1 | 0x84cd | 0x84e2 | 0x84fd | 0x8519 => {
                let target = match pc {
                    0x83d6 => 0x83dd,
                    0x8436 => 0x843e,
                    0x8487 => 0x848b,
                    0x84b1 => 0x84b8,
                    0x84cd => 0x84d5,
                    0x84e2 => 0x84e8,
                    0x84fd => 0x8505,
                    0x8519 => 0x8530,
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO == 0) {
                    target
                } else {
                    pc + 2
                }
            }
            0x83f6 | 0x83fa | 0x83fe | 0x8406 | 0x840a | 0x840e | 0x8428 | 0x842c | 0x8430
            | 0x843c | 0x8446 | 0x844c | 0x8450 | 0x8459 | 0x846b | 0x8473 | 0x8477 | 0x847b
            | 0x847f | 0x851f => {
                let target = match pc {
                    0x8446 => 0x844a,
                    0x8459 | 0x846b => 0x848b,
                    0x8473 | 0x8477 | 0x847b | 0x847f => 0x8483,
                    0x851f => 0x8530,
                    _ => 0x8467,
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    target
                } else {
                    pc + 2
                }
            }
            0x8416 | 0x8463 | 0x84f0 | 0x8420 => {
                let taken = if pc == 0x8420 {
                    w.cpu.p & CARRY == 0
                } else {
                    w.cpu.p & CARRY != 0
                };
                let target = match pc {
                    0x8416 => 0x8452,
                    0x8420 => 0x8424,
                    0x8463 => 0x8467,
                    0x84f0 => 0x84f4,
                    _ => unreachable!(),
                };
                if w.branch(taken) {
                    target
                } else {
                    pc + 2
                }
            }
            0x83db | 0x84b6 => {
                w.branch(true);
                if pc == 0x83db {
                    0x83e0
                } else {
                    0x84bb
                }
            }
            0x83c9 | 0x849f | 0x8505 | 0x8507 | 0x8509 | 0x850b | 0x850d | 0x8526 | 0x852a
            | 0x8532 | 0x853d => {
                let address = match pc {
                    0x83c9 => 0x99,
                    0x849f => 0,
                    0x8505 => 0x6c,
                    0x8507 => 0x3a,
                    0x8509 => 0x3c,
                    0x850b => 0x50,
                    0x850d => 0x5e,
                    0x8526 => 0x56,
                    0x852a => 0x5d,
                    0x8532 => 0x94,
                    0x853d => 0xee,
                    _ => unreachable!(),
                };
                let value = if matches!(pc, 0x83c9 | 0x849f | 0x8526 | 0x852a | 0x8532) {
                    w.cpu.a & 255
                } else {
                    0
                };
                w.dp_store(address, value, 1);
                pc + 2
            }
            0x83cb | 0x848b | 0x84d7 | 0x84f4 | 0x850f | 0x8523 | 0x853a | 0x853f | 0x8546
            | 0x8549 => {
                let address = match pc {
                    0x83cb => 0x03f4,
                    0x848b => 0x0132,
                    0x84d7 => 0x010c,
                    0x84f4 => 0x012d,
                    0x850f => 0x0351,
                    0x8523 => 0x02e0,
                    0x853a => 0x046c,
                    0x853f => 0x0476,
                    0x8546 => 0x0402,
                    0x8549 => 0x0403,
                    _ => unreachable!(),
                };
                let value = match pc {
                    0x848b | 0x84f4 => w.cpu.x,
                    0x84d7 | 0x8523 => w.cpu.a & 255,
                    _ => 0,
                };
                w.drain(16);
                w.write(w.absolute(address), value, 1);
                pc + 3
            }
            0x83e6 | 0x84c1 | 0x8501 | 0x8536 => {
                w.drain(24);
                w.write(
                    match pc {
                        0x83e6 => 0x7ef36f,
                        0x84c1 => 0x7ec017,
                        0x8501 => 0x7ef3cc,
                        0x8536 => 0x7ec005,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 255,
                    1,
                );
                pc + 4
            }
            0x8542 | 0x8544 => {
                w.dp_increment(if pc == 0x8542 { 0x11 } else { 0x16 }, 1, true);
                pc + 2
            }
            0x83ce | 0x83d8 | 0x83dd | 0x83e0 | 0x83ea | 0x848e | 0x8492 | 0x8496 | 0x84a5
            | 0x84a9 | 0x84b3 | 0x84b8 | 0x84bb | 0x84c5 | 0x84d1 | 0x84da | 0x84e4 | 0x8512
            | 0x852c => {
                use PropertiesChild::*;
                let (child, entry, far) = match pc {
                    0x83ce => (BunnyStatus, 0x02856a, true),
                    0x83d8 => (DungeonExit, 0x02e4a3, false),
                    0x83dd => (SpecialExit, 0x02e9bc, false),
                    0x83e0 => (SongList, 0x02c463, true),
                    0x83ea => (HudRefill, 0x0ddb75, true),
                    0x848e => (AnimatedTiles, 0x00d394, true),
                    0x8492 => (Tilesets, 0x00e19b, true),
                    0x8496 => (ScreenPalette, 0x02c692, false),
                    0x84a5 => (OverworldPalettes, 0x0ed5a8, true),
                    0x84a9 => (BackgroundColor, 0x0ed618, true),
                    0x84b3 => (PaletteCache, 0x02c65f, false),
                    0x84b8 => (PaletteCache, 0x02c6eb, false),
                    0x84bb => (FixedColorAndScroll, 0x0bfe70, true),
                    0x84c5 => (Follower, 0x099efc, true),
                    0x84d1 => (PlayerGraphics, 0x00d4ed, true),
                    0x84da => (SpriteReload, 0x09c499, true),
                    0x84e4 => (MirrorPortal, 0x09af89, true),
                    0x8512 => (ResetPlayer, 0x028b0c, false),
                    0x852c => (BunnyPalettes, 0x0ed6dd, true),
                    _ => unreachable!(),
                };
                let successor = pc + if far { 4 } else { 3 };
                self.call = Some(Call {
                    child,
                    returned: successor,
                    sp: w.cpu.sp,
                });
                w.operand_then_internal(if far { 3 } else { 2 });
                if far {
                    w.push(u16::from(w.cpu.bank), 1);
                }
                w.push(successor - 1, 2);
                w.cpu.bank = (entry >> 16) as u8;
                w.cpu.pc = entry as u16;
                successor
            }
            _ => unreachable!("properties PC {pc:04x}"),
        };
        if self.call.is_none() {
            w.cpu.pc = self.pc;
            if self.pc == 0x854c {
                self.call = Some(Call {
                    child: PropertiesChild::SongBankTail,
                    returned: self.caller_return_pc,
                    sp: self.caller_sp,
                });
            }
        }
        Some(w.cycles)
    }
}

/// AdjustLinkBunnyStatus, including its real far return to properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BunnyStatus {
    pc: u16,
    complete: bool,
}
impl Default for BunnyStatus {
    fn default() -> Self {
        Self {
            pc: 0x856a,
            complete: false,
        }
    }
}
impl BunnyStatus {
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
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (2, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0x856a => {
                w.drain(24);
                let value = w.read(0x7ef357, 1);
                w.load(value, 1);
                pc + 4
            }
            0x856e => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x8582
                } else {
                    pc + 2
                }
            }
            0x8570 => {
                w.immediate(0, 1);
                pc + 2
            }
            0x8572 | 0x857d => {
                w.dp_store(if pc == 0x8572 { 0x5d } else { 0x56 }, 0, 1);
                pc + 2
            }
            0x8574 | 0x8577 | 0x857a | 0x857f => {
                w.drain(16);
                w.write(
                    w.absolute(match pc {
                        0x8574 => 0x03f5,
                        0x8577 => 0x03f6,
                        0x857a => 0x03f7,
                        0x857f => 0x02e0,
                        _ => unreachable!(),
                    }),
                    0,
                    1,
                );
                pc + 3
            }
            0x8582 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("bunny status PC {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::super::native_work::compare_instruction;
    use super::*;

    fn fixture(
        rom: &[u8],
        pc: u16,
        case: usize,
        position: snes::CpuRasterPosition,
    ) -> (
        NativeNmiContext,
        CpuMasterTimeline,
        snes::RomCpuTimingProbe,
        Vec<u8>,
    ) {
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, rom, 0x2000);
        for (i, value) in machine.ram.iter_mut().enumerate() {
            *value = (i.wrapping_mul(37) ^ (i >> 8) ^ case.wrapping_mul(19)) as u8;
        }
        machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
        machine.ram[0x136] = 0;
        machine.ram[0xf357] = [0, 1, 255][case % 3];
        let low = [
            0, 1, 2, 3, 5, 6, 7, 8, 0x18, 0x1f, 0x2f, 0x40, 0x43, 0x45, 0x47, 0xe1, 0xe3, 0xf2,
            0xff,
        ][case % 19];
        let cpu = NativeNmiContext {
            a: 0xb700 | low,
            x: [0, 1, 63, 127, 255][case % 5],
            y: [0, 0x58, 0x5a, 255][case % 4],
            d: 0,
            db: [0, 2, 0x80, 0x82][case % 4],
            bank: 2,
            pc,
            sp: 0x1fa,
            p: 0x30 | [0, 1, 2, 0x43, 0x80, 0xc3][case % 6],
        };
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.dp = cpu.d;
        machine.cpu.db = cpu.db;
        machine.cpu.k = cpu.bank;
        machine.cpu.pc = cpu.pc;
        machine.cpu.sp = cpu.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(cpu.p);
        let ram = machine.ram.clone();
        let clock = CpuMasterTimeline::at_raster(
            case as u64,
            position,
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let source = snes::RomCpuTimingProbe::new(
            machine,
            clock.clone(),
            snes::SourcePpuReadState::snes9x_reset(),
        )
        .unwrap();
        (cpu, source.timeline().clone(), source, ram)
    }

    #[test]
    fn properties_instructions_and_child_frames_match_source() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        // Every instruction boundary in the pristine cartridge's caller.
        // Callee bodies are separate owners and are not executed by this test.
        let pcs = [
            0x83c7, 0x83c9, 0x83cb, 0x83ce, 0x83d2, 0x83d4, 0x83d6, 0x83d8, 0x83db, 0x83dd, 0x83e0,
            0x83e4, 0x83e6, 0x83ea, 0x83ee, 0x83f0, 0x83f2, 0x83f4, 0x83f6, 0x83f8, 0x83fa, 0x83fc,
            0x83fe, 0x8400, 0x8402, 0x8404, 0x8406, 0x8408, 0x840a, 0x840c, 0x840e, 0x8410, 0x8412,
            0x8414, 0x8416, 0x8418, 0x841a, 0x841e, 0x8420, 0x8422, 0x8424, 0x8426, 0x8428, 0x842a,
            0x842c, 0x842e, 0x8430, 0x8432, 0x8434, 0x8436, 0x8438, 0x843a, 0x843c, 0x843e, 0x8440,
            0x8444, 0x8446, 0x8448, 0x844a, 0x844c, 0x844e, 0x8450, 0x8452, 0x8454, 0x8457, 0x8459,
            0x845b, 0x845d, 0x8461, 0x8463, 0x8465, 0x8467, 0x846b, 0x846d, 0x846f, 0x8471, 0x8473,
            0x8475, 0x8477, 0x8479, 0x847b, 0x847d, 0x847f, 0x8481, 0x8483, 0x8487, 0x8489, 0x848b,
            0x848e, 0x8492, 0x8496, 0x8499, 0x849b, 0x849f, 0x84a1, 0x84a5, 0x84a9, 0x84ad, 0x84af,
            0x84b1, 0x84b3, 0x84b6, 0x84b8, 0x84bb, 0x84bf, 0x84c1, 0x84c5, 0x84c9, 0x84cb, 0x84cd,
            0x84cf, 0x84d1, 0x84d5, 0x84d7, 0x84da, 0x84de, 0x84e0, 0x84e2, 0x84e4, 0x84e8, 0x84ea,
            0x84ee, 0x84f0, 0x84f2, 0x84f4, 0x84f7, 0x84fb, 0x84fd, 0x84ff, 0x8501, 0x8505, 0x8507,
            0x8509, 0x850b, 0x850d, 0x850f, 0x8512, 0x8515, 0x8519, 0x851b, 0x851f, 0x8521, 0x8523,
            0x8526, 0x8528, 0x852a, 0x852c, 0x8530, 0x8532, 0x8534, 0x8536, 0x853a, 0x853d, 0x853f,
            0x8542, 0x8544, 0x8546, 0x8549,
        ];
        let mut checked = 0;
        let mut calls = 0;
        for pc in pcs {
            for case in 0..38 {
                for position in [
                    snes::CpuRasterPosition::new(100, 532),
                    snes::CpuRasterPosition::new(224, 1350),
                    snes::CpuRasterPosition::new(261, 1350),
                ] {
                    let (mut cpu, mut clock, mut source, mut ram) =
                        fixture(&rom, pc, case, position);
                    let mut work = PreOverworldProperties::new(0x83c6, 0x1fc);
                    work.pc = pc;
                    compare_instruction(
                        &format!("properties pc={pc:04x} case={case}"),
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
                    if let Some(call) = work.call {
                        calls += 1;
                        let boundary = (cpu, clock.clone(), ram.clone());
                        assert!(work
                            .step(&mut cpu, &mut clock, &mut |_, _| panic!("callee owns bus"))
                            .is_none());
                        assert_eq!((cpu, clock.clone(), ram.clone()), boundary);
                        if call.child != PropertiesChild::SongBankTail {
                            // The child returns the live result; the caller
                            // validates the frame without replacing registers.
                            cpu.bank = 2;
                            cpu.pc = call.returned;
                            cpu.sp = call.sp;
                            cpu.a = 0xabcd;
                            cpu.x = 37;
                            cpu.y = 91;
                            let returned = cpu;
                            work.return_from_child(&cpu);
                            assert_eq!(cpu, returned);
                            assert_eq!(work.child(), None);
                            assert!(!work.complete());
                        }
                    }
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, pcs.len() * 114);
        assert_eq!(calls, 20 * 114); // 19 real calls and the song-bank tail.
        eprintln!("properties Source instruction comparisons={checked} call boundaries={calls}");
    }

    #[test]
    fn properties_bunny_child_returns_on_live_source_frame() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for case in 0..12 {
            let (mut cpu, mut clock, mut source, mut ram) =
                fixture(&rom, 0x83ce, case, snes::CpuRasterPosition::new(224, 1300));
            let mut properties = PreOverworldProperties::new(0x83c6, 0x1fc);
            properties.pc = 0x83ce;
            compare_instruction(
                "properties bunny JSL",
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                &rom,
                |cpu, clock, bus| {
                    properties.step(cpu, clock, bus).unwrap();
                },
            );
            assert_eq!(properties.child(), Some(PropertiesChild::BunnyStatus));
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x856a, 0x1f7));
            let mut bunny = BunnyStatus::default();
            for _ in 0..12 {
                if bunny.complete() {
                    break;
                }
                compare_instruction(
                    "properties bunny child",
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        bunny.step(cpu, clock, bus).unwrap();
                    },
                );
            }
            assert!(bunny.complete());
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x83d2, 0x1fa));
            let returned = cpu;
            properties.return_from_child(&cpu);
            assert_eq!(properties.child(), None);
            assert_eq!(cpu, returned);
            assert_eq!(ram, source.snes().ram);
            compare_instruction(
                "properties resumes after bunny",
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                &rom,
                |cpu, clock, bus| {
                    properties.step(cpu, clock, bus).unwrap();
                },
            );
        }
    }

    #[test]
    fn properties_song_bank_tail_consumes_original_near_frame() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let (mut cpu, mut clock, mut source, mut ram) =
            fixture(&rom, 0x8549, 0, snes::CpuRasterPosition::new(224, 1300));
        // LDA $0136 sees no pending reload, so this exercises the real
        // conditional-load RTS without requiring the separate live APU owner.
        let mut work = PreOverworldProperties::new(0x83c6, 0x1fc);
        work.pc = 0x8549;
        compare_instruction(
            "properties tail store",
            &mut cpu,
            &mut clock,
            &mut source,
            &mut ram,
            &rom,
            |cpu, clock, bus| {
                work.step(cpu, clock, bus).unwrap();
            },
        );
        assert_eq!(work.child(), Some(PropertiesChild::SongBankTail));
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x854c, 0x1fa));
        let mut song = super::super::song_bank::SongBank::from_entry(0x02854c);
        for _ in 0..8 {
            if song.complete() {
                break;
            }
            compare_instruction(
                "properties song tail",
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                &rom,
                |cpu, clock, bus| {
                    song.step(cpu, clock, bus).unwrap();
                },
            );
        }
        assert!(song.complete());
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x83c6, 0x1fc));
        work.return_from_child(&cpu);
        assert!(work.complete());
        assert_eq!(ram, source.snes().ram);
    }
}
