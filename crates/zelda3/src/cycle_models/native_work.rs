//! Shared instruction bus work for retained native initialization owners.
use super::{
    intro_clear::Access,
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::{cpu_timeline::CpuSynchronousTimelineEvent, CpuMasterTimeline};

/// Ordinary owners need only access timestamps. Device reads such as HVBJOY
/// also need the CPU's processed beam, sampled at the actual bus semantic.
pub(crate) trait WorkBus {
    /// Typed live eligibility for a composed inactive actor return. This is
    /// ownership metadata, not a CPU bus access or a timing charge.
    fn inactive_sprite_body_mask(&mut self) -> u16 { 0 }

    /// The actual CLI/BRA return has reached INC $1A. This observes the
    /// caller boundary without adding a bus transaction or clock charge.
    fn begin_main_iteration(&mut self, _clock: &CpuMasterTimeline) {}

    fn access(&mut self, at: u64, access: Access) -> u16;
    fn observe_clock(&mut self, _clock: &CpuMasterTimeline) {}
    fn end_scanline(&mut self, _at: u64) {}
    fn retire_event(&mut self, event: CpuSynchronousTimelineEvent, clock: &CpuMasterTimeline) -> u32 {
        self.timeline_event(event, clock.clock_master_cycles());
        0
    }
    fn timeline_event(&mut self, event: CpuSynchronousTimelineEvent, processed_at: u64) {
        if let CpuSynchronousTimelineEvent::HMax {
            completed_field_index,
            completed_scanline,
            event_timestamp,
            ..
        } = event {
            self.end_scanline(processed_at);
            if completed_scanline == 224 {
                self.access(event_timestamp.master_cycles(), Access::Vblank {
                    field: completed_field_index,
                    at: event_timestamp.master_cycles(),
                });
            }
        }
    }
}
impl<F: FnMut(u64, Access) -> u16> WorkBus for F {
    fn access(&mut self, at: u64, access: Access) -> u16 {
        self(at, access)
    }
}
pub(crate) struct BeamBus<F> {
    callback: F,
    beam: Option<snes::cpu_timeline::CpuSynchronousBeamPosition>,
}
impl<F> BeamBus<F> {
    pub(crate) fn new(callback: F) -> Self {
        Self {
            callback,
            beam: None,
        }
    }
}
impl<F: FnMut(u64, Access, Option<snes::cpu_timeline::CpuSynchronousBeamPosition>) -> u16> WorkBus
    for BeamBus<F>
{
    fn access(&mut self, at: u64, access: Access) -> u16 {
        (self.callback)(at, access, self.beam)
    }
    fn observe_clock(&mut self, clock: &CpuMasterTimeline) {
        self.beam = clock.synchronous_beam_position();
    }
}

#[cfg(test)]
pub(super) fn compare_instruction(
    label: &str,
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    source: &mut snes::RomCpuTimingProbe,
    ram: &mut [u8],
    rom: &[u8],
    step: impl FnOnce(&mut NativeNmiContext, &mut CpuMasterTimeline, &mut dyn FnMut(u64, Access) -> u16),
) {
    compare_instruction_with_ppu(label, cpu, clock, source, ram, rom, None, step);
}

#[cfg(test)]
pub(super) fn compare_instruction_with_ppu(
    label: &str,
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    source: &mut snes::RomCpuTimingProbe,
    ram: &mut [u8],
    rom: &[u8],
    ppu: Option<&mut snes::PpuState>,
    step: impl FnOnce(&mut NativeNmiContext, &mut CpuMasterTimeline, &mut dyn FnMut(u64, Access) -> u16),
) {
    compare_instruction_with_devices(label, cpu, clock, source, ram, rom, ppu, |_, _| None, step);
}

#[cfg(test)]
pub(crate) fn compare_instruction_with_devices(
    label: &str,
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    source: &mut snes::RomCpuTimingProbe,
    ram: &mut [u8],
    rom: &[u8],
    mut ppu: Option<&mut snes::PpuState>,
    mut device: impl FnMut(u64, Access) -> Option<u16>,
    step: impl FnOnce(&mut NativeNmiContext, &mut CpuMasterTimeline, &mut dyn FnMut(u64, Access) -> u16),
) {
    use snes::SourceCpuBusAccessKind as Kind;
    let normalize = |a: u32| {
        if a & 0x40_0000 == 0 && a & 65535 < 0x6000 {
            a & 65535
        } else {
            a
        }
    };
    let operand = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc + 1);
    // JSR (abs,X) fetches its two operand bytes separately. Both are
    // instruction fetches, rather than reads owned by the native data bus.
    let split_jsr_operand=cpu.pc>=0x8000 && rom[((usize::from(cpu.bank)&0x7f)*0x8000)+(usize::from(cpu.pc)&0x7fff)]==0xfc;
    let mut first_read = true;
    let expected = source
        .step()
        .unwrap()
        .accesses
        .into_iter()
        .filter_map(|a| {
            let address = normalize(a.address);
            let (access, value) = match a.kind {
                Kind::Read { value, width } => {
                    let operand_fetch = first_read && a.address == operand;
                    first_read = false;
                    if operand_fetch || (split_jsr_operand && a.address==operand+1) {
                        return None;
                    }
                    (Access::Read(address, width), value)
                }
                Kind::Write { value, width } => (Access::Write(address, value, width), 0),
                _ => return None,
            };
            Some((a.timestamp.master_cycles(), access, value))
        })
        .collect::<Vec<_>>();
    let mut actual = Vec::new();
    step(cpu, clock, &mut |at, access| {
        if let Some(value) = device(at, access) {
            actual.push((at, access, value));
            return value;
        }
        let mut read = |a: u32| {
            let a = normalize(a);
            if a < 0x2000 {
                ram[a as usize]
            } else if (0x2134..=0x2136).contains(&a) {
                ppu.as_mut()
                    .expect("PPU multiplier bus owner")
                    .read(a as u8)
            } else if (0x7e0000..0x800000).contains(&a) {
                ram[a as usize - 0x7e0000]
            } else {
                assert_ne!(a & 0x8000, 0, "test read escaped ROM/WRAM {a:06x}");
                rom[((a >> 16) as usize & 127) * 0x8000 + (a as usize & 0x7fff)]
            }
        };
        let (access, value) = match access {
            Access::Read(a, width) => {
                let value = if width == 1 {
                    u16::from(read(a))
                } else {
                    u16::from_le_bytes([read(a), read(a + 1)])
                };
                (Access::Read(normalize(a), width), value)
            }
            Access::Write(a, value, width) => {
                let a = normalize(a);
                if (0x2100..0x2140).contains(&a) {
                    assert_eq!(width, 1);
                    ppu.as_mut()
                        .expect("PPU bus owner")
                        .write(a as u8, value as u8);
                    actual.push((at, Access::Write(a, value, width), 0));
                    return 0;
                }
                let offset = if a >= 0x7e0000 {
                    a as usize - 0x7e0000
                } else {
                    a as usize
                };
                ram[offset..offset + usize::from(width)]
                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                (Access::Write(a, value, width), 0)
            }
            Access::Vblank { .. } => return 0,
        };
        actual.push((at, access, value));
        value
    });
    assert_eq!(actual, expected, "{label}");
    let c = &source.snes().cpu;
    assert_eq!(
        *cpu,
        NativeNmiContext {
            a: c.a,
            x: c.x,
            y: c.y,
            d: c.dp,
            db: c.db,
            bank: c.k,
            pc: c.pc,
            sp: c.sp,
            p: c.pack_flags()
        },
        "{label}"
    );
    assert_eq!(*clock, *source.timeline(), "{label}");
}
impl<B: WorkBus> Work<'_, B> {
    /// Slow-ROM operand fetch followed by one internal CPU cycle. Indexed
    /// addressing, JSR/JSL and BRL drain these transactions separately: an
    /// HMax between them must synchronize devices before the internal cycle.
    pub(super) fn operand_then_internal(&mut self, width: u8) {
        self.drain(8 * u32::from(width));
        self.drain(6);
    }
    pub(super) fn read_io(&mut self, address: u16, width: u8) -> u16 {
        let mut value = 0;
        for byte in 0..width {
            self.bus.observe_clock(self.clock);
            value |= self.bus.access(
                self.clock.clock_master_cycles(),
                Access::Read(u32::from(address) + u32::from(byte), 1),
            ) << (byte * 8);
            self.drain(6);
        }
        value
    }

    pub(super) fn adc_byte(&mut self, rhs: u16) {
        assert_eq!(self.cpu.p & 8, 0, "native arithmetic is binary");
        let old = self.cpu.a & 255;
        let sum = old + rhs + u16::from(self.cpu.p & CARRY != 0);
        self.cpu.p = (self.cpu.p & !0x41)
            | u8::from(sum > 255)
            | if (!(old ^ rhs) & (old ^ sum) & 0x80) != 0 {
                0x40
            } else {
                0
            };
        self.load(sum & 255, 1);
    }

    pub(super) fn sbc(&mut self, rhs: u16, width: u8) {
        assert_eq!(self.cpu.p & 8, 0, "native arithmetic is binary");
        let mask = if width == 1 { 255 } else { 65535 };
        let sign = if width == 1 { 0x80 } else { 0x8000 };
        let old = self.cpu.a & mask;
        let result = i32::from(old) - i32::from(rhs) - i32::from(self.cpu.p & CARRY == 0);
        let value = result as u16 & mask;
        self.cpu.p = (self.cpu.p & !0x41)
            | u8::from(result >= 0)
            | if ((old ^ rhs) & (old ^ value) & sign) != 0 {
                0x40
            } else {
                0
            };
        self.load(value, width);
    }

    pub(super) fn load(&mut self, value: u16, width: u8) {
        self.cpu.a = if width == 1 {
            (self.cpu.a & 0xff00) | (value & 255)
        } else {
            value
        };
        self.flags(value, width);
    }
    pub(super) fn compare_immediate(&mut self, value: u16, width: u8) {
        self.drain(8 * u32::from(width));
        let left = if width == 1 {
            self.cpu.a & 255
        } else {
            self.cpu.a
        };
        self.cpu.p = (self.cpu.p & !CARRY) | u8::from(left >= value);
        self.flags(
            if width == 1 {
                left.wrapping_sub(value) & 255
            } else {
                left.wrapping_sub(value)
            },
            width,
        );
    }
    pub(super) fn compare_x_immediate(&mut self, value: u16, width: u8) {
        self.drain(8 * u32::from(width));
        let left = if width == 1 {
            self.cpu.x & 255
        } else {
            self.cpu.x
        };
        self.cpu.p = (self.cpu.p & !CARRY) | u8::from(left >= value);
        self.flags(
            if width == 1 {
                left.wrapping_sub(value) & 255
            } else {
                left.wrapping_sub(value)
            },
            width,
        );
    }
    pub(super) fn shift_left(&mut self, width: u8) {
        self.drain(6);
        let old = self.cpu.a;
        let bit = if width == 1 { 0x80 } else { 0x8000 };
        self.cpu.p = (self.cpu.p & !CARRY) | u8::from(old & bit != 0);
        self.load(
            if width == 1 {
                (old << 1) & 255
            } else {
                old << 1
            },
            width,
        );
    }
    pub(super) fn increment_a(&mut self, width: u8) {
        self.drain(6);
        let value = self.cpu.a.wrapping_add(1);
        self.load(if width == 1 { value & 255 } else { value }, width);
    }
    pub(super) fn increment_y(&mut self) {
        self.drain(6);
        self.cpu.y = self.cpu.y.wrapping_add(1);
        self.flags(self.cpu.y, 2);
    }
    pub(super) fn decrement_x(&mut self) {
        self.drain(6);
        self.cpu.x = self.cpu.x.wrapping_sub(1);
        self.flags(self.cpu.x, 2);
    }
    pub(super) fn indirect_y_address(&mut self, pointer: u16) -> u32 {
        self.drain(8);
        let low = self.read(u32::from(pointer), 2);
        let bank = self.read(u32::from(pointer + 2), 1);
        ((u32::from(bank) << 16) | u32::from(low)).wrapping_add(u32::from(self.cpu.y)) & 0xff_ffff
    }
    pub(super) fn dp_read(&mut self, address: u16, width: u8) -> u16 {
        self.drain(8);
        self.read(u32::from(address), width)
    }
    pub(super) fn dp_store(&mut self, address: u16, value: u16, width: u8) {
        self.drain(8);
        self.write(u32::from(address), value, width);
    }
    pub(super) fn dp_increment(&mut self, address: u16, width: u8, increment: bool) {
        let old = self.dp_read(address, width);
        let value = if increment {
            old.wrapping_add(1)
        } else {
            old.wrapping_sub(1)
        };
        let value = if width == 1 { value & 255 } else { value };
        self.drain(6);
        self.flags(value, width);
        self.write_high_low(u32::from(address), value, width);
    }
    pub(super) fn indirect_read(&mut self, pointer: u16, width: u8) -> u16 {
        self.drain(8);
        let low = self.read(u32::from(pointer), 2);
        let bank = self.read(u32::from(pointer + 2), 1);
        self.read((u32::from(bank) << 16) | u32::from(low), width)
    }
    pub(super) fn swap_bytes(&mut self) {
        self.drain(12);
        self.cpu.a = self.cpu.a.swap_bytes();
        self.flags(self.cpu.a & 255, 1);
    }
    pub(super) fn mask(&mut self, value: u16, width: u8) {
        self.drain(8 * u32::from(width));
        let result = self.cpu.a & value;
        self.cpu.a = if width == 1 {
            (self.cpu.a & 0xff00) | (result & 255)
        } else {
            result
        };
        self.flags(result, width);
    }
    pub(super) fn branch(&mut self, taken: bool) -> bool {
        self.drain(8);
        if taken {
            self.drain(6);
        }
        taken
    }
    pub(super) fn absolute(&self, a: u16) -> u32 {
        (u32::from(self.cpu.db) << 16) | u32::from(a)
    }
    pub(super) fn width(&mut self, mask: u8, clear: bool) {
        self.drain(8);
        self.drain(6);
        if clear {
            self.cpu.p &= !mask;
        } else {
            self.cpu.p |= mask;
        }
        if self.cpu.p & 0x10 != 0 {
            self.cpu.x &= 255;
            self.cpu.y &= 255;
        }
    }
    pub(super) fn immediate(&mut self, value: u16, width: u8) {
        self.drain(8 * u32::from(width));
        self.cpu.a = if width == 1 {
            (self.cpu.a & 0xff00) | (value & 255)
        } else {
            value
        };
        self.flags(value, width);
    }
    pub(super) fn adc(&mut self, rhs: u16) {
        assert_eq!(self.cpu.p & 8, 0, "native palette arithmetic is binary");
        let old = self.cpu.a;
        let sum = u32::from(old) + u32::from(rhs) + u32::from(self.cpu.p & CARRY != 0);
        self.cpu.a = sum as u16;
        self.cpu.p = (self.cpu.p & !(CARRY | 0x40))
            | u8::from(sum > 65535)
            | if (!(old ^ rhs) & (old ^ self.cpu.a) & 0x8000) != 0 {
                0x40
            } else {
                0
            };
        self.flags(self.cpu.a, 2);
    }
    pub(super) fn clear_carry(&mut self) {
        self.drain(6);
        self.cpu.p &= !CARRY;
    }
    pub(super) fn tax_word(&mut self) {
        self.drain(6);
        self.cpu.x = self.cpu.a;
        self.flags(self.cpu.x, 2);
    }
    pub(super) fn increment_x(&mut self) {
        self.drain(6);
        self.cpu.x = self.cpu.x.wrapping_add(1);
        self.flags(self.cpu.x, 2);
    }
    pub(super) fn decrement_y(&mut self) {
        self.drain(6);
        self.cpu.y = self.cpu.y.wrapping_sub(1);
        self.flags(self.cpu.y, 2);
    }
    pub(super) fn rts(&mut self) {
        self.drain(12);
        self.cpu.pc = self.pull(2).wrapping_add(1);
        self.drain(6);
    }
    pub(super) fn rtl(&mut self) {
        self.drain(12);
        self.cpu.pc = self.pull(2).wrapping_add(1);
        self.cpu.bank = self.pull(1) as u8;
    }
}

