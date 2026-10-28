//! Opt-in, write-only audit sink. No ROM access or feedback into game state.
//! This is a counter observer, not the ordered retirement observer of M2.
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
    SINK.with(|sink| if let Some(s) = sink.borrow_mut().as_mut() {
        assert!(!s.active, "nested shadow main cut");
        s.active = true;
        s.start = master;
        s.clocks = 0;
        s.charges = 0;
    });
}
pub(crate) fn charge(clocks: u64) {
    SINK.with(|sink| if let Some(s) = sink.borrow_mut().as_mut().filter(|s| s.active) {
        s.clocks += clocks;
        s.charges += 1;
    });
}
pub(crate) fn main_end(master: u64) {
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
    static ENTRIES: RefCell<Option<(std::path::PathBuf, [bool; 2])>> = RefCell::new(
        crate::debug_env::var_os("ZELDA3_SHADOW_AUDIT_ENTRIES")
            .map(|path| (std::path::PathBuf::from(path), [false; 2])));
}
/// Logical replay supplies identity only; this never alters seeding or inputs.
pub fn iteration(index: Option<u64>) { ITERATION.with(|i| i.set(index)); }
/// Capture at the actual native main cut, after its opening handler.
pub(crate) fn entry(wram: &[u8], sram: &[u8]) {
    ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        let Some((dir, seen)) = entries.as_mut() else { return; };
        if wram[0x11] != 0 { return; }
        let slot = match wram[0x10] { 7 => 0, 9 => 1, _ => return };
        if seen[slot] { return; }
        let Some(index) = ITERATION.with(std::cell::Cell::get) else { return; };
        seen[slot] = true;
        let mut file = File::options().write(true).create_new(true)
            .open(dir.join(format!("native-entry-{index}.wram"))).expect("fresh native entry output");
        file.write_all(wram).expect("native entry image");
        let mut file = File::options().write(true).create_new(true)
            .open(dir.join(format!("native-entry-{index}.sram"))).expect("fresh native SRAM output");
        file.write_all(sram).expect("native entry SRAM");
    });
}
