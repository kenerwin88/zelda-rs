//! The cycle ledger: master cycles the original CPU would have spent in the
//! code the translated engine just ran.
//!
//! Translated routines charge the cost of the assembly they correspond to
//! (priced by `scripts/rom_block_costs.py` from the disassembly) as they
//! execute, so the ledger accumulates the frame's cost as a by-product of
//! running it. The ledger is thread-local: the engine runs one game per
//! thread, and a routine scope is a drop guard so early returns need no
//! bookkeeping. `ZELDA3_DEBUG_CYCLE_LEDGER=<dir>` records every annotated
//! routine's charge per host frame, which `scripts/cycle_ledger_check.py`
//! compares against the shadow-CPU profiles of the same hosts
//! (`ZELDA3_DEBUG_ROM_CPU_PROFILE`). See docs/parity/romless-exact-play.md.

use std::cell::{Cell, RefCell};
use std::io::Write;
use std::collections::BTreeMap;

#[path = "rom_block_costs.rs"]
mod rom_block_costs;

/// A source block marker is enabled only by the executing state's lagless
/// lane. Muted previews remain muted; default/ROM timing receives no charge.
pub struct BlockMarkers(bool);

pub fn blocks(lagless: bool) -> BlockMarkers { BlockMarkers(lagless) }

impl BlockMarkers {
    /// Static instruction work at this ROM block's documented entry M/X.
    #[inline]
    pub fn block(&self, address: u32) {
        if !self.0 || is_muted() { return; }
        let index = rom_block_costs::BLOCKS.binary_search_by_key(&address, |b| b.0)
            .expect("ROM block missing from generated table");
        self.retire(address, 0, rom_block_costs::BLOCKS[index].4);
    }

    /// Native branch outcome adds the 65816's internal taken cycle.
    pub fn taken(&self, pc: u32) {
        if !self.0 || is_muted() { return; }
        #[cfg(any(debug_assertions, test))] assert!(rom_block_costs::BLOCKS.iter().any(|b| b.5 == pc && pc != 0),
            "taken marker must name a generated conditional branch");
        self.retire(pc, 1, 6);
    }

    /// The executing native index, not a scene estimate, determines crossing.
    pub fn crossing(&self, pc: u32) {
        if !self.0 || is_muted() { return; }
        #[cfg(any(debug_assertions, test))] assert!(rom_block_costs::BLOCKS.iter().any(|b| b.6.contains(&pc)),
            "crossing marker must name a generated indexed read");
        self.retire(pc, 2, 6);
    }

    fn retire(&self, address: u32, kind: u8, clocks: u64) {
        charge(clocks);
        PROFILE.with(|profile| {
            if let Some(p) = profile.borrow_mut().as_mut().filter(|p| p.active) {
                let owner = LEDGER.with(|l| l.borrow().addresses.last().copied().unwrap_or(0));
                let row = p.blocks.entry((owner, address, kind)).or_default();
                row[0] += 1;
                row[1] += clocks;
            }
        });
        #[cfg(test)]
        MARKER_TRAIL.with(|trail| {
            if let Some(trail) = trail.borrow_mut().as_mut() {
                let owner = LEDGER.with(|l| l.borrow().addresses.last().copied().unwrap_or(0));
                trail.push((owner, address, kind, clocks));
            }
        });
    }
}

/// Take before the enclosing routine profile; kind 0=block, 1=taken, 2=crossing.
pub fn take_block_profile() -> BTreeMap<(u32, u32, u8), [u64; 2]> {
    PROFILE.with(|p| p.borrow_mut().as_mut()
        .map_or_else(BTreeMap::new, |p| std::mem::take(&mut p.blocks)))
}

/// Diagnostic work only; address zero is work outside every annotated scope.
#[derive(Default)]
struct RoutineProfile {
    blocks: BTreeMap<(u32, u32, u8), [u64; 2]>,
    active: bool,
    rows: BTreeMap<u32, [u64; 3]>, // calls, inclusive, exclusive
    fragment_path: Vec<u32>,
    fragment_depth: usize,
    object: Option<u16>,
    object_rows: BTreeMap<(u16, u32), [u64; 3]>,
    sprite: Option<(u8, u8, u8)>, // entry type, dispatch state, slot
    sprite_rows: BTreeMap<(u8, u8, u8, u32), [u64; 3]>,
}

