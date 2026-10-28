//! Conservative live-stream reconciliation before call/site alignment admission.
//! This deliberately cannot promote a routine difference into a price.
use crate::{Conservation,Identity,Result,ordinary_regions};
use serde_json::{json,Value};
use std::{collections::BTreeMap,fs::{self,File},io::{BufRead,BufReader,Write},path::Path};

fn rows(path:&Path)->Result<impl Iterator<Item=Result<Value>>> {
    Ok(BufReader::new(zstd::Decoder::new(File::open(path)?)?).lines()
        .map(|line|Ok(serde_json::from_str(&line?)?)))
}
pub fn reconcile(source:&Path,predictor:&Path,events:&Path,out:&Path)->Result<()> {
    reconcile_window(source,predictor,events,out,0,u64::MAX)
}
pub fn reconcile_window(source:&Path,predictor:&Path,events:&Path,out:&Path,lo:u64,hi:u64)->Result<()> {
    if hi<lo {return Err("reversed accounting window".into());}
    let coverage:Value=serde_json::from_reader(File::open(source.join("coverage.json"))?)?;
    let checked=coverage["checked_source_rows"].as_u64().ok_or("coverage absent")?;
    let identities=[Identity::bind(&source.join("source-instructions.jsonl.zst"))?,
        Identity::bind(&source.join("iterations.jsonl.zst"))?,Identity::bind(&source.join("coverage.json"))?,
        Identity::bind(&source.join("entry-kinds.jsonl.zst"))?,
        Identity::bind(predictor)?,Identity::bind(events)?];
    let mut instructions:BTreeMap<u64,Vec<Value>>=BTreeMap::new();
    let mut retained=0usize;
    for row in rows(&source.join("source-instructions.jsonl.zst"))? {
        let row=row?;let i=row["iteration"].as_u64().ok_or("instruction iteration")?;
        if i>=checked || !(lo..=hi).contains(&i) {continue;}
        retained+=1;
        if retained>200_000 {return Err("bounded accounting window exceeds 200000 instructions; split the diagnostic window".into());}
        let group=instructions.entry(i).or_default();
        if row["ordinal"].as_u64()!=Some(group.len() as u64) {return Err("source partition ordinal absent/duplicate".into());}
        group.push(row);
    }
    let mut entries=BTreeMap::new();
    for row in rows(&source.join("entry-kinds.jsonl.zst"))? {
        let row=row?;let i=row["iteration"].as_u64().ok_or("entry index")?;
        if !instructions.contains_key(&i) {continue;}
        let module=u8::try_from(row["module"].as_u64().ok_or("entry module")?)?;
        let sub=u8::try_from(row["submodule"].as_u64().ok_or("entry submodule")?)?;
        if row["kind"]!="ordinary_admitted" {return Err("accounting entry is not admitted".into());}
        ordinary_regions(module,sub,u32::try_from(row["pc"].as_u64().ok_or("entry PC")?)?)?;
        if entries.insert(i,format!("{module:02x}/{sub:02x}@008038")).is_some() {return Err("duplicate accounting entry".into());}
    }
    let mut source_totals=BTreeMap::new();
    for row in rows(&source.join("iterations.jsonl.zst"))? {
        let row=row?;let Some(i)=row["iteration"].as_u64() else {continue;};
        if instructions.contains_key(&i) && source_totals.insert(i,row).is_some() {return Err("duplicate source total".into());}
    }
    let mut native:BTreeMap<u64,Vec<Value>>=BTreeMap::new();let mut ends=BTreeMap::new();
    let mut active=None;let mut sequence=0u64;
    let event_reader:Box<dyn BufRead>=if events.extension().is_some_and(|s|s=="zst") {
        Box::new(BufReader::new(zstd::Decoder::new(File::open(events)?)?))
    } else {Box::new(BufReader::new(File::open(events)?))};
    for line in event_reader.lines() {
        let row:Value=serde_json::from_str(&line?)?;
        if row["event"]=="begin" {
            if active.is_some() {return Err("nested observer begin".into());}
            active=Some(row["iteration"].as_u64().ok_or("observer iteration")?);sequence=1;
            continue;
        }
        let i=active.ok_or("observer event outside main")?;
        if row["sequence"].as_u64()!=Some(sequence) {return Err("observer sequence absent/duplicate".into());}sequence+=1;
        if row["event"]=="charge" && instructions.contains_key(&i) {native.entry(i).or_default().push(row.clone());}
        if row["event"]=="end" {
            if ends.insert(i,row["ledger_master"].as_u64().ok_or("observer total")?).is_some() {return Err("duplicate observer main".into());}
            active=None;
        }
    }
    if active.is_some(){return Err("truncated observer main".into());}
    let mut lines=BufReader::new(File::open(predictor)?).lines();
    let h=lines.next().ok_or("predictor header")??;let h:Vec<_>=h.split(',').collect();
    let col=|name:&str|->Result<usize>{h.iter().position(|c|*c==name).ok_or_else(||format!("missing {name}").into())};
    let (index,main,vwf)=(col("iteration")?,col("main_master")?,col("vwf_master")?);
    let mut predictions=BTreeMap::new();
    for line in lines {
        let line=line?;let c:Vec<_>=line.split(',').collect();
        if c.len()!=h.len(){return Err("predictor row".into());}
        let i:u64=c[index].parse()?;
        if instructions.contains_key(&i) && predictions.insert(i,(c[main].parse::<u64>()?,c[vwf].parse::<u64>()?)).is_some() {return Err("duplicate predictor iteration".into());}
    }
    fs::create_dir(out)?;
    let mut blocks=zstd::Encoder::new(File::create(out.join("blocks.jsonl.zst"))?,3)?;
    let mut iterations=zstd::Encoder::new(File::create(out.join("iterations.jsonl.zst"))?,3)?;
    let mut residuals=zstd::Encoder::new(File::create(out.join("residuals.jsonl.zst"))?,3)?;
    let mut joined=0;let mut missing_native=0;
    let mut interrupted_iterations=0;
    for (i,source_rows) in &instructions {
        let entry=entries.get(i).ok_or("missing admitted source entry")?;
        let source_sum=source_rows.iter().try_fold(0u64,|a,r|a.checked_add(r["master"].as_u64().ok_or("source clocks")?).ok_or("source overflow"))?;
        let independent=source_totals.get(i).ok_or("missing independent source total")?;
        if independent["source_master"].as_u64()!=Some(source_sum) || independent["source_instructions"].as_u64()!=Some(source_rows.len() as u64) {return Err("source partition conservation".into());}
        let mut after_interrupt=false;let mut resumed_instructions=0;
        for (ordinal,r) in source_rows.iter().enumerate() {
            resumed_instructions+=usize::from(after_interrupt);
            writeln!(blocks,"{}",json!({"iteration":i,"entry_variant":entry,"pc":r["pc"],"ordinal":r["ordinal"],
            "m8":r["m8"],"x8":r["x8"],"source_master":r["master"],"native_master":null,
            "signed_pair_error":null,"interrupt_after":r["interrupt_after"],"native_site":null,
            "status":if after_interrupt {"unmapped_resume_handoff"}else{"source_call_site_unmapped"}}))?;
            if r["interrupt_after"]==true {
                after_interrupt=true;
                writeln!(residuals,"{}",json!({"iteration":i,"domain":"main_instruction","reason":"unmapped_resume_handoff",
                    "source_pc":r["pc"],"source_resume_pc":source_rows.get(ordinal+1).map(|r|&r["pc"]),
                    "master_clocks":null,"native_site_candidates":[],"entry_variant":entry}))?;
            }
        }
        interrupted_iterations+=usize::from(after_interrupt);
        let Some(&observed)=ends.get(i) else {
            missing_native+=1;
            writeln!(iterations,"{}",json!({"iteration":i,"entry_variant":entry,"source_master":source_sum,"native_master":null,"status":"native_window_unobserved"}))?;
            continue;
        };
        let native_rows=native.get(i).map(Vec::as_slice).unwrap_or(&[]);
        let native_sum=native_rows.iter().try_fold(0u64,|a,r|a.checked_add(r["clocks"].as_u64().ok_or("native clocks")?).ok_or("native overflow"))?;
        let &(main,vwf)=predictions.get(i).ok_or("missing predictor total")?;
        if native_sum!=observed || native_sum.checked_add(vwf)!=Some(main) {return Err("native observer/predictor conservation".into());}
        let c=Conservation {source:source_sum,native:main,paired_source:0,paired_native:0,
            unmatched_source:source_sum,unmatched_native:main,signed_error:0,unequal_pairs:0};
        c.check()?;
        for r in native_rows {writeln!(blocks,"{}",json!({"iteration":i,"entry_variant":entry,"pc":r["pc"],"native_site":r["site"],
            "call":r["call"],"declared_width":r["declared_width"],"source_master":null,"native_master":r["clocks"],
            "signed_pair_error":null,"retirement_stage":"unknown","domain":r["domain"],"status":"native_call_source_unmapped"}))?;}
        writeln!(blocks,"{}",json!({"iteration":i,"domain":"vwf_separate","source_master":null,"native_master":vwf,"status":"vwf_unmapped"}))?;
        writeln!(iterations,"{}",json!({"iteration":i,"entry_variant":entry,"conservation":c,"source_independent_equal":true,
            "native_independent_equal":true,"unmapped_resume_handoff":after_interrupt,"resumed_source_instructions":resumed_instructions,
            "alignment":"unproved_call_site_mapping","exact":false}))?;
        joined+=1;
    }
    blocks.finish()?;iterations.finish()?;residuals.finish()?;
    for id in &identities {id.verify()?;}
    let report=json!({"identities":identities,"source_missing_ordinary_iterations":instructions.len(),
        "iteration_window":[lo,hi],
        "source_and_native_counter_reconciled":joined,"native_window_unobserved":missing_native,
        "unmapped_resume_iterations":interrupted_iterations,
        "ordered_pairing_admitted":false,"pricing_verdict":"STOP","reason":"live call/site mapping not proved"});
    serde_json::to_writer_pretty(File::create(out.join("coverage.json"))?,&report)?;
    serde_json::to_writer_pretty(File::create(out.join("worklist.json"))?,&json!({"validated_positive_gaps":[],
        "negative_price_repairs":[],"top_gaps_by_clocks":[],"top_gaps_by_missing_holds_covered":[],"admission":"STOP"}))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn compressed(path:&Path,text:&str) {
        fs::write(path,zstd::encode_all(text.as_bytes(),1).unwrap()).unwrap();
    }
    #[test]
    fn independent_counters_reject_deleted_instruction_and_wrong_observer_total() {
        let root=std::env::temp_dir().join(format!("shadow-accounting-{}",std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("coverage.json"),r#"{"checked_source_rows":1}"#).unwrap();
        compressed(&root.join("entry-kinds.jsonl.zst"),"{\"iteration\":0,\"kind\":\"ordinary_admitted\",\"module\":9,\"submodule\":0,\"pc\":32824}\n");
        let instructions="{\"iteration\":0,\"ordinal\":0,\"pc\":32824,\"master\":60}\n{\"iteration\":0,\"ordinal\":1,\"pc\":32825,\"master\":40}\n";
        compressed(&root.join("source-instructions.jsonl.zst"),instructions);
        compressed(&root.join("iterations.jsonl.zst"),"{\"iteration\":0,\"source_master\":100,\"source_instructions\":2}\n");
        let predictor=root.join("predictor.csv");
        fs::write(&predictor,"iteration,main_master,vwf_master\n0,110,10\n").unwrap();
        let events=root.join("events.jsonl");
        let stream="{\"event\":\"begin\",\"iteration\":0}\n{\"event\":\"charge\",\"sequence\":1,\"clocks\":100,\"site\":\"site-a\"}\n{\"event\":\"end\",\"sequence\":2,\"ledger_master\":100}\n";
        fs::write(&events,stream).unwrap();
        let good=root.join("good");reconcile(&root,&predictor,&events,&good).unwrap();
        let report:Value=serde_json::from_reader(File::open(good.join("coverage.json")).unwrap()).unwrap();
        assert_eq!(report["source_and_native_counter_reconciled"],1);
        assert_eq!(report["ordered_pairing_admitted"],false);
        fs::write(&events,stream.replace("\"ledger_master\":100","\"ledger_master\":99")).unwrap();
        assert!(reconcile(&root,&predictor,&events,&root.join("wrong-counter")).is_err());
        fs::write(&events,stream).unwrap();
        compressed(&root.join("source-instructions.jsonl.zst"),instructions.lines().next().unwrap());
        assert!(reconcile(&root,&predictor,&events,&root.join("deleted-instruction")).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
