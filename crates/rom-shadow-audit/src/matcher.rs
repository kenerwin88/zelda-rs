//! M4 compressed inventory join. This reports claims and blockers; a static
//! range, equal count, or equal cost cannot silently create an ordered pair.
use crate::{Identity, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json,Value};
use std::{collections::{BTreeMap,BTreeSet},fs::{self,File},io::{BufRead,BufReader,Write},path::Path};
mod generated { include!("../../zelda3/src/rom_block_costs.rs"); }
#[derive(Deserialize)]
pub struct Claim {
    name:String, entry:u32, pc_end:u32, pcs:Vec<(u32,u8)>, native_file:String,
    native_function:String, native_line:u32, source_sha256:String,
    exclusion_proved:bool, blocker:String, review:String,
}
#[derive(Default,Serialize)]
struct Exposure { master:u64,source_entry_visits:u64,native_scope_entries:u64,iterations:u64,
    estimated_holds:u64,zero_lag_fp_risk:u64,entry_count_mismatch_iterations:u64 }
fn reader(p:&Path)->Result<Box<dyn BufRead>> {
    let f=File::open(p)?;
    Ok(if p.extension().is_some_and(|v|v=="zst") {Box::new(BufReader::new(zstd::Decoder::new(f)?))}
        else {Box::new(BufReader::new(f))})
}
fn next(lines:&mut impl Iterator<Item=std::io::Result<String>>)->Result<Value> {
    Ok(serde_json::from_str(&lines.next().ok_or("missing ordered inventory row")??)?)
}
fn number(v:&Value)->Result<u64> {v.as_u64().ok_or_else(||"missing unsigned audit counter".into())}
/// Inputs are identity-bound, and every row conserves source and native work.
/// No numeric cost is used to select an instruction owner.
pub fn run(source:&Path,native:&Path,predictor:&Path,inverses:&Path,catalog:&Path,out:&Path)->Result<()> {
    let claims:Vec<Claim>=serde_json::from_reader(File::open(catalog)?)?;
    let mut ids=BTreeMap::new();
    ids.insert("matcher_binary".to_string(),Identity::bind(&std::env::current_exe()?)?);
    ids.insert("block_table".to_string(),Identity::bind(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../zelda3/src/rom_block_costs.rs"))?);
    for (name,p) in [("source",source),("native",native),("predictor",predictor),("inverses",inverses),("catalog",catalog)] {
        ids.insert(name.to_string(),Identity::bind(p)?);
    }
    for c in &claims {
        if c.exclusion_proved {return Err("M4 inventory cannot admit an unimplemented exclusion proof".into());}
        let id=Identity::bind(Path::new(&c.native_file))?;
        if id.sha256!=c.source_sha256 {return Err("stale native site catalog".into());}
        ids.insert(c.native_file.clone(),id);
    }
    fs::create_dir(out)?;
    let mut s=reader(source)?.lines();let mut n=reader(native)?.lines();let mut inv=reader(inverses)?.lines();
    let mut pred=reader(predictor)?.lines();
    let header=pred.next().ok_or("predictor header absent")??;
    let cols:Vec<_>=header.split(',').collect();
    let col=|name:&str|->Result<usize>{cols.iter().position(|v|*v==name).ok_or_else(||format!("missing predictor {name}").into())};
    let (ic,mc,vc,lc,hc)=(col("iteration")?,col("main_master")?,col("vwf_master")?,col("lag_nmis")?,col("native_held")?);
    let mut pc_markers=BTreeMap::<u32,Vec<u32>>::new();
    for &(a,b,_,_,_,_,_) in generated::BLOCKS {for pc in a..b {pc_markers.entry(pc).or_default().push(a);}}
    let memberships:Vec<BTreeSet<_>>=claims.iter().map(|c|c.pcs.iter().copied().collect()).collect();
    let mut exposures:Vec<Exposure>=claims.iter().map(|_|Exposure::default()).collect();
    let mut ambiguity=BTreeMap::<(u32,u8,String),[u64;2]>::new();
    let mut native_unknown=BTreeMap::<String,[u64;2]>::new();
    let mut native_claims=BTreeMap::<String,BTreeSet<u32>>::new();
    let mut rows=zstd::Encoder::new(File::create(out.join("conservation.jsonl.zst"))?,3)?;
    let (mut count,mut source_total,mut native_total)=(0u64,0u64,0u64);
    for line in &mut s {
        let a:Value=serde_json::from_str(&line?)?;let b=next(&mut n)?;let iv=next(&mut inv)?;
        let line=pred.next().ok_or("predictor ended early")??;
        let v:Vec<_>=line.split(',').collect();if v.len()!=cols.len(){return Err("predictor row width".into());}
        let p=|index:usize|->Result<u64>{Ok(v[index].parse()?)};
        if number(&a["iteration"])?!=count || number(&b["iteration"])?!=count || number(&iv["iteration"])?!=count || p(ic)?!=count {
            return Err("missing/duplicate/reordered inventory iteration".into());
        }
        let source_master=number(&a["source_master"])?;let native_master=p(mc)?;
        if number(&b["ledger_master"])?+p(vc)?!=native_master {return Err("native independent counter mismatch".into());}
        let sites=b["sites"].as_array().ok_or("native sites absent")?;
        let site_sum=sites.iter().try_fold(0u64,|sum,r|->Result<u64>{Ok(sum+number(&r[2][0])?)})?;
        if site_sum!=number(&b["ledger_master"])? {return Err("native site conservation".into());}
        let mut markers=BTreeSet::new();let mut marker_master=0;
        for r in b["blocks"].as_array().ok_or("native blocks absent")? {
            if number(&r[2])?==0 {markers.insert(number(&r[1])? as u32);}
            marker_master+=number(&r[4][0])?;
        }
        let mut model_pc=BTreeMap::<u32,BTreeSet<String>>::new();
        for r in b["model_steps"].as_array().ok_or("native model witnesses absent")? {
            if let Some(pc)=r[2].as_u64(){
                let site=r[1].as_str().ok_or("model site")?;
                model_pc.entry(pc as u32).or_default().insert(site.into());
                native_claims.entry(format!("model_sites/{site}")).or_default().insert(pc as u32);
            }
        }
        let mut other=0;
        for field in ["scalar_sites","model_sites"] {
            for r in b[field].as_array().ok_or("native charge-kind sites absent")? {
                let clocks=number(&r[2][0])?;other+=clocks;
                let key=format!("{field}/{}",r[1].as_str().ok_or("native site name")?);
                let z=native_unknown.entry(key).or_default();z[0]+=clocks;z[1]+=number(&r[2][1])?;
            }
        }
        if other+marker_master!=number(&b["ledger_master"])? {return Err("native marker/model/scalar partition".into());}
        let mut sum=0;let mut instructions=0;let mut source_calls=vec![0u64;claims.len()];let mut work=vec![0u64;claims.len()];
        for r in a["pc_work"].as_array().ok_or("source PC inventory absent")? {
            let pc=number(&r[0])? as u32;let width=number(&r[1])? as u8;
            if width>3 {return Err("invalid source M/X inventory width".into());}
            let occurrences=number(&r[2][0])?;let clocks=number(&r[2][1])?;sum+=clocks;instructions+=occurrences;
            let mut names=Vec::new();
            for (j,c) in claims.iter().enumerate() {
                if memberships[j].contains(&(pc,width)) {
                    work[j]+=clocks;names.push(format!("outside_charge_exclusion_unproved/{}::{}:{}",c.native_file,c.native_function,c.native_line));
                }
                if pc==c.entry {source_calls[j]+=occurrences;}
            }
            if let Some(sites)=model_pc.get(&pc) {for site in sites {names.push(format!("model_width_call_and_order_unproved/{site}"));}}
            if let Some(blocks)=pc_markers.get(&pc) {for block in blocks.iter().filter(|p|markers.contains(*p)) {
                names.push(format!("generated_marker_call_and_order_unproved/{block:06x}"));
            }}
            if names.is_empty(){names.push(format!("no_declared_native_instruction_set/{pc:06x}/width-{width}"));}
            names.sort();names.dedup();
            let z=ambiguity.entry((pc,width,names.join(";"))).or_default();z[0]+=occurrences;z[1]+=clocks;
        }
        if sum!=source_master || instructions!=number(&a["instructions"])? {return Err("source PC conservation".into());}
        for (j,c) in claims.iter().enumerate() {
            let native_calls=b["paths"].as_array().ok_or("native paths")?.iter().filter(|r|r[0].as_array().is_some_and(|p|p.last().and_then(Value::as_u64)==Some(u64::from(c.entry))))
                .try_fold(0u64,|sum,r|->Result<u64>{Ok(sum+r[1].as_array().ok_or("native call vector")?.len() as u64)})?;
            let z=&mut exposures[j];z.master+=work[j];z.source_entry_visits+=source_calls[j];z.native_scope_entries+=native_calls;
            z.iterations+=u64::from(work[j]>0);z.entry_count_mismatch_iterations+=u64::from(source_calls[j]!=native_calls);
            for x in iv["inverses"].as_array().ok_or("inverse rows")? {
                let target=number(&x[0])?;let threshold=number(&x[1][1])?;
                if p(hc)?<target && target<=p(lc)? && work[j]>=threshold {z.estimated_holds+=1;}
                if target==1 && p(lc)?==0 && work[j]>0 && work[j]>=threshold {z.zero_lag_fp_risk+=1;}
            }
        }
        writeln!(rows,"{}",json!({"iteration":count,"source":source_master,"native":native_master,
            "paired_source":0,"paired_native":0,"U":source_master,"O":native_master,"E":0,
            "signed_delta":i128::from(source_master)-i128::from(native_master),"class_d":source_master}))?;
        source_total+=source_master;native_total+=native_master;count+=1;
    }
    rows.finish()?;
    // A compressed inventory is not an ordered source/native stream. Explicitly
    // withhold all positive admission, including zero-cost unknown events.
    let mut worklist:Vec<_>=claims.iter().zip(&exposures).map(|(c,e)|json!({"name":c.name,
        "source_entry":c.entry,"end_exclusive":c.pc_end,"instruction_set":c.pcs,
        "native_site":format!("{}::{}:{}",c.native_file,c.native_function,c.native_line),
        "classification":"d","immediately_priceable":false,"blocker":c.blocker,
        "outside_charge_exclusion_proved":false,"source_call_reconstruction_available":false,
        "review":c.review,"exposure":e})).collect();
    worklist.sort_by_key(|v|std::cmp::Reverse(v["exposure"]["estimated_holds"].as_u64().unwrap_or(0)));
    let mut files=BTreeMap::new();
    files.insert("coverage.json",json!({"verified_rows":count,"source_master":source_total,"native_master":native_total,
        "signed_delta":i128::from(source_total)-i128::from(native_total),"class_d_master":source_total,
        "class_a_master":0,"class_b_master":0,"class_c_master":0,"ordered_pairing_complete":false,
        "warning":"PC count inventory and claim catalog are not an ordered ownership proof"}));
    files.insert("worklist.json",json!({"uncharged_instruction_ranges":[],"blocked_candidates":worklist,
        "estimates":"source instruction exposure, completed-model inverse, fixed current context; overlapping scores"}));
    files.insert("native-ambiguities.json",json!(native_unknown.into_iter().map(|(site,v)| {
        let claim=native_claims.get(&site);
        json!({"site":site,"master":v[0],"events":v[1],"observed_model_instruction_set":claim,
            "ambiguity":if claim.is_some(){"model_source_call_width_and_order_unproved"}else{"scalar_instruction_set_unavailable"}})
    }).collect::<Vec<_>>()));
    files.insert("identities.json",serde_json::to_value(&ids)?);
    for (name,value) in files {let mut f=File::create(out.join(name))?;serde_json::to_writer_pretty(&mut f,&value)?;writeln!(f)?;}
    let mut out_pc=zstd::Encoder::new(File::create(out.join("instruction-ownership.jsonl.zst"))?,3)?;
    for ((pc,width,reason),v) in ambiguity {writeln!(out_pc,"{}",json!({"pc":pc,"width":width,"class":"d","ambiguity":reason,"occurrences":v[0],"master":v[1]}))?;}
    out_pc.finish()?;
    for id in ids.values(){id.verify()?;}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_cardinality_and_zero_native_work_do_not_admit_a_gap() {
        let root=std::env::temp_dir().join(format!("matcher-inventory-{}-{}",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();
        let src=root.join("src.jsonl");let nat=root.join("nat.jsonl");let pred=root.join("pred.csv");
        let inv=root.join("inv.jsonl");let cat=root.join("cat.json");let implementation=root.join("helper.rs");
        fs::write(&implementation,"fn helper() {}\n").unwrap();
        fs::write(&src,"{\"iteration\":0,\"source_master\":14,\"instructions\":1,\"pc_work\":[[1,3,[1,14]]]}\n").unwrap();
        fs::write(&nat,json!({"iteration":0,"ledger_master":0,"sites":[[[1],"helper.rs:1:1",[0,1]]],
            "blocks":[],"scalar_sites":[[[1],"helper.rs:1:1",[0,1]]],"model_sites":[],"model_steps":[],
            "paths":[[[1],[0]]]}).to_string()+"\n").unwrap();
        fs::write(&pred,"iteration,main_master,vwf_master,lag_nmis,native_held\n0,0,0,0,0\n").unwrap();
        fs::write(&inv,"{\"iteration\":0,\"inverses\":[[1,[0,100]]]}\n").unwrap();
        fs::write(&cat,json!([{"name":"helper","entry":1,"pc_end":2,"pcs":[[1,3]],
            "native_file":implementation,"native_function":"helper","native_line":1,
            "source_sha256":Identity::bind(&implementation).unwrap().sha256,
            "exclusion_proved":false,"blocker":"outside ancestor instruction set unknown","review":"fixture"}]).to_string()).unwrap();
        let out=root.join("out");run(&src,&nat,&pred,&inv,&cat,&out).unwrap();
        let coverage:Value=serde_json::from_reader(File::open(out.join("coverage.json")).unwrap()).unwrap();
        assert_eq!(coverage["class_d_master"],14);assert_eq!(coverage["class_c_master"],0);
        let work:Value=serde_json::from_reader(File::open(out.join("worklist.json")).unwrap()).unwrap();
        assert_eq!(work["uncharged_instruction_ranges"],json!([]));
        let unknown:Value=serde_json::from_reader(File::open(out.join("native-ambiguities.json")).unwrap()).unwrap();
        assert_eq!(unknown[0]["events"],1);assert_eq!(unknown[0]["master"],0);
        fs::remove_dir_all(root).unwrap();
    }
}
