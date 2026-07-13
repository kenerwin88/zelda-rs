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
  command-level audio events. Default playback now uses the modern backend;
  `DspParity` remains explicitly selectable as the exact oracle, and `TraceOnly`
  remains available for silent event diagnostics. The playable host accepts
  `ZELDA3_AUDIO_BACKEND=modern|dsp-parity|trace-only` as an operator override.
- `ModernAudioEngine` consumes `AudioEventFrame` directly and produces
  deterministic samples from APUI/music-port events and typed voice,
  envelope, echo, and global-parameter events. It reports per-frame coverage
  stats for understood, ignored, triggered, and active events; the complete
  standard-route quality gate has zero ignored events.
  Instrument playback is now owned by the modern path. The checked-in
  `assets/audio/modern_samples/manifest.json` maps 25 source slots in each of
  the overworld, dungeon, and credits contexts to 23 deduplicated BRR files,
  with compact echo-memory seeds for the same contexts. The build validates
  every mapping, loop offset, BRR terminator, range, and SHA-256 before packing
  the catalog. At runtime the game selects a bank when the corresponding
  engine event occurs and the mixer decodes the typed asset directly; normal
  playback no longer looks instruments up in a materialized SPC address space.
  `scripts/export_modern_sample_bank.py` is the reproducible offline importer
  for refreshing this reviewed pack from the reference upload blobs. Normal
  gameplay never loads those blobs; the raw upload API remains as a compatibility
  and oracle/checkpoint surface.
  Typed pitch slides now retain their target and duration in checkpoint state,
  advance once per rendered game frame, and land exactly on the target instead
  of treating `PitchSlide.frames` as an immediate jump.
  Typed envelopes now retain attack, decay, sustain, and release configuration;
  note triggers advance through those stages deterministically, duration expiry
  enters release, and explicit note-off events produce a release tail rather
  than discarding the envelope or cutting the voice immediately.
  Noise selection is now a checkpointed override of the underlying instrument
  timbre, so `SetNoise` survives the following `NoteOn` event and disabling it
  restores the selected tonal waveform instead of leaking noise into later notes.
  Modern SFX steps also carry route-derived stereo pan. The extractor computes
  it from the original signed left/right DSP volumes, the sequencer emits typed
  `SetPan` events, and the mixer applies checkpointed per-voice channel gains
  instead of duplicating one mono sum into every host channel.
  Route-derived echo ownership is now carried as typed `SetEchoSend` events.
  The modern engine renders a deterministic one-frame stereo delay with a
  checkpointed tail and handles typed echo volume, feedback, and enable-mask
  parameters. FIR coefficients and raw DSP echo-memory addressing remain
  explicitly unsupported. The trace-backed `1:01` program is now checked into
  the catalog and exercises echoed and dry voices in the same sound.
  SFX harvesting now bounds each command at the next transition in its source
  slot, preventing later retriggers from being folded into one false program.
  When SPC channel snapshots identify the owning voice, strong DSP writes from
  other music voices are retained as context rather than promoted into the SFX.
  The 5000-frame combined route now resolves all 306 observed SFX commands to
  typed catalog programs with zero fallback commands. Its locked modern render
  digest is `627bba52...0b8f`, and `check_modern_audio_route.py` can enforce this
  with `--require-zero-unknown-sfx`. Trace frames record the exact known and
  unknown `(bank, id)` pairs instead of aggregate counts alone. Repeating
  program `2:0c` now retains its complete three-frame 42-to-47 slide; shorter
  route observations are modeled as retrigger interruptions of that program,
  rather than being mistaken for separate pitch-phase variants.
  APUI port 0 is consumed only by music sequencing. It is no longer duplicated
  as synthetic SFX bank 6, and checkpoint-restored `music.apui00` participates
  in music selection when the live queue port is empty. A checkpoint sweep at
  100k-frame intervals confirmed the false `6:01` note is gone and exposed the
  next real coverage frontier: 69 additional SFX IDs across later route segments.
  Focused 1000-frame segments at checkpoints 200k, 300k, and 400k now also pass
  with zero fallback commands. Their deterministic digests are respectively
  `3e442f2c...2d6c7`, `34a4ac8e...e231a`, and `fcf55175...d7805`.
  Context-only command `2:83` is represented by an empty typed program, so it
  remains coverage-accounted without inventing a generic fallback note.
  The complete 100k-through-1,000k checkpoint sweep now passes across 10,000
  sampled later-route frames with 464 known SFX commands and zero fallbacks;
  its combined deterministic digest is `4295f357...9d80a` with promoted music
  tracks rendered at the checkpoint windows. Context-only and
  no-key-on commands remain typed no-ops, including the active-owner suppression
  variant of `1:45`, while the same command still selects its audible program
  when that owner context is absent.
  A subsequent every-frame scan initially found 6,930 fallback frames across
  169 unique `(bank, id)` programs outside the sampled windows. Checkpoint-aware
  focused harvesting lifted 151 audible programs and retained 18 observed
  context-only/no-key-on commands as explicit typed no-ops. Extending the audit
  through the route's true final frame found three additional audible programs.
  After promoting all 172, the complete frame-0-through-1,073,092 standard route
  contains zero unknown SFX commands and never enters the generic fallback.
  Replay audio tracing now keeps one `ModernAudioSequencer` across frames;
  previously it reset on every log line and falsely treated held commands as
  new events. Active music detection now includes `last_music_control`. Track
  `scripts/extract_modern_music_catalog.py` reconstructs route-observed music
  notes from focused DSP traces as an offline authoring step, including voice,
  key-on offset, key-off duration, pitch, instrument, volume, and pan. Track
  `0x01` now carries the extracted 52-note, five-voice, 277-frame phrase after
  its observed six-frame lead-in; the earlier two-note placeholder loop is gone.
  Track `0x0b` now carries a separately reviewed 39-note, eight-voice,
  267-frame phrase after its observed 25-frame lead-in. Overlapping SFX key-ons
  were excluded from that promotion using voice ownership and instrument
  evidence. Track `0x03` now carries 122 reviewed instrument-10 score notes
  after its observed three-frame lead-in. Nested SPC channel ownership proved
  the previously included hard-panned instrument-1 notes were SFX. The extractor annotates notes
  within 12 frames of SFX commands with the command frame and exact delta; that
  made the overlapping command bursts on instruments 2, 11, 13, 18, and 20
  auditable without silently deleting coincident score notes. All 32 music
  tracks encountered on the standard route are now lifted; no route-observed
  track uses the heuristic music fallback.
  `scripts/compare_modern_music_route.py` is the event-parity gate for promoted
  tracks. It compares oracle and modern notes by absolute route frame, voice,
  mapped pitch, instrument, volume, and pan. The focused `0x01`, `0x03`, and
  `0x0b` captures all pass; the gate already caught and forced promotion of four
  initially omitted `0x01` notes rather than accepting a truncated phrase.
  Replay traces also render a persistent `ModernAudioEngine` beside the DSP
  oracle and record modern sample hash, peak, active/ignored event counts, and
  left/right energy. `scripts/check_modern_audio_route.py` rejects silent
  note-on or active-voice frames, unsupported events, single-voice pan reversal,
  and deterministic hash drift. The current focused route digests are
  `5cf13b85...afee1` for the 900-frame `0x01` capture and
  `79d5b892...b6c3e` for the 1320-frame `0x0b` capture. The 3500-frame combined
  route through promoted track `0x03` is `04b2e8a5...b3d2` after removing the
  duplicated instrument-1 SFX notes from the music catalog.
  Later-route track `0x16` is now lifted from its true frame-598693 transition
  through frame 600755 rather than from a checkpoint-local midpoint. Its
  instrument-9 score contains 735 exact notes across six voices after a
  three-frame lead-in. The continuous extraction gate caught and replaced an
  earlier checkpoint splice that had omitted 25 notes at frame 600000. Exact
  event parity and zero-fallback rendering now pass across the 2077-frame
  focused window with digest `c176797b...9ff2`.
  Track `0x11` now covers its bounded frame-106653 through frame-112076
  lifecycle with 523 exact score notes on instruments 9, 10, and 17 after a
  three-frame lead-in. Music extraction now reads authoritative nested
  `queue.sfx_channels` ownership, which removed 446 overlapping SFX key-ons
  from the candidate score. Typed note-origin events mark music versus SFX in
  replay traces, so event parity no longer guesses ownership from instrument
  IDs. Music durations and engine voice lifetimes are `u16`; the route includes
  sustained notes up to 416 frames without clipping. Exact event parity and
  zero-fallback rendering pass across 5438 frames with digest
  `967af57a...f9e0f`.
  Track `0x09` now covers its bounded frame-504921 through frame-506301
  lifecycle with 415 exact score notes across all eight voices on instruments
  9, 10, 17, 20, and 24 after a three-frame lead-in. Replay note tracing now
  assigns pan sequentially to each key-on, so same-frame music and SFX reuse of
  one voice retains two distinct origins and pans. Its exact event-parity and
  zero-fallback render gate spans 1393 frames with digest
  `56c7aaec...8e77c3`.
  Track `0x02` now covers its bounded frame-503431 through frame-503639
  lifecycle with 125 exact score notes across all eight voices on instruments
  11, 17, and 19 after a four-frame lead-in. Its exact event-parity and
  zero-fallback render gate spans 221 frames with digest
  `072d6da9...515fde`.
  Track `0x1a` now covers its bounded frame-644752 through frame-650490
  lifecycle with 959 exact score notes across all eight voices on instruments
  10 and 14 after a three-frame lead-in. The full 5750-frame lifecycle contains
  733 independently cataloged SFX commands, all resolved without fallback;
  exact event parity and the deterministic render gate pass with digest
  `32f54c37...f610`.
  Track `0x19` now covers its bounded frame-20457 through frame-20866
  lifecycle with 53 exact instrument-10 score notes across six voices after a
  16-frame lead-in. Exact event parity and zero-fallback rendering pass across
  the 420-frame focused window with digest `114b8415...605ed`.
  Track `0x04` covers frame 296528 through frame 297710 with 296 exact
  instrument-18 notes across five voices after a four-frame lead-in; its
  1193-frame parity/render digest is `658a52d7...cebaa`. Track `0x0a` covers
  frame 335215 through frame 337088 with 58 exact notes on instruments 9 and
  15 after a three-frame lead-in, digest `5949fa56...27348`. Track `0x0c`
  covers frame 89603 through frame 91072 with 768 exact notes on instruments
  2, 11, 14, and 17 after a four-frame lead-in, digest
  `ccfecfe3...1da49`.
  Reviewed music notes are no longer expanded into tens of thousands of Rust
  struct-literal lines. `assets/audio/modern_music.tsv` is the compact,
  reviewable parity-authoring source; the `zelda3` build validates it and packs
  each note into a nine-byte runtime asset. `modern_music_catalog.rs` is now a
  small typed decoder over those included assets.
  Track `0x1c` covers frame 378760 through frame 381311 with 181 exact
  instrument-9 notes across five voices after a 32-frame lead-in, digest
  `ac08fb0f...e4822`. Track `0x17` covers frame 91415 through frame 91790
  with 32 exact instrument-14 notes after a five-frame lead-in, digest
  `0d446fde...387b`. Track `0x05` covers frame 44885 through frame 45625
  with 175 exact notes on instruments 9, 10, and 22 after a three-frame
  lead-in, digest `ae38f182...8dcc`. Track `0x14` covers frame 40939 through
  frame 43490 with 206 exact instrument-21 notes after a 62-frame lead-in,
  digest `83ea4d65...015a`.
  That longer track exposed a live-state priority bug: after APUI port 0
  cleared, stale queued/music-control fields could override the authoritative
  `last_music_control` active track and reset the modern sequence. Active-track
  selection now prefers `last_music_control`, with a focused regression test.
  Track `0x07` covers frame 49640 through frame 50381 with 63 exact notes on
  instruments 0, 9, 10, and 22 across six voices. Its first notes key on in the
  same frame as the live track command, so the packed sequencer now explicitly
  emits zero-lead notes at position zero; the focused regression and exact
  parity gate pass with digest `7ca476d0...1007e`.
  The remaining bounded tracks (`0x08`, `0x0d`, `0x0e`, `0x10`, `0x12`,
  `0x13`, `0x15`, `0x18`, and `0x1b` through `0x22`) also pass exact
  note-event parity. The compact catalog contains 19,390 reviewed notes across
  all 32 route tracks. A streaming, checkpoint-free render gate now covers the
  complete 1,073,092-frame standard route with zero unknown SFX, zero ignored
  events, no silent active voices, valid stereo pan energy, and deterministic
  digest `f1bb20b5...a291`.
  Sample-level waveform parity is not yet closed. Trace output now records
  modern-versus-DSP mean absolute difference, maximum difference, and exact
  sample count per frame. The first 1300 route frames establish a baseline mean
  absolute difference of 1433.48, maximum difference 15648, and 4428 exact
  samples out of 1,911,000. The modern renderer now decodes the original BRR
  instruments from the loaded SPC sample directory, retains exact DSP pitch
  words, signed stereo volumes, ADSR/GAIN registers, echo-send state, and
  intra-frame key-on offsets in the packed route catalog, and uses the SNES
  four-tap Gaussian interpolator. It also synthesizes on the original
  534-sample-per-frame DSP clock before applying the same nearest-neighbor
  output resampling as the parity renderer. On the same 1300-frame gate this
  lowers mean absolute difference to 1412.73 and raises exact samples to 14362
  (maximum difference 15335). The renderer now also applies Zelda's observed
  signed 96/128 master-volume stage, track-relative global DSP automation,
  RAM-backed echo delay, the eight-tap FIR ring and feedback writeback, the
  shared FLG-rate noise LFSR, pitch modulation, and sample-accurate music
  key-on/key-off and global-register events. Exact SFX pitch/stereo/ADSR facts
  are kept in a compact reviewed sidecar; captured key-on offsets are excluded
  because they vary with live scheduler phase and are not program invariants.
  With all of those structurally faithful stages enabled, the same 1300-frame
  gate reports mean absolute difference 1378.05, maximum difference 18790,
  and 14378 exact samples when using the static bank copy plus procedural
  fallback. The stricter trace oracle now supplies the live initialized SPC
  directory and BRR memory. On that source, the initial mean difference was
  1459.88 with 12376 exact samples. SFX programs now retain only stable
  command-relative frame delays and 64-sample scheduler-boundary indices;
  absolute replay frames and phase-dependent sample offsets are not runtime
  catalog data. The live timer scheduler converts those facts back into key-on
  offsets from its current timer remainder. That lowers the initialized-sample
  result to 1457.73 and raises exact samples to 13526. The 5000-frame modern
  route gate remains green with zero unknown SFX and digest
  `2fee7207...c273cdb`. At that checkpoint the suspected dominant gap was
  deterministic initialized-bank packaging plus residual dry-voice alignment.
  Subsequent waveform work proved the bank upload is already deterministically
  materialized and found two larger runtime gaps. The modern engine now retains
  the DSP pitch counter across key-ons, waits for the first BRR decode overflow,
  advances the counter for silent/noise/fallback voices, and marks semantic
  sequencer frames explicitly so quiet frames cannot reinterpret pending raw
  APU ports as new notes. Track `0x01` was also found to have been promoted from
  a prematurely bounded capture: its catalog now covers 240 notes through the
  complete 660-frame observed lifecycle, and track-relative voice-register
  automation preserves its live pitch and volume modulation. Coincident score
  notes were removed from SFX `1:01` and `1:2c`, and SFX variant matching now
  prefers the exact engine voice-ownership context. With exact signed DSP shift
  arithmetic, the first 1300 frames now report mean absolute difference 421.34,
  maximum difference 15428, and 466418 exact samples out of 1911000. Frame 303
  is phase-aligned with mean difference 4.01; the continued phrase at frame 668
  begins with mean difference 0.74. The 5000-frame route safety gate remained
  green with zero unknown SFX and digest `03192c0a...3c5b` while those residuals
  were being isolated. Tonal instrument IDs now map modulo the three tonal
  oscillators; explicit `SetNoise` is the only path to noise, so music instrument
  15 no longer aliases the noise waveform. Subsequent receipt-capacity, key-on
  snapshot, checkpoint, BRR-END, and live filtered-loop fixes close sample-level
  waveform parity on the complete 1,073,092-frame standard route: every rendered
  sample matches the DSP-compatible oracle. The independent modern route quality
  gate also passes all 1,073,092 frames with zero ignored events, zero unknown
  SFX, no sustained active-voice silence, valid pan energy, and deterministic
  digest `5f51f387...1cc0889`. Checkpointed BRR voices retain their live 19-sample
  Gaussian window and filter history, then decode subsequent blocks on demand
  from SPC RAM with bounded storage rather than relying on a static predecode
  cycle or a later checkpoint refresh.
