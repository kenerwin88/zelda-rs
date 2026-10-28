//! Opt-in, write-only audit sink. No ROM access or feedback into game state.
//! Ordered metadata never feeds back into the ledger or scheduling.
use std::{cell::RefCell, fs::File, io::{BufWriter, Write}};

struct Sink {
    out: BufWriter<File>,
    active: bool,
    start: u64,
    clocks: u64,
    charges: u64,
    ordinal: u64,
}
thread_local! {
    static SINK: RefCell<Option<Sink>> = RefCell::new(
        crate::debug_env::var_os("ZELDA3_SHADOW_AUDIT_COUNTERS").map(|path| Sink {
            out: BufWriter::new(File::options().write(true).create_new(true).open(path)
                .expect("fresh shadow counter output")),
            active: false, start: 0, clocks: 0, charges: 0, ordinal: 0,
        }));
}
pub(crate) fn main_begin(master: u64) {
    ordered_begin();
    SINK.with(|sink| if let Some(s) = sink.borrow_mut().as_mut() {
        assert!(!s.active, "nested shadow main cut");
        s.active = true;
        s.start = master;
        s.clocks = 0;
        s.charges = 0;
    });
}
#[track_caller]
pub(crate) fn charge(clocks: u64) {
    ordered_charge(clocks, std::panic::Location::caller());
    SINK.with(|sink| if let Some(s) = sink.borrow_mut().as_mut().filter(|s| s.active) {
        s.clocks += clocks;
        s.charges += 1;
    });
}
pub(crate) fn main_end(master: u64) {
    ordered_end();
    SINK.with(|sink| if let Some(s) = sink.borrow_mut().as_mut().filter(|s| s.active) {
        let independent = master - s.start;
        assert_eq!(s.clocks, independent, "shadow exclusive charge conservation");
        writeln!(s.out, "{{\"ordinal\":{},\"charges\":{},\"observed_master\":{},\"independent_master\":{},\"domain\":\"native_ledger_main\"}}",
            s.ordinal, s.charges, s.clocks, independent).expect("shadow counters");
        s.out.flush().expect("shadow counter flush");
        s.ordinal += 1;
        s.active = false;
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn observer_cannot_charge_or_unmute_the_ledger() {
        let path = std::env::temp_dir().join(format!("shadow-counter-test-{}", std::process::id()));
        super::SINK.with(|sink| *sink.borrow_mut() = Some(super::Sink {
            out: std::io::BufWriter::new(std::fs::File::create(&path).unwrap()),
            active: false, start: 0, clocks: 0, charges: 0, ordinal: 0,
        }));
        let before = crate::cycle_ledger::master();
        super::main_begin(before);
        crate::cycle_ledger::charge(17);
        crate::cycle_ledger::muted(|| crate::cycle_ledger::charge(999));
        crate::cycle_ledger::charge(0);
        super::main_end(crate::cycle_ledger::master());
        assert_eq!(crate::cycle_ledger::master(), before + 17);
        let row = std::fs::read_to_string(&path).unwrap();
        assert!(row.contains("\"charges\":2"));
        assert!(row.contains("\"observed_master\":17,\"independent_master\":17"));
        super::SINK.with(|sink| *sink.borrow_mut() = None);
        std::fs::remove_file(path).unwrap();
    }
}

thread_local! {
    static ITERATION: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
    static ENTRIES: RefCell<Option<(std::path::PathBuf, [bool; 2], Option<std::collections::BTreeSet<u64>>)>> = RefCell::new(
        crate::debug_env::var_os("ZELDA3_SHADOW_AUDIT_ENTRIES")
            .map(|path| (std::path::PathBuf::from(path), [false; 2],
                crate::debug_env::var("ZELDA3_SHADOW_AUDIT_ITERATIONS").ok()
                    .map(|v|v.split(',').map(|s|s.parse().expect("entry iteration ID")).collect()))));
}
/// Logical replay supplies identity only; this never alters seeding or inputs.
pub fn iteration(index: Option<u64>) { ITERATION.with(|i| i.set(index)); }
/// Capture at the actual native main cut, after its opening handler.
pub(crate) fn entry(wram: &[u8], sram: &[u8]) {
    ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        let Some((dir, seen, selected)) = entries.as_mut() else { return; };
        if wram[0x11] != 0 { return; }
        let slot = match wram[0x10] { 7 => 0, 9 => 1, _ => return };
        let Some(index) = ITERATION.with(std::cell::Cell::get) else { return; };
        if let Some(selected)=selected {
            if !selected.contains(&index) {return;}
        } else if seen[slot] { return; }
        seen[slot] = true;
        let mut file = File::options().write(true).create_new(true)
            .open(dir.join(format!("native-entry-{index}.wram"))).expect("fresh native entry output");
        file.write_all(wram).expect("native entry image");
        let mut file = File::options().write(true).create_new(true)
            .open(dir.join(format!("native-entry-{index}.sram"))).expect("fresh native SRAM output");
        file.write_all(sram).expect("native entry SRAM");
    });
}

