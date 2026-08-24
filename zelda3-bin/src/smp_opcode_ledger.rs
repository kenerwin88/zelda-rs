//! Controlled, trace-only pinned-Snes9x SPC700 opcode ledger generator.

use serde::Serialize;
use serde_json::{json, Value};
use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};

use crate::snes9x_compare::{
    acquire_snes9x_compare_lock, compact_delta_integer_sequence_with_zstd,
};

const ABI_VERSION: i32 = 1;
const SEED_FIELD_COUNT: usize = 31;
const STATE_FIELD_COUNT: usize = 174;
const SCALAR_STATE_FIELD_COUNT: usize = 46;
const BUS_FIELD_COUNT: usize = 6;
const PC: u16 = 0x0200;
#[cfg(test)]
const FIXTURE_RELATIVE_PATH: &str =
    "external/snes9x-libretro/fixtures/snes9x-spc700-op-step-ledger.jsonl";

const SPLIT_STAGES: &[(u8, u8)] = &[
    (0x7e, 2),
    (0x8f, 3),
    (0xaa, 2),
    (0xaf, 2),
    (0xba, 3),
    (0xbf, 2),
    (0xc4, 3),
    (0xc5, 4),
    (0xc6, 3),
    (0xc7, 5),
    (0xc9, 4),
    (0xca, 3),
    (0xcb, 3),
    (0xcc, 4),
    (0xd4, 3),
    (0xd5, 3),
    (0xd6, 3),
    (0xd7, 5),
    (0xd8, 3),
    (0xd9, 3),
    (0xda, 4),
    (0xdb, 3),
    (0xe4, 2),
    (0xe5, 3),
    (0xe6, 2),
    (0xe7, 4),
    (0xe9, 2),
    (0xeb, 2),
    (0xec, 2),
    (0xf4, 2),
    (0xf5, 2),
    (0xf6, 2),
    (0xf7, 4),
    (0xf8, 2),
    (0xf9, 2),
    (0xfa, 4),
    (0xfb, 2),
];

const STATE_FIELDS: [&str; SCALAR_STATE_FIELD_COUNT + 2] = [
    "case_id",
    "stage",
    "state_case_id",
    "pc",
    "a",
    "x",
    "y",
    "sp",
    "psw",
    "opcode_number",
    "opcode_cycle",
    "scratch_rd",
    "scratch_wr",
    "scratch_dp",
    "scratch_sp",
    "scratch_ya",
    "scratch_bit",
    "smp_clock",
    "dsp_clock",
    "ipl_enabled",
    "dsp_address",
    "ram_f8",
    "ram_f9",
    "timer0_enabled",
    "timer0_target",
    "timer0_stage1",
    "timer0_stage2",
    "timer0_stage3",
    "timer1_enabled",
    "timer1_target",
    "timer1_stage1",
    "timer1_stage2",
    "timer1_stage3",
    "timer2_enabled",
    "timer2_target",
    "timer2_stage1",
    "timer2_stage2",
    "timer2_stage3",
    "cpu_port0",
    "cpu_port1",
    "cpu_port2",
    "cpu_port3",
    "smp_port0",
    "smp_port1",
    "smp_port2",
    "smp_port3",
    "step_index",
    "overflow",
];

#[derive(Clone, Debug, Serialize)]
struct LedgerCase {
    id: u16,
    group: &'static str,
    opcode: u8,
    variant: String,
    expected_stages: u8,
    branch_obligations: Vec<String>,
    seed: [i32; SEED_FIELD_COUNT],
    ram_overrides: Vec<[u16; 2]>,
    dsp_overrides: Vec<[u8; 2]>,
}