#[derive(Debug, Default)]
struct Ledger {
    last_host_master: u64,
    last_host_silent_calls: u64,
    /// Charges recorded by nested scopes that have closed, per open scope
    /// depth, so a routine's record excludes its annotated callees (a self
    /// cost, comparable with the profiler's own-instruction totals).
    nested: Vec<u64>,
    addresses: Vec<u32>,
    /// Completed routine self charges this host frame: (ROM address, master).
    completed: Vec<(u32, u64)>,
}

thread_local! {
    #[cfg(test)]
    static MARKER_TRAIL: RefCell<Option<Vec<(u32, u32, u8, u64)>>> = const { RefCell::new(None) };
    /// The running total, a plain cell so a charge is one add on the hot
    /// path (annotated routines charge many times per frame).
    static MASTER: Cell<u64> = const { Cell::new(0) };
    /// Depth of `muted` regions: work the engine runs on a probe clone (a
    /// CPU-timing preview) is not work the original CPU did.
    static MUTED: Cell<u32> = const { Cell::new(0) };
    /// Routine scopes opened so far on this thread (an annotation census).
    static SCOPES_OPENED: Cell<u64> = const { Cell::new(0) };
    static HDMA_PROGRAM_EPOCH: Cell<u64> = const { Cell::new(0) };
    /// Probed calls that charged nothing and opened no scope: translated
    /// routines the ledger does not price yet (see `probe_annotation`).
    static SILENT_CALLS: Cell<u64> = const { Cell::new(0) };
    static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
    static PROFILE: RefCell<Option<RoutineProfile>> = const { RefCell::new(None) };
}

/// Enable collection for one logical iteration (the CLI owns the output).
pub fn profile_iteration(enabled: bool) {
    PROFILE.with(|profile| *profile.borrow_mut() = enabled.then(RoutineProfile::default));
}

pub(crate) fn profile_main_begin() {
    PROFILE.with(|profile| {
        if let Some(profile) = profile.borrow_mut().as_mut() {
            profile.active = true;
            LEDGER.with(|ledger| {
                for address in &ledger.borrow().addresses {
                    profile.rows.entry(*address).or_default()[0] += 1;
                }
            });
        }
    });
}

pub(crate) fn profile_main_end() {
    PROFILE.with(|profile| {
        if let Some(profile) = profile.borrow_mut().as_mut() { profile.active = false; }
    });
}

/// VWF's separate counter is deliberately emitted separately by the caller.
pub fn take_routine_profile() -> BTreeMap<u32, [u64; 3]> {
    PROFILE.with(|profile| profile.borrow_mut().take().map_or_else(BTreeMap::new, |p| p.rows))
}

/// Take the object breakdown before taking the ordinary routine profile.
/// The key is (subtype << 8 | object ID, original routine address).
pub fn take_dungeon_object_profile() -> BTreeMap<(u16, u32), [u64; 3]> {
    PROFILE.with(|profile| profile.borrow_mut().as_mut()
        .map_or_else(BTreeMap::new, |p| std::mem::take(&mut p.object_rows)))
}

/// Diagnostic identity only: charges keep their original routine ownership.
pub(crate) struct DungeonObjectProfile(bool);

pub(crate) fn dungeon_object_profile(object: u16) -> DungeonObjectProfile {
    let owns = !is_muted() && PROFILE.with(|profile| {
        if let Some(p) = profile.borrow_mut().as_mut().filter(|p| p.active) {
            assert!(p.object.replace(object).is_none());
            true
        } else { false }
    });
    DungeonObjectProfile(owns)
}

impl Drop for DungeonObjectProfile {
    fn drop(&mut self) {
        if !self.0 { return; }
        PROFILE.with(|profile| {
            if let Some(p) = profile.borrow_mut().as_mut() { p.object = None; }
        });
    }
}

