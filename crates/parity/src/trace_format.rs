//! The pinned Snes9x trace core's binary record format (`Z3TRACE1`).
//!
//! The instrumented core (`external/snes9x-libretro/patches/zelda3-trace-binary-format.patch`)
//! writes one file: an 8-byte magic followed by records. Every record is a
//! little-endian `u16` length (excluding the length field) followed by a
//! fixed 106-byte machine-state header and a tag/length/value tail whose
//! tags are event specific. This module is the single Rust decoder; the
//! Python mirror lives in `scripts/snes9x_trace_format.py`. Both render the
//! same canonical JSON object per record, which is the shape the earlier
//! JSON Lines trace used, so downstream tooling keeps its field names.

use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use serde_json::{json, Map, Value};

/// File magic; also encodes the format version.
pub const MAGIC: &[u8; 8] = b"Z3TRACE1";
/// Fixed machine-state header length in bytes.
pub const HEADER_LEN: usize = 106;
/// Length of the `nmi_ppu_register_operands` capture.
pub const PPU_OPERAND_COUNT: usize = 31;

pub const KIND_FRAME: u8 = 1;
pub const KIND_VIDEO: u8 = 2;
pub const KIND_NMI: u8 = 3;
pub const KIND_NMI_RESUME: u8 = 4;
pub const KIND_PC: u8 = 5;
pub const KIND_DMA: u8 = 6;
pub const KIND_HDMA_START: u8 = 7;
pub const KIND_HDMA_END: u8 = 8;
pub const KIND_RNG_PPU_READ: u8 = 9;
pub const KIND_PPU_READ: u8 = 10;
pub const KIND_PPU_WRITE: u8 = 11;
pub const KIND_RNG_WRITE: u8 = 12;
pub const KIND_WRAM_WRITE: u8 = 13;
pub const KIND_PIXEL_WRITE: u8 = 14;

pub const STAGE_NONE: u8 = 0;
pub const STAGE_ENTRY: u8 = 1;
pub const STAGE_RETURN: u8 = 2;
pub const STAGE_PRESENTED: u8 = 3;

pub const TAG_ADDRESS: u8 = 1;
pub const TAG_VALUE: u8 = 2;
pub const TAG_H_LATCHED: u8 = 3;
pub const TAG_CHANNEL: u8 = 4;
pub const TAG_SOURCE: u8 = 5;
pub const TAG_B_ADDRESS: u8 = 6;
pub const TAG_BYTES: u8 = 7;
pub const TAG_MODE: u8 = 8;
pub const TAG_FIXED: u8 = 9;
pub const TAG_DECREMENT: u8 = 10;
pub const TAG_VRAM_ADDRESS: u8 = 11;
pub const TAG_CHANNELS: u8 = 12;
pub const TAG_CHANNEL_STATE: u8 = 13;
pub const TAG_PIX: u8 = 20;
pub const TAG_Z1: u8 = 21;
pub const TAG_Z2: u8 = 22;
pub const TAG_TILE: u8 = 23;
pub const TAG_TILE_NUMBER: u8 = 24;
pub const TAG_TILE_ADDRESS: u8 = 25;
pub const TAG_TILE_CACHE_VALID: u8 = 26;
pub const TAG_TILE_CACHE_HASH: u8 = 27;
pub const TAG_OBJ_TILE: u8 = 28;
pub const TAG_OBJ_LINE: u8 = 29;
pub const TAG_OBJ_X: u8 = 30;
pub const TAG_OBJ_Y: u8 = 31;
pub const TAG_OBJ_CACHE: u8 = 32;
pub const TAG_OBJ_TILE_NUMBER: u8 = 33;
pub const TAG_OBJ_CACHE_VALID: u8 = 34;
pub const TAG_OBJ_CACHE_HASH: u8 = 35;

/// `(kind, name)` pairs in canonical order.
pub const KIND_NAMES: [(u8, &str); 14] = [
    (KIND_FRAME, "frame"),
    (KIND_VIDEO, "video"),
    (KIND_NMI, "nmi"),
    (KIND_NMI_RESUME, "nmi-resume"),
    (KIND_PC, "pc"),
    (KIND_DMA, "dma"),
    (KIND_HDMA_START, "hdma-start"),
    (KIND_HDMA_END, "hdma-end"),
    (KIND_RNG_PPU_READ, "rng-ppu-read"),
    (KIND_PPU_READ, "ppu-read"),
    (KIND_PPU_WRITE, "ppu-write"),
    (KIND_RNG_WRITE, "rng-write"),
    (KIND_WRAM_WRITE, "wram-write"),
    (KIND_PIXEL_WRITE, "pixel-write"),
];

