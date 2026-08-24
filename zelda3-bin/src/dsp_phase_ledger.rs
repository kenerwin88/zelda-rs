//! Controlled, trace-only pinned-Snes9x 32-phase DSP ledger generator.

use serde::Serialize;
use serde_json::{json, Value};
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};

use crate::snes9x_compare::{
    acquire_snes9x_compare_lock, compact_delta_integer_sequence_with_zstd,
};

const ABI_VERSION: i32 = 2;
const PINNED_SOURCE_REVISION: &str = "921f9f7b83660eb44ad263022a57a4a029057c37";
const PINNED_PATCH_NAMES: [&str; 4] = [
    "zelda3-trace.patch",
    "zelda3-trace-obj-cache.patch",
    "zelda3-spc-opcode-ledger.patch",
    "zelda3-dsp-phase-ledger.patch",
];
const EXPECTED_STATE_SIZE: usize = 642;
const CASE_COUNT: usize = 5_440;
const ENCODED_SIZE_CAP: u64 = 512 * 1024;
const REQUIRED_BRANCH_MASK: u64 = (1 << 24) - 1;
const BRANCH_SOURCE_MAP: [&str; 24] = [
    "envelope-release",
    "adsr-enabled",
    "gain-direct",
    "gain-linear-decrease",
    "gain-exponential-decrease",
    "gain-mode-6",
    "gain-mode-7-two-slope",
    "counter-tick",
    "pitch-modulation",
    "key-on-delay",
    "noise",
    "soft-reset-or-brr-end",
    "key-off",
    "key-on",
    "echo-voice-enable",
    "brr-decode",
    "brr-loop",
    "global-mute",
    "echo-write-enable",
    "echo-length-latch",
    "echo-ring-wrap",
    "misc29-every-other-sample",
    "misc30-latch-key-state",
    "misc30-noise-counter",
];
const SUBOPERATION_SOURCE_MAP: [(i64, &str); 16] = [
    (1, "voice_V1"),
    (2, "voice_V2"),
    (4, "voice_V4"),
    (5, "voice_V5"),
    (6, "voice_V6"),
    (7, "voice_V7"),
    (8, "voice_V8"),
    (9, "voice_V9"),
    (10, "voice_V3a"),
    (11, "voice_V3b"),
    (12, "voice_V3c"),
    (27, "misc_27"),
    (28, "misc_28"),
    (29, "misc_29"),
    (30, "misc_30"),
    (50, "echo_phase"),
];
#[cfg(test)]
const FIXTURE_RELATIVE_PATH: &str =
    "external/snes9x-libretro/fixtures/snes9x-spc-dsp-phase-ledger.jsonl";

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Operation {
    Clock,
    Read,
    Write,
    BlockedWrite,
}

