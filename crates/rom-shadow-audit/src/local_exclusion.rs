//! Local exclusion is a boundary proof, not subtraction of inclusive totals.
//! Only the reviewed synchronous table-helper boundary is certified initially.
use super::*;

/// A source edit at the same line must not silently reuse the reviewed escape
/// proof. Re-pin only after reviewing the synchronous helper/caller closure.
const BOUNDARY_SOURCES:&[(&str,&str)] = &[
        ("crates/zelda3/src/dungeon/object_draw.rs","534ee7793fd1c109874864fce3b483be74514db39e44c034a33e5d9e25c11d75"),
        ("crates/zelda3/src/dungeon/objects.rs","88ae50e8f15ce2a270515ddc486dc4b66b36b3b08dae0b05b6e813fb7bca006c"),
        ("crates/zelda3/src/dungeon/room_load.rs","dd28016c1a93a3cd21353c9f2d53e8ccd309028e5ee2533cfbbdac0297da5840"),
        ("crates/zelda3/src/misc.rs","3dd8721ab8becd05cd053c0815661133e26f46cc0034316dff6150a65282b353"),
];
pub(super) fn verify_certificate_sources()->Result<()> { verify_certificate_sources_at(Path::new(".")) }
fn verify_certificate_sources_at(root:&Path)->Result<()> {
    verify_sources_at(root,BOUNDARY_SOURCES)
}
fn verify_sources_at(root:&Path,sources:&[(&str,&str)])->Result<()> {
    for &(path,expected) in sources {
        if Identity::bind(&root.join(path))?.sha256!=expected {
            return Err(format!("local exclusion certificate needs source review: {path}").into());
        }
    }
    Ok(())
}

/// Opaque ancestor scalars need a reviewed self-only instruction set. Keep the
/// list narrow; changing a callsite fails closed until its ownership is reviewed.
fn ancestor_scalar_disjoint(scope:&Scope, e:&Charge)->bool {
    let mut site=e.site.rsplitn(3,':');let _=site.next();
    let line=site.next().and_then(|s|s.parse::<u32>().ok());let file=site.next();
    match (scope.pc,file,line) {
        (0x80b5,Some("crates/zelda3/src/misc.rs"),Some(2664))=>e.clocks==240,
        // Room loader's own $873A call, $87xx/$8812..88E3 control;
        // object bodies are delegated, never included in these formulas.
        (0x1873a,Some("crates/zelda3/src/dungeon/room_load.rs"),Some(l))=>
            [328,543,670,675,678,682,696,706,711,730,736,747,752,755,758,761,763,1244].contains(&l),
        // Stream parser $88E4..893B, excluding dispatched object bodies.
        (0x188e4,Some("crates/zelda3/src/dungeon/room_load.rs"),Some(l))=>
            [1413,1422,1426,1429,1436,1448,1459,1464].contains(&l),
        // Subtype-1 decode/dispatch only: $893C..8980, no selected body.
        (0x1893c,Some("crates/zelda3/src/dungeon/room_load.rs"),Some(1488))=>e.clocks==652,
        (0x1893c,Some("crates/zelda3/src/dungeon/room_load.rs"),Some(1498))=>e.clocks==228,
        _=>false,
    }
}
fn helper_pc(pc:u32)->bool {(0x193f7..=0x19428).contains(&pc)}