/// Unsigned (`false`) or signed (`true`) 32-bit scalar tags with their JSON key.
const SCALAR_TAGS: [(u8, &str, bool); 28] = [
    (TAG_ADDRESS, "address", false),
    (TAG_VALUE, "value", false),
    (TAG_H_LATCHED, "h_latched", false),
    (TAG_CHANNEL, "channel", false),
    (TAG_SOURCE, "source", false),
    (TAG_B_ADDRESS, "b_address", false),
    (TAG_BYTES, "bytes", false),
    (TAG_MODE, "mode", false),
    (TAG_FIXED, "fixed", false),
    (TAG_DECREMENT, "decrement", false),
    (TAG_VRAM_ADDRESS, "vram_address", false),
    (TAG_CHANNELS, "channels", false),
    (TAG_PIX, "pix", true),
    (TAG_Z1, "z1", true),
    (TAG_Z2, "z2", true),
    (TAG_TILE, "tile", true),
    (TAG_TILE_NUMBER, "tile_number", true),
    (TAG_TILE_ADDRESS, "tile_address", true),
    (TAG_TILE_CACHE_VALID, "tile_cache_valid", true),
    (TAG_TILE_CACHE_HASH, "tile_cache_hash", false),
    (TAG_OBJ_TILE, "obj_tile", true),
    (TAG_OBJ_LINE, "obj_line", true),
    (TAG_OBJ_X, "obj_x", true),
    (TAG_OBJ_Y, "obj_y", true),
    (TAG_OBJ_CACHE, "obj_cache", true),
    (TAG_OBJ_TILE_NUMBER, "obj_tile_number", false),
    (TAG_OBJ_CACHE_VALID, "obj_cache_valid", true),
    (TAG_OBJ_CACHE_HASH, "obj_cache_hash", false),
];

pub fn kind_name(kind: u8) -> Option<&'static str> {
    KIND_NAMES
        .iter()
        .find(|(id, _)| *id == kind)
        .map(|(_, name)| *name)
}

pub fn kind_from_name(name: &str) -> Option<u8> {
    KIND_NAMES
        .iter()
        .find(|(_, candidate)| *candidate == name)
        .map(|(id, _)| *id)
}

pub fn stage_name(stage: u8) -> Option<&'static str> {
    match stage {
        STAGE_ENTRY => Some("entry"),
        STAGE_RETURN => Some("return"),
        STAGE_PRESENTED => Some("presented"),
        _ => None,
    }
}

/// One tag/length/value tail entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tlv {
    pub tag: u8,
    pub payload: Vec<u8>,
}

/// One decoded HDMA channel from a `TAG_CHANNEL_STATE` payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HdmaChannelState {
    pub channel: u8,
    pub source: u32,
    pub table_address: u16,
    pub indirect: u8,
    pub line_count: u8,
    pub repeat: u8,
    pub do_transfer: u8,
    pub b_address: u8,
    pub mode: u8,
    pub data: Vec<u8>,
}

/// One trace record: the fixed machine-state header plus its tail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceRecord {
    pub kind: u8,
    pub stage: u8,
    pub run: u64,
    pub frame: u32,
    pub v: i16,
    pub cycles: i32,
    pub pc: u32,
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub carry: u8,
    pub p: u8,
    pub main: u8,
    pub sub: u8,
    pub subsub: u8,
    pub frame_counter: u8,
    pub room: u16,
    pub lights_out: u8,
    pub palette_countdown: u8,
    pub palette_direction: u8,
    pub link_y: u16,
    pub link_x: u16,
    pub bg2_v: u16,
    pub bg2_h: u16,
    pub mosaic_target: u8,
    pub spotlight_radius: u16,
    pub spotlight_state: u16,
    pub spotlight_var4_low: u8,
    pub spotlight_lower_cursor: u16,
    pub rng_seed: u8,
    pub nmi_latch: u8,
    pub nmi_disable: u8,
    pub nmi_pending: u8,
    pub joypad_high: u8,
    pub joypad_low: u8,
    pub joypad_high_filtered: u8,
    pub joypad_low_filtered: u8,
    pub nmi_ppu_register_operands: [u8; PPU_OPERAND_COUNT],
    pub return_address: u32,
    pub stack: [u8; 4],
    pub tail: Vec<Tlv>,
}

