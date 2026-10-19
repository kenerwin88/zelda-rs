//! Module08 overlay dispatch and complete callee composition on one live clock.
use super::{overworld_overlays::OverworldOverlays,overlay_map_loader::OverlayMapLoader,
    native_work::{Work,WorkBus},nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct OverworldOverlaysCaller {pc:u16,parent:Option<OverworldOverlays>,loader:Option<OverlayMapLoader>,complete:bool}
impl Default for OverworldOverlaysCaller {
    fn default()->Self {Self {pc:0x83bf,parent:None,loader:None,complete:false}}
}
impl OverworldOverlaysCaller {
    pub(crate) fn complete(&self)->bool {self.complete}
    pub(crate) fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        if self.complete {return None;}
        if let Some(loader)=&mut self.loader {
            let cycles=loader.step(cpu,clock,bus);
            if loader.complete() {self.parent.as_mut().unwrap().return_from_map_loader(cpu);self.loader=None;}
            return cycles;
        }
        if let Some(parent)=&mut self.parent {
            if parent.complete() {self.parent=None;self.pc=0x83c6;}
            else {
                let cycles=parent.step(cpu,clock,bus);
                if parent.map_loader_pending() {self.loader=Some(OverlayMapLoader::default());}
                return cycles;
            }
        }
        assert_eq!((cpu.bank,cpu.pc,cpu.d,cpu.p&0x38),(2,self.pc,0,0x30));
        let mut w=Work {cpu,clock,bus,cycles:0};w.fetch();
        self.pc=match self.pc {
            0x83bf=>{let v=w.dp_read(0x11,1);w.load(v,1);0x83c1}
            0x83c1=>{w.shift_left(1);0x83c2}
            0x83c2=>{w.drain(6);w.cpu.x=w.cpu.a&255;w.flags(w.cpu.x,1);0x83c3}
            0x83c3=>{
                w.drain(8);w.drain(6);w.drain(8);
                let target=w.read(0x0283b9+u32::from(w.cpu.x),2);
                assert_eq!(target,0xaf19,"Module08 overlay owner reached another submodule");
                w.push(0x83c5,2);self.parent=Some(OverworldOverlays::new(target));target
            }
            0x83c6=>{w.rtl();self.complete=true;w.cpu.pc}
            _=>unreachable!("Module08 overlay caller PC {:04x}",self.pc),
        };
        w.cpu.pc=self.pc;Some(w.cycles)
    }
}