impl LedgerCase {
    fn base(id: u16, opcode: u8) -> Self {
        let mut seed = [0; SEED_FIELD_COUNT];
        seed[0] = i32::from(PC);
        seed[1] = 0x81;
        seed[2] = 3;
        seed[3] = 5;
        seed[4] = 0xef;
        seed[5] = 0;
        seed[6] = 0;
        seed[7] = 0x2a;
        seed[8] = 0x58;
        seed[9] = 0x59;
        seed[10] = 0;
        seed[11] = 0;
        // Each timer is enabled but deliberately away from a rollover in the
        // canonical matrix. Dedicated supplement cases own timer boundaries.
        seed[12..27].copy_from_slice(&[1, 5, 11, 2, 3, 1, 7, 19, 3, 4, 1, 9, 5, 4, 5]);
        seed[27..31].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);
        let mut case = Self {
            id,
            group: "opcode",
            opcode,
            variant: "canonical".to_string(),
            expected_stages: split_stage_count(opcode),
            branch_obligations: Vec::new(),
            seed,
            ram_overrides: Vec::new(),
            dsp_overrides: Vec::new(),
        };
        case.patch_common_program();
        case
    }

    fn patch_common_program(&mut self) {
        for (address, value) in [
            (PC, self.opcode),
            (PC + 1, 0x40),
            (PC + 2, 0x03),
            (PC + 3, 0x7f),
            (0x0003, 0x40),
            (0x0004, 0x03),
            (0x0040, 0x81),
            (0x0041, 0x03),
            (0x0043, 0x40),
            (0x0044, 0x03),
            (0x0140, 0x42),
            (0x0141, 0x03),
            (0x0340, 0x42),
            (0x0341, 0x24),
        ] {
            self.set_ram(address, value);
        }
    }

    fn set_ram(&mut self, address: u16, value: u8) {
        if let Some(entry) = self
            .ram_overrides
            .iter_mut()
            .find(|entry| entry[0] == address)
        {
            entry[1] = u16::from(value);
        } else {
            self.ram_overrides.push([address, u16::from(value)]);
        }
    }

    fn set_psw_flag(&mut self, mask: u8, enabled: bool) {
        if enabled {
            self.seed[5] |= i32::from(mask);
        } else {
            self.seed[5] &= !i32::from(mask);
        }
    }
}

fn split_stage_count(opcode: u8) -> u8 {
    SPLIT_STAGES
        .iter()
        .find_map(|&(candidate, stages)| (candidate == opcode).then_some(stages))
        .unwrap_or(1)
}

fn clone_variant(
    cases: &mut Vec<LedgerCase>,
    base_opcode: u8,
    name: impl Into<String>,
    obligations: &[&str],
    update: impl FnOnce(&mut LedgerCase),
) {
    let mut case = LedgerCase::base(cases.len() as u16, base_opcode);
    case.variant = name.into();
    case.branch_obligations = obligations
        .iter()
        .map(|value| (*value).to_string())
        .collect();
    update(&mut case);
    cases.push(case);
}

