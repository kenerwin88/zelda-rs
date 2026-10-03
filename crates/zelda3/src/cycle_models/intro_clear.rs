//! Interruptible intro WRAM column clear. Each step retires one native
//! instruction; CPU registers and the live bus own all values across NMI.
use super::nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO};
use snes::{cpu_timeline::CpuSynchronousTimelineEvent, CpuMasterTimeline};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Width,
    Index,
    Zero,
    Store(u8),
    Dex,
    DexAgain,
    Compare,
    Branch,
    SaveIndex,
    Transfer,
    Carry,
    Subtract,
    SaveStop,
    ByteWidth,
    Return,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroClear {
    phase: Phase,
}
impl Default for IntroClear {
    fn default() -> Self {
        Self {
            phase: Phase::Width,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    Read(u32, u8),
    Write(u32, u16, u8),
    Vblank { field: u64, at: u64 },
}
impl IntroClear {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        use Phase::*;
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db), (12, self.pc(), 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.phase = match self.phase {
            Width => {
                w.drain(8);
                w.drain(6);
                w.cpu.p &= !0x30;
                Index
            }
            Index => {
                w.drain(8);
                w.cpu.x = w.read(0xc8, 2);
                w.flags(w.cpu.x);
                Zero
            }
            Zero => {
                w.drain(16);
                w.cpu.a = 0;
                w.flags(0);
                Store(0)
            }
            Store(i) => {
                w.drain(24);
                w.write(
                    0x7e2000 + u32::from(i) * 0x2000 + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                if i == 14 {
                    Dex
                } else {
                    Store(i + 1)
                }
            }
            Dex | DexAgain => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1);
                w.flags(w.cpu.x);
                if self.phase == Dex {
                    DexAgain
                } else {
                    Compare
                }
            }
            Compare => {
                w.drain(8);
                let stop = w.read(0xca, 2);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.x >= stop);
                w.flags(w.cpu.x.wrapping_sub(stop));
                Branch
            }
            Branch => {
                w.drain(8);
                if w.cpu.p & ZERO == 0 {
                    w.drain(6);
                    Store(0)
                } else {
                    SaveIndex
                }
            }
            SaveIndex => {
                w.drain(8);
                w.write(0xc8, w.cpu.x, 2);
                Transfer
            }
            Transfer => {
                w.drain(6);
                w.cpu.a = w.cpu.x;
                w.flags(w.cpu.a);
                Carry
            }
            Carry => {
                w.drain(6);
                w.cpu.p |= CARRY;
                Subtract
            }
            Subtract => {
                w.drain(16);
                let old = w.cpu.a;
                let rhs = 0x400 + u32::from(w.cpu.p & CARRY == 0);
                w.cpu.a = old.wrapping_sub(rhs as u16);
                w.cpu.p = (w.cpu.p & !(CARRY | 0x40))
                    | u8::from(u32::from(old) >= rhs)
                    | if (old ^ w.cpu.a) & (old ^ 0x400) & 0x8000 != 0 {
                        0x40
                    } else {
                        0
                    };
                w.flags(w.cpu.a);
                SaveStop
            }
            SaveStop => {
                w.drain(8);
                w.write(0xca, w.cpu.a, 2);
                ByteWidth
            }
            ByteWidth => {
                w.drain(8);
                w.drain(6);
                w.cpu.p |= 0x30;
                w.cpu.x &= 255;
                w.cpu.y &= 255;
                Return
            }
            Return => {
                w.drain(12);
                w.cpu.pc = w.read(u32::from(w.cpu.sp + 1), 2).wrapping_add(1);
                w.cpu.sp += 2;
                w.cpu.bank = w.read(u32::from(w.cpu.sp + 1), 1) as u8;
                w.cpu.sp += 1;
                Complete
            }
            Complete => unreachable!(),
        };
        if !self.complete() {
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
    fn pc(&self) -> u16 {
        use Phase::*;
        match self.phase {
            Width => 0xc1a0,
            Index => 0xc1a2,
            Zero => 0xc1a4,
            Store(i) => 0xc1a7 + u16::from(i) * 4,
            Dex => 0xc1e3,
            DexAgain => 0xc1e4,
            Compare => 0xc1e5,
            Branch => 0xc1e7,
            SaveIndex => 0xc1e9,
            Transfer => 0xc1eb,
            Carry => 0xc1ec,
            Subtract => 0xc1ed,
            SaveStop => 0xc1f0,
            ByteWidth => 0xc1f2,
            Return => 0xc1f4,
            Complete => unreachable!(),
        }
    }
}
struct Work<'a, B> {
    cpu: &'a mut NativeNmiContext,
    clock: &'a mut CpuMasterTimeline,
    bus: B,
    cycles: u64,
}
impl<B: FnMut(u64, Access) -> u16> Work<'_, B> {
    fn fetch(&mut self) {
        self.cycles += 8;
        self.clock.advance_synchronous_pcbase_opcode_fetch(8);
    }
    fn drain(&mut self, n: u32) {
        self.cycles += u64::from(n);
        let bus = &mut self.bus;
        self.clock
            .advance_synchronous_after_semantics_with(n, |event, _| {
                if let CpuSynchronousTimelineEvent::HMax {
                    completed_field_index,
                    completed_scanline: 224,
                    event_timestamp,
                    ..
                } = event
                {
                    bus(
                        event_timestamp.master_cycles(),
                        Access::Vblank {
                            field: completed_field_index,
                            at: event_timestamp.master_cycles(),
                        },
                    );
                }
                Ok::<u32, String>(0)
            })
            .unwrap();
    }
    fn read(&mut self, a: u32, width: u8) -> u16 {
        let v = (self.bus)(self.clock.clock_master_cycles(), Access::Read(a, width));
        self.drain(8 * u32::from(width));
        v
    }
    fn write(&mut self, a: u32, v: u16, width: u8) {
        (self.bus)(self.clock.clock_master_cycles(), Access::Write(a, v, width));
        self.drain(8 * u32::from(width));
    }
    fn flags(&mut self, v: u16) {
        self.cpu.p = (self.cpu.p & !(ZERO | NEGATIVE))
            | if v == 0 { ZERO } else { 0 }
            | ((v >> 8) as u8 & NEGATIVE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourceCpuBusAccessKind as Kind, SourcePpuReadState,
    };
    #[test]
    fn each_clear_instruction_matches_rom_bus_registers_and_clock_across_pauses() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        for seed in [0u8, 0x7f, 0xff] {
            for length in [2u16, 0x400] {
                for (v, h) in [(153, 816), (224, 1338), (261, 1350)] {
                    for odd in [false, true] {
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        for (i, b) in machine.ram.iter_mut().enumerate() {
                            *b = (i ^ (i >> 8)) as u8 ^ seed;
                        }
                        machine.ram[0xc8..0xca].copy_from_slice(&0x1ffeu16.to_le_bytes());
                        machine.ram[0xca..0xcc].copy_from_slice(&(0x1ffe - length).to_le_bytes());
                        machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                        let mut native = NativeNmiContext {
                            a: 0x0cc1,
                            x: 0xfc,
                            y: 0xff,
                            d: 0,
                            db: 0,
                            bank: 12,
                            pc: 0xc1a0,
                            sp: 0x1fc,
                            p: 0x30 | (seed & 0xc7),
                        };
                        machine.cpu.a = native.a;
                        machine.cpu.x = native.x;
                        machine.cpu.y = native.y;
                        machine.cpu.sp = native.sp;
                        machine.cpu.pc = native.pc;
                        machine.cpu.k = native.bank;
                        machine.cpu.e = false;
                        machine.cpu.unpack_flags(native.p);
                        let mut ram = machine.ram.clone();
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
                        let mut clear = IntroClear::default();
                        let mut stores = 0;
                        while !clear.complete() {
                            let expected = source
                                .step()
                                .unwrap()
                                .accesses
                                .into_iter()
                                .filter_map(|a| {
                                    if !(a.address < 0x2000
                                        || (0x7e2000..=0x7fffff).contains(&a.address))
                                    {
                                        return None;
                                    }
                                    let (op, value) = match a.kind {
                                        Kind::Read { value, width } => {
                                            (Access::Read(a.address, width), value)
                                        }
                                        Kind::Write { value, width } => {
                                            (Access::Write(a.address, value, width), 0)
                                        }
                                        _ => return None,
                                    };
                                    Some((a.timestamp.master_cycles(), op, value))
                                })
                                .collect::<Vec<_>>();
                            let mut actual = Vec::new();
                            clear
                                .step(&mut native, &mut clock, |at, op| {
                                    let offset = |a: u32| {
                                        if a >= 0x7e0000 {
                                            a as usize - 0x7e0000
                                        } else {
                                            a as usize
                                        }
                                    };
                                    let value = match op {
                                        Access::Read(a, w) => {
                                            let i = offset(a);
                                            if w == 1 {
                                                u16::from(ram[i])
                                            } else {
                                                u16::from_le_bytes([ram[i], ram[i + 1]])
                                            }
                                        }
                                        Access::Write(a, v, w) => {
                                            assert_eq!(w, 2);
                                            let i = offset(a);
                                            ram[i..i + 2].copy_from_slice(&v.to_le_bytes());
                                            if a >= 0x7e2000 {
                                                stores += 1;
                                            }
                                            0
                                        }
                                        Access::Vblank { .. } => return 0,
                                    };
                                    actual.push((at, op, value));
                                    value
                                })
                                .unwrap();
                            assert_eq!(
                                actual, expected,
                                "seed={seed} length={length} v={v} h={h} odd={odd}"
                            );
                            let c = &source.snes().cpu;
                            assert_eq!(
                                (
                                    native.a,
                                    native.x,
                                    native.y,
                                    native.sp,
                                    native.pc,
                                    native.bank,
                                    native.db,
                                    native.d,
                                    native.p
                                ),
                                (c.a, c.x, c.y, c.sp, c.pc, c.k, c.db, c.dp, c.pack_flags())
                            );
                            assert_eq!(clock, *source.timeline());
                            clear = clear.clone(); // each retired instruction is a resumable boundary
                        }
                        assert_eq!(stores, usize::from(length / 2) * 15);
                        assert_eq!(ram, source.snes().ram);
                    }
                }
            }
        }
    }
}
