//! Split out of `snes9x_compare.rs` by topic (rng_trace). Mechanical move:
//! bodies are unchanged; private items became `pub(crate)` and `super::`
//! paths became `crate::` (the parent's super is the crate root).

use super::*;

#[derive(Debug)]
pub(crate) struct OracleRngTraceEvent {
    pub(crate) run: u32,
    pub(crate) pc: u64,
    pub(crate) value: u8,
    pub(crate) carry: u8,
}

pub(crate) fn oracle_rng_sample_from_record(
    record: &parity::trace_format::TraceRecord,
    expected_trace_run: u32,
    execution_frame: u32,
) -> Result<Option<RomRandomSample>, String> {
    if record.kind != parity::trace_format::KIND_RNG_WRITE {
        return Ok(None);
    }
    let event = OracleRngTraceEvent {
        run: u32::try_from(record.run)
            .map_err(|_| format!("live oracle RNG trace run {} overflows", record.run))?,
        pc: u64::from(record.pc),
        value: u8::try_from(record.value().unwrap_or(0))
            .map_err(|_| "live oracle RNG trace value exceeds one byte".to_string())?,
        carry: record.carry,
    };
    if event.pc & 0xffff != CARTRIDGE_RNG_STORE_PC_LOW16 {
        return Ok(None);
    }
    if event.run != expected_trace_run {
        return Err(format!(
            "live oracle RNG trace run {} arrived while expecting trace run {expected_trace_run} for Rust execution frame {execution_frame}",
            event.run,
        ));
    }
    if event.carry > 1 {
        return Err(format!(
            "live oracle RNG trace run {expected_trace_run} for execution frame {execution_frame} has invalid carry {}",
            event.carry
        ));
    }
    Ok(Some(RomRandomSample::with_carry(
        execution_frame,
        event.value,
        event.carry != 0,
    )))
}

pub(crate) fn trace_events_with_rom_rng(configured: Option<&str>) -> String {
    let mut events = configured
        .into_iter()
        .flat_map(|events| events.split(','))
        .map(str::trim)
        .filter(|event| !event.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !events.iter().any(|event| event == "rom-rng") {
        events.push("rom-rng".to_owned());
    }
    events.join(",")
}

pub(crate) struct LiveOracleRngTrace {
    pub(crate) path: PathBuf,
    pub(crate) reader: Option<parity::trace_format::TraceReader<fs::File>>,
}

impl LiveOracleRngTrace {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path, reader: None }
    }

    pub(crate) fn samples_for_run(
        &mut self,
        trace_run: u32,
        execution_frame: u32,
    ) -> Result<Vec<RomRandomSample>, String> {
        if self.reader.is_none() {
            match parity::trace_format::open_trace(&self.path, 0) {
                Ok(reader) => self.reader = Some(reader),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(Vec::new());
                }
                Err(error) => {
                    return Err(format!(
                        "failed to open live oracle RNG trace {}: {error}",
                        self.path.display()
                    ));
                }
            }
        }
        let reader = self.reader.as_mut().expect("reader initialized above");
        let mut samples = Vec::new();
        loop {
            let record = match reader.next_record() {
                Ok(Some(record)) => record,
                Ok(None) => break,
                Err(error) => {
                    return Err(format!(
                        "failed to read live oracle RNG trace {}: {error}",
                        self.path.display()
                    ))
                }
            };
            if let Some(sample) =
                oracle_rng_sample_from_record(&record, trace_run, execution_frame)?
            {
                samples.push(sample);
            }
        }
        Ok(samples)
    }
}

pub(crate) fn validate_oracle_rng_samples_for_run(
    expected: &[RomRandomSample],
    cursor: &mut usize,
    run: u32,
    actual: &[RomRandomSample],
) -> Result<(), String> {
    if expected
        .get(*cursor)
        .is_some_and(|sample| sample.execution_frame < run)
    {
        return Err(format!(
            "RNG script expected an unobserved cartridge call at frame {} before source run {run}",
            expected[*cursor].execution_frame
        ));
    }
    let start = *cursor;
    while expected
        .get(*cursor)
        .is_some_and(|sample| sample.execution_frame == run)
    {
        *cursor += 1;
    }
    let expected_run = &expected[start..*cursor];
    if expected_run == actual {
        return Ok(());
    }
    Err(format!(
        "RNG script/source mismatch at frame {run}: script={expected_run:?}, source={actual:?}"
    ))
}