fn opcode_cases() -> Vec<LedgerCase> {
    let mut cases = (0u16..=255)
        .map(|opcode| LedgerCase::base(opcode, opcode as u8))
        .collect::<Vec<_>>();

    for &(opcode, flag_mask) in &[
        (0xf0, 0x02),
        (0xd0, 0x02),
        (0xb0, 0x01),
        (0x90, 0x01),
        (0x70, 0x40),
        (0x50, 0x40),
        (0x30, 0x80),
        (0x10, 0x80),
    ] {
        clone_variant(
            &mut cases,
            opcode,
            "condition-inverted",
            &["conditional-branch-opposite"],
            |case| case.set_psw_flag(flag_mask, true),
        );
    }

    for opcode in (0x03u8..=0xf3).step_by(0x10) {
        let bit = opcode >> 5;
        clone_variant(
            &mut cases,
            opcode,
            "bit-condition-inverted",
            &["bit-branch-opposite"],
            |case| case.set_ram(0x0040, 0x81 ^ (1 << bit)),
        );
    }

    for opcode in [0x2e, 0xde] {
        clone_variant(
            &mut cases,
            opcode,
            "compare-equal",
            &["compare-branch-opposite"],
            |case| case.set_ram(0x0040, case.seed[1] as u8),
        );
    }
    clone_variant(
        &mut cases,
        0x6e,
        "decrement-to-zero",
        &["decrement-branch-opposite"],
        |case| case.set_ram(0x0040, 1),
    );
    clone_variant(
        &mut cases,
        0xfe,
        "decrement-y-to-zero",
        &["decrement-y-branch-opposite"],
        |case| case.seed[3] = 1,
    );
    clone_variant(
        &mut cases,
        0x9e,
        "division-upper-path",
        &["division-path-opposite"],
        |case| {
            case.seed[2] = 3;
            case.seed[3] = 10;
        },
    );

    for (opcode, variants) in [
        (0xdf, [(true, true), (true, false), (false, true)]),
        (0xbe, [(true, true), (true, false), (false, true)]),
    ] {
        for (first, second) in variants {
            clone_variant(
                &mut cases,
                opcode,
                format!("decimal-first-{first}-second-{second}"),
                &["decimal-first", "decimal-second"],
                |case| {
                    case.seed[1] = 1;
                    if opcode == 0xdf {
                        case.set_psw_flag(0x01, first);
                        case.set_psw_flag(0x08, second);
                    } else {
                        case.set_psw_flag(0x01, !first);
                        case.set_psw_flag(0x08, !second);
                    }
                },
            );
        }
    }

    clone_variant(
        &mut cases,
        0xca,
        "carry-set",
        &["split-carry-opposite"],
        |case| case.set_psw_flag(0x01, true),
    );
    assert_eq!(cases.len(), 292);
    cases
}

fn supplement_cases(start_id: u16) -> Vec<LedgerCase> {
    let mut cases = Vec::new();
    let read_addresses = [
        0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfd, 0xfe, 0xff, 0xf0,
    ];
    for address in read_addresses {
        let mut case = LedgerCase::base(start_id + cases.len() as u16, 0x78);
        case.group = "mmio-bus";
        case.variant = format!("read-{address:02x}");
        case.set_ram(PC + 1, 0x5a);
        case.set_ram(PC + 2, address);
        cases.push(case);
    }

    let writes = [
        (0xf1, 0x00, "control-none"),
        (0xf1, 0x10, "control-clear-low"),
        (0xf1, 0x20, "control-clear-high"),
        (0xf1, 0x30, "control-clear-both"),
        (0xf1, 0x07, "control-enable-timers"),
        (0xf1, 0x07, "control-retain-enabled-timers"),
        (0xf2, 0x35, "dsp-address"),
        (0xf3, 0x66, "dsp-write-accepted"),
        (0xf3, 0x77, "dsp-write-blocked"),
        (0xf4, 0x14, "port0"),
        (0xf5, 0x25, "port1"),
        (0xf6, 0x36, "port2"),
        (0xf7, 0x47, "port3"),
        (0xf8, 0x58, "ram-f8"),
        (0xf9, 0x69, "ram-f9"),
        (0xfa, 0x0a, "timer0-target"),
        (0xfb, 0x0b, "timer1-target"),
        (0xfc, 0x0c, "timer2-target"),
        (0xfd, 0x7d, "unhandled-write"),
    ];
    for (address, value, name) in writes {
        let mut case = LedgerCase::base(start_id + cases.len() as u16, 0x8f);
        case.group = "mmio-bus";
        case.variant = name.to_string();
        case.set_ram(PC + 1, value);
        case.set_ram(PC + 2, address);
        if name == "control-retain-enabled-timers" {
            case.seed[12] = 1;
            case.seed[17] = 1;
            case.seed[22] = 1;
        }
        if name == "dsp-write-blocked" {
            case.seed[7] = 0xaa;
        } else if name == "dsp-write-accepted" {
            case.seed[7] = 0x2a;
        }
        cases.push(case);
    }

    let mut ipl = LedgerCase::base(start_id + cases.len() as u16, 0xe5);
    ipl.group = "mmio-bus";
    ipl.variant = "ipl-overlay-enabled".to_string();
    ipl.seed[6] = 1;
    ipl.set_ram(PC + 1, 0xc0);
    ipl.set_ram(PC + 2, 0xff);
    cases.push(ipl);

    for (name, opcode, timer_seed) in [
        (
            "timer-unit-rollovers",
            0x00,
            [0, 5, 127, 2, 3, 1, 7, 127, 3, 4, 1, 2, 15, 1, 5],
        ),
        (
            "timer-multi-rollovers",
            0xef,
            [1, 5, 125, 2, 3, 0, 7, 125, 3, 4, 1, 2, 13, 1, 5],
        ),
        (
            "timer-counter-wrap",
            0x00,
            [1, 3, 127, 2, 15, 1, 4, 127, 3, 15, 1, 2, 15, 1, 15],
        ),
    ] {
        let mut case = LedgerCase::base(start_id + cases.len() as u16, opcode);
        case.group = "mmio-bus";
        case.variant = name.to_string();
        case.seed[12..27].copy_from_slice(&timer_seed);
        cases.push(case);
    }
    assert_eq!(cases.len(), 35);
    cases
}

