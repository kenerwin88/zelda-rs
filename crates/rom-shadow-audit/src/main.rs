//! CPU-only development entry point. Never initializes a renderer.
use rom_shadow_audit::{*, Result};
use std::{collections::BTreeMap, fs::{self, File}, io::{BufRead, BufReader, Read, Write, BufWriter}, path::{Path, PathBuf}, process::Command, time::Instant};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use snes::{Snes9xColdCpuExecutor as Cpu, SourceCpuAcceptedInterrupt as Interrupt, SourceCpuBusAccessKind as Access};

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut f = File::options().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut f,value)?; writeln!(f)?; Ok(())
}
fn git(args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git").args(args).output()?;
    if !out.status.success() { return Err("git identity failed".into()); } Ok(out.stdout)
}
fn bind_tree(path: &Path, prefix: &str, ids: &mut BTreeMap<String,Identity>) -> Result<()> {
    let mut entries = fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        if e.file_type()?.is_symlink() { return Err("symlink inside ledger identity tree".into()); }
        let key = format!("{prefix}/{}", e.file_name().to_string_lossy());
        if e.file_type()?.is_dir() { bind_tree(&e.path(), &key, ids)?; }
        else { ids.insert(key, Identity::bind(&e.path())?); }
    } Ok(())
}
fn freeze(rom: &Path,sram: &Path,input: &Path,ledger: &Path) -> Result<Manifest> {
    let mut identities = BTreeMap::new();
    for (name,path) in [("rom",rom),("sram",sram),("input",input),("binary", std::env::current_exe()?.as_path()),
        ("block_table",Path::new("crates/zelda3/src/rom_block_costs.rs")), ("block_entries",Path::new("scripts/rom_block_entries.json")),
        ("import_map_and_contract",Path::new("crates/rom-shadow-audit/src/lib.rs")),("auditor_source",Path::new("crates/rom-shadow-audit/src/main.rs")),
        ("ordered_pairing",Path::new("crates/rom-shadow-audit/src/ordered.rs")),("stream_accounting",Path::new("crates/rom-shadow-audit/src/accounting.rs")),
        ("source_executor",Path::new("crates/snes/src/cpu_synchronous_executor/source_cpu.rs"))] {
        identities.insert(name.into(),Identity::bind(path)?);
    }
    bind_tree(ledger,"full-v2",&mut identities)?;
    // Include untracked implementation in the dirty identity; git diff alone omits it.
    let mut dirty = git(&["diff","--binary","HEAD"])?;
    for p in git(&["ls-files","--others","--exclude-standard"])?.split(|b| *b == b'\n').filter(|p| !p.is_empty()) {
        dirty.extend(p); dirty.extend(fs::read(Path::new(std::str::from_utf8(p)?))?);
    }
    Ok(Manifest { schema:1,commit:String::from_utf8(git(&["rev-parse","HEAD"])?)?.trim().into(),dirty_diff_sha256:hash(&dirty),
        features:vec!["rom-shadow-audit".into(),"snes/rom-timing".into(),"snes/parity-debug".into()], identities,
        cuts:CUTS.into(), clock_unit:"master_clocks".into(),domains:DOMAINS.iter().map(|s|s.to_string()).collect(),
        seed_policy:"source reset once, no reseeds; native existing logical replay policy (separate receipt)".into(),source_checkpoint_schema:9,completion:"incomplete".into() })
}
struct Images { input: zstd::Decoder<'static, BufReader<File>>, image: Vec<u8> }
impl Images {
    fn new(path: &Path) -> Result<Self> { Ok(Self {input:zstd::Decoder::new(File::open(path)?)?,image:vec![0;0x20000]}) }
    fn next(&mut self) -> Result<&[u8]> {
        let mut xor = [0;0x20000]; self.input.read_exact(&mut xor)?;
        for (a,b) in self.image.iter_mut().zip(xor) { *a ^= b; } Ok(&self.image)
    }
}
#[derive(Default, Serialize, Deserialize, Debug)]
struct Iteration {
    index:u64, open_host:usize, close_host:usize, host_buttons:u16, joypad:Option<u16>, lag_nmis:u32, irqs:u32,
    rng:Vec<(u8,bool)>,apu_reads:Vec<(u8,u8)>,apu_writes:Vec<(u8,u8)>,
}
fn pc(cpu:&Cpu)->u32 { let c=&cpu.machine().snes().cpu; (u32::from(c.k)<<16)|u32::from(c.pc) }
fn entry_kind(module:u8,sub:u8,next_pc:u32)-> &'static str {
    if next_pc!=0x008038 {"resumed"}
    else if ordinary_regions(module,sub,next_pc).is_ok() {"ordinary_admitted"}
    else {"ordinary_unadmitted"}
}
fn checkpoint_hash(cpu:&Cpu)->Result<String> { Ok(hash(&bincode::serialize(&cpu.capture_quiescent_checkpoint()?)?)) }
fn entry(cpu:&Cpu)->Result<Value> {
    let s=cpu.machine().snes(); let c=&s.cpu;
    Ok(json!({"registers":c,"module":s.ram[0x10],"submodule":s.ram[0x11],"stack_page_sha256":hash(&s.ram[0x100..0x200]),"stack_page":s.ram[0x100..0x200],
        "wram_sha256":hash(&s.ram),"timestamp":cpu.machine().timestamp().master_cycles(),
        "raster":format!("{:?}",cpu.raster_position()),"dma":s.dma,
        "hardware_checkpoint_sha256":checkpoint_hash(cpu)?,"checkpoint_schema":9,
        "checkpoint_storage":"memory only; raw checkpoint contains ROM and is never persisted"}))
}
fn checkpoint_trial(cpu:&Cpu)->Result<Value> {
    let checkpoint=cpu.capture_quiescent_checkpoint()?;
    let mut a=Cpu::from_quiescent_checkpoint(checkpoint.clone())?;
    let mut receipts=Vec::with_capacity(256);
    for _ in 0..256 { receipts.push(a.step()?); }
    let end=checkpoint_hash(&a)?;
    // Restore the same retained owner, including APU/timeline/event cursor.
    a.restore_quiescent_checkpoint(checkpoint)?;
    for (i,expected) in receipts.iter().enumerate() {
        let actual=a.step()?;
        if actual != *expected { return Ok(json!({"equal":false,"first_step":i,"expected":format!("{expected:?}"),"actual":format!("{actual:?}")})); }
    }
    Ok(json!({"equal":checkpoint_hash(&a)?==end,"instructions":256,"receipt_fields":"all transactions, bus accesses, interrupt and timestamps","end_checkpoint_sha256":end}))
}
/// A bounded fork owns no host input: it stops if a second host is needed.
/// The original retained source owner is never stepped or mutated here.
fn native_trial(cpu:&Cpu,dir:&Path,index:u64)->Result<Value> {
    let wram=zstd::decode_all(File::open(dir.join(format!("native-entry-{index}.wram.zst")))?)?;
    let sram=zstd::decode_all(File::open(dir.join(format!("native-entry-{index}.sram.zst")))?)?;
    if wram.len()!=0x20000 {return Err("native WRAM length".into());}
    let regions=ordinary_regions(wram[0x10],wram[0x11],pc(cpu))?;
    let source=&cpu.machine().snes().ram;
    let raw_differences:Vec<_>=source.iter().zip(&wram).enumerate().filter(|(_, (a,b))|a!=b)
        .map(|(i,(a,b))|json!({"offset":i,"source":a,"native":b})).collect();
    let copied_differences=regions.iter().flat_map(|r|r.clone()).filter(|&i|source[i]!=wram[i]).count();
    let checkpoint=cpu.capture_quiescent_checkpoint()?;
    let mut reference=Cpu::from_quiescent_checkpoint(checkpoint.clone())?;
    // Retain only a bounded witness, then drop this diagnostic owner before
    // creating the imported fork. At most one diagnostic CPU is live.
    let mut reference_steps=Vec::new();
    for _ in 0..100_000 {
        let step=reference.step()?;
        let done=step.origin_pc==0x00805d || step.accepted_interrupt.is_some();
        reference_steps.push((step,reference.machine().snes().cpu.clone()));
        if done {break;}
    }
    let reference_end=checkpoint_hash(&reference)?;
    drop(reference);
    let mut fork=Cpu::from_quiescent_checkpoint(checkpoint)?;
    fork.rebase_private_timing_regions_at(cpu.machine().timestamp(),&cpu.machine().snes().cpu,
        &wram,&sram,&regions,&[0..sram.len()])?;
    let mut master=0u64;let mut path=Sha256::new();
    for (instruction,(expected,reference_registers)) in reference_steps.iter().enumerate() {
        let actual=fork.step()?;
        if *expected!=actual {
            return Ok(json!({"equal":false,"status":"first_precise_divergence","instruction":instruction,
                "reference_pc":expected.origin_pc,"fork_pc":actual.origin_pc,"reference_receipt":format!("{expected:?}"),"fork_receipt":format!("{actual:?}"),
                "reference_registers":reference_registers,"fork_registers":fork.machine().snes().cpu,
                "copied_ranges":regions,"raw_entry_differences":raw_differences,"copied_entry_differences":copied_differences}));
        }
        path.update(expected.origin_pc.to_le_bytes());path.update([expected.opcode]);
        master+=expected.transactions.iter().map(|t|u64::from(t.duration_master_cycles)).sum::<u64>();
        if expected.origin_pc==0x00805d {
            let equal=reference_end==checkpoint_hash(&fork)?;
            return Ok(json!({"equal":equal,"status":if equal {"ordinary_slice_equal"}else{"output_or_hardware_divergence"},
                "instructions":instruction+1,"master":master,"path_sha256":format!("{:x}",path.finalize()),
                "return_pc":pc(&fork),"all_receipt_fields_equal":true,"all_output_and_hardware_equal":equal,
                "copied_ranges":regions,"private_ranges":ORDINARY_PRIVATE_V1,"copied_entry_differences":copied_differences,
                "raw_entry_differences":raw_differences,"register_policy":"retained proved source entry; native has no CPU register image"}));
        }
        if expected.accepted_interrupt.is_some() {
            return Ok(json!({"equal":false,"status":"unmapped_resume_handoff","instruction":instruction,"pc":pc(&fork),
                "reason":"no admitted native continuation register/retirement map; do not import a fresh iteration image"}));
        }
    }
    Ok(json!({"equal":false,"status":"instruction_limit"}))
}
fn image_check(expected:&[u8],cpu:&Cpu,which:&str,index:u64,host:usize)->Option<Value> {
    let actual=&cpu.machine().snes().ram;
    expected.iter().zip(actual).position(|(a,b)|a!=b).map(|i|json!({"reason":"source_image_mismatch","image":which,"iteration":index,"host":host,
        "offset":i,"expected":expected[i],"actual":actual[i],"pc":pc(cpu),"registers":cpu.machine().snes().cpu}))
}
fn row_check(actual:&Iteration,expected:&Iteration)->Option<Value> {
    let a=serde_json::to_value(actual).unwrap(); let e=serde_json::to_value(expected).unwrap();
    a.as_object().unwrap().iter().find(|(k,v)| e.get(*k)!=Some(*v))
        .map(|(k,v)|json!({"reason":"source_row_mismatch","iteration":actual.index,"field":k,"expected":e[k],"actual":v}))
}
#[derive(Default,Serialize)]
struct Bench { hosts:u64,instructions:u64,seconds:f64,first_host:usize,last_host:usize }

