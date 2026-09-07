//! Portable project import/export. Modeled bytes are absent from pack-shell.bin:
//! every such byte is regenerated from typed sources or an explicit unused span.
use crate::{
    ensure, json_bytes, project_codec as codec, read_json,
    room::{self, Pack, ROOM_COUNT},
    write_new, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use zelda3_map::*;

const COMPAT_FORMAT: &str = "zelda3_dungeon_compatibility_v1";
fn modeled(index: usize) -> bool {
    (3..=55).contains(&index) || index == 58 || index == 59
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub asset: usize,
    pub offset: usize,
    pub length: usize,
    pub baseline_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnusedSpan {
    pub asset: usize,
    pub offset: usize,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    pub format: String,
    pub base_pack_sha256: String,
    pub shell_sha256: String,
    pub placements: BTreeMap<String, Placement>,
    pub unused_spans: Vec<UnusedSpan>,
}

pub struct Project {
    pub world: DungeonWorld,
    pub compatibility: Compatibility,
    shell: Vec<u8>,
}

fn verify_asset_names(pack: &Pack) -> Result<()> {
    let fixed = [
        (3, "kDungeonRoom"),
        (4, "kDungeonRoomOffs"),
        (5, "kDungeonRoomDoorOffs"),
        (6, "kDungeonRoomHeaders"),
        (7, "kDungeonRoomHeadersOffs"),
        (8, "kDungeonRoomChests"),
        (9, "kDungeonRoomTeleMsg"),
        (10, "kDungeonPitsHurtPlayer"),
        (46, "kDungeonRoomDefault"),
        (47, "kDungeonRoomDefaultOffs"),
        (48, "kDungeonRoomOverlay"),
        (49, "kDungeonRoomOverlayOffs"),
        (50, "kDungeonSecrets"),
        (51, "kDungAttrsForTile_Offs"),
        (52, "kDungAttrsForTile"),
        (53, "kMovableBlockDataInit"),
        (54, "kTorchDataInit"),
        (55, "kTorchDataJunk"),
        (58, "kDungeonSprites"),
        (59, "kDungeonSpriteOffs"),
    ];
    for (index, name) in fixed {
        ensure(
            pack.names().get(index).map(String::as_str) == Some(name),
            format!("asset {index} must be {name}"),
        )?;
    }
    for (n, &(suffix, _, _, _, _)) in room::ENTRANCE_FIELDS.iter().enumerate() {
        ensure(
            pack.names().get(11 + n) == Some(&format!("kEntranceData_{suffix}")),
            "entrance asset order mismatch",
        )?;
        let index = if suffix == "musicTrack" { 45 } else { 28 + n };
        ensure(
            pack.names().get(index) == Some(&format!("kStartingPoint_{suffix}")),
            "starting point asset order mismatch",
        )?;
    }
    ensure(
        pack.names().get(44).map(String::as_str) == Some("kStartingPoint_entrance"),
        "starting point entrance asset mismatch",
    )
}

fn register(
    compat: &mut Compatibility,
    pack: &Pack,
    key: &str,
    asset: usize,
    offset: usize,
    length: usize,
) -> Result<()> {
    let bytes = room::take(pack.asset(asset)?, offset, length)?;
    ensure(
        compat
            .placements
            .insert(
                key.into(),
                Placement {
                    asset,
                    offset,
                    length,
                    baseline_sha256: room::sha256(bytes),
                },
            )
            .is_none(),
        format!("duplicate source {key}"),
    )
}

fn resource<T>(
    compat: &mut Compatibility,
    pack: &Pack,
    kind: &str,
    resources: &mut BTreeMap<String, T>,
    asset: usize,
    offset: usize,
    decode: impl FnOnce(&[u8], usize) -> Result<(T, usize)>,
) -> Result<String> {
    let prefix = format!("{kind}/");
    if let Some((key, _)) = compat
        .placements
        .iter()
        .find(|(key, p)| key.starts_with(&prefix) && p.asset == asset && p.offset == offset)
    {
        return Ok(key[prefix.len()..].into());
    }
    let id = format!("resource-{:03}", resources.len());
    let (source, end) = decode(pack.asset(asset)?, offset)?;
    register(
        compat,
        pack,
        &format!("{kind}/{id}"),
        asset,
        offset,
        end - offset,
    )?;
    resources.insert(id.clone(), source);
    Ok(id)
}

fn navigation(pack: &Pack, starting: bool) -> Result<Vec<Entrance>> {
    let first = if starting { 28 } else { 11 };
    let rooms = codec::words(pack.asset(first)?)?;
    let mut records = vec![json!({}); rooms.len()];
    for (n, &(suffix, field, size, values, signed)) in room::ENTRANCE_FIELDS.iter().enumerate() {
        let index = if starting && suffix == "musicTrack" {
            45
        } else {
            first + n
        };
        let data = pack.asset(index)?;
        ensure(
            data.len() == records.len() * size * values,
            format!("{field} navigation table length mismatch"),
        )?;
        for (i, record) in records.iter_mut().enumerate() {
            let mut decoded = Vec::new();
            for j in 0..values {
                let at = (i * values + j) * size;
                decoded.push(json!(if size == 2 {
                    room::word(data, at)? as i64
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
    records
        .into_iter()
        .map(|v| Ok(serde_json::from_value(v)?))
        .collect()
}

impl Project {
    pub fn from_pack(pack: &Pack) -> Result<Self> {
        verify_asset_names(pack)?;
        let mut c = Compatibility {
            format: COMPAT_FORMAT.into(),
            base_pack_sha256: pack.hash().into(),
            shell_sha256: String::new(),
            placements: BTreeMap::new(),
            unused_spans: Vec::new(),
        };
        let mut world = DungeonWorld {
            format: FORMAT.into(),
            ..DungeonWorld::default()
        };
        for id in 0..ROOM_COUNT {
            let program = resource(
                &mut c,
                pack,
                "programs",
                &mut world.programs,
                3,
                pack.offset(4, id)?,
                codec::decode_program,
            )?;
            let header = resource(
                &mut c,
                pack,
                "headers",
                &mut world.headers,
                6,
                pack.offset(7, id)?,
                |data, at| Ok((codec::decode_header(data, at)?, at + 14)),
            )?;
            let actors = resource(
                &mut c,
                pack,
                "actors",
                &mut world.actors,
                58,
                pack.offset(59, id)?,
                codec::decode_actors,
            )?;
            let door_reference = resource(
                &mut c,
                pack,
                "doors",
                &mut world.door_references,
                3,
                pack.offset(5, id)?,
                codec::decode_doors,
            )?;
            let secrets = resource(
                &mut c,
                pack,
                "secrets",
                &mut world.secrets,
                50,
                room::word(pack.asset(50)?, id * 2)? as usize,
                codec::decode_secrets,
            )?;
            world.rooms.push(RoomDefinition {
                id: id as u16,
                header,
                program,
                actors,
                door_reference,
                secrets,
                telepathy_message: room::word(pack.asset(9)?, id * 2)?,
            });
        }
        for (asset, table, kind) in [(46, 47, "defaults"), (48, 49, "overlays")] {
            let mut ids = Vec::new();
            for at in codec::words(pack.asset(table)?)? {
                ids.push(if kind == "defaults" {
                    resource(
                        &mut c,
                        pack,
                        "lists",
                        &mut world.object_lists,
                        asset,
                        at as usize,
                        codec::decode_pass,
                    )?
                } else {
                    resource(
                        &mut c,
                        pack,
                        "overlays",
                        &mut world.overlay_programs,
                        asset,
                        at as usize,
                        codec::decode_overlay,
                    )?
                });
            }
            if kind == "defaults" {
                world.default_layouts = ids;
            } else {
                world.overlays = ids;
            }
        }
        for at in codec::words(pack.asset(51)?)? {
            world.attribute_themes.push(resource(
                &mut c,
                pack,
                "attributes",
                &mut world.attributes,
                52,
                at as usize,
                |data, at| Ok((room::take(data, at, 128)?.to_vec(), at + 128)),
            )?);
        }
        world.entrances = navigation(pack, false)?;
        let starting = navigation(pack, true)?;
        ensure(
            starting.len() == pack.asset(44)?.len(),
            "starting point count mismatch",
        )?;
        world.starting_points = starting
            .into_iter()
            .zip(pack.asset(44)?)
            .map(|(spawn, &entrance)| StartingPoint { spawn, entrance })
            .collect();
        ensure(pack.asset(8)?.len() % 3 == 0, "truncated chest table")?;
        for bytes in pack.asset(8)?.chunks_exact(3) {
            let raw = room::word(bytes, 0)?;
            world.chests.push(Chest {
                room: raw & 0x7fff,
                big: raw & 0x8000 != 0,
                item: bytes[2],
            });
        }
        world.damaging_pit_rooms = codec::words(pack.asset(10)?)?;
        ensure(
            pack.asset(53)?.len() % 4 == 0,
            "truncated movable block table",
        )?;
        for bytes in pack.asset(53)?.chunks_exact(4) {
            world.movable_blocks.push(MovableBlock {
                room: room::word(bytes, 0)?,
                tilemap_position: room::word(bytes, 2)?,
            });
        }
        let data = pack.asset(54)?;
        let mut pos = 0;
        while pos < data.len() {
            let room = room::word(data, pos)?;
            pos += 2;
            let mut positions = Vec::new();
            while room::word(data, pos)? != 0xffff {
                positions.push(room::word(data, pos)?);
                pos += 2;
            }
            pos += 2;
            world.torches.push(TorchRoom { room, positions });
        }
        world.torch_tail_words = codec::words(pack.asset(55)?)?;
        for (key, index) in [
            ("room_programs", 4),
            ("room_doors", 5),
            ("room_headers", 7),
            ("chests", 8),
            ("telepathy", 9),
            ("damaging_pits", 10),
            ("default_layouts", 47),
            ("overlays", 49),
            ("attribute_themes", 51),
            ("movable_blocks", 53),
            ("torches", 54),
            ("torch_tail", 55),
            ("room_actors", 59),
        ] {
            register(
                &mut c,
                pack,
                &format!("tables/{key}"),
                index,
                0,
                pack.asset(index)?.len(),
            )?;
        }
        register(&mut c, pack, "tables/room_secrets", 50, 0, ROOM_COUNT * 2)?;
        for i in 11..=45 {
            register(
                &mut c,
                pack,
                &format!("navigation/{}", pack.names()[i]),
                i,
                0,
                pack.asset(i)?.len(),
            )?;
        }
        // Every unused byte is explicitly separated from the modeled content.
        // The shell contains zero for ALL dungeon assets, including pointer tables.
        let mut shell = pack.data().to_vec();
        for index in (0..pack.names().len()).filter(|i| modeled(*i)) {
            let data = pack.asset(index)?;
            let mut used = vec![false; data.len()];
            for p in c.placements.values().filter(|p| p.asset == index) {
                used[p.offset..p.offset + p.length].fill(true);
            }
            let mut pos = 0;
            while pos < data.len() {
                if used[pos] {
                    pos += 1;
                    continue;
                }
                let start = pos;
                while pos < data.len() && !used[pos] {
                    pos += 1;
                }
                c.unused_spans.push(UnusedSpan {
                    asset: index,
                    offset: start,
                    bytes: data[start..pos].to_vec(),
                });
            }
            shell[pack.range(index)?].fill(0);
        }
        c.shell_sha256 = room::sha256(&shell);
        let project = Self {
            world,
            compatibility: c,
            shell,
        };
        let (rebuilt, _) = project.build()?;
        ensure(
            rebuilt == pack.data(),
            "project export did not reproduce original whole pack",
        )?;
        Ok(project)
    }

    pub fn load(root: &Path) -> Result<Self> {
        let world: DungeonWorld = serde_json::from_slice(&std::fs::read(root.join("world.json"))?)?;
        let compatibility =
            serde_json::from_slice(&std::fs::read(root.join("compatibility/manifest.json"))?)?;
        let shell = std::fs::read(root.join("compatibility/pack-shell.bin"))?;
        let project = Self {
            world,
            compatibility,
            shell,
        };
        project.build()?;
        Ok(project)
    }

    pub fn save_new(&self, root: &Path) -> Result<()> {
        self.build()?;
        if let Some(parent) = root.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir(root)?;
        write_new(
            &root.join("world.json"),
            &json_bytes(&serde_json::to_value(&self.world)?)?,
        )?;
        write_new(
            &root.join("compatibility/manifest.json"),
            &json_bytes(&serde_json::to_value(&self.compatibility)?)?,
        )?;
        write_new(&root.join("compatibility/pack-shell.bin"), &self.shell)?;
        write_new(&root.join("README.txt"),b"world.json is the canonical dungeon source. Shared resources have explicit IDs.\ncompatibility/ contains fixed storage placements and unused bytes. pack-shell.bin has all modeled dungeon assets zeroed.\nBuild with: dungeon-project build --project THIS_DIRECTORY --out NEW_PACK.dat\nNo original pack or ROM is required to build. Runtime consumption still uses the compatibility pack.\n")?;
        Ok(())
    }

    fn pointer(&self, kind: &str, id: &str, asset: usize) -> Result<u16> {
        let p = self
            .compatibility
            .placements
            .get(&format!("{kind}/{id}"))
            .ok_or_else(|| format!("missing placement for {kind}/{id}"))?;
        ensure(p.asset == asset, "resource is in the wrong legacy asset")?;
        Ok(u16::try_from(p.offset)?)
    }

    fn sources(&self) -> Result<BTreeMap<String, Vec<u8>>> {
        let w = &self.world;
        w.validate()?;
        ensure(
            w.format == FORMAT,
            format!("unsupported world format: {}", w.format),
        )?;
        ensure(
            w.rooms.len() == ROOM_COUNT,
            "world must contain exactly 320 rooms",
        )?;
        let rooms: BTreeMap<_, _> = w.rooms.iter().map(|r| (r.id, r)).collect();
        ensure(
            rooms.len() == ROOM_COUNT && rooms.keys().copied().eq(0..ROOM_COUNT as u16),
            "room IDs must contain 0..319 exactly once",
        )?;
        let mut sources = BTreeMap::new();
        for (id, h) in &w.headers {
            sources.insert(format!("headers/{id}"), codec::encode_header(h)?);
        }
        for (id, p) in &w.programs {
            ensure(
                (p.default_layout as usize) < w.default_layouts.len(),
                "room references a missing default layout",
            )?;
            sources.insert(format!("programs/{id}"), codec::encode_program(p)?);
        }
        for (id, a) in &w.actors {
            sources.insert(format!("actors/{id}"), codec::encode_actors(a)?);
        }
        for (id, d) in &w.door_references {
            sources.insert(format!("doors/{id}"), codec::encode_doors(d)?);
        }
        for (id, p) in &w.object_lists {
            sources.insert(format!("lists/{id}"), codec::encode_pass(p)?);
        }
        for (id, p) in &w.overlay_programs {
            sources.insert(format!("overlays/{id}"), codec::encode_overlay(p)?);
        }
        for (id, s) in &w.secrets {
            sources.insert(format!("secrets/{id}"), codec::encode_secrets(s)?);
        }
        for (id, a) in &w.attributes {
            ensure(a.len() == 128, "attribute resource must have 128 entries")?;
            sources.insert(format!("attributes/{id}"), a.clone());
        }
        let mut table = |name: &str, bytes: Vec<u8>| {
            sources.insert(format!("tables/{name}"), bytes);
        };
        for (name, kind, asset) in [
            ("room_programs", "programs", 3),
            ("room_headers", "headers", 6),
            ("room_actors", "actors", 58),
            ("room_doors", "doors", 3),
            ("room_secrets", "secrets", 50),
        ] {
            let mut ptrs = Vec::new();
            for r in rooms.values() {
                let id = match kind {
                    "programs" => &r.program,
                    "headers" => &r.header,
                    "actors" => &r.actors,
                    "doors" => &r.door_reference,
                    _ => &r.secrets,
                };
                ensure(
                    match kind {
                        "programs" => w.programs.contains_key(id),
                        "headers" => w.headers.contains_key(id),
                        "actors" => w.actors.contains_key(id),
                        "doors" => w.door_references.contains_key(id),
                        _ => w.secrets.contains_key(id),
                    },
                    format!("room {} has a missing {kind} resource", r.id),
                )?;
                ptrs.push(self.pointer(kind, id, asset)?);
            }
            table(name, codec::word_bytes(ptrs));
        }
        table(
            "telepathy",
            codec::word_bytes(rooms.values().map(|r| r.telepathy_message)),
        );
        for (name, kind, asset, ids) in [
            ("default_layouts", "lists", 46, &w.default_layouts),
            ("overlays", "overlays", 48, &w.overlays),
            ("attribute_themes", "attributes", 52, &w.attribute_themes),
        ] {
            let mut ptrs = Vec::new();
            for id in ids {
                ensure(
                    if kind == "lists" {
                        w.object_lists.contains_key(id)
                    } else if kind == "overlays" {
                        w.overlay_programs.contains_key(id)
                    } else {
                        w.attributes.contains_key(id)
                    },
                    format!("missing {kind} resource {id}"),
                )?;
                ptrs.push(self.pointer(kind, id, asset)?);
            }
            table(name, codec::word_bytes(ptrs));
        }
        let mut bytes = Vec::new();
        for chest in &w.chests {
            ensure(chest.room < ROOM_COUNT as u16, "chest room out of range")?;
            bytes.extend((chest.room | if chest.big { 0x8000 } else { 0 }).to_le_bytes());
            bytes.push(chest.item);
        }
        table("chests", bytes);
        ensure(
            w.damaging_pit_rooms.iter().all(|r| *r < ROOM_COUNT as u16),
            "damaging pit room out of range",
        )?;
        table(
            "damaging_pits",
            codec::word_bytes(w.damaging_pit_rooms.iter().copied()),
        );
        let mut bytes = Vec::new();
        for b in &w.movable_blocks {
            ensure(
                b.room < ROOM_COUNT as u16,
                "movable block room out of range",
            )?;
            bytes.extend(b.room.to_le_bytes());
            bytes.extend(b.tilemap_position.to_le_bytes());
        }
        table("movable_blocks", bytes);
        let mut bytes = Vec::new();
        for t in &w.torches {
            ensure(
                t.room < ROOM_COUNT as u16 || (t.room == 0xffff && t.positions.is_empty()),
                "torch room out of range",
            )?;
            ensure(
                t.positions.iter().all(|p| *p != 0xffff),
                "torch position encodes terminator",
            )?;
            bytes.extend(t.room.to_le_bytes());
            bytes.extend(codec::word_bytes(t.positions.iter().copied()));
            bytes.extend([255, 255]);
        }
        table("torches", bytes);
        table(
            "torch_tail",
            codec::word_bytes(w.torch_tail_words.iter().copied()),
        );
        for (starting, spawns) in [
            (false, w.entrances.clone()),
            (
                true,
                w.starting_points.iter().map(|p| p.spawn.clone()).collect(),
            ),
        ] {
            ensure(
                spawns.iter().all(|e| e.room < ROOM_COUNT as u16),
                "entrance room out of range",
            )?;
            for &(suffix, field, size, count, _) in room::ENTRANCE_FIELDS {
                let mut bytes = Vec::new();
                for e in &spawns {
                    let value = serde_json::to_value(e)?;
                    let values = if count == 1 {
                        vec![value[field].clone()]
                    } else {
                        value[field].as_array().ok_or("navigation array")?.clone()
                    };
                    for v in values {
                        let n = v.as_i64().ok_or("navigation integer")?;
                        if size == 2 {
                            bytes.extend((n as u16).to_le_bytes());
                        } else {
                            bytes.push(n as u8);
                        }
                    }
                }
                sources.insert(
                    format!(
                        "navigation/k{}_{suffix}",
                        if starting {
                            "StartingPoint"
                        } else {
                            "EntranceData"
                        }
                    ),
                    bytes,
                );
            }
        }
        sources.insert(
            "navigation/kStartingPoint_entrance".into(),
            w.starting_points.iter().map(|p| p.entrance).collect(),
        );
        Ok(sources)
    }

    pub fn build(&self) -> Result<(Vec<u8>, Value)> {
        let c = &self.compatibility;
        ensure(
            c.format == COMPAT_FORMAT,
            "unsupported compatibility format",
        )?;
        ensure(
            room::sha256(&self.shell) == c.shell_sha256,
            "pack shell hash mismatch",
        )?;
        let shell = Pack::parse(self.shell.clone())?;
        verify_asset_names(&shell)?;
        for i in (0..shell.names().len()).filter(|i| modeled(*i)) {
            ensure(
                shell.asset(i)?.iter().all(|b| *b == 0),
                "pack shell must not contain modeled dungeon bytes",
            )?;
        }
        let sources = self.sources()?;
        ensure(
            sources.keys().eq(c.placements.keys()),
            "source resources and compatibility placements differ",
        )?;
        let mut result = self.shell.clone();
        let mut written = vec![None; result.len()];
        let mut changed = Vec::new();
        let mut write = |asset: usize, offset: usize, bytes: &[u8], label: &str| -> Result<()> {
            ensure(
                modeled(asset),
                "compatibility placement targets an unrelated asset",
            )?;
            room::take(shell.asset(asset)?, offset, bytes.len())?;
            let start = shell.range(asset)?.start + offset;
            for (i, &byte) in bytes.iter().enumerate() {
                if let Some((previous, owner)) = &written[start + i] {
                    ensure(
                        *previous == byte,
                        format!(
                            "shared byte conflict: {label} and {owner} at asset {asset} offset {}",
                            offset + i
                        ),
                    )?;
                } else {
                    written[start + i] = Some((byte, label.to_owned()));
                }
                result[start + i] = byte;
            }
            Ok(())
        };
        for (key, bytes) in &sources {
            let p = &c.placements[key];
            ensure(bytes.len()==p.length,format!("{key}: encoded length {} differs from allocation {}; relocation is not supported",bytes.len(),p.length))?;
            if room::sha256(bytes) != p.baseline_sha256 {
                changed.push(key);
            }
            write(p.asset, p.offset, bytes, key)?;
        }
        for (i, gap) in c.unused_spans.iter().enumerate() {
            write(
                gap.asset,
                gap.offset,
                &gap.bytes,
                &format!("unused span {i}"),
            )?;
        }
        for i in (0..shell.names().len()).filter(|i| modeled(*i)) {
            ensure(
                written[shell.range(i)?].iter().all(Option::is_some),
                format!("asset {i} contains bytes with no source"),
            )?;
        }
        let hash = room::sha256(&result);
        let receipt = json!({"format":"zelda3_dungeon_project_build_v1","base_pack_sha256":c.base_pack_sha256,
            "output_pack_sha256":hash,"byte_identical":hash==c.base_pack_sha256,"changed_sources":changed,
            "world_sha256":room::sha256(&serde_json::to_vec(&self.world)?),"compatibility_export":true,
            "room_count":self.world.rooms.len(),"modeled_asset_count":55,"requires_original_pack":false,"runtime_parity_verified":false});
        Ok((result, receipt))
    }

    pub fn export_tiled(&self, out: &Path) -> Result<()> {
        let (data, _) = self.build()?;
        crate::tiled::export_world(&Pack::parse(data)?, out)
    }

    /// Apply existing position-only Tiled edits into canonical typed resources.
    /// The original compatibility manifest and its baseline hashes stay intact.
    pub fn import_tiled(&mut self, path: &Path) -> Result<()> {
        let (data, _) = self.build()?;
        let pack = Pack::parse(data)?;
        let map = read_json(path)?;
        let native = crate::tiled::import_map(&pack, &map)?;
        let (compiled, _) = room::compile_room(&pack, &native)?;
        let updated = Self::from_pack(&Pack::parse(compiled)?)?;
        let previous = std::mem::replace(&mut self.world, updated.world);
        if let Err(error) = self.build() {
            self.world = previous;
            return Err(error);
        }
        Ok(())
    }

    pub fn import_tiled_world(&mut self, path: &Path) -> Result<()> {
        let (data, _) = self.build()?;
        let (compiled, _) = crate::tiled::compile_world(&Pack::parse(data)?, path)?;
        let updated = Self::from_pack(&Pack::parse(compiled)?)?;
        let previous = std::mem::replace(&mut self.world, updated.world);
        if let Err(error) = self.build() {
            self.world = previous;
            return Err(error);
        }
        Ok(())
    }
}

pub fn main() {
    let usage="dungeon-project export --base-pack PACK --out NEW_DIR\ndungeon-project build --project DIR --out NEW_PACK.dat\ndungeon-project validate --project DIR\ndungeon-project export-tiled --project DIR --out NEW_DIR\ndungeon-project import-tiled --project DIR (--map ROOM.tmj | --world dungeons.world) --out NEW_DIR\ndungeon-project preview --project DIR --rom ROM --room ID --out NEW_DIR (requires preview feature)";
    let run = || -> Result<()> {
        let mut args = std::env::args_os().skip(1);
        let command = args.next().ok_or(usage)?;
        if command == "--help" || command == "-h" {
            println!("{usage}");
            return Ok(());
        }
        let command = command.to_str().ok_or("invalid command")?;
        let allowed: &[&str] = match command {
            "export" => &["--base-pack", "--out"],
            "build" | "export-tiled" => &["--project", "--out"],
            "validate" => &["--project"],
            "import-tiled" => &["--project", "--map", "--world", "--out"],
            "preview" => &["--project", "--rom", "--room", "--out"],
            _ => return Err(usage.into()),
        };
        let mut options = BTreeMap::new();
        while let Some(key) = args.next() {
            let key = key.into_string().map_err(|_| "invalid option")?;
            ensure(
                allowed.contains(&key.as_str()),
                format!("unknown option {key}"),
            )?;
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {key}"))?;
            ensure(
                !value.to_string_lossy().starts_with("--"),
                format!("missing value for {key}"),
            )?;
            ensure(
                options.insert(key.clone(), PathBuf::from(value)).is_none(),
                format!("duplicate option {key}"),
            )?;
        }
        let path = |key: &str| -> Result<&Path> {
            options
                .get(key)
                .map(PathBuf::as_path)
                .ok_or_else(|| format!("required option {key}").into())
        };
        for key in allowed {
            if *key != "--map" && *key != "--world" {
                path(key)?;
            }
        }
        if command == "import-tiled" {
            ensure(
                options.contains_key("--map") != options.contains_key("--world"),
                "choose exactly one of --map or --world",
            )?;
        }
        if command == "export" {
            Project::from_pack(&Pack::parse(std::fs::read(path("--base-pack")?)?)?)?
                .save_new(path("--out")?)?;
        } else {
            let mut project = Project::load(path("--project")?)?;
            match command {
                "build" | "validate" => {
                    let (data, receipt) = project.build()?;
                    if command == "build" {
                        crate::cli::write_build(path("--out")?, &data, &receipt)?;
                    }
                    println!("{}", serde_json::to_string_pretty(&receipt)?);
                }
                "export-tiled" => project.export_tiled(path("--out")?)?,
                "import-tiled" => {
                    if options.contains_key("--world") {
                        project.import_tiled_world(path("--world")?)?;
                    } else {
                        project.import_tiled(path("--map")?)?;
                    }
                    project.save_new(path("--out")?)?;
                }
                "preview" => {
                    #[cfg(feature = "preview")]
                    {
                        let room = path("--room")?.to_str().ok_or("invalid room")?;
                        let room = if let Some(hex) = room.strip_prefix("0x") {
                            usize::from_str_radix(hex, 16)?
                        } else {
                            room.parse()?
                        };
                        let (data, _) = project.build()?;
                        crate::preview::export_preview(
                            &Pack::parse(data)?,
                            &std::fs::read(path("--rom")?)?,
                            room,
                            None,
                            path("--out")?,
                        )?;
                    }
                    #[cfg(not(feature = "preview"))]
                    return Err(
                        "rebuild with cargo build -p zelda3-map-tools --features preview".into(),
                    );
                }
                _ => unreachable!(),
            }
        }
        if command != "build" && command != "validate" {
            println!("wrote {}", path("--out")?.display());
        }
        Ok(())
    };
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}