- `ModernAudioSequencer` now sits before that engine in the modern backend. It
  consumes a typed, engine-authored command bus and expands `PlayMusic` and
  panned `PlaySfx` commands into `SetTempo`, `SetEnvelope`, `NoteOn`, and
  `NoteOff` events. Gameplay's last-write-wins sound latches become typed
  commands at NMI; APUI bytes are projected from the same commands only for the
  legacy oracle, diagnostics, and old save formats. The default modern path is
  therefore no longer deriving primary intent from APUI or DSP register bytes.
  Same-voice catalog steps are scheduled across their declared frame durations
  instead of being flattened into one frame, while steps on different voices
  still begin together. Per-voice lifetimes and pending steps are checkpointed,
  and expired voices no longer contaminate context-dependent variant selection.
  The full modern sequencer and renderer state—including music phase, oscillator
  phase, envelopes, slides, and echo history—is now part of audio save states.
  Snapshot v6 stores `ModernAudioRuntime` directly: its typed command queue,
  sequencer, renderer, canonical sample-bank ID, and compatibility-only C-save
  fields, without embedding APUI transport bytes or a 64 KiB SPC address
  space. Version 1-5 payloads remain readable, and legacy payloads are decoded
  once through the compatibility gateway before entering typed runtime state.
  The modern sequencer and renderer are checkpointed; backend selection remains
  host configuration and returns to Modern unless the host reapplies an override.
  Legacy checkpoints still decode through the prior audio snapshot schema, with
  modern state initialized only where those older files had no such payload.
  The DSP-compatible path remains the exact parity oracle and still feeds trace
  diagnostics. The playable Modern backend no longer advances the legacy SPC
  control interpreter or DSP: queued `EngineAudioCommand` values go directly
  through `ModernAudioSequencer`, and all host PCM is rendered by
  `ModernAudioEngine`.
  Modern callbacks do not require a live `SpcPlayer`; regression tests pin both
  that ownership boundary and the legacy SPC/DSP state across Modern callbacks.
  Modern acknowledgements are typed command batches. Normal NMI and
  `zelda_is_music_playing` inspect that semantic state instead of reading APUI
  ports, while ordinary reset/C-style load restarts Modern sequence and
  renderer state at a defined command boundary instead of retaining future
  voices or echo state.
  Normal Cargo builds are now modern-only: the `SpcPlayer` module, raw pointer,
  DSP interpreter, and legacy DSP snapshot payload are compiled only with the
  `audio-oracle` feature. `zelda3-bin --features audio-oracle` retains trace and
  parity diagnostics; a default build rejects DSP backend selection and
  `--audio-trace-log` instead of silently constructing the legacy runtime.
  The fixed C save-state byte layout keeps inert APU/DSP-sized compatibility
  slots, but modern-only `AudioState` no longer owns raw SPC RAM. Its
  `ModernAudioRuntime` owns a typed 16-entry command history, pending batch,
  input batch, acknowledgement batch, sequencer, and renderer. Saved-music
  ports and the startup timer live in an explicitly compatibility-only record;
  C-style loads import those fields through the legacy gateway.
  Raw song-bank uploads and the shadow RAM used by trace tools now compile only
  with `audio-oracle`.
  Audio checkpoints use the portable version-6 schema in both build modes. Its
  required payload contains the typed modern runtime, canonical sample-bank ID,
  compatibility record, and configuration state. Oracle builds append
  legacy SPC/DSP continuation as an explicitly flagged opaque sidecar; normal
  builds can load those checkpoints by ignoring the sidecar, and oracle builds
  can load normal checkpoints by constructing a fresh diagnostic oracle.
  Versions 2-5 migrate in either build. Historical version-1 oracle checkpoints
  migrate through an oracle build.
  `scripts/check_audio_build_modes.py` is the build gate: it checks the
  oracle-enabled binary, builds the default binary, and rejects any default
  executable that still exports `spc_player` or `SpcPlayer` symbols.
  Audio checkpoint blobs carry a `Z3AU` header, explicit format version, payload
  length, and capability flags. The loader still accepts supported pre-header
  payloads and rejects malformed or unknown future versions deterministically.
  A semantic conformance matrix now complements the full-route DSP oracle gate.
  Its named scenarios inject typed engine commands directly—without APUI bytes
  or replay-frame coordinates—and cover catalogued SFX, overlapping effects, music
  interruption, sample-exact snapshot continuation, and ordinary music restore.
  A shrinking property-based command-state generator now complements those
  named cases. It draws legal batches from the full set of lifted music tracks
  and catalogued SFX commands, mixes in clear/stop/control transitions, queue
  backlogs, render gaps, and arbitrary snapshot round-trips, then compares a
  continuous modern engine with the restored engine event-for-event and
  sample-for-sample across 256 generated programs per test run. Each rendered
  frame also requires bounded voice counts, zero unknown or fallback SFX, and
  zero events ignored by the renderer. Proptest retains and shrinks any failure
  into a small command sequence that can be promoted to a named regression.
  Every rendered scenario requires zero unknown SFX programs, zero heuristic
  fallbacks, and zero ignored modern-renderer events. The route oracle remains
  the independent sample-level proof, while this matrix prevents that proof
  from becoming coupled to the timing of one recorded route.
