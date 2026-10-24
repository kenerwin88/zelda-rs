# Native lagless input replay and SPC census

Worktree: `zelda3-rs-codex2`, branch `codex/lagless-replay`, based on
`480d0494587840a04c2cd11862da314b676668dd`. All executions in this report
are renderless. No timing-engine or checkpoint-layout fields were changed.

## Native movies

`zelda3 --record-lagless <directory>` records the shipped
lagless game to `<directory>/input.z3lag`, including feature-off release builds.
The diagnostic `ZELDA3_RECORD_INPUT_SESSION=<directory> zelda3` also works
when `parity-debug` is enabled; its switch now uses `debug_env::var_os!`.
Session metadata, initial SRAM, crash checkpoints and live input events remain
available; `--rom-timing` still records its existing cartridge-frame input script.
Developer warps remain forbidden during recording. A native session must
start at a fresh boot; a warm replay/developer state is rejected.

The version-1 wire format is little endian:

| Bytes | Meaning |
|---|---|
| 8 | `Z3LGIN01` magic/version |
| 8 | Embedded asset pack FNV-1a-64 |
| 8 | Initial SRAM FNV-1a-64 |
| 4 + 4 | Audio samples per call and channel count |
| 4 + variable | JSON launch-settings length and settings |
| 8192 | Initial SRAM, embedded rather than an external path |
| 2 per iteration | Controller word in the engine/frontend bit order |

Launch settings carry language, aspect width and audio configuration. External
MSU audio is rejected: its files would be additional inputs. Version, assets,
SRAM checksum, settings bounds and partial input words are checked on load.
Audio shape bounds are 1–65,536 samples and 1–32 channels per call; low-rate
and multichannel requests are retained rather than assuming a stereo 735-sample
device. No PCM is stored in the movie.
Every input word is flushed before its iteration runs, allowing recovery of
complete words after a gameplay panic. FNV checks identify accidental changes;
they are not cryptographic authentication. Reproducibility claims use the same
engine build; a new engine's behavior is not guaranteed by an unchanged asset hash.

No RNG outputs or APU answers belong in a native movie. In
`crates/zelda3/src/misc.rs:268`, native RNG adds the frame counter to its seed,
then shifts/XORs it, returning a deterministic byte and false carry. Cartridge
replay overrides that algorithm with beam-counter-dependent bytes and carry.
Native `$2140` reads come from the persistent SPC/command-acknowledgement state,
advanced by the engine's audio calls. Replay therefore advances audio even
without a sound device or renderer. Host wall time and audio-device underflow
are not inputs to the game.

```sh
BIN=target/parity/zelda3
LEDGER=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
$BIN --convert-lagless "$LEDGER" "$C/initial.srm" target/lagless-proof/full.z3lag
$BIN --replay-lagless target/lagless-proof/full.z3lag --renderless \
  --state-hash-every 100 --save-state-at 700000:target/lagless-proof/mid.json \
  > target/lagless-proof/native-a.jsonl
$BIN --replay-lagless target/lagless-proof/full.z3lag --renderless \
  --state-hash-every 100 > target/lagless-proof/native-b.jsonl
$BIN --replay-lagless target/lagless-proof/full.z3lag --renderless \
  --load-state target/lagless-proof/mid.json --state-hash-every 100 \
  > target/lagless-proof/native-resume.jsonl
```

Omit `--renderless` for windowed playback. Already-rendered PCM is converted to
the playback device's sample/channel count without advancing engine audio again.
Endpoint interpolation and mono/stereo mixing have unit coverage. Windowed
playback and live recording UI were not executed in this GPU-free task.
Renderless playback still retires the display
publication and audio queues as play does. Replay SRAM writes are isolated in
a temporary directory. `--frames N` is an absolute iteration count, including
when loading a save; it cannot exceed the movie length. Hashes label zero-based
iterations **after** execution. Hash interval must be positive and the last
newly executed iteration is always included. `--save-state FILE` saves the final boundary;
repeatable `--save-state-at N:FILE` saves after N completed iterations.

The portable save is deliberately a **reconstruction save**, not a bincode
`ZeldaState` snapshot. Existing snapshots skip `lag_free` and the polygon/thread
continuations (`zelda_rtl.rs:10524` onward), among other fields. The save stores
the complete movie's hash, next iteration and v2 canonical hash. Loading
re-executes the prefix silently and verifies that boundary before continuing.
This preserves omitted runtime state without modifying the timing machinery,
but load time is linear in the saved position. The movie is still required.
The v2 hash excludes hidden audio, display and timing state; matching v2 hashes
are not proof of pixel/PCM equivalence or complete-state serialization.

