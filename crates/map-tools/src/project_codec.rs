//! Compatibility codecs for the portable model. Only this tools layer knows
//! the original encodings; zelda3-map itself contains domain data alone.
use crate::{
    ensure,
    room::{self, take, word},
    Result,
};
use serde_json::{json, Value};
use zelda3_map::*;

pub fn door(raw: u16) -> Door {
    Door {
        direction: [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ][(raw & 3) as usize],
        slot: ((raw >> 4) & 15) as u8,
        door_type: (raw >> 8) as u8,
        reserved_bits: ((raw >> 2) & 3) as u8,
    }
}
pub fn door_word(d: &Door) -> Result<u16> {
    ensure(
        d.slot < 16 && d.reserved_bits < 4,
        "door slot/reserved bits out of range",
    )?;
    let dir = match d.direction {
        Direction::North => 0,
        Direction::South => 1,
        Direction::West => 2,
        Direction::East => 3,
    };
    let raw = ((d.door_type as u16) << 8)
        | ((d.slot as u16) << 4)
        | ((d.reserved_bits as u16) << 2)
        | dir;
    ensure(raw != 0xffff, "door encodes terminator")?;
    Ok(raw)
}
pub fn decode_doors(data: &[u8], start: usize) -> Result<(Vec<Door>, usize)> {
    let mut pos = start;
    let mut doors = Vec::new();
    while word(data, pos)? != 0xffff {
        doors.push(door(word(data, pos)?));
        pos += 2;
    }
    Ok((doors, pos + 2))
}
pub fn encode_doors(doors: &[Door]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for d in doors {
        out.extend(door_word(d)?.to_le_bytes());
    }
    out.extend([255, 255]);
    Ok(out)
}
pub fn decode_pass(data: &[u8], start: usize) -> Result<(ObjectPass, usize)> {
    let mut pos = start;
    let mut objects = Vec::new();
    while ![0xffff, 0xfff0].contains(&word(data, pos)?) {
        objects.push(serde_json::from_value(room::decode_object(take(
            data, pos, 3,
        )?)?)?);
        pos += 3;
    }
    let doors = if word(data, pos)? == 0xfff0 {
        let (doors, end) = decode_doors(data, pos + 2)?;
        pos = end;
        Some(doors)
    } else {
        pos += 2;
        None
    };
    Ok((ObjectPass { objects, doors }, pos))
}
pub fn encode_pass(pass: &ObjectPass) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for obj in &pass.objects {
        out.extend(room::encode_object(&serde_json::to_value(obj)?)?);
    }
    if let Some(doors) = &pass.doors {
        out.extend([0xf0, 0xff]);
        out.extend(encode_doors(doors)?);
    } else {
        out.extend([255, 255]);
    }
    Ok(out)
}
pub fn decode_program(data: &[u8], start: usize) -> Result<(RoomProgram, usize)> {
    let prefix = take(data, start, 2)?;
    let (a, pos) = decode_pass(data, start + 2)?;
    let (b, pos) = decode_pass(data, pos)?;
    let (c, pos) = decode_pass(data, pos)?;
    Ok((
        RoomProgram {
            floor_low: prefix[0] & 15,
            floor_high: prefix[0] >> 4,
            default_layout: prefix[1] >> 2,
            starting_quadrant: prefix[1] & 3,
            passes: [a, b, c],
        },
        pos,
    ))
}
pub fn encode_program(program: &RoomProgram) -> Result<Vec<u8>> {
    ensure(
        program.floor_low < 16
            && program.floor_high < 16
            && program.default_layout < 64
            && program.starting_quadrant < 4,
        "room prefix fields out of range",
    )?;
    let mut out = vec![
        program.floor_low | (program.floor_high << 4),
        (program.default_layout << 2) | program.starting_quadrant,
    ];
    for pass in &program.passes {
        out.extend(encode_pass(pass)?);
    }
    Ok(out)
}
pub fn decode_header(data: &[u8], start: usize) -> Result<RoomSettings> {
    let h = take(data, start, 14)?;
    Ok(RoomSettings {
        background_mode: h[0] >> 5,
        collision_mode: (h[0] >> 2) & 7,
        dark: h[0] & 1 != 0,
        reserved_flag: h[0] & 2 != 0,
        palette: h[1],
        tile_theme: h[2],
        sprite_graphics: h[3],
        collision_effect: h[4],
        tags: [h[5], h[6]],
        travel_planes: [
            h[7] & 3,
            (h[7] >> 2) & 3,
            (h[7] >> 4) & 3,
            h[7] >> 6,
            h[8] & 3,
        ],
        travel_plane_reserved: h[8] >> 2,
        travel_destinations: [h[9], h[10], h[11], h[12], h[13]],
    })
}
pub fn encode_header(h: &RoomSettings) -> Result<Vec<u8>> {
    ensure(
        h.background_mode < 8
            && h.collision_mode < 8
            && h.travel_planes.iter().all(|p| *p < 4)
            && h.travel_plane_reserved < 64,
        "header fields out of range",
    )?;
    let mut out = vec![
        (h.background_mode << 5)
            | (h.collision_mode << 2)
            | ((h.reserved_flag as u8) << 1)
            | h.dark as u8,
        h.palette,
        h.tile_theme,
        h.sprite_graphics,
        h.collision_effect,
        h.tags[0],
        h.tags[1],
        h.travel_planes[0]
            | (h.travel_planes[1] << 2)
            | (h.travel_planes[2] << 4)
            | (h.travel_planes[3] << 6),
        h.travel_planes[4] | (h.travel_plane_reserved << 2),
    ];
    out.extend(h.travel_destinations);
    Ok(out)
}
pub fn decode_actors(data: &[u8], start: usize) -> Result<(ActorProgram, usize)> {
    let (source, end) = room::decode_sprites(data, start)?;
    let mut records = Vec::new();
    for record in source["records"].as_array().ok_or("actors")? {
        if let Some(raw) = record.get("control_raw") {
            records.push(ActorRecord::Control {
                command: raw[0].as_u64().ok_or("command")? as u8,
                argument: raw[1].as_u64().ok_or("argument")? as u8,
            });
        } else {
            let mut value = record.clone();
            value["kind"] = json!("actor");
            records.push(serde_json::from_value(value)?);
        }
    }
    Ok((
        ActorProgram {
            sorting: source["sorting_raw"].as_u64().ok_or("sorting")? as u8,
            records,
        },
        end,
    ))
}
pub fn encode_actors(program: &ActorProgram) -> Result<Vec<u8>> {
    let records: Vec<Value> = program
        .records
        .iter()
        .map(|record| match record {
            ActorRecord::Control { command, argument } => {
                json!({"control_raw":[command,argument,0xe4]})
            }
            ActorRecord::Actor {
                id,
                x,
                y,
                x_flags,
                y_flags,
            } => json!({"id":id,"x":x,"y":y,"x_flags":x_flags,"y_flags":y_flags}),
        })
        .collect();
    room::encode_sprites(&json!({"sorting_raw":program.sorting,"records":records}))
}
pub fn decode_secrets(data: &[u8], start: usize) -> Result<(Vec<Secret>, usize)> {
    let mut pos = start;
    let mut records = Vec::new();
    while word(data, pos)? != 0xffff {
        records.push(Secret {
            position_word: word(data, pos)?,
            item: take(data, pos + 2, 1)?[0],
        });
        pos += 3;
    }
    Ok((records, pos + 2))
}
pub fn encode_secrets(records: &[Secret]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for record in records {
        ensure(
            record.position_word < 0x8000,
            "secret position has reserved high bit",
        )?;
        out.extend(record.position_word.to_le_bytes());
        out.push(record.item);
    }
    out.extend([255, 255]);
    Ok(out)
}
pub fn words(data: &[u8]) -> Result<Vec<u16>> {
    ensure(data.len() % 2 == 0, "word table has odd length")?;
    (0..data.len()).step_by(2).map(|i| word(data, i)).collect()
}
pub fn word_bytes(words: impl IntoIterator<Item = u16>) -> Vec<u8> {
    words.into_iter().flat_map(u16::to_le_bytes).collect()
}

pub fn decode_overlay(data: &[u8], start: usize) -> Result<(Vec<OverlayObject>, usize)> {
    let mut pos = start;
    let mut out = Vec::new();
    while word(data, pos)? != 0xffff {
        let raw = take(data, pos, 3)?;
        out.push(OverlayObject {
            x: raw[0] >> 2,
            y: raw[1] >> 2,
            command: raw[2],
            unused_x_bits: raw[0] & 3,
            unused_y_bits: raw[1] & 3,
        });
        pos += 3;
    }
    Ok((out, pos + 2))
}
pub fn encode_overlay(objects: &[OverlayObject]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for o in objects {
        ensure(
            o.x < 64 && o.y < 64 && o.unused_x_bits < 4 && o.unused_y_bits < 4,
            "overlay fields out of range",
        )?;
        let x = (o.x << 2) | o.unused_x_bits;
        let y = (o.y << 2) | o.unused_y_bits;
        ensure(x != 255 || y != 255, "overlay encodes terminator")?;
        out.extend([x, y, o.command]);
    }
    out.extend([255, 255]);
    Ok(out)
}
