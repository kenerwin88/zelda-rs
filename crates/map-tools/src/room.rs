use crate::{array, ensure, integer, keys, unchanged, without, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, ops::Range};

pub const FORMAT: &str = "zelda3_dungeon_room_v1";
pub const ROOM_COUNT: usize = 0x140;
pub const SIGNATURE: &[u8] = b"Zelda3_v0     \n\0";
pub const ROOM_ASSETS: &[(usize, &str)] = &[
    (3, "kDungeonRoom"),
    (4, "kDungeonRoomOffs"),
    (5, "kDungeonRoomDoorOffs"),
    (6, "kDungeonRoomHeaders"),
    (7, "kDungeonRoomHeadersOffs"),
    (58, "kDungeonSprites"),
    (59, "kDungeonSpriteOffs"),
];

// Asset suffix, JSON field, bytes per value, values per entrance, signed byte.
pub const ENTRANCE_FIELDS: &[(&str, &str, usize, usize, bool)] = &[
    ("rooms", "room", 2, 1, false),
    ("relativeCoords", "relative_coords", 1, 8, false),
    ("scrollX", "scroll_x", 2, 1, false),
    ("scrollY", "scroll_y", 2, 1, false),
    ("playerX", "player_x", 2, 1, false),
    ("playerY", "player_y", 2, 1, false),
    ("cameraX", "camera_x", 2, 1, false),
    ("cameraY", "camera_y", 2, 1, false),
    ("blockset", "blockset", 1, 1, false),
    ("floor", "floor", 1, 1, true),
    ("palace", "palace", 1, 1, true),
    ("doorwayOrientation", "doorway_orientation", 1, 1, false),
    ("startingBg", "starting_bg", 1, 1, false),
    ("quadrant1", "quadrant1", 1, 1, false),
    ("quadrant2", "quadrant2", 1, 1, false),
    ("doorSettings", "door_settings", 2, 1, false),
    ("musicTrack", "music_track", 1, 1, false),
];

