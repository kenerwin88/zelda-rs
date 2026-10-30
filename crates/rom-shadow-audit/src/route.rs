//! Ordered route join. Reuses call/marker mapping, with no local exclusion
//! waiver and no selection of an occurrence by its numeric cost.
use super::*;
use std::io::Read;
use sha2::{Digest,Sha256};
#[path="chunks.rs"]
mod chunks;
fn label_id(names:&mut BTreeMap<(String,String),usize>,class:&str,site:String)->usize {
    let next=names.len();*names.entry((class.into(),site)).or_insert(next)
}
fn compact_native(v:&Value)->Result<Native> {
    let mut n=Native::default();let mut stack=Vec::<(u64,u32)>::new();let mut ids=BTreeSet::new();
    for (sequence,e) in v["ordered_events"].as_array().ok_or("ordered native stream unavailable")?.iter().enumerate() {
        match e[0].as_u64().ok_or("ordered native kind")? {
            0=>{let id=e[1].as_u64().ok_or("scope ID")?;let pc=e[2].as_u64().ok_or("scope PC")? as u32;
                if !ids.insert(id){return Err("duplicate_native_scope_id".into());}
                n.scopes.push(Scope{id,pc,parent:stack.last().map(|s|s.0),site:e[3].as_str().ok_or("scope site")?.into()});
                if let Some(w)=e[4].as_u64(){n.local_widths.insert(id,u16::try_from(w)?);}stack.push((id,pc));},
            1=>{if stack.pop()!=Some((e[1].as_u64().ok_or("exit ID")?,e[2].as_u64().ok_or("exit PC")? as u32)){return Err("native_scope_crosses_iteration_cut".into());}},
            2=>{let call=e[1].as_u64();if call!=stack.last().map(|s|s.0){return Err("native_charge_scope_mismatch".into());}
                n.charges.push(Charge{call,pc:e[2].as_u64().map(|v|v as u32),kind:e[3].as_u64().map(|v|v as u8),
                    width:serde_json::from_value(e[4].clone())?,clocks:e[5].as_u64().ok_or("charge clocks")?,
                    site:e[6].as_str().ok_or("charge site")?.into(),model:serde_json::from_value(e[7].clone())?,
                    sequence:sequence as u64,domain:"main_instruction".into()});},
            _=>return Err("unknown ordered native event".into()),
        }
    }
    if !stack.is_empty(){return Err("unclosed ordered native scope".into());}
    n.total=Some(v["ledger_master"].as_u64().ok_or("native independent total")?);Ok(n)
}
pub(crate) fn source_row(input:&mut impl Read)->Result<Option<(u64,u64,Vec<Instruction>)>> {
    let mut h=[0u8;24];if input.read(&mut h[..1])?==0{return Ok(None);}input.read_exact(&mut h[1..])?;
    let u=|b:&[u8]|u64::from_le_bytes(b.try_into().unwrap());
    let (iteration,count,total)=(u(&h[..8]),u(&h[8..16]),u(&h[16..]));
    if count>2_000_000{return Err("bounded source iteration exceeded".into());}
    let mut rows=Vec::with_capacity(count as usize);
    for ordinal in 0..count {
        let mut b=[0u8;23];input.read_exact(&mut b)?;
        let word=|at|u16::from_le_bytes([b[at],b[at+1]]);
        let long=|at|u32::from_le_bytes(b[at..at+4].try_into().unwrap());
        rows.push(Instruction{iteration,ordinal,pc:long(0),retired_next_pc:Some(long(4)),master:u64::from(long(8)),
            sp:word(12),dp:word(14),x:(b[22]&16!=0).then(||word(16)),y:(b[22]&16!=0).then(||word(18)),db:b[20],opcode:b[21],
            m8:b[22]&1!=0,x8:b[22]&2!=0,emulation:b[22]&4!=0,interrupt_after:b[22]&8!=0,
            next_pc:long(4),next_sp:0});
    }
    for j in 0..rows.len(){rows[j].next_sp=rows.get(j+1).map_or(rows[j].sp,|r|r.sp);}
    if sum(&rows)!=total{return Err("source ordered counter mismatch".into());}Ok(Some((iteration,total,rows)))
}
/// Decode the already implemented preparation model's width contract in the
/// auditor only. It contains no prices and cannot affect the native model.
fn prep_widths(rom:&[u8])->Result<BTreeMap<u32,(bool,bool)>> {
    let mut todo=vec![(0x85fc,true,true)];let mut seen=BTreeSet::new();let mut out=BTreeMap::new();
    while let Some((pc,m,x))=todo.pop() {
        if !(0x85fc..0x8781).contains(&pc)||!seen.insert((pc,m,x)){continue;}
        if out.insert(pc,(m,x)).is_some_and(|v|v!=(m,x)){return Err("ambiguous preparation width contract".into());}
        let op=byte(rom,pc)?;let next=pc+length(op,m,x);let (mut nm,mut nx)=(m,x);
        if op==0xc2{let b=byte(rom,pc+1)?;nm&=b&32==0;nx&=b&16==0;}
        if op==0xe2{let b=byte(rom,pc+1)?;nm|=b&32!=0;nx|=b&16!=0;}
        if matches!(op,0x60|0x6b){continue;}
        if op==0x4c{todo.push(((pc&0xff0000)|u32::from(byte(rom,pc+1)?)|(u32::from(byte(rom,pc+2)?)<<8),nm,nx));continue;}
        if op&0x1f==0x10||op==0x80{let target=(next as i64+i64::from(byte(rom,pc+1)? as i8)) as u32;todo.push((target,nm,nx));if op==0x80{continue;}}
        todo.push((next,nm,nx));
    }Ok(out)
}
fn model_pairs(n:&Native,rows:&[Instruction],c:&Calls,mapping:&BTreeMap<u64,u64>,owned:&mut BTreeSet<usize>,
    widths:&BTreeMap<u32,(bool,bool)>,anchors:&[(u64,usize,usize)])->Vec<(usize,usize)> {
    let parents:BTreeMap<_,_>=n.scopes.iter().map(|s|(s.id,s.parent)).collect();
    let mut positions=BTreeMap::<u32,Vec<usize>>::new();for(j,r)in rows.iter().enumerate(){positions.entry(r.pc).or_default().push(j);}
    let returns:BTreeSet<_>=rows.iter().filter(|r|matches!(r.opcode,0x60|0x6b)&&!(0xe843..0xe852).contains(&r.pc)).map(|r|r.pc).collect();
    let mut out=Vec::new();let(mut at,mut cursor)=(0,0);
    while at<n.charges.len(){
        let start=at;
        while at<n.charges.len()&&n.charges[at].model.as_ref().is_some_and(|v|v.len()==1){
            let pc=n.charges[at].model.as_ref().unwrap()[0].0;at+=1;
            if pc.is_some_and(|pc|returns.contains(&pc)){break;}
        }
        if start==at{at+=1;continue;}
        let steps=&n.charges[start..at];let Some(first)=steps[0].model.as_ref().unwrap()[0].0 else{continue;};
        let mut parent=steps[0].call;let mut ancestor=None;
        while let Some(id)=parent{if let Some(&call)=mapping.get(&id){ancestor=Some(call);break;}parent=parents.get(&id).copied().flatten();}
        let Some(ancestor)=ancestor else{continue;};
        let candidates:Vec<_>=positions.get(&first).into_iter().flatten().copied().filter(|&j|{
            if j<cursor||j+steps.len()>rows.len(){return false;}
            if anchors.iter().any(|&(seq,lo,hi)|(seq<steps[0].sequence&&hi>=j)||(seq>steps.last().unwrap().sequence&&lo<j+steps.len())){return false;}
            steps.iter().zip(&rows[j..]).enumerate().all(|(k,(e,r))|{
                let(pc,_,child)=e.model.as_ref().unwrap()[0];let mut p=Some(c.owners[j+k]);
                while p.is_some_and(|p|p!=ancestor){p=c.frames[p.unwrap()as usize].parent;}
                pc==Some(r.pc)&&widths.get(&r.pc)==Some(&(r.m8,r.x8))&&r.dp==0&&!r.emulation
                    &&(!(0x85fc..0x8781).contains(&r.pc)||r.db==0)
                    &&child==(0xe843..0xe852).contains(&r.pc)&&p==Some(ancestor)&&!owned.contains(&(j+k))
            })
        }).collect();
        // Identity first. Cost never chooses one of several occurrences.
        if candidates.len()!=1{continue;}let j=candidates[0];
        for k in 0..steps.len(){owned.insert(j+k);out.push((j+k,start+k));}cursor=j+steps.len();
    }out
}
pub fn run(rom:&Path,source:&Path,native:&Path,predictor:&Path,out:&Path)->Result<()> {
    run_chunks(rom,source,native,predictor,out,false)
}
pub fn run_chunks(rom:&Path,source:&Path,native:&Path,predictor:&Path,out:&Path,prune:bool)->Result<()> {
    let root=source.parent().ok_or("ordered source parent")?;
    let manifest_path=root.join("manifest.json");let manifest:Manifest=serde_json::from_reader(File::open(&manifest_path)?)?;manifest.verify()?;
    let coverage:Value=serde_json::from_reader(File::open(root.join("coverage.json"))?)?;
    let expected=coverage["checked_source_rows"].as_u64().ok_or("source verified boundary")?;
    let native_identity=if native.is_dir(){native.join("producer.json")}else{native.to_path_buf()};
    let mut bound=vec![Identity::bind(rom)?,Identity::bind(source)?,Identity::bind(&native_identity)?,Identity::bind(predictor)?,Identity::bind(&manifest_path)?,
        Identity::bind(&root.join("iterations.jsonl.zst"))?,Identity::bind(&root.join("coverage.json"))?,Identity::bind(&std::env::current_exe()?)?];
    if let Some(parent)=native.parent(){let executable=parent.join("feature-zelda3");if executable.exists(){bound.push(Identity::bind(&executable)?);}}
    fs::create_dir(out)?;let rom=fs::read(rom)?;
    let mut input=zstd::Decoder::new(File::open(source)?)?;
    let mut natives:Box<dyn Iterator<Item=Result<Value>>>=if native.is_dir(){Box::new(chunks::Rows::new(native,&out.join("chunk-identities.jsonl"),prune)?)}else{Box::new(json_rows(native)?)};
    let mut aggregate=zstd::Encoder::new(File::create(out.join("native-aggregate.jsonl.zst"))?,3)?;
    let mut magic=[0u8;8];input.read_exact(&mut magic)?;if &magic!=b"Z3M4OP01"{return Err("unsupported ordered source schema".into());}
    let totals_path=root.join("iterations.jsonl.zst");let mut independent=json_rows(&totals_path)?;
    let prediction=predictions(predictor)?;let mut widths:BTreeMap<_,_>=graphics_widths::WIDTHS.iter().map(|&(p,m,x)|(p,(m,x))).collect();widths.extend(prep_widths(&rom)?);
    let mut summaries=zstd::Encoder::new(File::create(out.join("iterations.jsonl.zst"))?,3)?;
    let mut ownership=zstd::Encoder::new(File::create(out.join("ownership.bin.zst"))?,3)?;
    ownership.write_all(b"Z3M4OW01")?;let mut global_names=crate::ownership::Names::new();
    let mut signed=zstd::Encoder::new(File::create(out.join("signed-residuals.jsonl.zst"))?,3)?;
    let(mut checked,mut s,mut native_total,mut ps,mut pn,mut a_master,mut b_master,mut unequal)=(0u64,0u64,0u64,0u64,0u64,0u64,0u64,0u64);
    let mut failures=BTreeMap::<String,u64>::new();
    while let Some((i,total,rows))=source_row(&mut input)? {
        if i!=checked{return Err("ordered source iteration sequence".into());}
        let original=independent.next().ok_or("source independent total EOF")??;
        let mut path=Sha256::new();for r in &rows{path.update(r.pc.to_le_bytes());path.update([r.opcode]);}
        if original["iteration"]!=i||original["source_master"]!=total||original["source_instructions"]!=rows.len() as u64
            ||original["path_sha256"]!=format!("{:x}",path.finalize()){return Err("independent source order/count/clock mismatch".into());}
        let v=natives.next().ok_or("ordered native EOF")??;if v["iteration"]!=i{return Err("ordered source/native iteration mismatch".into());}
        writeln!(aggregate,"{}",chunks::aggregate(&v)?)?;
        let p=*prediction.get(&i).ok_or("predictor row absent")?;
        let mut label_names=BTreeMap::new();
        let initial=label_id(&mut label_names,"d","source_call_frame_unmapped".into());
        let mut labels=vec![initial;rows.len()];
        let(mut paired_source,mut paired_native,mut block_source,mut model_source)=(0u64,0u64,0u64,0u64);
        let mut zero_or_ownership_failures=rows.len()+v["ordered_events"].as_array().map_or(1,Vec::len);
        match (calls(&rows),compact_native(&v)) {
            (Ok(c),Ok(n))=>{
                if n.charges.iter().map(|e|e.clocks).sum::<u64>()+p.vwf!=p.main{return Err("ordered native independent mismatch".into());}
                let(mapping,_)=map_scopes(&n,&c,&rows)?;let(nb,_)=native_blocks(&n,&mapping);let sb=blocks(&rom,&rows,&c)?;
                let mut sc=BTreeMap::<(u64,u32),usize>::new();let mut nc=BTreeMap::<(u64,u32),usize>::new();
                let mut sites=BTreeMap::<(u64,u32),BTreeSet<String>>::new();
                for b in &sb{*sc.entry((b.call,b.pc)).or_default()+=1;}
                for b in &nb{*nc.entry((b.call,b.pc)).or_default()+=1;sites.entry((b.call,b.pc)).or_default().insert(b.site.clone());}
                let mut counts=BTreeMap::<(u64,u32),u64>::new();let mut bykey=BTreeMap::new();
                for b in nb{let k=counts.entry((b.call,b.pc)).or_default();bykey.insert((b.call,b.pc,*k),b);*k+=1;}
                let mut se=Vec::new();let mut ne=Vec::new();let mut meta=BTreeMap::new();
                for b in &sb{let key=(b.call,b.pc,b.occurrence);
                    if b.ordinals.iter().any(|&j|rows[j].dp!=0){continue;}
                    if sc.get(&(b.call,b.pc))!=nc.get(&(b.call,b.pc))||sites.get(&(b.call,b.pc)).is_none_or(|s|s.len()!=1){continue;}
                    let Some(n)=bykey.get(&key).filter(|n|n.m==b.m&&n.x==b.x&&n.extras==b.extras)else{continue;};
                    let k=Key{call:b.call,pc:b.pc,kind:0,occurrence:b.occurrence,m8:b.m,x8:b.x};
                    se.push(Event{key:k.clone(),clocks:b.clocks,site:n.site.clone(),domain:"instruction".into()});
                    meta.insert(key,(b,n.sequence));
                }
                let mut ns:Vec<_>=bykey.iter().collect();ns.sort_by_key(|(_,b)|b.sequence);
                for(key,b)in ns{if meta.contains_key(key){ne.push(Event{key:Key{call:b.call,pc:b.pc,kind:0,occurrence:key.2,m8:b.m,x8:b.x},clocks:b.clocks,site:b.site.clone(),domain:"instruction".into()});}}
                let(_,pairs)=ordered::pair(&se,&ne)?;let mut owned=BTreeSet::new();let mut anchors=Vec::new();
                let mut used_native=BTreeSet::new();let mut local_cost_errors=0usize;
                let mut source_sites=BTreeMap::<u64,BTreeSet<String>>::new();
                let mut model_claims=BTreeMap::<u32,BTreeSet<String>>::new();
                for e in &n.charges{if let Some(call)=e.call.and_then(|id|mapping.get(&id)){source_sites.entry(*call).or_default().insert(e.site.clone());}}
                for e in &n.charges{if let Some(steps)=&e.model{for &(pc,_,_)in steps{if let Some(pc)=pc{
                    model_claims.entry(pc).or_default().insert(format!("{}/{}",if steps.len()>1{"aggregate_model_child_order_unproved"}else{"model_call_width_or_order_unmapped"},e.site));
                }}}}
                let call_labels:BTreeMap<_,_>=c.frames.iter().map(|f|(f.id,label_id(&mut label_names,"d",
                    format!("scalar_instruction_set_or_call_alias_unproved/{:06x}/{:?}",f.entry,source_sites.get(&f.id))))).collect();
                let model_labels:BTreeMap<_,_>=model_claims.iter().map(|(pc,claims)|(*pc,label_id(&mut label_names,"d",format!("{claims:?}")))).collect();
                for(j,r)in rows.iter().enumerate(){labels[j]=model_labels.get(&r.pc).copied().unwrap_or(call_labels[&c.owners[j]]);}
                for b in &sb{
                    let key=(b.call,b.pc,b.occurrence);let n=bykey.get(&key);
                    let reason=if sc.get(&(b.call,b.pc))!=nc.get(&(b.call,b.pc)){"generated_marker_occurrence_count_mismatch"}
                        else if n.is_some_and(|n|n.m!=b.m||n.x!=b.x){"generated_marker_width_mismatch"}
                        else if n.is_some_and(|n|n.extras!=b.extras){"generated_marker_branch_or_page_extra_mismatch"}
                        else{"generated_marker_call_site_or_order_unproved"};
                    let id=label_id(&mut label_names,"d",format!("{reason}/{:06x}/{:?}",b.pc,sites.get(&(b.call,b.pc))));
                    for &j in &b.ordinals{labels[j]=id;}
                }
                for pair in pairs {
                    if pair.status!="paired"{writeln!(signed,"{}",json!({"iteration":i,"residual":pair}))?;continue;}
                    let (b,sequence)=meta.get(&(pair.key.call,pair.key.pc,pair.key.occurrence)).unwrap();
                    anchors.push((*sequence,b.ordinals[0],*b.ordinals.last().unwrap()));
                    if !used_native.insert(*sequence){return Err("duplicate native retirement owner".into());}
                    for e in &bykey[&(pair.key.call,pair.key.pc,pair.key.occurrence)].extra_sites {
                        used_native.insert(e["sequence"].as_u64().ok_or("extra event sequence")?);
                    }
                    let id=label_id(&mut label_names,"a",pair.site.clone());
                    for &j in &b.ordinals{if !owned.insert(j){return Err("duplicate ordered source owner".into());}labels[j]=id;}
                    paired_source+=pair.source;paired_native+=pair.native;block_source+=pair.source;
                    if pair.delta!=0{unequal+=1;local_cost_errors+=1;writeln!(signed,"{}",json!({"iteration":i,"residual":pair}))?;}
                }
                // Marker anchors constrain model order as well as the model's
                // own contiguous stream and source-call ownership.
                for (j,e) in model_pairs(&n,&rows,&c,&mapping,&mut owned,&widths,&anchors){
                    let e=&n.charges[e];labels[j]=label_id(&mut label_names,"b",e.site.clone());paired_source+=rows[j].master;paired_native+=e.clocks;model_source+=rows[j].master;
                    if !used_native.insert(e.sequence){return Err("duplicate native model retirement owner".into());}
                    let delta=i128::from(rows[j].master)-i128::from(e.clocks);
                    if delta!=0{unequal+=1;local_cost_errors+=1;writeln!(signed,"{}",json!({"iteration":i,"pc":rows[j].pc,"ordinal":j,"site":e.site,"signed_delta":delta}))?;}
                }
                zero_or_ownership_failures=rows.len()-owned.len()+n.charges.len()-used_native.len()+local_cost_errors;
                let mut unmatched=BTreeMap::<(String,Option<u32>,Option<u8>,String),(u64,u64,u64)>::new();
                for e in n.charges.iter().filter(|e|!used_native.contains(&e.sequence)){
                    let reason=match &e.model{Some(v)if v.len()!=1=>"aggregate_model_child_order_unproved",Some(v)if v[0].0.is_none()=>"model_pc_unavailable",
                        Some(_)=>"model_call_width_or_order_unmapped",None if e.pc.is_some()=>"generated_marker_call_width_extra_or_order_unmapped",None=>"scalar_instruction_set_unavailable"};
                    let pc=e.pc.or_else(||e.model.as_ref().filter(|s|s.len()==1).and_then(|s|s[0].0));
                    let v=unmatched.entry((e.site.clone(),pc,e.kind,reason.into())).or_insert((0,0,e.sequence));v.0+=e.clocks;v.1+=1;
                }
                if unmatched.values().map(|v|v.0).sum::<u64>()+p.vwf!=p.main-paired_native{return Err("native unmatched signed decomposition".into());}
                writeln!(signed,"{}",json!({"iteration":i,"native_unmatched":unmatched.into_iter().map(|((site,pc,kind,reason),(clocks,events,first))|
                    json!([site,pc,kind,-i128::from(clocks),events,first,reason])).collect::<Vec<_>>(),"separate_vwf_delta":-i128::from(p.vwf)}))?;
            }
            (Err(e),_)|(_,Err(e))=>{let reason=e.to_string();*failures.entry(reason.clone()).or_default()+=1;let id=label_id(&mut label_names,"d",reason);labels.fill(id);},
        }
        if paired_source>total||paired_native>p.main{return Err("ordered source/native ownership overflow".into());}
        let conservation=Conservation{source:total,native:p.main,paired_source,paired_native,
            unmatched_source:total-paired_source,unmatched_native:p.main-paired_native,
            signed_error:i128::from(paired_source)-i128::from(paired_native),unequal_pairs:zero_or_ownership_failures};conservation.check()?;
        let mut names=BTreeMap::<usize,usize>::new();let mut runs=Vec::<(usize,usize,usize,u64)>::new();
        for (j,label) in labels.into_iter().enumerate(){let next=names.len();let id=*names.entry(label).or_insert(next);
            if let Some(last)=runs.last_mut().filter(|r|r.2==id){last.1=j;last.3+=rows[j].master;}else{runs.push((j,j,id,rows[j].master));}}
        let mut named:Vec<_>=label_names.into_iter().filter_map(|(name,old)|names.get(&old).map(|&id|(id,name))).collect();named.sort_by_key(|v|v.0);
        crate::ownership::write_row(&mut ownership,i,rows.len(),&named,&runs,&mut global_names)?;
        writeln!(summaries,"{}",json!({"iteration":i,"conservation":conservation,"a_master":block_source,"b_master":model_source}))?;
        s+=total;native_total+=p.main;ps+=paired_source;pn+=paired_native;a_master+=block_source;b_master+=model_source;checked+=1;
        if checked%1000==0{eprintln!("ordered join rows={checked}/{expected}");}
    }
    if let Some(row)=natives.next(){row?;return Err("native rows beyond source boundary".into());}
    // The source summary can contain the final returned main before its next
    // host boundary has verified. It is outside the admitted ordered stream.
    let mut unverified_summary_suffix=0u64;
    for row in independent {let row=row?;if row["iteration"].as_u64()!=Some(expected+unverified_summary_suffix){return Err("independent source suffix sequence".into());}unverified_summary_suffix+=1;}
    summaries.finish()?;ownership.finish()?;signed.finish()?;aggregate.finish()?;
    let mut dictionary:Vec<_>=global_names.into_iter().map(|(name,id)|(id,name)).collect();dictionary.sort_by_key(|v|v.0);
    serde_json::to_writer(&mut File::create(out.join("ownership-names.json"))?,&dictionary)?;
    if checked!=expected{return Err("ordered source file ended before verified boundary".into());}
    manifest.verify()?;for id in &bound{id.verify()?;}
    let mut f=File::create(out.join("coverage.json"))?;serde_json::to_writer_pretty(&mut f,&json!({"verified_ordered_rows":checked,
        "source":s,"native":native_total,"paired_source":ps,"paired_native":pn,"U":s-ps,"O":native_total-pn,
        "E":i128::from(ps)-i128::from(pn),"a_master":a_master,"b_master":b_master,"unequal_pairs":unequal,
        "call_or_native_frame_failures":failures,"class_c_master":0,"identities":bound,"unverified_summary_suffix":unverified_summary_suffix,
        "pricing_admission":"STOP; no exclusion proof is waived"}))?;Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn model_occurrence_is_not_selected_by_equal_cost() {
        let rows:Vec<_>=[14,20].into_iter().enumerate().map(|(j,master)|Instruction{iteration:0,ordinal:j as u64,
            pc:1,opcode:0xea,master,m8:true,x8:true,dp:0,db:0,sp:511,next_pc:1,next_sp:511,
            interrupt_after:false,retired_next_pc:Some(1),x:None,y:None,emulation:false}).collect();
        let c=Calls{frames:vec![Call{id:0,parent:None,entry:1,aliases:[1].into(),return_pc:0,return_sp:511,entry_sp:511,closed:true}],owners:vec![0,0],rules:BTreeMap::new()};
        let mut n=Native::default();n.scopes.push(Scope{id:1,pc:1,parent:None,site:"model".into()});
        n.charges.push(Charge{model:Some(vec![(Some(1),14,false)]),call:Some(1),pc:None,kind:None,width:None,clocks:14,site:"model".into(),sequence:0,domain:"main_instruction".into()});
        let widths=[(1,(true,true))].into();let mapping=[(1,0)].into();
        assert!(model_pairs(&n,&rows,&c,&mapping,&mut BTreeSet::new(),&widths,&[]).is_empty());
        let mut unique=rows.clone();unique[1].pc=2;n.charges[0].sequence=2;
        assert_eq!(model_pairs(&n,&unique,&c,&mapping,&mut BTreeSet::new(),&widths,&[]),vec![(0,0)]);
        assert!(model_pairs(&n,&unique,&c,&mapping,&mut BTreeSet::new(),&widths,&[(1,1,1)]).is_empty(),"a model cannot cross an ordered marker anchor");
        n.charges[0].clocks=20;
        let pair=model_pairs(&n,&unique,&c,&mapping,&mut BTreeSet::new(),&widths,&[]);
        assert_eq!(pair,vec![(0,0)]);assert_eq!(i128::from(unique[pair[0].0].master)-i128::from(n.charges[pair[0].1].clocks),-6);
        assert_eq!(model_pairs(&n,&rows[..1],&Calls{owners:vec![0],..c},&mapping,&mut BTreeSet::new(),&widths,&[]),vec![(0,0)]);
    }
    #[test]
    fn native_zero_clock_or_wrong_scope_events_cannot_disappear() {
        let v=json!({"ledger_master":0,"ordered_events":[[0,0,0x8034,"site",null],[2,0,null,null,null,0,"unknown",null],[1,0,0x8034]]});
        let n=compact_native(&v).unwrap();assert_eq!(n.charges.len(),1);assert_eq!(n.charges[0].clocks,0);
        let mut wrong=v;wrong["ordered_events"][1][1]=json!(9);assert!(compact_native(&wrong).is_err());
    }
}
