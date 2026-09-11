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

#[derive(Debug, Default)]
struct Ledger {
    /// Charges recorded by nested scopes that have closed, per open scope
    /// depth, so a routine's record excludes its annotated callees (a self
    /// cost, comparable with the profiler's own-instruction totals).
    nested: Vec<u64>,
    /// Completed routine self charges this host frame: (ROM address, master).
    completed: Vec<(u32, u64)>,
}

thread_local! {
    /// The running total, a plain cell so a charge is one add on the hot
    /// path (annotated routines charge many times per frame).
    static MASTER: Cell<u64> = const { Cell::new(0) };
    /// Depth of `muted` regions: work the engine runs on a probe clone (a
    /// CPU-timing preview) is not work the original CPU did.
    static MUTED: Cell<u32> = const { Cell::new(0) };
    /// Routine scopes opened so far on this thread (an annotation census).
    static SCOPES_OPENED: Cell<u64> = const { Cell::new(0) };
    /// Probed calls that charged nothing and opened no scope: translated
    /// routines the ledger does not price yet (see `probe_annotation`).
    static SILENT_CALLS: Cell<u64> = const { Cell::new(0) };
    static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
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
    let muted = MUTED.with(Cell::get) > 0;
    if !muted {
        LEDGER.with(|ledger| ledger.borrow_mut().nested.push(0));
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

/// Append this host's routine charges to `<dir>/ledger.csv` when
/// `ZELDA3_DEBUG_CYCLE_LEDGER` is set, then forget them.
pub fn flush_host(host: u32) {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        if ledger.completed.is_empty() {
            return;
        }
        if let Some(dir) = crate::debug_env::var_os("ZELDA3_DEBUG_CYCLE_LEDGER") {
            let dir = std::path::PathBuf::from(dir);
            let _ = std::fs::create_dir_all(&dir);
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

#[cfg(test)]
mod annotation_probe_tests {
    use super::*;

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
