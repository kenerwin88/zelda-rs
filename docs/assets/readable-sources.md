# Readable Asset Sources

Runtime assets are generated from a user-provided ROM and stay outside git.
Readable migrated assets live under `generated/zelda3_assets/assets_src/`, which
is ignored for the same reason as generated `.bin` assets: it still contains
extracted vanilla game data.

For visual art replacement, use the canonical RGBA art workflow first:

```text
docs/assets/modder-rgba-workflow.md
```

That workflow starts from `generated/zelda3_assets/atlas/art_tiles.png`, which
deduplicates palette and flip variants into a cleaner editable sheet. The
lower-level source formats below remain useful for extraction, packing, and
parity implementation work.

The runtime continues to consume byte-exact packed assets so C parity stays
unchanged. The build script packs generated readable sources into the embedded
asset table when the manifest provides `source_file` and `source_format`;
unsupported assets continue through the binary fallback path.

## Tilemaps

During ROM extraction, tilemap assets are not written as loose binary files.
The extractor writes JSON sources under
`generated/zelda3_assets/assets_src/tilemaps/`, and the build script packs those
JSON files into the embedded asset table.

Rectangular byte-grid tilemaps use the `zelda3_byte_tilemap_v1` schema:

- `asset` and `asset_index` identify the generated asset.
- `width` and `height` describe the rectangular byte grid.
- `rows` stores each tile id as a JSON number from `0` through `255`.
- `canonical_sha1` records the source binary hash from the extraction run.

Variable-length background tilemap payloads use the
`zelda3_byte_stream_tilemap_v1` schema:

- `asset` and `asset_index` identify the generated asset.
- `values` stores the raw byte stream in 32-byte chunks for readable diffs.
- `canonical_sha1` records the source binary hash from the extraction run.

Verify the migrated source pipeline with:

```sh
python3 scripts/test_asset_source_build.py
python3 scripts/test_extract_asset_sources.py
python3 scripts/test_navigation_json.py
python3 scripts/test_tilemap_json.py
python3 scripts/test_palette_json.py
```

## Palettes

During ROM extraction, palette-like assets are also written as JSON sources
under `generated/zelda3_assets/assets_src/palettes/`. This includes all assets
with `Palette` in the generated name plus `kHudPalData`.

Palettes use the `zelda3_snes_palette_v1` schema:

- `asset` and `asset_index` identify the generated asset.
- `color_encoding` records the raw encoding as SNES BGR555 little-endian.
- `colors[].snes_bgr15` stores the exact 15-bit SNES color word used for
  packing.
- `colors[].rgb888` stores a readable preview color derived from that word.
- `canonical_sha1` records the source binary hash from the extraction run.

## Navigation Tables

Entrance, starting point, overworld exit, and special exit tables are grouped by
record instead of stored as parallel `.bin` arrays. During extraction, these
assets are written under `generated/zelda3_assets/assets_src/navigation/`:

- `dungeon_entrances.json` uses `zelda3_dungeon_entrances_v1`.
- `starting_points.json` uses `zelda3_starting_points_v1`.
- `overworld_exits.json` uses `zelda3_overworld_exits_v1`.
- `special_exits.json` uses `zelda3_special_exits_v1`.

Each record includes the legacy table fields as named JSON numbers. Signed
legacy fields, such as dungeon `floor`/`palace`, exit `unk1`/`unk3`, and special
exit `tab4` through `tab7`, are stored as signed JSON numbers and packed back
to the original little-endian byte representation by the build script.

## Dialogue

During ROM extraction, the dialogue asset gets two readable files under
`generated/zelda3_assets/assets_src/dialogue/`:

- `dialogue_catalog.json` uses `zelda3_dialogue_catalog_v1`.
- `dialogue_source.json` uses `zelda3_dialogue_source_v1`.

The catalog is the semantic bridge for inspection and parity work. It preserves:

- source asset hashes for `kDialogue` and `kDialogueMap`;
- language/map config from asset `096`;
- dictionary expansion records for every message;
- original raw message bytes and expanded bytes;
- a parsed operation stream for glyphs and US dialogue commands such as
  `player_name`, `number`, `color`, `wait`, `speed`, `line1` through `line3`,
  `choose`, and `end_message`;
- a lossy `preview_text` field for quick human inspection.

Dynamic runtime values are intentionally kept as operations instead of being
resolved during extraction. For example, a player-name command stays
`player_name`; it is not replaced by a particular save-slot name.

The source file is the editable authority for building `kDialogue`. Each message
stores `source_text` using literal glyph text plus explicit control tags such as `[line1]`, `[wait 03]`,
`[color 02]`, `[player_name]`, `[choose]`, and `[end_message]`. Bracketed
button/symbol glyphs, such as `[A]` and `[Up]`, keep their glyph names.

Extraction verifies each generated `source_text` by compiling it back to the
expanded bytecode recorded in the catalog. The build script uses the shared
`zelda3-dialogue` source compiler to pack `dialogue_source.json` into a valid
`kDialogue` asset with uncompressed message bytecode and an empty dictionary
table. This intentionally trades the original ROM compression for a simpler
authoring path while keeping the runtime message-state machine unchanged.
Generated messages also include `expanded_sha1`; when present, the Rust source
compiler validates it against the compiled message bytes. This makes extracted
source files parity-checked by default. Deliberate edits should update or remove
that per-message hash so the source change is explicit.
When `zelda3-bin` packs assets, `dialogue_source.json` is required for
`kDialogue`; a stale `094-kDialogue.bin` is not used as a fallback. The packer
also embeds a named `kDialogueSourceSemantic` sidecar derived from that source
file. The sidecar payload is a self-identifying serialized table of
`DialogueIrOp` messages, so the modern GPU dialogue path reads source-derived
semantic IR directly without reparsing compiled `kDialogue` bytes.

The canonical art atlas also exports editable dialogue VWF glyph sheets under
`generated/zelda3_assets/atlas/`. `dialogue_vwf_glyphs.png` and
`dialogue_vwf_glyphs.json` include the main grayscale glyph cells plus
`palette_bg3_text_color_00` through `palette_bg3_text_color_0f` cells generated
from `hud_pal_data.json`, so semantic `[color xx]` commands can select colored
PNG glyph variants directly.
