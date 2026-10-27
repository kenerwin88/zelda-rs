# Lagless audio: note capture and the exact-reference boundary

**A full-route player-audibility verdict is not verified. The previous mixed-PCM SNR ranking is withdrawn: it cannot distinguish a shifted song from an extra or changed sound.** The repaired cold reference gets past the HDMA panic, then first differs from the original PCM at host **7164**. That prevents treating later native notes as the original soundtrack. The cold candidate consumed the entire input tape, but its audio-context inventory has no ending-module frames; input completion is not a verified playback of the full route. No whole-route extra/missing-note count, category verdict, or “20 most audible” listening set is claimed.

## Reference repair and current boundary

The previous harness enabled ROM startup timing in `load_default_play_state()` but never supplied the cartridge bytes. `NativeDialogueCallerClock::advance` reads ROM-backed HDMA tables through `native_hdma_table_byte`; an empty cartridge caused the panic at host 2329. The capture plan now requires `rom` for the reference and loads those bytes before frame zero. No dialogue clock, timing price, marker, or gameplay implementation changed.

The CPU loop follows the play frontend's order: `zelda_run_frame`, `zelda_render_audio`, unused-audio discard, then CPU display publication. It creates no GPU resources. The smoke runner follows that order too, but its default loader is **ROM-less**, with fixed 735-pair output callbacks. As `docs/parity/state-ledger.md` states, completion of that runner does not establish ROM parity. Its completion is a regression result, not an original-audio proof. The candidate must also demonstrate route milestones; consuming every logical input alone is insufficient.

The first repaired cold probe passed the old panic and checked PCM hashes until host 7164, where it stopped with:

- Native SHA-256: `7c33ea3a7452dc74aefb4729fb38e7304cffdf42b22b141830e096eccf46a05d`
- Original SHA-256: `ebab2d074b171d729d1c2fe0680e85a22dc895ebfa1cf1b94b751e51ee1c8a05`
- Input: `$0040`; stereo pairs: 533.

The independent boundary capture reproduced the same hashes. `reference-proof2.json` independently checks **all 7,164 matching frames (3,819,211 stereo pairs)** and the failing frame. Context at failure is module `$09`, submodule `$00`, room field `$0104`. It also verifies every selected PCM block and rebuilds all emitted note snapshots from **583,110 raw DSP writes**. The verified prefix has **1,802 key-ons and 3,145 key-off writes**; three more key-ons belong to the failing host and are excluded from the verified prefix. No source state, source RNG, forced APU reads, timing receipts, or cached audio are installed. Failed reference captures have no success `complete.json`; the failure journal names the verified prefix and mismatching frame. A matching prefix must not be described as a full-route original. `boundary-command-context2.json` retains nearby source iterations and native publications: source iteration 6437 writes port 2 `$31` and port 3 `$0B`; no corresponding nonzero native publication appears before the hash failure. This locates a command boundary, not a proven root cause.

## Note observations

`--render-audio-cpu` now saves compressed `dsp.jsonl.zst`, `notes.jsonl.zst`, and `hashes.jsonl.zst`. The existing `ZELDA3_DEBUG_PCM_WINDOWS` plan switch still goes through `zelda3::debug_env`; no new environment switch was added.

The DSP stream comes from `AudioEventFrame.events[].parity_dsp` returned by the **actual live audio callback**. It is not reconstructed from gameplay commands or run through another APU. Every register write retains absolute native sample coordinate and SPC phase (1/32 sample), host, input iteration, module/submodule/room, and music latches. Every nonzero KON/KOFF voice bit records voice, source number, pitch, signed left/right volume, ADSR and gain as written. Repeated key-off writes remain in the raw edge inventory. Later pitch, envelope, source and volume writes remain in the full DSP stream.

`scripts/lagless_audio_notes.py` independently rebuilds note snapshots from the raw writes, checks the Rust-emitted snapshots when available, and associates first key-off/retrigger boundaries and subsequent register contours. Its sequence alignment uses voice/source/pitch exact anchors and retains insertion, deletion and substitution results. It accepts explicit monotone matched-song command anchors, and separates absolute shift, command-relative shift, and timing relative to the earliest exact note pair across all eight voices. One late voice cannot disappear behind its own independent time origin.