- `ModernSfxCatalog` adds the first data-backed SFX programs for menu cursor,
  sword, rupee pickup, door/stairs, and damage. Known `PlaySfx` commands expand
  into typed program events including envelope, note duration, pitch slide, and
  noise selection; unknown commands still use a marked heuristic fallback.
  Audio trace output now reports modern SFX known/unknown counts and the modern
  program hash beside the existing DSP/sample parity fields.
  SFX authoring now has one versioned source of truth:
  `assets/audio/modern_sfx.json`. It contains all 342 reviewed programs, their
  contextual variants and steps, 570 exact DSP records, and 80 intra-step pitch
  events. The build rejects an unknown schema, unreviewed runtime programs,
  invalid voices, and invalid waveform names, then serializes the validated
  document into a compact bincode asset in Cargo `OUT_DIR`. The runtime embeds
  only that binary and lazily reconstructs the typed lookup tables; it does not
  parse JSON, load ROM data, or compile generated SFX struct literals.
  `modern_sfx_catalog.rs` now retains only the stable types, asset decoder,
  lookup policy, hashes, and focused regression tests. The former generated
  full-route Rust catalog and separate DSP/pitch TSV inputs have been removed.
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
- Focused audio traces now accept
  `ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME_RANGE=start:end`, so a harvested SFX
  window can include DSP writes from every frame in the command window instead
  of only the command's first frame. A bounded real-route harvest over the first
  40 SFX occurrences promoted conservative trace-backed catalog programs for
  `PlaySfx` commands `0:03`, `2:0a`, and `2:24`; context-heavy `1:01` and
  repeated commands `1:2b`, `1:2c`, and `2:0c` remain harvest artifacts until
  their command-owned variants can be named and selected safely.
