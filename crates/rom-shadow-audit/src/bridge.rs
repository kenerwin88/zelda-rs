//! Continuous-lane bridge. No fork, native state import, or pricing feedback.
//! Rules fail closed: a source call is identified by its real return PC/SP;
//! generated blocks own decoded opcodes, never inclusive callee work.
use crate::{ordered::{self, Event, Key}, Conservation, Identity, Manifest, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::{BTreeMap, BTreeSet}, fs::{self, File}, io::{BufRead, BufReader, Write}, path::Path};

#[path = "graphics_widths.rs"]
mod graphics_widths;
#[path = "local_exclusion.rs"]
mod local_exclusion;
mod generated { include!("../../zelda3/src/rom_block_costs.rs"); }
const LENGTHS:[u8;256]=[
    2,2,2,2,2,2,2,2,1,2,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,3,1,1,3,3,3,4,
    3,2,4,2,2,2,2,2,1,2,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,3,1,1,3,3,3,4,
    1,2,2,2,3,2,2,2,1,2,1,1,3,3,3,4,
    2,2,2,2,3,2,2,2,1,3,1,1,4,3,3,4,
    1,2,3,2,2,2,2,2,1,2,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,3,1,1,3,3,3,4,
    2,2,3,2,2,2,2,2,1,2,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,3,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,2,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,3,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,2,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,3,1,1,3,3,3,4,
    2,2,2,2,2,2,2,2,1,2,1,1,3,3,3,4,
    2,2,2,2,3,2,2,2,1,3,1,1,3,3,3,4,
];
fn byte(rom:&[u8],pc:u32)->Result<u8> {
    if pc&0xffff<0x8000 || pc>>16&0x7f>=0x40 {return Err("non-LoROM code address".into());}
    rom.get(((pc>>16&0x7f)*0x8000+(pc&0x7fff)) as usize).copied().ok_or_else(||"ROM decode bounds".into())
}
fn length(op:u8,m:bool,x:bool)->u32 {
    u32::from(LENGTHS[op as usize])+u32::from(match op {
        0x09|0x29|0x49|0x69|0x89|0xa9|0xc9|0xe9=>!m,
        0xa0|0xa2|0xc0|0xe0=>!x,_=>false})
}
#[derive(Clone, Debug, Deserialize)]
pub struct Instruction {
    pub iteration:u64, pub ordinal:u64, pub pc:u32, pub opcode:u8, pub master:u64,
    pub m8:bool, pub x8:bool, pub dp:u16, pub db:u8, pub sp:u16,
    pub next_pc:u32, pub next_sp:u16, pub interrupt_after:bool,
    #[serde(default)] pub retired_next_pc:Option<u32>,
    #[serde(default)] pub x:Option<u16>, #[serde(default)] pub y:Option<u16>,
    #[serde(default)] pub emulation:bool,
}
#[derive(Debug, Serialize)]
struct Call {
    id:u64, parent:Option<u64>, entry:u32, aliases:BTreeSet<u32>,
    return_pc:u32, return_sp:u16, entry_sp:u16, closed:bool,
}
#[derive(Debug)]
struct Calls { frames:Vec<Call>, owners:Vec<u64>, rules:BTreeMap<&'static str,u64> }
fn hit(rules:&mut BTreeMap<&'static str,u64>,rule:&'static str) {*rules.entry(rule).or_default()+=1;}
fn calls(rows:&[Instruction])->Result<Calls> {
    if rows.first().map(|r|r.pc)!=Some(0x8038) || rows.last().map(|r|r.pc)!=Some(0x805d) {return Err("incomplete continuous iteration".into());}
    let mut out=Calls {frames:vec![Call {id:0,parent:None,entry:0x8034,aliases:[0x8034,0x8038].into(),
        return_pc:0,return_sp:0,entry_sp:rows[0].sp,closed:true}],owners:Vec::new(),rules:BTreeMap::new()};
    let mut stack=vec![0usize];
    for (j,r) in rows.iter().enumerate() {
        if r.ordinal!=j as u64 {return Err("noncontiguous source opcodes".into());}
        let owner=*stack.last().ok_or("lost source call root")?;out.owners.push(owner as u64);
        let Some(next)=rows.get(j+1) else {break;};
        let target=r.retired_next_pc.unwrap_or(if r.interrupt_after {next.pc}else{r.next_pc});
        let sp=if r.interrupt_after {next.sp}else{r.next_sp};
        if target!=next.pc || sp!=next.sp {return Err("main opcode adjacency/interrupt continuation".into());}
        if r.interrupt_after {hit(&mut out.rules,"continuous_interrupt_cut_preserves_call");}
        match r.opcode {
            0x20|0x22|0xfc=>{
                let bytes=if r.opcode==0x22 {3}else{2};
                if r.sp.checked_sub(bytes)!=Some(sp) {return Err("call push width mismatch".into());}
                let id=out.frames.len();
                out.frames.push(Call {id:id as u64,parent:Some(owner as u64),entry:target,aliases:[target].into(),
                    return_pc:r.pc+if r.opcode==0x22 {4}else{3},return_sp:r.sp,entry_sp:sp,closed:false});
                stack.push(id);hit(&mut out.rules,"jsr_jsl_real_return_frame");
            }
            0x60|0x6b=>{
                if r.sp.checked_add(if r.opcode==0x6b {3}else{2})!=Some(sp) {return Err("return pull width mismatch".into());}
                if let Some(at)=stack.iter().rposition(|&id|id!=0 && out.frames[id].return_pc==target && out.frames[id].return_sp==sp) {
                    if at+1<stack.len() {hit(&mut out.rules,"pulled_frame_double_return");}
                    for id in stack.drain(at..) {out.frames[id].closed=true;}
                    hit(&mut out.rules,"rts_rtl_verified_return");
                } else if sp<=out.frames[owner].entry_sp && sp>r.sp {
                    // RTS consumes a locally pushed address. It did not consume
                    // the real caller's frame; selected body inherits that call.
                    out.frames[owner].aliases.insert(target);hit(&mut out.rules,"local_rts_dispatch_inherits_frame");
                } else {return Err(format!("unmapped return {:06x}->{target:06x} sp {:x}->{sp:x}",r.pc,r.sp).into());}
            }
            0x4c|0x5c|0x6c|0x7c|0xdc=>{
                if sp!=r.sp {return Err("tail jump changed source stack".into());}
                out.frames[owner].aliases.insert(target);hit(&mut out.rules,"jmp_jml_shared_tail_inherits_frame");
            }
            _=>{}
        }
    }
    if stack.len()!=1 {return Err("unclosed source calls".into());}
    Ok(out)
}

#[derive(Clone,Debug)]
struct Block { call:u64, pc:u32, end:u32, m:bool, x:bool, clocks:u64, ordinals:Vec<usize>, extras:Vec<(u32,u8)>, occurrence:u64 }
fn blocks(rom:&[u8],rows:&[Instruction],calls:&Calls)->Result<Vec<Block>> {
    let mut by_call:BTreeMap<u64,Vec<usize>>=BTreeMap::new();
    for (i,&call) in calls.owners.iter().enumerate() {by_call.entry(call).or_default().push(i);}
    let table:BTreeMap<_,_>=generated::BLOCKS.iter().map(|b|(b.0,b)).collect();
    let mut result=Vec::new();
    for (call,indices) in by_call {
        let mut at=0;let mut occurrences:BTreeMap<u32,u64>=BTreeMap::new();
        while at<indices.len() {
            let first=&rows[indices[at]];
            let Some(b)=table.get(&first.pc) else {at+=1;continue;};
            let mut pc=b.0;let (mut m,mut x)=(b.2,b.3);let mut owned=Vec::new();let mut extras=Vec::new();let mut clocks=0;let mut valid=true;
            while pc<b.1 {
                let Some(&idx)=indices.get(at+owned.len()) else {valid=false;break;};
                let r=&rows[idx];
                let op=byte(rom,pc)?;
                if r.pc!=pc || r.opcode!=op || r.m8!=m || r.x8!=x || r.dp&255!=0 || r.emulation {valid=false;break;}
                let len=length(op,m,x);let end=pc+len;
                if end>b.1 {valid=false;break;}
                let next=r.retired_next_pc.unwrap_or_else(||rows.get(idx+1).map_or(r.next_pc,|r|r.pc));
                if pc==b.5 && next!=end {extras.push((pc,1));}
                if b.6.contains(&pc) {
                    let index=match op&0x1f {0x19=>r.y,0x1d=>r.x,_=>None};
                    let Some(index)=index else {valid=false;break;};
                    let low=byte(rom,pc+1)?;
                    if u16::from(low)+index>255 {extras.push((pc,2));}
                }
                owned.push(idx);clocks+=r.master;
                if op==0xc2 {let bits=byte(rom,pc+1)?;m&=bits&0x20==0;x&=bits&0x10==0;}
                if op==0xe2 {let bits=byte(rom,pc+1)?;m|=bits&0x20!=0;x|=bits&0x10!=0;}
                pc=end;
            }
            if valid && pc==b.1 {
                let occurrence=occurrences.entry(b.0).or_default();
                result.push(Block {call,pc:b.0,end:b.1,m:b.2,x:b.3,clocks,ordinals:owned.clone(),extras,occurrence:*occurrence});
                *occurrence+=1;at+=owned.len();
            } else {at+=1;}
        }
    }
    result.sort_by_key(|b|b.ordinals[0]);Ok(result)
}

fn json_rows(path:&Path)->Result<impl Iterator<Item=Result<Value>>> {
    let input:Box<dyn BufRead>=if path.extension().is_some_and(|e|e=="zst") {
        Box::new(BufReader::new(zstd::Decoder::new(File::open(path)?)?))
    }else{Box::new(BufReader::new(File::open(path)?))};
    Ok(input.lines().map(|l|Ok(serde_json::from_str(&l?)?)))
}
#[derive(Debug,Serialize)]
struct Scope {id:u64,pc:u32,parent:Option<u64>,site:String}
#[derive(Clone,Debug,Serialize)]
struct Charge {#[serde(default)] model:Option<Vec<(Option<u32>,u32,bool)>>,call:Option<u64>,pc:Option<u32>,kind:Option<u8>,width:Option<(bool,bool)>,clocks:u64,site:String,sequence:u64,domain:String}
#[derive(Default)]
struct Native {scopes:Vec<Scope>,charges:Vec<Charge>,total:Option<u64>,local_widths:BTreeMap<u64,u16>}
fn read_native(input:&mut impl Iterator<Item=Result<Value>>)->Result<Option<(u64,Native)>> {
    let Some(begin)=input.next() else {return Ok(None);};let begin=begin?;
    if begin["event"]!="begin" {return Err("native event outside cut".into());}
    let i=begin["iteration"].as_u64().ok_or("native iteration")?;
    if begin["calls"].as_array().is_none_or(|a|!a.is_empty()) {return Err("native scope active across main cut".into());}
    let mut n=Native::default();let mut stack=Vec::new();let mut ids=BTreeSet::new();let mut sequence=1;
    for row in input {
        let r=row?;if r["sequence"].as_u64()!=Some(sequence) {return Err("native order".into());}sequence+=1;
        match r["event"].as_str() {
            Some("enter")=>{
                let id=r["call"].as_u64().ok_or("native scope id")?;let pc=r["pc"].as_u64().ok_or("native scope pc")? as u32;
                if !ids.insert(id) {return Err("duplicate native scope id".into());}
                if let Some(width)=r["local_width"].as_u64() {n.local_widths.insert(id,u16::try_from(width)?);}
                n.scopes.push(Scope {id,pc,parent:stack.last().map(|&(id,_)|id),site:r["site"].as_str().ok_or("native scope site")?.into()});
                stack.push((id,pc));
            }
            Some("exit")=>{
                let id=r["call"][0].as_u64().ok_or("native exit id")?;let pc=r["call"][1].as_u64().ok_or("native exit pc")? as u32;
                if stack.pop()!=Some((id,pc)) {return Err("native scope nesting".into());}
            }
            Some("charge")=>{
                if r["call"][0].as_u64()!=stack.last().map(|&(id,_)|id) {return Err("native charge scope ownership".into());}
                let path:Vec<_>=r["path"].as_array().ok_or("native charge path")?.iter()
                    .map(|v|Ok((v[0].as_u64().ok_or("path id")?,v[1].as_u64().ok_or("path pc")? as u32))).collect::<Result<_>>()?;
                if path!=stack {return Err("native charge path mismatch".into());}
                n.charges.push(Charge {model:serde_json::from_value(r["model_instructions"].clone())?,call:stack.last().map(|&(id,_)|id),pc:r["pc"].as_u64().map(|v|v as u32),
                    kind:r["kind"].as_u64().map(|v|v as u8),width:serde_json::from_value(r["declared_width"].clone())?,
                    clocks:r["clocks"].as_u64().ok_or("native charge clocks")?,site:r["site"].as_str().ok_or("native charge site")?.into(),
                    sequence:sequence-1,domain:r["domain"].as_str().ok_or("native charge domain")?.into()});
            }
            Some("end")=>{
                if !stack.is_empty() {return Err("unclosed native scopes".into());}
                n.total=Some(r["ledger_master"].as_u64().ok_or("native total")?);return Ok(Some((i,n)));
            }
            _=>return Err("unknown/nested native event".into()),
        }
        if n.scopes.len()+n.charges.len()>2_000_000 {return Err("bounded single-iteration native event budget".into());}
    }
    Err("truncated native stream".into())
}

#[derive(Debug)]
struct NativeBlock {call:u64, pc:u32, m:bool,x:bool, clocks:u64,site:String,extras:Vec<(u32,u8)>, extra_sites:Vec<Value>, sequence:u64}
/// Exact entries pair by occurrence only when cardinalities agree. Native
/// fragments at the root and interior priced-child aliases may share a call,
/// but cannot manufacture a new ROM call or a second opcode owner.
fn map_scopes(n:&Native,c:&Calls,rows:&[Instruction])->Result<(BTreeMap<u64,u64>,BTreeMap<&'static str,u64>)> {
    let mut source:BTreeMap<u32,Vec<u64>>=BTreeMap::new();
    let mut visited=BTreeSet::new();
    // A late tail of an older frame can execute after a younger child has
    // visited the same PC. Order by executed alias entry, not frame creation.
    for (r,&call) in rows.iter().zip(&c.owners) {
        if c.frames[call as usize].aliases.contains(&r.pc) && visited.insert((r.pc,call)) {
            source.entry(r.pc).or_default().push(call);
        }
    }
    let mut entries:BTreeMap<u32,Vec<u64>>=BTreeMap::new();let mut parents=BTreeMap::new();
    for s in &n.scopes {parents.insert(s.id,s.parent);entries.entry(s.pc).or_default().push(s.id);}
    let mut mapped=BTreeMap::new();let mut rules=BTreeMap::new();
    let mut source_visits:BTreeMap<(u32,u64),usize>=BTreeMap::new();
    for (r,&call) in rows.iter().zip(&c.owners) {*source_visits.entry((r.pc,call)).or_default()+=1;}
    for (pc,ids) in &entries {
        if *pc==0x8034 {for id in ids {mapped.insert(*id,0);}hit(&mut rules,"native_root_fragments");continue;}
        if let Some(src)=source.get(pc).filter(|s|s.len()==ids.len()) {
            for (&id,&call) in ids.iter().zip(src) {mapped.insert(id,call);hit(&mut rules,"entry_alias_occurrence_bijection");}
        }
    }
    // Interior scopes can partition their proved parent's exclusive opcodes.
    // Require one such entry occurrence per source PC in that very call.
    let mut peers:BTreeMap<(u32,u64),usize>=BTreeMap::new();
    for scope in &n.scopes {
        if let Some(parent)=scope.parent.and_then(|p|mapped.get(&p).copied()) {*peers.entry((scope.pc,parent)).or_default()+=1;}
    }
    for (pc,ids) in &entries {
        for id in ids {
            if mapped.contains_key(id){continue;}
            let Some(parent)=parents[id].and_then(|p|mapped.get(&p).copied()) else {continue;};
            let count=source_visits.get(&(*pc,parent)).copied().unwrap_or(0);
            if peers.get(&(*pc,parent))==Some(&count) && count>0 {mapped.insert(*id,parent);hit(&mut rules,"priced_child_interior_partition");}
        }
    }
    // A native descendant must be a source descendant (or a partition of the
    // same frame). Reject aliases whose apparent PC agreement crosses calls.
    let mut invalid=Vec::new();
    for (&id,&call) in &mapped {
        if let Some(parent)=parents[&id].and_then(|p|mapped.get(&p).copied()) {
            let mut ancestor=Some(call);
            while ancestor.is_some_and(|a|a!=parent) {ancestor=c.frames[ancestor.unwrap() as usize].parent;}
            if ancestor.is_none(){invalid.push(id);}
            else if call==parent {hit(&mut rules,"same_source_frame_native_partition");}
            else {hit(&mut rules,"native_path_projects_source_ancestors");}
        }
    }
    for id in invalid {mapped.remove(&id);hit(&mut rules,"rejected_cross_call_alias");}
    Ok((mapped,rules))
}
fn native_blocks<'a>(n:&'a Native,mapping:&BTreeMap<u64,u64>)->(Vec<NativeBlock>,Vec<&'a Charge>) {
    let mut blocks:Vec<NativeBlock>=Vec::new();let mut unmatched=Vec::new();
    for e in &n.charges {
        if e.model.is_some() { unmatched.push(e); continue; }
        let call=e.call.and_then(|id|mapping.get(&id).copied());
        let pc=e.pc;let kind=e.kind;
        if let (Some(call),Some(pc),Some(0))=(call,pc,kind) {
            if let Some((m,x))=e.width {
                blocks.push(NativeBlock {call,pc,m,x,clocks:e.clocks,site:e.site.clone(),
                    extras:Vec::new(),extra_sites:Vec::new(),sequence:e.sequence});continue;
            }
        }
        if let (Some(call),Some(pc),Some(kind @ (1|2)))=(call,pc,kind) {
            // Extras belong only to the latest block in this logical call,
            // and the generated instruction membership is checked at pairing.
            if let Some(b)=blocks.iter_mut().rev().find(|b|b.call==call) {
                b.extras.push((pc,kind));b.extra_sites.push(json!(e));b.clocks+=e.clocks;continue;
            }
        }
        unmatched.push(e);
    }
    (blocks,unmatched)
}

/// Match complete contiguous model instruction streams, never a PC multiset.
/// Real source call reconstruction is already verified. A unique stream must
/// stay under the mapped native ancestor, with exact per-PC widths and costs.
/// Aggregated own/child booking is intentionally withheld: its charge order
/// differs from source retirement and bank-wrap work may be booked on the last
/// child. Its witnesses remain available, but cannot become an apparent pair.
fn model_spans(n:&Native, rows:&[Instruction], c:&Calls, mapping:&BTreeMap<u64,u64>,
    owned:&mut BTreeSet<usize>, anchors:&[(u64,usize,usize)]) -> Vec<(usize,usize,Vec<usize>)> {
    let parents:BTreeMap<_,_>=n.scopes.iter().map(|s|(s.id,s.parent)).collect();
    let widths:BTreeMap<_,_>=graphics_widths::WIDTHS.iter().map(|&(p,m,x)|(p,(m,x))).collect();
    let mut positions:BTreeMap<u32,Vec<usize>>=BTreeMap::new();
    for (j,r) in rows.iter().enumerate() {positions.entry(r.pc).or_default().push(j);}
    let returns:BTreeSet<_>=rows.iter().filter(|r|matches!(r.opcode,0x60|0x6b)
        && !(0xe843..0xe852).contains(&r.pc)).map(|r|r.pc).collect();
    let mut result=Vec::new();let mut at=0;let mut cursor=0;
    while at<n.charges.len() {
        let start=at;
        while at<n.charges.len() && n.charges[at].model.as_ref().is_some_and(|s|s.len()==1) {
            let pc=n.charges[at].model.as_ref().unwrap()[0].0;at+=1;
            // A completed non-byte-reader call is a retirement boundary.
            // The next modeled call need not include its unpriced caller.
            // Pair each entire bounded stream uniquely; never bridge the gap.
            if pc.is_some_and(|p|returns.contains(&p)) {break;}
        }
        if at==start {at+=1;continue;}
        let steps=&n.charges[start..at];
        let Some(first)=steps[0].model.as_ref().unwrap()[0].0 else {continue;};
        let mut ancestor=steps[0].call;
        let mut source_ancestor=None;
        while let Some(id)=ancestor {
            if let Some(&call)=mapping.get(&id) {source_ancestor=Some(call);break;}
            ancestor=parents.get(&id).copied().flatten();
        }
        let Some(ancestor)=source_ancestor else {continue;};
        let candidates:Vec<_>=positions.get(&first).into_iter().flatten().copied().filter(|&j| {
            if j<cursor || j+steps.len()>rows.len() {return false;}
            if anchors.iter().any(|&(sequence,lo,hi)|
                (sequence<steps[0].sequence && hi>=j) ||
                (sequence>steps.last().unwrap().sequence && lo<j+steps.len())) {return false;}
            steps.iter().zip(&rows[j..]).enumerate().all(|(offset,(e,r))| {
                let (pc,clocks,child)=e.model.as_ref().unwrap()[0];
                let mut parent=Some(c.owners[j+offset]);
                while parent.is_some_and(|p|p!=ancestor) {parent=c.frames[parent.unwrap() as usize].parent;}
                pc==Some(r.pc) && u64::from(clocks)==e.clocks && e.clocks==r.master
                    && widths.get(&r.pc)==Some(&(r.m8,r.x8)) && r.dp&255==0 && !r.emulation
                    && child==(r.pc>=0xe843 && r.pc<0xe852)
                    && parent==Some(ancestor) && !owned.contains(&(j+offset))
            })
        }).collect();
        // Ambiguous identical spans remain unmatched; no guessed call pairing.
        if candidates.len()!=1 {continue;}
        let j=candidates[0];let ordinals:Vec<_>=(j..j+steps.len()).collect();
        owned.extend(&ordinals);cursor=j+steps.len();result.push((start,at,ordinals));
    }
    result
}

#[derive(Deserialize)]
struct SourceTotal {iteration:Option<u64>,source_master:u64,source_instructions:u64}
#[derive(Clone,Copy)]
struct Prediction {main:u64,vwf:u64,missing:u64,shortfall:u64}
fn predictions(path:&Path)->Result<BTreeMap<u64,Prediction>> {
    let mut lines=BufReader::new(File::open(path)?).lines();
    let h=lines.next().ok_or("predictor header")??;let h:Vec<_>=h.split(',').collect();
    let col=|name|h.iter().position(|s|*s==name).ok_or("predictor column");
    let (i,m,v,l,n)=(col("iteration")?,col("main_master")?,col("vwf_master")?,col("lag_nmis")?,col("native_held")?);
    let mut result=BTreeMap::new();
    for (expected,line) in lines.enumerate() {
        let line=line?;let c:Vec<_>=line.split(',').collect();if c.len()!=h.len(){return Err("predictor row width".into());}
        let id:u64=c[i].parse()?;if id!=expected as u64 {return Err("predictor sequence".into());}
        result.insert(id,Prediction {main:c[m].parse()?,vwf:c[v].parse()?,missing:c[l].parse::<u64>()?.saturating_sub(c[n].parse()?),shortfall:c[col("cost_shortfall")?].parse()?});
    }Ok(result)
}

struct Outputs {
    blocks:zstd::Encoder<'static,File>, iterations:zstd::Encoder<'static,File>, residuals:zstd::Encoder<'static,File>, calls:zstd::Encoder<'static,File>,
    gaps:Vec<Value>, overcharges:Vec<Value>, summaries:Vec<Value>, rules:BTreeMap<&'static str,u64>,
}
fn sum(rows:&[Instruction])->u64 {rows.iter().map(|r|r.master).sum()}
fn process(rom:&[u8],rows:&[Instruction],n:&Native,p:Prediction,total:&SourceTotal,out:&mut Outputs)->Result<()> {
    let i=rows[0].iteration;let source=sum(rows);
    if source!=total.source_master || rows.len() as u64!=total.source_instructions {return Err("independent source counter mismatch".into());}
    let charged:u64=n.charges.iter().map(|e|e.clocks).sum();
    if n.total!=Some(charged) || charged.checked_add(p.vwf)!=Some(p.main) {return Err("independent native counter mismatch".into());}
    let mut source_events=Vec::new();let mut native_events=Vec::new();let mut owned=BTreeSet::new();let mut mapped_native=0;
    let mut model_paired=0u64;let mut model_sequences=BTreeSet::new();let mut model_ordinals=BTreeSet::new();
    let mut gap_clocks=0;let mut gap_ordinals=BTreeSet::new();
    let mut source_blocks=Vec::new();let mut native_sites=BTreeMap::new();let mut reason=None;
    match calls(rows) {
        Err(e)=>reason=Some(e.to_string()),
        Ok(c)=>{
            for (rule,count) in &c.rules {*out.rules.entry(rule).or_default()+=count;}
            let (mapping,rules)=map_scopes(n,&c,rows)?;
            for (rule,count) in rules {*out.rules.entry(rule).or_default()+=count;}
            for frame in &c.frames {writeln!(out.calls,"{}",json!({"iteration":i,"source_frame":frame}))?;}
            let mut sites_by_call:BTreeMap<u64,Vec<Value>>=BTreeMap::new();
            for scope in &n.scopes {
                let call=mapping.get(&scope.id);
                writeln!(out.calls,"{}",json!({"iteration":i,"native_scope":scope,"source_call":call}))?;
                if let Some(call)=call {sites_by_call.entry(*call).or_default().push(json!({"scope":scope.id,"entry_pc":scope.pc,"site":scope.site}));}
            }
            let (nb,unmatched)=native_blocks(n,&mapping);
            source_blocks=blocks(rom,rows,&c)?;
            let mut sc:BTreeMap<(u64,u32),usize>=BTreeMap::new();let mut nc:BTreeMap<(u64,u32),usize>=BTreeMap::new();let mut sites:BTreeMap<(u64,u32),BTreeSet<String>>=BTreeMap::new();
            for b in &source_blocks {*sc.entry((b.call,b.pc)).or_default()+=1;}
            for b in &nb {*nc.entry((b.call,b.pc)).or_default()+=1;sites.entry((b.call,b.pc)).or_default().insert(b.site.clone());}
            let valid:BTreeSet<_>=sc.iter().filter(|(k,count)|nc.get(k)==Some(*count) && sites.get(k).is_some_and(|s|s.len()==1)).map(|(k,_)|*k).collect();
            let mut counts:BTreeMap<(u64,u32),u64>=BTreeMap::new();let mut natives=BTreeMap::new();
            for b in nb {
                let count=counts.entry((b.call,b.pc)).or_default();let occurrence=*count;*count+=1;
                natives.insert((b.call,b.pc,occurrence),b);
            }
            // An instruction is eligible only if its complete decoded block,
            // call occurrence, width and taken/page extras agree independently.
            let eligible:BTreeSet<_>=source_blocks.iter().filter(|b|valid.contains(&(b.call,b.pc)) &&
                natives.get(&(b.call,b.pc,b.occurrence)).is_some_and(|n|n.m==b.m && n.x==b.x && n.extras==b.extras))
                .map(|b|(b.call,b.pc,b.occurrence)).collect();
            for b in &source_blocks {
                let k=(b.call,b.pc,b.occurrence);
                if !eligible.contains(&k) {continue;}
                let native=&natives[&k];
                source_events.push(Event {key:Key {call:b.call,pc:b.pc,kind:0,occurrence:b.occurrence,m8:b.m,x8:b.x},
                    clocks:b.clocks,site:native.site.clone(),domain:"instruction".into()});
                for ordinal in &b.ordinals {if !owned.insert(*ordinal) {return Err("overlapping generated source blocks".into());}}
                native_sites.insert(k,json!({"end":b.end,"source_ordinals":b.ordinals,"taken_page_extras":b.extras,
                    "call_path":call_path(&c,b.call),"native_sequence":native.sequence,"native_extra_sites":native.extra_sites}));
            }
            let mut ordered_native:Vec<_>=natives.iter().collect();ordered_native.sort_by_key(|(_,b)|b.sequence);
            for (k,b) in ordered_native {
                if !eligible.contains(k) {
                    writeln!(out.residuals,"{}",json!({"iteration":i,"reason":"native_block_width_extras_or_occurrence_unmapped",
                        "domain":"marker_instruction","pc":b.pc,"native_site":b.site,"source_call":b.call,
                        "occurrence":k.2,"master_clocks":b.clocks,"taken_page_extras":b.extras}))?;
                    continue;
                }
                mapped_native+=b.clocks;
                native_events.push(Event {key:Key {call:b.call,pc:b.pc,kind:0,occurrence:k.2,m8:b.m,x8:b.x},
                    clocks:b.clocks,site:b.site.clone(),domain:"instruction".into()});
            }
            let anchors:Vec<_>=native_sites.values().map(|v| {
                let ord=v["source_ordinals"].as_array().unwrap();
                (v["native_sequence"].as_u64().unwrap(),ord[0].as_u64().unwrap() as usize,
                    ord.last().unwrap().as_u64().unwrap() as usize)
            }).collect();
            for (start,end,ordinals) in model_spans(n,rows,&c,&mapping,&mut owned,&anchors) {
                let charges=&n.charges[start..end];
                let clocks:u64=charges.iter().map(|e|e.clocks).sum();
                model_paired+=clocks;model_ordinals.extend(&ordinals);
                model_sequences.extend(charges.iter().map(|e|e.sequence));
                writeln!(out.blocks,"{}",json!({"iteration":i,"model_pair":{
                    "rule":"unique_contiguous_graphics_pc_width_child_stream",
                    "source_ordinals":ordinals,"native_sequences":charges.iter().map(|e|e.sequence).collect::<Vec<_>>(),
                    "source_master":clocks,"native_master":clocks,"signed_error":0}}))?;
            }
            for e in &unmatched {
                if model_sequences.contains(&e.sequence) {continue;}
                let reason=match &e.model {
                    Some(steps) if steps.len()!=1=>"aggregate_graphics_booking_requires_child_order",
                    Some(steps) if steps[0].0.is_none()=>"graphics_pc_unavailable",
                    Some(_)=>"graphics_stream_path_width_call_or_occurrence_unmapped",
                    None=>"native_scalar_or_call_unmapped",
                };
                let witness_master=e.model.as_ref().map(|steps|steps.iter().map(|s|u64::from(s.1)).sum::<u64>());
                writeln!(out.residuals,"{}",json!({"iteration":i,"domain":e.domain,"reason":reason,
                    "native_site":e.site,"native_call":e.call,"master_clocks":e.clocks,"model_instructions":e.model,
                    "witness_master":witness_master,"witness_charge_equal":witness_master.map(|v|v==e.clocks)}))?;
            }
            // Local exclusion uses actual equal ordered pairs, never merely
            // eligible markers (which may subsequently fail the ordered join).
            let (_,exact_pairs)=ordered::pair(&source_events,&native_events)?;
            let mut paired_source=model_ordinals.clone();let mut paired_native=model_sequences.clone();
            for pair in exact_pairs.iter().filter(|r|r.status=="paired" && r.delta==0) {
                let meta=&native_sites[&(pair.key.call,pair.key.pc,pair.key.occurrence)];
                paired_source.extend(meta["source_ordinals"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap() as usize));
                paired_native.insert(meta["native_sequence"].as_u64().unwrap());
                for extra in meta["native_extra_sites"].as_array().unwrap() {paired_native.insert(extra["sequence"].as_u64().unwrap());}
            }
            for scope in &n.scopes {
                let Some(ordinals)=local_exclusion::helper_gap(&c,rows,n,&mapping,scope,&paired_source,&paired_native) else {continue;};
                if ordinals.is_empty() {continue;}
                for &j in &ordinals {if !gap_ordinals.insert(j) {return Err("overlapping local exclusion certificates".into());}}
                let clocks:u64=ordinals.iter().map(|&j|rows[j].master).sum();gap_clocks+=clocks;
                let call=mapping[&scope.id];
                out.gaps.push(json!({"iteration":i,"source_call":call,"call_path":call_path(&c,call),
                    "native_scope":scope.id,"native_site":scope.site,"entry":scope.pc,
                    "rule":"table_helper_synchronous_closed_leaf_v1","master_clocks":clocks,
                    "holds_touched":p.missing,"frequency":1,"source_ordinals":ordinals,
                    "instructions":ordinals.iter().map(|&j|json!({"pc":rows[j].pc,"ordinal":j,"master_clocks":rows[j].master,
                        "m8":rows[j].m8,"x8":rows[j].x8})).collect::<Vec<_>>(),
                    "required_fixture":"193F7/193FF M16/X16 width loop and inherited caller tail; once-only JSR ownership",
                    "pricing_status":"validated_exclusive_gap_not_applied"}));
            }
            // Keep every source opcode not owned by an admitted block. A native
            // Local certificates label a disjoint subset; gaps remain in U
            // until a separate pricing batch actually charges their opcodes.
            for (ordinal,r) in rows.iter().enumerate().filter(|(j,_)|!owned.contains(j)) {
                let call=c.owners[ordinal];
                let sites=sites_by_call.get(&call);
                writeln!(out.residuals,"{}",json!({"iteration":i,"domain":"main_instruction","reason":"source_call_or_block_unmapped",
                    "pc":r.pc,"end":r.pc+length(r.opcode,r.m8,r.x8),"ordinal":ordinal,"master_clocks":r.master,"call_path":call_path(&c,call),
                    "native_site_candidates":sites,"affected_missing_holds":p.missing,"missing_holds_covered":0,
                    "m8":r.m8,"x8":r.x8,"dp":r.dp,"db":r.db,"scalar_exclusion_proved":gap_ordinals.contains(&ordinal)}))?;
            }
        }
    }
    let (paired,paired_rows)=ordered::pair(&source_events,&native_events)?;
    let c=Conservation {source,native:p.main,paired_source:paired.paired_source+model_paired,paired_native:paired.paired_native+model_paired,
        unmatched_source:source-paired.paired_source-model_paired,unmatched_native:p.main-paired.paired_native-model_paired,
        signed_error:paired.signed_error,unequal_pairs:paired.unequal_pairs};
    let exact=c.check()?;
    for r in paired_rows {
        let metadata=native_sites.get(&(r.key.call,r.key.pc,r.key.occurrence));
        let row=json!({"iteration":i,"pair":r,"bridge":metadata,"missing_holds":p.missing});
        if r.status=="paired" && r.delta<0 {out.overcharges.push(row.clone());}
        writeln!(out.blocks,"{row}")?;
    }
    if let Some(reason)=&reason {writeln!(out.residuals,"{}",json!({"iteration":i,"reason":"source_call_frame_unmapped","detail":reason,
        "master_clocks":source,"native_master":p.main}))?;}
    let summary=json!({"iteration":i,"conservation":c,"missing_holds":p.missing,"source_instructions":rows.len(),
        "model_paired_master":model_paired,"decoded_source_blocks":source_blocks.len(),"mapped_native_block_clocks":mapped_native,
        "alignment":"continuous_source_ordered_marker_subset","exact":exact,"call_frame_failure":reason,
        "source_independent_equal":true,"native_independent_equal":true,"continuous_interrupt_cuts":rows.iter().filter(|r|r.interrupt_after).count(),
        "validated_exclusive_gap_clocks":gap_clocks,"missing_holds_covered":0,
        "combined_gap_inverse":{"added_main_clocks":gap_clocks,"full_count_shortfall":p.shortfall,
            "reaches_full_count_inverse":p.shortfall>0 && gap_clocks>=p.shortfall,"fixed_context_only":true},
        "unpaired_source_clocks":source-source_events.iter().map(|e|e.clocks).sum::<u64>()-model_paired,
        "unpaired_native_clocks":p.main-native_events.iter().map(|e|e.clocks).sum::<u64>()-model_paired});
    writeln!(out.iterations,"{summary}")?;out.summaries.push(summary);Ok(())
}
fn call_path(c:&Calls,mut id:u64)->Vec<u32> {
    let mut path=Vec::new();loop {let f=&c.frames[id as usize];path.push(f.entry);match f.parent {Some(p)=>id=p,None=>break,}}
    path.reverse();path
}

/// ROM SOURCE_OUT PREDICTOR EVENTS FRESH_OUT. Streams one source iteration;
/// never writes ROM bytes or interpreter checkpoints to the report.
pub fn run(rom:&Path,source:&Path,predictor:&Path,events:&Path,out:&Path)->Result<()> {
    local_exclusion::verify_certificate_sources()?;
    let source_manifest=if source.join("manifest-frozen.json").exists() {source.join("manifest-frozen.json")}else{source.join("manifest.json")};
    let manifest:Manifest=serde_json::from_reader(File::open(&source_manifest)?)?;manifest.verify()?;
    let coverage:Value=serde_json::from_reader(File::open(source.join("coverage.json"))?)?;
    let checked=coverage["checked_source_rows"].as_u64().ok_or("verified source prefix absent")?;
    let mut bound=vec![Identity::bind(rom)?,Identity::bind(&source.join("source-instructions.jsonl.zst"))?,
        Identity::bind(&source.join("iterations.jsonl.zst"))?,Identity::bind(&source.join("entry-kinds.jsonl.zst"))?,
        Identity::bind(&source.join("coverage.json"))?,Identity::bind(predictor)?,Identity::bind(events)?,
        Identity::bind(Path::new("crates/rom-shadow-audit/src/bridge.rs"))?,Identity::bind(Path::new("crates/rom-shadow-audit/src/ordered.rs"))?,
        Identity::bind(Path::new("crates/rom-shadow-audit/src/graphics_widths.rs"))?,
        Identity::bind(Path::new("crates/rom-shadow-audit/src/local_exclusion.rs"))?,
        Identity::bind(Path::new("crates/zelda3/src/rom_block_costs.rs"))?,Identity::bind(&std::env::current_exe()?)?,Identity::bind(&source_manifest)?];
    if bound[0].sha256!="66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb" {return Err("generated blocks ROM mismatch".into());}
    let rom=fs::read(rom)?;let prediction=predictions(predictor)?;let mut native=json_rows(events)?;
    let mut site_files=BTreeSet::new();
    let mut totals=BTreeMap::new();
    for row in json_rows(&source.join("iterations.jsonl.zst"))? {
        let t:SourceTotal=serde_json::from_value(row?)?;if let Some(i)=t.iteration {if totals.insert(i,t).is_some(){return Err("duplicate source total".into());}}
    }
    let mut ordinary=BTreeMap::new();
    for row in json_rows(&source.join("entry-kinds.jsonl.zst"))? {
        let r=row?;let i=r["iteration"].as_u64().ok_or("entry iteration")?;
        if i<checked && r["pc"]==0x8038 && matches!(r["kind"].as_str(),Some("ordinary_admitted"|"ordinary_unadmitted")) && prediction.get(&i).is_some_and(|p|p.missing>0) {
            if ordinary.insert(i,r).is_some(){return Err("duplicate ordinary entry".into());}
        }
    }
    fs::create_dir(out)?;
    let mut output=Outputs {blocks:zstd::Encoder::new(File::create(out.join("blocks.jsonl.zst"))?,3)?,
        iterations:zstd::Encoder::new(File::create(out.join("iterations.jsonl.zst"))?,3)?,residuals:zstd::Encoder::new(File::create(out.join("residuals.jsonl.zst"))?,3)?,
        calls:zstd::Encoder::new(File::create(out.join("calls.jsonl.zst"))?,3)?,
        gaps:Vec::new(),overcharges:Vec::new(),summaries:Vec::new(),rules:BTreeMap::new()};
    let mut pending=Vec::new();let mut seen=BTreeSet::new();
    let mut flush=|rows:&mut Vec<Instruction>,output:&mut Outputs,seen:&mut BTreeSet<u64>|->Result<()> {
        if rows.is_empty(){return Ok(());}
        let i=rows[0].iteration;
        if !seen.insert(i){return Err("duplicate source iteration group".into());}
        let (id,n)=read_native(&mut native)?.ok_or("missing native observation")?;
        if id!=i {return Err("native/source selected iteration order mismatch".into());}
        for site in n.scopes.iter().map(|s|&s.site).chain(n.charges.iter().map(|c|&c.site)) {
            let file=site.rsplitn(3,':').nth(2).ok_or("native site location")?;
            if !file.starts_with("crates/") || Path::new(file).components().any(|c|matches!(c,std::path::Component::ParentDir)) {return Err("native site outside source tree".into());}
            if site_files.insert(file.to_string()) {bound.push(Identity::bind(Path::new(file))?);}
        }
        process(&rom,rows,&n,prediction[&i],totals.get(&i).ok_or("source independent total absent")?,output)?;rows.clear();Ok(())
    };
    for row in json_rows(&source.join("source-instructions.jsonl.zst"))? {
        let row:Instruction=serde_json::from_value(row?)?;
        if !ordinary.contains_key(&row.iteration){continue;}
        if pending.first().is_some_and(|r:&Instruction|r.iteration!=row.iteration) {flush(&mut pending,&mut output,&mut seen)?;}
        if pending.len()>=1_000_000 {return Err("bounded bridge iteration instruction budget".into());}
        pending.push(row);
    }
    flush(&mut pending,&mut output,&mut seen)?;
    drop(flush);
    if read_native(&mut native)?.is_some() {return Err("native observation beyond selected source EOF".into());}
    if seen!=ordinary.keys().copied().collect(){return Err("missing source ordinary instruction capture".into());}
    output.blocks.finish()?;output.iterations.finish()?;output.residuals.finish()?;output.calls.finish()?;
    for id in &bound {id.verify()?;}manifest.verify()?;
    let write=|name:&str,value:&Value|->Result<()> {let mut f=File::options().create_new(true).write(true).open(out.join(name))?;serde_json::to_writer_pretty(&mut f,value)?;writeln!(f)?;Ok(())};
    write("coverage.json",&json!({"identities":bound,"source_checked":checked,"entry_variants":ordinary,
        "ordinary_missing_iterations":seen.len(),"summaries":output.summaries,"bridge_rule_counts":output.rules,
        "pricing_verdict":if output.gaps.is_empty(){"STOP"}else{"GO_LOCAL_ONLY"},"reason":"only certified local gaps may be priced; remaining scalar and call paths stay unresolved"}))?;
    let mut ranked=output.gaps.clone();ranked.sort_by_key(|r|std::cmp::Reverse(r["master_clocks"].as_u64().unwrap_or(0)));
    let mut touched=BTreeMap::new();
    for gap in &output.gaps {touched.insert(gap["iteration"].as_u64().unwrap(),gap["holds_touched"].as_u64().unwrap());}
    write("worklist.json",&json!({"validated_positive_gaps":output.gaps,"negative_price_repairs":output.overcharges,
        "top_gaps_by_clocks":ranked,"top_gaps_by_missing_holds_covered":[],"missing_holds_covered":0,
        "holds_touched":touched.values().sum::<u64>(),"affected_iterations":touched.keys().collect::<Vec<_>>(),
        "validated_gap_clocks":output.gaps.iter().map(|g|g["master_clocks"].as_u64().unwrap()).sum::<u64>(),
        "admission":if output.gaps.is_empty(){"STOP"}else{"GO_LOCAL_ONLY"},
        "combined_per_iteration_inverse":output.summaries.iter().filter(|s|s["validated_exclusive_gap_clocks"].as_u64().unwrap_or(0)>0)
            .map(|s|json!({"iteration":s["iteration"],"missing_holds":s["missing_holds"],"inverse":s["combined_gap_inverse"]})).collect::<Vec<_>>(),
        "reason":"only local boundary certificates admit gaps; holds touched are not recovery"}))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn r(pc:u32,op:u8,sp:u16,next:u32,next_sp:u16)->Instruction {
        Instruction {iteration:0,ordinal:0,pc,opcode:op,master:10,m8:true,x8:true,dp:0,db:0,sp,next_pc:next,next_sp,
            interrupt_after:false,retired_next_pc:Some(next),x:Some(0),y:Some(0),emulation:false}
    }
    fn trace(mut rows:Vec<Instruction>)->Vec<Instruction> {for (i,r) in rows.iter_mut().enumerate(){r.ordinal=i as u64;}rows}
    #[test] fn jsr_jsl_jml_and_interrupt_keep_one_call_identity() {
        let mut rows=trace(vec![r(0x8038,0x22,511,0x18000,508),r(0x18000,0x20,508,0x18100,506),
            r(0x18100,0x5c,506,0x28000,506),r(0x28000,0x60,506,0x18003,508),r(0x18003,0x6b,508,0x803c,511),
            r(0x803c,0xea,511,0x805d,511),r(0x805d,0xea,511,0x805e,511)]);
        rows[2].interrupt_after=true;rows[2].next_pc=0x80c9;rows[2].next_sp=502;
        let c=calls(&rows).unwrap();assert_eq!(c.owners,vec![0,1,2,2,1,0,0]);
        assert!(c.frames[2].aliases.contains(&0x28000));assert_eq!(c.rules["continuous_interrupt_cut_preserves_call"],1);
        rows[2].retired_next_pc=Some(0x28001);assert!(calls(&rows).is_err());
    }
    #[test] fn local_dispatch_and_pulled_double_return_are_distinct() {
        let rows=trace(vec![r(0x8038,0x20,511,0x9000,509),r(0x9000,0xf4,509,0x9003,507),
            r(0x9003,0x60,507,0x9100,509),r(0x9100,0x20,509,0x9200,507),r(0x9200,0x68,507,0x9201,509),
            r(0x9201,0x60,509,0x803b,511),r(0x803b,0xea,511,0x805d,511),r(0x805d,0xea,511,0x805e,511)]);
        let c=calls(&rows).unwrap();assert_eq!(c.owners,vec![0,1,1,1,2,2,0,0]);
        assert_eq!(c.rules["local_rts_dispatch_inherits_frame"],1);assert_eq!(c.rules["pulled_frame_double_return"],1);
        let mut bad=rows;bad[5].next_pc=0x8040;bad[5].retired_next_pc=Some(0x8040);assert!(calls(&bad).is_err());
    }
    #[test] fn decode_width_changes_and_rom_mirrors() {
        assert_eq!((length(0xa9,true,true),length(0xa9,false,true),length(0xa2,true,false)),(2,3,3));
        let rom=vec![0xea;0x10000];assert_eq!(byte(&rom,0x808000).unwrap(),byte(&rom,0x8000).unwrap());
        assert!(byte(&rom,0x7e8000).is_err());
    }
    #[test] fn generated_expansion_rejects_wrong_width_or_missing_opcode() {
        let mut rom=vec![0;0x10000];rom[0x781..0x784].copy_from_slice(&[0x8b,0x4b,0xab]);
        let mut rows=trace(vec![r(0x8781,0x8b,511,0x8782,510),r(0x8782,0x4b,510,0x8783,509),r(0x8783,0xab,509,0x8784,510)]);
        let c=Calls {frames:Vec::new(),owners:vec![0;3],rules:BTreeMap::new()};
        assert_eq!(blocks(&rom,&rows,&c).unwrap()[0].ordinals,vec![0,1,2]);
        rows[1].m8=false;assert!(blocks(&rom,&rows,&c).unwrap().is_empty());
        rows[1].m8=true;rows[1].pc+=1;assert!(blocks(&rom,&rows,&c).unwrap().is_empty());
    }
    fn scope(event:&str,id:u64,pc:u32)->Value {
        if event=="enter" {json!({"event":event,"call":id,"pc":pc})}
        else {json!({"event":event,"call":[id,pc]})}
    }
    fn fixture_native(events:Vec<Value>)->Native {
        let mut n=Native::default();let mut stack=Vec::new();
        for e in events {
            match e["event"].as_str().unwrap() {
                "enter"=>{let id=e["call"].as_u64().unwrap();n.scopes.push(Scope {id,pc:e["pc"].as_u64().unwrap() as u32,parent:stack.last().copied(),site:"fixture.rs:1:1".into()});stack.push(id);}
                "exit"=>{stack.pop();}
                _=>n.charges.push(Charge {call:e["call"][0].as_u64(),pc:e["pc"].as_u64().map(|v|v as u32),kind:e["kind"].as_u64().map(|v|v as u8),
                    model:None,width:serde_json::from_value(e["declared_width"].clone()).unwrap(),clocks:e["clocks"].as_u64().unwrap(),site:"fixture.rs:1:1".into(),sequence:e["sequence"].as_u64().unwrap(),domain:"marker_instruction".into()}),
            }
        }n
    }
    #[test] fn compact_stream_preserves_zero_charges_and_rejects_bad_paths_or_truncation() {
        let rows=vec![json!({"event":"begin","iteration":0,"calls":[]}),
            json!({"event":"enter","sequence":1,"call":5,"pc":32768,"site":"fixture.rs:1:1"}),
            json!({"event":"charge","sequence":2,"call":[5,32768],"path":[[5,32768]],"clocks":0,"pc":null,"kind":null,
                "declared_width":null,"site":"fixture.rs:2:1","domain":"scalar_unmapped"}),
            json!({"event":"exit","sequence":3,"call":[5,32768]}),json!({"event":"end","sequence":4,"ledger_master":0})];
        let mut stream=rows.clone().into_iter().map(Ok);
        let (i,n)=read_native(&mut stream).unwrap().unwrap();assert_eq!(i,0);assert_eq!(n.scopes.len(),1);assert_eq!(n.charges.len(),1);
        assert_eq!(n.charges[0].clocks,0);assert!(read_native(&mut stream).unwrap().is_none());
        let mut bad=rows.clone();bad[2]["path"]=json!([[6,32768]]);assert!(read_native(&mut bad.into_iter().map(Ok)).is_err());
        let mut bad=rows.clone();bad[3]["call"]=json!([6,32768]);assert!(read_native(&mut bad.into_iter().map(Ok)).is_err());
        assert!(read_native(&mut rows[..4].iter().cloned().map(Ok)).is_err());
    }
    #[test] fn priced_child_aliases_require_exclusive_parent_occurrences() {
        let rows=trace(vec![r(0x8038,0x20,511,0x9000,509),r(0x9000,0xea,509,0x9001,509),
            r(0x9001,0xea,509,0x9002,509),r(0x9002,0x60,509,0x803b,511),
            r(0x803b,0xea,511,0x805d,511),r(0x805d,0xea,511,0x805e,511)]);
        let c=calls(&rows).unwrap();
        let events=vec![scope("enter",10,0x9000),scope("enter",11,0x9001),scope("exit",11,0x9001),scope("exit",10,0x9000)];
        let n=fixture_native(events.clone());
        let (mapped,rules)=map_scopes(&n,&c,&rows).unwrap();assert_eq!(mapped[&10],1);assert_eq!(mapped[&11],1);
        assert_eq!(rules["priced_child_interior_partition"],1);
        let mut events=events;events.splice(3..3,[scope("enter",12,0x9001),scope("exit",12,0x9001)]);let duplicate=fixture_native(events);
        let (mapped,_)=map_scopes(&duplicate,&c,&rows).unwrap();assert!(!mapped.contains_key(&11));assert!(!mapped.contains_key(&12));
    }
    #[test] fn extras_follow_their_call_and_never_attach_to_another_call() {
        let charge=|id,pc,kind,width,clocks,seq|json!({"event":"charge","call":[id,0],"pc":pc,"kind":kind,
            "declared_width":width,"clocks":clocks,"site":"fixture.rs:1:1","sequence":seq});
        let n=fixture_native(vec![charge(1,0x9000,0,json!([true,true]),20,1),
            charge(2,0x9100,0,json!([false,true]),30,2),charge(1,0x9001,1,Value::Null,6,3),
            charge(3,0x9001,1,Value::Null,6,4)]);
        let (b,u)=native_blocks(&n,&[(1,100),(2,200),(3,300)].into());
        assert_eq!(b[0].extras,vec![(0x9001,1)]);assert_eq!(b[0].clocks,26);assert!(b[1].extras.is_empty());
        assert_eq!(u.len(),1);
    }
    #[test] fn shared_tail_occurrences_follow_execution_not_frame_creation() {
        let rows=trace(vec![r(0x8038,0x20,511,0x9000,509),r(0x9000,0x20,509,0x9100,507),
            r(0x9100,0x60,507,0x9003,509),r(0x9003,0x4c,509,0x9100,509),r(0x9100,0x60,509,0x803b,511),
            r(0x803b,0xea,511,0x805d,511),r(0x805d,0xea,511,0x805e,511)]);
        let c=calls(&rows).unwrap();
        let n=fixture_native(vec![scope("enter",10,0x9100),scope("exit",10,0x9100),scope("enter",11,0x9100),scope("exit",11,0x9100)]);
        let (mapped,_)=map_scopes(&n,&c,&rows).unwrap();assert_eq!(mapped[&10],2);assert_eq!(mapped[&11],1);
    }
    #[test] fn taken_extra_uses_retired_pc_across_interrupt_not_handler_pc() {
        let mut rom=vec![0xea;0x10000];rom[0x54f5]=0xd0;
        let mut rows=Vec::new();
        for pc in 0xd4ed..0xd4f5 {rows.push(r(pc,0xea,511,pc+1,511));}
        let mut branch=r(0xd4f5,0xd0,511,0x80c9,507);branch.interrupt_after=true;branch.retired_next_pc=Some(0xd500);
        rows.push(branch);let rows=trace(rows);
        let c=Calls {frames:Vec::new(),owners:vec![0;rows.len()],rules:BTreeMap::new()};
        assert_eq!(blocks(&rom,&rows,&c).unwrap()[0].extras,vec![(0xd4f5,1)]);
        let mut not_taken=rows;not_taken.last_mut().unwrap().retired_next_pc=Some(0xd4f7);
        assert!(blocks(&rom,&not_taken,&c).unwrap()[0].extras.is_empty());
    }
    #[test] fn page_extra_requires_actual_index_and_not_a_native_width_guess() {
        let mut rom=vec![0xea;0x10000];rom[0x5519..0x551c].copy_from_slice(&[0xbd,0xff,0x12]);
        let mut rows=Vec::new();let mut pc=0xd50f;
        while pc<0xd528 {
            let op=byte(&rom,pc).unwrap();let next=pc+length(op,true,true);let mut row=r(pc,op,511,next,511);row.x=Some(1);rows.push(row);pc=next;
        }
        let mut rows=trace(rows);let c=Calls {frames:Vec::new(),owners:vec![0;rows.len()],rules:BTreeMap::new()};
        assert_eq!(blocks(&rom,&rows,&c).unwrap()[0].extras,vec![(0xd519,2)]);
        let at=rows.iter().position(|r|r.pc==0xd519).unwrap();rows[at].x=Some(0);
        assert!(blocks(&rom,&rows,&c).unwrap()[0].extras.is_empty());
        rows[at].x=None;assert!(blocks(&rom,&rows,&c).unwrap().is_empty());
    }
    #[test]
    fn model_spans_reject_width_child_order_cost_duplicates_and_owned_opcodes() {
        let mut rows=trace(vec![r(0x8038,0x20,511,0xe843,509),
            r(0xe843,0xa7,509,0xe845,509),r(0xe845,0xa6,509,0xe847,509),
            r(0xe847,0x60,509,0x803b,511),r(0x803b,0xea,511,0x805d,511),r(0x805d,0xea,511,0x805e,511)]);
        rows[1].x8=false;rows[2].x8=false;
        let c=calls(&rows).unwrap();
        let mut n=Native::default();
        n.scopes.push(Scope{id:10,pc:0x8034,parent:None,site:"fixture".into()});
        for (seq,pc) in [0xe843,0xe845].into_iter().enumerate() {
            n.charges.push(Charge{model:Some(vec![(Some(pc),10,true)]),call:Some(10),pc:None,kind:None,width:None,
                clocks:10,site:"fixture".into(),sequence:seq as u64,domain:"scalar_unmapped".into()});
        }
        let mapping=[(10,0)].into();
        assert_eq!(model_spans(&n,&rows,&c,&mapping,&mut BTreeSet::new(),&[]).len(),1);
        assert!(model_spans(&n,&rows,&c,&mapping,&mut [1].into(),&[]).is_empty());
        let mut wrong=rows.clone();wrong[2].x8=true;
        assert!(model_spans(&n,&wrong,&c,&mapping,&mut BTreeSet::new(),&[]).is_empty());
        n.charges[0].model.as_mut().unwrap()[0].2=false;
        assert!(model_spans(&n,&rows,&c,&mapping,&mut BTreeSet::new(),&[]).is_empty());
        n.charges[0].model.as_mut().unwrap()[0].2=true;
        n.charges[0].clocks=11;
        assert!(model_spans(&n,&rows,&c,&mapping,&mut BTreeSet::new(),&[]).is_empty());
        n.charges[0].clocks=10;n.charges.swap(0,1);
        assert!(model_spans(&n,&rows,&c,&mapping,&mut BTreeSet::new(),&[]).is_empty());
        n.charges.swap(0,1);n.charges.push(n.charges[1].clone());
        assert!(model_spans(&n,&rows,&c,&mapping,&mut BTreeSet::new(),&[]).is_empty());
    }

    #[test]
    fn model_return_cut_keeps_unpriced_caller_on_source_unmatched_side() {
        let mut rows=vec![r(0xe2ce,0xab,511,0xe2cf,511),r(0xe2cf,0x6b,511,0x803c,514),
            r(0x803c,0x22,514,0xe5af,511),r(0xe5af,0xa2,511,0xe5b1,511)];
        rows[3].m8=false;
        let c=Calls {frames:vec![Call{id:0,parent:None,entry:0x8034,aliases:[0x8034].into(),
            return_pc:0,return_sp:0,entry_sp:511,closed:true}],owners:vec![0;4],rules:BTreeMap::new()};
        let mut n=Native::default();
        n.scopes.push(Scope{id:10,pc:0x8034,parent:None,site:"fixture".into()});
        for (seq,pc) in [0xe2ce,0xe2cf,0xe5af].into_iter().enumerate() {
            n.charges.push(Charge{model:Some(vec![(Some(pc),10,false)]),call:Some(10),pc:None,kind:None,width:None,
                clocks:10,site:"fixture".into(),sequence:seq as u64,domain:"scalar_unmapped".into()});
        }
        let mut owned=BTreeSet::new();
        let spans=model_spans(&n,&rows,&c,&[(10,0)].into(),&mut owned,&[]);
        assert_eq!(spans.len(),2);assert_eq!(owned,[0,1,3].into());
        assert!(!owned.contains(&2),"missing caller cannot retire as model work");
    }

}
