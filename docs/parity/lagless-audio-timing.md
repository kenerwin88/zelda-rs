# Lagless audio command timing

Measurement on `codex/lagless-audio-timing`, main `8b0f4ee2`. All reported runs are CPU-only.

## Full-route headline

The live-SPC census matches **67,251 command edges** by content/order. **95.90%** of intervals between consecutive matched commands differ by at most one presented frame at the source opening-host anchors; **91.58%** remain within one using the retained source timing bounds. **97.99%** of matched requests are in the same or adjacent logical iteration.

Cumulative timing is less close: the live lane finishes **4,778 presentation frames ahead** of the source interval (79.633 seconds on the common 60 Hz grid). Its largest matched-command lead is **4,836 frames / 80.600 seconds**. The median lead is 1,947 frames (32.450 seconds). Good local cue spacing does not imply an original-speed route.

All ten live receipt song-end polls are **5–7 logical iterations early**. Relative to their independently matched fanfare requests, their zero-poll durations are **eight at −1 presented frame and two at 0** under the source anchors. Conservative source timing bounds extend to **3 frames early**, with none late. These are request-to-gameplay-zero observations, not a measurement of PCM note release.

| Lane | Matched | Exact | Exactly 1 frame | 2–5 | >5 | Source only | Native only | Largest lead |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 67,321 | 1 | 0 | 0 | 67,320 | 322 | 683 | 4,699 frames |
| live | 67,251 | 1 | 0 | 0 | 67,250 | 392 | 718 | 4,836 frames |

The exact/1/2–5/>5 columns are **cumulative presentation offsets**, not logical-iteration offsets or differences between neighbouring cue intervals. All nonexact matched commands in these runs are early; there are zero late matches. The first boot SFX is the single exact anchor. Per-command bounds, module/submodule and current-song distributions are in each lane’s `summary.json` and `commands.jsonl.gz`.

| Lane | Consecutive matched gaps | Exact gap | Gap differs 1 frame | 2–5 | >5 |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 67,320 | 33,476 | 31,182 | 1,841 | 821 |
| live | 67,236 | 33,371 | 31,109 | 1,909 | 847 |

The live clock counts **1,473,123 main callbacks + 97,986 held NMIs + 5,109 masked upload callbacks = 1,576,218 presentations**. ROM-answer control counts **98,093 holds + 5,139 masked callbacks**, or **1,576,355 presentations**. Source host extent is **1,580,996 frames**. There is one ROM epoch and 15 live epochs (14 actual foreign-state reseeds); none resets the cumulative clock.

There are **392 source-only / 718 native-only** normalized live events. Of the source-only events, **228 are port-0 `$FF` bank-upload requests**, with no native command-queue port equivalent; the remaining count is **164**. Of the native-only events, **636 are port-3 `$0C` dialogue-click edges**. These are alignment/representation findings, not a claim of 392 missing and 718 extra audible sounds. All rows, including the other values and their contexts, remain in `unmatched.jsonl.gz`; the raw repeat inventory and bank-transfer ranges are retained separately.

## Receipt transition measurements

| Live transition group | Matched | Early / late | Source only / native only | Cumulative offset range (frames) |
| --- | ---: | ---: | ---: | --- |
| All track requests | 522 | 522 / 0 | 8 / 12 | −4,836 to −8 |
| Stops/fades/control requests | 572 | 572 / 0 | 229 / 6 | −4,773 to −14 |
| Receipt fanfare `$13` | 11 | 11 / 0 | 0 / 0 | −4,772 to −208 |
| Item-get cue values | 214 | 214 / 0 | 11 / 10 | −4,775 to −9 |
| First dungeon-entry track request | 111 | 111 / 0 | 0 / 0 | −4,340 to −37 |
| Ending/credits `$19/$1A` | 4 | 4 / 0 | 1 / 0 | −4,819 to −4,773 |

The dungeon track group follows a module-`$06` entry through module `$07` to its
first new track request. Looking only at the opening `$06` context would instead
find 99 unmatched `$FF` upload requests and miss the subsequent music. The lone
unmatched ending control is also `$FF`; it is retained, not counted as a lost song.

| Source zero iteration | Live zero iteration | Logical shift | Cumulative live offset (frames) | Request-to-zero delta (frames) | Source-bound interval |
| ---: | ---: | ---: | ---: | ---: | --- |
| 94854 | 94848 | -6 | -209 | -1 | [-1, -1] |
| 142088 | 142082 | -6 | -297 | +0 | [-1, +0] |
| 229035 | 229030 | -5 | -672 | +0 | [-2, +0] |
| 380881 | 380875 | -6 | -1013 | -1 | [-1, -1] |
| 462628 | 462622 | -6 | -1172 | -1 | [-1, -1] |
| 627256 | 627250 | -6 | -1758 | -1 | [-2, -1] |
| 735867 | 735861 | -6 | -2183 | -1 | [-2, -1] |
| 881734 | 881727 | -7 | -2846 | -1 | [-3, -1] |
| 1064734 | 1064728 | -6 | -3660 | -1 | [-2, -1] |
| 1318335 | 1318330 | -5 | -4189 | -1 | [-2, -1] |

