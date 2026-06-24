# Readable Asset Sources

Runtime assets are generated from a user-provided ROM and stay outside git.
Readable migrated assets live under `generated/zelda3_assets/assets_src/`, which
is ignored for the same reason as generated `.bin` assets: it still contains
extracted vanilla game data.

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
python3 scripts/test_tilemap_json.py
```
