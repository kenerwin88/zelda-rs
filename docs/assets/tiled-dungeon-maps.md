# Tiled dungeon maps with exact vanilla bytes

The Rust `dungeon-tiled` binary provides standard Tiled `.tmj` maps and a `.world`
index for all 320 dungeon rooms. The runtime still consumes the existing asset
pack. No runtime loader, gameplay code, timing code, or parity lock changes.

For modern sources that rebuild without the original pack, start with
[portable dungeon projects](portable-dungeon-projects.md). This page describes
the lower-level single-pack adapter and its preview controls.

The guarantee is:

```text
original pack → Tiled map → save in Tiled → compile → identical original pack
```

An edited map intentionally changes bytes. The compiler reports those changes
and preserves the current position-only editing constraints. This format
migration is not a new engine or an unrestricted map system.

## Open the generated maps

```sh
cargo build -p zelda3-map-tools

target/debug/dungeon-tiled export \
  --base-pack target/room-authoring/base.dat --all \
  --out target/room-authoring/my-tiled-project
```

In Tiled, open `room-104.tmj` (Link's house) or `room-061.tmj` (the castle
entrance). Use **World → Load World…** to open `dungeons.world`. The world index
places rooms in the engine's 16-column room-ID grid. Moving a map in this index
only changes its editor arrangement; it does not change engine adjacency.

All outputs are create-only, so choose a fresh export directory. Extracted maps
remain under ignored `target/` because they contain original game data.

For a single room:

```sh
target/debug/dungeon-tiled export \
  --base-pack target/room-authoring/base.dat --room 0x104 \
  --out target/room-authoring/house.tmj
```

## What the editor shows

Each map is 512×512 pixels with an 8-pixel grid and six object layers:

| Layer | Meaning | Editing |
| --- | --- | --- |
| Room objects, pass 1 | First BG1 object stream | Move on the 8-pixel grid |
| Room objects, pass 2 | BG2 object stream | Move on the 8-pixel grid |
| Room objects, pass 3 | Final BG1 object stream | Move on the 8-pixel grid |
| Actors | Sprites and spatial overlords | Move on the 16-pixel grid |
| Entrances | Room-local player spawn positions | Move in whole pixels |
| Doors | Named direction/type markers at engine draw anchors | Reference only |

Use Tiled's object selection tool to move markers. Points identify object draw
origins; they do not claim to be the object's footprint or collision rectangle.
Plain `export` produces markers only. The optional `preview` command below
adds room artwork and collision references.
There are no painted tile layers or editable tilesets yet. The overworld is not
converted by this command.

Map properties expose the room ID, inherited default layout, floor styles,
background and collision modes, palette, tile theme, sprite graphics set, and
room tags. They are currently reference values. Raw headers, packed flags,
sprite-control commands, unusual door records without a known draw anchor, and
other non-spatial data stay in the exact base pack.

Names, colors, visibility, and non-`zelda.*` custom properties are editor
annotations. Renaming an object does not change its game identity. Layer and
object array ordering can change when saving; stable IDs and `zelda.order`
restore the original runtime order. Keep the generated IDs, classes, and
`zelda.*` properties intact.

The importer rejects unsupported changes rather than silently dropping them:
added/deleted objects or layers, changed object classes, rotations, resizing,
off-grid coordinates, layer offsets, foreign tilesets, moved door markers, and
changed room settings. The native compiler also rejects edits to shared room
bytes. Adding new content and changing object types remain future compiler work.

## Artwork and collision previews

Build the optional Rust preview feature and export into a fresh directory:

```sh
cargo build --profile parity -p zelda3-map-tools --features preview

target/parity/dungeon-tiled preview \
  --base-pack target/room-authoring/base.dat --rom /path/to/zelda3.sfc \
  --room 0x104 --out target/room-authoring/house-visual

target/parity/dungeon-tiled preview \
  --base-pack target/room-authoring/base.dat --rom /path/to/zelda3.sfc \
  --room 0x61 --out target/room-authoring/castle-visual
```

Open `room.tmj` from either output directory in Tiled. The six original object
layers remain editable as before. Three locked image layers add room artwork
and independently toggleable BG2/BG1 collision overlays. A room covers all four
256-pixel quadrants, including unused quadrants visible in the house layout.

Collision colors: red for solid tiles/blocking objects, orange for partial
slopes, blue for water, purple for pits, yellow for special or conditional
attributes. These categories follow the indoor branches in the engine's tile
detection code; they do not prove reachability or resolve every interaction.
`preview.json` retains both exact attribute arrays (64 columns, row-major),
tile words, target palette, input hashes and an attribute-code legend.