/// Body-only attribution at $06:84E2's state dispatch. Timers and entry
/// instructions precede this guard; the captured type/state/slot never follow
/// a body which transforms or replaces its slot. Nested bodies restore their
/// caller's diagnostic identity, and muted previews do not acquire one.
pub(crate) struct SpriteBodyProfile(Option<Option<(u8, u8, u8)>>);

pub(crate) fn sprite_body_profile(kind: u8, state: u8, slot: u8) -> SpriteBodyProfile {
    let previous = if is_muted() { None } else { PROFILE.with(|profile| {
        profile.borrow_mut().as_mut().filter(|p| p.active).map(|p| {
            let previous = p.sprite.replace((kind, state, slot));
            p.sprite_rows.entry((kind, state, slot, 0x06_84e2)).or_default()[0] += 1;
            previous
        })
    }) };
    SpriteBodyProfile(previous)
}

impl Drop for SpriteBodyProfile {
    fn drop(&mut self) {
        let Some(previous) = self.0 else { return; };
        PROFILE.with(|profile| {
            if let Some(p) = profile.borrow_mut().as_mut() { p.sprite = previous; }
        });
    }
}

/// Take body attribution before taking the ordinary routine profile.
pub fn take_sprite_body_profile() -> BTreeMap<(u8, u8, u8, u32), [u64; 3]> {
    PROFILE.with(|profile| profile.borrow_mut().as_mut()
        .map_or_else(BTreeMap::new, |p| std::mem::take(&mut p.sprite_rows)))
}

/// Master cycles charged so far on this thread.
#[inline]
pub fn master() -> u64 {
    MASTER.with(Cell::get)
}

/// Charge master cycles to the routine being executed.
#[inline]
pub fn charge(master: u64) {
    if MUTED.with(Cell::get) == 0 {
        MASTER.with(|total| total.set(total.get() + master));
        PROFILE.with(|profile| {
            if let Some(profile) = profile.borrow_mut().as_mut().filter(|p| p.active) {
                LEDGER.with(|ledger| {
                    let ledger = ledger.borrow();
                    let address = ledger.addresses.last().copied().unwrap_or(0);
                    profile.rows.entry(address).or_default()[2] += master;
                    if let Some(object) = profile.object {
                        profile.object_rows.entry((object, address)).or_default()[2] += master;
                        if let Some(depth) = ledger.addresses.iter().position(|&a| a == 0x01893c) {
                            for &routine in &ledger.addresses[depth..] {
                                profile.object_rows.entry((object, routine)).or_default()[1] += master;
                            }
                        }
                    }
                    if let Some((kind, state, slot)) = profile.sprite {
                        // One body owns each charge. Caller/timer work outside
                        // the guard is deliberately absent from this table.
                        profile.sprite_rows.entry((kind, state, slot, address)).or_default()[2] += master;
                        profile.sprite_rows.entry((kind, state, slot, 0x06_84e2)).or_default()[1] += master;
                        if let Some(depth) = ledger.addresses.iter().position(|&a| a == 0x06_84e2) {
                            for &routine in &ledger.addresses[depth + 1..] {
                                if routine != 0x06_84e2 {
                                    profile.sprite_rows.entry((kind, state, slot, routine)).or_default()[1] += master;
                                }
                            }
                        }
                    }
                    if ledger.addresses.is_empty() {
                        profile.rows.entry(0).or_default()[1] += master;
                    }
                    for address in &ledger.addresses {
                        profile.rows.entry(*address).or_default()[1] += master;
                    }
                });
            }
        });
    }
}

/// Run `probe` with the ledger muted: charges are dropped and scopes record
/// nothing. For translated work executed on a clone to preview a CPU
/// schedule, which the original CPU never executed.
pub fn muted<R>(probe: impl FnOnce() -> R) -> R {
    MUTED.with(|depth| depth.set(depth.get() + 1));
    let result = probe();
    MUTED.with(|depth| depth.set(depth.get() - 1));
    result
}

