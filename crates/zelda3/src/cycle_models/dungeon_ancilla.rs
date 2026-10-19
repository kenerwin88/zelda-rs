//! Post-follower Sprite_Main stores and the real Ancilla_Main wrapper/dispatch.
//! Existing blanket work stays native; other bodies retain their actual stack.
use super::{
    native_work::WorkBus,
    nmi_stack::NativeNmiContext,
    sprite_main::{SpriteChild, SpriteMain},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AncillaTail {
    ReceiptOamReturned {
        slot: u8,
    },
    ReceiptDrawing {
        slot: u8,
        tail: super::receipt_drawing::ReceiptDrawingTail,
    },
    Receipt {
        slot: u8,
        tail: super::receipt_prefix::ReceiptTail,
    },
    Returned,
    Spark,
    Body {
        slot: u8,
        kind: u8,
        pc: u16,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonAncilla {
    entry_sp: Option<u16>,
    native_slot_return: bool,
    native_oam: bool,
    oam: Option<super::receipt_oam::ReceiptOam>,
    native_receipt: bool,
    receipt: Option<super::receipt_prefix::ReceiptPrefix>,
    drawing: Option<super::receipt_drawing::ReceiptDrawing>,
    work: SpriteMain,
    kind: u8,
    tail: Option<AncillaTail>,
}
impl Default for DungeonAncilla {
    fn default() -> Self {
        Self {
            entry_sp: None,
            native_slot_return: false,
            native_oam: false,
            oam: None,
            native_receipt: false,
            receipt: None,
            drawing: None,
            work: SpriteMain::after_follower(),
            kind: 0,
            tail: None,
        }
    }
}
impl DungeonAncilla {
    pub(crate) fn with_native_slot_return(mut self, enabled: bool) -> Self {
        self.native_slot_return = enabled;
        self
    }
    pub(crate) fn with_native_oam(mut self, enabled: bool) -> Self {
        self.native_oam = enabled;
        self
    }
    pub(crate) fn with_native_receipt(mut self, enabled: bool) -> Self {
        self.native_receipt = enabled;
        self
    }
    pub(crate) fn tail(&self) -> Option<AncillaTail> {
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
        let entry_sp = *self.entry_sp.get_or_insert(cpu.sp);
        if let Some(work) = &mut self.oam {
            let cycles = work.step(cpu, clock, bus);
            if work.complete() {
                if self.native_slot_return {
                    // ReceiptOam has executed the handler's actual RTS. The
                    // existing caller owns DEX/BPL and the wrapper's real return.
                    self.work.return_from_child(cpu);
                    self.oam = None;
                    self.drawing = None;
                    self.receipt = None;
                } else {
                    self.tail = Some(AncillaTail::ReceiptOamReturned { slot: cpu.x as u8 });
                }
            }
            return cycles;
        }
        if let Some(work) = &mut self.drawing {
            let cycles = work.step(cpu, clock, bus);
            if self.native_oam
                && work.tail() == Some(super::receipt_drawing::ReceiptDrawingTail::Draw)
            {
                self.oam = Some(Default::default());
            } else {
                self.tail = work.tail().map(|tail| AncillaTail::ReceiptDrawing {
                    slot: cpu.x as u8,
                    tail,
                });
            }
            return cycles;
        }
        if let Some(work) = &mut self.receipt {
            let cycles = work.step(cpu, clock, bus);
            if work.tail() == Some(super::receipt_prefix::ReceiptTail::Draw) {
                self.drawing = Some(Default::default());
            } else {
                self.tail = work.tail().map(|tail| AncillaTail::Receipt {
                    slot: cpu.x as u8,
                    tail,
                });
            }
            return cycles;
        }
        let before = (cpu.bank, cpu.pc);
        let cycles = self.work.step(cpu, clock, bus);
        if before == (8, 0x8330) {
            self.kind = cpu.a as u8;
        }
        self.tail = match self.work.child() {
            Some(SpriteChild::AncillaSpark) => {
                assert_eq!((cpu.bank, cpu.pc, cpu.db, cpu.sp), (8, 0x8f8e, 8, entry_sp.wrapping_sub(6)));
                Some(AncillaTail::Spark)
            }
            Some(SpriteChild::Ancilla) => {
                let pc = self
                    .work
                    .ancilla_child()
                    .expect("actual selected ancilla handler");
                assert_eq!((cpu.bank, cpu.pc, cpu.db, cpu.sp), (8, pc, 8, entry_sp.wrapping_sub(8)));
                assert!(cpu.x < 10);
                if self.native_receipt && self.kind == 0x22 {
                    assert_eq!(pc, 0xc38a);
                    self.receipt = Some(
                        super::receipt_prefix::ReceiptPrefix::default().with_native_movement(true),
                    );
                    None
                } else {
                    Some(AncillaTail::Body {
                        slot: cpu.x as u8,
                        kind: self.kind,
                        pc,
                    })
                }
            }
            Some(child) => unreachable!("post-follower child {child:?}"),
            None if (cpu.bank, cpu.pc) == (6, 0x8398) => {
                assert_eq!((cpu.sp, cpu.db, cpu.p & 0x30), (entry_sp, 6, 0x30));
                Some(AncillaTail::Returned)
            }
            None => None,
        };
        Some(cycles.unwrap())
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    r[0x314] = (case * 7) as u8;
    r[0x47] = case as u8;
    r[0xfdc] = (case / 16) as u8;
    r[0xfac] = if case / 680 % 4 == 3 {
        [1, 2, 3, 128, 255][case % 5]
    } else {
        0
    };
    r[0x11] = [0, 1, 15, 17][case / 10 % 4];
    r[0xfb3] = (case / 680 % 2) as u8;
    for k in 0..10 {
        r[0xc4a + k] = 0;
        r[0xc68 + k] = [0, 1, 128, 255][case / 40 % 4];
        r[0xc90 + k] = [0, 4, 8, 16][case / 160 % 4];
        r[0xc7c + k] = (case / 320 % 2) as u8;
        r[0xc04 + k] = 128;
        r[0xc18 + k] = 0;
        r[0xbfa + k] = 96;
        r[0xc0e + k] = 0;
    }
    r[0xc4a + case % 10] = (case / 10 % 68) as u8;
    for a in [0xe2, 0xe8] {
        r[a..a + 2].fill(0);
    }
    r[0xfe0..0xff8].fill(0);
    r[0xdd0] = 8;
    r[0xe20] = 0x73;
    r[0x37d] = 0;
}
#[cfg(test)]
pub(crate) fn seed_slot_return_case(r: &mut [u8], case: usize) {
    seed_case(r, case * 17);
    r[0x11] = 0;
    r[0xfac] = 0;
    r[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
    r[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
    for k in 0..10 {
        r[0xc4a + k] = if k < case % 11 { 0x22 } else if (case + k) % 3 == 0 { 0x20 } else { 0 };
        r[0xc54 + k] = 1;
        r[0x3b1 + k] = 56;
        r[0xc5e + k] = [9,0x17,0x42,2,3,4,5,6][(case+k)%8];
        r[0x3a4 + k] = 0;
        r[0x39f + k] = 0;
        r[0xc68 + k] = [0,1,3,7][(case+k)%4];
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_ancilla_matches_source_setup_allocators_timers_dispatch_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 3];
        let mut slots = std::collections::BTreeSet::new();
        let mut kinds = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..4096usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: 3,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x8369,
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
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(56, 946),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                clock,
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            let mut clock = source.timeline().clone();
            let mut work = DungeonAncilla::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("ancilla caller case={case} pc={pc:04x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                count += 1;
            }
            match work.tail().unwrap() {
                AncillaTail::ReceiptOamReturned { .. }
                | AncillaTail::ReceiptDrawing { .. }
                | AncillaTail::Receipt { .. } => unreachable!(),
                AncillaTail::Returned => tails[0] += 1,
                AncillaTail::Spark => tails[1] += 1,
                AncillaTail::Body { slot, kind, .. } => {
                    tails[2] += 1;
                    slots.insert(slot);
                    kinds.insert(kind);
                }
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [
            0x068369, 0x06837d, 0x068381, 0x068391, 0x068394, 0x088242, 0x088f8c, 0x08832d,
            0x088335, 0x08833c, 0x08835a, 0x08836b, 0x08837c, 0x08833b, 0x08824c,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        assert!(tails.iter().all(|n| *n > 0));
        assert_eq!(slots.len(), 10);
        assert_eq!(kinds.len(), 66);
        println!("native dungeon ancilla: 4096 cases, {count} source instructions, {} PCs, tails={tails:?}, all 10 slots and 66 selected kinds",visited.len());
    }
    #[test]
    fn dungeon_ancilla_slot_return_matches_source_multiple_receipts_blankets_and_real_wrapper_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 3];
        let mut slots = std::collections::BTreeSet::new();
        let mut kinds = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..1024usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_slot_return_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: 3,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x8369,
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
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(56, 946),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                clock,
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            let mut clock = source.timeline().clone();
            let mut work = DungeonAncilla::default().with_native_receipt(true).with_native_oam(true).with_native_slot_return(true);
            while work.tail().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("ancilla caller case={case} pc={pc:04x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                count += 1;
            }
            match work.tail().unwrap() {
                AncillaTail::ReceiptOamReturned { .. }
                | AncillaTail::ReceiptDrawing { .. }
                | AncillaTail::Receipt { .. } => unreachable!(),
                AncillaTail::Returned => tails[0] += 1,
                AncillaTail::Spark => tails[1] += 1,
                AncillaTail::Body { slot, kind, .. } => {
                    tails[2] += 1;
                    slots.insert(slot);
                    kinds.insert(kind);
                }
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [0x08c38a,0x08c690,0x08c6f1,0x088338,0x088339,0x08833b,0x08824b,0x08824c] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        assert_eq!(tails, [1024,0,0]);
        println!("native ancilla slot return: 1024 cases, {count} source instructions, {} PCs, zero through ten receipts with blanket slots and real wrapper return",visited.len());
    }
}
