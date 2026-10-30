//! Fail-closed contracts shared by the opt-in CPU auditor and its tests.
use std::{collections::BTreeMap, io::{BufRead, Read}, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[cfg(feature = "rom-shadow-audit")]
pub mod ordered;
#[cfg(feature = "rom-shadow-audit")]
pub mod accounting;
#[cfg(feature = "rom-shadow-audit")]
pub mod bridge;
#[cfg(feature = "rom-shadow-audit")]
pub mod diagnostic;
#[cfg(feature = "rom-shadow-audit")]
pub mod inventory;
#[cfg(feature = "rom-shadow-audit")]
pub mod ownership;
pub const CUTS: &str = "main:before-008038/after-00805d;label:008034;native:profile_main_begin/end";
pub const DOMAINS: &[&str] = &["main_instruction", "opening_nmi", "lag_nmi", "irq", "polygon", "dma", "hdma", "refresh", "idle", "poll_wait", "native_domain_unknown", "active_wall"];

pub fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
pub fn file_hash(path: &Path) -> Result<String> {
    let mut h = Sha256::new();
    let mut f = std::fs::File::open(path)?;
    let mut b = [0; 65536];
    loop { let n = f.read(&mut b)?; if n == 0 { break; } h.update(&b[..n]); }
    Ok(format!("{:x}", h.finalize()))
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Identity { pub path: PathBuf, pub bytes: u64, pub sha256: String }
impl Identity {
    pub fn bind(path: &Path) -> Result<Self> {
        Ok(Self { path: path.canonicalize()?, bytes: path.metadata()?.len(), sha256: file_hash(path)? })
    }
    pub fn verify(&self) -> Result<()> {
        if Self::bind(&self.path)? != *self { return Err(format!("identity changed: {}", self.path.display()).into()); }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32, pub commit: String, pub dirty_diff_sha256: String,
    pub features: Vec<String>, pub identities: BTreeMap<String, Identity>,
    pub cuts: String, pub clock_unit: String, pub domains: Vec<String>,
    pub seed_policy: String, pub source_checkpoint_schema: u8,
    pub completion: String,
}
impl Manifest {
    pub fn verify(&self) -> Result<()> {
        if self.schema != 1 || self.cuts != CUTS || self.clock_unit != "master_clocks" || self.source_checkpoint_schema != 9
            || self.domains != DOMAINS || self.identities.is_empty() {
            return Err("incompatible audit contract".into());
        }
        for identity in self.identities.values() { identity.verify()?; }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Residual {
    pub id: String, pub domain: String, pub reason: String,
    pub master_clocks: Option<i128>, pub first_iteration: Option<u64>, pub last_iteration: Option<u64>,
    pub source_pcs: Vec<u32>, pub native_candidates: Vec<String>, pub witness: Option<String>,
}
impl Residual {
    pub fn new(domain: &str, reason: &str, iteration: Option<u64>, clocks: Option<i128>) -> Self {
        let key = format!("{domain}/{reason}/{iteration:?}");
        Self { id: hash(key.as_bytes()), domain: domain.into(), reason: reason.into(), master_clocks: clocks,
            first_iteration: iteration, last_iteration: iteration, source_pcs: vec![], native_candidates: vec![], witness: None }
    }
}

/// Do not infer equality from a zero signed sum: every paired row must agree.
#[derive(Debug, Serialize, Deserialize)]
pub struct Conservation {
    pub source: u64, pub native: u64, pub paired_source: u64, pub paired_native: u64,
    pub unmatched_source: u64, pub unmatched_native: u64, pub signed_error: i128,
    pub unequal_pairs: usize,
}
impl Conservation {
    pub fn check(&self) -> Result<bool> {
        if self.paired_source.checked_add(self.unmatched_source) != Some(self.source)
            || self.paired_native.checked_add(self.unmatched_native) != Some(self.native)
            || i128::from(self.paired_source) - i128::from(self.paired_native) != self.signed_error
            || i128::from(self.source) - i128::from(self.native)
                != i128::from(self.unmatched_source) - i128::from(self.unmatched_native) + self.signed_error {
            return Err("conservation failure".into());
        }
        Ok(self.unmatched_source == 0 && self.unmatched_native == 0 && self.signed_error == 0 && self.unequal_pairs == 0)
    }
}

pub type Profile = BTreeMap<u64, BTreeMap<String, u64>>;
/// Strict exclusive CSV receipt reader; total is independently emitted by each producer.
/// VWF and owner zero are included. Inclusive rows are never added.
pub fn profile(reader: impl BufRead, lo: u64, hi: u64) -> Result<Profile> {
    if hi < lo { return Err("reversed profile selection".into()); }
    let mut lines = reader.lines();
    if lines.next().transpose()?.as_deref() != Some("iteration,routine,calls,inclusive,exclusive") {
        return Err("unexpected profile header".into());
    }
    let mut rows = Profile::new();
    for line in lines {
        let line = line?; let c: Vec<_> = line.split(',').collect();
        if c.len() != 5 { return Err("malformed profile row".into()); }
        let i: u64 = c[0].parse()?;
        if !(lo..=hi).contains(&i) { continue; }
        let routine = match c[1] { "total" | "vwf" => c[1].into(), a => format!("{:06x}", u32::from_str_radix(a, 16)? & 0x7fffff) };
        let _: u64 = c[2].parse()?; let _: u64 = c[3].parse()?;
        if rows.entry(i).or_default().insert(routine, c[4].parse()?).is_some() { return Err(format!("duplicate profile row {i}").into()); }
    }
    for i in lo..=hi {
        let row = rows.get(&i).ok_or_else(|| format!("absent iteration {i}"))?;
        let total = row.get("total").ok_or_else(|| format!("absent total {i}"))?;
        let sum = row.iter().filter(|(r, _)| r.as_str() != "total")
            .try_fold(0u64, |sum, (_, v)| sum.checked_add(*v).ok_or("counter overflow"))?;
        if sum != *total { return Err(format!("exclusive sum {sum} != {total} at {i}").into()); }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn read(s: &str, lo: u64, hi: u64) -> Result<Profile> {
        profile(std::io::Cursor::new(format!("iteration,routine,calls,inclusive,exclusive\n{s}")), lo, hi)
    }
    #[test] fn exclusive_includes_zero_and_vwf() {
        let p = read("0,000000,0,3,3\n0,008034,1,8,5\n0,vwf,0,2,2\n0,total,1,10,10\n",0,0).unwrap();
        assert_eq!(p[&0]["total"],10);
    }
    #[test] fn rejects_absent_duplicate_and_missing_total() {
        for s in ["", "0,008034,1,5,5\n", "0,total,1,0,0\n0,total,1,0,0\n", "0,808034,1,0,0\n0,008034,1,0,0\n0,total,1,0,0\n"] {
            assert!(read(s,0,0).is_err());
        }
        assert!(read("0,total,1,0,0\n",0,1).is_err());
    }
    #[test] fn rejects_nonconservation() { assert!(read("0,008034,1,5,5\n0,total,1,6,6\n",0,0).is_err()); }
    #[test] fn cancellation_is_not_exact() {
        let c = Conservation {source:10,native:10,paired_source:10,paired_native:10,unmatched_source:0,unmatched_native:0,signed_error:0,unequal_pairs:2};
        assert!(!c.check().unwrap());
    }
    #[test] fn null_is_not_zero() {
        let r = Residual::new("active_wall", "unmeasured", None, None);
        assert!(serde_json::to_string(&r).unwrap().contains("\"master_clocks\":null"));
    }
    #[test] fn changed_identity_rejected() {
        let p = std::env::temp_dir().join(format!("shadow-identity-{}", std::process::id()));
        std::fs::write(&p,b"first").unwrap(); let id = Identity::bind(&p).unwrap();
        id.verify().unwrap(); std::fs::write(&p,b"other").unwrap(); assert!(id.verify().is_err());
        std::fs::remove_file(p).unwrap();
    }
}

/// Version 1 ordinary-entry import map. The retained CPU owns direct page,
/// live stack, cartridge decoder pointers and translated-only representations.
/// Based on logical_replay::PORT_REPRESENTATION; valid only at 008038 in 07/00
/// or 09/00. Resume boundaries deliberately have no admitted import map.
pub const ORDINARY_PRIVATE_V1: &[(usize,usize,&str)] = &[
    (0,0x200,"source direct page and stack"),
    (0x648,0x64b,"native APU echo and feature flags"),
    (0xfa8,0xfaa,"per-sprite draw scratch"),
    (0x1234,0x1980,"translated tile upload staging"),
    (0x1b00,0x1cd0,"relocated spotlight representation"),
    (0x1cdd,0x1ce0,"cartridge dialogue source pointer"),
    (0x1f00,0x2000,"source polygon stack and workspace"),
    (0xe800,0xf000,"polygon bitmap representation"),
    (0x1dba0,0x1dd80,"native relocated HDMA table"),
];
pub fn ordinary_regions(module:u8,sub:u8,pc:u32)->Result<Vec<std::ops::Range<usize>>> {
    if !matches!(module,7|9)||sub!=0||pc!=0x008038 {return Err("unmapped native retirement boundary".into());}
    let mut regions=Vec::new();let mut start=0;
    for &(lo,hi,_) in ORDINARY_PRIVATE_V1 {
        let lo=if module==9 && lo==0x1234 {0x1100}else{lo};
        if start<lo {regions.push(start..lo);}start=hi;
    }
    if start<0x20000 {regions.push(start..0x20000);}Ok(regions)
}
#[cfg(test)]
mod import_tests {
    use super::*;
    #[test] fn every_byte_has_exactly_one_owner() {
        for module in [7,9] {
            let ranges=ordinary_regions(module,0,0x8038).unwrap();
            for i in 0..0x20000 {
                let copies=ranges.iter().filter(|r|r.contains(&i)).count();
                let private=ORDINARY_PRIVATE_V1.iter().filter(|(lo,hi,_)|{let lo=if module==9 && *lo==0x1234 {0x1100}else{*lo};(lo..*hi).contains(&i)}).count();
                assert_eq!(copies+private,1,"offset {i:x}");
            }
        }
    }
    #[test] fn resumes_are_unmapped() {
        assert!(ordinary_regions(9,0,0x02a4b5).is_err());
        assert!(ordinary_regions(7,1,0x8038).is_err());
    }
}