fn tree_bytes(root:&Path)->Result<u64> {
    let mut total=0u64;
    for entry in fs::read_dir(root)? {
        let entry=entry?;let meta=entry.metadata()?;
        total=total.checked_add(if entry.file_type()?.is_symlink(){0}else if meta.is_dir(){tree_bytes(&entry.path())?}else{meta.len()}).ok_or("disk counter overflow")?;
    }Ok(total)
}

fn source(args:&[String])->Result<()> {
    if !(6..=9).contains(&args.len()) { return Err("source ROM SRAM INPUT FULL_V2 HOSTS FRESH_OUT [NATIVE_ENTRIES|- [WITNESS_HOSTS|- [PREDICTOR]]]".into()); }
    let (rom,sram,input,ledger)=(Path::new(&args[0]),Path::new(&args[1]),Path::new(&args[2]),Path::new(&args[3]));
    let hosts:usize=args[4].parse()?; if hosts==0 { return Err("zero hosts".into()); }
    let native_dir=args.get(6).filter(|s|s.as_str()!="-").map(PathBuf::from);
    let witness_hosts:Vec<usize>=args.get(7).filter(|s|s.as_str()!="-").map(|s|s.split(',').map(str::parse).collect()).transpose()?.unwrap_or_default();
    if witness_hosts.len()>8 {return Err("at most eight witness hosts".into());}
    let out=PathBuf::from(&args[5]); fs::create_dir(&out)?;
    let mut manifest=freeze(rom,sram,input,ledger)?;
    let mut selected=std::collections::BTreeSet::new();
    if let Some(predictor)=args.get(8) {
        manifest.identities.insert("missing_predictor".into(),Identity::bind(Path::new(predictor))?);
        let mut lines=BufReader::new(File::open(predictor)?).lines();
        let h=lines.next().ok_or("missing predictor header")??;
        let h:Vec<_>=h.split(',').collect();
        let index=h.iter().position(|s|*s=="iteration").ok_or("predictor index")?;
        let lag=h.iter().position(|s|*s=="lag_nmis").ok_or("predictor lag")?;
        let held=h.iter().position(|s|*s=="native_held").ok_or("predictor held")?;
        for line in lines {
            let line=line?;let c:Vec<_>=line.split(',').collect();
            if c.len()!=h.len(){return Err("predictor row".into());}
            if c[lag].parse::<u64>()?>c[held].parse::<u64>()? {selected.insert(c[index].parse::<u64>()?);}
        }
    }
    if let Some(dir)=&native_dir {
        bind_tree(dir,"native_entries",&mut manifest.identities)?;
        let provenance:Value=serde_json::from_reader(File::open(dir.join("provenance.json"))?)?;
        for key in ["binary","stdout","predictor"] {
            let path=provenance[key].as_str().ok_or("native provenance path absent")?;
            let identity=Identity::bind(Path::new(path))?;
            if provenance[format!("{key}_sha256")]!=identity.sha256 {return Err("native provenance identity mismatch".into());}
            manifest.identities.insert(format!("native_{key}"),identity);
        }
    }
    write_json(&out.join("manifest.json"),&manifest)?;
    let mut residuals=zstd::Encoder::new(File::create(out.join("residuals.jsonl.zst"))?,3)?;
    let mut summaries=zstd::Encoder::new(File::create(out.join("iterations.jsonl.zst"))?,3)?;
    let mut entries=zstd::Encoder::new(File::create(out.join("entries.jsonl.zst"))?,3)?;
    let mut witness=zstd::Encoder::new(File::create(out.join("witness-trace.jsonl.zst"))?,3)?;
    let mut witness_rows=0;
    let mut blocks=zstd::Encoder::new(File::create(out.join("blocks.jsonl.zst"))?,3)?;
    let mut census=zstd::Encoder::new(File::create(out.join("entry-kinds.jsonl.zst"))?,3)?;
    let mut census_counts:BTreeMap<String,u64>=BTreeMap::new();
    let mut partitions=zstd::Encoder::new(File::create(out.join("source-instructions.jsonl.zst"))?,3)?;
    let mut buttons=vec![0u16;hosts]; let mut assigned=vec![false;hosts];
    for line in fs::read_to_string(input)?.lines().filter(|l|!l.is_empty()&&!l.starts_with('#')) {
        let (range,value)=line.split_once(' ').ok_or("invalid input")?;
        let (lo,hi)=range.split_once("..").unwrap_or((range,range)); let lo:usize=lo.parse()?;let hi:usize=hi.parse()?;
        if hi<lo {return Err("reversed input range".into());}
        for i in lo..=hi.min(hosts-1) { if assigned[i] {return Err("duplicate input host".into());} assigned[i]=true;buttons[i]=u16::from_str_radix(value.trim_start_matches("0x"),16)?; }
    }
    let mut rows=BufReader::new(File::open(ledger.join("iterations.jsonl"))?).lines();
    let mut pre=Images::new(&ledger.join("wram-pre-nmi.xor.zst"))?;
    let mut post=Images::new(&ledger.join("wram-post-nmi.xor.zst"))?;
    let mut cpu=Cpu::from_lorom_reset_with_sram(&fs::read(rom)?,Some(&fs::read(sram)?))?;
    cpu.enable_frame_capture(false);
    let mut current:Option<Iteration>=None;let mut opening:Option<Iteration>=None;
    let mut next=0;let mut depth=0u32;let mut lag_handler=false;let mut checked=0u64;
    let mut failure=None;let mut benchmarks:BTreeMap<String,Bench>=BTreeMap::new();
    let mut main_active=false;let mut main_depth=0u32;let mut previous_sp=cpu.machine().snes().cpu.sp;
    let mut previous_registers=cpu.machine().snes().cpu.clone();
    let mut ordinary=false;
    let mut pending_ordinary:Option<(u64,usize,u32)>=None;
    let mut master=0u64;let mut count=0u64;let mut path=Sha256::new();
    let mut native_trials=0;let mut native_equal=0;let mut resume_cut:Option<Value>=None;
    let mut entry_seen=[false;256];let mut resumed_seen=false;let mut trial_count=0;
    let mut total_instructions=0;let mut completed_hosts=0;let mut prelude=std::collections::VecDeque::new();
    let run_started=Instant::now();
    for (host,button) in buttons.iter().copied().enumerate() {
        cpu.set_libretro_joypad_words(button,0);
        let started=Instant::now();let mut host_count=0;let mut apu_upload=false;let mut polygon=false;let mut long_load=false;
        let result=cpu.run_until_main_loop_return_with_quiescent_step_hook(|step,exec|->Result<()> {
            host_count+=1; if host_count>1_000_000 {return Err("host instruction limit".into());}
            prelude.push_back(json!({"pc":step.origin_pc,"opcode":step.opcode,"start":step.started_at.master_cycles(),"end":step.ended_at.master_cycles()}));
            if prelude.len()>64 {prelude.pop_front();}
            if witness_hosts.contains(&host) {
                witness_rows+=1;if witness_rows>200_000 {return Err("witness row budget".into());}
                writeln!(witness,"{}",json!({"host":host,"pc":step.origin_pc,"start":step.started_at.master_cycles(),"end":step.ended_at.master_cycles(),
                    "line":exec.raster_position().scanline(),"irq":step.accepted_interrupt.is_some()}))?;
            }
            let snes=exec.machine().snes(); let module=snes.ram[0x10];polygon|=snes.ram[0x12a]!=0;
            if step.origin_pc==0x008038 && main_depth==0 {
                if let Some((index,entry_host,return_pc))=pending_ordinary.take() {
                    let kind=entry_kind(module,snes.ram[0x11],step.origin_pc);
                    ordinary=kind=="ordinary_admitted";
                    *census_counts.entry(format!("{kind}/{module:02x}/{:02x}",snes.ram[0x11])).or_default()+=1;
                    writeln!(census,"{}",json!({"iteration":index,"host":entry_host,"pc":step.origin_pc,
                        "handler_return_pc":return_pc,"entry_host":host,"module":module,"submodule":snes.ram[0x11],"kind":kind,
                        "status":"entry_kind_only_not_per_iteration_alignment"}))?;
                }
            }
            apu_upload|=(0x008888..0x008913).contains(&(step.origin_pc&0x7fffff));
            long_load|=module==6;
            // Capture BEFORE the next opcode, never reconstruct registers from WRAM.
            if pc(exec)==0x008038 && opening.is_none() && !entry_seen[usize::from(module)] && (!matches!(module,7|9) || snes.ram[0x11]==0) {
                entry_seen[usize::from(module)]=true;
                let i=current.as_ref().map(|r|r.index);
                writeln!(entries,"{}",json!({"kind":"ordinary_entry","iteration":i,"host":host,"module":module,"state":entry(exec)?}))?;
                if matches!(module,7|9) {
                    writeln!(entries,"{}",json!({"kind":"restore_reexecute","iteration":i,"result":checkpoint_trial(exec)?}))?;trial_count+=1;
                    fs::write(out.join(format!("entry-{}.wram",i.unwrap_or(0))),&snes.ram)?;
                    if let (Some(dir),Some(index))=(&native_dir,i) {
                        let result=native_trial(exec,dir,index)?;native_trials+=1;native_equal+=usize::from(result["equal"]==true);
                        writeln!(entries,"{}",json!({"kind":"native_seeded","iteration":index,"result":result}))?;
                    }
                }
            }
            if step.origin_pc==0x008038 { main_active=true;main_depth=0;master=0;count=0;path=Sha256::new(); }
            if main_active && main_depth==0 && previous_sp>>8==1 {
                let end=match step.accepted_interrupt {Some(Interrupt::Nmi{started_at,..}|Interrupt::Irq{started_at,..})=>started_at,None=>step.ended_at};
                let clocks:u64=step.transactions.iter().filter(|t|t.started_at<end).map(|t|u64::from(t.duration_master_cycles)).sum();
                master+=clocks;count+=1; path.update(step.origin_pc.to_le_bytes());path.update([step.opcode]);
                if ordinary && current.as_ref().is_some_and(|r|selected.contains(&r.index)) {
                    // One opcode is the smallest exclusive partition. Calls own
                    // their opcode only; callee work has separate rows. Dynamic
                    // call/return mapping to native scopes remains unproved.
                    writeln!(partitions,"{}",json!({"iteration":current.as_ref().unwrap().index,
                        "ordinal":count-1,"pc":step.origin_pc,"opcode":step.opcode,"master":clocks,
                        "m8":previous_registers.mf,"x8":previous_registers.xf,
                        "dp":previous_registers.dp,"db":previous_registers.db,"sp":previous_sp,
                        "next_pc":pc(exec),"next_sp":snes.cpu.sp,"interrupt_after":step.accepted_interrupt.is_some(),
                        "status":"source_partition_only"}))?;
                }
            }
            if step.origin_pc==0x00805d && main_active {
                writeln!(summaries,"{}",json!({"iteration":current.as_ref().map(|r|r.index),"host":host,"source_master":master,"source_instructions":count,
                    "path_sha256":format!("{:x}",path.clone().finalize()),"native_master":null,"alignment":"source_only","domains":"main_instruction; interrupt entry excluded"}))?;
                main_active=false;
            }
            if step.opcode==0x40 {main_depth=main_depth.saturating_sub(1);}
            if step.accepted_interrupt.is_some() {main_depth+=1;}
            previous_sp=snes.cpu.sp;
            previous_registers=snes.cpu.clone();
            let nmi=matches!(step.accepted_interrupt,Some(Interrupt::Nmi{..}));
            let stale=nmi&&opening.is_some();
            let returned=if step.opcode==0x40 {depth=depth.saturating_sub(1);depth==0} else {false};
            match step.accepted_interrupt {Some(Interrupt::Nmi{..})=>depth=1,Some(Interrupt::Irq{..})=>depth+=1,None=>{}}
            if stale||returned {
                lag_handler=false;
                if opening.is_some() {
                    let index=opening.as_ref().unwrap().index;
                    if let Some(w)=image_check(post.next()?,exec,"post",index,host) {failure=Some(w);return Err("source alignment stop".into());}
                    if let Some(finished)=current.take() {
                        let expected:Iteration=serde_json::from_str(&rows.next().ok_or("absent ledger row")??)?;
                        if expected.index!=checked {return Err("absent/duplicate/out-of-order ledger iteration".into());}
                        if let Some(w)=row_check(&finished,&expected) {failure=Some(w);return Err("source alignment stop".into());}
                        checked+=1;
                    }
                    current=opening.take();
                    let row=current.as_ref().unwrap();
                    let next_pc=pc(exec);
                    ordinary=false;
                    if matches!(next_pc,0x008034|0x008036|0x008038) && step.accepted_interrupt.is_none() {
                        // Handler return is not main entry: observe release from
                        // LDA $12 / BEQ $8034 to CLI $8038 before admitting it.
                        pending_ordinary=Some((row.index,host,next_pc));
                    } else {
                        pending_ordinary=None;
                        *census_counts.entry(format!("resumed/{module:02x}/{:02x}",snes.ram[0x11])).or_default()+=1;
                        writeln!(census,"{}",json!({"iteration":row.index,"host":host,"pc":next_pc,
                            "module":module,"submodule":snes.ram[0x11],"kind":"resumed","status":"unmapped_resume_handoff"}))?;
                    }
                    if step.accepted_interrupt.is_none() {return Ok(());}
                } else if main_active && !resumed_seen && depth==0 && matches!(module,7|9) {
                    resumed_seen=true;
                    let index=current.as_ref().map(|r|r.index);
                    resume_cut=Some(json!({"iteration":index,"host":host,"pc":pc(exec),"module":module,"submodule":snes.ram[0x11]}));
                    let mut r=Residual::new("main_instruction","native_resume_unmapped",index,None);
                    r.source_pcs.push(pc(exec));r.witness=Some("entries.jsonl.zst/resumed_entry".into());
                    writeln!(residuals,"{}",serde_json::to_string(&r)?)?;
                    writeln!(entries,"{}",json!({"kind":"resumed_entry","iteration":current.as_ref().map(|r|r.index),"host":host,"state":entry(exec)?,"restore_reexecute":checkpoint_trial(exec)?,"native_seeded":{"status":"unmapped_resume_handoff","source_pc":pc(exec),"reason":"native entry image and endpoint cannot prove continuation registers, live locals or retired-opcode cursor"}}))?;trial_count+=1;
                }
            }
            match step.accepted_interrupt {
                Some(Interrupt::Nmi{..}) if snes.ram[0x12]==0 => {
                    if let Some(w)=image_check(pre.next()?,exec,"pre",next,host) {failure=Some(w);return Err("source alignment stop".into());}
                    if let Some(row)=current.as_mut(){row.close_host=host;}
                    opening=Some(Iteration{index:next,open_host:host,host_buttons:button,..Default::default()});next+=1;return Ok(());
                }
                Some(Interrupt::Nmi{..})=>{if let Some(r)=current.as_mut(){r.lag_nmis+=1;}lag_handler=true;}
                Some(Interrupt::Irq{..})=>{if let Some(r)=current.as_mut(){r.irqs+=1;}}
                None=>{}
            }
            let in_open=opening.is_some();let Some(row)=opening.as_mut().or(current.as_mut()) else{return Ok(());};
            for a in &step.accesses {
                if (a.address>>16)&0x7f>=0x40 {continue;}
                let reg=a.address&0xffff;
                match a.kind {
                    Access::Read{value,width} if in_open && reg==0x4218 =>row.joypad=Some(if width>=2{value}else{(row.joypad.unwrap_or(0)&0xff00)|(value&255)}),
                    Access::Read{value,..} if in_open && reg==0x4219 =>row.joypad=Some((row.joypad.unwrap_or(0)&255)|((value&255)<<8)),
                    Access::Read{value,width} if (0x2140..=0x2143).contains(&reg)&&!in_open&&!lag_handler =>{
                        for byte in 0..u32::from(width).min(0x2144-reg){row.apu_reads.push(((reg+byte-0x2140)as u8,(value>>(8*byte))as u8));}
                    }
                    Access::Write{value,width}=>{
                        if (0x2140..=0x2143).contains(&reg){for byte in 0..u32::from(width).min(0x2144-reg){row.apu_writes.push(((reg+byte-0x2140)as u8,(value>>(8*byte))as u8));}}
                        if step.origin_pc&0x7fffff==0x0dba7c && reg==0xfa1 {row.rng.push((value as u8,snes.cpu.c));}
                    }
                    _=>{}
                }
            } Ok(())
        });
        let seconds=started.elapsed().as_secs_f64();total_instructions+=host_count;
        let s=cpu.machine().snes();
        let category=if host<81 {"reset"} else if polygon {"polygon"} else if apu_upload {"apu_upload"} else if long_load {"long_load"}
            else if matches!(s.ram[0x10],7|9)&&s.ram[0x11]==0 {"ordinary_gameplay"}else{"other"};
        let b=benchmarks.entry(category.into()).or_default();if b.hosts==0 {b.first_host=host;} b.last_host=host;b.hosts+=1;b.instructions+=host_count;b.seconds+=seconds;
        drop(cpu.take_dsp_samples());
        if let Err(e)=result {if failure.is_none(){failure=Some(json!({"reason":"source_execution_stop","host":host,"pc":pc(&cpu),"error":e.to_string()}));}break;}
        completed_hosts+=1;
        if host%1000==0 {
            if tree_bytes(out.parent().ok_or("output parent")?)? > 2_800_000_000 {
                failure=Some(json!({"reason":"incomplete_disk_budget","host":host}));break;
            }
            eprintln!("source host={host} checked={checked} instructions={total_instructions}");}
    }
    if let Some(w)=&failure {write_json(&out.join("first-divergence.json"),w)?;write_json(&out.join("prelude.json"),&prelude)?;}
    let reason=if failure.is_some(){"alignment_stop"}else{"bounded_source_prefix_native_handoff_unproved"};
    for domain in DOMAINS {
        let r=Residual::new(domain,if *domain=="main_instruction"{reason}else{"unpaired_domain"},Some(checked),None);
        writeln!(residuals,"{}",serde_json::to_string(&r)?)?;
    }
    writeln!(blocks,"{}",json!({"status":"unpaired","reason":reason,"source_master":null,"native_master":null}))?;
    residuals.finish()?;summaries.finish()?;entries.finish()?;blocks.finish()?;witness.finish()?;census.finish()?;partitions.finish()?;
    write_json(&out.join("entry-census.json"),&json!({"counts":census_counts,"checked_source_rows":checked,
        "note":"last entry may be unclosed; cross only indices below checked_source_rows","verified_eof":false}))?;
    manifest.verify()?;
    write_json(&out.join("coverage.json"),&json!({"pass":false,"m2_verdict":"STOP","reason":reason,"checked_source_rows":checked,"source_hosts":completed_hosts,
        "source_opcodes":total_instructions,"verified_eof":false,"all_route_iterations":1473123,"checkpoint_trials":trial_count,
        "native_seeded_trials":native_trials,"native_seeded_equal":native_equal,"first_unmapped_resume":resume_cut,
        "missing_iteration_intervals":[[checked,1473122]],"valid_source_prefix_end":checked.checked_sub(1),
        "native_seeded":"ordinary slices only; resumed handoff not admitted","seconds":run_started.elapsed().as_secs_f64()}))?;
    write_json(&out.join("benchmarks.json"),&benchmarks)?;
    write_json(&out.join("worklist.json"),&json!({"validated_positive_gaps":[],"negative_price_repairs":[],"admission":"STOP; no source/native pairing"}))?;
    Ok(())
}

