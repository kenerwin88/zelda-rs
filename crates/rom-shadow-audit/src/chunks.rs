//! Closed native chunks are read once in order. Explicit pruning retains a
//! compressed-byte identity and decoded stream hash before deleting raw input.
use crate::{Identity,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{fs::{self,File},io::{BufRead,BufReader,BufWriter,Write},path::{Path,PathBuf},time::Duration};
pub struct Rows {
    dir:PathBuf,next:u64,lines:Option<std::io::Lines<Box<dyn BufRead>>>,identity:Option<Identity>,
    meta:Value,seen:u64,digest:Sha256,receipts:BufWriter<File>,prune:bool,finished:bool,
}
impl Rows {
    pub fn new(dir:&Path,receipts:&Path,prune:bool)->Result<Self>{
        Ok(Self{dir:dir.into(),next:0,lines:None,identity:None,meta:Value::Null,seen:0,digest:Sha256::new(),
            receipts:BufWriter::new(File::options().create_new(true).write(true).open(receipts)?),prune,finished:false})
    }
    fn advance(&mut self)->Result<Option<Value>>{
        loop {
            if let Some(lines)=self.lines.as_mut(){
                if let Some(line)=lines.next(){let line=line?;let value:Value=serde_json::from_str(&line)?;
                    if value["iteration"].as_u64()!=Some(self.next+self.seen){return Err("native chunk row sequence".into());}
                    self.digest.update(line.as_bytes());self.digest.update(b"\n");self.seen+=1;
                    if self.seen>self.meta["rows"].as_u64().ok_or("chunk rows")?{return Err("native chunk row overflow".into());}
                    return Ok(Some(value));
                }
                self.lines=None;
                if self.meta["rows"].as_u64()!=Some(self.seen){return Err("native chunk premature EOF".into());}
                let id=self.identity.take().ok_or("missing chunk identity")?;id.verify()?;
                writeln!(self.receipts,"{}",json!({"input":id,"range":self.meta,
                    "decoded_sha256":format!("{:x}",self.digest.clone().finalize()),"pruned":self.prune}))?;
                self.receipts.flush()?;
                if self.prune {fs::remove_file(&id.path)?;}
                self.next+=self.seen;
            }
            let done=self.dir.join(format!("{:012}.done.json",self.next));
            if done.exists(){
                let meta:Value=serde_json::from_reader(File::open(&done)?)?;
                let rows=meta["rows"].as_u64().ok_or("native chunk row count")?;
                if rows==0||rows>1000||meta["first"].as_u64()!=Some(self.next)||meta["last"].as_u64()!=Some(self.next+rows-1){return Err("native chunk interval".into());}
                let path=self.dir.join(format!("{:012}.zst",self.next));self.identity=Some(Identity::bind(&path)?);
                let input:Box<dyn BufRead>=Box::new(BufReader::new(zstd::Decoder::new(File::open(&path)?)?));
                self.lines=Some(input.lines());self.meta=meta;self.seen=0;self.digest=Sha256::new();continue;
            }
            let eof=self.dir.join("EOF.json");
            if eof.exists(){let eof:Value=serde_json::from_reader(File::open(eof)?)?;
                if eof["rows"].as_u64()!=Some(self.next){return Err("native chunk missing interval before EOF".into());}
                self.finished=true;return Ok(None);
            }
            if self.dir.join("FAILED.json").exists(){return Err("native chunk producer failed".into());}
            std::thread::sleep(Duration::from_millis(200));
        }
    }
}
impl Iterator for Rows {
    type Item=Result<Value>;
    fn next(&mut self)->Option<Self::Item>{if self.finished{return None;}match self.advance(){Ok(Some(v))=>Some(Ok(v)),Ok(None)=>None,Err(e)=>{self.finished=true;Some(Err(e))}}}
}

/// Reconstruct the compact counter view from the ordered stream. This is a
/// native-only decomposition; no source clock or numeric matching is involved.
pub fn aggregate(v:&Value)->Result<Value>{
    type PathKey=Vec<u32>;
    let mut stack=Vec::<(u64,u32)>::new();let mut calls=std::collections::BTreeMap::<u64,(PathKey,u64)>::new();
    let mut sites=std::collections::BTreeMap::<(PathKey,String),[u64;2]>::new();
    let mut scalars=sites.clone();let mut model_sites=sites.clone();
    let mut blocks=std::collections::BTreeMap::<(PathKey,u32,u8,Option<(bool,bool)>),[u64;2]>::new();
    let mut models=std::collections::BTreeMap::<(PathKey,String,Option<u32>,bool),[u64;2]>::new();let mut total=0u64;
    for e in v["ordered_events"].as_array().ok_or("ordered aggregate events")?{
        match e[0].as_u64().ok_or("aggregate kind")?{
            0=>{let id=e[1].as_u64().ok_or("aggregate scope ID")?;let pc=e[2].as_u64().ok_or("aggregate scope PC")?as u32;stack.push((id,pc));
                if calls.insert(id,(stack.iter().map(|r|r.1).collect(),0)).is_some(){return Err("duplicate aggregate scope".into());}},
            1=>{if stack.pop()!=Some((e[1].as_u64().ok_or("aggregate exit")?,e[2].as_u64().ok_or("aggregate exit PC")?as u32)){return Err("aggregate scope nesting".into());}},
            2=>{let call=e[1].as_u64();if call!=stack.last().map(|r|r.0){return Err("aggregate charge owner".into());}
                let path:Vec<_>=stack.iter().map(|r|r.1).collect();let site=e[6].as_str().ok_or("aggregate site")?.to_string();let clocks=e[5].as_u64().ok_or("aggregate clocks")?;
                total+=clocks;calls.entry(call.unwrap_or(u64::MAX)).or_insert((path.clone(),0)).1+=clocks;
                let s=sites.entry((path.clone(),site.clone())).or_default();s[0]+=clocks;s[1]+=1;
                if let Some(pc)=e[2].as_u64(){let width=serde_json::from_value(e[4].clone())?;
                    let s=blocks.entry((path.clone(),pc as u32,e[3].as_u64().ok_or("marker kind")?as u8,width)).or_default();s[0]+=clocks;s[1]+=1;}
                if let Some(steps)=e[7].as_array(){let s=model_sites.entry((path.clone(),site.clone())).or_default();s[0]+=clocks;s[1]+=1;
                    for step in steps{let s=models.entry((path.clone(),site.clone(),step[0].as_u64().map(|v|v as u32),step[2].as_bool().ok_or("model child")?)).or_default();s[0]+=step[1].as_u64().ok_or("model work")?;s[1]+=1;}
                }else if e[2].is_null(){let s=scalars.entry((path,site)).or_default();s[0]+=clocks;s[1]+=1;}
            },
            _=>return Err("aggregate event kind".into()),
        }
    }
    if !stack.is_empty()||v["ledger_master"].as_u64()!=Some(total){return Err("native aggregate counter conservation".into());}
    let mut paths=std::collections::BTreeMap::<PathKey,Vec<u64>>::new();for(_, (path,work))in calls{paths.entry(path).or_default().push(work);}
    Ok(json!({"iteration":v["iteration"],"ledger_master":total,"paths":paths.into_iter().collect::<Vec<_>>(),
        "sites":sites.into_iter().map(|((p,s),v)|(p,s,v)).collect::<Vec<_>>(),
        "scalar_sites":scalars.into_iter().map(|((p,s),v)|(p,s,v)).collect::<Vec<_>>(),
        "model_sites":model_sites.into_iter().map(|((p,s),v)|(p,s,v)).collect::<Vec<_>>(),
        "blocks":blocks.into_iter().map(|((p,pc,k,w),v)|(p,pc,k,w,v)).collect::<Vec<_>>(),
        "model_steps":models.into_iter().map(|((p,s,pc,c),v)|(p,s,pc,c,v)).collect::<Vec<_>>()}))
}
