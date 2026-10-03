//! Lamp cone scroll publication retains facing/doorway selection and clamps
//! through its source-width stores and actual far return.

#[cfg(test)]
mod tests {
    use super::super::{intro_clear::Access, native_work::compare_instruction};
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn lamp_cone_matches_source_guards_doorway_selection_clamps_and_scroll_stores() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x00f53e..0x00f566 {
            assert_eq!(
                table_byte(a),
                Some(rom[a as usize & 0x7fff]),
                "lamp table {a:06x}"
            );
        }
        let mut visited = BTreeSet::new();
        let mut instructions = 0;
        for dark in 0..2 {
            for submodule in [0, 15, 20] {
                for doorway in 0..4 {
                    for facing in [0, 2, 4, 6] {
                        for case in 0..64usize {
                            let mut machine = snes::Snes::new();
                            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                            let r = &mut machine.ram;
                            r[0x458] = dark;
                            r[0x11] = submodule;
                            r[0x6c] = doorway;
                            r[0x2f] = facing;
                            for (i, at) in
                                [0x20, 0x22, 0xe0, 0xe2, 0xe6, 0xe8].into_iter().enumerate()
                            {
                                let value = (case as u16)
                                    .wrapping_mul([1237, 997, 113, 2267, 229, 3617][i])
                                    .wrapping_add([0, 8, 0, 127, 255, 128][i]);
                                r[at..at + 2].copy_from_slice(&value.to_le_bytes());
                            }
                            r[0x1fc] = 0xe4;
                            r[0x1fd] = 0x87;
                            r[0x1fe] = 2;
                            let mut cpu = NativeNmiContext {
                                a: 0xab00,
                                x: 3,
                                y: 7,
                                d: 0,
                                db: 0,
                                bank: 0,
                                pc: 0xf567,
                                sp: 0x1fb,
                                p: 0x30 | (case as u8 & 1),
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
                            let timeline = CpuMasterTimeline::at_raster(
                                700 + case as u64,
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
                            let mut work = LampCone::default();
                            for _ in 0..200 {
                                if work.complete() {
                                    break;
                                }
                                visited.insert(cpu.pc);
                                compare_instruction(&format!("lamp dark={dark} sub={submodule} door={doorway} facing={facing} case={case} pc={:04x}",cpu.pc),&mut cpu,&mut clock,&mut source,&mut ram,&rom,|cpu,clock,bus|{
     work.step(cpu,clock,|at,access|{
      if let Access::Read(a,width)=access {if a&0x408000==0x8000 {for b in 0..width {
       let a=a+u32::from(b);assert_eq!(table_byte(a),Some(rom[(a>>16)as usize*0x8000+(a as usize&0x7fff)]),"lamp table read {a:06x}");
      }}}
      bus(at,access)
     }).unwrap();
    });
                                instructions += 1;
                            }
                            assert!(work.complete());
                            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87e5, 0x1fe));
                            assert_eq!(ram, source.snes().ram);
                        }
                    }
                }
            }
        }
        for pc in [
            0xf566, 0xf59f, 0xf5b0, 0xf5e9, 0xf5f2, 0xf5ff, 0xf600, 0xf632, 0xf63b, 0xf648,
        ] {
            assert!(visited.contains(&pc), "uncovered lamp path {pc:04x}");
        }
        eprintln!(
            "6144 lamp-cone fixtures, {instructions} instruction boundaries, {} distinct PCs",
            visited.len()
        );
    }
}