pub fn sha256(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

pub fn take(data: &[u8], start: usize, len: usize) -> Result<&[u8]> {
    let end = start.checked_add(len).ok_or("asset range overflow")?;
    data.get(start..end)
        .ok_or_else(|| format!("truncated asset at offset {start}, need {len} bytes").into())
}

pub fn word(data: &[u8], offset: usize) -> Result<u16> {
    let b = take(data, offset, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}

fn dword(data: &[u8], offset: usize) -> Result<usize> {
    let b = take(data, offset, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
}

pub struct Pack {
    data: Vec<u8>,
    names: Vec<String>,
    ranges: Vec<Range<usize>>,
    hash: String,
}

impl Pack {
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        ensure(
            data.len() >= 88 && data.get(..16) == Some(SIGNATURE),
            "invalid asset pack signature",
        )?;
        let count = dword(&data, 80)?;
        let name_size = dword(&data, 84)?;
        ensure(count <= (data.len() - 88) / 4, "truncated asset size table")?;
        let name_start = 88 + count * 4;
        let raw = take(&data, name_start, name_size)?;
        ensure(
            raw.last() == Some(&0),
            "asset names must be null terminated",
        )?;
        let names: Vec<String> = std::str::from_utf8(&raw[..raw.len() - 1])?
            .split('\0')
            .map(str::to_owned)
            .collect();
        ensure(
            names.len() == count
                && names.iter().all(|s| !s.is_empty())
                && names.iter().collect::<HashSet<_>>().len() == count,
            "asset names must be unique and match the asset count",
        )?;
        let mut offset = name_start + name_size;
        let mut ranges = Vec::with_capacity(count);
        for i in 0..count {
            let size = dword(&data, 88 + i * 4)?;
            offset = offset.checked_add(3).ok_or("asset alignment overflow")? & !3;
            take(&data, offset, size)?;
            ranges.push(offset..offset + size);
            offset += size;
        }
        for &(i, name) in ROOM_ASSETS {
            ensure(
                names.get(i).map(String::as_str) == Some(name),
                format!("asset {i} must be {name}"),
            )?;
        }
        let hash = sha256(&data);
        let pack = Self {
            data,
            names,
            ranges,
            hash,
        };
        for i in [4, 5, 7, 59] {
            ensure(
                pack.asset(i)?.len() == ROOM_COUNT * 2,
                format!("{} must describe {ROOM_COUNT} rooms", pack.names[i]),
            )?;
        }
        Ok(pack)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn hash(&self) -> &str {
        &self.hash
    }
    pub fn names(&self) -> &[String] {
        &self.names
    }
    pub fn range(&self, index: usize) -> Result<Range<usize>> {
        self.ranges
            .get(index)
            .cloned()
            .ok_or_else(|| format!("missing asset {index}").into())
    }
    pub fn asset(&self, index: usize) -> Result<&[u8]> {
        Ok(&self.data[self.range(index)?])
    }
    pub fn offset(&self, table: usize, room: usize) -> Result<usize> {
        ensure(room < ROOM_COUNT, "room_id out of range")?;
        Ok(word(self.asset(table)?, room * 2)? as usize)
    }
    fn named_index(&self, name: &str) -> Result<usize> {
        self.names
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| format!("missing asset {name}").into())
    }
    pub fn entrances(&self) -> Result<Vec<Value>> {
        let rooms = self.asset(self.named_index("kEntranceData_rooms")?)?;
        ensure(rooms.len() % 2 == 0, "invalid entrance room table length")?;
        let count = rooms.len() / 2;
        let mut records: Vec<Value> = (0..count).map(|index| json!({"index": index})).collect();
        for &(suffix, field, size, values, signed) in ENTRANCE_FIELDS {
            let data = self.asset(self.named_index(&format!("kEntranceData_{suffix}"))?)?;
            ensure(
                data.len() == count * size * values,
                format!("entrance {field} table length mismatch"),
            )?;
            for (i, record) in records.iter_mut().enumerate() {
                let mut decoded = Vec::new();
                for j in 0..values {
                    let at = (i * values + j) * size;
                    decoded.push(json!(if size == 2 {
                        word(data, at)? as i64
                    } else if signed {
                        data[at] as i8 as i64
                    } else {
                        data[at] as i64
                    }));
                }
                record[field] = if values == 1 {
                    decoded.remove(0)
                } else {
                    json!(decoded)
                };
            }
        }
        Ok(records)
    }
}

pub fn decode_object(raw: &[u8]) -> Result<Value> {
    let raw = take(raw, 0, 3)?;
    let value = word(raw, 0)?;
    let index = raw[2];
    if value & 0xfc == 0xfc {
        return Ok(json!({"kind":"type2", "id":index & 63,
            "x":((value & 3) << 4) | (value >> 12), "y":(((value >> 8) & 15) << 2) | (index >> 6) as u16}));
    }
    let mut obj = json!({"kind":"type1", "id":index, "x":(value & 255) >> 2, "y":value >> 10});
    if index >= 0xf8 {
        obj["kind"] = json!("type3");
        obj["id"] = json!(((index as u16 & 7) << 4) | (((value >> 8) & 3) << 2) | (value & 3));
    } else {
        obj["width_bits"] = json!(value & 3);
        obj["height_bits"] = json!((value >> 8) & 3);
    }
    Ok(obj)
}

pub fn encode_object(obj: &Value) -> Result<Vec<u8>> {
    let kind = obj["kind"].as_str().ok_or("object kind must be a string")?;
    keys(
        obj,
        if kind == "type1" {
            &["kind", "id", "x", "y", "width_bits", "height_bits"]
        } else {
            &["kind", "id", "x", "y"]
        },
        "object",
    )?;
    let x = integer(
        &obj["x"],
        0,
        if kind == "type2" { 63 } else { 62 },
        "object.x",
    )? as u16;
    let y = integer(&obj["y"], 0, 63, "object.y")? as u16;
    let (value, index) = match kind {
        "type2" => (
            0xfc | (x >> 4) | ((y >> 2) << 8) | ((x & 15) << 12),
            integer(&obj["id"], 0, 63, "object.id")? as u16 | ((y & 3) << 6),
        ),
        "type1" => (
            (x << 2)
                | (y << 10)
                | integer(&obj["width_bits"], 0, 3, "object.width_bits")? as u16
                | ((integer(&obj["height_bits"], 0, 3, "object.height_bits")? as u16) << 8),
            integer(&obj["id"], 0, 247, "object.id")? as u16,
        ),
        "type3" => {
            let id = integer(&obj["id"], 0, 127, "object.id")? as u16;
            (
                (x << 2) | (y << 10) | (id & 3) | (((id >> 2) & 3) << 8),
                0xf8 | (id >> 4),
            )
        }
        _ => return Err(format!("unknown object kind: {kind}").into()),
    };
    ensure(
        value != 0xffff && value != 0xfff0,
        "object encodes a stream terminator",
    )?;
    Ok(vec![value as u8, (value >> 8) as u8, index as u8])
}

pub fn decode_layout(data: &[u8], start: usize) -> Result<(Value, usize)> {
    let prefix = take(data, start, 2)?;
    let mut pos = start + 2;
    let mut layers = Vec::new();
    for _ in 0..3 {
        let mut objects = Vec::new();
        while ![0xffff, 0xfff0].contains(&word(data, pos)?) {
            objects.push(decode_object(take(data, pos, 3)?)?);
            pos += 3;
        }
        let mut doors = Value::Null;
        if word(data, pos)? == 0xfff0 {
            let mut values = Vec::new();
            pos += 2;
            while word(data, pos)? != 0xffff {
                values.push(word(data, pos)?);
                pos += 2;
            }
            doors = json!(values);
        }
        pos += 2;
        layers.push(json!({"objects":objects, "doors_raw":doors}));
    }
    Ok((json!({"prefix_raw":prefix, "layers":layers}), pos))
}

pub fn encode_layout(layout: &Value) -> Result<Vec<u8>> {
    keys(layout, &["prefix_raw", "layers"], "layout")?;
    let prefix = array(&layout["prefix_raw"], "layout.prefix_raw")?;
    ensure(prefix.len() == 2, "layout.prefix_raw must have two bytes")?;
    let mut result = Vec::new();
    for b in prefix {
        result.push(integer(b, 0, 255, "layout prefix")? as u8);
    }
    let layers = array(&layout["layers"], "layout.layers")?;
    ensure(layers.len() == 3, "layout must have three layers")?;
    for layer in layers {
        keys(layer, &["objects", "doors_raw"], "layer")?;
        for obj in array(&layer["objects"], "layer.objects")? {
            result.extend(encode_object(obj)?);
        }
        if !layer["doors_raw"].is_null() {
            result.extend([0xf0, 0xff]);
            for door in array(&layer["doors_raw"], "doors_raw")? {
                result.extend((integer(door, 0, 65534, "door")? as u16).to_le_bytes());
            }
        }
        result.extend([0xff, 0xff]);
    }
    Ok(result)
}

pub fn decode_sprites(data: &[u8], start: usize) -> Result<(Value, usize)> {
    let sorting = take(data, start, 1)?[0];
    let mut pos = start + 1;
    let mut records = Vec::new();
    while take(data, pos, 1)?[0] != 255 {
        let raw = take(data, pos, 3)?;
        let (y, x, id) = (raw[0], raw[1], raw[2]);
        records.push(if id == 0xe4 && [0xfd, 0xfe].contains(&y) {
            json!({"control_raw":raw})
        } else {
            json!({"id":id, "x":x & 31, "y":y & 31, "x_flags":x & 0xe0, "y_flags":y & 0xe0})
        });
        pos += 3;
    }
    Ok((json!({"sorting_raw":sorting, "records":records}), pos + 1))
}

pub fn encode_sprites(sprites: &Value) -> Result<Vec<u8>> {
    keys(sprites, &["sorting_raw", "records"], "sprites")?;
    let mut out = vec![integer(&sprites["sorting_raw"], 0, 255, "sprite sorting")? as u8];
    for record in array(&sprites["records"], "sprite records")? {
        if record.get("control_raw").is_some() {
            keys(record, &["control_raw"], "sprite control")?;
            let raw = array(&record["control_raw"], "sprite control")?;
            ensure(raw.len() == 3, "sprite control must have three bytes")?;
            let bytes: Vec<u8> = raw
                .iter()
                .map(|b| integer(b, 0, 255, "sprite control byte").map(|b| b as u8))
                .collect::<Result<_>>()?;
            ensure(
                [0xfd, 0xfe].contains(&bytes[0]) && bytes[2] == 0xe4,
                "invalid sprite control record",
            )?;
            out.extend(bytes);
            continue;
        }
        keys(record, &["id", "x", "y", "x_flags", "y_flags"], "sprite")?;
        let x = integer(&record["x"], 0, 31, "sprite.x")? as u8;
        let y = integer(&record["y"], 0, 31, "sprite.y")? as u8;
        let xf = integer(&record["x_flags"], 0, 224, "sprite.x_flags")? as u8;
        let yf = integer(&record["y_flags"], 0, 224, "sprite.y_flags")? as u8;
        ensure(
            xf.is_multiple_of(32) && yf.is_multiple_of(32),
            "sprite flags must occupy only the upper three bits",
        )?;
        let id = integer(&record["id"], 0, 255, "sprite.id")? as u8;
        ensure(
            y | yf != 255 && !(id == 0xe4 && [0xfd, 0xfe].contains(&(y | yf))),
            "sprite encodes a terminator or control record",
        )?;
        out.extend([y | yf, x | xf, id]);
    }
    out.push(255);
    Ok(out)
}

pub fn export_room(pack: &Pack, room: usize) -> Result<Value> {
    ensure(room < ROOM_COUNT, "room_id out of range")?;
    Ok(
        json!({"format":FORMAT, "base_pack_sha256":pack.hash(), "room_id":room,
        "header_raw":take(pack.asset(6)?, pack.offset(7, room)?, 14)?,
        "layout":decode_layout(pack.asset(3)?, pack.offset(4, room)?)?.0,
        "sprites":decode_sprites(pack.asset(58)?, pack.offset(59, room)?)?.0,
        "entrances":pack.entrances()?.into_iter().filter(|e| e["room"] == room).collect::<Vec<_>>() }),
    )
}

fn check_scope(original: &Value, source: &Value) -> Result<()> {
    keys(
        source,
        &[
            "format",
            "base_pack_sha256",
            "room_id",
            "header_raw",
            "layout",
            "sprites",
            "entrances",
        ],
        "room",
    )?;
    // Remove only the fields whose edits are supported, then compare everything
    // else (including record counts/order, opaque data and unknown fields).
    let mut normalized = source.clone();
    for (old, new) in array(&original["layout"]["layers"], "layers")?.iter().zip(
        normalized["layout"]["layers"]
            .as_array_mut()
            .ok_or("layers")?,
    ) {
        let a = array(&old["objects"], "objects")?;
        let b = new["objects"].as_array_mut().ok_or("objects")?;
        ensure(a.len() == b.len(), "v1 cannot add or remove objects")?;
        for (a, b) in a.iter().zip(b) {
            unchanged(
                &without(a, &["x", "y"])?,
                &without(b, &["x", "y"])?,
                "object identity/size",
            )?;
            b["x"] = a["x"].clone();
            b["y"] = a["y"].clone();
        }
    }
    let a = array(&original["sprites"]["records"], "sprites")?;
    let b = normalized["sprites"]["records"]
        .as_array_mut()
        .ok_or("sprites")?;
    ensure(a.len() == b.len(), "v1 cannot add or remove sprite records")?;
    for (a, b) in a.iter().zip(b) {
        unchanged(
            &without(a, &["x", "y"])?,
            &without(b, &["x", "y"])?,
            "sprite identity/flags",
        )?;
        if a.get("x").is_some() {
            b["x"] = a["x"].clone();
            b["y"] = a["y"].clone();
        }
    }
    let a = array(&original["entrances"], "entrances")?;
    let b = normalized["entrances"]
        .as_array_mut()
        .ok_or("entrances must be an array")?;
    ensure(a.len() == b.len(), "v1 preserves entrance record count")?;
    for (a, b) in a.iter().zip(b) {
        unchanged(
            &without(a, &["player_x", "player_y"])?,
            &without(b, &["player_x", "player_y"])?,
            "entrance identity/camera/settings",
        )?;
        for field in ["player_x", "player_y"] {
            let value = integer(&b[field], 0, 65535, field)?;
            ensure(
                value >> 9 == integer(&a[field], 0, 65535, field)? >> 9,
                format!("{field}: spawn must stay within the original room's 512px bounds"),
            )?;
            b[field] = a[field].clone();
        }
    }
    unchanged(original, &normalized, "room settings/structure")
}

pub fn compile_room(pack: &Pack, source: &Value) -> Result<(Vec<u8>, Value)> {
    ensure(source["format"] == FORMAT, format!("expected {FORMAT}"))?;
    ensure(
        source["base_pack_sha256"] == pack.hash(),
        "base pack SHA-256 mismatch; export again from this exact pack",
    )?;
    let room = integer(&source["room_id"], 0, ROOM_COUNT as i64 - 1, "room_id")? as usize;
    let layout = encode_layout(&source["layout"])?;
    let sprites = encode_sprites(&source["sprites"])?;
    check_scope(&export_room(pack, room)?, source)?;
    let mut result = pack.data.clone();
    let mut changes = Vec::new();
    let mut patch = |asset: usize, start: usize, payload: &[u8], label: &str| -> Result<()> {
        let old = take(pack.asset(asset)?, start, payload.len())?;
        let differences: Vec<_> = old
            .iter()
            .zip(payload)
            .enumerate()
            .filter_map(|(i, (a, b))| (a != b).then_some(start + i))
            .collect();
        if differences.is_empty() {
            return Ok(());
        }
        if asset == 3 || asset == 58 {
            for other in 0..ROOM_COUNT {
                if other == room {
                    continue;
                }
                let begin = pack.offset(if asset == 3 { 4 } else { 59 }, other)?;
                let end = if asset == 3 {
                    decode_layout(pack.asset(asset)?, begin)?.1
                } else {
                    decode_sprites(pack.asset(asset)?, begin)?.1
                };
                ensure(!differences.iter().any(|d| (begin..end).contains(d)), format!("{label} shares edited bytes with room 0x{other:03x}; relocation is not supported"))?;
                if asset == 3 {
                    let begin = pack.offset(5, other)?;
                    let mut end = begin;
                    while word(pack.asset(3)?, end)? != 0xffff {
                        end += 2;
                    }
                    ensure(
                        !differences.iter().any(|d| (begin..end + 2).contains(d)),
                        format!("{label} overlaps room 0x{other:03x} door data"),
                    )?;
                }
            }
        }
        let absolute = pack.range(asset)?.start + start;
        result[absolute..absolute + payload.len()].copy_from_slice(payload);
        changes.push(
            json!({"asset":pack.names[asset], "asset_index":asset, "offset":start,
            "length":payload.len(), "changed_bytes":differences.len(), "label":label}),
        );
        Ok(())
    };
    patch(3, pack.offset(4, room)?, &layout, "room layout")?;
    patch(58, pack.offset(59, room)?, &sprites, "room sprites")?;
    // All other entrance bytes were compared above. Patch only the two editable
    // coordinate tables, retaining full-table receipt ranges for compatibility.
    for (suffix, field) in [("playerX", "player_x"), ("playerY", "player_y")] {
        let index = pack.named_index(&format!("kEntranceData_{suffix}"))?;
        let mut payload = pack.asset(index)?.to_vec();
        for record in array(&source["entrances"], "entrances")? {
            let at = integer(
                &record["index"],
                0,
                (payload.len() / 2) as i64 - 1,
                "entrance index",
            )? as usize
                * 2;
            payload[at..at + 2]
                .copy_from_slice(&(integer(&record[field], 0, 65535, field)? as u16).to_le_bytes());
        }
        patch(index, 0, &payload, "entrances")?;
    }
    let receipt = json!({"format":"zelda3_room_build_receipt_v1", "room_id":room,
        "base_pack_sha256":pack.hash(), "output_pack_sha256":sha256(&result),
        "byte_identical":result == pack.data, "changes":changes, "runtime_parity_verified":false});
    Ok((result, receipt))
}