fn profile_file(path:&Path,lo:u64,hi:u64)->Result<Profile> {
    if path.extension().is_some_and(|s|s=="zst") {
        profile(BufReader::new(zstd::Decoder::new(File::open(path)?)?),lo,hi)
    } else {profile(BufReader::new(File::open(path)?),lo,hi)}
}
fn receipts(args:&[String])->Result<()> {
    if args.len()!=5{return Err("receipts SOURCE_CSV NATIVE_CSV LO HI FRESH_OUT".into());}
    let lo=args[2].parse()?;let hi=args[3].parse()?;
    let source_identity=Identity::bind(Path::new(&args[0]))?;let native_identity=Identity::bind(Path::new(&args[1]))?;
    let s=profile_file(Path::new(&args[0]),lo,hi)?;
    let n=profile_file(Path::new(&args[1]),lo,hi)?;
    let out=Path::new(&args[4]);fs::create_dir(out)?;
    let mut rows=BufWriter::new(File::create(out.join("counter-reproduction.jsonl"))?);
    let mut residuals=zstd::Encoder::new(File::create(out.join("residuals.jsonl.zst"))?,3)?;
    for i in lo..=hi {
        // Routine labels alone are not retirement alignment. Preserve both unmatched sides.
        let c=Conservation{source:s[&i]["total"],native:n[&i]["total"],paired_source:0,paired_native:0,
            unmatched_source:s[&i]["total"],unmatched_native:n[&i]["total"],signed_error:0,unequal_pairs:0};
        c.check()?;
        for key in s[&i].keys().chain(n[&i].keys()).collect::<std::collections::BTreeSet<_>>() {
            if key=="total" {continue;}
            let delta=i128::from(*s[&i].get(key).unwrap_or(&0))-i128::from(*n[&i].get(key).unwrap_or(&0));
            if delta!=0 {
                let mut r=Residual::new("main_instruction",&format!("routine_ownership_unproved/{key}"),Some(i),Some(delta));
                r.native_candidates.push(key.clone());writeln!(residuals,"{}",serde_json::to_string(&r)?)?;
            }
        }
        writeln!(rows,"{}",json!({"iteration":i,"conservation":c,"alignment":"unproved","source_exclusive_sum_checked":true,"native_exclusive_sum_checked":true}))?;
    }
    write_json(&out.join("identity.json"),&json!({"source":source_identity,"native":native_identity,"lo":lo,"hi":hi,"cuts":CUTS}))?;
    residuals.finish()?;source_identity.verify()?;native_identity.verify()?;
    Ok(())
}