pub(super) struct Work<'a, B> {
    pub(super) cpu: &'a mut NativeNmiContext,
    pub(super) clock: &'a mut CpuMasterTimeline,
    pub(super) bus: B,
    pub(super) cycles: u64,
}
impl<B: WorkBus> Work<'_, B> {
    pub(super) fn fetch(&mut self) {
        self.cycles += 8;
        self.clock.advance_synchronous_pcbase_opcode_fetch(8);
    }
    pub(super) fn drain(&mut self, n: u32) {
        self.cycles += u64::from(n);
        let bus = &mut self.bus;
        self.clock
            .advance_synchronous_after_semantics_with_clock(n, |event, clock| {
                Ok::<u32, String>(bus.retire_event(event, clock))
            })
            .unwrap();
    }
    pub(super) fn read(&mut self, a: u32, width: u8) -> u16 {
        // The source memory map cannot service a direct word across one of
        // its 4 KiB blocks. Each byte owns its own bus timestamp there.
        if width == 2 && a & 0xfff == 0xfff {
            let low = self.read(a, 1);
            let high = self.read(a + 1, 1);
            return low | high << 8;
        }
        self.bus.observe_clock(self.clock);
        let value = self
            .bus
            .access(self.clock.clock_master_cycles(), Access::Read(a, width));
        self.drain(8 * u32::from(width));
        value
    }
    pub(super) fn write(&mut self, a: u32, value: u16, width: u8) {
        self.write_ordered(a, value, width, false);
    }
    /// RMW instructions and stack pushes write the high byte first when a
    /// word crosses the source map's direct-transaction boundary.
    pub(super) fn write_high_low(&mut self, a: u32, value: u16, width: u8) {
        self.write_ordered(a, value, width, true);
    }
    fn write_ordered(&mut self, a: u32, value: u16, width: u8, high_first: bool) {
        if width == 2 && a & 0xfff == 0xfff {
            let bytes = if high_first { [1, 0] } else { [0, 1] };
            for byte in bytes {
                self.write(a + byte, (value >> (byte * 8)) & 255, 1);
            }
            return;
        }
        self.bus.observe_clock(self.clock);
        self.bus.access(
            self.clock.clock_master_cycles(),
            Access::Write(a, value, width),
        );
        self.drain(8 * u32::from(width));
    }
    pub(super) fn write_io(&mut self, address: u16, value: u16, width: u8) {
        for byte in 0..width {
            self.bus.observe_clock(self.clock);
            self.bus.access(
                self.clock.clock_master_cycles(),
                Access::Write(
                    u32::from(address) + u32::from(byte),
                    (value >> (byte * 8)) & 255,
                    1,
                ),
            );
            self.drain(6);
        }
    }
    pub(super) fn push(&mut self, value: u16, width: u8) {
        self.write_high_low(u32::from(self.cpu.sp + 1 - u16::from(width)), value, width);
        self.cpu.sp -= u16::from(width);
    }
    pub(super) fn pull(&mut self, width: u8) -> u16 {
        let value = self.read(u32::from(self.cpu.sp + 1), width);
        self.cpu.sp += u16::from(width);
        value
    }
    pub(super) fn flags(&mut self, value: u16, width: u8) {
        self.cpu.p = (self.cpu.p & !(NEGATIVE | ZERO))
            | if value == 0 { ZERO } else { 0 }
            | ((if width == 1 { value } else { value >> 8 }) as u8 & NEGATIVE);
    }
}
