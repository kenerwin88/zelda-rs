//! Standard Tiled JSON points; opaque data is preserved by the exact base pack.
use crate::{
    array, ensure, integer, json_bytes, read_json,
    room::{self, Pack, ROOM_COUNT},
    unchanged, without, write_new, Result,
};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

pub const FORMAT: &str = "zelda3_tiled_dungeon_v1";
pub const PREVIEW_IMAGES: &[(&str, &str, &str)] = &[
    ("artwork", "room.png", "Room artwork (generated)"),
    (
        "collision_bg2",
        "collision-bg2.png",
        "Collision BG2 (generated)",
    ),
    (
        "collision_bg1",
        "collision-bg1.png",
        "Collision BG1 (generated)",
    ),
];

/// Add derived editor layers without changing the authored runtime objects.
/// IDs 7..9 are reserved for these references; ordinary exports retain six layers.
pub fn with_preview_layers(map: &Value, compiled_hash: &str) -> Result<Value> {
    let props = property_values(map)?;
    let mut out = map.clone();
    let layers = out["layers"].as_array_mut().ok_or("map layers")?;
    layers.retain(|l| l["id"].as_u64().is_some_and(|id| id <= 6));
    for (i, &(role, image, name)) in PREVIEW_IMAGES.iter().enumerate().rev() {
        layers.insert(0,json!({"id":7+i,"type":"imagelayer","name":name,"image":image,
            "imagewidth":512,"imageheight":512,"x":0,"y":0,"opacity":1,"visible":i==0,"locked":true,
            "properties":properties(json!({"preview_role":role,"room_id":props["zelda.room_id"],
                "base_pack_sha256":props["zelda.base_pack_sha256"],"compiled_pack_sha256":compiled_hash}))}));
    }
    out["nextlayerid"] = json!(10);
    Ok(out)
}

fn validate_preview_layer(layer: &Value, id: i64, room: usize, pack: &Pack) -> Result<()> {
    let (role, image, _) = PREVIEW_IMAGES[(id - 7) as usize];
    unchanged(&layer["type"], &json!("imagelayer"), "preview layer type")?;
    unchanged(&layer["image"], &json!(image), "preview image")?;
    for field in ["x", "y", "offsetx", "offsety"] {
        numeric_default(layer, field, 0)?;
    }
    for field in ["parallaxx", "parallaxy"] {
        numeric_default(layer, field, 1)?;
    }
    for field in ["imagewidth", "imageheight"] {
        numeric_default(layer, field, 512)?;
    }
    for field in ["repeatx", "repeaty"] {
        ensure(
            layer.get(field).map_or(true, |v| v == false),
            "preview images cannot repeat",
        )?;
    }
    for field in ["objects", "data", "chunks", "layers"] {
        ensure(
            layer.get(field).is_none(),
            "preview layer cannot contain runtime content",
        )?;
    }
    let props = property_values(layer)?;
    let hash = props["zelda.compiled_pack_sha256"]
        .as_str()
        .ok_or("preview compiled hash")?;
    ensure(
        hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid preview compiled hash",
    )?;
    unchanged(
        &props,
        &property_values(&json!({"properties":properties(json!({"preview_role":role,
        "room_id":room,"base_pack_sha256":pack.hash(),"compiled_pack_sha256":hash}))}))?,
        "preview properties",
    )
}
const LAYERS: &[(&str, &str)] = &[
    ("Room objects — pass 1 (BG1)", "#81a7ff"),
    ("Room objects — pass 2 (BG2)", "#aa8cff"),
    ("Room objects — pass 3 (BG1)", "#65d6b4"),
    ("Actors", "#ffac66"),
    ("Entrances", "#ffe066"),
    ("Doors (reference)", "#ff7991"),
];
// Engine DOOR_POSITION_UP/DOWN/LEFT/RIGHT byte addresses, divided by two
// before drawing into a 64-word-wide tilemap. These are reference anchors.
const DOOR_ANCHORS: [[usize; 12]; 4] = [
    [
        0x21c, 0x23c, 0x25c, 0x39c, 0x3bc, 0x3dc, 0x121c, 0x123c, 0x125c, 0x139c, 0x13bc, 0x13dc,
    ],
    [
        0xd1c, 0xd3c, 0xd5c, 0xb9c, 0xbbc, 0xbdc, 0x1d1c, 0x1d3c, 0x1d5c, 0x1b9c, 0x1bbc, 0x1bdc,
    ],
    [
        0x784, 0xf84, 0x1784, 0x78a, 0xf8a, 0x178a, 0x7c4, 0xfc4, 0x17c4, 0x7ca, 0xfca, 0x17ca,
    ],
    [
        0x7b4, 0xfb4, 0x17b4, 0x7ae, 0xfae, 0x17ae, 0x7f4, 0xff4, 0x17f4, 0x7ee, 0xfee, 0x17ee,
    ],
];

