# Lagless differences gate

Worktree `/Users/missingno/Documents/zelda3-rs-codex`, branch
`codex/lagless-gaps`, base `85451fc2f96b9e5cf2a7c2c5e91f5bc1b7fdfe11`.
The reduced `--no-default-features --features parity-debug` build is the product.
All commands here are CPU-only. The supplied logical ledger and oracle cache
are read-only. No commit, push, `cargo fmt`, GPU, Snes9x, or cached-A/V run.

## Receipt outcomes

All ten pendant/crystal sequences complete under the engine's live SPC without
an intervening reseed. Each probe starts from a ledger seed before the receipt
song is issued. `--single-seed` suppresses both periodic and resync seeds after
that initial seed. The ledger RNG and recorded inputs remain installed; this is
an isolated receipt-outcome test, not an input-only full-quest proof.

The comparison searches for a stable outdoor `09/00` boundary, with receipt
ancillas `$22/$3E` gone, within 2,000 iterations of the engine boundary. It
requires exact main module/submodule, room, Link X/Y, and **every byte
of `$F000..$F4FF`**, including inventory and save flags. The match is against
the ledger at the shifted boundary. It does not claim subsequent inputs played
at unshifted iteration indices remain equivalent forever.

| Seed | Engine stable | Ledger stable | Shift | Room | Link X/Y | Pendants | Crystals |
| ---: | ---: | ---: | ---: | --- | --- | --- | --- |
| 94000 | 95163 | 95163 | 0 | `$0C9` | 3920/1579 | `$04` | `$00` |
| 140000 | 142380 | 142380 | 0 | `$063` | 296/3131 | `$06` | `$00` |
| 228000 | 229336 | 229336 | 0 | `$077` | 2288/123 | `$07` | `$00` |
| 380000 | 383469 | 383667 | -198 | `$04A` | 3920/1595 | `$07` | `$02` |
| 460000 | 465767 | 465847 | -80 | `$028` | 1912/3819 | `$07` | `$12` |
| 626000 | 629663 | 629876 | -213 | `$059` | 152/203 | `$07` | `$52` |
| 734000 | 738023 | 738219 | -196 | `$0DB` | 504/1979 | `$07` | `$72` |
| 880000 | 884125 | 884358 | -233 | `$00E` | 3256/3531 | `$07` | `$76` |
| 1064000 | 1066932 | 1066994 | -62 | `$098` | 296/3290 | `$07` | `$77` |
| 1316000 | 1320740 | 1320931 | -191 | `$0D6` | 3848/315 | `$07` | `$7F` |

The room `$33` pendant must start at 140000, before song `$13` is issued at
141449. Seeding at 142000 restarts a song whose cartridge playback is already
nearly over, misses the recorded dialogue-dismissal window, and leaves the
census in the receipt when it reseeds at 144000. The uninterrupted probe opens
the message at 141568 and returns to the cartridge's routing by 142222; both
reach the identical stable outdoor state at 142380. Restarted SPC state is an
explicit property of `seed_memory_from_foreign_image`, not evidence of a stuck
receipt in continuous native execution. The analogous crystal probes at
460000 and 1316000 also start before the receipt song, rather than restarting
it from the 462000/1318000 mid-song seeds.

Accepted: **song-end-poll timing**. Receipt callers test zero/nonzero only;
removing lag changes elapsed SPC time per logical iteration, and foreign seeds
do not restore SPC state. The observed branches move dialogue/crystal phases,
not the final awarded item or saved outcome. No song duration or guessed
clock offset was introduced.

The census observes the actual gameplay poll return, without supplying values
or advancing the clock. A zero/nonzero disagreement with a cartridge poll
starts the causal classification. It ends on compared-WRAM reconvergence or a
fresh seed. Polygon and lag categories retain their existing precedence;
ordinary WRAM and unmatched commands/RNG during the remaining causal span are
reported separately as song-end-poll effects. A room number alone never starts
this classification. A changed poll is not by itself an outcome proof: the ten
uninterrupted checks above provide that evidence for this route.