## Determinism and comparison with the logical ledger

The converter writes 1,473,123 inputs in 2,954,582 bytes (SHA-256
`290b88711cfb486c3ac1c28a2e75191b10ce63efe4d2e4af81a469c2b1e7ce1e`).
Engine frame k receives iteration k+1's
bit-reversed joypad word, matching `logical_replay.rs:1350`; the last frame
receives zero when there is no next iteration. No cartridge RNG, APU values,
seeds or WRAM images enter the movie.

The separate diagnostic `--logical-replay ... --native-rng --live-apu-reads
--frames 6000 --report 8` uses the same pure-input policy. Its first ordinary
WRAM divergence is iteration **4363**, module `09/00`, address `$0FA1`
(RNG seed). The ledger's first RNG draws occur there: `$22` and `$45`, both
with clear carry; the engine finishes that iteration with seed `$DD`, versus
the cartridge's `$45`. Later addresses diverge as that random stream affects play.
The first cartridge APU read is iteration 1916 (a bank-upload handshake), not
the first ordinary gameplay divergence. This comparison retains the census's
representation exclusions, lag-carried differences and separate polygon
classification: it is not a claim that every raw WRAM byte matched before
4363. The diagnostic reports 4,337 clean iterations out of the first 6,000,
and 26 polygon-WRAM differences among 730 polygon iterations.
Converting the cartridge pad stream does not establish quest completion in
native play: RNG, APU answers and iteration timing have different ownership.

Both cold replays completed **all 1,473,123 iterations**, exit 0. Each emitted
14,733 v2 hashes (every 100 iterations, plus the final iteration); their JSONL
outputs are byte-identical, SHA-256
`35e9c77cbab0e3b390503f1b4426dc8a5f1f19f798ead53ae3e0c233d8f7b92e`.
Final state hash: **`6e574bd945180f6b`**. These are sampled v2 hashes, not an
every-iteration full-machine-state comparison.

The reconstruction save after 700,000 iterations holds v2 hash
`efa19480ff1198e6`; the resume run verified this boundary and completed with
exit 0. **All 7,733 resumed hash samples match the cold-run suffix**, including
the final hash; resumed JSONL SHA-256
`5044582c38d9882a5f0fa86bfae0ad9cabdaea4fcd7ca7ddc985b76855be56a4`.
Evidence: `native-a.jsonl`,
`native-b.jsonl`, `native-resume.jsonl`, `mid.json` and `native-proof.json`
under `target/lagless-proof/`.

## Audio clock in the shipped game

`audio.rs:957` (`zelda_render_audio`) pops queued commands and advances audio
once per frontend call. `audio.rs:981` normalizes 735 samples at 44.1 kHz
(or typical other output-rate requests) to **534 native DSP samples**; requests
from 400 through 534 advance verbatim. The driver clock is persistent, not
reset per frame. `spc_driver_clock.rs:592` (`advance`) advances
`534 * 32 = 17,088` SPC cycles for a normal call, executes the SPC program,
collects DSP events and retains outstanding transfers/port writes.

The ROM-less launch has no startup CPU owner (`main.rs:load_romless_play_state`),
so `audio.rs:1020-1037` selects `clock.advance`, rather than
`advance_at_cpu_boundary`. Song-end polls (`audio.rs:889-916`) return the
current internal APUI00 latch when no original-timing authority is live.
Thus the ordinary lagless audio clock does **not** require executing the
exact-timing CPU cycle model. It already has a fixed native audio window.

The gameplay latch is populated at frame entry (`rtl_diagnostics.rs:353`)
with `zelda_is_music_playing()` (`audio.rs:1135`): without MSU this is the
zero/nonzero music acknowledgement from the internal driver queue. It is a
boolean observation of the preceding audio window, not a replay of the raw
cartridge port byte. The receipt callers only test zero/nonzero. Pendant
receipt polling is at `ancilla/items.rs:526` (also the resumed prefix at 1450);
the crystal sparkle poll at `ancilla/items.rs:745` issues music `$1A` and
transmutes the receipt to a rising crystal when the song ends.

It still shares cartridge-oriented **transport positioning**, rather than
being a wholly independent native clock: `spc_driver_clock.rs:1313` maps
frame-indexed nominal V225/H84 NMI port-write positions into SPC cycles;
`advance` also retains the SPC lookahead/host-port transport. Song-bank transfers
use their existing host-transfer state (`audio.rs:1265-1284`). Removing all
cartridge scheduling assumptions would require an explicit native frame origin,
command timestamps within that native window, and preserved upload handshake,
SPC timer, acknowledgement and pending-event ownership. It would change song-end
poll timing and needs focused transport/upload regression coverage plus native
audio validation. No guessed timing offset or audio-clock change was made.
The polygon thread separately uses the cycle ledger to schedule its lagless
phases (`rtl_poly.rs:294-319`); this is not the ordinary SPC clock's owner.