type NoArg = unsafe extern "C" fn() -> c_int;
type OneArg = unsafe extern "C" fn(c_int) -> c_int;
type TwoArg = unsafe extern "C" fn(c_int, c_int) -> c_int;
type ThreeArg = unsafe extern "C" fn(c_int, *const c_int, c_int) -> c_int;

struct LedgerCore {
    handle: *mut c_void,
    abi_version: NoArg,
    seed_field_count: NoArg,
    state_field_count: NoArg,
    begin: ThreeArg,
    set_ram: TwoArg,
    set_dsp: TwoArg,
    seal: NoArg,
    step: NoArg,
    state_value: OneArg,
    bus_count: NoArg,
    bus_value: unsafe extern "C" fn(c_int, c_int) -> c_int,
    ram_diff_count: NoArg,
    ram_diff_value: unsafe extern "C" fn(c_int, c_int) -> c_int,
    dsp_diff_count: NoArg,
    dsp_diff_value: unsafe extern "C" fn(c_int, c_int) -> c_int,
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
                abi_version: symbol(handle, "zelda3_snes9x_debug_smp_ledger_abi_version")?,
                seed_field_count: symbol(
                    handle,
                    "zelda3_snes9x_debug_smp_ledger_seed_field_count",
                )?,
                state_field_count: symbol(
                    handle,
                    "zelda3_snes9x_debug_smp_ledger_state_field_count",
                )?,
                begin: symbol(handle, "zelda3_snes9x_debug_smp_ledger_begin")?,
                set_ram: symbol(handle, "zelda3_snes9x_debug_smp_ledger_set_ram")?,
                set_dsp: symbol(handle, "zelda3_snes9x_debug_smp_ledger_set_dsp")?,
                seal: symbol(handle, "zelda3_snes9x_debug_smp_ledger_seal")?,
                step: symbol(handle, "zelda3_snes9x_debug_smp_ledger_step")?,
                state_value: symbol(handle, "zelda3_snes9x_debug_smp_ledger_state_value")?,
                bus_count: symbol(handle, "zelda3_snes9x_debug_smp_ledger_bus_count")?,
                bus_value: symbol(handle, "zelda3_snes9x_debug_smp_ledger_bus_value")?,
                ram_diff_count: symbol(handle, "zelda3_snes9x_debug_smp_ledger_ram_diff_count")?,
                ram_diff_value: symbol(handle, "zelda3_snes9x_debug_smp_ledger_ram_diff_value")?,
                dsp_diff_count: symbol(handle, "zelda3_snes9x_debug_smp_ledger_dsp_diff_count")?,
                dsp_diff_value: symbol(handle, "zelda3_snes9x_debug_smp_ledger_dsp_diff_value")?,
                overflow: symbol(handle, "zelda3_snes9x_debug_smp_ledger_overflow")?,
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

#[derive(Default)]
struct LedgerRows {
    states: Vec<[i64; SCALAR_STATE_FIELD_COUNT + 2]>,
    dsp: Vec<[i64; 4]>,
    bus: Vec<[i64; BUS_FIELD_COUNT + 2]>,
    ram_diffs: Vec<[i64; 4]>,
    dsp_diffs: Vec<[i64; 4]>,
}

fn capture(core: &LedgerCore, cases: &[LedgerCase]) -> Result<LedgerRows, String> {
    if unsafe { (core.abi_version)() } != ABI_VERSION
        || unsafe { (core.seed_field_count)() } as usize != SEED_FIELD_COUNT
        || unsafe { (core.state_field_count)() } as usize != STATE_FIELD_COUNT
    {
        return Err("trace core SPC ledger ABI/schema mismatch".to_string());
    }
    let mut rows = LedgerRows::default();
    for case in cases {
        if unsafe {
            (core.begin)(
                i32::from(case.id),
                case.seed.as_ptr(),
                SEED_FIELD_COUNT as i32,
            )
        } != 1
        {
            return Err(format!("case {} begin failed", case.id));
        }
        for &[address, value] in &case.ram_overrides {
            if unsafe { (core.set_ram)(i32::from(address), i32::from(value)) } != 1 {
                return Err(format!("case {} RAM seed failed", case.id));
            }
        }
        for &[address, value] in &case.dsp_overrides {
            if unsafe { (core.set_dsp)(i32::from(address), i32::from(value)) } != 1 {
                return Err(format!("case {} DSP seed failed", case.id));
            }
        }
        if unsafe { (core.seal)() } != 1 {
            return Err(format!("case {} seal failed", case.id));
        }
        for stage in 0..case.expected_stages {
            let opcode_cycle = unsafe { (core.step)() };
            let expected_cycle = if stage + 1 == case.expected_stages {
                0
            } else {
                i32::from(stage + 1)
            };
            if opcode_cycle != expected_cycle || unsafe { (core.overflow)() } != 0 {
                return Err(format!(
                    "case {} stage {stage}: opcode_cycle {opcode_cycle}, expected {expected_cycle}",
                    case.id
                ));
            }
            let mut state = [0i64; SCALAR_STATE_FIELD_COUNT + 2];
            state[0] = i64::from(case.id);
            state[1] = i64::from(stage);
            for field in 0..SCALAR_STATE_FIELD_COUNT {
                state[field + 2] = i64::from(unsafe { (core.state_value)(field as i32) });
            }
            rows.states.push(state);
            for address in 0..128 {
                rows.dsp.push([
                    i64::from(case.id),
                    i64::from(stage),
                    address as i64,
                    i64::from(unsafe {
                        (core.state_value)((SCALAR_STATE_FIELD_COUNT + address) as i32)
                    }),
                ]);
            }
            let bus_count = unsafe { (core.bus_count)() };
            for event in 0..bus_count {
                let mut row = [0i64; BUS_FIELD_COUNT + 2];
                row[0] = i64::from(case.id);
                row[1] = i64::from(stage);
                for field in 0..BUS_FIELD_COUNT {
                    row[field + 2] = i64::from(unsafe { (core.bus_value)(event, field as i32) });
                }
                rows.bus.push(row);
            }
            for difference in 0..unsafe { (core.ram_diff_count)() } {
                rows.ram_diffs.push([
                    i64::from(case.id),
                    i64::from(stage),
                    i64::from(unsafe { (core.ram_diff_value)(difference, 0) }),
                    i64::from(unsafe { (core.ram_diff_value)(difference, 1) }),
                ]);
            }
            for difference in 0..unsafe { (core.dsp_diff_count)() } {
                rows.dsp_diffs.push([
                    i64::from(case.id),
                    i64::from(stage),
                    i64::from(unsafe { (core.dsp_diff_value)(difference, 0) }),
                    i64::from(unsafe { (core.dsp_diff_value)(difference, 1) }),
                ]);
            }
        }
    }
    Ok(rows)
}

fn write_fixture(core_path: &Path, output: &Path) -> Result<(), String> {
    let receipt_path = PathBuf::from(format!("{}.json", core_path.display()));
    let receipt: Value = serde_json::from_slice(
        &fs::read(&receipt_path).map_err(|error| format!("{}: {error}", receipt_path.display()))?,
    )
    .map_err(|error| format!("{}: {error}", receipt_path.display()))?;
    if receipt["variant"] != "trace" {
        return Err("SPC ledger generation requires a traced pinned core".to_string());
    }

    let mut cases = opcode_cases();
    cases.extend(supplement_cases(cases.len() as u16));
    assert_eq!(cases.len(), 327);
    assert_eq!(
        cases
            .iter()
            .map(|case| usize::from(case.expected_stages))
            .sum::<usize>(),
        439
    );
    let core = LedgerCore::load(core_path)?;
    let rows = capture(&core, &cases)?;
    assert_eq!(rows.states.len(), 439);
    assert_eq!(rows.dsp.len(), 439 * 128);

    let provenance = json!({
        "kind": "provenance",
        "schema": "pinned-snes9x-spc700-op-step-ledger-v1",
        "abi_version": ABI_VERSION,
        "core": receipt,
        "classifier": {
            "atomic_opcode_count": 219,
            "split_opcode_count": 37,
            "split_opcodes_and_stages": SPLIT_STAGES,
            "source_control_flow_case_count": 292,
            "supplement_case_count": 35,
            "total_case_count": 327,
            "total_stage_count": 439,
        },
        "seed": {
            "ram": "byte=(address*37+(address>>8)*13+case_id*17+0x5a)&0xff, then sparse overrides",
            "pc": PC,
            "seed_field_count": SEED_FIELD_COUNT,
            "state_field_count": STATE_FIELD_COUNT,
        }
    });
    let data = json!({
        "kind": "spc-op-step-ledger",
        "cases": cases,
        "state_sequence": compact_delta_integer_sequence_with_zstd(STATE_FIELDS, rows.states, true),
        "dsp_state_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id", "stage", "address", "value"], rows.dsp, true),
        "bus_event_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id", "stage", "kind", "address", "value", "clocks", "clock_before", "clock_after"], rows.bus, true),
        "ram_diff_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id", "stage", "address", "value"], rows.ram_diffs, true),
        "dsp_diff_sequence": compact_delta_integer_sequence_with_zstd(
            ["case_id", "stage", "address", "value"], rows.dsp_diffs, true),
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
    Ok(())
}

