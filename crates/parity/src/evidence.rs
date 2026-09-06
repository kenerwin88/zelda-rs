use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

const INDEX_MAGIC: &[u8; 8] = b"Z3PTIDX1";
const INDEX_SCHEMA: u32 = 1;
const CACHE_SCHEMA: u64 = 1;
const CACHE_KIND: &str = "zelda3-content-addressed-oracle-evidence";
const EVENT_BYTES: usize = 32;
const RECORD_BYTES: u32 = 65;
const MISSING_U32: u32 = u32::MAX;

#[derive(Debug)]
struct TraceFields {
    event: String,
    run: u32,
    frame: Option<u32>,
    pc: Option<u32>,
    address: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TraceIndexHeader {
    pub schema: u32,
    pub kind: String,
    pub source: String,
    pub source_sha256: String,
    pub source_bytes: u64,
    pub manifest: String,
    pub manifest_sha256: String,
    pub comparison_start_frame: u32,
    pub records: u64,
    pub record_bytes: u32,
}

#[derive(Clone, Debug)]
struct TraceIndexRecord {
    run: u32,
    host_frame: u32,
    internal_frame: u32,
    pc: u32,
    address: u32,
    source_offset: u64,
    source_length: u32,
    event: String,
}

#[derive(Clone, Debug, Default)]
pub struct TraceQuery {
    pub host_frame: Option<u32>,
    pub run: Option<u32>,
    pub internal_frame: Option<u32>,
    pub pc: Option<u32>,
    pub wram: Option<(u32, u32)>,
    pub events: Vec<String>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CacheInventory {
    pub entries: u64,
    pub artifacts: u64,
    pub bytes: u64,
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let file =
        File::open(path).map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub struct Sha256Digest(Sha256);

impl Sha256Digest {
    pub fn new() -> Self {
        Self(Sha256::new())
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    /// Feed canonical RGB bytes from one or more tightly packed RGBA chunks.
    /// Packing a bounded block before updating SHA-256 preserves the exact byte
    /// stream while avoiding one digest call per pixel. The scratch storage is
    /// stack-owned and independent of frame size.
    pub fn update_rgb_from_rgba<'a, I>(&mut self, chunks: I)
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        const PACKED_BYTES: usize = 12 * 1024;
        let mut packed = [0u8; PACKED_BYTES];
        let mut used = 0usize;
        for chunk in chunks {
            for pixel in chunk.chunks_exact(4) {
                if used + 3 > packed.len() {
                    self.update(&packed[..used]);
                    used = 0;
                }
                packed[used..used + 3].copy_from_slice(&pixel[..3]);
                used += 3;
            }
        }
        if used != 0 {
            self.update(&packed[..used]);
        }
    }

    /// Feed signed samples in the canonical little-endian interleaved format
    /// without issuing one digest update per sample.
    pub fn update_i16_le(&mut self, samples: &[i16]) {
        const PACKED_BYTES: usize = 8 * 1024;
        let mut packed = [0u8; PACKED_BYTES];
        let mut used = 0usize;
        for sample in samples {
            if used + 2 > packed.len() {
                self.update(&packed[..used]);
                used = 0;
            }
            packed[used..used + 2].copy_from_slice(&sample.to_le_bytes());
            used += 2;
        }
        if used != 0 {
            self.update(&packed[..used]);
        }
    }

    pub fn finish(self) -> String {
        format!("{:x}", self.0.finalize())
    }
}

impl Default for Sha256Digest {
    fn default() -> Self {
        Self::new()
    }
}

fn load_json(path: &Path) -> Result<Value, String> {
    let data =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let value: Value = serde_json::from_slice(&data)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
    if !value.is_object() {
        return Err(format!("{} is not a JSON object", path.display()));
    }
    Ok(value)
}

fn manifest_start_frame(manifest: &Value, path: &Path) -> Result<u32, String> {
    let value = manifest
        .get("timing")
        .and_then(|timing| timing.get("start_frame"))
        .or_else(|| {
            manifest
                .get("cache_identity")
                .and_then(|identity| identity.get("start_frame"))
        })
        .and_then(Value::as_u64)
        .unwrap_or(0);
    u32::try_from(value)
        .map_err(|_| format!("{} timing.start_frame is outside u32", path.display()))
}

fn canonical_path(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", path.display()))
}

fn temporary_path(path: &Path, label: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("evidence");
    path.with_file_name(format!(".{name}.{}.{}.tmp", std::process::id(), label))
}

pub fn build_trace_index(
    trace_path: &Path,
    manifest_path: &Path,
    output_path: &Path,
) -> Result<TraceIndexHeader, String> {
    let trace_path = canonical_path(trace_path)?;
    let manifest_path = canonical_path(manifest_path)?;
    let manifest = load_json(&manifest_path)?;
    let start_frame = manifest_start_frame(&manifest, &manifest_path)?;
    let source_file = File::open(&trace_path)
        .map_err(|error| format!("cannot open {}: {error}", trace_path.display()))?;
    let mut source = BufReader::new(source_file);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let record_path = temporary_path(output_path, "records");
    let final_path = temporary_path(output_path, "index");
    let result = (|| {
        let record_file = File::create(&record_path)
            .map_err(|error| format!("cannot create {}: {error}", record_path.display()))?;
        let mut records_output = BufWriter::new(record_file);
        let mut digest = Sha256::new();
        // The pinned core writes Z3TRACE1 binary records: an 8-byte magic
        // followed by `u16`-framed records. The digest covers the raw bytes
        // so a changed source is detected exactly as before.
        let mut magic = [0_u8; 8];
        source
            .read_exact(&mut magic)
            .map_err(|error| format!("cannot read {}: {error}", trace_path.display()))?;
        if &magic != crate::trace_format::MAGIC {
            return Err(format!(
                "{} is not a Z3TRACE1 binary trace",
                trace_path.display()
            ));
        }
        digest.update(magic);
        let mut offset = 8_u64;
        let mut record_count = 0_u64;
        let mut previous_run = None;
        let mut framed = Vec::new();
        loop {
            let mut length_bytes = [0_u8; 2];
            match source.read_exact(&mut length_bytes) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(error) => return Err(format!("cannot read {}: {error}", trace_path.display())),
            }
            let body_length = usize::from(u16::from_le_bytes(length_bytes));
            framed.clear();
            framed.extend_from_slice(&length_bytes);
            framed.resize(2 + body_length, 0);
            source
                .read_exact(&mut framed[2..])
                .map_err(|error| format!("cannot read {}: {error}", trace_path.display()))?;
            let length = framed.len();
            digest.update(&framed);
            let decoded =
                crate::trace_format::TraceRecord::parse(&framed[2..]).map_err(|error| {
                    format!(
                        "invalid trace record {}:{}: {error}",
                        trace_path.display(),
                        record_count + 1
                    )
                })?;
            let fields = TraceFields {
                event: decoded.event().to_string(),
                run: u32::try_from(decoded.run).map_err(|_| {
                    format!(
                        "trace run overflow at {}:{}",
                        trace_path.display(),
                        record_count + 1
                    )
                })?,
                frame: Some(decoded.frame),
                pc: Some(decoded.pc),
                address: decoded.address(),
            };
            if fields.event.as_bytes().len() > EVENT_BYTES {
                return Err(format!(
                    "trace event name is longer than {EVENT_BYTES} bytes at {}:{}",
                    trace_path.display(),
                    record_count + 1
                ));
            }
            let host_frame = start_frame.checked_add(fields.run).ok_or_else(|| {
                format!(
                    "host frame overflow at {}:{}",
                    trace_path.display(),
                    record_count + 1
                )
            })?;
            if previous_run.is_some_and(|previous| fields.run < previous) {
                return Err(format!(
                    "trace retro_run regressed at {}:{}",
                    trace_path.display(),
                    record_count + 1
                ));
            }
            previous_run = Some(fields.run);
            let source_length = u32::try_from(length).map_err(|_| {
                format!(
                    "trace line is too long at {}:{}",
                    trace_path.display(),
                    record_count + 1
                )
            })?;
            let record = TraceIndexRecord {
                run: fields.run,
                host_frame,
                internal_frame: fields.frame.unwrap_or(MISSING_U32),
                pc: fields.pc.unwrap_or(MISSING_U32),
                address: fields.address.unwrap_or(MISSING_U32),
                source_offset: offset,
                source_length,
                event: fields.event,
            };
            write_record(&mut records_output, &record)?;
            offset += u64::from(source_length);
            record_count += 1;
        }
        records_output
            .flush()
            .map_err(|error| format!("cannot flush {}: {error}", record_path.display()))?;

        let header = TraceIndexHeader {
            schema: INDEX_SCHEMA,
            kind: "zelda3-trace-seek-index".to_string(),
            source: trace_path.display().to_string(),
            source_sha256: format!("{:x}", digest.finalize()),
            source_bytes: offset,
            manifest: manifest_path.display().to_string(),
            manifest_sha256: sha256_file(&manifest_path)?,
            comparison_start_frame: start_frame,
            records: record_count,
            record_bytes: RECORD_BYTES,
        };
        let encoded_header = serde_json::to_vec(&header)
            .map_err(|error| format!("cannot encode trace index header: {error}"))?;
        let header_length = u32::try_from(encoded_header.len())
            .map_err(|_| "trace index header is too large".to_string())?;
        let output_file = File::create(&final_path)
            .map_err(|error| format!("cannot create {}: {error}", final_path.display()))?;
        let mut output = BufWriter::new(output_file);
        output
            .write_all(INDEX_MAGIC)
            .and_then(|()| output.write_all(&header_length.to_le_bytes()))
            .and_then(|()| output.write_all(&encoded_header))
            .map_err(|error| format!("cannot write {}: {error}", final_path.display()))?;
        let mut records_input = BufReader::new(
            File::open(&record_path)
                .map_err(|error| format!("cannot open {}: {error}", record_path.display()))?,
        );
        io::copy(&mut records_input, &mut output)
            .map_err(|error| format!("cannot write {}: {error}", final_path.display()))?;
        output
            .flush()
            .map_err(|error| format!("cannot flush {}: {error}", final_path.display()))?;
        fs::rename(&final_path, output_path).map_err(|error| {
            format!(
                "cannot publish trace index {}: {error}",
                output_path.display()
            )
        })?;
        Ok(header)
    })();
    let _ = fs::remove_file(&record_path);
    let _ = fs::remove_file(&final_path);
    result
}

fn write_record(output: &mut impl Write, record: &TraceIndexRecord) -> Result<(), String> {
    for value in [
        record.run,
        record.host_frame,
        record.internal_frame,
        record.pc,
        record.address,
    ] {
        output
            .write_all(&value.to_le_bytes())
            .map_err(|error| format!("cannot write trace index record: {error}"))?;
    }
    output
        .write_all(&record.source_offset.to_le_bytes())
        .and_then(|()| output.write_all(&record.source_length.to_le_bytes()))
        .map_err(|error| format!("cannot write trace index record: {error}"))?;
    let event = record.event.as_bytes();
    output
        .write_all(&[event.len() as u8])
        .and_then(|()| output.write_all(event))
        .and_then(|()| output.write_all(&vec![0_u8; EVENT_BYTES - event.len()]))
        .map_err(|error| format!("cannot write trace index record: {error}"))
}

fn read_u32(input: &mut impl Read) -> Result<u32, String> {
    let mut bytes = [0_u8; 4];
    input
        .read_exact(&mut bytes)
        .map_err(|error| format!("cannot read trace index: {error}"))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_record(input: &mut impl Read) -> Result<TraceIndexRecord, String> {
    let run = read_u32(input)?;
    let host_frame = read_u32(input)?;
    let internal_frame = read_u32(input)?;
    let pc = read_u32(input)?;
    let address = read_u32(input)?;
    let mut offset_bytes = [0_u8; 8];
    input
        .read_exact(&mut offset_bytes)
        .map_err(|error| format!("cannot read trace index: {error}"))?;
    let source_offset = u64::from_le_bytes(offset_bytes);
    let source_length = read_u32(input)?;
    let mut event_length = [0_u8; 1];
    let mut event_bytes = [0_u8; EVENT_BYTES];
    input
        .read_exact(&mut event_length)
        .and_then(|()| input.read_exact(&mut event_bytes))
        .map_err(|error| format!("cannot read trace index: {error}"))?;
    let event_length = usize::from(event_length[0]);
    if event_length > EVENT_BYTES {
        return Err("trace index has an invalid event length".to_string());
    }
    let event = std::str::from_utf8(&event_bytes[..event_length])
        .map_err(|error| format!("trace index has invalid event text: {error}"))?
        .to_string();
    Ok(TraceIndexRecord {
        run,
        host_frame,
        internal_frame,
        pc,
        address,
        source_offset,
        source_length,
        event,
    })
}

fn read_index_header(input: &mut impl Read) -> Result<(TraceIndexHeader, u64), String> {
    let mut magic = [0_u8; 8];
    input
        .read_exact(&mut magic)
        .map_err(|error| format!("cannot read trace index: {error}"))?;
    if &magic != INDEX_MAGIC {
        return Err("unsupported trace index magic".to_string());
    }
    let header_length = read_u32(input)? as usize;
    if header_length > 1024 * 1024 {
        return Err("trace index header exceeds 1 MiB".to_string());
    }
    let mut encoded = vec![0_u8; header_length];
    input
        .read_exact(&mut encoded)
        .map_err(|error| format!("cannot read trace index header: {error}"))?;
    let header: TraceIndexHeader = serde_json::from_slice(&encoded)
        .map_err(|error| format!("cannot parse trace index header: {error}"))?;
    if header.schema != INDEX_SCHEMA
        || header.kind != "zelda3-trace-seek-index"
        || header.record_bytes != RECORD_BYTES
    {
        return Err(format!(
            "unsupported trace index schema in header: {header:?}"
        ));
    }
    Ok((header, 12 + header_length as u64))
}

fn canonical_lorom_pc(pc: u32) -> u32 {
    let address = pc & 0xffff;
    if address < 0x8000 {
        return pc;
    }
    (((pc >> 16) & 0x7f | 0x80) << 16) | address
}

fn record_matches(record: &TraceIndexRecord, query: &TraceQuery) -> bool {
    if query
        .host_frame
        .is_some_and(|value| record.host_frame != value)
        || query.run.is_some_and(|value| record.run != value)
        || query
            .internal_frame
            .is_some_and(|value| record.internal_frame != value)
        || query.pc.is_some_and(|value| {
            record.pc == MISSING_U32 || canonical_lorom_pc(record.pc) != canonical_lorom_pc(value)
        })
        || query.wram.is_some_and(|(first, last)| {
            record.address == MISSING_U32
                || record.address < first
                || record.address > last
                || !matches!(record.event.as_str(), "wram" | "wram-write")
        })
    {
        return false;
    }
    query.events.is_empty()
        || query.events.iter().any(|event| {
            record.event == *event
                || (event == "wram" && record.event == "wram-write")
                || (event == "ppu" && matches!(record.event.as_str(), "ppu-read" | "ppu-write"))
        })
}

fn lower_bound_run(
    index: &mut (impl Read + Seek),
    records_offset: u64,
    record_count: u64,
    target: u32,
) -> Result<u64, String> {
    let mut low = 0_u64;
    let mut high = record_count;
    while low < high {
        let middle = low + (high - low) / 2;
        let offset = records_offset
            .checked_add(
                middle
                    .checked_mul(u64::from(RECORD_BYTES))
                    .ok_or_else(|| "trace index seek overflow".to_string())?,
            )
            .ok_or_else(|| "trace index seek overflow".to_string())?;
        index
            .seek(SeekFrom::Start(offset))
            .map_err(|error| format!("cannot seek trace index: {error}"))?;
        let record = read_record(index)?;
        if record.run < target {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    Ok(low)
}

pub fn query_trace_index(
    index_path: &Path,
    query: &TraceQuery,
    output: &mut impl Write,
) -> Result<(TraceIndexHeader, usize), String> {
    let index_file = File::open(index_path)
        .map_err(|error| format!("cannot open {}: {error}", index_path.display()))?;
    let index_bytes = index_file
        .metadata()
        .map_err(|error| format!("cannot inspect {}: {error}", index_path.display()))?
        .len();
    let mut index = BufReader::new(index_file);
    let (header, header_bytes) = read_index_header(&mut index)?;
    let records_bytes = header
        .records
        .checked_mul(u64::from(RECORD_BYTES))
        .ok_or_else(|| "trace index length overflow".to_string())?;
    let expected_index_bytes = header_bytes
        .checked_add(records_bytes)
        .ok_or_else(|| "trace index length overflow".to_string())?;
    if index_bytes != expected_index_bytes {
        return Err(format!(
            "trace index length mismatch: expected {expected_index_bytes}, found {index_bytes}"
        ));
    }
    let source_path = PathBuf::from(&header.source);
    let metadata = fs::metadata(&source_path).map_err(|error| {
        format!(
            "cannot inspect indexed trace {}: {error}",
            source_path.display()
        )
    })?;
    if metadata.len() != header.source_bytes || sha256_file(&source_path)? != header.source_sha256 {
        return Err(format!(
            "indexed trace changed after indexing: {}",
            source_path.display()
        ));
    }
    let manifest_path = PathBuf::from(&header.manifest);
    if sha256_file(&manifest_path)? != header.manifest_sha256 {
        return Err(format!(
            "indexed manifest changed after indexing: {}",
            manifest_path.display()
        ));
    }
    let mut source = File::open(&source_path).map_err(|error| {
        format!(
            "cannot open indexed trace {}: {error}",
            source_path.display()
        )
    })?;
    let host_run = match query.host_frame {
        Some(host_frame) => host_frame.checked_sub(header.comparison_start_frame),
        None => None,
    };
    if query.host_frame.is_some() && host_run.is_none() {
        return Ok((header, 0));
    }
    if query.run.is_some() && host_run.is_some() && query.run != host_run {
        return Ok((header, 0));
    }
    let target_run = query.run.or(host_run);
    let first_record = match target_run {
        Some(run) => lower_bound_run(&mut index, header_bytes, header.records, run)?,
        None => 0,
    };
    let first_offset = header_bytes
        .checked_add(
            first_record
                .checked_mul(u64::from(RECORD_BYTES))
                .ok_or_else(|| "trace index seek overflow".to_string())?,
        )
        .ok_or_else(|| "trace index seek overflow".to_string())?;
    index
        .seek(SeekFrom::Start(first_offset))
        .map_err(|error| format!("cannot seek trace index: {error}"))?;
    let mut matched = 0_usize;
    for _ in first_record..header.records {
        let record = read_record(&mut index)?;
        if target_run.is_some_and(|run| record.run > run) {
            break;
        }
        if !record_matches(&record, query) {
            continue;
        }
        let source_end = record
            .source_offset
            .checked_add(u64::from(record.source_length));
        if source_end.is_none() || source_end.unwrap() > header.source_bytes {
            return Err("trace index record points outside its source".to_string());
        }
        source
            .seek(SeekFrom::Start(record.source_offset))
            .map_err(|error| format!("cannot seek {}: {error}", source_path.display()))?;
        let mut framed = vec![0_u8; record.source_length as usize];
        source
            .read_exact(&mut framed)
            .map_err(|error| format!("cannot read {}: {error}", source_path.display()))?;
        // Render the exact source record as its canonical JSON object so the
        // query output stays line-oriented for callers.
        let decoded = crate::trace_format::TraceRecord::parse(framed.get(2..).unwrap_or(&[]))
            .map_err(|error| {
                format!("corrupt trace record in {}: {error}", source_path.display())
            })?;
        let mut line = serde_json::to_vec(&decoded.to_json())
            .map_err(|error| format!("cannot encode query output: {error}"))?;
        line.push(b'\n');
        output
            .write_all(&line)
            .map_err(|error| format!("cannot write query output: {error}"))?;
        matched += 1;
        if query.limit.is_some_and(|limit| matched >= limit) {
            break;
        }
    }
    Ok((header, matched))
}

pub fn parse_pc(text: &str) -> Result<u32, String> {
    let normalized = text
        .trim()
        .trim_start_matches('$')
        .trim_start_matches("0x")
        .replace([':', '_'], "");
    let value =
        u32::from_str_radix(&normalized, 16).map_err(|_| format!("invalid 24-bit PC: {text}"))?;
    if value > 0xff_ffff {
        return Err(format!("PC is outside the 24-bit address space: {text}"));
    }
    Ok(value)
}

pub fn parse_wram_range(text: &str) -> Result<(u32, u32), String> {
    let mut parts = text.splitn(2, '-');
    let parse = |value: &str| {
        u32::from_str_radix(value.trim().trim_start_matches("0x"), 16)
            .map_err(|_| format!("invalid WRAM address or range: {text}"))
    };
    let first = parse(parts.next().unwrap_or_default())?;
    let last = parts.next().map(parse).transpose()?.unwrap_or(first);
    if first > last || last > 0x1ffff {
        return Err(format!("invalid WRAM address or range: {text}"));
    }
    Ok((first, last))
}

fn stable_hash(value: &Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("cannot encode cache identity: {error}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn safe_artifact_name(name: &str) -> bool {
    let path = Path::new(name);
    !path.is_absolute()
        && path.components().count() != 0
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

pub fn verify_oracle_cache_entry(cache: &Path) -> Result<CacheInventory, String> {
    let metadata = fs::symlink_metadata(cache)
        .map_err(|error| format!("cannot inspect oracle cache {}: {error}", cache.display()))?;
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "oracle cache entry is not a directory: {}",
            cache.display()
        ));
    }
    let manifest_path = cache.join("cache-manifest.json");
    let manifest = load_json(&manifest_path)
        .map_err(|error| format!("incomplete oracle cache entry {}: {error}", cache.display()))?;
    if manifest.get("schema").and_then(Value::as_u64) != Some(CACHE_SCHEMA)
        || manifest.get("kind").and_then(Value::as_str) != Some(CACHE_KIND)
    {
        return Err(format!(
            "unsupported oracle cache manifest: {}",
            manifest_path.display()
        ));
    }
    let directory_key = cache
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "oracle cache has no UTF-8 directory key: {}",
                cache.display()
            )
        })?;
    if manifest.get("cache_key").and_then(Value::as_str) != Some(directory_key) {
        return Err(format!(
            "oracle cache directory/key mismatch: {}",
            cache.display()
        ));
    }
    let identity = manifest.get("cache_identity").ok_or_else(|| {
        format!(
            "oracle cache manifest has no identity: {}",
            manifest_path.display()
        )
    })?;
    if stable_hash(identity)? != directory_key {
        return Err(format!(
            "oracle cache identity hash mismatch: {}",
            cache.display()
        ));
    }
    let artifacts: BTreeMap<String, String> = serde_json::from_value(
        manifest
            .get("artifact_sha256")
            .cloned()
            .ok_or_else(|| format!("invalid artifact inventory: {}", manifest_path.display()))?,
    )
    .map_err(|error| {
        format!(
            "invalid artifact inventory {}: {error}",
            manifest_path.display()
        )
    })?;
    let mut inventory = CacheInventory {
        entries: 1,
        artifacts: 0,
        bytes: 0,
    };
    for (name, expected) in artifacts {
        if !safe_artifact_name(&name) {
            return Err(format!(
                "unsafe artifact name {name:?} in {}",
                manifest_path.display()
            ));
        }
        let path = cache.join(&name);
        let mut current = cache.to_path_buf();
        let components = Path::new(&name).components().collect::<Vec<_>>();
        let mut metadata = None;
        for (index, component) in components.iter().enumerate() {
            let Component::Normal(component) = component else {
                unreachable!("safe_artifact_name accepted only normal components")
            };
            current.push(component);
            let current_metadata = fs::symlink_metadata(&current)
                .map_err(|_| format!("immutable oracle cache is corrupt: {}", current.display()))?;
            let final_component = index + 1 == components.len();
            if current_metadata.file_type().is_symlink()
                || (final_component && !current_metadata.file_type().is_file())
                || (!final_component && !current_metadata.file_type().is_dir())
            {
                return Err(format!(
                    "immutable oracle cache is corrupt: {}",
                    current.display()
                ));
            }
            metadata = Some(current_metadata);
        }
        let metadata = metadata.expect("safe artifact path has at least one component");
        if sha256_file(&path)? != expected {
            return Err(format!(
                "immutable oracle cache is corrupt: {}",
                path.display()
            ));
        }
        inventory.artifacts += 1;
        inventory.bytes += metadata.len();
    }
    Ok(inventory)
}

pub fn verify_oracle_cache_root(root: &Path) -> Result<CacheInventory, String> {
    if !root.exists() {
        return Ok(CacheInventory {
            entries: 0,
            artifacts: 0,
            bytes: 0,
        });
    }
    if !root.is_dir() {
        return Err(format!(
            "oracle cache root is not a directory: {}",
            root.display()
        ));
    }
    let mut directories = fs::read_dir(root)
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?;
    directories.sort_by_key(|entry| entry.file_name());
    let mut inventory = CacheInventory {
        entries: 0,
        artifacts: 0,
        bytes: 0,
    };
    for entry in directories {
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot inspect {}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "oracle cache root contains a symlinked entry: {}",
                entry.path().display()
            ));
        }
        if !file_type.is_dir() {
            continue;
        }
        let entry_inventory = verify_oracle_cache_entry(&entry.path())?;
        inventory.entries += entry_inventory.entries;
        inventory.artifacts += entry_inventory.artifacts;
        inventory.bytes += entry_inventory.bytes;
    }
    Ok(inventory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_rgba_rgb_updates_match_the_canonical_per_pixel_stream() {
        let rows = [
            [1u8, 2, 3, 4, 5, 6, 7, 8],
            [9u8, 10, 11, 12, 13, 14, 15, 16],
        ];
        let mut canonical = Sha256Digest::new();
        for row in &rows {
            for pixel in row.chunks_exact(4) {
                canonical.update(&pixel[..3]);
            }
        }
        let mut packed = Sha256Digest::new();
        packed.update_rgb_from_rgba(rows.iter().map(<[u8; 8]>::as_slice));
        assert_eq!(packed.finish(), canonical.finish());
    }

    #[test]
    fn packed_i16_updates_match_the_canonical_little_endian_stream() {
        let samples = [i16::MIN, -1, 0, 1, i16::MAX];
        let mut canonical = Sha256Digest::new();
        for sample in samples {
            canonical.update(&sample.to_le_bytes());
        }
        let mut packed = Sha256Digest::new();
        packed.update_i16_le(&samples);
        assert_eq!(packed.finish(), canonical.finish());
    }

    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "zparity-evidence-{}-{nonce}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Write a binary trace fixture from canonical JSON records.
    fn write_binary_trace(path: &Path, lines: &[&str]) {
        let mut bytes = crate::trace_format::MAGIC.to_vec();
        for line in lines {
            let value: serde_json::Value = serde_json::from_str(line).unwrap();
            bytes.extend(
                crate::trace_format::TraceRecord::from_json(&value)
                    .unwrap()
                    .encode_framed(),
            );
        }
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn trace_index_maps_resumed_runs_and_queries_original_records() {
        let temporary = TestDirectory::new();
        let trace = temporary.0.join("trace.bin");
        let manifest = temporary.0.join("manifest.json");
        let index = temporary.0.join("trace.zpti");
        write_binary_trace(
            &trace,
            &[
                "{\"event\":\"frame\",\"run\":74,\"frame\":75,\"pc\":32849}",
                "{\"event\":\"wram-write\",\"run\":85,\"frame\":87,\"pc\":1960771,\"address\":3328,\"value\":9}",
            ],
        );
        fs::write(&manifest, "{\"timing\":{\"start_frame\":31200}}\n").unwrap();

        let header = build_trace_index(&trace, &manifest, &index).unwrap();
        assert_eq!(header.records, 2);
        assert_eq!(header.comparison_start_frame, 31200);
        let mut output = Vec::new();
        let query = TraceQuery {
            host_frame: Some(31285),
            pc: Some(parse_pc("9d:eb43").unwrap()),
            wram: Some(parse_wram_range("0d00-0fff").unwrap()),
            events: vec!["wram".to_string()],
            ..TraceQuery::default()
        };
        let (_, matched) = query_trace_index(&index, &query, &mut output).unwrap();
        assert_eq!(matched, 1);
        let rendered: serde_json::Value =
            serde_json::from_str(String::from_utf8(output).unwrap().trim()).unwrap();
        assert_eq!(rendered["event"], "wram-write");
        assert_eq!(rendered["run"], 85);
        assert_eq!(rendered["frame"], 87);
        assert_eq!(rendered["pc"], 1960771);
        assert_eq!(rendered["address"], 3328);
        assert_eq!(rendered["value"], 9);
    }

    #[test]
    fn trace_index_accepts_content_addressed_cache_coordinates() {
        let manifest = serde_json::json!({
            "cache_identity": {"start_frame": 31200}
        });
        assert_eq!(
            manifest_start_frame(&manifest, Path::new("cache-manifest.json")).unwrap(),
            31200
        );
    }

    #[test]
    fn trace_index_rejects_nonmonotonic_run_coordinates() {
        let temporary = TestDirectory::new();
        let trace = temporary.0.join("trace.bin");
        let manifest = temporary.0.join("manifest.json");
        let index = temporary.0.join("trace.zpti");
        write_binary_trace(
            &trace,
            &[
                "{\"event\":\"frame\",\"run\":2}",
                "{\"event\":\"frame\",\"run\":1}",
            ],
        );
        fs::write(&manifest, "{}\n").unwrap();
        assert!(build_trace_index(&trace, &manifest, &index)
            .unwrap_err()
            .contains("retro_run regressed"));
    }

    #[test]
    fn trace_query_rejects_a_changed_source() {
        let temporary = TestDirectory::new();
        let trace = temporary.0.join("trace.bin");
        let manifest = temporary.0.join("manifest.json");
        let index = temporary.0.join("trace.zpti");
        write_binary_trace(&trace, &["{\"event\":\"frame\",\"run\":1}"]);
        fs::write(&manifest, "{}\n").unwrap();
        build_trace_index(&trace, &manifest, &index).unwrap();
        write_binary_trace(&trace, &["{\"event\":\"frame\",\"run\":2}"]);
        let error = query_trace_index(&index, &TraceQuery::default(), &mut Vec::new()).unwrap_err();
        assert!(error.contains("changed after indexing"));
    }

    #[test]
    fn trace_query_rejects_a_changed_coordinate_manifest() {
        let temporary = TestDirectory::new();
        let trace = temporary.0.join("trace.bin");
        let manifest = temporary.0.join("manifest.json");
        let index = temporary.0.join("trace.zpti");
        write_binary_trace(&trace, &["{\"event\":\"frame\",\"run\":1}"]);
        fs::write(&manifest, "{\"timing\":{\"start_frame\":10}}\n").unwrap();
        build_trace_index(&trace, &manifest, &index).unwrap();
        fs::write(&manifest, "{\"timing\":{\"start_frame\":11}}\n").unwrap();
        let error = query_trace_index(&index, &TraceQuery::default(), &mut Vec::new()).unwrap_err();
        assert!(error.contains("manifest changed after indexing"));
    }

    #[test]
    fn cache_verifier_checks_identity_and_artifact_bytes() {
        let temporary = TestDirectory::new();
        let identity = serde_json::json!({"schema": 1, "start_frame": 31200});
        let key = stable_hash(&identity).unwrap();
        let cache = temporary.0.join(&key);
        fs::create_dir(&cache).unwrap();
        let artifact = cache.join("oracle.state");
        fs::write(&artifact, b"oracle").unwrap();
        let manifest = serde_json::json!({
            "schema": 1,
            "kind": CACHE_KIND,
            "cache_key": key,
            "cache_identity": identity,
            "artifact_sha256": {"oracle.state": sha256_file(&artifact).unwrap()}
        });
        fs::write(
            cache.join("cache-manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(
            verify_oracle_cache_root(&temporary.0).unwrap(),
            CacheInventory {
                entries: 1,
                artifacts: 1,
                bytes: 6
            }
        );
        fs::write(&artifact, b"tampered").unwrap();
        assert!(verify_oracle_cache_root(&temporary.0)
            .unwrap_err()
            .contains("corrupt"));
    }

    #[test]
    fn cache_verifier_accepts_nested_files_but_rejects_path_escape() {
        assert!(safe_artifact_name(
            "oracle-checkpoints/frame-00005000/oracle.state"
        ));
        assert!(!safe_artifact_name("../oracle.state"));
        assert!(!safe_artifact_name("/tmp/oracle.state"));

        let temporary = TestDirectory::new();
        let identity = serde_json::json!({"schema": 1, "nested": true});
        let key = stable_hash(&identity).unwrap();
        let cache = temporary.0.join(&key);
        let checkpoint = cache.join("oracle-checkpoints/frame-00005000");
        fs::create_dir_all(&checkpoint).unwrap();
        let artifact = checkpoint.join("oracle.state");
        fs::write(&artifact, b"oracle").unwrap();
        let name = "oracle-checkpoints/frame-00005000/oracle.state";
        let manifest = serde_json::json!({
            "schema": 1,
            "kind": CACHE_KIND,
            "cache_key": key,
            "cache_identity": identity,
            "artifact_sha256": {name: sha256_file(&artifact).unwrap()}
        });
        fs::write(
            cache.join("cache-manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        assert_eq!(verify_oracle_cache_entry(&cache).unwrap().artifacts, 1);
    }
}
