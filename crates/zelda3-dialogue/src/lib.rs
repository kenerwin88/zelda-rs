//! Pure dialogue source, bytecode, and IR model.
//!
//! This crate intentionally has no dependency on `zelda3`, WRAM, PPU state, or
//! renderer data. It is the modern text authority layer that legacy runtime
//! compatibility and parity tooling can both consume.

pub const TEXT_COMMAND_START_US: u8 = 0x67;
pub const TEXT_CMD_NEXT_PIC: u8 = 0;
pub const TEXT_CMD_CHOOSE: u8 = 1;
pub const TEXT_CMD_ITEM: u8 = 2;
pub const TEXT_CMD_NAME: u8 = 3;
pub const TEXT_CMD_WINDOW: u8 = 4;
pub const TEXT_CMD_NUMBER: u8 = 5;
pub const TEXT_CMD_POSITION: u8 = 6;
pub const TEXT_CMD_SCROLL_SPD: u8 = 7;
pub const TEXT_CMD_SELCHG: u8 = 8;
pub const TEXT_CMD_CHOOSE3: u8 = 10;
pub const TEXT_CMD_CHOOSE2: u8 = 11;
pub const TEXT_CMD_SCROLL: u8 = 12;
pub const TEXT_CMD_1: u8 = 13;
pub const TEXT_CMD_2: u8 = 14;
pub const TEXT_CMD_3: u8 = 15;
pub const TEXT_CMD_COLOR: u8 = 16;
pub const TEXT_CMD_WAIT: u8 = 17;
pub const TEXT_CMD_SOUND: u8 = 18;
pub const TEXT_CMD_SPEED: u8 = 19;
pub const TEXT_CMD_WAITKEY: u8 = 23;
pub const TEXT_CMD_END_MESSAGE: u8 = 24;
pub const TEXT_CMD_IS_LETTER: u8 = 25;
pub const FORMAT_DIALOGUE_SOURCE: &str = "zelda3_dialogue_source_v1";

pub const US_COMMAND_NAMES: [&str; 25] = [
    "next_pic",
    "choose",
    "item",
    "player_name",
    "window",
    "number",
    "position",
    "scroll_speed",
    "selection_change",
    "unused_crash",
    "choose3",
    "choose2",
    "scroll",
    "line1",
    "line2",
    "line3",
    "color",
    "wait",
    "sound",
    "speed",
    "unused_mark",
    "unused_mark2",
    "unused_clear",
    "wait_key",
    "end_message",
];