impl Default for TraceRecord {
    fn default() -> Self {
        Self {
            kind: 0,
            stage: 0,
            run: 0,
            frame: 0,
            v: 0,
            cycles: 0,
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            carry: 0,
            p: 0,
            main: 0,
            sub: 0,
            subsub: 0,
            frame_counter: 0,
            room: 0,
            lights_out: 0,
            palette_countdown: 0,
            palette_direction: 0,
            link_y: 0,
            link_x: 0,
            bg2_v: 0,
            bg2_h: 0,
            mosaic_target: 0,
            spotlight_radius: 0,
            spotlight_state: 0,
            spotlight_var4_low: 0,
            spotlight_lower_cursor: 0,
            rng_seed: 0,
            nmi_latch: 0,
            nmi_disable: 0,
            nmi_pending: 0,
            joypad_high: 0,
            joypad_low: 0,
            joypad_high_filtered: 0,
            joypad_low_filtered: 0,
            nmi_ppu_register_operands: [0; PPU_OPERAND_COUNT],
            return_address: 0,
            stack: [0; 4],
            tail: Vec::new(),
        }
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self.position + count;
        if end > self.bytes.len() {
            return Err(format!(
                "trace record truncated at byte {} (needs {count} more)",
                self.position
            ));
        }
        let slice = &self.bytes[self.position..end];
        self.position = end;
        Ok(slice)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }
    fn u32(&mut self) -> Result<u32, String> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    fn u64(&mut self) -> Result<u64, String> {
        let bytes = self.take(8)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(array))
    }
}

fn scalar_from_payload(payload: &[u8], signed: bool) -> Option<i64> {
    let value = match payload.len() {
        1 => u32::from(payload[0]),
        2 => u32::from(u16::from_le_bytes([payload[0], payload[1]])),
        4 => u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]),
        _ => return None,
    };
    Some(if signed && payload.len() == 4 {
        i64::from(value as i32)
    } else {
        i64::from(value)
    })
}

