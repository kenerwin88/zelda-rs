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

use std::cell::RefCell;
use std::io::Write;

#[derive(Debug, Default)]
struct Ledger {
    master: u64,
    /// Depth of `muted` regions: work the engine runs on a probe clone (a
    /// CPU-timing preview) is not work the original CPU did.
    muted: u32,
    /// Charges recorded by nested scopes that have closed, per open scope
    /// depth, so a routine's record excludes its annotated callees (a self
    /// cost, comparable with the profiler's own-instruction totals).
    nested: Vec<u64>,
    /// Completed routine self charges this host frame: (ROM address, master).
    completed: Vec<(u32, u64)>,
}

thread_local! {
    static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
}

/// Master cycles charged so far on this thread.
pub fn master() -> u64 {
    LEDGER.with(|ledger| ledger.borrow().master)
}

/// Charge master cycles to the routine being executed.
#[inline]
pub fn charge(master: u64) {
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        if ledger.muted == 0 {
            ledger.master += master;
        }
    });
}

/// Run `probe` with the ledger muted: charges are dropped and scopes record
/// nothing. For translated work executed on a clone to preview a CPU
/// schedule, which the original CPU never executed.
pub fn muted<R>(probe: impl FnOnce() -> R) -> R {
    LEDGER.with(|ledger| ledger.borrow_mut().muted += 1);
    let result = probe();
    LEDGER.with(|ledger| ledger.borrow_mut().muted -= 1);
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
    LEDGER.with(|ledger| {
        let mut ledger = ledger.borrow_mut();
        let muted = ledger.muted > 0;
        if !muted {
            ledger.nested.push(0);
        }
        RoutineScope {
            address,
            started: ledger.master,
            muted,
        }
    })
}

impl Drop for RoutineScope {
    fn drop(&mut self) {
        if self.muted {
            return;
        }
        LEDGER.with(|ledger| {
            let mut ledger = ledger.borrow_mut();
            let inclusive = ledger.master - self.started;
            let nested = ledger.nested.pop().unwrap_or(0);
            if let Some(parent) = ledger.nested.last_mut() {
                *parent += inclusive;
            }
            ledger.completed.push((self.address, inclusive - nested));
        });
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