A device-independent native clock would also separate its simulation budget
from the requested output-buffer size: the current 400–534 special case can
advance a different budget for unusual device rates, and is also used for
partial-window SPC probes. The movie records the actual window, so this does
not introduce an unrecorded replay input. If the intended product contract is
exactly 60 iterations/s and 32,000 DSP samples/s, a fractional accumulator would
schedule 533, 533, 534 samples rather than 534 every iteration. That is an
explicit cadence/poll-timing change requiring validation, not a missing RNG/APU
recording field; it was not made here.

## Full APU census

The cartridge-answer full run uses the retained clean-main binary; the live-SPC
full run uses `target/lagless-proof/final-zelda3` (SHA-256
`7d8ed9390314c4b3ad4294b6854c725b28be7df9dff59475385dd0ecaa72ad31`).
Engine/audio/timing code is unchanged. Both read the same source ledger, cache
SRAM and cartridge RNG stream. The execution-policy difference is whether APU
polls use cartridge answers or the engine's SPC-derived latch. Command logs,
summaries and provenance are retained under `target/lagless-proof/`.

The live run exposed severe comparator overhead once many WRAM bytes differed:
`wram_image()` now exports an owned 128 KB image, and the old histogram/lag loops
called it for individual differing bytes. This looked stalled but was progressing
at only a few iterations/s. `logical_replay.rs:1588` now exports once per
iteration, after game/audio work, and reuses that immutable image throughout the
read-only comparison. No game writes or timing rules move across this boundary.
The divergent seeded span 142,000–143,999 has byte-identical old/new output
(SHA-256 `5531651a3cb0f39853af578cff07c2982fc46456e6167753b22f4e384d6ea4d2`).
An additional optimized cartridge-read run matched the original output through
234,000 iterations and was deliberately stopped there; it is not a second full
cartridge-read run. The original full census and unoptimized live prefix are
retained. See `census-optimization-proof.json`.

Seeds contain WRAM/VRAM/CGRAM/OAM/SRAM and PPU registers, not the cartridge's
complete SPC machine. `seed_memory_from_foreign_image` (`zelda_rtl.rs:14831`)
rebuilds music from WRAM save ports. Consequently a reseeded live-SPC segment
measures this census's reconstruction policy, not exact restoration of the
cartridge's elapsed SPC state. The native movie/resume proof above does not use
these seeds and reconstructs its own continuous native SPC execution.

```sh
$BIN --logical-replay "$LEDGER" "$C/initial.srm" --report 0 --seed-every 2000
$BIN --logical-replay "$LEDGER" "$C/initial.srm" --report 0 --seed-every 2000 --live-apu-reads
python3 scripts/logical_census_summary.py target/lagless-proof/census-rom.txt
python3 scripts/logical_census_summary.py target/lagless-proof/census-live.txt
```

The cartridge-read census completed all 1,473,123 iterations in 739 segments,
with **no ordinary WRAM divergence and no panics**. The summary reports all
738 non-intro segments fully clean (1,471,123 iterations); the 2,000-iteration
intro segment is separate. It reports 26,500 polygon-thread iterations with
5,931 polygon-WRAM differences, plus 227 lag-caused WRAM differences, outside
the ordinary divergence count. RNG extra/unconsumed and ordinary unmatched
APU commands are zero. Two replayed APU-read transitions remain unconsumed.
The final raw line reports 1,466,939 clean iterations and first ordinary WRAM
divergence `None`; these counters classify/exclude timing-defined bytes, rather
than asserting equality of every raw byte. Video comparison is 0/0 by design.

Both censuses completed the entire ledger, exit 0, with no panics. Live-SPC
execution took 1,032.739 seconds (the initial unoptimized live attempt was
stopped and retained as a prefix; it is not used as a complete result).

| Measure | Cartridge APU answers | Live SPC |
|---|---:|---:|
| Iterations | 1,473,123 | 1,473,123 |
| Segment records, including intro | 739 | 742 |
| Non-intro segments | 738 | 741 |
| Ordinary-WRAM-clean non-intro segments | 738 | 731 |
| Fully clean non-intro segments under summary rules | 738 | 730 |
| Ordinary-WRAM-divergent segments / iterations | 0 / 0 | 10 / 2,109 |
| Panicked segments | 0 | 0 |
| Polygon-active iterations | 26,500 | 26,504 |
| Polygon-classified WRAM differences | 5,931 | 14,705 |
| Lag-classified WRAM differences | 227 | 623 |
| RNG unused / extra draws | 0 / 0 | 239 / 557 |
| Ordinary unmatched engine / ROM commands | 0 / 0 | 3 / 4 |
| Unconsumed replayed APU-read transitions | 2 | 0 (replay disabled) |

