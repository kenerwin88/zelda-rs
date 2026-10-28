//! Ordered, exclusive accounting. A caller must prove the partition and call
//! mapping before constructing keys; routine-name equality is not that proof.
use crate::{Conservation, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Key {
    pub call: u64,
    pub pc: u32,
    pub kind: u8,
    pub occurrence: u64,
    pub m8: bool,
    pub x8: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub key: Key,
    pub clocks: u64,
    pub site: String,
    /// Only instruction partitions with proven disjoint ownership may pair.
    pub domain: String,
}
#[derive(Debug, Serialize)]
pub struct Row {
    pub key: Key,
    pub site: String,
    pub domain: String,
    pub source: u64,
    pub native: u64,
    pub delta: i128,
    pub status: &'static str,
}

/// Strict occurrence order. On a mismatch consume only a provably absent
/// occurrence. Crossing/reordered paths stay unmatched on both sides.
/// Scalars/VWF remain explicit, even when their numeric totals agree.
pub fn pair(source:&[Event],native:&[Event])->Result<(Conservation,Vec<Row>)> {
    let mut c=Conservation {source:0,native:0,paired_source:0,paired_native:0,
        unmatched_source:0,unmatched_native:0,signed_error:0,unequal_pairs:0};
    for e in source {c.source=c.source.checked_add(e.clocks).ok_or("source overflow")?;}
    for e in native {c.native=c.native.checked_add(e.clocks).ok_or("native overflow")?;}
    let mut rows=Vec::new();let (mut s,mut n)=(0,0);
    while s<source.len() || n<native.len() {
        let a=source.get(s);let b=native.get(n);
        let matches=|a:&Event,b:&Event| a.key==b.key && a.site==b.site && a.domain=="instruction" && b.domain=="instruction";
        let (a,b,status)=match (a,b) {
            (Some(a),Some(b)) if matches(a,b)=>{s+=1;n+=1;(Some(a),Some(b),"paired")},
            (Some(a),Some(_)) if !native[n..].iter().any(|v|matches(a,v))=>{s+=1;(Some(a),None,"source_unmatched")},
            (Some(_),Some(b)) if !source[s..].iter().any(|v|matches(v,b))=>{n+=1;(None,Some(b),"native_unmatched")},
            (Some(a),Some(b))=>{s+=1;n+=1;(Some(a),Some(b),"ordered_path_mismatch")},
            (Some(a),None)=>{s+=1;(Some(a),None,"source_unmatched")},
            (None,Some(b))=>{n+=1;(None,Some(b),"native_unmatched")},
            _=>unreachable!(),
        };
        if status=="ordered_path_mismatch" {
            // Preserve each PC/site separately; never net two ownership failures.
            for (e,is_source) in [(a.unwrap(),true),(b.unwrap(),false)] {
                let (sc,nc)=if is_source {(e.clocks,0)}else{(0,e.clocks)};
                c.unmatched_source+=sc;c.unmatched_native+=nc;
                c.unequal_pairs+=1;
                rows.push(Row {key:e.key.clone(),site:e.site.clone(),domain:e.domain.clone(),source:sc,native:nc,
                    delta:i128::from(sc)-i128::from(nc),status});
            }
            continue;
        }
        let sc=a.map_or(0,|e|e.clocks);let nc=b.map_or(0,|e|e.clocks);
        let delta=i128::from(sc)-i128::from(nc);
        if status=="paired" {
            c.paired_source+=sc;c.paired_native+=nc;c.signed_error+=delta;
            c.unequal_pairs+=usize::from(delta!=0);
        } else {
            c.unmatched_source+=sc;c.unmatched_native+=nc;
            // Identity/domain mismatches are failures even for zero-clock
            // observations; clock conservation alone cannot see their absence.
            c.unequal_pairs+=1;
        }
        let e=b.or(a).unwrap();
        rows.push(Row {key:e.key.clone(),site:e.site.clone(),domain:e.domain.clone(),source:sc,native:nc,delta,status});
    }
    c.check()?;Ok((c,rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn e(pc:u32,clocks:u64)->Event {Event {key:Key {call:1,pc,kind:0,occurrence:0,m8:true,x8:true},clocks,
        site:format!("site-{pc:x}"),domain:"instruction".into()}}
    #[test] fn missing_duplicate_and_wrong_cost_keep_signed_sites() {
        let source=vec![e(0x8038,12),e(0x8039,18)];
        let (c,r)=pair(&source,&[e(0x8039,20)]).unwrap();
        assert_eq!((c.unmatched_source,c.signed_error),(12,-2));
        assert_eq!((r[0].key.pc,r[0].delta),(0x8038,12));
        assert_eq!((&*r[1].site,r[1].delta),("site-8039",-2));
        let (c,r)=pair(&source,&[e(0x8038,12),e(0x8038,12),e(0x8039,18)]).unwrap();
        assert_eq!(c.unmatched_native,12);assert_eq!(r[1].delta,-12);
    }
    #[test] fn cancellation_and_reordering_never_pass() {
        let a=vec![e(1,10),e(2,10)];
        let (c,r)=pair(&a,&[e(1,8),e(2,12)]).unwrap();
        assert!(!c.check().unwrap());assert_eq!(c.signed_error,0);
        assert_eq!((r[0].delta,r[1].delta),(2,-2));
        assert!(!pair(&a,&[e(2,10),e(1,10)]).unwrap().0.check().unwrap());
    }
    #[test] fn width_call_and_domain_are_not_aliases() {
        for variant in 0..4 {
            let a=e(1,10);let mut b=a.clone();
            match variant {0=>b.key.x8=false,1=>b.key.call=2,2=>b.domain="vwf_separate".into(),_=>b.site="wrong_site".into()}
            let (c,_)=pair(&[a],&[b]).unwrap();
            assert_eq!((c.unmatched_source,c.unmatched_native),(10,10));
        }
    }
    #[test] fn zero_clock_duplicate_or_unmapped_domain_is_not_exact() {
        let (c,r)=pair(&[],&[e(1,0)]).unwrap();
        assert!(!c.check().unwrap());assert_eq!(r[0].status,"native_unmatched");
        let mut scalar=e(1,0);scalar.domain="scalar_unmapped".into();
        assert!(!pair(&[e(1,0)],&[scalar]).unwrap().0.check().unwrap());
    }
}