- The SFX extractor now labels each keyed voice as `owned_by_command`,
  `weak_update`, or `carried_over` based on whether source/envelope ownership
  writes occurred inside the focused command window. Rust snippets and variant
  signatures only use command-owned steps; weak/context voices remain in JSON
  evidence so catalog promotion no longer absorbs incidental music or ambient
  voice activity.
- Repeated command-owned programs are now grouped as explicit variants instead
  of marking the whole SFX command ambiguous. Variant groups get stable
  signature hashes, generated names such as `trace_sfx_01_2c_v00`, and context
  signatures carrying source slot plus owned/context voice masks. The checked-in
  modern catalog and sequencer understand this metadata through
  `ModernSfxContextSignature` and contextual lookup, while broad harvest output
  remains reviewable under `target/` before any large variant set is promoted.
  With this grouping, the first 40 real-route focused SFX occurrences harvest as
  `programs=7`, `lifted=7`, and `gaps=0`.
- `scripts/extract_rom_audio_catalog.py` is the offline ROM/asset-pack bridge
  for modern audio source discovery. It reads `zelda3_assets.dat` sound-bank
  payloads extracted from the ROM, reconstructs SPC RAM upload blocks, reports
  written ranges, pointer-table candidates, technical sequence names, and can
  cross-link route-harvested modern SFX evidence. This is intentionally an
  import-time tool: the modern runtime does not depend on ROM/SPC/DSP structure
  to play audio.