impl TraceRecord {
    /// Decode one record body (everything after the `u16` length).
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_LEN {
            return Err(format!(
                "trace record body is {} bytes; the fixed header needs {HEADER_LEN}",
                bytes.len()
            ));
        }
        let mut cursor = Cursor { bytes, position: 0 };
        let mut record = TraceRecord {
            kind: cursor.u8()?,
            stage: cursor.u8()?,
            run: cursor.u64()?,
            frame: cursor.u32()?,
            v: cursor.u16()? as i16,
            cycles: cursor.u32()? as i32,
            pc: cursor.u32()?,
            a: cursor.u16()?,
            x: cursor.u16()?,
            y: cursor.u16()?,
            s: cursor.u16()?,
            carry: cursor.u8()?,
            p: cursor.u8()?,
            main: cursor.u8()?,
            sub: cursor.u8()?,
            subsub: cursor.u8()?,
            frame_counter: cursor.u8()?,
            room: cursor.u16()?,
            lights_out: cursor.u8()?,
            palette_countdown: cursor.u8()?,
            palette_direction: cursor.u8()?,
            link_y: cursor.u16()?,
            link_x: cursor.u16()?,
            bg2_v: cursor.u16()?,
            bg2_h: cursor.u16()?,
            mosaic_target: cursor.u8()?,
            spotlight_radius: cursor.u16()?,
            spotlight_state: cursor.u16()?,
            spotlight_var4_low: cursor.u8()?,
            spotlight_lower_cursor: cursor.u16()?,
            rng_seed: cursor.u8()?,
            nmi_latch: cursor.u8()?,
            nmi_disable: cursor.u8()?,
            nmi_pending: cursor.u8()?,
            joypad_high: cursor.u8()?,
            joypad_low: cursor.u8()?,
            joypad_high_filtered: cursor.u8()?,
            joypad_low_filtered: cursor.u8()?,
            ..TraceRecord::default()
        };
        record
            .nmi_ppu_register_operands
            .copy_from_slice(cursor.take(PPU_OPERAND_COUNT)?);
        record.return_address = cursor.u32()?;
        record.stack.copy_from_slice(cursor.take(4)?);
        debug_assert_eq!(cursor.position, HEADER_LEN);
        while cursor.position < bytes.len() {
            let tag = cursor.u8()?;
            let length = usize::from(cursor.u8()?);
            let payload = cursor.take(length)?.to_vec();
            record.tail.push(Tlv { tag, payload });
        }
        if kind_name(record.kind).is_none() {
            return Err(format!("unknown trace record kind {}", record.kind));
        }
        Ok(record)
    }

    /// Encode the record body (everything after the `u16` length).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN + 16);
        out.push(self.kind);
        out.push(self.stage);
        out.extend_from_slice(&self.run.to_le_bytes());
        out.extend_from_slice(&self.frame.to_le_bytes());
        out.extend_from_slice(&(self.v as u16).to_le_bytes());
        out.extend_from_slice(&(self.cycles as u32).to_le_bytes());
        out.extend_from_slice(&self.pc.to_le_bytes());
        for value in [self.a, self.x, self.y, self.s] {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.extend_from_slice(&[
            self.carry,
            self.p,
            self.main,
            self.sub,
            self.subsub,
            self.frame_counter,
        ]);
        out.extend_from_slice(&self.room.to_le_bytes());
        out.extend_from_slice(&[
            self.lights_out,
            self.palette_countdown,
            self.palette_direction,
        ]);
        for value in [self.link_y, self.link_x, self.bg2_v, self.bg2_h] {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.push(self.mosaic_target);
        out.extend_from_slice(&self.spotlight_radius.to_le_bytes());
        out.extend_from_slice(&self.spotlight_state.to_le_bytes());
        out.push(self.spotlight_var4_low);
        out.extend_from_slice(&self.spotlight_lower_cursor.to_le_bytes());
        out.extend_from_slice(&[
            self.rng_seed,
            self.nmi_latch,
            self.nmi_disable,
            self.nmi_pending,
            self.joypad_high,
            self.joypad_low,
            self.joypad_high_filtered,
            self.joypad_low_filtered,
        ]);
        out.extend_from_slice(&self.nmi_ppu_register_operands);
        out.extend_from_slice(&self.return_address.to_le_bytes());
        out.extend_from_slice(&self.stack);
        debug_assert_eq!(out.len(), HEADER_LEN);
        for tlv in &self.tail {
            out.push(tlv.tag);
            out.push(tlv.payload.len() as u8);
            out.extend_from_slice(&tlv.payload);
        }
        out
    }

    /// The framed bytes as the core writes them: `u16` length then the body.
    pub fn encode_framed(&self) -> Vec<u8> {
        let body = self.encode();
        let mut out = Vec::with_capacity(body.len() + 2);
        out.extend_from_slice(&(body.len() as u16).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    pub fn event(&self) -> &'static str {
        kind_name(self.kind).unwrap_or("unknown")
    }

    pub fn stage_name(&self) -> Option<&'static str> {
        stage_name(self.stage)
    }

    pub fn tlv(&self, tag: u8) -> Option<&Tlv> {
        self.tail.iter().find(|tlv| tlv.tag == tag)
    }

    /// An unsigned scalar tail field (`address`, `value`, `channel`, ...).
    pub fn scalar(&self, tag: u8) -> Option<u32> {
        self.tlv(tag)
            .and_then(|tlv| scalar_from_payload(&tlv.payload, false))
            .map(|value| value as u32)
    }

    pub fn address(&self) -> Option<u32> {
        self.scalar(TAG_ADDRESS)
    }

    pub fn value(&self) -> Option<u32> {
        self.scalar(TAG_VALUE)
    }

    pub fn hdma_channel_states(&self) -> Vec<HdmaChannelState> {
        self.tail
            .iter()
            .filter(|tlv| tlv.tag == TAG_CHANNEL_STATE)
            .filter_map(|tlv| {
                let p = &tlv.payload;
                if p.len() < 14 {
                    return None;
                }
                let data_len = usize::from(p[13]);
                Some(HdmaChannelState {
                    channel: p[0],
                    source: u32::from_le_bytes([p[1], p[2], p[3], p[4]]),
                    table_address: u16::from_le_bytes([p[5], p[6]]),
                    indirect: p[7],
                    line_count: p[8],
                    repeat: p[9],
                    do_transfer: p[10],
                    b_address: p[11],
                    mode: p[12],
                    data: p[14..(14 + data_len).min(p.len())].to_vec(),
                })
            })
            .collect()
    }

    /// The canonical JSON object for this record (the former JSON Lines shape).
    pub fn to_json(&self) -> Value {
        let mut object = Map::new();
        object.insert("event".into(), json!(self.event()));
        object.insert("run".into(), json!(self.run));
        object.insert("frame".into(), json!(self.frame));
        object.insert("v".into(), json!(self.v));
        object.insert("cycles".into(), json!(self.cycles));
        object.insert("pc".into(), json!(self.pc));
        object.insert("a".into(), json!(self.a));
        object.insert("x".into(), json!(self.x));
        object.insert("y".into(), json!(self.y));
        object.insert("s".into(), json!(self.s));
        object.insert("carry".into(), json!(self.carry));
        object.insert("p".into(), json!(self.p));
        object.insert("main".into(), json!(self.main));
        object.insert("sub".into(), json!(self.sub));
        object.insert("subsub".into(), json!(self.subsub));
        object.insert("frame_counter".into(), json!(self.frame_counter));
        object.insert("room".into(), json!(self.room));
        object.insert("lights_out".into(), json!(self.lights_out));
        object.insert("palette_countdown".into(), json!(self.palette_countdown));
        object.insert("palette_direction".into(), json!(self.palette_direction));
        object.insert("link_y".into(), json!(self.link_y));
        object.insert("link_x".into(), json!(self.link_x));
        object.insert("bg2_v".into(), json!(self.bg2_v));
        object.insert("bg2_h".into(), json!(self.bg2_h));
        object.insert("mosaic_target".into(), json!(self.mosaic_target));
        object.insert("spotlight_radius".into(), json!(self.spotlight_radius));
        object.insert("spotlight_state".into(), json!(self.spotlight_state));
        object.insert("spotlight_var4_low".into(), json!(self.spotlight_var4_low));
        object.insert(
            "spotlight_lower_cursor".into(),
            json!(self.spotlight_lower_cursor),
        );
        object.insert("rng_seed".into(), json!(self.rng_seed));
        object.insert("nmi_latch".into(), json!(self.nmi_latch));
        object.insert("nmi_disable".into(), json!(self.nmi_disable));
        object.insert("nmi_pending".into(), json!(self.nmi_pending));
        object.insert("joypad_high".into(), json!(self.joypad_high));
        object.insert("joypad_low".into(), json!(self.joypad_low));
        object.insert(
            "joypad_high_filtered".into(),
            json!(self.joypad_high_filtered),
        );
        object.insert(
            "joypad_low_filtered".into(),
            json!(self.joypad_low_filtered),
        );
        object.insert(
            "nmi_ppu_register_operands".into(),
            json!(self.nmi_ppu_register_operands.to_vec()),
        );
        object.insert("return_address".into(), json!(self.return_address));
        object.insert("stack1".into(), json!(self.stack[0]));
        object.insert("stack2".into(), json!(self.stack[1]));
        object.insert("stack3".into(), json!(self.stack[2]));
        object.insert("stack4".into(), json!(self.stack[3]));
        if let Some(stage) = self.stage_name() {
            object.insert("stage".into(), json!(stage));
        }
        let mut channel_state = Vec::new();
        for tlv in &self.tail {
            if tlv.tag == TAG_CHANNEL_STATE {
                continue;
            }
            if let Some((_, key, signed)) = SCALAR_TAGS.iter().find(|(tag, _, _)| *tag == tlv.tag) {
                if let Some(value) = scalar_from_payload(&tlv.payload, *signed) {
                    object.insert((*key).into(), json!(value));
                    continue;
                }
            }
            object.insert(format!("tag_{}", tlv.tag), json!(tlv.payload));
        }
        for state in self.hdma_channel_states() {
            channel_state.push(json!({
                "channel": state.channel,
                "source": state.source,
                "table_address": state.table_address,
                "indirect": state.indirect,
                "line_count": state.line_count,
                "repeat": state.repeat,
                "do_transfer": state.do_transfer,
                "b_address": state.b_address,
                "mode": state.mode,
                "data": state.data,
            }));
        }
        if matches!(self.kind, KIND_HDMA_START | KIND_HDMA_END) {
            object.insert("channel_state".into(), Value::Array(channel_state));
        }
        Value::Object(object)
    }

    /// Build a record from the canonical JSON object (fixtures, tests, and
    /// the `trace-encode` converter). Header fields absent from the object
    /// default to zero; unknown keys are ignored.
    pub fn from_json(value: &Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or("trace record JSON must be an object")?;
        let event = object
            .get("event")
            .and_then(Value::as_str)
            .ok_or("trace record JSON lacks an event name")?;
        let kind = kind_from_name(event).ok_or_else(|| format!("unknown trace event {event:?}"))?;
        let u = |key: &str| -> u64 {
            object
                .get(key)
                .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                .unwrap_or(0)
        };
        let mut record = TraceRecord {
            kind,
            stage: match object.get("stage").and_then(Value::as_str) {
                Some("entry") => STAGE_ENTRY,
                Some("return") => STAGE_RETURN,
                Some("presented") => STAGE_PRESENTED,
                _ => STAGE_NONE,
            },
            run: u("run"),
            frame: u("frame") as u32,
            v: u("v") as i16,
            cycles: u("cycles") as i32,
            pc: u("pc") as u32,
            a: u("a") as u16,
            x: u("x") as u16,
            y: u("y") as u16,
            s: u("s") as u16,
            carry: u("carry") as u8,
            p: u("p") as u8,
            main: u("main") as u8,
            sub: u("sub") as u8,
            subsub: u("subsub") as u8,
            frame_counter: u("frame_counter") as u8,
            room: u("room") as u16,
            lights_out: u("lights_out") as u8,
            palette_countdown: u("palette_countdown") as u8,
            palette_direction: u("palette_direction") as u8,
            link_y: u("link_y") as u16,
            link_x: u("link_x") as u16,
            bg2_v: u("bg2_v") as u16,
            bg2_h: u("bg2_h") as u16,
            mosaic_target: u("mosaic_target") as u8,
            spotlight_radius: u("spotlight_radius") as u16,
            spotlight_state: u("spotlight_state") as u16,
            spotlight_var4_low: u("spotlight_var4_low") as u8,
            spotlight_lower_cursor: u("spotlight_lower_cursor") as u16,
            rng_seed: u("rng_seed") as u8,
            nmi_latch: u("nmi_latch") as u8,
            nmi_disable: u("nmi_disable") as u8,
            nmi_pending: u("nmi_pending") as u8,
            joypad_high: u("joypad_high") as u8,
            joypad_low: u("joypad_low") as u8,
            joypad_high_filtered: u("joypad_high_filtered") as u8,
            joypad_low_filtered: u("joypad_low_filtered") as u8,
            return_address: u("return_address") as u32,
            stack: [
                u("stack1") as u8,
                u("stack2") as u8,
                u("stack3") as u8,
                u("stack4") as u8,
            ],
            ..TraceRecord::default()
        };
        if let Some(operands) = object
            .get("nmi_ppu_register_operands")
            .and_then(Value::as_array)
        {
            for (slot, value) in record
                .nmi_ppu_register_operands
                .iter_mut()
                .zip(operands.iter())
            {
                *slot = value.as_u64().unwrap_or(0) as u8;
            }
        }
        for (tag, key, signed) in SCALAR_TAGS {
            let Some(value) = object.get(key) else {
                continue;
            };
            let payload = if signed {
                (value.as_i64().unwrap_or(0) as i32 as u32)
                    .to_le_bytes()
                    .to_vec()
            } else {
                let raw = value.as_u64().unwrap_or(0) as u32;
                match tag {
                    TAG_CHANNEL | TAG_B_ADDRESS | TAG_MODE | TAG_FIXED | TAG_DECREMENT
                    | TAG_CHANNELS => vec![raw as u8],
                    TAG_H_LATCHED | TAG_VRAM_ADDRESS => (raw as u16).to_le_bytes().to_vec(),
                    _ => raw.to_le_bytes().to_vec(),
                }
            };
            record.tail.push(Tlv { tag, payload });
        }
        if let Some(states) = object.get("channel_state").and_then(Value::as_array) {
            for state in states {
                let g = |key: &str| state.get(key).and_then(Value::as_u64).unwrap_or(0);
                let data: Vec<u8> = state
                    .get("data")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .map(|v| v.as_u64().unwrap_or(0) as u8)
                            .collect()
                    })
                    .unwrap_or_default();
                let mut payload = vec![g("channel") as u8];
                payload.extend_from_slice(&(g("source") as u32).to_le_bytes());
                payload.extend_from_slice(&(g("table_address") as u16).to_le_bytes());
                payload.extend_from_slice(&[
                    g("indirect") as u8,
                    g("line_count") as u8,
                    g("repeat") as u8,
                    g("do_transfer") as u8,
                    g("b_address") as u8,
                    g("mode") as u8,
                    data.len() as u8,
                ]);
                payload.extend_from_slice(&data);
                record.tail.push(Tlv {
                    tag: TAG_CHANNEL_STATE,
                    payload,
                });
            }
        }
        Ok(record)
    }
}

