//! Real overlord wrapper, slot loop and boulder gates after Ancilla_Main.
use super::{native_work::WorkBus, nmi_stack::NativeNmiContext, sprite_main::{SpriteChild, SpriteMain}};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverlordTail { Returned, Slot { slot: u8, kind: u8 }, Boulder }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonOverlord { work: SpriteMain, tail: Option<OverlordTail>, entry_sp: Option<u16> }
impl Default for DungeonOverlord {
    fn default() -> Self { Self { work: SpriteMain::after_ancilla(), tail: None, entry_sp: None } }
}
impl DungeonOverlord {
    pub(crate) fn tail(&self) -> Option<OverlordTail> { self.tail }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline, bus: impl WorkBus) -> Option<u64> {
        if self.tail.is_some() { return None; }
        let entry_sp = *self.entry_sp.get_or_insert(cpu.sp);
        let cycles = self.work.step(cpu, clock, bus);
        self.tail = match self.work.child() {
            Some(SpriteChild::Overlord) => {
                assert_eq!((cpu.bank,cpu.pc,cpu.sp,cpu.db),(9,0xb793,entry_sp.wrapping_sub(8),9));
                assert!(cpu.x < 8);
                Some(OverlordTail::Slot { slot: cpu.x as u8, kind: cpu.a as u8 })
            },
            Some(SpriteChild::OverworldSpawner) => {
                assert_eq!((cpu.bank,cpu.pc,cpu.sp,cpu.db),(9,0xb724,entry_sp.wrapping_sub(6),9));
                Some(OverlordTail::Boulder)
            },
            Some(child) => unreachable!("overlord caller child {child:?}"),
            None if (cpu.bank,cpu.pc)==(6,0x83a1) => {
                assert_eq!((cpu.sp,cpu.db,cpu.x,cpu.p & 0x30),(entry_sp,6,15,0x30));
                Some(OverlordTail::Returned)
            },
            None => None,
        };
        cycles
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    r[0xb00..0xb08].fill(0);
    r[0x1b]=(case%2) as u8;
    r[0x11]=[0,0,0,1,15,255][case/2%6];
    r[0xfc1]=[0,0,1,128][case/12%4];
    r[0xffd]=[0,1,255][case/48%3];
    r[0xffe]=[0,1,62,63,127,255][case/144%6];
    r[0xb9a]=case as u8;
    if case/864%2!=0 { r[0xb00+case%8]=(case/8%26+1) as u8; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_overlord_matches_source_all_guards_slots_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 3];
        let mut slots = std::collections::BTreeSet::new();
        let mut kinds = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..8192usize {
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
                pc: 0x8398,
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
                snes::CpuRasterPosition::new(68, 1262),
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
            let mut work = DungeonOverlord::default();
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
                OverlordTail::Returned=>tails[0]+=1,
                OverlordTail::Boulder=>tails[1]+=1,
                OverlordTail::Slot {slot,kind}=>{tails[2]+=1;slots.insert(slot);kinds.insert(kind);},
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [0x068398,0x09b773,0x09b776,0x09b783,0x09b785,0x09b787,0x09b78a,0x09b78c,0x09b78f,0x09b790,0x09b792,0x09b714,0x09b716,0x09b71b,0x09b722,0x09b77c,0x09b77d,0x06839c,0x06839f] {
            assert!(visited.contains(&pc),"{pc:06x}");
        }
        assert!(tails.iter().all(|n|*n>0));assert_eq!(slots.len(),8);assert_eq!(kinds.len(),26);
        println!("native dungeon overlord: 8192 cases, {count} source instructions, {} PCs, tails={tails:?}, all8 slots and26 selected kinds",visited.len());
    }
}
