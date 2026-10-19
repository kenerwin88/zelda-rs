//! Full overworld map32 build on the caller's live CPU and nested near frames.
use super::{native_decompress::NativeDecompression, map32_interlace::Map32Interlace,
    map32_definition::Map32Definition, native_work::{Work,WorkBus},
    nmi_stack::{NativeNmiContext,CARRY,ZERO}};
use snes::CpuMasterTimeline;
#[derive(Clone,Debug,PartialEq,Eq)]
enum Child { Decompress(NativeDecompression), Interlace(Map32Interlace), Definition(Map32Definition) }
impl Child {
    fn complete(&self)->bool {match self {Self::Decompress(w)=>w.complete(),Self::Interlace(w)=>w.complete(),Self::Definition(w)=>w.complete()}}
    fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        match self {Self::Decompress(w)=>w.step(cpu,clock,bus),Self::Interlace(w)=>w.step(cpu,clock,bus),Self::Definition(w)=>w.step(cpu,clock,bus)}
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct OverworldQuadrants {pc:u16,child:Option<Child>,returned_sp:u16,calls:Vec<(u16,u16)>,complete:bool}
impl Default for OverworldQuadrants {fn default()->Self {Self {pc:0xf54a,child:None,returned_sp:0,calls:Vec::new(),complete:false}}}
impl OverworldQuadrants {
    pub(crate) fn complete(&self)->bool {self.complete}
    pub(crate) fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        if self.complete {return None;}
        if let Some(child)=&mut self.child {
            let cycles=child.step(cpu,clock,bus);
            if child.complete() {assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,self.pc,self.returned_sp));self.child=None;}
            return cycles;
        }
        assert_eq!((cpu.bank,cpu.pc,cpu.d,cpu.p&8),(2,self.pc,0,0));
        let mut w=Work {cpu,clock,bus,cycles:0};let pc=self.pc;
        let m=if w.cpu.p&0x20!=0 {1} else {2};let a=w.cpu.a&if m==1 {255} else {65535};
        w.fetch();let mut call=None;
        self.pc=match pc {
            0xf54a | 0xf55d | 0xf570 | 0xf582 | 0xf5a9 | 0xf5ac | 0xf5af | 0xf629 => {
                let at=match pc {0xf5a9=>0,0xf5ac=>2,0xf5af=>4,0xf629=>6,_=>0x8a};
                let v=w.dp_read(at,m);w.load(v,m);pc+2
            }
            0xf54c | 0xf562 | 0xf578 | 0xf58a | 0xf619 => {w.shift_left(m);pc+1}
            0xf54d | 0xf563 | 0xf579 | 0xf58b => {let v=w.dp_read(if pc==0xf54d {0x8a} else {0},m);w.adc(v);pc+2}
            0xf54f | 0xf565 | 0xf57b | 0xf58d => {w.tax_word();pc+1}
            0xf550 | 0xf557 | 0xf566 | 0xf57c | 0xf58e | 0xf5b2 | 0xf5b7 | 0xf5d4 | 0xf5d9 | 0xf5f3 | 0xf5f8 | 0xf600 | 0xf606 | 0xf610 => {
                let v=match pc {0xf550=>0x7e,0xf557|0xf566=>0x2000,0xf57c|0xf58e=>0x3000,
                    0xf5b2|0xf5d4=>0x4400,0xf5f3=>0x4000,0xf606=>0xffff,0xf610=>0x10,_=>0x7f};
                w.immediate(v,m);pc+1+u16::from(m)
            }
            0xf553 | 0xf555 | 0xf560 | 0xf576 | 0xf588 | 0xf595 | 0xf59b | 0xf5a1 | 0xf5a7 | 0xf5b5 | 0xf5ba | 0xf5cc | 0xf5d2 | 0xf5d7 | 0xf5dc | 0xf5eb | 0xf5ee | 0xf5f1 | 0xf5f6 | 0xf5fb | 0xf613 | 0xf61f | 0xf62f => {
                let at=match pc {0xf553|0xf5ba|0xf5dc|0xf5ee=>2,0xf555=>5,0xf59b=>3,
                    0xf5a1|0xf5cc=>0xc8,0xf5a7|0xf5d2=>0xc9,0xf5eb=>4,0xf5f6=>8,0xf5fb=>0xa,
                    0xf613=>0xd,0xf61f|0xf62f=>6,_=>0};
                w.dp_store(at,if pc==0xf61f {w.cpu.y} else {a},m);pc+2
            }
            0xf55a | 0xf56d | 0xf57f => {
                let sp=w.cpu.sp;w.operand_then_internal(2);w.push(pc+2,2);
                self.calls.push((pc+3,sp));0xf595
            }
            0xf55f => {w.drain(6);w.load(a.wrapping_add(1),m);pc+1}
            0xf569 | 0xf572 | 0xf584 | 0xf591 | 0xf597 | 0xf62b => {w.clear_carry();pc+1}
            0xf56a | 0xf573 | 0xf585 | 0xf592 | 0xf598 | 0xf62c => {
                w.drain(16);w.adc(match pc {0xf56a|0xf592=>0x40,0xf573=>8,0xf585=>9,0xf598=>0x80,_=>0xc0});pc+3
            }
            0xf59d | 0xf5a3 | 0xf5c8 | 0xf5ce => {
                w.drain(24);let base=match pc {0xf59d=>0x02f94d,0xf5a3=>0x02f94e,0xf5c8=>0x02fb2d,_=>0x02fb2e};
                let v=w.read(base+u32::from(w.cpu.x),m);w.load(v,m);pc+4
            }
            0xf5ab | 0xf5ae | 0xf5b1 | 0xf602 => {w.drain(6);w.push(a,m);pc+1}
            0xf5bc | 0xf5de => {w.drain(6);w.push(w.cpu.x,2);pc+1}
            0xf5bd | 0xf5df | 0xf5fd => {w.width(if pc==0xf5fd {0x20} else {0x30},false);pc+2}
            0xf5c2 | 0xf5e4 | 0xf604 => {w.width(0x30,true);pc+2}
            0xf5bf | 0xf5c4 | 0xf5e1 | 0xf5e6 | 0xf61c => {
                self.returned_sp=w.cpu.sp;w.operand_then_internal(2);w.push(pc+2,2);
                call=Some(match pc {0xf5bf|0xf5e1=>Child::Decompress(NativeDecompression::bank02()),
                    0xf5c4=>Child::Interlace(Map32Interlace::new(true)),0xf5e6=>Child::Interlace(Map32Interlace::new(false)),
                    _=>Child::Definition(Map32Definition::default())});pc+3
            }
            0xf5c7 | 0xf5e9 => {w.drain(12);w.cpu.x=w.pull(2);w.flags(w.cpu.x,2);pc+1}
            0xf5ea | 0xf5ed | 0xf5f0 => {w.drain(12);let v=w.pull(m);w.load(v,m);pc+1}
            0xf5ff => {w.drain(6);w.push(u16::from(w.cpu.db),1);pc+1}
            0xf603 | 0xf636 => {w.drain(12);let v=w.pull(1);w.cpu.db=v as u8;w.flags(v,1);pc+1}
            0xf609 => {w.drain(16);w.write(w.absolute(0x4440),a,m);pc+3}
            0xf60c | 0xf60e => {w.dp_store(if pc==0xf60c {6} else {0xb},0,m);pc+2}
            0xf615 | 0xf61a => {w.cpu.y=w.dp_read(if pc==0xf615 {0xb} else {6},2);w.flags(w.cpu.y,2);pc+2}
            0xf617 => {let at=w.indirect_y_address(8);let v=w.read(at,m);w.load(v,m);pc+2}
            0xf621 | 0xf623 => {w.dp_increment(0xb,m,true);pc+2}
            0xf625 => {w.dp_increment(0xd,m,false);pc+2}
            0xf627 => {if w.branch(w.cpu.p&ZERO==0) {0xf615} else {pc+2}}
            0xf631 => {w.compare_immediate(0x1000,m);pc+3}
            0xf634 => {if w.branch(w.cpu.p&CARRY==0) {0xf610} else {pc+2}}
            0xf637 => {
                w.rts();
                if let Some((returned,sp))=self.calls.pop() {assert_eq!((w.cpu.bank,w.cpu.pc,w.cpu.sp),(2,returned,sp));}
                else {self.complete=true;}
                w.cpu.pc
            }
            _=>unreachable!("overworld quadrants PC {pc:04x}"),
        };
        if let Some(child)=call {
            w.cpu.pc=match &child {Child::Decompress(_)=>0xfebb,Child::Interlace(_)=>if pc==0xf5c4 {0xf638} else {0xf679},Child::Definition(_)=>0xf691};self.child=Some(child);
        } else {w.cpu.pc=self.pc;}
        Some(w.cycles)
    }
}