/// Reads framed records from any byte source, tracking the absolute byte
/// offset so callers that re-open the file per host can resume exactly.
pub struct TraceReader<R: Read> {
    reader: BufReader<R>,
    offset: u64,
    body: Vec<u8>,
}

impl<R: Read> TraceReader<R> {
    /// Wrap a source already positioned at `offset` bytes into the trace. At
    /// offset zero the file magic is consumed and validated first.
    pub fn new(inner: R, offset: u64) -> io::Result<Self> {
        let mut reader = Self {
            reader: BufReader::with_capacity(1 << 16, inner),
            offset,
            body: Vec::with_capacity(512),
        };
        if offset == 0 {
            reader.consume_magic()?;
        }
        Ok(reader)
    }

    fn consume_magic(&mut self) -> io::Result<()> {
        let mut magic = [0u8; 8];
        match self.reader.read_exact(&mut magic) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error),
        }
        if &magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "not a Z3TRACE1 binary trace (magic {:?}); JSON Lines traces are no longer produced by the pinned core",
                    String::from_utf8_lossy(&magic)
                ),
            ));
        }
        self.offset = 8;
        Ok(())
    }

    /// Absolute byte offset of the next unread record.
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// The next complete record, or `None` at a clean end of data. A record
    /// whose length prefix is present but whose body has not been fully
    /// written yet is also reported as `None` without advancing.
    pub fn next_record(&mut self) -> io::Result<Option<TraceRecord>> {
        if self.offset == 0 {
            self.consume_magic()?;
            if self.offset == 0 {
                return Ok(None);
            }
        }
        let mut length = [0u8; 2];
        let available = self.reader.fill_buf()?;
        if available.is_empty() {
            return Ok(None);
        }
        match self.reader.read_exact(&mut length) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(error) => return Err(error),
        }
        let length = usize::from(u16::from_le_bytes(length));
        self.body.clear();
        self.body.resize(length, 0);
        match self.reader.read_exact(&mut self.body) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!(
                        "trace record at byte {} promised {length} bytes but the file ended",
                        self.offset
                    ),
                ))
            }
            Err(error) => return Err(error),
        }
        let record = TraceRecord::parse(&self.body)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        self.offset += 2 + length as u64;
        Ok(Some(record))
    }
}

