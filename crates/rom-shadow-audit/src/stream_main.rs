//! Bounded, lossless audit stream projections. No game or ROM execution.
use rom_shadow_audit::{Identity,Result};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,fs::File,io::{BufRead,BufReader,Read,Write},path::Path};
#[derive(Deserialize,Serialize)]
struct NativeOrder { iteration:u64,ledger_master:u64,ordered_events:serde_json::Value }
fn run(mode:&str,input:&Path,output:&Path)->Result<()> {
    if matches!(mode,"ownership-json"|"ownership-compact"){
        let names_path=input.parent().ok_or("ownership parent")?.join("ownership-names.json");
        let names:Vec<(u64,(String,String))>=serde_json::from_reader(File::open(names_path)?)?;
        for (j,n)in names.iter().enumerate(){if n.0!=j as u64{return Err("ownership dictionary sequence".into());}}
        let names:Vec<_>=names.into_iter().map(|v|v.1).collect();
        let mut input=zstd::Decoder::new(File::open(input)?)?;let mut magic=[0;8];input.read_exact(&mut magic)?;
        if &magic!=b"Z3M4OW01"{return Err("ownership schema".into());}
        let mut out:Box<dyn Write>=if output==Path::new("-"){Box::new(std::io::BufWriter::new(std::io::stdout()))}
            else{Box::new(zstd::Encoder::new(File::options().create_new(true).write(true).open(output)?,3)?.auto_finish())};
        let mut count=0;while let Some(row)=rom_shadow_audit::ownership::read_row(&mut input,&names,mode=="ownership-compact")?{
            if row["iteration"]!=count{return Err("ownership iteration order".into());}writeln!(out,"{row}")?;count+=1;
        }out.flush()?;return Ok(());
    }
    let identity=Identity::bind(input)?;let mut count=0u64;
    let mut out=zstd::Encoder::new(File::options().create_new(true).write(true).open(output)?,3)?;
    match mode {
        "native-order"=>{
            for line in BufReader::new(zstd::Decoder::new(File::open(input)?)?).lines(){
                let row:NativeOrder=serde_json::from_str(&line?)?;
                if row.iteration!=count{return Err("native projection iteration order".into());}
                serde_json::to_writer(&mut out,&row)?;writeln!(out)?;count+=1;
            }
        },
        "source-inventory"=>{
            let mut source=zstd::Decoder::new(File::open(input)?)?;let mut magic=[0;8];source.read_exact(&mut magic)?;
            if &magic!=b"Z3M4OP01"{return Err("source projection schema".into());}
            loop {
                let mut h=[0;24];if source.read(&mut h[..1])?==0{break;}source.read_exact(&mut h[1..])?;
                let u=|b:&[u8]|u64::from_le_bytes(b.try_into().unwrap());
                let (id,n,master)=(u(&h[..8]),u(&h[8..16]),u(&h[16..]));if id!=count||n>2_000_000{return Err("source projection row".into());}
                let mut pcs=BTreeMap::<(u32,u8),[u64;2]>::new();let mut digest=Sha256::new();let mut sum=0;
                for _ in 0..n {
                    let mut b=[0;23];source.read_exact(&mut b)?;
                    let pc=u32::from_le_bytes(b[..4].try_into().unwrap());let clocks=u64::from(u32::from_le_bytes(b[8..12].try_into().unwrap()));let width=b[22]&3;
                    let row=pcs.entry((pc,width)).or_default();row[0]+=1;row[1]+=clocks;sum+=clocks;
                    digest.update(pc.to_le_bytes());digest.update([width]);digest.update(clocks.to_le_bytes());
                }
                if sum!=master{return Err("source projection clock conservation".into());}
                let row=serde_json::json!({"iteration":id,"instructions":n,"source_master":master,
                    "ordered_sha256":format!("{:x}",digest.finalize()),"pc_work":pcs.into_iter().map(|((pc,w),v)|(pc,w,v)).collect::<Vec<_>>()});
                writeln!(out,"{row}")?;count+=1;
            }
        },
        _=>return Err("rom-audit-stream native-order|source-inventory INPUT FRESH_OUTPUT.zst".into()),
    }
    out.finish()?;identity.verify()?;
    let receipt=serde_json::json!({"input":identity,"output":Identity::bind(output)?,"rows":count,
        "binary":Identity::bind(&std::env::current_exe()?)?,"projection":mode});
    let mut f=File::options().create_new(true).write(true).open(output.with_extension("receipt.json"))?;
    serde_json::to_writer_pretty(&mut f,&receipt)?;Ok(())
}
fn main()->Result<()> {
    let a:Vec<_>=std::env::args().skip(1).collect();if a.len()!=3{return Err("rom-audit-stream MODE INPUT FRESH_OUTPUT.zst".into());}
    run(&a[0],Path::new(&a[1]),Path::new(&a[2]))
}
