//! Lower garnish wrapper/loop and selected countdown/allocation/dispatch.
use super::{garnish_single::{GarnishSingle,GarnishSingleTail},native_work::WorkBus,nmi_stack::NativeNmiContext,sprite_main::{SpriteChild,SpriteMain}};
use snes::CpuMasterTimeline;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum LowerGarnishTail { Returned, Body {slot:u8,kind:u8,pc:u16} }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct DungeonLowerGarnish {work:SpriteMain,single:Option<GarnishSingle>,tail:Option<LowerGarnishTail>,entry_sp:Option<u16>}
impl Default for DungeonLowerGarnish {
    fn default()->Self {Self {work:SpriteMain::lower_garnish(),single:None,tail:None,entry_sp:None}}
}
impl DungeonLowerGarnish {
    pub(crate) fn tail(&self)->Option<LowerGarnishTail> {self.tail}
    pub(crate) fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        if self.tail.is_some() {return None;}
        let entry_sp = *self.entry_sp.get_or_insert(cpu.sp);
        if let Some(work)=&mut self.single {
            let cycles=work.step(cpu,clock,bus);
            match work.tail() {
                Some(GarnishSingleTail::Returned)=>{self.work.return_from_child(cpu);self.single=None;},
                Some(GarnishSingleTail::Body {kind,pc})=>{assert_eq!((cpu.bank,cpu.db,cpu.sp),(9,9,entry_sp.wrapping_sub(8)));self.tail=Some(LowerGarnishTail::Body {slot:cpu.x as u8,kind,pc});},
                None=>{},
            }
            return cycles;
        }
        let cycles=self.work.step(cpu,clock,bus);
        match self.work.child() {
            Some(SpriteChild::Garnish)=>{assert_eq!((cpu.bank,cpu.pc,cpu.db,cpu.sp),(9,0xb0bf,9,entry_sp.wrapping_sub(8)));assert!(cpu.x<15);self.single=Some(GarnishSingle::new(cpu.a as u8));},
            Some(child)=>unreachable!("lower garnish child {child:?}"),
            None if (cpu.bank,cpu.pc)==(6,0x83ae)=>{assert_eq!((cpu.sp,cpu.db,cpu.p&0x30),(entry_sp,6,0x30));self.tail=Some(LowerGarnishTail::Returned);},
            None=>{},
        }
        cycles
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r:&mut[u8],case:usize) {
    let k=case%15;let v=case/15;
    r[0xfb4]=[0,1,255][v/22%3];
    r[0x1f800..0x1f81e].fill(0);
    r[0x1f800+k]=(v%22+1) as u8;
    r[0x1f90e+k]=[0,1,2,7,128,255][v/66%6];
    r[0xfb3]=(v/396%2) as u8;
    r[0x1f968+k]=(v/792%2) as u8;
    r[0x11]=[0,1,17][v/1584%3];r[0xfc1]=[0,128][v/4752%2];
    r[0xfe0..0xff8].fill(0);
    r[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());r[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lower_garnish_matches_source_countdown_allocation_dispatch_and_real_loop_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 2];
        let mut slots = std::collections::BTreeSet::new();
        let mut kinds = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..142560usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: 255,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x83aa,
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
                snes::CpuRasterPosition::new(73, 278),
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
            let mut work = DungeonLowerGarnish::default();
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
                LowerGarnishTail::Returned=>tails[0]+=1,
                LowerGarnishTail::Body {slot,kind,..}=>{tails[1]+=1;slots.insert(slot);kinds.insert(kind);},
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [0x0683aa,0x09b07f,0x09b087,0x09b099,0x09b0bf,0x09b0ca,0x09b0d7,0x09b0f0,0x09b0fe,0x09b10c,0x09b11c,0x09b123,0x09b09f,0x09b08a,0x09b08b] {assert!(visited.contains(&pc),"{pc:06x}");}
        assert!(tails.iter().all(|n|*n>0));assert_eq!(slots.len(),15);assert_eq!(kinds.len(),22);
        println!("native lower garnish: 142560 cases, {count} source instructions, {} PCs, tails={tails:?}, all15 slots and22 kinds and three allocator entries",visited.len());
    }
}