pub(crate) fn run_generate_command(args: &[String]) {
    if args.len() != 2 {
        eprintln!("usage: --generate-snes9x-smp-opcode-ledger TRACE_CORE OUTPUT");
        std::process::exit(2);
    }
    let _lane = acquire_snes9x_compare_lock();
    if let Err(error) = write_fixture(Path::new(&args[0]), Path::new(&args[1])) {
        eprintln!("failed to generate pinned Snes9x SPC ledger: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snes9x_compare::tests::expand_delta_sequence;

    #[test]
    fn manifest_exhausts_source_classifier_and_branch_matrix() {
        assert_eq!(SPLIT_STAGES.len(), 37);
        assert_eq!(
            SPLIT_STAGES
                .iter()
                .map(|entry| usize::from(entry.1))
                .sum::<usize>(),
            107
        );
        let cases = opcode_cases();
        assert_eq!(cases.len(), 292);
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.variant == "canonical")
                .count(),
            256
        );
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.expected_stages == 1)
                .count(),
            254
        );
        assert_eq!(
            cases
                .iter()
                .map(|case| usize::from(case.expected_stages))
                .sum::<usize>(),
            364
        );
        assert_eq!(supplement_cases(292).len(), 35);
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
            "pinned-snes9x-spc700-op-step-ledger-v1"
        );
        assert_eq!(records[0]["classifier"]["atomic_opcode_count"], 219);
        assert_eq!(records[0]["classifier"]["split_opcode_count"], 37);
        assert_eq!(
            records[0]["classifier"]["split_opcodes_and_stages"],
            serde_json::to_value(SPLIT_STAGES).unwrap()
        );
        assert_eq!(records[0]["classifier"]["total_case_count"], 327);
        assert_eq!(records[0]["classifier"]["total_stage_count"], 439);
        assert_eq!(records[0]["abi_version"], ABI_VERSION);
        assert_eq!(records[0]["seed"]["seed_field_count"], SEED_FIELD_COUNT);
        assert_eq!(records[0]["seed"]["state_field_count"], STATE_FIELD_COUNT);
        assert_eq!(records[0]["core"]["variant"], "trace");
        assert_eq!(records[0]["core"]["source_tag"], "1.63");
        assert_eq!(
            records[0]["core"]["source_revision"],
            "921f9f7b83660eb44ad263022a57a4a029057c37"
        );
        assert_eq!(
            records[0]["core"]["core_sha256"].as_str().unwrap().len(),
            64
        );
        assert_eq!(records[0]["core"]["patches"].as_array().unwrap().len(), 3);
        assert!(records[0]["core"]["patches"][2]
            .as_str()
            .unwrap()
            .ends_with("zelda3-spc-opcode-ledger.patch"));
        assert_eq!(records[1]["cases"].as_array().unwrap().len(), 327);
        assert_eq!(records[1]["state_sequence"]["record_count"], 439);
        assert_eq!(records[1]["dsp_state_sequence"]["record_count"], 439 * 128);
        for key in [
            "state_sequence",
            "dsp_state_sequence",
            "bus_event_sequence",
            "ram_diff_sequence",
            "dsp_diff_sequence",
        ] {
            assert_eq!(
                records[1][key]["encoding"],
                "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
            );
            assert_eq!(
                records[1][key]["expanded_sha256"].as_str().unwrap().len(),
                64
            );
        }

        let state_rows = expand_delta_sequence(&records[1]["state_sequence"]);
        let dsp_rows = expand_delta_sequence(&records[1]["dsp_state_sequence"]);
        let bus_rows = expand_delta_sequence(&records[1]["bus_event_sequence"]);
        let ram_diff_rows = expand_delta_sequence(&records[1]["ram_diff_sequence"]);
        let dsp_diff_rows = expand_delta_sequence(&records[1]["dsp_diff_sequence"]);
        assert_eq!(state_rows.len(), 439);
        assert_eq!(dsp_rows.len(), 439 * 128);
        assert!(!bus_rows.is_empty());
        assert!(!ram_diff_rows.is_empty());
        assert!(!dsp_diff_rows.is_empty());

        let mut state_index = 0;
        for case in records[1]["cases"].as_array().unwrap() {
            let case_id = case["id"].as_i64().unwrap();
            let opcode = case["opcode"].as_i64().unwrap();
            let expected_stages = case["expected_stages"].as_u64().unwrap() as usize;
            for stage in 0..expected_stages {
                let row = &state_rows[state_index];
                assert_eq!(row[0], case_id);
                assert_eq!(row[1], stage as i64);
                assert_eq!(row[2], case_id);
                assert_eq!(row[9], opcode);
                assert_eq!(
                    row[10],
                    if stage + 1 == expected_stages {
                        0
                    } else {
                        (stage + 1) as i64
                    }
                );
                state_index += 1;
            }
        }
        assert_eq!(state_index, state_rows.len());

        for (state_row, dsp_chunk) in state_rows.iter().zip(dsp_rows.chunks_exact(128)) {
            for (address, dsp_row) in dsp_chunk.iter().enumerate() {
                assert_eq!(dsp_row[0], state_row[0]);
                assert_eq!(dsp_row[1], state_row[1]);
                assert_eq!(dsp_row[2], address as i64);
            }
        }
    }
}