fn properties(values: Value) -> Value {
    json!(values
        .as_object()
        .expect("internal property object")
        .iter()
        .map(|(k, v)| json!({
            "name":format!("zelda.{k}"), "type":if v.is_i64() { "int" } else { "string" }, "value":v
        }))
        .collect::<Vec<_>>())
}

pub fn property_values(owner: &Value) -> Result<Value> {
    let mut result = Map::new();
    let mut seen = HashSet::new();
    if let Some(props) = owner.get("properties") {
        for prop in array(props, "properties")? {
            let name = prop["name"].as_str().ok_or("invalid custom property")?;
            ensure(seen.insert(name), format!("duplicate property {name}"))?;
            if name.starts_with("zelda.") {
                result.insert(name.to_owned(), prop["value"].clone());
            }
        }
    }
    Ok(Value::Object(result))
}

fn object_label(obj: &Value) -> String {
    if obj["kind"] == "type3" {
        let name = match obj["id"].as_i64() {
            Some(0x19) => Some("Chest"),
            Some(0x2f) => Some("Pot"),
            Some(0x2d) => Some("Agahnim altar"),
            Some(0x2e) => Some("Agahnim windows"),
            _ => None,
        };
        if let Some(name) = name {
            return name.to_owned();
        }
    }
    format!(
        "{} object {:02X}",
        obj["kind"].as_str().unwrap_or(""),
        obj["id"].as_i64().unwrap_or(0)
    )
}

fn point(id: usize, name: String, kind: &str, x: i64, y: i64, props: Value) -> Value {
    json!({"id":id, "name":name, "type":kind, "x":x, "y":y, "width":0, "height":0,
        "rotation":0, "visible":true, "point":true, "properties":properties(props)})
}

// Only used with values just decoded from validated binary streams.
fn n(value: &Value) -> i64 {
    value.as_i64().expect("decoded integer")
}