Evidence under `target/lagless-gaps/`: `receipt-<seed>.{jsonl,log}`,
`receipt-outcomes.json` (matched boundaries and SHA-256 of complete save blocks
and traces), `receipt-status.json`. The 140000 probe has its own log and trace;
all ten probes exited 0 and ran to their requested endpoint.

## Incidental APU reads

The two leftover transitions were comparator misclassification, not missed
song-end branches. In the iris loop, `$00:F3A3` reads `$2137` to latch the beam,
`$00:F3A6` executes **16-bit `LDA $213F`**, and `$00:F3A9` replaces A with
`LDA $213D`, followed by `AND #$00FF`. The high byte of `$213F` is `$2140`;
its value is incidental and discarded. The source ledger records that bus
read, but its `apu_reads` records do not retain the originating PC. The native
iris computes the table without spinning until scanline 192.

| Iteration | Context after iteration | Lag NMIs | Raw port reads | Distinct values | Native gameplay polls | Classification |
| ---: | --- | ---: | ---: | --- | ---: | --- |
| 249449 | `10/01`, room `$118`, opening iris | 1 | 506 | `$0E -> $00` | 0 | incidental iris status-read high byte |
| 590972 | `07/0F`, room `$059`, dungeon entry iris | 2 | 1156 | `$0B -> $00` | 0 | incidental iris status-read high byte |

Both short probes have ordinary WRAM completely clean, RNG exact and APU
commands matched. The first probe covers 1,451 iterations from 248000; the
second covers 974 from 590000. Dumps include both WRAM images at the event.
The classification requires the iris caller context, source lag, only port-0
reads, and **no observed native gameplay song-end poll**. Other unused reads
remain unexplained and fail the gate. The raw transitions remain counted in
`apu_reads incidental=`, separately from unused gameplay reads; no audio
command, poll return or simulated clock changed.

Evidence: `target/lagless-gaps/unused-read-{249449,590972}.log` and the
corresponding dump directories. Source evidence is the iris work at
`crates/zelda3/src/cycle_models/spotlight.rs` (`$F3A3/$F3A6/$F3A9`) and the
unchanged bytes of the supplied ROM, inspected without executing an oracle.

## Lag audit and fixes

The complete cartridge-answer inventory contains **227 iterations**, with every
address and both byte values retained in `target/lagless-gaps/lag-inventory.json`.
An overworld context's room field can retain the previous indoor room.

| Context: module/submodule, room | Iterations | Addresses / cause | Resolution |
| --- | ---: | --- | --- |
| `$0B/00`, `$182` | 170 | `$0CFD`, rupee sound delay | Accepted: lag changes when sound port 1 is consumed |
| `$0E/02`, `$182` | 40 | `$0CFD` | Same accepted sound-queue cause |
| `$09/00`, `$115` | 8 | `$0CFD` | Same accepted sound-queue cause |
| `$09/00`, `$103` | 2 | `$0CFD` | Same accepted sound-queue cause |
| `$0E/07`, `$028` | 2 | Map-upload post-NMI sample overrun | Accepted: next main step appears in the late reference sample |
| `$0E/0A`, `$00E` / `$098` | 2 | Flute-upload post-NMI sample overrun | Same accepted sampling cause |
| `$07/02`, `$058` | 3 | `$00B2`, exploding-wall width counter | **Fixed**, not a timing exception |

`HudRefillPrefix` at `$0D:DC02..DC15` increments `$0CFD` if sound port 1 is
idle, otherwise clears it. A lag NMI can consume the queued sound before that
branch. The difference can persist until another queued sound resets the
counter. Only this audited address receives the ordinary lag-carried exception;
the presence of lag alone no longer accepts arbitrary gameplay differences.

The four overruns are **471536, 473561, 885929, 1068135**. All have five lag NMIs,
seven IRQs, and a recorded handler-return line of 225. The ledger producer's
`stale_opening` path closes an unreturned opening handler at the next NMI;
its post image can already contain the next map/flute main step. At 471536,
the reference pre image and engine agree on map state 2, frame counter `$F1`,
animation countdown 7 and Link DMA countdown 15. The late post image has
state 3 / `$F2` / 6 / 14. The following iteration reconverges without a seed.
All four reconverge on the immediately following iteration in the full trace.

