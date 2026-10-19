//! Overlay map32 quadrant composition. Decompression, byte interlacing and
//! packed definitions consume their actual nested near frames on one clock.
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
pub(crate) struct OverlayQuadrant {pc:u16,child:Option<Child>,returned_sp:u16,complete:bool}
impl Default for OverlayQuadrant {fn default()->Self {Self {pc:0xf7cb,child:None,returned_sp:0,complete:false}}}
impl OverlayQuadrant {
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
            0xf7cb | 0xf7ef | 0xf7f2 | 0xf7f5 | 0xf870 => {
                let v=w.dp_read(match pc {0xf7cb=>0x8a,0xf7ef=>0,0xf7f2=>2,0xf7f5=>4,_=>6},m);w.load(v,m);pc+2
            }
            0xf7cd | 0xf85f => {w.shift_left(m);pc+1}
            0xf7ce => {let v=w.dp_read(0x8a,m);w.adc(v);pc+2}
            0xf7d0 | 0xf860 => {w.tax_word();pc+1}
            0xf7d1 | 0xf7d8 | 0xf7f8 | 0xf7fd | 0xf81a | 0xf81f | 0xf839 | 0xf83e | 0xf846 | 0xf84c | 0xf856 => {
                w.immediate(match pc {0xf7d1=>0x7e,0xf7d8|0xf839=>0x4000,
                    0xf7f8|0xf81a=>0x4400,0xf84c=>0xffff,0xf856=>0x10,_=>0x7f},m);pc+1+u16::from(m)
            }
            0xf7d4 | 0xf7d6 | 0xf7db | 0xf7e1 | 0xf7e7 | 0xf7ed | 0xf7fb | 0xf800 | 0xf812 | 0xf818 | 0xf81d | 0xf822 | 0xf831 | 0xf834 | 0xf837 | 0xf83c | 0xf841 | 0xf859 | 0xf85b | 0xf861 | 0xf866 | 0xf876 => {
                let at=match pc {0xf7d4|0xf800|0xf822|0xf834=>2,0xf7d6=>5,
                    0xf7e1=>3,0xf7e7|0xf812=>0xc8,0xf7ed|0xf818=>0xc9,
                    0xf831=>4,0xf83c=>8,0xf841=>0xa,0xf859=>0xd,0xf866|0xf876=>6,_=>0};
                if matches!(pc,0xf859) {w.dp_store(at,a,m);}
                else if matches!(pc,0xf85b|0xf861) {w.cpu.y=w.dp_read(if pc==0xf85b {0xb} else {6},2);w.flags(w.cpu.y,2);}
                else if pc==0xf866 {w.dp_store(at,w.cpu.y,2);}
                else {w.dp_store(at,a,m);}
                pc+2
            }
            0xf7dd | 0xf872 => {w.clear_carry();pc+1}
            0xf7de | 0xf873 => {w.drain(16);w.adc(if pc==0xf7de {0x80} else {0xc0});pc+3}
            0xf7e3 | 0xf7e9 | 0xf80e | 0xf814 => {
                w.drain(24);let base=match pc {0xf7e3=>0x02f94d,0xf7e9=>0x02f94e,0xf80e=>0x02fb2d,_=>0x02fb2e};
                let v=w.read(base+u32::from(w.cpu.x),m);w.load(v,m);pc+4
            }
            0xf7f1 | 0xf7f4 | 0xf7f7 | 0xf848 => {w.drain(6);w.push(a,m);pc+1}
            0xf802 | 0xf824 => {w.drain(6);w.push(w.cpu.x,2);pc+1}
            0xf803 | 0xf825 | 0xf843 => {w.width(if pc==0xf843 {0x20} else {0x30},false);pc+2}
            0xf808 | 0xf82a | 0xf84a => {w.width(0x30,true);pc+2}
            0xf805 | 0xf80a | 0xf827 | 0xf82c | 0xf863 => {
                self.returned_sp=w.cpu.sp;w.operand_then_internal(2);w.push(pc+2,2);
                call=Some(match pc {0xf805|0xf827=>Child::Decompress(NativeDecompression::bank02()),
                    0xf80a=>Child::Interlace(Map32Interlace::new(true)),0xf82c=>Child::Interlace(Map32Interlace::new(false)),
                    _=>Child::Definition(Map32Definition::default())});
                pc+3
            }
            0xf80d | 0xf82f => {w.drain(12);w.cpu.x=w.pull(2);w.flags(w.cpu.x,2);pc+1}
            0xf830 | 0xf833 | 0xf836 => {w.drain(12);let v=w.pull(m);w.load(v,m);pc+1}
            0xf845 => {w.drain(6);w.push(u16::from(w.cpu.db),1);pc+1}
            0xf849 | 0xf87d => {w.drain(12);let v=w.pull(1);w.cpu.db=v as u8;w.flags(v,1);pc+1}
            0xf84f => {w.drain(16);w.write(w.absolute(0x4440),a,m);pc+3}
            0xf852 | 0xf854 => {w.dp_store(if pc==0xf852 {6} else {0xb},0,m);pc+2}
            0xf85d => {let at=w.indirect_y_address(8);let v=w.read(at,m);w.load(v,m);pc+2}
            0xf868 | 0xf86a => {w.dp_increment(0xb,m,true);pc+2}
            0xf86c => {w.dp_increment(0xd,m,false);pc+2}
            0xf86e => {if w.branch(w.cpu.p&ZERO==0) {0xf85b} else {pc+2}}
            0xf878 => {w.compare_immediate(0x1000,m);pc+3}
            0xf87b => {if w.branch(w.cpu.p&CARRY==0) {0xf856} else {pc+2}}
            0xf87e => {w.rts();self.complete=true;w.cpu.pc}
            _=>unreachable!("overlay quadrant PC {pc:04x}"),
        };
        if let Some(child)=call {
            w.cpu.pc=match &child {Child::Decompress(_)=>0xfebb,Child::Interlace(_)=>if pc==0xf80a {0xf638} else {0xf679},Child::Definition(_)=>0xf691};
            self.child=Some(child);
        } else {w.cpu.pc=self.pc;}
        Some(w.cycles)
    }
}