All ten source/native fanfare requests used for these duration differences are independently paired in the command alignment. The ROM-answer control observes zero in the same logical iteration as source for all ten, but its request-to-gameplay-zero durations are 4–6 presented frames longer at the anchors. Its forced source answers can keep gameplay waiting after the native SPC would have ended; this is not an audible-duration claim. The live run reproduces the early-zero diagnosis without changing a poll answer, clock or price.

## Verification

Both full census gates **PASS**, complete=True, **1,473,123 iterations each**, unexplained zero. Analysis checks contiguous records, zero lost native publications, completed Zstandard streams and `callbacks == 1 + held + masked` on every iteration. It retains **11,746,718 source writes**, including **5,947,694 bank-transport writes**, rather than dropping upload iterations.

- Default parity build: exit 0, **zero warnings**. No runtime/audio/pacing library code changed.
- Fresh clean-main default 200k baseline: recorded at `8b0f4ee2`, clean tree; candidate matches **all 200,000 records and EOF**, including state/display/SRAM/PCM/input/context.
- Zelda default library: **2,612 passed, 1 known spotlight failure, 3 ignored**. Reduced (`--no-default-features --features parity-debug`): **2,147 passed, 0 failed, 2 ignored**.
- SNES CPU-only library, each feature set: **504 passed, 0 failed, 7 ignored**.
- Report tests: **8 passed**. The script ran both real CLI lanes for **3,000 iterations each**, with census gates and report generation passing. `git diff --check` passes.

Complete logs, hashes and checks are bound by `verification.json` and `provenance.json` under the evidence directory. The existing lib-test-only warnings are separate from the warning-free ordinary build.

## Scope

This measures **command publication**, not audible PCM onset or exact source CPU
bus timing. Both lanes run the complete logical census with `--seed-every 2000`.
The ROM lane supplies source APU poll answers; `--live-apu-reads` uses the shipped
engine's persistent native SPC. Both retain source RNG and conditional foreign
state reseeds. This is evidence about the shipped mechanisms on this diagnostic
route, not a cold native movie's soundtrack or quest equivalence proof.

## Reproduce

```sh
cargo build --profile parity -p zelda3-bin
python3 scripts/lagless_audio_timing.py \
  --binary target/parity/zelda3 \
  --ledger /Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2 \
  --sram /Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18/initial.srm \
  --output target/lagless-audio-timing-repro
python3 -m unittest discover -s scripts -p test_lagless_audio_timing.py
```

