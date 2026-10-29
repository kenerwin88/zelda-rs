//! Streaming source-only domain observations. No scheduling or price feedback.
use crate::Result;
use snes::{SourceCpuAcceptedInterrupt as Interrupt, SourceCpuStepReceipt};
use std::{collections::BTreeMap, io::Write};

#[derive(Default)]
pub struct Domains {
    index:u64,
    owner:&'static str,
    stack:Vec<&'static str>,
    rows:BTreeMap<&'static str,[u64;6]>,
}
impl Domains {
    fn flush(&mut self,out:&mut impl Write)->Result<()> {
        writeln!(out,"{}",serde_json::json!({"iteration":self.index,"domains":self.rows,
            "columns":["wall","cpu","refresh","dma","instructions","refresh_edges"]}))?;
        self.rows.clear();Ok(())
    }
    pub fn observe(&mut self,step:&SourceCpuStepReceipt,nmi_boolean:u8,next:u64,sp:u16,
        refresh:u64,dma:u64,out:&mut impl Write)->Result<()> {
        if step.origin_pc==0x8038 && self.stack.is_empty() {self.owner="main";}
        let owner=if self.stack.is_empty() && sp>>8!=1 {"poly"}
            else if self.owner.is_empty(){"idle"}else{self.owner};
        let interrupt=match step.accepted_interrupt {
            Some(Interrupt::Nmi{started_at,..})=>Some((started_at,true)),
            Some(Interrupt::Irq{started_at,..})=>Some((started_at,false)),None=>None};
        let end=interrupt.map_or(step.ended_at,|(at,_)|at);
        let row=self.rows.entry(owner).or_default();
        row[0]+=end.master_cycles()-step.started_at.master_cycles();
        row[1]+=step.transactions.iter().filter(|t|t.started_at<end).map(|t|u64::from(t.duration_master_cycles)).sum::<u64>();
        row[2]+=40*refresh;row[3]+=dma;row[4]+=1;
        row[5]+=u64::from(interrupt.is_some() && refresh!=0);
        if step.opcode==0x40 {self.owner=self.stack.pop().unwrap_or("idle");}
        if step.origin_pc==0x805d && self.stack.is_empty(){self.owner="idle";}
        if let Some((at,nmi))=interrupt {
            let new_owner=if nmi && nmi_boolean==0 {
                self.flush(out)?;self.index=next;self.stack.clear();self.owner="idle";"entry"
            }else if nmi{"lag"}else{"irq"};
            self.stack.push(self.owner);self.owner=new_owner;
            let row=self.rows.entry(new_owner).or_default();
            row[0]+=step.ended_at.master_cycles()-at.master_cycles();
            row[1]+=step.transactions.iter().filter(|t|t.started_at>=at).map(|t|u64::from(t.duration_master_cycles)).sum::<u64>();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test] fn interrupt_edge_is_split_and_refresh_ambiguity_is_retained() {
        use snes::{CpuMasterTimestamp as T,SourceCpuAcceptedInterrupt as I,SourceCpuStepReceipt as S};
        let step=S {origin_pc:0x9000,opcode:0xea,started_at:T::new(100),ended_at:T::new(200),
            accesses:vec![],transactions:vec![],accepted_interrupt:Some(I::Nmi{started_at:T::new(150),interrupted_pc:0x9001})};
        let mut d=super::Domains::default();d.index=41;d.owner="main";
        let mut out=Vec::new();d.observe(&step,0,42,511,1,0,&mut out).unwrap();
        let row:serde_json::Value=serde_json::from_slice(&out).unwrap();
        assert_eq!(row["iteration"],41);assert_eq!(row["domains"]["main"],serde_json::json!([50,0,40,0,1,1]));
        assert_eq!(d.index,42);assert_eq!(d.rows["entry"],[50,0,0,0,0,0]);
    }
    #[test] fn streaming_flush_preserves_signed_wall_residual() {
        let mut d=super::Domains::default();d.index=42;
        d.rows.insert("main",[100,60,40,0,2,0]);
        let mut out=Vec::new();d.flush(&mut out).unwrap();
        let v:serde_json::Value=serde_json::from_slice(&out).unwrap();
        assert_eq!(v["iteration"],42);assert_eq!(v["domains"]["main"][0],100);
        assert!(d.rows.is_empty());
    }
}
