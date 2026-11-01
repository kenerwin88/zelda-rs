//! Sparse, read-only projection of retained ordered source witnesses.
//! Usage: --coverage-source ORDERED.bin.zst COHORT.json FRESH.csv
//! COHORT contains {"rows":[logical iteration IDs]}. No CPU is run and no
//! oracle or ledger files are modified. Missing source rows remain explicit.
use std::{collections::{BTreeMap, BTreeSet}, error::Error, fs::File,
    io::{BufWriter, Read, Write}, path::Path};
use sha2::{Digest, Sha256};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn project(bytes: &[u8], count: u64, total: u64) -> Result<BTreeMap<(u32, u32, u8), [u64; 2]>> {
    if bytes.len() as u64 != count * 23 || count == 0 { return Err("invalid source row size".into()); }
    let records: Vec<_> = bytes.chunks_exact(23).collect();
    let long = |r: &[u8], at: usize| u32::from_le_bytes(r[at..at+4].try_into().unwrap());
    let sp = |r: &[u8]| u16::from_le_bytes(r[12..14].try_into().unwrap());
    if long(records[0], 0) != 0x8038 || long(records[records.len()-1], 0) != 0x805d {
        return Err("source row lacks main entry/return".into());
    }
    let mut stack = Vec::<(u32, u16)>::new();
    let mut rows = BTreeMap::<(u32, u32, u8), [u64; 2]>::new();
    for (j, r) in records.iter().enumerate() {
        let pc = long(r, 0) & 0x7fffff;
        let owner = stack.last().map_or(0x8034, |f| f.0);
        let row = rows.entry((owner, pc, r[22] & 3)).or_default();
        row[0] += 1;
        row[1] += u64::from(long(r, 8));
        if let Some(next) = records.get(j+1) {
            let target = long(r, 4) & 0x7fffff;
            if target != long(next, 0) & 0x7fffff { return Err("nonadjacent source instructions".into()); }
            let next_sp = sp(next);
            if matches!(r[21], 0x20 | 0x22 | 0xfc) {
                if sp(r).checked_sub(if r[21] == 0x22 { 3 } else { 2 }) != Some(next_sp) {
                    return Err("source call stack width".into());
                }
                stack.push((target, next_sp));
            }
            // Same exclusive ownership convention as the ledger profiler:
            // tail jumps retain their frame; real and double returns pop it.
            while stack.last().is_some_and(|frame| next_sp > frame.1) { stack.pop(); }
        }
    }
    if !stack.is_empty() || rows.values().map(|v| v[1]).sum::<u64>() != total {
        return Err("source coverage conservation/stack failure".into());
    }
    Ok(rows)
}

pub fn run(args: Vec<String>) -> Result<()> {
    if args.len() != 3 { return Err("--coverage-source ORDERED.bin.zst COHORT.json FRESH.csv".into()); }
    let cohort: serde_json::Value = serde_json::from_reader(File::open(&args[1])?)?;
    let selected: BTreeSet<u64> = cohort["rows"].as_array().ok_or("cohort rows")?.iter()
        .map(|v| v.as_u64().ok_or("cohort iteration")).collect::<std::result::Result<_, _>>()?;
    let mut source = zstd::Decoder::new(File::open(&args[0])?)?;
    let mut magic = [0; 8]; source.read_exact(&mut magic)?;
    if &magic != b"Z3M4OP01" { return Err("unsupported ordered source schema".into()); }
    let mut output = BufWriter::new(File::options().write(true).create_new(true).open(&args[2])?);
    writeln!(output, "iteration,routine,pc,m8,x8,count,master")?;
    let mut seen = BTreeSet::new();
    let mut expected = 0u64;
    let mut totals = Vec::new();
    loop {
        let mut header = [0; 24];
        if source.read(&mut header[..1])? == 0 { break; }
        source.read_exact(&mut header[1..])?;
        let word = |at| u64::from_le_bytes(header[at..at+8].try_into().unwrap());
        let (id, count, total) = (word(0), word(8), word(16));
        if id != expected || count > 2_000_000 { return Err("source row order/bound".into()); }
        expected += 1;
        if !selected.contains(&id) {
            if std::io::copy(&mut source.by_ref().take(count * 23), &mut std::io::sink())? != count * 23 {
                return Err("truncated unselected source row".into());
            }
            continue;
        }
        let mut bytes = vec![0; count as usize * 23]; source.read_exact(&mut bytes)?;
        let rows = project(&bytes, count, total)?;
        for ((owner, pc, width), [n, clocks]) in rows {
            writeln!(output, "{id},{owner:06x},{pc:06x},{},{},{n},{clocks}", width & 1, (width >> 1) & 1)?;
        }
        seen.insert(id);
        totals.push(serde_json::json!({"iteration":id,"instructions":count,"source_main":total,
            "ordered_row_sha256":format!("{:x}",Sha256::digest(&bytes)),"residual":0}));
    }
    output.flush()?;
    let receipt = serde_json::json!({"source":args[0],"source_rows":expected,"selected":selected.len(),
        "extracted":seen.len(),"missing":selected.difference(&seen).collect::<Vec<_>>(),"iterations":totals});
    let path = Path::new(&args[2]).with_extension("receipt.json");
    serde_json::to_writer_pretty(File::options().write(true).create_new(true).open(path)?, &receipt)?;
    println!("source rows={expected} selected={} extracted={} missing={}",selected.len(),seen.len(),selected.len()-seen.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(pc: u32, next: u32, sp: u16, opcode: u8) -> Vec<u8> {
        let mut b = vec![0; 23]; b[..4].copy_from_slice(&pc.to_le_bytes());
        b[4..8].copy_from_slice(&next.to_le_bytes()); b[8..12].copy_from_slice(&14u32.to_le_bytes());
        b[12..14].copy_from_slice(&sp.to_le_bytes()); b[21]=opcode; b[22]=3; b
    }
    #[test]
    fn children_and_tail_jumps_keep_exclusive_ownership() {
        let bytes = [record(0x8038,0x9000,0x1ff,0x20),record(0x9000,0x9010,0x1fd,0x4c),
            record(0x9010,0x803b,0x1fd,0x60),record(0x803b,0x805d,0x1ff,0x4c),
            record(0x805d,0x8034,0x1ff,0x80)].concat();
        let rows = project(&bytes,5,70).unwrap();
        assert_eq!(rows[&(0x9000,0x9010,3)],[1,14]);
        assert_eq!(rows[&(0x8034,0x803b,3)],[1,14]);
        assert!(project(&bytes,5,71).is_err());
        assert!(project(&bytes[..bytes.len()-1],5,70).is_err());
    }
}
