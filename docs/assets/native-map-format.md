# Native modern map data: migration direction

The target is modern map data as the canonical source **and eventually the
runtime representation**. Tiled is an authoring adapter for that model. The
original packed map streams should become an optional compatibility export,
not a permanent dependency of every map.

New JSON files cannot themselves have the same bytes as the original binary
format. The two useful contracts are exact legacy export of untouched content,
and identical engine behavior when that content is loaded natively. During
migration we retain both, without equating an asset round trip to gameplay proof.

## Current boundary

The [portable project workflow](portable-dungeon-projects.md) now owns all 55
dungeon assets as typed modern sources in `world.json`. It rebuilds an untouched
whole pack exactly without the original pack or ROM. Shared resources are
explicit; the compatibility shell has every modeled dungeon asset zeroed.
The `zelda3-map` crate owns the content types and standalone validation, and
`zelda3-map-tools` owns the legacy compatibility codecs and CLI commands.

Tiled remains an adapter: a project can export maps, import position edits back
into its typed sources, and generate artwork/collision previews. The older
single-room JSON and Tiled-only exports still require their base pack.

The normal game build/runtime still consumes its existing asset pack. Modern
project compilation is currently an opt-in source path; native runtime loading
and removal of packed map decoding have not happened yet.

The optional `preview` feature adds generated artwork and collision overlays.
It calls the existing room producers in a fresh construction state through the
`zelda3/map-preview` feature. Ordinary game builds do not enable that feature.
Preview image layers are derived editor data and never replace gameplay content.

## Migration boundaries

1. **Complete, portable room model — implemented for the stock dungeon assets.**
   Versioned Rust definitions describe ordered object passes, actors and control
   records, doors, entrances, room settings, inherited layouts, rewards and
   travel links. They preserve unknown bits explicitly.
   All 320 rooms and their shared dungeon dependencies are included. The schema
   is strict and versioned; later versions must implement explicit migrations.
2. **Separate meaning from original storage — implemented.** Shared room programs
   are explicit resources. Original pointer aliases, stream boundaries and
   byte-preservation information live in a separate compatibility manifest.
   Unrelated graphics/audio resources remain in their existing formats.
3. **Make modern sources canonical at build time — opt-in project compiler implemented.**
   It generates exact stock dungeon sections from the complete model, with
   tests for shared storage and whole-pack equality. The engine can still
   consume generated legacy views, but project editing no longer requires the
   original map streams as its source of truth. Wiring this into the normal game
   asset build remains separate work.
4. **Introduce native runtime consumption.** Decode/validate the modern model at
   asset load, outside frame execution. Migrate individual map consumers to
   ordered typed records, preserving the original observable reads, writes,
   object order and scheduler boundaries. Preserve any stream offset that is
   observable as compatibility metadata until its consumer is migrated. Merely
   generating an old byte stream at startup is not the final native design.
5. **Remove the legacy runtime dependency after proof.** Keep a deterministic
   legacy exporter for regression checks. Require unchanged-stock engine-state
   comparison and the repository's cold Snes9x video/audio gates for each runtime
   migration. Only then retire the original runtime map decoder.

Adding objects and reallocating streams can be developed against the complete
model and compatibility exporter. Stock maps must retain their original layout
unless edited. Modded content intentionally changes data; stock parity remains
the regression contract, not a claim that a custom map matches the original ROM.

## Preview acceptance

- Untouched Tiled maps, including generated image layers, compile to the exact
  original pack.
- Editing a supported object changes the intended bytes and regenerates its
  artwork and collision data from those compiled bytes.
- Generated previews are bound to the compiled content hash; validation reports
  stale artwork after an edit.
- Construction snapshots remain distinct from live room state: no player,
  actors, save progress, animated frames, darkness, or PPU color effects are
  simulated in the background artwork.
