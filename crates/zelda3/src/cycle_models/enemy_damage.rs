//! Enemy damage initialization retains its compressed source pointer and
//! expands each nibble from the live output buffer, including its untouched tail.
use super::{
    intro_clear::Access,
    native_decompress::NativeDecompression,
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EnemyDamage {
    pc: u16,
    child: Option<NativeDecompression>,
    complete: bool,
}
impl Default for EnemyDamage {
    fn default() -> Self {
        Self {
            pc: 0xfe71,
            child: None,
            complete: false,
        }
    }
}
impl EnemyDamage {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        self.child
            .as_ref()
            .map_or(0x02fe71, NativeDecompression::routine)
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        if let Some(child) = self.child.as_mut() {
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.child = None;
                assert_eq!((cpu.bank, cpu.pc), (2, self.pc));
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (2, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xfe71 | 0xfe75 | 0xfe79 | 0xfe7d | 0xfe81 | 0xfe85 | 0xfe8c | 0xfe90 | 0xfe94 => {
                w.immediate(
                    match self.pc {
                        0xfe71 | 0xfe7d | 0xfe8c => 0,
                        0xfe75 | 0xfe90 => 0x40,
                        0xfe79 | 0xfe94 => 0x7f,
                        0xfe81 => 0xe8,
                        _ => 3,
                    },
                    1,
                );
                self.pc + 2
            }
            0xfe73 | 0xfe77 | 0xfe7b | 0xfe7f | 0xfe83 | 0xfe87 | 0xfe8e | 0xfe92 | 0xfe96 => {
                w.dp_store(
                    match self.pc {
                        0xfe73 | 0xfe8e => 0,
                        0xfe77 | 0xfe92 => 1,
                        0xfe7b | 0xfe96 => 2,
                        0xfe7f => 0xc8,
                        0xfe83 => 0xc9,
                        _ => 0xca,
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0xfe89 => {
                w.drain(22);
                w.push(0xfe8b, 2);
                w.cpu.pc = 0xfebb;
                self.child = Some(NativeDecompression::bank02());
                0xfe8c
            }
            0xfe98 => {
                w.width(0x10, true);
                0xfe9a
            }
            0xfe9a => {
                w.drain(16);
                w.cpu.x = 0;
                w.flags(0, 2);
                0xfe9d
            }
            0xfe9d => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, 2);
                0xfe9e
            }
            0xfe9e => {
                let address = w.indirect_y_address(0);
                let v = w.read(address, 1);
                w.load(v, 1);
                0xfea0
            }
            0xfea0 => {
                w.drain(6);
                w.push(w.cpu.a, 1);
                0xfea1
            }
            0xfea1..=0xfea4 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | ((w.cpu.a & 1) as u8);
                w.load((w.cpu.a & 255) >> 1, 1);
                self.pc + 1
            }
            0xfea5 | 0xfeac => {
                w.drain(24);
                w.write(
                    0x7f6000 + u32::from(w.cpu.x) + u32::from(self.pc == 0xfeac),
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 4
            }
            0xfea9 => {
                w.drain(12);
                let v = w.pull(1);
                w.load(v, 1);
                0xfeaa
            }
            0xfeaa => {
                w.mask(15, 1);
                0xfeac
            }
            0xfeb0 => {
                w.increment_y();
                0xfeb1
            }
            0xfeb1 | 0xfeb2 => {
                w.increment_x();
                self.pc + 1
            }
            0xfeb3 => {
                w.compare_x_immediate(0x1000, 2);
                0xfeb6
            }
            0xfeb6 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    0xfe9e
                } else {
                    0xfeb8
                }
            }
            0xfeb8 => {
                w.width(0x30, false);
                0xfeba
            }
            0xfeba => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("enemy damage continuation {:04x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
        }
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
    fn enemy_damage_loader_matches_source_and_preserves_decompression_tail() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for (v, h) in [(224, 1338), (261, 1350)] {
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 8)) as u8;
            }
            machine.ram[0x1fb..0x1fe].copy_from_slice(&[0x59, 0x80, 0]);
            let mut native = NativeNmiContext {
                a: 0xa59f,
                x: 0x71,
                y: 0x83,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0xfe71,
                sp: 0x1fa,
                p: 0x31,
            };
            machine.cpu.a = native.a;
            machine.cpu.x = native.x;
            machine.cpu.y = native.y;
            machine.cpu.k = 2;
            machine.cpu.pc = native.pc;
            machine.cpu.sp = native.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(native.p);
            let mut ram = machine.ram.clone();
            let tail = ram[0x146c0..0x14800].to_vec();
            let mut clock = CpuMasterTimeline::at_raster(
                0,
                CpuRasterPosition::new(v, h),
                CpuBusWorkload::default(),
                CpuFieldTiming::non_interlace(v == 261),
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = EnemyDamage::default();
            let mut count = 0;
            while !work.complete() {
                let label = format!("enemy pc={:04x}", native.pc);
                super::super::native_work::compare_instruction(
                    &label,
                    &mut native,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                assert_eq!(ram, source.snes().ram, "{label}");
                count += 1;
                assert!(count < 100000);
            }
            assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1fd));
            assert_eq!(&ram[0x146c0..0x14800], &tail);
            for i in 0..0x800 {
                assert_eq!(
                    &ram[0x16000 + i * 2..0x16002 + i * 2],
                    &[ram[0x14000 + i] >> 4, ram[0x14000 + i] & 15]
                );
            }
        }
    }
}
