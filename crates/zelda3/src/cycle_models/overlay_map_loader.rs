//! The overlay loader owns both children and retires their real near frames.
use super::{overlay_quadrant::OverlayQuadrant,overlay_map8::OverlayMap8,
    native_work::{Work,WorkBus},nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;
#[derive(Clone,Debug,PartialEq,Eq)]
enum Child {Quadrant(OverlayQuadrant),Map8(OverlayMap8)}
impl Child {
    fn complete(&self)->bool {match self {Self::Quadrant(w)=>w.complete(),Self::Map8(w)=>w.complete()}}
    fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        match self {Self::Quadrant(w)=>w.step(cpu,clock,bus),Self::Map8(w)=>w.step(cpu,clock,bus)}
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct OverlayMapLoader {pc:u16,child:Option<Child>,returned_sp:u16,complete:bool}
impl Default for OverlayMapLoader {fn default()->Self {Self {pc:0xfd0d,child:None,returned_sp:0,complete:false}}}
impl OverlayMapLoader {
    pub(crate) fn complete(&self)->bool {self.complete}
    pub(crate) fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        if self.complete {return None;}
        if let Some(child)=&mut self.child {
            let cycles=child.step(cpu,clock,bus);
            if child.complete() {assert_eq!((cpu.bank,cpu.pc,cpu.sp),(2,self.pc,self.returned_sp));self.child=None;}
            return cycles;
        }
        assert_eq!((cpu.bank,cpu.pc,cpu.d,cpu.p&8),(2,self.pc,0,0));
        let mut w=Work {cpu,clock,bus,cycles:0};let pc=self.pc;w.fetch();let mut call=None;
        self.pc=match pc {
            0xfd0d => {w.width(0x30,true);pc+2}
            0xfd0f | 0xfd19 => {self.returned_sp=w.cpu.sp;w.operand_then_internal(2);w.push(pc+2,2);
                call=Some(if pc==0xfd0f {Child::Quadrant(OverlayQuadrant::default())} else {Child::Map8(OverlayMap8::default())});pc+3}
            0xfd12 => {w.immediate(0x1000,2);pc+3}
            0xfd15 => {w.dp_store(0xcc,w.cpu.a,2);pc+2}
            0xfd17 => {w.width(0x30,false);pc+2}
            0xfd1c => {w.immediate(4,1);pc+2}
            0xfd1e => {w.dp_store(0x17,w.cpu.a&255,1);pc+2}
            0xfd20 => {w.drain(16);w.write(w.absolute(0x710),w.cpu.a&255,1);pc+3}
            0xfd23 => {w.dp_increment(0x11,1,true);pc+2}
            0xfd25 => {w.rts();self.complete=true;w.cpu.pc}
            _=>unreachable!("overlay loader PC {pc:04x}"),
        };
        if let Some(child)=call {w.cpu.pc=if pc==0xfd0f {0xf7cb} else {0xfd26};self.child=Some(child);}
        else {w.cpu.pc=self.pc;}
        Some(w.cycles)
    }
}