The preview uses the engine's existing room, graphics, palette and collision
producers in a fresh, disposable state. It draws the initial room using the
unfaded target palette. It does not run frames, read saves, use the GPU, render
actors, or reproduce lighting, color math and animation. It currently requires
a room with a direct entrance and chooses its first entrance; the house and
castle entrance are the initial verified examples. A preview is not a gameplay
or parity receipt.

After moving markers, save the map. `validate` and `build` report
`preview_stale: true` when its images describe older compiled bytes. Regenerate
from the edited map into a fresh directory:

```sh
target/parity/dungeon-tiled preview \
  --base-pack target/room-authoring/base.dat --rom /path/to/zelda3.sfc \
  --room 0x104 --map target/room-authoring/house-visual/room.tmj \
  --out target/room-authoring/house-visual-v2
```

Compile the new `room.tmj` with the usual `build --map` command. Preview layers
never supply runtime tiles or collision data. Their transforms and identities
are validated; their visibility and opacity are editor choices. Keep the PNGs
beside the map when copying an exported preview directory.

The broader target is a [native modern room format](native-map-format.md), with
complete room definitions replacing the original source data and eventually
the runtime's packed streams. This preview step does not switch runtime loaders.

## Compile a map or the entire dungeon world

```sh
target/debug/dungeon-tiled build \
  --base-pack target/room-authoring/base.dat \
  --map target/room-authoring/my-tiled-project/room-104.tmj \
  --out target/room-authoring/my-house.dat

target/debug/dungeon-tiled build \
  --base-pack target/room-authoring/base.dat \
  --world target/room-authoring/my-tiled-project/dungeons.world \
  --out target/room-authoring/my-dungeons.dat
```

Use `validate` instead of `build` and omit `--out` to check without writing a
pack. The world build requires each of the 320 room IDs exactly once; missing
or duplicated rooms are errors. Its room paths must stay within the world
directory. All maps are checked against the same base, and conflicting byte
edits are rejected.

The `.dat.json` receipt records base/output SHA-256 hashes and whether the
**whole output pack** is byte-identical. A world build also reports changed
rooms and changed byte count. Load the result using the existing isolated
[room playtest workflow](dungeon-room-authoring.md#playtesting).

The original pack is still required: it supplies unrelated assets, original
offsets, sharing relationships, padding, and opaque data. Maps are bound to its
SHA-256. These files are a modern authoring representation with lossless import,
not a replacement for every game asset or a standalone source-only distribution.

## Verification

```sh
cargo test -p zelda3-map-tools

ZELDA3_ROOM_TEST_PACK="$PWD/target/room-authoring/base.dat" \
  cargo test -p zelda3-map-tools --test corpus real_corpus -- --ignored --nocapture

# With a Tiled executable: read and re-save every real room in Tiled first.
ZELDA3_ROOM_TEST_PACK="$PWD/target/room-authoring/base.dat" \
ZELDA3_TILED_BIN=/path/to/Tiled \
  cargo test -p zelda3-map-tools --test corpus real_tiled_editor -- --ignored --nocapture

# Preview contracts and real house/castle previews, including Tiled saves.
cargo test --profile parity -p zelda3-map-tools --features preview
ZELDA3_ROOM_TEST_PACK="$PWD/target/room-authoring/base.dat" \
ZELDA3_ROM=/path/to/zelda3.sfc ZELDA3_TILED_BIN=/path/to/Tiled \
  cargo test --profile parity -p zelda3-map-tools --features preview \
  --test preview preview -- --ignored --nocapture
```

The corpus check compares the entire compiled pack for each of the 320 rooms.
The editor check uses Tiled's `--export-map json` to ensure real editor output
survives the importer, then builds all 320 editor-saved maps together. Unit contracts cover pixel/grid conversion, deterministic
ordering, exact edited byte output, rejected edits, and world builds.

These commands and tests are entirely Rust, in `crates/map-tools`. The authoring
tools build independently of the game and its generated assets. Existing map
exports keep the same format and base-pack identity checks.

Byte-identical vanilla output is a content conversion proof. It is not a cold
Snes9x gameplay/video/audio receipt. The pre-existing runtime blockers documented
in the earlier room-authoring guide are not changed by this tool.

References: [Tiled JSON map format](https://doc.mapeditor.org/en/stable/reference/json-map-format/)
and [Tiled world format](https://doc.mapeditor.org/en/stable/manual/worlds/).
