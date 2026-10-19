//! Dungeon camera updates on the caller's live CPU, including signed velocity
//! loops, half-rate BG1 carry, scratch stores and the shared mirror/RTS tail.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, CARRY, ZERO}};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonCamera { pc: u16, complete: bool }

impl Default for DungeonCamera {
    fn default() -> Self { Self { pc: 0xba31, complete: false } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dungeon_camera_matches_each_source_instruction_for_signed_motion_and_mirror_tails() {
        let rom=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
        let mut visited=std::collections::BTreeSet::new();
        for vy in [0u8,1,3,127,128,253,255] {
            for vx in [0u8,1,3,127,128,253,255] {
                for case in 0..24 {
                    let mut machine=snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
                    seed_case(&mut machine.ram,vy,vx,case);
                    machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xc0,0x87]);
                    let mut cpu=NativeNmiContext {a:0x1100,x:0,y:6,d:0,db:0,bank:2,pc:0xba31,sp:0x1fa,p:0x73};
                    machine.cpu.a=cpu.a; machine.cpu.x=cpu.x; machine.cpu.y=cpu.y;
                    machine.cpu.k=cpu.bank; machine.cpu.pc=cpu.pc; machine.cpu.sp=cpu.sp;
                    machine.cpu.e=false; machine.cpu.unpack_flags(cpu.p);
                    let mut ram=machine.ram.clone();
                    let clock=CpuMasterTimeline::at_raster(17,snes::CpuRasterPosition::new(224,1300),
                        snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
                    let mut source=snes::RomCpuTimingProbe::new(machine,clock,snes::SourcePpuReadState::snes9x_reset()).unwrap();
                    let mut clock=source.timeline().clone();
                    let mut work=DungeonCamera::default();
                    for _ in 0..15000 {
                        if work.complete() {break;}
                        visited.insert(cpu.pc);
                        super::super::native_work::compare_instruction(
                            &format!("camera vy={vy} vx={vx} case={case} pc={:04x}",cpu.pc),
                            &mut cpu,&mut clock,&mut source,&mut ram,&rom,
                            |cpu,clock,bus| {work.step(cpu,clock,bus).unwrap();});
                    }
                    assert!(work.complete());
                    assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,0x87c1,0x1fc));
                    assert_eq!(ram,source.snes().ram);
                }
            }
        }
        for pc in [0xba46,0xba67,0xba83,0xba8c,0xba97,0xbaaf,0xbab2,0xbaf2,0xbb0e,0xbb17,0xbb22,0xbb3a,0xbb3d,0xbb7b,0xbb85] {
            assert!(visited.contains(&pc),"unexercised camera branch {pc:04x}");
        }
    }
}