impl Operation {
    fn abi(self) -> i32 {
        match self {
            Self::Clock => 0,
            Self::Read => 1,
            Self::Write => 2,
            Self::BlockedWrite => 3,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct LedgerCase {
    id: u16,
    group: &'static str,
    profile: u8,
    start_phase: u8,
    debt: u16,
    operation: Operation,
    address: u8,
    value: u8,
}

fn manifest() -> Vec<LedgerCase> {
    let mut cases = Vec::with_capacity(CASE_COUNT);
    for profile in 0..=1 {
        for tick in 0..256u16 {
            cases.push(LedgerCase {
                id: cases.len() as u16,
                group: "phase-chain",
                profile,
                start_phase: (tick & 31) as u8,
                debt: 1,
                operation: Operation::Clock,
                address: 0,
                value: 0,
            });
        }
    }
    for profile in 0..=1 {
        for phase in 0..32u8 {
            for debt in 0..=64u16 {
                cases.push(LedgerCase {
                    id: cases.len() as u16,
                    group: "f3-read",
                    profile,
                    start_phase: phase,
                    debt,
                    operation: Operation::Read,
                    address: 0x7c,
                    value: 0,
                });
            }
        }
    }
    for phase in 0..32u8 {
        for debt in [0u16, 1, 32, 64] {
            for (operation, address, value) in [
                (Operation::Write, 0x0c, 0x51),
                (Operation::Write, 0x28, 0x62),
                (Operation::Write, 0x29, 0x73),
                (Operation::Write, 0x4c, 0x55),
                (Operation::Write, 0x7c, 0xff),
                (Operation::BlockedWrite, 0xaa, 0x84),
            ] {
                cases.push(LedgerCase {
                    id: cases.len() as u16,
                    group: "f3-write",
                    profile: 0,
                    start_phase: phase,
                    debt,
                    operation,
                    address,
                    value,
                });
            }
        }
    }
    assert_eq!(cases.len(), CASE_COUNT);
    cases
}

fn seeded_ram(profile: u8) -> Vec<u8> {
    let mut ram = (0..65536u32)
        .map(|address| (address * 37 + (address >> 8) * 13 + u32::from(profile) * 97 + 0x5a) as u8)
        .collect::<Vec<_>>();
    for voice in 0..8usize {
        let directory = 0x2000 + voice * 4;
        let brr = 0x3000 + voice * 0x20;
        ram[directory] = brr as u8;
        ram[directory + 1] = (brr >> 8) as u8;
        ram[directory + 2] = brr as u8;
        ram[directory + 3] = (brr >> 8) as u8;
        ram[brr] = if voice & 1 != 0 { 0 } else { 3 };
    }
    ram
}

type NoArg = unsafe extern "C" fn() -> c_int;
type OneArg = unsafe extern "C" fn(c_int) -> c_int;
type TwoArg = unsafe extern "C" fn(c_int, c_int) -> c_int;
type ThreeArg = unsafe extern "C" fn(c_int, c_int, c_int) -> c_int;
type FourArg = unsafe extern "C" fn(c_int, c_int, c_int, c_int) -> c_int;
type CopyBytes = unsafe extern "C" fn(*mut u8, c_int) -> c_int;

struct LedgerCore {
    handle: *mut c_void,
    abi_version: NoArg,
    state_capacity: NoArg,
    begin: ThreeArg,
    end: unsafe extern "C" fn(),
    operate: FourArg,
    finalize_capture: NoArg,
    state_size: NoArg,
    state_byte: OneArg,
    copy_ram: CopyBytes,
    ram_diff_count: NoArg,
    ram_diff_value: TwoArg,
    sample_count: NoArg,
    sample_value: TwoArg,
    suboperation_count: NoArg,
    suboperation_value: TwoArg,
    phase: NoArg,
    clock_before: NoArg,
    clock_after: NoArg,
    config_value: OneArg,
    branch_evaluated_mask: unsafe extern "C" fn() -> u64,
    branch_taken_mask: unsafe extern "C" fn() -> u64,
    branch_not_taken_mask: unsafe extern "C" fn() -> u64,
    overflow: NoArg,
}

impl LedgerCore {
    fn load(path: &Path) -> Result<Self, String> {
        let path =
            CString::new(path.as_os_str().as_encoded_bytes()).map_err(|error| error.to_string())?;
        let handle = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
        if handle.is_null() {
            return Err(dlerror());
        }
        unsafe {
            Ok(Self {
                handle,
                abi_version: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_abi_version")?,
                state_capacity: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_state_capacity")?,
                begin: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_begin")?,
                end: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_end")?,
                operate: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_operate")?,
                finalize_capture: symbol(
                    handle,
                    "zelda3_snes9x_debug_dsp_ledger_finalize_capture",
                )?,
                state_size: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_state_size")?,
                state_byte: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_state_byte")?,
                copy_ram: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_copy_ram")?,
                ram_diff_count: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_ram_diff_count")?,
                ram_diff_value: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_ram_diff_value")?,
                sample_count: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_sample_count")?,
                sample_value: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_sample_value")?,
                suboperation_count: symbol(
                    handle,
                    "zelda3_snes9x_debug_dsp_ledger_suboperation_count",
                )?,
                suboperation_value: symbol(
                    handle,
                    "zelda3_snes9x_debug_dsp_ledger_suboperation_value",
                )?,
                phase: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_phase")?,
                clock_before: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_clock_before")?,
                clock_after: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_clock_after")?,
                config_value: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_config_value")?,
                branch_evaluated_mask: symbol(
                    handle,
                    "zelda3_snes9x_debug_dsp_ledger_branch_evaluated_mask",
                )?,
                branch_taken_mask: symbol(
                    handle,
                    "zelda3_snes9x_debug_dsp_ledger_branch_taken_mask",
                )?,
                branch_not_taken_mask: symbol(
                    handle,
                    "zelda3_snes9x_debug_dsp_ledger_branch_not_taken_mask",
                )?,
                overflow: symbol(handle, "zelda3_snes9x_debug_dsp_ledger_overflow")?,
            })
        }
    }
}

impl Drop for LedgerCore {
    fn drop(&mut self) {
        unsafe { libc::dlclose(self.handle) };
    }
}

unsafe fn symbol<T: Copy>(handle: *mut c_void, name: &str) -> Result<T, String> {
    let name = CString::new(name).unwrap();
    let pointer = unsafe { libc::dlsym(handle, name.as_ptr()) };
    if pointer.is_null() {
        return Err(dlerror());
    }
    Ok(unsafe { std::mem::transmute_copy(&pointer) })
}

fn dlerror() -> String {
    let pointer = unsafe { libc::dlerror() };
    if pointer.is_null() {
        "dynamic loader returned no diagnostic".to_string()
    } else {
        unsafe { std::ffi::CStr::from_ptr(pointer.cast::<c_char>()) }
            .to_string_lossy()
            .into_owned()
    }
}

#[derive(Clone, Debug, Serialize)]
struct Checkpoint {
    id: u16,
    profile: u8,
    tick: u16,
    phase: u8,
    state_sha256: String,
    ram_sha256: String,
}

#[derive(Default)]
struct Rows {
    checkpoints: Vec<Checkpoint>,
    checkpoint_state_xor: Vec<[i64; 3]>,
    profile_initial_ram_diffs: Vec<[i64; 3]>,
    cases: Vec<[i64; 22]>,
    state_xor: Vec<[i64; 3]>,
    ram_diffs: Vec<[i64; 3]>,
    samples: Vec<[i64; 4]>,
    suboperations: Vec<[i64; 5]>,
}

fn read_state(core: &LedgerCore) -> Result<Vec<u8>, String> {
    let state_size = unsafe { (core.state_size)() } as usize;
    if state_size != EXPECTED_STATE_SIZE {
        return Err(format!(
            "copy_state size {state_size}, expected {EXPECTED_STATE_SIZE}"
        ));
    }
    Ok((0..state_size)
        .map(|index| unsafe { (core.state_byte)(index as i32) } as u8)
        .collect())
}

fn read_ram(core: &LedgerCore) -> Result<Vec<u8>, String> {
    let mut ram = vec![0; 65536];
    let copied = unsafe { (core.copy_ram)(ram.as_mut_ptr(), ram.len() as c_int) };
    if copied != ram.len() as c_int {
        return Err(format!(
            "trace core copied {copied} RAM bytes, expected 65536"
        ));
    }
    Ok(ram)
}

fn append_state_xor(rows: &mut Vec<[i64; 3]>, id: usize, state: &[u8], previous: &mut [u8]) {
    for (index, (&value, prior)) in state.iter().zip(previous.iter_mut()).enumerate() {
        rows.push([id as i64, index as i64, i64::from(value ^ *prior)]);
        *prior = value;
    }
}

fn checkpoint(
    rows: &mut Rows,
    profile: u8,
    tick: u16,
    state: &[u8],
    ram: &[u8],
    previous: &mut [u8],
) {
    let id = rows.checkpoints.len() as u16;
    rows.checkpoints.push(Checkpoint {
        id,
        profile,
        tick,
        phase: (tick & 31) as u8,
        state_sha256: parity::evidence::sha256_bytes(state),
        ram_sha256: parity::evidence::sha256_bytes(ram),
    });
    append_state_xor(
        &mut rows.checkpoint_state_xor,
        usize::from(id),
        state,
        previous,
    );
}

fn capture_case(
    core: &LedgerCore,
    case: &LedgerCase,
    start_checkpoint_ref: usize,
    rows: &mut Rows,
    previous_state: &mut [u8],
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let result = unsafe {
        (core.operate)(
            case.operation.abi(),
            i32::from(case.debt),
            i32::from(case.address),
            i32::from(case.value),
        )
    };
    if unsafe { (core.overflow)() } != 0 {
        return Err(format!("case {} overflowed", case.id));
    }
    let state = read_state(core)?;
    let ram = read_ram(core)?;
    let evaluated = unsafe { (core.branch_evaluated_mask)() };
    let taken = unsafe { (core.branch_taken_mask)() };
    let not_taken = unsafe { (core.branch_not_taken_mask)() };
    if evaluated != taken | not_taken || taken & not_taken & !evaluated != 0 {
        return Err(format!("case {} branch ownership is inconsistent", case.id));
    }
    rows.cases.push([
        i64::from(case.id),
        i64::from(case.profile),
        start_checkpoint_ref as i64,
        i64::from(case.start_phase),
        i64::from(case.debt),
        i64::from(case.operation.abi()),
        i64::from(case.address),
        i64::from(case.value),
        i64::from(result),
        i64::from(unsafe { (core.phase)() }),
        i64::from(unsafe { (core.clock_before)() }),
        i64::from(unsafe { (core.clock_after)() }),
        evaluated as u32 as i64,
        (evaluated >> 32) as i64,
        taken as u32 as i64,
        (taken >> 32) as i64,
        not_taken as u32 as i64,
        (not_taken >> 32) as i64,
        EXPECTED_STATE_SIZE as i64,
        i64::from(unsafe { (core.ram_diff_count)() }),
        i64::from(unsafe { (core.sample_count)() }),
        i64::from(unsafe { (core.suboperation_count)() }),
    ]);
    append_state_xor(
        &mut rows.state_xor,
        usize::from(case.id),
        &state,
        previous_state,
    );
    for difference in 0..unsafe { (core.ram_diff_count)() } {
        rows.ram_diffs.push([
            i64::from(case.id),
            i64::from(unsafe { (core.ram_diff_value)(difference, 0) }),
            i64::from(unsafe { (core.ram_diff_value)(difference, 1) }),
        ]);
    }
    for sample in 0..unsafe { (core.sample_count)() } {
        rows.samples.push([
            i64::from(case.id),
            i64::from(sample),
            i64::from(unsafe { (core.sample_value)(sample, 0) }),
            i64::from(unsafe { (core.sample_value)(sample, 1) }),
        ]);
    }
    for operation in 0..unsafe { (core.suboperation_count)() } {
        rows.suboperations.push([
            i64::from(case.id),
            i64::from(operation),
            i64::from(unsafe { (core.suboperation_value)(operation, 0) }),
            i64::from(unsafe { (core.suboperation_value)(operation, 1) }),
            i64::from(unsafe { (core.suboperation_value)(operation, 2) }),
        ]);
    }
    Ok((state, ram))
}

fn capture(core: &LedgerCore, cases: &[LedgerCase]) -> Result<Rows, String> {
    if unsafe { (core.abi_version)() } != ABI_VERSION
        || unsafe { (core.state_capacity)() } < EXPECTED_STATE_SIZE as i32
    {
        return Err("trace core DSP ledger ABI/schema mismatch".to_string());
    }
    let mut rows = Rows::default();
    let mut previous_state = vec![0u8; EXPECTED_STATE_SIZE];
    let mut previous_checkpoint_state = vec![0u8; EXPECTED_STATE_SIZE];

    for profile in 0..=1u8 {
        if unsafe { (core.begin)(i32::from(profile), 0, 1) } != 1 {
            return Err(format!("profile {profile} checkpoint begin failed"));
        }
        let initial_state = read_state(core)?;
        let initial_ram = read_ram(core)?;
        let seeded = seeded_ram(profile);
        for (address, (&actual, seeded)) in initial_ram.iter().zip(seeded).enumerate() {
            if actual != seeded {
                rows.profile_initial_ram_diffs.push([
                    i64::from(profile),
                    address as i64,
                    i64::from(actual),
                ]);
            }
        }
        checkpoint(
            &mut rows,
            profile,
            0,
            &initial_state,
            &initial_ram,
            &mut previous_checkpoint_state,
        );
        let first_case = usize::from(profile) * 256;
        for tick in 0..256usize {
            let case = &cases[first_case + tick];
            let start_ref = usize::from(profile) * 257 + tick;
            let (state, ram) = capture_case(core, case, start_ref, &mut rows, &mut previous_state)?;
            checkpoint(
                &mut rows,
                profile,
                tick as u16 + 1,
                &state,
                &ram,
                &mut previous_checkpoint_state,
            );
        }
        unsafe { (core.end)() };
    }

    for case in &cases[512..] {
        if unsafe { (core.begin)(i32::from(case.profile), i32::from(case.start_phase), 1) } != 1 {
            return Err(format!("case {} begin failed", case.id));
        }
        let start_ref = usize::from(case.profile) * 257 + usize::from(case.start_phase);
        let start_state = read_state(core)?;
        let start_ram = read_ram(core)?;
        let expected = &rows.checkpoints[start_ref];
        if parity::evidence::sha256_bytes(&start_state) != expected.state_sha256
            || parity::evidence::sha256_bytes(&start_ram) != expected.ram_sha256
        {
            return Err(format!(
                "case {} does not reproduce checkpoint {start_ref}",
                case.id
            ));
        }
        capture_case(core, case, start_ref, &mut rows, &mut previous_state)?;
        unsafe { (core.end)() };
    }
    if rows.checkpoints.len() != 514 || rows.cases.len() != CASE_COUNT {
        return Err("DSP ledger checkpoint/case count mismatch".to_string());
    }
    Ok(rows)
}

fn prove_copy_state_capture_neutrality(core: &LedgerCore) -> Result<(), String> {
    if unsafe { (core.begin)(0, 0, 1) } != 1 {
        return Err("copy_state neutrality begin failed".to_string());
    }
    let initial_state = read_state(core)?;
    let initial_ram = read_ram(core)?;
    if unsafe { (core.operate)(Operation::Clock.abi(), 0, 0, 0) } != 0 {
        return Err("copy_state neutrality recapture failed".to_string());
    }
    if read_state(core)? != initial_state || read_ram(core)? != initial_ram {
        return Err("copy_state projection is not state/RAM idempotent".to_string());
    }
    unsafe { (core.end)() };

    let mut observations = Vec::new();
    for capture in [1, 0] {
        if unsafe { (core.begin)(1, 27, capture) } != 1 {
            return Err("copy_state neutrality paired run failed".to_string());
        }
        if capture == 0 && unsafe { (core.state_size)() } != 0 {
            return Err("uncaptured begin performed an initial copy_state".to_string());
        }
        if unsafe { (core.operate)(Operation::Clock.abi(), 64, 0, 0) } != 0 {
            return Err("copy_state neutrality paired run failed".to_string());
        }
        if capture == 0 && unsafe { (core.state_size)() } != 0 {
            return Err("uncaptured operate performed copy_state".to_string());
        }
        if unsafe { (core.finalize_capture)() } != 1 {
            return Err("copy_state neutrality final capture failed".to_string());
        }
        let samples = (0..unsafe { (core.sample_count)() })
            .map(|sample| unsafe {
                (
                    (core.sample_value)(sample, 0),
                    (core.sample_value)(sample, 1),
                )
            })
            .collect::<Vec<_>>();
        observations.push((
            read_state(core)?,
            read_ram(core)?,
            samples,
            unsafe { (core.phase)() },
            unsafe { (core.clock_after)() },
        ));
        unsafe { (core.end)() };
    }
    if observations[0] != observations[1] {
        return Err("ledger capture changes state, RAM, samples, phase, or debt".to_string());
    }
    Ok(())
}

fn verify_core_receipt(core_path: &Path, receipt: &Value) -> Result<Vec<Value>, String> {
    if receipt["variant"] != "trace"
        || receipt["core_version"] != "1.63"
        || receipt["source_tag"] != "1.63"
        || receipt["source_revision"] != PINNED_SOURCE_REVISION
    {
        return Err(
            "DSP ledger core receipt is not the pinned traced Snes9x 1.63 source".to_string(),
        );
    }
    let actual_core_sha256 = parity::evidence::sha256_file(core_path)
        .map_err(|error| format!("{}: {error}", core_path.display()))?;
    if receipt["core_sha256"] != actual_core_sha256 {
        return Err("DSP ledger dylib bytes do not match the adjacent build receipt".to_string());
    }
    let receipt_core = receipt["core"]
        .as_str()
        .ok_or_else(|| "trace receipt core path is missing".to_string())?;
    if fs::canonicalize(receipt_core).map_err(|error| error.to_string())?
        != fs::canonicalize(core_path).map_err(|error| error.to_string())?
    {
        return Err("DSP ledger receipt belongs to a different dylib path".to_string());
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let lock_path = repo_root.join("external/snes9x-libretro/oracle-lock.json");
    let lock_sha256 = parity::evidence::sha256_file(&lock_path)
        .map_err(|error| format!("{}: {error}", lock_path.display()))?;
    if receipt["oracle_lock_sha256"] != lock_sha256 {
        return Err("DSP ledger core was built against a different source lock".to_string());
    }

    let patch_paths = receipt["patches"]
        .as_array()
        .ok_or_else(|| "trace receipt has no ordered patch stack".to_string())?;
    let patch_hashes = receipt["patch_sha256s"]
        .as_array()
        .ok_or_else(|| "trace receipt has no ordered patch hashes".to_string())?;
    if patch_paths.len() != PINNED_PATCH_NAMES.len() || patch_hashes.len() != patch_paths.len() {
        return Err("trace receipt patch stack length mismatch".to_string());
    }
    let mut verified = Vec::with_capacity(patch_paths.len());
    let mut aggregate = Vec::new();
    for (index, expected_name) in PINNED_PATCH_NAMES.iter().enumerate() {
        let receipt_path = Path::new(
            patch_paths[index]
                .as_str()
                .ok_or_else(|| "trace receipt patch path is not text".to_string())?,
        );
        if receipt_path.file_name().and_then(|name| name.to_str()) != Some(expected_name) {
            return Err(format!(
                "trace receipt patch {index} is not {expected_name}"
            ));
        }
        let current_path = repo_root
            .join("external/snes9x-libretro/patches")
            .join(expected_name);
        let bytes = fs::read(&current_path)
            .map_err(|error| format!("{}: {error}", current_path.display()))?;
        let hash = parity::evidence::sha256_bytes(&bytes);
        if patch_hashes[index] != hash {
            return Err(format!("trace receipt patch {expected_name} hash is stale"));
        }
        aggregate.extend_from_slice(expected_name.as_bytes());
        aggregate.push(0);
        aggregate.extend_from_slice(&bytes);
        aggregate.push(0);
        verified.push(json!({"path": current_path, "sha256": hash}));
    }
    if receipt["patch_sha256"] != parity::evidence::sha256_bytes(&aggregate) {
        return Err("trace receipt aggregate patch digest mismatch".to_string());
    }
    Ok(verified)
}

fn write_fixture(core_path: &Path, output: &Path) -> Result<(), String> {
    let receipt_path = PathBuf::from(format!("{}.json", core_path.display()));
    let receipt: Value = serde_json::from_slice(
        &fs::read(&receipt_path).map_err(|error| format!("{}: {error}", receipt_path.display()))?,
    )
    .map_err(|error| format!("{}: {error}", receipt_path.display()))?;
    let patch_hashes = verify_core_receipt(core_path, &receipt)?;
    let cases = manifest();
    let core = LedgerCore::load(core_path)?;
    prove_copy_state_capture_neutrality(&core)?;
    let rows = capture(&core, &cases)?;
    let combine =
        |row: &[i64; 22], low: usize| row[low] as u32 as u64 | ((row[low + 1] as u64) << 32);
    let evaluated_union = rows
        .cases
        .iter()
        .fold(0u64, |mask, row| mask | combine(row, 12));
    let taken_union = rows
        .cases
        .iter()
        .fold(0u64, |mask, row| mask | combine(row, 14));
    let not_taken_union = rows
        .cases
        .iter()
        .fold(0u64, |mask, row| mask | combine(row, 16));
    if evaluated_union != REQUIRED_BRANCH_MASK
        || taken_union != REQUIRED_BRANCH_MASK
        || not_taken_union != REQUIRED_BRANCH_MASK
    {
        return Err(format!(
            "DSP branch coverage incomplete: evaluated={evaluated_union:#x}, true={taken_union:#x}, false={not_taken_union:#x}"
        ));
    }
    let ram_seed_sha256 = (0..=1u8)
        .map(|profile| parity::evidence::sha256_bytes(&seeded_ram(profile)))
        .collect::<Vec<_>>();
    let provenance = json!({
        "kind": "provenance",
        "schema": "pinned-snes9x-spc-dsp-phase-ledger-v1",
        "abi_version": ABI_VERSION,
        "core": receipt,
        "source": {
            "phase_count": 32,
            "copy_state_size": EXPECTED_STATE_SIZE,
            "profiles": 2,
            "case_count": CASE_COUNT,
            "checkpoint_count": rows.checkpoints.len(),
            "branch_source_map": BRANCH_SOURCE_MAP,
            "suboperation_source_map": SUBOPERATION_SOURCE_MAP,
            "branch_evaluated_mask_union": evaluated_union,
            "branch_taken_mask_union": taken_union,
            "branch_not_taken_mask_union": not_taken_union,
            "separate_echo_buffer": unsafe { (core.config_value)(0) },
            "stereo_switch": unsafe { (core.config_value)(1) },
            "mute_mask": unsafe { (core.config_value)(2) },
            "interpolation_method": unsafe { (core.config_value)(3) },
            "copy_state_capture_neutral": true,
            "ram_seed_formula": "formula fill, then DIR $2000+$4v start/loop=$3000+$20v and BRR header alternates $00/$03",
            "ram_seed_sha256": ram_seed_sha256,
            "reset_owner": "SMP::power; declared APU RAM seed+DIR/BRR overrides; DSP::power; SPC_DSP::write register seed; active profile source KON all voices; DSP::synchronize 256; source GAIN7 voice6; DSP::synchronize 256; source KON voice0 only; DSP::synchronize phase clocks",
            "ordered_patch_sha256": patch_hashes,
        }
    });
    let data = json!({
        "kind": "spc-dsp-phase-ledger",
        "checkpoints": rows.checkpoints,
        "checkpoint_state_xor_sequence": compact_delta_integer_sequence_with_zstd(
            ["checkpoint_id","byte_index","xor_value"], rows.checkpoint_state_xor, true),
        "profile_initial_ram_diff_sequence": compact_delta_integer_sequence_with_zstd(
            ["profile","address","value"], rows.profile_initial_ram_diffs, true),
        "case_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id","profile","start_checkpoint_ref","start_phase","debt","operation","address","value","result","end_phase","clock_before","clock_after","branch_evaluated_lo","branch_evaluated_hi","branch_taken_lo","branch_taken_hi","branch_not_taken_lo","branch_not_taken_hi","state_size","ram_diff_count","sample_count","suboperation_count"],
            rows.cases, true),
        "copy_state_xor_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id","byte_index","xor_value"], rows.state_xor, true),
        "ram_diff_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id","address","value"], rows.ram_diffs, true),
        "sample_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id","sample_index","left","right"], rows.samples, true),
        "suboperation_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id","operation_index","kind","voice","argument"], rows.suboperations, true),
    });
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        output,
        format!(
            "{}\n{}\n",
            serde_json::to_string(&provenance).unwrap(),
            serde_json::to_string(&data).unwrap()
        ),
    )
    .map_err(|error| format!("{}: {error}", output.display()))?;
    let size = fs::metadata(output)
        .map_err(|error| error.to_string())?
        .len();
    if size > ENCODED_SIZE_CAP {
        return Err(format!(
            "encoded fixture is {size} bytes, cap is {ENCODED_SIZE_CAP}"
        ));
    }
    Ok(())
}

