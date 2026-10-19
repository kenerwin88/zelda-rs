//! Selected garnish countdown, allocation and real RTS dispatch to its body.
use super::{native_work::{Work,WorkBus},nmi_stack::{NativeNmiContext,ZERO},oam_allocate::OamAllocate};
use snes::CpuMasterTimeline;
const SIZES:[u8;22]=[4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,8,4,4,4,8,16];
const BODIES:[u16;22]=[0xb6c0,0xb6e1,0xb627,0xb5bb,0xb526,0xb4fb,0xb49e,0xb4c6,0xb429,0xb3ee,0xb3c2,0xb34f,0,0xb55d,0xb591,0xb306,0xb2b2,0xb520,0xb216,0xb3bc,0xb178,0xf0cb];
pub(crate) fn table_byte(a:u32)->Option<u8> {
    match a {
        0x09b0a0..=0x09b0b5=>Some(SIZES[(a-0x09b0a0) as usize]),
        0x09b124..=0x09b14f=>{let i=(a-0x09b124) as usize;Some(BODIES[i/2].to_le_bytes()[i&1])},
        _=>None,
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum GarnishSingleTail { Returned, Body {kind:u8,pc:u16} }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct GarnishSingle {kind:u8,allocator:Option<OamAllocate>,allocator_return:u16,tail:Option<GarnishSingleTail>}
impl GarnishSingle {
    pub(crate) fn new(kind:u8)->Self {assert!((1..=22).contains(&kind));Self {kind,allocator:None,allocator_return:0,tail:None}}
    pub(crate) fn tail(&self)->Option<GarnishSingleTail> {self.tail}
    pub(crate) fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        if self.tail.is_some() {return None;}
        if let Some(work)=&mut self.allocator {
            let cycles=work.step(cpu,clock,bus);
            if work.complete() {assert_eq!((cpu.bank,cpu.pc,cpu.db),(9,self.allocator_return,9));self.allocator=None;}
            return cycles;
        }
        assert_eq!((cpu.bank,cpu.db,cpu.d,cpu.p&8),(9,9,0,0));
        let mut w=Work {cpu,clock,bus,cycles:0};
        let pc=w.cpu.pc;let m=if w.cpu.p&0x20!=0 {1}else{2};let xw=if w.cpu.p&0x10!=0 {1}else{2};
        w.fetch();
        let next=match pc {
            0xb0bf=>{w.compare_immediate(5,1);pc+2},
            0xb0c1|0xb0c8|0xb0ce|0xb0d5|0xb0e0|0xb0e6|0xb0db|0xb0f4|0xb102=>{
                let z=w.cpu.p&ZERO!=0;
                let (take,target)=match pc {0xb0c1=>(z,0xb0ca),0xb0c8|0xb0ce|0xb0d5=>(if pc==0xb0c8||pc==0xb0d5 {!z}else{z},0xb0dd),0xb0e0=>(z,0xb104),0xb0e6=>(z,0xb0f6),0xb0db=>(true,0xb123),_=>(true,0xb110)};
                if w.branch(take) {target}else{pc+2}
            },
            0xb0c3=>{let v=w.dp_read(0x11,1);w.load(v,1);pc+2},
            0xb0c5=>{w.drain(16);let v=w.read(w.absolute(0xfc1),1);w.load((w.cpu.a|v)&255,1);pc+3},
            0xb0ca|0xb0e2|0xb0e8|0xb0f6|0xb104|0xb110=>{
                w.drain(24);let base=match pc {0xb0ca=>0x7ff90e,0xb0e2=>0x7ff968,_=>0x7ff800};let v=w.read(base+u32::from(w.cpu.x),1);w.load(v,1);pc+4
            },
            0xb0d0|0xb114|0xb11f=>{w.drain(6);let mask=if m==1 {255}else{65535};let v=w.cpu.a.wrapping_sub(1)&mask;w.cpu.a=(w.cpu.a&!mask)|v;w.flags(v,m);pc+1},
            0xb0d1|0xb0d7=>{w.drain(24);w.write(if pc==0xb0d1 {0x7ff90e}else{0x7ff800}+u32::from(w.cpu.x),w.cpu.a&255,1);pc+4},
            0xb0dd=>{w.drain(16);w.cpu.y=w.read(w.absolute(0xfb3),1);w.flags(w.cpu.y,1);pc+3},
            0xb0ec|0xb0fa|0xb108|0xb11b=>{w.drain(6);w.cpu.y=w.cpu.a&if xw==1 {255}else{65535};w.flags(w.cpu.y,xw);pc+1},
            0xb0ed|0xb0fb|0xb109|0xb11c=>{
                w.drain(16);if xw==2 {w.drain(6);}let v=w.read(w.absolute(if pc==0xb11c {0xb124}else{0xb09f})+u32::from(w.cpu.y),m);w.load(v,m);pc+3
            },
            0xb0f0|0xb0fe|0xb10c=>{
                let target=match pc {0xb0f0=>0xba94,0xb0fe=>0xba8c,_=>0xba80};w.operand_then_internal(3);w.push(9,1);w.push(pc+3,2);w.cpu.bank=13;
                self.allocator=Some(OamAllocate::new(target));self.allocator_return=pc+4;target
            },
            0xb115|0xb121=>{w.operand_then_internal(1);if pc==0xb115 {w.cpu.p&=!0x30;}else{w.cpu.p|=0x30;w.cpu.x&=255;w.cpu.y&=255;}pc+2},
            0xb117=>{w.mask(255,2);pc+3},
            0xb11a=>{w.shift_left(2);pc+1},
            0xb120=>{w.drain(6);w.push(w.cpu.a,2);pc+1},
            0xb123=>{
                w.rts();
                self.tail=Some(if w.cpu.pc==0xb09c {GarnishSingleTail::Returned}else {assert_eq!(w.cpu.pc,BODIES[usize::from(self.kind-1)]);GarnishSingleTail::Body {kind:self.kind,pc:w.cpu.pc}});
                w.cpu.pc
            },
            _=>unreachable!("garnish single {pc:04x}"),
        };
        w.cpu.pc=next;Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lower_garnish_tables_include_only_66_legitimate_data_bytes() {
        let rom=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
        for a in (0x09b0a0..=0x09b0b5).chain(0x09b124..=0x09b14f) {
            assert_eq!(table_byte(a),Some(rom[((a>>16)&0x7f) as usize*0x8000+(a as usize&0x7fff)]));
        }
        for a in [0x09b09f,0x09b0b6,0x09b123,0x09b150] {assert_eq!(table_byte(a),None);}
    }
}