// This stack is observer-owned. Native scope IDs are not asserted to be ROM
// call IDs: flattened call paths require a separately proved mapping.
struct Ordered {
    out: BufWriter<Box<dyn Write>>, active: bool, sequence: u64, next_call: u64,
    calls: Vec<(u64,u32)>, clocks: u64,
    marker: Option<(u32,u8,Option<(bool,bool)>, &'static std::panic::Location<'static>)>,
    model: Option<Vec<(Option<u32>, u32, bool)>>,
    range: (u64,u64),
    selected: Option<std::collections::BTreeSet<u64>>,
}
fn event_writer(path:std::path::PathBuf)->BufWriter<Box<dyn Write>> {
    let file=File::options().write(true).create_new(true).open(&path).expect("fresh ordered audit");
    let writer:Box<dyn Write>=if path.extension().is_some_and(|s|s=="zst") {
        Box::new(zstd::Encoder::new(file,3).expect("ordered compressor").auto_finish())
    } else {Box::new(file)};
    BufWriter::new(writer)
}
thread_local! {
    static ORDERED: RefCell<Option<Ordered>> = RefCell::new(
        crate::debug_env::var_os("ZELDA3_SHADOW_AUDIT_EVENTS").map(|path| {
            let range=crate::debug_env::var("ZELDA3_SHADOW_AUDIT_RANGE").ok()
                .map(|v| {let (lo,hi)=v.split_once('-').expect("shadow range LO-HI");
                    (lo.parse().expect("shadow lower bound"),hi.parse().expect("shadow upper bound"))})
                .expect("ordered audit requires a bounded iteration range");
            assert!(range.0<=range.1);
            let selected=crate::debug_env::var("ZELDA3_SHADOW_AUDIT_ITERATIONS").ok()
                .map(|v|v.split(',').map(|s|s.parse().expect("shadow iteration ID")).collect());
            Ordered {out:event_writer(std::path::PathBuf::from(path)),
                active:false,sequence:0,next_call:0,calls:Vec::new(),clocks:0,marker:None,model:None,range,selected}
        }));
}
fn emit(s:&mut Ordered,value:serde_json::Value) {
    writeln!(s.out,"{value}").expect("ordered audit write");
    s.sequence+=1;
}
fn ordered_begin() {
    ORDERED.with(|o| if let Some(s)=o.borrow_mut().as_mut() {
        let i=ITERATION.with(std::cell::Cell::get);
        s.active=i.is_some_and(|i| (s.range.0..=s.range.1).contains(&i) && s.selected.as_ref().is_none_or(|set|set.contains(&i)));
        s.clocks=0;s.sequence=0;s.marker=None;s.model=None;
        if s.active {emit(s,serde_json::json!({"event":"begin","iteration":i,"calls":s.calls,
            "call_mapping":"native_scope_only_unproved","entry_alignment":"requires_source_join"}));}
    });
}
fn ordered_end() {
    ORDERED.with(|o| if let Some(s)=o.borrow_mut().as_mut().filter(|s|s.active) {
        assert!(s.marker.is_none() && s.model.is_none());
        emit(s,serde_json::json!({"event":"end","iteration":ITERATION.with(std::cell::Cell::get),
            "sequence":s.sequence,"ledger_master":s.clocks,"vwf":"separate_predictor_domain"}));
        s.out.flush().expect("ordered audit flush");s.active=false;
    });
}
pub(crate) fn marker(pc:u32,kind:u8,width:Option<(bool,bool)>,site:&'static std::panic::Location<'static>) {
    ORDERED.with(|o| if let Some(s)=o.borrow_mut().as_mut().filter(|s|s.active) {
        assert!(s.marker.replace((pc,kind,width,site)).is_none());
    });
}
fn ordered_charge(clocks:u64,site:&'static std::panic::Location<'static>) {
    ORDERED.with(|o| if let Some(s)=o.borrow_mut().as_mut().filter(|s|s.active) {
        let marker=s.marker.take();
        let model=s.model.take();
        let site=marker.map_or(site,|m|m.3);
        s.clocks+=clocks;
        emit(s,serde_json::json!({"event":"charge","iteration":ITERATION.with(std::cell::Cell::get),"sequence":s.sequence,
            "clocks":clocks,"call":s.calls.last(),"path":s.calls,
            "model_instructions":model,"pc":marker.map(|m|m.0),"kind":marker.map(|m|m.1),"declared_width":marker.and_then(|m|m.2),
            "domain":if marker.is_some(){"marker_instruction"}else{"scalar_unmapped"},
            "site":format!("{}:{}:{}",site.file(),site.line(),site.column())}));
    });
}
#[track_caller]
pub(crate) fn scope_enter(pc:u32) {
    scope_enter_with_width(pc,None);
}
#[track_caller]
fn scope_enter_with_width(pc:u32,local_width:Option<u16>) {
    let site=std::panic::Location::caller();
    ORDERED.with(|o| if let Some(s)=o.borrow_mut().as_mut() {
        let call=s.next_call;s.next_call+=1;s.calls.push((call,pc));
        if s.active {
            emit(s,serde_json::json!({"event":"enter","sequence":s.sequence,"call":call,"pc":pc,
                "site":format!("{}:{}:{}",site.file(),site.line(),site.column()),"local_width":local_width}));}
    });
}
pub(crate) fn scope_exit(pc:u32) {
    ORDERED.with(|o| if let Some(s)=o.borrow_mut().as_mut() {
        let call=s.calls.pop().expect("observer scope stack");assert_eq!(call.1,pc);
        if s.active {emit(s,serde_json::json!({"event":"exit","sequence":s.sequence,"call":call}));}
    });
}