/// Open a trace file at `offset` (0 reads the magic).
pub fn open_trace(path: &Path, offset: u64) -> io::Result<TraceReader<std::fs::File>> {
    let mut file = std::fs::File::open(path)?;
    if offset != 0 {
        file.seek(SeekFrom::Start(offset))?;
    }
    TraceReader::new(file, offset)
}

/// Every record of a trace file, in order.
pub fn read_all(path: &Path) -> io::Result<Vec<TraceRecord>> {
    let mut reader = open_trace(path, 0)?;
    let mut records = Vec::new();
    while let Some(record) = reader.next_record()? {
        records.push(record);
    }
    Ok(records)
}

/// Whether a file starts with the binary magic.
pub fn is_binary_trace(path: &Path) -> bool {
    let mut magic = [0u8; 8];
    std::fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut magic))
        .map(|_| &magic == MAGIC)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TraceRecord {
        TraceRecord {
            kind: KIND_WRAM_WRITE,
            run: 1371214,
            frame: 1371215,
            v: 12,
            cycles: 952,
            pc: 0x1d_bd08,
            a: 0xa5,
            x: 2,
            y: 0,
            s: 0x1e4,
            carry: 1,
            p: 0x30,
            main: 7,
            frame_counter: 12,
            room: 0xa4,
            link_y: 5555,
            bg2_v: 5392,
            spotlight_radius: 126,
            spotlight_lower_cursor: 0x1234,
            return_address: 0xdcbc39,
            stack: [0x39, 0xbc, 0xdc, 0xb8],
            tail: vec![
                Tlv {
                    tag: TAG_ADDRESS,
                    payload: 0x0d12u32.to_le_bytes().to_vec(),
                },
                Tlv {
                    tag: TAG_VALUE,
                    payload: 0xa5u32.to_le_bytes().to_vec(),
                },
            ],
            ..TraceRecord::default()
        }
    }

    #[test]
    fn header_is_exactly_106_bytes_and_round_trips() {
        let record = sample();
        let body = record.encode();
        assert_eq!(body.len(), HEADER_LEN + 12);
        assert_eq!(TraceRecord::parse(&body).unwrap(), record);
    }

    #[test]
    fn json_round_trip_keeps_the_former_field_names() {
        let record = sample();
        let json = record.to_json();
        assert_eq!(json["event"], "wram-write");
        assert_eq!(json["address"], 0x0d12);
        assert_eq!(json["value"], 0xa5);
        assert_eq!(json["return_address"], 0xdcbc39);
        assert_eq!(json["stack4"], 0xb8);
        assert!(json.get("stage").is_none());
        assert_eq!(TraceRecord::from_json(&json).unwrap(), record);
    }

    #[test]
    fn framed_stream_reader_skips_magic_and_tracks_offsets() {
        let mut bytes = MAGIC.to_vec();
        let mut first = sample();
        first.kind = KIND_FRAME;
        first.stage = STAGE_RETURN;
        first.tail.clear();
        bytes.extend(first.encode_framed());
        bytes.extend(sample().encode_framed());
        let mut reader = TraceReader::new(std::io::Cursor::new(bytes.clone()), 0).unwrap();
        let a = reader.next_record().unwrap().unwrap();
        assert_eq!(a.event(), "frame");
        assert_eq!(a.stage_name(), Some("return"));
        let after_first = reader.offset();
        assert_eq!(after_first, 8 + 2 + HEADER_LEN as u64);
        let b = reader.next_record().unwrap().unwrap();
        assert_eq!(b, sample());
        assert!(reader.next_record().unwrap().is_none());
        // Re-open at the recorded offset, as the per-host adapter does.
        let mut resumed = TraceReader::new(
            std::io::Cursor::new(bytes[after_first as usize..].to_vec()),
            after_first,
        )
        .unwrap();
        assert_eq!(resumed.next_record().unwrap().unwrap(), sample());
    }

    #[test]
    fn hdma_channel_state_round_trips_through_json() {
        let json = json!({
            "event": "hdma-start", "run": 5, "channels": 3,
            "channel_state": [
                {"channel": 0, "source": 0x7e1234, "table_address": 0x1234, "indirect": 0,
                 "line_count": 4, "repeat": 1, "do_transfer": 1, "b_address": 0x0d, "mode": 2,
                 "data": [1, 2]},
                {"channel": 1, "source": 0x7f0010, "table_address": 0x0010, "indirect": 1,
                 "line_count": 0, "repeat": 0, "do_transfer": 0, "b_address": 0x26, "mode": 0,
                 "data": []}
            ]
        });
        let record = TraceRecord::from_json(&json).unwrap();
        let states = record.hdma_channel_states();
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].data, vec![1, 2]);
        let back = TraceRecord::parse(&record.encode()).unwrap().to_json();
        assert_eq!(back["channel_state"], json["channel_state"]);
        assert_eq!(back["channels"], 3);
    }

    #[test]
    fn rejects_json_lines_input_with_a_clear_message() {
        let error = TraceReader::new(std::io::Cursor::new(b"{\"event\":\"frame\"}\n".to_vec()), 0)
            .err()
            .expect("JSON input must be rejected");
        assert!(error.to_string().contains("Z3TRACE1"));
    }
}