pub fn export_map(pack: &Pack, room_id: usize) -> Result<Value> {
    let room = room::export_room(pack, room_id)?;
    let mut layers: Vec<_> = LAYERS.iter().enumerate().map(|(i,(name,color))| json!({
        "id":i+1, "name":name, "type":"objectgroup", "draworder":"index", "color":color,
        "opacity":1, "visible":true, "x":0, "y":0, "objects":[], "properties":properties(json!({"layer_id":i+1}))
    })).collect();
    let mut id = 1;
    let mut push = |layer: usize, name: String, kind: &str, x: i64, y: i64, props: Value| {
        layers[layer]["objects"]
            .as_array_mut()
            .expect("internal object layer")
            .push(point(id, name, kind, x, y, props));
        id += 1;
    };
    for (pass, layer) in array(&room["layout"]["layers"], "layers")?
        .iter()
        .enumerate()
    {
        for (order, obj) in array(&layer["objects"], "objects")?.iter().enumerate() {
            let mut props = without(obj, &["x", "y"])?;
            props["order"] = json!(order);
            push(
                pass,
                object_label(obj),
                "DungeonObject",
                n(&obj["x"]) * 8,
                n(&obj["y"]) * 8,
                props,
            );
        }
    }
    for (order, actor) in array(&room["sprites"]["records"], "sprites")?
        .iter()
        .enumerate()
    {
        if actor.get("control_raw").is_some() {
            continue;
        }
        let mut props = without(actor, &["x", "y"])?;
        props["order"] = json!(order);
        let name = if actor["x_flags"] == 0xe0 && actor["id"] != 0xe4 {
            "Overlord"
        } else {
            "Actor"
        };
        push(
            3,
            format!("{name} {:02X}", n(&actor["id"])),
            "DungeonActor",
            n(&actor["x"]) * 16,
            n(&actor["y"]) * 16,
            props,
        );
    }
    for entrance in array(&room["entrances"], "entrances")? {
        push(
            4,
            format!("Entrance {:02X}", n(&entrance["index"])),
            "DungeonEntrance",
            n(&entrance["player_x"]) % 512,
            n(&entrance["player_y"]) % 512,
            json!({"index":entrance["index"]}),
        );
    }
    for (pass, layer) in array(&room["layout"]["layers"], "layers")?
        .iter()
        .enumerate()
    {
        if let Some(doors) = layer["doors_raw"].as_array() {
            for (order, door) in doors.iter().enumerate() {
                let raw = n(door) as usize;
                let (direction, slot, ty) = (raw & 3, (raw >> 4) & 15, raw >> 8);
                if slot >= 12 {
                    continue;
                }
                let anchor = DOOR_ANCHORS[direction][slot] / 2;
                push(
                    5,
                    format!(
                        "{} door {ty:02X}",
                        ["North", "South", "West", "East"][direction]
                    ),
                    "DungeonDoor",
                    (anchor % 64 * 8) as i64,
                    (anchor / 64 * 8) as i64,
                    json!({"direction":(["north","south","west","east"][direction]),
                        "slot":slot, "door_type":ty, "pass":pass+1, "order":order}),
                );
            }
        }
    }
    layers[5]["locked"] = json!(true);
    let h = &room["header_raw"];
    let prefix = &room["layout"]["prefix_raw"];
    Ok(
        json!({"type":"map", "version":"1.10", "orientation":"orthogonal", "renderorder":"right-down",
        "infinite":false, "width":64, "height":64, "tilewidth":8, "tileheight":8,
        "nextlayerid":7, "nextobjectid":id, "backgroundcolor":"#202630", "tilesets":[], "layers":layers,
        "properties":properties(json!({"format":FORMAT, "base_pack_sha256":pack.hash(), "room_id":room_id,
            "default_layout":n(&prefix[1])>>2, "bg1_floor":n(&prefix[0])&15, "bg2_floor":n(&prefix[0])>>4,
            "bg2_mode":n(&h[0])>>5, "collision_mode":(n(&h[0])>>2)&7, "lights_out":n(&h[0])&1,
            "palette":h[1], "tile_theme":h[2], "sprite_graphics":h[3], "collision_effect":h[4],
            "primary_tag":h[5], "secondary_tag":h[6]}))}),
    )
}

fn index_by_id<'a>(items: &'a Value, label: &str) -> Result<BTreeMap<i64, &'a Value>> {
    let mut out = BTreeMap::new();
    for item in array(items, label)? {
        let id = integer(&item["id"], 1, i32::MAX as i64, label)?;
        ensure(
            out.insert(id, item).is_none(),
            format!("duplicate {label} id {id}"),
        )?;
    }
    Ok(out)
}

fn coordinate(value: &Value, grid: i64, label: &str) -> Result<i64> {
    let number = value
        .as_f64()
        .ok_or_else(|| format!("{label}: expected numeric coordinate"))?;
    // All room-local coordinates fit this bound. Check before the float-to-int
    // conversion so huge or non-finite JSON numbers cannot saturate silently.
    ensure(
        number.is_finite() && (0.0..512.0).contains(&number) && number % grid as f64 == 0.0,
        format!("{label}: coordinate must be within the room and snap to the {grid}-pixel grid"),
    )?;
    Ok(number as i64 / grid)
}

fn numeric_default(owner: &Value, field: &str, expected: i64) -> Result<()> {
    ensure(
        owner
            .get(field)
            .map_or(true, |v| v.as_f64() == Some(expected as f64)),
        format!("{field}: unsupported transform"),
    )
}

