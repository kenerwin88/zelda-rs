//! The two map32 byte lanes retain their indices, data bank and near frame.
use super::{native_work::{Work, WorkBus}, nmi_stack::{NativeNmiContext, CARRY}};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Map32Interlace { pc: u16, complete: bool }
impl Map32Interlace {
    pub(crate) fn new(high: bool) -> Self { Self { pc: if high {0xf638} else {0xf679}, complete:false } }
    pub(crate) fn complete(&self) -> bool { self.complete }
    pub(crate) fn step(&mut self, cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline,
        bus: impl WorkBus) -> Option<u64> {
        if self.complete {return None;}
        assert_eq!((cpu.bank,cpu.pc,cpu.d,cpu.p&8),(2,self.pc,0,0));
        let mut w = Work {cpu,clock,bus,cycles:0};let pc=self.pc;w.fetch();
        self.pc=match pc {
            0xf638 | 0xf64e | 0xf679 | 0xf68d => {w.width(0x20,false);pc+2}
            0xf641 | 0xf675 | 0xf682 => {w.width(0x30,true);pc+2}
            0xf63a | 0xf67b => {w.drain(6);w.push(u16::from(w.cpu.db),1);pc+1}
            0xf63b | 0xf67c => {w.immediate(0x7f,1);pc+2}
            0xf63d | 0xf67e => {w.drain(6);w.push(w.cpu.a&255,1);pc+1}
            0xf63e | 0xf67f | 0xf677 => {w.drain(12);let v=w.pull(1);w.cpu.db=v as u8;w.flags(v,1);pc+1}
            0xf63f | 0xf680 => {w.dp_store(2,w.cpu.a&255,1);pc+2}
            0xf643 | 0xf684 => {w.drain(16);w.cpu.x=0;w.flags(0,2);pc+3}
            0xf646 => {w.drain(16);w.cpu.y=1;w.flags(1,2);pc+3}
            0xf687 => {w.drain(6);w.cpu.y=w.cpu.x;w.flags(w.cpu.y,2);pc+1}
            0xf649 | 0xf688 => {w.immediate(0x4000,2);pc+3}
            0xf64c | 0xf68b => {w.dp_store(0,w.cpu.a,2);pc+2}
            0xf68f => {w.branch(true);0xf650}
            0xf650 | 0xf658 | 0xf660 | 0xf668 => {
                w.drain(16);if w.cpu.p&0x10==0 || w.cpu.x>255 {w.drain(6);}
                let v=w.read(w.absolute(0x4400).wrapping_add(u32::from(w.cpu.x)),1);w.load(v,1);pc+3
            }
            0xf653 | 0xf65b | 0xf663 | 0xf66b => {let at=w.indirect_y_address(0);w.write(at,w.cpu.a&255,1);pc+2}
            0xf655 | 0xf656 | 0xf65d | 0xf65e | 0xf665 | 0xf666 | 0xf66d | 0xf66e => {w.increment_y();pc+1}
            0xf657 | 0xf65f | 0xf667 | 0xf66f => {w.increment_x();pc+1}
            0xf670 => {w.compare_x_immediate(0x100,2);pc+3}
            0xf673 => {if w.branch(w.cpu.p&CARRY==0) {0xf650} else {pc+2}}
            0xf678 => {w.rts();self.complete=true;w.cpu.pc}
            _=>unreachable!("map32 interlace PC {pc:04x}"),
        };w.cpu.pc=self.pc;Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::native_work::compare_instruction;
    #[test]
    fn map32_high_and_low_interlace_keep_source_memory_and_data_bank() {
        let rom=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let mut boundaries=0;
        for case in 0..144usize {
            let high=case&1==0;
            let mut machine=snes::Snes::new();machine.cart.load(snes::CartType::LoRom,&rom,0x2000);
            for (i,v) in machine.ram.iter_mut().enumerate() {*v=(i.wrapping_mul(37)^(i>>8)^case.wrapping_mul(19)) as u8;}
            let returned=if high {0xf80du16} else {0xf82f};
            machine.ram[0x1f7..0x1f9].copy_from_slice(&returned.wrapping_sub(1).to_le_bytes());
            let mut cpu=NativeNmiContext {a:0xb701,x:0x8312,y:0x9f7c,d:0,
                db:[0,0x7f,0x80,0x82][case/2%4],bank:2,pc:if high {0xf638} else {0xf679},
                sp:0x1f6,p:[0,1,2,0x43,0x80,0xc3][case/2%6]};
            let saved_db=cpu.db;
            machine.cpu.a=cpu.a;machine.cpu.x=cpu.x;machine.cpu.y=cpu.y;
            machine.cpu.dp=cpu.d;machine.cpu.db=cpu.db;machine.cpu.k=cpu.bank;
            machine.cpu.pc=cpu.pc;machine.cpu.sp=cpu.sp;machine.cpu.e=false;machine.cpu.unpack_flags(cpu.p);
            let mut ram=machine.ram.clone();let clock=CpuMasterTimeline::at_raster(case as u64,
                [snes::CpuRasterPosition::new(100,532),snes::CpuRasterPosition::new(224,1338),
                 snes::CpuRasterPosition::new(261,1350)][case/2%3],
                snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
            let mut source=snes::RomCpuTimingProbe::new(machine,clock,
                snes::SourcePpuReadState::snes9x_reset()).unwrap();let mut clock=source.timeline().clone();
            let mut work=Map32Interlace::new(high);
            for _ in 0..4096 {
                if work.complete() {break;}
                compare_instruction(&format!("interlace case={case} high={high} pc={:04x}",cpu.pc),
                    &mut cpu,&mut clock,&mut source,&mut ram,&rom,
                    |cpu,clock,bus| {work.step(cpu,clock,bus).unwrap();});
                assert_eq!(ram,source.snes().ram);boundaries+=1;
            }
            assert!(work.complete());assert_eq!((cpu.bank,cpu.pc,cpu.sp,cpu.db),(2,returned,0x1f8,saved_db));
        }
        eprintln!("native map32 interlace Source cases=144 instruction boundaries={boundaries}");
    }
}
