# Readable Asset Sources

`assets_src/` contains hand-readable source assets that can be packed back into
the generated runtime assets under `generated/zelda3_assets/`. The extractor
also emits readable migrated assets under
`generated/zelda3_assets/assets_src/`.

The runtime still consumes byte-exact generated assets so C parity stays
unchanged. Source assets in this directory should always have a verifier that
proves they pack back to the canonical extracted bytes.

## Tilemaps

During ROM extraction, tilemap assets are no longer written as loose binary
files. The extractor writes JSON sources under
`generated/zelda3_assets/assets_src/tilemaps/`, and the build script packs those
JSON files into the embedded asset table.

Rectangular byte-grid tilemaps use the `zelda3_byte_tilemap_v1` schema:

- `asset` and `asset_index` identify the generated asset it replaces.
- `width` and `height` describe the rectangular byte grid.
- `rows` stores each tile id as a JSON number from `0` through `255`.
- `canonical_sha1` records the source binary hash from the extraction run.

Variable-length background tilemap payloads use the
`zelda3_byte_stream_tilemap_v1` schema:

- `asset` and `asset_index` identify the generated asset it replaces.
- `values` stores the raw byte stream in 32-byte chunks for readable diffs.
- `canonical_sha1` records the source binary hash from the extraction run.

Verify the migrated source pipeline with:

```sh
python3 scripts/test_asset_source_build.py
python3 scripts/test_extract_asset_sources.py
python3 scripts/test_tilemap_json.py
```