/// Observe an otherwise unannotated boundary without opening a ledger scope,
/// changing profiling, or unmuting preview work. This guard owns metadata only.
pub(crate) struct LocalScope(Option<u32>);
#[track_caller]
pub(crate) fn local_scope(pc:u32,width:Option<u16>) -> LocalScope {
    if crate::cycle_ledger::is_muted() { return LocalScope(None); }
    scope_enter_with_width(pc,width);
    LocalScope(Some(pc))
}
impl Drop for LocalScope {
    fn drop(&mut self) { if let Some(pc)=self.0 { scope_exit(pc); } }
}

#[cfg(test)]
mod ordered_tests {
    #[test]
    fn ordered_observation_preserves_sites_and_muting() {
        let path=std::env::temp_dir().join(format!("shadow-ordered-test-{}",std::process::id()));
        super::ORDERED.with(|o|*o.borrow_mut()=Some(super::Ordered {
            out:std::io::BufWriter::new(Box::new(std::fs::File::create(&path).unwrap())),active:false,
            sequence:0,next_call:0,calls:Vec::new(),clocks:0,marker:None,model:None,range:(0,42),selected:Some([42].into_iter().collect()),
        }));
        super::iteration(Some(41));
        super::main_begin(crate::cycle_ledger::master());
        crate::cycle_ledger::charge(3);
        super::main_end(crate::cycle_ledger::master());
        super::iteration(Some(42));
        let before=crate::cycle_ledger::master();super::main_begin(before);
        {
            let _scope=crate::cycle_ledger::routine(0x008034);
            crate::cycle_ledger::blocks(true).block(0x008781);
            super::model(vec![(Some(0xe843),17,true)]);
            crate::cycle_ledger::charge(17);
            crate::cycle_ledger::muted(|| {
                let _local = super::local_scope(0x0193ff,None);
                let _scope=crate::cycle_ledger::routine(0x008781);
                crate::cycle_ledger::blocks(true).block(0x008781);
            });
        }
        super::main_end(crate::cycle_ledger::master());
        assert_eq!(crate::cycle_ledger::master()-before,69);
        let text=std::fs::read_to_string(&path).unwrap();
        let rows:Vec<serde_json::Value>=text.lines().map(|l|serde_json::from_str(l).unwrap()).collect();
        let charges:Vec<_>=rows.iter().filter(|r|r["event"]=="charge").collect();
        assert_eq!(charges.len(),2);
        assert_eq!(charges[0]["pc"],0x8781);
        assert_eq!(charges[0]["declared_width"],serde_json::json!([true,true]));
        assert!(charges[0]["site"].as_str().unwrap().contains("rom_shadow_audit.rs:"));
        assert_eq!(charges[1]["domain"],"scalar_unmapped");
        assert_eq!(charges[1]["model_instructions"],serde_json::json!([[0xe843,17,true]]));
        assert_eq!(charges[0]["call"],charges[1]["call"]);
        assert_eq!(rows.last().unwrap()["ledger_master"],69);
        super::ORDERED.with(|o|*o.borrow_mut()=None);super::iteration(None);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn compressed_event_writer_finishes_without_retaining_raw_events() {
        use std::io::Write;
        let path=std::env::temp_dir().join(format!("shadow-event-compressed-{}.zst",std::process::id()));
        {
            let mut writer=super::event_writer(path.clone());
            writeln!(writer,"{{\"event\":\"test\"}}").unwrap();
        }
        assert_eq!(zstd::decode_all(std::fs::File::open(&path).unwrap()).unwrap(),b"{\"event\":\"test\"}\n");
        std::fs::remove_file(path).unwrap();
    }
}

/// One existing charge's instruction/child witness, never an additional charge.
/// Occurrence is (iteration, charge sequence, instruction index) under the
/// recorded native scope path. Null PCs and unequal sums remain residuals.
pub(crate) fn model(steps: Vec<(Option<u32>, u32, bool)>) {
    if crate::cycle_ledger::is_muted() { return; }
    ORDERED.with(|o| if let Some(s)=o.borrow_mut().as_mut().filter(|s|s.active) {
        assert!(s.model.replace(steps).is_none(), "unconsumed graphics witness");
    });
}
pub(crate) fn observing() -> bool {
    !crate::cycle_ledger::is_muted() && ORDERED.with(|o|o.borrow().as_ref().is_some_and(|s|s.active))
}

#[cfg(test)]
mod m3a_serialization_tests {
    #[test]
    fn m3a_graphics_pc_witnesses_do_not_enter_serialized_state() {
        use crate::cycle_models::{graphics_convert,graphics_load,tilesets,decompress};
        let mut conversion=graphics_convert::ConversionWork::new(true,&[],1).step().unwrap();
        let bytes=bincode::serialize(&conversion).unwrap();conversion.pc=0;
        assert_eq!(bytes,bincode::serialize(&conversion).unwrap());
        let restored: graphics_convert::Instruction=bincode::deserialize(&bytes).unwrap();
        assert_eq!(restored.audit_pc(),None,"restored dev metadata cannot claim PC zero");
        let mut sheet=graphics_load::Instruction{master_cycles:24,routine:0xe583,publication:None,pc:Some(0xe583)};
        let bytes=bincode::serialize(&sheet).unwrap();sheet.pc=None;
        assert_eq!(bytes,bincode::serialize(&sheet).unwrap());
        let mut tileset=tilesets::Instruction{master_cycles:24,routine:0xe19b,publication:None,pc:Some(0xe19b)};
        let bytes=bincode::serialize(&tileset).unwrap();tileset.pc=None;
        assert_eq!(bytes,bincode::serialize(&tileset).unwrap());
        let mut decoder=decompress::DecompressInstruction{master_cycles:24,in_get_next_byte:false,action:None,pc:Some(0xe772)};
        let bytes=bincode::serialize(&decoder).unwrap();decoder.pc=None;
        assert_eq!(bytes,bincode::serialize(&decoder).unwrap());
    }
}
