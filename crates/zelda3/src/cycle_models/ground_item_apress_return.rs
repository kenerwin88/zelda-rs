//! Chest A-press return through the cached tile-action dispatch. Opening a
//! chest selects action 5, whose follow-up handler is the shared RTS. The two
//! near returns consume their real caller stack on the retained hardware bus.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;

const ACTION_TARGETS: [u16; 6] = [0xaaa1, 0xb1ca, 0xb2ed, 0xb322, 0xb527, 0xb5bf];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let offset = address.checked_sub(0x079b92)? as usize;
    Some(ACTION_TARGETS.get(offset / 2)?.to_le_bytes()[offset & 1])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GroundItemApressReturn {
    pc: u16,
    returns: u8,
}
impl Default for GroundItemApressReturn {
    fn default() -> Self {
        Self {
            pc: 0xb5ba,
            returns: 0,
        }
    }
}
impl GroundItemApressReturn {
    pub(crate) fn complete(&self) -> bool {
        self.returns == 2
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
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
            0xb5ba => {
                w.branch(true);
                0xb5bf
            }
            0xb5bf => {
                w.rts();
                self.returns += 1;
                assert_eq!(w.cpu.pc, if self.returns == 1 { 0x9c41 } else { 0x81b3 });
                w.cpu.pc
            }
            0x9c41 => {
                w.drain(16);
                let value = w.read(w.absolute(0x36c), 1);
                assert_eq!(value, 5, "chest return must retain its own A-press action");
                w.load(value, 1);
                0x9c44
            }
            0x9c44 => {
                w.drain(16);
                w.write(w.absolute(0x306), w.cpu.a & 255, 1);
                0x9c47
            }
            0x9c47 => {
                w.shift_left(1);
                0x9c48
            }
            0x9c48 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                0x9c49
            }
            0x9c49 => {
                w.operand_then_internal(2);
                let target = w.read(0x079b92 + u32::from(w.cpu.x), 2);
                assert_eq!(target, 0xb5bf);
                target
            }
            _ => unreachable!("native chest A-press return {:04x}", self.pc),
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
    fn ground_item_apress_return_matches_source_hardware_and_live_stack() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x079b92..0x079b9e {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        let mut count = 0;
        for case in 0..256u16 {
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram[0x36c] = 5;
            machine.ram[0x1f1..0x1f5].copy_from_slice(&[0x40, 0x9c, 0xb2, 0x81]);
            let mut cpu = NativeNmiContext {
                a: (case << 8) | case,
                x: case,
                y: 255 - case,
                d: 0,
                db: if case & 2 != 0 { 7 } else { 0 },
                bank: 7,
                pc: 0xb5ba,
                sp: 0x1f0,
                p: 0x30 | (case as u8 & 0xc3),
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
                    if case & 1 == 0 { 20 } else { 261 },
                    if case & 1 == 0 { 1086 } else { 1350 },
                ),
                CpuBusWorkload::default(),
                CpuFieldTiming::non_interlace(case & 1 != 0),
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = GroundItemApressReturn::default();
            while !work.complete() {
                let label = format!("A-press return case={case} pc={:04x}", cpu.pc);
                super::super::native_work::compare_instruction_with_devices(
                    &label,
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    None,
                    |_, access| {
                        let super::super::intro_clear::Access::Read(address, 2) = access else {
                            return None;
                        };
                        table_byte(address)
                            .map(|low| u16::from_le_bytes([low, table_byte(address + 1).unwrap()]))
                    },
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                count += 1;
            }
            assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.x), (7, 0x81b3, 0x1f4, 10));
            assert_eq!(ram, source.snes().ram);
        }
        println!("chest A-press source comparison: 256 cases, {count} instructions");
    }
}
