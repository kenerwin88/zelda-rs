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
    assert_eq!(
        sequence["encoding"],
        "columnar-signed-delta-zero-rle-varint-base64-v1"
    );
    let fields = sequence["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let record_count = sequence["record_count"].as_u64().unwrap() as usize;
    let bytes = decode_base64(sequence["data_base64"].as_str().unwrap());
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
                assert!(run_length != 0 && values.len() + run_length <= record_count);
                values.resize(values.len() + run_length, previous);
            } else {
                let zigzag = encoded_delta - 1;
                let delta = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
                previous = previous.checked_add(delta).unwrap();
                values.push(previous);
            }
        }
        assert!(column.is_empty());
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
