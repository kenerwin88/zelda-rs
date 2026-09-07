# Dungeon room authoring prototype

For editing through a standard map editor, use the
[Tiled dungeon map workflow](tiled-dungeon-maps.md). It compiles through the
same lossless codec described here.

The Rust `dungeon-room` binary exports a dungeon room from an existing asset pack,
validates position edits, and produces a separate pack. The compiler does not
change runtime code, the ROM, vanilla assets, or parity locks. Outputs are
create-only: choose a new name to rebuild, rather than overwriting a base pack.

## Supported edits

This first version moves existing content within the original room allocation:

- `layout.layers[].objects[].x/y`: position in the room's 8-pixel grid. Objects
  retain their original type, size bits, layer, and ordering. Type 1 and type 3
  objects cannot use x=63 because that encodes another object family.
- `sprites.records[].x/y`: position in the room's 16-pixel grid. Normal sprites
  and overlords retain IDs, subtype/floor flags, ordering, and slot behavior;
  special control records remain opaque and unchanged.
- `entrances[].player_x/player_y`: absolute player spawn coordinates in pixels,
  restricted to the original 512-pixel room bounds. Other entrance fields remain
  unchanged. This edits entrance tables, **not** the separate starting-point
  tables used when loading a saved game.

Room headers, the two layout prefix bytes, doors (including the distinction
between no door stream and an empty door stream), and unknown bits survive
unchanged. Default room layouts, chest rewards, room tags, and room-specific
engine behavior are inherited from the base pack. Object/enemy insertion,
deletion, resizing, type changes, new IDs, and entrance redirection are not yet
supported. These constraints preserve offsets and record structure while the
content/timing boundary is being tested.

Validation checks formats, ranges, base-pack identity, edit scope, and shared
data ownership. It rejects changed bytes referenced by another room, including
shared suffixes and door-table references. It does not prove collision safety,
reachability, or correct custom-map execution. Moving an object onto a wall or
an actor into an obstacle can still produce an unplayable room.

## Export, edit, build

Use a current, playable asset pack (including its required dialogue/timing
sidecars). The normal binary's `--dump-asset-pack` command exports its embedded
pack when that binary is available. Keep extracted content and built packs under
an ignored directory such as `target/room-authoring/`.

```sh
cargo build -p zelda3-map-tools

target/debug/dungeon-room export \
  --base-pack /path/to/zelda3_assets.dat --room 0x61 \
  --out target/room-authoring/castle.json

# First prove an untouched round trip. The receipt must say byte_identical: true.
target/debug/dungeon-room build \
  --base-pack /path/to/zelda3_assets.dat \
  --source target/room-authoring/castle.json \
  --out target/room-authoring/unchanged.dat
```

Edit the exported JSON in a text editor and save a copy as
`target/room-authoring/castle-edited.json`. For stock room `0x61`, these three
edits move an object, a guard, and the entrance spawn. Array indexes start at 0.
Check the identity and original value before editing; a different base layout
may put different content at these indexes.

| Field | Expected identity | Before | After |
| --- | --- | --- | --- |
| `layout.layers[0].objects[90].x` | type1, id 57, y 28, size bits 0/0 | 12 | 14 |
| `sprites.records[1].x` | id 75, y 18, flags 0/0 | 13 | 12 |
| `entrances[0].player_x` | entrance index 4 | 760 | 752 |

```sh
target/debug/dungeon-room validate \
  --base-pack /path/to/zelda3_assets.dat \
  --source target/room-authoring/castle-edited.json

target/debug/dungeon-room build \
  --base-pack /path/to/zelda3_assets.dat \
  --source target/room-authoring/castle-edited.json \
  --out target/room-authoring/castle.dat
```

The `castle.dat.json` receipt records input/output SHA-256 hashes and changed
assets/byte counts. A build receipt never claims runtime parity. For this example
only three bytes change: one object position byte, one sprite position byte, and
one entrance coordinate byte. All pack sizes, offsets, padding, other assets,
and trailing data are preserved.

## Playtesting

An existing playable binary can load the pack through its supported asset
override. The wrapper requires an explicit save directory so the experiment can
have its own saves; `--dry-run` prints the exact command and overrides.