pub(crate) fn is_muted() -> bool {
    MUTED.with(Cell::get) != 0
}

/// Actual native programming changes invalidate an immutable HDMA lease.
/// Payload writes do not: they cannot change the transfer's bus duration.
pub(crate) fn note_hdma_program_write() {
    if !is_muted() { HDMA_PROGRAM_EPOCH.with(|epoch| epoch.set(epoch.get().wrapping_add(1))); }
}

pub(crate) fn hdma_program_epoch() -> u64 {
    HDMA_PROGRAM_EPOCH.with(Cell::get)
}

/// A routine scope: created at the routine's entry with its ROM address,
/// it records the routine's charge when dropped.
#[must_use = "bind the scope to a local so it lives until the routine returns"]
pub struct RoutineScope {
    address: u32,
    started: u64,
    muted: bool,
}

/// Enter an annotated routine at its ROM address.
pub fn routine(address: u32) -> RoutineScope {
    enter_routine(address, true)
}

/// Attribute an existing charge to its source call frame only during a
/// routine profile. A resumed fragment does not report another call entry;
/// a fragment already inside that frame does not create recursive ancestry.
/// With profiling off, neither ordinary ledger records nor annotation counts
/// change. This helper must never introduce a new instruction price.
pub(crate) fn profile_routine(address: u32, entered: bool) -> Option<RoutineScope> {
    if !profiling() || is_muted()
        || LEDGER.with(|ledger| ledger.borrow().addresses.last().copied()) == Some(address) {
        return None;
    }
    Some(enter_routine(address, entered))
}

/// A translated helper can continue the current source frame without a
/// new source call. Preserve the ordinary ledger outside routine profiling.
pub(crate) fn routine_continuation(address: u32) -> Option<RoutineScope> {
    if profiling() { profile_routine(address, false) } else { Some(routine(address)) }
}

/// A branch/JMP tail keeps the real source call frame. Retain the established
/// body label in ordinary ledger output when routine profiling is disabled.
pub(crate) fn tail_routine(body: u32, frame: u32) -> Option<RoutineScope> {
    if profiling() { profile_routine(frame, false) } else { Some(routine(body)) }
}

fn enter_routine(address: u32, entered: bool) -> RoutineScope {
    let muted = MUTED.with(Cell::get) > 0;
    if !muted {
        LEDGER.with(|ledger| {
            let mut ledger = ledger.borrow_mut();
            ledger.nested.push(0);
            ledger.addresses.push(address);
        });
        PROFILE.with(|profile| {
            if let Some(profile) = profile.borrow_mut().as_mut().filter(|p| p.active && p.fragment_depth == 0 && entered) {
                profile.rows.entry(address).or_default()[0] += 1;
                if let Some(object) = profile.object {
                    profile.object_rows.entry((object, address)).or_default()[0] += 1;
                }
                if let Some((kind, state, slot)) = profile.sprite.filter(|_| address != 0x06_84e2) {
                    profile.sprite_rows.entry((kind, state, slot, address)).or_default()[0] += 1;
                }
            }
        });
        SCOPES_OPENED.with(|count| count.set(count.get() + 1));
    }
    RoutineScope {
        address,
        started: master(),
        muted,
    }
}

impl Drop for RoutineScope {
    fn drop(&mut self) {
        if self.muted {
            return;
        }
        let inclusive = master() - self.started;
        LEDGER.with(|ledger| {
            let mut ledger = ledger.borrow_mut();
            let nested = ledger.nested.pop().unwrap_or(0);
            assert_eq!(ledger.addresses.pop(), Some(self.address));
            if let Some(parent) = ledger.nested.last_mut() {
                *parent += inclusive;
            }
            ledger.completed.push((self.address, inclusive - nested));
        });
    }
}

/// Probed calls so far that charged nothing and opened no scope.
pub fn silent_calls() -> u64 {
    SILENT_CALLS.with(Cell::get)
}