- `scripts/decode_rom_audio_sequences.py` is the second offline source-discovery
  stage. It consumes `rom-audio-catalog.json`, reconstructs the sound-bank RAM
  images from the asset pack, rejects pointer-table-like candidates, and walks
  bounded non-emulating bytecode previews for importer review. Route-provenance
  SFX targets also get a route-gated compact SFX decoder that understands the
  SPC SFX bytecode shape (length/volume prefixes, instrument commands, notes,
  pitch slides, terminators, and loops) before applying generic pointer-table
  rejection. A real decode over the current extracted catalog emits
  `target/rom-audio-catalog/decoded-sequences.json` and `.md` with `658/875`
  candidate streams decoded (`419` high confidence, `139` medium, `100` low,
  `217` rejected).
- Audio trace JSON now includes diagnostic `sfx_channels` snapshots with each
  SPC channel's resolved SFX sound id and sequence pointer. The SFX extractor
  preserves those pointers as `sequence_provenance`, the harvest report counts
  sequence links per program, and `scripts/build_modern_sound_index.py` joins
  harvest provenance to `decoded-sequences.json` to emit
  `modern-sound-index.json`/`.md`. The current bounded first-40-occurrence
  route harvest links all `7/7` focused SFX programs with `0` ambiguous and `0`
  unlinked programs. This is still an offline authoring bridge: the authored
  catalogs do not load ROM data at runtime.
- `scripts/promote_modern_sound_assets.py` promotes that linked evidence into a
  modern-owned asset manifest. It chooses one canonical primary decoded sequence
  per command/variant, retains alternate ROM-derived sequence evidence for
  reviewer comparison, and keeps the playable `modern_program` steps separate
  from ROM/SPC/DSP evidence. The current first-40-occurrence manifest emits
  `target/modern-sound-assets/modern-sound-assets.json` and `.md` with `33`
  assets, `33/33` primary sequence links, `8` review-ready assets, `25`
  low-confidence needs-review assets, and `0` blocked assets. This remains an
  offline evidence/promotion artifact. Review-ready programs are copied into
  the canonical `modern_sfx.json` authoring source; low-confidence evidence
  remains outside the runtime asset until reviewed.

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
