use serde_json::Value;

const FIXTURE: &str =
    include_str!("../../../external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CpuApuAccess {
    pub frame: u32,
    pub port: u8,
    pub value: u8,
    pub v_counter: u16,
    pub cpu_cycle: u16,
    pub program_counter: u32,
    pub apu_cycle_before: u32,
    pub apu_cycle_after: u32,
    pub is_read: bool,
}

impl CpuApuAccess {
    pub(crate) fn absolute_master_cycle(self) -> u64 {
        crate::CpuFieldTiming::NON_INTERLACE_EVEN.master_cycles_at(
            u64::from(self.frame),
            crate::CpuRasterPosition::new(self.v_counter, self.cpu_cycle),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SmpOutputPortWrite {
    pub absolute_cycle: u32,
    pub port: u8,
    pub value: u8,
    pub origin_pc: u16,
    pub opcode: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SmpInstructionBoundary {
    pub origin_pc: u16,
    pub opcode: u8,
    pub absolute_start_cycle: u32,
    pub absolute_end_cycle: u32,
    pub op_step_calls: u8,
    pub max_continuation_opcode_cycle: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CpuTimingTransaction {
    pub frame: u64,
    pub start_field_index: u64,
    pub end_field_index: u64,
    pub kind: u8,
    pub duration: u8,
    pub origin_pc: u32,
    pub opcode: u8,
    pub start_v_counter: u16,
    pub start_cpu_cycle: u16,
    pub end_v_counter: u16,
    pub end_cpu_cycle: u16,
    pub cpu_model_identity: u8,
    pub cpu_model_5a22: u8,
    pub start_wram_refresh_position: u16,
    pub end_wram_refresh_position: u16,
}

impl CpuTimingTransaction {
    pub(crate) fn absolute_start_master_cycle(self) -> u64 {
        fixture_cpu_cycles_at(
            self.start_field_index,
            self.start_v_counter,
            self.start_cpu_cycle,
        )
    }

    pub(crate) fn absolute_end_master_cycle(self) -> u64 {
        fixture_cpu_cycles_at(self.end_field_index, self.end_v_counter, self.end_cpu_cycle)
    }
}

fn fixture_cpu_cycles_at(field_index: u64, scanline: u16, cpu_cycle: u16) -> u64 {
    let timing = crate::CpuFieldTiming::NON_INTERLACE_EVEN;
    let mut within_field =
        u64::from(scanline) * u64::from(crate::MASTER_CYCLES_PER_SCANLINE) + u64::from(cpu_cycle);
    // Snes9x may retain V240:H1360+ until a later AddCycles transaction drains
    // the short-line HMax. Only a post-HMax V241+ coordinate has consumed the
    // missing four physical clocks.
    if timing.field_is_odd(field_index) && scanline > 240 {
        within_field -= 4;
    }
    timing.field_start_master_cycles(field_index) + within_field
}

pub(crate) fn records() -> Vec<Value> {
    FIXTURE
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub(crate) fn cpu_apu_accesses(bootstrap: &Value) -> Vec<CpuApuAccess> {
    let sequence = decode_delta_sequence(&bootstrap["cpu_apu_access_sequence"]);
    sequence
        .rows
        .iter()
        .map(|row| CpuApuAccess {
            frame: sequence.value(row, "frame") as u32,
            port: sequence.value(row, "port") as u8,
            value: sequence.value(row, "value") as u8,
            v_counter: sequence.value(row, "v_counter") as u16,
            cpu_cycle: sequence.value(row, "cpu_cycle") as u16,
            program_counter: sequence.value(row, "program_counter") as u32,
            apu_cycle_before: sequence.value(row, "apu_cycle_before") as u32,
            apu_cycle_after: sequence.value(row, "apu_cycle_after") as u32,
            is_read: sequence.value(row, "is_read") != 0,
        })
        .collect()
}

pub(crate) fn split_first_cc_cpu_accesses(
    accesses: &[CpuApuAccess],
) -> (&[CpuApuAccess], &[CpuApuAccess]) {
    let reset_write_count = 4;
    assert_eq!(
        accesses[..reset_write_count]
            .iter()
            .map(|event| (event.port, event.value, event.is_read))
            .collect::<Vec<_>>(),
        [(0, 0, false), (1, 0, false), (2, 0, false), (3, 0, false)]
    );
    let handshake_start = accesses[reset_write_count..]
        .windows(2)
        .position(|pair| {
            pair[0].is_read
                && pair[1].is_read
                && pair[0].value == 0
                && pair[1].value == 0
                && pair[0].program_counter == pair[1].program_counter
                && pair[1].absolute_master_cycle() == pair[0].absolute_master_cycle() + 6
                && pair[0].apu_cycle_after == 2_397
                && pair[1].apu_cycle_after == 2_398
        })
        .expect("fixture omitted the first resumable-IPL handshake read")
        + reset_write_count;
    let handshake_end = accesses[handshake_start..]
        .iter()
        .position(|event| event.is_read && event.port == 0 && event.value == 0xcc)
        .expect("fixture omitted the first CPU read acknowledging CC")
        + handshake_start;
    (
        &accesses[..reset_write_count],
        &accesses[handshake_start..=handshake_end],
    )
}

pub(crate) fn smp_output_port_writes(bootstrap: &Value) -> Vec<SmpOutputPortWrite> {
    let sequence = decode_delta_sequence(&bootstrap["smp_output_port_write_sequence"]);
    sequence
        .rows
        .iter()
        .map(|row| SmpOutputPortWrite {
            absolute_cycle: sequence.value(row, "absolute_cycle") as u32,
            port: sequence.value(row, "port") as u8,
            value: sequence.value(row, "value") as u8,
            origin_pc: sequence.value(row, "origin_pc") as u16,
            opcode: sequence.value(row, "opcode") as u8,
        })
        .collect()
}

pub(crate) fn first_cc_output_port_writes(writes: &[SmpOutputPortWrite]) -> &[SmpOutputPortWrite] {
    let cc_index = writes
        .iter()
        .position(|event| event.port == 0 && event.value == 0xcc)
        .expect("fixture omitted the first SMP CC acknowledgement");
    &writes[..=cc_index]
}

pub(crate) fn smp_instruction_boundaries(bootstrap: &Value) -> Vec<SmpInstructionBoundary> {
    let sequence = &bootstrap["smp_instruction_boundary_sequence"];
    assert_eq!(sequence["encoding"], "repeated-span-v1");
    let mut boundaries =
        Vec::with_capacity(sequence["instruction_count"].as_u64().unwrap() as usize);
    for span in sequence["spans"].as_array().unwrap() {
        let span_start = span["absolute_start_cycle"].as_u64().unwrap() as u32;
        let stride = span["repeat_cycle_stride"].as_u64().unwrap() as u32;
        for repeat in 0..span["repeat_count"].as_u64().unwrap() as u32 {
            let base = span_start + repeat * stride;
            for instruction in span["instructions"].as_array().unwrap() {
                boundaries.push(SmpInstructionBoundary {
                    origin_pc: instruction["origin_pc"].as_u64().unwrap() as u16,
                    opcode: instruction["opcode"].as_u64().unwrap() as u8,
                    absolute_start_cycle: base
                        + instruction["start_cycle_offset"].as_u64().unwrap() as u32,
                    absolute_end_cycle: base
                        + instruction["end_cycle_offset"].as_u64().unwrap() as u32,
                    op_step_calls: instruction["op_step_calls"].as_u64().unwrap() as u8,
                    max_continuation_opcode_cycle: instruction["max_continuation_opcode_cycle"]
                        .as_u64()
                        .unwrap() as u8,
                });
            }
        }
    }
    assert_eq!(
        boundaries.len(),
        sequence["instruction_count"].as_u64().unwrap() as usize
    );
    boundaries
}

pub(crate) fn cpu_timing_transactions_through_first_cc(
    bootstrap: &Value,
) -> Vec<CpuTimingTransaction> {
    let cpu_accesses = cpu_apu_accesses(bootstrap);
    let (_, first_cc_handshake) = split_first_cc_cpu_accesses(&cpu_accesses);
    let first_cc = *first_cc_handshake
        .last()
        .expect("fixture omitted the first CPU CC acknowledgement");
    // Every recorded Snes9x source transaction costs at least one six-master-
    // cycle S-CPU cycle, so this is a source-derived upper bound rather than a
    // fixture-size guess.
    let prefix_limit = first_cc.absolute_master_cycle().div_ceil(6) as usize + 1;
    let sequence =
        decode_delta_sequence_prefix(&bootstrap["cpu_timing_transaction_sequence"], prefix_limit);
    let mut physical_field_index = 0u64;
    let mut transactions = sequence
        .rows
        .iter()
        .map(|row| {
            let start_v_counter = sequence.value(row, "start_v_counter") as u16;
            let end_v_counter = sequence.value(row, "end_v_counter") as u16;
            let start_field_index = physical_field_index;
            let end_field_index = start_field_index + u64::from(end_v_counter < start_v_counter);
            physical_field_index = end_field_index;
            CpuTimingTransaction {
                frame: sequence.value(row, "frame") as u64,
                start_field_index,
                end_field_index,
                kind: sequence.value(row, "kind") as u8,
                duration: sequence.value(row, "duration") as u8,
                origin_pc: sequence.value(row, "origin_pc") as u32,
                opcode: sequence.value(row, "opcode") as u8,
                start_v_counter,
                start_cpu_cycle: sequence.value(row, "start_cpu_cycle") as u16,
                end_v_counter,
                end_cpu_cycle: sequence.value(row, "end_cpu_cycle") as u16,
                cpu_model_identity: sequence.value(row, "cpu_model_identity") as u8,
                cpu_model_5a22: sequence.value(row, "cpu_model_5a22") as u8,
                start_wram_refresh_position: sequence.value(row, "start_wram_refresh_position")
                    as u16,
                end_wram_refresh_position: sequence.value(row, "end_wram_refresh_position") as u16,
            }
        })
        .collect::<Vec<_>>();
    let first_cc_transaction = transactions
        .iter()
        .position(|transaction| {
            transaction.frame == u64::from(first_cc.frame)
                && transaction.kind == 2
                && transaction.start_v_counter == first_cc.v_counter
                && transaction.start_cpu_cycle == first_cc.cpu_cycle
        })
        .unwrap_or_else(|| {
            let same_timestamp = transactions
                .iter()
                .filter(|transaction| {
                    transaction.frame == u64::from(first_cc.frame)
                        && transaction.start_v_counter == first_cc.v_counter
                        && transaction.start_cpu_cycle == first_cc.cpu_cycle
                })
                .collect::<Vec<_>>();
            panic!(
                "timing fixture omitted the getset transaction for first CC {first_cc:?}; same timestamp: {same_timestamp:?}"
            )
        });
    transactions.truncate(first_cc_transaction + 1);
    transactions
}

pub(crate) fn visit_cpu_timing_transactions(
    bootstrap: &Value,
    mut visit: impl FnMut(CpuTimingTransaction),
) {
    let sequence = &bootstrap["cpu_timing_transaction_sequence"];
    assert_eq!(
        sequence["fields"],
        serde_json::json!([
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
        ])
    );
    let encoding = sequence["encoding"].as_str().unwrap();
    assert_eq!(
        encoding,
        "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
    );
    let record_count = sequence["record_count"].as_u64().unwrap() as usize;
    let compressed = decode_base64(sequence["data_base64"].as_str().unwrap());
    let encoded = zstd::stream::decode_all(compressed.as_slice()).unwrap();
    let mut remaining = encoded.as_slice();
    let mut columns = Vec::with_capacity(13);
    for _ in 0..13 {
        let column_len = read_varint(&mut remaining) as usize;
        let (column, rest) = remaining.split_at(column_len);
        remaining = rest;
        columns.push(DeltaColumnDecoder::new(column, record_count));
    }
    assert!(remaining.is_empty());

    let mut physical_field_index = 0u64;
    for _ in 0..record_count {
        let mut row = [0i64; 13];
        for (value, column) in row.iter_mut().zip(&mut columns) {
            *value = column.next().expect("fixture column ended early");
        }
        let start_field_index = physical_field_index;
        let end_field_index = start_field_index + u64::from(row[7] < row[5]);
        physical_field_index = end_field_index;
        visit(CpuTimingTransaction {
            frame: row[0] as u64,
            start_field_index,
            end_field_index,
            kind: row[1] as u8,
            duration: row[2] as u8,
            origin_pc: row[3] as u32,
            opcode: row[4] as u8,
            start_v_counter: row[5] as u16,
            start_cpu_cycle: row[6] as u16,
            end_v_counter: row[7] as u16,
            end_cpu_cycle: row[8] as u16,
            cpu_model_identity: row[9] as u8,
            cpu_model_5a22: row[10] as u8,
            start_wram_refresh_position: row[11] as u16,
            end_wram_refresh_position: row[12] as u16,
        });
    }
    assert!(columns.iter_mut().all(|column| column.next().is_none()));
}

struct DeltaColumnDecoder<'a> {
    encoded: &'a [u8],
    previous: i64,
    repeated_remaining: usize,
    remaining_values: usize,
}

impl<'a> DeltaColumnDecoder<'a> {
    fn new(encoded: &'a [u8], remaining_values: usize) -> Self {
        Self {
            encoded,
            previous: 0,
            repeated_remaining: 0,
            remaining_values,
        }
    }
}

impl Iterator for DeltaColumnDecoder<'_> {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_values == 0 {
            assert!(self.encoded.is_empty() && self.repeated_remaining == 0);
            return None;
        }
        if self.repeated_remaining == 0 {
            let encoded_delta = read_varint(&mut self.encoded);
            if encoded_delta == 0 {
                self.repeated_remaining = read_varint(&mut self.encoded) as usize;
                assert!(
                    self.repeated_remaining != 0
                        && self.repeated_remaining <= self.remaining_values
                );
            } else {
                let zigzag = encoded_delta - 1;
                let delta = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
                self.previous = self.previous.checked_add(delta).unwrap();
                self.remaining_values -= 1;
                return Some(self.previous);
            }
        }
        self.repeated_remaining -= 1;
        self.remaining_values -= 1;
        Some(self.previous)
    }
}

struct DecodedDeltaSequence {
    fields: Vec<String>,
    rows: Vec<Vec<i64>>,
}

impl DecodedDeltaSequence {
    fn value(&self, row: &[i64], field: &str) -> i64 {
        row[self
            .fields
            .iter()
            .position(|candidate| candidate == field)
            .unwrap_or_else(|| panic!("fixture sequence omitted field {field}"))]
    }
}

fn decode_delta_sequence(sequence: &Value) -> DecodedDeltaSequence {
    decode_delta_sequence_prefix(
        sequence,
        sequence["record_count"].as_u64().unwrap() as usize,
    )
}

fn decode_delta_sequence_prefix(sequence: &Value, prefix_limit: usize) -> DecodedDeltaSequence {
    let encoding = sequence["encoding"].as_str().unwrap();
    assert!(matches!(
        encoding,
        "columnar-signed-delta-zero-rle-varint-base64-v1"
            | "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
    ));
    let fields = sequence["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let declared_record_count = sequence["record_count"].as_u64().unwrap() as usize;
    let record_count = prefix_limit.min(declared_record_count);
    let mut bytes = decode_base64(sequence["data_base64"].as_str().unwrap());
    if encoding == "columnar-signed-delta-zero-rle-varint-zstd-base64-v1" {
        bytes = zstd::stream::decode_all(bytes.as_slice()).unwrap();
    }
    let mut encoded = bytes.as_slice();
    let mut columns = Vec::with_capacity(fields.len());
    for _ in &fields {
        let column_len = read_varint(&mut encoded) as usize;
        let (mut column, rest) = encoded.split_at(column_len);
        encoded = rest;
        let mut values = Vec::with_capacity(record_count);
        let mut previous = 0i64;
        while values.len() < record_count {
            let encoded_delta = read_varint(&mut column);
            if encoded_delta == 0 {
                let run_length = read_varint(&mut column) as usize;
                assert!(run_length != 0);
                values.resize((values.len() + run_length).min(record_count), previous);
            } else {
                let zigzag = encoded_delta - 1;
                let delta = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
                previous = previous.checked_add(delta).unwrap();
                values.push(previous);
            }
        }
        columns.push(values);
    }
    assert!(encoded.is_empty());

    let mut rows = vec![vec![0; fields.len()]; record_count];
    for (field, column) in columns.into_iter().enumerate() {
        for (row, value) in column.into_iter().enumerate() {
            rows[row][field] = value;
        }
    }
    DecodedDeltaSequence { fields, rows }
}

fn read_varint(bytes: &mut &[u8]) -> u64 {
    let mut value = 0u64;
    let mut shift = 0u32;
    loop {
        let (&byte, rest) = bytes.split_first().expect("truncated fixture varint");
        *bytes = rest;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return value;
        }
        shift += 7;
        assert!(shift < 64, "fixture varint overflowed u64");
    }
}

fn decode_base64(encoded: &str) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(encoded.len() / 4 * 3);
    for quartet in encoded.as_bytes().chunks_exact(4) {
        let a = base64_digit(quartet[0]);
        let b = base64_digit(quartet[1]);
        let c = (quartet[2] != b'=').then(|| base64_digit(quartet[2]));
        let d = (quartet[3] != b'=').then(|| base64_digit(quartet[3]));
        decoded.push((a << 2) | (b >> 4));
        if let Some(c) = c {
            decoded.push((b << 4) | (c >> 2));
            if let Some(d) = d {
                decoded.push((c << 6) | d);
            }
        }
    }
    decoded
}

fn base64_digit(byte: u8) -> u8 {
    match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => panic!("invalid fixture base64 digit"),
    }
}

