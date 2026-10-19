//! Module08 full screen caller, real map32/map8 children and live returns.
use super::{overworld_quadrants::OverworldQuadrants,overlay_map8::OverlayMap8,
    native_work::{Work,WorkBus},nmi_stack::{NativeNmiContext,CARRY,ZERO,NEGATIVE}};
use snes::CpuMasterTimeline;
#[derive(Clone,Debug,PartialEq,Eq)]
enum Child {Quadrants(OverworldQuadrants),Map8(OverlayMap8)}
impl Child {
    fn complete(&self)->bool {match self {Self::Quadrants(w)=>w.complete(),Self::Map8(w)=>w.complete()}}
    fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        match self {Self::Quadrants(w)=>w.step(cpu,clock,bus),Self::Map8(w)=>w.step(cpu,clock,bus)}
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct OverworldScreenBuild {pc:u16,child:Option<Child>,returned_sp:u16,calls:Vec<(u8,u16,u16)>,event_overlay:bool,complete:bool}
impl Default for OverworldScreenBuild {fn default()->Self {Self {pc:0x83bf,child:None,returned_sp:0,calls:Vec::new(),event_overlay:false,complete:false}}}
impl OverworldScreenBuild {
    pub(crate) fn complete(&self)->bool {self.complete}
    pub(crate) fn event_overlay_pending(&self)->bool {self.event_overlay}
    pub(crate) fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        if self.complete || self.event_overlay {return None;}
        if let Some(child)=&mut self.child {
            let cycles=child.step(cpu,clock,bus);
            if child.complete() {assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,self.pc,self.returned_sp));self.child=None;}
            return cycles;
        }
        let pc=self.pc;
        assert_eq!((cpu.bank,cpu.pc,cpu.d,cpu.p&8),(if (0xdd40..=0xdd60).contains(&pc) {14} else {2},pc,0,0));
        let mut w=Work {cpu,clock,bus,cycles:0};
        let m=if w.cpu.p&0x20!=0 {1} else {2};let x=if w.cpu.p&0x10!=0 {1} else {2};let a=w.cpu.a&if m==1 {255} else {65535};
        w.fetch();let mut child=None;
        self.pc=match pc {
            0x83bf=>{let v=w.dp_read(0x11,m);w.load(v,m);pc+2}
            0x83c1=>{w.shift_left(m);pc+1}
            0x83c2=>{w.drain(6);w.cpu.x=a;w.flags(a,x);pc+1}
            0x83c3=>{
                w.drain(8);w.drain(6);w.drain(8);let target=w.read(0x0283b9+u32::from(w.cpu.x),2);
                assert_eq!(target,0xedb9);let sp=w.cpu.sp;w.push(0x83c5,2);self.calls.push((2,0x83c6,sp));target
            }
            0x83c6=>{w.rtl();assert!(self.calls.is_empty());self.complete=true;w.cpu.pc}
            0xedb9 | 0xed8d=>{
                let sp=w.cpu.sp;w.operand_then_internal(2);w.push(pc+2,2);self.calls.push((2,pc+3,sp));
                if pc==0xedb9 {0xed59} else {0xeec5}
            }
            0xeec7 | 0xed9d=>{
                self.returned_sp=w.cpu.sp;w.operand_then_internal(2);w.push(pc+2,2);
                child=Some(if pc==0xeec7 {Child::Quadrants(Default::default())} else {Child::Map8(OverlayMap8::from_entry(0xfd37))});pc+3
            }
            0xed59 | 0xed90 | 0xeda0=>{w.width(0x20,true);pc+2}
            0xeec5 | 0xef5e=>{w.width(0x30,true);pc+2}
            0xed8b | 0xed9b | 0xedab=>{w.width(0x20,false);pc+2}
            0xef40 | 0xef77=>{w.width(0x30,false);pc+2}
            0xed5b | 0xed5e | 0xed61 | 0xed7f | 0xef60=>{let v=w.dp_read(match pc {0xed5b|0xed7f=>0x84,0xed5e=>0x86,0xed61=>0x88,_=>0x8a},m);w.load(v,m);pc+2}
            0xed5d | 0xed60 | 0xed63 | 0xdd4a=>{w.drain(6);w.push(a,m);pc+1}
            0xed64 | 0xef2c | 0xef42 | 0xef54=>{w.cpu.x=w.dp_read(0x8a,x);w.flags(w.cpu.x,x);pc+2}
            0xed66 | 0xef64=>{w.drain(24);let v=w.read(if pc==0xed66 {0x02f88d} else {0x02edc5}+u32::from(w.cpu.x),m);w.load(v,m);pc+4}
            0xed6a | 0xed78 | 0xed85 | 0xef08 | 0xef4c | 0xef5a=>{w.mask(match pc {0xed6a=>255,0xed78=>0xf80,0xed85=>0x3e,0xef08=>0x1fff,0xef4c=>0x20,_=>2},m);pc+1+u16::from(m)}
            0xed6d | 0xeeeb | 0xeef0 | 0xef4e | 0xef5c=>{if w.branch(w.cpu.p&ZERO!=0) {match pc {0xed6d=>0xed8b,0xef4e=>0xef54,0xef5c=>0xef79,_=>0xef29}} else {pc+2}}
            0xed6f | 0xed92 | 0xedad | 0xedbc | 0xeecd | 0xeef7 | 0xef02 | 0xef0c | 0xef17 | 0xef29 | 0xef69 | 0xef70=>{
                w.immediate(match pc {0xed6f=>0x390,0xed92=>0xffff,0xedad=>4,0xedbc=>0x10,0xeecd=>0xdc4,
                    0xeef7=>0xda4,0xef02=>0xda6,0xef0c|0xef69=>0xdb4,0xef17|0xef70=>0xdb5,_=>0x20f},m);pc+1+u16::from(m)
            }
            0xed72 | 0xed7d | 0xed89 | 0xed95 | 0xeda3 | 0xeda6 | 0xeda9 | 0xedaf | 0xedbe=>{w.dp_store(match pc {0xed72|0xeda9=>0x84,0xed7d|0xeda3=>0x88,0xed89|0xeda6=>0x86,0xed95=>0xc8,0xedaf=>0x17,_=>0x10},a,m);pc+2}
            0xed74 | 0xed81=>{w.drain(6);w.cpu.p|=CARRY;pc+1}
            0xed75 | 0xed82=>{w.drain(16);w.sbc(if pc==0xed75 {0x400} else {0x10},m);pc+3}
            0xed7b | 0xef62=>{w.shift_left(m);pc+1}
            0xed7c=>{w.swap_bytes();pc+1}
            0xed88=>{w.drain(6);w.cpu.p=(w.cpu.p&!CARRY)|((a&1) as u8);w.load(a>>1,m);pc+1}
            0xed97 | 0xed99 | 0xedb6 | 0xedc0 | 0xedc2=>{w.dp_store(match pc {0xed97=>0xca,0xed99=>0xcc,0xedb6=>0x13,0xedc0=>0xb0,_=>0x11},0,m);pc+2}
            0xeda2 | 0xeda5 | 0xeda8 | 0xdd5f=>{w.drain(12);let v=w.pull(m);w.load(v,m);pc+1}
            0xedb1=>{w.drain(16);w.write(w.absolute(0x710),a,m);pc+3}
            0xedb4=>{w.dp_increment(0x11,m,true);pc+2}
            0xedb8 | 0xedc4 | 0xef79=>{w.rts();let (bank,returned,sp)=self.calls.pop().expect("screen build near return");assert_eq!((w.cpu.bank,w.cpu.pc,w.cpu.sp),(bank,returned,sp));w.cpu.pc}
            0xeeca=>{w.drain(8*u32::from(x));w.cpu.x=0x1e;w.flags(w.cpu.x,x);pc+1+u16::from(x)}
            0xeed0 | 0xeed4 | 0xeed8 | 0xeedc | 0xeee0 | 0xeefa | 0xef13 | 0xef1a | 0xef6c | 0xef73 | 0xdd50 | 0xdd55=>{
                w.drain(24);let base=match pc {0xeed0=>0x7e4000,0xeed4|0xeed8=>0x7e4020,0xeedc=>0x7e4040,0xeee0=>0x7e4060,0xef1a|0xef73=>0x7e2002,0xdd50=>0x7efa00,0xdd55=>0x7ef800,_=>0x7e2000};w.write(base+u32::from(w.cpu.x),a,m);pc+4
            }
            0xeee4 | 0xeee5 | 0xef24 | 0xef25=>{w.decrement_x();pc+1}
            0xeee6=>{if w.branch(w.cpu.p&NEGATIVE==0) {0xeed0} else {pc+2}}
            0xeee8 | 0xdd4d=>{w.drain(16);w.cpu.x=w.read(w.absolute(if pc==0xeee8 {0x696} else {0x4ac}),x);w.flags(w.cpu.x,x);pc+3}
            0xeeed | 0xeef2 | 0xef2e | 0xef37 | 0xef44=>{w.compare_x_immediate(match pc {0xeeed=>0xffff,0xeef2=>0x8000,0xef2e=>0x33,0xef37=>0x2f,_=>0x80},x);pc+1+u16::from(x)}
            0xeef5 | 0xef46=>{if w.branch(w.cpu.p&CARRY!=0) {if pc==0xeef5 {0xef07} else {0xef54}} else {pc+2}}
            0xeefe | 0xef0f | 0xef20 | 0xef50=>{
                let sp=w.cpu.sp;w.drain(24);w.drain(6);w.push(u16::from(w.cpu.bank),1);w.push(pc+3,2);
                self.calls.push((2,pc+4,sp));w.cpu.bank=14;
                if pc==0xef50 {self.event_overlay=true;0xf652} else {0xdd40}
            }
            0xef05=>{w.branch(true);0xef1a}
            0xef07 | 0xdd54=>{w.drain(6);w.load(if pc==0xef07 {w.cpu.x} else {w.cpu.y},m);pc+1}
            0xef0b | 0xef63 | 0xef68=>{w.drain(6);w.cpu.x=a;w.flags(a,x);pc+1}
            0xef1e | 0xef1f | 0xdd59 | 0xdd5a=>{w.increment_x();pc+1}
            0xef26=>{w.drain(16);w.write(w.absolute(0x696),0,m);pc+3}
            0xef31 | 0xef3a=>{if w.branch(w.cpu.p&ZERO==0) {if pc==0xef31 {0xef37} else {0xef40}} else {pc+2}}
            0xef33 | 0xef3c=>{w.drain(24);w.write(if pc==0xef33 {0x7e22a8} else {0x7e2bb2},a,m);pc+4}
            0xef48 | 0xef56=>{w.drain(24);let v=w.read(0x7ef280+u32::from(w.cpu.x),m);w.load(v,m);pc+4}
            0xdd40 | 0xdd45=>{w.compare_immediate(if pc==0xdd40 {0xdc5} else {0xdc9},m);pc+3}
            0xdd43 | 0xdd48=>{if w.branch(w.cpu.p&ZERO!=0) {0xdd60} else {pc+2}}
            0xdd4b=>{w.drain(6);w.push(w.cpu.x,x);pc+1}
            0xdd4c=>{w.drain(6);w.cpu.y=w.cpu.x;w.flags(w.cpu.y,x);pc+1}
            0xdd5b=>{w.drain(16);w.write(w.absolute(0x4ac),w.cpu.x,x);pc+3}
            0xdd5e=>{w.drain(12);w.cpu.x=w.pull(x);w.flags(w.cpu.x,x);pc+1}
            0xdd60=>{w.rtl();let (bank,returned,sp)=self.calls.pop().expect("memorized tile far return");assert_eq!((w.cpu.bank,w.cpu.pc,w.cpu.sp),(bank,returned,sp));w.cpu.pc}
            _=>unreachable!("screen build PC {pc:04x}"),
        };
        if let Some(child)=child {w.cpu.pc=match &child {Child::Quadrants(_)=>0xf54a,Child::Map8(_)=>0xfd37};self.child=Some(child);}
        else {w.cpu.pc=self.pc;}
        Some(w.cycles)
    }
}
