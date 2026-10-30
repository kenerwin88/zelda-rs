//! Streaming source retirement inventory. Unknown ownership is never free work.
//! Only verified logical rows are committed; the final open row is discarded.
use crate::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, HashMap}, io::Write};

#[derive(Default)]
pub struct Inventory {
    ordered: Vec<u8>,
    extras: std::collections::HashSet<u32>,
    pending: HashMap<(u32, u8), [u64; 2]>,
    totals: BTreeMap<(u32, u8), [u64; 2]>,
    digest: Sha256,
    count: u64,
    clocks: u64,
    pub verified_rows: u64,
}
#[path = "inventory_ordered.rs"]
mod ordered_stream;
#[derive(Serialize)]
pub struct PcWork {
    pub pc: u32,
    pub width: u8,
    pub occurrences: u64,
    pub master: u64,
    pub ownership: &'static str,
    pub blocker: &'static str,
}
impl Inventory {
    pub fn retire(&mut self, pc: u32, m8: bool, x8: bool, clocks: u64) {
        let width = u8::from(m8) | (u8::from(x8) << 1);
        let row = self.pending.entry((pc, width)).or_default();
        row[0] += 1; row[1] += clocks;
        self.digest.update(pc.to_le_bytes());
        self.digest.update([width]); self.digest.update(clocks.to_le_bytes());
        self.count += 1; self.clocks += clocks;
    }
    pub fn verify_row(&mut self, iteration: u64, out: &mut impl Write) -> Result<()> {
        if iteration != self.verified_rows { return Err("inventory noncontiguous verified row".into()); }
        let count: u64 = self.pending.values().map(|r|r[0]).sum();
        let clocks: u64 = self.pending.values().map(|r|r[1]).sum();
        if (count, clocks) != (self.count, self.clocks) { return Err("inventory source conservation".into()); }
        // Retain the ordered digest and all per-PC work. No dynamic occurrence
        // pairing may be inferred from this deliberately compressed inventory.
        let work: Vec<_> = self.pending.iter().map(|(&(pc,width),&v)|(pc,width,v)).collect();
        let mut work = work; work.sort_unstable_by_key(|r|(r.0,r.1));
        writeln!(out,"{}",serde_json::json!({"iteration":iteration,"instructions":count,
            "source_master":clocks,"ordered_sha256":format!("{:x}",self.digest.clone().finalize()),
            "pc_work":work}))?;
        for (key, value) in self.pending.drain() {
            let row = self.totals.entry(key).or_default();
            row[0] += value[0]; row[1] += value[1];
        }
        self.count=0; self.clocks=0; self.digest=Sha256::new(); self.verified_rows+=1;
        Ok(())
    }
    pub fn rows(&self) -> Vec<PcWork> {
        self.totals.iter().map(|(&(pc,width),&v)|PcWork {pc,width,occurrences:v[0],master:v[1],
            ownership:"d",blocker:"native_dynamic_instruction_claim_not_paired"}).collect()
    }
}

/// A resume pair requires the next instruction and the *retired* prefix, not
/// merely a publication stage or equal numeric work. Scope IDs are native IDs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Handoff {
    pub iteration: u64, pub next_pc: u32, pub retired: u64,
    pub stage: String, pub scopes: Vec<u32>,
}
pub fn handoff(source: &Handoff, native: &Handoff) -> &'static str {
    if source.iteration != native.iteration {"resume_iteration_mismatch"}
    else if native.stage.is_empty() {"native_continuation_stage_unavailable"}
    else if source.stage != native.stage {"native_continuation_stage_mismatch"}
    else if source.next_pc != native.next_pc {"native_continuation_pc_mismatch"}
    else if source.retired != native.retired {"native_retired_cursor_mismatch"}
    else if source.scopes != native.scopes {"native_scope_call_mapping_unproved"}
    else {"paired_retirement_metadata_only"}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn verified_rows_exclude_open_suffix_and_preserve_width() {
        let mut i=Inventory::default(); let mut out=Vec::new();
        i.retire(0x8038,true,true,14); i.retire(0x8038,false,true,22);
        i.verify_row(0,&mut out).unwrap(); i.retire(0xdead,true,true,100);
        assert_eq!(i.rows().len(),2);
        assert_eq!(i.rows().iter().map(|r|r.master).sum::<u64>(),36);
        assert!(i.verify_row(2,&mut out).is_err());
    }
    #[test] fn resumed_a95b_requires_cursor_and_scope_not_publication() {
        let a=Handoff {iteration:1916,next_pc:0x0da95b,retired:10555,
            stage:"LinkOam::suffix_step".into(),scopes:vec![0x8034,0xda18e]};
        assert_eq!(handoff(&a,&a),"paired_retirement_metadata_only");
        let mut b=a.clone(); b.retired-=1;
        assert_eq!(handoff(&a,&b),"native_retired_cursor_mismatch");
        b=a.clone(); b.next_pc=0xda958;
        assert_eq!(handoff(&a,&b),"native_continuation_pc_mismatch");
        b=a.clone(); b.scopes.pop();
        assert_eq!(handoff(&a,&b),"native_scope_call_mapping_unproved");
        b=a.clone();b.stage="publication".into();
        assert_eq!(handoff(&a,&b),"native_continuation_stage_mismatch");
    }
    #[test] fn missing_duplicate_and_wrong_owner_have_separate_signed_residuals() {
        use crate::ordered::{pair,Event,Key};
        let e=Event {key:Key {call:1,pc:0xda95b,kind:0,occurrence:0,m8:true,x8:true},
            clocks:14,site:"LinkOam::suffix_step".into(),domain:"instruction".into()};
        assert_eq!(pair(&[e.clone()],&[]).unwrap().1[0].delta,14);
        assert_eq!(pair(&[e.clone()],&[e.clone(),e.clone()]).unwrap().1[1].delta,-14);
        let mut wrong=e.clone(); wrong.site="publication".into();
        let (c,r)=pair(&[e],&[wrong]).unwrap();
        assert!(!c.check().unwrap());
        assert_eq!(r.iter().map(|r|r.delta).collect::<Vec<_>>(),vec![14,-14]);
    }
}