/// Guard placed at the entry of a translated routine (or around a dispatch
/// to one) whose annotation may be missing: when it drops without any
/// charge or scope having happened since it was taken, the call is counted
/// in `silent_calls`. A budget derived from the ledger is complete only
/// while no probed call was silent since its reference point. The
/// heuristic accepts a routine that charges anything at all (an unannotated
/// body calling an annotated helper passes), so it detects missing
/// annotations, not partial ones. Muted regions are not probed.
#[must_use = "bind the probe to a local so it observes the whole call"]
pub struct AnnotationProbe {
    scopes: u64,
    master: u64,
    muted: bool,
}

pub fn probe_annotation() -> AnnotationProbe {
    AnnotationProbe {
        scopes: SCOPES_OPENED.with(Cell::get),
        master: master(),
        muted: MUTED.with(Cell::get) > 0,
    }
}

impl Drop for AnnotationProbe {
    fn drop(&mut self) {
        if self.muted {
            return;
        }
        if SCOPES_OPENED.with(Cell::get) == self.scopes && master() == self.master {
            SILENT_CALLS.with(|count| count.set(count.get() + 1));
        }
    }
}

/// Charge a routine whose cost does not depend on its inputs.
pub fn charge_routine(address: u32, master: u64) {
    let _scope = routine(address);
    charge(master);
}

/// Instruction continuations can retire inside their ordinary call scope.
/// Reuse that frame rather than reporting a recursive call per instruction.
pub(crate) fn charge_in_routine(address: u32, master: u64) {
    let current = LEDGER.with(|ledger| ledger.borrow().addresses.last().copied());
    if current == Some(address) {
        charge(master);
    } else {
        charge_routine(address, master);
    }
}

/// Restore call ancestry for a flattened instruction continuation while
/// profiling. Ordinary ledger prices and retirement remain unchanged.
pub(crate) fn with_profile_call_path<R>(path: &[u32], retire: impl FnOnce() -> R) -> R {
    if path.is_empty() || MUTED.with(Cell::get) != 0 || !profiling() { return retire(); }
    let base = LEDGER.with(|ledger| ledger.borrow().addresses.clone());
    let mut full = base.clone();
    for &address in path {
        if full.last() != Some(&address) { full.push(address); }
    }
    PROFILE.with(|profile| {
        let mut profile = profile.borrow_mut();
        let profile = profile.as_mut().unwrap();
        // Fragments retain a single logical call until their path leaves
        // that frame. A byte-reader entry after its caller's next JSR is a
        // new call; another instruction inside that byte reader is not.
        for depth in base.len()..full.len() {
            if profile.fragment_path.get(..=depth) != full.get(..=depth) {
                profile.rows.entry(full[depth]).or_default()[0] += 1;
            }
        }
        profile.fragment_path = full;
        profile.fragment_depth += 1;
    });
    fn enter<R>(path: &[u32], retire: impl FnOnce() -> R) -> R {
        let Some((&address, rest)) = path.split_first() else { return retire(); };
        let current = LEDGER.with(|ledger| ledger.borrow().addresses.last().copied());
        let _scope = (current != Some(address)).then(|| routine(address));
        enter(rest, retire)
    }
    let result = enter(path, retire);
    PROFILE.with(|profile| profile.borrow_mut().as_mut().unwrap().fragment_depth -= 1);
    result
}

pub(crate) fn profiling() -> bool {
    PROFILE.with(|profile| profile.borrow().as_ref().is_some_and(|p| p.active))
}