This is an observation/comparison tool, **not a completed category attribution study**. Voice ownership and cold-run command identities still need validation against the timing study. Its provisional relative timing counts cannot be called music/SFX audibility counts. There is no verified breakdown for port-2/3 SFX, the 636 native-only dialogue edges, the 164 non-`$FF` source-only commands, or song endings versus fanfare.

## Evidence and checks

All new artifacts are under `target/lagless-audio-pcm/`:

- `reference2-probe.log`: the repaired harness's first PCM mismatch at host 7164. Its interrupted compressed buffers are diagnostic, not a finalized full-prefix proof.
- `reference2-boundary/`, `reference2-boundary.log`, `reference-proof2.json`: independent cold boundary capture, per-frame proof, note verification and failure evidence. Two three-second `boundary-*.wav` files compare cached original to cold reference for diagnosing reference integrity. The first difference is 353 stereo pairs into host 7164; maximum signed-16-bit error in the retained window is 95. These are not an original/lagless listening pair and do not supply an audibility verdict. Disposable raw PCM was deleted after verified WAV extraction.
- `lagless2-full/`, `lagless2-full.log`: completed cold input-tape capture: **1,497,140 callbacks / 1,473,123 logical inputs**, 799,472,760 stereo sample pairs, 2,271.184 seconds. No full-route raw PCM was stored. The independent verifier checked **119,058,409 DSP writes**, **557,573 key-ons**, **1,086,284 key-off writes**, and every callback hash-journal record. All Rust note snapshots matched reconstruction from the raw DSP stream. `candidate-note-verification2.json` binds the compressed streams and lists their complete DSP-active context inventory. There are **zero DSP-active frames in ending modules `$19/$1A`**; the source timing inventory has **651 commands in those modules**. The last observed DSP context is module `$0E`, submodule `$02`, room field `$0104`. This capture is full-input-tape coverage, not certified full-route coverage.
- `timing-category-inventory2.json`, `source-route-audio-contexts2.json`: read-only timing-study inventories; the requested 636 dialogue edges and 164 non-`$FF` source-only commands are confirmed as command counts, not audible-note findings.
- `verification2.json`: final checks, source/artifact hashes and explicit unfinished scope.
- `notes2-ledger-check.log`: default **200,000 records and EOF matched unchanged**, 583.772 seconds. Existing logical/state ledgers were only read.
- `notes2-python-tests.log`: **24 passed** audio analysis tests, including missing-note alignment, pitch substitutions, polyphonic KON, repeated KOFF, uniform song shifts, and one delayed voice.
- `notes2-callback-tests.log`: the existing 1,000-callback native/product resampling equivalence test passed.
- `notes2-default-build-final2.log`, `notes2-feature-off-build.log`: ordinary default and feature-off builds; warning-free.

`report-v1.md` preserves the previous report. The old `listen/` WAVs remain exploratory evidence; their waveform ranking is withdrawn. No new `listen2/` index or WAVs are represented as a whole-route top-20 result.

The read-only oracle ledger hash remains `8b849c8ce1040e65016daeccf311768a2b7cfd1017ec7f7f6f48899a1903c6d7`; initial SRAM and cartridge hashes are bound in `reference-proof2.json`.

Sparse PCM has a 256 MiB per-capture cap; full-route DSP and note streams are compressed. Disk space was checked before long runs. The study is approximately **735 MiB**, under 3 GB. No GPU run, Snes9x execution, oracle-cache write, ledger modification, `cargo fmt`, commit, or push was performed.

## Reproduce the boundary

Build:

```sh
ZELDA3_ASSETS_DIR=/Users/missingno/Documents/zelda3-rs/generated/zelda3_assets \
  cargo build --offline --profile parity -p zelda3-bin
```

Use a new JSON plan with `output`, `ranges: [[6984,7164]]`, and `rom: "/Users/missingno/Documents/zelda3-rs/zelda3.sfc"`. With isolated `ZELDA3_SAVE_DIR` and `ZELDA3_DEBUG_PCM_WINDOWS` pointing to that plan:

```sh
target/parity/zelda3 --render-audio-cpu <cache>/oracle-av-hashes.jsonl <cache>/initial.srm 7165
```

It must stop on a PCM mismatch rather than certify later notes. For a successful reference capture, the note tool requires the read-only oracle ledger and independently checks every hash before comparing notes. Completed prefix analyses still do not establish a full-route verdict.