pub(crate) fn run_generate_command(args: &[String]) {
    if args.len() != 2 {
        eprintln!("usage: --generate-snes9x-dsp-phase-ledger TRACE_CORE OUTPUT");
        std::process::exit(2);
    }
    let _lane = acquire_snes9x_compare_lock();
    if let Err(error) = write_fixture(Path::new(&args[0]), Path::new(&args[1])) {
        eprintln!("failed to generate pinned Snes9x DSP ledger: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snes9x_compare::tests::expand_delta_sequence;

    fn v3(voice: i64) -> Vec<(i64, i64, i64)> {
        vec![(10, voice, 0), (11, voice, 0), (12, voice, 0)]
    }

    fn expected_phase_suboperations(phase: usize) -> Vec<(i64, i64, i64)> {
        let voice = |kind, voice| (kind, voice, 0);
        let echo = |phase| (50, -1, phase);
        let misc = |phase| (phase, -1, phase);
        match phase {
            0 => vec![voice(5, 0), voice(2, 1)],
            1 => {
                let mut operations = vec![voice(6, 0)];
                operations.extend(v3(1));
                operations
            }
            2 | 5 | 8 | 11 | 14 => {
                let base = (phase / 3) as i64;
                vec![voice(7, base), voice(1, base + 3), voice(4, base + 1)]
            }
            3 | 6 | 9 | 12 | 15 => {
                let base = (phase / 3 - 1) as i64;
                vec![voice(8, base), voice(5, base + 1), voice(2, base + 2)]
            }
            4 | 7 | 10 | 13 | 16 | 19 => {
                let base = if phase == 19 {
                    5
                } else {
                    (phase / 3 - 1) as i64
                };
                let mut operations = vec![voice(9, base), voice(6, base + 1)];
                operations.extend(v3(base + 2));
                operations
            }
            17 => vec![voice(1, 0), voice(7, 5), voice(4, 6)],
            18 => vec![voice(8, 5), voice(5, 6), voice(2, 7)],
            20 => vec![voice(1, 1), voice(7, 6), voice(4, 7)],
            21 => vec![voice(8, 6), voice(5, 7), voice(2, 0)],
            22 => vec![voice(10, 0), voice(9, 6), voice(6, 7), echo(22)],
            23 => vec![voice(7, 7), echo(23)],
            24 => vec![voice(8, 7), echo(24)],
            25 => vec![voice(11, 0), voice(9, 7), echo(25)],
            26 => vec![echo(26)],
            27 => vec![misc(27), echo(27)],
            28 => vec![misc(28), echo(28)],
            29 => vec![misc(29), echo(29)],
            30 => vec![misc(30), voice(12, 0), echo(30)],
            31 => vec![voice(4, 0), voice(1, 2)],
            _ => unreachable!(),
        }
    }

    #[test]
    fn manifest_exhausts_phase_debt_and_f3_hazard_matrix() {
        let cases = manifest();
        assert_eq!(cases.len(), CASE_COUNT);
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.group == "phase-chain")
                .count(),
            512
        );
        assert_eq!(
            cases.iter().filter(|case| case.group == "f3-read").count(),
            4160
        );
        assert_eq!(
            cases.iter().filter(|case| case.group == "f3-write").count(),
            768
        );
        for address in [0x0c, 0x28, 0x29, 0x4c, 0x7c] {
            assert_eq!(
                cases
                    .iter()
                    .filter(|case| matches!(case.operation, Operation::Write)
                        && case.address == address)
                    .count(),
                128
            );
        }
        assert_eq!(
            cases
                .iter()
                .filter(|case| matches!(case.operation, Operation::BlockedWrite)
                    && case.address == 0xaa)
                .count(),
            128
        );
    }

    #[test]
    fn core_receipt_binds_dylib_source_lock_and_ordered_patch_bytes() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let core = std::env::temp_dir().join(format!(
            "zelda3-dsp-ledger-core-receipt-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::write(&core, b"controlled trace core bytes").unwrap();
        let patch_paths = PINNED_PATCH_NAMES
            .iter()
            .map(|name| {
                repo_root
                    .join("external/snes9x-libretro/patches")
                    .join(name)
            })
            .collect::<Vec<_>>();
        let patch_bytes = patch_paths
            .iter()
            .map(|path| fs::read(path).unwrap())
            .collect::<Vec<_>>();
        let patch_sha256s = patch_bytes
            .iter()
            .map(|bytes| parity::evidence::sha256_bytes(bytes))
            .collect::<Vec<_>>();
        let mut aggregate = Vec::new();
        for (name, bytes) in PINNED_PATCH_NAMES.iter().zip(&patch_bytes) {
            aggregate.extend_from_slice(name.as_bytes());
            aggregate.push(0);
            aggregate.extend_from_slice(bytes);
            aggregate.push(0);
        }
        let lock = repo_root.join("external/snes9x-libretro/oracle-lock.json");
        let mut receipt = json!({
            "variant": "trace",
            "core_version": "1.63",
            "source_tag": "1.63",
            "source_revision": PINNED_SOURCE_REVISION,
            "core": fs::canonicalize(&core).unwrap(),
            "core_sha256": parity::evidence::sha256_file(&core).unwrap(),
            "oracle_lock_sha256": parity::evidence::sha256_file(&lock).unwrap(),
            "patches": patch_paths,
            "patch_sha256s": patch_sha256s,
            "patch_sha256": parity::evidence::sha256_bytes(&aggregate),
        });
        assert_eq!(verify_core_receipt(&core, &receipt).unwrap().len(), 4);
        receipt["core_sha256"] = Value::from("stale");
        assert!(verify_core_receipt(&core, &receipt).is_err());
        receipt["core_sha256"] = Value::from(parity::evidence::sha256_file(&core).unwrap());
        receipt["patch_sha256s"][3] = Value::from("stale");
        assert!(verify_core_receipt(&core, &receipt).is_err());
        fs::remove_file(core).unwrap();
    }

    #[test]
    fn compact_fixture_is_canonical_and_reconstructable() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../")
            .join(FIXTURE_RELATIVE_PATH);
        let lines =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let records = lines
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);
        assert_eq!(
            records[0]["schema"],
            "pinned-snes9x-spc-dsp-phase-ledger-v1"
        );
        assert_eq!(records[0]["abi_version"], ABI_VERSION);
        assert_eq!(records[0]["source"]["case_count"], CASE_COUNT);
        assert_eq!(records[0]["source"]["checkpoint_count"], 514);
        assert_eq!(
            records[0]["source"]["branch_source_map"],
            json!(BRANCH_SOURCE_MAP)
        );
        assert_eq!(
            records[0]["source"]["suboperation_source_map"],
            json!(SUBOPERATION_SOURCE_MAP)
        );
        for field in [
            "branch_evaluated_mask_union",
            "branch_taken_mask_union",
            "branch_not_taken_mask_union",
        ] {
            assert_eq!(records[0]["source"][field], REQUIRED_BRANCH_MASK);
        }
        assert_eq!(records[0]["source"]["separate_echo_buffer"], 0);
        assert_eq!(records[0]["source"]["stereo_switch"], 65535);
        assert_eq!(records[0]["source"]["mute_mask"], 0);
        assert_eq!(records[0]["source"]["interpolation_method"], 2);
        assert_eq!(records[0]["source"]["copy_state_capture_neutral"], true);
        assert_eq!(
            records[0]["source"]["ordered_patch_sha256"]
                .as_array()
                .unwrap()
                .len(),
            4
        );

        let checkpoints = records[1]["checkpoints"].as_array().unwrap();
        assert_eq!(checkpoints.len(), 514);
        for (checkpoint_id, checkpoint) in checkpoints.iter().enumerate() {
            let profile = checkpoint_id / 257;
            let tick = checkpoint_id % 257;
            assert_eq!(checkpoint["id"], checkpoint_id);
            assert_eq!(checkpoint["profile"], profile);
            assert_eq!(checkpoint["tick"], tick);
            assert_eq!(checkpoint["phase"], tick & 31);
        }
        let checkpoint_xor = expand_delta_sequence(&records[1]["checkpoint_state_xor_sequence"]);
        assert_eq!(checkpoint_xor.len(), 514 * EXPECTED_STATE_SIZE);
        let mut current = vec![0u8; EXPECTED_STATE_SIZE];
        let mut checkpoint_states = Vec::with_capacity(514);
        for (row_index, row) in checkpoint_xor.iter().enumerate() {
            let checkpoint_id = row_index / EXPECTED_STATE_SIZE;
            let byte_index = row_index % EXPECTED_STATE_SIZE;
            assert_eq!(row[0], checkpoint_id as i64);
            assert_eq!(row[1], byte_index as i64);
            current[byte_index] ^= row[2] as u8;
            if byte_index + 1 == EXPECTED_STATE_SIZE {
                assert_eq!(
                    checkpoints[checkpoint_id]["state_sha256"],
                    parity::evidence::sha256_bytes(&current)
                );
                checkpoint_states.push(current.clone());
            }
        }

        let initial_diffs = expand_delta_sequence(&records[1]["profile_initial_ram_diff_sequence"]);
        let ram_diffs = expand_delta_sequence(&records[1]["ram_diff_sequence"]);
        let mut checkpoint_rams = Vec::with_capacity(514);
        for profile in 0..=1usize {
            let mut ram = seeded_ram(profile as u8);
            let mut previous_address = None;
            for row in initial_diffs.iter().filter(|row| row[0] == profile as i64) {
                let address = row[1] as usize;
                assert!(previous_address.is_none_or(|prior| prior < address));
                previous_address = Some(address);
                ram[address] = row[2] as u8;
            }
            for tick in 0..=256usize {
                let checkpoint_id = profile * 257 + tick;
                assert_eq!(
                    checkpoints[checkpoint_id]["ram_sha256"],
                    parity::evidence::sha256_bytes(&ram)
                );
                checkpoint_rams.push(ram.clone());
                if tick < 256 {
                    let case_id = profile * 256 + tick;
                    let mut previous_address = None;
                    for row in ram_diffs.iter().filter(|row| row[0] == case_id as i64) {
                        let address = row[1] as usize;
                        assert!(previous_address.is_none_or(|prior| prior < address));
                        previous_address = Some(address);
                        ram[address] = row[2] as u8;
                    }
                }
            }
        }

        let manifest_cases = manifest();
        let cases = expand_delta_sequence(&records[1]["case_sequence"]);
        assert_eq!(cases.len(), CASE_COUNT);
        assert!(records[1].get("cases").is_none());
        let state = expand_delta_sequence(&records[1]["copy_state_xor_sequence"]);
        assert_eq!(state.len(), CASE_COUNT * EXPECTED_STATE_SIZE);
        let mut current = vec![0u8; EXPECTED_STATE_SIZE];
        let mut case_states = Vec::with_capacity(CASE_COUNT);
        for (row_index, row) in state.iter().enumerate() {
            let case_id = row_index / EXPECTED_STATE_SIZE;
            let byte_index = row_index % EXPECTED_STATE_SIZE;
            assert_eq!(row[0], case_id as i64);
            assert_eq!(row[1], byte_index as i64);
            current[byte_index] ^= row[2] as u8;
            if byte_index + 1 == EXPECTED_STATE_SIZE {
                case_states.push(current.clone());
            }
        }
        let samples = expand_delta_sequence(&records[1]["sample_sequence"]);
        let suboperations = expand_delta_sequence(&records[1]["suboperation_sequence"]);
        assert_eq!(
            cases.iter().map(|row| row[19] as usize).sum::<usize>(),
            ram_diffs.len()
        );
        assert_eq!(
            cases.iter().map(|row| row[20] as usize).sum::<usize>(),
            samples.len()
        );
        assert_eq!(
            cases.iter().map(|row| row[21] as usize).sum::<usize>(),
            suboperations.len()
        );
        for (case_id, row) in cases.iter().enumerate() {
            let case = &manifest_cases[case_id];
            let expected_group = match case_id {
                0..512 => "phase-chain",
                512..4672 => "f3-read",
                4672..CASE_COUNT => "f3-write",
                _ => unreachable!(),
            };
            assert_eq!(case.group, expected_group);
            assert_eq!(row[0], case_id as i64);
            assert_eq!(row[1], i64::from(case.profile));
            assert_eq!(row[3], i64::from(case.start_phase));
            assert_eq!(row[4], i64::from(case.debt));
            assert_eq!(row[5], i64::from(case.operation.abi()));
            assert_eq!(row[6], i64::from(case.address));
            assert_eq!(row[7], i64::from(case.value));
            assert!((0..514).contains(&row[2]));
            assert_eq!(row[10], 0);
            assert_eq!(row[18], EXPECTED_STATE_SIZE as i64);
            if row[5] == Operation::BlockedWrite.abi() as i64 {
                assert_eq!(row[11], row[4]);
                assert_eq!(row[9], row[3]);
                assert_eq!(case_states[case_id], checkpoint_states[row[2] as usize]);
            } else {
                assert_eq!(row[11], 0);
                assert_eq!(row[9], (row[3] + row[4]) & 31);
                if row[5] == Operation::Read.abi() as i64 && row[4] == 0 {
                    assert_eq!(case_states[case_id], checkpoint_states[row[2] as usize]);
                }
            }
            let mut post_ram = checkpoint_rams[row[2] as usize].clone();
            let mut previous_address = None;
            for difference in ram_diffs
                .iter()
                .filter(|difference| difference[0] == case_id as i64)
            {
                let address = difference[1] as usize;
                assert!(previous_address.is_none_or(|prior| prior < address));
                previous_address = Some(address);
                post_ram[address] = difference[2] as u8;
            }
            if case_id < 512 {
                assert_eq!(
                    parity::evidence::sha256_bytes(&post_ram),
                    checkpoints[row[2] as usize + 1]["ram_sha256"]
                );
            }
        }
        for address in [0x0c, 0x28, 0x29, 0x4c, 0x7c] {
            assert!(cases.iter().enumerate().any(|(case_id, row)| {
                row[5] == Operation::Write.abi() as i64
                    && row[6] == address
                    && row[4] == 0
                    && case_states[case_id] != checkpoint_states[row[2] as usize]
            }));
        }
        for phase in 0..32usize {
            let actual = suboperations
                .iter()
                .filter(|row| row[0] == phase as i64)
                .map(|row| (row[2], row[3], row[4]))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected_phase_suboperations(phase), "phase {phase}");
        }
        assert!(fs::metadata(path).unwrap().len() <= ENCODED_SIZE_CAP);
    }
}