/// Exhaustive frame-local predicate. The boundary certificate also establishes
/// synchronous ownership: stores/read helpers have no ledger calls, mute,
/// deferred task, publication or retained continuation. A new boundary requires
/// its own certificate; PC/name/cardinality agreement alone is insufficient.
pub(super) fn helper_gap(c:&Calls, rows:&[Instruction], n:&Native, mapping:&BTreeMap<u64,u64>,
    scope:&Scope, paired_source:&BTreeSet<usize>, paired_native:&BTreeSet<u64>) -> Option<Vec<usize>> {
    if !matches!(scope.pc,0x193f7|0x193ff)
        || !scope.site.starts_with("crates/zelda3/src/dungeon/object_draw.rs:222:") {return None;}
    let call=*mapping.get(&scope.id)?;let frame=&c.frames[call as usize];
    if frame.entry!=scope.pc || !frame.closed || frame.aliases.len()!=1
        || mapping.values().filter(|&&v|v==call).count()!=1 {return None;}
    // The actual observed synchronous caller must select this same source
    // parent. The parent's JMP alias is retained; its final fallthrough is
    // deliberately outside the helper scope and cannot create a helper call.
    let parent=n.scopes.iter().find(|s|Some(s.id)==scope.parent)?;
    if parent.pc!=0x193dc || !parent.site.starts_with("crates/zelda3/src/dungeon/objects.rs:1052:")
        || mapping.get(&parent.id).copied()!=frame.parent {return None;}
    if n.scopes.iter().any(|s|s.parent==Some(scope.id))
        || c.frames.iter().any(|f|f.parent==Some(call)) {return None;}
    let ordinals:Vec<_>=c.owners.iter().enumerate().filter_map(|(j,&id)|(id==call).then_some(j)).collect();
    if ordinals.is_empty() || ordinals.iter().any(|&j| {
        let r=&rows[j];!helper_pc(r.pc) || r.m8 || r.x8 || r.dp!=0 || r.emulation || r.interrupt_after
            || matches!(r.opcode,0x20|0x22|0xfc|0x4c|0x5c|0x6c|0x7c|0xdc|0x40)
    }) || rows[*ordinals.last()?].pc!=0x19428 {return None;}
    // Actual native loop argument, not routine-name agreement. Each $9408
    // source loop pass must have the same width as this native call.
    if n.local_widths.get(&scope.id).copied().filter(|&w|w>0)!=
        Some(u16::try_from(ordinals.iter().filter(|&&j|rows[j].pc==0x19408).count()).ok()?) {return None;}
    let by_id:BTreeMap<_,_>=n.scopes.iter().map(|s|(s.id,s)).collect();
    let mut ancestors=BTreeSet::new();let mut at=scope.parent;
    while let Some(id)=at {ancestors.insert(id);at=by_id.get(&id)?.parent;}
    let overlapping_blocks:BTreeSet<_>=generated::BLOCKS.iter().filter(|b|b.0<=0x19428 && b.1>0x193f7).map(|b|b.0).collect();
    for e in &n.charges {
        if e.call==Some(scope.id) {
            // Even a zero-clock unknown is not evidence of instruction pairing.
            if !paired_native.contains(&e.sequence) {return None;}
        } else {
            // Explicit PC claims outside the boundary must not claim its work.
            if e.pc.is_some_and(|pc|helper_pc(pc) || overlapping_blocks.contains(&pc)) || e.model.as_ref().is_some_and(|m|
                m.iter().any(|s|s.0.is_some_and(helper_pc))) {return None;}
            if let Some(owner)=e.call.and_then(|id|by_id.get(&id)) {
                if ancestors.contains(&owner.id) && !paired_native.contains(&e.sequence)
                    && e.model.as_ref().is_some_and(|m|m.is_empty() || m.iter().any(|s|s.0.is_none())
                        || m.iter().map(|s|u64::from(s.1)).sum::<u64>()!=e.clocks) {return None;}
                if ancestors.contains(&owner.id) && !paired_native.contains(&e.sequence)
                    && e.pc.is_none() && e.model.is_none() && !ancestor_scalar_disjoint(owner,e) {return None;}
            }
        }
    }
    Some(ordinals.into_iter().filter(|j|!paired_source.contains(j)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_edit_at_same_site_cannot_reuse_certificate() {
        let source=Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root=std::env::temp_dir().join(format!("shadow-exclusion-pins-{}-{}",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        for &(path,_) in BOUNDARY_SOURCES {
            let dest=root.join(path);fs::create_dir_all(dest.parent().unwrap()).unwrap();
            fs::copy(source.join(path),dest).unwrap();
        }
        // Main may have legitimately superseded the reviewed M3b source pins.
        // Test mutation detection with independent fixture bindings; never
        // renew the runtime certificate merely to make this test pass.
        let hashes:Vec<_>=BOUNDARY_SOURCES.iter().map(|(p,_)|Identity::bind(&root.join(p)).unwrap().sha256).collect();
        let pins:Vec<_>=BOUNDARY_SOURCES.iter().zip(&hashes).map(|((p,_),h)|(*p,h.as_str())).collect();
        assert_eq!(verify_certificate_sources_at(&root).is_ok(),pins==BOUNDARY_SOURCES);
        assert!(verify_sources_at(&root,&pins).is_ok());
        let path=root.join(BOUNDARY_SOURCES[0].0);
        let mut bytes=fs::read(&path).unwrap();bytes.extend_from_slice(b"\n// changed without moving the certified sites\n");fs::write(path,bytes).unwrap();
        assert!(verify_sources_at(&root,&pins).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    fn fixture()->(Calls,Vec<Instruction>,Native,BTreeMap<u64,u64>) {
        let c=Calls {frames:vec![
            Call{id:0,parent:None,entry:0x1893c,aliases:[0x1893c,0x193dc].into(),return_pc:0,return_sp:511,entry_sp:509,closed:true},
            Call{id:1,parent:Some(0),entry:0x193ff,aliases:[0x193ff].into(),return_pc:0x193e5,return_sp:509,entry_sp:507,closed:true}],
            owners:vec![1,1,1],rules:BTreeMap::new()};
        let rows=[(0x193ff,0xa5),(0x19408,0xbd),(0x19428,0x60)].into_iter().enumerate().map(|(j,(pc,opcode))|Instruction {
            iteration:0,ordinal:j as u64,pc,opcode,master:32,m8:false,x8:false,dp:0,db:0,sp:507,
            next_pc:pc+2,next_sp:507,interrupt_after:false,retired_next_pc:None,x:Some(0),y:Some(0),emulation:false}).collect();
        let n=Native {scopes:vec![Scope{id:10,pc:0x193dc,parent:None,site:"crates/zelda3/src/dungeon/objects.rs:1052:30".into()},
            Scope{id:11,pc:0x193ff,parent:Some(10),site:"crates/zelda3/src/dungeon/object_draw.rs:222:22".into()}],charges:vec![],total:Some(0),local_widths:[(11,1)].into()};
        (c,rows,n,[(10,0),(11,1)].into())
    }
    fn charge(call:u64,pc:Option<u32>)->Charge {
        Charge {call:Some(call),pc,kind:None,width:None,clocks:0,model:None,site:"unknown:1:1".into(),sequence:1,domain:"scalar_unmapped".into()}
    }
    #[test]
    fn closed_boundary_admits_only_source_only_opcodes() {
        let (c,rows,n,m)=fixture();
        assert_eq!(helper_gap(&c,&rows,&n,&m,&n.scopes[1],&[0].into(),&BTreeSet::new()),Some(vec![1,2]));
        assert!(helper_gap(&c,&rows,&n,&m,&n.scopes[1],&[0,1,2].into(),&BTreeSet::new()).unwrap().is_empty());
    }
    #[test]
    fn zero_subtree_unmatched_is_insufficient_counterexamples() {
        for case in 0..14 {
            let (mut c,mut rows,mut n,mut m)=fixture();
            match case {
                0=>{m.remove(&11);},
                1=>{m.insert(12,1);}, // nonunique mapping
                2=>{n.charges.push(charge(11,None));}, // unknown zero charge
                3=>{n.charges.push(charge(10,None));}, // escaping ancestor scalar
                4=>{n.charges.push(charge(10,Some(0x19408)));}, // claimed elsewhere
                5=>{c.frames[1].closed=false;},
                6=>{c.frames[1].aliases.insert(0x19408);}, // shared tail
                7=>{rows[0].interrupt_after=true;}, // unmapped retirement cut
                8=>{rows[0].m8=true;},
                9=>{rows[0].dp=1;},
                10=>{n.scopes.push(Scope{id:12,pc:0x9000,parent:Some(11),site:"child".into()});},
                11=>{n.local_widths.insert(11,2);},
                12=>{n.local_widths.clear();},
                _=>{c.frames[0].parent=Some(1);}, // hidden source child
            }
            assert!(helper_gap(&c,&rows,&n,&m,&n.scopes[1],&BTreeSet::new(),&BTreeSet::new()).is_none(),"case {case}");
        }
    }
    #[test]
    fn paired_scope_charge_and_reviewed_self_only_ancestor_are_distinct() {
        let (c,rows,mut n,m)=fixture();
        n.charges.push(charge(11,None));
        assert!(helper_gap(&c,&rows,&n,&m,&n.scopes[1],&BTreeSet::new(),&[1].into()).is_some());
        n.charges[0].call=Some(10); // paired elsewhere must still reject a PC escape
        n.charges[0].pc=Some(0x193ff);
        assert!(helper_gap(&c,&rows,&n,&m,&n.scopes[1],&BTreeSet::new(),&[1].into()).is_none());
        let scope=Scope{id:0,pc:0x1893c,parent:None,site:String::new()};
        let mut e=charge(0,None);e.site="crates/zelda3/src/dungeon/room_load.rs:1488:13".into();e.clocks=652;
        assert!(ancestor_scalar_disjoint(&scope,&e));e.clocks+=1;assert!(!ancestor_scalar_disjoint(&scope,&e));
    }
}