The accepted signature requires that exact phase relationship: map/flute
module and submodule; state `2 -> 3`; latch `1 -> 0`; one frame-counter
increment; one animation/DMA decrement; the appropriate pending NMI request;
and differences confined to those controls and the map stripe or flute CHR
buffer. Other lag-frame differences remain unexplained. Evidence:
`map-{,pre-}471536/`, corresponding logs, and the complete classification trace.

At **536087, 560831, 562394**, `RoomDraw_ExplodingWallSegment` left `$00B2=18`
after its Rust `for` loop. The source's word store at `$09:ACE7` and decrement
loop at `$09:AD1D` leave the word zero. Lagless now writes the word initial
count and its terminal zero through the existing owner. This also fixes the
otherwise stale high byte. The cartridge-timing path retains its baseline
behavior. These three differences are removed, rather than classified away.

## Native polygon scenes

Main `85451fc2` already includes module `$1A` in
`triforce_room_poly_thread_is_active`. The common dispatcher runs that native
worker before main, and NMI runs its lag phases, exactly as for `$19` and the
crystal maidens. No second ending renderer was added. The full cartridge-answer
census covers **4,093 Triforce/entry polygon iterations and 3,908 ending polygon
iterations with zero compared-WRAM differences**. No GPU/video result is claimed.

The remaining polygon differences are crystal-worker phase and aftermath
differences, including the overlapping module `$0E` message and the ensuing
warp/iris states while one side still has an active thread. The native worker
retains real drawing progress; the ordinary crystal scheduler derives additional
worker phases from main's measured work, and foreign seeds do not contain that
continuation. These phases can differ while item, room, position and save outcome
still agree at the shifted stable boundary proved above. The bitmap remains
exempt as `POLYHEDRAL_BUFFER`; no angle, pixel or RAM-result lookup was introduced.

In live-SPC census segment 736000, reseeding an active crystal thread also shifts
**19 consumed and 19 extra RNG demands** between iterations. These are random
cutscene effects following worker completion, not a changed item outcome.
They are counted as polygon RNG alignment, separately from ordinary and
song-poll RNG differences. The scoped reproduction is `poly-rng.log` and
`poly-rng.jsonl`; the room `$AC` uninterrupted outcome remains the exact saved
outcome at engine 738023 / ledger 738219.

## Exact boot intro

The first **2,000 iterations are now part of the normal gate**, with polygon
and lag WRAM exceptions disabled. The comparison additionally restores the
worker's live `$1F00..$1F09`, `$1F0C..$1F0D`, and `$1F3F..$1FFD` storage to
the compared set. CPU stack bytes and the agreed bitmap representation
exclusions remain. The old comparator hid 26 initialization differences;
examining worker storage after correcting initialization exposed another 300
partial-worker iterations before the outgoing file-select boundary.

Two reset fixes establish the same boot storage: initialize WRAM with the
cartridge's `$55` power-on fill before the native reset clear, and leave the
animated-tile source unset until the intro initializes its tiles. This matches
the early `$C00D/$C013` countdowns and `$0ADC/$0ADD` source word without
introducing delayed writes selected by a replay frame number.

`zelda_rtl/intro_poly_schedule.rs` stores **3,300 bytes of work runs**, in nine
module/submodule phases covering 730 iterations, including the outgoing
file-select initialization. Each run gives before-main and pre-NMI native
worker work. A unit is one `PolyWorker::step`; `COMPLETE` runs through the
current render's go/upload publication and returns to its wait state. Worker
instruction prices and raster positions do not select work. The native bus
cursor remains an internal requirement of the shared worker implementation.
The remaining 1,270 iterations use ordinary lagless rules.

