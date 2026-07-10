# Semantic Parity Status

This file records the current implementation status for
`semantic_parity_migration.md`.

## Active Layers

- Byte parity remains the default strict oracle. It still compares WRAM, SRAM,
  VRAM, CGRAM, OAM, and PPU-visible registers unless a caller explicitly chooses
  a graduated semantic comparator.
- Semantic snapshots are available beside byte parity and can be printed with
  `--trace-semantic-state`. They name frame, player, world, Map16 load, sprite,
  ancilla, and PPU-facing fields.
- Typed RAM views provide the source-address-backed extraction layer for frame
  control, player state, world/camera state, sprite slots, ancilla slots, and
  overworld Map16 load state.
- Mutable typed slot views now back the central sprite/ancilla coordinate
  getters, setters, movement helpers, sprite spawn-coordinate writes, and a
  small set of hookshot/dynamic-spawn slot writes. They also back canonical
  sprite velocity helpers for target-speed approach, inversion, wall bounce,
  halving, and applying projected speed, plus early ancilla spawn velocity and
  Z writes for rod shots, arrows, sword beams, falling prizes, blast-wall
  fireballs, dug-up flute setup, boomerangs, bombs, snoring, hit stars, bird
  setup, ice-rod shots/sparkles, Somaria block movement setup, boomerang
  return acceleration, bomb bounce response, revival/duck Z setup, and Ether
  spell velocity transitions. They also cover flute Z bounces, weather-vane
  debris slot velocity/Z replay, and liftable thrown-object velocity/Z
  settling, plus player hookshot drag lookup/countdown, gravestone slot
  movement cleanup, Somaria block Z checks, and Byrna/shovel ancilla
  initialization fields. Player item/action code now routes ancilla type
  checks and clears through typed slot views. Current sprite coordinate
  latching and local attract/ending
  coordinate helpers now delegate through those paths, so new call sites can
  migrate without reintroducing raw lo/hi/subpixel byte packing.
- Lanmola flat trail reads now route through a named byte-backed reader. This
  remains a compatibility surface, not graduated owned state: the Lanmola trail
  uses 192 flat raw slots across Moldorm and Beamos history pages, while the
  native Moldorm/Beamos history models intentionally cover 128-slot banks.
- `GameFrameOutput` now exists as an additive typed output boundary for runtime,
  render, and audio-facing facts. The audio portion exposes `AudioRouteState`
  and `AudioEventFrame`: host/sample statistics and DSP-write compatibility
  hashes remain available, but replay diagnostics can now also hash stable
  command-level audio events. Default playback still uses the existing DSP
  parity backend; new modern audio consumers should depend on typed event
  frames and treat direct DSP/SPC access as parity or diagnostic code.
- An opt-in `ModernAudioEngine` now consumes `AudioEventFrame` directly and can
  produce deterministic non-DSP samples from APUI/music-port events and typed
  voice key/parameter events. It reports per-frame coverage stats for understood,
  ignored, triggered, and active events. This is an architectural foothold, not
  a finished music renderer: echo/global DSP events are intentionally counted as
  ignored until modeled at the typed-event level.
- `ModernAudioSequencer` now sits before that engine in the modern backend. It
  translates APUI/music route state into typed `PlayMusic`, `PlaySfx`,
  `SetTempo`, `SetEnvelope`, `NoteOn`, and `NoteOff` events, so the opt-in
  modern path is no longer deriving its primary intent from DSP register writes.
  The DSP-compatible path remains the exact parity oracle and still feeds trace
  diagnostics.
- `ModernSfxCatalog` adds the first data-backed SFX programs for menu cursor,
  sword, rupee pickup, door/stairs, and damage. Known `PlaySfx` commands expand
  into typed program events including envelope, note duration, pitch slide, and
  noise selection; unknown commands still use a marked heuristic fallback.
  Audio trace output now reports modern SFX known/unknown counts and the modern
  program hash beside the existing DSP/sample parity fields.
- `scripts/extract_modern_sfx_catalog.py` is the bridge from DSP parity to
  modern SFX authoring. It reads Rust audio trace JSONL, detects SFX command
  transitions with the same slot mapping as `ModernAudioSequencer`, and lifts
  focused `dsp_write_events` windows into reviewable modern program candidates.
  Missing or ambiguous traces are reported as coverage gaps instead of silently
  generating catalog entries. The intended workflow is to run a broad audio
  trace to find SFX command frames, rerun focused frames with
  `ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME=<frame>`, then review the generated JSON
  or Rust snippet before replacing hand-authored catalog entries.
- `scripts/harvest_modern_sfx_catalog.py` automates that workflow for replay
  routes. It can run the Rust replay binary to capture a broad audio trace,
  discover SFX command frames, rerun focused traces for each selected command,
  and feed those focused windows through the extractor. Outputs land under
  `target/modern-sfx-harvest/` by default: `modern-sfx-harvest.json` for
  coverage and candidate data, `modern-sfx-candidates.rs` for reviewable Rust
  snippets, and `modern-sfx-harvest.md` for a human coverage report. Use
  `--max-occurrences` for bounded exploratory runs and `--fail-on-gaps` when
  gating a known route slice.

## Graduated Subsystems

### OverworldMap16Load

`OverworldMap16Load` is the first graduated subsystem. Rust stores the typed
runtime state in `OverworldMap16LoadState` while keeping the legacy WRAM bytes
materialized for checkpoints, replay tooling, and the default byte gate.

The explicit graduated comparator may narrow raw WRAM comparison for the
private Map16 load bytes only when semantic snapshots prove these fields match:

- `world.map16_load_src`
- `world.map16_load_dst`
- `world.map16_load_y_unit`

The default lockstep path does not use this narrowing, so existing byte parity
remains the strongest regression gate.

## Subsystems Still Requiring Byte Parity

All other game state still requires strict byte parity. Player movement,
camera/scroll, inventory/menu state, sprites, ancilla, shared scratch, and
PPU/audio-facing effects have semantic fields for diagnostics. Sprite and
ancilla semantic extraction now uses typed slot views over WRAM, but they are
not graduated because their owned-state storage, edge-route coverage, and
output-specific parity gates are not yet strong enough to replace raw WRAM
comparison.

Shared scratch remains byte-only until call-site lifetime proves it is local.
PPU and audio behavior remain output-gated separately from gameplay semantics.
The typed audio event boundary is diagnostic and preparatory, not a graduated
replacement for sample/DSP parity.