The first live-SPC ordinary WRAM divergence is **94853**. These are not only
unused port-read differences: all three pendant cases change the main-module
routing byte, and the crystal cases change receipt/cutscene phase (with OAM
shadow bytes first in address order). The raw crystal snapshot below confirms
changed ancilla type and music state, while Link X/Y still match at that boundary.
The ten ordinary divergent spans are:

| Segment start | Context | First ordinary difference | First WRAM address | Ordinary differing iterations | Lag-classified iterations |
|---|---|---:|---|---:|---:|
| 94000 | Pendant `$37` / room `$C8` | 94853 | `$0010` (main module) | 4 | 30 |
| 142000 | Pendant `$39` / room `$33` | 142088 | `$0010` (main module) | 1589 | 323 |
| 228000 | Pendant `$38` / room `$07` | 229033 | `$0010` (main module) | 5 | 30 |
| 380000 | Crystal / room `$5A` | 380880 | `$0839` (OAM shadow) | 71 | 4 |
| 462000 | Crystal / room `$06` | 462628 | `$0835` (OAM shadow) | 81 | 1 |
| 626000 | Crystal / room `$29` | 627256 | `$0839` (OAM shadow) | 70 | 1 |
| 734000 | Crystal / room `$AC` | 735867 | `$0835` (OAM shadow) | 79 | 1 |
| 880000 | Crystal / room `$DE` | 881732 | `$0835` (OAM shadow) | 70 | 1 |
| 1064000 | Crystal / room `$90` | 1064733 | `$0835` (OAM shadow) | 70 | 4 |
| 1318000 | Crystal / room `$A4` | 1318335 | `$0835` (OAM shadow) | 70 | 1 |

The second pendant span is substantial: after the cartridge's zero poll at
142088, the segment has 1,589 ordinary plus 323 lag-classified differing
iterations, followed by a fresh seed at 144000. It must not be described as an
audio-only discrepancy. The shorter pendant transitions and crystal transitions
also change compared WRAM, even when their initiating condition is only a
zero/nonzero song-end poll.

There are **80 changed segment records** in the union of the logs, including
three live-only resync records. All are enumerated in the appendix. The other
70 changed records concern acknowledgement/re-send cadence, read accounting,
reseed/segment boundaries, or polygon-phase classifications; an ordinary-clean
record can still have polygon-classified WRAM changes. No Triforce-room or
credits segment (modules `$19/$1A`, `misc.rs:2272-2273`) differs. The last changed
record starts at 1318000, well before the ending/credits.

The extra live resync points are **629845, 884327 and 1066963**. Statistics at
these boundaries are not directly comparable as same-length segments: for
example, baseline 628000 covers 2,000 iterations, while live 628000 covers
1,845 and live 629845 covers 155. Their engine-command totals 518 + 5 equal
the baseline's 523. Fresh-segment first-frame command exclusions also affect
reported totals; do not interpret every raw count delta as a missing sound.

Both modes still install the cartridge RNG stream. Once the poll moves control
flow, the live census can leave cartridge RNG draws unused or request extra
ones; the existing census answers exhausted per-iteration queues with a zero
valued diagnostic sample (`rom_random.rs:204-214`). Those 239/557 counts are
census control-flow drift, not a native movie's random-number policy. Native
movies use the internal algorithm, and the full native replay proof uses no
cartridge samples or foreign-state seeds. The census is not a full-route
cold native soundtrack/quest-outcome equivalence proof.

Evidence: `census-{rom,live}.txt`, `census-{rom,live}-summary.txt`,
`census-differences.json` (all field values and start contexts),
`ledger-receipt-polls.json`, and `census-live-status.json`.

The crystal difference is not only an OAM-shadow/poll timestamp change. A cold
live-SPC replay dumped iteration **380880** (room `$5A`): Link X/Y and the main
module match, but ancilla slot 4 type at `$0C4E` is **`$3E` versus `$22`**
(rising crystal versus item receipt), and current/last music at `$0130/$0133`
is **`$1A` versus `$13`**. The internal song-end answer advances the actual
receipt/cutscene state one iteration before the cartridge's zero poll. There
are 684 raw differing bytes, including representation/scratch differences;
that raw count is not the census's filtered divergence count. Inspect
`crystal-wram/{engine,rom}-380880.bin` and `crystal-wram-proof.json`.