/// Append this host's routine charges to `<dir>/ledger.csv` when
/// `ZELDA3_DEBUG_CYCLE_LEDGER` is set, then forget them.
pub fn flush_host(host: u32) {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        let current_master = master();
        let charged = current_master - ledger.last_host_master;
        ledger.last_host_master = current_master;
        let current_silent_calls = silent_calls();
        let silent = current_silent_calls - ledger.last_host_silent_calls;
        ledger.last_host_silent_calls = current_silent_calls;
        if let Some(dir) = crate::debug_env::var_os!("ZELDA3_DEBUG_CYCLE_LEDGER") {
            let dir = std::path::PathBuf::from(dir);
            let _ = std::fs::create_dir_all(&dir);
            // Completed scopes omit unscoped charges and can span a flush.
            // Preserve the independent counter delta, including hosts with
            // no completed scope, rather than treating their CSV sum as a
            // complete CPU-work measurement. These are charged work cycles,
            // not a raster clock or proof that every instruction was priced.
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true).append(true).open(dir.join("host_totals.csv"))
            {
                let scoped = ledger.completed.iter().map(|(_, cycles)| cycles).sum::<u64>();
                let _ = writeln!(file, "{host},{charged},{scoped},{silent},{}", ledger.nested.len());
            }
            if let Ok(file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("ledger.csv"))
            {
                let mut writer = std::io::BufWriter::new(file);
                for (address, master) in &ledger.completed {
                    let _ = writeln!(writer, "{host},{address:06x},{master}");
                }
            }
        }
        ledger.completed.clear();
    });
}

/// Compare a native caller with ROM self work without attributing its
/// separately annotated callees to that caller.
#[cfg(test)]
pub(crate) fn measure_routine_self_work(address: u32, run: impl FnOnce()) -> u64 {
    let start = LEDGER.with(|ledger| ledger.borrow().completed.len());
    run();
    LEDGER.with(|ledger| ledger.borrow().completed[start..].iter()
        .filter(|(pc, _)| *pc == address).map(|(_, cycles)| *cycles).sum())
}

/// Execute an unmodified ROM fixture and compare its exclusive instruction
/// work with the native marker trail. `owns` describes source ownership (it
/// can include JMP tails/local stack pushes), never a cycle formula. The
/// fixture supplies the normal RTS/RTL sentinel at $8000 and CPU registers.
#[cfg(test)]
pub(crate) fn assert_rom_marker_work(
    routine: u32,
    machine: snes::Snes,
    owns: impl Fn(&snes::Snes) -> bool,
    native: impl FnOnce(),
) -> snes::RomCpuTimingProbe {
    use snes::{CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
        RomCpuTimingProbe, SourcePpuReadState};
    let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
    let mut source = RomCpuTimingProbe::new(machine, clock, SourcePpuReadState::snes9x_reset()).unwrap();
    let mut expected = 0;
    let mut source_work = BTreeMap::<u32, (u64, u64)>::new();
    // Complete loader fixtures also execute their unmodified decompression
    // children, even when the ownership predicate excludes those children.
    for _ in 0..1_000_000 {
        let owned = owns(source.snes());
        let pc = (u32::from(source.snes().cpu.k) << 16) | u32::from(source.snes().cpu.pc);
        let step = source.step().unwrap();
        if owned {
            let clocks = step.transactions.iter()
                .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
            expected += clocks;
            let row = source_work.entry(pc).or_default();
            row.0 += 1;
            row.1 += clocks;
        }
        if source.snes().cpu.pc == 0x8000 { break; }
    }
    assert_eq!(source.snes().cpu.pc, 0x8000, "ROM fixture did not return");
    MARKER_TRAIL.with(|t| assert!(t.borrow_mut().replace(Vec::new()).is_none()));
    native();
    let trail = MARKER_TRAIL.with(|t| t.borrow_mut().take().unwrap());
    let actual: u64 = trail.iter().filter(|t| t.0 == routine).map(|t| t.3).sum();
    assert_eq!(actual, expected,
        "routine {routine:06x}: native marker trail {trail:?}; source instruction work {source_work:x?}");
    source
}

#[cfg(test)]
mod annotation_probe_tests {
    use super::*;