#[test]
fn compact_fixture_sequences_reconstruct_declared_record_counts() {
    let fixture = records();
    let bootstrap = &fixture[2];
    let cpu_accesses = cpu_apu_accesses(bootstrap);
    assert_eq!(
        cpu_accesses.len(),
        bootstrap["cpu_apu_access_sequence"]["record_count"]
            .as_u64()
            .unwrap() as usize
    );
    assert!(
        cpu_accesses.last().unwrap().absolute_master_cycle()
            > cpu_accesses.first().unwrap().absolute_master_cycle()
    );
    assert_eq!(
        smp_output_port_writes(bootstrap).len(),
        bootstrap["smp_output_port_write_sequence"]["record_count"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        smp_instruction_boundaries(bootstrap).len(),
        bootstrap["smp_instruction_boundary_sequence"]["instruction_count"]
            .as_u64()
            .unwrap() as usize
    );
}

#[test]
fn cpu_timing_decoder_preserves_pending_short_hmax_and_physical_field_rollover() {
    let fixture = records();
    let bootstrap = &fixture[2];
    let timing = crate::CpuFieldTiming::NON_INTERLACE_EVEN;
    let mut pending_short_hmax = None;
    let mut field_rollover = None;
    visit_cpu_timing_transactions(bootstrap, |transaction| {
        if pending_short_hmax.is_none()
            && timing.field_is_odd(transaction.start_field_index)
            && transaction.start_v_counter == 240
            && transaction.end_v_counter == 240
            && transaction.end_cpu_cycle >= crate::SHORT_SCANLINE_END_CYCLE as u16
        {
            pending_short_hmax = Some(transaction);
        }
        if field_rollover.is_none() && transaction.end_v_counter < transaction.start_v_counter {
            field_rollover = Some(transaction);
        }
    });

    let pending = pending_short_hmax.expect("fixture omitted an odd V240 pending-HMax transaction");
    assert_eq!(pending.start_field_index, pending.end_field_index);
    assert_eq!(
        pending.absolute_end_master_cycle() - pending.absolute_start_master_cycle(),
        u64::from(pending.duration)
    );

    let rollover = field_rollover.expect("fixture omitted a V261-to-V0 field rollover");
    assert_eq!(rollover.end_field_index, rollover.start_field_index + 1);
    assert!(rollover.absolute_end_master_cycle() >= rollover.absolute_start_master_cycle());
}
