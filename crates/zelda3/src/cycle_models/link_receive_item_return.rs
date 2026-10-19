//! Link_ReceiveItem's retained return. HUD refresh owns its nested calls;
//! dash cancellation and the caller's saved registers retire on the same bus.
use super::{
    hud_update::HudUpdate,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinkReceiveItemReturn {
    pc: u16,
    hud: Option<HudUpdate>,
    complete: bool,
}

impl Default for LinkReceiveItemReturn {
    fn default() -> Self {
        Self {
            pc: 0x9a0b,
            hud: None,
            complete: false,
        }
    }
}

impl LinkReceiveItemReturn {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn hud_active(&self) -> bool {
        self.hud.is_some()
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.hud_active() {
            0x0ddb7f
        } else if (0x9195..=0x91b8).contains(&self.pc) {
            0x079195
        } else {
            0x079a0b
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
        if let Some(hud) = &mut self.hud {
            let cycles = hud.step(cpu, clock, bus);
            if hud.complete() {
                assert_eq!((cpu.bank, cpu.pc), (7, 0x9a22));
                self.pc = 0x9a22;
                self.hud = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (7, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x9a0b | 0x9195 => {
                w.drain(16);
                let value = w.read(w.absolute(if self.pc == 0x9a0b { 0x2d8 } else { 0x372 }), 1);
                w.load(value, 1);
                self.pc + 3
            }
            0x9a0e | 0x9a12 | 0x9a16 | 0x9a1a | 0x91a0 => {
                w.compare_immediate(
                    match self.pc {
                        0x9a0e => 0x20,
                        0x9a12 => 0x37,
                        0x9a16 => 0x38,
                        0x9a1a => 0x39,
                        _ => 0x1e,
                    },
                    1,
                );
                self.pc + 2
            }
            0x9a10 | 0x9a14 | 0x9a18 | 0x9a1c | 0x9198 | 0x91a2 => {
                let taken = if self.pc == 0x91a2 {
                    w.cpu.p & ZERO == 0
                } else {
                    w.cpu.p & ZERO != 0
                };
                if w.branch(taken) {
                    if self.pc == 0x9198 {
                        0x91b8
                    } else if self.pc == 0x91a2 {
                        0x91a7
                    } else {
                        0x9a22
                    }
                } else {
                    self.pc + 2
                }
            }
            0x9a1e => {
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(0x9a21, 2);
                w.cpu.bank = 13;
                w.cpu.pc = 0xdb7f;
                self.hud = Some(HudUpdate::refresh_icon());
                return Some(w.cycles);
            }
            0x9a22 => {
                w.item_call();
                0x9195
            }
            0x919a => {
                w.drain(6);
                w.push(w.cpu.x, 1);
                0x919b
            }
            0x919b => {
                w.item_x(4, 1);
                0x919d
            }
            0x919d => {
                w.drain(16);
                let value = w.read(w.absolute(0xc4a) + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                0x91a0
            }
            0x91a4 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xc4a) + u32::from(w.cpu.x), 0, 1);
                0x91a7
            }
            0x91a7 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                0x91a8
            }
            0x91a8 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    0x919d
                } else {
                    0x91aa
                }
            }
            0x91aa | 0x9a25 => {
                w.drain(12);
                w.cpu.x = w.pull(1);
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x91ab | 0x91b0 | 0x91b5 => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0x91ab => 0x374,
                        0x91b0 => 0x372,
                        _ => 0x32b,
                    }),
                    0,
                    1,
                );
                self.pc + 3
            }
            0x91ae | 0x91b3 => {
                w.dp_store(if self.pc == 0x91ae { 0x5e } else { 0x50 }, 0, 1);
                self.pc + 2
            }
            0x91b8 => {
                w.rts();
                w.cpu.pc
            }
            0x9a26 => {
                w.clear_carry();
                0x9a27
            }
            0x9a27 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                0x9a28
            }
            0x9a28 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("native Link receipt return {:04x}", self.pc),
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
    fn link_receipt_return_matches_source_hud_dash_and_live_stack() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        for item in 0..76u8 {
            for variant in 0..8u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, byte) in machine.ram.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7)) as u8;
                }
                let r = &mut machine.ram;
                r[0xf340..0xf37c].fill(0);
                r[0x202..0x205].fill(0);
                if variant != 0 {
                    r[0x202] = item % 21;
                    r[0xf340..0xf354].fill(1);
                    r[0xf340] = variant % 5;
                    r[0xf34f] = 1;
                    r[0xf35c] = variant;
                }
                r[0xf36c] = [8, 24, 72, 160][usize::from(variant % 4)];
                r[0xf36d] = r[0xf36c] - variant;
                r[0xf36e] = [0, 1, 127, 128][usize::from(variant % 4)];
                r[0xf362..0xf364]
                    .copy_from_slice(&[0u16, 1, 99, 999][usize::from(variant % 4)].to_le_bytes());
                r[0xf36f] = if variant == 0 { 255 } else { variant };
                r[0xf377] = [0, 1, 9, 70][usize::from(variant % 4)];
                r[0x2d8] = item;
                r[0x372] = variant & 1;
                for slot in 0..5 {
                    r[0xc4a + slot] = if (slot + usize::from(variant)) % 3 == 0 {
                        0x1e
                    } else {
                        0x22
                    };
                }
                let saved_x = item.wrapping_mul(3);
                let saved_db = variant;
                r[0x1ec..0x1f1].copy_from_slice(&[saved_x, saved_db, 0xb9, 0xb5, 7]);
                let mut cpu = NativeNmiContext {
                    a: 0x909,
                    x: 4,
                    y: u16::from(item),
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc: 0x9a0b,
                    sp: 0x1eb,
                    p: 0x30,
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
                    4585,
                    CpuRasterPosition::new(247, 34),
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
                let mut work = LinkReceiveItemReturn::default();
                let mut count = 0;
                while !work.complete() {
                    let label = format!(
                        "Link item={item:02x} variant={variant} pc={:06x}",
                        (u32::from(cpu.bank) << 16) | u32::from(cpu.pc)
                    );
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| {
                            let super::super::intro_clear::Access::Read(address, width) = access
                            else {
                                return None;
                            };
                            if address & 0xff0000 != 0x0d0000 || address & 0x8000 == 0 {
                                return None;
                            }
                            let mut value = 0;
                            for byte in 0..width {
                                value |= u16::from(
                                    crate::ZeldaState::native_load_hud_table_byte(
                                        address + u32::from(byte),
                                    )
                                    .expect("native HUD data"),
                                ) << (8 * byte);
                            }
                            Some(value)
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    count += 1;
                    assert!(count < 10000);
                }
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.x),
                    (7, 0xb5ba, 0x1f0, saved_db, u16::from(saved_x))
                );
                assert_eq!(ram, source.snes().ram);
                instructions += count;
            }
        }
        println!("Link receipt source comparison: 608 cases, {instructions} instructions");
    }
}