    #[test]
    fn generated_markers_preserve_lane_muting_and_routine_ownership() {
        let start = master();
        blocks(false).block(0x1e9bd0);
        blocks(false).taken(0x1e9bd3);
        blocks(false).crossing(0x1e9bd0);
        muted(|| blocks(true).block(0x1e9bd0));
        assert_eq!(master(), start);
        profile_iteration(true);
        profile_main_begin();
        {
            let _scope = routine(0x1e8b19);
            let markers = blocks(true);
            markers.block(0x1e9bd0);
            markers.taken(0x1e9bd3);
            markers.crossing(0x1e9bd0);
        }
        profile_main_end();
        assert_eq!(master() - start, 60);
        assert_eq!(take_block_profile(), BTreeMap::from([
            ((0x1e8b19, 0x1e9bd0, 0), [1, 48]),
            ((0x1e8b19, 0x1e9bd0, 2), [1, 6]),
            ((0x1e8b19, 0x1e9bd3, 1), [1, 6]),
        ]));
        assert_eq!(take_routine_profile()[&0x1e8b19], [1, 60, 60]);
    }

    #[test]
    #[should_panic(expected = "generated conditional branch")]
    fn taken_marker_rejects_a_nonbranch_pc() { blocks(true).taken(0x1e9bd0); }

    #[test]
    #[should_panic(expected = "generated indexed read")]
    fn crossing_marker_rejects_a_nonindexed_pc() { blocks(true).crossing(0x1e9bdd); }

    #[test]
    fn object_identity_preserves_routine_totals_and_excludes_muted_previews() {
        profile_iteration(true);
        profile_main_begin();
        {
            let _object = dungeon_object_profile(0x123);
            let _dispatch = routine(0x01893c);
            charge(880);
            charge(100);
            charge_routine(0x019895, 506);
            muted(|| {
                let _preview = dungeon_object_profile(0x234);
                charge_routine(0x019895, 9000);
            });
            charge(42);
        }
        charge_routine(0x01893c, 7);
        profile_main_end();
        let objects = take_dungeon_object_profile();
        let routines = take_routine_profile();
        assert_eq!(objects[&(0x123, 0x01893c)], [1, 1528, 1022]);
        assert_eq!(objects[&(0x123, 0x019895)], [1, 506, 506]);
        assert_eq!(objects.len(), 2);
        assert_eq!(routines[&0x01893c], [2, 1535, 1029]);
        assert_eq!(routines[&0x019895], [1, 506, 506]);
        assert_eq!(objects.values().map(|r| r[2]).sum::<u64>(), 1528);
    }

