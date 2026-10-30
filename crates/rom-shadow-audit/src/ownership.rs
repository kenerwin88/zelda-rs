//! Lossless ordinal-run encoding. Global names avoid repeating long ambiguity
//! certificates on every iteration; clocks and instruction ordinals are exact.
use crate::Result;
use serde_json::{json,Value};
use std::{collections::BTreeMap,io::{Read,Write}};
pub type Names=BTreeMap<(String,String),u64>;
pub fn number(out:&mut impl Write,mut n:u64)->Result<()> {
    loop {let byte=(n&127)as u8;n>>=7;out.write_all(&[byte|if n!=0{128}else{0}])?;if n==0{return Ok(());}}
}
fn read_number(input:&mut impl Read)->Result<Option<u64>> {
    let mut n=0u64;
    for shift in (0..70).step_by(7){let mut b=[0];if input.read(&mut b)?==0{return if shift==0{Ok(None)}else{Err("truncated ownership integer".into())};}
        if shift==63&&b[0]>1{return Err("ownership integer overflow".into());}
        n|=u64::from(b[0]&127)<<shift;if b[0]&128==0{return Ok(Some(n));}}
    Err("ownership integer overflow".into())
}
pub fn write_row(out:&mut impl Write,iteration:u64,instructions:usize,
    names:&[(usize,(String,String))],runs:&[(usize,usize,usize,u64)],global:&mut Names)->Result<()> {
    number(out,iteration)?;number(out,instructions as u64)?;number(out,runs.len()as u64)?;
    let mut ids=Vec::with_capacity(names.len());for (id,name)in names{if *id!=ids.len(){return Err("ownership name order".into());}
        let next=global.len()as u64;ids.push(*global.entry(name.clone()).or_insert(next));}
    let mut cursor=0;for &(lo,hi,id,master)in runs{if lo!=cursor||hi<lo{return Err("ownership range gap".into());}
        number(out,(hi-lo+1)as u64)?;number(out,*ids.get(id).ok_or("ownership name missing")?)?;number(out,master)?;cursor=hi+1;}
    if cursor!=instructions{return Err("ownership incomplete row".into());}Ok(())
}
pub fn read_row(input:&mut impl Read,names:&[(String,String)],compact:bool)->Result<Option<Value>> {
    let Some(i)=read_number(input)?else{return Ok(None);};
    let mut required=||->Result<u64>{read_number(input)?.ok_or_else(||"truncated ownership row".into())};
    let instructions:u64=required()?;let count:u64=required()?;
    if count>instructions||instructions>2_000_000{return Err("ownership row bound".into());}
    let mut cursor=0;let mut local=BTreeMap::new();let mut owners=Vec::new();let mut runs=Vec::new();
    for _ in 0..count{let len=required()?;let id=required()?;let clocks=required()?;
        if len==0||len>instructions-cursor{return Err("ownership run overflow".into());}
        let name=names.get(id as usize).ok_or("ownership global name absent")?;
        let output_id=if compact{id}else{let next=local.len()as u64;*local.entry(id).or_insert_with(||{owners.push((next,name));next})};
        runs.push((cursor,cursor+len-1,output_id,clocks));cursor+=len;
    }
    if cursor!=instructions{return Err("ownership incomplete decoded row".into());}
    Ok(Some(if compact{json!({"iteration":i,"runs":runs,"source_instructions":instructions})}
        else{json!({"iteration":i,"owners":owners,"runs":runs,"source_instructions":instructions})}))
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn ordinal_runs_roundtrip_and_reject_truncation(){
        let names=vec![(0,("d".into(),"unknown scalar".into())),(1,("a".into(),"site.rs:1".into()))];
        let runs=vec![(0,100,0,4200),(101,101,1,14),(102,103,0,72)];let mut bytes=Vec::new();let mut global=Names::new();
        write_row(&mut bytes,1916,104,&names,&runs,&mut global).unwrap();
        let dictionary=names.iter().map(|x|x.1.clone()).collect::<Vec<_>>();
        let decoded=read_row(&mut bytes.as_slice(),&dictionary,false).unwrap().unwrap();
        assert_eq!(decoded,json!({"iteration":1916,"owners":names,"runs":runs,"source_instructions":104}));
        bytes.pop();assert!(read_row(&mut bytes.as_slice(),&dictionary,false).is_err());
        assert!(read_number(&mut [255u8;10].as_slice()).is_err());
    }
}