impl DungeonCamera {
    pub(crate) fn complete(&self) -> bool { self.complete }

    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline,
        bus: impl WorkBus) -> Option<u64> {
        if self.complete { return None; }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x18), (2, self.pc, 0, 0, 0x10));
        let mut w = Work { cpu, clock, bus, cycles: 0 };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xba31 | 0xbb7b | 0xbb85 => { w.width(0x20, pc != 0xbb85); pc + 2 }
            0xba33 | 0xba46 | 0xbad4 => { w.immediate(if pc == 0xba46 {0} else {1}, 2); pc + 3 }
            0xba38 | 0xba3f | 0xba4b | 0xba59 | 0xba72 | 0xba85 | 0xba90 | 0xba9c
            | 0xbaa3 | 0xbabd | 0xbad9 | 0xbae4 | 0xbafd | 0xbb10 | 0xbb1b | 0xbb27
            | 0xbb2e | 0xbb48 | 0xbb5f | 0xbb7d | 0xbb81 => {
                let address = match pc {
                    0xba38 => 0x78, 0xba3f => 0x24, 0xba4b => 0x20,
                    0xba59 | 0xba72 => 0x30, 0xba85 | 0xbb10 => 0x0e,
                    0xba90 | 0xbb81 => 0xe8, 0xba9c | 0xbb27 | 0xbb5f => 0xa0,
                    0xbaa3 | 0xbb2e => 0, 0xbabd => 0xe6, 0xbad9 => 0x22,
                    0xbae4 | 0xbafd => 0x31, 0xbb1b | 0xbb7d => 0xe2, 0xbb48 => 0xe0,
                    _ => unreachable!(),
                };
                let value = w.dp_read(address, 2); w.load(value, 2); pc + 2
            }
            0xba36 | 0xba49 | 0xba57 | 0xba9a | 0xbab2 | 0xbac1 | 0xbad7 | 0xbae2
            | 0xbb25 | 0xbb3d | 0xbb4c | 0xbb7f | 0xbb83 => {
                let address = match pc {
                    0xba36 | 0xbad7 => 0, 0xba49 | 0xba57 | 0xbae2 => 0x0e,
                    0xba9a => 0xe8, 0xbab2 | 0xbb3d => 6, 0xbac1 | 0xbb83 => 0xe6,
                    0xbb25 => 0xe2, 0xbb4c | 0xbb7f => 0xe0, _ => unreachable!(),
                };
                w.dp_store(address, w.cpu.a, 2); pc + 2
            }
            0xba3a | 0xba50 | 0xba5b | 0xba74 | 0xbadb | 0xbae6 | 0xbaff => {
                w.mask(if matches!(pc,0xba50|0xbadb) {0x1ff} else {0xff}, 2); pc + 3
            }
            0xba41 | 0xba62 | 0xba77 | 0xba9e | 0xbaaa | 0xbaed | 0xbb02 | 0xbb29 | 0xbb35 | 0xbb61 => {
                let value = match pc {
                    0xba41 | 0xba9e | 0xbb29 | 0xbb61 => 0xffff,
                    0xbaaa | 0xbb35 => 0x7000, _ => 0x80,
                };
                w.compare_immediate(value, 2); pc + 3
            }
            0xba3d | 0xba44 | 0xba5e | 0xba65 | 0xba7a | 0xba81 | 0xba83 | 0xba8a | 0xba95
            | 0xbaa1 | 0xbaad | 0xbad2 | 0xbae9 | 0xbaf0 | 0xbb05 | 0xbb0c | 0xbb0e | 0xbb15
            | 0xbb20 | 0xbb2c | 0xbb38 | 0xbb5d | 0xbb64 | 0xbb69 | 0xbb6d | 0xbb71 | 0xbb75 | 0xbb79 => {
                let z = w.cpu.p & ZERO != 0; let c = w.cpu.p & CARRY != 0;
                let (take,target) = match pc {
                    0xba3d => (z,0xba49), 0xba44 => (!z,0xba49), 0xba5e => (z,0xbad4),
                    0xba65 => (!c,0xba6f), 0xba7a => (!c,0xba85), 0xba81 => (c,0xba90),
                    0xba83 | 0xba8a => (!c,0xbad1), 0xba95 | 0xbaa1 => (z,0xbad1),
                    0xbaad => (!c,0xbab2), 0xbad2 => (!z,0xba70), 0xbae9 => (z,0xbb5f),
                    0xbaf0 => (!c,0xbafa), 0xbb05 => (!c,0xbb10), 0xbb0c => (c,0xbb1b),
                    0xbb0e | 0xbb15 => (!c,0xbb5c), 0xbb20 | 0xbb2c => (z,0xbb5c),
                    0xbb38 => (!c,0xbb3d), 0xbb5d => (!z,0xbafb), 0xbb64 => (z,0xbb85),
                    0xbb69 | 0xbb71 | 0xbb75 => (z,0xbb7b), 0xbb6d => (c,0xbb7b),
                    0xbb79 => (!z,0xbb85), _ => unreachable!(),
                };
                if w.branch(take) {target} else {pc+2}
            }
            0xba4d => { w.drain(6); w.cpu.p |= CARRY; pc + 1 }
            0xba4e => { let value=w.dp_read(0x0e,2); w.sbc(value,2); pc+2 }
            0xba53 | 0xba97 | 0xbab7 | 0xbac6 | 0xbade | 0xbb22 | 0xbb42 | 0xbb51 => { w.clear_carry(); pc+1 }
            0xba54 | 0xbadf => { w.drain(16); w.adc(if pc==0xba54 {12} else {8}); pc+3 }
            0xba60 | 0xba70 | 0xbaeb | 0xbafb => {
                w.cpu.x=w.dp_read(if pc<0xbaeb {0xa7} else {0xa6},1); w.flags(w.cpu.x,1); pc+2
            }
            0xba67 | 0xbaf2 => { w.drain(16); w.load(w.cpu.a ^ 0xff,2); pc+3 }
            0xba6a | 0xbacc | 0xbacd | 0xbaf5 | 0xbb57 | 0xbb58 => { w.increment_a(2); pc+1 }
            0xba6b | 0xba6d | 0xbaf6 | 0xbaf8 => { w.dp_increment(0,2,false); pc+2 }
            0xba6f | 0xbafa => { w.drain(6); w.cpu.y=w.cpu.a & 255; w.flags(w.cpu.y,1); pc+1 }
            0xba7c | 0xbab4 | 0xbac3 | 0xbb07 | 0xbb3f | 0xbb4e => {
                let address=match pc { 0xba7c|0xbac3=>0x618, 0xbab4=>0x622, 0xbb07|0xbb4e=>0x61c, _=>0x620 };
                w.drain(16); let value=w.read(address,2); w.load(value,2); pc+3
            }
            0xba7f | 0xbb0a | 0xba87 | 0xbb12 | 0xba92 | 0xbb1d => {
                let value=match pc {
                    0xba7f | 0xbb0a => w.dp_read(0x0e,2),
                    0xba87 | 0xbb12 => { w.drain(16); w.read(if pc==0xba87 {0x61a} else {0x61e},2) }
                    _ => {
                        let base=if pc==0xba92 {0x600} else {0x608};
                        w.drain(16);
                        if (base&255)+u32::from(w.cpu.x)>255 {w.drain(6);}
                        w.read(base+u32::from(w.cpu.x),2)
                    }
                };
                w.cpu.p=(w.cpu.p & !CARRY)|u8::from(w.cpu.a>=value); w.flags(w.cpu.a.wrapping_sub(value),2);
                pc+if matches!(pc,0xba7f|0xbb0a) {2} else {3}
            }
            0xba8c..=0xba8f | 0xbb17..=0xbb1a => { w.drain(6); w.cpu.x=(w.cpu.x+1)&255; w.flags(w.cpu.x,1); pc+1 }
            0xba98 | 0xbab8 | 0xbabf | 0xbac7 | 0xbb23 | 0xbb43 | 0xbb4a | 0xbb52 => {
                let address=match pc {0xbab8|0xbb43=>4, 0xbabf|0xbb4a=>6, _=>0};
                let value=w.dp_read(address,2); w.adc(value); pc+2
            }
            0xbaa5 | 0xbb30 => { w.dp_store(4,0,2); pc+2 }
            0xbaa7 | 0xbb32 => { w.drain(6); let old=w.cpu.a; w.cpu.p=(w.cpu.p&!CARRY)|(old as u8&1); w.load(old>>1,2); pc+1 }
            0xbaa8 | 0xbb33 => {
                let old=w.dp_read(4,2); let value=(old>>1)|if w.cpu.p&CARRY!=0 {0x8000} else {0};
                w.cpu.p=(w.cpu.p&!CARRY)|(old as u8&1); w.drain(6); w.flags(value,2); w.write_high_low(4,value,2); pc+2
            }
            0xbaaf | 0xbb3a => { w.drain(16); w.load(w.cpu.a|0xf000,2); pc+3 }
            0xbaba | 0xbac9 | 0xbace | 0xbb45 | 0xbb54 | 0xbb59 => {
                let address=match pc {0xbaba=>0x622,0xbac9=>0x618,0xbace=>0x61a,0xbb45=>0x620,0xbb54=>0x61c,_=>0x61e};
                w.drain(16); w.write(address,w.cpu.a,2); pc+3
            }
            0xbad1 | 0xbb5c => { w.drain(6); w.cpu.y=w.cpu.y.wrapping_sub(1)&255; w.flags(w.cpu.y,1); pc+1 }
            0xbb66 => { w.drain(16); w.cpu.x=w.read(0x414,1); w.flags(w.cpu.x,1); pc+3 }
            0xbb6b | 0xbb6f | 0xbb73 | 0xbb77 => { w.compare_x_immediate(match pc {0xbb6b=>6,0xbb6f=>4,0xbb73=>3,_=>2},1); pc+2 }
            0xbb87 => { w.rts(); self.complete=true; w.cpu.pc }
            _ => unreachable!("dungeon camera {pc:04x}"),
        };
        w.cpu.pc=self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(ram:&mut [u8],vy:u8,vx:u8,case:usize) {
                    for (address,value) in [
                        (0x20,[0u16,110,120,250,511,65535][case%6]),
                        (0x22,[65535u16,511,250,120,110,0][case%6]),
                        (0x24,[0u16,3,65535][case%3]),(0xa0,if case&8==0 {0x20} else {65535}),
                        (0xe0,65535),(0xe2,100),(0xe6,0),(0xe8,100),
                        (0x618,120),(0x61a,122),(0x61c,120),(0x61e,122),
                        (0x620,[0u16,0x7fff,0x8000,0xffff][case%4]),
                        (0x622,[0u16,0x7fff,0x8000,0xffff][case%4]),
                    ] { ram[address..address+2].copy_from_slice(&value.to_le_bytes()); }
                    for a in (0x600..0x610).step_by(2) {
                        ram[a..a+2].copy_from_slice(&[0u16,100,103,65535][case%4].to_le_bytes());
                    }
                    ram[0x30]=vy; ram[0x31]=vx; ram[0x78]=(case&1)as u8;
                    ram[0xa6]=((case/4)%2*2)as u8; ram[0xa7]=((case/4)%2*2)as u8;
                    ram[0x414]=(case%8)as u8;
}
