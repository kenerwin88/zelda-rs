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

### The editable authority: `assets/dialogue/messages.toml`

The editable authority for building `kDialogue` is the **tracked, committed**
`assets/dialogue/messages.toml` at the repo root — not the generated JSON. It is
plain TOML: a `format = "zelda3_dialogue_source_v1"` header followed by one
`[[message]]` table per message with an `id` and a `text` body. Bodies use TOML
literal strings (`'''…'''`), so glyph text and bracketed control tags pass
through verbatim with no escaping, and `#` comments are allowed between entries.
Each `text` uses literal glyph text plus explicit control tags such as `[line1]`,
`[wait 03]`, `[color 02]`, `[player_name]`, `[choose]`, and `[end_message]`;
bracketed button/symbol glyphs such as `[A]` and `[Up]` keep their glyph names.

Edit `messages.toml`, rebuild, and the change appears in-game: the build script
uses the shared `zelda3-dialogue` compiler to pack the messages into a valid
`kDialogue` asset with uncompressed message bytecode and an empty dictionary
table. This intentionally trades the original ROM compression for a simpler
authoring path while keeping the runtime message-state machine unchanged.

The `generated/…/dialogue_source.json` written during extraction is now a
bootstrap fallback and inspection artifact only; hand-edits belong in the tracked
`messages.toml`, which the build reads first, so extraction never clobbers them.

### The parity lock: `assets/dialogue/messages.sha1`

Vanilla parity is enforced by a **separate** tracked lock file,
`assets/dialogue/messages.sha1` (`format = "zelda3_dialogue_sha1_lock_v1"`, one
`[[message]]` per id with an `expanded_sha1`). When the lock is present the build
compiles each message and verifies its expanded bytecode against the blessed
hash, failing the build with `drifted from the parity lock` (or `missing from the
parity lock`) if an edit changes a message without re-blessing. Keeping the lock
out of the content file means `messages.toml` stays a clean, diff-friendly text
of just the dialogue.

After a deliberate edit, regenerate the lock so the new bytes become the blessed
baseline:

```
target/parity/zelda3 --bless-dialogue [<messages.toml> <messages.sha1>]
```

With no arguments it re-blesses `assets/dialogue/messages.toml` →
`assets/dialogue/messages.sha1` in place.

### Build precedence and overrides

`build.rs` resolves the dialogue source and lock in this order:

1. `ZELDA3_DIALOGUE_MESSAGES` / `ZELDA3_DIALOGUE_SHA_LOCK` env overrides — point
   the build at an alternate dialogue file (alt-language packs, tests) without
   touching the tracked authority. Overriding the messages **without** also
   overriding the lock builds **unlocked** (the alternate file owns its own
   parity contract).
2. the tracked `assets/dialogue/messages.toml` + `assets/dialogue/messages.sha1`.
3. the generated `dialogue_source.json` (unlocked) as a bootstrap fallback.

`kDialogue` is always source-built; a stale `094-kDialogue.bin` is never a
fallback. The packer also embeds a named `kDialogueSourceSemantic` sidecar
derived from the same messages. The sidecar payload is a self-identifying
serialized table of `DialogueIrOp` messages, so the modern GPU dialogue path
reads source-derived semantic IR directly without reparsing compiled `kDialogue`
bytes.

### Regenerating the on-disk `zelda3_assets.dat`

The replay/parity flows (`--replay-save`, `zparity`, `validate_all_parity.py`)
load the on-disk `zelda3_assets.dat` via `find_asset_pack` (ROM dir, then cwd,
then repo root) — not the binary's embedded pack. Because the source-authoritative
build makes the `kDialogueSourceSemantic` sidecar **required**, a legacy restool
pack (or any pack predating the sidecar) is rejected at load with
`asset pack contains kDialogue but is missing required kDialogueSourceSemantic`,
which fails every replay at frame 0. Regenerate the on-disk pack so it matches the
current binary:

```sh
cargo build --profile parity -p zelda3-bin
target/parity/zelda3 --dump-asset-pack zelda3_assets.dat
```

`--dump-asset-pack` writes the embedded (build.rs-packed, source-authoritative)
pack, so the emitted file is byte-identical to what the binary runs with,
sidecar included. The root `zelda3_assets.dat` is gitignored (a local artifact).

### Dialogue parity under source authority

The uncompressed source-authored `kDialogue` is **behaviorally byte-exact** with
the C oracle. `Text_GenerateMessagePointers` stages a 3-byte-per-message pointer
table at WRAM `TEXT_DIALOGUE_POINTERS` (`0x171c0`); because source messages are
uncompressed, those pointers (byte offsets) differ from the ROM-compressed
oracle. This divergence is transient and confined to that scratch region:
from-scratch C-vs-Rust WRAM dumps differ only in the pointer table at boot
(frame 2000: 848 bytes, all inside `0x171c0`), and are **zero-diff** once the
game is running — including during an active on-screen message (frame 60000) and
the ending credits where the table is regenerated (frame 1033000). It never
cascades into game logic.

Because the divergence is byte-exact-behavioral-equivalent scratch, the pointer
table span `[0x171c0, 0x1766a)` is masked out of the parity fingerprint on both
sides — `FINGERPRINT_MASK_RANGES` in `crates/parity/src/fingerprint.rs` and
`IsFingerprintMaskedWramOffset` in the C oracle's `src/main.c` — and the golden
was recaptured with the wider mask (`manifest.mask` length 450 → 1644). With that,
`zparity check` no longer flags the transient dialogue pointers and can see past
boot to any genuine divergence.

The canonical art atlas also exports editable dialogue VWF glyph sheets under
`generated/zelda3_assets/atlas/`. `dialogue_vwf_glyphs.png` and
`dialogue_vwf_glyphs.json` include the main grayscale glyph cells plus
`palette_bg3_text_color_00` through `palette_bg3_text_color_0f` cells generated
from `hud_pal_data.json`, so semantic `[color xx]` commands can select colored
PNG glyph variants directly.
