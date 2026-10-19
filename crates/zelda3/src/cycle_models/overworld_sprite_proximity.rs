//! Outdoor sprite cell scans, allocation and real near/far returns.
use super::{native_work::{Work,WorkBus},nmi_stack::{NativeNmiContext,CARRY,ZERO,NEGATIVE}};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(address:u32)->Option<u8> {
    let a=address&0x7fffff;
    match a {0x09c5b7..=0x09c5ba=>Some([16,240,1,255][(a-0x09c5b7) as usize]),
        0x09c5f6..=0x09c5f9=>Some([16,240,1,255][(a-0x09c5f6) as usize]),
        0x09c731..=0x09c738=>Some([128,64,32,16,8,4,2,1][(a-0x09c731) as usize]),_=>None}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outdoor_proximity_scan_and_allocations_match_original_boundaries() {
        let rom=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
        let mut boundaries=0;let mut visited=std::collections::BTreeSet::new();
        for case in 0..640usize {
            let mut machine=snes::Snes::new();machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
            for (i,b) in machine.ram.iter_mut().enumerate() {*b=(i.wrapping_mul(37)^(i>>8)^case.wrapping_mul(19)) as u8;}
            machine.ram[0x11]=[0,1,20,255][case%4];
            machine.ram[0xfb7]=(case%2) as u8;
            machine.ram[0x69e]=[0,1,255][case/4%3];machine.ram[0x69f]=[0,1,255][case/12%3];
            for (at,v) in [(0xe2,0x210u16),(0xe8,0x230),(0xfb8,[0,0x400][case/36%2]),
                (0xfba,[0,0x400][case/72%2]),(0xfbc,0),(0xfbe,0)] {
                crate::types::write_le_u16(&mut machine.ram,at,v);
            }
            machine.ram[0x1df80..0x1ef80].fill([0,1,0x58,0xd0,0xeb,0x53,0xf3,0xf4,0xf5,0xff][case/8%10]);
            machine.ram[0x1ef80..0x1f180].fill(if case/80%2==0 {0} else {255});
            machine.ram[0xdd0..0xde0].fill(if case/160%2==0 {0} else {8});
            machine.ram[0xb00..0xb08].fill(if case/160%2==0 {0} else {1});
            machine.ram[0xe20..0xe30].fill(0x41);machine.ram[0xdb0..0xdc0].fill(if case/320%2==0 {0} else {1});
            if case>=512 {
                machine.ram[0x11]=1;machine.ram[0x69e]=1;machine.ram[0x69f]=1;
                crate::types::write_le_u16(&mut machine.ram,0xfb8,0x400);
                crate::types::write_le_u16(&mut machine.ram,0xfba,0x400);
                machine.ram[0x1df80..0x1ef80].fill([1,0x58,0xd0,0xeb,0x53,0xf3,0xf4,0xf5,0xff][case%9]);
                machine.ram[0x1ef80..0x1f180].fill(0);
                machine.ram[0xdd0..0xde0].fill(0);machine.ram[0xb00..0xb08].fill(0);
            }
            machine.ram[0x1f7..0x1fa].copy_from_slice(&[0x3e,0x83,6]);
            let mut cpu=NativeNmiContext {a:0xb701,x:0x81,y:0x42,d:0,db:0,bank:9,pc:0xc58f,sp:0x1f6,p:0x30|(case as u8&0xc3)};
            machine.cpu.a=cpu.a;machine.cpu.x=cpu.x;machine.cpu.y=cpu.y;machine.cpu.k=cpu.bank;
            machine.cpu.pc=cpu.pc;machine.cpu.sp=cpu.sp;machine.cpu.e=false;machine.cpu.unpack_flags(cpu.p);
            let mut ram=machine.ram.clone();let clock=CpuMasterTimeline::at_raster(case as u64,
                [snes::CpuRasterPosition::new(100,532),snes::CpuRasterPosition::new(224,1338),snes::CpuRasterPosition::new(261,1350)][case%3],
                snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
            let mut source=snes::RomCpuTimingProbe::new(machine,clock,snes::SourcePpuReadState::snes9x_reset()).unwrap();
            let mut clock=source.timeline().clone();let mut work=OverworldSpriteProximity::default();
            for _ in 0..50000 {
                if work.complete() {break;}
                visited.insert(cpu.pc);
                let label=format!("outdoor proximity case={case} pc={:04x}",cpu.pc);
                super::super::native_work::compare_instruction_with_devices(&label,&mut cpu,&mut clock,&mut source,&mut ram,&rom,None,
                    |_,access|match access {
                        super::super::intro_clear::Access::Read(a,width) if a&0x8000!=0 && a<0x7e0000=> {
                            let read=|a|table_byte(a).unwrap_or_else(||panic!("unowned proximity operand {a:06x}; {label}"));
                            Some(if width==1 {u16::from(read(a))} else {u16::from_le_bytes([read(a),read(a+1)])})
                        }
                        _=>None,
                    },|cpu,clock,bus|{work.step(cpu,clock,bus).unwrap();});
                assert_eq!(ram,source.snes().ram,"{label}");boundaries+=1;
            }
            assert!(work.complete());assert_eq!((cpu.bank,cpu.pc,cpu.sp),(6,0x833f,0x1f9));
        }
        assert!(visited.contains(&0xc7be),"sprite allocation must be exercised");
        assert!(visited.contains(&0xc825),"overlord allocation must be exercised");
        eprintln!("native outdoor proximity cases=640 original boundaries={boundaries} instruction sites={}",visited.len());
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct OverworldSpriteProximity {pc:u16,calls:Vec<(u16,u16)>,complete:bool}
impl Default for OverworldSpriteProximity {fn default()->Self {Self {pc:0xc58f,calls:Vec::new(),complete:false}}}
fn adc<B:WorkBus>(w:&mut Work<'_,B>,rhs:u16,m:u8) {
    if m==2 {w.adc(rhs);return;}
    let a=w.cpu.a&255;let rhs=rhs&255;let sum=a+rhs+u16::from(w.cpu.p&CARRY!=0);
    w.cpu.p=(w.cpu.p&!(CARRY|0x40))|u8::from(sum>255)
        |if (!(a^rhs)&(a^sum)&128)!=0 {0x40} else {0};
    w.load(sum&255,1);
}
impl OverworldSpriteProximity {
    pub(crate) fn complete(&self)->bool {self.complete}
    pub(crate) fn step(&mut self,cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,bus:impl WorkBus)->Option<u64> {
        if self.complete {return None;}
        assert_eq!((cpu.bank,cpu.pc,cpu.d,cpu.p&0x18),(9,self.pc,0,0x10));
        let mut w=Work {cpu,clock,bus,cycles:0};let pc=self.pc;
        let m=if w.cpu.p&0x20!=0 {1} else {2};let a=w.cpu.a&if m==1 {255} else {65535};w.fetch();
        self.pc=match pc {
            0xc58f | 0xc590=>{w.drain(6);w.push(u16::from(if pc==0xc58f {w.cpu.db} else {9}),1);pc+1}
            0xc591 | 0xc59c | 0xc5b5=>{w.drain(12);w.cpu.db=w.pull(1) as u8;w.flags(u16::from(w.cpu.db),1);pc+1}
            0xc59d | 0xc5b6=>{w.rtl();assert!(self.calls.is_empty());self.complete=true;w.cpu.pc}
            0xc596 | 0xc599 | 0xc5a5 | 0xc5af | 0xc5e3 | 0xc622 | 0xc72a | 0xc76c | 0xc776=>{
                let sp=w.cpu.sp;w.operand_then_internal(2);w.push(pc+2,2);self.calls.push((pc+3,sp));
                match pc {0xc596|0xc5a5=>0xc5bb,0xc599|0xc5af=>0xc5fa,0xc5e3|0xc622=>0xc6f5,0xc72a=>0xc739,0xc76c=>0xc770,_=>0xc80b}
            }
            0xc5f5 | 0xc634 | 0xc730 | 0xc76f | 0xc779 | 0xc7ae | 0xc80a | 0xc815 | 0xc880=>{
                w.rts();let (returned,sp)=self.calls.pop().expect("proximity near frame");assert_eq!((w.cpu.bank,w.cpu.pc,w.cpu.sp),(9,returned,sp));w.cpu.pc
            }
            0xc592 | 0xc5c5 | 0xc5cd | 0xc5d4 | 0xc5db | 0xc5e8 | 0xc604 | 0xc60c | 0xc613 | 0xc61a | 0xc627 | 0xc6f7 | 0xc705 | 0xc719 | 0xc71f | 0xc73b | 0xc74f | 0xc760 | 0xc7bc | 0xc7cf | 0xc7d8 | 0xc7df | 0xc7e6 | 0xc823 | 0xc834 | 0xc845 | 0xc84c | 0xc853=>{
                let at=match pc {0xc592=>0x11,0xc5c5|0xc613=>0xe2,0xc5cd|0xc61a=>0xe3,
                    0xc5d4|0xc604=>0xe8,0xc5db|0xc60c=>0xe9,0xc5e8|0xc705|0xc719=>0xc,0xc627|0xc6f7|0xc71f=>0xe,
                    0xc7df|0xc7e6|0xc84c|0xc853=>1,_=>0};let v=w.dp_read(at,m);w.load(v,m);pc+2
            }
            0xc59e | 0xc5a8 | 0xc5bd | 0xc5fc | 0xc871=>{w.drain(16);let v=w.read(w.absolute(match pc {0xc59e|0xc5a8=>0xfb7,0xc5bd=>0x69f,0xc5fc=>0x69e,_=>0x40a}),m);w.load(v,m);pc+3}
            0xc5a1 | 0xc5ab | 0xc71b | 0xc762 | 0xc7da | 0xc7e1 | 0xc847 | 0xc84e=>{w.mask(match pc {0xc5a1|0xc5ab=>1,0xc762=>7,0xc7e1|0xc84e=>3,_=>0xf0},m);pc+2}
            0xc594 | 0xc5ad | 0xc5c0 | 0xc5ff | 0xc74b | 0xc77e | 0xc784 | 0xc78a | 0xc78e | 0xc792 | 0xc79d | 0xc810=>{
                if w.branch(w.cpu.p&ZERO!=0) {match pc {0xc594=>0xc59e,0xc5ad=>0xc5b2,0xc5c0=>0xc5f5,0xc5ff=>0xc634,0xc74b=>0xc76f,
                    0xc77e|0xc784=>0xc79a,0xc78a|0xc78e|0xc792=>0xc798,0xc79d=>0xc7af,_=>0xc816}} else {pc+2}
            }
            0xc5a3 | 0xc76a | 0xc796 | 0xc7a4 | 0xc7a9 | 0xc83d=>{
                if w.branch(w.cpu.p&ZERO==0) {match pc {0xc5a3=>0xc5a8,0xc76a=>0xc76f,0xc796=>0xc79a,0xc7a4=>0xc7ab,0xc7a9=>0xc7af,_=>0xc842}} else {pc+2}
            }
            0xc5c2 | 0xc601 | 0xc5f3 | 0xc632 | 0xc7ac | 0xc813=>{
                if w.branch(w.cpu.p&NEGATIVE==0) {match pc {0xc5c2=>0xc5c5,0xc601=>0xc604,0xc5f3=>0xc5e3,0xc632=>0xc622,0xc7ac=>0xc79a,_=>0xc80d}} else {pc+2}
            }
            0xc700 | 0xc70e=>{if w.branch(w.cpu.p&CARRY!=0) {0xc72e} else {pc+2}}
            0xc774=>{if w.branch(w.cpu.p&CARRY==0) {0xc77a} else {pc+2}}
            0xc5b2=>{w.drain(16);let old=w.read(w.absolute(0xfb7),1);let v=old.wrapping_add(1)&255;w.drain(6);w.flags(v,1);w.write(w.absolute(0xfb7),v,1);pc+3}
            0xc5bb | 0xc5fa=>{w.drain(8);w.cpu.y=0;w.flags(0,1);pc+2}
            0xc5c4 | 0xc603=>{w.increment_y();w.cpu.y&=255;w.flags(w.cpu.y,1);pc+1}
            0xc5c7 | 0xc5ea | 0xc606 | 0xc629 | 0xc73d | 0xc754 | 0xc7f0 | 0xc7fa | 0xc83f | 0xc85d | 0xc867=>{w.clear_carry();pc+1}
            0xc5c8 | 0xc5cf | 0xc607 | 0xc60e | 0xc767 | 0xc7b1 | 0xc818=>{
                w.drain(16);let base=match pc {0xc5c8=>0xc5b7,0xc5cf=>0xc5b9,0xc607=>0xc5f6,0xc60e=>0xc5f8,_=>0xc731};
                let v=w.read(w.absolute(base)+u32::from(w.cpu.y),m);
                if matches!(pc,0xc767) {w.load(a&v,m);} else if matches!(pc,0xc7b1|0xc818) {w.load(a|v,m);} else {adc(&mut w,v,m);}pc+3
            }
            0xc5cb | 0xc5d2 | 0xc5d9 | 0xc5df | 0xc5ee | 0xc60a | 0xc611 | 0xc618 | 0xc61e | 0xc62d | 0xc703 | 0xc717 | 0xc71d | 0xc727 | 0xc741 | 0xc747 | 0xc758 | 0xc75e=>{
                let at=match pc {0xc5cb|0xc618|0xc62d=>0xe,0xc5d2|0xc61e=>0xf,0xc5d9|0xc60a|0xc5ee=>0xc,0xc5df|0xc611=>0xd,
                    0xc717=>1,0xc741=>5,0xc747=>7,0xc758=>2,0xc75e=>4,_=>0};w.dp_store(at,a,m);pc+2
            }
            0xc5d6 | 0xc615 | 0xc6f9 | 0xc707 | 0xc82d=>{w.drain(6);w.cpu.p|=CARRY;pc+1}
            0xc5d7 | 0xc5dd | 0xc616 | 0xc61c | 0xc82e=>{w.drain(8*u32::from(m));w.sbc(match pc {0xc5d7|0xc616=>0x30,0xc82e=>0xf3,_=>0},m);pc+1+u16::from(m)}
            0xc5e1 | 0xc620 | 0xc77a | 0xc780 | 0xc786 | 0xc798 | 0xc80b=>{w.drain(8);w.cpu.x=match pc {0xc5e1|0xc620=>0x15,0xc77a=>4,0xc780=>5,0xc786=>13,0xc798=>14,_=>7};w.flags(w.cpu.x,1);pc+2}
            0xc5e6 | 0xc625 | 0xc6f5 | 0xc739 | 0xc74d | 0xc7ba | 0xc821=>{w.width(0x20,true);pc+2}
            0xc5f0 | 0xc62f | 0xc710 | 0xc72e | 0xc743 | 0xc75a | 0xc7c1 | 0xc828=>{w.width(0x20,false);pc+2}
            0xc5eb | 0xc62a | 0xc73e | 0xc755 | 0xc840=>{w.drain(8*u32::from(m));adc(&mut w,match pc {0xc5eb|0xc62a=>16,0xc73e=>0xdf80,0xc755=>0xef80,_=>8},m);pc+1+u16::from(m)}
            0xc5f2 | 0xc631 | 0xc7ab | 0xc812=>{w.decrement_x();w.cpu.x&=255;w.flags(w.cpu.x,1);pc+1}
            0xc6fa | 0xc708=>{w.drain(16);let v=w.read(w.absolute(if pc==0xc6fa {0xfbc} else {0xfbe}),m);w.sbc(v,m);pc+3}
            0xc6fd | 0xc70b=>{w.drain(16);let v=w.read(w.absolute(if pc==0xc6fd {0xfb8} else {0xfba}),m);w.cpu.p=(w.cpu.p&!CARRY)|u8::from(a>=v);w.flags(a.wrapping_sub(v),m);pc+3}
            0xc702 | 0xc712=>{w.swap_bytes();pc+1}
            0xc713 | 0xc714 | 0xc7b8 | 0xc81f | 0xc7d1..=0xc7d4 | 0xc836..=0xc839=>{w.shift_left(m);pc+1}
            0xc715 | 0xc725=>{let v=w.dp_read(0,m);w.load(a|v,m);pc+2}
            0xc721..=0xc724 | 0xc751..=0xc753 | 0xc7e8 | 0xc7e9 | 0xc855 | 0xc856=>{w.drain(6);w.cpu.p=(w.cpu.p&!CARRY)|((a&1) as u8);w.load(a>>1,m);pc+1}
            0xc729 | 0xc7b6 | 0xc81d=>{w.drain(6);w.push(w.cpu.x,1);pc+1}
            0xc72d | 0xc7c3 | 0xc82a=>{w.drain(12);w.cpu.x=w.pull(1);w.flags(w.cpu.x,1);pc+1}
            0xc745 | 0xc75c | 0xc7ca=>{w.immediate(if pc==0xc7ca {8} else {0x7f},m);pc+2}
            0xc749 | 0xc765 | 0xc770 | 0xc7af | 0xc7c4 | 0xc816 | 0xc82b=>{let v=w.indirect_read(if matches!(pc,0xc765|0xc7af|0xc816) {2} else {5},m);w.load(v,m);pc+2}
            0xc764=>{w.drain(6);w.cpu.y=a;w.flags(a,1);pc+1}
            0xc772 | 0xc77c | 0xc782 | 0xc788 | 0xc78c | 0xc790 | 0xc794 | 0xc7a2=>{w.compare_immediate(match pc {0xc772=>0xf4,0xc77c|0xc788=>0x58,0xc782=>0xd0,0xc78c=>0xeb,0xc790=>0x53,0xc794=>0xf3,_=>0x41},m);pc+2}
            0xc79a | 0xc79f | 0xc7a6 | 0xc7ed | 0xc7f7 | 0xc80d | 0xc85a | 0xc864=>{
                w.drain(16);let base=match pc {0xc79a=>0xdd0,0xc79f=>0xe20,0xc7a6=>0xdb0,0xc7ed=>0xd30,0xc7f7=>0xd20,0xc80d=>0xb00,0xc85a=>0xb10,_=>0xb20};let v=w.read(w.absolute(base)+u32::from(w.cpu.x),m);w.load(v,m);pc+3
            }
            0xc7b4 | 0xc81b=>{w.drain(8);let low=w.read(2,2);let bank=w.read(4,1);w.write((u32::from(bank)<<16)|u32::from(low),a,m);pc+2}
            0xc7b7 | 0xc81e=>{w.drain(6);w.load(w.cpu.x,m);pc+1}
            0xc7b9 | 0xc820=>{w.drain(6);w.cpu.x=a&255;w.flags(w.cpu.x,1);pc+1}
            0xc7be | 0xc7c7 | 0xc7cc | 0xc7d5 | 0xc7dc | 0xc7e3 | 0xc7ea | 0xc7f4 | 0xc7fe | 0xc825 | 0xc830 | 0xc842 | 0xc849 | 0xc850 | 0xc857 | 0xc861 | 0xc86b | 0xc874=>{
                w.drain(16);w.drain(6);let base=match pc {0xc7be=>0xbc0,0xc7c7=>0xe20,0xc7cc=>0xdd0,0xc7d5=>0xd10,0xc7dc=>0xd00,0xc7e3|0xc7f4=>0xd30,0xc7ea|0xc7fe=>0xd20,0xc825=>0xb48,0xc830=>0xb00,0xc842=>0xb08,0xc849=>0xb18,0xc850|0xc861=>0xb10,0xc857|0xc86b=>0xb20,_=>0xcca};w.write(w.absolute(base)+u32::from(w.cpu.x),a,m);pc+3
            }
            0xc7c6=>{w.drain(6);w.load(a.wrapping_sub(1)&255,m);pc+1}
            0xc7f1 | 0xc7fb | 0xc85e | 0xc868=>{w.drain(16);let v=w.read(w.absolute(if matches!(pc,0xc7f1|0xc85e) {0xfbd} else {0xfbf}),m);adc(&mut w,v,m);pc+3}
            0xc801 | 0xc804 | 0xc807 | 0xc86e | 0xc877 | 0xc87a | 0xc87d=>{w.drain(16);w.drain(6);w.write(w.absolute(match pc {0xc801=>0xf20,0xc804=>0xe30,0xc807=>0xcba,0xc86e=>0xb40,0xc877=>0xb30,0xc87a=>0xb28,_=>0xb38})+u32::from(w.cpu.x),0,m);pc+3}
            0xc833=>{w.drain(6);w.push(a,m);pc+1}
            0xc83a=>{w.drain(12);w.cpu.y=w.pull(1);w.flags(w.cpu.y,1);pc+1}
            0xc83b=>{w.drain(8);w.cpu.p=(w.cpu.p&!CARRY)|u8::from(w.cpu.y>=1);w.flags(w.cpu.y.wrapping_sub(1)&255,1);pc+2}
            _=>unreachable!("overworld proximity PC {pc:04x}"),
        };w.cpu.pc=self.pc;Some(w.cycles)
    }
}