The schedule was derived once by advancing the native worker to the immutable
ledger's live-workspace boundaries, without installing reference bytes in the
engine. Complete renders were replaced by `COMPLETE`, then equal work pairs
were run-length encoded. The derivation executable, source patch, complete log,
and expanded work records are retained as `intro-derivation-zelda3`,
`intro-derivation-source.patch`, `intro-derive-menu.log`, and
`intro-work-schedule.json`. The shipped path contains no derivation CLI, target
images, angle/pixel/state tables, source interpreter or runtime ledger reads.
The frozen schedule independently replays **2,000/2,000 clean iterations**.

The intro bank upload finishes before the ledger opens iteration 0 (host 81).
The native bootstrap likewise loads that bank synchronously before main uses
it; it consumes no completed logical iterations. There are no APUI reads in
ledger iterations 0–729 from which to derive another per-iteration handshake
wait. No synthetic hold or audio-clock offset was added. The later reads at
1916–1933 are the iris's status-word high byte, not an intro bank-upload poll.

The schedule is restricted to cold lagless boot; foreign-memory seeds cancel
it. A module/submodule jump, including an intro-skip button, exits the schedule
and resumes ordinary lagless rules. Both feature configurations were checked
with the enhanced skip option and a latched Start edge at iteration 114:
file select is reached immediately and runs 20 further ordinary iterations
without a retained-work panic. Evidence: `intro-skip-check{,-reduced}.log`.
No frame-orchestrator refactor was needed; only the existing before-main and
pre-NMI hooks were extended. All added state fields are serde-skipped, so the
serialized machine layout is unchanged.

## Regression comparison

The interrupted run's main executables and freshly recorded main baselines were
retained and reviewed. Both 200k baselines identify main `85451fc2` and executable
SHA-256 `085c958b6bd0efa045a4152cea5da63dcecbec315a44237f5dd99ed5119a4301`.
The default-mode comparison matches all **200,000 records and EOF**.

Lagless intentionally changes the state hash in **all 200,000 records** and
the display hash in **199,988 records**. Every input, SRAM, PCM, main module,
submodule and room field matches main in every record. The complete list,
including every before/after hash and exact field/range grouping, is
`lag-free-changed-records.json`.

The v2 state hash includes all raw WRAM, so untouched power-on-fill bytes and
the native worker's real stack/publication state continue to change it after
the intro. The display hash includes inactive Mode-7 multiply registers and
their scanline copies. The native worker actually writes those registers;
the previous atomic renderer calculated the rotation in locals. Early CGRAM
also changes with the corrected power-on storage.

Paired renderless presented-state dumps cover hosts 1, 8, 27, 55, 730, 2000 and
200000. At host 200000 they differ at four retained `$55` fill bytes
(`$0C00/$0C01/$1C00/$1C01`) and 11 thread publication/stack bytes. VRAM, OAM,
CGRAM and both CHR identity tables are byte-identical. All 224 scanline records
differ only in the unused Mode-7 A coefficient (`0 -> -38`); current Mode 1
does not consume it. These captures are observational: their first/last ledger
records exactly match the corresponding complete baseline/final streams.
See `regression-attribution.json` and `{main,final}-regression-dumps/`.
This explains retained hash changes; it is not a GPU pixel comparison.

## Final census and verification

Both final full censuses exited **0**, and the summary gate returned **PASS:
zero unexplained differences, complete coverage of all 1,473,123 iterations**.
The first 2,000 iterations are included in the normal segment count and are
clean in both APU modes. Neither census panicked.

The following table applies the same updated summary rules before and after:
intro included, and unused gameplay reads counted against fully clean segments.
The supplied legacy 738/738 count excluded intro and did not count those reads
against its fully-clean total. Polygon-active counts below likewise include
the intro's 730 active iterations.