## Validation and retained evidence

The fresh asset pack was already byte-identical to
`/Users/missingno/Documents/zelda3-rs/target/logical-ledger/assets.dat`.
The clean-main executable is retained as
`target/lagless-proof/clean-main-zelda3`, SHA-256
`e08fa8157852d3ca5024e3ce91f4e376c086aff65f0f0c03b4a28c20e3f164bd`.
Its native-movie magic and new flags are absent, independently confirming that
the retained baseline executable predates these additions.

The 200k comparisons used this command pattern for each of `default` and
`lag-free` (choose fresh baseline filenames when reproducing):

```sh
python3 scripts/state_ledger.py record target/lagless-proof/clean-main-$MODE-200k.jsonl.zst \
  --frames 200000 --mode "$MODE" --binary target/lagless-proof/clean-main-zelda3
python3 scripts/state_ledger.py compare target/lagless-proof/clean-main-$MODE-200k.jsonl.zst \
  --frames 200000 --mode "$MODE" --binary target/parity/zelda3
```

- Clean main, default: 200,000 records, exit 0 (284.027 seconds).
- Clean main, lag-free: 200,000 records, exit 0 (375.323 seconds).
- Post-change comparisons: all 200,000 records and EOF match in both modes,
  exit 0 (default 523.215 seconds; lag-free 570.192 seconds). The records include
  v2 state, display, SRAM, PCM, input and module/room context. This is regression
  preservation of the existing paths, not a GPU or cartridge A/V comparison.