    #[test]
    fn flattened_call_paths_count_entries_and_conserve_instruction_work() {
        profile_iteration(true);
        let parent = routine(0x00e19b);
        profile_main_begin();
        for path in [&[0x00e19b, 0x00e583][..], &[0x00e19b, 0x00e583],
            &[0x00e19b, 0x00e583, 0x00e772], &[0x00e19b, 0x00e583, 0x00e772, 0x00e843],
            &[0x00e19b, 0x00e583, 0x00e772, 0x00e843], &[0x00e19b, 0x00e583, 0x00e772],
            &[0x00e19b, 0x00e583, 0x00e772, 0x00e843], &[0x00e19b, 0x00e583]] {
            with_profile_call_path(path, || charge_in_routine(*path.last().unwrap(), 10));
        }
        muted(|| with_profile_call_path(&[0x00e19b, 0x00f000], || charge_in_routine(0x00f000, 1000)));
        drop(parent);
        profile_main_end();
        let rows = take_routine_profile();
        assert_eq!(rows[&0x00e19b], [1, 80, 0]);
        assert_eq!(rows[&0x00e583], [1, 80, 30]);
        assert_eq!(rows[&0x00e772], [1, 50, 20]);
        assert_eq!(rows[&0x00e843], [2, 30, 30]);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows.values().map(|row| row[2]).sum::<u64>(), 80);
    }

    #[test]
    fn profile_fragments_keep_one_source_call_and_the_original_ledger() {
        profile_iteration(false);
        let opened = SCOPES_OPENED.with(Cell::get);
        let completed = LEDGER.with(|ledger| ledger.borrow().completed.len());
        assert!(profile_routine(0x02_fd87, true).is_none());
        assert_eq!(SCOPES_OPENED.with(Cell::get), opened);
        assert_eq!(LEDGER.with(|ledger| ledger.borrow().completed.len()), completed);

        profile_iteration(true);
        profile_main_begin();
        let before = master();
        {
            let _parent = profile_routine(0x02_fd46, true);
            charge(46);
        }
        for _ in 0..2 {
            let _parent = profile_routine(0x02_fd46, false);
            {
                let _child = profile_routine(0x02_fd87, true);
                charge(224);
                let _same_frame = profile_routine(0x02_fd87, false);
                charge(10);
            }
            for _ in 0..3 {
                let _child = profile_routine(0x02_fd87, false);
                charge(20);
            }
            muted(|| {
                let _preview = profile_routine(0x02_fd87, true);
                charge(999);
            });
        }
        profile_main_end();
        let rows = take_routine_profile();
        assert_eq!(master() - before, 634);
        assert_eq!(rows[&0x02_fd46], [1, 634, 46]);
        assert_eq!(rows[&0x02_fd87], [2, 588, 588]);
        assert_eq!(rows.values().map(|r| r[2]).sum::<u64>(), 634);
    }

    #[test]
    fn sprite_bodies_preserve_selection_and_exclude_entry_timers_and_previews() {
        profile_iteration(true);
        profile_main_begin();
        let before = master();
        {
            let _single = routine(0x06_84e2);
            charge(48);
            let _continuation = routine_continuation(0x06_84e2);
            charge_routine(0x06_83f2, 128);
            {
                let _body = sprite_body_profile(0x41, 9, 3);
                let _tail = tail_routine(0x06_8510, 0x06_84e2);
                charge(24);
                charge_routine(0x06_9000, 80);
                muted(|| {
                    let _preview = sprite_body_profile(0x42, 8, 1);
                    charge_routine(0x06_a000, 999);
                });
                {
                    let _nested = sprite_body_profile(0x42, 8, 1);
                    charge(12);
                }
                charge(6);
            }
            charge(2);
        }
        profile_main_end();
        let bodies = take_sprite_body_profile();
        let rows = take_routine_profile();
        assert_eq!(master() - before, 300);
        assert_eq!(rows[&0x06_84e2], [1, 300, 92]);
        assert_eq!(bodies[&(0x41, 9, 3, 0x06_84e2)], [1, 110, 30]);
        assert_eq!(bodies[&(0x41, 9, 3, 0x06_9000)], [1, 80, 80]);
        assert_eq!(bodies[&(0x42, 8, 1, 0x06_84e2)], [1, 12, 12]);
        assert_eq!(bodies.len(), 3);
        assert_eq!(bodies.values().map(|r| r[2]).sum::<u64>(), 122);
    }

    #[test]
    fn routine_profile_reconciles_nested_muted_and_unscoped_main_work() {
        profile_iteration(true);
        charge(100); // Before main: excluded.
        let parent = routine(0x008034);
        profile_main_begin();
        charge(10);
        {
            let _child = routine(0x018000);
            charge(20);
            muted(|| charge_routine(0x028000, 500));
        }
        charge_routine(0x038000, 0); // Coverage includes zero-priced calls.
        drop(parent);
        charge(3);
        profile_main_end();
        charge_routine(0x0080c9, 1000); // Closing handler: excluded.
        let rows = take_routine_profile();
        assert_eq!(rows[&0x008034], [1, 30, 10]);
        assert_eq!(rows[&0x018000], [1, 20, 20]);
        assert_eq!(rows[&0x038000], [1, 0, 0]);
        assert_eq!(rows[&0], [0, 3, 3]);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows.values().map(|row| row[2]).sum::<u64>(), 33);
    }

    #[test]
    fn a_probed_call_is_silent_only_when_it_neither_charges_nor_opens_a_scope() {
        let before = silent_calls();
        {
            let _probe = probe_annotation();
        }
        assert_eq!(silent_calls(), before + 1);
        {
            let _probe = probe_annotation();
            charge(16);
        }
        assert_eq!(silent_calls(), before + 1);
        {
            let _probe = probe_annotation();
            let _scope = routine(0x00_841e);
        }
        assert_eq!(silent_calls(), before + 1);
        muted(|| {
            let _probe = probe_annotation();
        });
        assert_eq!(silent_calls(), before + 1);
    }
}