| Measurement | Before: cartridge APU | After: cartridge APU | Before: live SPC | After: live SPC |
| --- | ---: | ---: | ---: | ---: |
| Iterations | 1,473,123 | 1,473,123 | 1,473,123 | 1,473,123 |
| Segments, including intro | 739 | 739 | 742 | 742 |
| Fully clean segments under current rules | 737 | **739** | 731 | **742** |
| Ordinary-WRAM-clean segments | 739 | 739 | 732 | 742 |
| Ordinary unexplained WRAM iterations | 0 | **0** | 2,109 | **0** |
| Polygon-active iterations | 27,230 | 27,230 | 27,234 | 27,234 |
| Polygon-classified WRAM iterations | 5,957 | 5,931 | 14,731 | 14,705 |
| Lag-classified WRAM iterations | 227 | 224 | 623 | 224 |
| Song-poll-classified WRAM iterations | 0 | 0 | 0 | 2,505 |
| Ordinary RNG unused / extra | 0 / 0 | **0 / 0** | 239 / 557 | **0 / 0** |
| Song-poll RNG unused / extra | 0 / 0 | 0 / 0 | 0 / 0 | 220 / 538 |
| Polygon RNG unused / extra | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 |
| Unused gameplay APU transitions | 2 | **0** | 0 | **0** |
| Classified incidental iris transitions | 0 | 2 | 0 | 0 |
| Ordinary unmatched engine / ROM commands | 0 / 0 | **0 / 0** | 3 / 4 | **0 / 0** |
| Panics | 0 | 0 | 0 | 0 |
| Gate unexplained count | 28 | **0** | 2,938 | **0** |

The live song count includes 396 iterations previously hidden by the blanket
lag exception: `2,109 + 396 = 2,505`. This is a more precise causal assignment,
not an additional execution divergence. The three removed width-counter
differences explain the remaining lag-count reduction. The 26 removed intro
initialization differences explain the polygon-count reduction; the newly
compared partial-worker storage is also exact.

The final compared-WRAM partition is complete and disjoint:

| Category | Cartridge APU iterations | Live SPC iterations | Justification |
| --- | ---: | ---: | --- |
| Clean | 1,466,968 | 1,455,689 | No difference in the compared representation |
| Accepted: rupee sound queue | 220 | 220 | Audited `$0CFD` branch and lag-carried counter |
| Accepted: post-NMI upload sample overrun | 4 | 4 | Audited map/flute phase signature and next-iteration reconvergence |
| Accepted: polygon thread / aftermath | 5,931 | 14,705 | NMI-driven worker/foreign-seed phase; crystal stable outcomes proved |
| Accepted: song-end poll | 0 | 2,505 | Actual zero/nonzero poll disagreement; all ten uninterrupted outcomes proved |
| Unexplained | **0** | **0** | Gate rejects ordinary misses |
| Total | **1,473,123** | **1,473,123** | Every iteration accounted for |

Other accepted domains use their own units and are not added to that partition:

| Category | Cartridge APU | Live SPC | Justification |
| --- | ---: | ---: | --- |
| Incidental iris read transitions | 2 | 0 | Discarded `$213F` high byte; no native gameplay poll |
| Song-poll RNG unused / extra | 0 / 0 | 220 / 538 | Recorded RNG demand shifts with the receipt branch |
| Polygon RNG unused / extra | 0 / 0 | 19 / 19 | Random cutscene effects shift with seeded worker completion |
| Lag-only commands | 271 | 1,144 | Commands delivered by extra cartridge NMIs |
| Polygon-thread commands | 180 | 386 | Commands during the NMI-driven thread phase |
| Song-poll commands | 0 | 7 | Commands moved by the observed receipt branch |
| Held-port re-sends | 1,884 | 1,737 | Native acknowledgement retains the already delivered value |

Engine / ROM command totals are 65,273 / 67,288 with cartridge answers and
64,734 / 67,287 with live SPC; ordinary unmatched commands are zero. These are
window-matching measurements, not a subtraction identity between command totals.
Video is deliberately **0/0**, since GPU/video execution was forbidden.