pub fn import_map(pack: &Pack, tiled: &Value) -> Result<Value> {
    ensure(tiled.is_object(), "map must be an object")?;
    let props = property_values(tiled)?;
    unchanged(&props["zelda.format"], &json!(FORMAT), "format")?;
    unchanged(
        &props["zelda.base_pack_sha256"],
        &json!(pack.hash()),
        "base pack SHA-256",
    )?;
    let room_id = integer(&props["zelda.room_id"], 0, ROOM_COUNT as i64 - 1, "room id")? as usize;
    let expected = export_map(pack, room_id)?;
    unchanged(&props, &property_values(&expected)?, "map properties")?;
    for field in [
        "type",
        "orientation",
        "infinite",
        "width",
        "height",
        "tilewidth",
        "tileheight",
        "tilesets",
    ] {
        unchanged(&tiled[field], &expected[field], field)?;
    }
    let mut actual_layers = index_by_id(&tiled["layers"], "layer")?;
    for id in 7..=9 {
        if let Some(layer) = actual_layers.remove(&id) {
            validate_preview_layer(layer, id, room_id, pack)?;
        }
    }
    let expected_layers = index_by_id(&expected["layers"], "layer")?;
    ensure(
        actual_layers.keys().eq(expected_layers.keys()),
        "layer IDs changed",
    )?;
    let mut room = room::export_room(pack, room_id)?;
    let mut seen = HashSet::new();
    for (layer_id, layer) in actual_layers {
        let original = expected_layers[&layer_id];
        unchanged(&layer["type"], &json!("objectgroup"), "layer type")?;
        unchanged(
            &property_values(layer)?,
            &property_values(original)?,
            "layer properties",
        )?;
        for field in ["x", "y", "offsetx", "offsety"] {
            numeric_default(layer, field, 0)?;
        }
        for field in ["parallaxx", "parallaxy"] {
            numeric_default(layer, field, 1)?;
        }
        let objects = index_by_id(&layer["objects"], "object")?;
        let expected_objects = index_by_id(&original["objects"], "object")?;
        ensure(
            objects.keys().eq(expected_objects.keys()),
            "object IDs/layer membership changed",
        )?;
        for (id, obj) in objects {
            ensure(seen.insert(id), format!("duplicate global object id {id}"))?;
            let baseline = expected_objects[&id];
            let metadata = property_values(obj)?;
            unchanged(&metadata, &property_values(baseline)?, "object properties")?;
            unchanged(
                obj.get("type")
                    .or_else(|| obj.get("class"))
                    .unwrap_or(&Value::Null),
                &baseline["type"],
                "object type",
            )?;
            if let Some(class) = obj.get("class") {
                unchanged(class, &baseline["type"], "object class")?;
            }
            unchanged(&obj["point"], &json!(true), "object shape")?;
            for field in ["width", "height", "rotation"] {
                numeric_default(obj, field, 0)?;
            }
            for field in [
                "gid", "template", "polygon", "polyline", "text", "ellipse", "capsule",
            ] {
                ensure(
                    obj.get(field).is_none(),
                    format!("object {field} is not supported"),
                )?;
            }
            let grid = if layer_id <= 3 {
                8
            } else if layer_id == 4 {
                16
            } else {
                1
            };
            let x = coordinate(&obj["x"], grid, "x")?;
            let y = coordinate(&obj["y"], grid, "y")?;
            if layer_id == 6 {
                unchanged(&json!(x), &baseline["x"], "door x (reference only)")?;
                unchanged(&json!(y), &baseline["y"], "door y (reference only)")?;
            } else if layer_id <= 4 {
                let order =
                    integer(&metadata["zelda.order"], 0, i32::MAX as i64, "object order")? as usize;
                let target = if layer_id <= 3 {
                    &mut room["layout"]["layers"][layer_id as usize - 1]["objects"][order]
                } else {
                    &mut room["sprites"]["records"][order]
                };
                target["x"] = json!(x);
                target["y"] = json!(y);
            } else {
                let target = room["entrances"]
                    .as_array_mut()
                    .ok_or("entrances")?
                    .iter_mut()
                    .find(|e| e["index"] == metadata["zelda.index"])
                    .ok_or("missing entrance")?;
                target["player_x"] = json!((n(&target["player_x"]) / 512) * 512 + x);
                target["player_y"] = json!((n(&target["player_y"]) / 512) * 512 + y);
            }
        }
    }
    Ok(room)
}