```sh
target/debug/dungeon-room play \
  --binary target/parity/zelda3 --rom /path/to/zelda3.sfc \
  --pack target/room-authoring/castle.dat \
  --save-dir target/room-authoring/castle-saves --dry-run
```

Remove `--dry-run` to launch. This boots normally; it does not warp to the room.
Do not use a checkpoint containing a different embedded asset pack as proof
that the mod loaded.

The optional `dungeon-room-playtest` example also supports a recorded controller
route, a room-entry snapshot, and interactive play after reaching the room. It
reuses the main game's engine, input parser, renderer, and frontend. It neither
patches game RAM nor reads/writes the user's save files. Pass `--sram` explicitly
when the input route requires a particular initial save. Its GPU captures share
the parity tools' exclusion lock.

```sh
cargo build --profile parity -p zelda3-bin --example dungeon-room-playtest
target/parity/examples/dungeon-room-playtest \
  --rom /path/to/zelda3.sfc --pack target/room-authoring/castle.dat \
  --input-script /path/to/continuous-input.txt --sram /path/to/initial.srm \
  --frames 35000 --room 0x61 --out target/room-authoring/castle-playtest --play
```

Building needs the usual generated assets; use `ZELDA3_ASSETS_DIR` if they live
elsewhere. The output directory must not already exist. Without `--play`, the
harness stops at the first frame in the requested room with main module 7 and
submodule 0. This is an entry observation, not a complete traversal test.
It writes transitions, WRAM, player/sprite coordinates, and a GPU screenshot.
Engine failures produce `failure.json`. `--snapshot-frame N` captures a specified
zero-based input frame while the requested room is loaded, including during
entry/cutscenes; these receipts explicitly say `snapshot_only: true`.

## Validation and current acceptance boundary

```sh
cargo test -p zelda3-map-tools
ZELDA3_ROOM_TEST_PACK=/path/to/zelda3_assets.dat \
  cargo test -p zelda3-map-tools --test corpus real_corpus -- --ignored --nocapture
```

The second command verifies all 320 real room exports, their Tiled equivalents,
and the combined world by recompiling to an identical **whole pack**. The
ROM-free tests independently encode the runtime's object/sprite format rules,
check precisely which bytes change, and test malformed input and shared data.

The codecs, CLI commands, entrance-table reader, and tests live in the standalone
`crates/map-tools` Rust crate. Building these tools needs no ROM, generated game
assets, or interpreter. Existing JSON and Tiled exports remain compatible. The
game does not depend on this crate. Real-corpus tests are explicitly ignored by
default and fail if selected without their required local inputs.

At the initial implementation on base commit
`da5d3be6d07aaa7565c267c1265bae0656d66c85`, full runtime acceptance is blocked:

- The main binary fails to compile because `snes9x_semantic_receipts.rs` lacks a
  `SpriteMainProgress::TrinexxHeadDrawSetup(_)` match arm. The optional playtest
  example builds without that adapter.
- The unchanged base pack, booted through the playtest example with the local
  continuous route and initial SRAM, fails at input frame 2507 in `nmi.rs:291`:
  `an active-scanout NMI without a dialogue owner overlaps an in-flight text publication`.
  It has not reached the castle entrance at that point.

Neither issue is patched by this authoring work. They block a verified castle
entry/traversal and cold Snes9x acceptance; a successful pack round trip must not
be reported as those proofs. Follow the repository's current parity workflow
when those runtime issues are addressed. Do not weaken timing checks or re-bless
vanilla content to accommodate a mod.

A narrower normal-boot experiment reached room `0x104` at input frame 2320,
before the dialogue failure. Moving layer-0 object 7 two grid cells right and
sprite record 0 one grid cell left changed the room tile data and moved the NPC
from x=2472 to x=2456. Both runs were in main module 7 / submodule 15; this is a
cutscene/entry snapshot, not controllable-room or traversal acceptance. The
entrance-table edit did not move Link at saved-game startup, as expected from
the separate starting-point tables. These observations do not verify the
castle guard or entrance edit in play.