Before/after logs, summaries and machine-readable totals are under
`target/lagless-gaps/`: `before-{rom,live}-complete.log`,
`before-{rom,live}-summary.txt`, `before-census-totals.json`,
`final-{rom,live}-census.log`, `final-{rom,live}-summary.txt`, and
`final-census-totals.json`. The earlier interrupted logs are preserved separately.
The complete before-censuses use retained `probe-zelda3`: its engine behavior is
main's, with poll observation and read-only WRAM-export reuse for comparison
speed. Its prefix segment/polygon records match main byte-for-byte in both APU
modes (`comparison-optimization-proof.json`). The 200k baselines themselves use
the actual unmodified main executable, not that probe.
The final classification trace confirms all 220 counter cases and four overruns,
and contains no entry at any of the three fixed exploding-wall iterations.
`polygon-inventory.json` records every remaining cartridge polygon context and
address, including save/room/health differences during the shifted warp phases;
those are not claims of byte equality before the stable boundary.

The gate was also exercised against clean and deliberately invalid evidence.
It rejects ordinary and timing-hidden intro misses, hidden intro RNG misses,
unused reads, panics, missing terminal records, missing coverage and gaps.
All nine checks behave as expected (`final-gate-checks.json`).

Completed verification:

| Check | Result |
| --- | --- |
| Reduced parity binary build | Exit 0, no warnings |
| Default parity binary build | Exit 0, no warnings |
| Reduced Zelda library suite | **1,989 passed, 0 failed, 2 ignored**, exit 0 |
| Default Zelda library suite | **2,454 passed, 1 known failure, 3 ignored**, exit 101 |
| SNES library suites, both feature sets | **504 passed, 0 failed, 7 ignored** each |
| Default mode vs freshly recorded main ledger | **200,000 records and EOF match**, exit 0 |
| Reduced lagless vs main ledger | All 200,000 records compared; intentional state/display changes exhaustively listed |
| Default lagless vs reduced final ledger | **200,000 records and EOF match**, exit 0 |
| Full cartridge-answer census / summary gate | Exit 0 / PASS, complete, unexplained 0 |
| Full live-SPC census / summary gate | Exit 0 / PASS, complete, unexplained 0 |
| Intro skip, both feature configurations | Immediate file select at iteration 114; 20 further iterations pass |
| `git diff --check` | Pass |

The sole default failure is the permitted existing
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
Library test compilation has zero reduced warnings and the same 24 existing
default test-fixture warnings documented in `rom-timing-gate.md`; both ordinary
binary builds are warning-free. Library suites were built with Cargo's `--lib
--no-run`, then their retained test executables were run directly with
`ZELDA3_ASSETS=$PWD/zelda3_assets.dat`. This allows independent CPU checks without
holding Cargo's artifact lock. No renderer/GPU test suite was invoked.

Final binaries:

- Reduced: `target/lagless-gaps/verified-reduced-zelda3`, SHA-256
  `6fc19e27d25277325e4cb1b9e01868cca15aa36e10835346224f4786f1708a8d`.
- Default: `target/lagless-gaps/verified-default-zelda3`, SHA-256
  `58f919a400b87e73dc8415ff5cf5b8c7f6a89054bd12a300d17e6b8c6c4e88a5`.

Reproduce the two final gates with the supplied read-only `LEDGER` and `C`:

```sh
BIN=target/lagless-gaps/verified-reduced-zelda3
ZELDA3_SAVE_DIR=/tmp/lagless-check-rom "$BIN" --logical-replay "$LEDGER" "$C/initial.srm" \
  --report 0 --seed-every 2000 > /tmp/lagless-check-rom.log
ZELDA3_SAVE_DIR=/tmp/lagless-check-live "$BIN" --logical-replay "$LEDGER" "$C/initial.srm" \
  --report 0 --seed-every 2000 --live-apu-reads > /tmp/lagless-check-live.log
python3 scripts/logical_census_summary.py /tmp/lagless-check-rom.log --gate --expect-iterations 1473123
python3 scripts/logical_census_summary.py /tmp/lagless-check-live.log --gate --expect-iterations 1473123
```

HEAD remains `85451fc2` on `codex/lagless-gaps`. No commit, push, `cargo fmt`,
GPU, `--video`, cached-A/V or Snes9x-core run was performed. The oracle cache
and the other worktree were read-only throughout.