const TEXT_RENDER_COMMAND_LENGTHS_US: [u8; 25] = [
    0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0,
];
const TEXT_RENDER_SOUND_COMMAND_PARAMS: [u8; 1] = [45];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DialogueDecodedCommand {
    pub param: u8,
    pub command: u8,
    pub multibyte: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DialogueIrOp {
    pub offset: usize,
    pub raw: Vec<u8>,
    pub command: u8,
    pub param: u8,
    pub multibyte: bool,
    pub kind: DialogueIrKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DialogueIrKind {
    Glyph { code: u8 },
    NextPic,
    Choice { options: u8 },
    Item,
    PlayerName,
    Window { style: u8 },
    Number { slot: u8 },
    Position { position: u8 },
    ScrollSpeed { speed: u8 },
    SelectionChange,
    Scroll,
    Line { line: u8 },
    Color { color: u8 },
    Wait { duration: u8 },
    Sound { sound: u8 },
    Speed { speed: u8 },
    WaitKey,
    EndMessage,
    Unused { command: u8 },
    Unknown { command: u8 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DialogueGlyphPlacement {
    pub op_offset: usize,
    pub glyph_code: u8,
    pub line: u8,
    pub x: i16,
    pub y: i16,
    pub width: u8,
    pub color: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DialogueRuntimeSubstitutions {
    pub player_name: Vec<u8>,
    pub number_pairs: [u8; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DialogueSourceDocument {
    pub format: String,
    #[serde(default)]
    pub message_count: Option<usize>,
    pub messages: Vec<DialogueSourceMessage>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DialogueSourceMessage {
    pub id: usize,
    pub source_text: String,
    #[serde(default)]
    pub expanded_sha1: Option<String>,
}

pub const LEGACY_VWF_LINE_Y: [i16; 3] = [0, 16, 32];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueSourceCompileError {
    pub offset: usize,
    pub message: String,
}

impl DialogueSourceCompileError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for DialogueSourceCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "offset {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for DialogueSourceCompileError {}

pub fn decode_dialogue_byte(
    dialogue_flags: u8,
    byte: u8,
    next: Option<u8>,
) -> DialogueDecodedCommand {
    let (param, command, multibyte) = if dialogue_flags & 1 == 0 {
        decode_us_dialogue_byte(byte, next)
    } else {
        decode_extended_dialogue_byte(byte, next)
    };
    DialogueDecodedCommand {
        param,
        command,
        multibyte,
    }
}

pub fn parse_dialogue_ir(dialogue_flags: u8, decoded: &[u8]) -> Vec<DialogueIrOp> {
    let mut ops = Vec::new();
    let mut offset = 0usize;
    while offset < decoded.len() {
        let byte = decoded[offset];
        let decoded_command =
            decode_dialogue_byte(dialogue_flags, byte, decoded.get(offset + 1).copied());
        let raw_len = 1 + usize::from(decoded_command.multibyte);
        let raw = decoded[offset..decoded.len().min(offset + raw_len)].to_vec();
        let kind = ir_kind(decoded_command.command, decoded_command.param);
        let is_end = decoded_command.command == TEXT_CMD_END_MESSAGE;
        ops.push(DialogueIrOp {
            offset,
            raw,
            command: decoded_command.command,
            param: decoded_command.param,
            multibyte: decoded_command.multibyte,
            kind,
        });
        offset += raw_len;
        if is_end {
            break;
        }
    }
    ops
}

pub fn dialogue_ir_op_at(
    dialogue_flags: u8,
    decoded: &[u8],
    offset: usize,
) -> Option<DialogueIrOp> {
    let byte = *decoded.get(offset)?;
    let decoded_command =
        decode_dialogue_byte(dialogue_flags, byte, decoded.get(offset + 1).copied());
    let raw_len = 1 + usize::from(decoded_command.multibyte);
    let raw = decoded[offset..decoded.len().min(offset + raw_len)].to_vec();
    Some(DialogueIrOp {
        offset,
        raw,
        command: decoded_command.command,
        param: decoded_command.param,
        multibyte: decoded_command.multibyte,
        kind: ir_kind(decoded_command.command, decoded_command.param),
    })
}

pub fn layout_dialogue_ir(ir: &[DialogueIrOp], glyph_widths: &[u8]) -> Vec<DialogueGlyphPlacement> {
    let mut placements = Vec::new();
    let mut line = 0usize;
    let mut x_by_line = [0i16; 3];
    let mut color = None;
    for op in ir {
        match &op.kind {
            DialogueIrKind::Glyph { code } => {
                let width = glyph_widths.get(usize::from(*code)).copied().unwrap_or(0);
                placements.push(DialogueGlyphPlacement {
                    op_offset: op.offset,
                    glyph_code: *code,
                    line: line as u8,
                    x: x_by_line[line],
                    y: LEGACY_VWF_LINE_Y[line],
                    width,
                    color,
                });
                x_by_line[line] = x_by_line[line].wrapping_add(i16::from(width));
            }
            DialogueIrKind::Line { line: requested } => {
                line = usize::from(requested.saturating_sub(1)).min(2);
            }
            DialogueIrKind::Color { color: next_color } => {
                color = Some(*next_color);
            }
            DialogueIrKind::EndMessage => break,
            _ => {}
        }
    }
    placements
}

pub fn expand_runtime_dialogue_ir(
    ir: &[DialogueIrOp],
    substitutions: &DialogueRuntimeSubstitutions,
) -> Vec<DialogueIrOp> {
    let mut expanded = Vec::new();
    for op in ir {
        match &op.kind {
            DialogueIrKind::PlayerName => {
                for &code in &substitutions.player_name {
                    expanded.push(synthetic_glyph_op(op.offset, code));
                }
            }
            DialogueIrKind::Number { slot } => {
                let pair = substitutions
                    .number_pairs
                    .get(usize::from(*slot >> 1))
                    .copied()
                    .unwrap_or(0);
                let digit = if slot & 1 != 0 {
                    pair >> 4
                } else {
                    pair & 0x0f
                };
                expanded.push(synthetic_glyph_op(op.offset, 0x34 + digit));
            }
            _ => expanded.push(op.clone()),
        }
    }
    expanded
}

pub fn expand_runtime_render_dialogue_ir(
    ir: &[DialogueIrOp],
    substitutions: &DialogueRuntimeSubstitutions,
) -> Vec<DialogueIrOp> {
    let mut expanded = Vec::new();
    let mut render_offset = 0usize;
    for op in ir {
        match &op.kind {
            DialogueIrKind::PlayerName => {
                for &code in &substitutions.player_name {
                    expanded.push(synthetic_glyph_op(render_offset, code));
                    render_offset += 1;
                }
            }
            DialogueIrKind::Number { slot } => {
                let pair = substitutions
                    .number_pairs
                    .get(usize::from(*slot >> 1))
                    .copied()
                    .unwrap_or(0);
                let digit = if slot & 1 != 0 {
                    pair >> 4
                } else {
                    pair & 0x0f
                };
                expanded.push(synthetic_glyph_op(render_offset, 0x34 + digit));
                render_offset += 1;
            }
            DialogueIrKind::Window { .. }
            | DialogueIrKind::Position { .. }
            | DialogueIrKind::Color { .. } => {
                let mut render_op = op.clone();
                render_op.offset = render_offset;
                expanded.push(render_op);
            }
            _ => {
                let mut render_op = op.clone();
                render_op.offset = render_offset;
                expanded.push(render_op);
                render_offset += op.raw.len().max(1);
            }
        }
    }
    expanded
}

pub fn visible_dialogue_ir_prefix(
    ir: &[DialogueIrOp],
    render_read_pos: usize,
) -> Vec<DialogueIrOp> {
    ir.iter()
        .take_while(|op| op.offset < render_read_pos)
        .cloned()
        .collect()
}

pub fn compile_source_text(source_text: &str) -> Result<Vec<u8>, DialogueSourceCompileError> {
    let mut data = Vec::new();
    let mut offset = 0usize;
    while offset < source_text.len() {
        let rest = &source_text[offset..];
        let ch = rest.chars().next().expect("non-empty source slice");
        if ch == '[' {
            let end = rest
                .find(']')
                .ok_or_else(|| DialogueSourceCompileError::new(offset, "unterminated tag"))?;
            let tag = &rest[1..end];
            append_source_tag(source_text, offset, tag, &mut data)?;
            offset += end + 1;
            continue;
        }
        let code = glyph_code_for_char(ch).ok_or_else(|| {
            DialogueSourceCompileError::new(offset, format!("cannot encode character {ch:?}"))
        })?;
        data.push(code);
        offset += ch.len_utf8();
    }
    Ok(data)
}

pub fn compile_dialogue_source_document(
    source: &DialogueSourceDocument,
) -> Result<Vec<u8>, DialogueSourceCompileError> {
    let compiled_messages = compile_dialogue_source_messages(source)?;

    let empty_dictionary = pack_memblk_arrays(&[Vec::new()]);
    let dialogue_pack =
        pack_memblk_arrays(&[empty_dictionary, pack_memblk_arrays(&compiled_messages)]);
    Ok(pack_memblk_arrays(&[dialogue_pack]))
}

pub fn compile_dialogue_source_ir_table(
    source: &DialogueSourceDocument,
) -> Result<Vec<Vec<DialogueIrOp>>, DialogueSourceCompileError> {
    let compiled_messages = compile_dialogue_source_messages(source)?;
    Ok(compiled_messages
        .iter()
        .map(|message| parse_dialogue_ir(0, message))
        .collect())
}

fn compile_dialogue_source_messages(
    source: &DialogueSourceDocument,
) -> Result<Vec<Vec<u8>>, DialogueSourceCompileError> {
    if source.format != FORMAT_DIALOGUE_SOURCE {
        return Err(DialogueSourceCompileError::new(
            0,
            format!("dialogue source is not {FORMAT_DIALOGUE_SOURCE}"),
        ));
    }
    if let Some(message_count) = source.message_count {
        if message_count != source.messages.len() {
            return Err(DialogueSourceCompileError::new(
                0,
                format!(
                    "dialogue source message_count is {message_count}, expected {}",
                    source.messages.len()
                ),
            ));
        }
    }

    let mut compiled_messages = Vec::with_capacity(source.messages.len());
    for (expected_id, message) in source.messages.iter().enumerate() {
        if message.id != expected_id {
            return Err(DialogueSourceCompileError::new(
                expected_id,
                format!("message {expected_id} id mismatch: got {}", message.id),
            ));
        }
        let compiled = compile_source_text(&message.source_text).map_err(|err| {
            DialogueSourceCompileError::new(
                err.offset,
                format!("message {expected_id} source_text compile failed: {err}"),
            )
        })?;
        if let Some(expected_sha1) = &message.expanded_sha1 {
            let actual_sha1 = sha1_hex(&compiled);
            if !expected_sha1.eq_ignore_ascii_case(&actual_sha1) {
                return Err(DialogueSourceCompileError::new(
                    expected_id,
                    format!(
                        "message {expected_id} expanded_sha1 is {expected_sha1}, expected {actual_sha1}"
                    ),
                ));
            }
        }
        compiled_messages.push(compiled);
    }

    Ok(compiled_messages)
}

fn sha1_hex(data: &[u8]) -> String {
    use sha1::{Digest, Sha1};

    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn synthetic_glyph_op(offset: usize, code: u8) -> DialogueIrOp {
    DialogueIrOp {
        offset,
        raw: vec![code],
        command: TEXT_CMD_IS_LETTER,
        param: code,
        multibyte: false,
        kind: DialogueIrKind::Glyph { code },
    }
}

pub fn glyph_text(code: u8) -> Option<&'static str> {
    Some(match code {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "D",
        4 => "E",
        5 => "F",
        6 => "G",
        7 => "H",
        8 => "I",
        9 => "J",
        10 => "K",
        11 => "L",
        12 => "M",
        13 => "N",
        14 => "O",
        15 => "P",
        16 => "Q",
        17 => "R",
        18 => "S",
        19 => "T",
        20 => "U",
        21 => "V",
        22 => "W",
        23 => "X",
        24 => "Y",
        25 => "Z",
        26 => "a",
        27 => "b",
        28 => "c",
        29 => "d",
        30 => "e",
        31 => "f",
        32 => "g",
        33 => "h",
        34 => "i",
        35 => "j",
        36 => "k",
        37 => "l",
        38 => "m",
        39 => "n",
        40 => "o",
        41 => "p",
        42 => "q",
        43 => "r",
        44 => "s",
        45 => "t",
        46 => "u",
        47 => "v",
        48 => "w",
        49 => "x",
        50 => "y",
        51 => "z",
        52 => "0",
        53 => "1",
        54 => "2",
        55 => "3",
        56 => "4",
        57 => "5",
        58 => "6",
        59 => "7",
        60 => "8",
        61 => "9",
        62 => "!",
        63 => "?",
        64 => "-",
        65 => ".",
        66 => ",",
        67 => "[...]",
        68 => ">",
        69 => "(",
        70 => ")",
        71 => "[Ankh]",
        72 => "[Waves]",
        73 => "[Snake]",
        74 => "[LinkL]",
        75 => "[LinkR]",
        76 => "\"",
        77 => "[Up]",
        78 => "[Down]",
        79 => "[Left]",
        80 => "[Right]",
        81 => "'",
        82 => "[1HeartL]",
        83 => "[1HeartR]",
        84 => "[2HeartL]",
        85 => "[3HeartL]",
        86 => "[3HeartR]",
        87 => "[4HeartL]",
        88 => "[4HeartR]",
        89 => " ",
        90 => "<",
        91 => "[A]",
        92 => "[B]",
        93 => "[X]",
        94 => "[Y]",
        _ => return None,
    })
}

fn pack_memblk_arrays(items: &[Vec<u8>]) -> Vec<u8> {
    assert!(!items.is_empty(), "memblk array must not be empty");
    let payload_before_last = items[..items.len() - 1].iter().map(Vec::len).sum::<usize>();
    let wide_offsets = payload_before_last >= 65536;
    let mut data = Vec::new();
    let mut offset = 0usize;
    for item in &items[..items.len() - 1] {
        offset += item.len();
        if wide_offsets {
            data.extend_from_slice(&(offset as u32).to_le_bytes());
        } else {
            data.extend_from_slice(&(offset as u16).to_le_bytes());
        }
    }
    for item in items {
        data.extend_from_slice(item);
    }
    let marker = if wide_offsets {
        8192 + items.len() - 1
    } else {
        items.len() - 1
    };
    data.extend_from_slice(&(marker as u16).to_le_bytes());
    data
}

fn append_source_tag(
    source_text: &str,
    offset: usize,
    tag: &str,
    data: &mut Vec<u8>,
) -> Result<(), DialogueSourceCompileError> {
    if let Some(code) = bracket_glyph_tag_code(tag) {
        data.push(code);
        return Ok(());
    }
    let parts = tag.split_whitespace().collect::<Vec<_>>();
    if parts.is_empty() {
        return Err(DialogueSourceCompileError::new(offset, "empty tag"));
    }
    if parts[0] == "glyph" && parts.len() == 2 {
        data.push(parse_source_byte_param(offset, parts[1])?);
        return Ok(());
    }
    let Some(command) = US_COMMAND_NAMES.iter().position(|name| *name == parts[0]) else {
        return Err(DialogueSourceCompileError::new(
            offset,
            format!("unknown tag [{tag}]"),
        ));
    };
    let param_len = TEXT_RENDER_COMMAND_LENGTHS_US[command];
    if param_len == 0 {
        if parts.len() != 1 {
            return Err(DialogueSourceCompileError::new(
                offset,
                format!("[{tag}] does not take a parameter"),
            ));
        }
        data.push(TEXT_COMMAND_START_US + command as u8);
        return Ok(());
    }
    if parts.len() != 2 {
        return Err(DialogueSourceCompileError::new(
            offset,
            format!("[{tag}] needs one byte parameter"),
        ));
    }
    if param_len != 1 {
        return Err(DialogueSourceCompileError::new(
            offset,
            format!("unsupported parameter count {param_len} for [{tag}]"),
        ));
    }
    let param = parse_source_byte_param(
        offset + source_text[offset..].find(parts[1]).unwrap_or(0),
        parts[1],
    )?;
    data.push(TEXT_COMMAND_START_US + command as u8);
    data.push(param);
    Ok(())
}

fn parse_source_byte_param(offset: usize, value: &str) -> Result<u8, DialogueSourceCompileError> {
    let text = value.strip_prefix("0x").unwrap_or(value);
    u8::from_str_radix(text, 16).map_err(|_| {
        DialogueSourceCompileError::new(offset, format!("invalid byte parameter {value:?}"))
    })
}

fn glyph_code_for_char(ch: char) -> Option<u8> {
    match ch {
        'A'..='Z' => Some(ch as u8 - b'A'),
        'a'..='z' => Some(26 + ch as u8 - b'a'),
        '0'..='9' => Some(52 + ch as u8 - b'0'),
        '!' => Some(62),
        '?' => Some(63),
        '-' => Some(64),
        '.' => Some(65),
        ',' => Some(66),
        '>' => Some(68),
        '(' => Some(69),
        ')' => Some(70),
        '"' => Some(76),
        '\'' => Some(81),
        ' ' => Some(89),
        '<' => Some(90),
        _ => None,
    }
}

fn bracket_glyph_tag_code(tag: &str) -> Option<u8> {
    (0u8..=u8::MAX).find(|code| {
        glyph_text(*code).and_then(|text| text.strip_prefix('[')?.strip_suffix(']')) == Some(tag)
    })
}

fn decode_us_dialogue_byte(byte: u8, next: Option<u8>) -> (u8, u8, bool) {
    if byte < TEXT_COMMAND_START_US || byte >= 0x80 {
        return (
            if byte >= 0x80 { 26 } else { byte },
            TEXT_CMD_IS_LETTER,
            false,
        );
    }
    let command = byte - TEXT_COMMAND_START_US;
    if TEXT_RENDER_COMMAND_LENGTHS_US[command as usize] != 0 {
        (next.unwrap_or(0), command, true)
    } else {
        (0, command, false)
    }
}

fn decode_extended_dialogue_byte(byte: u8, next: Option<u8>) -> (u8, u8, bool) {
    if byte < 0x7f {
        (byte, TEXT_CMD_IS_LETTER, false)
    } else if byte < 0x87 {
        match byte - 0x7f {
            0 => (0, TEXT_CMD_END_MESSAGE, false),
            1 => (0, TEXT_CMD_SCROLL, false),
            2 => (0, TEXT_CMD_WAITKEY, false),
            3 => (0, TEXT_CMD_1, false),
            4 => (0, TEXT_CMD_2, false),
            5 => (0, TEXT_CMD_3, false),
            6 | 7 => (0, TEXT_CMD_NAME, false),
            _ => unreachable!(),
        }
    } else {
        match next.unwrap_or(0) {
            b @ 0x00..=0x0f => (b & 0x0f, TEXT_CMD_WAIT, true),
            b @ 0x10..=0x1f => (b & 0x0f, TEXT_CMD_COLOR, true),
            b @ 0x20..=0x2f => (b & 0x0f, TEXT_CMD_NUMBER, true),
            b @ 0x30..=0x3f => (b & 0x0f, TEXT_CMD_SPEED, true),
            b @ 0x40..=0x4f => (
                TEXT_RENDER_SOUND_COMMAND_PARAMS[(b & 0x0f) as usize],
                TEXT_CMD_SOUND,
                true,
            ),
            0x80 => (0, TEXT_CMD_CHOOSE, true),
            0x81 => (0, TEXT_CMD_CHOOSE2, true),
            0x82 => (0, TEXT_CMD_CHOOSE3, true),
            0x83 => (0, TEXT_CMD_SELCHG, true),
            0x84 => (0, TEXT_CMD_ITEM, true),
            0x85 => (0, TEXT_CMD_NEXT_PIC, true),
            0x86 => (2, TEXT_CMD_WINDOW, true),
            0x87 => (0, TEXT_CMD_POSITION, true),
            0x88 => (1, TEXT_CMD_POSITION, true),
            _ => (26, TEXT_CMD_IS_LETTER, false),
        }
    }
}

fn ir_kind(command: u8, param: u8) -> DialogueIrKind {
    match command {
        TEXT_CMD_IS_LETTER => DialogueIrKind::Glyph { code: param },
        TEXT_CMD_NEXT_PIC => DialogueIrKind::NextPic,
        TEXT_CMD_CHOOSE => DialogueIrKind::Choice { options: 2 },
        TEXT_CMD_ITEM => DialogueIrKind::Item,
        TEXT_CMD_NAME => DialogueIrKind::PlayerName,
        TEXT_CMD_WINDOW => DialogueIrKind::Window { style: param },
        TEXT_CMD_NUMBER => DialogueIrKind::Number { slot: param },
        TEXT_CMD_POSITION => DialogueIrKind::Position { position: param },
        TEXT_CMD_SCROLL_SPD => DialogueIrKind::ScrollSpeed { speed: param },
        TEXT_CMD_SELCHG => DialogueIrKind::SelectionChange,
        9 | 20 | 21 | 22 => DialogueIrKind::Unused { command },
        TEXT_CMD_CHOOSE3 => DialogueIrKind::Choice { options: 3 },
        TEXT_CMD_CHOOSE2 => DialogueIrKind::Choice { options: 2 },
        TEXT_CMD_SCROLL => DialogueIrKind::Scroll,
        TEXT_CMD_1 => DialogueIrKind::Line { line: 1 },
        TEXT_CMD_2 => DialogueIrKind::Line { line: 2 },
        TEXT_CMD_3 => DialogueIrKind::Line { line: 3 },
        TEXT_CMD_COLOR => DialogueIrKind::Color { color: param },
        TEXT_CMD_WAIT => DialogueIrKind::Wait { duration: param },
        TEXT_CMD_SOUND => DialogueIrKind::Sound { sound: param },
        TEXT_CMD_SPEED => DialogueIrKind::Speed { speed: param },
        TEXT_CMD_WAITKEY => DialogueIrKind::WaitKey,
        TEXT_CMD_END_MESSAGE => DialogueIrKind::EndMessage,
        _ => DialogueIrKind::Unknown { command },
    }
}

// -------------------------------------------------------------------------
// Editable TOML message source.
//
// `messages.toml` is the human-authored dialogue content (id + literal-string
// text, with `#` comments allowed). Editing it and rebuilding is all that is
// needed to change the dialogue — the text compiles straight to `kDialogue`.
// -------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DialogueMessagesDocument {
    pub format: String,
    #[serde(default, rename = "message")]
    pub messages: Vec<DialogueMessageEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DialogueMessageEntry {
    pub id: usize,
    pub text: String,
}

/// Parse `messages.toml` into a [`DialogueMessagesDocument`].
pub fn parse_messages_document(toml_str: &str) -> Result<DialogueMessagesDocument, String> {
    toml::from_str(toml_str).map_err(|err| err.to_string())
}

fn compile_messages_to_bytes(
    doc: &DialogueMessagesDocument,
) -> Result<Vec<Vec<u8>>, DialogueSourceCompileError> {
    if doc.format != FORMAT_DIALOGUE_SOURCE {
        return Err(DialogueSourceCompileError::new(
            0,
            format!(
                "dialogue messages format is {}, expected {FORMAT_DIALOGUE_SOURCE}",
                doc.format
            ),
        ));
    }
    let mut compiled = Vec::with_capacity(doc.messages.len());
    for (expected_id, message) in doc.messages.iter().enumerate() {
        if message.id != expected_id {
            return Err(DialogueSourceCompileError::new(
                expected_id,
                format!("message {expected_id} id mismatch: got {}", message.id),
            ));
        }
        let bytes = compile_source_text(&message.text).map_err(|err| {
            DialogueSourceCompileError::new(
                err.offset,
                format!("message {expected_id} text compile failed: {err}"),
            )
        })?;
        compiled.push(bytes);
    }
    Ok(compiled)
}

/// Compile a [`DialogueMessagesDocument`] into an uncompressed `kDialogue` asset.
pub fn compile_messages_document(
    doc: &DialogueMessagesDocument,
) -> Result<Vec<u8>, DialogueSourceCompileError> {
    let compiled_messages = compile_messages_to_bytes(doc)?;
    let empty_dictionary = pack_memblk_arrays(&[Vec::new()]);
    let dialogue_pack =
        pack_memblk_arrays(&[empty_dictionary, pack_memblk_arrays(&compiled_messages)]);
    Ok(pack_memblk_arrays(&[dialogue_pack]))
}

/// Compile a [`DialogueMessagesDocument`] into the per-message semantic IR table
/// (the `kDialogueSourceSemantic` sidecar payload).
pub fn compile_messages_ir_table(
    doc: &DialogueMessagesDocument,
) -> Result<Vec<Vec<DialogueIrOp>>, DialogueSourceCompileError> {
    let compiled_messages = compile_messages_to_bytes(doc)?;
    Ok(compiled_messages
        .iter()
        .map(|message| parse_dialogue_ir(0, message))
        .collect())
}

/// Emit `messages.toml`, preferring escape-free literal strings for the text.
pub fn serialize_messages_toml(doc: &DialogueMessagesDocument) -> String {
    let mut out = format!("format = \"{}\"\n", doc.format);
    for message in &doc.messages {
        out.push_str("\n[[message]]\n");
        out.push_str(&format!("id = {}\n", message.id));
        out.push_str(&format!("text = {}\n", toml_string_scalar(&message.text)));
    }
    out
}

/// Render a string as a TOML value, preferring a multi-line literal (`'''…'''`,
/// which escapes nothing, allows `'`/`"`/`[`) and falling back to a basic string
/// only for the rare text that contains `'''`, a trailing `'`, or a control char.
fn toml_string_scalar(s: &str) -> String {
    if s.is_empty() {
        return "\"\"".to_string();
    }
    let has_control = s.chars().any(|c| c.is_control());
    if !has_control && !s.contains("'''") && !s.ends_with('\'') {
        return format!("'''{s}'''");
    }
    let mut escaped = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04X}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_messages_document() -> DialogueMessagesDocument {
        DialogueMessagesDocument {
            format: FORMAT_DIALOGUE_SOURCE.to_string(),
            messages: vec![
                DialogueMessageEntry {
                    id: 0,
                    text: "AB[end_message]".to_string(),
                },
                DialogueMessageEntry {
                    id: 1,
                    text: "I'm \"here\"![line2]C".to_string(),
                },
            ],
        }
    }

    #[test]
    fn toml_messages_compile_identically_to_legacy_source_document() {
        let messages = sample_messages_document();
        let via_toml = compile_messages_document(&messages).unwrap();

        let via_json = compile_dialogue_source_document(&DialogueSourceDocument {
            format: FORMAT_DIALOGUE_SOURCE.to_string(),
            message_count: Some(2),
            messages: messages
                .messages
                .iter()
                .map(|m| DialogueSourceMessage {
                    id: m.id,
                    source_text: m.text.clone(),
                    expanded_sha1: None,
                })
                .collect(),
        })
        .unwrap();

        assert_eq!(via_toml, via_json);
    }

    #[test]
    fn messages_toml_round_trips_through_literal_strings() {
        let doc = sample_messages_document();
        let text = serialize_messages_toml(&doc);
        // Apostrophes/quotes/brackets survive as an escape-free literal string.
        assert!(text.contains("text = '''I'm \"here\"![line2]C'''"));
        assert!(text.contains("text = '''AB[end_message]'''"));
        assert_eq!(parse_messages_document(&text).unwrap(), doc);
    }

    #[test]
    fn empty_text_serializes_as_basic_empty_string() {
        let doc = DialogueMessagesDocument {
            format: FORMAT_DIALOGUE_SOURCE.to_string(),
            messages: vec![DialogueMessageEntry {
                id: 0,
                text: String::new(),
            }],
        };
        let text = serialize_messages_toml(&doc);
        assert!(text.contains("text = \"\""));
        assert_eq!(parse_messages_document(&text).unwrap(), doc);
    }

    #[test]
    fn parses_us_dialogue_glyphs_and_commands() {
        let ops = parse_dialogue_ir(
            0,
            &[
                0,
                1,
                TEXT_COMMAND_START_US + TEXT_CMD_2,
                0x78,
                3,
                TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
            ],
        );

        assert_eq!(ops.len(), 5);
        assert_eq!(ops[0].kind, DialogueIrKind::Glyph { code: 0 });
        assert_eq!(ops[1].kind, DialogueIrKind::Glyph { code: 1 });
        assert_eq!(ops[2].kind, DialogueIrKind::Line { line: 2 });
        assert_eq!(ops[3].kind, DialogueIrKind::Wait { duration: 3 });
        assert_eq!(ops[3].raw, vec![0x78, 3]);
        assert!(ops[3].multibyte);
        assert_eq!(ops[4].kind, DialogueIrKind::EndMessage);
    }

    #[test]
    fn maps_us_high_bytes_to_lowercase_a_like_runtime() {
        let decoded = decode_dialogue_byte(0, 0x88, None);

        assert_eq!(decoded.command, TEXT_CMD_IS_LETTER);
        assert_eq!(decoded.param, 26);
        assert!(!decoded.multibyte);
    }

    #[test]
    fn parses_extended_dialogue_commands() {
        let ops = parse_dialogue_ir(1, &[0x7f, 0x87, 0x10, 0x87, 0x80]);

        assert_eq!(ops[0].kind, DialogueIrKind::EndMessage);
        assert_eq!(ops.len(), 1);

        let color = parse_dialogue_ir(1, &[0x87, 0x12]);
        assert_eq!(color[0].kind, DialogueIrKind::Color { color: 2 });
        assert_eq!(color[0].raw, vec![0x87, 0x12]);

        let choose = parse_dialogue_ir(1, &[0x87, 0x80]);
        assert_eq!(choose[0].kind, DialogueIrKind::Choice { options: 2 });
    }

    #[test]
    fn exposes_human_glyph_labels_for_source_and_debugging() {
        assert_eq!(glyph_text(0), Some("A"));
        assert_eq!(glyph_text(67), Some("[...]"));
        assert_eq!(glyph_text(91), Some("[A]"));
        assert_eq!(glyph_text(255), None);
    }

    #[test]
    fn compiles_editable_source_text_to_legacy_bytecode() {
        let bytecode = compile_source_text("Hi[...][line1][color 02][A][end_message]").unwrap();

        assert_eq!(bytecode, vec![7, 34, 67, 0x74, 0x77, 2, 91, 0x7f]);
    }

    #[test]
    fn compiles_dialogue_source_document_to_uncompressed_dialogue_asset() {
        let asset = compile_dialogue_source_document(&DialogueSourceDocument {
            format: FORMAT_DIALOGUE_SOURCE.to_string(),
            message_count: Some(2),
            messages: vec![
                DialogueSourceMessage {
                    id: 0,
                    source_text: "AB[end_message]".to_string(),
                    expanded_sha1: None,
                },
                DialogueSourceMessage {
                    id: 1,
                    source_text: "C".to_string(),
                    expanded_sha1: None,
                },
            ],
        })
        .unwrap();

        assert_eq!(
            asset,
            vec![
                0x02, 0x00, // outer offset: dictionary block length
                0x00, 0x00, // empty dictionary block
                0x03, 0x00, // message block offset: message 0 length
                0x00, 0x01, 0x7f, // message 0
                0x02, // message 1
                0x01, 0x00, // message block marker: 2 items
                0x01, 0x00, // outer marker: dictionary + messages
                0x00, 0x00, // kDialogue marker: one dialogue pack
            ]
        );
    }

    #[test]
    fn compiles_dialogue_source_document_to_semantic_ir_table() {
        let table = compile_dialogue_source_ir_table(&DialogueSourceDocument {
            format: FORMAT_DIALOGUE_SOURCE.to_string(),
            message_count: Some(2),
            messages: vec![
                DialogueSourceMessage {
                    id: 0,
                    source_text: "A[color 02]B[end_message]".to_string(),
                    expanded_sha1: None,
                },
                DialogueSourceMessage {
                    id: 1,
                    source_text: "C".to_string(),
                    expanded_sha1: None,
                },
            ],
        })
        .unwrap();

        assert_eq!(table.len(), 2);
        assert_eq!(table[0][0].kind, DialogueIrKind::Glyph { code: 0 });
        assert_eq!(table[0][1].kind, DialogueIrKind::Color { color: 2 });
        assert_eq!(table[0][2].kind, DialogueIrKind::Glyph { code: 1 });
        assert_eq!(table[0][3].kind, DialogueIrKind::EndMessage);
        assert_eq!(table[1][0].kind, DialogueIrKind::Glyph { code: 2 });
    }

    #[test]
    fn dialogue_source_document_rejects_stale_expanded_sha1() {
        let err = compile_dialogue_source_document(&DialogueSourceDocument {
            format: FORMAT_DIALOGUE_SOURCE.to_string(),
            message_count: Some(1),
            messages: vec![DialogueSourceMessage {
                id: 0,
                source_text: "AB[end_message]".to_string(),
                expanded_sha1: Some("efd7522b39c721b5b97d5cee7b95a6568571310c".to_string()),
            }],
        })
        .unwrap_err();

        assert_eq!(err.offset, 0);
        assert_eq!(
            err.message,
            "message 0 expanded_sha1 is efd7522b39c721b5b97d5cee7b95a6568571310c, expected d5620b62b131744f6c918ca3d47c39809c49bcc4"
        );
    }

    #[test]
    fn dialogue_source_document_rejects_non_contiguous_message_ids() {
        let err = compile_dialogue_source_document(&DialogueSourceDocument {
            format: FORMAT_DIALOGUE_SOURCE.to_string(),
            message_count: Some(1),
            messages: vec![DialogueSourceMessage {
                id: 7,
                source_text: "A".to_string(),
                expanded_sha1: None,
            }],
        })
        .unwrap_err();

        assert_eq!(err.offset, 0);
        assert_eq!(err.message, "message 0 id mismatch: got 7");
    }

    #[test]
    fn dialogue_source_document_rejects_stale_message_count() {
        let err = compile_dialogue_source_document(&DialogueSourceDocument {
            format: FORMAT_DIALOGUE_SOURCE.to_string(),
            message_count: Some(2),
            messages: vec![DialogueSourceMessage {
                id: 0,
                source_text: "A".to_string(),
                expanded_sha1: None,
            }],
        })
        .unwrap_err();

        assert_eq!(
            err.message,
            "dialogue source message_count is 2, expected 1"
        );
    }

    #[test]
    fn expands_runtime_dialogue_commands_into_glyph_ir() {
        let ir = vec![
            DialogueIrOp {
                offset: 0,
                raw: vec![TEXT_COMMAND_START_US + TEXT_CMD_NAME],
                command: TEXT_CMD_NAME,
                param: 0,
                multibyte: false,
                kind: DialogueIrKind::PlayerName,
            },
            DialogueIrOp {
                offset: 1,
                raw: vec![TEXT_COMMAND_START_US + TEXT_CMD_NUMBER, 1],
                command: TEXT_CMD_NUMBER,
                param: 1,
                multibyte: true,
                kind: DialogueIrKind::Number { slot: 1 },
            },
            DialogueIrOp {
                offset: 3,
                raw: vec![TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE],
                command: TEXT_CMD_END_MESSAGE,
                param: 0,
                multibyte: false,
                kind: DialogueIrKind::EndMessage,
            },
        ];
        let expanded = expand_runtime_dialogue_ir(
            &ir,
            &DialogueRuntimeSubstitutions {
                player_name: vec![0, 1],
                number_pairs: [0x42, 0],
            },
        );

        assert_eq!(
            expanded.iter().map(|op| &op.kind).collect::<Vec<_>>(),
            vec![
                &DialogueIrKind::Glyph { code: 0 },
                &DialogueIrKind::Glyph { code: 1 },
                &DialogueIrKind::Glyph { code: 56 },
                &DialogueIrKind::EndMessage,
            ]
        );
        assert_eq!(expanded[0].offset, 0);
        assert_eq!(expanded[1].offset, 0);
        assert_eq!(expanded[2].offset, 1);
    }

    #[test]
    fn expands_runtime_render_dialogue_ir_with_decoded_stream_offsets() {
        let ir = parse_dialogue_ir(
            0,
            &[
                TEXT_COMMAND_START_US + TEXT_CMD_COLOR,
                2,
                TEXT_COMMAND_START_US + TEXT_CMD_NAME,
                TEXT_COMMAND_START_US + TEXT_CMD_NUMBER,
                1,
                TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
            ],
        );
        let expanded = expand_runtime_render_dialogue_ir(
            &ir,
            &DialogueRuntimeSubstitutions {
                player_name: vec![0, 1],
                number_pairs: [0x42, 0],
            },
        );

        assert_eq!(
            expanded.iter().map(|op| &op.kind).collect::<Vec<_>>(),
            vec![
                &DialogueIrKind::Color { color: 2 },
                &DialogueIrKind::Glyph { code: 0 },
                &DialogueIrKind::Glyph { code: 1 },
                &DialogueIrKind::Glyph { code: 56 },
                &DialogueIrKind::EndMessage,
            ]
        );
        assert_eq!(
            expanded.iter().map(|op| op.offset).collect::<Vec<_>>(),
            vec![0, 0, 1, 2, 3]
        );

        let visible = visible_dialogue_ir_prefix(&expanded, 1);
        assert_eq!(
            visible.iter().map(|op| &op.kind).collect::<Vec<_>>(),
            vec![
                &DialogueIrKind::Color { color: 2 },
                &DialogueIrKind::Glyph { code: 0 },
            ]
        );
        let layout = layout_dialogue_ir(&visible, &[5]);
        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0].glyph_code, 0);
        assert_eq!(layout[0].color, Some(2));
    }

    #[test]
    fn reports_source_compile_errors_with_offsets() {
        let err = compile_source_text("A[bad_tag]").unwrap_err();

        assert_eq!(err.offset, 1);
        assert_eq!(err.message, "unknown tag [bad_tag]");
    }

    #[test]
    fn lays_out_glyphs_from_ir_using_legacy_vwf_line_positions() {
        let ir = parse_dialogue_ir(
            0,
            &[
                0,
                1,
                TEXT_COMMAND_START_US + TEXT_CMD_2,
                2,
                3,
                TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
            ],
        );
        let mut widths = vec![0u8; 256];
        widths[0] = 5;
        widths[1] = 6;
        widths[2] = 7;
        widths[3] = 8;

        let layout = layout_dialogue_ir(&ir, &widths);

        assert_eq!(
            layout,
            vec![
                DialogueGlyphPlacement {
                    op_offset: 0,
                    glyph_code: 0,
                    line: 0,
                    x: 0,
                    y: 0,
                    width: 5,
                    color: None,
                },
                DialogueGlyphPlacement {
                    op_offset: 1,
                    glyph_code: 1,
                    line: 0,
                    x: 5,
                    y: 0,
                    width: 6,
                    color: None,
                },
                DialogueGlyphPlacement {
                    op_offset: 3,
                    glyph_code: 2,
                    line: 1,
                    x: 0,
                    y: 16,
                    width: 7,
                    color: None,
                },
                DialogueGlyphPlacement {
                    op_offset: 4,
                    glyph_code: 3,
                    line: 1,
                    x: 7,
                    y: 16,
                    width: 8,
                    color: None,
                },
            ]
        );
    }

    #[test]
    fn layout_tracks_active_text_color_per_glyph() {
        let ir = parse_dialogue_ir(
            0,
            &[
                0,
                TEXT_COMMAND_START_US + TEXT_CMD_COLOR,
                2,
                1,
                TEXT_COMMAND_START_US + TEXT_CMD_COLOR,
                3,
                2,
                TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
            ],
        );
        let widths = vec![5u8; 256];

        let layout = layout_dialogue_ir(&ir, &widths);

        assert_eq!(layout[0].glyph_code, 0);
        assert_eq!(layout[0].color, None);
        assert_eq!(layout[1].glyph_code, 1);
        assert_eq!(layout[1].color, Some(2));
        assert_eq!(layout[2].glyph_code, 2);
        assert_eq!(layout[2].color, Some(3));
    }
}