pub fn compile_map(pack: &Pack, tiled: &Value) -> Result<(Vec<u8>, Value)> {
    let (data, mut receipt) = room::compile_room(pack, &import_map(pack, tiled)?)?;
    receipt["authoring_format"] = json!(FORMAT);
    let hashes: Vec<_> = array(&tiled["layers"], "layers")?
        .iter()
        .filter(|l| (7..=9).contains(&l["id"].as_i64().unwrap_or(0)))
        .map(|l| property_values(l).map(|p| p["zelda.compiled_pack_sha256"].clone()))
        .collect::<Result<_>>()?;
    if !hashes.is_empty() {
        receipt["preview_stale"] = json!(hashes
            .iter()
            .any(|hash| hash != &receipt["output_pack_sha256"]));
    }
    Ok((data, receipt))
}

pub fn export_world(pack: &Pack, out: &Path) -> Result<()> {
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::create_dir(out)?;
    let mut maps = Vec::new();
    for room in 0..ROOM_COUNT {
        let name = format!("room-{room:03x}.tmj");
        write_new(&out.join(&name), &json_bytes(&export_map(pack, room)?)?)?;
        maps.push(json!({"fileName":name, "width":512, "height":512, "x":(room%16)*512, "y":(room/16)*512}));
    }
    write_new(
        &out.join("dungeons.world"),
        &json_bytes(&json!({"type":"world", "maps":maps, "onlyShowAdjacentMaps":true}))?,
    )
}

pub fn compile_world(pack: &Pack, path: &Path) -> Result<(Vec<u8>, Value)> {
    let world = read_json(path)?;
    ensure(world["type"] == "world", "expected a Tiled world")?;
    let maps = array(&world["maps"], "world maps")?;
    ensure(
        maps.len() == ROOM_COUNT,
        "a dungeon world must contain all 320 room maps",
    )?;
    ensure(
        world
            .get("patterns")
            .map_or(true, |p| p.as_array().is_some_and(Vec::is_empty)),
        "pattern-generated world maps are not supported",
    )?;
    let root = path
        .canonicalize()?
        .parent()
        .ok_or("world directory missing")?
        .to_path_buf();
    let mut seen = HashSet::new();
    let mut result = pack.data().to_vec();
    let mut patches = BTreeMap::new();
    let mut receipts = Vec::new();
    for entry in maps {
        let source = root
            .join(
                entry["fileName"]
                    .as_str()
                    .ok_or("world map entry must have a fileName")?,
            )
            .canonicalize()?;
        ensure(
            source.starts_with(&root) && source.extension().is_some_and(|e| e == "tmj"),
            "world maps must be .tmj files inside the world directory",
        )?;
        let (data, receipt) = compile_map(pack, &read_json(&source)?)?;
        let room = integer(&receipt["room_id"], 0, ROOM_COUNT as i64 - 1, "room id")?;
        ensure(
            seen.insert(room),
            format!("duplicate room 0x{room:03x} in world"),
        )?;
        if receipt["byte_identical"] == true {
            continue;
        }
        for change in array(&receipt["changes"], "changes")? {
            let asset = n(&change["asset_index"]) as usize;
            let begin = pack.range(asset)?.start + n(&change["offset"]) as usize;
            let end = begin + n(&change["length"]) as usize;
            for offset in begin..end {
                if data[offset] == pack.data()[offset] {
                    continue;
                }
                if let Some(previous) = patches.insert(offset, data[offset]) {
                    ensure(
                        previous == data[offset],
                        format!("conflicting room edits at pack offset {offset}"),
                    )?;
                }
                result[offset] = data[offset];
            }
        }
        receipts.push(receipt);
    }
    let receipt = json!({"format":"zelda3_tiled_world_build_v1", "map_count":seen.len(),
        "base_pack_sha256":pack.hash(), "output_pack_sha256":room::sha256(&result),
        "byte_identical":result == pack.data(), "changed_bytes":patches.len(), "edited_rooms":receipts,
        "runtime_parity_verified":false});
    Ok((result, receipt))
}
