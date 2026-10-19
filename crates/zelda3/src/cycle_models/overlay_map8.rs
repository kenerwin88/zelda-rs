//! Overlay map16-to-map8 conversion, including its nested row/chunk returns.
use super::{native_work::{Work,WorkBus},nmi_stack::{NativeNmiContext,CARRY,ZERO}};
use snes::CpuMasterTimeline;
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct OverlayMap8 {pc:u16,calls:Vec<(u16,u16)>,complete:bool}
impl Default for OverlayMap8 {fn default()->Self {Self {pc:0xfd26,calls:Vec::new(),complete:false}}}
impl OverlayMap8 {
    pub(crate) fn from_entry(pc:u16)->Self {
        assert!(matches!(pc,0xfd26|0xfd37));
        Self {pc,calls:Vec::new(),complete:false}
    }
    pub(crate) fn complete(&self)->bool {self.complete}
    pub(crate) fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        if self.complete {return None;}
        assert_eq!((cpu.bank,cpu.pc,cpu.d,cpu.p&8),(2,self.pc,0,0));
        let mut w=Work {cpu,clock,bus,cycles:0};let pc=self.pc;
        let m=if w.cpu.p&0x20!=0 {1} else {2};let a=w.cpu.a&if m==1 {255} else {65535};w.fetch();
        self.pc=match pc {
            0xfd26 | 0xfd37 => {w.drain(6);w.push(u16::from(w.cpu.db),1);pc+1}
            0xfd27 | 0xfd38 => {w.immediate(0xf,m);pc+2}
            0xfd29 | 0xfd3a => {w.drain(6);w.push(a,m);pc+1}
            0xfd2a | 0xfd3b | 0xfd85 => {w.drain(12);let v=w.pull(1);w.cpu.db=v as u8;w.flags(v,1);pc+1}
            0xfd2b | 0xfd3c => {w.width(0x30,true);pc+2}
            0xfd83 => {w.width(0x30,false);pc+2}
            0xfd2d | 0xfd32 | 0xfd3e | 0xfd43 | 0xfd54 | 0xfd95 | 0xfdd2 | 0xfe09 => {
                w.immediate(match pc {0xfd2d=>0x4000,0xfd3e=>0x2000,0xfd32|0xfd43=>0x7e,0xfdd2=>0x800,_=>0x10},m);pc+1+u16::from(m)
            }
            0xfd30 | 0xfd41 | 0xfd46 | 0xfd4e | 0xfd57 | 0xfd62 | 0xfd6a | 0xfd75 | 0xfd7d | 0xfd98 | 0xfdc6 | 0xfdd0 | 0xfdd5 | 0xfde2 | 0xfdf1 | 0xfdf7 | 0xfe0c | 0xfe44 => {
                let at=match pc {0xfd30|0xfd41=>4,0xfd46=>6,0xfd4e|0xfd62|0xfd75=>0x84,0xfd57=>8,
                    0xfd6a|0xfd7d=>0x88,0xfdc6|0xfdd0|0xfdf7=>2,0xfe0c=>0xc,0xfe44=>0xe,_=>0};
                w.dp_store(at,a,m);pc+2
            }
            0xfd35 => {w.branch(true);0xfd46}
            0xfd48 | 0xfd5c | 0xfd64 | 0xfd6f | 0xfd77 | 0xfd87 | 0xfd91 | 0xfdc4 | 0xfdd7 | 0xfdeb | 0xfdfb => {
                let at=match pc {0xfd64|0xfd77|0xfdc4=>0x88,0xfd91=>0x86,0xfdd7=>2,0xfdeb|0xfdfb=>0,_=>0x84};
                let v=w.dp_read(at,m);w.load(v,m);pc+2
            }
            0xfd4a | 0xfddf | 0xfded | 0xfe40 => {w.clear_carry();pc+1}
            0xfd4b | 0xfdee | 0xfe41 => {w.drain(8*u32::from(m));w.adc(match pc {0xfd4b=>0x1000,0xfdee=>0x400,_=>0x40});pc+3}
            0xfd50 | 0xfd52 => {w.dp_store(if pc==0xfd50 {0xa} else {0xe},0,m);pc+2}
            0xfd59 | 0xfd6c | 0xfde8 => {
                let sp=w.cpu.sp;w.operand_then_internal(2);w.push(pc+2,2);self.calls.push((pc+3,sp));
                if pc==0xfde8 {0xfdf7} else {0xfd87}
            }
            0xfd5e | 0xfd71 | 0xfd89 => {w.drain(6);w.cpu.p|=CARRY;pc+1}
            0xfd5f | 0xfd72 | 0xfd8a => {w.drain(8*u32::from(m));w.sbc(if pc==0xfd8a {0x410} else {0x80},m);pc+3}
            0xfd66 | 0xfd79 => {w.drain(6);w.load(a.wrapping_sub(1),m);pc+1}
            0xfd67 | 0xfd7a | 0xfd8d | 0xfda2 | 0xfda9 | 0xfdb5 | 0xfdbc | 0xfdcd => {
                w.mask(match pc {0xfd67|0xfd7a=>0x1f,0xfda2|0xfdb5=>0x3f,0xfdcd=>0xf,_=>0x1fff},m);pc+3
            }
            0xfd7f | 0xfdc0 | 0xfe3b => {w.dp_increment(match pc {0xfd7f=>8,0xfdc0=>0,_=>0xc},m,false);pc+2}
            0xfd81 | 0xfdc2 | 0xfe3d => {if w.branch(w.cpu.p&ZERO==0) {match pc {0xfd81=>0xfd59,0xfdc2=>0xfd9a,_=>0xfe0e}} else {pc+2}}
            0xfd90 | 0xfdac | 0xfdbf | 0xfe1a => {w.drain(6);w.cpu.y=a;w.flags(w.cpu.y,2);pc+1}
            0xfd93 | 0xfdd9..=0xfdde | 0xfe17..=0xfe19 => {w.shift_left(m);pc+1}
            0xfd94 | 0xfda5 | 0xfdb8 => {w.tax_word();pc+1}
            0xfd9a | 0xfdad => {let at=w.indirect_y_address(4);let v=w.read(at,m);w.load(v,m);pc+2}
            0xfd9c | 0xfdaf => {w.drain(16);w.drain(6);w.write(w.absolute(0x500).wrapping_add(u32::from(w.cpu.x)),a,m);pc+3}
            0xfd9f | 0xfda0 | 0xfdb2 | 0xfdb3 | 0xfe03 | 0xfe04 | 0xfe29 | 0xfe2a | 0xfe39 | 0xfe3a => {w.increment_x();pc+1}
            0xfda1 | 0xfdb4 | 0xfe3f => {w.drain(6);w.load(w.cpu.x,m);pc+1}
            0xfda6 | 0xfda7 | 0xfdb9 | 0xfdba | 0xfe13 | 0xfe14 => {w.increment_y();pc+1}
            0xfda8 | 0xfdbb => {w.drain(6);w.load(w.cpu.y,m);pc+1}
            0xfdc8 => {w.compare_immediate(0x10,m);pc+3}
            0xfdcb => {if w.branch(w.cpu.p&CARRY==0) {0xfdd7} else {pc+2}}
            0xfde0 => {let v=w.dp_read(0,m);w.adc(v);pc+2}
            0xfde4 | 0xfdf3 => {w.drain(24);let v=w.read(if pc==0xfde4 {0x02f889} else {0x02f88b},m);w.load(v,m);pc+4}
            0xfdf9 | 0xfe07 => {w.cpu.x=w.dp_read(if pc==0xfdf9 {0xa} else {0xe},2);w.flags(w.cpu.x,2);pc+2}
            0xfdfd => {let v=w.dp_read(0xcc,m);w.load(a|v,m);pc+2}
            0xfdff | 0xfe1e | 0xfe25 | 0xfe2e | 0xfe35 => {
                w.drain(24);let base=match pc {0xfdff=>0x7f4000,0xfe25|0xfe35=>0x7f2040,_=>0x7f2000};
                w.write(base+u32::from(w.cpu.x),a,m);pc+4
            }
            0xfe05 => {w.dp_store(0xa,w.cpu.x,2);pc+2}
            0xfe0e => {w.cpu.y=w.dp_read(2,2);w.flags(w.cpu.y,2);pc+2}
            0xfe10 | 0xfe1b | 0xfe22 | 0xfe2b | 0xfe32 => {
                w.drain(16);w.drain(6);let base=match pc {0xfe10=>0x500,0xfe1b=>0x8000,0xfe22=>0x8004,0xfe2b=>0x8002,_=>0x8006};
                let v=w.read(w.absolute(base).wrapping_add(u32::from(w.cpu.y)),m);w.load(v,m);pc+3
            }
            0xfe15 => {w.dp_store(2,w.cpu.y,2);pc+2}
            0xfe46 => {w.rts();let (returned,sp)=self.calls.pop().expect("map8 row owns its near frame");
                assert_eq!((w.cpu.bank,w.cpu.pc,w.cpu.sp),(2,returned,sp));w.cpu.pc}
            0xfd86 => {w.rts();assert!(self.calls.is_empty());self.complete=true;w.cpu.pc}
            _=>unreachable!("overlay map8 PC {pc:04x}"),
        };w.cpu.pc=self.pc;Some(w.cycles)
    }
}