- Parity build: exit 0, no warnings.
- Zelda library test build: exit 0, exactly 24 existing warnings.
- Release feature-off check: exit 0, no warnings.
- CPU-only library tests: SNES 504 passed / 7 ignored; Zelda 2,454 passed /
  3 ignored / 1 known failure. The only failure is
  `cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
  Combined command exits 101 as expected; Zelda execution took 2,048.32 seconds.
- Native format/converter/device audio/live replay: 4 tests passed; both live-recorder modes:
  2 tests passed.
- A headless live recording was consumed by the native reader and replayed for
  1,000 boot iterations with input edges; every per-iteration v2 hash matched.
- CLI save/resume: all 500 resumed per-iteration hashes match the corresponding
  suffix of a 1,000-iteration cold run. Tampered save hash and zero hash interval
  both reject with exit 2. Evidence: `cli-smoke.json` and `smoke-*.jsonl`.
- The final executable also matches the full native proof's first 61 sampled
  hashes through iteration 6000 (`native-final-smoke.jsonl`).

Required build/test commands were run as follows; the combined library command
has the single known failure listed above:

```sh
cargo build --profile parity -p zelda3-bin
cargo test --profile parity -p zelda3 --lib --no-run
CARGO_TARGET_DIR=target/release-check cargo check --release -p zelda3-bin --no-default-features
ZELDA3_ASSETS=$PWD/zelda3_assets.dat cargo test --profile parity -p zelda3 -p snes --lib
```

No GPU runs, oracle-cache writes, `cargo fmt`, commits or pushes were performed.


## Every changed census segment

This table includes every nonidentical segment or polygon record, not just the
ten ordinary-WRAM-divergent spans. Context is the segment's **start** module and
room from its source seed; an overworld room field can be a retained prior
interior identifier. Values are cartridge-answer → live-SPC. “ACK cadence”
means changed command/re-send accounting with ordinary WRAM clean and no
changed polygon record. “Metadata” alone is not a gameplay difference. Full
unabridged fields are in `target/lagless-proof/census-differences.json`.

| Start | Start module / room | Context | Changed measurements |
|---|---|---|---|
| 94000 | `07` / `$0C8` | Pendant `$37` / room `$C8`; receipt state differs | first 94853 at $0010 (ROM 07/00); lag commands 0→2; ordinary WRAM 0→4; lag WRAM 0→30 |
| 142000 | `07` / `$033` | Pendant `$39` / room `$33`; receipt state differs | first 142088 at $0010 (ROM 0e/02); engine commands 33→0; RNG unused 0→29; lag commands 0→33; ordinary WRAM 0→1589; lag WRAM 0→323 |
| 144000 | `07` / `$10A` | Post-receipt reseed metadata; ordinary WRAM clean | reseed false→true |
| 146000 | `07` / `$10A` | ACK cadence; ordinary WRAM clean | engine commands 77→79; held re-sends 2→0 |
| 148000 | `09` / `$012` | ACK cadence; ordinary WRAM clean | engine commands 79→83; held re-sends 4→0 |
| 150000 | `09` / `$012` | ACK cadence; ordinary WRAM clean | engine commands 96→98; held re-sends 2→0 |
| 164000 | `0b` / `$182` | ACK cadence; ordinary WRAM clean | engine commands 126→127 |
| 166000 | `07` / `$114` | ACK cadence; ordinary WRAM clean | engine commands 53→54; held re-sends 2→1 |
| 168000 | `09` / `$114` | ACK cadence; ordinary WRAM clean | engine commands 45→46; held re-sends 1→0 |
| 170000 | `09` / `$114` | ACK cadence; ordinary WRAM clean | engine commands 15→17; held re-sends 2→0 |
| 172000 | `0b` / `$181` | ACK cadence; ordinary WRAM clean | engine commands 45→49; held re-sends 4→0 |
| 174000 | `09` / `$100` | ACK cadence; ordinary WRAM clean | engine commands 130→135; held re-sends 4→0 |
| 176000 | `09` / `$115` | ACK cadence; ordinary WRAM clean | engine commands 62→66; held re-sends 4→0 |
| 178000 | `09` / `$109` | ACK cadence; ordinary WRAM clean | engine commands 92→97; held re-sends 7→2 |
| 180000 | `09` / `$008` | ACK cadence; ordinary WRAM clean | engine commands 96→102; held re-sends 6→0 |
| 182000 | `09` / `$008` | ACK cadence; ordinary WRAM clean | engine commands 66→70; held re-sends 4→0 |
| 184000 | `09` / `$0E1` | ACK cadence; ordinary WRAM clean | engine commands 75→77; held re-sends 2→0 |
| 186000 | `09` / `$0E1` | ACK cadence; ordinary WRAM clean | engine commands 55→58; held re-sends 2→0 |
| 188000 | `0e` / `$0F1` | ACK cadence; ordinary WRAM clean | engine commands 108→111; held re-sends 2→0 |
| 190000 | `07` / `$0E4` | ACK cadence; ordinary WRAM clean | engine commands 172→174; held re-sends 2→0 |
| 194000 | `09` / `$0E4` | ACK cadence; ordinary WRAM clean | engine commands 42→43 |
| 214000 | `0e` / `$027` | ACK cadence; ordinary WRAM clean | engine commands 62→64; held re-sends 2→0 |
| 228000 | `07` / `$007` | Pendant `$38` / room `$07`; receipt state differs | first 229033 at $0010 (ROM 07/00); engine commands 50→52; held re-sends 2→0; lag commands 0→4; ordinary WRAM 0→5; lag WRAM 0→30 |
| 230000 | `09` / `$077` | ACK cadence; ordinary WRAM clean | engine commands 106→107 |
| 234000 | `07` / `$0F9` | ACK cadence; ordinary WRAM clean | engine commands 61→64; held re-sends 2→0 |
| 236000 | `07` / `$0E6` | ACK cadence; ordinary WRAM clean | engine commands 74→78; held re-sends 4→0 |
| 238000 | `09` / `$0E6` | ACK cadence; ordinary WRAM clean | engine commands 98→100; held re-sends 2→0 |
| 240000 | `0e` / `$0E6` | ACK cadence; ordinary WRAM clean | engine commands 58→61; held re-sends 3→0 |
| 244000 | `0e` / `$180` | ACK cadence; ordinary WRAM clean | engine commands 129→132; held re-sends 3→0 |
| 246000 | `09` / `$100` | ACK cadence; ordinary WRAM clean | engine commands 124→128; held re-sends 4→0 |
| 248000 | `09` / `$100` | Read accounting / ACK cadence; ordinary WRAM clean | engine commands 94→102; read unused 1→0; held re-sends 8→0 |
| 250000 | `0f` / `$118` | ACK cadence; ordinary WRAM clean | engine commands 110→119; held re-sends 9→0 |
| 252000 | `09` / `$118` | ACK cadence; ordinary WRAM clean | engine commands 82→84; held re-sends 1→0 |
| 254000 | `07` / `$126` | ACK cadence; ordinary WRAM clean | engine commands 101→104; held re-sends 3→0 |
| 256000 | `09` / `$126` | ACK cadence; ordinary WRAM clean | engine commands 82→84; held re-sends 1→0 |
| 258000 | `09` / `$060` | ACK cadence; ordinary WRAM clean | engine commands 106→107 |
| 292000 | `15` / `$020` | ACK cadence; ordinary WRAM clean | engine commands 224→225; held re-sends 1→0 |
| 294000 | `09` / `$020` | ACK cadence; ordinary WRAM clean | engine commands 107→112; held re-sends 4→0 |
| 296000 | `09` / `$115` | ACK cadence; ordinary WRAM clean | engine commands 120→124; held re-sends 3→0 |
| 298000 | `10` / `$112` | ACK cadence; ordinary WRAM clean | engine commands 123→129; held re-sends 4→0 |
| 300000 | `09` / `$115` | ACK cadence; ordinary WRAM clean | engine commands 73→75; held re-sends 1→0 |
| 304000 | `07` / `$123` | ACK cadence; ordinary WRAM clean | engine commands 130→133; held re-sends 3→0 |
| 306000 | `09` / `$123` | ACK cadence; ordinary WRAM clean | engine commands 81→86; held re-sends 2→0 |
| 308000 | `07` / `$120` | ACK cadence; ordinary WRAM clean | engine commands 65→67; held re-sends 2→0 |
| 310000 | `09` / `$120` | ACK cadence; ordinary WRAM clean | engine commands 51→58; held re-sends 6→0 |
| 312000 | `07` / `$124` | ACK cadence; ordinary WRAM clean | engine commands 122→126; held re-sends 4→0 |
| 314000 | `09` / `$124` | ACK cadence; ordinary WRAM clean | engine commands 114→116; held re-sends 2→0 |
| 316000 | `09` / `$124` | ACK cadence; ordinary WRAM clean | engine commands 72→77; held re-sends 5→0 |
| 318000 | `09` / `$124` | ACK cadence; ordinary WRAM clean | engine commands 80→82; held re-sends 1→0 |
| 322000 | `09` / `$11A` | ACK cadence; ordinary WRAM clean | engine commands 57→58 |
| 324000 | `07` / `$04A` | ACK cadence; ordinary WRAM clean | engine commands 54→55; held re-sends 19→18 |
| 326000 | `07` / `$04A` | ACK cadence; ordinary WRAM clean | engine commands 60→61; held re-sends 19→18 |
| 332000 | `07` / `$02B` | ACK cadence; ordinary WRAM clean | engine commands 84→85; held re-sends 17→16 |
| 342000 | `07` / `$01A` | ACK cadence; ordinary WRAM clean | engine commands 121→122; held re-sends 19→18 |
| 348000 | `07` / `$03A` | ACK cadence; ordinary WRAM clean | engine commands 45→46; held re-sends 2→1 |
| 350000 | `07` / `$04A` | ACK cadence; ordinary WRAM clean | engine commands 74→75; held re-sends 2→1 |
| 352000 | `07` / `$009` | ACK cadence; ordinary WRAM clean | engine commands 88→89; held re-sends 2→1 |
| 354000 | `07` / `$03A` | ACK cadence; ordinary WRAM clean | engine commands 76→79; held re-sends 42→39 |
| 356000 | `07` / `$03A` | ACK cadence; ordinary WRAM clean | engine commands 60→62; held re-sends 2→0 |
| 370000 | `07` / `$01B` | ACK cadence; ordinary WRAM clean | engine commands 64→66; held re-sends 22→20 |
| 376000 | `07` / `$05A` | ACK cadence; ordinary WRAM clean | engine commands 151→153; held re-sends 2→0 |
| 378000 | `07` / `$05A` | ACK cadence; ordinary WRAM clean | engine commands 164→166; held re-sends 2→0 |
| 380000 | `07` / `$05A` | Crystal / room `$5A`; receipt state differs | first 380880 at $0839 (ROM 07/00); engine unmatched 0→1; held re-sends 0→1; RNG extra 0→30; RNG unused 0→30; ordinary WRAM 0→71; lag WRAM 0→4; poly active 1044→1045; poly WRAM 808→1045 |
| 462000 | `07` / `$006` | Crystal / room `$06`; receipt state differs | first 462628 at $0835 (ROM 07/00); engine commands 271→0; RNG extra 0→160; RNG unused 0→29; ROM unmatched 0→1; lag commands 17→270; poly commands 17→0; ordinary WRAM 0→81; lag WRAM 0→1; poly WRAM 1054→1290 |
| 590000 | `07` / `$029` | Read accounting only; ordinary WRAM clean | read unused 1→0 |
| 626000 | `07` / `$029` | Crystal / room `$29`; receipt state differs | first 627256 at $0839 (ROM 07/00); engine commands 169→98; RNG extra 0→84; RNG unused 0→30; ROM unmatched 0→1; lag commands 0→70; ordinary WRAM 0→70; lag WRAM 0→1; poly WRAM 0→673 |
| 628000 | `0e` / `$029` | Crystal aftermath / polygon phase; ordinary WRAM clean | engine commands 523→518; compared ROM commands 523→518; length 2000→1845; reseed false→true; lag commands 0→20; poly commands 0→20; poly WRAM 14→1707 |
| 629845 | `10` / `$059` | Post-crystal resync; new live-only record | length 155; ordinary WRAM clean; reseed true; engine/ROM commands 5/5 |
| 632000 | `07` / `$056` | ACK cadence; ordinary WRAM clean | engine commands 85→87; held re-sends 2→0 |
| 634000 | `09` / `$057` | ACK cadence; ordinary WRAM clean | engine commands 98→102; held re-sends 2→0 |
| 734000 | `12` / `$0AC` | Crystal / room `$AC`; receipt state differs | first 735867 at $0835 (ROM 07/00); engine commands 155→154; RNG extra 0→6; RNG unused 0→11; ROM unmatched 0→1; ordinary WRAM 0→79; lag WRAM 0→1; poly WRAM 0→53 |
| 736000 | `07` / `$0AC` | Crystal aftermath / polygon phase; ordinary WRAM clean | engine commands 463→465; RNG extra 0→19; RNG unused 0→19; reseed false→true; lag commands 25→17; poly commands 23→17; poly WRAM 735→1999 |
| 880000 | `07` / `$0DE` | Crystal / room `$DE`; receipt state differs | first 881732 at $0835 (ROM 07/00); engine unmatched 0→1; held re-sends 0→1; RNG extra 0→30; RNG unused 0→30; ordinary WRAM 0→70; lag WRAM 0→1; poly active 195→197; poly WRAM 0→197 |
| 882000 | `07` / `$0DE` | Crystal aftermath / polygon phase; ordinary WRAM clean | engine commands 547→537; reseed false→true; lag commands 0→173; poly commands 0→164; poly WRAM 0→1958 |
| 884000 | `0e` / `$0DE` | Crystal aftermath / polygon phase; ordinary WRAM clean | engine commands 140→62; compared ROM commands 141→62; length 2000→327; held re-sends 0→1; reseed false→true; lag commands 0→1; poly commands 0→28; lag WRAM 1→0; poly WRAM 0→288 |
| 884327 | `10` / `$00E` | Post-crystal resync; new live-only record | length 1673; ordinary WRAM clean; reseed true; engine/ROM commands 78/78 |
| 1064000 | `07` / `$090` | Crystal / room `$90`; receipt state differs | first 1064733 at $0835 (ROM 07/00); engine unmatched 0→1; held re-sends 0→1; RNG extra 0→30; RNG unused 0→31; lag commands 0→17; poly commands 0→17; ordinary WRAM 0→70; lag WRAM 0→4; poly active 1192→1193; poly WRAM 0→1193 |
| 1066000 | `0e` / `$090` | Crystal aftermath / polygon phase; ordinary WRAM clean | engine commands 271→244; compared ROM commands 271→245; length 2000→963; reseed false→true; poly commands 0→18; poly WRAM 0→744 |
| 1066963 | `10` / `$098` | Post-crystal resync; new live-only record | length 1037; ordinary WRAM clean; reseed true; engine/ROM commands 26/26 |
| 1318000 | `07` / `$0A4` | Crystal / room `$A4`; receipt state differs | first 1318335 at $0835 (ROM 07/00); engine commands 328→0; RNG extra 0→198; RNG unused 0→30; ROM unmatched 0→1; lag commands 18→326; poly commands 18→0; ordinary WRAM 0→70; lag WRAM 0→1; poly WRAM 1356→1594 |

## Merged with rom-timing gate

Integrated on `codex/lagless-replay2` over main `85451fc2`, without commit/push.
Native recording, session events and `input.z3lag` stay ungated; the cartridge
`input.txt` writer is isolated in `cartridge_input_recording` behind `rom-timing`.
Play recording/finish calls and developer-warp protection work in both builds.
`--record-lagless` also compiles without `parity-debug`; the environment switch
retains that diagnostic gate. Stale cartridge-only test references were corrected.

Both parity binary builds and the feature-off release check pass warning-free.
Library test compilation has the expected 24 default warnings and zero reduced
warnings. All four replay tests and both recorder tests pass in each feature set.
Reduced conversion is byte-identical to the earlier movie. Two 20,000-iteration
reduced replays match all 201 hashes; their 200 scheduled checkpoints match the
full-proof `native-a.jsonl` prefix (iteration 19,999 is an additional repeat check).
All three 200k comparisons match every record and EOF against the clean-main
baselines: both modes with the default binary, lag-free with the reduced binary.
Evidence and binary hashes: `target/lagless-proof/gate-validation.json`,
`gate-proof.json`, and adjacent `gate-*` logs. No GPU/Snes9x execution or
oracle-cache writes; no `cargo fmt`, commit or push.