/// Cross only verified source rows. Predictor module fields describe the closing
/// image and must never be substituted for source entry module/substage.
fn census(args:&[String])->Result<()> {
    if args.len()!=3 {return Err("census SOURCE_OUT PREDICTOR FRESH_OUT".into());}
    let source=Path::new(&args[0]);let predictor=Path::new(&args[1]);let out=Path::new(&args[2]);
    let coverage:Value=serde_json::from_reader(File::open(source.join("coverage.json"))?)?;
    let checked=coverage["checked_source_rows"].as_u64().ok_or("source coverage absent")?;
    let bound=[Identity::bind(&source.join("entry-kinds.jsonl.zst"))?,Identity::bind(predictor)?,
        Identity::bind(&source.join("coverage.json"))?];
    let mut kinds=BTreeMap::new();
    for line in BufReader::new(zstd::Decoder::new(File::open(source.join("entry-kinds.jsonl.zst"))?)?).lines() {
        let row:Value=serde_json::from_str(&line?)?;let i=row["iteration"].as_u64().ok_or("entry index")?;
        if i>=checked {continue;}
        if kinds.insert(i,row).is_some(){return Err("duplicate source entry".into());}
    }
    if kinds.len() as u64!=checked || (0..checked).any(|i|!kinds.contains_key(&i)) {return Err("absent source entry".into());}
    let mut lines=BufReader::new(File::open(predictor)?).lines();
    let header=lines.next().ok_or("absent predictor header")??;
    let header:Vec<_>=header.split(',').collect();
    let col=|name:&str|->Result<usize>{header.iter().position(|s|*s==name).ok_or_else(||format!("absent predictor column {name}").into())};
    let (index,lag,held)=(col("iteration")?,col("lag_nmis")?,col("native_held")?);
    fs::create_dir(out)?;
    let mut missing=zstd::Encoder::new(File::create(out.join("missing-entry-kinds.jsonl.zst"))?,3)?;
    let mut table:BTreeMap<String,[u64;3]>=BTreeMap::new();
    let mut total=0u64;let mut missing_iterations=0u64;let mut missing_holds=0u64;
    for line in lines {
        let line=line?;let cells:Vec<_>=line.split(',').collect();
        if cells.len()!=header.len(){return Err("malformed predictor row".into());}
        let i:u64=cells[index].parse()?;if i!=total{return Err("predictor absent/duplicate/out of order".into());} total+=1;
        let lag:u64=cells[lag].parse()?;let held:u64=cells[held].parse()?;
        let deficit=lag.saturating_sub(held);
        let entry=kinds.get(&i);
        let key=entry.map(|r|format!("{}/{:02x}/{:02x}",r["kind"].as_str().unwrap(),r["module"].as_u64().unwrap(),r["submodule"].as_u64().unwrap()))
            .unwrap_or_else(||"unexecuted_source_suffix".into());
        let row=table.entry(key.clone()).or_default();row[0]+=1;row[1]+=u64::from(deficit>0);row[2]+=deficit;
        if deficit>0 {
            missing_iterations+=1;missing_holds+=deficit;
            writeln!(missing,"{}",json!({"iteration":i,"missing_holds":deficit,"entry_kind":key,"source_entry":entry}))?;
        }
    }
    missing.finish()?;
    if total<checked {return Err("predictor shorter than verified source prefix".into());}
    for id in &bound {id.verify()?;}
    write_json(&out.join("entry-kind-table.json"),&json!({"identities":bound,"source_checked":checked,
        "predictor_iterations":total,"full_predictor":total==1473123,"full_source":checked==1473123 && coverage["verified_eof"]==true,
        "columns":["iterations","missing_iterations","missing_holds"],"table":table,
        "missing_iterations":missing_iterations,"missing_holds":missing_holds}))?;
    write_json(&out.join("worklist.json"),&json!({"validated_positive_gaps":[],"negative_price_repairs":[],
        "admission":"STOP","reason":"entry census is not ordered call/site ownership proof; no prices admitted",
        "top_gaps_by_clocks":[],"top_gaps_by_missing_holds_covered":[]}))?;
    Ok(())
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("source")=>source(&args[1..]),Some("receipts")=>receipts(&args[1..]),Some("census")=>census(&args[1..]),
        Some("account") if args.len()==5=>accounting::reconcile(Path::new(&args[1]),Path::new(&args[2]),Path::new(&args[3]),Path::new(&args[4])),
        Some("account") if args.len()==7=>accounting::reconcile_window(Path::new(&args[1]),Path::new(&args[2]),Path::new(&args[3]),Path::new(&args[4]),args[5].parse()?,args[6].parse()?),
        Some("verify") if args.len()==2=>serde_json::from_reader::<_,Manifest>(File::open(&args[1])?)?.verify(),
        _=>Err("rom-shadow-audit: source | receipts | verify (explicit CPU-only commands required)".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn census_does_not_admit_other_substages_or_resumes() {
        assert_eq!(entry_kind(7,0,0x8038),"ordinary_admitted");
        assert_eq!(entry_kind(9,0,0x8038),"ordinary_admitted");
        assert_eq!(entry_kind(7,15,0x8038),"ordinary_unadmitted");
        assert_eq!(entry_kind(6,0,0x8038),"ordinary_unadmitted");
        assert_eq!(entry_kind(7,0,0xda95b),"resumed");
        assert_eq!(entry_kind(9,0,0x2a4b5),"resumed");
    }
    #[test] fn rejects_event_sequence_and_host_misalignment() {
        let a=Iteration{index:3,open_host:12,apu_reads:vec![(0,2),(0,3)],..Default::default()};
        let mut b=Iteration{index:3,open_host:12,apu_reads:vec![(0,2),(0,3)],..Default::default()};
        assert!(row_check(&a,&b).is_none());
        b.apu_reads.reverse();assert_eq!(row_check(&a,&b).unwrap()["field"],"apu_reads");
        b.apu_reads=a.apu_reads.clone();b.open_host=13;assert_eq!(row_check(&a,&b).unwrap()["field"],"open_host");
    }
    #[test] fn truncated_image_is_not_zero_work() {
        let p=std::env::temp_dir().join(format!("shadow-image-{}",std::process::id()));
        let compressed=zstd::encode_all(&b"short"[..],1).unwrap();fs::write(&p,compressed).unwrap();
        assert!(Images::new(&p).unwrap().next().is_err());fs::remove_file(p).unwrap();
    }
}
