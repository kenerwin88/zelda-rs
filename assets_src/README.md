# Readable Asset Sources

`assets_src/` contains hand-readable source assets that can be packed back into
the generated runtime assets under `generated/zelda3_assets/`.

The runtime still consumes byte-exact generated assets so C parity stays
unchanged. Source assets in this directory should always have a verifier that
proves they pack back to the canonical generated `.bin` bytes.

## Tilemaps

`tilemaps/light_overworld_tilemap.json` is the first vertical slice. It uses the
`zelda3_byte_tilemap_v1` schema:

- `asset` and `asset_index` identify the generated asset it replaces.
- `width` and `height` describe the rectangular byte grid.
- `rows` stores each tile id as a JSON number from `0` through `255`.
- `canonical_sha1` records the source binary hash from the extraction run.

Verify the slice with:

```sh
python3 scripts/tilemap_json.py verify \
  --input-json assets_src/tilemaps/light_overworld_tilemap.json \
  --canonical-bin generated/zelda3_assets/assets/067-kLightOverworldTilemap.bin
```