The script runs the two lanes sequentially, isolates SRAM persistence, refuses to
replace existing traces and restricts output to this checkout's `target/`.
`--frames N` makes an explicitly counted prefix. `--analyze-only` rebuilds reports
from completed traces. Zstandard's CLI is required for reading the compressed
observations. No renderer, external emulator or oracle-cache writer is invoked.
The binary's reusable observation flag is `--logical-replay ...
--audio-timing-trace FILE.jsonl.zst`; it does not add audio calls or change clocks.

## Coordinates and matching

Each source `$2140–$2143` write, including zero clears, equal repeats and bank
payload bytes, remains in `rom.jsonl.zst` / `live.jsonl.zst`. Native observations
are the compatibility port projections of every command-queue publication, with
its actual audio/presentation callback index. The trace checks for intermediate
pushes that a last-publication snapshot would miss; analysis rejects any such
loss. Native compatibility snapshots are not literal hardware bus writes.

Source frame zero is the ledger's first opening host, **81**. Source positions
use `open_host - 81`, so lag and masked upload fields count. Native frame zero is
the first replay callback. Its clock counts **iterations + held NMIs + masked
upload callbacks**. Masked callbacks are reported separately; dropping them would
omit presentation/audio time. These cumulative clocks never reset at a foreign
state seed. The secondary epoch coordinate subtracts each side's clock at an
actual reseed; it does not pretend the census is continuous native play.

Positive offsets mean native is late; negative means early. Milliseconds are
frame-equivalent presentation time on a common **60 Hz** grid (`frames * 1000/60`).
The shipped frontend's `platform::frame_delay` repeats 17/17/16 ms, averaging
60 presentations/s. For NTSC frame-equivalent milliseconds instead, multiply
the stored frame offset by `1000/60.0988`; these two common-grid conventions
must not be confused with subtracting elapsed times on two different clocks.
They are not wall-clock recordings, a measurement of physical NTSC crystal drift,
or native SPC instruction timestamps. The existing audio window advances 534
DSP pairs per callback, or 16.6875 ms of the 32 kHz output grid; that separate
sample clock has not been retimed to match the nominal presentation grid.

The source ledger stores order and iteration host bounds, not each write's bus
clock. Initial ordinary NMI port groups are assigned to their opening host;
later writes carry the conservative opening-to-next-opening interval. Every
matched row includes the point offset **and** signed frame/millisecond bounds.
Headline exact/1/2–5/>5 buckets use that stated opening-host convention, not an
invented subframe timestamp. The summaries separately count definitely early,
definitely late, bounds containing zero and uncertain source frames.

Primary commands are transitions to a nonzero value per port, with zero writes
updating the latch. Thus a clear followed by the same SFX is a new command, while
an unchanged acknowledgement resend is not a new audible request. The second
inventory retains **every nonzero runtime source write and native queue
projection**, including equal repeats; zero clears remain in the full trace.
Native held queue projections can repeat music/ambient values much more often
than literal source NMI writes. Their raw count difference is not a missing-sound
count.

An explicit IPL upload header (ports 2,3,1,0 with token `$CC`) and its terminating
zero-run header delimit bank payload bytes. They remain in the raw trace, with
write-index ranges and hashes in `bank-transfers.jsonl.gz`. The stop/upload music
request and ordinary port writes before/after the payload stay in the command
inventory. Long ordinary lag-handler write streams are **not** discarded by a
write-count threshold.

Matching preserves `(port, value)` content and order, never crossing a native
reseed epoch. At a mismatch it seeks the anchor requiring the fewest skipped
events within 64 events on either side, breaking ties by logical iteration
distance. Matches must be within 32 logical iterations. Every skipped event is
reported as source-only or native-only, with module/submodule, room and song.
Repeated identical commands do not have unique identities in this ledger: this
is a reproducible bounded alignment convention, not proof of a unique pairing
or that every unmatched queue projection is an audible omission. The logical
iteration-offset distribution is separate from presentation-frame timing.
The `gap_*` distribution additionally measures the change in presentation
offset between consecutive matched commands within an epoch: it compares their
inter-command intervals, without erasing the accumulated route lead.
Module/submodule, room and current-song annotations are iteration-entry context;
they are not a source producer-PC trace or a mid-call context sample.

## Transitions and song-end polls

`song-changes.jsonl.gz` contains port-0 track requests `$01–$EF`;
`music-controls.jsonl.gz` retains stops, fades and other `$F0–$FF` controls.
`transitions.jsonl.gz` additionally identifies receipt fanfare `$13`, item-get
cue values (SFX2 low six bits `$0F`, sword SFX1 `$2C`, from
`misc.rs::complete_ancilla_add_item_receipt`), dungeon-entry module `$06`, and
ending/credits modules `$19/$1A`. Groups can overlap, and item-get cue values
can be reused by other callers; individual context remains inspectable.
The first new track after a `$06` entry is also tagged while the entry continues
into `$07`; leaving those modules or reseeding clears that pending association.

`song-ends.jsonl.gz` pairs first nonzero-to-zero **gameplay poll observations**.
The native observations come from the existing song-end helper. Source receipt
candidates use fanfare `$13` and only port-0 reads of `$13`/zero, outside explicit
bank transfers; long/mixed upload handshakes are counted separately. The source
ledger has no caller PCs, so these are bounded receipt-poll candidates, not a
new source call-site trace. No ending/credits poll is fabricated: those music
requests and controls are measured as transitions even when no gameplay branch
polls their completion.

Each matched end includes its logical iteration shift and, when both same-song
requests exist in that seed epoch, request-to-zero callback/frame durations.
This separates an early zero within a receipt sequence from the accumulated
route clock lead. It extends the diagnosis in `cost-pacing.md` (live song-end
section preceding main `a28e327d`); it does not attribute the result to tempo,
SPC fidelity or a particular pacing charge. Seeding can restore a song without
replaying its original command, so absent request durations remain absent.

## Retained data

All generated evidence is under `target/lagless-audio-timing/`:

- `rom.jsonl.zst`, `live.jsonl.zst`: complete iteration, context, source-write,
  native-publication, poll and callback-count observations.
- `rom/`, `live/`: `summary.json`, `commands.jsonl.gz`,
  `raw-nonzero-writes.jsonl.gz`, `unmatched.jsonl.gz`, `song-changes.jsonl.gz`,
  `music-controls.jsonl.gz`, `song-ends.jsonl.gz`, `transitions.jsonl.gz`, and
  `bank-transfers.jsonl.gz`. Summaries include every module/submodule and song,
  command kind, transition group, early/late counts and five worst offsets.
- `provenance.json`, `report.md`: binary/input/source hashes and a compact table.
- `main-default-200k.jsonl.zst` and its metadata, baseline/candidate binaries,
  build logs, complete library logs and the default state-ledger comparison.

No optional PCM windows were rendered. A queue-publication offset does not by
itself identify a corresponding audible onset, especially across a bank upload.
The five worst command rows remain in each summary for subsequent SPC probes.

Disk exhaustion interrupted the initial censuses and comparison. Those outputs
are retained in `interrupted/` and do not count as completed evidence. Only
disposable Cargo build artifacts were cleaned; the measurements restarted from
iteration zero. No GPU, `--video`, cached-av, Snes9x execution, cargo GPU tests,
oracle-cache writes, `cargo fmt`, commit or push were performed.