use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LampCone {
    pc: u32,
    complete: bool,
}
impl Default for LampCone {
    fn default() -> Self {
        Self {
            pc: 0x00f567,
            complete: false,
        }
    }
}
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    crate::ZeldaState::native_lamp_cone_table_byte(address)
}
impl LampCone {
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
            0xf566 | 0xf5ff | 0xf648 => {
                w.rtl();
                self.complete = true;
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0xf567 => {
                let (operand, next) = (0x0458, 0xf56a);
                w.drain(16);
                let value = w.read(w.absolute(operand), m);
                w.load(value, m);
                next
            }
            0xf56a | 0xf570 | 0xf581 | 0xf588 => {
                let (operand, next) = match pc {
                    0xf56a => (0xf566, 0xf56c),
                    0xf570 => (0xf566, 0xf572),
                    0xf581 => (0xf5b2, 0xf583),
                    0xf588 => (0xf59f, 0xf58a),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    operand
                } else {
                    next
                }
            }
            0xf56c | 0xf574 | 0xf57c | 0xf58a | 0xf591 | 0xf59f | 0xf5a6 | 0xf5b7 | 0xf5bf
            | 0xf5cb | 0xf5d3 | 0xf600 | 0xf608 | 0xf614 | 0xf61c => {
                let (operand, next) = match pc {
                    0xf56c => (0x0011, 0xf56e),
                    0xf574 => (0x002f, 0xf576),
                    0xf57c => (0x006c, 0xf57e),
                    0xf58a => (0x0000, 0xf58c),
                    0xf591 => (0x0022, 0xf593),
                    0xf59f => (0x0000, 0xf5a1),
                    0xf5a6 => (0x0020, 0xf5a8),
                    0xf5b7 => (0x0022, 0xf5b9),
                    0xf5bf => (0x00e2, 0xf5c1),
                    0xf5cb => (0x0020, 0xf5cd),
                    0xf5d3 => (0x00e8, 0xf5d5),
                    0xf600 => (0x0020, 0xf602),
                    0xf608 => (0x00e8, 0xf60a),
                    0xf614 => (0x0022, 0xf616),
                    0xf61c => (0x00e2, 0xf61e),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.load(value, m);
                next
            }
            0xf56e | 0xf58c | 0xf5a1 | 0xf5ab => {
                let (operand, next) = match pc {
                    0xf56e => (0x0014, 0xf570),
                    0xf58c => (0x0004, 0xf58f),
                    0xf5a1 => (0x0004, 0xf5a4),
                    0xf5ab => (0x0080, 0xf5ae),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0xf572 => {
                let (operand, next) = (0x0030, 0xf574);
                w.width(operand as u8, true);
                next
            }
            0xf576 | 0xf57e | 0xf583 | 0xf597 | 0xf5a8 => {
                let (operand, next) = match pc {
                    0xf576 => (0x00ff, 0xf579),
                    0xf57e => (0x00ff, 0xf581),
                    0xf583 => (0x00fe, 0xf586),
                    0xf597 => (0x00ff, 0xf59a),
                    0xf5a8 => (0x00ff, 0xf5ab),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(w.cpu.a & value & mask, m);
                next
            }
            0xf579 | 0xf5bd | 0xf5c9 | 0xf5d1 | 0xf5fb | 0xf606 | 0xf612 | 0xf61a | 0xf644 => {
                let (operand, next) = match pc {
                    0xf579 => (0x0000, 0xf57b),
                    0xf5bd => (0x0000, 0xf5bf),
                    0xf5c9 => (0x00e0, 0xf5cb),
                    0xf5d1 => (0x0000, 0xf5d3),
                    0xf5fb => (0x00e6, 0xf5fd),
                    0xf606 => (0x0000, 0xf608),
                    0xf612 => (0x00e6, 0xf614),
                    0xf61a => (0x0000, 0xf61c),
                    0xf644 => (0x00e0, 0xf646),
                    _ => unreachable!(),
                };
                w.dp_store(operand, w.cpu.a & mask, m);
                next
            }
            0xf57b | 0xf587 | 0xf59c => {
                let next = match pc {
                    0xf57b => 0xf57c,
                    0xf587 => 0xf588,
                    0xf59c => 0xf59d,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.x = w.cpu.a & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0xf586 => {
                let next = 0xf587;
                w.shift_left(m);
                next
            }
            0xf58f | 0xf5b5 => {
                let (operand, next) = match pc {
                    0xf58f => (0xf59c, 0xf591),
                    0xf5b5 => (0xf600, 0xf5b7),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & CARRY != 0) {
                    operand
                } else {
                    next
                }
            }
            0xf593 | 0xf5c4 | 0xf5d8 | 0xf5dd | 0xf5e2 | 0xf60d | 0xf621 | 0xf626 | 0xf62b => {
                let next = match pc {
                    0xf593 => 0xf594,
                    0xf5c4 => 0xf5c5,
                    0xf5d8 => 0xf5d9,
                    0xf5dd => 0xf5de,
                    0xf5e2 => 0xf5e3,
                    0xf60d => 0xf60e,
                    0xf621 => 0xf622,
                    0xf626 => 0xf627,
                    0xf62b => 0xf62c,
                    _ => unreachable!(),
                };
                w.clear_carry();
                next
            }
            0xf594 => {
                let (operand, next) = (0x0008, 0xf597);
                w.drain(8 * u32::from(m));
                let value = operand;
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0xf59a | 0xf59d => {
                let (operand, next) = match pc {
                    0xf59a => (0xf5ab, 0xf59c),
                    0xf59d => (0xf5b2, 0xf59f),
                    _ => unreachable!(),
                };
                if w.branch(true) {
                    operand
                } else {
                    next
                }
            }
            0xf5a4 | 0xf5ae | 0xf5f0 | 0xf639 => {
                let (operand, next) = match pc {
                    0xf5a4 => (0xf59c, 0xf5a6),
                    0xf5ae => (0xf5b2, 0xf5b0),
                    0xf5f0 => (0xf5f6, 0xf5f2),
                    0xf639 => (0xf63f, 0xf63b),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & CARRY == 0) {
                    operand
                } else {
                    next
                }
            }
            0xf5b0 | 0xf5b1 => {
                let next = match pc {
                    0xf5b0 => 0xf5b1,
                    0xf5b1 => 0xf5b2,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & xmask;
                w.flags(w.cpu.x, x);
                next
            }
            0xf5b2 => {
                let (operand, next) = (0x0004, 0xf5b5);
                w.drain(8 * u32::from(x));
                let value = operand;
                let left = w.cpu.x & xmask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & xmask, x);
                next
            }
            0xf5b9 | 0xf5c1 | 0xf5cd | 0xf5d5 | 0xf5f6 | 0xf602 | 0xf60a | 0xf616 | 0xf61e
            | 0xf63f => {
                let next = match pc {
                    0xf5b9 => 0xf5ba,
                    0xf5c1 => 0xf5c2,
                    0xf5cd => 0xf5ce,
                    0xf5d5 => 0xf5d6,
                    0xf5f6 => 0xf5f7,
                    0xf602 => 0xf603,
                    0xf60a => 0xf60b,
                    0xf616 => 0xf617,
                    0xf61e => 0xf61f,
                    0xf63f => 0xf640,
                    _ => unreachable!(),
                };
                w.drain(6);
                w.cpu.p |= CARRY;
                next
            }
            0xf5ba | 0xf5ce | 0xf603 | 0xf617 => {
                let (operand, next) = match pc {
                    0xf5ba => (0x0077, 0xf5bd),
                    0xf5ce => (0x0058, 0xf5d1),
                    0xf603 => (0x0072, 0xf606),
                    0xf617 => (0x0058, 0xf61a),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.sbc(value, m);
                next
            }
            0xf5c2 | 0xf5d6 | 0xf60b | 0xf61f => {
                let (operand, next) = match pc {
                    0xf5c2 => (0x0000, 0xf5c4),
                    0xf5d6 => (0x0000, 0xf5d8),
                    0xf60b => (0x0000, 0xf60d),
                    0xf61f => (0x0000, 0xf621),
                    _ => unreachable!(),
                };
                let value = w.dp_read(operand, m);
                w.sbc(value, m);
                next
            }
            0xf5c5 | 0xf5d9 | 0xf5de | 0xf5e3 | 0xf60e | 0xf622 | 0xf627 | 0xf62c => {
                let (operand, next) = match pc {
                    0xf5c5 => (0xf53e, 0xf5c9),
                    0xf5d9 => (0xf546, 0xf5dd),
                    0xf5de => (0xf54e, 0xf5e2),
                    0xf5e3 => (0xf556, 0xf5e7),
                    0xf60e => (0xf546, 0xf612),
                    0xf622 => (0xf53e, 0xf626),
                    0xf627 => (0xf54e, 0xf62b),
                    0xf62c => (0xf556, 0xf630),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand + u32::from(w.cpu.x), m);
                if m == 1 {
                    w.adc_byte(value);
                } else {
                    w.adc(value);
                }
                next
            }
            0xf5e7 | 0xf630 => {
                let (operand, next) = match pc {
                    0xf5e7 => (0xf5ec, 0xf5e9),
                    0xf630 => (0xf635, 0xf632),
                    _ => unreachable!(),
                };
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    operand
                } else {
                    next
                }
            }
            0xf5e9 | 0xf632 => {
                let (operand, next) = match pc {
                    0xf5e9 => (0x0000, 0xf5ec),
                    0xf632 => (0x0000, 0xf635),
                    _ => unreachable!(),
                };
                w.drain(8 * u32::from(m));
                let value = operand;
                w.load(value, m);
                next
            }
            0xf5ec | 0xf635 => {
                let (operand, next) = match pc {
                    0xf5ec => (0xf55e, 0xf5f0),
                    0xf635 => (0xf55e, 0xf639),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand + u32::from(w.cpu.x), m);
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & mask, m);
                next
            }
            0xf5f2 | 0xf63b => {
                let (operand, next) = match pc {
                    0xf5f2 => (0xf55e, 0xf5f6),
                    0xf63b => (0xf55e, 0xf63f),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand + u32::from(w.cpu.x), m);
                w.load(value, m);
                next
            }
            0xf5f7 | 0xf640 => {
                let (operand, next) = match pc {
                    0xf5f7 => (0xf556, 0xf5fb),
                    0xf640 => (0xf556, 0xf644),
                    _ => unreachable!(),
                };
                w.drain(24);
                let value = w.read(operand + u32::from(w.cpu.x), m);
                w.sbc(value, m);
                next
            }
            0xf5fd | 0xf646 => {
                let (operand, next) = match pc {
                    0xf5fd => (0x0030, 0xf5ff),
                    0xf646 => (0x0030, 0xf648),
                    _ => unreachable!(),
                };
                w.width(operand as u8, false);
                next
            }
            _ => unreachable!("lamp cone continuation {pc:06x}"),
        };
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}
