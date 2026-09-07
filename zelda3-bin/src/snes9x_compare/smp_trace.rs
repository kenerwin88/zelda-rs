//! Split out of `snes9x_compare.rs` by topic (smp_trace). Mechanical move:
//! bodies are unchanged; private items became `pub(crate)` and `super::`
//! paths became `crate::` (the parent's super is the crate root).

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SmpBootstrapInstructionStep {
    pub(crate) absolute_start_cycle: u64,
    pub(crate) absolute_end_cycle: u64,
    pub(crate) origin_pc: i32,
    pub(crate) opcode: i32,
    pub(crate) boundary_opcode_cycle: i32,
    pub(crate) op_step_calls: i32,
    pub(crate) max_continuation_opcode_cycle: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SmpBootstrapPatternInstruction {
    pub(crate) start_cycle_offset: u64,
    pub(crate) end_cycle_offset: u64,
    pub(crate) origin_pc: i32,
    pub(crate) opcode: i32,
    pub(crate) boundary_opcode_cycle: i32,
    pub(crate) op_step_calls: i32,
    pub(crate) max_continuation_opcode_cycle: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SmpBootstrapInstructionSpan {
    pub(crate) absolute_start_cycle: u64,
    pub(crate) absolute_end_cycle: u64,
    pub(crate) repeat_count: usize,
    pub(crate) repeat_cycle_stride: u64,
    pub(crate) instructions: Vec<SmpBootstrapPatternInstruction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SmpBootstrapInstructionSequence {
    pub(crate) encoding: &'static str,
    pub(crate) instruction_count: usize,
    pub(crate) absolute_start_cycle: u64,
    pub(crate) absolute_end_cycle: u64,
    pub(crate) spans: Vec<SmpBootstrapInstructionSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SmpBootstrapDeltaSequence {
    pub(crate) encoding: &'static str,
    pub(crate) fields: Vec<&'static str>,
    pub(crate) record_count: usize,
    pub(crate) expanded_sha256: String,
    pub(crate) data_base64: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FramedApuPortAccess {
    pub(crate) frame: u32,
    pub(crate) access: crate::libretro_core::LibretroApuPortWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FramedCpuTimingTransaction {
    pub(crate) frame: u32,
    pub(crate) transaction: crate::libretro_core::LibretroCpuTimingTransaction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct FramedSmpInstruction {
    pub(crate) frame: u32,
    pub(crate) instruction: crate::libretro_core::LibretroSmpInstruction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SmpPostHandoffAnchor {
    pub(crate) handoff_cycle: u64,
    pub(crate) final_cpu_access: FramedApuPortAccess,
    pub(crate) final_cpu_timing_transaction: FramedCpuTimingTransaction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FirstNmiApuAnchor {
    pub(crate) access: FramedApuPortAccess,
    pub(crate) completed_timing_transaction: FramedCpuTimingTransaction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Snes9xRetroRunTrace {
    pub(crate) entry: serde_json::Value,
    pub(crate) return_event: serde_json::Value,
    pub(crate) hdma_events: Vec<serde_json::Value>,
    pub(crate) video_events: Vec<serde_json::Value>,
    pub(crate) raw_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OrdinalApuPortAccess {
    pub(crate) cpu_transaction_ordinal: usize,
    pub(crate) access: crate::libretro_core::LibretroApuPortWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OrdinalSmpOutputPortWrite {
    pub(crate) cpu_transaction_ordinal: usize,
    pub(crate) write: crate::libretro_core::LibretroSmpOutputPortWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct CpuTimingGapReceipt {
    pub(crate) previous_transaction_ordinal: usize,
    pub(crate) next_transaction_ordinal: usize,
    pub(crate) previous_end_v_counter: i32,
    pub(crate) previous_end_cpu_cycle: i32,
    pub(crate) next_start_v_counter: i32,
    pub(crate) next_start_cpu_cycle: i32,
    pub(crate) elapsed_master_cycles: i64,
}

pub(crate) struct PendingFirstNmiReturnFixture {
    pub(crate) final_path: PathBuf,
    pub(crate) temporary_path: PathBuf,
    pub(crate) writer: BufWriter<fs::File>,
    pub(crate) installed: bool,
}

impl PendingFirstNmiReturnFixture {
    pub(crate) fn create(path: impl AsRef<Path>) -> Result<Self, String> {
        let final_path = path.as_ref().to_path_buf();
        let temporary_path =
            PathBuf::from(format!("{}.tmp-{}", final_path.display(), process::id()));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .map_err(|error| {
                format!(
                    "failed to create temporary first-NMI return fixture {}: {error}",
                    temporary_path.display()
                )
            })?;
        Ok(Self {
            final_path,
            temporary_path,
            writer: BufWriter::new(file),
            installed: false,
        })
    }

    pub(crate) fn install(&mut self) -> Result<(), String> {
        self.writer.flush().map_err(|error| {
            format!(
                "failed to flush temporary first-NMI return fixture {}: {error}",
                self.temporary_path.display()
            )
        })?;
        self.writer.get_ref().sync_all().map_err(|error| {
            format!(
                "failed to sync temporary first-NMI return fixture {}: {error}",
                self.temporary_path.display()
            )
        })?;
        fs::rename(&self.temporary_path, &self.final_path).map_err(|error| {
            format!(
                "failed to atomically install first-NMI return fixture {}: {error}",
                self.final_path.display()
            )
        })?;
        self.installed = true;
        Ok(())
    }
}

pub(crate) fn compact_engine_state_mismatches(rust: &[u8], oracle: &[u8]) -> Vec<String> {
    let byte = |ram: &[u8], address: usize| ram.get(address).copied().unwrap_or_default();
    let word = |ram: &[u8], address: usize| {
        u16::from_le_bytes([byte(ram, address), byte(ram, address.saturating_add(1))])
    };
    let mut mismatches = Vec::new();
    for (name, address) in [
        ("main_module", 0x0010),
        ("submodule", 0x0011),
        ("subsubmodule", 0x00b0),
        ("frame_counter", 0x001a),
    ] {
        let rust_value = byte(rust, address);
        let oracle_value = byte(oracle, address);
        if rust_value != oracle_value {
            mismatches.push(format!(
                "{name} rust=0x{rust_value:02x} oracle=0x{oracle_value:02x}"
            ));
        }
    }
    for (name, address) in [
        ("dungeon_room", 0x00a0),
        ("link_x", 0x0022),
        ("link_y", 0x0020),
        ("bg2_h", 0x00e2),
        ("bg2_v", 0x00e8),
    ] {
        let rust_value = word(rust, address);
        let oracle_value = word(oracle, address);
        if rust_value != oracle_value {
            mismatches.push(format!(
                "{name} rust=0x{rust_value:04x} oracle=0x{oracle_value:04x}"
            ));
        }
    }
    for slot in 0..16 {
        for (name, base) in [
            ("state", 0x0dd0),
            ("type", 0x0e20),
            ("x_low", 0x0d10),
            ("x_high", 0x0d30),
            ("x_subpixel", 0x0d70),
            ("x_velocity", 0x0d50),
            ("y_low", 0x0d00),
            ("y_high", 0x0d20),
            ("y_subpixel", 0x0d60),
            ("y_velocity", 0x0d40),
            ("direction", 0x0de0),
            ("head_direction", 0x0eb0),
            ("graphics", 0x0dc0),
            ("ai_state", 0x0d80),
            ("wall_collision", 0x0e70),
            ("subtype", 0x0e30),
            ("subtype2", 0x0e80),
            ("delay_main", 0x0df0),
            ("delay_aux1", 0x0e00),
        ] {
            let rust_value = byte(rust, base + slot);
            let oracle_value = byte(oracle, base + slot);
            if rust_value != oracle_value {
                mismatches.push(format!(
                    "sprite[{slot}].{name} rust=0x{rust_value:02x} oracle=0x{oracle_value:02x}"
                ));
            }
        }
    }
    mismatches
}

pub(crate) fn push_unsigned_varint(bytes: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        bytes.push((value as u8) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

pub(crate) fn base64_bytes(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let value = u32::from(chunk[0]) << 16
            | u32::from(chunk.get(1).copied().unwrap_or(0)) << 8
            | u32::from(chunk.get(2).copied().unwrap_or(0));
        encoded.push(ALPHABET[((value >> 18) & 0x3f) as usize] as char);
        encoded.push(ALPHABET[((value >> 12) & 0x3f) as usize] as char);
        encoded.push(if chunk.len() >= 2 {
            ALPHABET[((value >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() == 3 {
            ALPHABET[(value & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    encoded
}

pub(crate) fn compact_delta_integer_sequence<const N: usize>(
    fields: [&'static str; N],
    rows: impl IntoIterator<Item = [i64; N]>,
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(fields, rows, false)
}

pub(crate) fn compact_delta_integer_sequence_with_zstd<const N: usize>(
    fields: [&'static str; N],
    rows: impl IntoIterator<Item = [i64; N]>,
    use_zstd: bool,
) -> SmpBootstrapDeltaSequence {
    let rows = rows.into_iter().collect::<Vec<_>>();
    let mut encoded = Vec::new();
    let mut expanded = Vec::new();
    for row in &rows {
        for value in row {
            expanded.extend_from_slice(&value.to_le_bytes());
        }
    }
    for field in 0..N {
        let mut column = Vec::new();
        let mut previous = 0i64;
        let mut row = 0;
        while row < rows.len() {
            let delta = rows[row][field] - previous;
            previous = rows[row][field];
            if delta != 0 {
                let zigzag = ((delta << 1) ^ (delta >> 63)) as u64;
                push_unsigned_varint(&mut column, zigzag + 1);
                row += 1;
                continue;
            }
            let mut run_length = 1usize;
            while row + run_length < rows.len() && rows[row + run_length][field] == previous {
                run_length += 1;
            }
            push_unsigned_varint(&mut column, 0);
            push_unsigned_varint(&mut column, run_length as u64);
            row += run_length;
        }
        push_unsigned_varint(&mut encoded, column.len() as u64);
        encoded.extend_from_slice(&column);
    }
    let encoded = if use_zstd {
        zstd::stream::encode_all(encoded.as_slice(), 19)
            .expect("in-memory CPU timing fixture compression failed")
    } else {
        encoded
    };
    SmpBootstrapDeltaSequence {
        encoding: if use_zstd {
            "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
        } else {
            "columnar-signed-delta-zero-rle-varint-base64-v1"
        },
        fields: fields.into_iter().collect(),
        record_count: rows.len(),
        expanded_sha256: parity::evidence::sha256_bytes(&expanded),
        data_base64: base64_bytes(&encoded),
    }
}

pub(crate) fn compact_cpu_apu_accesses(
    accesses: &[FramedApuPortAccess],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence(
        [
            "frame",
            "port",
            "value",
            "output_sample",
            "v_counter",
            "cpu_cycle",
            "program_counter",
            "apu_cycle_before",
            "apu_cycle_after",
            "smp_clock_before",
            "smp_clock_after",
            "smp_pc_before",
            "smp_pc_after",
            "smp_opcode_before",
            "smp_opcode_after",
            "smp_opcode_cycle_before",
            "smp_opcode_cycle_after",
            "is_read",
            "cpu_model_5a22",
            "wram_refresh_position",
            "cpu_model_identity",
        ],
        accesses.iter().map(|framed| {
            let access = &framed.access;
            [
                i64::from(framed.frame),
                i64::from(access.port),
                i64::from(access.value),
                i64::from(access.output_sample),
                i64::from(access.v_counter),
                i64::from(access.cpu_cycle),
                i64::from(access.program_counter),
                i64::from(access.apu_cycle_before),
                i64::from(access.apu_cycle_after),
                i64::from(access.smp_clock_before),
                i64::from(access.smp_clock_after),
                i64::from(access.smp_pc_before),
                i64::from(access.smp_pc_after),
                i64::from(access.smp_opcode_before),
                i64::from(access.smp_opcode_after),
                i64::from(access.smp_opcode_cycle_before),
                i64::from(access.smp_opcode_cycle_after),
                i64::from(access.is_read),
                i64::from(access.cpu_model_5a22),
                i64::from(access.wram_refresh_position),
                i64::from(access.cpu_model_identity),
            ]
        }),
    )
}

pub(crate) fn compact_ordinal_cpu_apu_accesses(
    accesses: &[OrdinalApuPortAccess],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        [
            "cpu_transaction_ordinal",
            "port",
            "value",
            "output_sample",
            "v_counter",
            "cpu_cycle",
            "program_counter",
            "apu_cycle_before",
            "apu_cycle_after",
            "smp_clock_before",
            "smp_clock_after",
            "dsp_clock_before",
            "dsp_clock_after",
            "dsp_phase_before",
            "dsp_phase_after",
            "smp_pc_before",
            "smp_pc_after",
            "smp_opcode_before",
            "smp_opcode_after",
            "smp_opcode_cycle_before",
            "smp_opcode_cycle_after",
            "is_read",
            "cpu_model_5a22",
            "wram_refresh_position",
            "cpu_model_identity",
        ],
        accesses.iter().map(|ordinal| {
            let access = &ordinal.access;
            [
                ordinal.cpu_transaction_ordinal as i64,
                i64::from(access.port),
                i64::from(access.value),
                i64::from(access.output_sample),
                i64::from(access.v_counter),
                i64::from(access.cpu_cycle),
                i64::from(access.program_counter),
                i64::from(access.apu_cycle_before),
                i64::from(access.apu_cycle_after),
                i64::from(access.smp_clock_before),
                i64::from(access.smp_clock_after),
                i64::from(access.dsp_clock_before),
                i64::from(access.dsp_clock_after),
                i64::from(access.dsp_phase_before),
                i64::from(access.dsp_phase_after),
                i64::from(access.smp_pc_before),
                i64::from(access.smp_pc_after),
                i64::from(access.smp_opcode_before),
                i64::from(access.smp_opcode_after),
                i64::from(access.smp_opcode_cycle_before),
                i64::from(access.smp_opcode_cycle_after),
                i64::from(access.is_read),
                i64::from(access.cpu_model_5a22),
                i64::from(access.wram_refresh_position),
                i64::from(access.cpu_model_identity),
            ]
        }),
        true,
    )
}

pub(crate) fn compact_ordinal_smp_output_port_writes(
    writes: &[OrdinalSmpOutputPortWrite],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        [
            "cpu_transaction_ordinal",
            "absolute_cycle",
            "port",
            "value",
            "origin_pc",
            "opcode",
            "opcode_cycle",
            "v_counter",
            "cpu_cycle",
            "cpu_program_counter",
            "cpu_reference_time",
            "cpu_remainder",
            "smp_clock",
            "next_pc",
            "dsp_clock",
            "dsp_phase",
            "output_sample",
        ],
        writes.iter().map(|ordinal| {
            let write = ordinal.write;
            [
                ordinal.cpu_transaction_ordinal as i64,
                write.absolute_cycle as i64,
                i64::from(write.port),
                i64::from(write.value),
                i64::from(write.origin_pc),
                i64::from(write.opcode),
                i64::from(write.opcode_cycle),
                i64::from(write.v_counter),
                i64::from(write.cpu_cycle),
                i64::from(write.cpu_program_counter),
                i64::from(write.cpu_reference_time),
                i64::from(write.cpu_remainder),
                i64::from(write.smp_clock),
                i64::from(write.next_pc),
                i64::from(write.dsp_clock),
                i64::from(write.dsp_phase),
                i64::from(write.output_sample),
            ]
        }),
        true,
    )
}

pub(crate) fn compact_cpu_timing_transactions(
    transactions: &[FramedCpuTimingTransaction],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        [
            "frame",
            "kind",
            "duration",
            "origin_pc",
            "opcode",
            "start_v_counter",
            "start_cpu_cycle",
            "end_v_counter",
            "end_cpu_cycle",
            "cpu_model_identity",
            "cpu_model_5a22",
            "start_wram_refresh_position",
            "end_wram_refresh_position",
        ],
        transactions.iter().map(|framed| {
            let transaction = &framed.transaction;
            [
                i64::from(framed.frame),
                i64::from(transaction.kind),
                i64::from(transaction.duration),
                i64::from(transaction.origin_pc),
                i64::from(transaction.opcode),
                i64::from(transaction.start_v_counter),
                i64::from(transaction.start_cpu_cycle),
                i64::from(transaction.end_v_counter),
                i64::from(transaction.end_cpu_cycle),
                i64::from(transaction.cpu_model_identity),
                i64::from(transaction.cpu_model_5a22),
                i64::from(transaction.start_wram_refresh_position),
                i64::from(transaction.end_wram_refresh_position),
            ]
        }),
        true,
    )
}

pub(crate) fn first_nmi_apui_anchor_indices(
    accesses: &[FramedApuPortAccess],
    transactions: &[FramedCpuTimingTransaction],
) -> Result<Option<(usize, usize)>, String> {
    let Some(access_index) = accesses.iter().position(|framed| {
        let access = &framed.access;
        access.is_read && access.port == 0 && (access.program_counter & 0x00ff_ffff) == 0x0080e4
    }) else {
        return Ok(None);
    };
    let access = &accesses[access_index];
    let transaction_index = transactions
        .iter()
        .position(|framed| {
            let transaction = &framed.transaction;
            framed.frame == access.frame
                && transaction.kind == 2
                && (transaction.origin_pc & 0x00ff_ffff) == 0x0080e1
                && transaction.opcode == 0xad
                && transaction.start_v_counter == access.access.v_counter
                && transaction.start_cpu_cycle == access.access.cpu_cycle
        })
        .ok_or_else(|| {
            "first $8080e1 APU read has no matching completed kind-2 timing transaction".to_string()
        })?;
    Ok(Some((access_index, transaction_index)))
}

pub(crate) fn first_nmi_dma_setup_stop_index(
    transactions: &[FramedCpuTimingTransaction],
) -> Result<Option<usize>, String> {
    let Some(index) = transactions
        .iter()
        .position(|framed| (framed.transaction.origin_pc & 0x00ff_ffff) == 0x008a35)
    else {
        return Ok(None);
    };
    let transaction = transactions[index].transaction;
    if transaction.kind != 0 || transaction.opcode != 0x8d {
        return Err(format!(
            "first $008a35 transaction is not the expected STA raw fetch: {:?}",
            transactions[index]
        ));
    }
    Ok(Some(index))
}

pub(crate) fn first_nmi_dma_transaction_slice(
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
) -> Result<Option<(usize, usize, usize)>, String> {
    let Some(fetch_index) = transactions.iter().position(|transaction| {
        transaction.kind == 0
            && (transaction.origin_pc & 0x00ff_ffff) == 0x008a35
            && transaction.opcode == 0x8d
    }) else {
        return Ok(None);
    };
    let fetch = transactions[fetch_index];
    if fetch.start_v_counter != 226
        || fetch.start_cpu_cycle != 714
        || fetch.end_v_counter != 226
        || fetch.end_cpu_cycle != 722
    {
        return Err(format!(
            "$008a35 raw fetch does not continue the pinned V226:H714->H722 anchor: {fetch:?}"
        ));
    }
    let completion_index = transactions[fetch_index..]
        .iter()
        .position(|transaction| {
            transaction.kind == 2
                && (transaction.origin_pc & 0x00ff_ffff) == 0x008a35
                && transaction.opcode == 0x8d
        })
        .map(|relative| fetch_index + relative)
        .ok_or("$008a35 STA $420b has no completed kind-2 semantic transaction")?;
    let successor_index = transactions[completion_index + 1..]
        .iter()
        .position(|transaction| {
            transaction.kind == 0 && (transaction.origin_pc & 0x00ff_ffff) == 0x008a38
        })
        .map(|relative| completion_index + 1 + relative)
        .ok_or("completed $008a35 STA $420b has no following $008a38 raw fetch")?;
    Ok(Some((fetch_index, completion_index, successor_index)))
}

pub(crate) fn first_nmi_return_start_index(
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
) -> Result<Option<usize>, String> {
    let candidates = transactions
        .iter()
        .enumerate()
        .filter(|(_, transaction)| {
            transaction.kind == 0
                && (transaction.origin_pc & 0x00ff_ffff) == FIRST_NMI_RETURN_START_PC
        })
        .collect::<Vec<_>>();
    let Some((index, transaction)) = candidates.first().copied() else {
        return Ok(None);
    };
    if candidates.len() != 1 {
        return Err(format!(
            "expected one $008a38 successor fetch, observed {}",
            candidates.len()
        ));
    }
    if transaction.opcode != 0x8c
        || transaction.start_v_counter != 227
        || transaction.start_cpu_cycle != 742
        || transaction.end_v_counter != 227
        || transaction.end_cpu_cycle != 750
        || transaction.cpu_model_5a22 != 2
        || transaction.start_wram_refresh_position != 534
        || transaction.end_wram_refresh_position != 534
    {
        return Err(format!(
            "$008a38 successor fetch does not match the committed V227:H742->H750 M2 receipt: {transaction:?}"
        ));
    }
    Ok(Some(index))
}

pub(crate) fn cpu_transaction_ordinal_at_start(
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
    v_counter: i32,
    cpu_cycle: i32,
    cpu_program_counter: i32,
) -> Result<Option<usize>, String> {
    let candidates = transactions
        .iter()
        .enumerate()
        .filter(|(_, transaction)| {
            transaction.kind == 2
                && transaction.start_v_counter == v_counter
                && transaction.start_cpu_cycle == cpu_cycle
                && transaction.origin_pc >> 16 == cpu_program_counter >> 16
                && (0..=4)
                    .contains(&((cpu_program_counter & 0xffff) - (transaction.origin_pc & 0xffff)))
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [index] => Ok(Some(*index)),
        _ => Err(format!(
            "APUI receipt at V{v_counter}:H{cpu_cycle} PC ${cpu_program_counter:06x} matches multiple CPU transactions: {candidates:?}"
        )),
    }
}

pub(crate) fn cpu_transaction_ordinal_containing(
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
    v_counter: i32,
    cpu_cycle: i32,
    cpu_program_counter: i32,
) -> Result<Option<usize>, String> {
    if transactions.is_empty() {
        return Err("cannot join a receipt to an empty CPU transaction sequence".into());
    }
    let candidates = transactions
        .iter()
        .enumerate()
        .filter(|(_, transaction)| {
            let pc_delta = (cpu_program_counter & 0xffff) - (transaction.origin_pc & 0xffff);
            let contains_raster = if transaction.start_v_counter == transaction.end_v_counter {
                v_counter == transaction.start_v_counter
                    && transaction.start_cpu_cycle <= cpu_cycle
                    && cpu_cycle < transaction.end_cpu_cycle
            } else {
                (v_counter == transaction.start_v_counter
                    && transaction.start_cpu_cycle <= cpu_cycle)
                    || (v_counter == transaction.end_v_counter
                        && cpu_cycle < transaction.end_cpu_cycle)
            };
            contains_raster
                && transaction.origin_pc >> 16 == cpu_program_counter >> 16
                && (0..=4).contains(&pc_delta)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [index] => Ok(Some(*index)),
        _ => Err(format!(
            "SMP output receipt at V{v_counter}:H{cpu_cycle} PC ${cpu_program_counter:06x} matches multiple CPU transactions: {candidates:?}"
        )),
    }
}

pub(crate) fn read_snes9x_retro_run_trace(
    path: &Path,
    expected_run: u32,
) -> Result<Option<Snes9xRetroRunTrace>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read Snes9x core trace {}: {error}",
            path.display()
        )
    })?;
    let records = parity::trace_format::read_all(path).map_err(|error| {
        format!(
            "failed to read Snes9x core trace {}: {error}",
            path.display()
        )
    })?;
    let mut entry = None;
    let mut return_event = None;
    let mut hdma_events = Vec::new();
    let mut video_events = Vec::new();
    let mut saw_run = false;
    for record in &records {
        let mut event = record.to_json();
        // The core names HDMA rows `hdma-start`/`hdma-end`; this receipt
        // reads them as the `hdma` domain with a start/end stage.
        if let Some(phase) = record.event().strip_prefix("hdma-") {
            event["event"] = serde_json::Value::String("hdma".to_string());
            event["stage"] = serde_json::Value::String(phase.to_string());
        }
        if event["run"].as_u64() != Some(u64::from(expected_run)) {
            continue;
        }
        saw_run = true;
        match (event["event"].as_str(), event["stage"].as_str()) {
            (Some("frame"), Some("entry")) => {
                if entry.replace(event).is_some() {
                    return Err(format!("run {expected_run} has duplicate frame-entry rows"));
                }
            }
            (Some("frame"), Some("return")) => {
                if return_event.replace(event).is_some() {
                    return Err(format!(
                        "run {expected_run} has duplicate frame-return rows"
                    ));
                }
            }
            (Some("hdma"), Some("start" | "end")) => hdma_events.push(event),
            // The maintained trace core always publishes this direct
            // retro_run scanout milestone; it is not controlled by the
            // optional event-domain mask. Retain it rather than pretending
            // `frame,hdma` suppresses a source-owned receipt.
            (Some("video"), Some("presented")) => video_events.push(event),
            (event, stage) => {
                return Err(format!(
                    "run {expected_run} has unexpected trace domain/stage {event:?}/{stage:?}"
                ));
            }
        }
    }
    if !saw_run {
        return Ok(None);
    }
    let entry = entry.ok_or_else(|| format!("run {expected_run} has no frame-entry row"))?;
    let return_event =
        return_event.ok_or_else(|| format!("run {expected_run} has no frame-return row"))?;
    if video_events.len() != 1 {
        return Err(format!(
            "run {expected_run} has {} video/presented rows, expected exactly one",
            video_events.len()
        ));
    }
    for (pair_index, pair) in hdma_events.chunks(2).enumerate() {
        if pair.len() != 2 || pair[0]["stage"] != "start" || pair[1]["stage"] != "end" {
            return Err(format!(
                "run {expected_run} HDMA pair {pair_index} is not an ordered start/end bracket"
            ));
        }
    }
    Ok(Some(Snes9xRetroRunTrace {
        entry,
        return_event,
        hdma_events,
        video_events,
        raw_sha256: parity::evidence::sha256_bytes(&bytes),
    }))
}

pub(crate) fn validate_first_nmi_return_cpu_slice(
    trace: &Snes9xRetroRunTrace,
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
) -> Result<Vec<CpuTimingGapReceipt>, String> {
    if trace.entry["v"].as_i64() != Some(225)
        || trace.entry["cycles"].as_i64() != Some(6)
        || trace.entry["pc"].as_i64() != Some(0x008036)
    {
        return Err(format!(
            "run-81 continuation has the wrong direct entry receipt, expected V225:H6 PC $008036: {}",
            trace.entry
        ));
    }
    if trace.return_event["v"].as_i64() != Some(225)
        || trace.return_event["cycles"].as_i64() != Some(94)
        || trace.return_event["pc"].as_i64() != Some(0x0080c9)
    {
        return Err(format!(
            "run-81 continuation has the wrong direct return receipt, expected V225:H94 PC $0080c9: {}",
            trace.return_event
        ));
    }
    let first = transactions
        .first()
        .ok_or("run-81 continuation CPU slice is empty")?;
    let terminal = transactions.last().unwrap();
    if first.start_v_counter != 227 || first.start_cpu_cycle != 742 {
        return Err(format!(
            "run-81 continuation CPU slice does not start at V227:H742: {first:?}"
        ));
    }
    if terminal.end_v_counter != 225 || terminal.end_cpu_cycle != 94 {
        return Err(format!(
            "run-81 continuation CPU slice does not end at the direct V225:H94 return: {terminal:?}"
        ));
    }
    let mut rollovers = 0;
    let mut gaps = Vec::new();
    for (index, transaction) in transactions.iter().enumerate() {
        if transaction.start_v_counter == 261 && transaction.end_v_counter == 0 {
            rollovers += 1;
        } else if transaction.end_v_counter < transaction.start_v_counter {
            return Err(format!(
                "CPU transaction {index} has an unexpected raster reversal: {transaction:?}"
            ));
        }
        if let Some(next) = transactions.get(index + 1) {
            let mut end_position =
                i64::from(transaction.end_v_counter) * 1364 + i64::from(transaction.end_cpu_cycle);
            let mut next_position =
                i64::from(next.start_v_counter) * 1364 + i64::from(next.start_cpu_cycle);
            if next.start_v_counter < transaction.end_v_counter {
                if transaction.end_v_counter != 261 || next.start_v_counter != 0 {
                    return Err(format!(
                        "CPU transaction {index} has an unexpected raster reversal before its successor: current={transaction:?}; next={next:?}"
                    ));
                }
                rollovers += 1;
                next_position += 262 * 1364;
            }
            if transaction.end_v_counter == 0 && transaction.start_v_counter == 261 {
                end_position += 262 * 1364;
                next_position += 262 * 1364;
            }
            if next_position < end_position {
                return Err(format!(
                    "CPU transaction {index} overlaps or reorders its successor: current={transaction:?}; next={next:?}"
                ));
            }
            if next_position > end_position {
                gaps.push(CpuTimingGapReceipt {
                    previous_transaction_ordinal: index,
                    next_transaction_ordinal: index + 1,
                    previous_end_v_counter: transaction.end_v_counter,
                    previous_end_cpu_cycle: transaction.end_cpu_cycle,
                    next_start_v_counter: next.start_v_counter,
                    next_start_cpu_cycle: next.start_cpu_cycle,
                    elapsed_master_cycles: next_position - end_position,
                });
            }
        }
    }
    if rollovers != 1 {
        return Err(format!(
            "run-81 continuation CPU slice has {rollovers} V261->V0 rollovers, expected exactly one"
        ));
    }
    Ok(gaps)
}

pub(crate) fn trailing_dma_events_after_first_nmi(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<Vec<crate::libretro_core::LibretroDmaLedgerEvent>, String> {
    let Some((outer, selected)) = first_nmi_dma_ledger_slice(events)? else {
        return Err(
            "same-retro_run continuation is missing the committed first-NMI DMA prefix".into(),
        );
    };
    let last = events
        .iter()
        .rposition(|event| event.fields[1] == outer)
        .ok_or("selected first-NMI DMA outer vanished from the complete ledger")?;
    if events[..=last]
        .iter()
        .filter(|event| event.fields[1] == outer)
        .count()
        != selected.len()
    {
        return Err("first-NMI DMA outer is interleaved with later ledger ownership".into());
    }
    let trailing = events[last + 1..].to_vec();
    validate_complete_dma_outers(&trailing)?;
    Ok(trailing)
}

pub(crate) fn validate_complete_dma_outers(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<(), String> {
    let mut index = 0;
    while index < events.len() {
        let begin = &events[index];
        if begin.fields[0] != 0 || begin.fields[9] != 1 {
            return Err(format!(
                "DMA continuation does not begin with a completed outer-begin row: {begin:?}"
            ));
        }
        let outer = begin.fields[1];
        let end = events[index..]
            .iter()
            .position(|event| event.fields[1] == outer && event.fields[0] == 4)
            .map(|relative| index + relative)
            .ok_or_else(|| format!("DMA outer {outer} has no outer-end row"))?;
        if events[index..=end]
            .iter()
            .any(|event| event.fields[1] != outer || event.fields[9] != 1)
        {
            return Err(format!(
                "DMA outer {outer} has mixed ownership or an incomplete row"
            ));
        }
        let mut cursor = index + 1;
        let mut global_byte_ordinal = 0;
        while cursor < end {
            let channel_begin = &events[cursor];
            if channel_begin.fields[0] != 1 {
                return Err(format!(
                    "DMA outer {outer} expected a channel-begin row at event {cursor}: {channel_begin:?}"
                ));
            }
            let channel = channel_begin.fields[2];
            cursor += 1;
            let mut channel_byte_ordinal = 0;
            while cursor < end && events[cursor].fields[0] == 2 {
                let byte = &events[cursor];
                if byte.fields[2] != channel
                    || byte.fields[3] != global_byte_ordinal
                    || byte.fields[4] != channel_byte_ordinal
                {
                    return Err(format!(
                        "DMA outer {outer} has a non-contiguous byte receipt at event {cursor}: {byte:?}"
                    ));
                }
                global_byte_ordinal += 1;
                channel_byte_ordinal += 1;
                cursor += 1;
            }
            let channel_end = events.get(cursor).ok_or_else(|| {
                format!("DMA outer {outer} ended before channel {channel} completion")
            })?;
            if channel_end.fields[0] != 3 || channel_end.fields[2] != channel {
                return Err(format!(
                    "DMA outer {outer} has no matching channel-end row for channel {channel}: {channel_end:?}"
                ));
            }
            cursor += 1;
        }
        index = end + 1;
    }
    Ok(())
}

pub(crate) fn semantic_receipts_from_dma_ledger(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<Vec<OriginalTimingSemanticReceipt>, String> {
    validate_complete_dma_outers(events)?;
    let mut receipts = Vec::new();
    for event in events.iter().filter(|event| event.fields[0] == 4) {
        let channel_mask = u8::try_from(event.fields[2]).map_err(|_| {
            format!(
                "completed Snes9x DMA outer has invalid channel mask {}",
                event.fields[2]
            )
        })?;
        // Snes9x still records the outer `$420b` semantic for a zero mask,
        // but no DMA publication occurred. Keep validating that chronology
        // above, then omit it from Zelda's semantic receipt stream.
        if channel_mask != 0 {
            receipts.push(OriginalTimingSemanticReceipt::DmaPublicationCompleted { channel_mask });
        }
    }
    Ok(receipts)
}

pub(crate) fn first_nmi_dma_ledger_slice(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<Option<(i32, Vec<crate::libretro_core::LibretroDmaLedgerEvent>)>, String> {
    let Some(begin) = events.iter().find(|event| {
        event.fields[0] == 0
            && event.fields[2] == 0x07
            && (event.fields[27] & 0x00ff_ffff) == 0x008a38
    }) else {
        return Ok(None);
    };
    let outer = begin.fields[1];
    let selected = events
        .iter()
        .filter(|event| event.fields[1] == outer)
        .cloned()
        .collect::<Vec<_>>();
    validate_first_nmi_dma_ledger(&selected)?;
    Ok(Some((outer, selected)))
}

pub(crate) fn validate_first_nmi_dma_ledger(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<(), String> {
    let Some(first) = events.first() else {
        return Err("selected DMA ledger is empty".to_string());
    };
    let outer = first.fields[1];
    if first.fields[0] != 0 || first.fields[2] != 0x07 || first.fields[9] != 1 {
        return Err(format!(
            "invalid first-NMI DMA outer-begin event: {first:?}"
        ));
    }
    let last = events.last().expect("checked nonempty");
    if last.fields[0] != 4
        || last.fields[1] != outer
        || last.fields[2] != 0x07
        || last.fields[9] != 1
    {
        return Err(format!("invalid first-NMI DMA outer-end event: {last:?}"));
    }
    if events.iter().any(|event| event.fields[1] != outer) {
        return Err("selected DMA ledger crosses outer-transfer ownership".to_string());
    }

    let channel_markers = events
        .iter()
        .filter(|event| matches!(event.fields[0], 1 | 3))
        .map(|event| (event.fields[0], event.fields[2], event.fields[9]))
        .collect::<Vec<_>>();
    if channel_markers
        != vec![
            (1, 0, 1),
            (3, 0, 1),
            (1, 1, 1),
            (3, 1, 1),
            (1, 2, 1),
            (3, 2, 1),
        ]
    {
        return Err(format!(
            "first-NMI DMA channel ownership is not the ordered 0/1/2 mask-$07 sequence: {channel_markers:?}"
        ));
    }

    let bytes = events
        .iter()
        .filter(|event| event.fields[0] == 2)
        .collect::<Vec<_>>();
    if bytes.is_empty() {
        return Err("first-NMI DMA ledger contains no byte transactions".to_string());
    }
    let mut channel_ordinals = [0i32; 8];
    for (global, event) in bytes.iter().enumerate() {
        let channel = usize::try_from(event.fields[2])
            .ok()
            .filter(|channel| *channel < 8)
            .ok_or_else(|| format!("invalid DMA byte channel in {event:?}"))?;
        if event.fields[3] != global as i32
            || event.fields[4] != channel_ordinals[channel]
            || event.fields[9] != 1
        {
            return Err(format!(
                "non-contiguous or incomplete first-NMI DMA byte receipt at global ordinal {global}: {event:?}"
            ));
        }
        let expected_a_bus = (event.fields[51] << 16) | event.fields[50];
        let expected_remaining = (event.fields[49] - 1) & 0xffff;
        let a_increment = if event.fields[47] != 0 {
            0
        } else if event.fields[48] != 0 {
            -1
        } else {
            1
        };
        let expected_a_address = (event.fields[50] + a_increment) & 0xffff;
        if !matches!(event.fields[5], 0 | 1)
            || event.fields[6] != expected_a_bus
            || !(0x2100..=0x21ff).contains(&event.fields[7])
            || !(0..=0xff).contains(&event.fields[8])
            || event.fields[63] != expected_remaining
            || event.fields[64] != expected_a_address
        {
            return Err(format!(
                "incomplete or inconsistent first-NMI DMA A/B semantic receipt at global ordinal {global}: {event:?}"
            ));
        }
        channel_ordinals[channel] += 1;
    }
    Ok(())
}

pub(crate) fn compact_dma_ledger(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        DMA_LEDGER_FIELDS,
        events.iter().map(|event| event.fields.map(i64::from)),
        true,
    )
}

pub(crate) fn compact_byte_snapshot(bytes: &[u8]) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        ["byte"],
        bytes.iter().map(|byte| [i64::from(*byte)]),
        true,
    )
}

pub(crate) fn compact_smp_output_port_writes(
    writes: &[crate::libretro_core::LibretroSmpOutputPortWrite],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence(
        [
            "absolute_cycle",
            "port",
            "value",
            "origin_pc",
            "opcode",
            "opcode_cycle",
            "v_counter",
            "cpu_cycle",
            "cpu_program_counter",
            "cpu_reference_time",
            "cpu_remainder",
            "smp_clock",
            "next_pc",
            "dsp_clock",
            "dsp_phase",
            "output_sample",
        ],
        writes.iter().map(|write| {
            [
                write.absolute_cycle as i64,
                i64::from(write.port),
                i64::from(write.value),
                i64::from(write.origin_pc),
                i64::from(write.opcode),
                i64::from(write.opcode_cycle),
                i64::from(write.v_counter),
                i64::from(write.cpu_cycle),
                i64::from(write.cpu_program_counter),
                i64::from(write.cpu_reference_time),
                i64::from(write.cpu_remainder),
                i64::from(write.smp_clock),
                i64::from(write.next_pc),
                i64::from(write.dsp_clock),
                i64::from(write.dsp_phase),
                i64::from(write.output_sample),
            ]
        }),
    )
}

pub(crate) fn append_smp_instruction_frame(
    accumulated: &mut Vec<crate::libretro_core::LibretroSmpInstruction>,
    frame: Vec<crate::libretro_core::LibretroSmpInstruction>,
) {
    let mut frame = frame.into_iter().peekable();
    if accumulated
        .last()
        .zip(frame.peek())
        .is_some_and(|(left, right)| {
            left.absolute_cycle == right.absolute_cycle
                && left.program_counter == right.program_counter
                && left.opcode == right.opcode
        })
    {
        *accumulated.last_mut().unwrap() = frame.next().unwrap();
    }
    accumulated.extend(frame);
}

pub(crate) fn append_framed_smp_instruction_frame(
    accumulated: &mut Vec<FramedSmpInstruction>,
    frame_index: u32,
    frame: Vec<crate::libretro_core::LibretroSmpInstruction>,
) {
    let mut frame = frame
        .into_iter()
        .map(|instruction| FramedSmpInstruction {
            frame: frame_index,
            instruction,
        })
        .peekable();
    if accumulated
        .last()
        .zip(frame.peek())
        .is_some_and(|(left, right)| {
            left.instruction.absolute_cycle == right.instruction.absolute_cycle
                && left.instruction.program_counter == right.instruction.program_counter
                && left.instruction.opcode == right.instruction.opcode
        })
    {
        *accumulated.last_mut().unwrap() = frame.next().unwrap();
    }
    accumulated.extend(frame);
}

pub(crate) fn smp_bootstrap_handoff_index(
    instructions: &[crate::libretro_core::LibretroSmpInstruction],
) -> Option<usize> {
    instructions.windows(2).position(|pair| {
        pair[0].program_counter == 0xfffb
            && pair[0].opcode == 0x1f
            && pair[1].program_counter == 0x0800
    })
}

pub(crate) fn framed_smp_bootstrap_handoff_index(
    instructions: &[FramedSmpInstruction],
) -> Option<usize> {
    instructions.windows(2).position(|pair| {
        pair[0].instruction.program_counter == 0xfffb
            && pair[0].instruction.opcode == 0x1f
            && pair[1].instruction.program_counter == 0x0800
    })
}

pub(crate) fn smp_instruction_frame_cycle(instruction: &FramedSmpInstruction) -> i64 {
    i64::from(instruction.instruction.output_sample) * 32
        + i64::from(instruction.instruction.dsp_phase)
        + i64::from(instruction.instruction.smp_clock)
}

pub(crate) fn smp_instruction_bracket_for_apu_access(
    instructions: &[FramedSmpInstruction],
    access: &FramedApuPortAccess,
) -> Result<(usize, usize), String> {
    let owning_boundary = |apu_cycle: i32| {
        instructions
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, framed)| {
                (framed.frame == access.frame
                    && smp_instruction_frame_cycle(framed) <= i64::from(apu_cycle))
                .then_some(index)
            })
    };
    let before = owning_boundary(access.access.apu_cycle_before).ok_or_else(|| {
        format!(
            "frame {} has no SMP boundary owning pre-sync APU cycle {}",
            access.frame, access.access.apu_cycle_before
        )
    })?;
    let after = owning_boundary(access.access.apu_cycle_after).ok_or_else(|| {
        format!(
            "frame {} has no SMP boundary owning post-sync APU cycle {}",
            access.frame, access.access.apu_cycle_after
        )
    })?;
    if after < before {
        return Err(format!(
            "SMP boundary order regressed across APU sync: {before} -> {after}"
        ));
    }
    Ok((before, after))
}

pub(crate) fn framed_smp_instruction_digest_fields() -> &'static [&'static str] {
    &[
        "frame",
        "absolute_cycle",
        "program_counter",
        "opcode",
        "a",
        "x",
        "y",
        "stack_pointer",
        "status",
        "timer0_stage1",
        "timer0_stage2",
        "timer0_stage3",
        "output_sample",
        "dsp_phase",
        "smp_clock",
        "direct_page_0_11[0..12]",
        "boundary_opcode_cycle",
        "op_step_calls",
        "max_continuation_opcode_cycle",
    ]
}

pub(crate) fn framed_smp_instruction_digest(instructions: &[FramedSmpInstruction]) -> String {
    let mut bytes = Vec::with_capacity(instructions.len() * 128);
    for framed in instructions {
        bytes.extend_from_slice(&framed.frame.to_le_bytes());
        let instruction = &framed.instruction;
        bytes.extend_from_slice(&instruction.absolute_cycle.to_le_bytes());
        for value in [
            instruction.program_counter,
            instruction.opcode,
            instruction.a,
            instruction.x,
            instruction.y,
            instruction.stack_pointer,
            instruction.status,
            instruction.timer0_stage1,
            instruction.timer0_stage2,
            instruction.timer0_stage3,
            instruction.output_sample,
            instruction.dsp_phase,
            instruction.smp_clock,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in instruction.direct_page_0_11 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [
            instruction.boundary_opcode_cycle,
            instruction.op_step_calls,
            instruction.max_continuation_opcode_cycle,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    parity::evidence::sha256_bytes(&bytes)
}

pub(crate) fn compact_framed_smp_instructions(
    instructions: &[FramedSmpInstruction],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        [
            "frame",
            "absolute_cycle",
            "program_counter",
            "opcode",
            "a",
            "x",
            "y",
            "stack_pointer",
            "status",
            "timer0_stage1",
            "timer0_stage2",
            "timer0_stage3",
            "output_sample",
            "dsp_phase",
            "smp_clock",
            "direct_page_0",
            "direct_page_1",
            "direct_page_2",
            "direct_page_3",
            "direct_page_4",
            "direct_page_5",
            "direct_page_6",
            "direct_page_7",
            "direct_page_8",
            "direct_page_9",
            "direct_page_10",
            "direct_page_11",
            "boundary_opcode_cycle",
            "op_step_calls",
            "max_continuation_opcode_cycle",
        ],
        instructions.iter().map(|framed| {
            let instruction = &framed.instruction;
            [
                i64::from(framed.frame),
                instruction.absolute_cycle as i64,
                i64::from(instruction.program_counter),
                i64::from(instruction.opcode),
                i64::from(instruction.a),
                i64::from(instruction.x),
                i64::from(instruction.y),
                i64::from(instruction.stack_pointer),
                i64::from(instruction.status),
                i64::from(instruction.timer0_stage1),
                i64::from(instruction.timer0_stage2),
                i64::from(instruction.timer0_stage3),
                i64::from(instruction.output_sample),
                i64::from(instruction.dsp_phase),
                i64::from(instruction.smp_clock),
                i64::from(instruction.direct_page_0_11[0]),
                i64::from(instruction.direct_page_0_11[1]),
                i64::from(instruction.direct_page_0_11[2]),
                i64::from(instruction.direct_page_0_11[3]),
                i64::from(instruction.direct_page_0_11[4]),
                i64::from(instruction.direct_page_0_11[5]),
                i64::from(instruction.direct_page_0_11[6]),
                i64::from(instruction.direct_page_0_11[7]),
                i64::from(instruction.direct_page_0_11[8]),
                i64::from(instruction.direct_page_0_11[9]),
                i64::from(instruction.direct_page_0_11[10]),
                i64::from(instruction.direct_page_0_11[11]),
                i64::from(instruction.boundary_opcode_cycle),
                i64::from(instruction.op_step_calls),
                i64::from(instruction.max_continuation_opcode_cycle),
            ]
        }),
        true,
    )
}

pub(crate) fn smp_bootstrap_steps_match(
    left: &SmpBootstrapInstructionStep,
    right: &SmpBootstrapInstructionStep,
) -> bool {
    left.absolute_end_cycle - left.absolute_start_cycle
        == right.absolute_end_cycle - right.absolute_start_cycle
        && left.origin_pc == right.origin_pc
        && left.opcode == right.opcode
        && left.boundary_opcode_cycle == right.boundary_opcode_cycle
        && left.op_step_calls == right.op_step_calls
        && left.max_continuation_opcode_cycle == right.max_continuation_opcode_cycle
}

pub(crate) fn smp_bootstrap_span(
    steps: &[SmpBootstrapInstructionStep],
    pattern_len: usize,
    repeat_count: usize,
) -> SmpBootstrapInstructionSpan {
    let absolute_start_cycle = steps[0].absolute_start_cycle;
    let repeat_cycle_stride = steps[pattern_len - 1].absolute_end_cycle - absolute_start_cycle;
    let absolute_end_cycle = steps[pattern_len * repeat_count - 1].absolute_end_cycle;
    let instructions = steps[..pattern_len]
        .iter()
        .map(|step| SmpBootstrapPatternInstruction {
            start_cycle_offset: step.absolute_start_cycle - absolute_start_cycle,
            end_cycle_offset: step.absolute_end_cycle - absolute_start_cycle,
            origin_pc: step.origin_pc,
            opcode: step.opcode,
            boundary_opcode_cycle: step.boundary_opcode_cycle,
            op_step_calls: step.op_step_calls,
            max_continuation_opcode_cycle: step.max_continuation_opcode_cycle,
        })
        .collect();
    SmpBootstrapInstructionSpan {
        absolute_start_cycle,
        absolute_end_cycle,
        repeat_count,
        repeat_cycle_stride,
        instructions,
    }
}

pub(crate) fn compact_smp_bootstrap_steps(
    steps: &[SmpBootstrapInstructionStep],
) -> Vec<SmpBootstrapInstructionSpan> {
    let mut spans = Vec::new();
    let mut literal_start = 0;
    let mut index = 0;
    while index < steps.len() {
        let mut best = None::<(usize, usize, usize)>;
        for pattern_len in 1..=16.min((steps.len() - index) / 2) {
            let mut repeat_count = 1;
            while index + (repeat_count + 1) * pattern_len <= steps.len()
                && (0..pattern_len).all(|offset| {
                    smp_bootstrap_steps_match(
                        &steps[index + offset],
                        &steps[index + repeat_count * pattern_len + offset],
                    )
                })
            {
                repeat_count += 1;
            }
            let saved_instructions = (repeat_count - 1) * pattern_len;
            if repeat_count >= 2
                && best.is_none_or(|(best_saved, best_len, _)| {
                    (saved_instructions, pattern_len) > (best_saved, best_len)
                })
            {
                best = Some((saved_instructions, pattern_len, repeat_count));
            }
        }
        let Some((_, pattern_len, repeat_count)) = best else {
            index += 1;
            continue;
        };
        if literal_start < index {
            let literal = &steps[literal_start..index];
            spans.push(smp_bootstrap_span(literal, literal.len(), 1));
        }
        let repeated = &steps[index..index + pattern_len * repeat_count];
        spans.push(smp_bootstrap_span(repeated, pattern_len, repeat_count));
        index += pattern_len * repeat_count;
        literal_start = index;
    }
    if literal_start < steps.len() {
        let literal = &steps[literal_start..];
        spans.push(smp_bootstrap_span(literal, literal.len(), 1));
    }
    spans
}

pub(crate) fn compact_smp_bootstrap_instruction_sequence(
    instructions: &[crate::libretro_core::LibretroSmpInstruction],
    first_cc: &crate::libretro_core::LibretroSmpOutputPortWrite,
) -> Result<SmpBootstrapInstructionSequence, String> {
    let cc_index = instructions
        .iter()
        .position(|instruction| {
            instruction.program_counter == first_cc.origin_pc
                && instruction.opcode == first_cc.opcode
                && instruction.absolute_cycle <= first_cc.absolute_cycle
        })
        .ok_or_else(|| {
            "SMP trace has no instruction boundary owning the first CC write".to_string()
        })?;
    if cc_index + 1 >= instructions.len() {
        return Err("SMP trace has no successor boundary after the first CC write".to_string());
    }
    if instructions[cc_index + 1].absolute_cycle != first_cc.absolute_cycle {
        return Err(format!(
            "first CC write is at cycle {}, but its successor boundary is at {}",
            first_cc.absolute_cycle,
            instructions[cc_index + 1].absolute_cycle
        ));
    }
    let handoff_index = smp_bootstrap_handoff_index(instructions)
        .ok_or_else(|| "SMP trace has no final $fffb/1f -> $0800 handoff".to_string())?;
    let mut steps = Vec::with_capacity(handoff_index + 1);
    for index in 0..=handoff_index {
        let instruction = &instructions[index];
        let absolute_end_cycle = instructions[index + 1].absolute_cycle;
        if absolute_end_cycle < instruction.absolute_cycle {
            return Err(format!(
                "SMP instruction cycle regressed at trace index {index}: {} -> {absolute_end_cycle}",
                instruction.absolute_cycle
            ));
        }
        steps.push(SmpBootstrapInstructionStep {
            absolute_start_cycle: instruction.absolute_cycle,
            absolute_end_cycle,
            origin_pc: instruction.program_counter,
            opcode: instruction.opcode,
            boundary_opcode_cycle: instruction.boundary_opcode_cycle,
            op_step_calls: instruction.op_step_calls,
            max_continuation_opcode_cycle: instruction.max_continuation_opcode_cycle,
        });
    }
    if steps.first().map(|step| step.absolute_start_cycle) != Some(0) {
        return Err(
            "SMP bootstrap instruction trace does not begin at reset cycle zero".to_string(),
        );
    }
    let absolute_end_cycle = steps.last().unwrap().absolute_end_cycle;
    Ok(SmpBootstrapInstructionSequence {
        encoding: "repeated-span-v1",
        instruction_count: steps.len(),
        absolute_start_cycle: 0,
        absolute_end_cycle,
        spans: compact_smp_bootstrap_steps(&steps),
    })
}

pub(crate) fn snes9x_smp_trace_provenance(
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let lock: serde_json::Value = serde_json::from_str(include_str!(
        "../../../external/snes9x-libretro/oracle-lock.json"
    ))?;
    let source_revision = lock
        .get("source_revision")
        .and_then(serde_json::Value::as_str)
        .ok_or("oracle-lock.json has no source_revision")?;
    let trace_patch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-trace.patch");
    Ok(serde_json::json!({
        "kind": "provenance",
        "schema": 1,
        "core": {
            "library_name": oracle.library_name,
            "library_version": oracle.library_version,
            "sha256": parity::evidence::sha256_file(Path::new(core_path))?,
        },
        "rom": {
            "sha256": parity::evidence::sha256_file(Path::new(rom_path))?,
        },
        "source": {
            "revision": source_revision,
            "trace_patch_sha256": parity::evidence::sha256_file(&trace_patch)?,
        },
        "cpu_to_smp_ratio": {
            "numerator": 15_664,
            "denominator": 328_125,
        },
    }))
}

pub(crate) fn write_snes9x_smp_first_nmi_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
) -> Result<(), Box<dyn Error>> {
    let mut provenance = snes9x_smp_trace_provenance(core_path, rom_path, oracle)?;
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl");
    provenance
        .as_object_mut()
        .ok_or("Snes9x trace provenance is not a JSON object")?
        .insert(
            "prefix_fixture".to_string(),
            serde_json::json!({
                "path": "external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl",
                "sha256": parity::evidence::sha256_file(&prefix_fixture)?,
                "terminal_event": "final_$fffb/1f_to_$0800_ipl_handoff",
            }),
        );
    serde_json::to_writer(&mut *writer, &provenance)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub(crate) fn write_snes9x_first_nmi_dma_setup_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    load_sram_path: Option<&Path>,
    initial_sram: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut provenance = snes9x_smp_trace_provenance(core_path, rom_path, oracle)?;
    let core_receipt_path = PathBuf::from(format!("{core_path}.json"));
    let core_receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&core_receipt_path).map_err(|error| {
            format!(
                "first-NMI DMA-setup core has no readable build receipt {}: {error}",
                core_receipt_path.display()
            )
        })?)?;
    let recorded_core_sha = core_receipt["core_sha256"]
        .as_str()
        .ok_or("first-NMI DMA-setup core receipt has no core_sha256")?;
    if recorded_core_sha != provenance["core"]["sha256"] {
        return Err("first-NMI DMA-setup core receipt does not bind the loaded dylib".into());
    }
    if core_receipt["variant"] != "trace"
        || core_receipt["source_revision"] != provenance["source"]["revision"]
        || core_receipt["patch_sha256s"]
            .as_array()
            .is_none_or(|patches| patches.is_empty())
    {
        return Err("first-NMI DMA-setup core receipt is not a pinned trace build".into());
    }
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl");
    let provenance = provenance
        .as_object_mut()
        .ok_or("Snes9x trace provenance is not a JSON object")?;
    provenance.insert(
        "prefix_fixture".to_string(),
        serde_json::json!({
            "path": "external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl",
            "sha256": parity::evidence::sha256_file(&prefix_fixture)?,
            "terminal_event": "completed_$0080e1_lda_$2140_semantic_and_kind2_timing",
        }),
    );
    provenance.insert(
        "core_build_receipt".to_string(),
        serde_json::json!({
            "schema": core_receipt["schema"],
            "variant": core_receipt["variant"],
            "source_revision": core_receipt["source_revision"],
            "patch_sha256": core_receipt["patch_sha256"],
            "patch_sha256s": core_receipt["patch_sha256s"],
            "sha256": parity::evidence::sha256_file(&core_receipt_path)?,
        }),
    );
    provenance.insert(
        "initial_sram".to_string(),
        first_nmi_dma_setup_initial_sram_provenance(load_sram_path, initial_sram)?,
    );
    serde_json::to_writer(&mut *writer, &provenance)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub(crate) fn write_snes9x_first_nmi_dma_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    load_sram_path: Option<&Path>,
    initial_sram: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut provenance = snes9x_smp_trace_provenance(core_path, rom_path, oracle)?;
    let core_receipt_path = PathBuf::from(format!("{core_path}.json"));
    let core_receipt: serde_json::Value = serde_json::from_slice(&fs::read(&core_receipt_path)?)?;
    let patch_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-dma-ledger.patch");
    let patch_sha256 = parity::evidence::sha256_file(&patch_path)?;
    let patch_sha256s = core_receipt["patch_sha256s"]
        .as_array()
        .ok_or("first-NMI DMA core receipt has no patch_sha256s")?;
    if core_receipt["variant"] != "trace"
        || core_receipt["source_revision"] != provenance["source"]["revision"]
        || core_receipt["core_sha256"] != provenance["core"]["sha256"]
        || patch_sha256s.len() != 5
        || patch_sha256s.last().and_then(serde_json::Value::as_str) != Some(patch_sha256.as_str())
    {
        return Err(
            "first-NMI DMA capture requires the pinned five-patch DMA-ledger trace core".into(),
        );
    }
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl");
    let provenance = provenance
        .as_object_mut()
        .ok_or("Snes9x trace provenance is not a JSON object")?;
    provenance.insert(
        "prefix_fixture".to_string(),
        serde_json::json!({
            "path": "external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl",
            "sha256": parity::evidence::sha256_file(&prefix_fixture)?,
            "terminal_event": "excluded_$008a35_raw_fetch_V226_H714_to_H722",
        }),
    );
    provenance.insert(
        "core_build_receipt".to_string(),
        serde_json::json!({
            "schema": core_receipt["schema"],
            "variant": core_receipt["variant"],
            "source_revision": core_receipt["source_revision"],
            "patch_sha256": core_receipt["patch_sha256"],
            "patch_sha256s": core_receipt["patch_sha256s"],
            "sha256": parity::evidence::sha256_file(&core_receipt_path)?,
        }),
    );
    provenance.insert(
        "initial_sram".to_string(),
        first_nmi_dma_setup_initial_sram_provenance(load_sram_path, initial_sram)?,
    );
    provenance.insert(
        "dma_ledger_patch".to_string(),
        serde_json::json!({
            "path": "external/snes9x-libretro/patches/zelda3-dma-ledger.patch",
            "sha256": patch_sha256,
            "core_route_selector": false,
            "buffer_scope": "one retro_run",
            "overflow_policy": "fail-closed",
        }),
    );
    serde_json::to_writer(&mut *writer, &provenance)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub(crate) fn write_snes9x_first_nmi_return_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    load_sram_path: Option<&Path>,
    initial_sram: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut provenance = snes9x_smp_trace_provenance(core_path, rom_path, oracle)?;
    let core_receipt_path = PathBuf::from(format!("{core_path}.json"));
    let core_receipt: serde_json::Value = serde_json::from_slice(&fs::read(&core_receipt_path)?)?;
    let dma_patch_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-dma-ledger.patch");
    let dma_patch_sha256 = parity::evidence::sha256_file(&dma_patch_path)?;
    let patch_sha256s = core_receipt["patch_sha256s"]
        .as_array()
        .ok_or("first-NMI return core receipt has no patch_sha256s")?;
    if core_receipt["variant"] != "trace"
        || core_receipt["source_revision"] != provenance["source"]["revision"]
        || core_receipt["core_sha256"] != provenance["core"]["sha256"]
        || patch_sha256s.len() != 5
        || patch_sha256s.last().and_then(serde_json::Value::as_str)
            != Some(dma_patch_sha256.as_str())
    {
        return Err(
            "first-NMI return capture requires the pinned five-patch DMA-ledger trace core".into(),
        );
    }
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma.jsonl");
    let provenance = provenance
        .as_object_mut()
        .ok_or("Snes9x trace provenance is not a JSON object")?;
    provenance.insert(
        "prefix_fixture".to_string(),
        serde_json::json!({
            "path": "external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma.jsonl",
            "sha256": parity::evidence::sha256_file(&prefix_fixture)?,
            "terminal_event": "completed_$008a35_sta_$420b_07_and_observed_$008a38_raw_fetch",
        }),
    );
    provenance.insert(
        "core_build_receipt".to_string(),
        serde_json::json!({
            "schema": core_receipt["schema"],
            "variant": core_receipt["variant"],
            "source_revision": core_receipt["source_revision"],
            "patch_sha256": core_receipt["patch_sha256"],
            "patch_sha256s": core_receipt["patch_sha256s"],
            "sha256": parity::evidence::sha256_file(&core_receipt_path)?,
        }),
    );
    provenance.insert(
        "initial_sram".to_string(),
        first_nmi_dma_setup_initial_sram_provenance(load_sram_path, initial_sram)?,
    );
    provenance.insert(
        "trace_domains".to_string(),
        serde_json::json!({
            "generic_text": ["frame", "hdma"],
            "implicit_generic_text": ["video/presented"],
            "cpu_timing": "all_source_transactions_in_one_retro_run",
            "dma": "all_complete_outers_after_committed_prefix",
            "cpu_apui": "all_joined_post_anchor_accesses_with_dsp_before_after",
            "smp_output_ports": "all_joined_post_anchor_writes",
            "capacity_policy": "fail_closed",
            "core_route_selector": false,
        }),
    );
    serde_json::to_writer(&mut *writer, &provenance)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub(crate) fn first_nmi_dma_setup_initial_sram_provenance(
    load_sram_path: Option<&Path>,
    initial_sram: &[u8],
) -> Result<serde_json::Value, Box<dyn Error>> {
    let path = load_sram_path.ok_or(
        "first-NMI DMA-setup capture requires --load-sram routes/full_run/comparisons/continuous-audio/initial.srm",
    )?;
    let source_sram = fs::read(path).map_err(|error| {
        format!(
            "failed to read first-NMI DMA-setup initial SRAM {}: {error}",
            path.display()
        )
    })?;
    if source_sram != initial_sram {
        return Err(
            "first-NMI DMA-setup SRAM file does not match the bytes loaded into Snes9x".into(),
        );
    }
    if initial_sram.len() != FIRST_NMI_DMA_SETUP_INITIAL_SRAM_BYTES {
        return Err(format!(
            "first-NMI DMA-setup initial SRAM has {} bytes, expected {}",
            initial_sram.len(),
            FIRST_NMI_DMA_SETUP_INITIAL_SRAM_BYTES
        )
        .into());
    }
    let sha256 = parity::evidence::sha256_bytes(initial_sram);
    if sha256 != FIRST_NMI_DMA_SETUP_INITIAL_SRAM_SHA256 {
        return Err(format!(
            "first-NMI DMA-setup initial SRAM SHA-256 is {sha256}, expected {FIRST_NMI_DMA_SETUP_INITIAL_SRAM_SHA256}"
        )
        .into());
    }
    let slot_marker = initial_sram
        .get(
            FIRST_NMI_DMA_SETUP_VALID_SLOT_MARKER_OFFSET
                ..FIRST_NMI_DMA_SETUP_VALID_SLOT_MARKER_OFFSET + 2,
        )
        .ok_or("first-NMI DMA-setup initial SRAM has no save-slot marker")?;
    if slot_marker != [0xaa, 0x55] {
        return Err(
            "first-NMI DMA-setup initial SRAM has no AA55 valid-slot marker at $03e5".into(),
        );
    }
    Ok(serde_json::json!({
        "source": "routes/full_run/comparisons/continuous-audio/initial.srm",
        "sha256": sha256,
        "bytes": initial_sram.len(),
        "valid_slot_marker": {
            "offset": FIRST_NMI_DMA_SETUP_VALID_SLOT_MARKER_OFFSET,
            "bytes": [0xaa, 0x55],
        },
    }))
}

pub(crate) fn write_snes9x_smp_bootstrap_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    initial_oracle_state: &[u8],
) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer(
        &mut *writer,
        &snes9x_smp_trace_provenance(core_path, rom_path, oracle)?,
    )?;
    writer.write_all(b"\n")?;

    let snd = crate::snes9x_apu_tools::snes9x_snapshot_block(initial_oracle_state, b"SND")
        .map_err(|error| format!("failed to parse initial Snes9x SND block: {error}"))?;
    const RAM_BYTES: usize = 0x1_0000;
    const SMP_INT_COUNT: usize = 41;
    if snd.len() < RAM_BYTES + SMP_INT_COUNT * 4 {
        return Err(format!("initial Snes9x SND block is truncated: {} bytes", snd.len()).into());
    }
    let value = |index: usize| {
        let start = RAM_BYTES + index * 4;
        u32::from_le_bytes(snd[start..start + 4].try_into().unwrap())
    };
    let status = ((value(8) != 0) as u8) << 7
        | ((value(9) != 0) as u8) << 6
        | ((value(10) != 0) as u8) << 5
        | ((value(11) != 0) as u8) << 4
        | ((value(12) != 0) as u8) << 3
        | ((value(13) != 0) as u8) << 2
        | ((value(14) != 0) as u8) << 1
        | (value(15) != 0) as u8;
    let timers = (0..3)
        .map(|timer| {
            let base = 20 + timer * 5;
            serde_json::json!({
                "enabled": value(base) != 0,
                "target": value(base + 1),
                "stage1_ticks": value(base + 2),
                "stage2_ticks": value(base + 3),
                "stage3_ticks": value(base + 4),
            })
        })
        .collect::<Vec<_>>();
    let (cpu_model_5a22, cpu_model_identity, wram_refresh_position) = oracle
        .debug_cpu_timing_model()
        .ok_or("SMP-bootstrap trace requires Snes9x CPU timing-model instrumentation")?;
    serde_json::to_writer(
        &mut *writer,
        &serde_json::json!({
            "kind": "reset-state",
            "absolute_cycle": 0,
            "clock": value(0) as i32,
            "opcode": value(1),
            "opcode_cycle": value(2),
            "pc": value(3),
            "sp": value(4),
            "a": value(5),
            "x": value(6),
            "y": value(7),
            "status": status,
            "ipl_rom_enabled": value(16) != 0,
            "dsp_address": value(17),
            "auxiliary_ram": [value(18), value(19)],
            "output_ports": &snd[0xf4..0xf8],
            "timers": timers,
            "ram_nonzero_bytes": snd[..RAM_BYTES].iter().filter(|byte| **byte != 0).count(),
            "cpu_reference_time": 0,
            "cpu_remainder": 0,
            "cpu_model_5a22": cpu_model_5a22,
            "cpu_model_identity": cpu_model_identity,
            "wram_refresh_position": wram_refresh_position,
        }),
    )?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}
