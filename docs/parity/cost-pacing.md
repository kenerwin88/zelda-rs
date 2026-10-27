# Lagless cost accounting

New replays use the corrected `full-v2` ledger; `full` remains the immutable
backup used by earlier receipts below. See [Continuous lagless 2](#continuous-lagless-2).

Partial stopping point on `codex/cost-pacing`, based on main `276b4b92`.
Production cost holds are enabled. This batch implements the overworld map
charge group and two dispatcher self charges; room/object/stair, choice-menu,
dungeon-map and gameplay annotation groups remain unfinished. The complete
verification and count tables below are filled from retained CPU receipts.

## Runtime rule

The shipped `cycle_ledger` instruction prices are the work counter. Source
`lag_nmis` never enters the runtime. All new prices are gated by `lag_free`;
preview decompression passes `None` and remains uncharged. Default and
ROM-timing paths retain their previous ledger values and scheduling.

Let `F = 1364*262 = 357368`, `H` be the preceding ordinary closing NMI handler's CPU+DMA
work, `C` be main's charged work plus the existing VWF counter, and `W` be
CPU ownership between the NMI handler's return and the polygon V-IRQ, plus
the IRQ handler. Without an active worker, `W = 0`. The defensive caps keep
`F-H-W` positive. The candidate elapsed estimates are:

- P0: `H + floor(C*F/(F-W))`.
- P1: `floor(C*F/(F-H-W))`.
- P2: `floor((C + 40*ceil(C/1324))*F/(F-H-W))`.

Holds are `floor(elapsed/F)`, counting equality. P2 currently drives production.
Every main callback captures its own preceding ordinary closing handler. The
handler value is retained before presentation holds overwrite the engine's
last-callback counter; held callbacks cannot replace it or re-arm accounting.
Masked song uploads retain the capture request until their actual closing NMI.
The worker also retains that closing handler's upload fact; presentation-only
lag callbacks must not replace either part of its next leading handoff.
The worker stays ledger-muted, and its programmed
NMI-to-V-IRQ window reduces the available field instead of becoming main work.
Existing polygon phase execution is not repeated in presentation holds.

`native_wait_until_line(L, loop_master)` is a generic raster barrier. For each
variant it places the current work at `225*1364 + elapsed + prior_waits`,
modulo `F`, then adds enough whole spin passes to reach L if below it. Iris
uses line 192 and the source's 178-clock taken pass at `$00:F3A3..F3B2`;
the existing final 172-clock pass remains charged exactly once. The coarse
iris scheduler has been removed; the full comparison below checks
the raster-barrier replacement against its retained before counters.
There are no scene thresholds, scene multipliers or duration tables.

The existing song uploader continues to own masked callbacks and its SPC
acknowledgements. Masked elapsed frames are separate from accepted lag NMIs.

## Accuracy progression

Each complete cartridge pass covers 1,473,123 iterations, 39,575 lagged
iterations and 102,677 source lag NMIs. Predictions below include all
iterations; exact/within-one use lagged iterations only.

| Stage | Rule | Predicted frames | Exact lagged | Within one lagged | False positives |
| --- | --- | ---: | ---: | ---: | ---: |
| Original counter | P0 | 65,092 | 16,581 | 36,540 | 0 |
| Map charge group | P0 | 74,432 | 16,876 | 37,147 | 0 |
| Original counter | P1 | 70,082 | 19,057 | 37,049 | 0 |
| Map charge group | P1 | 81,210 | 19,518 | 37,722 | 0 |
| Original counter | P2 | 73,700 | 21,385 | 37,094 | 0 |
| Map charge group | P2 | 85,324 | 21,579 | 37,772 | 0 |
| Original production | Coarse iris | 11,178 | 10,115 | 30,283 | 0 |
| First production trial | P2; last-held handler history | 89,780 | 27,602 | 38,244 | 17 |
| Final dispatcher + runtime batch | P2; ordinary closing handoff | 95,663 | 30,512 | 38,375 | 0 |

The map-only full census gate is PASS, unexplained zero. Production was
unchanged at this stage: 11,178 accepted holds and 2,843 masked callbacks.
The next stage combines the two small dispatcher self charges, the generic
raster barrier, polygon ownership, production holds and the sound boundary.

## Source-derived charge group

Prices came from this checkout's `scripts/rom_block_costs.py` and the pinned
ROM, with 16-bit accumulator/index entry where required. Retained listings
are in `target/cost-pacing/source-costs/`. Each new charge has its address
range in a source comment; calls price only the call instruction, while
native child bodies price their executed work.

| Work | Instruction derivation |
| --- | --- |
| Bank-02 map decompression | `$02:FEBB..FF5C`: header/short or extended length dispatch, literal, byte-fill, word-fill, incrementing-fill and copy loops, including taken branches per emitted byte. `$02:FF5F..FF6D` costs 190 per ordinary input byte or 246 at a LoROM pointer wrap; immutable native stream pointers supply that distinction. Preview decoding is free. |
| Quadrant setup/interlace | `$02:F595..F60E` setup/calls: 1,812. High interlace `$F638..F678`: `282 + 64*552 + 63*6 + 92`; low interlace `$F679..F68F -> F650..F678`: same loop with 294 setup. |
| Quadrant tile/row control | `$02:F610..F637`: 390 per tile plus the actual inner taken branches, 56 per row prefix, 142 per row suffix, actual outer taken branches, and 70 final return. Both whole-quadrant and store-cursor paths price the same work. |
| Map32 definitions | `$02:F691..F69A`: 110 compare; cache hit adds 24, miss adds the taken branch plus 3,166 for `$F69D..F7AB`. Four stores and return `$F7AD..F7CA` cost 580, split as 176/102/130/172 when executing the store cursor. |
| Map16 to Map8 | `$02:FD46..FD86`: wrapper/source/Y/loop control; `$FD87..FDC2`: 224 setup plus sixteen paired stripe-copy loops of 594 and actual branches; `$FDC4..FDF3`: destination setup with its actual upper/lower-half branch; `$FDF7..FE46`: two headers and 32 four-store tile loops per row, including returns and actual loop branches. |
| Auxiliary graphics dispatcher | `$02:AB88..ABAC`: four event-bit read/mask/write triples, 384. `$ABB0..ABC5`: calls/request stores/submodule/return, 298. Existing graphics child prices are unchanged. |
| New map/GFX dispatcher | `$02:ABC6..ABD9`: stores and calls only, 306. |

No new scroll-loop ledger price was added: on the lagless call path the
existing `scroll_master_cycles` already enters `lag_free_vwf_master` through
the RenderText command accounting. Charging the pixel-copy loop again would
double-count it. Choice commands and fresh sprite/lamp prefix coverage remain
incomplete. This refines the earlier direct-charge-only audit without treating
its missing-self-charge list as an inclusive profiler.

## Sound boundary and logical comparison

Post-call holds alone exposed an ordering failure at iteration 162653: native
HUD refill saw queued effect `$60` and reset `$0CFD`; the source had consumed
that effect during the long main call and incremented the counter. The total
lag counts already agreed. The cost boundary now retains the pending effect
queues before HUD refill whenever the general rule has crossed an NMI. The
first presentation callback publishes that queue; the final callback publishes
commands queued by the remaining suffix. The source operation is the NMI's
`$00:811E..812D` transfer-and-clear. This uses work and queued commands only,
with no iteration, message, module or source-lag lookup.

The command comparator now also accepts an identical raw source port write
in its existing k/k+1 comparison window. Native held NMIs can clear/echo a
port between equal requests that the source edge filter coalesces. For example,
source iterations 4148–4150 each write port 3 value `$0C`; the prior comparator
wrongly reported native resends on 4149–4150 as extras. Absent requests still
fail. The `$0CFD` classification rule was not widened.

This retains a sound queue boundary, not a complete suspended CPU, partial
map display or glyph-by-glyph PCM timing model. All scenes still publish most
completed main state atomically before their presentation callbacks. Logical
census PASS is separate from video, unseeded-route and exact audio acceptance.

## Verification receipts

Evidence is retained under `target/cost-pacing/`. `before-default-zelda3` was
built from the clean requested main before editing and recorded a fresh
200,000-frame default state/display/SRAM/PCM/input/context baseline. The first
changed default comparison matches all 200,000 records and EOF; the final
build comparison is recorded separately.

`group1.csv` is the full cartridge census after map charges, before production
rule changes. `final-{rom,live}.csv` contain the final candidates and all three
work/raster/worker variants. Original `predicted*` columns retain the work-only
P0/P1/P2 formulas for comparison with the preceding measurement. `cost_p*`
columns expose the new runtime variants. `native_held` is selected by P2; `native_elapsed` is masked song upload time.
The final CSVs also expose `cost_entry_handler`, distinct from the
legacy diagnostic's literal `preceding_nmi_master` after a held callback.

`before-probes` and `after-probes` retain fresh uninterrupted live-SPC windows
for eight crystal/pendant phases and iteration 162000–163000, including PCM.
The aborted `trial-before-sound-*` censuses are diagnostic prefixes, not full
receipts. The initial library invocation omitted `ZELDA3_ASSETS` and failed
asset-dependent fixtures; only the corrected, asset-backed final suites count
as validation.

## Reproduction

```sh
LEDGER=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
CACHE=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
BIN=target/cost-pacing/worker-final-zelda3
ZELDA3_SAVE_DIR=/tmp/cost-pacing-repro "$BIN" --logical-replay "$LEDGER" "$CACHE/initial.srm" \
  --report 0 --seed-every 2000 --pacing-predictor /tmp/cost-pacing-rom.csv \
  > /tmp/cost-pacing-rom.log
python3 scripts/logical_census_summary.py /tmp/cost-pacing-rom.log --gate --expect-iterations 1473123
# Repeat with --live-apu-reads and separate output/save paths.
```

No GPU, `--video`, cached-A/V, Snes9x execution, cargo GPU tests, `cargo fmt`,
commit or push was used. The supplied ledger and cache are read-only. Analysis
scripts live in `/tmp`; no scratch program was added to the repository.

## Final route results

P2 wins the full-route comparison by both total absolute count error and
exact lagged-iteration counts. It drives every lagless main iteration; the
coarse iris scheduler is removed. These are CPU logical-state/count gates
with periodic seeds and source RNG/APU answers where selected, not exact
video/audio or an unseeded quest proof.

| Mode / rule | Exact lagged / 39,575 | Within one / 39,575 | Held or predicted / 102,677 | Absolute error | Zero-lag false positives |
| --- | ---: | ---: | ---: | ---: | ---: |
| Cartridge answers: P0 | 26,101 | 37,453 | 84,148 | 18,631 | 0 |
| Cartridge answers: P1 | 28,940 | 38,260 | 92,213 | 13,644 | 0 |
| Cartridge answers: P2 production | 30,512 | 38,375 | 95,663 | 12,100 | 0 |
| Live SPC: P0 | 25,997 | 37,348 | 84,083 | 19,626 | 131 |
| Live SPC: P1 | 28,824 | 38,140 | 92,133 | 14,754 | 131 |
| Live SPC: P2 production | 30,384 | 38,253 | 95,580 | 13,251 | 131 |

Accepted holds can include overcounts. Capping each iteration at its source count gives:

| Mode | Capped restored frames | Missing frames | Excess frames | Masked elapsed callbacks |
| --- | ---: | ---: | ---: | ---: |
| Cartridge answers | 93,120 | 9,557 | 2,543 | 2,825 |
| Live SPC | 92,503 | 10,174 | 3,077 | 2,810 |

The first production trial restored 89,780 / 89,807 frames (cartridge / live),
but presentation holds replaced the next main/worker's ordinary handler
cost and upload fact with the last held callback's facts. This introduced
8,199 cartridge polygon-classified differences. Retaining both ordinary
closing-handler facts and using them for the worker's leading phase fixes
that regression; the final classifications below are reported separately
from equality. No exception was widened to accept it.

Cartridge WRAM partitions as **1,473,109 clean + 14 lag-classified**.
Live SPC partitions as **1,457,730 clean + 14,741 polygon + 638 song-poll +
14 lag-classified**. Both have unexplained zero. The final false-positive
rates on zero-lag iterations are **0% cartridge / 0.00914% live**.

### Complete category table

Paired cells are **cartridge / live**. Exact and within-one counts use only
that category's lagged iterations. Under/over count frame deficits/excesses,
not net error. Every source-lag category and any category with a false
positive is shown; the JSON/CSV receipts retain all zero-lag categories too.

| Entry | Source lag frames | Lagged calls | Held C / L | Exact C / L | Within one C / L | Under C / L | Over C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 4,418 | 13,162 / 13,065 | 4,212 / 4,102 | 4,418 / 4,308 | 4 / 324 | 202 / 425 | 0 / 96 |
| `07/02` | 8,770 | 1,976 | 7,714 / 7,714 | 1,189 / 1,189 | 1,807 / 1,807 | 1,068 / 1,068 | 12 / 12 | 0 / 0 |
| `06/00` | 8,196 | 142 | 8,239 / 8,239 | 75 / 75 | 142 / 142 | 12 / 12 | 55 / 55 | 0 / 0 |
| `07/0E` | 7,979 | 4,952 | 7,729 / 7,729 | 4,757 / 4,757 | 4,923 / 4,923 | 250 / 250 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 157 | 6,323 / 6,161 | 129 / 129 | 150 / 150 | 146 / 308 | 3 / 3 | 0 / 0 |
| `09/23` | 6,054 | 1,179 | 5,396 / 5,396 | 663 / 663 | 1,085 / 1,085 | 736 / 736 | 78 / 78 | 0 / 0 |
| `0F/01` | 5,432 | 4,879 | 5,029 / 5,029 | 4,476 / 4,476 | 4,879 / 4,879 | 403 / 403 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 251 | 4,588 / 4,588 | 35 / 35 | 251 / 251 | 0 / 0 | 216 / 216 | 0 / 0 |
| `07/07` | 4,060 | 2,919 | 3,234 / 3,234 | 2,117 / 2,117 | 2,905 / 2,905 | 828 / 828 | 2 / 2 | 0 / 0 |
| `07/00` | 3,679 | 2,960 | 1,929 / 2,031 | 1,208 / 1,208 | 2,960 / 2,960 | 1,751 / 1,751 | 1 / 103 | 0 / 18 |
| `09/01` | 3,479 | 251 | 3,751 / 3,751 | 15 / 15 | 215 / 215 | 0 / 0 | 272 / 272 | 0 / 0 |
| `07/0F` | 2,992 | 2,718 | 2,799 / 2,799 | 2,525 / 2,525 | 2,718 / 2,718 | 193 / 193 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 115 | 2,748 / 2,748 | 39 / 39 | 76 / 76 | 37 / 37 | 113 / 113 | 0 / 0 |
| `10/01` | 2,672 | 2,669 | 2,669 / 2,672 | 2,666 / 2,664 | 2,669 / 2,668 | 3 / 4 | 0 / 4 | 0 / 0 |
| `08/02` | 2,576 | 157 | 3,332 / 3,251 | 0 / 0 | 0 / 0 | 16 / 97 | 772 / 772 | 0 / 0 |
| `07/15` | 2,333 | 1,735 | 2,167 / 2,167 | 1,577 / 1,577 | 1,731 / 1,731 | 166 / 166 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 1,855 | 1,996 / 1,996 | 1,850 / 1,850 | 1,855 / 1,855 | 5 / 5 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 90 | 1,787 / 1,787 | 0 / 0 | 16 / 16 | 191 / 191 | 133 / 133 | 0 / 0 |
| `09/00` | 1,253 | 1,192 | 86 / 86 | 25 / 25 | 1,192 / 1,192 | 1,167 / 1,167 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 56 | 979 / 979 | 0 / 0 | 2 / 2 | 140 / 140 | 54 / 54 | 0 / 0 |
| `07/06` | 984 | 603 | 935 / 935 | 572 / 572 | 592 / 592 | 51 / 51 | 2 / 2 | 0 / 0 |
| `11/00` | 973 | 17 | 1,012 / 1,012 | 0 / 0 | 1 / 1 | 0 / 0 | 39 / 39 | 0 / 0 |
| `07/01` | 928 | 928 | 922 / 922 | 922 / 922 | 928 / 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 157 | 833 / 857 | 13 / 13 | 154 / 152 | 10 / 21 | 137 / 172 | 0 / 0 |
| `05/00` | 639 | 11 | 549 / 549 | 0 / 0 | 0 / 0 | 90 / 90 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 251 | 0 / 0 | 0 / 0 | 0 / 0 | 508 / 508 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 391 | 391 / 391 | 310 / 310 | 391 / 391 | 81 / 81 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 287 | 397 / 397 | 235 / 235 | 287 / 287 | 52 / 52 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 239 | 330 / 330 | 182 / 182 | 237 / 237 | 61 / 61 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 100 | 282 / 282 | 10 / 10 | 88 / 88 | 98 / 98 | 17 / 17 | 0 / 0 |
| `07/16` | 303 | 101 | 0 / 0 | 0 / 0 | 0 / 0 | 303 / 303 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 16 | 287 / 287 | 3 / 3 | 16 / 16 | 0 / 0 | 13 / 13 | 0 / 0 |
| `09/16` | 265 | 233 | 81 / 81 | 49 / 49 | 233 / 233 | 184 / 184 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 223 | 84 / 84 | 60 / 60 | 223 / 223 | 163 / 163 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 81 | 245 / 245 | 79 / 79 | 81 / 81 | 0 / 0 | 2 / 2 | 0 / 0 |
| `09/0F` | 242 | 16 | 320 / 320 | 0 / 0 | 0 / 0 | 0 / 0 | 78 / 78 | 0 / 0 |
| `01/00` | 235 | 4 | 155 / 155 | 0 / 0 | 0 / 0 | 80 / 80 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 36 | 217 / 217 | 28 / 28 | 31 / 31 | 16 / 16 | 5 / 5 | 0 / 0 |
| `10/00` | 157 | 157 | 157 / 217 | 157 / 154 | 157 / 155 | 0 / 1 | 0 / 61 | 0 / 0 |
| `17/01` | 141 | 3 | 114 / 114 | 0 / 0 | 0 / 0 | 27 / 27 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 9 | 138 / 138 | 5 / 5 | 8 / 8 | 3 / 3 | 2 / 2 | 0 / 0 |
| `09/20` | 138 | 28 | 148 / 148 | 18 / 18 | 28 / 28 | 0 / 0 | 10 / 10 | 0 / 0 |
| `0E/05` | 138 | 119 | 2 / 2 | 1 / 1 | 101 / 101 | 136 / 136 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 11 | 48 / 48 | 1 / 1 | 8 / 8 | 66 / 66 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 93 | 17 / 17 | 7 / 7 | 93 / 93 | 86 / 86 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 28 | 112 / 112 | 0 / 0 | 28 / 28 | 0 / 0 | 28 / 28 | 0 / 0 |
| `09/0E` | 83 | 16 | 88 / 88 | 11 / 11 | 16 / 16 | 0 / 0 | 5 / 5 | 0 / 0 |
| `15/07` | 83 | 79 | 84 / 84 | 78 / 78 | 79 / 79 | 0 / 0 | 1 / 1 | 0 / 0 |
| `19/00` | 72 | 4 | 89 / 89 | 0 / 0 | 2 / 2 | 1 / 1 | 18 / 18 | 0 / 0 |
| `07/13` | 71 | 47 | 71 / 71 | 47 / 47 | 47 / 47 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 65 | 9 / 9 | 6 / 6 | 65 / 65 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 4 | 73 / 73 | 0 / 0 | 3 / 3 | 0 / 0 | 5 / 5 | 0 / 0 |
| `0B/2B` | 66 | 66 | 16 / 16 | 16 / 16 | 66 / 66 | 50 / 50 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 | 74 / 74 | 0 / 0 | 2 / 2 | 0 / 0 | 10 / 10 | 0 / 0 |
| `15/03` | 64 | 12 | 62 / 62 | 9 / 9 | 11 / 11 | 4 / 4 | 2 / 2 | 0 / 0 |
| `00/05` | 62 | 62 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 4 | 80 / 80 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 0 / 0 |
| `0B/1A` | 60 | 4 | 78 / 78 | 0 / 0 | 0 / 0 | 0 / 0 | 18 / 18 | 0 / 0 |
| `1A/12` | 60 | 3 | 70 / 70 | 0 / 0 | 1 / 1 | 0 / 0 | 10 / 10 | 0 / 0 |
| `0B/29` | 59 | 51 | 18 / 18 | 2 / 2 | 51 / 51 | 45 / 45 | 4 / 4 | 0 / 0 |
| `18/00` | 59 | 2 | 64 / 64 | 1 / 1 | 1 / 1 | 0 / 0 | 5 / 5 | 0 / 0 |
| `1A/00` | 59 | 3 | 68 / 68 | 0 / 0 | 2 / 2 | 0 / 0 | 9 / 9 | 0 / 0 |
| `1A/1A` | 59 | 3 | 69 / 69 | 0 / 0 | 1 / 1 | 0 / 0 | 10 / 10 | 0 / 0 |
| `1A/1C` | 59 | 3 | 69 / 69 | 0 / 0 | 1 / 1 | 0 / 0 | 10 / 10 | 0 / 0 |
| `1A/04` | 58 | 3 | 68 / 68 | 0 / 0 | 1 / 1 | 0 / 0 | 10 / 10 | 0 / 0 |
| `1A/08` | 58 | 3 | 68 / 68 | 0 / 0 | 1 / 1 | 0 / 0 | 10 / 10 | 0 / 0 |
| `1A/0A` | 58 | 3 | 69 / 69 | 0 / 0 | 1 / 1 | 0 / 0 | 11 / 11 | 0 / 0 |
| `1A/18` | 58 | 3 | 68 / 68 | 0 / 0 | 1 / 1 | 0 / 0 | 10 / 10 | 0 / 0 |
| `1A/0E` | 57 | 3 | 68 / 68 | 0 / 0 | 1 / 1 | 0 / 0 | 11 / 11 | 0 / 0 |
| `1A/10` | 56 | 3 | 67 / 67 | 0 / 0 | 1 / 1 | 0 / 0 | 11 / 11 | 0 / 0 |
| `1A/06` | 55 | 3 | 65 / 65 | 0 / 0 | 1 / 1 | 0 / 0 | 10 / 10 | 0 / 0 |
| `07/03` | 53 | 53 | 0 / 0 | 0 / 0 | 53 / 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 2 | 59 / 59 | 0 / 0 | 0 / 0 | 0 / 0 | 6 / 6 | 0 / 0 |
| `11/07` | 51 | 17 | 51 / 51 | 17 / 17 | 17 / 17 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 51 | 51 / 51 | 51 / 51 | 51 / 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 2 | 57 / 57 | 0 / 0 | 0 / 0 | 0 / 0 | 6 / 6 | 0 / 0 |
| `0B/1C` | 48 | 40 | 18 / 18 | 10 / 10 | 40 / 40 | 30 / 30 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 1 | 47 / 47 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 1 | 46 / 46 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 | 45 / 45 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 14 | 42 / 42 | 13 / 0 | 14 / 7 | 0 / 41 | 1 / 42 | 0 / 14 |
| `1A/20` | 41 | 1 | 124 / 124 | 0 / 0 | 0 / 0 | 0 / 0 | 83 / 83 | 0 / 0 |
| `09/12` | 32 | 16 | 0 / 0 | 0 / 0 | 0 / 0 | 32 / 32 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 4 | 32 / 32 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 4 | 21 / 21 | 0 / 0 | 0 / 0 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 13 | 20 / 20 | 4 / 4 | 13 / 13 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 4 | 12 / 12 | 0 / 0 | 0 / 0 | 12 / 12 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 23 | 23 / 23 | 23 / 23 | 23 / 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 14 | 0 / 0 | 0 / 0 | 14 / 14 | 14 / 14 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 1 | 16 / 16 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 0 / 0 |
| `01/02` | 7 | 7 | 0 / 0 | 0 / 0 | 7 / 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 3 | 6 / 6 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 4 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 3 | 3 / 3 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 1 | 2 / 2 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 | 0 / 68 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 68 | 0 / 3 |

### Iris replacement

The comparison tags every iteration where the former scheduler held,
including shared callers rather than a hand-picked module subset.

| Same iris calls | Before coarse scheduler | Final general rule |
| --- | ---: | ---: |
| Exact counts | 10,115 | 10,446 |
| Within one | 11,174 | 11,178 |
| Total absolute error | 1,073 | 732 |
| Held NMIs | 11,178 | 11,519 |

The common footprint is 11,178 calls and 12,251 source lag NMIs. The generic wait barrier is at least as accurate, so the coarse scheduler was removed.

### Crystal and pendant phases

Each cell is **source presentation frames / clean-main before / final**.
Masked song callbacks are included in presentation duration, not accepted
lag totals. The final stable boundaries retain matching room, Link position
and the entire `$F000..$F4FF` save block in all eight uninterrupted probes.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 26 / 67 | 5 / 1 / 6 | 17 / 1 / 22 | 2 / 2 / 2 | 35 / 34 / 34 | 16 / 16 / 16 |
| 380000 | 84 / 26 / 68 | 7 / 1 / 6 | 24 / 1 / 21 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 460000 | 84 / 27 / 67 | 7 / 1 / 6 | 24 / 1 / 22 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 626000 | 91 / 26 / 71 | 9 / 1 / 7 | 24 / 1 / 22 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 734000 | 84 / 26 / 68 | 7 / 1 / 6 | 25 / 1 / 23 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 880000 | 84 / 26 / 67 | 7 / 1 / 6 | 26 / 1 / 23 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1064000 | 84 / 26 / 67 | 7 / 1 / 5 | 25 / 1 / 23 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1316000 | 84 / 26 / 66 | 7 / 1 / 5 | 25 / 1 / 23 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |

The table is a mixed result: most crystal builds improve substantially, while P2 can overcount the pendant build. There is no per-crystal correction. Parent/child annotation gaps and the rule's repeated-handler approximation remain visible.

### Exceptions and rupee audio

| Mode | Final classified differences | Final $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14741, "song-end-poll": 638}` | 10 |

Before: 224 lag-classified iterations in the full map-only census, consistent with the prior audited 220 `$0CFD` cases plus four map/flute overruns. The fresh 162000–163000 before probe has 210 `$0CFD` differences; the final probe has 0. The existing start/carry rule is unchanged.

Rupee requests remain on the same 27 logical iterations in the first refill burst. Before, every presentation gap is 8 frames. Final gaps are `[24, 8, 8, 8, 8, 10, 25, 16, 8, 8, 8, 8, 8, 14, 28, 12, 8, 8, 8, 8, 8, 14, 12, 8, 8, 8]`; source gaps at those same requests are `[24, 8, 8, 8, 8, 12, 25, 14, 8, 8, 8, 8, 8, 19, 26, 8, 8, 8, 8, 8, 8, 14, 12, 8, 8, 8]`. Drawing intervals are longer, while dismissal/input waits still have 8-frame cadence. This is partial cadence restoration.

The first band/envelope PCM-onset gap changes from 8.127 to 23.644 frames; the native first request gap changes from 8 to 24. All measured PCM gaps and samples are retained in `final-probe-metrics.json`. The 7.9–8.7 kHz / threshold-600 detector is an onset diagnostic, not exact PCM acceptance.

### Largest remaining deficits and known causes

These are audited accounting gaps, not exclusive per-child attributions:
the supplied ledger has no per-iteration main-PC profile. Missing and excess
frames are kept separate; a net total can hide both.

| Entry | Missing source frames | Excess holds | Known remaining gap |
| --- | ---: | ---: | --- |
| `07/00` | 1,751 | 1 | Ordinary gameplay sprite/control annotation gaps. |
| `09/00` | 1,167 | 0 | Ordinary gameplay sprite/control annotation gaps. |
| `07/02` | 1,068 | 12 | Room-loader parent and object/drawing self prices remain incomplete. |
| `07/07` | 828 | 2 | Falling-transition/room-loader parents and drawing work remain incomplete. |
| `09/23` | 736 | 78 | Shared mirror/overlay parents and remaining children; field-overhead estimate. |
| `09/04` | 508 | 0 | Sprite reload/scan children and the reload parent remain unpriced. |
| `0F/01` | 403 | 0 | Raster barrier improves counts; caller/CPU/interrupt estimates still leave closing misses. |
| `07/16` | 303 | 0 | Peg-update map/tile work remains outside this charge batch. |
| `07/0E` | 250 | 0 | Spiral-stair parent, Link stair handling and room/object prices remain incomplete. |
| `07/0F` | 193 | 0 | Landing-wipe/caller instruction coverage and field-overhead approximation. |
| `0E/0A` | 191 | 133 | Flute/map dispatcher and drawing loops remain incomplete. |
| `09/16` | 184 | 0 | Mosaic-return sprite/control/dispatcher coverage remains incomplete. |
| `07/15` | 166 | 0 | Warp/room-load parents and object work remain incomplete. |
| `09/0D` | 163 | 0 | Mosaic-transition parent/sprite/control prices remain incomplete. |
| `08/00` | 146 | 3 | Properties/prefix/suffix and graphics-parent prices remain incomplete. |
| `0E/07` | 140 | 54 | Map dispatcher/drawing loops and field-overhead estimate remain incomplete. |

### Remaining work

This is the requested verified partial stopping point. No duration table or
context-specific suppression was added. The largest remaining frame deficits
are listed above rather than hidden by net totals. Known unfinished work:

- Overworld properties/overlay/screen-build parents, `PrepTransAuxGfx` self work,
  sprite reload/initial-scroll parents and remaining overlay/bomb-door work.
- Room-loader parents, Type1/subtype1 object dispatch, 1x3/5x4/pushable-block
  drawing and spiral-stair parents. Existing charged children remain in C.
- Text choice commands and fresh sprite/lamp prefix; scroll copy is already in
  the VWF aggregate and must not be charged twice.
- Dungeon-map dispatcher, border and room-row loops; ordinary sprite/control
  gaps in gameplay; unpriced boot/cutscene helpers.
- Generic repeat-handler and refresh estimates are approximations. A large
  preceding update/DMA handler can overstate intervening lag-handler cost;
  current prices can understate actual main work. Matching a count does not
  establish internal event or partial-display placement.

False positives are not suppressed by category. In live-SPC comparisons some
native/source phase shifts are already classified by the existing song/polygon
contract; a native loader can run against a source zero-lag opening context.
The final table reports those counts and classifications explicitly.

### Validation scope

| Check | Result |
| --- | --- |
| Final default and reduced ordinary builds | PASS; no warnings |
| Fresh clean-main default baseline versus final build | 200,000 records and EOF match |
| Full final cartridge-answer census | PASS; 1,473,123 iterations, unexplained zero |
| Full final live-SPC census | PASS; 1,473,123 iterations, unexplained zero |
| Zelda library case coverage, default | 2,459 pass; only the known spotlight failure; 3 ignored |
| Zelda library case coverage, reduced | 1,994 pass; zero failures; 2 ignored |
| Final changed-module pacing/handoff regressions | 5 pass in each build |
| SNES library suites, both configurations | 504 pass, zero failures, 7 ignored each |
| Eight stable crystal/pendant source outcomes | PASS |
| Complete pricing / exact publication / full audio acceptance | Not established |
| `git diff --check` | PASS |

The full Zelda suites ran on the sound-boundary candidate before coarse-iris
retirement and the late closing-handler/worker-handoff repairs. All five final
changed-module cases were then rerun in both configurations. `lib-coverage.json`
checks current test-name coverage from the complete suites plus these final
regressions, excludes the one retired coarse-iris test, and lists no missing
current cases. This is combined validation, not a claim that a second monolithic
suite was run after the late repairs. Final full-route gates and the default
baseline comparison use the final worker-handoff binaries.

Default lib-test compilation retains the 24 known unused-fixture warnings;
reduced lib-test compilation and both ordinary binaries are warning-free.
The short 162000-start sound-boundary summary rejects full-route contiguity
(`complete=False`) despite unexplained zero; it is a focused diagnostic, not a
full gate receipt. Only the final complete terminal gates above count as PASS.

The exact allowed default failure is
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
`verification.json`, `lib-coverage.json`, `iris-comparison.json`, final metrics,
phase/outcome/PCM records and `artifact-hashes.json` bind the evidence. Earlier
`trial-*`, `general-*`, `closing-*` and with-floor files retain diagnostics;
they do not replace the final worker-handoff receipts.

## Continuation on `codex/cost-pacing2` (main `f3d45cc7`)

This is a further partial accounting batch. It advances all five requested
work groups but does not finish their instruction coverage. New prices are
lagless-only. Default and ROM-timing ledger charges are unchanged. No source
lag count, scene duration, threshold, multiplier or correction table enters
the runtime. Evidence for this continuation is in `target/cost-pacing2/`;
the earlier `target/cost-pacing/` receipts remain intact.

### Additional instruction coverage

Listings are retained in `source-costs/`, generated by
`scripts/rom_block_costs.py` against this checkout's ROM. Entry M/X widths
matter: the room parent starts M8/X8 and sets REP #$30, the object drawers
use M16/X16, and the peg-copy helper starts M8/X16 before REP #$20.
Inline jump-table bytes following a JSL are data, not instructions.

| Group | Added source work | Boundaries / exclusions |
| --- | --- | --- |
| 1 | `$01:873A..88E3` room parent: header/floor/object call instructions, sixteen seven-word clear passes, two eleven-pass line-pointer copies, 99 movable-block comparisons and the actual torch-stream scan. Common subtype-1 wrapper/size work at `$01:8B89/8A92/8B79/8C37`, `$01:8D5D..8D7D`; 1x3 `$01:8D80..8D9D`, 5x4 `$01:9AEE..9B17`, 2x2 `$01:9895..98AD`, five-tile `$01:B279..B292`; block/torch setup `$01:B4D6..B539`. Stair parent `$02:9054..906B`, `$02:8AAF..8ACE`, and Link branches `$07:F2C1..F390`. | Type1 parsing/table dispatch was already priced at `$01:88E4/893C`; it was not added again. Drawing child bodies remain separate from call/parent prices. Many subtype bodies and stair helpers are still incomplete. |
| 2 | `PrepTransAuxGfx` self work `$00:DF1A..DF4E`; screen-build/ambient wrapper `$02:ED25..EDC4`; reload/scroll parents `$02:ABED..AC39`, `$09:C49D..C4AB`; sprite presence stream `$09:C4AC..C55D`, full proximity-scan control `$09:C55E..C5F5`, shared coordinate bounds `$09:C6F5..C730`; peg dispatch/copies `$02:972A..97B1`, four-bank attribute walk `$01:C22A..C27C`. | The peg walk charges the actual 8,192 attribute reads and their $66/$67/other branches. Presence records distinguish ordinary/trap/sentinel paths. Properties/overlay selection parents, initial-scroll parents and deeper sprite activation children remain incomplete. |
| 3 | Overworld player-control parent `$02:A53C..A5EB`: actual control/input, entrance, super-bomb and world branches, call instructions and returns. | A read-only lexical call-site audit flags parents without a direct charge that call priced children. The retained `annotation-gaps.json` is a candidate worklist, not a type-resolved call graph or completeness proof; aliases, price-helper wrappers and test-only methods require manual filtering. This identified the unpriced overworld player-control parent. Dungeon player-control was already priced; ordinary gameplay sprite gaps remain. The scratch audit lives in `/tmp/cost2-annotation-audit.py`. |
| 4 | Dungeon/flute/world-map dispatchers `$0A:E0B0..E0B6/E0DC..E0E1/B730..B736/B98B..B991` and the real `$00:8781/879C` jump-table helpers; border loops `$0A:E449..E4E8` and five-row caller `$0A:E579..E599`; two/three-choice self paths `$0E:CD1A..CD87/CE83..CEF0/CEF7..CF6D/CF72..CFE1`. | Choice reloads exclude decoder/VWF children and the existing 96-clock cursor resets. Scroll-copy work stays only in the VWF aggregate. The interface already charges the fresh sprite/Link/HUD/lamp call prefix; remaining child instruction gaps are not replaced with an aggregate estimate. Room-row bodies, map initialization, ChooseItem and many map children remain incomplete. |
| 5 | Retained ordinary and lag-handler costs; generic first/repeated field accounting below. | NMI update work is removed from repeated lag handlers without changing the existing ledger or publishing extra game state. The predictor CSV adds `cost_lag_handler`. |

### Intervening NMI accounting

The preceding ordinary handler's actual CPU and DMA cost `H` still owns
main's entry. While that handler runs, the native NMI measures the CPU and
DMA inside its update/joypad branch. A lag NMI's `$00:813A` taken branch
costs 310 clocks through that header instead of 434 for the update path.
Removing that 124-clock difference plus the actual update/joypad CPU and DMA
leaves lag-handler cost `L`. The register/RTI tail and any polygon interrupt
work remain. Both `H` and `L` retain the ordinary closing handoff; held
callbacks cannot overwrite them.

For P1/P2 let `E = H + W(H)` and `R = L + W(L)`, with the existing defensive
field caps, `F = 357368`, and charged main work `C` (P2 also adds the existing
40-clock refresh stalls per 1,324 work clocks). Set `B = E + C`:

- If `B < F`, no NMI has been crossed.
- Otherwise `N = 1 + floor((B - F)/(F - R))` lag NMIs are crossed.
- Wall time is `B + N*R`; the existing raster waits are then added.

This accepts equality and repeats the lag handler rather than the preceding
ordinary graphics DMA. P0 remains the prior diagnostic formula. No category
is suppressed. Two new tests check first/repeated-field equality boundaries
and compare the stripped ordinary-handler price with a real native latched
NMI, including retention across the closing handoff.

`handler-full-diagnostic.json` records the native-ledger handler costs on
source-lagged calls. Median ordinary / lag costs are 90,356 / 3,030 for
`08/02`, 33,922 / 3,201 for `0E/07`, and 36,358 / 3,086 for `0E/0A`.
The flute ordinary-handler 95th percentile reaches 113,696 clocks, versus
3,418 for its lag handler. Repeating that ordinary update/DMA explains the
excess direction; these observations do not supply runtime prices.

This is still an estimate: later audio/polygon transactions can differ from
the preceding handler's retained non-update work, HDMA/main instruction
coverage is incomplete, and completed native main state generally publishes
atomically. CPU census success is not exact video/audio acceptance.

### Full cartridge progression

Every row covers 1,473,123 iterations, 39,575 lagged calls and 102,677 source
lag frames. Counts below use production P2. Missing/excess are separate
nonnegative totals; exact/within-one use lagged calls only.

| Stage | Held frames | Exact lagged | Within one | Missing | Excess | Zero-lag FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Main, prior receipt | 95,663 | 30,512 | 38,375 | 9,557 | 2,543 | 0 |
| group1 | 96,344 | 30,635 | 38,380 | 9,156 | 2,823 | 0 |
| group2 | 97,929 | 30,815 | 38,843 | 7,831 | 3,083 | 0 |
| group3 | 97,929 | 30,815 | 38,843 | 7,831 | 3,083 | 0 |
| group4 | 97,929 | 30,815 | 38,843 | 7,831 | 3,083 | 0 |
| final-rom | 91,177 | 29,971 | 38,669 | 11,591 | 91 | 0 |

Group 3 adds 407,345,676 charged main clocks on 315,476 iterations but crosses
no additional field boundary on this route; its count row equals group 2.
`group3-work-delta.json` retains the work deltas.

All four intermediate cartridge gates and both final gates PASS with
unexplained zero and complete=True. The prior main row is the retained
previous-batch receipt, not a new cartridge census. The clean-main default
regression baseline below was recorded afresh.

### Final rule comparison

| Mode / rule | Held / predicted | Exact lagged | Within one | Missing | Excess | FP | Absolute error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Cartridge / cost_p0 | 86,183 | 26,371 | 38,073 | 16,545 | 51 | 0 | 16,596 |
| Cartridge / cost_p1 | 87,089 | 26,913 | 38,203 | 15,673 | 85 | 0 | 15,758 |
| Cartridge / cost_p2 | 91,177 | 29,971 | 38,669 | 11,591 | 91 | 0 | 11,682 |
| Live SPC / cost_p0 | 86,117 | 26,267 | 37,967 | 17,082 | 522 | 131 | 17,604 |
| Live SPC / cost_p1 | 87,021 | 26,801 | 38,082 | 16,235 | 579 | 131 | 16,814 |
| Live SPC / cost_p2 | 91,106 | 29,846 | 38,549 | 12,171 | 600 | 131 | 12,771 |

Masked elapsed callbacks are 2,842 cartridge / 2,821 live, separate from
accepted lag holds. Source-capped held totals are 91,086 / 90,506.

The repeated-handler correction removes update-DMA overcounts but exposes
more missing main work; a lower excess total does not imply better overall
count accuracy. Prices and the lag-handler distinction follow the executed
instructions rather than a count-error target. No category-specific hold
correction was introduced.

### Complete final category table

Paired cells are cartridge / live. Every source-lag category and every
zero-lag false-positive category is included. JSON receipts retain all
other zero-lag categories. Exact/within-one refer to lagged calls only.

| Entry | Source lag | Lagged calls | Held C / L | Exact C / L | Within one C / L | Missing C / L | Excess C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 4,418 | 12,055 / 11,966 | 3,509 / 3,402 | 4,418 / 4,308 | 909 / 1,226 | 0 / 228 | 0 / 98 |
| `07/02` | 8,770 | 1,976 | 7,739 / 7,739 | 1,168 / 1,168 | 1,767 / 1,767 | 1,031 / 1,031 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 142 | 7,681 / 7,681 | 0 / 0 | 0 / 0 | 515 / 515 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 4,952 | 7,671 / 7,671 | 4,710 / 4,710 | 4,889 / 4,889 | 308 / 308 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 157 | 6,018 / 5,867 | 0 / 0 | 13 / 13 | 448 / 599 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 1,179 | 5,288 / 5,288 | 615 / 615 | 1,104 / 1,104 | 766 / 766 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 4,879 | 5,020 / 5,020 | 4,467 / 4,467 | 4,879 / 4,879 | 412 / 412 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 251 | 4,076 / 4,076 | 0 / 0 | 206 / 206 | 296 / 296 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 2,919 | 3,219 / 3,219 | 2,100 / 2,100 | 2,897 / 2,897 | 841 / 841 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 2,960 | 1,883 / 1,977 | 1,164 / 1,164 | 2,960 / 2,960 | 1,796 / 1,796 | 0 / 94 | 0 / 16 |
| `09/01` | 3,479 | 251 | 3,426 / 3,426 | 198 / 198 | 251 / 251 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 2,718 | 2,793 / 2,793 | 2,519 / 2,519 | 2,718 / 2,718 | 199 / 199 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 115 | 2,592 / 2,592 | 35 / 35 | 115 / 115 | 80 / 80 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 2,669 | 2,669 / 2,671 | 2,666 / 2,664 | 2,669 / 2,668 | 3 / 4 | 0 / 3 | 0 / 0 |
| `08/02` | 2,576 | 157 | 2,522 / 2,462 | 149 / 149 | 150 / 150 | 54 / 114 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 1,735 | 2,151 / 2,151 | 1,566 / 1,566 | 1,722 / 1,722 | 182 / 182 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 1,855 | 1,996 / 1,996 | 1,850 / 1,850 | 1,855 / 1,855 | 5 / 5 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 90 | 1,582 / 1,582 | 18 / 18 | 36 / 36 | 263 / 263 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,192 | 84 / 84 | 23 / 23 | 1,192 / 1,192 | 1,169 / 1,169 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 56 | 897 / 897 | 0 / 0 | 28 / 28 | 168 / 168 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 603 | 941 / 941 | 563 / 563 | 600 / 600 | 43 / 43 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 17 | 929 / 929 | 0 / 0 | 0 / 0 | 44 / 44 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 928 | 922 / 922 | 922 / 922 | 928 / 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 157 | 691 / 714 | 150 / 150 | 150 / 150 | 15 / 24 | 0 / 32 | 0 / 0 |
| `05/00` | 639 | 11 | 504 / 504 | 0 / 0 | 0 / 0 | 135 / 135 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 251 | 251 / 251 | 0 / 0 | 245 / 245 | 257 / 257 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 391 | 391 / 391 | 310 / 310 | 391 / 391 | 81 / 81 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 287 | 383 / 383 | 221 / 221 | 287 / 287 | 66 / 66 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 239 | 324 / 324 | 177 / 177 | 234 / 234 | 67 / 67 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 100 | 261 / 261 | 8 / 8 | 95 / 95 | 102 / 102 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 101 | 303 / 303 | 101 / 101 | 101 / 101 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 16 | 254 / 254 | 0 / 0 | 12 / 12 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 233 | 81 / 81 | 49 / 49 | 233 / 233 | 184 / 184 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 223 | 84 / 84 | 60 / 60 | 223 / 223 | 163 / 163 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 81 | 239 / 239 | 77 / 77 | 81 / 81 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 16 | 240 / 240 | 14 / 14 | 16 / 16 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 4 | 151 / 151 | 0 / 0 | 0 / 0 | 84 / 84 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 36 | 207 / 207 | 23 / 23 | 33 / 33 | 21 / 21 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 157 | 157 / 208 | 157 / 154 | 157 / 155 | 0 / 1 | 0 / 52 | 0 / 0 |
| `17/01` | 141 | 3 | 105 / 105 | 0 / 0 | 0 / 0 | 36 / 36 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 9 | 130 / 130 | 7 / 7 | 7 / 7 | 9 / 9 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 28 | 120 / 120 | 10 / 10 | 28 / 28 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 119 | 2 / 2 | 1 / 1 | 101 / 101 | 136 / 136 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 11 | 47 / 47 | 1 / 1 | 8 / 8 | 67 / 67 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 93 | 16 / 16 | 6 / 6 | 93 / 93 | 87 / 87 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 28 | 84 / 84 | 28 / 28 | 28 / 28 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 16 | 77 / 77 | 10 / 10 | 16 / 16 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 79 | 83 / 83 | 79 / 79 | 79 / 79 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 4 | 78 / 78 | 2 / 2 | 3 / 3 | 1 / 1 | 7 / 7 | 0 / 0 |
| `07/13` | 71 | 47 | 70 / 70 | 46 / 46 | 47 / 47 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 65 | 9 / 9 | 6 / 6 | 65 / 65 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 4 | 65 / 65 | 1 / 1 | 4 / 4 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 66 | 16 / 16 | 16 / 16 | 66 / 66 | 50 / 50 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 | 60 / 60 | 0 / 0 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 12 | 57 / 57 | 7 / 7 | 11 / 11 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 62 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 4 | 60 / 60 | 3 / 3 | 4 / 4 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 4 | 58 / 58 | 2 / 2 | 4 / 4 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 3 | 59 / 59 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 51 | 14 / 14 | 6 / 6 | 51 / 51 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 2 | 57 / 57 | 1 / 1 | 1 / 1 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 3 | 56 / 56 | 2 / 2 | 2 / 2 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 3 | 58 / 58 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 3 | 58 / 58 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 3 | 58 / 58 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 3 | 56 / 56 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 3 | 55 / 55 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 3 | 54 / 54 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 | 0 / 0 | 0 / 0 | 53 / 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 2 | 53 / 53 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 17 | 46 / 46 | 12 / 12 | 17 / 17 | 5 / 5 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 51 | 51 / 51 | 51 / 51 | 51 / 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 2 | 50 / 50 | 1 / 1 | 2 / 2 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 40 | 18 / 18 | 10 / 10 | 40 / 40 | 30 / 30 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 1 | 44 / 44 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 1 | 43 / 43 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 | 42 / 42 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 14 | 42 / 42 | 13 / 0 | 14 / 7 | 0 / 41 | 1 / 42 | 0 / 14 |
| `1A/20` | 41 | 1 | 124 / 124 | 0 / 0 | 0 / 0 | 0 / 0 | 83 / 83 | 0 / 0 |
| `09/12` | 32 | 16 | 16 / 16 | 0 / 0 | 16 / 16 | 16 / 16 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 4 | 32 / 32 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 4 | 25 / 25 | 0 / 0 | 3 / 3 | 5 / 5 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 13 | 19 / 19 | 3 / 3 | 13 / 13 | 10 / 10 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 4 | 16 / 16 | 0 / 0 | 0 / 0 | 8 / 8 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 23 | 23 / 23 | 23 / 23 | 23 / 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 14 | 0 / 0 | 0 / 0 | 14 / 14 | 14 / 14 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 1 | 14 / 14 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 | 0 / 0 | 0 / 0 | 7 / 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 3 | 6 / 6 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 4 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 3 | 3 / 3 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 1 | 2 / 2 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 | 0 / 59 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 59 | 0 / 3 |

### Crystal / pendant phases and rupee gaps

Each phase cell is source / prior main runtime / this continuation in
presentation frames, including masked callbacks. These are fresh seeded,
uninterrupted windows; stable-outcome checks match room, Link XY and the
whole save block in all eight cases.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 67 / 65 | 5 / 6 / 5 | 17 / 22 / 17 | 2 / 2 / 2 | 35 / 34 / 34 | 16 / 16 / 16 |
| 380000 | 84 / 68 / 64 | 7 / 6 / 5 | 24 / 21 / 16 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 460000 | 84 / 67 / 65 | 7 / 6 / 5 | 24 / 22 / 17 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 626000 | 91 / 71 / 69 | 9 / 7 / 6 | 24 / 22 / 17 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 734000 | 84 / 68 / 65 | 7 / 6 / 5 | 25 / 23 / 17 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 880000 | 84 / 67 / 65 | 7 / 6 / 5 | 26 / 23 / 18 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1064000 | 84 / 67 / 64 | 7 / 5 / 5 | 25 / 23 / 17 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1316000 | 84 / 66 / 65 | 7 / 5 / 5 | 25 / 23 / 18 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |

| Mode | Classified WRAM differences | $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14737, "song-end-poll": 636}` | 10 |

The 162000–163000 probe has 0 $0CFD-only exceptions. The same 27
logical iterations request rupee ticks in the first burst. Their gaps are:

- Prior main: `[24, 8, 8, 8, 8, 10, 25, 16, 8, 8, 8, 8, 8, 14, 28, 12, 8, 8, 8, 8, 8, 14, 12, 8, 8, 8]`.
- Final: `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`.
- Source presentation intervals at those iterations: `[24, 8, 8, 8, 8, 12, 25, 14, 8, 8, 8, 8, 8, 19, 26, 8, 8, 8, 8, 8, 8, 14, 12, 8, 8, 8]`.

The final first band/envelope PCM onset interval is 23.002
frames. `final-probe-metrics.json` retains the PCM samples/onset gaps.
This is the same 7.9–8.7 kHz / threshold-600 onset diagnostic, not exact
PCM acceptance or exact intra-NMI source command timestamps.

### Remaining coverage at this stopping point

The largest final cartridge deficits are listed without netting excess
holds against them. These are entry categories, not exclusive causal
attributions to one missing child.

| Entry | Missing | Excess |
| --- | ---: | ---: |
| `07/00` | 1,796 | 0 |
| `09/00` | 1,169 | 0 |
| `07/02` | 1,031 | 0 |
| `0E/02` | 909 | 0 |
| `07/07` | 841 | 0 |
| `09/23` | 766 | 0 |
| `06/00` | 515 | 0 |
| `08/00` | 448 | 0 |
| `0F/01` | 412 | 0 |
| `07/0E` | 308 | 0 |
| `09/03` | 296 | 0 |
| `0E/0A` | 263 | 0 |
| `09/04` | 257 | 0 |
| `07/0F` | 199 | 0 |
| `09/16` | 184 | 0 |
| `07/15` | 182 | 0 |

Still unpriced or incomplete:

- Additional subtype-1/2/3 object bodies, room/header children, falling/warp
  parents, stair movement/landing/reposition children beyond the priced parts.
- Overworld properties/overlay selection parents and suffixes, initial-scroll
  setup/control, Sprite_ResetAll_noDisable bulk clears and deeper sprite
  activation children, mosaic/overlay/bomb-door coverage.
- Ordinary gameplay sprites/control helpers, particularly dungeon gameplay.
  The lexical candidate audit is not a substitute for source path validation.
- Dungeon map room-row bodies, initialization/floor-list loops and remaining
  map children; ChooseItem and remaining fresh sprite child work.
- Phase-specific lag-handler audio/polygon work and more exact refresh/HDMA
  placement. Total cost coverage does not establish partial display timing.

### Continuation validation

| Check | Result |
| --- | --- |
| Final ordinary default / reduced builds | PASS; both warning-free |
| Fresh main default baseline vs final default | PASS; 200,000 records and EOF |
| Four intermediate full cartridge gates | PASS; 1,473,123 iterations each |
| Final full cartridge and live-SPC gates | PASS; unexplained zero, complete=True |
| Zelda lib, default | 2461 pass; 1 failure; 3 ignored; all current cases observed |
| Zelda lib, reduced | 1996 pass; 0 failure; 2 ignored; all current cases observed |
| Pacing regressions, both configurations | 7 pass each, including both new handler tests |
| SNES lib, both configurations | 504 pass, 0 failures, 7 ignored each |
| Eight stable crystal/pendant outcomes | PASS |
| `git diff --check` | PASS |
| Complete instruction coverage / exact A/V | Not established |

The sole allowed default failure remains
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
Default lib-test compilation retains the existing unused-fixture warnings;
ordinary builds are warning-free. These are complete final-source suites,
not a union of pre-change and filtered test runs.

`verification.json`, `gates.json`, full CSV/JSON category metrics, phase and
stable-outcome receipts, `candidate-source-sha256.json` and
`artifact-hashes.json` bind the retained CPU evidence. Source ledgers and
`.git/parity-oracle-cache` were read-only. No GPU, `--video`, cached-A/V,
Snes9x execution, cargo GPU tests, formatting, commit or push was used.

Reproduce the final cartridge gate with the earlier command using
`BIN=target/cost-pacing2/final-reduced-zelda3` and fresh output/save paths;
append `--live-apu-reads` for the live-SPC gate. The final predictor adds
`cost_lag_handler` beside `cost_entry_handler`; existing columns are retained.

## Continuation on `codex/cost-pacing3` (main `05cde80f`)

This verified partial batch examines the five requested groups in order and
adds the source work below. It does not complete their instruction coverage.
Evidence is retained in `target/cost-pacing3/`; earlier receipts are preserved.
Every new ledger charge is lagless-only, with its source address in a comment.
The default/ROM-timing ledger paths and source-lag classification rules are
unchanged. Prices come from `scripts/rom_block_costs.py` and the pinned ROM,
not count errors. Listings are retained in `source-costs/`.
`source-price-check.json` independently re-sums 63 instruction blocks with
the correct entry widths and confirms the annotation arithmetic.

### Source work and accounting boundaries

| Group | Added work | Boundaries / remaining gaps |
| --- | --- | --- |
| 1: ordinary gameplay | Room-clear sprite scan `$09:AF61..AF74`, stopping at the actual first blocking sprite; ripple parent `$1E:FF8D..FFDD` and coordinate wrapper `$06:84BD..84C0`; rat control suffix `$05:A8CB..A95A`, with actual state/delay/wall/world branches. | The shared overlord tail, ripple drawing, coordinates, collision, movement, recoil, RNG and sound children already have their own prices. Rat drawing and its first animation instructions were already priced. Several lexical candidates were wrappers or native continuations, not missing bodies. Most ordinary sprite/control coverage remains incomplete. |
| 2: dungeon loads | Shelf end `$01:B2E1..B2F5`, bar segment `$01:B2F6..B305`, single-pot parent `$01:B395..B3DE`, 8x8 parent `$01:A7DC..A7EF`, 3x2 drawer `$01:9D04..9D28`, water-holding rows `$01:9D6F..9D95`. | M16/X16 prices; water rows use the actual height, 3x2 uses two real row passes. The pot excludes the already-priced 2x2 tail; 8x8 excludes four already-priced 4x4 children. Remaining object bodies, header/room, falling/warp and stair children are not claimed complete. |
| 3: text | ChooseItem parent `$0E:CD88..CDC7` and selected-item pointer/variant/eight-byte-copy body `$0E:CE14..CE6A`. | M8/X8; countdown/input branches are selected before mutation. The countdown tail branch inherits its child's return. The VWF aggregate already owns command dispatch/glyph work; refresh already owns its six-row copy. No duplicate scroll or refresh price was added. Next/previous inventory-scan bodies and more fresh-sprite children remain unpriced. |
| 4: overworld | Sprite_ResetAll call `$09:C44E`, fixed prefix `$09:C452..C479`, 4,096 room-mask clears `$09:C47B..C480`, 512 loaded-mask clears and eight history stores `$09:C482..C498`; worker ownership correction described below. | Each clear is priced from its actual store/decrement/branch loop, once in its corresponding native helper. Properties/overlay selection parents, initial-scroll setup, deeper activation and mosaic/bomb-door work remain incomplete. |
| 5: iris | Closing-module parent `$02:9982..99C6` and shared table-control parent `$02:9A19..9A35`, including actual indoor/area/direction/goal branches and child call instructions. | Sprite_Main, Link movement/animation/OAM and the table builder retain their existing child prices. `$02:9334..9356` landing-iris dispatch/suffix was confirmed already priced, so no duplicate `07/0F` parent charge was added. OpenSpotlight_Next2 and deeper residual child work remain incomplete. |

### Crystal versus pendant: enabled worker ownership

The retained probe states show the pendant properties/overlay/build calls
with `$012A=0`, while the seven crystal sequences retain `$012A=1` on entry.
This is a source CPU-ownership difference, not a different duration assigned
to a crystal, area, overlay or sprite set. The properties main-work totals
are also comparable: roughly 13.1–14.6 million clocks before these additions.
The old pacing predicate excluded module 08 because the native render-worker
helper recognized only its gameplay/dialogue/triforce callers.

The ROM's `$00:8144..8149` instead tests the enable byte independently of
module routing. `NMI_SwitchThread` `$00:822D` programs `$4209/$420A` and
`$4200=$A1`; `Interrupt_IRQ` `$00:82D8` returns ownership at that programmed
V-IRQ. Even with GO clear, `Polyhedral_RunThread` `$09:F81D..F81F` spins
until that interrupt. The lagless pacing worker share now follows the native
enable byte in every module, using the existing hardware deadline and IRQ
price. Worker instructions remain excluded from main's work counter.
No module, scene or lag-count table supplies this ownership interval.

A regression checks identical enabled-worker ownership across every
main-module value, and zero ownership with the byte clear or lagless disabled.
The render-worker execution predicate and default/ROM-timing scheduling are
unchanged. This remains aggregate CPU pacing, not partial-display or exact
A/V acceptance.

The first global-enable trial produced 48 new cartridge false positives,
all in `00/07`, plus 20 more excess intro holds. That trial and its complete
receipts remain retained. Auditing this trial also identified a separate generic entry/lag
ownership error: the final enable byte was being applied retrospectively to
the preceding NMI as well as to intervening lag NMIs.

`native_begin_main_cost` now retains the preceding handler's worker share
before main can activate or deactivate `$012A`. Only subsequent lag NMIs
use the live byte. A call shorter than a field cannot retroactively create
a preceding worker slice by enabling the thread; a clear cannot erase an
already-consumed slice. A second regression covers both transitions and
checks that a newly enabled worker still owns an intervening NMI. This is
hardware event order, with no module exclusion or price adjustment. The
entry-worker correction has fresh builds, both full final censuses, probes,
a 200k default comparison and complete library suites of its own.

The entry-share correction restores crystal iris opening to 34 source
frames in all seven probes. It does not eliminate the 48 `00/07` intro
false positives; their main/handler/worker estimate remains incomplete.
They remain in the final table and are not suppressed by a module rule.

For P1/P2, the earlier repeated-handler formula now uses
`E = H + retained_entry_worker` and `R = L + live_lag_worker`.
Field size, refresh pricing and handler instruction prices are unchanged.
P0 retains its diagnostic formula with the entry worker share.

`p2-runtime-agreement.json` confirms runtime accepted holds equal P2 on
every one of the 1,473,123 iterations in each final census, including all
zero-lag categories. No count category is excluded from production pacing.

A concrete next ordinary-gameplay audit is the raw
`Sprite_ReturnIfInactive` / `Sprite_ReturnIfPaused` predicate
(`$06:D9EC..DA08`): the plain native predicate is uncharged, while the bank-5
and bank-1A entry helpers price their own copies and call that same predicate.
The unpriced Octorok parent is one direct caller; adding a blanket charge to
the shared predicate would double-charge the already-priced wrappers.
`source-costs/next-inactive-helper.txt` retains its instruction listing.
This is a source-audit continuation point, not an added price in this batch.

### Full cartridge progression

Each full pass covers 1,473,123 iterations, 39,575 lagged calls and 102,677
source lag frames. Missing and excess are separate nonnegative totals;
exact/within-one use lagged calls only. The main row is the retained prior
batch receipt; the 200k default regression baseline was recorded afresh.

| Stage | Held | Exact lagged | Within one | Missing | Excess | Zero-lag FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Main, prior receipt | 91,177 | 29,971 | 38,669 | 11,591 | 91 | 0 |
| group1 | 91,179 | 29,973 | 38,669 | 11,589 | 91 | 0 |
| group2 | 91,189 | 29,973 | 38,678 | 11,579 | 91 | 0 |
| group3 | 91,189 | 29,973 | 38,678 | 11,579 | 91 | 0 |
| group4 | 92,508 | 30,484 | 38,867 | 10,328 | 159 | 48 |
| group5 | 92,511 | 30,487 | 38,867 | 10,325 | 159 | 48 |
| Entry worker handoff | 92,503 | 30,489 | 38,867 | 10,320 | 146 | 48 |

Group 3 adds 2,160,228 main clocks on 1,494 iterations without crossing
another field boundary; `group3-work-delta.json` retains the deltas.

All five intermediate cartridge gates and both final full gates PASS,
unexplained zero and complete=True. The final entry-worker handoff has its own complete cartridge and live
classification traces; the earlier final-rom run binds group 5 before that
correction. Source-state equality is separate from lag-count
accuracy and exact A/V acceptance.

### Final rules

| Mode / rule | Predicted | Exact lagged | Within one | Missing | Excess | FP | Absolute error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Cartridge / cost_p0 | 87,564 | 26,862 | 38,124 | 15,327 | 214 | 48 | 15,541 |
| Cartridge / cost_p1 | 88,386 | 27,403 | 38,258 | 14,432 | 141 | 48 | 14,573 |
| Cartridge / cost_p2 | 92,503 | 30,489 | 38,867 | 10,320 | 146 | 48 | 10,466 |
| Live SPC / cost_p0 | 87,483 | 26,759 | 38,021 | 15,913 | 719 | 179 | 16,632 |
| Live SPC / cost_p1 | 88,282 | 27,292 | 38,130 | 15,099 | 704 | 179 | 15,803 |
| Live SPC / cost_p2 | 92,397 | 30,362 | 38,735 | 11,010 | 730 | 179 | 11,740 |

Masked elapsed callbacks (separate from accepted holds): 2,838 / 2,809 cartridge / live. Source-capped holds: 92,357 / 91,667.

### Complete final category table

Paired cells are cartridge / live. Every source-lag category and every
zero-lag false-positive category is included. The JSON receipts retain all
other zero-lag categories. Exact/within-one use lagged calls only.

| Entry | Source lag | Lagged calls | Held C / L | Exact C / L | Within one C / L | Missing C / L | Excess C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 4,418 | 12,055 / 11,960 | 3,509 / 3,402 | 4,418 / 4,308 | 909 / 1,227 | 0 / 223 | 0 / 96 |
| `07/02` | 8,770 | 1,976 | 7,747 / 7,747 | 1,168 / 1,168 | 1,775 / 1,775 | 1,023 / 1,023 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 142 | 7,834 / 7,834 | 0 / 0 | 0 / 0 | 362 / 362 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 4,952 | 7,672 / 7,672 | 4,710 / 4,710 | 4,890 / 4,890 | 307 / 307 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 157 | 6,314 / 6,084 | 19 / 19 | 143 / 139 | 152 / 382 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 1,179 | 5,445 / 5,445 | 615 / 615 | 1,134 / 1,134 | 609 / 609 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 4,879 | 5,164 / 5,164 | 4,611 / 4,611 | 4,879 / 4,879 | 268 / 268 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 251 | 4,076 / 4,076 | 0 / 0 | 206 / 206 | 296 / 296 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 2,919 | 3,219 / 3,219 | 2,100 / 2,100 | 2,897 / 2,897 | 841 / 841 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 2,960 | 1,888 / 1,988 | 1,169 / 1,169 | 2,960 / 2,960 | 1,791 / 1,791 | 0 / 100 | 0 / 18 |
| `09/01` | 3,479 | 251 | 3,426 / 3,426 | 198 / 198 | 251 / 251 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 2,718 | 2,793 / 2,793 | 2,519 / 2,519 | 2,718 / 2,718 | 199 / 199 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 115 | 2,592 / 2,592 | 35 / 35 | 115 / 115 | 80 / 80 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 2,669 | 2,669 / 2,673 | 2,666 / 2,664 | 2,669 / 2,668 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 157 | 2,574 / 2,486 | 155 / 152 | 157 / 153 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 1,735 | 2,151 / 2,151 | 1,566 / 1,566 | 1,722 / 1,722 | 182 / 182 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 1,855 | 1,996 / 1,996 | 1,850 / 1,850 | 1,855 / 1,855 | 5 / 5 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 90 | 1,637 / 1,637 | 18 / 18 | 36 / 36 | 208 / 208 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,192 | 84 / 84 | 23 / 23 | 1,192 / 1,192 | 1,169 / 1,169 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 56 | 897 / 897 | 0 / 0 | 28 / 28 | 168 / 168 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 603 | 941 / 941 | 563 / 563 | 600 / 600 | 43 / 43 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 17 | 930 / 930 | 0 / 0 | 0 / 0 | 43 / 43 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 928 | 922 / 922 | 922 / 922 | 928 / 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 157 | 706 / 741 | 157 / 153 | 157 / 153 | 0 / 17 | 0 / 52 | 0 / 0 |
| `05/00` | 639 | 11 | 511 / 511 | 0 / 0 | 0 / 0 | 128 / 128 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 251 | 502 / 502 | 245 / 245 | 251 / 251 | 6 / 6 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 391 | 391 / 391 | 310 / 310 | 391 / 391 | 81 / 81 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 287 | 384 / 384 | 222 / 222 | 287 / 287 | 65 / 65 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 239 | 324 / 324 | 177 / 177 | 234 / 234 | 67 / 67 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 100 | 271 / 271 | 8 / 8 | 100 / 100 | 92 / 92 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 101 | 303 / 303 | 101 / 101 | 101 / 101 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 16 | 254 / 254 | 0 / 0 | 12 / 12 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 233 | 81 / 81 | 49 / 49 | 233 / 233 | 184 / 184 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 223 | 84 / 84 | 60 / 60 | 223 / 223 | 163 / 163 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 81 | 239 / 239 | 77 / 77 | 81 / 81 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 16 | 240 / 240 | 14 / 14 | 16 / 16 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 4 | 153 / 153 | 0 / 0 | 0 / 0 | 82 / 82 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 36 | 213 / 213 | 23 / 23 | 34 / 34 | 15 / 15 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 157 | 157 / 236 | 157 / 154 | 157 / 155 | 0 / 1 | 0 / 80 | 0 / 0 |
| `17/01` | 141 | 3 | 108 / 108 | 0 / 0 | 0 / 0 | 33 / 33 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 9 | 132 / 132 | 7 / 7 | 7 / 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 28 | 120 / 120 | 10 / 10 | 28 / 28 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 119 | 2 / 2 | 1 / 1 | 101 / 101 | 136 / 136 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 11 | 47 / 47 | 1 / 1 | 8 / 8 | 67 / 67 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 93 | 16 / 16 | 6 / 6 | 93 / 93 | 87 / 87 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 28 | 84 / 84 | 28 / 28 | 28 / 28 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 16 | 77 / 77 | 10 / 10 | 16 / 16 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 79 | 83 / 83 | 79 / 79 | 79 / 79 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 4 | 76 / 76 | 2 / 2 | 3 / 3 | 1 / 1 | 5 / 5 | 0 / 0 |
| `07/13` | 71 | 47 | 70 / 70 | 46 / 46 | 47 / 47 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 65 | 9 / 9 | 6 / 6 | 65 / 65 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 4 | 65 / 65 | 1 / 1 | 4 / 4 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 66 | 16 / 16 | 16 / 16 | 66 / 66 | 50 / 50 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 | 60 / 60 | 0 / 0 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 12 | 59 / 59 | 7 / 7 | 12 / 12 | 5 / 5 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 62 | 62 / 62 | 62 / 62 | 62 / 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 4 | 60 / 60 | 3 / 3 | 4 / 4 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 4 | 58 / 58 | 2 / 2 | 4 / 4 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 3 | 59 / 59 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 51 | 14 / 14 | 6 / 6 | 51 / 51 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 2 | 59 / 59 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 3 | 56 / 56 | 2 / 2 | 2 / 2 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 3 | 58 / 58 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 3 | 58 / 58 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 3 | 58 / 58 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 3 | 56 / 56 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 3 | 55 / 55 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 3 | 54 / 54 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 | 0 / 0 | 0 / 0 | 53 / 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 2 | 53 / 53 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 17 | 46 / 46 | 12 / 12 | 17 / 17 | 5 / 5 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 51 | 51 / 51 | 51 / 51 | 51 / 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 2 | 50 / 50 | 1 / 1 | 2 / 2 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 40 | 18 / 18 | 10 / 10 | 40 / 40 | 30 / 30 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 1 | 44 / 44 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 1 | 43 / 43 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 | 42 / 42 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 14 | 35 / 35 | 8 / 0 | 14 / 7 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 1 | 122 / 122 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 0 / 0 |
| `09/12` | 32 | 16 | 32 / 32 | 16 / 16 | 16 / 16 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 4 | 32 / 32 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 4 | 29 / 29 | 3 / 3 | 4 / 4 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 13 | 19 / 19 | 3 / 3 | 13 / 13 | 10 / 10 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 4 | 20 / 20 | 0 / 0 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 23 | 23 / 23 | 23 / 23 | 23 / 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 14 | 62 / 62 | 14 / 14 | 14 / 14 | 0 / 0 | 48 / 48 | 48 / 48 |
| `15/05` | 14 | 1 | 14 / 14 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 | 0 / 0 | 0 / 0 | 7 / 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 3 | 15 / 15 | 0 / 0 | 0 / 0 | 0 / 0 | 9 / 9 | 0 / 0 |
| `12/09` | 4 | 4 | 4 / 4 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 3 | 3 / 3 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 1 | 5 / 5 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 |
| `00/06` | 2 | 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 | 0 / 89 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 89 | 0 / 3 |

### Crystal / pendant phases, exceptions and rupee gaps

Cells are source / prior batch / this batch, in presentation frames including
masked callbacks. Fresh uninterrupted seeded windows retain stable outcome
matches for room, Link XY and the whole save block in all eight cases.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 / 66 | 5 / 5 / 5 | 17 / 17 / 17 | 2 / 2 / 2 | 35 / 34 / 34 | 16 / 16 / 16 |
| 380000 | 84 / 64 / 85 | 7 / 5 / 7 | 24 / 16 / 23 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 460000 | 84 / 65 / 84 | 7 / 5 / 7 | 24 / 17 / 24 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 626000 | 91 / 69 / 89 | 9 / 6 / 9 | 24 / 17 / 24 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 734000 | 84 / 65 / 85 | 7 / 5 / 7 | 25 / 17 / 25 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 880000 | 84 / 65 / 84 | 7 / 5 / 7 | 26 / 18 / 26 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1064000 | 84 / 64 / 84 | 7 / 5 / 7 | 25 / 17 / 25 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1316000 | 84 / 65 / 83 | 7 / 5 / 7 | 25 / 18 / 25 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |

Crystal properties are within two (all but one within one), all crystal
overlays exact, and builds within one. Crystal iris opening is exact in
all seven cases. The pendant properties phase is now one frame longer than source; the
source-priced bulk reset crosses a boundary the prior partial counter did
not cross. Any residual phase differences remain explicit; no price was adjusted to
restore an exact phase count.

| Mode | Classified WRAM differences | $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14734, "song-end-poll": 638}` | 10 |

The 162000–163000 probe has 0 $0CFD-only exceptions.
The same 27 logical iterations request rupee ticks. Their presentation gaps:

- Prior batch: `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`.
- This batch: `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`.
- Source: `[24, 8, 8, 8, 8, 12, 25, 14, 8, 8, 8, 8, 8, 19, 26, 8, 8, 8, 8, 8, 8, 14, 12, 8, 8, 8]`.

The first PCM band/envelope onset interval remains 23.002
frames. The retained PCM diagnostics and all onset gaps use the previous
7.9–8.7 kHz / threshold-600 detector; this is not exact PCM acceptance or
an intra-NMI timestamp proof.

### Remaining coverage at this stopping point

These are entry-category deficits, not exclusive causal assignments.

| Entry | Missing | Excess |
| --- | ---: | ---: |
| `07/00` | 1,791 | 0 |
| `09/00` | 1,169 | 0 |
| `07/02` | 1,023 | 0 |
| `0E/02` | 909 | 0 |
| `07/07` | 841 | 0 |
| `09/23` | 609 | 0 |
| `06/00` | 362 | 0 |
| `07/0E` | 307 | 0 |
| `09/03` | 296 | 0 |
| `0F/01` | 268 | 0 |
| `0E/0A` | 208 | 0 |
| `07/0F` | 199 | 0 |
| `09/16` | 184 | 0 |
| `07/15` | 182 | 0 |
| `0E/07` | 168 | 0 |
| `09/0D` | 163 | 0 |

The unpriced children listed in the source-work table remain open. Ordinary
gameplay still has many sprite/control gaps; the object/header/stair, map
and fresh-text-child coverage is incomplete. More exact lag-handler audio,
polygon and HDMA/refresh placement also remains. No new default or ROM-timing
ledger price, scene correction, count-fitted multiplier or duration table was
introduced.

### Validation

| Check | Result |
| --- | --- |
| Final default / reduced ordinary builds | PASS; both warning-free |
| Fresh main default baseline versus final default | PASS; 200,000 records and EOF match |
| Five intermediate full cartridge gates | PASS; 1,473,123 iterations each |
| Final full cartridge / live-SPC gates | PASS; unexplained zero, complete=True |
| Zelda lib, default | 2463 pass; 1 failure; 3 ignored; all listed cases observed |
| Zelda lib, reduced | 1998 pass; 0 failure; 2 ignored; all listed cases observed |
| Pacing regressions, both configurations | 9 pass each, including worker routing and entry/lag ownership regressions |
| SNES lib, both configurations | 504 pass; 0 failures; 7 ignored each |
| Eight stable crystal/pendant outcomes | PASS |
| `git diff --check` | PASS |
| Complete instruction coverage / exact A/V | Not established |

The sole default lib failure is the known
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
Default lib-test builds retain their existing unused-fixture warnings;
ordinary builds are warning-free. Results are complete final-source suites. Superseded pre-correction
library runs were interrupted and are not counted.

`verification.json`, `gates.json`, stage/final CSV and JSON metrics, phase
and stable-outcome receipts, `candidate-source-sha256.json`, and
`artifact-hashes.json` bind this CPU evidence. The initial probe launch
omitted its required seed-file argument and aborted before running any
iterations; only the corrected complete probe supplies the phase results.
Source ledgers and `.git/parity-oracle-cache` were read-only. No GPU,
`--video`, cached-A/V, Snes9x execution, cargo GPU tests, `cargo fmt`,
commit or push was used. Scratch scripts remain in `/tmp`.

Reproduce with the earlier full-census command using
`BIN=target/cost-pacing3/final-reduced-zelda3` and fresh save/output paths;
append `--live-apu-reads` for the live-SPC gate.

## Continuation on `codex/cost-pacing4` (main `b6cd47fb`)

This is a verified partial continuation. Evidence is retained in
`target/cost-pacing4/`; earlier batches remain intact. New instruction charges
are lagless-only, carry source addresses, and come from
`scripts/rom_block_costs.py` against the pinned ROM. No module exclusion,
scene threshold, multiplier or duration table was added. Default and ROM-timing
ledger prices and the logical comparison classifications are unchanged.
`source-price-check.json` independently re-sums 49 instruction blocks and
confirms the instruction-price sequence of the bank-1D inactive copy.

### Worker ownership: the first interrupt matters

`$012A` enables stack exchange, not a permanently designated NMI-to-IRQ
worker interval. Both NMI `$00:82C9..82CF` and IRQ `$00:8324..832A` exchange
the current stack with `$1F0A`. An already-programmed V-IRQ can therefore
switch **main to worker first**, and the next NMI switches back to main.
The GO spin at `$09:F81D..F81F` still owns the CPU in its assigned interval.

The retained CPU-only, in-repository instruction-model trace
`native-intro-trace.log` shows this directly: at host 890 main starts at
V231, completes at V141, and V144 IRQ switches the waiting main to the
worker; host 891 NMI interrupts `$09:FE65` and returns to main. The palette
and scene-OAM bodies are actually executed. The trace prices iteration 660
at 227,338 main clocks; the prior native counter has 206,912. This missing
intro-parent work remains open. Treating main as unavailable until V144
was incorrect, rather than evidence that these bodies should be discounted.
This trace runs the repository's native CPU models with capture disabled;
it does not load or execute the external Snes9x/libretro core.

The pacing rule now retains IRQ programming and the stack orientation,
and divides main work at actual enable/clear/IRQ-line changes. A late
activation cannot own NMIs that preceded its store. In the reversed
orientation, main can work until IRQ; IRQ-to-NMI wall time then includes
its IRQ handler, and only the terminating NMI handler is added afterward.
The final audit removes an extra IRQ-handler charge from that interval.
The normal NMI-to-IRQ interval retains its existing IRQ-handler price.
P0 remains a diagnostic; P1/P2 advance the segmented ownership windows.
Refresh is still `40*ceil(main/1324)`, with cumulative rounding across
segments. Existing raster barriers and entry/lag-handler prices are retained.

The ordinary NMI only programs V-IRQ when `$0128` requests it; zero does
not disable an earlier timer. Conversely, song-upload callers explicitly
restore `$4200=$81`, disabling V-IRQ (`$02:8564..8566`, `$02:9C06..9C08`,
`$02:A046..A048`, `$09:F66F..F671`). The tracker now retains that transition.
A foreign post-NMI seed uses the saved stack to identify the returned
context; an inactive stale polygon stack does not prove IRQ is enabled.
The ordinary programmed line is 128, while the active-thread path uses `$FF`.
No module number participates in these decisions.

An initial trial omitted the upload reset and inferred IRQ enable from a
stale stack. Its complete phase probes caught greatly extended crystal
loads. Those interrupted census prefixes and the failed phase trial remain
explicitly superseded diagnostics. The corrected probe restores every
crystal phase before the final full validation. Twelve pacing regressions
cover the existing rules plus late activation, IRQ-first ownership, and
stale-stack/upload initialization. Worker execution remains ledger-muted.

### Additional source work, in requested order

| Group | Added work / audit | Remaining boundary |
| --- | --- | --- |
| 1 | Hardware stack ownership and enable-store order above. | Intro animation/control parents remain partially unpriced; real intro lag can consequently remain missing. No scene is suppressed. |
| 2 | Raw inactive/paused entries `$06:D9EC..DA08`, including actual short-circuit and double-return paths; Medusa parent `$1D:C7EB..C852`, including world, subtype, floor and spawn-result branches. | Already-priced bank-5/1A entries call the bare predicate and keep their old prices. The lexical audit found 257 raw-call parents without a direct self charge and two priced wrappers; it is not a typed call graph. Most sprite/control bodies remain open. |
| 3 | Prison-cell drawer `$01:9C44..9CC5`, eight-row big decor `$01:9DA2..9DD8` with its executed width, and twelve-by-twelve lamp cone `$01:9E06..9E2F`; all M16/X16 with actual plane/loop branches. | Additional object/header/stair children in the requested transition/load categories remain unpriced. |
| 4 | Previous/next inventory scans `$0E:CDC8..CDEB` and `$0E:CDED..CE12`, M8/X8: each visited slot, wrap, reserved slot, inventory/next-byte tests and rejected-slot increment/decrement. | The already-priced `$0E:CE14` child owns the successful tail return. Other fresh-text sprite/control children beyond group 2's shared predicate remain incomplete. |
| 5 | Live-SPC excess work audit, ordinary seed IRQ-line selection and the reversed interval's IRQ-handler ownership check. | No song-end answer, song duration or executed-work price was changed to remove excess holds. |

The inventory sign audit covers 58,097 source `0E/02` entry images and
finds negative bytes in aliased slots `$F359` and `$F360`. Presence in an
entry image does not prove the scan visits that slot. For a negative byte,
the new accounting stops at the shared sign-test prefix: the existing
semantic scan selects nonzero values where the ROM can reject them. No new
rejected-slot increment/decrement is charged when native did not execute it.
That selection mismatch remains open; default selection behavior is unchanged.

`ordinary-sprite-frequency.json` measures state-9 slot visits at ordinary
source gameplay entry, not inclusive instruction time. Dungeon counts include
spike block 212,895, crystal switch 208,701 and Medusa 180,926; outdoor mirror
portal has 161,315 visits and was confirmed already priced. This identifies
frequent work without assuming every active slot executes every child.
The spike-block parent remains a high-frequency continuation candidate.

The addition-only stage deltas retain 1,947,917,706 extra main clocks on
1,225,029 iterations for group 2, 5,425,978 on 120 iterations for group 3,
and 249,306 on 1,354 iterations for the group-4 trial. Group 4 crosses no
additional field boundary. None of these stages has a negative work delta.
`stage-work-deltas.json` records the measurements; they did not supply prices.

### Live-SPC excess: changed work versus changed pricing

The prior full paired CSV audit covers all 119 excess calls in the four
requested categories (96 text, 18 dungeon gameplay, two iris, three mirror).
Every one has different main/VWF work between cartridge-answer and live-SPC
runs. For example, iteration 463374 has 1,360 versus 584,212 VWF clocks;
iteration 383634 has 201,950 versus 13,670,378 total main clocks. These are
additional text/load paths, not equal-work calls assigned different holds
solely by a threshold. `prior-live-excess-work.json` retains every call.

The source song-end branch `$08:C606..C610` either continues drawing the
item or selects the rising-crystal continuation. The pendant poll likewise
holds its item timer or opens its message. Live answers can move message
rendering and the following screen load to different logical iterations.
The final paired audit below measures the remaining excess after the generic
IRQ corrections. Those executed children keep their instruction prices;
removing their holds would erase real native work. This does not establish
exact song timing, intra-NMI SPC sampling or exact audio/video parity.

The final library suites use six default and four reduced isolated libtest
processes, with one test thread each. `library-shards.json` binds their
executables, complete disjoint case lists and individual process exits;
the union is the entire listed suite, including the normally ignored cases.
The interrupted duplicate serial run is not counted as library validation.

### Full cartridge progression

Each complete pass contains 1,473,123 iterations, 39,575 lagged calls and
102,677 source lag frames. Missing/excess are nonnegative separate totals;
exact/within-one count lagged calls only. Immutable CPU binaries allow the
full census passes to finish independently; none shares writable save paths.

| Stage | Held | Exact lagged | Within one | Missing | Excess | Zero-lag FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Prior batch | 92,503 | 30,489 | 38,867 | 10,320 | 146 | 48 |
| Group 1 | 92,338 | 30,480 | 38,872 | 10,339 | 0 | 0 |
| Group 2 | 92,516 | 30,656 | 38,874 | 10,161 | 0 | 0 |
| Group 3 | 92,533 | 30,656 | 38,883 | 10,144 | 0 | 0 |
| Group 4 | 92,533 | 30,656 | 38,883 | 10,144 | 0 | 0 |
| Group 5 / final | 92,533 | 30,656 | 38,883 | 10,144 | 0 | 0 |

The four intermediate groups retain their intermediate IRQ model; group 5 includes
the final IRQ interval/seed correction and conservative signed-scan prefix.
All four intermediate cartridge gates and both final full gates PASS,
unexplained zero and complete=True. Superseded interrupted prefixes are
not counted as gates.

### Final rule comparison

| Mode / rule | Held | Exact lagged | Within one | Missing | Excess | FP | Absolute error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Cartridge / cost_p0 | 87,561 | 27,023 | 38,132 | 15,188 | 72 | 0 | 15,260 |
| Cartridge / cost_p1 | 88,426 | 27,579 | 38,267 | 14,251 | 0 | 0 | 14,251 |
| Cartridge / cost_p2 | 92,533 | 30,656 | 38,883 | 10,144 | 0 | 0 | 10,144 |
| Live SPC / cost_p0 | 87,480 | 26,920 | 38,029 | 15,773 | 576 | 131 | 16,349 |
| Live SPC / cost_p1 | 88,322 | 27,468 | 38,139 | 14,917 | 562 | 131 | 15,479 |
| Live SPC / cost_p2 | 92,427 | 30,529 | 38,751 | 10,833 | 583 | 131 | 11,416 |

Masked callbacks (separate from accepted holds): 2,835 / 2,811, cartridge / live.
Runtime holds equal P2 on every iteration in both final passes.

### Complete final category table

Paired cells are cartridge / live. Every source-lag and zero-lag
false-positive category is included; JSON retains all other categories.
Exact/within-one use lagged calls only.

| Entry | Source lag | Lagged calls | Held C / L | Exact C / L | Within one C / L | Missing C / L | Excess C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 4,418 | 12,070 / 11,977 | 3,524 / 3,417 | 4,418 / 4,308 | 894 / 1,211 | 0 / 224 | 0 / 97 |
| `07/02` | 8,770 | 1,976 | 7,802 / 7,802 | 1,217 / 1,217 | 1,780 / 1,780 | 968 / 968 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 142 | 7,839 / 7,839 | 0 / 0 | 0 / 0 | 357 / 357 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 4,952 | 7,677 / 7,677 | 4,711 / 4,711 | 4,894 / 4,894 | 302 / 302 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 157 | 6,314 / 6,084 | 19 / 19 | 143 / 139 | 152 / 382 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 1,179 | 5,448 / 5,448 | 618 / 618 | 1,134 / 1,134 | 606 / 606 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 4,879 | 5,176 / 5,176 | 4,623 / 4,623 | 4,879 / 4,879 | 256 / 256 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 251 | 4,076 / 4,076 | 0 / 0 | 206 / 206 | 296 / 296 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 2,919 | 3,276 / 3,276 | 2,157 / 2,157 | 2,897 / 2,897 | 784 / 784 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 2,960 | 1,894 / 1,992 | 1,175 / 1,175 | 2,960 / 2,960 | 1,785 / 1,785 | 0 / 98 | 0 / 17 |
| `09/01` | 3,479 | 251 | 3,426 / 3,426 | 198 / 198 | 251 / 251 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 2,718 | 2,800 / 2,800 | 2,526 / 2,526 | 2,718 / 2,718 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 115 | 2,592 / 2,592 | 35 / 35 | 115 / 115 | 80 / 80 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 2,669 | 2,669 / 2,673 | 2,666 / 2,664 | 2,669 / 2,668 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 157 | 2,574 / 2,486 | 155 / 152 | 157 / 153 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 1,735 | 2,163 / 2,163 | 1,578 / 1,578 | 1,722 / 1,722 | 170 / 170 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 1,855 | 1,996 / 1,996 | 1,850 / 1,850 | 1,855 / 1,855 | 5 / 5 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 90 | 1,637 / 1,637 | 18 / 18 | 36 / 36 | 208 / 208 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,192 | 84 / 84 | 23 / 23 | 1,192 / 1,192 | 1,169 / 1,169 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 56 | 897 / 897 | 0 / 0 | 28 / 28 | 168 / 168 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 603 | 943 / 943 | 564 / 564 | 601 / 601 | 41 / 41 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 17 | 931 / 931 | 0 / 0 | 0 / 0 | 42 / 42 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 928 | 922 / 922 | 922 / 922 | 928 / 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 157 | 706 / 741 | 157 / 153 | 157 / 153 | 0 / 17 | 0 / 52 | 0 / 0 |
| `05/00` | 639 | 11 | 512 / 512 | 0 / 0 | 0 / 0 | 127 / 127 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 251 | 502 / 502 | 245 / 245 | 251 / 251 | 6 / 6 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 391 | 391 / 391 | 310 / 310 | 391 / 391 | 81 / 81 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 287 | 384 / 384 | 222 / 222 | 287 / 287 | 65 / 65 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 239 | 326 / 326 | 179 / 179 | 234 / 234 | 65 / 65 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 100 | 271 / 271 | 8 / 8 | 100 / 100 | 92 / 92 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 101 | 303 / 303 | 101 / 101 | 101 / 101 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 16 | 254 / 254 | 0 / 0 | 12 / 12 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 233 | 84 / 84 | 52 / 52 | 233 / 233 | 181 / 181 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 223 | 88 / 88 | 64 / 64 | 223 / 223 | 159 / 159 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 81 | 239 / 239 | 77 / 77 | 81 / 81 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 16 | 240 / 240 | 14 / 14 | 16 / 16 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 4 | 151 / 151 | 0 / 0 | 0 / 0 | 84 / 84 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 36 | 213 / 213 | 23 / 23 | 34 / 34 | 15 / 15 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 157 | 157 / 236 | 157 / 154 | 157 / 155 | 0 / 1 | 0 / 80 | 0 / 0 |
| `17/01` | 141 | 3 | 108 / 108 | 0 / 0 | 0 / 0 | 33 / 33 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 9 | 132 / 132 | 7 / 7 | 7 / 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 28 | 120 / 120 | 10 / 10 | 28 / 28 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 119 | 4 / 4 | 2 / 2 | 102 / 102 | 134 / 134 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 11 | 47 / 47 | 1 / 1 | 8 / 8 | 67 / 67 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 93 | 18 / 18 | 8 / 8 | 93 / 93 | 85 / 85 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 28 | 84 / 84 | 28 / 28 | 28 / 28 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 16 | 77 / 77 | 10 / 10 | 16 / 16 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 79 | 83 / 83 | 79 / 79 | 79 / 79 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 4 | 71 / 71 | 3 / 3 | 4 / 4 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 47 | 70 / 70 | 46 / 46 | 47 / 47 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 65 | 9 / 9 | 6 / 6 | 65 / 65 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 4 | 65 / 65 | 1 / 1 | 4 / 4 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 66 | 16 / 16 | 16 / 16 | 66 / 66 | 50 / 50 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 | 60 / 60 | 0 / 0 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 12 | 59 / 59 | 7 / 7 | 12 / 12 | 5 / 5 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 62 | 62 / 62 | 62 / 62 | 62 / 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 4 | 60 / 60 | 3 / 3 | 4 / 4 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 4 | 58 / 58 | 2 / 2 | 4 / 4 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 3 | 59 / 59 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 51 | 14 / 14 | 6 / 6 | 51 / 51 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 2 | 59 / 59 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 3 | 56 / 56 | 2 / 2 | 2 / 2 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 3 | 58 / 58 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 3 | 58 / 58 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 3 | 58 / 58 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 3 | 57 / 57 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 3 | 56 / 56 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 3 | 55 / 55 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 3 | 54 / 54 | 2 / 2 | 3 / 3 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 | 0 / 0 | 0 / 0 | 53 / 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 2 | 53 / 53 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 17 | 46 / 46 | 12 / 12 | 17 / 17 | 5 / 5 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 51 | 51 / 51 | 51 / 51 | 51 / 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 2 | 50 / 50 | 1 / 1 | 2 / 2 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 40 | 19 / 19 | 11 / 11 | 40 / 40 | 29 / 29 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 1 | 44 / 44 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 1 | 43 / 43 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 | 42 / 42 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 14 | 35 / 35 | 8 / 0 | 14 / 7 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 1 | 38 / 38 | 0 / 0 | 0 / 0 | 3 / 3 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 16 | 32 / 32 | 16 / 16 | 16 / 16 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 4 | 32 / 32 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 4 | 29 / 29 | 3 / 3 | 4 / 4 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 13 | 19 / 19 | 3 / 3 | 13 / 13 | 10 / 10 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 4 | 20 / 20 | 0 / 0 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 23 | 23 / 23 | 23 / 23 | 23 / 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 14 | 0 / 0 | 0 / 0 | 14 / 14 | 14 / 14 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 1 | 14 / 14 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 | 0 / 0 | 0 / 0 | 7 / 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 3 | 6 / 6 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 4 | 4 / 4 | 4 / 4 | 4 / 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 3 | 3 / 3 | 3 / 3 | 3 / 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 1 | 2 / 2 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 | 0 / 89 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 89 | 0 / 3 |

The targeted intro/ending excess and the remaining live additions are
explicit above. Final live excess-call work audit:

| Entry | Live excess calls | Calls with identical main/VWF work |
| --- | ---: | ---: |
| `0E/02` | 97 | 0 |
| `07/00` | 17 | 0 |
| `10/00` | 2 | 0 |
| `16/04` | 3 | 0 |

### Crystal / pendant phases, exceptions and rupee gaps

Cells are source / prior batch / final, in presentation frames including
masked callbacks. All eight stable room/Link-XY/whole-save outcomes match
a source outcome. The final seven crystal phase tables retain the prior
values; all seven iris openings are 34 source/native frames.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 66 / 66 | 5 / 5 / 5 | 17 / 17 / 17 | 2 / 2 / 2 | 35 / 34 / 34 | 16 / 16 / 16 |
| 380000 | 84 / 85 / 85 | 7 / 7 / 7 | 24 / 23 / 23 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 460000 | 84 / 84 / 84 | 7 / 7 / 7 | 24 / 24 / 24 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 626000 | 91 / 89 / 89 | 9 / 9 / 9 | 24 / 24 / 24 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 734000 | 84 / 85 / 85 | 7 / 7 / 7 | 25 / 25 / 25 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 880000 | 84 / 84 / 84 | 7 / 7 / 7 | 26 / 26 / 26 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1064000 | 84 / 84 / 84 | 7 / 7 / 7 | 25 / 25 / 25 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |
| 1316000 | 84 / 83 / 83 | 7 / 7 / 7 | 25 / 25 / 25 | 2 / 2 / 2 | 34 / 34 / 34 | 16 / 16 / 16 |

| Mode | Classified WRAM differences | $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14735, "song-end-poll": 637}` | 10 |

The 162000–163000 probe has 0 $0CFD-only exceptions.
The same 27 logical iterations request rupee ticks. Presentation gaps:

- Prior batch: `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`.
- Final: `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`.
- Source: `[24, 8, 8, 8, 8, 12, 25, 14, 8, 8, 8, 8, 8, 19, 26, 8, 8, 8, 8, 8, 8, 14, 12, 8, 8, 8]`.

The first retained PCM onset interval is 23.002 frames using the
unchanged 7.9–8.7 kHz / threshold-600 detector. This is a diagnostic, not
exact PCM or intra-NMI timestamp acceptance.

### Remaining coverage at this stopping point

These are entry-category deficits, not exclusive causal assignments.

| Entry | Missing | Excess |
| --- | ---: | ---: |
| `07/00` | 1,785 | 0 |
| `09/00` | 1,169 | 0 |
| `07/02` | 968 | 0 |
| `0E/02` | 894 | 0 |
| `07/07` | 784 | 0 |
| `09/23` | 606 | 0 |
| `06/00` | 357 | 0 |
| `07/0E` | 302 | 0 |
| `09/03` | 296 | 0 |
| `0F/01` | 256 | 0 |
| `0E/0A` | 208 | 0 |
| `07/0F` | 192 | 0 |
| `09/16` | 181 | 0 |
| `07/15` | 170 | 0 |
| `0E/07` | 168 | 0 |
| `09/0D` | 159 | 0 |
| `08/00` | 152 | 0 |
| `0E/05` | 134 | 0 |

Next work remains the measured frequent sprite/control bodies (especially
spike-block and the intro parents), additional object/header/stair children,
and fresh-text sprite children beyond the shared predicates. Negative-byte
inventory selection equivalence, exact IRQ/HDMA/refresh placement and live
song timing remain open. Complete instruction coverage and exact A/V are
not established. No default/ROM-timing price or comparison exception was
added to improve these counts.

### Validation

| Check | Result |
| --- | --- |
| Final ordinary default / reduced builds | PASS; both warning-free |
| Fresh clean-main default baseline versus final default | PASS; 200,000 records and EOF match |
| Four intermediate cartridge gates | PASS; 1,473,123 iterations each |
| Final cartridge / live-SPC full gates | PASS; unexplained zero, complete=True |
| Zelda lib, default | 2,467 pass; 1 failure; 3 ignored; every listed case observed |
| Zelda lib, reduced | 2,002 pass; 0 failure; 2 ignored; every listed case observed |
| Pacing regressions, both configurations | 12 pass each |
| SNES lib, both configurations | 504 pass; 0 failures; 7 ignored each |
| Eight stable crystal/pendant outcomes | PASS |
| `git diff --check` | PASS |
| Complete instruction coverage / exact A/V | Not established |

The sole default lib failure is the existing
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
Lib-test builds retain their existing unused-fixture warnings; ordinary
builds are warning-free. Interrupted pre-final library runs are diagnostics
and are not included in these complete final-source suite counts.

`verification.json`, full stage/final CSV and JSON metrics, classification
traces, phase/stable/PCM receipts, source listings,
`candidate-source-sha256.json` and `artifact-hashes.json` bind this CPU
evidence. The supplied ledgers and oracle cache were read-only. No GPU,
`--video`, cached-A/V, external Snes9x, cargo GPU tests, `cargo fmt`, commit
or push was used. Scratch scripts are in `/tmp`.

Reproduce the final full census with the earlier command, setting
`BIN=target/cost-pacing4/bound-zelda3`, fresh save/output paths and
`--pacing-predictor`; append `--live-apu-reads` for the live-SPC pass.


## Continuation on `codex/cost-pacing5` (main `16d6f653`)

This batch starts with measurement instead of a lexical candidate search.
CPU evidence is retained in `target/cost-pacing5/`. The source profiler runs
this repository's real 65816 executor from reset, with the same ROM, SRAM
and input as the supplied read-only logical ledger. It does not invoke an
external emulator or renderer.

### Routine coverage tool

`ZELDA3_LEDGER_PROFILE=LO-HI,...` on `logical_frame_ledger` now writes
`routine-profile.csv`: iteration, ROM call target, calls, inclusive master
clocks and exclusive master clocks. It tracks JSR, indexed JSR and JSL
frames, including a call immediately followed by interrupt acceptance.
It uses the pre-instruction stack for ownership, recovers the interrupted
successor rather than the handler vector as the call target, and does not
insert handler calls into the main stack. Main begins at `$00:8038` and
ends after `$00:805D`; the root is keyed `$00:8034` to match native scopes.
Interrupt handlers and the polygon stack are excluded. Transaction durations
count CPU bus/internal work; interrupt entry, DMA/HDMA and refresh stalls
are excluded. Incomplete final main iterations are omitted.

`ZELDA3_LEDGER_PROFILE_ONLY=1` suppresses the ordinary iteration and XOR
image streams, saving time and disk for diagnostic runs. The output must
be a fresh directory. Profiling does not require picture capture.

Native `--logical-replay --routine-profile FILE` collects the live
`cycle_ledger` scope stack between the actual main counter's start and
its closing NMI. `--routine-profile-iterations LO-HI,...` restricts collection
without changing replay or seeding. Zero-priced calls remain visible;
muted probes and polygon work stay excluded. `000000` denotes unscoped
charges. `vwf` is the separate VWF work counter, deliberately left separate
rather than assigned to an invented routine. A `total` row preserves the
independent pacing counter. These switches require `parity-debug`.

`scripts/routine_coverage_diff.py SOURCE NATIVE` joins identical iteration
sets, checks exclusive-sum conservation against both totals, and ranks
source minus native clocks. It rejects missing iterations, duplicate rows
and inconsistent totals. `--windows` selects iterations; `--sort inclusive`
changes the ranking; `--json` and `--csv` retain every row, including every
source call target with no corresponding native scope. Bank mirrors are
normalized. Negative differences remain visible; no cap hides overpricing.

```sh
cargo build --profile parity -p snes --example logical_frame_ledger
cargo build --profile parity -p zelda3-bin --no-default-features --features parity-debug
LEDGER=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
CACHE=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
WINDOWS=1934,94000-96000
ZELDA3_LEDGER_PROFILE="$WINDOWS" ZELDA3_LEDGER_PROFILE_ONLY=1 \
  target/parity/examples/logical_frame_ledger saves/zelda3.sfc \
  "$CACHE/initial.srm" "$CACHE/input.txt" 110000 /tmp/source-coverage
ZELDA3_SAVE_DIR=/tmp/native-coverage-save target/parity/zelda3 \
  --logical-replay "$LEDGER" "$CACHE/initial.srm" --frames 96001 \
  --seed-every 2000 --report 0 --routine-profile /tmp/native-coverage.csv \
  --routine-profile-iterations "$WINDOWS" > /tmp/native-coverage.log
python3 scripts/routine_coverage_diff.py /tmp/source-coverage/routine-profile.csv \
  /tmp/native-coverage.csv --json /tmp/routine-ranking.json --csv /tmp/routine-ranking.csv
python3 -m unittest discover -s scripts -p test_routine_coverage_diff.py
```

These are raw call-target/scope comparisons, not an automatic price generator.
The ROM's tail jumps retain the last JSR/JSL frame: a dispatcher's exclusive
work can include its tail-jumped implementation. Native scopes sometimes
use an interior entry or absorb already-priced child instructions. For
example, `$00:E766` and `$00:E783` include decompression already charged at
`$00:E772` and `$00:E78F`; `$02:F691` map-definition and `$09:C6F5` proximity
work already have instruction charges inside native callers. A missing scope
there does not establish missing clocks. Inclusive differences overlap and
must never be summed or charged along with their children. Separate VWF
work and modeled raster waits also require interpretation. Song uploads
have their own masked-callback/SPC owner; their raw source poll clocks are
not an instruction-price worklist. Seeded native replay can follow different
work in timing-defined intro and live-SPC windows. Inspect call/state agreement
before treating a positive row as missing instruction coverage.

### ROM-derived additions

The first clean large missing routine in the intro ranking is
`Text_GenerateMessagePointers`, `$0E:D3EB..D44A`. The native asset-length
implementation produced the pointers without charging the ROM's token scan.
The new lagless-only price walks the encoded messages' actual token/argument
paths using the ROM length table; it includes both physical message banks,
end markers, pointer publications and the bank-switch overwrite. It excludes
the caller's JSL. The instruction prices from `scripts/rom_block_costs.py` sum to
16,480,686 clocks over the pinned corpus, exactly matching the independently collected source routine.
The existing instruction-by-instruction source test now checks the aggregate
price as well, for real messages and synthetic command/marker edge cases at
multiple raster phases.

The intro ranking also exposes `AnimateSceneSprite_MoveTriangle`,
`$0C:C9F1..CA4B`. Its new lagless-only price follows each actual zero,
positive or negative velocity branch, with M8/X8 and all eight actor indices
inside their indexed pages. A CPU-only test compares both charged clocks
and coordinate writes against executed ROM instructions across 50 combinations
of slot and velocity. No parent or child is charged twice. Other large rows
remain diagnostic work, rather than guessed aggregate charges.


The control at iteration 1934 measures 1,803,552 source versus 1,803,574
native clocks (difference -22). Its decompression body and byte-reader
charges agree exactly. The source cutoff is the `$00:805D` completion
store, whereas native also charges the real `$00:805F` 22-clock branch
back to the wait loop. This boundary difference is retained explicitly,
rather than fitted away. Some other routine differences cancel even in
this control because native scopes group instructions differently.
`control-final-diff.json` retains the complete decomposition.

The separate pendant validation covers every iteration 94000–96000:
341,838,800 source versus 245,674,122 native clocks, a raw difference of
96,164,678. Its largest source-only target is the song uploader (24,703,142
clocks), whose masked-callback ownership is separate. The ranking also
exposes already-priced map/proximity children without matching scopes and
real remaining player/control work. It must not be turned into a single
parent charge. `pendant-diff.json` and `pendant-ranking.txt` retain the result.


The combined sample checks both new routines per iteration, not only in
aggregate: `$0E:D3EB` has one source/native call and 16,480,686 clocks;
`$0C:C9F1` has 4,308 source/native calls and 2,192,168 clocks. Calls,
inclusive and exclusive clocks match on every sampled iteration for both
routines (`source-price-routine-agreement.json`). Together they remove
18,672,854 clocks from the combined raw coverage gap without adding a
parent charge or fitting a field count.

The final live-excess work audit covers every one of the 135 excess calls.
None has the same main/VWF work counters as the cartridge-answer run of
that iteration (`final-live-excess-work.json`). Counts by entry are
`07/00`: 15, `0E/02`: 99, `07/18`: 14, `10/00`: 2, `10/01`: 1,
`16/04`: 3 and `08/01`: 1. Their 582 excess holds are retained as different
executed work, separately from the cartridge price audit's zero excess.

### Sample coverage and ranked result

The final joined sample contains 3,829 complete iterations.
The sampling population is the first 193,000 logical iterations, not the
whole route: up to 32 quantiles of source-lagged calls per requested category,
plus the complete first intro, later attract-intro parents (146000–146900),
and pendant window (94000–96000). The mirror sample contains all 12 lagged
calls in that prefix; the load sample contains all 26. This is not a
full-route weighted routine census. Every requested parent present in this
prefix is represented.

| Entry | Sample iterations | Source-lagged sample | Source clocks | Native clocks | Raw difference |
| --- | ---: | ---: | ---: | ---: | ---: |
| `07/00` | 947 | 34 | 99,344,194 | 86,575,618 | +12,768,576 |
| `09/00` | 769 | 32 | 137,967,036 | 77,207,094 | +60,759,942 |
| `07/02` | 32 | 32 | 36,874,172 | 34,993,346 | +1,880,826 |
| `0E/02` | 155 | 40 | 46,127,862 | 45,973,776 | +154,086 |
| `07/07` | 32 | 32 | 17,057,778 | 16,375,822 | +681,956 |
| `09/23` | 12 | 12 | 26,739,308 | 26,174,240 | +565,068 |
| `06/00` | 26 | 26 | 628,180,578 | 493,530,656 | +134,649,922 |
| `00/00` | 1 | 1 | 638,910 | 220,648 | +418,262 |
| `00/01` | 25 | 11 | 40,471,972 | 33,580,788 | +6,891,184 |
| `00/02` | 1 | 1 | 800,976 | 793,036 | +7,940 |
| `00/03` | 87 | 0 | 8,934,248 | 7,961,744 | +972,504 |
| `00/04` | 882 | 0 | 89,113,774 | 79,480,746 | +9,633,028 |
| `00/05` | 123 | 62 | 18,546,470 | 17,018,398 | +1,528,072 |
| `00/06` | 84 | 2 | 9,276,454 | 7,567,080 | +1,709,374 |
| `00/07` | 175 | 14 | 25,889,694 | 22,286,554 | +3,603,140 |
| `00/0A` | 1 | 1 | 801,078 | 793,042 | +8,036 |
| `00/0B` | 87 | 0 | 8,947,812 | 7,966,956 | +980,856 |

The combined source total is 1,382,428,874; native is
1,093,175,802; raw source minus native is 289,253,072.
Before pricing the raw difference was 307,925,926. Every
iteration's exclusive rows reconcile to its independent total on both sides.
`combined-{before,final}-diff.json` retains inclusive/exclusive rankings,
per-iteration totals and every source target with no native scope.

Largest raw exclusive gaps after pricing (subject to the scope/owner caveats above):

| ROM target | Source exclusive | Native exclusive | Difference | No native scope | Resolution in batch 6 |
| --- | ---: | ---: | ---: | --- | --- |
| `008888` | 141,834,220 | 0 | +141,834,220 | yes | (c) Song-transfer/APU owner; 141,834,220 source poll clocks, outside accepted-main pricing. See masked-interval reconciliation below. |
| `00e609` | 39,470,052 | 44,260 | +39,425,792 | no | (b) Loader tail conversion already priced. Inclusive 115,521,682 / 115,521,682 on every iteration after ancestry repair. |
| `00e60d` | 32,700,984 | 38,712 | +32,662,272 | no | (b) Same loader ownership repair. Inclusive 103,821,718 / 103,821,718 on every iteration. |
| `00e583` | 28,206,146 | 54,150 | +28,151,996 | no | (b) Exact except two file-select tail aliases; their common InitializeTilesets parents are each exact (22,703,874 combined). No added price. |
| `00e783` | 22,572,686 | 0 | +22,572,686 | yes | (a/b) Decoder already priced under E78F; add the real 128-clock destination prologue. Inclusive 34,156,226 / 34,156,226, every iteration. |
| `0ecb5e` | 20,601,368 | 0 | +20,601,368 | yes | (c) Existing VWF counter owns glyph work; no duplicate ledger charge. Joint text-owner arithmetic below. |
| `00e766` | 18,242,600 | 0 | +18,242,600 | yes | (a/b) Decoder already priced under E772; explicit fixed-buffer callers add 128 clocks. Four initializer aliases reconcile at their exact common E19B parent. |
| `0080b5` | 14,879,202 | 918,960 | +13,960,242 | no | (b) Scope ended before module routing. Keep it open through the JML target; control 1934 inclusive 1,786,402 / 1,786,402. Other child coverage remains open. |
| `00e6b7` | 25,808,080 | 12,465,636 | +13,342,444 | no | (b) Raw low conversion tail stays with LoadCommonSprites. Two file-select aliases reconcile at exact E19B parents; no added price. |
| `0ec9fd` | 12,849,104 | 0 | +12,849,104 | yes | (c) Existing VWF counter owns dispatch/handler work; joint text-owner arithmetic below. No parent-plus-child charge. |
| `00d5ce` | 11,667,700 | 0 | +11,667,700 | yes | (a/b) Wrong conversion variant: D5CE costs 3808 per tile, DFB8 costs 3736. Correct explicit callers; 66 calls total 11,667,700 on both sides, every iteration. |
| `09c44e` | 10,189,788 | 0 | +10,189,788 | yes | (b) Existing reset clears, now scoped at the call target. Exclusive 10,189,788 / 10,189,788; inclusive child shortfall 179,538 is DisableAll, not reset self work. |
| `0cc534` | 10,053,710 | 0 | +10,053,710 | yes | (a) Price dispatch and actual title-actor branches/children. Inclusive 117,190,814 and exclusive 10,053,710 match every sampled iteration. |
| `02f691` | 9,277,008 | 0 | +9,277,008 | yes | (b) Existing map cache/store charges, including cursor fragments; inclusive 9,277,008 / 9,277,008 every iteration. No added price. |
| `09c6f5` | 8,576,088 | 0 | +8,576,088 | yes | (b) Existing bounds/index price, now scoped at C6F5. Exclusive 8,576,088 / 8,576,088. Price C739 self separately; remaining 22,086 belongs to C770 activation. |
There are 613 source targets without a native scope in the combined sample.
These rows are retained even when their work is already bundled elsewhere.
The original 2,928-iteration paired native audit is strictly addition-only:
17,569,120 clocks across 702 iterations, exactly the two newly priced routines,
with no negative delta or other changed work (`sample-work-deltas.json`).

### Full census progression

| Stage | Cartridge held | Missing | Excess | Zero-lag FP |
| --- | ---: | ---: | ---: | ---: |
| Main / prior batch | 92,533 | 10,144 | 0 | 0 |
| Pointer scan + triangle motion / final | 92,580 | 10,097 | 0 | 0 |

Both final full gates PASS: 1,473,123 iterations each, unexplained zero,
complete=True. Runtime holds equal P2 on every iteration. This is one
meaningful pricing group, followed by full cartridge and live-SPC censuses.

| Mode | Held | Exact lagged | Within one | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 92,580 | 30,656 | 38,884 | 10,097 | 0 | 0 |
| live | 92,474 | 30,529 | 38,752 | 10,785 | 582 | 131 |

Masked elapsed callbacks, separate from accepted holds: 2,839 cartridge / 2,814 live.

The retained live-SPC excess is reported separately from cartridge pricing
accuracy; these runs can execute different message/load work after song polls.
No charge, song duration or comparison exception was fitted to erase it.

### Category table

Paired cells are cartridge / live. Every category with source lag or a
zero-lag false positive is included. Full JSON retains the other categories.

| Entry | Source lag | Missing C / L | Excess C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 894 / 1,210 | 0 / 230 | 0 / 99 |
| `07/02` | 8,770 | 968 / 968 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 357 / 357 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 302 / 302 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 152 / 382 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 606 / 606 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 256 / 256 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 296 / 296 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 784 / 784 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,785 / 1,785 | 0 / 91 | 0 / 15 |
| `09/01` | 3,479 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 80 / 80 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 170 / 170 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 5 / 5 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 208 / 208 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,169 / 1,169 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 168 / 168 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 41 / 41 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 42 / 42 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 52 | 0 / 0 |
| `05/00` | 639 | 127 / 127 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 6 / 6 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 81 / 81 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 65 / 65 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 65 / 65 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 92 / 92 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 181 / 181 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 159 / 159 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 84 / 84 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 15 / 15 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 80 | 0 / 0 |
| `17/01` | 141 | 33 / 33 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 7 / 7 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 134 / 134 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 20 / 20 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 85 / 85 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 50 / 50 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 5 / 5 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 5 / 5 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 29 / 29 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 3 / 3 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 3 / 3 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 10 / 10 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 4 / 4 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 14 / 14 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 89 | 0 / 3 |

### Crystal / pendant phases, exceptions and rupee gaps

Cells are source / final presentation frames. All phase values and all eight
stable source room/Link-XY/whole-save outcomes retain the preceding batch.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 66 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 89 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

| Mode | Classified WRAM differences | $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14734, "song-end-poll": 635}` | 10 |

The 162000–163000 probe has 0 $0CFD-only exceptions. The same 27 logical
iterations request rupee ticks. Final request gaps remain
`[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`.
The unchanged source gaps are `[24, 8, 8, 8, 8, 12, 25, 14, 8, 8, 8, 8, 8, 19, 26, 8, 8, 8, 8, 8, 8, 14, 12, 8, 8, 8]`.
The first retained PCM onset interval is 23.002 frames with the
unchanged detector; this is diagnostic rather than exact PCM acceptance.

### Validation and stopping point

| Check | Result |
| --- | --- |
| Final default / reduced ordinary builds | PASS, both warning-free |
| Source profiler example build | PASS, warning-free |
| Fresh clean-main default baseline vs final default | PASS, 200,000 records and EOF |
| Full cartridge / live-SPC census gates | PASS, unexplained zero, complete=True |
| Zelda lib, default | 2,469 pass; known spotlight failure; 3 ignored |
| Zelda lib, reduced | 2,004 pass; no failures; 2 ignored |
| SNES lib, both configurations | 504 pass; no failures; 7 ignored each |
| Pacing regressions, both configurations | 12 pass each |
| Routine-profile tests, both configurations | 2 pass each |
| Diagnostic join tests | 4 pass |
| ROM token-scan price test and triangle branch/write test | PASS |
| Stable crystal/pendant outcomes | 8 source matches |
| `git diff --check` | PASS |

The full libraries are isolated libtest shards with a complete disjoint case
union recorded in `library-shards.json`; no GPU test target was run. The sole
default failure remains
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
Lib-test fixture warnings are unchanged; ordinary builds are warning-free.

This is a verified partial pricing stopping point. The routine-diff tool is
implemented and exercised across every requested category and intro parent
present in the sampled prefix. Broad player/control and loader residuals
remain; raw call-target deficits require scope/ownership checks before more
prices are added. Whole-route routine profiling, complete instruction coverage,
exact raster scheduling, unseeded quest parity and exact A/V are not established.

`verification.json`, source/binary hashes, both full predictor CSVs, classification
traces, combined routine dumps/rankings and phase/stable/PCM receipts retain
the evidence. The supplied logical ledgers and oracle cache were read-only.
No GPU, `--video`, cached-A/V, external Snes9x, cargo GPU tests, `cargo fmt`,
commit or push was used. Scratch drivers are in `/tmp`.

## Continuation on `codex/cost-pacing6` (main `b9fa7580`)

Evidence is retained in `target/cost-pacing6/`. This batch classifies all fifteen
previous raw rows, adds instruction work where proved, and retains the unresolved
ordinary-instruction remainder. Prices are lagless-only and derived from the
ROM instructions for executed native work. No scene threshold, multiplier,
lag-count lookup, parent-plus-child price, or default/ROM-timing price was added.

### Scope repair and per-iteration proof

The resolution column above refers to the same 3,829-iteration prefix sample.
`final-prefix-agreement.json` retains source/native inclusive and exclusive
totals and every mismatching iteration. Opening a module or loader scope does
not add clocks. Flattened sheet continuations now expose their native call
composition while profiling, count virtual call entries rather than retired
instructions, and retain tail conversion in its caller. A conservation test
checks repeated byte-reader entries, nested totals and exclusive-sum identity.
The source selector sorts/merges windows and uses binary lookup; the CPU
execution and accounting boundaries are unchanged.

| Work | Source inclusive | Native inclusive | Per-iteration result |
| --- | ---: | ---: | --- |
| `$00882e` | 5,790,310 | 5,790,310 | exact |
| `$1bbbf4` | 6,075,484 | 6,075,484 | exact |
| `$09c739` | 3,035,086 | 3,013,000 | 6 child-residual iterations |
| `$1debf3` | 3,192,288 | 3,192,288 | exact |
| `$00d5ce` | 11,667,700 | 11,667,700 | exact |
| `$00e783` | 34,156,226 | 34,156,226 | exact |
| `$00e75c` | 13,190,954 | 13,190,954 | exact |
| `$00e609` | 115,521,682 | 115,521,682 | exact |
| `$00e60d` | 103,821,718 | 103,821,718 | exact |
| `$0cc534` | 117,190,814 | 117,190,814 | exact |
| `$02f691` | 9,277,008 | 9,277,008 | exact |
| `$00e19b` | 349,405,434 | 349,405,434 | exact |

`C739` self clocks are exactly 3,013,000 on both sides; its 22,086 inclusive
shortfall is the still-unpriced `C770` child on six iterations. Similarly,
`C44E` self work matches while `C22F` disable work remains unpriced. These
child gaps are not charged again as a reset/proximity parent.

The four remaining E766 aliases are iterations 25, 729, 146148 and 146886.
Their common E19B inclusive totals are respectively 9,331,474, 10,808,714,
10,417,920 and 11,895,160 on both sides. The two file-select E583/E6B7
aliases are 729 and 146886. Their flattened common-sheet ancestry is still
reported differently, but the same parent proof excludes missing aggregate
work. Store-cursor F691 rows can still count call fragments; their clocks
are exact on every iteration. The full 80B5 module scope has 627 exact
inclusive controls, including 1934. Other iterations retain real descendant
coverage gaps; a matching control is not a claim of full module coverage.

### ROM-derived instruction additions

`source-price-blocks.json` independently re-sums 26 basic blocks using
`scripts/rom_block_costs.py`, with the actual M/X widths and system-bank IO
access prices. Listings are in `source-costs/`. Executed-ROM branch tests
cover entrance/tile paths, all eight Armos coordinator states and title actor
phases/subtypes. They compare charged work with CPU transactions, independent
of observed lag counts.

| Group | Added work | Exclusions |
| --- | --- | --- |
| 1 | `$00:882E..8887` outdoor tile attributes; `$1B:BBF4..BD79` entrance tests and actual descending lookup passes. | Early exits, second comparisons, follower checks and table branches follow native operands; drawing children keep their own prices. |
| 2 | `$1D:EBF3..EDD5` Armos dispatch, timer/state paths, six-knight rotation and multiply/sign paths; `$09:C739..C76F` presence/loaded tests; `$00:D5CE..D618` animated low conversion; `$00:E75C/E766/E783` fixed-buffer prologues. | Child calls price only their call instruction at the parent. `C770` activation remains incomplete. The 3808-clock low variant replaces the 3736-clock price only at audited callers and only in lagless mode. |
| 3 | `$0C:C534` actor dispatch, actual title-actor initializer/animation branches and drawing wrappers, plus contracting motion. | Existing C972 OAM drawing and C9F1 motion are not repriced; tail instructions stay with C534, and ordinary called children retain scopes. |

The paired prefix work audit is addition-only: 30,875,150 new clocks.
The raw prefix difference falls from 289,253,072 to 258,377,922.
No negative work delta occurs. `prefix-work-deltas.json` retains every iteration.

### Masked uploads, raster and VWF owners

The original 06/00 sample has 628,180,578 source and 493,530,656 native
instruction clocks: a raw gap of 134,649,922. Of that, 116,651,698 belongs
to fifteen `$00:8888` song-transfer calls. The read-only source ledger
records 338 host intervals beyond `lag_nmis + 1` for those calls, an interval
width of 120,790,384 clocks. Poll CPU work plus the 4,138,686-clock remainder
for refresh, other instructions and endpoint phase equals that interval
width. This is an accounting envelope, not a new instruction price. There
are no VWF or iris-raster-call clocks in these 26 iterations. Native bank-1
transfers use the SPC receiver/caller clock and do not report those 338 as
extra `native_elapsed` callbacks; accepted holds and masked time must not be
added together as instruction work. `source-upload-ownership.json` retains
the individual ledger boundaries and poll clocks.

Group 2 adds 186,368 real instruction clocks to these 06/00 calls: 179,712
for low conversion and 6,656 for destination setup. Therefore the final
raw gap is 134,463,554 = 116,651,698 upload-owned + 17,811,856 remaining
signed ordinary-instruction difference. The latter is not proved to be
a wholly missing-price sum: it still contains object/header/control gaps
and offsetting native scope/price differences. It remains open, rather
than being turned into a loader parent charge.

Across the prefix, `$8888` has 141,834,220 source poll clocks and 410
hidden source host intervals (146,520,880 clocks), leaving 4,686,660 for
the same envelope terms. No upload-duration or song-end answer was changed.

The two text rows C9FD/CB5E contain 33,450,472 source-exclusive clocks.
The existing native VWF counter already contains 33,803,306. Adding CAB8
wrapper work (213,436) and the C448 prefix difference (157,150) gives
33,821,058, leaving a 17,752-clock boundary/coverage residual against VWF.
`prefix-owner-rows.json` preserves the per-iteration rows; this residual is
not a fitted runtime correction. No dispatch or glyph body was charged a
second time. Exact text-owner partitioning remains diagnostic work.

### Category-restricted rankings and whole-route sampling

The whole-route population is all 1,473,123 supplied logical iterations.
The sample takes up to 32 quantiles of source-lagged calls in each of the
twenty largest missing categories, including all eleven 05/00 calls: 619
iterations total. `whole-selection.json` retains each population count and
every selected index. Unlike the prior prefix, this includes late-route
loads, mirror calls and gameplay through iteration 1,451,722. Source work
comes from the repository CPU executor from reset; no renderer or external
emulator runs. All overlapping prefix/whole source rows are byte-identical.

| Entry | Whole samples | First | Last | Source clocks | Native clocks | Raw gap |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `05/00` | 11 | 1915 | 1394043 | 218,198,060 | 177,352,272 | +40,845,788 |
| `07/0F` | 32 | 1916 | 1394061 | 14,755,136 | 12,593,148 | +2,161,988 |
| `0E/02` | 32 | 1934 | 1449430 | 37,758,870 | 37,592,214 | +166,656 |
| `07/00` | 32 | 4089 | 1451722 | 15,732,288 | 12,178,650 | +3,553,638 |
| `0F/01` | 32 | 4272 | 1323648 | 14,344,778 | 11,295,972 | +3,048,806 |
| `08/00` | 32 | 4289 | 1320895 | 644,830,344 | 443,834,896 | +200,995,448 |
| `0E/07` | 32 | 5291 | 1156740 | 206,744,352 | 178,864,202 | +27,880,150 |
| `09/03` | 32 | 6223 | 1322111 | 191,144,100 | 183,017,000 | +8,127,100 |
| `06/00` | 32 | 10425 | 1323649 | 805,621,926 | 612,376,096 | +193,245,830 |
| `07/02` | 32 | 10962 | 1438636 | 56,071,046 | 52,794,854 | +3,276,192 |
| `07/0E` | 32 | 12364 | 1438313 | 25,536,400 | 24,684,744 | +851,656 |
| `09/00` | 32 | 35294 | 1322789 | 12,687,190 | 8,339,536 | +4,347,654 |
| `07/07` | 32 | 83118 | 1434509 | 13,439,140 | 12,867,172 | +571,968 |
| `07/15` | 32 | 83963 | 1393746 | 25,107,642 | 24,110,156 | +997,486 |
| `0E/05` | 32 | 111396 | 111516 | 15,015,754 | 6,376,370 | +8,639,384 |
| `09/2E` | 32 | 168344 | 1161553 | 40,620,584 | 33,539,276 | +7,081,308 |
| `09/0D` | 32 | 181070 | 705663 | 9,944,256 | 9,763,010 | +181,246 |
| `09/16` | 32 | 181145 | 705756 | 12,119,612 | 11,874,950 | +244,662 |
| `09/23` | 32 | 192053 | 1289766 | 49,340,350 | 48,149,112 | +1,191,238 |
| `0E/0A` | 32 | 497583 | 1285063 | 172,351,410 | 143,569,572 | +28,781,838 |

Whole-route `09/00` ranking, exclusive source minus native; negative rows
remain in the complete JSON/TXT artifact. Inclusive deficits overlap.

| Target | Source exclusive | Native exclusive | Difference |
| --- | ---: | ---: | ---: |
| `07d9d8` | 191,334 | 0 | +191,334 |
| `00ee21` | 178,396 | 0 | +178,396 |
| `0684e2` | 315,646 | 151,158 | +164,488 |
| `06dc10` | 153,120 | 6,578 | +146,542 |
| `02f3b9` | 130,652 | 0 | +130,652 |
| `08a7ab` | 129,996 | 0 | +129,996 |
| `07d798` | 129,788 | 0 | +129,788 |
| `07e245` | 117,700 | 0 | +117,700 |
| `02bb90` | 107,316 | 0 | +107,316 |
| `06e72f` | 99,476 | 0 | +99,476 |
| `06f7ef` | 90,272 | 0 | +90,272 |
| `06f836` | 89,328 | 0 | +89,328 |

Whole-route `07/00` ranking, exclusive source minus native; negative rows
remain in the complete JSON/TXT artifact. Inclusive deficits overlap.

| Target | Source exclusive | Native exclusive | Difference |
| --- | ---: | ---: | ---: |
| `07d9d8` | 234,554 | 0 | +234,554 |
| `06e72f` | 149,816 | 0 | +149,816 |
| `06e883` | 129,100 | 0 | +129,100 |
| `07e245` | 125,664 | 0 | +125,664 |
| `00d61c` | 97,248 | 0 | +97,248 |
| `07d798` | 95,970 | 0 | +95,970 |
| `0080b5` | 101,164 | 7,680 | +93,484 |
| `09b0b6` | 251,386 | 169,818 | +81,568 |
| `1e8b19` | 110,082 | 28,950 | +81,132 |
| `0dfa70` | 70,296 | 0 | +70,296 |
| `06f7ef` | 68,646 | 0 | +68,646 |
| `08a7ab` | 68,608 | 0 | +68,608 |

The prefix category gap falls from 60,759,942 to 46,260,246 for 09/00,
and from 12,768,576 to 10,315,970 for 07/00. The leading proved additions
are entrance/tile work and C739 in 09/00, and fixed-buffer prologues plus
Armos work in 07/00. The sprite bank-5 dispatcher was already priced;
its existing 844-clock wrapper/dispatch sequence now exposes real scopes.
Remaining tile classification, velocity, ordinary sprite/control and
object/header gaps are retained in the new route-wide rankings.

The independent whole sample totals 2,581,363,238 source / 2,045,173,202 native.
The union has 4,431 iterations and a raw difference of
765,417,754; exclusive rows reconcile to the independent total
on every iteration. This is a quantile sample, not a weighted full-route
routine census.

### Full census progression

| Stage | Held | Missing | Excess | Zero-lag FP |
| --- | ---: | ---: | ---: | ---: |
| Main / prior batch | 92,580 | 10,097 | 0 | 0 |
| Entrance / tile | 92,585 | 10,092 | 0 | 0 |
| Armos / presence / loader entries / conversion | 92,771 | 9,906 | 0 | 0 |
| Intro actors | 92,778 | 9,899 | 0 | 0 |
| Final build / profile accounting | 92,778 | 9,899 | 0 | 0 |

All three meaningful cartridge pricing stages and both final full gates
PASS with unexplained zero and complete=True. Each contains 1,473,123
iterations. Every runtime hold equals P2; cartridge excess and false
positives remain zero. Live-SPC can execute different work and is reported
separately. Intermediate binaries are immutable; their launch-time source
hash files can include later edits and are not build bindings. Final receipts
are bound to `final-source-sha256.json` and final binary hashes.

| Mode | Held | Exact lagged | Within one | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 92,778 | 30,785 | 38,946 | 9,899 | 0 | 0 |
| live | 92,671 | 30,654 | 38,814 | 10,592 | 586 | 131 |

### Complete category table

Paired cells are cartridge / live; every category with source lag or a false
positive is included.

| Entry | Source lag | Missing C / L | Excess C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 894 / 1,211 | 0 / 229 | 0 / 98 |
| `07/02` | 8,770 | 968 / 968 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 351 / 351 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 302 / 302 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 56 / 290 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 542 / 542 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 256 / 256 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 296 / 296 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 784 / 784 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,785 / 1,785 | 0 / 93 | 0 / 16 |
| `09/01` | 3,479 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 80 / 80 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 170 / 170 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 200 / 200 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,165 / 1,165 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 168 / 168 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 41 / 41 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 42 / 42 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 |
| `05/00` | 639 | 127 / 127 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 81 / 81 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 65 / 65 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 65 / 65 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 92 / 92 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 181 / 181 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 159 / 159 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 84 / 84 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 12 / 12 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 |
| `17/01` | 141 | 33 / 33 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 7 / 7 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 134 / 134 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 20 / 20 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 85 / 85 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 50 / 50 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 5 / 5 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 29 / 29 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 3 / 3 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 3 / 3 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 10 / 10 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 |

### Crystal / pendant phases, exceptions and rupee gaps

Cells are source / final presentation frames. Corrected instruction work
adds one properties frame in seven probes; other phases retain their prior
values. All eight stable room/Link-XY/whole-save outcomes match source.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 67 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 86 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 86 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 85 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

| Mode | Classified WRAM differences | $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14735, "song-end-poll": 636}` | 10 |

The 162000–163000 probe has 0 $0CFD-only exceptions. The same 27 rupee
requests retain gaps `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`. The PCM onset
gap remains 23.002 frames with the unchanged detector;
this is a diagnostic, not exact PCM acceptance. Concatenated probe absolute
frame indices shift with preceding extra work; request intervals do not.

### Validation and stopping point

| Check | Result |
| --- | --- |
| Final ordinary default / reduced builds | PASS, both warning-free |
| Source profiler build | PASS, warning-free |
| Fresh clean-main default baseline vs final default | PASS, 200,000 records and EOF |
| Three stage cartridge gates; final cartridge / live-SPC gates | PASS, complete=True, unexplained zero |
| Zelda lib, default | 2,473 pass; 1 failure; 3 ignored |
| Zelda lib, reduced | 2,008 pass; 0 failure; 2 ignored |
| SNES lib, both configurations | 504 pass, no failures, 7 ignored each |
| Pacing / profile regressions, both configurations | 12 / 3 pass |
| Executed-ROM price path matrices, both configurations | 3 pass |
| Diagnostic join tests | 4 pass |
| ROM basic-block audit | 26 independently checked |
| Stable crystal/pendant outcomes | 8 source matches |
| `git diff --check` | PASS |

The sole default lib failure is the known
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
All listed library cases occur in a complete disjoint shard union. Lib-test
fixture warnings remain separate from warning-free ordinary builds.

This is a verified partial instruction-coverage stopping point. The original
large raw rows have explicit price/scope/owner resolutions, with residual
child work shown rather than silently absorbed. The 06/00 ordinary remainder,
exact VWF owner boundary, and new route-wide sprite/control leaders are still
open. Complete instruction coverage, exact raster scheduling, unseeded-route
parity and exact A/V are not established.

`verification.json`, final source/binary hashes, full predictor CSVs, per-iteration
proofs, category rankings, library shard manifest, phase/stable/PCM receipts
and the source selection retain the evidence. The supplied ledgers and
oracle cache were read-only. No GPU, `--video`, cached-A/V, external Snes9x,
cargo GPU tests, `cargo fmt`, commit or push was used. Scratch drivers remain
in `/tmp`. The interrupted initial whole-profile attempt and failed synthetic
fixture setup are superseded diagnostics, not gate receipts. The first final
validation attempt was also superseded after a muted-preview profile-counting
bug was found. The final binaries and all final checks were rebuilt and rerun;
`v1-superseded/` retains the earlier diagnostics. Final profile collection
matches the unprofiled group-3 census on all 1,473,123 main/VWF/hold/masked/P2
rows (`profile-execution-agreement.json`).

## Continuation on `codex/cost-pacing7` (main `4ba8a61d`)

Evidence is retained in `target/cost-pacing7/`. This is a verified partial
instruction-coverage stopping point. The properties overshoot is corrected,
and the first occurrence-weighted gameplay group is priced. Four requested
gameplay targets and individual sprite/garnish/effect bodies remain open;
the load-category pass separates owners and retains rankings, but adds no
load-only runtime group in this batch. No price uses observed lag counts.

### Properties: instruction work versus masked-upload ownership

The per-iteration profiler does not show an excess ordinary-instruction sum
in the original properties calls. It shows missing ordinary work alongside
long masked presentation time. Source `$00:8888` polling belongs to the
existing SPC transfer owner; it is not another main-work price. Fresh cold
source rows and before/final native rows are retained for all 27 selected
iterations in `properties-per-iteration.json`. The before cartridge probe
work counters match the retained prior full census on every selected index.

| Source iteration | Source upload polling | Source ordinary | Before native ordinary | Final native ordinary | Remaining signed gap |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 95127 | 8,173,754 | 13,934,920 | 13,785,428 | 13,880,010 | 54,910 |
| 95229 | 8,729,604 | 13,935,328 | 13,785,428 | 13,880,010 | 55,318 |
| 383631 | 8,354,452 | 13,928,050 | 13,791,640 | 13,885,990 | 42,060 |
| 465811 | 8,182,780 | 13,893,212 | 13,750,896 | 13,845,246 | 47,966 |
| 629840 | 8,556,068 | 15,261,190 | 15,100,066 | 15,218,632 | 42,558 |
| 738183 | 8,191,712 | 14,005,638 | 13,866,764 | 13,961,114 | 44,524 |
| 884322 | 8,186,462 | 13,989,524 | 13,847,918 | 13,942,268 | 47,256 |
| 1066958 | 8,109,584 | 13,976,402 | 13,842,012 | 13,936,362 | 40,040 |
| 1320895 | 8,592,834 | 13,779,826 | 13,644,982 | 13,739,332 | 40,494 |

The runtime previously began the SPC transfer and drained its masked
callbacks before presenting the holds for the ordinary work that preceded
the upload command. The ROM reaches `$02:855D` after that work. Lagless
uploads now retain the native work position at the command, present only
that prefix's accepted holds, and then start the same receiver protocol.
Any ordinary suffix remains after the transfer. Prefix NMIs cannot replace
the main's ordinary closing-handler receipt. The clock uses the existing
P2 work/refresh/interrupt calculation; it reads no source iteration, lag count,
scene threshold or duration table. Default and ROM-timing paths retain their
previous charges and scheduling.

Prefix NMIs retain the source $12 latch, so they cannot consume pending
graphics updates early. The saved completed-main latch is restored only
at the post-upload closing NMI. The regression retains pending graphics
through every prefix callback and proves that the closing NMI consumes them.

The ordering-only full cartridge control changes masked time while retaining
92,778 holds and 9,899 missing holds, with excess/FP zero. An initial
instruction-interleaving trial did not change the probes and was reverted.
The final correction also prices actual missing properties work: song-list
selection/copies and palette copies. The later gameplay high-plane conversion
charge is independently derived as well.

### Source-derived additions and ownership boundaries

| Work | ROM instruction derivation | Boundary |
| --- | --- | --- |
| Song list `$02:C463..C4BF` | Actual progress/sword branches; 64 primary and 96 secondary byte-copy passes, loop branches, final music byte and return. | No upload polling. |
| Palette copies `$1B:EF30/EF4B`, EF7B helper | Actual words/rows, 270-clock single-bank or 318-clock paired-store passes, row setup and actual loop branches. | Partial EF4B ranges own each instruction once; empty/final-empty slices do not repeat entry/return. Direct gear-copy implementations bypass EF7B and remain open. |
| Tile reset `$07:D798..D7D7` | 23 M16 word clears and RTS, 914 clocks. | No collision/classification parent charge. |
| Tile lookup `$07:D9D8..DA27`, `$DC2A..DC47` | Actual indoor/floor/cheat branches and indirect dispatch. | Outdoor `$00:882E` stays a child; selected tile effects remain partially unpriced. |
| Sprite hitbox `$06:F7EF..F835`, overlap `$F836..F863` | Actual negative-Z exit, coordinate/extent stores, Y-then-X high-byte and extent exits. | No collision parent deficit is absorbed. |
| Sprite tile lookup `$06:E883..E8D5`; direction prefix `$E72F..E73E` | Actual floor/indoor lookup or outdoor wrapper; flags/direction and indoor branch at the direction entry. | `$00882E` is separate; shared tile-property work after E740/E773 remains open. |
| High expansion `$00:D61C..D66D` | Actual tiles, eight row-copy passes per tile, source pointer-wrap and loop branches. | The existing E19B/DF4F conversions are separate owners. |
| Subtractive palette `$00:EE21..EE77` | Original channel comparisons, actual decrement/taken branches, word stores and range loop. | No palette child or restore parent duplicate. |
| HUD `$0D:FA70/FA85` | Classic FA70 template: 330 MVN bytes at 52 each plus caller self work, 17,574; standalone FA85 wrapper 270. | FB91 scopes existing children. Rebuild does not charge the standalone FA85 entry; enhanced non-ROM template variants stay outside the new FA70 price. |

`scripts/rom_block_costs.py` independently re-sums 60 shipped basic blocks
in `source-price-blocks.json`; listings are in `source-costs/`. Twelve
executed-ROM price-test families pass in both configurations, including
partial palette-row cuts, pointer wraps, negative Z, overlap exits,
indoor/floor/cheat branches, all song-list selections and HUD caller self
work excluding its actual callees. The sample work audit is addition-only: 25,426,870 new clocks over 4,450 joined iterations.
Negative per-iteration work deltas: zero. Raw ancestry and executed-work
mismatches remain visible in `price-routine-agreement.json`.

The D61C raw excess on twelve sample iterations is an entry alias: source
D619 loads Y=6 and falls through to the same body. After excluding that
still-unpriced 24-clock LDY per source entry, source D61C plus D619 body
work matches native D61C exactly on all twelve iterations; see
`high-expansion-alias-proof.json`. No high-expansion price was adjusted.
The overlap row is exact except iteration 620430: four calls on both sides,
2,218 source / 2,918 native clocks. The totals imply one native full two-axis
path instead of a source early Y exit. Its 700-clock operand/work difference
is retained, not fitted away; this is not an exact source-behavior claim.

EF7B's shared helper is priced, but all 108 sampled source EF7B rows still
bypass it through direct gear-copy implementations. Their 802,632 clocks
remain explicitly unpriced in this sample. The helper's executed-ROM test
is not claimed as route-wide integration of those direct-copy callers.

### Requested gameplay rows: resolution and occurrence weighting

The same whole-route source-lagged quantiles are reused: 32 calls each in
07/00 and 09/00. Occurrences below are exact within those samples; population
weighting by their 2,960 / 1,192 lagged-call populations is a prioritization
heuristic, not an unbiased full-route routine-frequency census. Large
clock totals on one rare call do not outrank repeated gameplay work solely
on their aggregate clocks.

| Target | Sample occurrences 07/00; 09/00 | Resolution |
| --- | ---: | --- |
| `0684e2` | 32; 32 | already-scoped; residual open: Existing entry/timers/state dispatch and 068510 inactive tail already own clocks. Source tail-dispatched actor bodies can remain in 0684e2; price those individual bodies, not this aggregate deficit. |
| `07d9d8` | 31; 30 | priced partial: Lookup and indirect dispatch only. Outdoor 00882e child retains its own price; selected effect bodies remain open. |
| `07d798` | 31; 30 | priced: All 23 M16 word clears plus RTS, 914 clocks; independently executed ROM. |
| `07e245` | 29; 26 | open: Speed selection, direction resolution and movement continuation still need source-instruction pricing. Do not absorb the child movement/sand-drag deficit in a parent scalar. |
| `09b0b6` | 25; 19 | already-scoped; residual open: Slot/countdown/OAM allocation/RTS dispatch already priced. Source tail garnish handlers remain inside this frame; price missing handler bodies individually. |
| `06f7ef` | 18; 29 | priced: Negative-Z early exit and complete hitbox coordinate/extent path; actual operand branch, no callees. |
| `06f836` | 18; 29 | priced: Actual Y-then-X high-byte and extent exits; arithmetic prices include source path even when native sentinel guard rejects early. |
| `06e883` | 16; 26 | priced: Floor/indoor branch, actual indoor lookup or outdoor call wrapper, scratch/return. 00882e child excluded. |
| `06dc10` | 16; 23 | owned elsewhere: DC10 call instruction is already priced. DC13 draw instructions, E41E coordinate preparation and DC66 shadows have separate native scopes. Raw source tail ancestry is not another draw price. |
| `06e72f` | 13; 24 | priced partial: E72F..E73E prefix only. Shared E740/E773 property body remains open; its E883 child is separately priced. |
| `1e8b19` | 18; 4 | already-scoped; residual open: 290-clock RTS dispatch already charged, with 8B11 long wrapper separate. Source tail actor bodies remain inside this scope; their per-type bodies still need prices. |
| `02bb90` | 0; 32 | open: Camera per-axis/per-pixel branch and boundary-check work remain open. New price requires actual signed velocity, bounds and overlay branches. |
| `08a7ab` | 5; 2 | open: ancilla_draw_explosion and safe OAM publication remain unpriced; no parent aggregate charge added. |
| `00d61c` | 4; 2 | priced: High-plane expansion counts actual tile/row passes and pointer-wrap branch. E19B/DF4F conversion owner is separate. |
| `0dfa70` | 4; 1 | priced: ROM HUD template path: 330 MVN bytes and caller self work. FB91 owns existing child work; FA85 standalone entry is separately priced, never added to FA70. |
| `02f3b9` | 0; 6 | open: buffer_and_build_map16_stripes_x / write_overworld_scroll_stripe_step has no new price. ROM row/column control and copy work still need audited ownership and executed-ROM tests. |
| `00ee21` | 0; 1 | priced: All color comparisons, actual per-channel subtract/taken branches, stores and loop control; no callees. |

The native whole-route sample was rerun with the final binary. Every source
and native exclusive sum reconciles to its independent total; all negative
rows remain in the JSON rankings. Native DC13/E41E/DC66 drawing ownership
and existing sprite/garnish dispatchers are not repriced from parent gaps.
The next repeated unpriced targets include velocity and camera work;
striping, explosion drawing and individual actor/garnish/effect handlers
also remain open.

Final whole-route `07/00` ranking, exclusive clocks:

| Target | Source | Native | Difference | Source-lagged sample occurrences |
| --- | ---: | ---: | ---: | ---: |
| `06e72f` | 149,816 | 23,040 | +126,776 | 13 |
| `07e245` | 125,664 | 0 | +125,664 | 29 |
| `0080b5` | 101,164 | 7,680 | +93,484 | 32 |
| `09b0b6` | 251,386 | 169,818 | +81,568 | 25 |
| `1e8b19` | 110,082 | 28,950 | +81,132 | 18 |
| `07807f` | 77,530 | 0 | +77,530 | 31 |
| `08a7ab` | 68,608 | 0 | +68,608 | 5 |
| `0684e2` | 234,276 | 165,940 | +68,336 | 32 |
| `07d077` | 66,740 | 0 | +66,740 | 31 |
| `1dc222` | 73,850 | 15,220 | +58,630 | 13 |
| `0ed7fe` | 58,224 | 0 | +58,224 | 1 |
| `05a64a` | 57,772 | 0 | +57,772 | 1 |

Final whole-route `09/00` ranking, exclusive clocks:

| Target | Source | Native | Difference | Source-lagged sample occurrences |
| --- | ---: | ---: | ---: | ---: |
| `0684e2` | 315,646 | 151,158 | +164,488 | 32 |
| `06dc10` | 153,120 | 6,578 | +146,542 | 23 |
| `02f3b9` | 130,652 | 0 | +130,652 | 6 |
| `08a7ab` | 129,996 | 0 | +129,996 | 2 |
| `07e245` | 117,700 | 0 | +117,700 | 26 |
| `02bb90` | 107,316 | 0 | +107,316 | 32 |
| `06f228` | 83,296 | 0 | +83,296 | 5 |
| `06e72f` | 99,476 | 16,632 | +82,844 | 24 |
| `1d9f64` | 77,220 | 0 | +77,220 | 4 |
| `07807f` | 74,746 | 0 | +74,746 | 30 |
| `09c5fa` | 71,742 | 0 | +71,742 | 18 |
| `09c5bb` | 71,278 | 0 | +71,278 | 14 |

### Load categories: transfer owner and ordinary remainder

| Entry | Samples | Source total | Source SPC polling owner | Final native work | Signed ordinary difference |
| --- | ---: | ---: | ---: | ---: | ---: |
| `05/00` | 11 | 218,198,060 | 0 | 180,612,226 | +37,585,834 |
| `0E/07` | 32 | 206,744,352 | 0 | 178,864,202 | +27,880,150 |
| `0E/0A` | 32 | 172,351,410 | 0 | 144,056,772 | +28,294,638 |
| `09/2E` | 32 | 40,620,584 | 0 | 33,675,448 | +6,945,136 |
| `0E/05` | 32 | 15,015,754 | 0 | 6,376,370 | +8,639,384 |
| `08/00` | 32 | 644,830,344 | 196,437,512 | 446,928,578 | +1,464,254 |
| `06/00` | 32 | 805,621,926 | 170,801,370 | 615,017,632 | +19,802,924 |

The ordinary column is source total minus the `$008888` polling owner
minus native work. It is a signed remainder, not a wholly missing-price
sum: scope aliases, other owners and changed executed work can offset gaps.
`whole-upload-owners.json` retains every selected source host boundary and
separates bank-transfer intervals from non-upload wall/endpoint intervals.
The 08/00 sample has 566 transfer-owned hidden host intervals plus two
non-upload intervals; 06/00 has 485 transfer-owned intervals. These wall
intervals are not additional instruction charges or an equality claim with
native bank-1 `native_elapsed`.

No load-only price group was shipped. Leading ordinary rows remain:
05/00 default/manual graphics, 0E/07 and 0E/0A mode-7 character transfer,
09/2E red/green restoration, 0E/05 desert-iris builder/calculation, and
06/00 object/header/control work. Child-inclusive deficits must not be added
together. The CPU VRAM stores at `$00:E399` are ordinary instructions, not
SPC masked-upload work: `unshipped-mode7-price.json` derives 1,696,052 clocks
and matches every sampled source call in both map categories. This is an
unshipped price candidate, not a new runtime or census result.

### Full cartridge progression

| Stage | Held | Missing | Excess | Zero-lag FP |
| --- | ---: | ---: | ---: | ---: |
| Main / prior batch | 92,778 | 9,899 | 0 | 0 |
| Ordering-only control | 92,778 | 9,899 | 0 | 0 |
| Properties instruction group | 92,893 | 9,784 | 0 | 0 |
| Final gameplay group / command-position prefix | 93,078 | 9,599 | 0 | 0 |

Both final full gates PASS, complete=True, unexplained zero, with 1,473,123
iterations each. Every runtime hold equals P2 on every row. Cartridge
production excess and zero-lag FP remain zero. Live-SPC executes different
work and is reported separately. Intermediate ordering trials are retained
as controls; final receipts bind the actual command-position prefix capture
and final source/binary hashes.

| Mode | Held | Exact lagged | Within one | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 93,078 | 31,014 | 38,971 | 9,599 | 0 | 0 |
| live | 92,965 | 30,907 | 38,863 | 10,244 | 532 | 104 |

### Complete category table

Paired cells are cartridge / live.

| Entry | Source lag | Missing C / L | Excess C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 892 / 1,161 | 0 / 175 | 0 / 71 |
| `07/02` | 8,770 | 968 / 968 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 315 / 315 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 302 / 302 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 533 / 533 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 255 / 255 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 296 / 296 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 735 / 735 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,721 / 1,721 | 0 / 93 | 0 / 16 |
| `09/01` | 3,479 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 79 / 79 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 168 / 168 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 191 / 191 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,164 / 1,164 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 168 / 168 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 41 / 41 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 39 / 39 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 |
| `05/00` | 639 | 118 / 118 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 43 / 43 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 64 / 64 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 92 / 92 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 181 / 181 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 159 / 159 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 84 / 84 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 12 / 12 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 |
| `17/01` | 141 | 32 / 32 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 5 / 5 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 134 / 134 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 16 / 16 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 85 / 85 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 42 / 42 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 29 / 29 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 2 / 2 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 |

### Crystal / pendant phases, exceptions and rupee gaps

Cells are source / final presentation frames. Every properties phase is
within one of source. Overlay/build/iris/stabilize values retain the prior
batch, and all eight stable room/Link-XY/whole-save outcomes match source.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 66 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 85 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

| Mode | Classified WRAM differences | $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14735, "song-end-poll": 636}` | 10 |

The 162000–163000 probe has 0 $0CFD-only exceptions. The same 27 requests
retain gaps `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`. The unchanged PCM detector
retains its 23.002-frame onset gap and all measured intervals. These are CPU/PCM
diagnostics, not exact A/V acceptance. Preceding extra work can shift the
concatenated probe's absolute frame indices without changing the intervals.

### Verification and stopping point

| Check | Result |
| --- | --- |
| Default / reduced ordinary builds | PASS, both warning-free |
| Fresh main-default 200k state ledger versus final default | PASS, 200,000 records and EOF |
| Two control/group cartridge gates; both final full gates | PASS, complete=True, unexplained zero |
| Zelda lib, default | 2,483 pass; 1 failure; 3 ignored |
| Zelda lib, reduced | 2,018 pass; 0 failure; 2 ignored |
| SNES lib, both configurations | 504 pass, no failures, 7 ignored each |
| Pacing / profile regressions, both configurations | 13 / 3 pass |
| Executed-ROM price families, both configurations | 12 pass |
| ROM basic-block audit | 60 independently checked shipped blocks |
| Diagnostic join tests | 4 pass |
| Stable crystal/pendant outcomes | 8 source matches |
| `git diff --check` | PASS |

The sole default lib failure remains the known
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
Library cases form complete disjoint shard unions. Lib-test fixture warnings
are separate from warning-free ordinary builds.

This stopping point does not establish complete instruction coverage, an
exact unseeded route, exact raster scheduling or exact A/V. Requested
remaining gameplay bodies and load-only pricing are explicitly open above.
The supplied ledgers and oracle cache were read-only. No GPU, `--video`,
cached-A/V, external Snes9x, cargo GPU tests, `cargo fmt`, commit or push
was used. Scratch drivers remain in `/tmp`; failed initial baseline launches
and the out-of-domain synthetic direction fixture are superseded diagnostics,
not gate receipts. The first final validation is superseded after the prefix update-latch
issue was found. Corrected binaries and complete final checks were rebuilt
and rerun; `v1-superseded/` retains the earlier diagnostics. All final
receipts use the corrected frozen source. `verification.json`, artifact hashes, full predictor CSVs, rankings,
per-iteration profiles, source-cost listings, phase/PCM/stable receipts and
library manifest retain the evidence.

## Continuation on `codex/cost-pacing8` (main `5ab6afa6`)

Evidence is retained in `target/cost-pacing8/`. This is a verified partial
instruction-coverage stopping point. The mode-7 transfer is shipped, the
ordinary-load group is priced, and gameplay speed-selection/control prefixes
are added. Individual object wrappers, movement tails, camera/striping,
explosion drawing and actor/garnish/effect bodies remain open. No price reads
source lag counts or uses a scene duration, multiplier or fitted table.

### Source-derived work and ownership

| Work | ROM derivation and ownership |
| --- | --- |
| Mode-7 characters `$00:E399..E3D1` | `248 + 4096*408 + 4095*6 + 66 = 1,696,052` clocks. CPU VRAM stores, not DMA or SPC polling. All 32 sampled calls in 0E/07 and 0E/0A agree per call, including call counts and inclusive/exclusive work; `final-mode7-call-agreement.json` retains every row. Executed-ROM tests also compare transferred VRAM and prove no added default charge. |
| Default graphics `$00:E2D0..E33A` | 378,062 caller/raw-plane-copy clocks. Lagless replaces the generic E5AF conversion price with the actual E2D0 loop; the same native graphics publication runs ledger-muted. Default/ROM-timing keep their old conversion. Manual child bodies are excluded. |
| Manual graphics `$00:E33B..E36C` | `92 + 256*830 + 255*6 + 64 = 214,166` caller/copy clocks; actual E766 decompression and E843 byte reads are separate children. The formerly uncharged manual calls now charge their actual compressed streams. |
| Red/green restoration `$00:F0C7..F131` | 224 actual color comparisons/updates, taken branches, stores, loop control and countdown completion. No parent aggregate gap. |
| Desert iris `$07:EA27..EBD9`, `$ECDC..ED2B` | Builder follows actual edge tests, radius exhaustion, shape result, signed clamps, doubled/wrapped source table bounds, mirror stores, loop and opening/closing suffix. Calculation separately prices hardware division/multiplication instructions, lookup page crossings and opening branches. The builder charges the JSR only. Partial timing previews retain their existing ownership. |
| Dungeon animation `$00:D337..D393` | `522 + 256*518 + 255*6 + 158 = 134,818` caller/rotation clocks. E783 decompression and D5CE conversion remain children; retained decompression also gets its real 128-clock E783 prefix. |
| Dungeon palette caller `$02:C630..C68B` | 1,192 self clocks. Palette children and the existing C769 cache-copy tail remain separate. |
| Dungeon entrance `$02:D8B3..DB6D`, spawn `$DC55..DD87` | Actual game-over backup, follower/restart/spawn selection, progress/entrance/music branches, 64 seven-word record-copy passes and 32 fourteen-word clear passes. No called child bodies or aggregate fitted remainder. |
| Velocity `$07:E245..E372` | Actual blocked/swim/running/collision exits, water speed selection, diagonal speed, pit acceleration/deceleration, lookup page crossing, signed direction components and Z setup. Only BRL instructions into MovePosition, swim, sand-drag, dash and common-return tails are included; their bodies remain open. |
| Player control `$07:807F..8106` | Actual damage, cape, disable timer, boomerang, blink, health and handler-dispatch branches. SFX call instruction only; selected state-handler bodies remain separate. |

All 20 executed-ROM price-test families pass in both configurations. New
fixtures cover full/default/manual copies, both palette channels and countdown
wrap, iris clipping/opening/closing/radius exhaustion, entrance/spawn/progress
branches, 232 velocity-selection fixtures, and 145 damage/control fixtures.
`source-price-blocks.json` retains 220 decoded listing blocks (including
explicitly open tails); listings come from `scripts/rom_block_costs.py`.
These listing counts are not a claim that every listed block is shipped.
`price-routine-agreement.json` keeps all route-sample operand, scope and
ancestry disagreements; isolated source tests are not route-wide equivalence.

### Ordinary remainders and row resolution

| Entry | Before ordinary signed gap | Final ordinary signed gap |
| --- | ---: | ---: |
| `05/00` | 37,585,834 | 3,831,300 |
| `09/2E` | 6,945,136 | 333,968 |
| `0E/05` | 8,639,384 | 310,626 |
| `06/00` | 19,802,924 | 13,032,096 |
| `0E/07` | 27,880,150 | 743,318 |
| `0E/0A` | 28,294,638 | 1,157,806 |

These are signed source-total minus SPC-poll-owner minus native-work
remainders, not additive missing-price budgets. `load-resolutions.json`
resolves the leading retained rows and labels remaining bodies open. E766
and E843 were already scoped: the new work belongs to their missing manual
callers. Map decompression, interlace and stripe-copy children in 09/2E
already have instruction prices. C630 tail cache work is owned at C769.
The 893C object dispatcher and 873A room/header control are already scoped;
tail object wrappers must be priced individually. B2AF drawing reuses the
existing 9895 helper, so adding 516 per raw source call would double-charge
its existing 506-clock body; exact caller alias separation and the 10-clock
entry difference remain open. D619 high-expansion bodies are owned at D61C;
the 24-clock LDY wrapper remains open. Other object fill/door/adjacency,
sprite-cache/reset/load and custom-attribute rows remain open, not assumed
priced from their parents.

### Nine-category gameplay ordering

The ranking now includes 07/00, 09/00, 07/02, 07/07, 09/23, 0F/01,
07/0F, 09/16 and 09/0D. Each uses the retained 32 whole-route source-lagged
quantiles; the final binary is independently reprofiled on the identical
indices. Occurrences are exact within the samples. Occurrence/sample-size
times category lagged-iteration population is a prioritization heuristic,
not an unbiased full-route routine-frequency census. Every signed row,
including negatives, remains in `final-gameplay-weighted-rankings.json`
and `final-category-rankings.json`; all exclusive sums reconcile to their
independent source/native totals.

`pricing-order-main.json` recomputes the nine-category order before pricing;
`gameplay-resolutions.json` records ownership. Frequent 0684E2/1E8B19 actor
and 09B0B6 garnish dispatchers already own their dispatch work; tail bodies
stay open. E41A coordinate wrappers, DC13/E41E/DC66 drawing, DrawMultiple
and shared inactive guards retain their existing prices. F312 configure
work and its generic raster barrier are not an additive parent deficit.
The next clear repeated missing target was E245; its speed/direction prefix
is now priced, followed by 807F control self work. MovePosition, selected
state handlers, camera, striping, explosion and remaining individual
actor/garnish/effect bodies are still a worklist.

### Full cartridge progression

| Stage | Held | Missing | Excess | Zero-lag FP |
| --- | ---: | ---: | ---: | ---: |
| Main / prior batch | 93,078 | 9,599 | 0 | 0 |
| Mode-7 transfer | 93,353 | 9,324 | 0 | 0 |
| Ordinary graphics/palette/iris/animation callers | 93,729 | 8,948 | 0 | 0 |
| Entrance and gameplay prefixes | 93,771 | 8,906 | 0 | 0 |

Every group has a complete cartridge census with `--pacing-predictor`.
Both final full gates PASS, complete=True, unexplained zero, with 1,473,123
iterations each. Every runtime hold equals P2 on every row. Cartridge
excess and zero-lag FP remain zero. Live-SPC executes different work and
is reported separately.

| Mode | Held | Exact lagged | Within one | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 93,771 | 31,215 | 39,104 | 8,906 | 0 | 0 |
| live | 93,658 | 31,108 | 38,996 | 9,552 | 533 | 104 |

### Complete category table

Paired cells are cartridge / live.

| Entry | Source lag | Missing C / L | Excess C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 892 / 1,162 | 0 / 173 | 0 / 70 |
| `07/02` | 8,770 | 968 / 968 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 241 / 241 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 302 / 302 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 533 / 533 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 255 / 255 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 296 / 296 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 733 / 733 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,717 / 1,717 | 0 / 96 | 0 / 17 |
| `09/01` | 3,479 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 79 / 79 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 168 / 168 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 56 / 56 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,164 / 1,164 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 28 / 28 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 41 / 41 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 28 / 28 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 |
| `05/00` | 639 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 43 / 43 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 60 / 60 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 29 / 29 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 181 / 181 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 159 / 159 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 12 / 12 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 |
| `17/01` | 141 | 6 / 6 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 5 / 5 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 45 / 45 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 85 / 85 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 42 / 42 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 29 / 29 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 2 / 2 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |

### Crystal / pendant phases, exceptions and rupee gaps

Cells are source / final presentation frames. Every phase is within one;
all eight stable room/Link-XY/whole-save outcomes match source.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 66 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 85 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

| Mode | Classified WRAM differences | $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14733, "song-end-poll": 637}` | 10 |

The 162000–163000 probe retains zero $0CFD-only exceptions. The same 27
rupee requests retain gaps `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`.
The unchanged PCM detector retains its 23.002-frame onset gap and every
measured interval. Preceding new work can shift concatenated absolute
frame indices without changing these intervals. These remain CPU/PCM
diagnostics, not exact A/V acceptance.

### Verification and stopping point

| Check | Result |
| --- | --- |
| Default / reduced ordinary builds | PASS, both warning-free |
| Fresh main-default 200k ledger versus final default | PASS, 200,000 records and EOF |
| Three group cartridge gates; final live-SPC gate | PASS, complete=True, unexplained zero |
| Zelda lib, default | 2,491 pass; 1 failure; 3 ignored |
| Zelda lib, reduced | 2,026 pass; 0 failures; 2 ignored |
| SNES lib, both configurations | 504 pass, no failures, 7 ignored each |
| Pacing / profile regressions, both configurations | 13 / 3 pass |
| Executed-ROM price families, both configurations | 20 pass |
| Diagnostic join tests | 4 pass |
| Stable crystal/pendant outcomes | 8 source matches |
| `git diff --check` | PASS |

The sole default lib failure remains the known
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
The library manifest verifies complete disjoint shard unions. Fixture
warnings in lib-test builds are separate from warning-free ordinary builds.
The default baseline was freshly built and recorded from clean main
5ab6afa6 before edits; its binary/input/ledger hashes are retained.

The final joined sample adds 128,159,296 clocks over 4,450 iterations;
0 negative per-iteration deltas are retained in `sample-work-deltas.json`.
This is not complete instruction coverage, exact unseeded-route behavior,
raster scheduling or exact A/V acceptance. Open rows above remain open.
The supplied ledgers and oracle cache stayed read-only. No GPU, `--video`,
cached-A/V, external Snes9x, cargo GPU tests, `cargo fmt`, commit or push
was used. Scratch drivers remain in `/tmp`. `verification.json`, artifact
hashes, predictor CSVs, per-call agreement, source listings, rankings,
phase/PCM/stable receipts and the library manifest retain the evidence.

## Continuation on `codex/cost-pacing9` (main `a5e7cb2c`)

Evidence is retained in `target/cost-pacing9/`. This is a verified partial stopping point after movement, selected player-handler setup and camera instruction work. Striping, explosion, individual actor/garnish/effect tails, the 06/00 ordinary remainder and the remaining load groups are still open. No price reads source lag counts or uses a fitted scene threshold, multiplier or duration table.

### Source-derived work and ownership

| Work | ROM derivation and ownership |
| --- | --- |
| MovePosition `$07:E373..E3DB` | Coordinate save, handler/platform branches, two or three actual signed axis passes, loop branches and the two child JSRs. Moving floor, conveyor and sand drag are separate bodies. The Somaria exit bypasses integration. |
| Sand drag / return `$07:E3E0..E405`, `$E402..E405` | 546 drag/delta/return clocks, or only the 94-clock common return on the all-blocked collision exit. The velocity-selection prefix owns its BRL, never these bodies. |
| Swim `$07:E42A..E4D0` | Two real stroke-timer decrements, acceleration-mode/table selection, signed sum/clamp/direction branches, drag and loop control. Subpixel integration remains a separate tail. |
| Subpixel integration `$07:E4D3..E53E` | Actual signed component branches and both axis passes, header-collision branch, floor JSR and sand-drag BRL. No floor or sand-drag body in this price. |
| Dash in place `$07:E545..E592` | Joypad/water speed selection, actual direction-bit scan, sign/axis setup, early return or BRL to subpixel integration. Lagless now executes this ROM branch when the dash counter is below the ROM immediate 16; the default path retains its prior behavior. |
| Moving floor / conveyor `$07:E595..E670` | Actual guard exits, floor-direction branches and shared 346-clock E5CD body; conveyor predicates and its two signed axis passes. E595 excludes the separately priced shared tail. Invalid native belt-table indices remain unsupported, with no invented ROM-body price. |
| Default handler `$07:8109..812E` | Camera/bunny call instructions and the actual post-bunny branch to ground, auxiliary handling, bunny or RTS. The unported debug-cheat toggle at 8112..8117 remains unpriced. Ground/bunny bodies and BunnyTransformation remain separate open work. |
| Auxiliary handler `$07:8130..819F` | Scratch/property clearing, actual electrical/cape/indoors branches and child call/BRL instructions. Swimming/cape/sword reset, SFX and recoil/zap bodies are excluded. |
| Camera cache `$01:FF28..FFB5`, wrapper `$07:F514..F51C` | 1,776 fixed cache-copy clocks plus the actual indoor/outdoor wrapper branch, optional JSL and RTS. |
| Indoor camera caller `$07:E8F0..E900`, page deltas `$E9D3..EA05` | Caller branch/JSL/JML and return; 244 page-delta prefix clocks, then actual X/Y sign branches and quadrant child calls. Quadrant/door bodies are excluded. The page-delta suffix reuses the active routine scope. |
| Overworld camera `$02:BB90..BD61`, boundary `$BD62..BDBF` | Actual Z/velocity setup, one instruction-loop pass per real pixel of movement, changing camera/room-bound comparisons, optional boundary calls, scroll fractions/sign/clamps, overlay branches and special-room stores. Boundary counter rollover and equal-bound exits have separate ownership. Literal overlay operands and clamp values come directly from these ROM instructions. |

The retained listings and `source-price-instructions.json` come from `scripts/rom_block_costs.py`; a listing does not mean every reached block is priced. `open-next-work.json` and the open-stripes/open-explosion listings retain the next unshipped audit targets. Executed-ROM tests cover the movement/tail chains, signed axes, platform and floor/conveyor guards, swim timers/modes/clamps, all dash directions, selected handler branches, camera loop/boundary/parallax/clamp paths and camera callers. All 24 executed-ROM price families pass in both configurations. Source tests prove their exercised prices; they do not establish route-wide semantic or raster equivalence. `price-routine-agreement.json` retains all sampled scope/call/operand disagreements.

### Full cartridge progression

| Stage | Held | Missing | Excess | Zero-lag FP |
| --- | ---: | ---: | ---: | ---: |
| Main / prior batch | 93,771 | 8,906 | 0 | 0 |
| Movement and velocity tails | 93,789 | 8,888 | 0 | 0 |
| Selected handler setup | 93,789 | 8,888 | 0 | 0 |
| Camera | 93,793 | 8,884 | 0 | 0 |

Every group has a complete cartridge census with `--pacing-predictor`. Both final full gates PASS, complete=True, unexplained zero, with 1,473,123 iterations each. Every runtime hold equals P2 on every row. Cartridge excess and zero-lag FP remain zero. Live-SPC executes different work and is reported separately.

| Mode | Held | Exact lagged | Within one | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 93,793 | 31,236 | 39,104 | 8,884 | 0 | 0 |
| live | 93,680 | 31,129 | 38,996 | 9,529 | 532 | 104 |

### Refreshed rankings and remaining work

`movement-`, `handlers-` and `final-gameplay-weighted-rankings.json` refresh the nine-category occurrence-weighted ordering after each group. Each uses the same retained whole-route source-lagged quantiles; sample occurrences are exact, while population weighting is only a prioritization heuristic. All signed rows and independent total reconciliations remain in the category rankings and coverage receipts. Dispatch parents 0684E2, 1E8B19 and 09B0B6 remain separate from their open individual tail bodies.

| Ordinary category | Prior signed gap | Final signed gap |
| --- | ---: | ---: |
| `06/00` | 13,032,096 | 12,975,264 |
| `05/00` | 3,831,300 | 3,818,868 |
| `0E/0A` | 1,157,806 | 1,157,806 |
| `0E/07` | 743,318 | 743,318 |

These signed source-minus-SPC-owner-minus-native totals are accounting remainders, not additive budgets. This batch does not price the open object fill/door/adjacency wrappers, sprite cache/reset/load or custom attributes. D619 still needs its 24-clock LDY wrapper; B2AF still needs exact alias separation and its 10-clock entry difference without charging the existing 9895 body twice. Camera door/quadrant wrappers and other player-state bodies are also incomplete. The next requested work is striping, explosion and individual actor/garnish/effect tails, followed by the ordinary/load remainders.

### Complete category table

Paired cells are cartridge / live.

| Entry | Source lag | Missing C / L | Excess C / L | FP C / L |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 892 / 1,161 | 0 / 175 | 0 / 71 |
| `07/02` | 8,770 | 968 / 968 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 240 / 240 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 299 / 299 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 533 / 533 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 255 / 255 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 296 / 296 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 728 / 728 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,706 / 1,706 | 0 / 93 | 0 / 16 |
| `09/01` | 3,479 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 79 / 79 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 168 / 168 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 56 / 56 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,163 / 1,163 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 28 / 28 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 41 / 41 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 28 / 28 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 |
| `05/00` | 639 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 43 / 43 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 59 / 59 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 29 / 29 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 20 / 20 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 181 / 181 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 159 / 159 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 12 / 12 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 |
| `17/01` | 141 | 6 / 6 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 5 / 5 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 45 / 45 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 85 / 85 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 42 / 42 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 29 / 29 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 2 / 2 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |

### Crystal / pendant phases, exceptions and rupee gaps

Cells are source / final presentation frames. Every phase is within one; all eight stable room/Link-XY/whole-save outcomes match source.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 66 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 85 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

| Mode | Classified WRAM differences | $0CFD-only exceptions |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14731, "song-end-poll": 636}` | 10 |

The 162000–163000 probe retains zero $0CFD-only exceptions. The same 27 rupee requests retain gaps `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`. Every measured PCM onset interval from the unchanged detector is also unchanged from cost-pacing8. Absolute concatenated PCM frame positions can shift with preceding work. These are CPU/PCM diagnostics, not exact A/V acceptance.

### Verification and stopping point

| Check | Result |
| --- | --- |
| Default / reduced ordinary builds | PASS, both warning-free |
| Fresh main-default 200k ledger versus final default | PASS, 200,000 records and EOF |
| Three cartridge gates; final live-SPC gate | PASS, complete=True, unexplained zero |
| Zelda lib, default | 2,495 pass; 1 failure; 3 ignored |
| Zelda lib, reduced | 2,030 pass; 0 failures; 2 ignored |
| SNES lib, both configurations | 504 pass, zero failures, 7 ignored each |
| Pacing / profile regressions, both configurations | 13 / 3 pass |
| Executed-ROM price families, both configurations | 24 pass |
| Diagnostic join tests | 4 pass |
| Stable crystal/pendant outcomes | 8 source matches |
| `git diff --check` | PASS |

The sole default lib failure remains `cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`. The library manifest verifies complete disjoint shard unions. Fixture warnings in lib-test builds are separate from warning-free ordinary builds. The default baseline was freshly recorded from the clean main a5e7cb2c binary built before edits; binary/input/ledger hashes are retained.

The final joined sample adds 8,156,704 clocks over 4,450 iterations; 0 negative per-iteration deltas are retained in `sample-work-deltas.json`. This remains partial instruction coverage, not exact unseeded-route behavior, raster scheduling or exact A/V acceptance. All new charges and the dash branch are lagless-only; default/ROM-timing ledger paths retain their prior prices. The supplied ledgers and oracle cache stayed read-only. No GPU, `--video`, cached-A/V, external Snes9x, cargo GPU tests, `cargo fmt`, commit or push was used. Scratch drivers remain in `/tmp`. `verification.json`, artifact hashes, predictor CSVs, source listings, rankings, phase/PCM/stable receipts and the library manifest retain the evidence.

## Continuation on `codex/cost-pacing10` (main `9fccf9c9`)

Evidence is retained in `target/cost-pacing10/`; `complete/` binds the final binaries, full censuses, probes and library manifest. This is a verified partial CPU-only stopping point. Dynamic HDMA placement, remaining object/actor bodies and complete raster ownership remain open. No charge is fitted to a hold count, and no scene threshold, multiplier or duration table was introduced.

### Hold-targeted shortfalls

The fresh diagnostic census reproduces **every prior predictor column on all 1,473,123 iterations**. The new diagnostic inverse freezes the completed call's worker share, ownership checkpoints and raster wait, then binary-searches the first extra main clock appended after the existing work that reaches `lag_nmis * 357368` in P2. The source count is supplied by the replay CLI after main returns; it is never a runtime scheduling input. These are P2 counterfactuals, not measured missing instructions or additive prices. Equality is accepted; branch tests check the returned clock and the immediately preceding clock.

Near-miss means strictly less than 5% of a field; moderate is 5% to below 50%; large is at least 50%. These bands are diagnostic only.

| Band | Iterations | Missing holds | Total shortfall clocks |
| --- | ---: | ---: | ---: |
| near-miss | 2,575 | 2,575 | 19,545,507 |
| moderate | 4,336 | 4,336 | 396,964,918 |
| large | 1,428 | 1,973 | 480,730,934 |
| Total | 8,339 | 8,884 | 897,241,359 |

| Category | Near | Moderate | Large | Missing holds | Shortfall clocks |
| --- | ---: | ---: | ---: | ---: | ---: |
| `07/00` | 35 | 1,671 | 0 | 1,706 | 176,553,976 |
| `07/02` | 290 | 128 | 341 | 968 | 147,351,470 |
| `09/00` | 6 | 1,157 | 0 | 1,163 | 105,829,158 |
| `09/03` | 0 | 52 | 199 | 296 | 64,691,734 |
| `0F/01` | 0 | 0 | 255 | 255 | 62,052,126 |
| `06/00` | 0 | 16 | 126 | 240 | 56,230,736 |
| `07/0E` | 37 | 92 | 109 | 299 | 48,403,242 |
| `07/0F` | 2 | 175 | 15 | 192 | 29,352,760 |
| `09/23` | 94 | 422 | 17 | 533 | 29,337,356 |
| `07/07` | 559 | 104 | 43 | 728 | 22,277,878 |
| `0E/03` | 0 | 22 | 57 | 79 | 15,587,832 |
| `01/00` | 0 | 0 | 4 | 48 | 15,537,948 |
| `0F/00` | 0 | 0 | 55 | 55 | 13,143,612 |
| `12/03` | 0 | 0 | 43 | 43 | 10,426,822 |
| `07/15` | 91 | 49 | 15 | 168 | 10,294,444 |
| `0E/0A` | 9 | 11 | 25 | 56 | 9,931,928 |
| `0E/02` | 758 | 134 | 0 | 892 | 9,704,106 |
| `07/03` | 0 | 41 | 12 | 53 | 8,757,617 |
| `11/00` | 0 | 1 | 16 | 28 | 6,750,914 |
| `0E/07` | 0 | 17 | 11 | 28 | 5,110,444 |
| `07/06` | 5 | 26 | 8 | 41 | 5,071,006 |
| `05/00` | 0 | 2 | 9 | 20 | 5,010,748 |
| `09/11` | 0 | 2 | 14 | 20 | 4,558,616 |
| `08/00` | 0 | 9 | 10 | 19 | 3,147,723 |
| `09/01` | 6 | 47 | 0 | 53 | 3,075,450 |
| `07/12` | 46 | 2 | 6 | 59 | 2,668,876 |
| `00/01` | 0 | 0 | 7 | 7 | 2,232,590 |
| `01/02` | 0 | 0 | 7 | 7 | 2,214,698 |
| `09/16` | 180 | 1 | 0 | 181 | 1,635,944 |
| `09/0D` | 159 | 0 | 0 | 159 | 1,498,090 |
| `1B/02` | 0 | 0 | 2 | 5 | 1,250,214 |
| `09/2E` | 17 | 10 | 2 | 29 | 1,202,768 |
| `0B/24` | 56 | 29 | 0 | 85 | 1,174,196 |
| `17/01` | 0 | 1 | 2 | 6 | 1,167,576 |
| `0B/00` | 0 | 9 | 0 | 9 | 1,014,788 |
| `0E/05` | 26 | 19 | 0 | 45 | 796,320 |
| `09/2C` | 4 | 7 | 1 | 12 | 781,852 |
| `09/20` | 1 | 17 | 0 | 18 | 771,478 |
| `10/01` | 0 | 0 | 3 | 3 | 748,044 |
| `1A/00` | 0 | 0 | 1 | 2 | 645,146 |
| `04/01` | 0 | 0 | 2 | 2 | 632,078 |
| `1A/02` | 0 | 0 | 1 | 2 | 600,530 |
| `0B/2B` | 27 | 15 | 0 | 42 | 594,574 |
| `0B/17` | 57 | 2 | 0 | 59 | 556,638 |
| `0B/29` | 39 | 6 | 0 | 45 | 545,918 |
| `1A/14` | 0 | 0 | 1 | 2 | 529,562 |
| `1A/20` | 0 | 0 | 1 | 2 | 469,638 |
| `1A/16` | 0 | 0 | 1 | 2 | 426,194 |
| `15/03` | 0 | 4 | 0 | 4 | 349,142 |
| `09/0E` | 0 | 6 | 0 | 6 | 320,250 |
| `03/01` | 0 | 0 | 1 | 1 | 316,454 |
| `0B/28` | 0 | 3 | 0 | 3 | 312,272 |
| `09/0F` | 0 | 2 | 0 | 2 | 305,168 |
| `19/00` | 0 | 0 | 1 | 1 | 283,888 |
| `0B/1C` | 29 | 0 | 0 | 29 | 241,318 |
| `07/13` | 0 | 0 | 1 | 1 | 214,256 |
| `1A/1C` | 0 | 0 | 1 | 1 | 207,228 |
| `07/0B` | 0 | 0 | 1 | 1 | 200,586 |
| `1A/12` | 0 | 0 | 1 | 1 | 193,962 |
| `0B/19` | 1 | 3 | 0 | 4 | 193,390 |
| `0E/01` | 0 | 0 | 1 | 1 | 182,614 |
| `00/07` | 8 | 4 | 0 | 12 | 173,020 |
| `0B/26` | 0 | 1 | 0 | 1 | 162,922 |
| `00/00` | 0 | 1 | 0 | 1 | 126,240 |
| `0B/1A` | 0 | 2 | 0 | 2 | 122,844 |
| `07/18` | 1 | 5 | 0 | 6 | 119,290 |
| `1A/04` | 0 | 1 | 0 | 1 | 108,994 |
| `1A/18` | 0 | 1 | 0 | 1 | 108,994 |
| `1A/08` | 0 | 1 | 0 | 1 | 105,060 |
| `1A/1A` | 0 | 1 | 0 | 1 | 101,122 |
| `09/06` | 0 | 1 | 0 | 1 | 98,714 |
| `12/05` | 19 | 0 | 0 | 19 | 97,360 |
| `09/04` | 0 | 1 | 0 | 1 | 96,570 |
| `07/01` | 5 | 1 | 0 | 6 | 55,914 |
| `0B/25` | 0 | 1 | 0 | 1 | 32,086 |
| `0E/0B` | 3 | 1 | 0 | 4 | 31,347 |
| `00/06` | 2 | 0 | 0 | 2 | 13,242 |
| `19/01` | 1 | 0 | 0 | 1 | 11,790 |
| `08/02` | 2 | 0 | 0 | 2 | 7,964 |

### Missing-hold-only routine ranking

The new reset-to-route source profile and paired native profile cover **all 8,339 original missing-hold iterations**, including all 1,428 large cases, rather than a sampled or population-weighted ranking. No lag-exact iteration contributes. `before-targeted-ranking.json` and `final-targeted-ranking.json` retain all signed exclusive rows, independent total reconciliations and per-iteration totals. The latter ranks only the remaining missing population; `all-original-missing-final-ranking.json` keeps the fixed original population for before/after comparison. Source profiles agree exactly on the overlap with the prior retained source profiles (`source-overlap-agreement.json`). Inclusive deficits overlap and must not be summed.

| Routine | Before exclusive deficit | Resolution |
| --- | ---: | --- |
| `008888` | 1,058,327,660 | (c) SPC receiver / masked song-transfer polling; not accepted-main pricing. |
| `0ec9fd` | 411,707,458 | (c) Existing separate VWF work counter; no duplicate dispatch/glyph price. |
| `0ecb5e` | 284,613,300 | (c) Existing VWF work counter. |
| `01893c` | 182,806,612 | (b/a) Dispatcher already priced; source JMP tails remain in this frame. Individually resolve object bodies, never add this whole gap. |
| `02febb` | 179,646,312 | (b) Existing bank-02 decompressor, currently unscoped in ordinary native work. |
| `00f312` | 132,725,700 | (c/a) Includes V-counter polling already represented by the raster barrier; other iris work is separately priced. |
| `02fd87` | 116,661,600 | (b) Existing map8-row buffer prices. |
| `02ff5f` | 107,796,880 | (b) Existing bank-02 byte-reader prices; native scope mismatch. |
| `02f595` | 105,675,060 | (b) Existing quadrant setup/caller prices. |
| `02fdf7` | 61,649,280 | (b) Existing map8-row conversion prices. |
| `02f679` | 50,889,720 | (b) Existing low-byte interlace prices. |
| `02f638` | 50,872,800 | (b) Existing high-byte interlace prices. |
| `01b2af` | 50,388,948 | (b/a) Existing 9895 body alias; only unresolved wrapper differences are admissible. |
| `09c452` | 38,292,790 | Open / partial ownership; a signed row alone is not a new price. |
| `02f54a` | 35,540,132 | (b) Existing quadrant-loop prices; not a fresh additive budget. |
| `00e78b` | 33,737,550 | Open / partial ownership; a signed row alone is not a new price. |
| `02f3b9` | 25,443,804 | (a) X stripes and initial-scroll cursor now priced from their real ROM loops. |
| `02f482` | 24,557,432 | (a) Y stripes and initial-scroll cursor now priced from their real ROM loops. |
| `0684e2` | 23,971,046 | Open / partial ownership; a signed row alone is not a new price. |
| `09c5bb` | 16,397,728 | Open / partial ownership; a signed row alone is not a new price. |
| `06dc10` | 15,763,330 | Open / partial ownership; a signed row alone is not a new price. |
| `06e72f` | 15,252,298 | Open / partial ownership; a signed row alone is not a new price. |
| `09c176` | 13,641,836 | Open / partial ownership; a signed row alone is not a new price. |
| `02f435` | 13,593,184 | (a) First X half is a child; second falls through in its parent. |
| `02f50a` | 13,590,688 | (a) First Y half is a child; second falls through in its parent. |
| `1e8b19` | 10,708,926 | Open / partial ownership; a signed row alone is not a new price. |
| `018916` | 10,228,270 | Open / partial ownership; a signed row alone is not a new price. |
| `0ae5bc` | 9,681,760 | Open / partial ownership; a signed row alone is not a new price. |
| `09b0b6` | 9,649,618 | Open / partial ownership; a signed row alone is not a new price. |
| `02f7cb` | 9,477,888 | Open / partial ownership; a signed row alone is not a new price. |
| `09c327` | 8,555,302 | Open / partial ownership; a signed row alone is not a new price. |
| `01b759` | 8,384,020 | Open / partial ownership; a signed row alone is not a new price. |
| `06e4ab` | 7,213,196 | Open / partial ownership; a signed row alone is not a new price. |
| `0db871` | 7,053,240 | Open / partial ownership; a signed row alone is not a new price. |
| `02b1ff` | 6,876,394 | Open / partial ownership; a signed row alone is not a new price. |
| `07d077` | 6,583,540 | Open / partial ownership; a signed row alone is not a new price. |
| `01fe41` | 6,438,816 | Open / partial ownership; a signed row alone is not a new price. |
| `06e73c` | 6,419,584 | Open / partial ownership; a signed row alone is not a new price. |
| `09c22f` | 6,271,758 | Open / partial ownership; a signed row alone is not a new price. |
| `07807f` | 6,194,372 | Open / partial ownership; a signed row alone is not a new price. |

### Structural timing audit

`structural-profile.csv` records physical main-step wall time, CPU transaction clocks, main-thread DMA, refresh events and V-counter reads. Interrupt CPU and polygon-stack work remain excluded. Refresh during the instruction that accepts an interrupt can belong to its entry: `interrupt_refresh_edges` retains that ambiguity. The non-CPU/non-DMA/non-refresh residual bounds HDMA between `max(0, residual)` and `residual + 40 * edges`; signed residuals are retained. Nominal ledger host-interval widths, signed host-width residuals and IRQ counts are joined in the targeted reports. Host widths are envelopes (the short physical scanline and endpoint phases remain distinct), not another charge.

| Structural component on original missing iterations | Source | Final native |
| --- | ---: | ---: |
| Main DMA clocks | 38,913,600 | 38,913,600 |
| Main refresh clocks | 483,427,480 | 416,939,200 |
| HDMA clock bounds | 299,243,570 .. 299,243,850 | No separate general P2 term |
| V-counter reads / raster-wait clocks | 746,198 reads | 250,268 clocks |

Main DMA was systematically omitted: the old NMI accounting helper was a no-op outside NMI, and the direct main tilemap uploader bypassed even that helper. Lagless main now retains bus stalls separately from the instruction-only ledger, including ownership checkpoints, sound and song-upload prefix calculations. One `$420B` write owns 18 run clocks, 8 per selected channel and 8 per actual byte; grouped transfers pay run setup once. Muted preview work is excluded. Neither held callbacks nor default/ROM-timing paths repeat the charge.

Reset-proven controls at iterations 25 and 730 each measure **163,944** DMA clocks (tilemap erase). Iteration 1915 measures **295,224**: the same erase plus eight 2,048-byte uploads at `$00:8CD8`, each `26 + 2048*8 = 16,410`. In all three controls, main wall = CPU instructions + measured refresh + DMA exactly, with zero HDMA residual. The final native primitives cover those transfer counts. `source-dma-sites.log` retains the issuing PCs and raster positions; NMI DMA is excluded by stack/depth ownership in the structural profile.

P2 still approximates refresh as `40 * ceil((CPU + main_DMA)/1324)`. The report separately compares actual refresh against this formula evaluated on **source** work, avoiding a false rounding diagnosis caused merely by unpriced native instructions. Exact per-event placement across interrupt/worker gaps remains open. The separate lag-NMI control at host 2217 measures 2,922 CPU clocks and 3,002 body wall clocks, including 80 refresh clocks. Adding the 62-clock native interrupt entry gives 3,064 physical clocks against the retained native lag-handler budget of 3,112. The native budget already exceeds that control by 48 clocks; an unconditional extra handler-refresh charge would overcount it. The generic `$00:F3A3..F3B2` V-counter barrier already owns omitted wait passes; the final CSV exposes its wall charge, so those source poll instructions are not priced again as an instruction deficit.

The measured HDMA residual is retained as an open structural ownership issue. A flat per-field or per-line addition was not shipped: dynamic table updates, enable changes, worker intervals and existing VWF/timed owners need non-overlapping event placement. No residual or shortfall was used as an HDMA price. This batch does not claim that structural audit complete.

### ROM-derived additions and progression

| Work | Instruction/bus derivation |
| --- | --- |
| X/Y stripe builders | `$02:F3B9..F481` / `$F482..F549`: 16 real two-word buffer passes, actual quadrant branch, 16 four-word conversions in each half, taken loop branches and returns. First half has its own call scope; the fall-through second half stays in the parent. |
| Four stripe offset callers | `$02:F2F1..F310`, `$F325..F344`, `$F363..F37E`, `$F39D..F3B8`: headers, child calls, actual offset updates and return, excluding child bodies. |
| Initial scroll | `$02:F031..F1FB` and `$EF7A..EFE7`: real size/direction branches, Big-South subtraction passes derived from its input, actual 7/8 stripe loop, source offsets, terminator and caller suffix. Retained slices charge each completed stripe once and reuse the same builder prices. |
| Main tilemap uploader | `$00:90E3..90E6` wrapper (90) plus `$8CB0..8CE3` body (568) for supported pages. `$8CD8` owns one 2,048-byte DMA separately. |
| Main DMA | Hardware-derived run/channel/byte costs at the existing transfer primitives; erase `$00:8374/$838E/$83B0/$83CB` and main uploader `$00:8CD8`. No instruction-ledger DMA price. |

Listings are retained from `scripts/rom_block_costs.py` with actual M/X widths. Executed-ROM tests cover both stripe quadrant branches, directions and buffer rotations; all small/big initial-scroll directions, actual subtraction loops and single-step native slices; all 35 supported uploader pages; and all three tilemap erase entries. CPU and DMA are checked independently.

| Cartridge stage | Held | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: |
| Main / diagnostic baseline | 93,793 | 8,884 | 0 | 0 |
| dma-rom | 93,831 | 8,846 | 0 | 0 |
| stripes-rom | 93,831 | 8,846 | 0 | 0 |
| scroll-rom | 94,000 | 8,677 | 0 | 0 |
| final-rom | 94,100 | 8,577 | 0 | 0 |

All four cartridge groups have complete predictor censuses and gate receipts. Final cartridge and live-SPC gates PASS, complete=True, unexplained zero, 1,473,123 iterations each. Every runtime hold equals P2. Live-SPC executes different work. Its excess/FP increased from 532/104 to 587/131; cartridge excess/FP remain zero. These modes are reported separately.

| Mode | Held | Exact lagged | Within one | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 94,100 | 31,415 | 39,215 | 8,577 | 0 | 0 |
| live | 93,993 | 31,284 | 39,083 | 9,271 | 587 | 131 |

### Complete category table

Cells are cartridge / live.

| Category | Source lag | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 892 / 1,210 | 0 / 223 | 0 / 96 |
| `07/02` | 8,770 | 968 / 968 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 170 / 170 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 299 / 299 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 533 / 533 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 255 / 255 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 137 / 137 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 728 / 728 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,706 / 1,706 | 0 / 100 | 0 / 18 |
| `09/01` | 3,479 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 168 / 168 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 56 / 56 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,163 / 1,163 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 28 / 28 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 41 / 41 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 22 / 22 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 6 / 6 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 |
| `05/00` | 639 | 12 / 12 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 43 / 43 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 59 / 59 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 29 / 29 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 10 / 10 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 181 / 181 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 159 / 159 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 12 / 12 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 3 / 3 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 45 / 45 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 85 / 85 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 6 / 6 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 59 / 59 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 3 / 3 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 42 / 42 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 4 / 4 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 45 / 45 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 29 / 29 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 19 / 19 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |

### Phase, exception and rupee verification

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 66 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 85 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

Every source / final phase is within one; all eight stable room/Link-XY/whole-save outcomes match.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14735, "song-end-poll": 638}` | 10 |

The 162000–163000 probe has zero $0CFD-only exceptions. The same 27 requests retain gaps `[23, 8, 8, 8, 8, 10, 24, 15, 8, 8, 8, 8, 8, 14, 27, 12, 8, 8, 8, 8, 8, 14, 11, 8, 8, 8]`; every PCM onset interval from the unchanged detector is unchanged. Absolute concatenated PCM positions can move with preceding work. These are CPU/PCM diagnostics, not exact A/V acceptance.

### Verification

| Check | Result |
| --- | --- |
| Default / reduced ordinary builds | PASS, warning-free |
| Fresh main-default 200k state ledger | PASS, all records and EOF exact |
| Four cartridge gates and final live gate | PASS, complete and unexplained zero |
| Zelda lib, default | 2,500 pass; 1 failures; 3 ignored |
| Zelda lib, reduced | 2,035 pass; 0 failures; 2 ignored |
| SNES lib, both configurations | 504 pass, zero failures, 7 ignored each |
| Pacing / profile tests, both configurations | 14 / 3 pass |
| Executed-ROM price families, both configurations | 27 pass |
| Python shortfall and profile-join tests | PASS |
| `git diff --check` | PASS |

The sole default lib failure remains `cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`. The final library manifest verifies complete disjoint shard unions. Earlier intermediate-library runs were superseded; only `complete/library-shards.json` is final library evidence. Lib-test fixture warnings are separate from the warning-free ordinary builds. The baseline binary and 200k ledger were freshly built/recorded from clean main `9fccf9c9` before edits; provenance and hashes are retained.

Reproduce the diagnostic join with:

```sh
python3 scripts/pacing_shortfalls.py target/cost-pacing10/before-rom.csv \
  --source target/cost-pacing10/source-missing/routine-profile.csv \
  --native target/cost-pacing10/before-native-profile.csv \
  --structural target/cost-pacing10/source-missing/structural-profile.csv \
  --ledger /Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2/iterations.jsonl \
  --json /tmp/hold-targeted-report.json
```

This remains partial instruction coverage and approximate pacing, not unseeded-route or exact A/V proof. Remaining object JMP tails, actor bodies, sprite cache/load work and dynamic bus/raster ownership remain open. All new prices are lagless-only. The supplied cache/ledgers stayed read-only; no GPU, `--video`, cached-A/V, external Snes9x, cargo GPU tests, `cargo fmt`, commit or push was used. Scratch drivers stayed in `/tmp`; the new repository script is a reusable diagnostic with tests.

### Stable immutable HDMA follow-up (final verified group)

`hdma/` now contains the final binaries, full censuses, source hashes, library manifest and verification receipts. `complete/` above preserves the preceding verified group. The original shortfall distribution and complete source profiles remain unchanged; `hdma/final-targeted-ranking.json` re-ranks only the new remaining missing population.

The structural audit justified a general HDMA bus term. A completed main call may lease an unchanged, immutable ROM descriptor program. The existing hardware state machine derives each line's init, transfer, reload and termination stalls from $420C and $43x0–$43xA. No payload, category, source hold count or measured duration prices the term. Programs are cached by register signature. A write epoch rejects even a mask changed and restored within main; immutable descriptors avoid untracked WRAM table mutations.

The periodic event calculation excludes NMI/worker-owned intervals and includes refresh induced by main HDMA. It closes elapsed wall time monotonically, with a hardware capacity check. Unknown/mutable descriptors, program changes, main DMA, VWF, raster barriers, worker ownership changes and scheduled timed work decline this extra term to preserve their existing timing owners. The charge is lagless-only and applied after main returns; mid-call timed producers retain their existing scheduling. P2 still uses nominal fields and phase-agnostic rounded refresh, so this is approximate pacing rather than exact CPU/raster continuation.

The paired native instruction profiles on the original missing population are byte-for-byte identical before and after HDMA (`hdma/instruction-profile-agreement.json`); this group changes only bus timing.

Executed-ROM tests run $00:8034's real LDA/BEQ loop with spotlight and map near/far descriptors over short and multi-field spans. They compare real CPU, refresh and HDMA bus events, allowing only the retained endpoint uncertainty from rounded refresh/short scanlines. Tests also cover worker-event exclusion, immutable payload writes, mutable descriptors and changed/restored enable masks. No test derives prices from source lag counts.

| Group | Cartridge held | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: |
| Previous verified uploader/scroll group | 94,100 | 8,577 | 0 | 0 |
| Stable immutable HDMA | 95,739 | 6,938 | 0 | 0 |

Both full gates PASS, complete=True, unexplained zero, 1,473,123 iterations each. Every runtime hold equals P2. Live-SPC executes different work: excess/FP increased from 587/131 in the preceding group to 596/133; cartridge excess/FP remain zero. The complete table follows.

| Mode | Held | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: |
| rom | 95,739 | 6,938 | 0 | 0 |
| live | 95,632 | 7,641 | 596 | 133 |

| Structural clocks on all original missing iterations | Source | Final native |
| --- | ---: | ---: |
| Main DMA | 38,913,600 | 38,913,600 |
| Main refresh | 483,427,480 | 423,113,280 |
| HDMA | 299,243,570 .. 299,243,850 | 204,333,554 |

5,924 of the original 8,339 missing iterations admit the stable-program lease. The source column includes all original missing iterations and their longer source instruction paths; unsupported/mixed native owners remain uncharged by this new term. Per-iteration values and category sums are in `hdma/structural-original-missing.json`.

| Category | Source lag | Missing (cartridge / live) | Excess (cartridge / live) | FP (cartridge / live) |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 871 / 1,198 | 0 / 224 | 0 / 96 |
| `07/02` | 8,770 | 632 / 632 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 170 / 170 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 200 / 200 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 435 / 435 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 255 / 255 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 32 / 32 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 285 / 285 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 108 | 0 / 20 |
| `09/01` | 3,479 | 14 / 14 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 88 / 88 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 56 / 56 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 28 / 28 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 26 / 26 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 22 / 22 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 2 / 2 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 |
| `05/00` | 639 | 12 / 12 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 43 / 43 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 23 / 23 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 15 / 15 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 92 / 92 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 95 / 95 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 3 / 3 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase probes remain within one; stable room/Link-XY/whole-save outcomes match. Rupee requests and every retained PCM onset gap at 162000–163000 remain unchanged; that probe has zero $0CFD-only exceptions.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14741, "song-end-poll": 640}` | 10 |

Default/reduced ordinary builds are warning-free. The fresh main default 200k state ledger still matches every record and EOF. Final full libraries:

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,503 | 1 | 3 |
| reduced | 2,038 | 0 | 2 |

The only default failure is the previously known spotlight caller-return test. Both configurations pass all 17 pacing/HDMA tests, 3 profiling tests, 27 ROM price families, and the SNES suite (504 pass, 7 ignored). Python shortfall/profile join tests pass. Input/cache hashes are unchanged and `git diff --check` passes. Remaining mutable HDMA, mixed raster/worker ownership and unresolved routine bodies stay open. No GPU/cache write/fmt/commit/push was used.

## Continuation on `codex/cost-pacing11` (main `2a8c8207`)

Evidence is in `target/cost-pacing11/`. This is a verified **diagnostic checkpoint**, not the requested mutable-HDMA implementation or a new pricing group. Task 1 remains open; no new HDMA or instruction charge was added. All previous predictor columns match the batch-10 HDMA result on every row of both fresh full censuses. The baseline default binary and its 200k ledger were freshly built/recorded from clean main before edits.

### HDMA admission and ordering audit

The predictor now records `cost_hdma_decline`, frozen at main completion before the closing NMI can change the program or timing owners. Bits 0–9 mean VWF, main DMA, song upload, raster wait, worker change, scheduled timed work, program-write epoch, no entry program with HDMA currently enabled, program signature mismatch, and no entry program with HDMA currently disabled respectively. Admission is logically unchanged. Signature groups below are disjoint; individual reason bits overlap. No-entry-program includes unowned WRAM/ROM descriptors and calls that enabled HDMA after entry; it is not automatically classified as mutable.

An entry/exit WRAM comparison does not establish what each HDMA reload fetched: a byte can change and be restored during main. Several native routines charge a whole block before publishing its stores. Therefore a cycle-ledger timestamp on those setters alone is not an instruction-level write epoch. The existing retained HDMA state machine can read current bytes at each event, but the completed-call pacing path lacks the ordered main-write stream needed to feed it. A program reinitialised from its new base at the next line would also lose the current field's descriptor cursor. No snapshot-based surcharge was shipped. Mutable-table and mid-main-program-change executed-ROM regressions requested for the extension remain open.

The structural profile and predictor describe their respective executed main intervals. The existing native HDMA estimate exceeds the source structural upper bound on 99 original missing iterations (125,460 clocks in total); this is unchanged by the diagnostic patch. It is an interval/ownership investigation worklist, not a justified new price or proof that each native interval has equivalent work. `structural-original-missing.json` retains every row and per-category bound comparison.

| Original missing population: decline signature | Iterations | Source HDMA lower | Source HDMA upper | Native HDMA |
| --- | ---: | ---: | ---: | ---: |
| admitted | 5,924 | 219,554,872 | 219,554,872 | 204,333,554 |
| vwf | 806 | 20,572,942 | 20,572,942 | 0 |
| no-entry-program | 649 | 46,736,176 | 46,736,176 | 0 |
| raster-wait | 491 | 9,864,712 | 9,864,712 | 0 |
| main-dma+disabled | 196 | 0 | 240 | 0 |
| disabled | 70 | 0 | 0 | 0 |
| vwf+no-entry-program | 65 | 1,553,904 | 1,553,904 | 0 |
| raster-wait+program-write | 53 | 566,588 | 566,588 | 0 |
| song-upload+disabled | 19 | 0 | 0 | 0 |
| program-write+no-entry-program | 17 | 0 | 0 | 0 |
| main-dma+program-write | 16 | 210 | 210 | 0 |
| main-dma+program-write+program-mismatch | 8 | 0 | 40 | 0 |
| worker-change+disabled | 6 | 0 | 0 | 0 |
| worker-change | 6 | 241,938 | 241,938 | 0 |
| main-dma+worker-change+disabled | 4 | 0 | 0 | 0 |
| raster-wait+program-write+no-entry-program | 4 | 102,850 | 102,850 | 0 |
| main-dma+song-upload+program-write+program-mismatch | 2 | 32,916 | 32,916 | 0 |
| main-dma+program-write+no-entry-program | 1 | 0 | 0 | 0 |
| main-dma+song-upload+program-write+disabled | 1 | 16,462 | 16,462 | 0 |
| main-dma+worker-change+program-write+no-entry-program | 1 | 0 | 0 | 0 |

### Refresh deficit: work versus placement

Let `R(w) = 40 * ceil(w / 1324)`, source base `S = instructions + DMA`, native base `N = main_master + DMA`, and source HDMA lower bound `Hs`. `main_master` already includes VWF, so its counter is never added again. The decomposition is exact per iteration:

```text
source_refresh - native_refresh
  = [R(S) - R(N)]
  + [(R(S+Hs)-R(S)) - (R(N+native_HDMA)-R(N))]
  + [source_refresh - R(S+Hs)]
  + [R(N+native_HDMA) - native_refresh]
```

| Term on all original 8,339 missing iterations | Master clocks |
| --- | ---: |
| Total refresh deficit | 60,314,200 |
| CPU/DMA work difference through R | 57,180,960 |
| HDMA occupancy difference through R | 2,870,520 |
| Remaining source placement/endpoint residual | 262,720 |
| Native reported refresh discrepancy | 0 |

Both source HDMA bounds give the same aggregate residual: 262,720 clocks. These are aggregate signed placement/endpoint residuals across interrupted intervals, nominal field lengths and the ceil approximation; they are not missing CPU instructions. The roughly 60M deficit is overwhelmingly work fed into `R`, not a missing refresh rate. No production refresh rule was changed.

### Refreshed missing-only ranking

`final-targeted-ranking.json` joins the newly recorded native profile to the retained complete executed-ROM source profiles, selecting only the current missing iterations. It covers 6704 iterations, with no unprofiled selection. `final-category-rankings.json` also retains independent rankings for 07/00, 09/00, 0E/02 and 07/02. The population and ordering are unchanged because this checkpoint changes diagnostics only. These signed exclusive rows still require ownership resolution; task 3's new (a)-class prices were not implemented.

| Routine | Exclusive deficit | Source calls | Native calls |
| --- | ---: | ---: | ---: |
| `008888` | 1,034,671,028 | 133 | 0 |
| `0ec9fd` | 411,707,458 | 7,359 | 0 |
| `0ecb5e` | 284,613,300 | 6,488 | 0 |
| `01893c` | 168,385,036 | 66,237 | 66,237 |
| `00f312` | 132,725,700 | 548 | 1,644 |
| `02fd87` | 106,056,000 | 4,800 | 0 |
| `02fdf7` | 56,044,800 | 4,800 | 0 |
| `01b2af` | 44,097,876 | 85,461 | 0 |
| `02febb` | 42,527,808 | 710 | 0 |
| `09c452` | 36,536,240 | 104 | 0 |
| `02ff5f` | 23,103,810 | 121,599 | 0 |
| `02f595` | 21,709,680 | 204 | 0 |
| `0684e2` | 20,072,672 | 105,703 | 211,406 |
| `09c5bb` | 15,720,332 | 4,026 | 0 |
| `06e72f` | 15,143,392 | 14,599 | 14,599 |
| `06dc10` | 14,047,484 | 15,901 | 16,506 |
| `02f679` | 12,740,476 | 353 | 0 |
| `02f638` | 12,736,240 | 353 | 0 |
| `09c176` | 12,644,178 | 583 | 0 |
| `00e78b` | 10,612,710 | 28 | 0 |

### Fresh census and structural categories

Both full gates PASS, complete=True, unexplained zero; 1,473,123 iterations each. Cartridge excess/FP remain zero. Live-SPC retains its prior differing-work excess/FP. Every runtime hold equals P2.

| Mode | Held | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: |
| rom | 95,739 | 6,938 | 0 | 0 |
| live | 95,632 | 7,641 | 596 | 133 |

| Category | Source lag | Missing (rom / live) | Excess (rom / live) | FP (rom / live) |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 871 / 1,198 | 0 / 224 | 0 / 96 |
| `07/02` | 8,770 | 632 / 632 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 170 / 170 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 200 / 200 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 435 / 435 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 255 / 255 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 32 / 32 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 285 / 285 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 108 | 0 / 20 |
| `09/01` | 3,479 | 14 / 14 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 192 / 192 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 88 / 88 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 56 / 56 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 28 / 28 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 26 / 26 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 22 / 22 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 2 / 2 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 |
| `05/00` | 639 | 12 / 12 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 43 / 43 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 23 / 23 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 15 / 15 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 92 / 92 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 95 / 95 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 3 / 3 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |

The following structural sums use the fixed original missing population, not all iterations in a category.

| Category | Iterations | Source HDMA lower .. upper | Native HDMA | Native above source upper (iterations / clocks) |
| --- | ---: | --- | ---: | --- |
| `07/02` | 759 | 62,230,558 .. 62,230,558 | 57,624,932 | 0 / 0 |
| `09/03` | 251 | 42,624,598 .. 42,624,598 | 36,209,006 | 18 / 3,352 |
| `09/23` | 533 | 31,077,604 .. 31,077,604 | 20,727,686 | 9 / 57,928 |
| `07/0E` | 238 | 26,677,526 .. 26,677,526 | 25,397,592 | 0 / 0 |
| `0E/02` | 892 | 23,137,766 .. 23,137,766 | 1,025,820 | 2 / 15,320 |
| `07/00` | 1,706 | 17,229,890 .. 17,229,890 | 11,875,102 | 0 / 0 |
| `07/07` | 706 | 15,264,136 .. 15,264,136 | 14,791,274 | 3 / 126 |
| `0E/0A` | 45 | 13,657,742 .. 13,657,742 | 105,516 | 9 / 5,556 |
| `09/00` | 1,163 | 11,205,542 .. 11,205,542 | 6,125,132 | 1 / 34 |
| `0E/07` | 28 | 8,849,352 .. 8,849,352 | 0 | 0 / 0 |
| `07/15` | 155 | 7,116,982 .. 7,116,982 | 6,834,908 | 0 / 0 |
| `09/01` | 53 | 6,687,076 .. 6,687,076 | 4,978,600 | 1 / 342 |
| `0F/01` | 255 | 4,849,080 .. 4,849,080 | 0 | 0 / 0 |
| `07/0F` | 192 | 4,217,128 .. 4,217,128 | 0 | 0 / 0 |
| `07/06` | 39 | 3,572,052 .. 3,572,052 | 3,417,410 | 0 / 0 |
| `09/11` | 16 | 2,650,938 .. 2,650,938 | 1,830,350 | 2 / 1,156 |
| `09/2E` | 29 | 2,338,576 .. 2,338,576 | 1,379,862 | 1 / 58 |
| `09/16` | 181 | 1,719,078 .. 1,719,078 | 1,616,376 | 19 / 646 |
| `07/12` | 54 | 1,662,614 .. 1,662,614 | 1,596,614 | 0 / 0 |
| `09/0D` | 159 | 1,509,698 .. 1,509,698 | 1,321,586 | 26 / 884 |
| `09/2C` | 12 | 1,235,174 .. 1,235,174 | 1,217,882 | 0 / 0 |
| `0B/24` | 85 | 876,248 .. 876,248 | 859,784 | 0 / 0 |
| `09/20` | 18 | 872,028 .. 872,028 | 0 | 0 / 0 |
| `12/03` | 43 | 817,688 .. 817,688 | 0 | 0 / 0 |
| `07/03` | 53 | 617,072 .. 617,072 | 232,198 | 0 / 0 |
| `0B/19` | 4 | 599,812 .. 599,812 | 613,426 | 2 / 14,580 |
| `0E/05` | 45 | 599,004 .. 599,004 | 591,108 | 0 / 0 |
| `0F/00` | 55 | 593,206 .. 593,206 | 0 | 0 / 0 |
| `0B/17` | 59 | 560,972 .. 560,972 | 557,150 | 0 / 0 |
| `0B/28` | 3 | 497,902 .. 497,902 | 496,264 | 0 / 0 |
| `0B/29` | 45 | 427,860 .. 427,860 | 422,148 | 0 / 0 |
| `15/03` | 4 | 420,062 .. 420,062 | 417,950 | 0 / 0 |
| `0B/2B` | 42 | 403,914 .. 403,914 | 389,844 | 0 / 0 |
| `09/0F` | 2 | 295,384 .. 295,384 | 0 | 0 / 0 |
| `09/0E` | 6 | 284,730 .. 284,730 | 189,572 | 1 / 170 |
| `0B/1C` | 29 | 275,732 .. 275,732 | 273,926 | 0 / 0 |
| `0B/1A` | 2 | 269,820 .. 269,820 | 284,820 | 2 / 15,000 |
| `07/18` | 6 | 241,938 .. 241,938 | 0 | 0 / 0 |
| `0E/0B` | 4 | 190,200 .. 190,200 | 190,202 | 1 / 2 |
| `07/13` | 1 | 186,002 .. 186,002 | 179,560 | 0 / 0 |
| `12/05` | 19 | 180,652 .. 180,652 | 180,652 | 0 / 0 |
| `0B/26` | 1 | 143,242 .. 143,242 | 150,364 | 1 / 7,122 |
| `0B/00` | 9 | 85,572 .. 85,572 | 64,782 | 0 / 0 |
| `0B/25` | 1 | 79,634 .. 79,634 | 76,064 | 0 / 0 |
| `10/01` | 3 | 57,048 .. 57,048 | 0 | 0 / 0 |
| `07/01` | 6 | 57,014 .. 57,014 | 47,162 | 0 / 0 |
| `17/01` | 3 | 49,378 .. 49,378 | 0 | 0 / 0 |
| `09/04` | 1 | 28,524 .. 28,524 | 27,348 | 0 / 0 |
| `07/0B` | 1 | 9,508 .. 9,508 | 3,234 | 0 / 0 |
| `09/06` | 1 | 9,508 .. 9,508 | 6,568 | 0 / 0 |
| `0E/01` | 1 | 596 .. 596 | 3,780 | 1 / 3,184 |
| `11/00` | 17 | 210 .. 210 | 0 | 0 / 0 |
| `00/00` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `00/01` | 7 | 0 .. 0 | 0 | 0 / 0 |
| `00/06` | 2 | 0 .. 0 | 0 | 0 / 0 |
| `00/07` | 12 | 0 .. 0 | 0 | 0 / 0 |
| `01/00` | 4 | 0 .. 0 | 0 | 0 / 0 |
| `01/02` | 7 | 0 .. 0 | 0 | 0 / 0 |
| `04/01` | 2 | 0 .. 0 | 0 | 0 / 0 |
| `03/01` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `05/00` | 11 | 0 .. 40 | 0 | 0 / 0 |
| `06/00` | 142 | 0 .. 240 | 0 | 0 / 0 |
| `0E/03` | 79 | 0 .. 0 | 0 | 0 / 0 |
| `08/00` | 19 | 0 .. 0 | 0 | 0 / 0 |
| `1B/02` | 2 | 0 .. 0 | 0 | 0 / 0 |
| `08/02` | 2 | 0 .. 0 | 0 | 0 / 0 |
| `19/00` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `19/01` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/00` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/02` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/04` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/08` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/12` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/14` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/16` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/18` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/1A` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/1C` | 1 | 0 .. 0 | 0 | 0 / 0 |
| `1A/20` | 1 | 0 .. 0 | 0 | 0 / 0 |

### Phase, exception and default verification

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase probes remain within one; stable room/Link-XY/whole-save outcomes match. The 162000–163000 rupee probe retains the same request gaps and every PCM onset gap; zero $0CFD-only exceptions in that window.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14741, "song-end-poll": 640}` | 10 |

Ordinary default/reduced builds are warning-free. The new default binary matches the fresh main-default 200k state ledger at every record and EOF. Both full library manifests verify disjoint, complete test unions.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,504 | 1 | 3 |
| reduced | 2,039 | 0 | 2 |

The sole default failure remains the known spotlight caller-return test. Both configurations pass pacing/HDMA, profiling, executed-ROM price families and the SNES suite; Python audit tests pass. No GPU/video/cached-A/V/Snes9x/GPU tests, cache writes, cargo fmt, commit or push were used. Scratch drivers stayed in `/tmp`. No claim of mutable-HDMA completion, new price coverage or exact A/V acceptance is made.

## Continuation on `codex/cost-pacing12` (main `c698e229`)

Evidence is in `target/cost-pacing12/raster/`. This verified CPU-only group ships stable immutable HDMA composition with raster barriers (the raster portion of task 1a). VWF composition, late-enable leases (task 1b), WRAM descriptors and task 2 routine prices remain open. No lag count or measured residual prices work; no scene thresholds, multipliers or duration tables were added.

### Raster barrier composition

VWF's `$0E:C9FD` counter is CPU work already included once in `native_main_work`. Its separate HDMA admission was trialled and then reverted after 23 original missing rows exceeded their source HDMA upper bounds by 1,666 clocks in total (21 rows by 68 clocks, one by 102 and one by 136). The retained `target/cost-pacing12/vwf-rom.csv` full census passes (missing 6,178, excess/FP 0), but is rejected pricing evidence, not the shipped result. `target/cost-pacing12/vwf-rejected.json` and `target/cost-pacing12/vwf-structural-bounds.json` retain the rejection and every bound excursion. No clocks were subtracted or capped to make that trial pass.

The `$00:F3A3..F3B2` V-counter owner retains each barrier's main-work position, target line and real taken-loop instruction price (178 clocks in the system data bank; the final 172-clock pass remains ordinary work). At completion, the immutable program advances each ordinary-work interval, then the barrier waits on the resulting wall time. HDMA events inside that wait belong to the barrier and are excluded from the next interval. Refresh ceilings retain the cumulative CPU-plus-HDMA work counter across splits. Hardware-derived NMI/worker exclusions remain in force.

Executed-ROM regressions exercise both immutable descriptor families over multi-field instruction streams and the completed native lease, plus the actual `$00:F312` spotlight routine through its V-counter loop at four radii. They compare wall time and the HDMA outside polling independently, allowing the existing nominal-field/ceil-refresh endpoint uncertainty. Both configurations pass all 19 pacing/HDMA tests. Source listings are retained in `spotlight-listing.txt`.

The completed-call decline field reserves its former raster bit (3); that owner now composes. VWF (bit 0), DMA, song uploads, worker changes, retained scheduled work and program mutations still decline. Immutable descriptor admission is unchanged.

### Original missing population: structural bound checks

The join freezes the original 8,339 missing iterations and their batch-11 decline signatures. The newly priced raster signature stays below the source upper bound both in aggregate and on every individual iteration; no new above-bound iteration is introduced. Pre-existing admitted-program excursions are retained separately rather than hidden.

| Original decline signature | Iterations | Source HDMA lower .. upper | Prior native HDMA | New native HDMA |
| --- | ---: | --- | ---: | ---: |
| admitted | 5,924 | 219,554,872 .. 219,554,872 | 204,333,554 | 204,333,554 |
| vwf | 806 | 20,572,942 .. 20,572,942 | 0 | 0 |
| no-entry-program | 649 | 46,736,176 .. 46,736,176 | 0 | 0 |
| raster-wait | 491 | 9,864,712 .. 9,864,712 | 0 | 6,115,106 |
| main-dma+disabled | 196 | 0 .. 240 | 0 | 0 |
| disabled | 70 | 0 .. 0 | 0 | 0 |
| vwf+no-entry-program | 65 | 1,553,904 .. 1,553,904 | 0 | 0 |
| raster-wait+program-write | 53 | 566,588 .. 566,588 | 0 | 0 |
| song-upload+disabled | 19 | 0 .. 0 | 0 | 0 |
| program-write+no-entry-program | 17 | 0 .. 0 | 0 | 0 |
| main-dma+program-write | 16 | 210 .. 210 | 0 | 0 |
| main-dma+program-write+program-mismatch | 8 | 0 .. 40 | 0 | 0 |
| worker-change+disabled | 6 | 0 .. 0 | 0 | 0 |
| worker-change | 6 | 241,938 .. 241,938 | 0 | 0 |
| main-dma+worker-change+disabled | 4 | 0 .. 0 | 0 | 0 |
| raster-wait+program-write+no-entry-program | 4 | 102,850 .. 102,850 | 0 | 0 |
| main-dma+song-upload+program-write+program-mismatch | 2 | 32,916 .. 32,916 | 0 | 0 |
| main-dma+program-write+no-entry-program | 1 | 0 .. 0 | 0 | 0 |
| main-dma+song-upload+program-write+disabled | 1 | 16,462 .. 16,462 | 0 | 0 |
| main-dma+worker-change+program-write+no-entry-program | 1 | 0 .. 0 | 0 | 0 |

There are 99 pre-existing above-bound rows, totalling 125,460 clocks; none gained HDMA in this group. `final-structural-bounds.json` retains every excursion and all changed original-population iteration IDs; `structural-original-missing.csv` retains the row-by-row join for all 8,339 iterations. The no-entry-program signature alone does not establish a late enable: its 649 original rows lack a program-write epoch change, and their descriptor owners still need identification. No late-enable or mutable-descriptor surcharge is claimed.

### Full census progression

Both final full gates PASS, complete=True, unexplained zero. Each census has 1,473,123 iterations and every runtime hold equals P2.

| Cartridge stage | Held | Missing | Excess | FP |
| --- | ---: | ---: | ---: | ---: |
| main / batch 11 | 95,739 | 6,938 | 0 | 0 |
| raster only | 95,884 | 6,793 | 0 | 0 |

Live-SPC executes different work and is reported separately: held 95,777, missing 7,496, excess 596, FP 133.

| Category | Source lag | Missing (rom / live) | Excess (rom / live) | FP (rom / live) |
| --- | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 871 / 1,198 | 0 / 224 | 0 / 96 |
| `07/02` | 8,770 | 632 / 632 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 170 / 170 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 200 / 200 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 435 / 435 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 176 / 176 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 32 / 32 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 285 / 285 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 108 | 0 / 20 |
| `09/01` | 3,479 | 14 / 14 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 142 / 142 | 0 / 0 | 0 / 0 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 88 / 88 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 56 / 56 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 28 / 28 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 26 / 26 | 0 / 0 | 0 / 0 |
| `11/00` | 973 | 22 / 22 | 0 / 0 | 0 / 0 |
| `07/01` | 928 | 2 / 2 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 |
| `05/00` | 639 | 12 / 12 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 27 / 27 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 23 / 23 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 15 / 15 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 92 / 92 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 95 / 95 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 3 / 3 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 18 / 18 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |

### Phase, exception and rupee verification

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase probes remain within one; stable room/Link-XY/whole-save outcomes match. In the 162000–163000 rupee window, request and PCM gaps are unchanged from batch 11. Every onset and gap remains in `final-probe-metrics.json`; zero $0CFD-only exceptions occur in the window.

PCM gap median: 8.278090 -> 8.278090 frames. These are fresh native PCM probes, not exact A/V acceptance.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14740, "song-end-poll": 640}` | 10 |

### Verification and stopping point

Ordinary default/reduced builds are warning-free. The final default binary matches the freshly built clean-main default baseline on all 200,000 state-ledger records and EOF; the default/ROM-timing instruction prices are unchanged. Both library manifests verify complete, disjoint test unions.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,505 | 1 | 3 |
| reduced | 2,040 | 0 | 2 |

The only default failure is the known spotlight caller-return test. Both configurations pass pacing/HDMA, profiling, all 27 executed-ROM price families and the SNES suite; Python audit tests pass. `final-targeted-ranking.json` re-ranks only the remaining missing population with complete source/native profile coverage. Task 2 ownership resolutions and new routine prices are not claimed by this group.

The supplied cache and ledgers stayed read-only. No GPU, video, cached-A/V, external Snes9x, cargo GPU tests, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`. This is a verified partial pricing point; VWF, late enable and routine work remain for the next group.

## Continuation on `codex/cost-pacing13` (main `f0e5948a`)

Evidence is in `target/cost-pacing13/`. This verified CPU-only group corrects retained HDMA descriptor ownership, re-admits VWF instruction work to immutable bus composition, and identifies the 649 original no-entry-program rows. Instruction routine pricing (task 3) remains unfinished; no new instruction price was added.

### The 34-clock quantum and the composition preconditions

`MirrorWarp_FinalizeAndLoadDestination` (`$02:B260..B265`) and `Mirror_Func5` (`$02:9D5D..9D62`) execute `LDA #$2641; STA $4370`. They change only DMAP7/BBAD7. The native calls instead ran a full HDMA setup and replaced A1T7/A1B7/DASB7 with `$00:F2FB` / bank zero. On the lagless path they now execute the two register writes and preserve the descriptor address, indirect bank and live cursor. The default/ROM-timing branch keeps its previous behavior.

The retained `$0A:BDDD` table has two `$F0` descriptors: 224 repeating transfers, then a zero descriptor. The substituted `$00:F2FB` table has two `$F8` descriptors and remains live through line 224. For a single mode-1 channel the complete field costs are **9,474** and **9,508** clocks. A 42-clock extra transfer replaces the 8-clock terminal indirect fetch: **42 - 8 = 34**, hence 68/102/136 across two/three/four fields. This is a descriptor-program error, not a duplicated `$0E:C9FD` counter, a split-line subtraction or a fitted correction.

The terminal read itself exposed the no-entry-program cause: the final active indirect channel reads the zero descriptor as its low byte and the following ROM byte `$0A:BDE4 = $FF` as its high byte. That immutable byte is now owned. No payload value or observed residual sets a clock price.

The composer also requires a valid field initialization. NMI clears `$420C` at `$00:8135` and restores it at `$00:8222` / `$00:82C4`. If the leading handler can span V0/H20, the freshly initialized repeating program does not establish its retained descriptor cursor. The lease conservatively declines when the handler plus its ceil-refresh placement envelope reaches that hardware event (including the short return suffix in the envelope). It does not subtract an estimated number of lines. Bit 10 (`nmi-over-hdma-init`) reports this case; legacy bits 0 and 3 remain reserved. These leases need the actual masked-init/cursor handoff before they can compose.

Executed-ROM regressions compare both producer prefixes with the native DMA register state, including retained A2A/NTRL/DAS state, and verify both descriptor field totals. The actual DMA device regression distinguishes disable/init/late-enable from a fresh init (34 versus 66 clocks on its first dual-channel event). Existing executed-ROM multi-field and raster-barrier regressions remain green. VWF work still enters `native_main_work` once.

### Original no-entry-program inventory

The 649 original rows were replayed in 63 seeded CPU-only windows with entry/exit DMA register diagnostics. Their masks and programming remain unchanged within the calls. These windows establish descriptor identity; the full censuses below establish the final runtime result. The diagnostic executable/source trial is retained separately in `unconditional-init-trial/`, with the per-row inventory in `no-entry-descriptor-inventory.json`.

| Rows | Mask | Active descriptor/register owners |
| ---: | --- | --- |
| 480 | `$80` | Ch7: DMAP `$42`, BBAD `$1E`, A1 `$0A:BDDD`, indirect bank `$00` |
| 169 | `$C0` | Ch6: `$42/$0D`, A1 `$0A:BDCF`, indirect bank `$0A`; Ch7: `$42/$0F`, A1 `$0A:BDDD`, indirect bank `$00` |

`Attract_SetUpConclusionHDMA` writes A1T7/A1B7/DASB7 at `$0A:BC36/BC3B/BC40`, then `$9B=$80` at `$BC47`; it leaves channel 6 intact. `WorldMap_SetUpHDMA` programs the perspective descriptor addresses/banks at `$0A:BD4D..BD58`, `$BD64..BD6F` or `$BD88..BD93`, the indirect banks at `$BD98..BD9B`, then `$9B=$C0` at `$BDA2`. `InitializeMirrorHDMA` changes only the channel destinations/modes at `$00:FE31/FE37`, fills WRAM payloads and sets `$9B=$C0` at `$FE5B`. The subsequent NMI copies `$9B` to `$420C`; the descriptor program predates the iterations rather than being enabled late inside them. Source listings retain these instructions. The mutable WRAM region is indirect payload, not the descriptor stream, so payload writes do not change transfer counts.

Of the original 649 rows, **629** are admitted and **20** retain the masked-init decline. In the rejected unconditional-init trial, exactly those 20 rows exceeded source bounds; the hardware-event envelope identifies all 20 without row IDs, module checks, scene thresholds or residual prices. No mutable WRAM descriptor lease is added.

### Rowwise source bounds

The frozen population remains all 8,339 original missing iterations. `structural-original-missing.csv` joins every row; `final-structural-bounds.json` includes every current excursion and every one of the 99 pre-existing above-bound rows.

| Original signature | Rows | Source HDMA upper | Prior HDMA | Final HDMA |
| --- | ---: | ---: | ---: | ---: |
| admitted | 5,924 | 219,554,872 | 204,333,554 | 202,570,994 |
| VWF | 806 | 20,572,942 | 0 | 20,549,968 |
| raster wait | 491 | 9,864,712 | 6,115,106 | 6,115,106 |
| no entry program | 649 | 46,736,176 | 0 | 43,609,636 |
| VWF + no entry program | 65 | 1,553,904 | 0 | 1,553,560 |

All original VWF signatures stay at or below their source bound on **every row**, including the 23 rejected batch-12 rows. The VWF-only aggregate falls from the rejected trial's 20,551,634 to 20,549,968 clocks, exactly removing its 1,666-clock overrun quantum. No new above-bound row is introduced. Of the pre-existing 99, **72** now stay within bounds with their corrected descriptor program and **27** have the masked-init decline. **0** remain above. Their individual IDs, old/new clocks and bounds are retained. The 27 declined leases still need their retained-cursor handoff established.

### Full census metrics

Both complete census gates PASS with unexplained zero. Each covers 1,473,123 iterations, and every runtime hold count equals P2. Exact/within-one below count source-lagged iterations. Live-SPC executes different work.

| Stage | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| batch 12 / cartridge | 95,884 | 6,793 | 0 | 0 | 33,016 | 39,382 |
| batch 13 / cartridge | 96,899 | 5,778 | 0 | 0 | 34,020 | 39,393 |
| batch 13 / live SPC | 96,792 | 6,479 | 594 | 133 | 33,886 | 39,260 |

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 374 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,307 |
| `07/02` | 8,770 | 632 / 632 | 0 / 0 | 0 / 0 | 1,453 / 1,453 | 1,867 / 1,867 |
| `06/00` | 8,196 | 170 / 170 | 0 / 0 | 0 / 0 | 9 / 9 | 106 / 106 |
| `07/0E` | 7,979 | 200 / 200 | 0 / 0 | 0 / 0 | 4,772 / 4,772 | 4,932 / 4,932 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 176 / 176 | 0 / 0 | 0 / 0 | 4,703 / 4,703 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 287 / 287 | 0 / 0 | 0 / 0 | 2,632 / 2,632 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 107 | 0 / 20 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 142 / 142 | 0 / 0 | 0 / 0 | 2,576 / 2,576 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 | 2,666 / 2,664 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 88 / 88 | 0 / 0 | 0 / 0 | 1,647 / 1,647 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 26 / 26 | 0 / 0 | 0 / 0 | 577 / 577 | 603 / 603 |
| `11/00` | 973 | 22 / 22 | 0 / 0 | 0 / 0 | 1 / 1 | 11 / 11 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 8 / 8 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 27 / 27 | 0 / 0 | 0 / 0 | 364 / 364 | 391 / 391 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 | 232 / 232 | 287 / 287 |
| `07/12` | 391 | 23 / 23 | 0 / 0 | 0 / 0 | 216 / 216 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 90 / 90 | 0 / 0 | 0 / 0 | 143 / 143 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 3 / 3 | 0 / 0 | 0 / 0 | 7 / 7 | 8 / 8 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 | 46 / 46 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 | 28 / 28 | 40 / 40 |
| `1A/14` | 47 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `1A/02` | 46 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

### Phase, exception and rupee probes

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase probes remain within one, and the stable room/Link-XY/whole-save outcomes match. Values are source/native.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14743, "song-end-poll": 640}` | 10 |
| probes | `{}` | 0 |

In 162000–163000, the same 27 rupee requests remain at the same logical iterations. Their presentation gaps change at these positions (before → after): 1: 23 → 24, 7: 24 → 25, 8: 15 → 16, 23: 11 → 12. Every request gap, PCM onset and PCM gap remains in `final-probe-metrics.json`.
The PCM gap median changes from 8.278090 to 8.253745 frames. This reports native PCM timing, not exact A/V acceptance.

### Build and test verification; stopping point

The ordinary default and reduced builds are warning-free. The final default binary matches all 200,000 records and EOF against a fresh default binary/ledger built from the clean requested main before edits. Default/ROM-timing instruction prices are unchanged. Both library manifests verify complete disjoint test unions.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,507 | 1 | 3 |
| reduced | 2,042 | 0 | 2 |

The only default failure is the known spotlight caller-return test. Both configurations pass the pacing/HDMA, profiling, all executed-ROM price families and SNES suites; both Python audit suites pass. The interrupted incremental link failure and superseded trials are retained separately; final builds disable incremental compilation.

`pacing_shortfalls.py` and `routine_coverage_diff.py` retain the final missing-only ranking. Missing iterations lacking complete retained source/native profile coverage: **0** (listed in `verification.json`); no absent profile is presented as zero work.

Routine-price work remains open. The fresh missing-only ranking retains **66,237 source and native calls** to `$01:893C`, with an exclusive deficit of **168,385,036** clocks; that deficit is diagnostic, not a price. The bank-02 rows `$02:FD87/FDF7` already charge in `write_overworld_map8_row_step`; `$02:FEBB/FF5F` in `decompress_bank02_into` (preview passes `None`); `$02:F595/F638/F679` in `begin_overworld_quadrant_decode`. Their missing child scopes are not evidence of missing clocks, and no duplicate parent+child charge was added. The next price group starts with the remaining object tails, then resolves `$01:B2AF`, `$09:C452`, `$09:C5BB`, `$09:C176` and `$00:E78B`, preserving the requested ordering and the existing bank-02 ownership.

No GPU, video, cached-A/V, external Snes9x, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`. This is a verified HDMA stopping point; routine-price completion, masked-init cursor completion and exact A/V acceptance are not claimed.

## Continuation on `codex/cost-pacing14` (main `f8f837a0`)

Evidence is in `target/cost-pacing14/`. This first routine-pricing group adds source-derived object-dispatch tails for 46 ordinary IDs. This is a verified partial CPU-only stopping point. Remaining object tails, the subsequent unpriced routine bodies and masked-init cursor handoff remain open. No HDMA admission or hardware rule changed.

### Missing-only ranking and ownership

The pre-edit refresh uses the current missing population: **5,555 iterations / 5,778 missing holds**. Both `pacing_shortfalls.py` and `routine_coverage_diff.py` were rerun on matching source/native IDs; the retained batch-13 profile source hashes match the requested HEAD. `$01:893C` has 66,237 source and native calls and a 168,385,036-clock exclusive deficit before this group. A deficit is diagnostic, never an additive price.

The bank-02 audit confirms existing owners: `$02:FD87/FDF7` in `write_overworld_map8_row_step`, `$02:FEBB/FF5F` in `decompress_bank02_into` (preview passes `None`), and `$02:F595/F638/F679` in `begin_overworld_quadrant_decode`. No extra parent/child price was added. `$09:C452` already charges its fixed prefix, 4,096 room-mask clears, 512 loaded-mask clears and eight room-history stores; `$09:C5BB` has scan-column work in `advance_overworld_proximity_cell_scan`; `$00:E78B` mostly aliases the existing `$00:E78F` background model, with its two entry stores still an unshipped 48-clock prefix. Their absent child profile scopes do not establish wholly missing bodies. This is an ownership audit, not a claim of complete execution-path pricing for those families. `$01:B2AF` still needs its vertical-return alias resolved separately from the 506-clock `$01:9895` body; `$09:C176` and further rows remain open. `next-owner-audit.json` retains the signed alias rows and next source-work boundaries.

### ROM-derived object tail group

All new prices are lagless-only and have source-address comments. `object-tail-listings.txt` is generated by `scripts/rom_block_costs.py` with M16/X16. Setup, dimension helpers, taken branches, actual loop counts, JSR instructions and returns are priced from instructions. No observed hold count, scene threshold, multiplier or scene table supplies a price.

| Tail | ROM instruction ownership |
| --- | --- |
| `$01:8B0D..8B73` (`03/04`) | Setup/size and eight load/two-background-store triples per pass; native both-background helper has no child charge. |
| `$01:8C58/8C61` → `$B2AA/B29C` | Size/setup, initial decrement/branch and diagonal loops; `$B279` retains its independently charged five-store body. |
| `$01:8C6A/8CB9` → `$8C76..8CB8` | Size/setup and five direct dual-background triples per pass, diagonal step and return. |
| `$01:9111`, `$92FB`, `$9323`, `$9338` | Size, source/width setup, actual spacing/loop and returns; `$97ED`, `$8D80` or `$9895` owns its existing child stores. |
| `$01:9357`, `$936F`, `$9387`, `$9702`, `$971B` | Width/source, size and actual horizontal/vertical loop control; `$97F0` / `$8D80` retains child pricing. |

The new executed-ROM regression covers all 46 IDs, all 16 width/height combinations and both pointer planes (1,472 paths), checking complete instruction work and tilemap writes. It reads original jump targets and pointer tables from the ROM. The early fixture table/DB mistakes are superseded diagnostics; the final test uses `$01:8200` jump targets and DB zero for the `$00:9B52` tile data. No engine behavior was changed to satisfy fixtures.

### Full census metrics

Both complete census gates PASS with unexplained zero and 1,473,123 iterations each. Every runtime hold equals P2. Exact/within-one below count source-lagged iterations; live-SPC executes different work.

| Stage | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| batch 13 / cartridge | 96,899 | 5,778 | 0 | 0 | 34,020 | 39,393 |
| batch 14 / cartridge | 96,939 | 5,738 | 0 | 0 | 34,029 | 39,424 |
| batch 14 / live SPC | 96,832 | 6,435 | 590 | 132 | 33,895 | 39,292 |

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 370 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 602 / 602 | 0 / 0 | 0 / 0 | 1,460 / 1,460 | 1,890 / 1,890 |
| `06/00` | 8,196 | 169 / 169 | 0 / 0 | 0 / 0 | 9 / 9 | 107 / 107 |
| `07/0E` | 7,979 | 196 / 196 | 0 / 0 | 0 / 0 | 4,772 / 4,772 | 4,936 / 4,936 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 176 / 176 | 0 / 0 | 0 / 0 | 4,703 / 4,703 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 285 / 285 | 0 / 0 | 0 / 0 | 2,634 / 2,634 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 103 | 0 / 19 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 142 / 142 | 0 / 0 | 0 / 0 | 2,576 / 2,576 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 | 2,666 / 2,664 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 88 / 88 | 0 / 0 | 0 / 0 | 1,647 / 1,647 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 26 / 26 | 0 / 0 | 0 / 0 | 577 / 577 | 603 / 603 |
| `11/00` | 973 | 20 / 20 | 0 / 0 | 0 / 0 | 1 / 1 | 13 / 13 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 11 / 11 | 0 / 0 | 0 / 0 | 2 / 2 | 9 / 9 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 27 / 27 | 0 / 0 | 0 / 0 | 364 / 364 | 391 / 391 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 | 232 / 232 | 287 / 287 |
| `07/12` | 391 | 23 / 23 | 0 / 0 | 0 / 0 | 216 / 216 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 90 / 90 | 0 / 0 | 0 / 0 | 143 / 143 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 3 / 3 | 0 / 0 | 0 / 0 | 7 / 7 | 8 / 8 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 | 46 / 46 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 | 28 / 28 | 40 / 40 |
| `1A/14` | 47 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `1A/02` | 46 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

All 8,339 frozen original missing iterations remain at or below their measured source HDMA upper bound, with zero excursions. The existing 27 + 20 masked-init leases remain declined; no retained-cursor handoff is claimed. Their admission was left untouched while routine pricing takes priority. `final-structural-bounds.json` retains every joined row and instruction-work deltas: **14,871,534** additional main instruction clocks, with zero negative rows. `group-structural-check.json` also verifies unchanged membership of all 63 pre-existing masked-init guarded rows, including the 47 requested leases and 16 others.

### Phase, exception and rupee probes

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase probes remain within one, and stable room/Link-XY/whole-save outcomes match. Values are source/native.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14739, "song-end-poll": 639}` | 10 |
| probes | `{}` | 0 |

In 162000–163000, the same 27 rupee requests occur at the same logical iterations. Request-gap changes versus batch 13: none. Every presentation request gap and PCM onset/gap remains in `final-probe-metrics.json`.
PCM gap median: 8.253745 → 8.253745 frames. This is native PCM timing, not exact A/V acceptance.

### Verification and stopping point

Ordinary default/reduced builds are warning-free. The final default binary matches all 200,000 records and EOF against a fresh default binary/ledger built from clean requested main before edits. Default/ROM-timing ledger prices are unchanged. Both library manifests verify complete, disjoint test unions.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,508 | 1 | 3 |
| reduced | 2,043 | 0 | 2 |

The only default failure is the known spotlight caller-return test. Both configurations pass the object executed-ROM test, pacing/HDMA, profiling, all other executed-ROM price families and SNES suites. Both Python audit suites pass.

The final missing population is **5,546 iterations / 5,738 holds**. Final missing iterations without complete retained source/native profiles: **0**. Both ranking tools were rerun on that population; no absent profile is treated as zero. `$01:893C` retains a 152,449,174-clock exclusive deficit on the remaining cohort; this is diagnostic, not an additive price. Further object tails and requested routine work remain, so masked-init work was not started. This stops after the first verified routine-price group.

No GPU, video, cached-A/V, external Snes9x, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache stayed read-only. Exact A/V acceptance is not claimed.

## Continuation on `codex/cost-pacing15` (main `85970b5b`)

Evidence is in `target/cost-pacing15/`. This verified CPU-only group adds the direct 4x4 fill tail shared by ordinary IDs $C0/$C2. Object attribution is diagnostic and never supplies an additive price. Further routine work and masked-init cursor handoff remain open.

### Object attribution and loop ownership

The source capture runs the in-repo CPU from reset with the existing input and SRAM, without rendering or external Snes9x. Its object CSV partitions the existing $01:893C call frame by the actual ROM dispatch index and retains real JSR/JSL children; JMP tails keep their original frame. Native profiling records the same identity without moving any charge or scope. With --routine-profile NAME.csv the native sidecar is NAME.objects.csv; the source example emits object-profile.csv. Object keys use subtype 1 (ordinary), 2 (extended) or 3 (sized), followed by the two-digit ID. `object-deficits-before.json` retains every signed per-ID and child row on the 5,546-iteration / 5,738-hold missing cohort, and checks exact reconciliation against the retained source and native routine profiles.

| Subtype / ID | Draw target | Executions source/native | Source exclusive | Native exclusive | Difference | Net inclusive gap |

| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `161` | `018a89` | 2,475 / 2,475 | 35,971,638 | 2,178,000 | 33,793,638 | 34,278,054 |
| `162` | `018a89` | 2,454 / 2,454 | 35,745,930 | 2,159,520 | 33,586,410 | 34,066,878 |
| `1c0` | `018b94` | 2,479 / 2,479 | 24,387,220 | 2,181,520 | 22,205,700 | 22,205,700 |
| `1c2` | `018b94` | 1,230 / 1,230 | 11,466,188 | 1,082,400 | 10,383,788 | 10,383,788 |
| `160` | `018b7e` | 2,839 / 2,839 | 12,325,930 | 2,498,320 | 9,827,610 | 11,189,418 |
| `164` | `018aa4` | 665 / 665 | 6,219,292 | 585,200 | 5,634,092 | 5,824,282 |
| `163` | `018aa4` | 683 / 683 | 5,983,042 | 601,040 | 5,382,002 | 5,577,340 |
| `169` | `018ec3` | 2,214 / 2,214 | 5,991,816 | 1,948,320 | 4,043,496 | 5,103,336 |
| `1c6` | `018fa5` | 1,606 / 1,606 | 2,819,314 | 1,413,280 | 1,406,034 | 1,406,034 |
| `188` | `018f0c` | 503 / 503 | 1,832,866 | 442,640 | 1,390,226 | 1,026,054 |
| `1c3` | `018d9e` | 283 / 283 | 1,595,148 | 249,040 | 1,346,108 | 1,346,108 |
| `32f` | `01b395` | 2,520 / 2,520 | 6,528,978 | 5,253,858 | 1,275,120 | 0 |
| `20b` | `019813` | 395 / 395 | 1,551,560 | 332,590 | 1,218,970 | 1,218,970 |
| `208` | `019813` | 394 / 394 | 1,547,632 | 331,748 | 1,215,884 | 1,215,884 |
| `209` | `019813` | 388 / 388 | 1,524,064 | 326,696 | 1,197,368 | 1,197,368 |
| `20a` | `019813` | 374 / 374 | 1,469,072 | 314,908 | 1,154,164 | 1,154,164 |
| `171` | `019120` | 414 / 414 | 1,482,536 | 364,320 | 1,118,216 | 1,226,684 |
| `1a4` | `018ddc` | 224 / 224 | 1,309,776 | 197,120 | 1,112,656 | 1,971,088 |
| `18a` | `018ebe` | 140 / 140 | 1,223,112 | 123,200 | 1,099,912 | 1,166,832 |
| `122` | `018ef0` | 2,302 / 2,302 | 3,051,392 | 2,025,760 | 1,025,632 | 4,565,718 |
| `16b` | `018f8a` | 455 / 455 | 1,100,170 | 400,400 | 699,770 | 829,900 |
| `319` | `0198d0` | 444 / 444 | 1,200,870 | 522,144 | 678,726 | 473,290 |
| `16a` | `018f8a` | 421 / 421 | 1,015,490 | 370,480 | 645,010 | 765,416 |
| `15d` | `018f36` | 347 / 347 | 1,510,262 | 940,460 | 569,802 | 315,104 |
| `1a1` | `018bf4` | 256 / 256 | 780,818 | 225,280 | 555,538 | 2,326,930 |

The existing room-object loop already charges 332 per ordinary fetch/dispatch pass ($01:88E4..88FB), 336 per door pass ($01:8902..8914), the $FFF0 transition and final sentinel/RTS. The four layout/layer stream calls and line-pointer copies have their own existing parent price. No second loop price was added.

### ROM-derived fill price

The $01:8B94..8BDF listing from `scripts/rom_block_costs.py` gives 108 for dimension increments, 64 per row setup, 1,190 per 4x4 block, 186 per row tail, actual taken branches and the 42-clock RTS. The implementation charges actual width+1 and height+1 loop counts only in lagless mode; its tile-fill helper has no child price. The executed-ROM test now covers 48 ordinary IDs, all 16 sizes and both pointer planes (1,536 paths), checking instruction clocks and tilemap bytes. The profiling regression checks conservation and excludes muted previews.

### Full census metrics

Both complete census gates PASS with unexplained zero and 1,473,123 iterations each. Every runtime hold equals P2. Exact/within-one below count source-lagged iterations; live-SPC executes different work.

| Stage | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| batch 14 / cartridge | 96,939 | 5,738 | 0 | 0 | 34,029 | 39,424 |
| batch 15 / cartridge | 97,041 | 5,636 | 0 | 0 | 34,063 | 39,491 |
| batch 15 / live SPC | 96,934 | 6,340 | 597 | 134 | 33,929 | 39,358 |

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 377 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,307 |
| `07/02` | 8,770 | 549 / 549 | 0 / 0 | 0 / 0 | 1,470 / 1,470 | 1,933 / 1,933 |
| `06/00` | 8,196 | 155 / 155 | 0 / 0 | 0 / 0 | 11 / 11 | 118 / 118 |
| `07/0E` | 7,979 | 171 / 171 | 0 / 0 | 0 / 0 | 4,786 / 4,786 | 4,947 / 4,947 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 176 / 176 | 0 / 0 | 0 / 0 | 4,703 / 4,703 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 279 / 279 | 0 / 0 | 0 / 0 | 2,640 / 2,640 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 110 | 0 / 21 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 142 / 142 | 0 / 0 | 0 / 0 | 2,576 / 2,576 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 | 2,666 / 2,664 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 88 / 88 | 0 / 0 | 0 / 0 | 1,647 / 1,647 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 26 / 26 | 0 / 0 | 0 / 0 | 577 / 577 | 603 / 603 |
| `11/00` | 973 | 19 / 19 | 0 / 0 | 0 / 0 | 1 / 1 | 14 / 14 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 11 / 11 | 0 / 0 | 0 / 0 | 2 / 2 | 9 / 9 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 27 / 27 | 0 / 0 | 0 / 0 | 364 / 364 | 391 / 391 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 | 232 / 232 | 287 / 287 |
| `07/12` | 391 | 21 / 21 | 0 / 0 | 0 / 0 | 218 / 218 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 90 / 90 | 0 / 0 | 0 / 0 | 143 / 143 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 1 / 1 | 0 / 0 | 0 / 0 | 46 / 46 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 | 28 / 28 | 40 / 40 |
| `1A/14` | 47 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `1A/02` | 46 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

All 8,339 frozen original missing iterations remain at or below their measured source HDMA upper bound, with zero excursions. The existing 27 + 20 masked-init leases remain declined; no retained-cursor handoff is claimed. Their admission was left untouched while routine pricing takes priority. `final-structural-bounds.json` retains every joined row and instruction-work deltas; no original row loses main instruction clocks. The group adds 33,758,140 instruction clocks across those rows. The unchanged 63-row masked-init membership, including the requested 47 leases, is checked in group-structural-check.json.

### Phase, exception and rupee probes

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase probes remain within one, and stable room/Link-XY/whole-save outcomes match. Values are source/native.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14740, "song-end-poll": 641}` | 10 |
| probes | `{}` | 0 |

In 162000–163000, the same 27 rupee requests occur at the same logical iterations. Request-gap changes versus batch 14: none. Every presentation request gap and PCM onset/gap remains in `final-probe-metrics.json`.
PCM gap median: 8.253745 → 8.253745 frames. This is native PCM timing, not exact A/V acceptance.

### Verification and stopping point

Ordinary default/reduced builds are warning-free. The final default binary matches all 200,000 records and EOF against a fresh default binary/ledger built from clean requested main before edits. Default/ROM-timing ledger prices are unchanged. Both library manifests verify complete, disjoint test unions.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,509 | 1 | 3 |
| reduced | 2,044 | 0 | 2 |

The only default failure is the known spotlight caller-return test. Both configurations pass the object executed-ROM test, pacing/HDMA, profiling, all other executed-ROM price families and SNES suites. Both Python audit suites pass.

Final missing iterations without complete retained source/native profiles: **0**. Both ranking tools were rerun on the final missing population; no absent profile is treated as zero. Further object tails and requested routine work remain, so masked-init work was not started. This stops after the first verified routine-price group.

No GPU, video, cached-A/V, external Snes9x, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache stayed read-only. Exact A/V acceptance is not claimed.

### Second verified group: completed vertical 4×2 draws

Evidence is retained separately in `target/cost-pacing15/vertical-4x2/`. This group follows the direct fill group above. It prices the highest remaining IDs $61/$62 and their shared $90/$91 path, plus callers $49/$4A, $55/$56 and $65/$66 of the same $B220 body. Every new charge is lagless-only and derives from the actual ROM instructions and native loop count.

The ROM listing (`source-listings.txt`, from `scripts/rom_block_costs.py`) gives 94 clocks of wrapper work, the actual $B0BE or $B0AC size child, and $01:B220..B253: 32 setup + 960 per block + 6 per taken loop branch + 42 RTS. A retained CPU boundary returns before this eager-body charge; its staged owner remains disjoint. The executed-ROM regression covers 58 ordinary IDs, 16 sizes and both pointer planes (1,856 paths), checking clocks and tilemap writes.

The initially interrupted full-census outputs are preserved in `interrupted/`; both full censuses were restarted from iteration zero because clean seed boundaries can retain native state. Final logs/CSVs below are the complete restarted runs. Completed library test results were retained and unfinished exact test names rerun; the manifest verifies the complete disjoint union and records this recovery. Final source hashes match the edits being delivered.

| Stage | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| fill group / cartridge | 97,041 | 5,636 | 0 | 0 | 34,063 | 39,491 |
| vertical 4×2 / cartridge | 97,288 | 5,389 | 0 | 0 | 34,255 | 39,546 |
| vertical 4×2 / live SPC | 97,181 | 6,085 | 589 | 131 | 34,121 | 39,414 |

Both full census gates PASS with 1,473,123 iterations each and unexplained zero; every held count equals P2.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 369 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 414 / 414 | 0 / 0 | 0 / 0 | 1,569 / 1,569 | 1,969 / 1,969 |
| `06/00` | 8,196 | 108 / 108 | 0 / 0 | 0 / 0 | 46 / 46 | 130 / 130 |
| `07/0E` | 7,979 | 135 / 135 | 0 / 0 | 0 / 0 | 4,817 / 4,817 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 176 / 176 | 0 / 0 | 0 / 0 | 4,703 / 4,703 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 269 / 269 | 0 / 0 | 0 / 0 | 2,650 / 2,650 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 102 | 0 / 18 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 142 / 142 | 0 / 0 | 0 / 0 | 2,576 / 2,576 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 | 2,666 / 2,664 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 86 / 86 | 0 / 0 | 0 / 0 | 1,649 / 1,649 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 11 / 11 | 0 / 0 | 0 / 0 | 7 / 7 | 16 / 16 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 10 / 10 | 0 / 0 | 0 / 0 | 3 / 3 | 9 / 9 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 27 / 27 | 0 / 0 | 0 / 0 | 364 / 364 | 391 / 391 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 | 232 / 232 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 90 / 90 | 0 / 0 | 0 / 0 | 143 / 143 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 | 28 / 28 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts are source/native and remain within one; all eight stable room/Link-XY/whole-save outcomes match.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14743, "song-end-poll": 638}` | 10 |
| probes | `{}` | 0 |

The 162000–163000 probe retains 27 rupee requests at the same logical iterations. Request-gap changes versus the fill group: none. PCM gap median remains 8.253745 frames; all onsets and gaps are retained in `final-probe-metrics.json`. This is native PCM timing evidence.

All 8,339 frozen original missing iterations stay within their source HDMA upper bounds; none loses main instruction clocks. Additional main clocks versus the fill group: 79,893,854. HDMA admission and the 47 masked-init leases were not changed.

Both ordinary builds are warning-free. The default binary matches every record and EOF of the fresh clean-main 200k baseline. Default/ROM-timing prices remain unchanged.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,509 | 1 | 3 |
| reduced | 2,044 | 0 | 2 |

The only default failure is the known spotlight caller-return test. Pacing, profiling, executed-ROM object prices, the SNES library checks and Python audit suites pass.

The remaining missing cohort has 5,320 iterations and 5,389 holds, with zero absent retained routine profiles. `object-deficits-after.json` retains all per-ID and per-child source/native totals and execution counts on this cohort; the signed $01:893C exclusive deficit is 41,824,070 clocks. `object-attribution-before.csv` in the parent evidence directory also lists initial per-execution averages and execution counts for all 322 IDs.

| Subtype / ID | Draw target | Executions source/native | Source exclusive | Native exclusive | Difference | Net inclusive gap |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `160` | `018b7e` | 1,836 / 1,836 | 7,945,410 | 1,615,680 | 6,329,730 | 7,208,190 |
| `164` | `018aa4` | 608 / 608 | 5,723,710 | 535,040 | 5,188,670 | 5,362,558 |
| `163` | `018aa4` | 616 / 616 | 5,482,532 | 542,080 | 4,940,452 | 5,116,628 |
| `169` | `018ec3` | 1,912 / 1,912 | 5,209,416 | 1,682,560 | 3,526,856 | 4,442,298 |
| `1c6` | `018fa5` | 1,483 / 1,483 | 2,605,342 | 1,305,040 | 1,300,302 | 1,300,302 |
| `1c3` | `018d9e` | 217 / 217 | 1,294,388 | 190,960 | 1,103,428 | 1,103,428 |
| `20b` | `019813` | 356 / 356 | 1,398,368 | 299,752 | 1,098,616 | 1,098,616 |
| `209` | `019813` | 348 / 348 | 1,366,944 | 293,016 | 1,073,928 | 1,073,928 |
| `208` | `019813` | 347 / 347 | 1,363,016 | 292,174 | 1,070,842 | 1,070,842 |
| `20a` | `019813` | 330 / 330 | 1,296,240 | 277,860 | 1,018,380 | 1,018,380 |
| `18a` | `018ebe` | 127 / 127 | 1,127,914 | 111,760 | 1,016,154 | 1,076,860 |
| `188` | `018f0c` | 346 / 346 | 1,268,782 | 304,480 | 964,302 | 713,798 |
| `122` | `018ef0` | 1,953 / 1,953 | 2,588,304 | 1,718,640 | 869,664 | 3,933,152 |
| `32f` | `01b395` | 1,599 / 1,599 | 4,147,938 | 3,338,844 | 809,094 | 0 |
| `171` | `019120` | 291 / 291 | 986,962 | 256,080 | 730,882 | 807,124 |
| `1a4` | `018ddc` | 138 / 138 | 789,360 | 121,440 | 667,920 | 1,163,340 |
| `16b` | `018f8a` | 374 / 374 | 879,996 | 329,120 | 550,876 | 657,840 |
| `17c` | `0196e4` | 183 / 183 | 669,780 | 161,040 | 508,740 | 561,078 |
| `16a` | `018f8a` | 340 / 340 | 794,340 | 299,200 | 495,140 | 592,380 |
| `319` | `0198d0` | 308 / 308 | 832,298 | 362,208 | 470,090 | 328,410 |
| `20c` | `019813` | 144 / 144 | 565,632 | 121,248 | 444,384 | 444,384 |
| `20d` | `019813` | 138 / 138 | 542,064 | 116,196 | 425,868 | 425,868 |
| `15d` | `018f36` | 252 / 252 | 1,155,272 | 732,624 | 422,648 | 237,680 |
| `20f` | `019813` | 133 / 133 | 522,424 | 111,986 | 410,438 | 410,438 |
| `150` | `0196dc` | 186 / 186 | 559,256 | 163,680 | 395,576 | 448,772 |

On the final cohort, IDs $61/$62 and $C0/$C2 have zero signed exclusive and inclusive gaps. The unchanged membership of all 63 pre-existing masked-init guarded rows, including the requested 47 leases, is verified in `vertical-4x2/group-structural-check.json`.

Follow the remaining per-ID deficit order before lower ranking rows. The next vertical 2×2 family needs both its caller loops and the distinct $01:B2AF return (516 clocks versus the existing $9895 body at 506); adding the whole alias deficit would double-charge existing tile writes. Sized ID $2F illustrates the ownership issue: its exclusive gap is offset by native $9895 child work, with zero net inclusive gap. $09:C176 still needs branch-sensitive eager cache work kept separate from staged handoffs; $00:E78B still has the two 24-clock entry stores. These candidates were inspected but are not shipped. Other unpriced object bodies and the optional 47-lease cursor handoff remain open. This is the second verified stopping point.

No GPU, --video, cached-av, external Snes9x, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch scripts stayed in `/tmp`; the supplied ledgers/cache stayed read-only. Exact A/V acceptance is not claimed.

## Continuation on `codex/cost-pacing16` (main `8dbe2102`)

Evidence is retained in `target/cost-pacing16/group1/`. This is the first verified CPU-only group: ordinary IDs $60/$92/$93, $63/$64, and the shared $B2AF callers $67/$68/$7D, $78/$7B, $89, $3E/$4B, $3C and $88. It follows descending per-ID deficits, starting at $160, $164 and $163.

`source-listings.txt`, generated with `scripts/rom_block_costs.py` at M16/X16, retains all instruction derivations. $01:8B7E prices 46 setup, the actual $B0CC size-helper branch, 116 per pass, six per taken branch and 42 return. $01:8AA4 prices 88 setup, $B0AC (286), 1,272 per dual-background block, actual taken branches and 42 return. Shared callers price their own executed loops and size children. All additions are lagless-only; no count fitting or scene-specific prices were introduced.

The alias is resolved by distinct native entry helpers: real JSR $01:B2AF calls now own scope $01:B2AF and charge 516 once. $01:9895 retains 506. Both delegate only tile writes to an uncharged helper. The extra ten clocks come from TYA/CLC/ADC/TAY (66), versus four INYs (56). No 506-clock body is charged a second time. The executed-ROM regression covers 73 ordinary IDs, 16 sizes and both pointer planes (2,336 paths), comparing instruction clocks and tilemap bytes. A separate profiling regression checks the 32-call vertical default-size path, its distinct scope, muted previews and unchanged default charges.

`alias-reconciliation.json` verifies all 55,284 $B2AF calls across 13 object IDs on the prior missing cohort: source/native calls and both clock totals match, with no $9895 charge on those objects. IDs $160/$163/$164 now have exact parent call, inclusive and exclusive totals on that same cohort. The ten-clock return difference adds 552,840 clocks across these aliases.

| Stage | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| batch 15 / cartridge | 97,288 | 5,389 | 0 | 0 | 34,255 | 39,546 |
| group 1 / cartridge | 97,370 | 5,307 | 0 | 0 | 34,325 | 39,558 |
| group 1 / live SPC | 97,263 | 6,004 | 590 | 132 | 34,191 | 39,426 |

Both full census gates PASS with 1,473,123 iterations each and unexplained zero; every held count equals P2. Live-SPC executes different work: excess/FP increased from 589/131 to 590/132 in category `07/00`; category `0E/02` gained one missing hold. Cartridge excess/FP remain zero.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 370 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 369 / 369 | 0 / 0 | 0 / 0 | 1,610 / 1,610 | 1,973 / 1,973 |
| `06/00` | 8,196 | 94 / 94 | 0 / 0 | 0 / 0 | 55 / 55 | 135 / 135 |
| `07/0E` | 7,979 | 123 / 123 | 0 / 0 | 0 / 0 | 4,829 / 4,829 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 176 / 176 | 0 / 0 | 0 / 0 | 4,703 / 4,703 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 264 / 264 | 0 / 0 | 0 / 0 | 2,655 / 2,655 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 103 | 0 / 19 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 142 / 142 | 0 / 0 | 0 / 0 | 2,576 / 2,576 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 | 2,666 / 2,664 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 86 / 86 | 0 / 0 | 0 / 0 | 1,649 / 1,649 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 9 / 9 | 0 / 0 | 0 / 0 | 8 / 8 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 27 / 27 | 0 / 0 | 0 / 0 | 364 / 364 | 391 / 391 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 | 232 / 232 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 90 / 90 | 0 / 0 | 0 / 0 | 143 / 143 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 | 28 / 28 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts are source/native and remain within one; all eight stable room/Link-XY/whole-save outcomes match.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14742, "song-end-poll": 639}` | 10 |
| probes | `{}` | 0 |

The 162000–163000 probe retains 27 rupee requests at the same logical iterations. Request-gap changes versus batch 15: none. PCM gap median is 8.253745 frames; all onsets and gaps are retained in `final-probe-metrics.json`. This is native PCM timing evidence.

All 8,339 frozen original missing iterations stay within their source HDMA upper bounds; none loses main instruction clocks. Additional main clocks versus batch 15: 27,052,008. HDMA admission and the 47 masked-init leases were not changed.

Both ordinary builds are warning-free. The default binary matches every record and EOF of the fresh clean-main 200k baseline. Default/ROM-timing prices remain unchanged.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,510 | 1 | 3 |
| reduced | 2,045 | 0 | 2 |

The only default failure is the known spotlight caller-return test. Pacing, profiling, executed-ROM object prices, the SNES library checks and Python audit suites pass.

The remaining missing cohort has 5,250 iterations and 5,307 holds, with zero absent retained routine profiles. `object-deficits-after.json` retains all per-ID and per-child source/native totals and execution counts on this cohort; the signed $01:893C exclusive deficit is 20,062,734 clocks. The batch-15 attribution CSV retains initial per-execution averages and execution counts.

| Subtype / ID | Draw target | Executions source/native | Source exclusive | Native exclusive | Difference | Net inclusive gap |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `169` | `018ec3` | 1,515 / 1,515 | 4,141,524 | 1,333,200 | 2,808,324 | 3,533,550 |
| `1c3` | `018d9e` | 215 / 215 | 1,276,368 | 189,200 | 1,087,168 | 1,087,168 |
| `1c6` | `018fa5` | 1,061 / 1,061 | 1,856,774 | 933,680 | 923,094 | 923,094 |
| `20b` | `019813` | 299 / 299 | 1,174,472 | 251,758 | 922,714 | 922,714 |
| `209` | `019813` | 295 / 295 | 1,158,760 | 248,390 | 910,370 | 910,370 |
| `208` | `019813` | 292 / 292 | 1,146,976 | 245,864 | 901,112 | 901,112 |
| `20a` | `019813` | 274 / 274 | 1,076,272 | 230,708 | 845,564 | 845,564 |
| `18a` | `018ebe` | 99 / 99 | 905,886 | 87,120 | 818,766 | 866,088 |
| `122` | `018ef0` | 1,644 / 1,644 | 2,178,240 | 1,446,720 | 731,520 | 3,328,576 |
| `32f` | `01b395` | 1,430 / 1,430 | 3,709,772 | 2,986,192 | 723,580 | 0 |
| `171` | `019120` | 231 / 231 | 796,042 | 203,280 | 592,762 | 653,284 |
| `1a4` | `018ddc` | 120 / 120 | 686,400 | 105,600 | 580,800 | 1,011,600 |
| `16b` | `018f8a` | 347 / 347 | 812,542 | 305,360 | 507,182 | 606,424 |
| `16a` | `018f8a` | 309 / 309 | 714,254 | 271,920 | 442,334 | 530,708 |
| `17c` | `0196e4` | 143 / 143 | 537,396 | 125,840 | 411,556 | 452,454 |
| `319` | `0198d0` | 265 / 265 | 713,102 | 311,640 | 401,462 | 280,528 |
| `15d` | `018f36` | 225 / 225 | 1,040,220 | 661,536 | 378,684 | 213,534 |
| `20c` | `019813` | 106 / 106 | 416,368 | 89,252 | 327,116 | 327,116 |
| `150` | `0196dc` | 150 / 150 | 457,320 | 132,000 | 325,320 | 368,220 |
| `20d` | `019813` | 104 / 104 | 408,512 | 87,568 | 320,944 | 320,944 |
| `231` | `01a26d` | 94 / 94 | 397,432 | 79,148 | 318,284 | 318,284 |
| `1a1` | `018bf4` | 150 / 150 | 449,012 | 132,000 | 317,012 | 1,309,912 |
| `188` | `018f0c` | 300 / 300 | 1,118,398 | 812,398 | 306,000 | 0 |
| `20f` | `019813` | 98 / 98 | 384,944 | 82,516 | 302,428 | 302,428 |
| `134` | `019136` | 161 / 161 | 427,086 | 141,680 | 285,406 | 327,588 |

Sized ID $32F retains a 723,580-clock exclusive gap and ordinary $188 a 306,000-clock gap, both with zero inclusive gap: native child scopes already carry that work. These are attribution ownership differences, not additive prices. The remaining per-ID deficits and refreshed missing-only routine ranking are retained separately. Other object bodies, $09:C176, the $00:E78B entry prefix and the optional 47-lease cursor handoff remain open. This is a verified stopping point.

No GPU, --video, cached-av, external Snes9x, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch scripts stayed in `/tmp`; the supplied ledgers/cache stayed read-only. Exact A/V acceptance is not claimed.

### Second verified group: rail edges, fills and extended dual-background blocks

Evidence is retained separately in `target/cost-pacing16/group2/`. This group follows the refreshed order: ordinary $69, $C3/$D7, the shared $C5–$CA/$D1/$D2/$D9/$DF–$E8 row wrapper, and extended $08–$0F. It also prices the shared rail-edge entries $22/$5F/$8A. All prices are lagless-only and derive from actual ROM instructions and executed branch/loop counts.

`source-listings.txt` retains the `scripts/rom_block_costs.py` derivations at M16/X16. The $01:8EC3/$8EBE vertical rail bodies price their actual corner branch, 238 per store/control pass, taken branches and final tile/RTS; the horizontal $8EF0/$8EEB entries use the distinct $B2CA child. Size helper $B0AF and corner helper $B191 keep real child scopes and branch prices. $01:8D9E prices dimension increments (108), row setup (64), nine-store blocks (798), row control (186), actual branches and RTS. $01:8FA5 prices only dimension increments and 264 per row plus branches/RTS: its existing $8A44 child retains every block store. Direct extended dispatch $01:9813 prices 98 setup, four 732-clock dual-background columns, three taken branches and RTS in the parent frame, without inventing a JSR scope.

The executed-ROM regression covers 98 ordinary and eight extended IDs, 16 sizes, both pointer planes and three initial corner values for each rail entry (3,648 paths), checking instruction clocks and tilemap bytes. No HDMA admission rule, default/ROM-timing price, source exception, scene threshold or fitted multiplier was changed.

`price-reconciliation.json` verifies every routine row for the 30 reached newly priced IDs on group 1's remaining missing cohort: source/native execution counts and inclusive/exclusive clocks match exactly. This is instruction-work evidence, separate from the logical census and phase checks.

| Stage | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| group 1 / cartridge | 97,370 | 5,307 | 0 | 0 | 34,325 | 39,558 |
| group 2 / cartridge | 97,418 | 5,259 | 0 | 0 | 34,370 | 39,561 |
| group 2 / live SPC | 97,311 | 5,959 | 593 | 133 | 34,236 | 39,429 |

Both full census gates PASS with 1,473,123 iterations each and unexplained zero; every held count equals P2. Live-SPC executes different work and is reported separately: excess/FP increased from 590/132 to 593/133, while missing holds fell from 6,004 to 5,959. Cartridge excess/FP remain zero.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 373 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 341 / 341 | 0 / 0 | 0 / 0 | 1,635 / 1,635 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 176 / 176 | 0 / 0 | 0 / 0 | 4,703 / 4,703 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 264 / 264 | 0 / 0 | 0 / 0 | 2,655 / 2,655 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 106 | 0 / 20 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 142 / 142 | 0 / 0 | 0 / 0 | 2,576 / 2,576 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 | 2,666 / 2,664 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 76 / 76 | 0 / 0 | 0 / 0 | 1,659 / 1,659 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 27 / 27 | 0 / 0 | 0 / 0 | 364 / 364 | 391 / 391 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 | 232 / 232 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 90 / 90 | 0 / 0 | 0 / 0 | 143 / 143 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 12 / 12 | 0 / 0 | 0 / 0 | 28 / 28 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts are source/native and remain within one; all eight stable room/Link-XY/whole-save outcomes match.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14741, "song-end-poll": 640}` | 10 |
| probes | `{}` | 0 |

The 162000–163000 probe retains 27 rupee requests at the same logical iterations. Request-gap changes versus group 1: none. PCM gap median is 8.253745 frames; all onsets and gaps are retained in `final-probe-metrics.json`. This is native PCM timing evidence.

All 8,339 frozen original missing iterations stay within their source HDMA upper bounds; none loses main instruction clocks. Additional main clocks versus group 1: 22,681,648. HDMA admission and the 47 masked-init leases were not changed.

Both ordinary builds are warning-free. The default binary matches every record and EOF of the fresh clean-main 200k baseline. Default/ROM-timing prices remain unchanged.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,510 | 1 | 3 |
| reduced | 2,045 | 0 | 2 |

The only default failure is the known spotlight caller-return test. Pacing, profiling, executed-ROM object prices, the SNES library checks and Python audit suites pass.

The remaining missing cohort has 5,205 iterations and 5,259 holds, with zero absent retained routine profiles. `object-deficits-after.json` retains all per-ID and per-child source/native totals and execution counts on this cohort; the signed $01:893C exclusive deficit is 7,259,086 clocks. The batch-15 attribution CSV retains initial per-execution averages and execution counts.

| Subtype / ID | Draw target | Executions source/native | Source exclusive | Native exclusive | Difference | Net inclusive gap |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `32f` | `01b395` | 1,258 / 1,258 | 3,263,416 | 2,626,868 | 636,548 | 0 |
| `1a4` | `018ddc` | 120 / 120 | 686,400 | 105,600 | 580,800 | 1,011,600 |
| `171` | `019120` | 197 / 197 | 712,254 | 173,360 | 538,894 | 590,508 |
| `16b` | `018f8a` | 319 / 319 | 735,098 | 280,720 | 454,378 | 545,612 |
| `17c` | `0196e4` | 143 / 143 | 537,396 | 125,840 | 411,556 | 452,454 |
| `16a` | `018f8a` | 281 / 281 | 634,858 | 247,280 | 387,578 | 467,944 |
| `15d` | `018f36` | 223 / 223 | 1,033,368 | 657,688 | 375,680 | 211,998 |
| `319` | `0198d0` | 235 / 235 | 630,984 | 276,360 | 354,624 | 246,846 |
| `150` | `0196dc` | 150 / 150 | 457,320 | 132,000 | 325,320 | 368,220 |
| `188` | `018f0c` | 298 / 298 | 1,113,394 | 809,434 | 303,960 | 0 |
| `1a1` | `018bf4` | 143 / 143 | 426,998 | 125,840 | 301,158 | 1,245,664 |
| `231` | `01a26d` | 80 / 80 | 338,240 | 67,360 | 270,880 | 270,880 |
| `170` | `0190f9` | 134 / 134 | 382,676 | 117,920 | 264,756 | 303,080 |
| `1db` | `018f9d` | 253 / 253 | 467,808 | 222,640 | 245,168 | 245,168 |
| `134` | `019136` | 129 / 129 | 354,414 | 113,520 | 240,894 | 274,692 |
| `17a` | `018f8a` | 95 / 95 | 295,130 | 83,600 | 211,530 | 238,700 |
| `179` | `018f8a` | 98 / 98 | 297,284 | 86,240 | 211,044 | 239,072 |
| `1c4` | `018fa2` | 348 / 348 | 510,132 | 306,240 | 203,892 | 203,892 |
| `1da` | `0195ef` | 9 / 9 | 209,644 | 7,920 | 201,724 | 201,724 |
| `1ce` | `01921c` | 2 / 2 | 223,548 | 24,880 | 198,668 | 212,938 |
| `31c` | `01a31c` | 56 / 56 | 250,992 | 65,856 | 185,136 | 185,136 |
| `1a3` | `018c22` | 97 / 97 | 239,842 | 85,360 | 154,482 | 879,896 |
| `1a0` | `018be0` | 94 / 94 | 236,492 | 82,720 | 153,772 | 981,568 |
| `1dd` | `0193dc` | 80 / 80 | 209,244 | 70,400 | 138,844 | 446,004 |
| `1de` | `019429` | 160 / 160 | 277,192 | 140,800 | 136,392 | 136,392 |

Positive exclusive gaps with zero inclusive gaps, including $32F and $188, remain ownership differences rather than additive prices. The next unpriced object IDs and refreshed missing-only routine ranking are retained in this group. Deficit totals use each remaining missing cohort; changes in membership are separate from the fixed-cohort price reconciliation. $09:C176, the $00:E78B entry prefix and the optional 47-lease cursor handoff remain open. This is the second verified stopping point.

No GPU, --video, cached-av, external Snes9x, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch scripts stayed in `/tmp`; the supplied ledgers/cache stayed read-only. Exact A/V acceptance is not claimed.

## Continuation on `codex/cost-pacing17` (main `19d142e9`)

Evidence is retained in `target/cost-pacing17/`. This is a verified partial CPU-only pricing group outside $01:893C. It adds the ordinary $09:C5BB column-scan self work and a reusable hold-targeted ranking. Further actor/tail-body pricing and runtime masked-init cursor handoff remain open.

### Per-iteration hold-targeted attribution

`before-targeted-ranking.json` joins all 5,205 missing iterations to complete retained source/native routine profiles. `hold-targeted-attribution.csv` retains each routine’s signed source-exclusive minus native-exclusive clocks on each iteration, alongside the existing P2 inverse shortfall. No missing profile is treated as zero. The inverse is the minimum extra main work appended after existing checkpoints required to reach **all** source holds on that iteration; it freezes worker share and raster placement. It is diagnostic, never a runtime price. Each routine is independently replaced, so closure counts overlap and cannot be summed.

| Cohort | Missing iterations | Missing holds | Within 1k clocks | Within 5k clocks | Within 20k clocks |
| --- | ---: | ---: | ---: | ---: | ---: |
| Before | 5,205 | 5,259 | 123 | 579 | 1,242 |
| After | 5,202 | 5,256 | 124 | 578 | 1,239 |

Limits are inclusive and cumulative. The before cohort has 5,191 single-missing-hold iterations and 14 iterations missing multiple holds (54 holds beyond one per missing iteration).

| Routine | Independent complete closures | Signed exclusive gap | Ownership / disposition |
| --- | ---: | ---: | --- |
| `$00:f312` | 380 | 98,708,654 | V-counter polling already belongs to the raster barrier; no additive price. |
| `$06:84e2` | 296 | 15,925,104 | Tail-dispatched sprite bodies remain open; existing dispatcher/timers retained. A parent residual is not one body price. |
| `$00:80b5` | 233 | -166,434,932 | Main parent scope aliases priced native child work; signed aggregate is negative. |
| `$02:b1ff` | 146 | 5,356,012 | Mirror-warp tail dispatch remains partially priced; requires branch/body ownership, not one scalar. |
| `$06:dc10` | 111 | 13,432,974 | Existing DC13 drawing, E41E preparation and DC66 shadow owners; no duplicate draw price. |
| `$02:fd87` | 108 | 76,360,320 | Already charged in write_overworld_map8_row_step; missing child scope. |
| `$02:fdf7` | 107 | 40,352,256 | Already charged in write_overworld_map8_row_step; missing child scope. |
| `$00:8888` | 87 | 677,733,016 | Song-upload owner and separately elapsed masked callbacks; not additional main work. |
| `$09:c452` | 77 | 27,050,870 | Existing reset loops and prefix already charged; missing child scope. |
| `$09:c5bb` | 70 | 12,432,688 | Ordinary column path newly priced in this group; full-screen scan retains its existing price. |
| `$02:febb` | 66 | 20,852,418 | Existing decompression price; preview calls intentionally uncharged. |
| `$02:f679` | 58 | 6,063,456 | Existing quadrant/interlace price; missing child scope. |
| `$02:f638` | 58 | 6,061,440 | Existing quadrant/interlace price; missing child scope. |
| `$0e:cb5e` | 47 | 50,331,116 | Existing VWF/text work and ownership need normalization; no raw-gap price. |
| `$0e:c9fd` | 47 | 8,165,912 | Existing VWF/text work and ownership need normalization; no raw-gap price. |
| `$06:e41a` | 46 | 2,573,926 | Coordinate-prep entry alias; E41E body already charged. |
| `$1e:8b19` | 42 | 9,716,374 | Existing RTS dispatcher; individual tail actor bodies remain open. |
| `$02:ff5f` | 40 | 11,187,770 | Existing bank-02 stream-byte price; missing child scope. |
| `$05:b5d3` | 40 | 5,300,654 | Open body/alias audit; no aggregate price admitted. |
| `$06:dc66` | 38 | -9,034,092 | Existing native shadow body; net signed deficit is negative. |
| `$0e:cab8` | 38 | 522,688 | Existing VWF text ownership; no raw-gap price. |
| `$02:f7cb` | 34 | 3,967,488 | Open body/alias audit; no aggregate price admitted. |
| `$02:f595` | 33 | 10,535,580 | Existing quadrant decode price; missing child scope. |
| `$1e:fe78` | 33 | 3,208,684 | Open body/alias audit; no aggregate price admitted. |
| `$01:893c` | 31 | 7,259,086 | Object attribution remains diagnostic; no new object price in this group. |
| `$06:dbf0` | 28 | 2,201,150 | Open body/alias audit; no aggregate price admitted. |
| `$06:d22f` | 27 | 5,274,276 | Open body/alias audit; no aggregate price admitted. |
| `$05:98f5` | 26 | 1,009,466 | Open body/alias audit; no aggregate price admitted. |
| `$02:f54a` | 25 | 3,543,276 | Open body/alias audit; no aggregate price admitted. |
| `$05:d4d4` | 24 | 396,302 | Open body/alias audit; no aggregate price admitted. |

This order differs materially from total-clock ranking. The high-ranked dispatcher tails still need individual-body attribution; this group does not claim that work is complete. Of $C5BB’s 70 raw candidates, 67 carry the already-priced full-screen scan body. Only iterations 242110 (12 clocks short), 506744 (100) and 605875 (10) are direct 112-clock zero-scroll candidates in that raw list. This makes the ordinary wrapper a small, established price group, not a 70-hold forecast. The first admitted ordinary scan price is distinct from its already-priced whole-screen caller. The refreshed ranking and every after-cohort attribution row are retained in `final-targeted-ranking.json`.

### ROM instruction price and ownership

`source-proximity.txt` comes from `scripts/rom_block_costs.py 09c5bb` at M8/X8. $09:C5BB..C5C0 costs 64; zero scroll takes BEQ (+6) and $C5F5 RTS (42), totaling 112. Nonzero scroll adds $C5C2 BPL (16), either taken +6 or negative-path INY (14), and $C5C5..C5E1 setup (332). Each of 22 $C5E3..C5F3 passes costs 222, plus 21 taken branches at six clocks and the final RTS. The ordinary totals are 5,470 positive / 5,478 negative. Only JSR itself is in these self costs: $C6F5 and its children retain their existing prices. The full-screen 22-column path uses `advance_overworld_proximity_cell_scan`, does not call this ordinary wrapper, and retains its existing charges.

The new executed-ROM regression passes all 45 paths (five signed/zero deltas × three X scrolls × three Y scrolls), including wraps. It compares actual instruction-bus clocks outside children with native routine self work and checks zero default self charges. Every addition is lagless-only and has its ROM address in the source comment. No scene thresholds, multipliers, tables fitted to holds, or default/ROM-timing price changes were added.

`price-reconciliation.json` checks the frozen original missing cohort: all 2,343 rows with equal ordinary call counts have exactly equal source/native self clocks (signed gap zero). The 109 unequal-call rows are retained separately because the full-screen scan still owns its existing work outside the ordinary wrapper. No absent ordinary scope is treated as uncharged full-screen work.

### Masked initialization: established hardware and remaining handoff

`masked-handoff-hardware.txt` compares `DmaState::run_synchronous_hdma` with `HdmaTimingState` on actual immutable ROM descriptors $00:F2FB and $0A:BDDD. Both are advanced through actual preceding hardware events (100 or 225 lines), then disabled across initialization and re-enabled. Every event’s clocks and cursor/counter/transfer/termination state agree between the two hardware implementations. These are constructed hardware controls, not claims about a route iteration’s prior cursor.

The ROM disables $420C at $00:8135 and restores it at $00:8222/$82C4. A disabled V0 initialization costs zero, clears do-transfer and termination, and preserves A2A, DAS and NTRL. Late enable only changes the enable bits. The first late-enabled event consequently lacks the payload transfer (34 clocks for the two-channel control, 26 for the one-channel control), but later reload and termination depend on the retained cursor. With identical $F2FB programming, the 100-line control remains active after event 20 (98-clock reload), whereas the 225-line control reloads/terminates at event 15 (90 clocks) and then costs zero. Programming alone cannot select either history.

The current `Program` cache keys programming, not the prior field’s executed A2A/DAS/NTRL/transfer/termination handoff. The logical replay can seed DMA at segment starts, but the general lease does not continuously own those cursors across ordinary/held fields and masked initialization. Therefore this group admits **0 of the requested 47**: admitting them from a fresh descriptor program would be unsupported. All 27 + 20 remain declined; all 63 previously guarded rows retain their membership. Runtime cursor retention, enable-write timing and first-field versus later-field composition remain unfinished.

### Live-SPC excess / false-positive creep

`live-creep-deltas.json` retains **every** excess addition/removal across batch 16, reconciling 589/131 → 590/132 → 593/133. `live-creep-classifications.json` and the retained full logs identify the causal port reads. At 94,848 in group 1 and 142,082 in group 2 the live engine reads APUI00=0 while the source reads $13; their prior live runs first disagreed one logical iteration later. The live song has ended, so the engine advances into text initialization while the source remains in gameplay. The next iteration runs sprite decompression ($00:E772/$E843), then VWF rendering. These are different executed paths, not a pacing-rule coefficient.

| Batch transition | Iteration | Excess before → after | Main clocks before → after | VWF clocks before → after |
| --- | ---: | ---: | ---: | ---: |
| group1 | 94849 | 0 → 5 | 55,372 → 1,676,836 | 0 → 0 |
| group1 | 94852 | 0 → 3 | 109,136 → 1,114,914 | 0 → 1,065,630 |
| group2 | 142083 | 0 → 5 | 57,152 → 1,685,436 | 0 → 0 |
| group2 | 142086 | 0 → 4 | 109,560 → 1,440,718 | 0 → 1,391,010 |

For group 1, the old five-hold decompression iteration 94,850 becomes 94,849; VWF excess changes 94,852: 0→3, 94,853: 3→2, 94,854: 2→1. This leaves **+1 hold / +1 FP**. For group 2, the old five-hold iteration 142,084 becomes 142,083; VWF excess changes 142,086: 0→4 and 142,087: 4→3, leaving **+3 holds / +1 FP**. The crystal/polygon triplets around 380940, 627311 and 1064793 (group 1), and 380941, 462693 and 1064792 (group 2) move by one logical iteration with their song-end cause and contribute **zero net excess / FP**. Every moved row, including removals, is retained. No pacing-rule fix is justified by this creep; no live answer or charge is suppressed.

The windowed profiles are diagnostic only: `--windows` forces seeds where the uninterrupted census may continue, and they do not reproduce all full-census live paths. The uninterrupted 144k prefixes match every predictor column of their respective retained full censuses in all three builds (`creep-prefix-reconciliation.json`). They show $00:E772=1,178,584 and $00:E843=356,250 exclusive clocks in the five-hold decompression iteration; the following heavy VWF iterations contribute 1,065,630 clocks near 94,852 and 1,391,010 near 142,086. Additional presentation holds advance the live SPC, changing the song-end read and therefore which native work runs next. This explains why instruction additions can shift live-SPC work while the cartridge-answer census retains zero excess/FP.

The current pricing group changes live excess/FP 593/133 → 593/133; `current-live-creep.json` retains all 12 changed excess rows and every before/after song-poll observation. These are the 1/4/8-hold crystal/polygon triplets near 380940 and 627311 moving in opposite directions by one iteration; both triplets contribute zero net excess/FP.

### Full census verification

| Stage | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| batch 16 / cartridge | 97,418 | 5,259 | 0 | 0 | 34,370 | 39,561 |
| group / cartridge | 97,421 | 5,256 | 0 | 0 | 34,373 | 39,561 |
| group / live SPC | 97,314 | 5,956 | 593 | 133 | 34,239 | 39,429 |

Both full census gates PASS: 1,473,123 iterations each, unexplained zero, every runtime held count equal to P2. These are periodically seeded logical CPU gates; exact A/V acceptance is not claimed.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 373 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 341 / 341 | 0 / 0 | 0 / 0 | 1,635 / 1,635 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 176 / 176 | 0 / 0 | 0 / 0 | 4,703 / 4,703 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 264 / 264 | 0 / 0 | 0 / 0 | 2,655 / 2,655 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 106 | 0 / 20 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 142 / 142 | 0 / 0 | 0 / 0 | 2,576 / 2,576 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 3 / 4 | 0 / 5 | 0 / 0 | 2,666 / 2,664 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 76 / 76 | 0 / 0 | 0 / 0 | 1,659 / 1,659 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 27 / 27 | 0 / 0 | 0 / 0 | 364 / 364 | 391 / 391 |
| `0F/00` | 449 | 55 / 55 | 0 / 0 | 0 / 0 | 232 / 232 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 34 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts are source/native and within one. All eight stable room/Link-XY/whole-save outcomes match.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14741, "song-end-poll": 640}` | 10 |
| probes | `{}` | 0 |

The 162000–163000 probe retains 27 rupee requests at the same logical iterations. Request-gap changes versus batch 16: none. Native PCM gap median is 8.253745 frames; all onsets and gaps are in `final-probe-metrics.json`.

All 8,339 frozen original missing iterations stay within source HDMA upper bounds, with zero negative instruction-work rows. Additional main clocks on that cohort: 3,261,484. The 47 requested leases remain declined.

Both ordinary builds are warning-free. The default binary matches every record and EOF of a fresh 200k default baseline built from clean main `19d142e9` before editing.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,511 | 1 | 3 |
| reduced | 2,046 | 0 | 2 |

The default failure is only the known spotlight caller-return test. The default lib-test compilation retains the same 24 pre-existing warnings as batch 16; both ordinary binary builds are warning-free. Reduced, SNES library, executed-ROM proximity, pacing/profile and Python shortfall/coverage checks pass. `git diff --check` passes.

This is the verified stopping point for the ordinary column group. The hold-targeted report is complete for the initial cohort; further higher-ranked actor/tail-body pricing and production masked-init cursor handoff remain open.

No GPU, --video, cached-av, external Snes9x, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache stayed read-only.

## Continuation on `codex/cost-pacing18` (main `36610d13`)

Evidence is retained in `target/cost-pacing18/raster/`. This is the verified CPU-only raster-barrier group. Native child-scope normalization, sprite tail-body attribution/pricing and masked-init cursor handoff remain open. Prepared scratch profiling edits were not applied.

### Raster barrier: masked counter and bus sample

The ROM target is still line 192. The missing rule is `$00:F3AC AND #$00FF`: physical V256..261 reads as 0..5, so `$00:F3AF CMP #$00C0` / `$F3B2 BCC` waits through V0 to the next V192. The old native comparison used the unmasked physical counter and returned immediately. This is a field-counter wrap, not an extra instruction-body price or a scene-dependent target.

The generic barrier now retains the counter mask and instruction sample offset with each main-work checkpoint. `$00:F3A3 LDA $2137` latches after three eight-clock ROM fetches, at +24 clocks; H1339 on V255 still samples V255, while H1340 samples V256. The same rule is used by the ordinary wall-time owner and immutable-HDMA composition. The existing 178-clock taken pass and 172-clock final pass are unchanged. All 403 retained before-cohort F312 gaps are exact multiples of 178, consistent with omitted taken polling passes; this diagnostic did not set a runtime charge. HDMA/refresh inside waiting remain owned by the wall interval. No per-scene threshold, multiplier, lag-fitted table or instruction charge was added.

`source-raster-poll.txt` is the `scripts/rom_block_costs.py` listing. The executed-ROM regression covers seven physical lines (0, 191, 192, 225, 255, 256, 261) at six horizontal positions, including the last clock before and exact clock of the latch crossing. All 42 cases pass; the pre-existing executed-ROM HDMA/barrier composition tests also pass. `poll-edges-final.txt` retains the standalone CPU-only receipts. The initial fixture needed an APU bus owner because M16 LDA $213F also reads $2140; that fixture issue was corrected.

The mask-only trial and its partial censuses at `target/cost-pacing18/` were interrupted when the latch-edge check established the additional sample-offset rule. They are diagnostic only. Both final censuses below restarted from iteration zero on the final captured binary; the fresh main default baseline was built before edits.

`raster-deltas.json` retains every changed cartridge predictor row: 1,389 rows, 218 formerly missing iterations closed, 218 additional holds and 0 instruction-work changes. The signed raster-wait delta is 57,824,546 clocks. No cartridge excess was introduced.

The retained F312 before-cohort still has 185 missing iterations. Its raw replacement ranking has 165 independent closures: 163 with zero native raster wait and 2 with a nonzero wait. All remaining signed instruction-profile gaps are multiples of 178. `remaining-raster-ownership.json` retains the rows. The remaining lead is entry placement and preceding-work/interrupt ownership; this group does not claim all raster discrepancies are resolved, and adds no polling/body scalar.

### Upload / masked callback accounting

The independently captured audio branch was read at `/Users/missingno/Documents/zelda3-rs-codex/docs/parity/lagless-audio-vs-snes9x.md`; its capture is a different binary/history from this batch. `audio-branch-upload-accounting.json` reconciles its exact 2,474-window residual. Source spans contain 5,198 windows beyond one ordinary callback plus the accepted lag NMIs. Native emits 2,724 masked callbacks: 5,198 - 2,724 = 2,474.

On the 114 iterations that actually emit masked callbacks, source non-lag extras total 2,674 against 2,724 native masked callbacks, a signed residual of -50. The other iterations contribute +2,524, giving +2,474 overall. These are signed host-span counts; normal host-boundary phase contributes positive and negative rows, so they do not establish 2,474 missing uploads. The retained report includes every masked or unusually wide source-span row.

Upload/masked-NMI windows are only partly presented. `audio.rs::begin_runtime_song_bank_transfer` starts the native masked presentation owner only under `bank_id == 0`; bank 1/2 receivers can run without that owner. The 114 wide source spans with no native masked callback are 99 in `06/00` (2,199 extra source windows), 13 in `11/00` (292), one in `1B/02` (22), and one in `19/00` (15): 2,528 in total. Normal unmasked host-boundary phase contributes -4, and the existing masked group contributes -50, yielding 2,528 - 4 - 50 = 2,474. This is a concrete missing ownership lane, not grounds to price the full `$00:8888` CPU residual as main work. Its bank-1/2 runtime handoff fix remains open. For the bank-0 lane, `native_mask_song_upload_nmi` marks the first masked callback; `run_native_song_upload_frame` advances the host counter, captures display state and pushes APU state on each continuing transfer callback. The replay loop counts these as `native_elapsed` and calls `zelda_render_audio` for them. They are separate from accepted held NMIs. No blanket upload/main surcharge or live-SPC pacing suppression was added.

For the frozen 87 `$00:8888` closure candidates, this batch separately presents masked callbacks on 21 iterations and presents none on 66; their masked-callback total is 501. `upload-closure-cohort.json` retains every iteration. In the independent audio capture, the same IDs have 66 without masked callbacks, 21 with them, 1,921 source non-lag extras and 503 native masked callbacks.

This batch's live excess/FP is 593/133 -> 589/131; every changed excess row and the before/after source-port observations are retained in `current-live-creep.json`. Live-SPC executes different paths and is reported separately.

### Full census verification

| Stage | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| batch 17 / cartridge | 97,421 | 5,256 | 0 | 0 | 34,373 | 39,561 |
| raster / cartridge | 97,639 | 5,038 | 0 | 0 | 34,591 | 39,561 |
| raster / live SPC | 97,532 | 5,734 | 589 | 131 | 34,457 | 39,429 |

Both complete census gates PASS: 1,473,123 iterations each, unexplained zero, every runtime held count equal to P2. These are seeded logical CPU gates; full A/V acceptance is not claimed.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 369 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 341 / 341 | 0 / 0 | 0 / 0 | 1,635 / 1,635 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 84 / 84 | 0 / 0 | 0 / 0 | 4,795 / 4,795 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 264 / 264 | 0 / 0 | 0 / 0 | 2,655 / 2,655 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 102 | 0 / 18 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 76 / 76 | 0 / 0 | 0 / 0 | 1,659 / 1,659 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 27 / 27 | 0 / 0 | 0 / 0 | 260 / 260 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts are source/native and within one. All eight stable room/Link-XY/whole-save outcomes match.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14742, "song-end-poll": 638}` | 10 |
| probes | `{}` | 0 |

The 162000–163000 probe retains 27 rupee requests at the same logical iterations. Request gaps and PCM onsets/gaps are unchanged versus batch 17; PCM gap median is 8.253745 frames. Full values are in `final-probe-metrics.json`.

All 8,339 frozen original missing iterations stay within source HDMA upper bounds, with zero negative instruction-work rows. Instruction-work delta on that cohort is 0. The same 47 requested masked-init leases remain declined.

`final-targeted-ranking.json` re-ranks the remaining missing cohort, with complete retained source/native profiles. This is a refreshed raw ranking; the requested native child-scope normalization has not shipped, so its parent/child aliases must still be resolved before pricing.

| Routine | Independent complete closures | Signed exclusive gap |
| --- | ---: | ---: |
| `$06:84e2` | 302 | 15,352,924 |
| `$00:80b5` | 237 | -168,477,806 |
| `$00:f312` | 165 | 45,479,534 |
| `$02:b1ff` | 147 | 5,356,012 |
| `$06:dc10` | 115 | 13,268,610 |
| `$02:fd87` | 108 | 76,360,320 |
| `$02:fdf7` | 107 | 40,352,256 |
| `$00:8888` | 87 | 677,733,016 |
| `$09:c452` | 77 | 27,050,870 |
| `$09:c5bb` | 67 | 9,279,732 |
| `$02:febb` | 66 | 20,852,418 |
| `$02:f679` | 58 | 6,063,456 |
| `$02:f638` | 58 | 6,061,440 |
| `$0e:cb5e` | 47 | 50,331,116 |
| `$0e:c9fd` | 47 | 8,165,912 |
| `$06:e41a` | 46 | 2,421,680 |
| `$1e:8b19` | 44 | 9,555,400 |
| `$05:b5d3` | 42 | 5,256,982 |
| `$02:ff5f` | 40 | 11,187,770 |
| `$06:dc66` | 40 | -8,753,922 |

Both ordinary binary builds are warning-free. The default binary matches every record and EOF of the fresh 200k default baseline built from clean main `36610d13` before editing.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,512 | 1 | 3 |
| reduced | 2,047 | 0 | 2 |

The only default failure is the known spotlight caller-return test; reduced passes completely. Both SNES library suites, pacing/profile/executed-ROM checks and Python shortfall/coverage tests pass. Default lib-test compilation retains its pre-existing warnings; ordinary builds emit none. `git diff --check` passes.

This is the verified stopping point for the raster group. Task 2 child scopes, task 3 sprite-type attribution/pricing and task 5 cursor handoff remain unfinished; task 4 has the bank-specific callback-accounting result above; its bank-1/2 runtime correction remains open.

No GPU runs, --video, cached-av, external Snes9x execution, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push were used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache stayed read-only.

## Continuation on `codex/cost-pacing19` (main `24ce17d4`)

Evidence is retained in `target/cost-pacing19/children/`, with fresh main controls in its parent directory and a rejected bank-gate trial in `gate-trial/`. This is a verified **profiler-only group**: instruction work, holds, masked callbacks and every predictor column are unchanged against fresh main on both full censuses. The elapsed-field owner and remaining raster placement are still open. Native sprite-body attribution is available by type/state/slot; source body derivation and new actor prices have not shipped.

### Elapsed masked fields: rejected gate-only trial

The trial removed the bank-0-only presentation gate in `audio.rs` and selected native runtime upload streams for every bank in `misc.rs`. Both complete trial census gates PASS, with 1,473,123 iterations each and unexplained zero. Its diff, binaries and receipts are retained, but both trial source edits were reverted. Passing the logical gate does not establish continuous field coverage or correct command and handler timestamps.

The trial emits 5,297 cartridge masked callbacks and 5,272 live-SPC callbacks. Against strict source interiors, the per-iteration count comparison has 17 missing / 206 excess for cartridge and 109 / 273 for live SPC. These are comparisons with the strict interior class, not proof that those excess callbacks are false physical fields: the source also has 117 loader-edge hosts, isolated handler cuts and multiple-acceptance hosts. Live-SPC work can move between contexts. `elapsed-accounting.json` retains every affected iteration and the separate held-NMI missing/excess/FP counts. No residual total was used as a production target.

| Context | Source strict | Episode edges | Shipped masked rom/live | Rejected trial masked rom/live |
| --- | ---: | ---: | ---: | ---: |
| `06/00` | 2,163 | 50 | 0 / 0 | 2,245 / 2,241 |
| `08/00` | 2,513 | 54 | 2,602 / 2,505 | 2,601 / 2,506 |
| `08/01` | 0 | 0 | 0 / 23 | 0 / 23 |
| `10/00` | 0 | 0 | 0 / 25 | 0 / 25 |
| `11/00` | 282 | 8 | 0 / 0 | 292 / 293 |
| `15/00` | 22 | 0 | 24 / 24 | 24 / 24 |
| `16/04` | 0 | 0 | 0 / 23 | 0 / 23 |
| `17/01` | 69 | 2 | 72 / 72 | 71 / 72 |
| `18/00` | 23 | 1 | 24 / 24 | 24 / 24 |
| `19/00` | 15 | 1 | 0 / 0 | 17 / 18 |
| `1B/02` | 21 | 1 | 0 / 0 | 23 / 23 |

The 5,108 strict interiors / 5,225 complete-episode source hosts come from the independent audio-branch receipt scan, whose input hashes are retained. They are an investigation catalogue. Its 2,724 native callback count belongs to that independent capture; the fresh main controls here emit 2,722 cartridge and 2,696 live masked callbacks, unchanged by this group. Source held-surface/forced-blank evidence does not authorize a duration table or an atomic CPU return.

The gate change alone loses the required phase in four concrete places:

- `native_defer_song_upload` retains only `floor(elapsed / field)` and `(bank, stream)`, without the fractional caller position.
- `start_deferred_song_bank_transfer` uses the unpositioned receiver entry; the retained probe logs `song_upload unpositioned_command ... bank=1`. The positioned command API already has passing CPU-only source tests, but this lagless path does not use it.
- `run_native_song_upload_frame` checks whether the receiver was pending before the audio window instead of consuming its actual return forecast. The existing return-phase test proves the forecast can identify a return while the live receiver remains pending. Running one ordinary closing NMI on the following callback does not establish the real active-vblank enable edge.
- Main executes its semantic suffix before these presentation callbacks, and `native_pacing` is serde-skipped. There is no retained suspended CPU caller/phase through these fields and restores. Enabling banks 1/2 alone cannot satisfy the source-ordered register/handshake, latch and restore requirements.

`source-{dungeon-upload,overworld-upload,receiver}.txt` were generated with `scripts/rom_block_costs.py`. The positioned-command and receiver-return source tests pass. No edits were made to `spc_driver_clock.rs`, including its transport code.

| Capture | Held | Masked | Missing holds | Excess holds | FP holds | Callback-count duration component, pairs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Shipped / rom | 97,639 | 2,722 | 5,038 | 0 | 0 | -4,012,476 |
| Shipped / live | 97,532 | 2,696 | 5,734 | 589 | 131 | -4,083,498 |
| Rejected gate trial / rom | 97,639 | 5,297 | 5,038 | 0 | 0 | -2,637,426 |
| Rejected gate trial / live | 97,532 | 5,272 | 5,734 | 589 | 131 | -2,707,914 |

The duration component is `534 * (native callbacks - 1,580,998 source hosts)`, with callbacks counted as iterations + held + masked. The 534-pair output budget is unchanged. Source host cuts/terminal conventions remain analysis corrections; no gameplay holds implement them.

### Source call frames and sprite-body attribution

Bank-02 decompression now gives `$02:FEBB` and its actual `$02:FF5F` byte-reader calls their own profile frames. The actual interlace calls use `$02:F638` and `$F679`. Map8 row fragments retain the actual `$02:FD87` call frame, with the first-half `$FDF7` JSR distinguished from the second-half fallthrough. The provisional `$F595` and `$FD46` parent containers were removed after source reconciliation: the shared quadrant helper mixes caller frames, and `$FD46` is not a source call entry. Their retained caller ancestry remains open; no invented parent entries replace it. The existing 270-clock call/prologue charge is partitioned as 46 caller clocks + 224 child clocks; total work is unchanged.

Profile guards distinguish a real entry from a continued fragment, and avoid inventing recursive source calls when a native helper resumes the same frame. `$09:C44E` retains the reset body reached by fallthrough; a complete direct no-disable call enters `$09:C452`. Flattened reset continuations without a retained source call frame remain unresolved rather than being labelled as invented `$C452` calls. Ordinary non-profile ledger scopes are preserved.

`Sprite_ExecuteSingle` no longer reports its translated `after_entry` helper as a second source invocation during profiling. Its inactive body is entered by `$06:84E5 BEQ`, so the already-priced `$06:8510` tail stays in the `$06:84E2` source frame. For iteration 44,264, source `$0684E2` exclusive work is 4,258; the previous native row was 2,344 with 1,846 already priced under `$068510`. The corrected row is 4,190, leaving 68, not a new 1,914-clock price. Native calls fall from 32 to the source count of 16. No instruction charge was added.

| Routine | Source exclusive | Before native exclusive | After native exclusive | Exact after rows / reached rows |
| --- | ---: | ---: | ---: | ---: |
| `$02:f638` | 50,872,800 | 0 | 50,872,800 | 417 / 417 |
| `$02:f679` | 50,889,720 | 0 | 50,889,720 | 417 / 417 |
| `$02:fd87` | 116,661,600 | 0 | 116,661,600 | 165 / 165 |
| `$02:fdf7` | 61,649,280 | 0 | 61,649,280 | 165 / 165 |
| `$02:febb` | 179,646,312 | 0 | 178,111,600 | 417 / 421 |
| `$02:ff5f` | 107,796,880 | 0 | 107,209,400 | 417 / 421 |
| `$06:84e2` | 57,567,882 | 33,596,836 | 46,994,106 | 383 / 7,879 |
| `$06:8510` | 0 | 13,397,270 | 0 | 0 / 0 |
| `$09:c452` | 38,292,790 | 0 | 38,292,790 | 109 / 109 |

All 8,339 retained native profiles conserve instruction work exactly: exclusive routine work + the separately reported VWF work equals main work. The table reports actual source/native agreement, including remaining mismatches; it does not declare every parent/child alias resolved.

`--routine-profile PATH` additionally writes `PATH` with its extension replaced by `.sprites.csv`. It captures type, saved dispatch state and slot at the state-body entry, after entry/timer/dispatch work. All 127,856 recorded (iteration, type, state, slot) groups reconcile their child/self charges with their body totals. Body totals partition charges by the current body identity: nested body work belongs to the nested selection. Nested bodies restore their caller's identity; muted previews add neither calls nor work. A nonzero body total can include only dispatch or priced children and is not proof of complete actor-body pricing. State-zero rows share the inactive body; their stale type bytes are not new actor timing classes.

| Entry type | State | Body calls | Body work | Self work in $0684E2 | Zero-charged bodies |
| --- | --- | ---: | ---: | ---: | ---: |
| `$41` | `$09` | 1,878 | 15,718,126 | 683,592 | 0 |
| `$24` | `$09` | 2,066 | 7,117,510 | 676,740 | 0 |
| `$8a` | `$09` | 1,831 | 6,514,878 | 666,484 | 0 |
| `$be` | `$09` | 1,702 | 6,090,568 | 619,528 | 0 |
| `$9d` | `$09` | 1,589 | 6,061,188 | 578,396 | 0 |
| `$6c` | `$09` | 1,506 | 4,716,000 | 548,184 | 0 |
| `$55` | `$09` | 1,384 | 4,991,678 | 503,776 | 0 |
| `$25` | `$09` | 1,611 | 2,264,974 | 441,330 | 0 |
| `$0b` | `$09` | 1,474 | 9,014,272 | 386,530 | 0 |
| `$1e` | `$09` | 1,174 | 3,092,040 | 384,060 | 0 |
| `$5b` | `$09` | 913 | 3,353,784 | 332,332 | 0 |
| `$8b` | `$09` | 887 | 3,517,348 | 322,868 | 0 |
| `$a7` | `$09` | 855 | 4,279,688 | 311,220 | 0 |
| `$d1` | `$09` | 853 | 3,900,336 | 310,492 | 0 |
| `$80` | `$09` | 851 | 2,692,206 | 309,764 | 0 |
| `$e3` | `$09` | 943 | 1,900,002 | 294,040 | 0 |
| `$a5` | `$09` | 757 | 5,367,636 | 275,548 | 0 |
| `$c2` | `$09` | 748 | 9,774,070 | 272,272 | 0 |
| `$cf` | `$09` | 726 | 5,534,540 | 264,264 | 0 |
| `$8f` | `$09` | 725 | 2,035,762 | 263,900 | 0 |

The missing-only counterfactual ranking was refreshed after attribution, without changing its missing population or using it for prices:

| Routine | Before independent closures | After independent closures | After signed exclusive gap |
| --- | ---: | ---: | ---: |
| `$06:84e2` | 302 | 60 | 8,538,160 |
| `$00:80b5` | 237 | 237 | -5,632,124 |
| `$00:f312` | 165 | 165 | 45,479,534 |
| `$02:febb` | 66 | 0 | 1,534,712 |
| `$02:ff5f` | 40 | 0 | 587,480 |
| `$02:fd87` | 108 | 0 | 0 |
| `$02:fdf7` | 107 | 0 | 0 |
| `$09:c452` | 77 | 0 | 0 |

The retained F312 cohort remains 185 missing iterations and 165 independent raw closures: 163 with zero native raster wait, two with nonzero wait. The sample/mask barrier and all raster prices are unchanged. `$00:80B5` is a JML dispatcher whose source frame also retains module work; its eight dispatch instructions cost 240 clocks and are already charged. Its raw exclusive residual cannot be priced as a second dispatcher charge. Entry placement and preceding-work/interrupt ownership still require a runtime correction.

### Full CPU-only verification

Both final census gates PASS, each with 1,473,123 iterations and unexplained zero. Every runtime held count equals P2. Every predictor row/column and every classification byte matches the corresponding fresh main control, separately for cartridge answers and live SPC. These are seeded logical CPU gates; continuous physical-field coverage and exact A/V acceptance are not claimed.

| Mode | Held | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 97,639 | 5,038 | 0 | 0 | 34,591 | 39,561 |
| live | 97,532 | 5,734 | 589 | 131 | 34,457 | 39,429 |

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 369 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 341 / 341 | 0 / 0 | 0 / 0 | 1,635 / 1,635 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 84 / 84 | 0 / 0 | 0 / 0 | 4,795 / 4,795 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 264 / 264 | 0 / 0 | 0 / 0 | 2,655 / 2,655 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 102 | 0 / 18 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 76 / 76 | 0 / 0 | 0 / 0 | 1,659 / 1,659 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 27 / 27 | 0 / 0 | 0 / 0 | 260 / 260 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts are source/native and within one. The eight stable room/Link-XY/whole-save outcomes match. The complete probe outcomes and PCM are byte-identical to fresh main (`probe-byte-reconciliation.json`); the 27 rupee requests, gaps and PCM onsets in 162000–163000 are unchanged. PCM gap median is 8.253745 frames.

| Mode | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14742, "song-end-poll": 638}` | 10 |
| probes | `{}` | 0 |

All 8,339 original missing rows retain their source HDMA upper bounds and the same 47 declined masked-init leases. Both ordinary builds are warning-free. The final default binary matches every record and EOF of the fresh 200k main default state/display/SRAM/PCM/input/context ledger. Default/ROM-timing pricing and state/audio ledger behavior are unchanged.

| Configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,514 | 1 | 3 |
| reduced | 2,049 | 0 | 2 |

The only default failure is the known spotlight caller-return test; reduced passes completely. Both SNES library suites, profile/pacing/executed-ROM checks, song-bank source checks and Python coverage tests pass. Lib-test fixture warnings remain separate from warning-free ordinary builds. The earlier `profiles/` and `final/` candidates were superseded during the tail/parent audit; a recovery attempt correctly rejected its stale source boundary. Only `children/` binaries and manifests are delivery evidence.

This is the verified stopping point for the profile group. The bank-1/2 masked-owner integration, generic retained masked CPU phase/restores, real NMI-enable edges, remaining raster ownership and source-derived actor-body prices remain open. No GPU runs, --video, cached-av, external Snes9x execution, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push were used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache remained read-only. `git diff --check` passes.

## Continuation on `codex/cost-pacing20` (main `382264f6`)

Evidence is retained in `target/cost-pacing20/callers/`; fresh main controls and rejected upload-placement trials are in `target/cost-pacing20/`. This is a verified **spotlight caller-pricing group**, not acceptance of the elapsed masked-field owner or completion of the raster/actor work. No all-bank upload change is shipped.

### ROM-derived preceding work

`IrisSpotlight_close` now prices its missing `$00:F28B..F293` entry: REP, LDY, LDX and BRA, **92 clocks**. The existing shared `$F29D` prefix, `$F312` child and return keep their prices; the new caller frame correctly owns the shared instructions without pricing them again. Its self work is 1,016 clocks indoors / 1,130 outdoors.

`Dungeon_PrepExitWithSpotlight` now prices the instructions actually executed at `$02:99CA..9A18`: worker/polygon clears, indoor/outdoor work, entrance/progress/current-music branches, HUD and spotlight calls, and the submodule/return suffix. The worker-disable store is charged before its ownership checkpoint; the post-spotlight suffix is charged after the table returns. Each child still owns its own work. These additions are explicitly lagless-only. No charge is selected from a lag count, scene threshold or residual.

`source-{exit,close}-caller.txt` retain `scripts/rom_block_costs.py` listings. The new CPU-only ROM-path test compares caller self work on all **36** indoor/outdoor × entrance-track × progress × current-music fixtures. It executes the unmodified spotlight child with hardware OAM and APU ports, including the high-byte reads of `$2137` and `$213F`; no register response or child instruction is replaced. Both feature configurations pass. The initial fixture failures from unbound OAM/APU ports are retained separately.

### Elapsed-owner investigation: rejected placement candidates

The experiment retained the command remainder instead of only the integer prefix holds, placed `$02:855D/$02:9BFF` on the output clock, selected native streams for all banks, and forecast receiver return through the current output window. The first version exposed an additional ordering error: starting the receiver on the last prefix callback placed the command in the preceding audio interval. Deferring that start to the next callback corrected this one-field error without a route table.

The corrected placement candidate passes all eight phase probes within one, but still has per-iteration masked counts outside the source episode-edge envelope. Its interrupted cartridge/live prefixes contain 391,082 / 390,796 rows; they are rejected diagnostics, not full gates. The preceding candidate contains 728,191 rows and also fails phase tolerance. Their binaries, diffs, logs, probes and prefix extents are retained. All experimental runtime edits were reverted before the caller-pricing group was built.

The cached source receipt at host 37,686 has `NmiAccepted::LatchHeld`, `NmiHandlerCompleted`, the common-suffix completion, then `NmiAccepted::Open` in the same host. A single unmasked callback does not establish both hardware acceptances or the pending enable edge. The suspended semantic caller, restored fractional phase, real `$4200` writes, and source command/acceptance timestamps remain required. The 206 excess / 17 missing gate-trial iterations have **not** all been assigned proven individual causes; count sensitivity alone is not that attribution.

### Raster and actor frontier

The V-counter barrier and its `$00:F3AC` eight-bit mask/sample rule are unchanged. Before this group, a matched CPU-only probe at iteration 44,264 reaches the native poll at V254 (350,102 instruction clocks, 398,072 elapsed clocks before the poll); zero wait is correct at that entry. Missing preceding instructions can move it through the physical V256 wrap and produce a long wait. The new prices above come from those callers, not a fitted raster adjustment.

The retained 185-row raster cohort has 183 zero-wait entries before / **180 after**; 3 wait values change and 182 rows still have missing holds. Within the 165 raw parent-gap closures, zero-wait entries are 163 before / 163 after; 165 still have missing holds. This does not resolve the complete raster frontier. Per-row waits and new shortfalls are in `verification.json`.

No sprite-body price is added. `actor-cohort-inventory.json` narrows the native type/state body inventory to the 60 retained `$06:84E2` raw-gap closures. Counts in this inventory are not source body attribution or independent per-type closures; source derivation and descending closure-order pricing remain open.

### Complete CPU-only verification

Both final full census gates PASS with 1,473,123 iterations each and unexplained zero. Every runtime held count equals P2. Both fresh main controls also pass; they use the freshly built main default binary through the CLI's ROM-less logical-replay entry, rather than borrowing batch-19 totals. These are seeded logical CPU gates, not continuous physical-field or exact A/V proof.

| Capture | Held | Masked | Missing | Excess | FP | Exact lagged | Within one lagged | Audio callback component, pairs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| main / rom | 97,639 | 2,727 | 5,038 | 0 | 0 | 34,591 | 39,561 | -4,009,806 |
| main / live | 97,532 | 2,704 | 5,734 | 589 | 131 | 34,457 | 39,429 | -4,079,226 |
| final / rom | 97,642 | 2,729 | 5,035 | 0 | 0 | 34,594 | 39,561 | -4,007,136 |
| final / live | 97,535 | 2,697 | 5,731 | 589 | 131 | 34,460 | 39,429 | -4,081,362 |

The callback component is `534 * (iterations + held + masked - 1,580,998 source hosts)`. The 534-pair output budget is unchanged. Host cuts and terminal conventions remain analysis corrections. This table does not claim the rejected elapsed owner was integrated.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 369 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 341 / 341 | 0 / 0 | 0 / 0 | 1,635 / 1,635 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 84 / 84 | 0 / 0 | 0 / 0 | 4,795 / 4,795 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 264 / 264 | 0 / 0 | 0 / 0 | 2,655 / 2,655 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 102 | 0 / 18 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 76 / 76 | 0 / 0 | 0 / 0 | 1,659 / 1,659 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 | 0 / 0 | 0 / 0 | 33 / 33 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 24 / 24 | 0 / 0 | 0 / 0 | 263 / 263 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts are source/native and within one. All eight stable room/Link-XY/whole-save outcomes have source matches. In 162000–163000, all 27 rupee requests and all request/PCM onset gaps match fresh main; the PCM gap median remains 8.253745 frames. Probe PCM and classification files are byte-identical to fresh main; only three `main_master` diagnostic fields change in the outcome JSONL.

| Capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| main / rom | `{"lag": 14}` | 10 |
| main / live | `{"lag": 14, "polygon": 14741, "song-end-poll": 638}` | 10 |
| main / probes | `{}` | 0 |
| final / rom | `{"lag": 14}` | 10 |
| final / live | `{"lag": 14, "polygon": 14742, "song-end-poll": 638}` | 10 |
| final / probes | `{}` | 0 |

All 8,339 original missing rows stay at or below their source HDMA bounds, and the same 47 masked-init leases remain declined. Both ordinary builds are warning-free. The final default binary matches every record and EOF of the fresh main 200k state/display/SRAM/PCM/input/context ledger. Default/ROM-timing state and audio behavior are unchanged.

| Library configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,519 | 1 | 3 |
| reduced | 2,054 | 0 | 2 |

The only default failure is the known spotlight caller-return test; reduced passes completely. Both SNES library suites and the Python routine-coverage tests pass. Lib-test fixture warnings remain separate from the warning-free ordinary builds. Library results are the checked disjoint union of completed original prefixes and newly executed tails; in-flight tests finish before their original workers stop. `library-shards.json` retains every test name/status and binary hash. After the environment denied individual signals, tail completion is established by complete Rust test footers and exhaustive named statuses; unreaped child exit codes are not inferred. The repaired test-only bus fixture is bound separately in `source-binding.json`; every production source and ordinary binary stays fixed across the final censuses.

This is the verified stopping point for the caller-pricing group. The all-bank elapsed-field owner, individual attribution of the remaining gate-trial errors, retained CPU phase/restores and real NMI-enable edges, the remaining raster lead, and source-derived sprite bodies remain open. No GPU runs, --video, cached-av, external Snes9x execution, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push were used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache remained read-only. `git diff --check` passes.

## Continuation on `codex/cost-pacing21` (main `7f792f88`)

Evidence is retained in `target/cost-pacing21/group1/`, with fresh main controls in `target/cost-pacing21/`. This first verified group prices `Sprite_ReturnIfLifted`; it does not complete the remaining gameplay or masked-field owners. The latter stays with `codex/masked-fields`; no audio/upload source was edited.

### Restricted ranking and ROM-derived lift checks

The before population is exactly the **2,832 missing iterations** in `07/00` (1,673) and `09/00` (1,159). `initial-ranking.json` retains each iteration’s shortfall and signed source-minus-native exclusive gap for every routine, using the corrected child scopes from batch 19. Independent replacements overlap and are diagnostic, not additive instruction prices.

| Routine | Independent closures | Signed exclusive gap |
| --- | ---: | ---: |
| `$06:f228` | 7 | 3,624,854 |
| `$06:dc10` | 5 | 10,956,776 |
| `$06:84e2` | 2 | 5,957,890 |
| `$06:fb2a` | 2 | 1,223,848 |
| `$08:8eed` | 2 | 247,970 |
| `$00:80b5` | 1 | 6,778,904 |
| `$07:d077` | 1 | 6,451,452 |
| `$07:807f` | 1 | 6,090,466 |
| `$07:e245` | 1 | 5,926,830 |
| `$07:b7c7` | 1 | 3,638,476 |
| `$07:cdcb` | 1 | 3,134,950 |
| `$07:9873` | 1 | 2,020,302 |
| `$07:cf12` | 1 | 1,126,666 |
| `$05:a64a` | 1 | 1,036,260 |
| `$1d:af84` | 1 | 310,970 |
| `$00:d619` | 1 | 72,616 |
| `$07:8ad1` | 1 | 41,756 |

The first candidate, `$06:F228`, is the unpriced lift check. Its seven independent raw closures all belong to `09/00`. Prices come from `scripts/rom_block_costs.py` (`source-06f228.txt`): actual modal/floor branches, actual descending slot scan with early exit, type/velocity exclusions, running test, cached-pickup comparison, hitbox-call opcodes and the selected hit/miss/pickup suffix. `$06:F257` is a separate direct entry; the `$F228` fallthrough shares its body without inventing another call. Children retain their own prices. New charges and scopes are lagless-only. No count, scene threshold, multiplier or residual supplies a price.

The CPU-only `lift_check_self_prices_match_executed_rom_paths` test compares native work with the actual ROM address-range instructions in all **116** entry × slot × branch fixtures. It covers every scan stop and the pickup double return. The ordinary runtime state layout is unchanged. On the pickup double return, the source call frame closes at the first PLA; the native routine scope retains the following 70 clocks (second PLA and RTS). This is a known exclusive-attribution difference, not missing instruction work or authority for another price.

The next raw candidate, `$06:DC10`, also contains drawing work already charged under `$06:DC13`; its five raw closures are not authority for an additional draw charge. The next actionable candidates still need their source-call ancestry reconciled. `before-category-ranking.json` retains separate rankings for both categories. `after-targeted-ranking.json` re-evaluates the original frozen population after this group.

### Full CPU-only verification

Both final full census gates PASS, 1,473,123 iterations each, unexplained zero. Both fresh main controls also pass. Runtime holds equal P2 on every row. These seeded logical CPU checks do not establish continuous physical-field or exact A/V parity.

| Capture | Held | Masked | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| main / rom | 97,642 | 2,729 | 5,035 | 0 | 0 | 34,594 | 39,561 |
| main / live | 97,535 | 2,697 | 5,731 | 589 | 131 | 34,460 | 39,429 |
| final / rom | 97,649 | 2,726 | 5,028 | 0 | 0 | 34,601 | 39,561 |
| final / live | 97,542 | 2,703 | 5,728 | 593 | 132 | 34,467 | 39,428 |

The cartridge census closes **7** previously missing iterations and introduces 0 newly missing iterations. `verification.json` lists the actual closures separately from the independent counterfactuals.

Live SPC retains all seven targeted recoveries but moves its first unmatched song-end poll from iteration 229030 to **229029** (`engine=[0]`, source `[(0,19)]`). The existing song-end-poll classification records the resulting earlier module/message progression. A five-hold workload moves 229031 → 229030; a four-hold workload moves 229036 → 229033. The latter moves from a source-lagged iteration onto a source-zero-lag iteration: **four additional missing holds, four additional excess holds, and one additional false-positive iteration**. All other hold changes are the seven targeted recoveries. `live-hold-deltas.json` and `live-229000-classifications.json` retain the rows and work counters. This is changed live execution/placement following the poll mismatch; it is not authority to reduce a ROM instruction price. No audio/upload fix or exception widening is included.

Each category cell below is **before → after**, with cartridge/live values on each side. Exact and within-one counts use lagged iterations only.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | --- | --- | --- | --- | --- |
| `0E/02` | 12,964 | 49 / 369 → 49 / 373 | 0 / 223 → 0 / 223 | 0 / 96 → 0 / 96 | 4,369 / 4,259 → 4,369 / 4,259 | 4,418 / 4,308 → 4,418 / 4,307 |
| `07/02` | 8,770 | 341 / 341 → 341 / 341 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1,635 / 1,635 → 1,635 / 1,635 | 1,976 / 1,976 → 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 → 89 / 89 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 60 / 60 → 60 / 60 | 135 / 135 → 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 → 121 / 121 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4,831 / 4,831 → 4,831 / 4,831 | 4,952 / 4,952 → 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 → 19 / 253 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 138 / 134 → 138 / 134 | 157 / 153 → 157 / 153 |
| `09/23` | 6,054 | 365 / 365 → 365 / 365 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 814 / 814 → 814 / 814 | 1,179 / 1,179 → 1,179 / 1,179 |
| `0F/01` | 5,432 | 84 / 84 → 84 / 84 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4,795 / 4,795 → 4,795 / 4,795 | 4,879 / 4,879 → 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 → 9 / 9 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 242 / 242 → 242 / 242 | 251 / 251 → 251 / 251 |
| `07/07` | 4,060 | 264 / 264 → 264 / 264 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2,655 / 2,655 → 2,655 / 2,655 | 2,919 / 2,919 → 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 → 1,673 / 1,673 | 0 / 102 → 0 / 106 | 0 / 18 → 0 / 19 | 1,287 / 1,287 → 1,287 / 1,287 | 2,960 / 2,960 → 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 249 / 249 → 249 / 249 | 251 / 251 → 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 → 71 / 71 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2,647 / 2,647 → 2,647 / 2,647 | 2,718 / 2,718 → 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 → 37 / 37 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 78 / 78 → 78 / 78 | 115 / 115 → 115 / 115 |
| `10/01` | 2,672 | 1 / 2 → 1 / 2 | 0 / 5 → 0 / 5 | 0 / 0 → 0 / 0 | 2,668 / 2,666 → 2,668 / 2,666 | 2,669 / 2,668 → 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 → 2 / 90 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 155 / 152 → 155 / 152 | 157 / 153 → 157 / 153 |
| `07/15` | 2,333 | 76 / 76 → 76 / 76 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1,659 / 1,659 → 1,659 / 1,659 | 1,735 / 1,735 → 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1,855 / 1,855 → 1,855 / 1,855 | 1,855 / 1,855 → 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 → 20 / 20 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 70 / 70 → 70 / 70 | 90 / 90 → 90 / 90 |
| `09/00` | 1,253 | 1,159 / 1,159 → 1,152 / 1,152 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 33 / 33 → 40 / 40 | 1,192 / 1,192 → 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 56 / 56 → 56 / 56 | 56 / 56 → 56 / 56 |
| `07/06` | 984 | 23 / 23 → 23 / 23 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 580 / 580 → 580 / 580 | 603 / 603 → 603 / 603 |
| `11/00` | 973 | 7 / 7 → 7 / 7 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 10 / 10 → 10 / 10 | 17 / 17 → 17 / 17 |
| `07/01` | 928 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 927 / 927 → 927 / 927 | 928 / 928 → 928 / 928 |
| `08/01` | 706 | 0 / 17 → 0 / 17 | 0 / 53 → 0 / 53 | 0 / 0 → 0 / 0 | 157 / 153 → 157 / 153 | 157 / 153 → 157 / 153 |
| `05/00` | 639 | 6 / 6 → 6 / 6 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 5 / 5 → 5 / 5 | 11 / 11 → 11 / 11 |
| `09/04` | 508 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 250 / 250 → 250 / 250 | 251 / 251 → 251 / 251 |
| `12/03` | 472 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 389 / 389 → 389 / 389 | 391 / 391 → 391 / 391 |
| `0F/00` | 449 | 24 / 24 → 24 / 24 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 263 / 263 → 263 / 263 | 287 / 287 → 287 / 287 |
| `07/12` | 391 | 18 / 18 → 18 / 18 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 221 / 221 → 221 / 221 | 239 / 239 → 239 / 239 |
| `09/2E` | 363 | 10 / 10 → 10 / 10 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 90 / 90 → 90 / 90 | 100 / 100 → 100 / 100 |
| `07/16` | 303 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 101 / 101 → 101 / 101 | 101 / 101 → 101 / 101 |
| `09/11` | 274 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 15 / 15 → 15 / 15 | 16 / 16 → 16 / 16 |
| `09/16` | 265 | 88 / 88 → 88 / 88 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 145 / 145 → 145 / 145 | 233 / 233 → 233 / 233 |
| `09/0D` | 247 | 89 / 89 → 89 / 89 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 134 / 134 → 134 / 134 | 223 / 223 → 223 / 223 |
| `0E/0B` | 243 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 81 / 81 → 81 / 81 | 81 / 81 → 81 / 81 |
| `09/0F` | 242 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 14 / 14 → 14 / 14 | 16 / 16 → 16 / 16 |
| `01/00` | 235 | 48 / 48 → 48 / 48 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/2C` | 228 | 4 / 4 → 4 / 4 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 32 / 32 → 32 / 32 | 36 / 36 → 36 / 36 |
| `10/00` | 157 | 0 / 1 → 0 / 1 | 0 / 81 → 0 / 81 | 0 / 0 → 0 / 0 | 157 / 154 → 157 / 154 | 157 / 155 → 157 / 155 |
| `17/01` | 141 | 3 / 3 → 3 / 3 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 | 2 / 2 → 2 / 2 |
| `1B/02` | 139 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 7 / 7 → 7 / 7 | 9 / 9 → 9 / 9 |
| `09/20` | 138 | 5 / 5 → 5 / 5 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 23 / 23 → 23 / 23 | 28 / 28 → 28 / 28 |
| `0E/05` | 138 | 36 / 36 → 36 / 36 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 83 / 83 → 83 / 83 | 119 / 119 → 119 / 119 |
| `00/01` | 114 | 7 / 7 → 7 / 7 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4 / 4 → 4 / 4 | 11 / 11 → 11 / 11 |
| `0B/24` | 103 | 61 / 61 → 61 / 61 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 32 / 32 → 32 / 32 | 93 / 93 → 93 / 93 |
| `09/21` | 84 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 28 / 28 → 28 / 28 | 28 / 28 → 28 / 28 |
| `09/0E` | 83 | 5 / 5 → 5 / 5 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 11 / 11 → 11 / 11 | 16 / 16 → 16 / 16 |
| `15/07` | 83 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 79 / 79 → 79 / 79 | 79 / 79 → 79 / 79 |
| `19/00` | 72 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 4 / 4 → 4 / 4 |
| `07/13` | 71 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 47 / 47 → 47 / 47 | 47 / 47 → 47 / 47 |
| `0B/17` | 68 | 32 / 32 → 32 / 32 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 33 / 33 → 33 / 33 | 65 / 65 → 65 / 65 |
| `0B/28` | 68 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4 / 4 → 4 / 4 | 4 / 4 → 4 / 4 |
| `0B/2B` | 66 | 23 / 23 → 23 / 23 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 43 / 43 → 43 / 43 | 66 / 66 → 66 / 66 |
| `0B/19` | 64 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 4 / 4 → 4 / 4 |
| `15/03` | 64 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 11 / 11 → 11 / 11 | 12 / 12 → 12 / 12 |
| `00/05` | 62 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 62 / 62 → 62 / 62 | 62 / 62 → 62 / 62 |
| `0B/26` | 61 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 4 / 4 → 4 / 4 |
| `0B/1A` | 60 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 4 / 4 → 4 / 4 |
| `1A/12` | 60 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 3 / 3 → 3 / 3 |
| `0B/29` | 59 | 33 / 33 → 33 / 33 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 18 / 18 → 18 / 18 | 51 / 51 → 51 / 51 |
| `18/00` | 59 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 2 / 2 → 2 / 2 |
| `1A/00` | 59 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 2 / 2 → 2 / 2 |
| `1A/1A` | 59 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `1A/1C` | 59 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 3 / 3 → 3 / 3 |
| `1A/04` | 58 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `1A/08` | 58 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `1A/0A` | 58 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `1A/18` | 58 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `1A/0E` | 57 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `1A/10` | 56 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `1A/06` | 55 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `07/03` | 53 | 53 / 53 → 53 / 53 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 53 / 53 → 53 / 53 |
| `1A/0C` | 53 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 2 / 2 → 2 / 2 |
| `11/07` | 51 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 17 / 17 → 17 / 17 | 17 / 17 → 17 / 17 |
| `13/05` | 51 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 51 / 51 → 51 / 51 | 51 / 51 → 51 / 51 |
| `1A/1E` | 51 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 2 / 2 → 2 / 2 |
| `0B/1C` | 48 | 11 / 11 → 11 / 11 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 29 / 29 → 29 / 29 | 40 / 40 → 40 / 40 |
| `1A/14` | 47 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 | 1 / 1 → 1 / 1 |
| `1A/02` | 46 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 | 1 / 1 → 1 / 1 |
| `1A/16` | 45 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 |
| `07/18` | 41 | 6 / 41 → 6 / 41 | 0 / 35 → 0 / 35 | 0 / 14 → 0 / 14 | 8 / 0 → 8 / 0 | 14 / 7 → 14 / 7 |
| `1A/20` | 41 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/12` | 32 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 16 / 16 → 16 / 16 | 16 / 16 → 16 / 16 |
| `12/08` | 32 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4 / 4 → 4 / 4 | 4 / 4 → 4 / 4 |
| `0B/25` | 30 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4 / 4 → 4 / 4 | 4 / 4 → 4 / 4 |
| `0B/00` | 29 | 9 / 9 → 9 / 9 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4 / 4 → 4 / 4 | 13 / 13 → 13 / 13 |
| `0B/18` | 24 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4 / 4 → 4 / 4 | 4 / 4 → 4 / 4 |
| `12/02` | 23 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 23 / 23 → 23 / 23 | 23 / 23 → 23 / 23 |
| `12/05` | 19 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 19 / 19 → 19 / 19 | 19 / 19 → 19 / 19 |
| `00/07` | 14 | 12 / 12 → 12 / 12 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 | 14 / 14 → 14 / 14 |
| `15/05` | 14 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 | 1 / 1 → 1 / 1 |
| `01/02` | 7 | 7 / 7 → 7 / 7 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 7 / 7 → 7 / 7 |
| `00/0A` | 6 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `12/09` | 4 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 4 / 4 → 4 / 4 | 4 / 4 → 4 / 4 |
| `13/04` | 3 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 3 / 3 → 3 / 3 | 3 / 3 → 3 / 3 |
| `00/02` | 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 | 1 / 1 → 1 / 1 |
| `00/06` | 2 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 |
| `04/01` | 2 | 2 / 2 → 2 / 2 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 2 / 2 → 2 / 2 |
| `00/00` | 1 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 |
| `03/01` | 1 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 |
| `07/0B` | 1 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 |
| `09/06` | 1 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 |
| `0E/01` | 1 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 |
| `19/01` | 1 | 1 / 1 → 1 / 1 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 1 / 1 → 1 / 1 |
| `00/03` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `00/04` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `00/0B` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `01/01` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `01/03` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `01/04` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `01/05` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `03/00` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `03/02` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `03/03` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `03/04` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `04/00` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `04/02` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `04/03` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/04` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/05` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/08` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/09` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/0A` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/0C` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/0D` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/10` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/14` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/17` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/19` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `07/1A` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/02` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/05` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/07` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/08` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/09` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/0A` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/0B` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/0C` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/10` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/13` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/14` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/15` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/1F` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/22` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/2D` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `09/2F` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `0B/08` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `0B/1B` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `0B/27` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `0B/2A` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `0E/04` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `0E/08` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `0E/09` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/01` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/04` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/06` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/07` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/0A` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/0B` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/0C` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/0D` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/0E` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `12/0F` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `13/00` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `13/01` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `13/02` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `13/03` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/00` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/01` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/02` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/04` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/06` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/09` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/0A` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/0B` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `15/0C` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `16/00` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `16/01` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `16/02` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `16/03` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `16/04` | 0 | 0 / 0 → 0 / 0 | 0 / 90 → 0 / 90 | 0 / 3 → 0 / 3 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/01` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/03` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/05` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/07` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/09` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/0B` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/0D` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/0F` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/11` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/13` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/15` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/17` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/19` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/1B` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/1D` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/1F` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/21` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |
| `1A/22` | 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 | 0 / 0 → 0 / 0 |

Shortfall is the minimum extra main work in the frozen completed P2 inverse, with worker share and raster wait held fixed. It is not a source instruction deficit. Bands are <5%, 5–<50%, and ≥50% of a 357,368-clock field. Counts below are missing iterations, with missing holds in parentheses.

| Population | Before near / moderate / large | After near / moderate / large |
| --- | --- | --- |
| `all` | 1188 / 3565 / 228 (5035) | 1224 / 3522 / 228 (5028) |
| `07/00` | 8 / 1665 / 0 (1673) | 8 / 1665 / 0 (1673) |
| `09/00` | 5 / 1154 / 0 (1159) | 41 / 1111 / 0 (1152) |
| `09/23` | 197 / 168 / 0 (365) | 197 / 168 / 0 (365) |
| `07/02` | 151 / 177 / 13 (341) | 151 / 177 / 13 (341) |
| `07/07` | 238 / 26 / 0 (264) | 238 / 26 / 0 (264) |
| `07/0E` | 43 / 78 / 0 (121) | 43 / 78 / 0 (121) |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

Phase counts are source/native; all remain within one. All eight stable room/Link-XY/whole-save outcomes have source matches. All 27 rupee requests and every request/PCM-onset gap in 162000–163000 match fresh main; the PCM-gap median remains 8.253745 frames.

| Capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| main / rom | `{"lag": 14}` | 10 |
| main / live | `{"lag": 14, "polygon": 14742, "song-end-poll": 638}` | 10 |
| main / probes | `{}` | 0 |
| final / rom | `{"lag": 14}` | 10 |
| final / live | `{"lag": 14, "polygon": 14742, "song-end-poll": 639}` | 10 |
| final / probes | `{}` | 0 |

The final default binary matches every record and EOF of the fresh main default 200k state/display/SRAM/PCM/input/context ledger. Default/ROM-timing ledgers retain their existing charges. Both ordinary builds are warning-free. All 8,339 original missing rows remain within their source HDMA bounds, and the same 47 masked-init leases are declined.

| Library configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,520 | 1 | 3 |
| reduced | 2,055 | 0 | 2 |

The sole default library failure is the known spotlight caller-return fixture; reduced passes completely. Both SNES library suites and the Python routine-coverage tests pass. No GPU runs, `--video`, cached-av, external Snes9x execution, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push were used. Scratch drivers remain in `/tmp`; supplied ledger/cache files were read-only. This is the verified stopping point for the first price group; remaining prices and the later category cohort are still open.

## Continuation on `codex/cost-pacing22` (main `618fc6b5`)

This is a short, **diagnosis-only stopping point**. No production code or instruction price changed. The main instruction term accounts for **94.4% of the measured source-minus-native main-interval gap** on the 2,825 remaining missing iterations in `07/00` and `09/00`. It independently exceeds the frozen P2 inverse shortfall on 2,633 rows (93.2%). This supports investigating missing main work; it does not establish a single systematic omission or authorize a replacement price. No fix group is shipped.

Evidence is in `target/cost-pacing22/decomposition.{json,csv}`. Every selected iteration has source and native instruction, refresh, main DMA, bounded HDMA and raster components, native entry/intervening handler totals, source IRQ counts and handler return scanline, plus source WRAM features. The join uses the retained executed-ROM structural/routine profiles in `target/cost-pacing10/source-missing/` and verified batch-21 group-1 native profiles/censuses. Source and native exclusive routine sums equal their respective main counters on **every one of the 2,825 rows**. No lost native profiler scope explains this total gap.

| Main-interval component, master clocks | 07/00 source | 07/00 native | 09/00 source | 09/00 native |
| --- | ---: | ---: | ---: | ---: |
| Instructions | 547,156,224 | 363,621,676 | 372,948,230 | 275,058,640 |
| Refresh | 17,059,040 | 11,355,560 | 11,620,320 | 8,588,680 |
| Main DMA | 0 | 0 | 0 | 0 |
| HDMA lower bound / admitted estimate | 16,470,274 | 11,129,976 | 11,085,980 | 8,474,640 |
| Raster polls / wait | 0 | 0 | 0 | 0 |
| Main interval total | 580,685,538 | 386,107,212 | 395,654,530 | 292,121,960 |

All HDMA bounds collapse to an exact residual here: no selected source step has an interrupt/refresh-edge ambiguity. The main-interval gap is 298,110,896 clocks: 281,424,138 instructions (94.40%), 8,735,120 refresh (2.93%), and 7,951,638 HDMA (2.67%). DMA and raster polling are zero on both sides. Refresh and HDMA are measured over each side’s executed interval; these differences are not independent missing prices at equal instruction coverage.

| Population | Rows | Main instruction gap | Main wall gap | Frozen inverse shortfall | Instruction gap ≥ inverse |
| --- | ---: | ---: | ---: | ---: | ---: |
| `07/00` | 1,673 | 183,534,548 | 194,578,326 | 159,711,902 | 1,554 |
| `09/00` | 1,152 | 97,889,590 | 103,532,570 | 77,309,954 | 1,079 |
| `all` | 2,825 | 281,424,138 | 298,110,896 | 237,021,856 | 2,633 |

This inverse is the extra appended work needed to reach a source hold threshold with native ownership fixed. It is not actual source elapsed time or a price. The instruction gap is positive on all selected rows. On 192 rows it is smaller than the inverse; replacing main instructions alone would not clear their frozen threshold.

| Interrupt component, master clocks unless stated | 07/00 | 09/00 |
| --- | ---: | ---: |
| Native opening NMI, CPU + DMA together | 59,550,160 | 39,806,858 |
| Native intervening lag NMIs | 150,522 | 21,210 |
| Source accepted lag NMIs, count | 1,723 | 1,159 |
| Native accepted lag NMIs, count | 50 | 7 |
| Source IRQ events, count | 0 | 459 |
| Rows with source IRQ events | 0 | 228 |

Native elapsed reconciles exactly on every row as main instructions + main DMA + HDMA + refresh + raster wait + opening handler + native-held-count × retained lag-handler cost. Thus no additional native worker/IRQ-share term is hidden in this cohort’s elapsed counter. Every source entry has `$012A=0`; 2,597 rows have no IRQs, 227 have two and one has five. An IRQ event with the worker disabled is not evidence of worker execution; entry state alone also cannot prove every subsequent instruction’s ownership.

**The requested complete source elapsed decomposition is not available from these retained artifacts.** The source structural observer records main-stack steps while excluding interrupt bodies. The supplied logical ledger retains IRQ/NMI counts and a return scanline, not opening/return timestamps, NMI CPU versus OAM/VRAM DMA bytes, or per-IRQ durations. `null`/empty fields in the per-row artifacts mean unavailable, not zero. A host-width × field calculation includes idle time and host cuts; a return scanline is not a handler duration. Neither is substituted for these missing measurements. The 94.4% finding is restricted to the measured main interval, not the entire source field. This batch does not claim to have completed the full all-component decomposition.

The largest signed exclusive routine gap is `$06:E72F`, 14,898,766 clocks (5.3% of the main instruction gap); `$06:DC10` follows at 10,810,378 (3.8%). The latter contains known separately attributed drawing work. Raw aliases/ancestry and execution differences still require reconciliation. No single routine or established hardware omission explains a large share, so no constant, scene table, count multiplier or new charge was added.

### Feature correlations

Features were decoded afresh, read-only, from all 1,473,124 post-NMI and pre-NMI XOR images. Source entry features use image i; opening DMA operands use pre-NMI image i. This does not expose mid-main feature changes. Active sprites count the 16 nonzero `$0DD0` slots, ancilla the 10 nonzero `$0C4A` slots, Link state is `$5D`, parity is `$1A&1`, room is `$A0`, area is `$040A`, OAM cursor is `$90`, and OAM shadow slots count Y != `$F0` across 128 entries. OAM shadow use is not DMA bytes. BG2 scroll is `$E2/$E8`.

Pearson r below relates a numeric count to **inverse-shortfall severity among missing rows only**. These are descriptive correlations with no fitted timing values. Room/area IDs are treated as categorical; their numeric order has no timing meaning.

| Feature | 07/00 r | 09/00 r |
| --- | ---: | ---: |
| Active sprites | -0.3667 | -0.2437 |
| Ancilla | +0.0296 | +0.2331 |
| OAM shadow slots | -0.0217 | +0.1575 |
| OAM cursor | +0.3726 | +0.1881 |
| Frame parity | +0.0183 | -0.0238 |

The strongest meaningful categorical groupings are dungeon room in `07/00` (48 rooms, eta²=0.5751) and overworld area in `09/00` (42 areas, eta²=0.6497). These are within-cohort variance descriptions, not held-out predictions or source causes. High-cardinality raw scroll coordinates appear stronger (109/267 X/Y levels in dungeon and 637/665 outside) but mostly identify route location; they are not scroll-upload counts and do not establish a scroll cost. Link handler state has categorical eta²=0.0754 / 0.0471. Frame parity explains only 0.0333% / 0.0566% of severity variance.

Concrete state counts show load sensitivity without supplying a price: room `$3E` has 383 missing rows / 7,859 same-room controls (4.87%), mean inverse 106,922 clocks; overworld area `$72` has 129 / 2,440 (5.29%), mean inverse 78,762. In `07/00`, 14 active sprites have 179 / 9,042 missing rows (1.98%), versus 184 / 71,199 with eight (0.258%). In `09/00`, ten active sprites have 101 / 830 (12.17%), versus 125 / 18,388 with six (0.680%). Severity within the missing subset decreases with sprite count; the threshold-selected subset is not a valid basis for a positive or negative per-sprite price. Full group counts and mean instruction gaps are retained in JSON.

Opening-NMI upload operands are weak predictors: nonzero `$1000` upload offsets occur on only 9 / 10 rows, and `$14` BG upload on 10 / 12 (`07/00` / `09/00`). `$17` is nonzero on 5 / 65; `$18` packet and `$19` tile-destination flags are zero on every row. Upload-offset r is +0.0355 / +0.0625, and BG-upload r is -0.0117 / +0.0205. Post-handler flags have already been consumed, so the pre-handler values are used for this check. Actual NMI DMA byte counts remain unavailable; neither flags nor active OAM slots replace them.

### Verification boundaries and unchanged metrics

Every production and example source hash matches the previously verified batch-21 group-1 receipt. Both **retained full census gates were rechecked and PASS**, 1,473,123 iterations each, unexplained zero. Those censuses, phase/PCM probes and library suites were not rerun in this short diagnosis-only batch. Their bound outcomes below are retained evidence, not new executions. Both ordinary binaries were freshly built warning-free. A fresh default 200k baseline was recorded from current clean main `618fc6b5`; the unchanged default candidate matches every record and EOF. Default/ROM-timing source and ledgers are unchanged.

| Retained census | Held | Masked | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rom | 97,649 | 2,726 | 5,028 | 0 | 0 | 34,601 | 39,561 |
| live | 97,542 | 2,703 | 5,728 | 593 | 132 | 34,467 | 39,428 |

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 373 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,307 |
| `07/02` | 8,770 | 341 / 341 | 0 / 0 | 0 / 0 | 1,635 / 1,635 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 84 / 84 | 0 / 0 | 0 / 0 | 4,795 / 4,795 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 264 / 264 | 0 / 0 | 0 / 0 | 2,655 / 2,655 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 106 | 0 / 19 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 76 / 76 | 0 / 0 | 0 / 0 | 1,659 / 1,659 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,152 / 1,152 | 0 / 0 | 0 / 0 | 40 / 40 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 24 / 24 | 0 / 0 | 0 / 0 | 263 / 263 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

Retained source/native phase counts remain within one; all eight stable room/Link-XY/whole-save checks match. The retained rupee window 162000–163000 has the same 27 requests and all request/PCM gaps, median 8.253745 frames.

| Retained capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14742, "song-end-poll": 639}` | 10 |
| probes | `{}` | 0 |

| Retained library configuration | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,520 | 1 | 3 |
| reduced | 2,055 | 0 | 2 |

The only default library failure is the known spotlight caller-return fixture; reduced passes completely. The complete disjoint library manifest and binary hashes were revalidated. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch analysis programs remain in `/tmp`; supplied ledgers/cache were read-only. This stops at a source-bound diagnostic point, with no pricing fix and the exact source NMI/IRQ/DMA split still open.

## Masked elapsed fields

Single-measurement stopping point on `codex/masked-fields`, main `618fc6b5`
plus parked WIP `1ca3965a`. The WIP was inspected and retained unchanged as
the measurement candidate: lagless-only bank-1/2 presentation ownership,
native runtime streams, fractional command placement after prefix audio,
and receiver-return forecasting. Bank 0 retains its existing command start.
No per-routine instruction price was changed. **This is not acceptance of
the candidate.**

Exactly one fresh full cartridge-answer census was run, using the Reproduction
command above with `target/masked-fields-step/candidate-zelda3`, separate
save/output paths, `--seed-every 2000`, and no video. It completed all
1,473,123 iterations; the census gate PASSed with unexplained zero. The
warning-free binary was freshly built from the requested HEAD. Evidence,
source/binary/input hashes, the captured diff, full predictor CSV,
classification trace, gate summary and `measurement.json` are retained in
`target/masked-fields-step/`. The comparison uses batch 21 group 1's retained
`final-rom.csv`; every source hash in its source manifest was checked against
git main `618fc6b5`.

Masked counts below are cartridge answers. Missing/excess are summed **per
iteration** outside `[strict, strict + edges]`, not differences of context
totals.

| Context | Source strict | Edges | Main masked | Candidate masked | Candidate missing / excess | Main → candidate envelope error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `06/00` | 2,163 | 50 | 0 | 2,198 | 19 / 37 | 2,163 → 56 |
| `08/00` | 2,513 | 54 | 2,606 | 2,495 | 38 / 11 | 62 → 49 |
| `11/00` | 282 | 8 | 0 | 290 | 2 / 5 | 282 → 7 |
| `15/00` | 22 | 0 | 24 | 24 | 0 / 2 | 2 → 2 |
| `17/01` | 69 | 2 | 72 | 69 | 1 / 1 | 2 → 2 |
| `18/00` | 23 | 1 | 24 | 23 | 0 / 0 | 0 → 0 |
| `19/00` | 15 | 1 | 0 | 15 | 0 / 0 | 15 → 0 |
| `1B/02` | 21 | 1 | 0 | 23 | 0 / 1 | 21 → 1 |
| Total | 5,108 | 117 | 2,726 | 5,137 | 60 / 57 | 2,547 → 117 |

No context's envelope error increases. Accepted holds are byte-for-byte equal
as CSV values on every row: 97,649 holds, 5,028 missing, zero excess/FP, and
every runtime hold equals P2. The audio-duration callback component
`534 * (iterations + held + masked - 1,580,998)` changes from **−4,005,000
to −2,717,526 sample pairs**, an improvement of 1,287,474 pairs. The 534-pair
output budget is unchanged.

The largest residuals are `06/00` (56) and `08/00` (49), followed by `11/00`
(7): together 112 of the remaining 117. The concrete unresolved mechanisms
are command placement and the return/enable boundary:

- Bank 0 still drops the command remainder. Its return now uses the forecast;
  the measured `08/00` callbacks fall by 111 across 92 changed episodes.
  This locates a count-sensitive lane, but does not prove each missing field's
  source timestamp or individual cause.
- Banks 1/2 retain a remainder but still use estimated elapsed work and an
  output-clock anchor, without a suspended semantic CPU caller carried through
  adjacent calls/restores. Aggregate `06/00` and `11/00` totals near their
  envelopes therefore coexist with per-episode missing and excess fields.
- The forecast checks whether the port-clear plus return suffix fits the
  534-pair window, then runs one closing NMI. It does not retain the actual
  `$4200` enable edge and pending acceptance sequence. Batch 20's source
  host 37,686 demonstrates why one callback cannot establish that sequence.
  The unchanged two-field `15/00` excess and remaining single-field errors
  have not been individually attributed.

The older `target/masked-fields/return-*` full censuses and phase/default
checks predate `618fc6b5` (their source binding differs in the lifted-sprite
production file and its tests); several library shards are incomplete.
They were not reused as current-HEAD acceptance. In this bounded step no
live-SPC census, eight-phase sweep, default 200k comparison or library suites
were run. Thus only the fresh cartridge count/gate result is verified;
the full incremental acceptance remains unestablished. Production code is
unchanged from the parked WIP, and this report is left uncommitted.

No GPU runs, `--video`, cached-av, Snes9x, cargo GPU tests, oracle-cache
writes, cargo fmt, commit or push were used. Scratch analysis stayed in
`/tmp`; supplied source evidence was read-only.

### Verification-only follow-up (2026-10-25)

Completed the remaining CPU-only checks on the unchanged `1ca3965a` candidate.
Live-SPC and phase probes used the exact retained `candidate-zelda3` binary
(`ba78c60467c83466e21483b6a125681c638832570c13df63cd8da84fb89e5af9`).
The retained binary and a fresh default build both ran the default ledger
comparison. Production and test sources were rechecked unchanged. Results,
commands, binary/input hashes, full census/probe outputs and exhaustive library
reconciliation are retained in `target/masked-fields-step/acceptance/`.
Only this documentation was edited.

**Live-SPC full gate: PASS**, all 1,473,123 iterations, unexplained zero.
Every runtime hold equals P2. The comparison reads batch 21 group 1’s retained
`final-live.csv`; its recorded source hashes were verified against main
`618fc6b5`. **Live hold stability regresses**, despite the full gate passing:

| Live metric | Main | Candidate |
| --- | ---: | ---: |
| Held | 97,542 | 97,542 |
| Missing holds | 5,728 | 5,729 |
| Excess holds | 593 | 594 |
| False-positive iterations | 132 | 133 |
| Masked callbacks | 2,703 | 5,117 |

Twelve rows change held placement (`live-hold-deltas.json`). The song-end poll
mismatch moves 94,849 → 94,848 (`engine=[0]`, source `[(0,19)]`). Across
94,849–94,855 this shifts holds earlier and adds **one missing hold, one excess
hold and one false-positive iteration**. The other six changes follow the
1,064,728 → 1,064,729 poll movement and relocate three workloads by one iteration
without changing those aggregate errors. Classification traces retain the
observed song-end-poll progression; this is not independent attribution of
each masked field’s physical command/enable timestamp. No fix was attempted.

Live masked-envelope errors use the same per-iteration strict/edge bounds as
above. Total error improves **2,722 → 285** (candidate missing 157 / excess 128);
no context’s error increases. Live context placement differs from cartridge
answers, so contexts with zero source strict/edge counts are retained too.

| Context | Source strict | Edges | Main masked | Candidate masked | Candidate missing / excess | Main → candidate error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `06/00` | 2,163 | 50 | 0 | 2,203 | 16 / 38 | 2,163 → 54 |
| `08/00` | 2,513 | 54 | 2,512 | 2,403 | 137 / 13 | 166 → 150 |
| `08/01` | 0 | 0 | 23 | 22 | 0 / 22 | 23 → 22 |
| `10/00` | 0 | 0 | 25 | 24 | 0 / 24 | 25 → 24 |
| `11/00` | 282 | 8 | 0 | 289 | 3 / 5 | 282 → 8 |
| `15/00` | 22 | 0 | 24 | 24 | 0 / 2 | 2 → 2 |
| `16/04` | 0 | 0 | 23 | 22 | 0 / 22 | 23 → 22 |
| `17/01` | 69 | 2 | 72 | 69 | 1 / 1 | 2 → 2 |
| `18/00` | 23 | 1 | 24 | 23 | 0 / 0 | 0 → 0 |
| `19/00` | 15 | 1 | 0 | 15 | 0 / 0 | 15 → 0 |
| `1B/02` | 21 | 1 | 0 | 23 | 0 / 1 | 21 → 1 |
| Total | 5,108 | 117 | 2,703 | 5,117 | 157 / 128 | 2,722 → 285 |

**Phase/outcome checks: PASS.** Counts below are source / candidate.
All eight probes remain within one for every phase. All eight stable room,
Link-XY and whole-save outcomes match source and retained main.

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 64 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 84 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 83 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

The entire 162000–163000 rupee probe is unchanged: all 1,001 outcome rows and
all 2,537,568 PCM bytes match retained main. The same 27 requests, every request
gap and every PCM onset/gap match; PCM-gap median is 8.253745 frames.
That window has zero classified differences and zero `$0CFD`-only exceptions.
`rupee-window-reconciliation.json`, `results.json` and
`probe-classifications-summary.json` retain the comparisons.

**Default 200k state ledger: PASS.** A fresh default binary built from an
isolated `git archive 618fc6b5` recorded 200,000 consecutive schema-3 records.
Both the exact retained candidate binary and the fresh candidate default
binary matched every state/display/SRAM/PCM/input/context record and EOF.
The record and both comparisons completed successfully.

**Ordinary builds: PASS, default and reduced warning-free.** The fresh main
default build is warning-free too. The isolated main build initially lacked
asset/ROM paths; final compilation used existing assets and ROM through the
supported environment settings, with no source changes. Lib-test compilation
warnings remain separate from these warning-free ordinary builds.

**Library suites: complete.** Every listed test appears exactly once across
four disjoint completed shards per Zelda configuration; manifests verify the
full named union and retain each shard’s process exit and test footer.

| Library | Configuration | Pass | Fail | Ignored |
| --- | --- | ---: | ---: | ---: |
| Zelda | default | 2,523 | 1 | 3 |
| SNES | default | 504 | 0 | 7 |
| Zelda | reduced | 2,058 | 0 | 2 |
| SNES | reduced | 504 | 0 | 7 |

The only default Zelda failure is the expected
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`
fixture: its assertion at `spotlight_caller_return.rs:411` reads
`Some(1047868)` versus `Some(170442)`. Reduced Zelda and both SNES suites
pass completely. The known fixture
failure and the live hold-placement regression were reported without fixes.
No GPU run, `--video`, cached-av, Snes9x, cargo GPU test, oracle-cache write,
`cargo fmt`, commit or push was used. Scratch scripts stayed in `/tmp`; the
other working checkout’s requested retained evidence was read-only.

## Continuation on `codex/cost-pacing23` (main `56d9c92a`)

Evidence is in `target/cost-pacing23/corrected/`, with the fresh main default control in `target/cost-pacing23/`. This is a **verified partial pricing point**: the shared tile-property self work is priced through both entries, and the tile-collision dispatcher receives its own instruction price. The ten-routine scope is not complete. No residual from the parent, drawing aliases, selected garnish/actor bodies or Link tails was converted into a constant.

### ROM-derived group and ownership

`scripts/rom_block_costs.py` produced all ten retained source listings. New prices are lagless-only and cite their ROM address ranges. `$06:E72F` now keeps its real call frame across the fallthrough shared body; a direct `$06:E73C` call has its own frame. There is no invented nested `$E73C` call on the `$E72F` entry. Coordinate prices follow the actual indoor/outdoor paths and the ROM’s Y-before-X bounds order, including the real `$E6B7,Y` page crossing. Classification prices follow deflection, water exclusions, tile tables, slopes, pits, spikes and the actual return path. The `$E872` out-of-bounds JSR is separately attributed; lookup, slope and damage bodies are not folded into the caller. Existing retained continuation work does not receive the new atomic classification charge.

`$06:E4AB..E4DA`, plus the `$E4A1/$E4A8` branches, prices only the dispatcher: wall clear, flag/header checks, floor save/restore, actual JSR opcodes and the selected return. **The single-layer tail’s own instructions at `$E4DB..E5B7` remain unpriced**, so this is not a complete price for the source call frame.

The CPU-only ROM tests pass all **4,608 shared-body fixtures** and **20 dispatcher fixtures**, including both entries, slots, coordinate/bounds paths, floor layers, table crossings, deflection and water exclusions. They compare the executed ROM instructions with native self work, and verify that default calls add no new price. No source lag count, fitted residual, scene threshold, multiplier or scene table supplies a price.

The initial `group1/` trial is rejected. A final cohort audit found a 252-clock excess at iteration 682090: the price had used X bounds to decide whether the second-axis block ran, while `$06:E742/$E775` read Y first. The correction follows those operands, rather than subtracting a residual. Independent asymmetric X/Y fixtures were added; the focused replay of that iteration now matches all seven source calls, 8,092 exclusive and 12,956 inclusive clocks. All final builds, censuses and suites below were rerun for the correction.

### Frozen 2,825-row cohort: all ten requested routines

Exclusive clocks below use the original batch-22 `07/00` + `09/00` missing population, including rows this group closes. Calls are summed source / native-before / native-after. Per-call values are aggregate exclusive clocks divided by each lane’s own calls; a missing native scope is not an observed zero-cost native call. `top10-cohort-before-after.json` also retains the equal-call row counts and the range of row-mean deficits. Those means do not establish individual-call pairing.

| Routine | Calls source / before / after | Source clocks/call | Native before / after clocks/call | Signed gap before | Signed gap after |
| --- | ---: | ---: | ---: | ---: | ---: |
| `$06:e72f` | 14,365 / 14,365 / 14,365 | 1,230.392 | 193.235 / 1,230.392 | 14,898,766 | 0 |
| `$06:dc10` | 11,899 / 11,951 / 11,951 | 954.712 | 46.000 / 46.000 | 10,810,378 | 10,810,378 |
| `$09:b0b6` | 68,085 / 68,085 / 68,085 | 364.315 | 231.725 / 231.725 | 9,027,408 | 9,027,408 |
| `$1e:8b19` | 9,306 / 9,306 / 9,306 | 1,231.806 | 302.381 / 302.381 | 8,649,224 | 8,649,224 |
| `$00:80b5` | 2,825 / 2,825 / 2,825 | 2,635.652 | 240.000 / 240.000 | 6,767,718 | 6,767,718 |
| `$06:e4ab` | 9,262 / 0 / 9,262 | 723.142 | unscoped / 187.474 | 6,697,740 | 4,961,356 |
| `$07:d077` | 3,249 / 0 / 0 | 1,980.903 | unscoped / unscoped | 6,435,954 | 6,435,954 |
| `$07:807f` | 2,784 / 2,784 / 2,784 | 2,395.971 | 214.132 / 214.132 | 6,074,240 | 6,074,240 |
| `$06:e73c` | 5,516 / 0 / 5,516 | 1,082.655 | unscoped / 1,082.655 | 5,971,924 | 0 |
| `$07:e245` | 2,353 / 2,353 / 2,353 | 4,430.403 | 1,918.673 / 1,918.673 | 5,910,100 | 5,910,100 |

| Routine | Equal-call nonempty rows before / after | Before row-mean gap range | After row-mean gap range |
| --- | ---: | ---: | ---: |
| `$06:e72f` | 2,172 / 2,172 | 949.000 to 1,554.000 | 0.000 to 0.000 |
| `$06:dc10` | 2,287 / 2,287 | 346.000 to 2,110.000 | 346.000 to 2,110.000 |
| `$09:b0b6` | 2,272 / 2,272 | 0.000 to 1,331.667 | 0.000 to 1,331.667 |
| `$1e:8b19` | 1,455 / 1,455 | 92.000 to 4,338.000 | 92.000 to 4,338.000 |
| `$00:80b5` | 2,825 / 2,825 | 1,598.000 to 3,610.000 | 1,598.000 to 3,610.000 |
| `$06:e4ab` | 0 / 2,378 | unavailable | 0.000 to 936.000 |
| `$07:d077` | 0 / 0 | unavailable | unavailable |
| `$07:807f` | 2,784 / 2,784 | 148.000 to 3,226.000 | 148.000 to 3,226.000 |
| `$06:e73c` | 0 / 1,675 | unavailable | 0.000 to 0.000 |
| `$07:e245` | 2,353 / 2,353 | 2,390.000 to 5,544.000 | 2,390.000 to 5,544.000 |

Both shared entries now match source calls and exclusive clocks on every cohort row. The missing shared-body price was path-dependent, not a constant per call. The bounds, coordinate and classification branches above supply the prices. The remaining dispatcher-frame gap is also path-dependent and includes its unpriced tail.

`$06:DC10` is a confirmed attribution case: native instruction continuations place the drawing and shadow tail under `$06:DC13/$DC66`, while the source retains the `$DC10` call frame. Its before call counts also differ by 52; neither its raw exclusive deficit nor a sum of unrelated body labels supplies a new price. It is skipped. `$00:80B5` executes eight dispatcher instructions totalling **240** clocks, already charged natively. Its JML inherits module work in the source profiler; no child work is re-absorbed into that parent.

`$09:B0B6` already prices empty/countdown/allocation/RTS dispatch paths; selected garnish bodies remain open. `$1E:8B19` already prices its 290-clock RTS dispatch; selected actor bodies remain open. `$07:D077` still lacks the atomic entry/response annotation. `$07:807F` mixes priced entry work, native handler labels and unpriced tails. `$07:E245` has already priced movement tails under `$E373/$E3E0/$E42A/$E4D3` and other owners; its raw deficit is not authority for an additional price. Exact ancestry reconciliation and any remaining own work in these cases are unfinished.

### Route-wide comparison boundary

The fresh main and candidate ROM censuses also collected every native routine over the full route. `main-route-profile.json` and `final-route-profile.json` retain aggregate calls/inclusive/exclusive clocks and per-row means for all ten targets; their exclusive rows reconcile on every one of 1,473,123 iterations. The full route native totals below are **not** paired with full route source totals.

| Routine | Native calls before / after | Native clocks/call before / after |
| --- | ---: | ---: |
| `$06:e72f` | 1,289,963 / 1,289,963 | 193.211 / 1,255.593 |
| `$06:dc10` | 2,350,894 / 2,350,894 | 46.000 / 46.000 |
| `$09:b0b6` | 23,398,320 / 23,398,320 | 152.746 / 152.746 |
| `$1e:8b19` | 2,487,531 / 2,487,531 | 338.889 / 338.889 |
| `$00:80b5` | 1,473,123 / 1,473,123 | 487.114 / 487.114 |
| `$06:e4ab` | 0 / 1,368,796 | unscoped / 158.432 |
| `$07:d077` | 0 / 0 | unscoped / unscoped |
| `$07:807f` | 1,051,125 / 1,051,125 | 205.201 / 205.201 |
| `$06:e73c` | 0 / 699,780 | unscoped / 1,085.615 |
| `$07:e245` | 976,645 / 976,645 | 1,853.645 / 1,853.645 |

**The requested route-wide source/native call and per-call reconciliation remains unfinished.** The retained source profiles cover the frozen missing cohort, not every route iteration. A scratch CPU-only full route source aggregation was attempted, but did not complete within this short batch; its partial run is not used as route-wide evidence. No call-equivalence or constant-deficit claim is made for that missing comparison.

### Full CPU-only checks

Both fresh final full census gates PASS, 1,473,123 iterations each, unexplained zero. The fresh main ROM control reproduces every retained main work/hold/elapsed/shortfall row. Runtime holds equal P2 on every candidate row. Missing/excess are lag NMI counts; false positives count iterations. These seeded CPU checks do not establish continuous physical-field or exact A/V parity. Before live metrics are retained batch-21 group-1 evidence bound to the unchanged main implementation; they were not rerun as another live control.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 5,028 / 5,011 | 0 / 0 | 0 / 0 | 34,601 / 34,618 | 39,561 / 39,561 |
| live | 5,728 / 5,708 | 593 / 590 | 132 / 132 | 34,467 / 34,484 | 39,428 / 39,429 |

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 370 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 334 / 334 | 0 / 0 | 0 / 0 | 1,642 / 1,642 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 84 / 84 | 0 / 0 | 0 / 0 | 4,795 / 4,795 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 263 / 263 | 0 / 0 | 0 / 0 | 2,656 / 2,656 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,673 / 1,673 | 0 / 103 | 0 / 19 | 1,287 / 1,287 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 72 / 72 | 0 / 0 | 0 / 0 | 1,663 / 1,663 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,149 / 1,149 | 0 / 0 | 0 / 0 | 43 / 43 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 22 / 22 | 0 / 0 | 0 / 0 | 265 / 265 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 65 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 85 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 85 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 91 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 85 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 84 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts remain within one; all eight stable room/Link-XY/whole-save checks match. The 162000–163000 rupee window retains the same 27 requests and all request/PCM gaps as the previous verified group, median 8.253745 frames.

| Capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14741, "song-end-poll": 639}` | 10 |
| probes | `{}` | 0 |

The frozen 8,339 original missing rows gain 23,505,310 main instruction clocks, with no negative row. There are 0 HDMA upper-bound excursions. The same 47 masked-init leases remain declined.

Both ordinary builds are warning-free. The final default binary matches every record and EOF against a fresh clean-main default binary and fresh 200k state/display/SRAM/PCM/input/context ledger. Default/ROM-timing prices and behavior remain unchanged over that check.

| Library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,521 | 1 | 3 |
| reduced | 2,056 | 0 | 2 |

The sole default library failure is the known spotlight caller-return fixture; reduced passes completely. Both SNES suites and the Python routine-coverage tests pass. The manifests verify complete, disjoint library unions. No GPU run, `--video`, cached-av, external Snes9x execution, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache stayed read-only. `git diff --check` passes. This stops at the verified partial point above; full ten-routine and route-wide source reconciliation are not claimed.

## Continuation on `codex/cost-pacing24a` (main `dc86c1ef`)

Evidence is in `target/cost-pacing24a/`; scratch drivers are `/tmp/cost24a-*.py`. One verified group prices four of the twelve scoped Link routines. All twelve have ROM listings and frozen-cohort before/after measurements. The remaining eight are deliberately left open; no fitted residual or lag count supplies a price.

### ROM instruction work and ownership

`scripts/rom_block_costs.py` gives straight-line self costs at M8/X8 entry: `$07:CDCB..CE29` = 1,378, `$07:CE2A..CE84` = 1,326, `$07:CE85..CEC8` = 988, and `$07:9873..9895` = 486 master clocks. The probes price coordinate setup, the executed JSR instructions, width changes and RTS; `$07:D798` and each `$07:D9D8` child retain their own work. The direction-indexed word operands never cross a page for any of the four direction indices. The reset prices REP, ten word clears, SEP and RTS. These are lagless-only additions with address comments. No scene threshold, multiplier or scene table was added.

The shared `detect_player_movement` helper changes only the three requested entry paths. Its horizontal slope path remains outside this group. Existing instruction continuations execute their own retained models and do not call this atomic helper, so they receive no duplicate price. The other worktree's sprite/ancilla/overworld routines were not edited.

The new CPU-only executed-ROM regression covers 256 entry/direction/floor/mask/position fixtures, each with lagless and default native controls. It compares exclusive self work while executing the real child calls. Coordinate wrap, aligned masks, both floors, all four direction indices, and byte/word return widths are exercised. The existing collision-effect and acceleration-state regressions run in both library suites.

### Frozen 2,825-row cohort: all twelve routines

Calls below are source / native-before / native-after. Exclusive clocks/call are aggregate exclusive work divided by each lane's own calls; unscoped does not mean an observed zero-cost native call. Every newly priced routine matches source calls and exclusive clocks on every cohort row. `cohort-before-after.json` retains equal-call row counts and per-row mean ranges.

| Routine | Calls source / before / after | Source clocks/call | Native before / after clocks/call | Gap before | Gap after |
| --- | ---: | ---: | ---: | ---: | ---: |
| `$07:d077` | 3,249 / 0 / 0 | 1,980.903 | unscoped / unscoped | 6,435,954 | 6,435,954 |
| `$07:807f` | 2,784 / 2,784 / 2,784 | 2,395.971 | 214.132 / 214.132 | 6,074,240 | 6,074,240 |
| `$07:e245` | 2,353 / 2,353 / 2,353 | 4,430.403 | 1,918.673 / 1,918.673 | 5,910,100 | 5,910,100 |
| `$07:ba0a` | 2,754 / 0 / 0 | 1,988.130 | unscoped / unscoped | 5,475,310 | 5,475,310 |
| `$07:c4d4` | 2,757 / 0 / 0 | 1,433.880 | unscoped / unscoped | 3,953,208 | 3,953,208 |
| `$07:ce2a` | 2,738 / 0 / 2,738 | 1,326.000 | unscoped / 1,326.000 | 3,630,588 | 0 |
| `$07:b7c7` | 2,707 / 0 / 0 | 1,340.590 | unscoped / unscoped | 3,628,976 | 3,628,976 |
| `$07:d9d8` | 28,295 / 28,295 / 28,295 | 841.864 | 725.157 / 725.157 | 3,302,216 | 3,302,216 |
| `$07:cdcb` | 2,269 / 0 / 2,269 | 1,378.000 | unscoped / 1,378.000 | 3,126,682 | 0 |
| `$07:b64f` | 2,677 / 0 / 0 | 1,164.770 | unscoped / unscoped | 3,118,090 | 3,118,090 |
| `$07:ce85` | 2,305 / 0 / 2,305 | 988.000 | unscoped / 988.000 | 2,277,340 | 0 |
| `$07:9873` | 4,143 / 0 / 4,143 | 486.000 | unscoped / 486.000 | 2,013,498 | 0 |

The four routine gaps close by a summed **11,048,108 instruction clocks** on this unchanged cohort. This is instruction-work reconciliation; small hold changes are expected when other work in the iteration remains unpriced.

`$07:D077` still lacks its atomic item/coordinate/response self work. `$07:BA0A/$C4D4` still lack atomic velocity/direction selection and their indoor/outdoor collision-response tails. `$07:B7C7/$B64F` still lack atomic layer-dispatch and collision-response self work. Their retained instruction continuations are separate from these atomic paths; their unscoped cohort rows are not call-equivalence evidence. These bodies were not converted into constant per-call prices.

`$07:D9D8` already prices lookup and indirect dispatch. Its remaining gap includes selected tile-effect handlers reached by JMP; those handlers retain the source frame. The listing's prefix alone is not authority for the effect-body price. `$07:807F` mixes its already priced entry with source-inherited handler/tail work and native handler labels. `$07:E245` likewise has already priced movement tails under `$E373/$E3E0/$E42A/$E4D3` and other owners. Their raw gaps contain attribution differences as well as possibly missing work. Additional prices for these three were skipped pending exact path/ancestry reconciliation; a pure alias-only diagnosis for the whole deficit is not claimed.

### Route-wide exclusive clocks per call

Both fresh native ROM censuses profile all 1,473,123 iterations. Every exclusive routine sum reconciles to its main-instruction counter. Source clocks/call for the four straight-line routines are their instruction-derived invariant costs above, independently checked by the CPU-only fixtures and all cohort rows. Every final route-wide native call also has exactly that exclusive self cost.

| Routine | Source invariant clocks/call | Native calls before / after | Native clocks/call before / after |
| --- | ---: | ---: | ---: |
| `$07:d077` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:807f` | unavailable | 1,051,125 / 1,051,125 | 205.201 / 205.201 |
| `$07:e245` | unavailable | 976,645 / 976,645 | 1,853.645 / 1,853.645 |
| `$07:ba0a` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:c4d4` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:ce2a` | 1326 | 0 / 550,536 | unscoped / 1,326.000 |
| `$07:b7c7` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:d9d8` | unavailable | 6,713,923 / 6,713,923 | 767.095 / 767.095 |
| `$07:cdcb` | 1378 | 0 / 445,239 | unscoped / 1,378.000 |
| `$07:b64f` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:ce85` | 988 | 0 / 486,813 | unscoped / 988.000 |
| `$07:9873` | 486 | 0 / 1,742,042 | unscoped / 486.000 |

Full-route source call counts and empirical source per-call means for the other eight routines are unavailable in the retained source profile. That profile covers the frozen missing cohort. This short group does not claim full-route source/native call equivalence or twelve-routine completion. No partial source-route capture is used as full-route evidence.

### CPU-only verification

Both candidate full census gates and both fresh main controls PASS: 1,473,123 iterations each, unexplained zero. Runtime holds equal P2 on every candidate ROM row. The metrics are fresh executions. These periodically seeded logical checks do not establish continuous physical-field or exact A/V parity.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 5,011 / 4,998 | 0 / 0 | 0 / 0 | 34,618 / 34,631 | 39,561 / 39,561 |
| live | 5,708 / 5,695 | 590 / 590 | 132 / 132 | 34,484 / 34,497 | 39,429 / 39,429 |

## Continuation on `codex/cost-pacing24b` (main `dc86c1ef`)

Evidence is retained in `target/cost-pacing24b/`, with the completed group in `corrected/`. This short batch stops after one verified group: `$09:B5DE` garnish coordinate preparation and `$0E:D7FE` per-color whitening. The remaining requested bodies are assessed below; no second actor-body group was added. No Link-bank-07 routine or shared ledger helper was edited.

### Executed ROM work

`scripts/rom_block_costs.py` produced all twelve source listings; whitening uses M16/X8, the other entries M8/X8. New prices are lagless-only and include source-address comments. No lag count or fitted residual supplies a price.

`$09:B5DE..B612` prices the actual X-first/Y-second coordinate subtraction, branches, saved-byte pop and double-return exits: 366 total for X rejection, 568 total for Y rejection after X succeeds, or 518 for success. On rejection its source frame owns 296/498; the final 70 clocks are attributed to the dispatcher. Its existing native coordinate helper now owns the real source call frame. The caller’s JSR and drawing body are not included. The superseded `group1/` trial kept the correct total work but put the final PLA/RTS in the helper frame; `corrected/` fixes that ownership without changing any total instruction charge.

`$0E:D7FE..D839` prices each actual auxiliary palette input: 572 straight-line clocks plus, for each channel, 6 for the taken unclamped BCC or 24 for the executed immediate clamp. The existing 96-color whitening operation supplies the call count. The optional dim-flashes output remains unchanged; prices follow the cartridge’s +14 branches. No wrapper, caller, iteration threshold or per-scene price was added.

The CPU-only executed-ROM tests pass 288 garnish fixtures and 125 whitening fixtures. They exercise both garnish slot extremes, independent X/Y rejection, scroll wrap, all clamp boundary combinations including equality, and auxiliary colors different from main colors. They compare actual ROM instruction transactions with native self work, check whitening output, and verify default calls add zero price.

### Frozen 2,825-row cohort

This is batch 22’s fixed `07/00` + `09/00` missing cohort, including any iterations this group closes. Calls below are source / fresh main / candidate. Per-call clocks are aggregate exclusive clocks divided by each lane’s own calls, not individually paired calls. A missing native scope is reported as unscoped. `routine-cohort-before-after.json` retains equal-call row counts and row-mean gap ranges.

| Routine | Calls source / before / after | Source clocks/call | Native before / after clocks/call | Signed gap before | Signed gap after |
| --- | ---: | ---: | ---: | ---: | ---: |
| `$06:dc10` | 11,899 / 11,951 / 11,951 | 954.712 | 46.000 / 46.000 | 10,810,378 | 10,810,378 |
| `$09:b0b6` | 68,085 / 68,085 / 68,085 | 364.315 | 231.725 / 231.755 | 9,027,408 | 9,025,378 |
| `$1e:8b19` | 9,306 / 9,306 / 9,306 | 1,231.806 | 302.381 / 302.381 | 8,649,224 | 8,649,224 |
| `$06:84e2` | 45,200 / 45,200 / 45,200 | 560.855 | 431.116 / 431.116 | 5,864,200 | 5,864,200 |
| `$0e:d7fe` | 9,504 / 0 / 9,504 | 606.468 | unscoped / 606.468 | 5,763,870 | 0 |
| `$08:833c` | 2,551 / 2,551 / 2,551 | 2,595.067 | 550.841 / 550.841 | 5,214,820 | 5,214,820 |
| `$1d:c222` | 4,874 / 4,874 / 4,874 | 1,326.640 | 293.059 / 293.059 | 5,037,674 | 5,037,674 |
| `$06:d22f` | 1,268 / 0 / 0 | 3,607.391 | unscoped / unscoped | 4,574,172 | 4,574,172 |
| `$09:b5de` | 8,564 / 0 / 8,564 | 517.862 | unscoped / 517.862 | 4,434,966 | 0 |
| `$06:e991` | 1,310 / 0 / 0 | 3,358.147 | unscoped / unscoped | 4,399,172 | 4,399,172 |
| `$05:b5d3` | 4,399 / 4,399 / 4,399 | 1,552.964 | 554.872 / 554.872 | 4,390,606 | 4,390,606 |
| `$08:a7ab` | 871 / 0 / 0 | 4,751.830 | unscoped / unscoped | 4,138,844 | 4,138,844 |

**Summed exclusive gap closed: 10,200,866 master clocks.** Both newly priced routines match source calls and exclusive clocks on every cohort row; no negative row is hidden by aggregate cancellation. The dispatcher additionally gains the real double-return tail, explained below; the other nine raw routine gaps remain unchanged.

`$06:DC10` is skipped as the established attribution case: native drawing/shadow continuations use `$06:DC13/$DC66`, while source retains `$DC10`; its call counts also differ. No raw alias deficit was priced. `$06:D22F` similarly includes drawing JMP tails and an unannotated absorbable-dispatch prefix; it remains open rather than pricing that whole raw deficit.

`$09:B0B6` already prices its empty/countdown/allocation/RTS dispatch; selected garnish drawing bodies remain unpriced. The new `$09:B5DE` child is separate. On rejection, `$09:B608` unwinds its source frame; `$B609..B60A` (PLA/RTS, 70 clocks) then belongs to this dispatcher. That real tail reduces the dispatcher’s exclusive gap; it is not removed from the work total. `$1E:8B19`, `$1D:C222` and `$05:B5D3` already price their 290-clock RTS dispatches; their selected actor bodies retain the remaining mixed ownership/unpriced work. `$06:84E2` already prices entry, timer-call and state dispatch instructions, with remaining state/actor bodies in its source frame. `$08:833C` already prices OAM allocation, timer and indirect dispatch instructions; selected ancilla bodies remain open. Those five parent gaps are not new constant prices.

`$06:E991..EA03` remains unpriced: zero-speed exit, sign/swap branches, the actual byte-accumulator projection loop and output signs need their own annotations, separate from `$EAE8/$EAD1` children. `$08:A7AB..A80C` remains unpriced: the actual explosion-tile skip/draw loop, indexed table crossings and caller setup need annotation, separate from `$08:F702` OAM work. This group does not claim complete instruction/ancestry reconciliation for the unpriced targets.

### Full-route comparison boundary

Both ROM censuses collected all native routine clocks, checking the sum of exclusive rows on every one of 1,473,123 iterations. Their aggregate native calls and exclusive clocks/call follow.

| Routine | Native calls before / after | Native clocks/call before / after |
| --- | ---: | ---: |
| `$06:dc10` | 2,350,894 / 2,350,894 | 46.000 / 46.000 |
| `$09:b0b6` | 23,398,320 / 23,398,320 | 152.746 / 152.764 |
| `$1e:8b19` | 2,487,531 / 2,487,531 | 338.889 / 338.889 |
| `$06:84e2` | 22,612,952 / 22,612,952 | 324.869 / 324.869 |
| `$0e:d7fe` | 0 / 99,264 | unscoped / 604.927 |
| `$08:833c` | 384,393 / 384,393 | 568.539 / 568.539 |
| `$1d:c222` | 843,363 / 843,363 | 317.098 / 317.098 |
| `$06:d22f` | 0 / 0 | unscoped / unscoped |
| `$09:b5de` | 0 / 569,935 | unscoped / 516.672 |
| `$06:e991` | 0 / 0 | unscoped / unscoped |
| `$05:b5d3` | 1,245,791 / 1,245,791 | 568.909 / 568.909 |
| `$08:a7ab` | 0 / 0 | unscoped / unscoped |

**Full-route source/native per-call reconciliation remains unfinished.** Retained source routine evidence covers the frozen missing cohort, not all route iterations. This batch did not repeat batch 23’s incomplete expensive source-route attempt. Full-route source totals, full-route source gaps and individually paired-call equivalence are not claimed; the native route-wide table cannot substitute for them.

### Full CPU-only checks

The ownership repair leaves every work/hold/predictor column identical to the superseded trial on both complete censuses. Corrected runs below were nevertheless executed fresh. Fresh main controls and candidate full gates PASS in both cartridge-answer and live-SPC modes: 1,473,123 iterations each, unexplained zero, every runtime hold equal to P2. These periodically seeded logical checks do not establish continuous physical-field or exact A/V parity. Missing/excess below count lag NMIs; false positives count iterations.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 5,011 / 5,003 | 0 / 0 | 0 / 0 | 34,618 / 34,626 | 39,561 / 39,561 |
| live | 5,708 / 5,700 | 590 / 590 | 132 / 132 | 34,484 / 34,492 | 39,429 / 39,429 |

The cartridge census closes 8 previously missing iterations and introduces 0 newly missing iterations. The main metric for this long tail remains the 10,200,866 source instruction clocks closed on the fixed cohort.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 370 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 334 / 334 | 0 / 0 | 0 / 0 | 1,642 / 1,642 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 84 / 84 | 0 / 0 | 0 / 0 | 4,795 / 4,795 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 263 / 263 | 0 / 0 | 0 / 0 | 2,656 / 2,656 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,672 / 1,672 | 0 / 103 | 0 / 19 | 1,288 / 1,288 | 2,960 / 2,960 |

| `0F/01` | 5,432 | 79 / 79 | 0 / 0 | 0 / 0 | 4,800 / 4,800 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 263 / 263 | 0 / 0 | 0 / 0 | 2,656 / 2,656 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,670 / 1,670 | 0 / 103 | 0 / 19 | 1,290 / 1,290 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 72 / 72 | 0 / 0 | 0 / 0 | 1,663 / 1,663 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,137 / 1,137 | 0 / 0 | 0 / 0 | 55 / 55 | 1,192 / 1,192 |

| `09/00` | 1,253 | 1,149 / 1,149 | 0 / 0 | 0 / 0 | 43 / 43 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 22 / 22 | 0 / 0 | 0 / 0 | 265 / 265 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 64 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 84 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 83 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts remain within one; all eight stable room/Link-XY/whole-save checks match. The 162000–163000 rupee window retains the same 27 requests and all request/PCM gaps as batch 23, median 8.253745 frames.

| Capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14741, "song-end-poll": 639}` | 10 |
| probes | `{}` | 0 |

The frozen 8,339 original missing rows gain 11,256,934 main instruction clocks, with no negative row and 0 HDMA upper-bound excursions. The same 63 original missing rows retain the masked-init decline flag.

Both ordinary builds are warning-free. The candidate default binary matches every record and EOF against the fresh clean-main default binary's 200k state/display/SRAM/PCM/input/context ledger. Default/ROM-timing behavior and prices remain unchanged over this check.

| Library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,525 | 1 | 3 |
| reduced | 2,060 | 0 | 2 |

The only default failure is the known spotlight caller-return fixture; reduced passes completely. Default lib-test compilation retains the same 24 dead-code warnings as batch 23, separate from the warning-free ordinary builds. Both SNES suites and Python routine-coverage tests pass. The library manifest verifies complete disjoint test unions. No GPU run, `--video`, cached-av, external Snes9x execution, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. All supplied ledger/cache paths stayed read-only. `git diff --check` passes. Stopped after this one verified pricing group; the remaining eight routines and full-route source-call reconciliation are unfinished.

All eight phase sweeps remain within one. All eight stable room, Link XY and whole-save outcomes match source. Fresh main and candidate probe metrics and stable outcomes are retained separately. The 162000–163000 rupee probe retains all 27 requests and every request/PCM onset gap against fresh main, with PCM-gap median 8.25374531835206 frames.

| Stage / capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| main / rom | `{"lag": 14}` | 10 |
| main / live | `{"lag": 14, "polygon": 14740, "song-end-poll": 639}` | 10 |
| main / probes | `{}` | 0 |
| final / rom | `{"lag": 14}` | 10 |
| final / live | `{"lag": 14, "polygon": 14741, "song-end-poll": 639}` | 10 |
| final / probes | `{}` | 0 |

The original 8,339 missing rows gain 10,561,412 main instruction clocks, with no negative row. HDMA upper-bound excursions remain zero; the same 47 masked-init leases remain declined.

Both ordinary candidate builds and the fresh main default build are warning-free. The candidate default binary matches every state/display/SRAM/PCM/input/context record and EOF of a freshly recorded main 200k ledger. New scopes and prices are gated by lagless mode; default/ROM-timing accounting remains unchanged.

| Zelda library | Pass | Fail | Ignored |

## Continuation on `codex/cost-pacing25a` (main `48fbb09c`)

Evidence is in `target/cost-pacing25a/`; scratch drivers are `/tmp/cost25a-*.py`. One verified group adds partial self work to `$07:BA0A/$C4D4`. All eight requested routines have fresh ROM listings and cohort/native-route profiles. The response bodies and the six other gaps remain open.

### Instruction-derived selection and dispatch

`scripts/rom_block_costs.py` prices `$07:BA0A..BA34` and `$07:C4D4..C4FE` at M8/X8. Zero velocity executes LDA, untaken BNE and RTS: 82 clocks. A nonzero entry costs 46, the doorway comparison 56, and the direction path costs 68 or 78 outside an aligned doorway, or 78 or 110 inside it. TYA/LSR/STA/JSR adds 98. After the separately owned movement probe returns, LDA $1B/BNE costs 46 indoors or 70 including the outdoor BRL. Thus ordinary negative/positive paths cost 314/324 indoors and 338/348 outdoors; aligned-doorway low/high position paths cost 324/356 indoors and 348/380 outdoors. These totals cover the prefix only. No response-tail RTS is invented for nonzero velocity.

New charges and routine scopes are lagless-only, with source addresses in comments. The shared `start_movement_collision_checks` helper changes only these two requested entries. Its existing response dispatch is untouched. Retained default/ROM-timing continuations keep their instruction models and receive no new charge because lagless pricing is disabled there. No scene threshold, multiplier, residual or lag count supplies a price; no sprite/ancilla/overworld production routine was edited.

The CPU-only executed-ROM regression exercises 240 combinations of axis, indoor/outdoor location, doorway state, zero/positive/negative velocity and positions on both sides of $80, with default and lagless native controls. It executes real movement-probe children and stops source execution at the selected response entry or zero-velocity return. Only the wrapper prefix is counted; native exclusive clocks must match it, independently of the unpriced response body.

### Frozen 2,825-row cohort: eight requested routines

Calls are source / native-before / native-after. Means use each lane's own call count. Unscoped is not an observed zero-cost call. The source means include the complete source routine; the new native means cover the priced prefix, so a remaining gap is expected. `cohort-before-after.json` retains equal-call row counts and per-row mean gap ranges.

| Routine | Calls source / before / after | Source clocks/call | Native before / after clocks/call | Gap before | Gap after |
| --- | ---: | ---: | ---: | ---: | ---: |
| `$07:d077` | 3,249 / 0 / 0 | 1,980.903 | unscoped / unscoped | 6,435,954 | 6,435,954 |
| `$07:807f` | 2,784 / 2,784 / 2,784 | 2,395.971 | 214.132 / 214.132 | 6,074,240 | 6,074,240 |
| `$07:e245` | 2,353 / 2,353 / 2,353 | 4,430.403 | 1,918.673 / 1,918.673 | 5,910,100 | 5,910,100 |
| `$07:ba0a` | 2,754 / 0 / 2,754 | 1,988.130 | unscoped / 277.481 | 5,475,310 | 4,711,126 |
| `$07:c4d4` | 2,757 / 0 / 2,757 | 1,433.880 | unscoped / 239.765 | 3,953,208 | 3,292,176 |
| `$07:b7c7` | 2,707 / 0 / 0 | 1,340.590 | unscoped / unscoped | 3,628,976 | 3,628,976 |
| `$07:d9d8` | 28,295 / 28,295 / 28,295 | 841.864 | 725.157 / 725.157 | 3,302,216 | 3,302,216 |
| `$07:b64f` | 2,677 / 0 / 0 | 1,164.770 | unscoped / unscoped | 3,118,090 | 3,118,090 |

Summed cohort gap closed: **1,425,216 instruction clocks**. On every cohort row the added main-instruction work equals the new wrappers' exclusive work; this reduction does not come from reattributing previously charged child work.

`$07:D077` still lacks its atomic item/coordinate/response work. `$07:BA0A/$C4D4` now price selection and response dispatch but retain unpriced indoor/outdoor response tails. `$07:B7C7/$B64F` still lack atomic layer dispatch and response work; their instruction continuations are separate owners.

`$07:D9D8` prices lookup and indirect dispatch but selected JMP effect bodies inherit its source frame. `$07:807F` prices its entry while native handler labels divide source-inherited work. `$07:E245` prices selection while movement tails retain `$E373/$E3E0/$E42A/$E4D3` owners. No additional charge was added to these three: their raw deficits mix attribution with potentially missing instructions, and a pure alias-only explanation has not been established.

### Route-wide native exclusive clocks per call

Fresh before/after native ROM profiles cover all 1,473,123 iterations. Every exclusive sum reconciles to its main-instruction counter.

| Routine | Source clocks/call | Native calls before / after | Native clocks/call before / after |
| --- | ---: | ---: | ---: |
| `$07:d077` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:807f` | unavailable | 1,051,125 / 1,051,125 | 205.201 / 205.201 |
| `$07:e245` | unavailable | 976,645 / 976,645 | 1,853.645 / 1,853.645 |
| `$07:ba0a` | unavailable | 0 / 748,936 | unscoped / 227.390 |
| `$07:c4d4` | unavailable | 0 / 748,917 | unscoped / 220.985 |
| `$07:b7c7` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:d9d8` | unavailable | 6,713,923 / 6,713,923 | 767.095 / 767.095 |
| `$07:b64f` | unavailable | 0 / 0 | unscoped / unscoped |

Full-route source call counts and empirical source per-call means for these eight routines are unavailable in the retained source profile. That profile covers the frozen missing cohort. This short group does not claim full-route source/native call equivalence or eight-routine completion. No partial source-route capture is used as full-route evidence.

### CPU-only verification

Both candidate full census gates and both fresh main controls PASS: 1,473,123 iterations each, unexplained zero. Runtime holds equal P2 on every candidate ROM row. The metrics are fresh executions. These periodically seeded logical checks do not establish continuous physical-field or exact A/V parity.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,998 / 4,997 | 0 / 0 | 0 / 0 | 34,631 / 34,632 | 39,561 / 39,561 |
| live | 5,695 / 5,694 | 590 / 590 | 132 / 132 | 34,497 / 34,498 | 39,429 / 39,429 |

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 370 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 334 / 334 | 0 / 0 | 0 / 0 | 1,642 / 1,642 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 84 / 84 | 0 / 0 | 0 / 0 | 4,795 / 4,795 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 263 / 263 | 0 / 0 | 0 / 0 | 2,656 / 2,656 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,672 / 1,672 | 0 / 103 | 0 / 19 | 1,288 / 1,288 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 72 / 72 | 0 / 0 | 0 / 0 | 1,663 / 1,663 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,136 / 1,136 | 0 / 0 | 0 / 0 | 56 / 56 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 22 / 22 | 0 / 0 | 0 / 0 | 265 / 265 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 64 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 84 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 83 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All phase counts remain within one; all eight stable room/Link-XY/whole-save checks match. The 162000–163000 rupee window retains the same 27 requests and all request/PCM gaps as batch 23, median 8.253745 frames.

| Capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14741, "song-end-poll": 639}` | 10 |
| probes | `{}` | 0 |

The frozen 8,339 original missing rows gain 1,442,068 main instruction clocks, with no negative row and 0 HDMA upper-bound excursions. The same 63 original missing rows retain the masked-init decline flag.

Both ordinary builds are warning-free. The candidate default binary matches every record and EOF against the fresh clean-main default binary's 200k state/display/SRAM/PCM/input/context ledger. Default/ROM-timing behavior and prices remain unchanged over this check.

| Library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,526 | 1 | 3 |
| reduced | 2,061 | 0 | 2 |

The only default library failure is the known spotlight caller-return fixture; reduced passes completely. The original default shards 1 and 4 were interrupted after 520 and 380 completed tests. Their 113- and 252-test tails were rerun from the bound test binary; the manifest verifies the complete, disjoint named union. Completed original statuses are retained separately from recovery logs. Both SNES library suites and Python routine-coverage checks pass. `git diff --check` passes.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`; supplied ledger/cache and the other checkout’s evidence stayed read-only. This stops at the verified two-routine group; the remaining pricing and full-route source reconciliation are open.

The only default failure is the known spotlight caller-return fixture; reduced passes completely. Default lib-test compilation retains the same 24 dead-code warnings as batch 23, separate from the warning-free ordinary builds. Both SNES suites and Python routine-coverage tests pass. The library manifest verifies complete disjoint test unions. No GPU run, `--video`, cached-av, external Snes9x execution, cargo GPU tests, oracle-cache writes, cargo fmt, commit or push was used. All supplied ledger/cache paths stayed read-only. `git diff --check` passes. Stopped after one verified prefix-pricing group. The two response tails, the six other routine gaps and full-route source-call reconciliation remain open.

## Continuation on `codex/cost-pacing25b` (joint main `056f2cb1`)

Evidence is in `target/cost-pacing25b/`. One verified group prices `$06:E991` sprite speed projection and `$08:A7AB` explosion drawing. No bank-07 routine or shared helper changed.

### Combined main baseline

These controls were freshly built and executed from main with both batches 24a and 24b, plus the masked elapsed-field owner. They are the first joint census; individual prior batch deltas were not added together.

| Fresh main control | Missing | Excess | FP | Exact lagged | Within one |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,989 | 0 | 0 | 34,640 | 39,561 |
| live | 5,686 | 590 | 132 | 34,506 | 39,429 |

### ROM instruction work

`scripts/rom_block_costs.py` generated all ten M8/X8 listings. Every added scope and price is lagless-only, with source addresses in comments. No lag count, scene threshold, multiplier or residual supplies a price.

`$06:E991..EA03` includes the 128-clock zero-speed exit through `$E98E`, the actual input sign/axis-swap paths, and the byte accumulator loop. Each executed speed pass costs 168, the final not-taken BNE removes 6, and each actual subtraction replaces taken BCC(6) with SBC/INC(62), adding 56. The unsigned minor output before axis/sign restoration counts those executed subtractions, including byte-wrap cases; no second projection loop or shared-helper change was needed. The `$EAE8/$EAD1` children remain separate and unpriced on this caller path.

`$08:A7AB..A80C` includes 64 per tile test, 6 for a skipped tile, 1,100 for a drawn tile plus actual `$9D7C,X/$9D7E,X` page crossings, 130 per loop suffix plus taken BNE(6), and RTS(42). `$08:F702` child work and caller setup are excluded. The tile/flag/ext indices stay inside their pages over the existing 54-entry native tables; offset-word crossings use the actual four-byte table index.

Two CPU-only ROM fixtures compare native self work with executed instruction transactions: 182 sprite projection cases (zero speed, both slot extremes, both axes, signs, equality, large speeds and byte wrap) and 162 explosion cases (all nine frame groups, skip/draw paths, page crossings and clipping coordinates). Projection outputs also match ROM; default calls add zero price.

### Frozen 2,825-row cohort

This is the unchanged batch-22 `07/00` + `09/00` cohort, including newly closed iterations. Calls and exclusive clocks/call are each lane’s aggregate values, not individually paired calls. All ten scoped targets are reported.

| Routine | Calls source / before / after | Source clocks/call | Native before / after clocks/call | Signed gap before | Signed gap after |
| --- | ---: | ---: | ---: | ---: | ---: |
| `$06:dc10` | 11,899 / 11,951 / 11,951 | 954.712 | 46.000 / 46.000 | 10,810,378 | 10,810,378 |
| `$09:b0b6` | 68,085 / 68,085 / 68,085 | 364.315 | 231.755 / 231.755 | 9,025,378 | 9,025,378 |
| `$1e:8b19` | 9,306 / 9,306 / 9,306 | 1,231.806 | 302.381 / 302.381 | 8,649,224 | 8,649,224 |
| `$06:84e2` | 45,200 / 45,200 / 45,200 | 560.855 | 431.116 / 431.116 | 5,864,200 | 5,864,200 |
| `$08:833c` | 2,551 / 2,551 / 2,551 | 2,595.067 | 550.841 / 550.841 | 5,214,820 | 5,214,820 |
| `$1d:c222` | 4,874 / 4,874 / 4,874 | 1,326.640 | 293.059 / 293.059 | 5,037,674 | 5,037,674 |
| `$06:d22f` | 1,268 / 0 / 0 | 3,607.391 | unscoped / unscoped | 4,574,172 | 4,574,172 |
| `$06:e991` | 1,310 / 0 / 1,310 | 3,358.147 | unscoped / 3,358.147 | 4,399,172 | 0 |
| `$05:b5d3` | 4,399 / 4,399 / 4,399 | 1,552.964 | 554.872 / 554.872 | 4,390,606 | 4,390,606 |
| `$08:a7ab` | 871 / 0 / 871 | 4,751.830 | unscoped / 4,751.830 | 4,138,844 | 0 |

**Summed exclusive gap closed: 8,538,016 master clocks.** The two priced routines match source call counts and exclusive clocks on every frozen-cohort row; other requested raw routine gaps are unchanged.

`$06:DC10` is skipped as an attribution artifact: native drawing/shadow continuations use `$DC13/$DC66`, while source keeps `$DC10`, with different call counts. `$06:D22F` also contains drawing JMP tails and an absorbable-dispatch prefix; its raw gap is not a price.

`$09:B0B6`, `$1E:8B19`, `$1D:C222` and `$05:B5D3` already price their dispatch instructions; selected drawing/actor bodies in those source frames remain open. `$06:84E2` already prices entry/timer/state dispatch, and `$08:833C` already prices allocation/timer/indirect dispatch. None of those mixed body gaps was converted into a constant parent charge. No second group was attempted.

### Route-wide exclusive clocks per call

Both fresh native ROM censuses collect the full route and reconcile exclusive routine sums to main clocks on every one of 1,473,123 iterations.

| Routine | Native calls before / after | Native clocks/call before / after |
| --- | ---: | ---: |
| `$06:dc10` | 2,350,894 / 2,350,894 | 46.000 / 46.000 |
| `$09:b0b6` | 23,398,320 / 23,398,320 | 152.764 / 152.764 |
| `$1e:8b19` | 2,487,531 / 2,487,531 | 338.889 / 338.889 |
| `$06:84e2` | 22,612,952 / 22,612,952 | 324.869 / 324.869 |
| `$08:833c` | 384,393 / 384,393 | 568.539 / 568.539 |
| `$1d:c222` | 843,363 / 843,363 | 317.098 / 317.098 |
| `$06:d22f` | 0 / 0 | unscoped / unscoped |
| `$06:e991` | 0 / 65,748 | unscoped / 3,422.290 |
| `$05:b5d3` | 1,245,791 / 1,245,791 | 568.909 / 568.909 |
| `$08:a7ab` | 0 / 27,304 | unscoped / 2,707.973 |

**Full-route source/native per-call reconciliation remains unfinished.** Retained source routine evidence covers the frozen missing cohort, not all route iterations. The retained full source-route attempt stopped at about 201k iterations; it is not a complete route control. This short batch did not rerun that expensive attempt. Full-route source totals, full-route source gaps and individually paired-call equivalence are not claimed; the native route-wide table cannot substitute for them.

### Full CPU-only checks

Fresh main controls and candidate full gates PASS in both cartridge-answer and live-SPC modes: 1,473,123 iterations each, unexplained zero, every runtime hold equal to P2. These periodically seeded logical checks do not establish continuous physical-field or exact A/V parity. Missing/excess below count lag NMIs; false positives count iterations.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,989 / 4,986 | 0 / 0 | 0 / 0 | 34,640 / 34,643 | 39,561 / 39,561 |
| live | 5,686 / 5,683 | 590 / 590 | 132 / 132 | 34,506 / 34,509 | 39,429 / 39,429 |

The cartridge census closes 3 previously missing iterations and introduces 0 newly missing iterations. The main metric for this long tail remains the 8,538,016 source instruction clocks closed on the fixed cohort.

## Continuation on `codex/cost-pacing26a` (main `d7096adc`)

Evidence is in `target/cost-pacing26a/`; scratch drivers are `/tmp/cost26a-*.py`. This short batch stops after one verified partial response group for `$07:BA0A/$C4D4`. No sprite, ancilla, overworld, shared ledger or cycle-model production code was edited.

### Fresh main joint baseline

These fresh complete controls census merged batches 24a, 24b and 25a together, with the masked elapsed-field owner included. The main default binary was rebuilt while production sources were clean, then reproduced byte for byte in a second clean build. `main-clean-confirmed` binds its SHA-256; `main-joint-baseline.json` retains both controls.

| Main census | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,988 | 0 | 0 | 34,641 | 39,561 |
| live | 5,685 | 590 | 132 | 34,507 | 39,429 |

### Instruction work and ownership

`scripts/rom_block_costs.py` supplies the retained M8/X8 listings. The new indoor range is `$07:BA35..BAF3` / `$07:C4FF..C59D`: carrying/incapacitation short circuits, doorway/orthogonal/layer selection, collision bits and force-direction selection, doorway response, optional X alignment and lock clears. Only selected call/jump opcodes are charged; called and jumped-to bodies outside these ranges receive no new charge. JMP/BRL tails retain their existing source-frame attribution. Early RTS instructions are priced only on paths that execute them. Ordinary clear-floor indoor setup costs 482 clocks for either axis. The later slope/tile response beginning at `$BAF4/$C59E` is still open.

The new outdoor range is `$07:BEAF..BEF7` / `$07:C8E9..C92E`: axis-specific speed setup, pit tests and inline falling return, or liftable-tile selection/store. The ordinary non-dash, non-pit, non-liftable Y path costs 226 clocks; the X path costs 290 clocks without an orthogonal direction or 210 clocks with one. The later water/ledge/movement response starting at `$BEF8/$C92F` is still open.

All charges are lagless-only and carry ROM address comments. The minimal shared `start_movement_collision_checks` change invokes pricing after the real movement probe and before its actual response. Direct continuation/response consumers receive no additional charge. No native behavior, scene threshold, multiplier, residual, lag count, default charge or ROM-timing model was changed.

The CPU-only executed-ROM fixture covers 1,440 response cases across both axes, including carrying, incapacitation, doorway/orthogonal/layer branches, collision masks, locks, pit handler returns, speed and liftable setup. It executes real child instructions and counts only the priced source ranges. The expanded 240-case wrapper fixture executes through the full response and checks the selected source ranges against native exclusive work, with default and lagless controls. All 37 focused instruction-price tests pass.

`CalculateSnapScratch_Y` at `$07:C1FF..C23C` changes X before `$BABE` reads it. The final price follows this actual child result; the `$24` collision fixture exercises the post-call alignment branch. The initial build and focused results are retained under `initial-build/`; all final builds, suites and censuses use the corrected source binding.

### Frozen 2,825-row cohort

Calls are source / native-before / native-after; each mean uses its own lane's call count. Unscoped is not an observed zero-cost call. Source means cover each entire source routine; the two changed native means cover the accumulated entry and partial response work.

| Routine | Calls source / before / after | Source clocks/call | Native clocks/call before / after | Gap before | Gap after |
| --- | ---: | ---: | ---: | ---: | ---: |
| `$07:d077` | 3,249 / 0 / 0 | 1,980.903 | unscoped / unscoped | 6,435,954 | 6,435,954 |
| `$07:807f` | 2,784 / 2,784 / 2,784 | 2,395.971 | 214.132 / 214.132 | 6,074,240 | 6,074,240 |
| `$07:e245` | 2,353 / 2,353 / 2,353 | 4,430.403 | 1,918.673 / 1,918.673 | 5,910,100 | 5,910,100 |
| `$07:ba0a` | 2,754 / 2,754 / 2,754 | 1,988.130 | 277.481 / 581.081 | 4,711,126 | 3,875,012 |
| `$07:c4d4` | 2,757 / 2,757 / 2,757 | 1,433.880 | 239.765 / 469.001 | 3,292,176 | 2,660,172 |
| `$07:b7c7` | 2,707 / 0 / 0 | 1,340.590 | unscoped / unscoped | 3,628,976 | 3,628,976 |
| `$07:d9d8` | 28,295 / 28,295 / 28,295 | 841.864 | 725.157 / 725.157 | 3,302,216 | 3,302,216 |
| `$07:b64f` | 2,677 / 0 / 0 | 1,164.770 | unscoped / unscoped | 3,118,090 | 3,118,090 |

Summed gap closed: **1,468,118 instruction clocks**. On every cohort row the main-instruction increase equals the BA0A/C4D4 exclusive-clock increase. Existing child clocks were not reassigned.

The six unchanged routines remain open: `$D077` has unpriced item-dependent coordinate/dispatch and response work; `$B7C7/$B64F` have unpriced layer/direction/response work in their atomic paths, distinct from retained instruction continuations. `$807F` handler labels divide source-inherited control work; `$E245` excludes separately owned `$E373/$E3E0/$E42A/$E4D3` movement tails; `$D9D8` prices lookup/indirect dispatch while selected JMP effect bodies inherit its source frame. Those last three raw gaps mix attribution and potentially missing work. No pure alias-only conclusion, fitted residual price or completion claim is made. Fresh listings for all eight are retained.

### Route-wide native exclusive clocks per call

Fresh native profiles cover every one of 1,473,123 iterations. Exclusive routine sums reconcile with their main counters on every row. The retained source profile covers only the frozen missing cohort: full-route source call counts and empirical source clocks/call are unavailable. No partial source sample is substituted.

| Routine | Route-wide source clocks/call | Native calls before / after | Native clocks/call before / after |
| --- | ---: | ---: | ---: |
| `$07:d077` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:807f` | unavailable | 1,051,125 / 1,051,125 | 205.201 / 205.201 |
| `$07:e245` | unavailable | 976,645 / 976,645 | 1,853.645 / 1,853.645 |
| `$07:ba0a` | unavailable | 748,936 / 748,936 | 227.390 / 474.080 |
| `$07:c4d4` | unavailable | 748,917 / 748,917 | 220.985 / 454.403 |
| `$07:b7c7` | unavailable | 0 / 0 | unscoped / unscoped |
| `$07:d9d8` | unavailable | 6,713,923 / 6,713,923 | 767.095 / 767.095 |
| `$07:b64f` | unavailable | 0 / 0 | unscoped / unscoped |

### CPU-only verification

Both candidate full census gates and both fresh main controls PASS: 1,473,123 iterations each, unexplained zero. Runtime holds equal P2 on every candidate ROM row. Every category retains the same missing/excess/FP and exact/within-one hold metrics as fresh main. These periodically seeded logical CPU checks do not establish continuous physical-field or exact A/V parity. Missing/excess count lag NMIs; FP counts iterations.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,988 / 4,988 | 0 / 0 | 0 / 0 | 34,641 / 34,641 | 39,561 / 39,561 |
| live | 5,685 / 5,685 | 590 / 590 | 132 / 132 | 34,507 / 34,507 | 39,429 / 39,429 |

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 370 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 334 / 334 | 0 / 0 | 0 / 0 | 1,642 / 1,642 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 363 / 363 | 0 / 0 | 0 / 0 | 816 / 816 | 1,179 / 1,179 |

| `09/23` | 6,054 | 365 / 365 | 0 / 0 | 0 / 0 | 814 / 814 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 79 / 79 | 0 / 0 | 0 / 0 | 4,800 / 4,800 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 263 / 263 | 0 / 0 | 0 / 0 | 2,656 / 2,656 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,668 / 1,668 | 0 / 103 | 0 / 19 | 1,292 / 1,292 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 72 / 72 | 0 / 0 | 0 / 0 | 1,663 / 1,663 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,136 / 1,136 | 0 / 0 | 0 / 0 | 56 / 56 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 22 / 22 | 0 / 0 | 0 / 0 | 265 / 265 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |

| Crystal/pendant seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 64 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 84 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 83 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase sweeps remain within one. All eight stable room, Link XY and whole-save outcomes match source. Fresh main and candidate probe metrics and stable outcomes are retained separately. The 162000–163000 rupee probe retains all 27 requests and every request/PCM onset gap against fresh main, with PCM-gap median 8.25374531835206 frames.

| Stage / capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| main / rom | `{"lag": 14}` | 10 |
| main / live | `{"lag": 14, "polygon": 14740, "song-end-poll": 639}` | 10 |
| main / probes | `{}` | 0 |
| final / rom | `{"lag": 14}` | 10 |
| final / live | `{"lag": 14, "polygon": 14740, "song-end-poll": 639}` | 10 |
| final / probes | `{}` | 0 |

The original 8,339 missing rows gain 8,617,502 main instruction clocks, with no negative row. HDMA upper-bound excursions remain zero; the same 63 masked-init leases remain declined.

Both ordinary candidate builds and the fresh main default build are warning-free. The candidate default binary matches every state/display/SRAM/PCM/input/context record and EOF of a freshly recorded main 200k ledger. New scopes and prices are gated by lagless mode; default/ROM-timing accounting remains unchanged.

Default lib-test compilation retains the same 24 fixture dead-code warnings as the prior batch, separate from the warning-free ordinary builds.

| Zelda library | Pass | Fail | Ignored |

All phase counts remain within one; all eight stable room/Link-XY/whole-save checks match. The 162000–163000 rupee window retains the same 27 requests, request gaps and PCM onset gaps as batch 23; median PCM gap is 8.253745 frames. `final-probe-metrics.json` retains every request and gap.

| Capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| rom | `{"lag": 14}` | 10 |
| live | `{"lag": 14, "polygon": 14740, "song-end-poll": 639}` | 10 |
| probes (162000–163000) | `{}` | 0 |

The frozen 8,339 original missing rows gain 1,481,986 main instruction clocks, with no negative row and 0 HDMA upper-bound excursions. The same 63 original missing rows retain the masked-init decline flag.

Both final ordinary builds are warning-free. The final default binary matches every state/display/SRAM/PCM/input/context record and EOF against the fresh clean-main default 200k ledger. Default and ROM-timing behavior and charges remain unchanged over this check.

| Library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,529 | 1 | 3 |
| reduced | 2,064 | 0 | 2 |

The only default library failure is the known spotlight caller-return fixture; reduced passes completely. The library manifest verifies complete, disjoint named test unions. Both SNES library suites and Python routine-coverage checks pass. `git diff --check` passes.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache and the other checkout’s evidence remained read-only. This stops at one verified two-routine group. Remaining body prices and full-route source reconciliation are unfinished.

The sole default failure is the known spotlight caller-return fixture; reduced passes completely. Both SNES library suites and Python routine-coverage tests pass. The library manifest verifies complete disjoint named unions and binary identities. Lib-test fixture dead-code warnings are separate from the warning-free ordinary builds. `git diff --check` passes.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Supplied ledger/cache paths and the other checkout stayed read-only. Scratch drivers stayed in `/tmp`. This stops after one verified partial response group; later BA0A/C4D4 response work, the six other routines and full-route source reconciliation remain open.

## Continuation on `codex/cost-pacing26b` (joint main `ca939dcc`)

Evidence is in `target/cost-pacing26b/`; scratch drivers are `/tmp/cost26b-*.py`. One verified group prices seven garnish drawing paths within `$09:B0B6`. No bank-07 routine or cross-system helper changed. The two shared helpers changed are the local garnish sparkle/dust implementations; their only callers are the four corresponding garnish handlers. Other scoped routines remain open.

### Joint main baseline

These are fresh full controls from clean main `ca939dcc`, including 24a, 24b, 25a, 25b and the masked elapsed-field owner. This is the first joint 25a/25b census; individual prior deltas were not added.

| Fresh main control | Missing | Excess | FP | Exact lagged | Within one |

## Continuation on `codex/cost-pacing27a` (main `1ec81741`)

### Fresh joint main baseline

Evidence: `target/cost-pacing27a/`; scratch drivers: `/tmp/cost27a-*.py`. Main was built with a clean production tree before edits. `main-provenance.json` binds HEAD, tracked source objects and the fresh default binary. These are fresh joint controls with 25b, 26a and the masked elapsed-field owner; the individual historical deltas were not added to produce this baseline.

| Main census | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,985 | 0 | 0 | 34,644 | 39,561 |
| live | 5,684 | 592 | 132 | 34,510 | 39,429 |

### ROM instruction work

`scripts/rom_block_costs.py` generated fresh listings for all eight requested routines and the seven added garnish paths. `source-price-instructions.json` independently sums their listed instructions. Prices are lagless-only, cite source addresses and follow actual entry/draw/reject paths. No lag count, scene threshold, multiplier or residual supplies a price.

| Handler entry | Prefix through JSR prep | Successful drawing suffix |
| --- | ---: | ---: |
| `$09:b520` (sparkle) | 160 | 702 |
| `$09:b526` (simple-sparkle) | 152 | 702 |
| `$09:b3bc` (dust) | 168 | 526 |
| `$09:b3c2` (water-trail) | 160 | 526 |
| `$09:b5bb` (laser) | 46 | 524 |
| `$09:b4fb` (zoro) | 46 | 580 |
| `$09:b6c0` (snake) | 46 | 580 |

The prefix includes the actual BRA or third LSR for the two sparkle and two dust entries. Drawing suffixes include the OAM writes, shared finalizer JMP and RTS. Native character and sprite indices remain within the indexed table pages. `$09:B5DE` coordinate preparation keeps its existing child scope and price. On rejection only the new prefix executes; the existing helper retains ownership of the first unwind PLA, while its second PLA and RTS remain caller work. Handler bodies remain inside the source RTS-dispatch frame `$09:B0B6`; no synthetic nested call frame is introduced.

The CPU-only executed-ROM fixture covers 432 cases: all seven entries, every character frame, both garnish slot extremes, both sprite-index extremes, successful coordinate edges, and X/Y rejection including wrapped negatives. It checks exclusive caller work, total work, OAM bytes, deletion and zero default-mode charge. The first fixture attempt incorrectly attributed the prep child's 126-clock successful suffix to the caller; its rejected log is retained. Correcting the fixture boundary required no production-price change.

### Frozen 2,825-row cohort

This is the fixed batch-22 `07/00` + `09/00` cohort, including iterations subsequently closed. Calls and clocks/call are lane aggregates, not individually paired calls. All eight requested routines follow.

| Routine | Calls source / before / after | Source clocks/call | Native clocks/call before / after | Signed gap before | Signed gap after |
| --- | ---: | ---: | ---: | ---: | ---: |
| `$06:dc10` | 11,899 / 11,951 / 11,951 | 954.712 | 46.000 / 46.000 | 10,810,378 | 10,810,378 |
| `$09:b0b6` | 68,085 / 68,085 / 68,085 | 364.315 | 231.755 / 252.876 | 9,025,378 | 7,587,362 |
| `$1e:8b19` | 9,306 / 9,306 / 9,306 | 1,231.806 | 302.381 / 302.381 | 8,649,224 | 8,649,224 |
| `$06:84e2` | 45,200 / 45,200 / 45,200 | 560.855 | 431.116 / 431.116 | 5,864,200 | 5,864,200 |
| `$08:833c` | 2,551 / 2,551 / 2,551 | 2,595.067 | 550.841 / 550.841 | 5,214,820 | 5,214,820 |
| `$1d:c222` | 4,874 / 4,874 / 4,874 | 1,326.640 | 293.059 / 293.059 | 5,037,674 | 5,037,674 |
| `$06:d22f` | 1,268 / 0 / 0 | 3,607.391 | unscoped / unscoped | 4,574,172 | 4,574,172 |
| `$05:b5d3` | 4,399 / 4,399 / 4,399 | 1,552.964 | 554.872 / 554.872 | 4,390,606 | 4,390,606 |

**Summed exclusive gap closed: 1,438,016 master clocks.** `$09:B0B6` remains partially priced. Its call counts are unchanged, and every equal-call cohort row receiving added work stays at or below source exclusive clocks. Other scoped raw gaps are unchanged.

`$06:DC10` is skipped as an attribution artifact: source retains the `$DC10` frame while native drawing/shadow continuations use `$DC13/$DC66`, with differing call counts. `$06:D22F` is unscoped in native and includes absorbable dispatch plus drawing JMP tails; its raw gap cannot become a constant charge. The remaining unpriced paths are the drawing/actor bodies reached through the already-priced `$1E:8B19`, `$1D:C222`, `$05:B5D3` RTS dispatches, `$06:84E2` entry/timer/state dispatch, and `$08:833C` allocation/timer/indirect dispatch. Their dispatcher instructions are already covered; this batch leaves those body prices open. Concrete unpriced examples retained in `open-*.txt` are `$1E:9BC8` Zoro/Babasu selection and bodies under `$1E:8B19`, `$1D:E773` Vitreous-eye coordinate/AI work under `$1D:C222`, and `$05:967B` Zora/fireball selection and bodies under `$05:B5D3`. The retained native cohort selection diagnostic identifies these as frequently selected handlers; it does not assign the aggregate source gap to a particular handler. Other `$09:B0B6` garnish bodies beyond the seven selected paths also remain open.

### Route-wide clocks per call

Both fresh native ROM censuses reconcile exclusive routine sums to main instruction clocks on every one of 1,473,123 iterations.

| Routine | Native calls before / after | Native clocks/call before / after |
| --- | ---: | ---: |
| `$06:dc10` | 2,350,894 / 2,350,894 | 46.000 / 46.000 |
| `$09:b0b6` | 23,398,320 / 23,398,320 | 152.764 / 163.041 |
| `$1e:8b19` | 2,487,531 / 2,487,531 | 338.889 / 338.889 |
| `$06:84e2` | 22,612,952 / 22,612,952 | 324.869 / 324.869 |
| `$08:833c` | 384,393 / 384,393 | 568.539 / 568.539 |
| `$1d:c222` | 843,363 / 843,363 | 317.098 / 317.098 |
| `$06:d22f` | 0 / 0 | unscoped / unscoped |
| `$05:b5d3` | 1,245,791 / 1,245,791 | 568.909 / 568.909 |

**Full-route source/native reconciliation remains unfinished.** The retained source profile covers the frozen cohort; the earlier full-source attempt stopped around 201k iterations. This short group does not rerun it. Consequently full-route source clocks/call and route-wide source gaps are unavailable, and the native route table is not a source comparison. The executed-ROM fixtures establish the selected handler paths, not all remaining body paths or full-route semantic equivalence.

### Full CPU-only verification

Fresh main controls and candidate full gates PASS in cartridge-answer and live-SPC modes: 1,473,123 iterations each, unexplained zero, every runtime hold equal to P2. Missing/excess count lag NMIs; false positives count iterations. These periodically seeded logical checks do not establish continuous physical-field or exact A/V parity.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,985 / 4,980 | 0 / 0 | 0 / 0 | 34,644 / 34,649 | 39,561 / 39,561 |
| live | 5,684 / 5,679 | 592 / 592 | 132 / 132 | 34,510 / 34,515 | 39,429 / 39,429 |

The cartridge census closes 5 previously missing iterations and introduces 0 newly missing iterations. The fixed-cohort instruction gap reduction above is the main metric.

| Category | Source lag | Missing rom/live | Excess rom/live | FP rom/live | Exact lagged rom/live | Within one rom/live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0E/02` | 12,964 | 49 / 372 | 0 / 223 | 0 / 96 | 4,369 / 4,259 | 4,418 / 4,308 |
| `07/02` | 8,770 | 334 / 334 | 0 / 0 | 0 / 0 | 1,642 / 1,642 | 1,976 / 1,976 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 | 60 / 60 | 135 / 135 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 | 4,831 / 4,831 | 4,952 / 4,952 |
| `08/00` | 6,466 | 19 / 253 | 0 / 0 | 0 / 0 | 138 / 134 | 157 / 153 |
| `09/23` | 6,054 | 363 / 363 | 0 / 0 | 0 / 0 | 816 / 816 | 1,179 / 1,179 |
| `0F/01` | 5,432 | 74 / 74 | 0 / 0 | 0 / 0 | 4,805 / 4,805 | 4,879 / 4,879 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 | 242 / 242 | 251 / 251 |
| `07/07` | 4,060 | 263 / 263 | 0 / 0 | 0 / 0 | 2,656 / 2,656 | 2,919 / 2,919 |
| `07/00` | 3,679 | 1,668 / 1,668 | 0 / 105 | 0 / 19 | 1,292 / 1,292 | 2,960 / 2,960 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 | 249 / 249 | 251 / 251 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 | 2,647 / 2,647 | 2,718 / 2,718 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 | 78 / 78 | 115 / 115 |
| `10/01` | 2,672 | 1 / 2 | 0 / 5 | 0 / 0 | 2,668 / 2,666 | 2,669 / 2,668 |
| `08/02` | 2,576 | 2 / 90 | 0 / 0 | 0 / 0 | 155 / 152 | 157 / 153 |
| `07/15` | 2,333 | 72 / 72 | 0 / 0 | 0 / 0 | 1,663 / 1,663 | 1,735 / 1,735 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 | 1,855 / 1,855 | 1,855 / 1,855 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 | 70 / 70 | 90 / 90 |
| `09/00` | 1,253 | 1,135 / 1,135 | 0 / 0 | 0 / 0 | 57 / 57 | 1,192 / 1,192 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 | 56 / 56 | 56 / 56 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 | 580 / 580 | 603 / 603 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 | 10 / 10 | 17 / 17 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 | 927 / 927 | 928 / 928 |
| `08/01` | 706 | 0 / 17 | 0 / 53 | 0 / 0 | 157 / 153 | 157 / 153 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 | 5 / 5 | 11 / 11 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 | 250 / 250 | 251 / 251 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 | 389 / 389 | 391 / 391 |
| `0F/00` | 449 | 22 / 22 | 0 / 0 | 0 / 0 | 265 / 265 | 287 / 287 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 | 221 / 221 | 239 / 239 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 | 90 / 90 | 100 / 100 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 | 101 / 101 | 101 / 101 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 | 15 / 15 | 16 / 16 |
| `09/16` | 265 | 88 / 88 | 0 / 0 | 0 / 0 | 145 / 145 | 233 / 233 |
| `09/0D` | 247 | 89 / 89 | 0 / 0 | 0 / 0 | 134 / 134 | 223 / 223 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 | 81 / 81 | 81 / 81 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 | 14 / 14 | 16 / 16 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 | 32 / 32 | 36 / 36 |
| `10/00` | 157 | 0 / 1 | 0 / 81 | 0 / 0 | 157 / 154 | 157 / 155 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 2 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 | 7 / 7 | 9 / 9 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 | 23 / 23 | 28 / 28 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 | 83 / 83 | 119 / 119 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 | 4 / 4 | 11 / 11 |
| `0B/24` | 103 | 61 / 61 | 0 / 0 | 0 / 0 | 32 / 32 | 93 / 93 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 | 28 / 28 | 28 / 28 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 | 11 / 11 | 16 / 16 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 | 79 / 79 | 79 / 79 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 | 47 / 47 | 47 / 47 |
| `0B/17` | 68 | 32 / 32 | 0 / 0 | 0 / 0 | 33 / 33 | 65 / 65 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 | 43 / 43 | 66 / 66 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 | 11 / 11 | 12 / 12 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 | 62 / 62 | 62 / 62 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 | 3 / 3 | 4 / 4 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 4 / 4 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `0B/29` | 59 | 33 / 33 | 0 / 0 | 0 / 0 | 18 / 18 | 51 / 51 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 | 2 / 2 | 3 / 3 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 | 0 / 0 | 53 / 53 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 17 / 17 | 17 / 17 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 51 / 51 | 51 / 51 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 | 2 / 2 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 | 29 / 29 | 40 / 40 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/18` | 41 | 6 / 41 | 0 / 35 | 0 / 14 | 8 / 0 | 14 / 7 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 16 / 16 | 16 / 16 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 | 4 / 4 | 13 / 13 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 | 23 / 23 | 23 / 23 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 | 19 / 19 | 19 / 19 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 | 2 / 2 | 14 / 14 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 | 0 / 0 | 7 / 7 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 | 4 / 4 | 4 / 4 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 | 3 / 3 | 3 / 3 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 | 1 / 1 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 2 / 2 |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 | 0 / 0 | 1 / 1 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 0 / 90 | 0 / 3 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

| Seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |

### One instruction-derived group

`scripts/rom_block_costs.py` supplies the retained listings. `$07:D077..D103` now charges item/facing/spin coordinate setup, width changes, comparisons and the selected JSR/BRL opcodes. `$D2C5` RTS is included only for prefix paths that actually return there; ordinary probes receive no invented return. X8 indexed page crossings follow the actual offset. Normal item-0 setup costs 1,224 master clocks. `$D798`, `$D9D8` and the item-action child bodies remain separate. The retained early-main handoff bypasses the new scope and charge. Response work starting at `$D104` is still open.

`$07:D9D8` previously priced the lookup and indirect jump only. The selected effect bodies retain that real source call frame, so the new charge stays inside its existing native scope. Flat bodies now include RTS (42 clocks), ordinary collision (128), single absolute flag writes (136), four-shift flag writes (192), slopes (290/376), staircase variants (286/308/310/332), platform-dependent pit work (128/208), torch/hookshottable/door-flag writes (190/222/246) and aftermath (198). Other effects remain unpriced. The price uses decoded tile behavior and platform state, with ROM-address comments; it adds no scene table or fitted residual. Direct effect consumers and retained instruction continuations receive no second charge.

Both additions are lagless-only. No shared helper, sprite, ancilla, overworld, cycle-model or default/ROM-timing production pricing changed. New tests are in the existing `overworld_tests.rs`. The D077 fixture executes 252 item/facing/spin combinations and checks ordinary wrapper placement in both modes; the flat-effect fixture executes over 600 selected indoor/outdoor/platform paths from the real ROM tables. The existing lookup fixture now runs through the selected effect return. All 39 focused instruction-price tests pass.

### Frozen 2,825-row cohort

Calls are source / native before / native after. Each mean uses its own lane's call count; unscoped does not mean an observed zero-cost call. Source means cover the entire routine; D077 remains partially priced.

| Routine | Calls source / before / after | Source clocks/call | Native clocks/call before / after | Gap before | Gap after | Clocks closed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `$07:d077` | 3,249 / 0 / 3,249 | 1,980.903 | unscoped / 1,216.619 | 6,435,954 | 2,483,158 | 3,952,796 |
| `$07:807f` | 2,784 / 2,784 / 2,784 | 2,395.971 | 214.132 / 214.132 | 6,074,240 | 6,074,240 | 0 |
| `$07:e245` | 2,353 / 2,353 / 2,353 | 4,430.403 | 1,918.673 / 1,918.673 | 5,910,100 | 5,910,100 | 0 |
| `$07:b7c7` | 2,707 / 0 / 0 | 1,340.590 | unscoped / unscoped | 3,628,976 | 3,628,976 | 0 |
| `$07:d9d8` | 28,295 / 28,295 / 28,295 | 841.864 | 725.157 / 839.652 | 3,302,216 | 62,568 | 3,239,648 |
| `$07:b64f` | 2,677 / 0 / 0 | 1,164.770 | unscoped / unscoped | 3,118,090 | 3,118,090 | 0 |

Summed gap closed: **7,192,444 main instruction clocks**. The two exclusive-clock additions reconcile to the main-instruction increase on every cohort row. No changed routine exceeds its source exclusive clocks on any cohort row.

The other four routines were inspected in descending raw gap order and deferred under the short-batch throughput constraint. `$807F` JMP-dispatches to handlers that native accounting scopes separately; `$E245` similarly shares source ownership with `$E373/$E3E0/$E42A/$E4D3` tails that native accounting separates. Their raw deficits mix attribution and potentially unpriced work; this group does not claim either is a pure alias artifact. `$B7C7` and `$B64F` retain substantial unpriced atomic layer/direction/response branches (191 and 175 reachable instructions respectively), separate from their retained continuations. No residual charge is used to close these gaps.

### Whole-route native exclusive clocks

Both fresh native ROM profiles cover all 1,473,123 iterations and reconcile their exclusive sums to main clocks on every row. The retained source profile covers the frozen missing cohort only. Full-route empirical source clocks/call and source call counts are unavailable for these six variable-path routines; no cohort mean is substituted for that missing measurement.

| Routine | Route source clocks/call | Native calls before / after | Native clocks/call before / after | Route clocks added |
| --- | ---: | ---: | ---: | ---: |
| `$07:d077` | unavailable | 0 / 1,111,116 | unscoped / 1,220.747 | 1,356,391,498 |
| `$07:807f` | unavailable | 1,051,125 / 1,051,125 | 205.201 / 205.201 | 0 |
| `$07:e245` | unavailable | 976,645 / 976,645 | 1,853.645 / 1,853.645 | 0 |
| `$07:b7c7` | unavailable | 0 / 0 | unscoped / unscoped | 0 |
| `$07:d9d8` | unavailable | 6,713,923 / 6,713,923 | 767.095 / 854.759 | 588,565,582 |
| `$07:b64f` | unavailable | 0 / 0 | unscoped / unscoped | 0 |

### CPU-only verification

Both complete candidate census gates and both fresh main controls PASS: 1,473,123 iterations each, unexplained zero. Every candidate ROM runtime hold equals P2. Missing/excess count lag NMIs; FP counts iterations. These periodically seeded logical checks do not establish continuous physical-field or exact A/V parity.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,985 / 4,970 | 0 / 0 | 0 / 0 | 34,644 / 34,659 | 39,561 / 39,561 |
| live | 5,684 / 5,671 | 592 / 594 | 132 / 133 | 34,510 / 34,525 | 39,429 / 39,428 |

The live-SPC missing count improves, but excess rises by two and false-positive iterations rise by one net. Five FP iterations appear and four disappear; all changed FP/excess/worse-missing diagnostic rows are in the existing `song-end-poll` classification. On those rows cartridge-answer work and holds are unchanged, while live polling moves dialogue/upload work between iterations. `live-delta-diagnostics.json` retains the before/after rows and classifications. No instruction price was adjusted to fit this live regression.

Category cells are ROM / live, with main -> candidate in each lane.

| Category | Source lag | Missing ROM / live | Excess ROM / live | FP ROM / live | Exact lagged ROM / live | Within one ROM / live |
| --- | ---: | --- | --- | --- | --- | --- |
| `0E/02` | 12,964 | 49 -> 49 / 372 -> 374 | 0 -> 0 / 223 -> 223 | 0 -> 0 / 96 -> 96 | 4,369 -> 4,369 / 4,259 -> 4,259 | 4,418 -> 4,418 / 4,308 -> 4,307 |
| `07/02` | 8,770 | 334 -> 334 / 334 -> 334 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1,642 -> 1,642 / 1,642 -> 1,642 | 1,976 -> 1,976 / 1,976 -> 1,976 |
| `06/00` | 8,196 | 89 -> 89 / 89 -> 89 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 60 -> 60 / 60 -> 60 | 135 -> 135 / 135 -> 135 |
| `07/0E` | 7,979 | 121 -> 121 / 121 -> 121 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4,831 -> 4,831 / 4,831 -> 4,831 | 4,952 -> 4,952 / 4,952 -> 4,952 |
| `08/00` | 6,466 | 19 -> 19 / 253 -> 253 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 138 -> 138 / 134 -> 134 | 157 -> 157 / 153 -> 153 |
| `09/23` | 6,054 | 363 -> 363 / 363 -> 363 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 816 -> 816 / 816 -> 816 | 1,179 -> 1,179 / 1,179 -> 1,179 |
| `0F/01` | 5,432 | 79 -> 79 / 79 -> 79 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4,800 -> 4,800 / 4,800 -> 4,800 | 4,879 -> 4,879 / 4,879 -> 4,879 |
| `09/03` | 4,372 | 9 -> 9 / 9 -> 9 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 242 -> 242 / 242 -> 242 | 251 -> 251 / 251 -> 251 |
| `07/07` | 4,060 | 263 -> 261 / 263 -> 261 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2,656 -> 2,658 / 2,656 -> 2,658 | 2,919 -> 2,919 / 2,919 -> 2,919 |
| `07/00` | 3,679 | 1,668 -> 1,664 / 1,668 -> 1,664 | 0 -> 0 / 105 -> 107 | 0 -> 0 / 19 -> 20 | 1,292 -> 1,296 / 1,292 -> 1,296 | 2,960 -> 2,960 / 2,960 -> 2,960 |
| `09/01` | 3,479 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 249 -> 249 / 249 -> 249 | 251 -> 251 / 251 -> 251 |
| `07/0F` | 2,992 | 71 -> 71 / 71 -> 71 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2,647 -> 2,647 / 2,647 -> 2,647 | 2,718 -> 2,718 / 2,718 -> 2,718 |
| `0E/03` | 2,672 | 37 -> 37 / 37 -> 37 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 78 -> 78 / 78 -> 78 | 115 -> 115 / 115 -> 115 |
| `10/01` | 2,672 | 1 -> 1 / 2 -> 2 | 0 -> 0 / 5 -> 5 | 0 -> 0 / 0 -> 0 | 2,668 -> 2,668 / 2,666 -> 2,666 | 2,669 -> 2,669 / 2,668 -> 2,668 |
| `08/02` | 2,576 | 2 -> 2 / 90 -> 90 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 155 -> 155 / 152 -> 152 | 157 -> 157 / 153 -> 153 |
| `07/15` | 2,333 | 72 -> 72 / 72 -> 72 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1,663 -> 1,663 / 1,663 -> 1,663 | 1,735 -> 1,735 / 1,735 -> 1,735 |
| `19/02` | 2,001 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1,855 -> 1,855 / 1,855 -> 1,855 | 1,855 -> 1,855 / 1,855 -> 1,855 |
| `0E/0A` | 1,845 | 20 -> 20 / 20 -> 20 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 70 -> 70 / 70 -> 70 | 90 -> 90 / 90 -> 90 |
| `09/00` | 1,253 | 1,135 -> 1,126 / 1,135 -> 1,126 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 57 -> 66 / 57 -> 66 | 1,192 -> 1,192 / 1,192 -> 1,192 |
| `0E/07` | 1,065 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 56 -> 56 / 56 -> 56 | 56 -> 56 / 56 -> 56 |
| `07/06` | 984 | 23 -> 23 / 23 -> 23 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 580 -> 580 / 580 -> 580 | 603 -> 603 / 603 -> 603 |
| `11/00` | 973 | 7 -> 7 / 7 -> 7 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 10 -> 10 / 10 -> 10 | 17 -> 17 / 17 -> 17 |
| `07/01` | 928 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 927 -> 927 / 927 -> 927 | 928 -> 928 / 928 -> 928 |
| `08/01` | 706 | 0 -> 0 / 17 -> 17 | 0 -> 0 / 53 -> 53 | 0 -> 0 / 0 -> 0 | 157 -> 157 / 153 -> 153 | 157 -> 157 / 153 -> 153 |
| `05/00` | 639 | 6 -> 6 / 6 -> 6 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 5 -> 5 / 5 -> 5 | 11 -> 11 / 11 -> 11 |
| `09/04` | 508 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 250 -> 250 / 250 -> 250 | 251 -> 251 / 251 -> 251 |
| `12/03` | 472 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 389 -> 389 / 389 -> 389 | 391 -> 391 / 391 -> 391 |
| `0F/00` | 449 | 22 -> 22 / 22 -> 22 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 265 -> 265 / 265 -> 265 | 287 -> 287 / 287 -> 287 |
| `07/12` | 391 | 18 -> 18 / 18 -> 18 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 221 -> 221 / 221 -> 221 | 239 -> 239 / 239 -> 239 |
| `09/2E` | 363 | 10 -> 10 / 10 -> 10 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 90 -> 90 / 90 -> 90 | 100 -> 100 / 100 -> 100 |
| `07/16` | 303 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 101 -> 101 / 101 -> 101 | 101 -> 101 / 101 -> 101 |
| `09/11` | 274 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 15 -> 15 / 15 -> 15 | 16 -> 16 / 16 -> 16 |
| `09/16` | 265 | 88 -> 88 / 88 -> 88 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 145 -> 145 / 145 -> 145 | 233 -> 233 / 233 -> 233 |
| `09/0D` | 247 | 89 -> 89 / 89 -> 89 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 134 -> 134 / 134 -> 134 | 223 -> 223 / 223 -> 223 |
| `0E/0B` | 243 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 81 -> 81 / 81 -> 81 | 81 -> 81 / 81 -> 81 |
| `09/0F` | 242 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 14 -> 14 / 14 -> 14 | 16 -> 16 / 16 -> 16 |
| `01/00` | 235 | 48 -> 48 / 48 -> 48 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/2C` | 228 | 4 -> 4 / 4 -> 4 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 32 -> 32 / 32 -> 32 | 36 -> 36 / 36 -> 36 |
| `10/00` | 157 | 0 -> 0 / 1 -> 1 | 0 -> 0 / 81 -> 81 | 0 -> 0 / 0 -> 0 | 157 -> 157 / 154 -> 154 | 157 -> 157 / 155 -> 155 |
| `17/01` | 141 | 3 -> 3 / 3 -> 3 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 2 -> 2 / 2 -> 2 |
| `1B/02` | 139 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 7 -> 7 / 7 -> 7 | 9 -> 9 / 9 -> 9 |
| `09/20` | 138 | 5 -> 5 / 5 -> 5 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 23 -> 23 / 23 -> 23 | 28 -> 28 / 28 -> 28 |
| `0E/05` | 138 | 36 -> 36 / 36 -> 36 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 83 -> 83 / 83 -> 83 | 119 -> 119 / 119 -> 119 |
| `00/01` | 114 | 7 -> 7 / 7 -> 7 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 11 -> 11 / 11 -> 11 |
| `0B/24` | 103 | 61 -> 61 / 61 -> 61 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 32 -> 32 / 32 -> 32 | 93 -> 93 / 93 -> 93 |
| `09/21` | 84 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 28 -> 28 / 28 -> 28 | 28 -> 28 / 28 -> 28 |
| `09/0E` | 83 | 5 -> 5 / 5 -> 5 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 11 -> 11 / 11 -> 11 | 16 -> 16 / 16 -> 16 |
| `15/07` | 83 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 79 -> 79 / 79 -> 79 | 79 -> 79 / 79 -> 79 |
| `19/00` | 72 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 4 -> 4 / 4 -> 4 |
| `07/13` | 71 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 47 -> 47 / 47 -> 47 | 47 -> 47 / 47 -> 47 |
| `0B/17` | 68 | 32 -> 32 / 32 -> 32 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 33 -> 33 / 33 -> 33 | 65 -> 65 / 65 -> 65 |
| `0B/28` | 68 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `0B/2B` | 66 | 23 -> 23 / 23 -> 23 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 43 -> 43 / 43 -> 43 | 66 -> 66 / 66 -> 66 |
| `0B/19` | 64 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 4 -> 4 / 4 -> 4 |
| `15/03` | 64 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 11 -> 11 / 11 -> 11 | 12 -> 12 / 12 -> 12 |
| `00/05` | 62 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 62 -> 62 / 62 -> 62 | 62 -> 62 / 62 -> 62 |
| `0B/26` | 61 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 4 -> 4 / 4 -> 4 |
| `0B/1A` | 60 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 4 -> 4 / 4 -> 4 |
| `1A/12` | 60 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 3 -> 3 / 3 -> 3 |
| `0B/29` | 59 | 33 -> 33 / 33 -> 33 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 18 -> 18 / 18 -> 18 | 51 -> 51 / 51 -> 51 |
| `18/00` | 59 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 2 -> 2 / 2 -> 2 |
| `1A/00` | 59 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 2 -> 2 / 2 -> 2 |
| `1A/1A` | 59 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/1C` | 59 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 3 -> 3 / 3 -> 3 |
| `1A/04` | 58 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/08` | 58 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/0A` | 58 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/18` | 58 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/0E` | 57 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/10` | 56 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/06` | 55 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `07/03` | 53 | 53 -> 53 / 53 -> 53 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 53 -> 53 / 53 -> 53 |
| `1A/0C` | 53 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 2 -> 2 / 2 -> 2 |
| `11/07` | 51 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 17 -> 17 / 17 -> 17 | 17 -> 17 / 17 -> 17 |
| `13/05` | 51 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 51 -> 51 / 51 -> 51 | 51 -> 51 / 51 -> 51 |
| `1A/1E` | 51 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 2 -> 2 / 2 -> 2 |
| `0B/1C` | 48 | 11 -> 11 / 11 -> 11 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 29 -> 29 / 29 -> 29 | 40 -> 40 / 40 -> 40 |
| `1A/14` | 47 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 1 -> 1 / 1 -> 1 |
| `1A/02` | 46 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 1 -> 1 / 1 -> 1 |
| `1A/16` | 45 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `07/18` | 41 | 6 -> 6 / 41 -> 41 | 0 -> 0 / 35 -> 35 | 0 -> 0 / 14 -> 14 | 8 -> 8 / 0 -> 0 | 14 -> 14 / 7 -> 7 |
| `1A/20` | 41 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/12` | 32 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 16 -> 16 / 16 -> 16 | 16 -> 16 / 16 -> 16 |
| `12/08` | 32 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `0B/25` | 30 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `0B/00` | 29 | 9 -> 9 / 9 -> 9 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 13 -> 13 / 13 -> 13 |
| `0B/18` | 24 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `12/02` | 23 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 23 -> 23 / 23 -> 23 | 23 -> 23 / 23 -> 23 |
| `12/05` | 19 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 19 -> 19 / 19 -> 19 | 19 -> 19 / 19 -> 19 |
| `00/07` | 14 | 12 -> 12 / 12 -> 12 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 14 -> 14 / 14 -> 14 |
| `15/05` | 14 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 1 -> 1 / 1 -> 1 |
| `01/02` | 7 | 7 -> 7 / 7 -> 7 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 7 -> 7 / 7 -> 7 |
| `00/0A` | 6 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `12/09` | 4 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `13/04` | 3 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `00/02` | 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 1 -> 1 / 1 -> 1 |
| `00/06` | 2 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 |
| `04/01` | 2 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 |
| `00/00` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `03/01` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `07/0B` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `09/06` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `0E/01` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `19/01` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `00/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `00/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `00/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `01/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `01/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `01/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `01/05` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `03/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `03/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `03/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `03/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `04/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `04/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `04/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/05` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/08` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/0A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/0C` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/0D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/10` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/14` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/17` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/19` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/1A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/05` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/07` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/08` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/0A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/0C` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/10` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/13` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/14` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/15` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/1F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/22` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/2D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/2F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0B/08` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0B/1B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0B/27` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0B/2A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0E/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0E/08` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0E/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/06` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/07` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0C` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0E` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `13/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `13/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `13/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `13/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/06` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/0A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/0C` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 90 -> 90 | 0 -> 0 / 3 -> 3 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/05` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/07` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/0D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/0F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/11` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/13` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/15` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/17` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/19` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/1B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/1D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/1F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/21` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/22` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |

| Crystal/pendant seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 64 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 84 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 83 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase sweeps remain within one. All eight stable room, Link XY and whole-save outcomes match source. Fresh main and candidate outcomes are retained separately. The 162000–163000 rupee probe retains all 27 requests and every request/PCM onset gap against fresh main; PCM-gap median is 8.25374531835206 frames.

Probe classification rows below cover the 162000–163000 rupee window; ROM/live rows cover the complete route.

All phase counts remain within one; all eight stable room, Link XY and whole-save outcomes match source. The 162000–163000 rupee probe retains all 27 requests and every request/PCM onset gap; median PCM gap is 8.253745 frames. `final-probe-metrics.json` retains individual requests and gaps.

| Stage / capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| main / rom | `{"lag": 14}` | 10 |
| main / live | `{"lag": 14, "polygon": 14740, "song-end-poll": 639}` | 10 |
| main / probes | `{}` | 0 |
| final / rom | `{"lag": 14}` | 10 |
| final / live | `{"lag": 14, "polygon": 14739, "song-end-poll": 639}` | 10 |
| final / probes | `{}` | 0 |

The original 8,339 missing rows gain 1,727,044 main instruction clocks, with no negative row. HDMA upper-bound excursions remain zero; the same 63 masked-init leases remain declined.

Both ordinary candidate builds and the fresh main default build are warning-free. Candidate default matches every state/display/SRAM/PCM/input/context record and EOF of a freshly recorded main 200k ledger. All production changes are lagless-only; default/ROM-timing accounting is unchanged. Default lib-test compilation retains the known fixture warnings separately from those ordinary builds.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,531 | 1 | 3 |
| reduced | 2,066 | 0 | 2 |

The sole default library failure is the known spotlight caller-return fixture; reduced passes completely. The manifest verifies complete, disjoint named test unions. Both SNES library suites and Python routine-coverage tests pass. `git diff --check` passes.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Scratch drivers stayed in `/tmp`; supplied ledgers/cache and the other checkout's evidence remained read-only. This stops at one verified seven-handler group. Other body prices and full-route source reconciliation remain open.

| final / live | `{"lag": 14, "polygon": 14740, "song-end-poll": 640}` | 10 |
| final / probes | `{}` | 0 |

The original 8,339 missing rows gain 7,342,196 main instruction clocks, with no negative row and zero HDMA upper-bound excursions. The same 63 masked-init leases remain declined.

Both final ordinary builds and the fresh main default build are warning-free. Fresh final-source confirmations reproduce both verified binary hashes exactly (`final-build-confirmation.json`). Candidate default matches every state/display/SRAM/PCM/input/context record and EOF against the fresh main default 200k ledger. Default/ROM-timing charges and behavior remain unchanged over that check. Test-only fixture dead-code warnings remain separate from the ordinary builds.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,533 | 1 | 3 |
| reduced | 2,068 | 0 | 2 |

Default has only the known spotlight caller-return failure; reduced passes completely. The library manifest checks complete, disjoint named test unions and executable identities. Both SNES library suites and Python routine-coverage tests pass. `git diff --check` passes.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Supplied ledger/cache paths and the other checkout stayed read-only. Scratch drivers stayed in `/tmp`. This stops at one verified two-routine group; D077 response work, other tile effects, the four deferred routines and full-route source reconciliation remain open.

## Continuation on `codex/cost-pacing27b` (joint main `966d961b`)

Evidence is in `target/cost-pacing27b/`; scratch drivers are `/tmp/cost27b-*.py`. One verified partial group prices the short damage gates at `$06:F145/$F2B4` and the flat Vitreous-eye coordinate setup under `$1D:C222`. The production diff adds 40 lines. No bank-07 routine or shared ledger/model helper changed. The two shared sprite damage entry points changed minimally; their semantic branches and child implementations are unchanged.

### Fresh joint main baseline

Both controls were freshly built and executed from clean main `966d961b`, including 26a and 26b together and the masked elapsed-field owner. These joint results were measured rather than inferred by adding prior deltas. `baseline-provenance.json` binds the fresh default binary; it also recorded a new 200k default ledger before candidate verification.

| Fresh main control | Missing | Excess | FP | Exact lagged | Within one |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,980 | 0 | 0 | 34,649 | 39,561 |
| live | 5,679 | 592 | 132 | 34,515 | 39,429 |

### ROM-derived work

Fresh `scripts/rom_block_costs.py` listings cover all twelve requested entries plus `$1D:E773`. `source-price-instructions.json` independently sums the new blocks. Every new price is lagless-only and has its source address in a comment. There is no scene threshold, multiplier, table or price fitted to lag counts.

- `$06:F145..F15A`: 48 clocks for the disable gate, 102 for the stagger/hit test, and 80 for the layer test. Rejection takes its actual BNE (+6) and CLC/RTS (56). Disabled, stagger/hit-rejected, layer-rejected and successful-prefix costs are 110, 212, 292 and 230 respectively. Success stops at `$F15C`; shield/hit/damage response remains open. The partial charge has one `$F145` scope and does not invent frames for the fallthrough alternate entries.
- `$06:F2B4..F2D1`: entry/gates cost 64, optional layer loads 56, BNE 16 and the OAM-offset gate 56. Hit, layer and offset rejection cost 210, 260 and 316 including the real `$F321/$F3AF/$F3C7` tail. The accepted path charges three JSR opcodes plus BCC (154); overlap costs 346 through `$F2D1`, a miss costs 470 through return. Child hitbox bodies remain separately owned. Successful damage response beyond `$F2D3` remains open.
- `$1D:E773..E798`: 416 clocks of straight-line coordinate work, JSL draw (62) and JSR inactive (46), totaling 524 per eye call. Y is 0/2/4/6 and neither word table crosses a page. This is an RTS-dispatched handler inside `$1D:C222`, not a new nested source call. Its later animation, damage and AI instructions remain open.

The new existing-file CPU-only fixtures execute the ROM instructions, check caller-exclusive prices across slot extremes and every gate branch, and check zero default-mode charge. The damage fixture passes 768 executed-ROM cases, including 40 successful prefixes. The eye fixture covers ten slot/index cases, all four table indices and subtype wrap. Its initial default assertion incorrectly required existing drawing children to charge zero; only the new caller charge must be zero. That rejected fixture log is retained; no production price changed. These prefix fixtures deliberately stop at the first unpriced success PC; they do not claim complete damage/AI pricing.

### Frozen 2,825-row cohort

All twelve requested routines are shown. The fixed batch-22 `07/00` + `09/00` cohort includes iterations subsequently closed. Clocks/call are lane aggregates, not paired individual calls. Source and both native exclusive sums reconcile to main instructions on every selected row.

| Routine | Calls source / before / after | Source clocks/call | Native clocks/call before / after | Gap before | Gap after | Clocks closed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `$1e:8b19` | 9,306 / 9,306 / 9,306 | 1,231.806 | 302.381 / 302.381 | 8,649,224 | 8,649,224 | 0 |
| `$00:80b5` | 2,825 / 2,825 / 2,825 | 2,635.652 | 240.000 / 240.000 | 6,767,718 | 6,767,718 | 0 |
| `$06:84e2` | 45,200 / 45,200 / 45,200 | 560.855 | 431.116 / 431.116 | 5,864,200 | 5,864,200 | 0 |
| `$08:833c` | 2,551 / 2,551 / 2,551 | 2,595.067 | 550.841 / 550.841 | 5,214,820 | 5,214,820 | 0 |
| `$1d:c222` | 4,874 / 4,874 / 4,874 | 1,326.640 | 293.059 / 478.943 | 5,037,674 | 4,131,678 | 905,996 |
| `$06:d22f` | 1,268 / 0 / 0 | 3,607.391 | unscoped / unscoped | 4,574,172 | 4,574,172 | 0 |
| `$05:b5d3` | 4,399 / 4,399 / 4,399 | 1,552.964 | 554.872 / 554.872 | 4,390,606 | 4,390,606 | 0 |
| `$05:df6c` | 10,331 / 9,707 / 9,707 | 2,970.776 | 2,776.345 / 2,776.345 | 3,741,102 | 3,741,102 | 0 |
| `$1e:ff34` | 10,553 / 0 / 0 | 335.613 | unscoped / unscoped | 3,541,720 | 3,541,720 | 0 |
| `$06:f145` | 11,887 / 0 / 11,887 | 283.070 | unscoped / 212.485 | 3,364,856 | 839,048 | 2,525,808 |
| `$06:f2b4` | 9,197 / 0 / 9,197 | 356.943 | unscoped / 351.669 | 3,282,808 | 48,512 | 3,234,296 |
| `$08:f702` | 4,110 / 0 / 0 | 535.758 | unscoped / unscoped | 2,201,964 | 2,201,964 | 0 |

**Summed exclusive gap closed: 6,666,100 master clocks.** Whole-cohort main-instruction gap: 224,698,600 → 218,032,500. This is the primary result for the long tail. All equal-call cohort rows receiving added work stay at or below source exclusive clocks.

`$00:80B5` already prices all eight routing instructions (240); its remaining self gap is work in its JML target, so adding a routing constant would be wrong. `$1E:8B19`, `$06:84E2`, `$08:833C` and `$05:B5D3` likewise already price dispatch but leave selected actor/state/ancilla bodies open. `$1D:C222` is only partially reduced by the eye prefix above. `$06:D22F` remains an attribution/ownership skip: native has no matching absorbable-draw frame, and its JMP/drawing continuations cannot be priced as one flat residual. `$05:DF6C` already prices its entry, prep, per-entry loop and return; differing direct/R6/deferred entry counts make its raw aggregate gap unsuitable as a new flat price. `$1E:FF34` remains open: shared native axis movement has no bank-specific source scope, so the bank-1E alias must first be mapped to the actual callers. `$08:F702` remains open: the merged native safe-OAM writer combines the ROM coordinate helper with caller-owned character/flags/extended writes; charge only after mapping the actual JSR sites. These two are not declared fully priced or pure attribution artifacts. No bank-07 work belonging to the concurrent agent was changed.

### Route-wide clocks per call

Both fresh native ROM censuses profile all 1,473,123 iterations; every exclusive routine sum reconciles to its main-instruction counter.

| Routine | Native calls before / after | Native clocks/call before / after | Native exclusive clocks added |
| --- | ---: | ---: | ---: |
| `$1e:8b19` | 2,487,531 / 2,487,531 | 338.889 / 338.889 | 0 |
| `$00:80b5` | 1,473,123 / 1,473,123 | 487.114 / 487.114 | 0 |
| `$06:84e2` | 22,612,952 / 22,612,952 | 324.869 / 324.869 | 0 |
| `$08:833c` | 384,393 / 384,393 | 568.539 / 568.539 | 0 |
| `$1d:c222` | 843,363 / 843,363 | 317.098 / 343.823 | 22,538,812 |
| `$06:d22f` | 0 / 0 | unscoped / unscoped | 0 |
| `$05:b5d3` | 1,245,791 / 1,245,791 | 568.909 / 568.909 | 0 |
| `$05:df6c` | 2,367,479 / 2,367,479 | 2,514.531 / 2,514.531 | 0 |
| `$1e:ff34` | 0 / 0 | unscoped / unscoped | 0 |
| `$06:f145` | 0 / 1,618,230 | unscoped / 214.421 | 346,983,112 |
| `$06:f2b4` | 0 / 1,486,457 | unscoped / 334.048 | 496,548,376 |
| `$08:f702` | 0 / 0 | unscoped / unscoped | 0 |

**Full-route source/native comparison remains open.** The supplied complete source routine profile covers the frozen cohort; the prior full-source capture stopped near 201k. This short batch does not execute a new full source capture. Full-route source clocks/call and source gaps are therefore unavailable. The route-wide native additions above are not asserted to equal full-route source gap reductions.

### Full CPU-only verification

Both fresh main controls and both candidate full census gates PASS: 1,473,123 iterations each, unexplained zero; all runtime holds equal P2. Missing/excess count lag NMIs; FP counts iterations. These periodically seeded CPU checks do not establish continuous physical-field or exact A/V parity.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,980 / 4,964 | 0 / 0 | 0 / 0 | 34,649 / 34,665 | 39,561 / 39,561 |
| live | 5,679 / 5,668 | 592 / 597 | 132 / 134 | 34,515 / 34,531 | 39,429 / 39,428 |

The cartridge census closes 16 previously missing iterations and introduces 0 newly missing iterations.

Live-SPC adds 5 excess holds and 2 net false-positive iterations. `live-hold-deltas.json` retains all 54 changed hold/elapsed rows, all new/removed FP rows and both full song-poll logs. The first unmatched song-end read moves 94849 → 94848; the five-hold workload shifts 94850 → 94849 and subsequent VWF holds redistribute, adding 1 missing hold, 1 excess hold and 1 FP. The poll at 229030 moves to 229029; workloads shift 229031 → 229030 (5 holds) and 229036 → 229033 (4 holds), adding 4 missing, 4 excess and 1 FP. The crystal triplet near 380940/380950/381420 shifts by one iteration with zero net missing/excess/FP. These observed changes follow the live poll mismatch and change which work runs on a logical iteration; they are not authority to suppress a ROM instruction charge. Cartridge and live-SPC metrics remain separate.

Each category cell is main rom/live → final rom/live.

| Category | Source lag | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | --- | --- | --- | --- | --- |
| `0E/02` | 12,964 | 49/372 → 49/377 | 0/223 → 0/223 | 0/96 → 0/96 | 4,369/4,259 → 4,369/4,259 | 4,418/4,308 → 4,418/4,307 |
| `07/02` | 8,770 | 334/334 → 334/334 | 0/0 → 0/0 | 0/0 → 0/0 | 1,642/1,642 → 1,642/1,642 | 1,976/1,976 → 1,976/1,976 |
| `06/00` | 8,196 | 89/89 → 89/89 | 0/0 → 0/0 | 0/0 → 0/0 | 60/60 → 60/60 | 135/135 → 135/135 |
| `07/0E` | 7,979 | 121/121 → 121/121 | 0/0 → 0/0 | 0/0 → 0/0 | 4,831/4,831 → 4,831/4,831 | 4,952/4,952 → 4,952/4,952 |
| `08/00` | 6,466 | 19/253 → 19/253 | 0/0 → 0/0 | 0/0 → 0/0 | 138/134 → 138/134 | 157/153 → 157/153 |
| `09/23` | 6,054 | 363/363 → 363/363 | 0/0 → 0/0 | 0/0 → 0/0 | 816/816 → 816/816 | 1,179/1,179 → 1,179/1,179 |
| `0F/01` | 5,432 | 74/74 → 74/74 | 0/0 → 0/0 | 0/0 → 0/0 | 4,805/4,805 → 4,805/4,805 | 4,879/4,879 → 4,879/4,879 |
| `09/03` | 4,372 | 9/9 → 9/9 | 0/0 → 0/0 | 0/0 → 0/0 | 242/242 → 242/242 | 251/251 → 251/251 |
| `07/07` | 4,060 | 263/263 → 263/263 | 0/0 → 0/0 | 0/0 → 0/0 | 2,656/2,656 → 2,656/2,656 | 2,919/2,919 → 2,919/2,919 |
| `07/00` | 3,679 | 1,668/1,668 → 1,654/1,654 | 0/105 → 0/110 | 0/19 → 0/21 | 1,292/1,292 → 1,306/1,306 | 2,960/2,960 → 2,960/2,960 |
| `09/01` | 3,479 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 249/249 → 249/249 | 251/251 → 251/251 |
| `07/0F` | 2,992 | 71/71 → 71/71 | 0/0 → 0/0 | 0/0 → 0/0 | 2,647/2,647 → 2,647/2,647 | 2,718/2,718 → 2,718/2,718 |
| `0E/03` | 2,672 | 37/37 → 37/37 | 0/0 → 0/0 | 0/0 → 0/0 | 78/78 → 78/78 | 115/115 → 115/115 |
| `10/01` | 2,672 | 1/2 → 1/2 | 0/5 → 0/5 | 0/0 → 0/0 | 2,668/2,666 → 2,668/2,666 | 2,669/2,668 → 2,669/2,668 |
| `08/02` | 2,576 | 2/90 → 2/90 | 0/0 → 0/0 | 0/0 → 0/0 | 155/152 → 155/152 | 157/153 → 157/153 |
| `07/15` | 2,333 | 72/72 → 72/72 | 0/0 → 0/0 | 0/0 → 0/0 | 1,663/1,663 → 1,663/1,663 | 1,735/1,735 → 1,735/1,735 |
| `19/02` | 2,001 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1,855/1,855 → 1,855/1,855 | 1,855/1,855 → 1,855/1,855 |
| `0E/0A` | 1,845 | 20/20 → 20/20 | 0/0 → 0/0 | 0/0 → 0/0 | 70/70 → 70/70 | 90/90 → 90/90 |
| `09/00` | 1,253 | 1,135/1,135 → 1,133/1,133 | 0/0 → 0/0 | 0/0 → 0/0 | 57/57 → 59/59 | 1,192/1,192 → 1,192/1,192 |
| `0E/07` | 1,065 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 56/56 → 56/56 | 56/56 → 56/56 |
| `07/06` | 984 | 23/23 → 23/23 | 0/0 → 0/0 | 0/0 → 0/0 | 580/580 → 580/580 | 603/603 → 603/603 |
| `11/00` | 973 | 7/7 → 7/7 | 0/0 → 0/0 | 0/0 → 0/0 | 10/10 → 10/10 | 17/17 → 17/17 |
| `07/01` | 928 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 927/927 → 927/927 | 928/928 → 928/928 |
| `08/01` | 706 | 0/17 → 0/17 | 0/53 → 0/53 | 0/0 → 0/0 | 157/153 → 157/153 | 157/153 → 157/153 |
| `05/00` | 639 | 6/6 → 6/6 | 0/0 → 0/0 | 0/0 → 0/0 | 5/5 → 5/5 | 11/11 → 11/11 |
| `09/04` | 508 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 250/250 → 250/250 | 251/251 → 251/251 |
| `12/03` | 472 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 389/389 → 389/389 | 391/391 → 391/391 |
| `0F/00` | 449 | 22/22 → 22/22 | 0/0 → 0/0 | 0/0 → 0/0 | 265/265 → 265/265 | 287/287 → 287/287 |
| `07/12` | 391 | 18/18 → 18/18 | 0/0 → 0/0 | 0/0 → 0/0 | 221/221 → 221/221 | 239/239 → 239/239 |
| `09/2E` | 363 | 10/10 → 10/10 | 0/0 → 0/0 | 0/0 → 0/0 | 90/90 → 90/90 | 100/100 → 100/100 |
| `07/16` | 303 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 101/101 → 101/101 | 101/101 → 101/101 |
| `09/11` | 274 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 15/15 → 15/15 | 16/16 → 16/16 |
| `09/16` | 265 | 88/88 → 88/88 | 0/0 → 0/0 | 0/0 → 0/0 | 145/145 → 145/145 | 233/233 → 233/233 |
| `09/0D` | 247 | 89/89 → 89/89 | 0/0 → 0/0 | 0/0 → 0/0 | 134/134 → 134/134 | 223/223 → 223/223 |
| `0E/0B` | 243 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 81/81 → 81/81 | 81/81 → 81/81 |
| `09/0F` | 242 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 14/14 → 14/14 | 16/16 → 16/16 |
| `01/00` | 235 | 48/48 → 48/48 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/2C` | 228 | 4/4 → 4/4 | 0/0 → 0/0 | 0/0 → 0/0 | 32/32 → 32/32 | 36/36 → 36/36 |
| `10/00` | 157 | 0/1 → 0/1 | 0/81 → 0/81 | 0/0 → 0/0 | 157/154 → 157/154 | 157/155 → 157/155 |
| `17/01` | 141 | 3/3 → 3/3 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 2/2 → 2/2 |
| `1B/02` | 139 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 7/7 → 7/7 | 9/9 → 9/9 |
| `09/20` | 138 | 5/5 → 5/5 | 0/0 → 0/0 | 0/0 → 0/0 | 23/23 → 23/23 | 28/28 → 28/28 |
| `0E/05` | 138 | 36/36 → 36/36 | 0/0 → 0/0 | 0/0 → 0/0 | 83/83 → 83/83 | 119/119 → 119/119 |
| `00/01` | 114 | 7/7 → 7/7 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 11/11 → 11/11 |
| `0B/24` | 103 | 61/61 → 61/61 | 0/0 → 0/0 | 0/0 → 0/0 | 32/32 → 32/32 | 93/93 → 93/93 |
| `09/21` | 84 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 28/28 → 28/28 | 28/28 → 28/28 |
| `09/0E` | 83 | 5/5 → 5/5 | 0/0 → 0/0 | 0/0 → 0/0 | 11/11 → 11/11 | 16/16 → 16/16 |
| `15/07` | 83 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 79/79 → 79/79 | 79/79 → 79/79 |
| `19/00` | 72 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 4/4 → 4/4 |
| `07/13` | 71 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 47/47 → 47/47 | 47/47 → 47/47 |
| `0B/17` | 68 | 32/32 → 32/32 | 0/0 → 0/0 | 0/0 → 0/0 | 33/33 → 33/33 | 65/65 → 65/65 |
| `0B/28` | 68 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `0B/2B` | 66 | 23/23 → 23/23 | 0/0 → 0/0 | 0/0 → 0/0 | 43/43 → 43/43 | 66/66 → 66/66 |
| `0B/19` | 64 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 4/4 → 4/4 |
| `15/03` | 64 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 11/11 → 11/11 | 12/12 → 12/12 |
| `00/05` | 62 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 62/62 → 62/62 | 62/62 → 62/62 |
| `0B/26` | 61 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 4/4 → 4/4 |
| `0B/1A` | 60 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 4/4 → 4/4 |
| `1A/12` | 60 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 3/3 → 3/3 |
| `0B/29` | 59 | 33/33 → 33/33 | 0/0 → 0/0 | 0/0 → 0/0 | 18/18 → 18/18 | 51/51 → 51/51 |
| `18/00` | 59 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 2/2 → 2/2 |
| `1A/00` | 59 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 2/2 → 2/2 |
| `1A/1A` | 59 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/1C` | 59 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 3/3 → 3/3 |
| `1A/04` | 58 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/08` | 58 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/0A` | 58 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/18` | 58 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/0E` | 57 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/10` | 56 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/06` | 55 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `07/03` | 53 | 53/53 → 53/53 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 53/53 → 53/53 |
| `1A/0C` | 53 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 2/2 → 2/2 |
| `11/07` | 51 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 17/17 → 17/17 | 17/17 → 17/17 |
| `13/05` | 51 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 51/51 → 51/51 | 51/51 → 51/51 |
| `1A/1E` | 51 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 2/2 → 2/2 |
| `0B/1C` | 48 | 11/11 → 11/11 | 0/0 → 0/0 | 0/0 → 0/0 | 29/29 → 29/29 | 40/40 → 40/40 |
| `1A/14` | 47 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 1/1 → 1/1 |
| `1A/02` | 46 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 1/1 → 1/1 |
| `1A/16` | 45 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `07/18` | 41 | 6/41 → 6/41 | 0/35 → 0/35 | 0/14 → 0/14 | 8/0 → 8/0 | 14/7 → 14/7 |
| `1A/20` | 41 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/12` | 32 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 16/16 → 16/16 | 16/16 → 16/16 |
| `12/08` | 32 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `0B/25` | 30 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `0B/00` | 29 | 9/9 → 9/9 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 13/13 → 13/13 |
| `0B/18` | 24 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `12/02` | 23 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 23/23 → 23/23 | 23/23 → 23/23 |
| `12/05` | 19 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 19/19 → 19/19 | 19/19 → 19/19 |
| `00/07` | 14 | 12/12 → 12/12 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 14/14 → 14/14 |
| `15/05` | 14 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 1/1 → 1/1 |
| `01/02` | 7 | 7/7 → 7/7 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 7/7 → 7/7 |
| `00/0A` | 6 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `12/09` | 4 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `13/04` | 3 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `00/02` | 2 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 1/1 → 1/1 |
| `00/06` | 2 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 |
| `04/01` | 2 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 |
| `00/00` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `03/01` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `07/0B` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `09/06` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `0E/01` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `19/01` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `00/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `00/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `00/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `01/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `01/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `01/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `01/05` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `03/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `03/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `03/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `03/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `04/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `04/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `04/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/05` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/08` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/0A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/0C` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/0D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/10` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/14` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/17` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/19` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/1A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/05` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/07` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/08` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/0A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/0C` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/10` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/13` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/14` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/15` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/1F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/22` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/2D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/2F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0B/08` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0B/1B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0B/27` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0B/2A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0E/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0E/08` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0E/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/06` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/07` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0C` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0E` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `13/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `13/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `13/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `13/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/06` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/0A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/0C` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/04` | 0 | 0/0 → 0/0 | 0/90 → 0/90 | 0/3 → 0/3 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/05` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/07` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/0D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/0F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/11` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/13` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/15` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/17` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/19` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/1B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/1D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/1F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/21` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/22` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |

## Continuation on `codex/cost-pacing28a` (main `406d8402`)

### Fresh joint main baseline

Evidence: `target/cost-pacing28a/`; scratch drivers: `/tmp/cost28a-*.py`. The checkout began clean. The freshly built main default binary and its source identity are bound by `main-provenance.json`. These controls include both 27a and 26b plus the masked elapsed-field owner. Historical individual deltas are not added to estimate the joint result.

| Main census | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,965 | 0 | 0 | 34,664 | 39,561 |
| live | 5,666 | 594 | 133 | 34,530 | 39,428 |

### One instruction-derived group

Fresh `scripts/rom_block_costs.py` listings cover all eleven requested routines. `source-price-instructions.json` independently sums the instructions used by this group. Every added price is lagless-only, cites ROM addresses, and follows work actually executed. No lag residual, scene threshold, scene multiplier or scene table supplies a price.

* `$1D:9F64` Swamola: the complete atomic drawing entry now owns its head/history setup, the four actual segment passes, call opcodes and RTS. Positive/zero Y velocity costs 5,504 exclusive clocks; negative velocity costs 5,586. The two OAM-step branches and loop-closing branches supply the difference. Conversion and drawing child bodies retain separate ownership. Retained checkpoint helpers bypass this atomic entry and receive no duplicate full-body charge; their pricing remains open.
* `$09:C5FA` proximity row: zero delta costs 112 clocks. Active work charges coordinate setup, each executed pass through `$C622..C632`, and RTS. The ordinary 22-pass path costs 5,470 or 5,478 clocks for positive or negative scroll delta. `$C6F5` and its loading children keep their own scopes and prices.
* `$06:F5E0` action hitbox: dash, spin, inactive sword frames, held-item/position bypasses and ordinary facing/table work are charged separately. Offset sign branches use the actual wrapped byte result; indexed ROM reads add six clocks only on an actual page crossing. The shared hitbox-table helper itself is unchanged.

The new executed-ROM fixtures cover 432 hitbox operand combinations, 30 atomic Swamola combinations and 45 proximity-row combinations. They check exclusive clocks, action-hitbox outputs, zero added default-mode caller work, direction signs, slot extremes, and indexed table-page crossings. All 42 focused instruction-price tests pass. New tests are confined to the existing `sprite_tests.rs` and `sprite_main_draw_tests.rs`. The shared files `sprite/collision.rs` and `sprite/loading.rs` change only the requested `$06:F5E0` and `$09:C5FA` entries. Existing common child helpers and every routine owned by the other agent are unchanged.

### Frozen 2,825-row cohort

Calls are source / before / after. Means use each lane's own call count; unscoped is not a measured zero-cost call. Source evidence is the retained batch-22 cohort, including iterations subsequently closed.

| Routine | Calls source / before / after | Source clocks/call | Native clocks/call before / after | Gap before | Gap after | Clocks closed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `$07:807f` | 2,784 / 2,784 / 2,784 | 2,395.971 | 214.132 / 214.132 | 6,074,240 | 6,074,240 | 0 |
| `$07:e245` | 2,353 / 2,353 / 2,353 | 4,430.403 | 1,918.673 / 1,918.673 | 5,910,100 | 5,910,100 | 0 |
| `$07:b7c7` | 2,707 / 0 / 0 | 1,340.590 | unscoped / unscoped | 3,628,976 | 3,628,976 | 0 |
| `$07:b64f` | 2,677 / 0 / 0 | 1,164.770 | unscoped / unscoped | 3,118,090 | 3,118,090 | 0 |
| `$1d:9f64` | 506 / 0 / 506 | 5,516.640 | unscoped / 5,516.640 | 2,791,420 | 0 | 2,791,420 |
| `$06:dc64` | 2,762 / 0 / 0 | 1,007.812 | unscoped / unscoped | 2,783,576 | 2,783,576 | 0 |
| `$1e:fe78` | 9,362 / 0 / 0 | 280.392 | unscoped / unscoped | 2,625,028 | 2,625,028 | 0 |
| `$09:c5fa` | 533 / 0 / 533 | 4,569.295 | unscoped / 4,569.295 | 2,435,434 | 0 | 2,435,434 |
| `$1d:e95d` | 4,570 / 0 / 0 | 505.406 | unscoped / unscoped | 2,309,704 | 2,309,704 | 0 |
| `$06:f5e0` | 2,834 / 0 / 2,834 | 764.032 | unscoped / 764.032 | 2,165,266 | 0 | 2,165,266 |
| `$06:f121` | 10,665 / 0 / 0 | 190.000 | unscoped / unscoped | 2,026,350 | 2,026,350 | 0 |

**Summed exclusive gap closed: 7,392,120 main instruction clocks.** The three additions reconcile to the main-clock increase on every cohort row. Each priced routine has equal source/native calls and never exceeds source exclusive work on any cohort row.

The other eight routines were inspected and left unchanged:

* `$07:807F` already prices its entry through the JMP dispatch; native handlers have separate scopes. `$07:E245` similarly separates selection from `$E373/$E3E0/$E42A/$E4D3` velocity tails. Their raw gaps mix attribution and remaining body work, and are not declared pure alias artifacts. Existing executed-ROM entry/selection tests pass.
* `$07:B7C7` and `$07:B64F` retain the deferred layer/direction/response branches (191 and 175 reachable instructions). These remain real pricing work, deferred under the short-group scope.
* `$06:DC64` is an entry-attribution artifact: the default long entry scopes its LDA #10 at `$DC64`, while shared shadow instructions already execute and charge under `$DC66`. `$1E:FE78` is likewise an alias of the already-priced inactive predicate under `$06:D9EC`; bank-5 and bank-1A entries have explicit wrappers. Their bank-specific raw gaps are not charged again.
* `$1D:E95D` movement is still unpriced in the shared native Y-movement helper. Bank-specific caller ownership must be established before assigning every shared helper invocation to bank 1D. This is deferred, not called an alias closure.
* `$06:F121` is a real 190-clock far bank wrapper. Native calls collapse into the shared `$F145` entry, which the other agent owns. Selecting the actual far-call sites needs separate work; the shared routine and its callers were left untouched in this group.

### Route-wide exclusive clocks

Both native ROM profiles cover all 1,473,123 iterations and reconcile every iteration's routine exclusive sum to main instruction clocks. The available empirical source profile covers the frozen cohort only. Full-route empirical source call counts and source gaps remain unavailable for all eleven routines. The flat `$06:F121` wrapper costs 190 exclusive clocks for every call, directly from its six instructions; that constant is marked as ROM-derived, not a measured route mean. The other ten route source means are unavailable. No cohort mean substitutes for a full source-route measurement.

| Routine | Route source clocks/call | Native calls before / after | Native clocks/call before / after | Route clocks added |
| --- | ---: | ---: | ---: | ---: |
| `$07:807f` | unavailable | 1,051,125 / 1,051,125 | 205.201 / 205.201 | 0 |
| `$07:e245` | unavailable | 976,645 / 976,645 | 1,853.645 / 1,853.645 | 0 |
| `$07:b7c7` | unavailable | 0 / 0 | unscoped / unscoped | 0 |
| `$07:b64f` | unavailable | 0 / 0 | unscoped / unscoped | 0 |
| `$1d:9f64` | unavailable | 0 / 16,431 | unscoped / 5,515.129 | 90,619,084 |
| `$06:dc64` | unavailable | 2,111 / 2,111 | 1,086.207 / 1,086.207 | 0 |
| `$1e:fe78` | unavailable | 0 / 0 | unscoped / unscoped | 0 |
| `$09:c5fa` | unavailable | 0 / 230,536 | unscoped / 1,459.206 | 336,399,400 |
| `$1d:e95d` | unavailable | 0 / 0 | unscoped / unscoped | 0 |
| `$06:f5e0` | unavailable | 0 / 225,615 | unscoped / 784.786 | 177,059,432 |
| `$06:f121` | 190.000 (ROM constant) | 0 / 0 | unscoped / unscoped | 0 |

### CPU-only verification

Both complete candidate census gates and both fresh main controls PASS: 1,473,123 iterations each, unexplained zero. Every candidate ROM runtime hold equals P2. Missing/excess count lag NMIs; FP counts iterations. Periodically seeded logical checks do not establish continuous physical-field or exact A/V parity.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,965 / 4,950 | 0 / 0 | 0 / 0 | 34,664 / 34,679 | 39,561 / 39,561 |
| live | 5,666 / 5,646 | 594 / 589 | 133 / 131 | 34,530 / 34,545 | 39,428 / 39,429 |

Live diagnostics retain eight newly false-positive iterations and ten removed ones (net -2), plus ten rows with increased excess and zero rows with increased missing. The twenty unique diagnostic rows are classified as thirteen `song-end-poll` and seven `polygon` rows. `live-delta-diagnostics.json` retains their before/after ROM and live records. No instruction price was adjusted to fit these changes.

Category cells are ROM / live, with main -> candidate in each lane.

| Category | Source lag | Missing ROM / live | Excess ROM / live | FP ROM / live | Exact lagged ROM / live | Within one ROM / live |
| --- | ---: | --- | --- | --- | --- | --- |
| `0E/02` | 12,964 | 49 -> 49 / 374 -> 369 | 0 -> 0 / 223 -> 223 | 0 -> 0 / 96 -> 96 | 4,369 -> 4,369 / 4,259 -> 4,259 | 4,418 -> 4,418 / 4,307 -> 4,308 |
| `07/02` | 8,770 | 334 -> 334 / 334 -> 334 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1,642 -> 1,642 / 1,642 -> 1,642 | 1,976 -> 1,976 / 1,976 -> 1,976 |
| `06/00` | 8,196 | 89 -> 89 / 89 -> 89 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 60 -> 60 / 60 -> 60 | 135 -> 135 / 135 -> 135 |
| `07/0E` | 7,979 | 121 -> 121 / 121 -> 121 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4,831 -> 4,831 / 4,831 -> 4,831 | 4,952 -> 4,952 / 4,952 -> 4,952 |
| `08/00` | 6,466 | 19 -> 19 / 253 -> 253 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 138 -> 138 / 134 -> 134 | 157 -> 157 / 153 -> 153 |
| `09/23` | 6,054 | 363 -> 362 / 363 -> 362 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 816 -> 817 / 816 -> 817 | 1,179 -> 1,179 / 1,179 -> 1,179 |
| `0F/01` | 5,432 | 74 -> 74 / 74 -> 74 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4,805 -> 4,805 / 4,805 -> 4,805 | 4,879 -> 4,879 / 4,879 -> 4,879 |
| `09/03` | 4,372 | 9 -> 9 / 9 -> 9 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 242 -> 242 / 242 -> 242 | 251 -> 251 / 251 -> 251 |
| `07/07` | 4,060 | 261 -> 261 / 261 -> 261 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2,658 -> 2,658 / 2,658 -> 2,658 | 2,919 -> 2,919 / 2,919 -> 2,919 |
| `07/00` | 3,679 | 1,664 -> 1,659 / 1,664 -> 1,659 | 0 -> 0 / 107 -> 102 | 0 -> 0 / 20 -> 18 | 1,296 -> 1,301 / 1,296 -> 1,301 | 2,960 -> 2,960 / 2,960 -> 2,960 |
| `09/01` | 3,479 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 249 -> 249 / 249 -> 249 | 251 -> 251 / 251 -> 251 |
| `07/0F` | 2,992 | 71 -> 71 / 71 -> 71 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2,647 -> 2,647 / 2,647 -> 2,647 | 2,718 -> 2,718 / 2,718 -> 2,718 |
| `0E/03` | 2,672 | 37 -> 37 / 37 -> 37 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 78 -> 78 / 78 -> 78 | 115 -> 115 / 115 -> 115 |
| `10/01` | 2,672 | 1 -> 1 / 2 -> 2 | 0 -> 0 / 5 -> 5 | 0 -> 0 / 0 -> 0 | 2,668 -> 2,668 / 2,666 -> 2,666 | 2,669 -> 2,669 / 2,668 -> 2,668 |
| `08/02` | 2,576 | 2 -> 2 / 90 -> 90 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 155 -> 155 / 152 -> 152 | 157 -> 157 / 153 -> 153 |
| `07/15` | 2,333 | 72 -> 72 / 72 -> 72 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1,663 -> 1,663 / 1,663 -> 1,663 | 1,735 -> 1,735 / 1,735 -> 1,735 |
| `19/02` | 2,001 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1,855 -> 1,855 / 1,855 -> 1,855 | 1,855 -> 1,855 / 1,855 -> 1,855 |
| `0E/0A` | 1,845 | 20 -> 20 / 20 -> 20 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 70 -> 70 / 70 -> 70 | 90 -> 90 / 90 -> 90 |
| `09/00` | 1,253 | 1,126 -> 1,124 / 1,126 -> 1,124 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 66 -> 68 / 66 -> 68 | 1,192 -> 1,192 / 1,192 -> 1,192 |
| `0E/07` | 1,065 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 56 -> 56 / 56 -> 56 | 56 -> 56 / 56 -> 56 |
| `07/06` | 984 | 23 -> 23 / 23 -> 23 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 580 -> 580 / 580 -> 580 | 603 -> 603 / 603 -> 603 |
| `11/00` | 973 | 7 -> 7 / 7 -> 7 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 10 -> 10 / 10 -> 10 | 17 -> 17 / 17 -> 17 |
| `07/01` | 928 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 927 -> 927 / 927 -> 927 | 928 -> 928 / 928 -> 928 |
| `08/01` | 706 | 0 -> 0 / 17 -> 17 | 0 -> 0 / 53 -> 53 | 0 -> 0 / 0 -> 0 | 157 -> 157 / 153 -> 153 | 157 -> 157 / 153 -> 153 |
| `05/00` | 639 | 6 -> 6 / 6 -> 6 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 5 -> 5 / 5 -> 5 | 11 -> 11 / 11 -> 11 |
| `09/04` | 508 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 250 -> 250 / 250 -> 250 | 251 -> 251 / 251 -> 251 |
| `12/03` | 472 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 389 -> 389 / 389 -> 389 | 391 -> 391 / 391 -> 391 |
| `0F/00` | 449 | 22 -> 22 / 22 -> 22 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 265 -> 265 / 265 -> 265 | 287 -> 287 / 287 -> 287 |
| `07/12` | 391 | 18 -> 18 / 18 -> 18 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 221 -> 221 / 221 -> 221 | 239 -> 239 / 239 -> 239 |
| `09/2E` | 363 | 10 -> 10 / 10 -> 10 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 90 -> 90 / 90 -> 90 | 100 -> 100 / 100 -> 100 |
| `07/16` | 303 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 101 -> 101 / 101 -> 101 | 101 -> 101 / 101 -> 101 |
| `09/11` | 274 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 15 -> 15 / 15 -> 15 | 16 -> 16 / 16 -> 16 |
| `09/16` | 265 | 88 -> 86 / 88 -> 86 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 145 -> 147 / 145 -> 147 | 233 -> 233 / 233 -> 233 |
| `09/0D` | 247 | 89 -> 87 / 89 -> 87 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 134 -> 136 / 134 -> 136 | 223 -> 223 / 223 -> 223 |
| `0E/0B` | 243 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 81 -> 81 / 81 -> 81 | 81 -> 81 / 81 -> 81 |
| `09/0F` | 242 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 14 -> 14 / 14 -> 14 | 16 -> 16 / 16 -> 16 |
| `01/00` | 235 | 48 -> 48 / 48 -> 48 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/2C` | 228 | 4 -> 4 / 4 -> 4 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 32 -> 32 / 32 -> 32 | 36 -> 36 / 36 -> 36 |
| `10/00` | 157 | 0 -> 0 / 1 -> 1 | 0 -> 0 / 81 -> 81 | 0 -> 0 / 0 -> 0 | 157 -> 157 / 154 -> 154 | 157 -> 157 / 155 -> 155 |
| `17/01` | 141 | 3 -> 3 / 3 -> 3 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 2 -> 2 / 2 -> 2 |
| `1B/02` | 139 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 7 -> 7 / 7 -> 7 | 9 -> 9 / 9 -> 9 |
| `09/20` | 138 | 5 -> 5 / 5 -> 5 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 23 -> 23 / 23 -> 23 | 28 -> 28 / 28 -> 28 |
| `0E/05` | 138 | 36 -> 36 / 36 -> 36 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 83 -> 83 / 83 -> 83 | 119 -> 119 / 119 -> 119 |
| `00/01` | 114 | 7 -> 7 / 7 -> 7 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 11 -> 11 / 11 -> 11 |
| `0B/24` | 103 | 61 -> 60 / 61 -> 60 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 32 -> 33 / 32 -> 33 | 93 -> 93 / 93 -> 93 |
| `09/21` | 84 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 28 -> 28 / 28 -> 28 | 28 -> 28 / 28 -> 28 |
| `09/0E` | 83 | 5 -> 5 / 5 -> 5 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 11 -> 11 / 11 -> 11 | 16 -> 16 / 16 -> 16 |
| `15/07` | 83 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 79 -> 79 / 79 -> 79 | 79 -> 79 / 79 -> 79 |
| `19/00` | 72 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 4 -> 4 / 4 -> 4 |
| `07/13` | 71 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 47 -> 47 / 47 -> 47 | 47 -> 47 / 47 -> 47 |
| `0B/17` | 68 | 32 -> 31 / 32 -> 31 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 33 -> 34 / 33 -> 34 | 65 -> 65 / 65 -> 65 |
| `0B/28` | 68 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `0B/2B` | 66 | 23 -> 23 / 23 -> 23 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 43 -> 43 / 43 -> 43 | 66 -> 66 / 66 -> 66 |
| `0B/19` | 64 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 4 -> 4 / 4 -> 4 |
| `15/03` | 64 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 11 -> 11 / 11 -> 11 | 12 -> 12 / 12 -> 12 |
| `00/05` | 62 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 62 -> 62 / 62 -> 62 | 62 -> 62 / 62 -> 62 |
| `0B/26` | 61 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 4 -> 4 / 4 -> 4 |
| `0B/1A` | 60 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 4 -> 4 / 4 -> 4 |
| `1A/12` | 60 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 3 -> 3 / 3 -> 3 |
| `0B/29` | 59 | 33 -> 32 / 33 -> 32 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 18 -> 19 / 18 -> 19 | 51 -> 51 / 51 -> 51 |
| `18/00` | 59 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 2 -> 2 / 2 -> 2 |
| `1A/00` | 59 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 2 -> 2 / 2 -> 2 |
| `1A/1A` | 59 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/1C` | 59 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 3 -> 3 / 3 -> 3 |
| `1A/04` | 58 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/08` | 58 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/0A` | 58 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/18` | 58 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/0E` | 57 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/10` | 56 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `1A/06` | 55 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `07/03` | 53 | 53 -> 53 / 53 -> 53 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 53 -> 53 / 53 -> 53 |
| `1A/0C` | 53 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 2 -> 2 / 2 -> 2 |
| `11/07` | 51 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 17 -> 17 / 17 -> 17 | 17 -> 17 / 17 -> 17 |
| `13/05` | 51 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 51 -> 51 / 51 -> 51 | 51 -> 51 / 51 -> 51 |
| `1A/1E` | 51 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 2 -> 2 / 2 -> 2 |
| `0B/1C` | 48 | 11 -> 11 / 11 -> 11 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 29 -> 29 / 29 -> 29 | 40 -> 40 / 40 -> 40 |
| `1A/14` | 47 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 1 -> 1 / 1 -> 1 |
| `1A/02` | 46 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 1 -> 1 / 1 -> 1 |
| `1A/16` | 45 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `07/18` | 41 | 6 -> 6 / 41 -> 41 | 0 -> 0 / 35 -> 35 | 0 -> 0 / 14 -> 14 | 8 -> 8 / 0 -> 0 | 14 -> 14 / 7 -> 7 |
| `1A/20` | 41 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/12` | 32 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 16 -> 16 / 16 -> 16 | 16 -> 16 / 16 -> 16 |
| `12/08` | 32 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `0B/25` | 30 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `0B/00` | 29 | 9 -> 9 / 9 -> 9 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 13 -> 13 / 13 -> 13 |
| `0B/18` | 24 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `12/02` | 23 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 23 -> 23 / 23 -> 23 | 23 -> 23 / 23 -> 23 |
| `12/05` | 19 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 19 -> 19 / 19 -> 19 | 19 -> 19 / 19 -> 19 |
| `00/07` | 14 | 12 -> 12 / 12 -> 12 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 | 14 -> 14 / 14 -> 14 |
| `15/05` | 14 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 1 -> 1 / 1 -> 1 |
| `01/02` | 7 | 7 -> 7 / 7 -> 7 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 7 -> 7 / 7 -> 7 |
| `00/0A` | 6 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `12/09` | 4 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 4 -> 4 / 4 -> 4 | 4 -> 4 / 4 -> 4 |
| `13/04` | 3 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 3 -> 3 / 3 -> 3 | 3 -> 3 / 3 -> 3 |
| `00/02` | 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 | 1 -> 1 / 1 -> 1 |
| `00/06` | 2 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 |
| `04/01` | 2 | 2 -> 2 / 2 -> 2 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 2 -> 2 / 2 -> 2 |
| `00/00` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `03/01` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `07/0B` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `09/06` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `0E/01` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `19/01` | 1 | 1 -> 1 / 1 -> 1 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 1 -> 1 / 1 -> 1 |
| `00/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `00/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `00/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `01/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `01/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `01/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `01/05` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `03/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `03/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `03/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `03/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `04/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `04/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `04/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/05` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/08` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/0A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/0C` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/0D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/10` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/14` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/17` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/19` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `07/1A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/05` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/07` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/08` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/0A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/0C` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/10` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/13` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/14` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/15` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/1F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/22` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/2D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `09/2F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0B/08` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0B/1B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0B/27` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0B/2A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0E/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0E/08` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `0E/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/06` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/07` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0C` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0E` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `12/0F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `13/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `13/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `13/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `13/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/06` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/0A` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `15/0C` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/00` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/02` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `16/04` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 90 -> 90 | 0 -> 0 / 3 -> 3 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/01` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/03` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/05` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/07` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/09` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/0B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/0D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/0F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/11` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/13` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/15` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/17` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/19` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/1B` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/1D` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/1F` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/21` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |
| `1A/22` | 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 | 0 -> 0 / 0 -> 0 |

| Crystal/pendant seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 64 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 84 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 83 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase sweeps remain within one. All eight stable room, Link XY and whole-save outcomes match source in fresh main and final probes. The 162000–163000 rupee probe preserves all 27 requests and every request/PCM onset gap against fresh main; PCM-gap median is 8.25374531835206 frames.

Probe classification rows below cover only the 162000–163000 rupee window; rom/live rows cover the full route.

All phase counts remain within one, and all eight stable room, Link XY and whole-save outcomes match source. All 27 rupee requests and every request/PCM onset gap in 162000–163000 match fresh main; PCM-gap median is 8.253745 frames. Individual results are retained in `final-probe-metrics.json` and `final-stable-outcomes.json`.

| Stage / capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| main / rom | `{"lag": 14}` | 10 |
| main / live | `{"lag": 14, "polygon": 14739, "song-end-poll": 639}` | 10 |
| main / probes | `{}` | 0 |
| final / rom | `{"lag": 14}` | 10 |
| final / live | `{"lag": 14, "polygon": 14740, "song-end-poll": 641}` | 10 |
| final / probes | `{}` | 0 |

The original 8,339 missing rows gain 6,881,252 main instruction clocks, with no negative row. HDMA upper-bound excursions remain zero; the same 63 masked-init leases remain declined.

Both ordinary candidate builds and the fresh main default build are warning-free. Candidate default matches every state/display/SRAM/PCM/input/context record and EOF of fresh main's 200k ledger. Default/ROM-timing accounting is unchanged. Known lib-test fixture warnings remain separate from the ordinary builds.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,534 | 1 | 3 |
| reduced | 2,069 | 0 | 2 |

The sole default library failure is the known spotlight caller-return fixture; reduced passes entirely. The manifest checks complete, disjoint named test unions. Both SNES library suites, Python routine-coverage tests and `git diff --check` pass.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Scratch scripts stayed in `/tmp`; supplied ledgers/cache and the other checkout remained read-only. This stops at one verified group; remaining body prices and full-route source reconciliation are open.

| main / live | `{"lag": 14, "polygon": 14740, "song-end-poll": 640}` | 10 |
| main / probes | `{}` | 0 |
| final / rom | `{"lag": 14}` | 10 |
| final / live | `{"lag": 14, "polygon": 14740, "song-end-poll": 638}` | 10 |
| final / probes | `{}` | 0 |

The original 8,339 missing rows gain 7,709,962 main instruction clocks, with no negative row and zero HDMA upper-bound excursions. The same 63 masked-init leases remain declined.

Both final ordinary builds and the fresh main default build are warning-free. Final source confirmations reproduce both verified binary hashes exactly (`final-build-confirmation.json`). Candidate default matches every state/display/SRAM/PCM/input/context record and EOF against the freshly recorded main default 200k ledger. Default/ROM-timing charges remain unchanged. Test-only fixture warnings are separate from ordinary builds.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,536 | 1 | 3 |
| reduced | 2,071 | 0 | 2 |

Default retains only the known spotlight caller-return failure; reduced passes completely. The library manifest verifies complete, disjoint test unions and executable identities. Both SNES library suites and Python routine-coverage tests pass. `git diff --check` passes.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Supplied ledgers/cache and the other checkout stayed read-only. Scratch drivers stayed in `/tmp`. This stops at one verified three-routine group. The eight other routines, retained Swamola continuations and full-route empirical source reconciliation remain open.

## Continuation on `codex/cost-pacing28b` (joint main `22871f5f`)

Evidence is in `target/cost-pacing28b/`; scratch drivers are `/tmp/cost28b-*.py`. One verified group prices the Zoro caller body and its inlined garnish search under `$1E:8B19`. The production diff adds 40 lines. No bank-07 routine or shared helper changed; all other assigned body gaps remain open.

### Fresh joint main baseline

Both controls were freshly built and executed from clean main `22871f5f`, including 27a and 27b together and the masked elapsed-field owner. This is the first joint 27a/27b census; these results were measured rather than inferred by adding prior deltas. `baseline-provenance.json` binds the fresh default binary; it also recorded a new 200k default ledger before candidate verification.

| Fresh main control | Missing | Excess | FP | Exact lagged | Within one |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,950 | 0 | 0 | 34,679 | 39,561 |
| live | 5,647 | 590 | 132 | 34,545 | 39,429 |

### ROM-derived work

Fresh `scripts/rom_block_costs.py` listings cover all ten assigned entries and `$1E:9BC8`. `source-price-instructions.json` independently sums each added block. All added prices are lagless-only, with source addresses in comments.

- `$1E:9BC8..9BCD`: 54 clocks for the taken Zoro branch, 72 for the Babasu JMP path. Both are tail bodies in `$1E:8B19`; no new routine frame is invented.
- `$1E:9BD0..9C1C`: price the C gate and optional first-call check, draw/inactive call opcodes, active animation/movement call opcodes, actual delay/collision branches and subtype test. Child bodies retain their existing scopes and remaining child gaps are not included in this caller price.
- `$1E:9C1E..9C6A`: search setup 52; each occupied continuing pass 92; final occupied pass 86 plus PLX/RTS 70; a free-slot read/BEQ 62 followed by trail writes and PLX/RTS 674. The allocator result determines the actual number of slots examined. No shared allocation helper changed.

The CPU-only fixture executes ROM instructions across slot extremes, subtype wrap, first-call rejection/acceptance, delay/collision gates and free garnish slots 29/14/0 or exhaustion. It checks caller-exclusive clocks, state and garnish writes, plus zero added default-mode caller charge. All 128 executed cases pass. The initial fixture omitted the source current-object index; a first correction used $00FA rather than $0FA0. Both rejected logs are retained. The final fixture binds $0FA0 before source children reload X; production prices never changed. No price was fitted to held counts.

### Frozen 2,825-row cohort

The fixed batch-22 `07/00` + `09/00` cohort includes iterations subsequently closed. Clocks/call are lane aggregates, not paired calls. Source and native exclusive sums reconcile to main instructions on every selected row.

| Routine | Calls source / before / after | Source clocks/call | Native clocks/call before / after | Gap before | Gap after | Clocks closed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `$1e:8b19` | 9,306 / 9,306 / 9,306 | 1,231.806 | 302.381 / 314.234 | 8,649,224 | 8,538,920 | 110,304 |
| `$00:80b5` | 2,825 / 2,825 / 2,825 | 2,635.652 | 240.000 / 240.000 | 6,767,718 | 6,767,718 | 0 |
| `$06:84e2` | 45,200 / 45,200 / 45,200 | 560.855 | 431.116 / 431.116 | 5,864,200 | 5,864,200 | 0 |
| `$08:833c` | 2,551 / 2,551 / 2,551 | 2,595.067 | 550.841 / 550.841 | 5,214,820 | 5,214,820 | 0 |
| `$1d:c222` | 4,874 / 4,874 / 4,874 | 1,326.640 | 478.943 / 478.943 | 4,131,678 | 4,131,678 | 0 |
| `$06:d22f` | 1,268 / 0 / 0 | 3,607.391 | unscoped / unscoped | 4,574,172 | 4,574,172 | 0 |
| `$05:b5d3` | 4,399 / 4,399 / 4,399 | 1,552.964 | 554.872 / 554.872 | 4,390,606 | 4,390,606 | 0 |
| `$05:df6c` | 10,331 / 9,707 / 9,707 | 2,970.776 | 2,776.345 / 2,776.345 | 3,741,102 | 3,741,102 | 0 |
| `$1e:ff34` | 10,553 / 0 / 0 | 335.613 | unscoped / unscoped | 3,541,720 | 3,541,720 | 0 |
| `$08:f702` | 4,110 / 0 / 0 | 535.758 | unscoped / unscoped | 2,201,964 | 2,201,964 | 0 |

**Summed exclusive gap closed: 110,304 master clocks.** Whole-cohort main-instruction gap: 210,840,056 → 210,729,752. This is the primary result for the long tail. All equal-call cohort rows receiving added work stay at or below source exclusive clocks. The entire fixed-cohort reduction is the 72-clock Babasu JMP path on 1,532 calls; the newly priced Zoro branch is outside this fixed cohort. Most of the family body gap remains open.

`$00:80B5` already prices its routing instructions; its gap lies in JML targets. `$1E:8B19` is reduced only for the Zoro work above. `$06:84E2`, `$08:833C`, `$05:B5D3` and the rest of `$1D:C222` still need state/actor/ancilla body pricing. `$06:D22F` remains an attribution/ownership skip: native lacks a matching absorbable-draw frame, and JMP/drawing continuations cannot be priced as a flat residual. `$05:DF6C` already prices its ordinary body; differing direct/R6/deferred entries make its raw gap unsuitable for a flat charge. `$1E:FF34` still needs caller-specific movement-alias ownership. `$08:F702` still needs the actual JSR sites separated from caller-owned OAM writes. These last two remain open rather than declared fully priced artifacts.

### Route-wide clocks per call

Both fresh native ROM censuses profile all 1,473,123 iterations; every exclusive routine sum reconciles to its main-instruction counter.

| Routine | Native calls before / after | Native clocks/call before / after | Native exclusive clocks added |
| --- | ---: | ---: | ---: |
| `$1e:8b19` | 2,487,531 / 2,487,531 | 338.889 / 342.684 | 9,441,512 |
| `$00:80b5` | 1,473,123 / 1,473,123 | 487.114 / 487.114 | 0 |
| `$06:84e2` | 22,612,952 / 22,612,952 | 324.869 / 324.869 | 0 |
| `$08:833c` | 384,393 / 384,393 | 568.539 / 568.539 | 0 |
| `$1d:c222` | 843,363 / 843,363 | 343.823 / 343.823 | 0 |
| `$06:d22f` | 0 / 0 | unscoped / unscoped | 0 |
| `$05:b5d3` | 1,245,791 / 1,245,791 | 568.909 / 568.909 | 0 |
| `$05:df6c` | 2,367,479 / 2,367,479 | 2,514.531 / 2,514.531 | 0 |
| `$1e:ff34` | 0 / 0 | unscoped / unscoped | 0 |
| `$08:f702` | 0 / 0 | unscoped / unscoped | 0 |

**Full-route source/native comparison remains open.** The supplied complete source routine profile covers the frozen cohort; the prior full-source capture stopped near 201k. This short batch does not execute a new full source capture. Full-route source clocks/call and source gaps are therefore unavailable. The route-wide native additions above are not asserted to equal full-route source gap reductions.

### Full CPU-only verification

Both fresh main controls and both candidate full census gates PASS: 1,473,123 iterations each, unexplained zero; all runtime holds equal P2. Missing/excess count lag NMIs; FP counts iterations. These periodically seeded CPU checks do not establish continuous physical-field or exact A/V parity.

| Census | Missing before / after | Excess before / after | FP before / after | Exact lagged before / after | Within one before / after |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,950 / 4,950 | 0 / 0 | 0 / 0 | 34,679 / 34,679 | 39,561 / 39,561 |
| live | 5,647 / 5,647 | 590 / 590 | 132 / 132 | 34,545 / 34,545 | 39,429 / 39,429 |

The cartridge census closes 0 previously missing iterations and introduces 0 newly missing iterations.

`live-hold-deltas.json` retains every changed live hold/elapsed row, new and removed false positives and both source song-poll logs. Cartridge and live-SPC metrics remain separate.

Each category cell is main rom/live → final rom/live.

| Category | Source lag | Missing | Excess | FP | Exact lagged | Within one lagged |
| --- | ---: | --- | --- | --- | --- | --- |
| `0E/02` | 12,964 | 49/370 → 49/370 | 0/223 → 0/223 | 0/96 → 0/96 | 4,369/4,259 → 4,369/4,259 | 4,418/4,308 → 4,418/4,308 |
| `07/02` | 8,770 | 334/334 → 334/334 | 0/0 → 0/0 | 0/0 → 0/0 | 1,642/1,642 → 1,642/1,642 | 1,976/1,976 → 1,976/1,976 |
| `06/00` | 8,196 | 89/89 → 89/89 | 0/0 → 0/0 | 0/0 → 0/0 | 60/60 → 60/60 | 135/135 → 135/135 |
| `07/0E` | 7,979 | 121/121 → 121/121 | 0/0 → 0/0 | 0/0 → 0/0 | 4,831/4,831 → 4,831/4,831 | 4,952/4,952 → 4,952/4,952 |
| `08/00` | 6,466 | 19/253 → 19/253 | 0/0 → 0/0 | 0/0 → 0/0 | 138/134 → 138/134 | 157/153 → 157/153 |
| `09/23` | 6,054 | 363/363 → 363/363 | 0/0 → 0/0 | 0/0 → 0/0 | 816/816 → 816/816 | 1,179/1,179 → 1,179/1,179 |
| `0F/01` | 5,432 | 74/74 → 74/74 | 0/0 → 0/0 | 0/0 → 0/0 | 4,805/4,805 → 4,805/4,805 | 4,879/4,879 → 4,879/4,879 |
| `09/03` | 4,372 | 9/9 → 9/9 | 0/0 → 0/0 | 0/0 → 0/0 | 242/242 → 242/242 | 251/251 → 251/251 |
| `07/07` | 4,060 | 261/261 → 261/261 | 0/0 → 0/0 | 0/0 → 0/0 | 2,658/2,658 → 2,658/2,658 | 2,919/2,919 → 2,919/2,919 |
| `07/00` | 3,679 | 1,652/1,652 → 1,652/1,652 | 0/103 → 0/103 | 0/19 → 0/19 | 1,308/1,308 → 1,308/1,308 | 2,960/2,960 → 2,960/2,960 |
| `09/01` | 3,479 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 249/249 → 249/249 | 251/251 → 251/251 |
| `07/0F` | 2,992 | 71/71 → 71/71 | 0/0 → 0/0 | 0/0 → 0/0 | 2,647/2,647 → 2,647/2,647 | 2,718/2,718 → 2,718/2,718 |
| `0E/03` | 2,672 | 37/37 → 37/37 | 0/0 → 0/0 | 0/0 → 0/0 | 78/78 → 78/78 | 115/115 → 115/115 |
| `10/01` | 2,672 | 1/2 → 1/2 | 0/5 → 0/5 | 0/0 → 0/0 | 2,668/2,666 → 2,668/2,666 | 2,669/2,668 → 2,669/2,668 |
| `08/02` | 2,576 | 2/90 → 2/90 | 0/0 → 0/0 | 0/0 → 0/0 | 155/152 → 155/152 | 157/153 → 157/153 |
| `07/15` | 2,333 | 72/72 → 72/72 | 0/0 → 0/0 | 0/0 → 0/0 | 1,663/1,663 → 1,663/1,663 | 1,735/1,735 → 1,735/1,735 |
| `19/02` | 2,001 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1,855/1,855 → 1,855/1,855 | 1,855/1,855 → 1,855/1,855 |
| `0E/0A` | 1,845 | 20/20 → 20/20 | 0/0 → 0/0 | 0/0 → 0/0 | 70/70 → 70/70 | 90/90 → 90/90 |
| `09/00` | 1,253 | 1,123/1,123 → 1,123/1,123 | 0/0 → 0/0 | 0/0 → 0/0 | 69/69 → 69/69 | 1,192/1,192 → 1,192/1,192 |
| `0E/07` | 1,065 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 56/56 → 56/56 | 56/56 → 56/56 |
| `07/06` | 984 | 23/23 → 23/23 | 0/0 → 0/0 | 0/0 → 0/0 | 580/580 → 580/580 | 603/603 → 603/603 |
| `11/00` | 973 | 7/7 → 7/7 | 0/0 → 0/0 | 0/0 → 0/0 | 10/10 → 10/10 | 17/17 → 17/17 |
| `07/01` | 928 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 927/927 → 927/927 | 928/928 → 928/928 |
| `08/01` | 706 | 0/17 → 0/17 | 0/53 → 0/53 | 0/0 → 0/0 | 157/153 → 157/153 | 157/153 → 157/153 |
| `05/00` | 639 | 6/6 → 6/6 | 0/0 → 0/0 | 0/0 → 0/0 | 5/5 → 5/5 | 11/11 → 11/11 |
| `09/04` | 508 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 250/250 → 250/250 | 251/251 → 251/251 |
| `12/03` | 472 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 389/389 → 389/389 | 391/391 → 391/391 |
| `0F/00` | 449 | 22/22 → 22/22 | 0/0 → 0/0 | 0/0 → 0/0 | 265/265 → 265/265 | 287/287 → 287/287 |
| `07/12` | 391 | 18/18 → 18/18 | 0/0 → 0/0 | 0/0 → 0/0 | 221/221 → 221/221 | 239/239 → 239/239 |
| `09/2E` | 363 | 10/10 → 10/10 | 0/0 → 0/0 | 0/0 → 0/0 | 90/90 → 90/90 | 100/100 → 100/100 |
| `07/16` | 303 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 101/101 → 101/101 | 101/101 → 101/101 |
| `09/11` | 274 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 15/15 → 15/15 | 16/16 → 16/16 |
| `09/16` | 265 | 88/88 → 88/88 | 0/0 → 0/0 | 0/0 → 0/0 | 145/145 → 145/145 | 233/233 → 233/233 |
| `09/0D` | 247 | 89/89 → 89/89 | 0/0 → 0/0 | 0/0 → 0/0 | 134/134 → 134/134 | 223/223 → 223/223 |
| `0E/0B` | 243 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 81/81 → 81/81 | 81/81 → 81/81 |
| `09/0F` | 242 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 14/14 → 14/14 | 16/16 → 16/16 |
| `01/00` | 235 | 48/48 → 48/48 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/2C` | 228 | 4/4 → 4/4 | 0/0 → 0/0 | 0/0 → 0/0 | 32/32 → 32/32 | 36/36 → 36/36 |
| `10/00` | 157 | 0/1 → 0/1 | 0/81 → 0/81 | 0/0 → 0/0 | 157/154 → 157/154 | 157/155 → 157/155 |
| `17/01` | 141 | 3/3 → 3/3 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 2/2 → 2/2 |
| `1B/02` | 139 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 7/7 → 7/7 | 9/9 → 9/9 |
| `09/20` | 138 | 5/5 → 5/5 | 0/0 → 0/0 | 0/0 → 0/0 | 23/23 → 23/23 | 28/28 → 28/28 |
| `0E/05` | 138 | 36/36 → 36/36 | 0/0 → 0/0 | 0/0 → 0/0 | 83/83 → 83/83 | 119/119 → 119/119 |
| `00/01` | 114 | 7/7 → 7/7 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 11/11 → 11/11 |
| `0B/24` | 103 | 61/61 → 61/61 | 0/0 → 0/0 | 0/0 → 0/0 | 32/32 → 32/32 | 93/93 → 93/93 |
| `09/21` | 84 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 28/28 → 28/28 | 28/28 → 28/28 |
| `09/0E` | 83 | 5/5 → 5/5 | 0/0 → 0/0 | 0/0 → 0/0 | 11/11 → 11/11 | 16/16 → 16/16 |
| `15/07` | 83 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 79/79 → 79/79 | 79/79 → 79/79 |
| `19/00` | 72 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 4/4 → 4/4 |
| `07/13` | 71 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 47/47 → 47/47 | 47/47 → 47/47 |
| `0B/17` | 68 | 32/32 → 32/32 | 0/0 → 0/0 | 0/0 → 0/0 | 33/33 → 33/33 | 65/65 → 65/65 |
| `0B/28` | 68 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `0B/2B` | 66 | 23/23 → 23/23 | 0/0 → 0/0 | 0/0 → 0/0 | 43/43 → 43/43 | 66/66 → 66/66 |
| `0B/19` | 64 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 4/4 → 4/4 |
| `15/03` | 64 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 11/11 → 11/11 | 12/12 → 12/12 |
| `00/05` | 62 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 62/62 → 62/62 | 62/62 → 62/62 |
| `0B/26` | 61 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 4/4 → 4/4 |
| `0B/1A` | 60 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 4/4 → 4/4 |
| `1A/12` | 60 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 3/3 → 3/3 |
| `0B/29` | 59 | 33/33 → 33/33 | 0/0 → 0/0 | 0/0 → 0/0 | 18/18 → 18/18 | 51/51 → 51/51 |
| `18/00` | 59 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 2/2 → 2/2 |
| `1A/00` | 59 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 2/2 → 2/2 |
| `1A/1A` | 59 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/1C` | 59 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 3/3 → 3/3 |
| `1A/04` | 58 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/08` | 58 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/0A` | 58 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/18` | 58 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/0E` | 57 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/10` | 56 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `1A/06` | 55 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `07/03` | 53 | 53/53 → 53/53 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 53/53 → 53/53 |
| `1A/0C` | 53 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 2/2 → 2/2 |
| `11/07` | 51 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 17/17 → 17/17 | 17/17 → 17/17 |
| `13/05` | 51 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 51/51 → 51/51 | 51/51 → 51/51 |
| `1A/1E` | 51 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 2/2 → 2/2 |
| `0B/1C` | 48 | 11/11 → 11/11 | 0/0 → 0/0 | 0/0 → 0/0 | 29/29 → 29/29 | 40/40 → 40/40 |
| `1A/14` | 47 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 1/1 → 1/1 |
| `1A/02` | 46 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 1/1 → 1/1 |
| `1A/16` | 45 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `07/18` | 41 | 6/41 → 6/41 | 0/35 → 0/35 | 0/14 → 0/14 | 8/0 → 8/0 | 14/7 → 14/7 |
| `1A/20` | 41 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/12` | 32 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 16/16 → 16/16 | 16/16 → 16/16 |
| `12/08` | 32 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `0B/25` | 30 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `0B/00` | 29 | 9/9 → 9/9 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 13/13 → 13/13 |
| `0B/18` | 24 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `12/02` | 23 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 23/23 → 23/23 | 23/23 → 23/23 |
| `12/05` | 19 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 19/19 → 19/19 | 19/19 → 19/19 |
| `00/07` | 14 | 12/12 → 12/12 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 | 14/14 → 14/14 |
| `15/05` | 14 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 1/1 → 1/1 |
| `01/02` | 7 | 7/7 → 7/7 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 7/7 → 7/7 |
| `00/0A` | 6 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `12/09` | 4 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 4/4 → 4/4 | 4/4 → 4/4 |
| `13/04` | 3 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 3/3 → 3/3 | 3/3 → 3/3 |
| `00/02` | 2 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 | 1/1 → 1/1 |
| `00/06` | 2 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 |
| `04/01` | 2 | 2/2 → 2/2 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 2/2 → 2/2 |
| `00/00` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `03/01` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `07/0B` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `09/06` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `0E/01` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `19/01` | 1 | 1/1 → 1/1 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 1/1 → 1/1 |
| `00/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `00/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `00/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `01/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `01/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `01/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `01/05` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `03/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `03/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `03/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `03/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `04/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `04/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `04/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/05` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/08` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/0A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/0C` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/0D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/10` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/14` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/17` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/19` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `07/1A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/05` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/07` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/08` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/0A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/0C` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/10` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/13` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/14` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/15` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/1F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/22` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/2D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `09/2F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0B/08` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0B/1B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0B/27` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0B/2A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0E/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0E/08` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `0E/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/06` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/07` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0C` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0E` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `12/0F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `13/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `13/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `13/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `13/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/04` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/06` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/0A` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `15/0C` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/00` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/02` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `16/04` | 0 | 0/0 → 0/0 | 0/90 → 0/90 | 0/3 → 0/3 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/01` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/03` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/05` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/07` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/09` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/0B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/0D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/0F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/11` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/13` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/15` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/17` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/19` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/1B` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/1D` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/1F` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/21` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |
| `1A/22` | 0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 | 0/0 → 0/0 |

| Crystal/pendant seed | Properties | Overlays | Build | Iris init | Iris opening | Stabilize |
| ---: | --- | --- | --- | --- | --- | --- |
| 94000 | 65 / 64 | 5 / 5 | 17 / 17 | 2 / 2 | 35 / 35 | 16 / 16 |
| 380000 | 84 / 84 | 7 / 7 | 24 / 23 | 2 / 2 | 34 / 34 | 16 / 16 |
| 460000 | 84 / 84 | 7 / 7 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 626000 | 91 / 90 | 9 / 9 | 24 / 24 | 2 / 2 | 34 / 34 | 16 / 16 |
| 734000 | 84 / 84 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 880000 | 84 / 83 | 7 / 7 | 26 / 26 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1064000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |
| 1316000 | 84 / 83 | 7 / 7 | 25 / 25 | 2 / 2 | 34 / 34 | 16 / 16 |

All eight phase sweeps remain within one. All eight stable room, Link XY and whole-save outcomes match source in fresh main and final probes. The 162000–163000 rupee probe preserves all 27 requests and every request/PCM onset gap against fresh main; PCM-gap median is 8.25374531835206 frames.

Probe classification rows below cover only the 162000–163000 rupee window; rom/live rows cover the full route.

| Stage / capture | Classified differences | $0CFD-only |
| --- | --- | ---: |
| main / rom | `{"lag": 14}` | 10 |
| main / live | `{"lag": 14, "polygon": 14739, "song-end-poll": 639}` | 10 |
| main / probes | `{}` | 0 |
| final / rom | `{"lag": 14}` | 10 |
| final / live | `{"lag": 14, "polygon": 14739, "song-end-poll": 639}` | 10 |
| final / probes | `{}` | 0 |

The original 8,339 missing rows gain 116,496 main instruction clocks, with no negative row. HDMA upper-bound excursions remain zero; the same 63 masked-init leases remain declined.

Both ordinary candidate builds and the fresh main default build are warning-free. Candidate default matches every state/display/SRAM/PCM/input/context record and EOF of fresh main's 200k ledger. Default/ROM-timing accounting is unchanged. Known lib-test fixture warnings remain separate from the ordinary builds.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,537 | 1 | 3 |
| reduced | 2,072 | 0 | 2 |

The sole default library failure is the known spotlight caller-return fixture; reduced passes entirely. The manifest checks complete, disjoint named test unions. Both SNES library suites, Python routine-coverage tests and `git diff --check` pass.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Scratch scripts stayed in `/tmp`; supplied ledgers/cache and the other checkout remained read-only. This stops at one verified group; remaining body prices and full-route source reconciliation are open.

## Generated block-cost tooling on `codex/block-costs` (main `f7ccbc0c`)

Evidence: `target/block-costs/`. This batch stops at a verified tooling point: the generated table, lagless marker API, reusable ROM fixture helper, source/native block receipts, and an existing Zoro price converted to markers. **No new routine was priced; no additional instruction clocks were closed.** The requested 3–4 new leaders and the measured frozen-cohort block ranking remain open.

`rom_block_costs.py --emit-rust` reads `scripts/rom_block_entries.json` (bounded routine entries and entry M/X), walks source control flow and emits `crates/zelda3/src/rom_block_costs.rs`. Calls end blocks and price only their opcode. The generated rows carry address/end, entry M/X, static master clocks, conditional branch PC and indexed-read PCs; comments retain the variable-cost notes and ROM SHA-256. Callees must preserve the declared M/X, DP low byte is zero, and bus assumptions are explicit. Bounds must exclude inline dispatch data; dispatch arms need separate entries. Conflicting widths, overlapping decoding, dynamic width/DP restores, indirect dispatch and unresolved bus costs are rejected. Python tests regenerate and compare the checked-in table on the ROM, skipping that comparison when the ROM is absent.

A native site creates `let blocks = crate::cycle_ledger::blocks(self.lag_free)` and names source work with `blocks.block(0x1e9bd0)`, `blocks.taken(0x1e9bd3)`, or `blocks.crossing(indexed_read_pc)`. Extras must name an eligible generated instruction. Disabled markers and muted previews add no charge. Existing routine scopes retain ownership; markers create no scopes. The new generic `assert_rom_marker_work` helper executes an unmodified ROM CPU fixture, accepts an explicit exclusive-source ownership predicate for calls/JMP tails/local stack pushes, and compares its instruction transactions with the native marker trail under that routine owner. It returns the source machine for behavioral assertions.

Zoro `$1E:9BD0..9C6A` and its `$9BC8` dispatch use the generated blocks. The allocator result determines the actual rejected-slot loop; there is no fitted residual or scene price. The Zoro body has 30 one-line marker calls plus one marker-context declaration; its dispatch has three marker calls. The existing fixture now uses the generic helper instead of a private source stepping/counting loop. All 128 ROM cases retain the prior totals, state/garnish checks and zero default-mode caller charge. New ledger tests cover lane gating, muting, source ownership, branch extras and page extras.

`ZELDA3_LEDGER_BLOCK_PROFILE=1` adds `instruction-profile.csv` to the existing CPU-only source routine profiler, under the same selected complete iterations. Receipts preserve source call-frame owner, PC, entry M/X, executions, actual block entries (including computed-jump destinations) and instruction clocks; interrupts, refresh and DMA stalls use the existing routine-profile exclusions. Interrupt handlers do not introduce spurious entries in the interrupted main stream. Native `--routine-profile` now writes a companion `*.blocks.csv` with routine/PC/kind/count/clocks. `scripts/rom_block_worklist.py` partitions observed instructions at real control-flow boundaries, reconciles every iteration/owner to the source exclusive profile, and ranks blocks by executed source clocks. It distinguishes marker declarations from measured native marker charges and leaves ambiguous M/X attribution unresolved rather than counting a native charge twice. Legacy scalar prices are not block-resolved; no marker does not mean no price.

The retained frozen source directory (`cost-pacing10/source-missing`) has only routine and structural profiles, without PC receipts. Routine totals cannot identify which basic blocks executed, so the tool refuses to synthesize a frozen block ranking from them. A fresh source CPU capture with the block-profile flag is required before selecting the requested new leaders; use the retained frozen routine profile as the reconciliation anchor, so a recapture cannot silently redefine the source cohort. This short batch does not run that complete source recapture. The pipeline is instead exercised from reset over 200 CPU-only hosts: 11 selected complete iterations, 4,044 source PC rows, and every instruction clock reconciled before ranking. `worklist-smoke.json` is explicitly a smoke ranking, not the frozen gameplay cohort. Enabling the new source receipts leaves both its routine and structural profiles byte-identical to a matching capture with receipts disabled.

Regeneration and ranking interfaces:

```sh
python3 scripts/rom_block_costs.py --emit-rust > crates/zelda3/src/rom_block_costs.rs
python3 -m unittest discover -s scripts -p 'test_rom_block*.py'
python3 scripts/rom_block_worklist.py <source>/instruction-profile.csv <frozen-source>/routine-profile.csv \
  --cohort <batch-22>/decomposition.json --native <candidate>.blocks.csv --json <worklist>.json
```

The supplied context listed 4,950 ROM / 5,647 live missing NMIs. Fresh clean-main controls from the pinned HEAD are used below for before/after; a lower fresh baseline is not credited to the marker conversion.

CPU-only verification:

| Census | Missing before / after | Excess before / after | FP before / after |
| --- | ---: | ---: | ---: |
| rom | 4,936 / 4,936 | 0 / 0 | 0 / 0 |
| live | 5,633 / 5,633 | 590 / 590 | 132 / 132 |

Both fresh main controls and both candidate full census gates PASS: 1,473,123 iterations each, unexplained zero. Every predictor CSV column is byte-identical before/after in both lanes, not only the headline hold counts. Every cartridge runtime hold remains P2. The frozen 2,825-row native routine profile is byte-identical, and all route-wide native routine call/inclusive/exclusive aggregates are identical. The frozen main-instruction gap remains **203,337,632 master clocks**, with **0 clocks closed**. These periodically seeded CPU-only checks do not establish continuous physical-field or exact A/V parity.

The fresh main default binary was built before Rust edits. Candidate default matches every state/display/SRAM/PCM/input/context record and EOF in its freshly recorded 200k ledger. Ordinary default and reduced builds and the source-profiler build are warning-free; existing lib-test fixture warnings remain separate.

| Zelda library | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,542 | 1 | 3 |
| reduced | 2,077 | 0 | 2 |

The sole default library failure is the known spotlight caller-return fixture; reduced passes entirely. The shard manifest verifies complete, disjoint named test unions. Both SNES lib suites, the five block generator/worklist Python tests (including ROM regeneration), existing routine-coverage Python tests, the source/worklist smoke and `git diff --check` pass. `verification.json`, `provenance.json`, full predictor CSVs, native block receipts and the fixture/build/suite logs retain the evidence.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. The source smoke executes only the in-repo CPU model. Scratch drivers stayed in `/tmp`; supplied ledgers/cache and the other checkout remained read-only. The next pricing batch still needs the frozen source PC capture, its measured block ranking, and 3–4 new marker-based routine prices.

## Live song-end diagnosis on `codex/live-songend` (main `70e51911`)

Diagnosis-only stopping point. No audio clock, song duration, instruction price or pacing behavior changed. The existing code additions are opt-in observations under `ZELDA3_DEBUG_LAGLESS_SONG_POLL`; they record each native poll's priced instruction position, opening handler, SPC cursor and logical-iteration context. Evidence is in `target/live-songend/`, principally `poll-diagnosis.json`, the two stderr traces, audio maps, and source poll receipts extracted read-only from the pinned cache.

### What the measurements establish

**(b) is present:** lagless `read_apui00_for_song_end_poll` returns the frame-entry boolean projection of the preceding output callback's SPC acknowledgement. It does not advance the SPC to the read. Only the original-timing live-owner branch samples an intra-window clone. Boolean 1 versus raw SPC `$13` is intentional normalization; the gameplay branch tests zero versus nonzero. `advance_lagless` advances the persistent output cursor by 534 DSP pairs (17,088 SPC clocks) per output callback, including held and masked callbacks. The trace also retains `origin + local`, the actual SPC execution position; the nominal cursor is not the instruction-retirement clock, because audio execution includes the existing 19-cycle lookahead and instruction overshoot. `poll-diagnosis.json` records both coordinates.

The read position is not the current instruction counter verbatim. Fresh `scripts/rom_block_costs.py` listings for `$08:C3FD` and `$08:C606` show a 30-clock system-bank `LDA $2140`, followed by a 16-clock untaken branch. Native callers charge both before invoking the helper. Thus the priced I/O-access start is the recorded instruction count minus 22 (branch 16 plus I/O 6). This is an instruction position, excluding opening-handler time and bus stalls; it is not an exact CPU wall/bus timestamp. No price was added or altered.

| Requested iteration | Cartridge answer / priced read start | Live answer / priced read start | SPC cursor at poll (DSP pairs), cartridge / live |
| ---: | --- | --- | --- |
| 94848 | $13 / 17,668 | $00 / 17,668 | 55,208,658 / 55,208,658 |
| 94849 | $13 / 17,668 | no poll | 55,209,192 / no poll |
| 229029 | $13 / 18,678 | $01 / 18,678 | 132,687,786 / 132,687,786 |
| 229030 | $13 / 18,678 | $00 / 18,678 | 132,688,320 / 132,688,320 |

At 94849 the live lane has already left the polling branch and executes the five-hold workload. On this joint main, the second first mismatch is 229030; the historical 27b shift to 229029 is not reproduced as the current first mismatch. Both observations use uninterrupted full-route census execution, not forced-window seeds.

All ten retained song-cause clusters were sampled, including the three pendant and seven crystal clusters. Each row below pairs the current live first zero with the cartridge ledger's first zero following that cluster. Source receipt offsets are **candidate host-relative DSP sample positions** in the ledger's opening/closing host interval. They are quantized, may include multiple source polls and are not an exact CPU timestamp or an unambiguous iteration join.

| Retained cause | Current live first zero | Live SPC cursor (DSP pairs) | Priced read start, main instructions | Cartridge first zero | Source receipt candidates (host:sample offsets) |
| ---: | ---: | ---: | ---: | ---: | --- |
| 94849 | 94848 | 55,208,658 | 17,668 | 94854 | 103726:75 |
| 142083 | 142083 | 82,419,696 | 19,474 | 142088 | 154786:77; 154787:75 |
| 229030 | 229030 | 132,688,320 | 18,678 | 229035 | 249308:75; 249309:79 |
| 380876 | 380876 | 220,148,442 | 19,460 | 380881 | 413473:85 |
| 462622 | 462622 | 44,963,334 | 23,896 | 462628 | 500859:88 |
| 627250 | 627250 | 93,171,786 | 25,664 | 627256 | 677406:96; 677407:93 |
| 735861 | 735861 | 59,920,674 | 24,194 | 735867 | 792918:96; 792919:94 |
| 881728 | 881728 | 80,722,644 | 27,352 | 881734 | 947003:89; 947004:90 |
| 1064729 | 1064728 | 102,902,868 | 25,254 | 1064734 | 1143492:87 |
| 1318330 | 1318330 | 144,300,150 | 26,704 | 1318335 | 1416755:95 |

The cartridge remains nonzero through 94853 and 229034, first answering zero at 94854 and 229035. The historical one-iteration batch shifts therefore describe movement between builds, not a one-iteration live-to-cartridge gap. The sampled live zeroes precede the cartridge zeroes by several logical iterations. Their SPC acknowledgement is already zero at the callback boundary before the priced read, so advancing forward to that read cannot recover the source's still-playing value. An ignored within-iteration offset exists, but these measurements do not establish it as the cause of the early zeroes.

`source-output-clock-positions.json` joins the pinned source audio hash ledger to the poll receipt candidates, retaining cumulative output-grid positions for all ten sampled source-zero windows and the four requested iterations. These source sample-grid coordinates remain distinct from exact SPC execution or CPU bus timestamps. `relative-clock-history.json` additionally measures the first three music-request-to-cartridge-zero intervals: source host deltas are 638, 637 and 638; native cartridge-lane output callback deltas are 638, 638 and 638. The first and third therefore have no net callback-count discrepancy across those intervals. Matched command delivery and SPC initial state are still unmeasured, so this is not tempo proof.

`clock-accounting-audit.json` checks every iteration in both full lanes: callbacks equal one main callback plus held plus masked callbacks, with zero residual on all 1,473,123 rows. At every traced poll the SPC cursor equals the output callback clock up to one constant offset per explicit reseed epoch; that offset never changes inside an epoch. Thus no unaccounted SPC advance or double-counted hold/mask callback appears in these observations. This does not equate the callback clock with the source physical-field clock.

**(c) affects the clock path, but a defect is not isolated:** the native output clock counts actual held/masked callbacks, and instruction additions can change that count before a poll. The audio maps retain every such callback and reseed. That is not evidence that a particular hold or masked field is incorrectly counted. **(a) remains open:** unequal command/clock histories have not been eliminated, so these runs cannot attribute the early end to SPC/DSP fidelity or tempo. No matched-command, matched-state source/native song-duration experiment was performed.

**The exact source CPU position requested in scope remains unavailable.** The supplied iteration ledger stores APU values and host bounds, and the pinned receipt stores host-relative DSP offsets; neither stores the source CPU bus timestamp/PC for this join. No fresh source CPU capture or Snes9x execution was performed. Likewise the native priced counter is not a complete bus cursor. These limitations prevent treating a priced-offset rule as a verified general fix. This batch stops at the measured clock-mapping gap and early-zero counterexample.

### Current-main verification

The supplied historical 641-row count is not reproduced by this current-main invocation: the fresh live trace has 639 song-end-poll rows, 14 lag rows and 14,741 polygon rows. The fresh cartridge metrics are 4,936 missing / 0 excess / 0 FP; live is 5,633 / 590 / 132. These are current-main measurements, not an improvement claim against the older batch notes. Both censuses use the reduced diagnostic binary, `--seed-every 2000`, and the bound read-only full ledger/cache inputs.

Both fresh complete CPU census gates PASS, 1,473,123 iterations each, unexplained zero. These are periodically seeded logical gates, not continuous physical-field or exact A/V proof. There is no behavioral after-trial: the before/after columns below are identical because the change only logs observations. They are not two independent full runs per lane.

| Lane | Missing before / after | Excess before / after | FP before / after | Song-end classified rows before / after | Audio duration change |
| --- | ---: | ---: | ---: | ---: | ---: |
| rom | 4,936 / 4,936 | 0 / 0 | 0 / 0 | 0 / 0 | 0 (logging only) |
| live | 5,633 / 5,633 | 590 / 590 | 132 / 132 | 639 / 639 | 0 (logging only) |

`live-category-before-after.csv` retains every live category's missing/excess/FP values with the same logging-only interpretation. The full current category metrics are also in `live.metrics.json`; no category is omitted from those artifacts. Route audio durations are rom: 841,580,262 stereo pairs / 26299.383187 seconds, live: 841,507,104 stereo pairs / 26297.097000 seconds. These are output-grid durations, not a new measurement of source song tempo.

Both ordinary diagnostic builds and the clean-main default baseline build are warning-free. The fresh main default binary recorded a new 200k ledger; the diagnostic default build matches all state/display/SRAM/PCM/input/context records and EOF. Default/ROM-timing accounting is unchanged. Library results (test-only fixture warnings separate from ordinary builds):

| Library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,538 | 1 | 3 |
| reduced | 2,073 | 0 | 2 |

Default retains only the known spotlight caller-return fixture failure; reduced passes entirely. Both CPU-only SNES library suites also pass. Interrupted shards were recovered by running exactly the unobserved names from the saved test lists; the manifest checks the complete, disjoint union of observed results. `git diff --check` passes. Binary, source, input and output identities are bound by `verification.json` and `artifact-sha256.json`. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Supplied ledgers/cache stayed read-only; scratch scripts stayed in `/tmp`.


### Live category table (logging-only before / after)

| Module/submodule | Source lag | Missing before / after | Excess before / after | FP before / after |
| --- | ---: | ---: | ---: | ---: |
| `00/00` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `00/01` | 114 | 7 / 7 | 0 / 0 | 0 / 0 |
| `00/02` | 2 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/05` | 62 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/06` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `00/07` | 14 | 12 / 12 | 0 / 0 | 0 / 0 |
| `00/0A` | 6 | 0 / 0 | 0 / 0 | 0 / 0 |
| `00/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/00` | 235 | 48 / 48 | 0 / 0 | 0 / 0 |
| `01/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/02` | 7 | 7 / 7 | 0 / 0 | 0 / 0 |
| `01/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `01/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `03/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `03/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/01` | 2 | 2 / 2 | 0 / 0 | 0 / 0 |
| `04/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `04/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `05/00` | 639 | 6 / 6 | 0 / 0 | 0 / 0 |
| `06/00` | 8,196 | 89 / 89 | 0 / 0 | 0 / 0 |
| `07/00` | 3,679 | 1,651 / 1,651 | 103 / 103 | 19 / 19 |
| `07/01` | 928 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/02` | 8,770 | 334 / 334 | 0 / 0 | 0 / 0 |
| `07/03` | 53 | 53 / 53 | 0 / 0 | 0 / 0 |
| `07/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/06` | 984 | 23 / 23 | 0 / 0 | 0 / 0 |
| `07/07` | 4,060 | 261 / 261 | 0 / 0 | 0 / 0 |
| `07/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0B` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `07/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/0E` | 7,979 | 121 / 121 | 0 / 0 | 0 / 0 |
| `07/0F` | 2,992 | 71 / 71 | 0 / 0 | 0 / 0 |
| `07/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/12` | 391 | 18 / 18 | 0 / 0 | 0 / 0 |
| `07/13` | 71 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/15` | 2,333 | 72 / 72 | 0 / 0 | 0 / 0 |
| `07/16` | 303 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/18` | 41 | 41 / 41 | 35 / 35 | 14 / 14 |
| `07/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `07/1A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `08/00` | 6,466 | 253 / 253 | 0 / 0 | 0 / 0 |
| `08/01` | 706 | 17 / 17 | 53 / 53 | 0 / 0 |
| `08/02` | 2,576 | 90 / 90 | 0 / 0 | 0 / 0 |
| `09/00` | 1,253 | 1,118 / 1,118 | 0 / 0 | 0 / 0 |
| `09/01` | 3,479 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/03` | 4,372 | 9 / 9 | 0 / 0 | 0 / 0 |
| `09/04` | 508 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/06` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/0D` | 247 | 87 / 87 | 0 / 0 | 0 / 0 |
| `09/0E` | 83 | 5 / 5 | 0 / 0 | 0 / 0 |
| `09/0F` | 242 | 2 / 2 | 0 / 0 | 0 / 0 |
| `09/10` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/11` | 274 | 1 / 1 | 0 / 0 | 0 / 0 |
| `09/12` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/14` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/16` | 265 | 86 / 86 | 0 / 0 | 0 / 0 |
| `09/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/20` | 138 | 5 / 5 | 0 / 0 | 0 / 0 |
| `09/21` | 84 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/23` | 6,054 | 362 / 362 | 0 / 0 | 0 / 0 |
| `09/2C` | 228 | 4 / 4 | 0 / 0 | 0 / 0 |
| `09/2D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `09/2E` | 363 | 10 / 10 | 0 / 0 | 0 / 0 |
| `09/2F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/00` | 29 | 9 / 9 | 0 / 0 | 0 / 0 |
| `0B/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/17` | 68 | 31 / 31 | 0 / 0 | 0 / 0 |
| `0B/18` | 24 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/19` | 64 | 2 / 2 | 0 / 0 | 0 / 0 |
| `0B/1A` | 60 | 2 / 2 | 0 / 0 | 0 / 0 |
| `0B/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/1C` | 48 | 11 / 11 | 0 / 0 | 0 / 0 |
| `0B/24` | 103 | 60 / 60 | 0 / 0 | 0 / 0 |
| `0B/25` | 30 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/26` | 61 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0B/27` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/28` | 68 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/29` | 59 | 32 / 32 | 0 / 0 | 0 / 0 |
| `0B/2A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0B/2B` | 66 | 23 / 23 | 0 / 0 | 0 / 0 |
| `0E/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `0E/02` | 12,964 | 370 / 370 | 223 / 223 | 96 / 96 |
| `0E/03` | 2,672 | 37 / 37 | 0 / 0 | 0 / 0 |
| `0E/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/05` | 138 | 36 / 36 | 0 / 0 | 0 / 0 |
| `0E/07` | 1,065 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/08` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0E/0A` | 1,845 | 20 / 20 | 0 / 0 | 0 / 0 |
| `0E/0B` | 243 | 0 / 0 | 0 / 0 | 0 / 0 |
| `0F/00` | 449 | 22 / 22 | 0 / 0 | 0 / 0 |
| `0F/01` | 5,432 | 74 / 74 | 0 / 0 | 0 / 0 |
| `10/00` | 157 | 1 / 1 | 81 / 81 | 0 / 0 |
| `10/01` | 2,672 | 2 / 2 | 5 / 5 | 0 / 0 |
| `11/00` | 973 | 7 / 7 | 0 / 0 | 0 / 0 |
| `11/07` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/02` | 23 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/03` | 472 | 2 / 2 | 0 / 0 | 0 / 0 |
| `12/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/05` | 19 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/08` | 32 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/09` | 4 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0E` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `12/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/04` | 3 | 0 / 0 | 0 / 0 | 0 / 0 |
| `13/05` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/03` | 64 | 1 / 1 | 0 / 0 | 0 / 0 |
| `15/04` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/05` | 14 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/06` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/07` | 83 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0A` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `15/0C` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/00` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/02` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `16/04` | 0 | 0 / 0 | 90 / 90 | 3 / 3 |
| `17/01` | 141 | 3 / 3 | 0 / 0 | 0 / 0 |
| `18/00` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `19/00` | 72 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/01` | 1 | 1 / 1 | 0 / 0 | 0 / 0 |
| `19/02` | 2,001 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/00` | 59 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/01` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/02` | 46 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/03` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/04` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/05` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/06` | 55 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/07` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/08` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/09` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0A` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0C` | 53 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0E` | 57 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/0F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/10` | 56 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/11` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/12` | 60 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/13` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/14` | 47 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/15` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/16` | 45 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/17` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/18` | 58 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/19` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1A` | 59 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1B` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1C` | 59 | 1 / 1 | 0 / 0 | 0 / 0 |
| `1A/1D` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1E` | 51 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/1F` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/20` | 41 | 2 / 2 | 0 / 0 | 0 / 0 |
| `1A/21` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1A/22` | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `1B/02` | 139 | 2 / 2 | 0 / 0 | 0 / 0 |

## Continuation on `codex/cost-pacing29` (main `a28e327d`)

### Fresh joint main baseline

The documented joint baseline is ROM **4,936 missing / 0 excess / 0 FP**, live **5,633 / 590 / 132**. Fresh clean-main controls reproduce it; historical batch deltas are not added. Main was built before edits and recorded a fresh default 200k ledger. `target/cost-pacing29/main-provenance.json` binds the binary to the main tree. All evidence for this batch is in `target/cost-pacing29/`; scratch drivers remain in `/tmp`.

| Census | Missing before / after | Excess before / after | FP before / after |
| --- | ---: | ---: | ---: |
| rom | 4,936 / 4,907 | 0 / 0 | 0 / 0 |
| live | 5,633 / 5,606 | 590 / 592 | 132 / 132 |

### ROM marker work

Six requested parent gaps receive new prices, with two additional source entries for the shared absorbable entry and the damage trampoline. These are partial coverage additions: Pengator control before its inactivity test, Fall1 self work, dash-dust control through its preparation call, absorbable drawing selection, safe ancilla coordinates, and the far damage wrapper at two source-proven bank-$1E call sites. No existing scalar price is converted. The larger remaining handler bodies and the other requested routines stay open.

`scripts/rom_block_costs.py --emit-rust` supplies all 49 new blocks from bounded entries in `scripts/rom_block_entries.json`. The source ROM and entry M/X determine the prices. Every new runtime charge uses `blocks.block`, `blocks.taken` or `blocks.crossing`, gated by `lag_free`, with source addresses in comments. No lag count, scene threshold, multiplier or fitted residual supplies a price. The absorbable table page crossings use the ROM table addresses and actual sprite type; its double return leaves the second PLA/RTS with the caller. Drawing tails and called bodies retain their separate prices.

Clocks below are newly charged ROM instruction master clocks, measured from marker receipts. The cohort is the frozen batch-22 set of 2,825 iterations; route clocks cover all 1,473,123 native ROM census iterations. Lines are gross additions to native Rust, including comments and caller rewiring, excluding fixtures, generated rows and the entry manifest. Shared absorbable code is counted once.

| Source owner / work | Frozen-cohort clocks | Whole-route clocks | Native lines added |
| --- | ---: | ---: | ---: |
| `$1E:8B19` Pengator `$A196..A1BA` | 161,200 | 17,033,846 | 9 |
| `$06:84E2` Fall1 `$852E..8542` | 4,270 | 3,470,730 | 9 |
| `$08:833C` dash dust `$BC92..BCC3` | 82,046 | 22,279,586 | 20 |
| `$06:D22F` absorbable transient | 612,288 | 195,194,370 | 49 |
| `$06:D232` shared absorbable entry | 1,044 | 2,178,448 | shared above |
| `$08:F702` safe coordinates | 2,201,964 | 70,122,952 | 13 |
| `$06:F121` far damage wrapper | 11,590 | 1,148,930 | 9 |
| `$1E:C26B` damage trampoline | 6,344 | 628,888 | 15 |

Pricing implementation additions: **124 lines**; fixtures: **221 lines**; generated table: **98 lines**; entry manifest: **9 added / 1 removed**. The frozen main instruction gap changes from **203,337,632** to **200,256,886** master clocks: **3,080,746 clocks closed**. `routine-cohort-before-after.json` retains every changed exclusive owner and the independent new-marker totals; every per-routine exclusive delta equals its independently recorded new marker clocks. Whole-route native exclusive sums reconcile to the main counter on every iteration.

Seven `assert_rom_marker_work` fixtures exercise 543 unmodified-ROM cases: Pengator reset branches (32), Fall1 live/dead/offscreen paths (16), both absorbable entries, all 15 item types, sorting, CHR exclusion, allocation, double returns and phasing (420), safe coordinate boundaries (56), dash-dust branches (12), far damage gates (6), and its fixed trampoline (1). The shared absorbable fixture checks each source entry independently. These use the in-repo CPU only.

### CPU-only verification

Both fresh main controls and both candidate full census gates PASS: 1,473,123 iterations per lane, unexplained zero. Every candidate runtime hold equals P2. Missing/excess count lag NMIs; FP counts iterations. These periodically seeded logical gates do not establish continuous physical-field or exact A/V parity.

The ROM census closes 29 previously missing iterations and introduces 0 newly missing iterations. New false-positive iterations: ROM 0, live 6. Six old live FP iterations disappear, leaving the live total at 132; live excess increases by two. `live-hold-deltas.json` retains all shifted rows. Full predictor CSVs and category JSONs retain all before/after values.

All eight crystal/pendant windows keep every measured phase within one frame of source. Their phase durations and stable outcomes are identical before/after. The rupee-refill command and PCM onset measurements are also identical. `phase-verification.json`, `final-probe-metrics.json` and `final-stable-outcomes.json` retain these checks.

The candidate default binary matches every state/display/SRAM/PCM/input/context record and EOF against the fresh main 200k ledger. All new prices/scopes are lagless-only; default/ROM-timing accounting and the existing scalar prices are unchanged. Both ordinary builds are warning-free; the known lib-test-only fixture warnings are separate.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,549 | 1 | 3 |
| reduced | 2,084 | 0 | 2 |

Default retains only the known spotlight caller-return fixture failure; reduced passes entirely. The first optimized-suite run also exposed two pre-existing marker guard tests whose expected debug-assert panics were disabled by the parity profile. Two line replacements make those assertions active under `cfg(test)` as well as debug builds. Both ordinary binaries rebuilt byte-identically (`test-guard-build-equivalence.json`), so the complete census and 200k receipts remain bound to the final binaries. The two affected tests and all marker API/ROM fixtures were rerun on the rebuilt test binaries. The final shard manifest preserves the original complete, disjoint test union and these explicit replacements; original failing logs remain retained. Both CPU-only SNES lib suites, the five block generator/worklist Python tests (including exact regeneration), routine-coverage Python tests and `git diff --check` pass. `verification.json` binds the final source, builds, gates, metrics and library evidence; `artifact-sha256.json` records artifact identities.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Supplied ledger/cache metadata remained unchanged across all 1,512 input files. This batch stops at the verified CPU-only pricing point above.

## Continuation on `codex/cost-pacing31` (main `512f553d`)

### Fresh joint main baseline

The joint baseline is ROM **4,907 missing / 0 excess / 0 FP**, live **5,606 / 592 / 132**. Fresh main controls reproduce it. Main was built from the initially clean tree before pricing edits and recorded a fresh default 200k ledger. `target/cost-pacing31/main-provenance.json` binds that build; all batch evidence is under `target/cost-pacing31/`. Scratch drivers remain in `/tmp`. Historical batch deltas are not added.

| Census | Missing before / after | Excess before / after | FP before / after |
| --- | ---: | ---: | ---: |
| rom | 4,907 / 4,812 | 0 / 0 | 0 / 0 |
| live | 5,606 / 5,508 | 592 / 589 | 132 / 131 |

### Zero-native source entries

The largest near-zero row, `$06:DC10`, is an ownership alias: its coordinate JSR is already charged at DC10, drawing instructions at DC13, coordinate work at E41E and shadow work at DC66. No second drawing price is added. The inactive helper at `$1D:E8A2` is likewise already charged through the shared bank-6 predicate. Existing SetOam_XY scalar prices remain intact. `ownership-audit.json` records these exclusions.

The additions price whole executed entries at statically confirmed ROM callers. Far drawing, damage and collision wrappers exclude their already-priced children. The far tile lookup has its own E886 body; the scalar-priced E883 entry continues using the unmarked semantic helper. Movement retains the XY tail-jump owner and its separately called X/Y bodies. Raw movers and partial CPU continuations retain their existing accounting. Bank-1D/1E movement and the batch-30 owners are untouched. Some source callers remain unpriced; a complete entry price does not imply complete route coverage for that entry.

All prices come from `scripts/rom_block_costs.py --emit-rust`: 57 new blocks containing 405 ROM instructions. Entry M/X, actual branch outcomes and the actual ancilla slot determine charges. The generator now follows statically paired PHP/PLP width restores and rejects unknown or aliased status stacks; its regression is identical to the parallel batch-30 generator change. No scalar price is converted. No lag count, fitted residual, scene threshold, multiplier or scene table supplies a charge. Every new runtime marker and scope is gated by `lag_free`.

Clocks below are independent new-marker receipts. The frozen cohort contains 2,825 iterations; whole-route clocks cover 1,473,123 iterations. Lines are gross native additions including address comments and replaced caller lines, excluding fixtures, generated rows, manifest and documentation. Shared helper lines are listed once in the shared row.

| Source owner / work | Frozen-cohort clocks | Whole-route clocks | Native lines added |
| --- | ---: | ---: | ---: |
| `$06:DBF0` far large-draw wrapper | 1,608,160 | 272,453,540 | 110 |
| `$08:8EAD` ancilla hitbox | 1,835,734 | 38,662,088 | 10 |
| `$06:F2AA` far damage wrapper | 284,054 | 21,306,448 | 42 |
| `$06:E496` far collision wrapper | 694,860 | 139,935,924 | 127 |
| `$06:F70A` Link hitbox | 1,406,240 | 296,322,592 | 7 |
| `$06:F705` conditional Link hitbox | 814,504 | 36,718,888 | 7 |
| `$06:E886` entity tile body | 1,458,000 | 84,698,208 | shared |
| `$06:E87B` far entity tile wrapper | 380,000 | 22,652,750 | shared |
| `$05:FA00` bank-5 Y movement | 615,712 | 115,256,848 | shared |
| `$06:E92C` XY movement | 621,450 | 54,163,212 | 32 |
| `$06:E932` X movement | 306,694 | 39,307,740 | shared |
| `$06:E93E` Y movement | 475,672 | 49,108,912 | 4 |
| `$05:FE6F` both tree-top drawing paths | 1,600,368 | 6,331,198 | 21 |
| `$07:CEC9` horizontal-slope setup | 1,293,182 | 187,088,528 | 9 |
| `$0E:DE9A` map16 coordinate body | 838,656 | 229,360,768 | 4 |
| `$0D:B871` property reset body | 774,330 | 81,204,480 | 4 |
| `$05:F9ED` bank-5 XY wrapper | 103,314 | 24,003,018 | 50 |
| `$05:F9F4` bank-5 X wrapper | 137,238 | 32,086,458 | 2 |
| shared XY helper | included above | included above | 17 |
| shared X helper | included above | included above | 11 |
| shared axis helpers | included above | included above | 28 |
| shared far tile lookup and caller rewiring | included above | included above | 39 |

Native additions: **524 lines**; fixtures: **309**; generated table: **114**; generator/tests: **41**; entry manifest: **17 added / 1 removed**. `lines-added.json` retains every counted line and owner/shared group.

The frozen instruction gap changes from **200,256,886** to **185,008,718** master clocks: **15,248,168 clocks closed**. Whole-route additions total **1,730,661,600** clocks. Marker sums reconcile independently to the change in main instruction counters; routine-exclusive changes and source comparisons are retained in `routine-cohort-before-after.json`. Native exclusive sums reconcile to the main counter on every full-route iteration.

Eighteen `assert_rom_marker_work` fixtures exercise the new entries with the unmodified ROM and the in-repo CPU: hitbox branches, ancilla page crossings, complete bank wrappers, both movement signs and zero velocities at boundary slots, tail-jump ownership, tile lookup floor/indoor branches, all tree drawing states, slope directions and reset/map16 bodies. The XY fixture includes PHP's temporary stack byte when identifying its tail owner. These verify instruction charges; they do not establish semantic equivalence or A/V parity.

### CPU-only verification

Both fresh main and both final full census gates PASS: **1,473,123 iterations per lane**, unexplained zero. Every candidate runtime hold equals P2. Missing/excess count lag NMIs; FP counts iterations. These are periodically seeded logical gates.

ROM closes 95 previously missing iterations and introduces 0 newly missing iterations. Newly introduced FP iterations: ROM 0, live 7. Complete before/after predictor CSVs, category metrics and shifted-row lists are retained; totals above include both introduced and removed rows.

All eight crystal/pendant windows keep every measured phase within one frame of source. Phase and rupee-refill measurements are identical to fresh main. Stable outcomes are identical. `phase-verification.json`, `final-probe-metrics.json` and `final-stable-outcomes.json` retain the checks.

The final default binary matches every state/display/SRAM/PCM/input/context record and EOF in the fresh main 200k ledger. Default/ROM-timing accounting is unchanged. Both ordinary builds are warning-free; existing lib-test-only fixture warnings are separate.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,567 | 1 | 3 |
| reduced | 2,102 | 0 | 2 |

Default retains only the known spotlight caller-return failure; reduced passes entirely. The complete, disjoint library test union is bound in `library-shards.json`. Both CPU-only SNES library suites, all seven block-generator/worklist Python tests, routine-coverage tests, exact table regeneration and `git diff --check` pass. Eighteen focused marker fixtures pass on the final reduced test binary; the full default suite exercises them as well.

Preliminary candidates are retained separately: `trial1/` records the rejected broad damage-wrapper alias, while `trial2/` and `trial3/` record partial-coverage checks below the frozen 15M target. None supplies a final gate or acceptance claim. The final verification binds the corrected source and binaries independently.

`verification.json` and `artifact-sha256.json` bind the final evidence. Supplied ledger/cache size and mtime metadata remain unchanged across 1,512 files. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. This batch stops at the verified CPU-only pricing point above.

## Continuation on `codex/cost-pacing30` (main `ec6c16b6`)

### Fresh joint main baseline

Main's joint baseline is ROM **4,907 missing / 0 excess / 0 FP**, live **5,606 / 592 / 132**. Fresh clean-main builds and controls reproduce it. The default control recorded a fresh 200k ledger before edits. `target/cost-pacing30/main-provenance.json` binds that binary to main. Final evidence is in `target/cost-pacing30/final/`; earlier trials are retained separately and are superseded. Scratch drivers are in `/tmp`.

| Census | Main missing / excess / FP | Final missing / excess / FP |
| --- | ---: | ---: |
| rom | 4,907 / 0 / 0 | 4,874 / 0 / 0 |
| live | 5,606 / 592 / 132 | 5,571 / 590 / 132 |

### Complete selected bodies and clock closure

The frozen batch-22 cohort contains 2,825 iterations. `selection-ranking.json` retains the measured exclusive source/native gaps for all fourteen candidate owners. Raw dispatcher gaps include inherited bodies and existing child attribution: `$00:80B5` itself already prices all eight instructions. This group completes the selected Pengator, dash-dust, cardinal, diagonal and two bank-specific axis bodies, with eight shared source entries kept separate. Other module, sprite and ancilla bodies remain open. No existing scalar price was converted.

All new payments use `blocks.block`, `blocks.taken` or `blocks.crossing`, gated by `lag_free`, with ROM addresses in comments. `scripts/rom_block_costs.py` generates 205 new blocks covering 632 previously unmarked instructions. Bounded dispatch entries exclude inline data. Proven PHP/PLP pairs retain their saved M/X; an explicitly bounded terminal indirect jump prices only its opcode. No lag count, residual, scene threshold, multiplier or scene table supplies a price. Called bodies retain separate source frames; X movement executes the Y-array body at slot + $10.

The frozen main instruction gap falls from **200,256,886** to **183,843,904** master clocks: **16,412,982 clocks closed**, meeting the 15M target. Independently recorded new markers total **16,412,982** clocks. The existing-work/attribution remainder is **0**. `routine-cohort-before-after.json` retains every changed owner, source/native calls and exclusive totals.

| Source owner / completed work | Exclusive clocks closed (new markers) | Native lines added |
| --- | ---: | ---: |
| `$1E:8B19` Pengator control, all four states and recoil caller returns | 560,668 | 55 |
| `$00:8781` Pengator local-dispatch entry | 25,168 | shared with Pengator |
| `$07:B7C7` complete cardinal collision body | 3,709,040 | 199 |
| `$07:B64F` diagonal entry and shared tail | 3,118,090 | 15 |
| `$07:B660` complete shared direction-limit body | 540,242 | 120 shared |
| `$08:833C` remaining complete dash-dust draw loop | 125,584 | 15 |
| `$1E:FF34` bank-1E full Y/X-array body | 3,541,720 | 98 callers + shared |
| `$1D:E95D` bank-1D full Y/X-array body | 2,309,704 | 63 callers + shared |
| `$1E:FF1E` bank-1E XYZ wrapper | 187,740 | shared |
| `$1E:FF21` bank-1E XY entry | 560,254 | shared |
| `$1E:FF28` bank-1E X wrapper | 948,562 | shared |
| `$1D:E948` bank-1D XYZ wrapper | 135,000 | shared |
| `$1D:E94B` bank-1D XY entry | 203,546 | shared |
| `$1D:E952` bank-1D X wrapper | 447,664 | shared |

New markers equal the measured exclusive clock increase in every row. Line counts are gross nonblank implementation additions, including comments and caller rewiring; shared code is counted once. The movement/recoil entries share **65** lines. Total native additions are **642** lines including **12** blank lines; fixtures add **274** lines and the generated table adds **410**. `lines-added.json` retains the disjoint accounting.

The caller audit binds split AI/draw helpers and the bank-specific movement inside recoil to their ROM entries. The bank-1E recoil double return charges its second PLA/RTS to the caller; its ordinary RTS stays with the recoil frame. The first diagonal direction-limit call has its own `$07:B660` frame, while the fallthrough tail belongs to `$07:B64F`. These boundaries prevent parent/child duplication.

A ROM suffix fixture exposed a moving-floor order discrepancy at `$07:B801..B83E`: the ROM saves pre-addition Y/X bytes in `$00/$01` and uses those bytes to choose the probe order. Lagless now uses those saved values. Default/ROM-timing follows its existing path and retains its prices. This is a source-derived ordering repair, not a clock adjustment.

Fourteen `assert_rom_marker_work` fixtures execute unmodified ROM: all signed axis velocities and subpixel/wrap cases, both bank-specific X/XY/XYZ entries, real recoil movement callers, all Pengator states and early returns, the local-dispatch stack handoff, the full dust loop, and cardinal/diagonal primary-layer, collision-result and return-tail branches. Both configurations pass every marker fixture. Source prices do not establish complete source/native call equivalence; the per-owner call counts remain visible in the cohort audit.

### CPU-only verification

Both final full census gates PASS: **1,473,123 iterations per lane**, complete, unexplained zero. Every runtime hold equals P2. Missing/excess count lag NMIs; FP counts iterations. Routine and block receipts cover the frozen 2,825-row cohort, where every exclusive sum reconciles to the main-instruction counter. No whole-route per-owner aggregation is claimed. These periodically seeded logical gates do not establish continuous physical-field or exact A/V parity.

The ROM lane closes 33 previously missing iterations and introduces 0 newly missing iterations. Newly false-positive iterations: ROM 0, live 9. Full CSVs and `verification.json` retain every changed row.

All 48 measured phases across eight crystal/pendant windows remain within one frame of source. Their durations and stable room/Link/save outcomes are unchanged from main. The rupee command and PCM-onset measurements are retained separately. The final default binary matches every state/display/SRAM/PCM/input/context record and EOF of the fresh main 200k ledger. Both ordinary builds are warning-free; default lib-test compilation retains the known fixture-only dead-code warnings.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,561 | 1 | 3 |
| reduced | 2,096 | 0 | 2 |

Default retains only the known spotlight caller-return fixture failure; reduced passes entirely. The shard manifest verifies the complete disjoint test unions. Both CPU-only SNES lib suites, generator/worklist tests (including exact regeneration), routine-coverage tests and `git diff --check` pass. `verification.json` binds the final source, binaries, counters, gates, library results and phase evidence.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. All 1,512 supplied ledger/cache input files retain their size and mtime. The batch stops at this verified CPU-only pricing point.

### Joint merge of batches 30+31

Against main `b9145ee9`, all 338 caller rewirings survive. The ROM audit covers 39 shared functions and 129 wrapper calls, with no merge-induced parent/child double charge. The 632 and 405 new instructions are disjoint; the merged manifest is their exact union. Movement delegates once to the identical axis body. Only a duplicated Python test method needed removal. Evidence: `target/joint3031/verification.json`.

Inherited batch-30 issue: Stalfos-head recoil selects bank-1E movement, while `$06:DDEC` calls `$06:FF78` with bank-06 movement. This is unchanged from `211a36ac` and outside the authorized merge-fix scope.

| Full census | Main missing / excess / FP | Joint missing / excess / FP |
| --- | ---: | ---: |
| ROM | 4,812 / 0 / 0 | 4,779 / 0 / 0 |
| live | 5,508 / 589 / 131 | 5,475 / 589 / 131 |

Both gates PASS at 1,473,123 iterations, unexplained zero. All 48 crystal/pendant phases remain within one frame and unchanged from fresh main; stable outcomes match. Every default 200k ledger record and EOF matches the fresh archived-main baseline. Both ordinary builds are warning-free.

Default library: **2,579 pass / 1 known spotlight failure / 3 ignored**. Reduced: **2,114 / 0 / 2**. Both CPU-only SNES suites, all eight generator/worklist tests including exact regeneration, four routine-coverage tests and diff checks pass. Existing default test-fixture warnings remain separate from ordinary builds. All 1,512 supplied input files retain their size and mtime. These are CPU-only logical checks; the rebase and index remain untouched.

## Continuation on `codex/cost-pacing32` (main `b9145ee9`)

### Joint baseline and acceptance stop

The joint main baseline is ROM **4,812 missing / 0 excess / 0 FP**, live **5,508 / 589 / 131**. Retained fresh main controls reproduce it and both main census gates PASS. The fresh main default binary recorded the 200k control ledger. `target/cost-pacing32/main-provenance.json` binds that build; all batch evidence is under `target/cost-pacing32/`. Scratch drivers stay in `/tmp`.

**This candidate is not accepted. Both complete candidate census gates FAIL**, each with one unexplained WRAM iteration at **948559**, module `09/00`: `$0CFD` (`RUPEE_SFX_SOUND_DELAY`) is native `0`, source `1`. Both run all **1,473,123 iterations**. The target instruction coverage is reached, but the requested verified completion point is not reached. No gate or classification rule was widened and no valid price was removed to recover a lag count.

| Census | Missing before / after | Excess before / after | FP before / after |
| --- | ---: | ---: | ---: |
| rom | 4,812 / 4,724 | 0 / 0 | 0 / 0 |
| live | 5,508 / 5,425 | 589 / 594 | 131 / 133 |

At the failing iteration, main predicts/holds zero NMIs while the candidate predicts/holds one, matching the source count. The rupee counter differs only on the candidate. The retained `diagnostic-main*` and `diagnostic-final*` 948000–950000 probes reproduce this distinction. `$0D:DBFD..DC15` reads the queued sound and either increments or clears `$0CFD`; `$00:811E..812D` transfers and clears sound queues during NMI. The existing checkpoint at HUD entry only consumes queues when priced work has already crossed the field boundary. Matching the final hold count does not establish that this earlier read observed the source queue. The exact missing source interruption placement is unresolved. The existing comparator classifies this audited byte as lag-caused only when the hold counts differ or a prior mismatch is carried, so this equal-count case remains a failing gate.

### Instruction-derived candidate

The worklist was reranked by positive signed exclusive gap, without division by block count. `fresh-ranked-positive-gaps.json`, `ranked-zero-native.json` and `ownership-audit.json` retain the ranking and ownership exclusions. The batch-30 reserved self owners and existing batch-29/31 prices are unchanged. Rewired callers cite ROM call sites. All new charges and scopes are lagless-only markers, generated by `scripts/rom_block_costs.py --emit-rust`; the generator and existing scalar prices are unchanged. There are **133 new blocks / 645 ROM instructions**. No scene threshold, multiplier, fitted residual or lag-count-derived price was introduced.

Far wrappers exclude children. The bank-$1E movement wrappers exclude FF34; the far shadow wrapper excludes DC64/DC66. Projection excludes location predicates, whitening excludes D7FE, and the ancilla scan excludes its single-sprite child and double return. Partial continuations and ambiguous bank aliases remain raw. CF12 coverage is restricted to statically confirmed M8 entries. Complete entry pricing does not imply complete route caller coverage.

Independent direct new-marker receipts total **15,065,270 frozen-cohort clocks** over 2,825 iterations. A further **1,110** clocks are the existing `$06:E496` block markers newly reached through statically confirmed `$1E:8B2E` callers that formerly used the raw near helper. These are independently reconciled from before/after block receipts, not assigned as a residual. Thus the signed main-counter gap changes from **185,008,718** to **169,942,338**: **15,066,380 clocks closed in this unaccepted candidate**. Every changed routine-exclusive delta equals its direct markers plus the separately identified E496 coverage.

| Source owner / work | Frozen-cohort clocks | Whole-route clocks | Native lines added |
| --- | ---: | ---: | ---: |
| `$06:EA2D` location projection | 1,522,706 | 31,132,082 | 46 |
| `$08:8D68` ancilla sprite scan | 1,274,918 | 57,223,408 | 68 |
| `$0E:D757` whitening outer loop | 1,446,778 | 15,110,292 | 24 |
| `$06:DCEF` small-sprite drawing | 655,100 | 123,542,458 | 39 |
| `$06:FB2A` death poof drawing | 1,223,848 | 145,428,682 | 27 |
| `$07:B97C` double-layer check | 1,195,940 | 242,970,256 | 29 |
| `$05:925B` Beamos laser drawing | 1,141,544 | 24,267,016 | 21 |
| `$07:CF12` nearby tile probes | 324,064 | 17,166,044 | 16 |
| `$1A:FDB5` Mothula drawing | 1,113,780 | 35,777,740 | 23 |
| `$0E:DE49` special switch search | 921,600 | 242,212,322 | 35 |
| `$06:F1F6` point hitbox | 859,990 | 120,513,908 | 18 |
| `$0D:C0A6` Zazak drawing | 855,652 | 136,226,366 | 20 |
| `$06:E96C` bank-6 Z movement | 563,472 | 75,616,608 | 24 |
| `$1E:FF62` bank-1E Z movement | 110,778 | 14,694,314 | 18 |
| `$1E:FF21` bank-1E XY wrapper | 240,128 | 32,431,350 | 65 |
| `$1E:FF28` bank-1E X wrapper | 318,976 | 43,080,450 | 10 |
| `$06:F129` same-layer damage wrapper | 461,700 | 89,684,750 | 88 |
| `$1E:8B2E` bank-1E collision wrapper | 164,736 | 36,029,552 | 42 |
| `$06:DC54` far shadow wrapper | 387,600 | 111,465,210 | 90 |
| `$06:DBF8` far small-draw wrapper | 220,020 | 11,631,040 | 29 |
| `$06:EA22` far projection wrapper | 61,940 | 1,301,310 | 49 |
| shared Z helper | included above | included above | 12 |
| existing `$06:E496` coverage at confirmed callers | 1,110 | 9,324 | included in caller rows |

Direct new-marker route clocks total **1,607,505,158**; including newly reached E496 markers, **1,607,514,482**. Native exclusive sums reconcile to the main counter on every full-route iteration. `routine-cohort-before-after.json` retains signed owner gaps, both marker categories and their independent receipts. `lines-added.json` records **793 native added lines**, **291 fixture lines**, **266 generated table lines**, and **24 manifest additions / 1 removal**.

### Completed checks and retained failure

Twenty-one `assert_rom_marker_work` fixtures pass against the unmodified ROM on the in-repo CPU. Both ordinary builds are warning-free. Exact block-table regeneration, all seven block-generator/worklist Python tests, routine-coverage tests, both CPU-only SNES library suites and `git diff --check` pass. The final default binary matches every state/display/SRAM/PCM/input/context record and EOF in the fresh main default 200k ledger. Default/ROM-timing accounting is unchanged.

All eight crystal/pendant windows retain every measured phase within one of source. Phase, rupee-refill and stable-outcome measurements are identical before/after. `phase-verification.json`, `final-probe-metrics.json` and `final-stable-outcomes.json` retain these checks. Every candidate runtime hold equals P2, including the failing iteration; that equality is not a substitute for the census gate.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| default | 2,588 | 1 | 3 |
| reduced | 2,123 | 0 | 2 |

Default retains only the known spotlight caller-return failure; reduced passes entirely. Interrupted shards were completed by running exactly their unobserved test names. Original logs remain retained; `library-shards.json` checks the complete, disjoint original/recovery union and binds the test binaries. Lib-test-only fixture warnings remain separate from the ordinary builds.

ROM closes 88 previously missing iterations and introduces 0 newly missing iterations. Newly introduced FP iterations: ROM 0, live 12. Full metrics, category tables, classifications and shifted rows remain retained. These are periodically seeded CPU-only logical measurements; no physical-field or exact A/V acceptance is claimed.

`verification.json` explicitly records `accepted: false` and both failing gates. Source and binary identities are retained with the evidence; supplied ledger/cache metadata is unchanged across **1,512 files**. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. The batch stops with instruction prices verified and the sound-boundary census failure unresolved.

### Batch 32 replay onto `23984319`: conflict resolution and timing investigation

The paused rebase's ten conflicted files have been resolved in the working tree. Both documentation sides remain. Batch 30 retains the bank-$1D/$1E XY/X/XYZ and recoil owners; the duplicate batch-32 bank-$1E XY/X helpers are removed, and their callers use the existing bank-specific wrappers with the batch-32 ROM citations retained. The two Link collision callers retain batch 30's parent blocks and call batch 32's separately marked `$07:B97C` child. New Z, damage, collision, drawing and other nonoverlapping batch-32 work remains. No history-changing Git command was run. `git add` was attempted, but the sandbox denied creation of the worktree's `index.lock`; the index therefore still records conflicts despite the resolved file contents.

Evidence for this replay is under `target/cost-pacing32-merged/`. The merged manifest is the exact instruction union of main and the WIP commit: **128 blocks / 635 instructions** are new to main. Ten instructions shared by the original batch-32 XY/X wrappers and batch 30 keep their existing owner. Exact regeneration and all eight generator/worklist tests pass. The audit preserves all **164** main movement/recoil calls, checks **167** batch-32 source-confirmed caller groups and **40** bank-$1E collision call citations. All 21 batch-32 ROM marker fixtures pass on the rebuilt reduced test binary; four routine-coverage Python tests and diff checks pass.

The merged 948000–950000 diagnostic still fails at **948559**, `$0CFD`: native `0`, source `1`, with one native held NMI matching source. Fresh `23984319` is clean there and predicts zero holds, ending at **357,360** elapsed clocks, eight below the hardware field length. A temporary diagnostic of the merged HUD-entry checkpoint reports **272,996** main clocks / **315,756** elapsed clocks and sound queues `[11, 0]`. The diagnostic instrumentation has been removed. Consuming the queues at this checkpoint is not justified by its charged elapsed work. The eight new owners executed in the iteration reconcile independently to source instruction receipts: DBF8 760, DC54 190, DCEF 184, E96C 3,118, F129 380, F1F6 1,254, B97C 220 and DE49 800, totaling **6,906** clocks. These amounts are source-derived; no price has been removed to recover a count and the gate classifier is unchanged.

Both merged ordinary builds are warning-free. Every record and EOF of the fresh-main default 200k ledger matches. All **48** crystal/pendant phases remain within one frame of source, and stable outcomes match. Full main/merged censuses, library suites and the cold source interrupt trace were still running at this checkpoint; this is not an accepted candidate or a claim that the sound-boundary failure is fixed.

The fresh full main controls subsequently completed: both gates PASS at 1,473,123 iterations, unexplained zero. Their metrics exactly reproduce ROM **4,779 / 0 / 0** and live **5,475 / 589 / 131**. Both CPU-only SNES library suites pass (**504 passed / 7 ignored** each). A second audit confirms unchanged semantic movement/recoil call multiplicity in all **62** changed functions containing those operations. The source trace and merged census/library checks remain pending; `checkpoint-status.json` records this intermediate state explicitly.

Both merged full censuses subsequently completed all **1,473,123** iterations. Each gate FAILS with exactly **one unexplained iteration**, 948559 `$0CFD`; there are no additional unexplained cases. Every runtime hold equals P2. ROM missing/excess/FP changes from **4,779 / 0 / 0** to **4,681 / 0 / 0**; live changes from **5,475 / 589 / 131** to **5,382 / 594 / 133**. The reduced library suite completes with **2,135 pass / 0 fail / 2 ignored**. The default suite and cold interruption trace are still pending. All 1,512 supplied input files retain their size and mtime. These results do not accept the candidate.

#### Cold CPU interruption witness and unresolved scheduling

The cold in-repo CPU run completed **1,021,552 host calls** with the unchanged ROM/SRAM/input and `ZELDA3_LEDGER_PROFILE=948559-948559`, block profiling, and thread/step tracing over hosts 1021548–1021551. `948559-source-interruption.json` and `source-trace.log` retain the witness. The accepted lag NMI interrupts at **`$06:E470`**, after `$06:E46E` (`LDA $02`) and before `$06:E470` (`STA $0FA9`), in the shared **OAM coordinate preparation** body at `$06:E41E`. It is not an interruption within the new batch-32 wrappers or HUD.

| Source event | Absolute master-clock observation |
| --- | ---: |
| NMI entry completed; interrupted PC `$06:E470` | 365,067,186,918 |
| `$00:811E` queued sound read begins | 365,067,187,632 |
| `$00:812A` sound-1 queue clear completes | 365,067,187,788 |
| `$00:812D` sound-2 queue clear completes | 365,067,187,820 |
| `$0D:DB75` HUD entry begins | 365,067,254,958 |
| `$0D:DBFD` queued-sound read begins | 365,067,255,402 |
| `$0D:DC05` delay increment completes | 365,067,255,528 |

The thread trace's NMI timestamp is observed **after interrupt entry**, not its acceptance start. The interrupted PC itself comes from the CPU's accepted-interrupt receipt. The source executes the increment path and does not execute `$0D:DC15`; the sound-1 clear precedes the HUD read by **67,614** master clocks. The new cold per-instruction profile independently reproduces all eight new-owner totals (6,906 clocks), and its chronological trace proves that every executed new instruction precedes HUD entry.

No safe general charged-clock scheduling fix has been established at this stop. The merged native HUD checkpoint is only at 315,756 elapsed clocks, below the 357,368-clock field boundary, whereas the source interruption has already occurred in preceding coordinate work. The complete source main contains **403,292** instruction clocks versus native **310,832**, a **92,460-clock** coverage gap; a correct final hold count does not locate the interrupt within this incompletely priced prefix. Clearing early based on the completed hold count would assert an unproved placement. All new prices active in the failing iteration have proven source amounts and pre-HUD positions, so no batch-32 price was withdrawn as “unproven” merely to restore zero holds. The checkpoint rule, comparator/classifier, default/ROM-timing path and existing prices remain unchanged. **The candidate remains unaccepted and the requested general timing repair remains unfinished.**

#### Verified stop

The default library suite subsequently completed with **2,600 pass / 1 known spotlight failure / 3 ignored**; reduced is **2,135 / 0 / 2**. The shard manifests record complete, disjoint test unions and binary identities. An initial library invocation without the required asset environment was cancelled and retained as non-authoritative; the complete suites supply the asset path and run one test thread per shard. The reduced full-suite binary includes the temporary optional checkpoint diagnostic with its environment flag unset; all 21 marker fixtures were additionally rerun on the final reduced test binary after that diagnostic was removed. No pricing or scheduling behavior changed between those test builds.

Final `verification.json`, `artifact-sha256.json`, the working-tree diff and source hashes bind this **unaccepted** stop. Both complete merged gates still fail at the single audited equal-hold sound case; the precise source interruption is now established, but a general charged-clock repair is not. No valid ROM price was deleted, no gate was widened, and no iteration/module/byte exception was introduced. Conflict markers are absent from the file contents; Git still reports ten unmerged paths because index staging was denied. The rebase remains paused. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt or history-changing Git command was used.

### Batch 32 proportional checkpoint hypothesis on `e45f8487`: rejected and reverted

Tested the requested proportional projection at every in-main elapsed-time decision, without changing `native_finish_main_cost`, `native_cost_components`, the gate classifier or any instruction price. **The hypothesis does not fix 948559 and has been reverted.** Evidence is under `target/cost-pacing32-projection/`; the rejected implementation is retained as `hypothesis.patch`, with its warning-free reduced build and binary, six replay logs/predictor CSVs/classification traces, commands and `rejection.json`. Scratch orchestration stayed in `/tmp`.

The checkpoint audit found these sites (line numbers in the restored source):

| File:line | Placement decision |
| --- | --- |
| `crates/zelda3/src/hud.rs:544` → `zelda_rtl/rtl_native_pacing.rs:271` | HUD-entry sound-queue consumption; the sole sound-queue checkpoint caller |
| `crates/zelda3/src/zelda_rtl/rtl_native_pacing.rs:149` | Worker activation's IRQ/NMI phase and stack orientation |
| `crates/zelda3/src/zelda_rtl/rtl_native_pacing.rs:257` | Raster polling barrier's elapsed phase, including field wrap |
| `crates/zelda3/src/zelda_rtl/rtl_native_pacing.rs:385` | Accepted holds and residual phase before masked song upload |

`native_checkpoint_worker_ownership` at lines 136–143 only records charged work and ownership; it does not decide whether an NMI crossed. The completed-call hold calculation and diagnostic shortfall inverse are not in-main placement checkpoints. There is no second queue-consumption elapsed check elsewhere.

A material premise correction: the opening document's proportional P2 formula was superseded by the repeated-lag-handler and segmented-ownership sections above. At this HEAD, all four checkpoints already call `native_cost_elapsed`; it and completed-call accounting share `native_cost_components`. The HUD's retained 315,756 clocks are **already refresh/handler-adjusted P2 elapsed**, not raw main work. Completed-call accounting additionally admits the validated HDMA bus program after accounting closes; that admission is explicitly disabled for mid-call producers. A shared function therefore does not imply that an earlier incomplete prefix has crossed the same boundary as the completed call.

For the experiment, one helper evaluated exactly `(C + 40*ceil(C/1324))*F/(F-H-W)`, with existing entry-handler/entry-worker caps and accumulated raster waits, at all four checkpoint paths. The completed-call count rule remained untouched. Using the retained HUD-prefix witness (`C=272,996`, `H=34,480`, `W=0`, no raster wait), refresh is 8,280 and the proposed elapsed is **311,312**, still below **357,368**. Existing elapsed there is `272,996 + 8,280 + 34,480 = 315,756`. Neither model clears the queue before HUD. This arithmetic explains the reproduced failure without an iteration exception or an early clear based on the final count.

Fresh diagnostic replays covered **948000–950000 inclusive: 2,001 rows** for each of ROM/live and each of retained main, unchanged batch-32 and trial binaries. For both lanes, trial predictor CSVs, classification traces and replay logs are **byte-identical** to unchanged batch 32. Main has no unexplained row; unchanged batch 32 and trial each retain exactly one, 948559. The following byte receipts are identical in both lanes:

| Iteration | Source / main / batch-32 / trial holds | Trial `$0CFD` native / source | Classification |
| --- | --- | --- | --- |
| 948559 | 1 / 0 / 1 / 1 | 0 / 1 | unexplained |
| 948560 | 1 / 0 / 0 / 0 | 1 / 2 | carried lag |
| 948561 | 1 / 0 / 0 / 0 | 2 / 3 | carried lag |
| 948562 | 0 / 0 / 0 / 0 | 3 / 4 | carried lag |
| 948563 | 1 / 0 / 0 / 0 | 4 / 5 | carried lag |

Within this diagnostic window, missing/excess/FP is **40/0/0** for main and **37/0/0** for both unchanged batch 32 and trial, in each lane. At 948559, completed elapsed is main **357,360**, batch 32/trial **367,496**; final charged main work is **303,926** versus **310,832**. Full per-row receipts are retained. No other diagnostic row changes, and no per-row variant was attempted.

The failed 948559 prerequisite triggers the requested revert. No new full census, phase, default-200k or library acceptance is claimed for this rejected trial; those expensive acceptance runs were not repeated. The retained full-route comparisons remain the earlier unaccepted batch-32 results: main ROM **4,779/0/0**, live **5,475/589/131**; batch 32 ROM **4,681/0/0**, live **5,382/594/133**. All production source is restored byte-for-byte to `e45f8487`; only this documentation addition remains. Default/ROM-timing, prices and classifier are unchanged. No GPU execution/test, `--video`, cached-av, Snes9x, oracle-cache write, cargo fmt, commit/push or one-off repository script was used.

After reverting, the reduced ordinary binary was rebuilt warning-free and both 948000–950000 lanes rerun: their logs, predictor CSVs and classification traces are byte-identical to the unchanged HEAD controls. All **1,512** supplied ledger/cache files retain their size and mtime. `rejection.json` binds these restored-source checks, the restored binary hash and retained artifact hashes. `git diff --check` passes; branch and HEAD remain `codex/cost-pacing32` / `e45f8487`.

## General pre-HUD body prices after batch 32 (`50c43d34`, main `23984319`)

This continuation keeps all batch-32 prices and repairs 948559 by pricing previously missing executed bodies. The elapsed/checkpoint rule and census classifier are unchanged. Final evidence is under `target/cost-pacing35/final2/`; the parent directory retains fresh main controls and the first unaccepted group. `preliminary/` contains superseded fixture/build/diagnostic or prefix runs and supplies no final acceptance. Scratch orchestration remained in `/tmp`.

### Prefix owner ranking and ownership audit

The retained cold source profile and chronological trace establish which owners execute before the accepted NMI at `$06:E470`; HUD starts later at `$0D:DB75`. The table compares their **full-iteration exclusive** source/native amounts at 948559, with native taken from the unchanged merged batch-32 profile. It is a ranking of observed owners, not an instruction-level attribution of the native prefix. All 71 positive raw rows, in descending deficit order, are listed in `target/cost-pacing35/pre-hud-owner-deficits.csv` and the final `source-pricing-audit.json`.

| Source owner | Source exclusive | Native exclusive | Raw deficit |
| --- | ---: | ---: | ---: |
| `$06:D22F` absorbable selection and drawing tails | 19,862 | 3,016 | 16,846 |
| `$1D:F65F` dynamic spawning | 10,104 | 0 | 10,104 |
| `$1D:F94B` talking-tree dispatch and selected tails | 9,208 | 0 | 9,208 |
| `$1D:CB30` thief item spilling | 8,916 | 0 | 8,916 |
| `$0D:B818` property loading | 7,458 | 0 | 7,458 |
| `$06:84E2` sprite dispatch and inherited tails | 15,150 | 8,288 | 6,862 |
| `$06:E4AB` tile collision | 6,128 | 1,312 | 4,816 |
| `$06:EAE8` below-Link predicate | 3,082 | 0 | 3,082 |
| `$07:E245` Link velocity | 4,296 | 1,906 | 2,390 |
| `$07:807F` Link control | 2,516 | 198 | 2,318 |

Raw deficits are not all billable: tail jumps retain their caller in the source profiler, while native coordinate, shadow, inactivity and drawing helpers can already have separate owners. In particular, the absorbable row already receives separately charged shadow work. No second shadow/coordinate body is added. The talking-tree eye's complete `$1D:FB0A..FB85` tail contributes 4,972 of the F94B raw gap; the remaining tree dispatch/mouth work is not priced by this change. Other uncompleted owners remain open.

### Complete selected bodies and interruption ordering

The first six bodies fixed 948559, but both full gates exposed four equal-hold rupee-delay rows: 501743/501744 and 953083/953084. Those valid prices remain. The next group completes the still-missing collision-layer control, direction probes and their one-pixel step children, relative-position predicates, phasing, bounce control and absorbable control. The final 500000–502000 and 948000–954000 diagnostics are **8,002 clean rows per lane**, covering all three originating cases and their following rows. The first group's failed full receipts are retained as unaccepted evidence; no classifier or price was weakened to make them pass.

`scripts/rom_block_costs.py --emit-rust` supplies **165 new blocks describing 589 ROM instructions**. All **460** existing generated rows, including every batch-32 price, remain byte-identical. Every runtime addition is a lagless-only marker with a source-address comment. Actual slot-search passes, branch outcomes, clipping and ROM-table page crossings select work. No lag count, fitted residual, scene/iteration threshold, multiplier, exception or scene table supplies a price.

Called RNG, reset, coordinate, shadow, drawing, movement, projection, property and conveyor bodies retain separate ownership. The probe's two optional JSRs each price one real step body; the final step falls through in its parent. Phasing's double return closes its own frame at the first PLA; the ancestor's remaining PLA/RTS is not charged to this body. Other already-priced drawing paths are unchanged. Partial continuations retain their existing accounting.

At 948559, independent new-marker amounts exactly match the cold source instruction receipts:

| Body / selected shared tail | Source and native new clocks |
| --- | ---: |
| `$1D:F65F dynamic spawning` | 10,104 |
| `$1D:CB30 thief spilling` | 8,916 |
| `$0D:B818 property loading` | 7,458 |
| `$06:DD40 thin/tall drawing` | 6,530 |
| `$1D:FB0A tree eye` | 4,972 |
| `$06:E4DB layer control` | 4,816 |
| `$06:D051 absorbable control` | 3,396 |
| `$06:EAE8 below-Link predicate` | 3,082 |
| `$06:D2FA numbered drawing` | 2,516 |
| `$06:D0ED phasing` | 2,160 |
| `$06:EAD1 right-of-Link predicate` | 1,626 |
| `$06:D9C0 bounce control` | 1,092 |
| `$06:E5EE vertical probe` | 770 |
| `$06:E5B8 horizontal probe` | 550 |
| `$06:DC4C..DC65 selected drawing tails` | 446 |
| **Total new pre-HUD instructions** | **58,434** |

Every selected family has an invocation before the source interruption at `$06:E470`; all its executed new instructions finish before source HUD entry at `$0D:DB75`. Complete bodies can extend past the interruption itself. The old native HUD prefix was 272,996 main clocks. Its reconstructed new prefix is **331,430**; ordinary handler 34,480 plus refresh 10,040 gives **375,950**, crossing the **357,368** field boundary. The unchanged repeated-handler rule adds 3,030, giving **378,980 P2 checkpoint clocks**. This uses the existing segmented rule, not the rejected proportional projection or the completed-call hold count.

The full main counter changes from 310,832 to **369,386**: 58,434 new pre-HUD clocks plus **120 existing HUD clocks newly reached on the correct increment path**. The latter is not a new price. Source main remains 403,292, so this is still incomplete instruction coverage; it establishes the required queue/HUD ordering and census result, not exact physical interruption placement or A/V parity.

Twelve `assert_rom_marker_work` fixtures execute **408 unmodified-ROM cases** on the in-repo CPU. They check complete selected bodies, including signed/carry boundaries, slot and resource exhaustion, property crossings, clipping, phasing returns, collision guards and both probes' nested step frames. The thief fixture supplies the unmodified CPU's PPU-dependent RNG answers to native so both take the same branches. The absorbable control fixture supplies the real current-slot byte used by source callers to restore X. The existing absorbable selection fixture includes the newly priced drawing tails without dropping its prior cases. No ROM bytes or census acceptance predicates are changed.

### Per-routine clock closure

The frozen cohort has **2,825 iterations**; the route has **1,473,123**. Values below are independently recorded **new marker work**, with internal called-step frames separate from their parent probes. Shared drawing-tail markers are listed once. They are not allocations of the source/native residual or claims of equivalent call sequences.

| Body / source entry | Frozen-cohort clocks | Whole-route clocks |
| --- | ---: | ---: |
| `$06:E4DB` layer control | 6,492,844 | 849,587,648 |
| `$06:D2FA` numbered | 1,777,928 | 125,623,656 |
| `$1D:FB0A` tree eye | 1,062,628 | 40,296,284 |
| `$06:EAE8` below predicate | 1,005,098 | 162,787,856 |
| `$06:E5EE` vertical probe | 995,734 | 83,071,074 |
| `$06:EAD1` right predicate | 854,290 | 183,594,692 |
| `$06:E5B8` horizontal probe | 808,000 | 92,617,940 |
| `$06:DD40` thin | 674,006 | 82,737,860 |
| `$06:D051` absorbable control | 658,204 | 65,787,924 |
| `$1D:F65F` spawn | 654,174 | 32,903,590 |
| `$0D:B818` properties | 594,540 | 62,192,074 |
| `$06:D0ED` phasing | 414,248 | 69,135,748 |
| `$06:D9C0` bounce | 214,632 | 23,359,474 |
| `$06:DC4C/DC64` shared drawing tail | 53,190 | 5,953,712 |
| `$1D:CB30` thief | 34,576 | 42,014 |
| `$06:E5DA` horizontal called step | 1,560 | 334,360 |
| `$06:E610` vertical called step | 3,120 | 290,680 |
| **Total** | **16,298,772** | **1,880,316,586** |

The fresh pinned-HEAD cohort control reproduces the retained control's exclusive totals for every owner. The signed cohort instruction gap changes from **156,086,780** to **137,789,456**. Its **18,297,324** main-counter increase comprises the **16,298,772** new markers plus separately measured existing-work changes: **72,726 existing-marker clocks** and **1,925,826 scalar clocks**. These existing-work/call-path differences are not assigned as new prices. `routine-cohort-before-after.json` and `existing-work-accounting.json` retain every changed owner and the marker/scalar partition. The bounded HEAD controls are diagnostics; full-route acceptance comes from the gates below. Exclusive sums reconcile on every profiled route iteration; all 2,825 cohort totals also equal their corresponding predictor main counters.

### CPU-only verification

Fresh main `23984319` was built from its pinned archive and recorded a fresh default 200k ledger. Both main controls and both final candidate census gates PASS: **1,473,123 iterations each, unexplained zero**. Every final runtime hold equals P2. Missing/excess count lag NMIs; FP counts zero-source-lag iterations.

| Lane | Main missing / excess / FP | Final missing / excess / FP |
| --- | ---: | ---: |
| ROM | 4,779 / 0 / 0 | **4,584 / 0 / 0** |
| Live SPC | 5,475 / 589 / 131 | **5,284 / 593 / 133** |

ROM restores 195 additional missing lag frames; live restores 191, with four additional excess frames and two additional FP iterations in the totals. No valid instruction price is removed to suppress these counts. These are periodically seeded logical gates, not a continuous quest or exact A/V proof.

Every state/display/SRAM/PCM/input/context record and EOF of the fresh-main **default 200k** ledger matches. Default/ROM-timing accounting is unchanged. Both final ordinary builds are warning-free; existing lib-test-only warnings are separate. All **48** crystal/pendant phases remain within one frame of source and their stable outcomes match main.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| Default | 2,612 | 1 known spotlight caller-return failure | 3 |
| Reduced | 2,147 | 0 | 2 |

The complete, disjoint shard manifests bind the tested binaries and expected failure. Both CPU-only SNES suites pass (**504 / 0 / 7** each); all eight block-generator/worklist tests, four routine-coverage tests, exact table regeneration and `git diff --check` pass. All **1,512** supplied ledger/cache files retain their size and mtime. `verification.json`, source/binary hashes, command receipts, full census predictors/classifications and `artifact-sha256.json` bind the final point.

No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit/push or one-off repository script was used. Branch/HEAD remain `codex/cost-pacing32` / `50c43d34`. This stops at the verified CPU-only pricing point above.

## Verified partial continuation on `codex/cost-pacing36` (main `8b0f4ee2`)

Main's joint baseline is ROM **4,584 / 0 / 0**, live **5,284 / 593 / 133** (missing / excess / FP). Fresh pinned-main controls reproduce it. This is a **verified partial stop: the 15M frozen-cohort target remains unfinished**. The four selected bodies add **3,263,068 new marker clocks**; the independently measured signed cohort gap closes by **3,263,068 clocks**. Evidence is in `target/cost-pacing36/`; scratch orchestration stays in `/tmp`.

### Fresh ranking and recovered work

The interrupted run's two player-body edits were retained after instruction/caller review and rebuilt ROM fixtures. Its retained executable could not start (signal 9), so ordinary controls were rebuilt from a pinned `git archive 8b0f4ee2`; the default control recorded a fresh 200k ledger. The final worklist was recomputed with `scripts/rom_block_worklist.py` against the fresh main cohort block receipts and the unchanged frozen batch-22/source inputs. The cohort sets agree exactly: **2,825 iterations**. `ranked-owners.json` ranks **positive signed exclusive owner gaps**, not individual blocks. Fresh main's signed gap is **139,800,230 clocks**.

The leading `$06:DC64` shadow and `$1E:FE78` / `$1D:E8A2` inactive rows alias existing shared scalar owners, so no second body price was added. The selected owners were all zero-native in the fresh control. Larger uncompleted owners remain open. The two player bodies execute at their complete native call sites; no claim of exact physical NMI placement is made from a completed hold count.

### Whole executed bodies

The generator supplies **33 new blocks / 181 ROM instructions**; all **625** existing generated rows remain byte-identical. Runtime changes are lagless-only markers with source-address comments. The actual branches and table choices select instruction work. No scalar conversion, fitted lag residual, threshold, multiplier, scene table, checkpoint-rule change or classifier change was introduced.

- `$07:81CF JSR $9AC2`: sword cooldown owns its complete `$07:9AC2..9AE5` body; sword-swing/control children retain their ownership. Resumed children bypass this new complete-body entry.
- `$07:B657` and `$07:B848 JSR $B9B3`: moving-background velocity owns `$07:B9B3..B9F6`. Parent markers already own only the call opcode. Partial continuations keep the unmarked semantic entry.
- `$0D:A635 JSR $AB6E`: complete weapon-VRAM selection at `$0D:AB6E..ABC9`, with m16/x16 entry. The parent's existing scalar price includes only its call instruction.
- `$0D:A6DD JSR $ACD5`: complete sword-sparkle body at `$0D:ACD5..AD81`, m16/x16 entry. The separately priced `$0D:AFC0` child remains unchanged. The post-call `$0D:AD5D` suffix is included in full.

Four `assert_rom_marker_work` fixtures pass **2,275 unmodified-ROM cases** on the in-repo CPU: signed cooldown branches; collision/direction and coordinate boundaries; all 511 weapon poses, with held-item combinations inside the alternate table's ten-entry domain; and sword, handler/speed, facing, palette and sparkle branches. The first expanded fixture pass caught the omitted sparkle post-call block and an invalid alternate-table test combination; `marker36-tests-rebuilt.log` is superseded by `marker36-tests-final.log` and the final complete suites. No ROM bytes were patched.

| Source owner / body | Frozen-cohort new marker clocks | Native lines added |
| --- | ---: | ---: |
| `$07:B9B3` moving-background velocity | 975,264 | 21 |
| `$0D:ACD5` sword sparkle | 840,604 | 38 |
| `$07:9AC2` sword cooldown | 750,660 | 20 |
| `$0D:AB6E` weapon VRAM selection | 696,540 | 13 |
| **Total** | **3,263,068** | **92** |

Lines are gross native additions including source comments and caller rewiring, excluding **127 fixture lines**, **66 generated-table lines**, the manifest and documentation. `lines-added.json` records every counted native line. Cohort exclusive sums reconcile on every profiled iteration. Both bounded profile replays retain the same raw first WRAM divergence at 884327 (`$0674`, module `10/01`); they are diagnostics, not acceptance gates. Full-route acceptance comes from the separate complete census gates below. The source/native signed gap changes from **139,800,230** to **136,537,162**. `routine-cohort-before-after.json` separates the new markers from every existing-work or attribution delta; those deltas are not new instruction prices. The native new-marker amounts exceed the selected source-exclusive totals by 388 cooldown clocks and 2,446 sparkle clocks; the other two totals match. `selected-source-comparison.json` preserves these differences. These are executed native-work receipts, not proof of equivalent call sequences or exact interruption placement. No whole-route per-owner aggregation is claimed.

### Verification

Both fresh main controls and both final full census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero. Every predictor row has runtime holds equal to P2. Missing/excess count lag NMIs; FP counts zero-source-lag iterations. These are periodically seeded CPU logical gates, not continuous-route or exact A/V proof.

| Lane | Main missing / excess / FP | Final missing / excess / FP |
| --- | ---: | ---: |
| ROM | 4,584 / 0 / 0 | **4,558 / 0 / 0** |
| Live SPC | 5,284 / 593 / 133 | **5,258 / 593 / 133** |

All **48 crystal/pendant phases** remain within one frame of source; stable outcomes match fresh main. Every state/display/SRAM/PCM/input/context record and EOF of the fresh-main **default 200k** ledger matches. Default/ROM-timing accounting is unchanged. Both final ordinary builds are warning-free; existing lib-test-only dead-code warnings remain separate.

| Zelda library | Pass | Fail | Ignored |
| --- | ---: | ---: | ---: |
| Default | 2,616 | 1 known spotlight caller-return failure | 3 |
| Reduced | 2,151 | 0 | 2 |

Complete, disjoint shard manifests bind the test binaries and observed results. Both CPU-only SNES suites pass **504 / 0 / 7**; all eight block-generator/worklist tests, four routine-coverage tests, exact table regeneration and `git diff --check` pass. All **1,512** supplied ledger/cache files retain size and mtime. `verification.json`, source/binary hashes, command receipts and `artifact-sha256.json` bind this stop. Verified full predictors and raw profiles were deleted after summaries; `pruned-artifacts.json` retains their hashes. Compressed classifications and phase traces remain, with batch evidence below 3 GB.

One accidental `--help` probe entered interactive GPU initialization and failed with no suitable adapter; no rendering completed (`rebuilt-help.log`, `recovery.json`). All verification above uses CPU entry points. No `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Branch/HEAD remain `codex/cost-pacing36` / `8b0f4ee2`. The **15M pricing target is still open**.

## Verified batch 37 on `codex/cost-pacing37` (main `c751ec0b`)

**12,946,080 frozen-cohort clocks closed; the 12M target is met.** New marker work and the signed-gap reduction agree exactly: **136,537,162 → 123,591,082** across the unchanged **2,825-iteration** cohort. Batch 36's retained routine ranking and final baseline were reused; no main controls were rerun. Evidence is in `target/cost-pacing37/`.

The larger audited single-draw, shadow and inactive gaps already contain separately charged bodies; they receive no second price. This batch completes admissible garnish, player-control, damage, collision and drawing bodies at confirmed ROM callers. Larger partially priced owners remain open. Called coordinate, drawing, shadow, hitbox, projection and movement bodies retain their existing ownership; resumed fragments retain their previous entries.

The generator adds **156 blocks / 578 ROM instructions**; all **658 existing rows remain byte-identical**. Y-item early exits also newly execute the existing `$07:9AE5` RTS marker **291 times / 12,222 clocks**. That shared-row reuse is included in the total above; there is **no scalar or attribution delta**. `shared-return-accounting.json` records this reuse. All runtime prices are lagless-only ROM markers selected by actual branches and executed loops, with source comments and statically confirmed callers. No scalar conversion, fitted lag residual, scene threshold, multiplier, scene table or checkpoint/classifier change was introduced. The two bounded flag-normalization joins price only width-independent SEP instructions and are covered by ROM fixtures.

Sixteen `assert_rom_marker_work` fixtures pass **114 unmodified-ROM cases**, including early exits, shield tails, coordinate rejection, slot scans and both probe lanes. Shared garnish tails are exercised by their complete caller fixtures. Per-body figures below are native **new marker executions**, including separately listed shared tails once; they do not assert equivalent source/native call sequences or exact physical NMI placement. Lines are gross native additions, including caller rewiring/comments, excluding fixtures, generated rows and the manifest.

| Body / source entry | Frozen clocks closed | Native lines added |
| --- | ---: | ---: |
| `$09:B429` Lightning trail | 2,203,392 | 9 |
| `$06:F15C` Damage/shield body | 1,541,944 | 66 |
| `$09:B4C6` Kholdstare trail | 1,237,492 | 5 |
| `$07:9BAA` A-press control | 1,050,018 | 88 |
| `$07:9B0E` Y-item selector | 1,049,894 | 75 |
| `$0D:C26E` Tile-solidity probe | 733,608 | 14 |
| `$1D:D1AF` Large shadow wrapper/body | 689,430 | 16 |
| `$0D:BD20` Babusu draw | 665,952 | 8 |
| `$09:B459` Shared garnish collision tail | 664,034 | 15 |
| `$06:F645` Fast hitbox | 660,284 | 24 |
| `$06:C277` Statue collision scan | 627,316 | 34 |
| `$06:A3DC` Red Bari draw control | 577,660 | 5 |
| `$09:B70C` Shared big-OAM finalization | 537,516 | 3 |
| `$06:83EA` Far hitbox wrapper | 498,750 | 10 |
| `$06:F154` Same-layer entry | 194,400 | 12 |
| `$07:983A` Swimming-reset wrapper | 7,722 | 11 |
| `$09:B306` Bat-flame control | 5,440 | 8 |
| `$09:B3EE` Cannon-smoke control | 1,228 | 4 |
| **Total** | **12,946,080** | **407** |

Both final full census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero; every runtime hold equals P2. These are periodically seeded CPU logical gates, not continuous-route or exact A/V proof. Missing/excess count lag NMIs; FP counts zero-source-lag iterations.

| Lane | Retained main missing / excess / FP | Batch 37 missing / excess / FP |
| --- | ---: | ---: |
| ROM | 4,558 / 0 / 0 | **4,488 / 0 / 0** |
| Live SPC | 5,258 / 593 / 133 | **5,184 / 589 / 131** |

All **48 crystal/pendant phases** remain within one frame; stable outcomes match the retained main trace. The retained **default 200k ledger matches all records and EOF**. Default/ROM-timing accounting is unchanged. Both ordinary builds are warning-free; existing lib-test-only warnings remain separate.

- Default Zelda library: **2,632 pass / 1 fail / 3 ignored** (only the known spotlight caller-return failure).
- Reduced Zelda library: **2,167 pass / 0 fail / 2 ignored**.
- Both CPU-only SNES suites: **504 / 0 / 7**. All eight generator/worklist tests, four routine-coverage tests, exact table regeneration and `git diff --check` pass.

Complete disjoint library manifests, source/binary hashes, command receipts, per-routine clock/line summaries and `verification.json` bind this point. All **1,512** supplied ledger/cache files retain size and mtime. Raw predictors/profiles were deleted after summaries; hashes remain in the pruning manifests. Batch evidence remains below **3 GB**. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used.

## Verified batch 38 on `codex/cost-pacing38` (main `24af99e1`)

**12,232,874 frozen-cohort clocks closed; the 12M target is met.** New marker work equals the signed-gap reduction: **123,591,082 → 111,358,208**, on the unchanged **2,825 iterations**. Batch 37's retained routine data and baseline were reused; no main controls were rerun. Evidence is in `target/cost-pacing38/`.

The updated positive signed-owner ranking was audited from largest to smallest. Already-priced single-draw, velocity, layer, multi-draw, shadow and inactive/prep aliases receive no second price. Eligible work completes uncharged bodies and statically confirmed callers inside the larger dispatch owners. Small rows below are their required complete wrappers, children and shared tails. The blanket's existing explicit XY scalar charge is preserved; its raw writer is excluded. The spike/statue Y-predicate mismatch and drawing loops needing a data-bank pricing audit remain open.

The generator adds **116 blocks / 495 ROM instructions**; all **814 existing rows remain byte-identical**. Prices are lagless-only markers selected by actual branches and loops, with ROM call-site comments. Existing movement/damage marker bodies are reused. No scalar conversion, fitted lag residual, scene threshold, multiplier, scene table or classifier/checkpoint change was introduced. Complete diagonal entry is marked; checkpoint suffixes remain unmarked. Palette tail `$0E:D8F8` prices only width-independent SEP/RTL work at its two incoming index widths.

Thirteen new `assert_rom_marker_work` fixtures pass **66 unmodified-ROM cases**, covering fourteen entries; reused bodies retain their existing ROM fixtures. Values are **new native marker executions**, not proof of equivalent call sequences or physical NMI placement. Exclusive sums reconcile on every cohort iteration; there is **no scalar or attribution delta**. Lines are gross native additions, including comments/rewiring, excluding fixtures, generated rows, manifest and documentation. Zero lines means an existing marked child; caller lines are counted at the entry. The four damage entries share one implementation, listed once. Horizontal slope entry `$07:B969` is fixture-covered and contributes zero new frozen clocks.

| Body / source entry | Frozen clocks closed | Native lines added |
| --- | ---: | ---: |
| `$09:F0CB` Thrown-item debris | 1,827,558 | 30 |
| `$06:F121` Damage-to-Link far wrapper | 1,739,070 | 22 |
| `$06:F2AA` Damage-from-Link far wrapper | 1,322,824 | 4 |
| `$07:CCAB` Diagonal kickback | 920,002 | 46 |
| `$08:F6E1` Ancilla XY publication | 857,676 | 70 |
| `$1E:C267` Bank-1E damage pair | 709,484 | 70 |
| `$1E:D978` Agahnim drawing | 665,424 | 31 |
| `$05:FA00` Bank-5 axis body | 639,712 | 0 |
| `$08:CF35` Tower sparkle scan | 619,856 | 19 |
| `$09:B6E1` Mothula trail | 590,100 | 3 |
| `$0E:D83A` Palette flash restore | 520,312 | 54 |
| `$09:B49E` Babusu flash | 315,666 | 4 |
| `$1D:C211` Bank-1D damage pair | 268,588 | 40 |
| `$09:B70C` Shared big-OAM finalization | 266,862 | 2 |
| `$06:E92C` XY movement / Y tail | 197,138 | 12 |
| `$06:E93E` Y movement body | 172,432 | 0 |
| `$07:B956` Ordered slope checks | 129,396 | 20 |
| `$05:F9F4` Bank-5 X wrapper | 112,674 | 0 |
| `$06:D88B` Bank-6 damage pair | 99,562 | 24 |
| `$1E:C26B` Bank-1E damage-to-Link wrapper | 86,216 | 15 |
| `$05:F9ED` Bank-5 XY movement | 84,822 | 12 |
| `$06:E932` X movement wrapper | 66,750 | 0 |
| `$05:AB93` Bank-5 damage pair | 20,750 | 46 |
| Shared four-entry damage implementation | 0 | 21 |
| **Total** | **12,232,874** | **545** |

Both final full census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero; every runtime hold equals P2. Missing/excess count lag NMIs; FP counts zero-source-lag iterations. These are periodically seeded CPU logical gates, not continuous-route or exact A/V proof.

| Lane | Retained main missing / excess / FP | Batch 38 missing / excess / FP |
| --- | ---: | ---: |
| ROM | 4,488 / 0 / 0 | **4,433 / 0 / 0** |
| Live SPC | 5,184 / 589 / 131 | **5,129 / 589 / 131** |

All **48 crystal/pendant phases** remain within one frame; stable outcomes match batch 37. The retained **default 200k ledger matches all records and EOF**. Default/ROM-timing accounting is unchanged. Both ordinary builds are warning-free; existing lib-test-only warnings remain separate.

- Default Zelda library: **2,645 pass / 1 known spotlight caller-return failure / 3 ignored**.
- Reduced Zelda library: **2,180 pass / 0 fail / 2 ignored**.
- Both CPU-only SNES suites: **504 / 0 / 7**. All eight generator/worklist tests, four routine-coverage tests, exact table regeneration and `git diff --check` pass.

Complete disjoint library manifests, ROM/source/binary hashes, call-site audit, per-routine clock/line summaries and `verification.json` bind this point. All **1,512** supplied ledger/cache files retain size and mtime. Raw profiles/predictors were deleted after summaries; hashes remain in the pruning manifest. Evidence remains below **3 GB**. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used.

## Verified batch 39 on `codex/cost-pacing39` (main `31633b01`)

**13,994,280 frozen-cohort clocks closed; the 12M target is met.** New marker work equals the signed-gap reduction: **111,358,208 → 97,363,928**, on the unchanged **2,825 iterations**. The ranking reconciles to batch 38's retained cohort; no main controls were rerun. Evidence is in `target/cost-pacing39/`.

The owner audit preserves already-priced drawing, coordinate, velocity, layer, inactive and dispatch aliases. Selected bodies complete ground control, tile responses, Zazak, class-2 collision and boss drawing. Larger uncompleted owners remain open. Ground-layer ancilla collision's tile-3 upper-layer predicate differs from the ROM and remains unpriced. Complete entries are marked; checkpoint suffixes retain their raw entries. Called bodies keep separate ownership.

The generator adds **300 blocks / 1,179 ROM instructions**; all **930 existing rows remain byte-identical**. Its explicit data-bank option prices proven absolute register accesses in the two hardware-multiply draws; unknown/indexed register-window costs still fail shut. Fixtures cover the I/O bus costs, table crossing, signed quadrants, drawing loops, returns and control branches. Nine `assert_rom_marker_work` fixtures pass **66 unmodified-ROM cases**. Local dispatch retains the source's stack-pop ownership: `$00:8781` belongs to its child; `$8784..879B` belongs to the caller. No scalar conversion, lag fitting, scene threshold/multiplier/table, classifier or checkpoint change was introduced.

| Body / source entry | Frozen clocks closed | Native lines added |
| --- | ---: | ---: |
| `$07:81A0` Ground-control body | 3,961,714 | 128 |
| `$07:D104` Main tile-response suffix | 2,483,630 | 156 |
| `$1E:8920` Helmasaur tail | 1,600,220 | 60 |
| `$1E:919F` Zazak complete body | 1,400,988 | 98 |
| `$1D:BB70` Trinexx head draw | 1,120,784 | 40 |
| `$05:A64A` Lanmola draw | 1,036,260 | 63 |
| `$08:8C43` Ancilla class-2 inner | 818,876 | 30 |
| `$07:983A` Reused swimming reset | 719,004 | 0 |
| `$08:8BCF` Ancilla class-2 wrapper | 522,880 | 35 |
| `$1E:8A85` Multiply-delay body | 285,516 | 0 |
| `$00:8781` Shared local-dispatch entry | 44,408 | 0 |
| **Total** | **13,994,280** | **610** |

Lines are gross native additions, including comments/rewiring and shared visibility; fixtures, generated rows, manifest, generator and documentation are excluded. Shared-entry and delay marker lines are counted at their enclosing caller. Every cohort iteration's exclusive sum reconciles; there is **no scalar or existing-work attribution delta**. These are new native instruction-work receipts, not equivalent-call-sequence or physical-NMI proof. The bounded replay retains its diagnostic first WRAM divergence at 884327. An initial receipt's local-dispatch attribution was corrected before final census verification; its total clocks were unchanged.

Both final full census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero; every runtime hold equals P2. These are periodically seeded CPU logical gates, not continuous-route or exact A/V proof.

| Lane | Retained main missing / excess / FP | Batch 39 missing / excess / FP |
| --- | ---: | ---: |
| ROM | 4,433 / 0 / 0 | **4,352 / 0 / 0** |
| Live SPC | 5,129 / 589 / 131 | **5,052 / 593 / 132** |

All **48 crystal/pendant phases** remain within one frame; stable outcomes match batch 38. The retained **default 200k ledger matches all records and EOF**. Default/ROM-timing accounting is unchanged. Both ordinary builds are warning-free; existing lib-test-only warnings remain separate.

- Default Zelda library: **2,654 pass / 1 known spotlight caller-return failure / 3 ignored**.
- Reduced Zelda library: **2,189 pass / 0 fail / 2 ignored**.
- Both CPU-only SNES suites: **504 / 0 / 7**. Nine generator/worklist tests, four routine-coverage tests, exact table regeneration and `git diff --check` pass.

The complete library runs exposed an old setup-only assertion that now includes the response suffix. Its test-only correction compares full handler self work with the ROM; both affected helper users pass targeted rechecks in both modes. The manifests preserve the original complete runs and record these replacements; production sources stayed frozen throughout the censuses.

Complete library manifests, ROM/source/binary hashes, call-site audit and per-routine clock/line summaries bind this point. All **1,512** supplied ledger/cache files retain size and mtime. Raw predictors/profiles were deleted after their summaries; pruning hashes and compressed traces remain. Evidence is below **3 GB**. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used.

## Load/transition batch 1

**Verified partial stop on `codex/load-pacing1`, main `31633b01`.** Five complete bodies add **16,040,244 marker clocks** on the affected frozen cohort, exactly matching their retained source-exclusive work. ROM missing/excess/FP changes **4,433 / 0 / 0 → 4,346 / 0 / 0**; live changes **5,129 / 589 / 131 → 5,046 / 593 / 132**. Supplied main controls were reused. Evidence: `target/load-pacing1/`.

The worklist ranks independent exclusive gaps against each iteration's P2 inverse, by closures rather than clocks. It contains **1,499** missing iterations from forced-seed window profiling. Those windows add **228 diagnostic missing frames in `06/00` and 13 in `09/23`** relative to the supplied census; the other eleven category deficits agree. This is a bounded candidate worklist, not a recapture of the canonical missing-iteration set. ROM/assets/SRAM/input hashes match the retained cartridge-profile receipt.

The generator adds **63 blocks / 404 instructions**; all **930** existing rows remain identical. Eleven decoded ROM call sites bind the admitted bodies. The header and its two adjacent-door children own separate scopes, including the shared RTS exits exactly once per call. Markers retire at the executing body/prefix, with lagless and preview-muting guards. Five unmodified-ROM fixtures pass; selected call counts and exclusive clocks also match source on every frozen iteration in both the matched-window and canonical ROM profiles. No scalar conversion, fitted price, scene threshold/multiplier/table, or classifier change was introduced.

| Complete source body | New cohort marker clocks | Predicted closures | Observed joint gained frames on its calls |
| --- | ---: | ---: | ---: |
| `$01:B564` Room header | 2,105,584 | 8 | 73 |
| `$01:B759` Adjacent-door merge | 3,722,140 | 12 | 73 |
| `$01:B7EF` Adjacent-door loader | 2,241,262 | 9 | 73 |
| `$01:FE41` Overlay attributes | 6,438,816 | 8 | 8 |
| `$0E:942A` Custom attribute copy | 1,532,442 | 11 | 40 |

Matched-window replay observes **81 gained frames across 81 iterations**, with 0 losing holds. Per-body observed counts overlap and are not isolated marginal effects. New prices account for 16,040,244 clocks; the signed total-counter change is 16,040,244. `routine-cohort-before-after.json` keeps every existing-work/attribution delta separate. The superseded broad-window comparison used different window boundaries; its larger delta is retained separately and excluded from pricing credit. Whole-route gain is measured by the final censuses above.

Highest unresolved leaders remain open: mirror-table padding/loop mapping, the room/door dispatch trees, and excluded sprite loader/cache/disable/proximity owners. `$02:F7CB` reaches the existing priced quadrant decoder through a native alias; palette-cache and lamp-cone aliases receive no second price. No sprite, player, ancilla or garnish file was edited.

The live totals also regress outside the priced loader bodies: `0E/02` missing **360 → 364**, and excluded `07/00` excess **102 → 106**, FP **18 → 19**. These remain open; no price or classifier was fitted to suppress them. The targeted load gains reproduce in both lanes.

Both final full census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero and all holds equal P2. These are periodically seeded CPU logical gates.

| Entry | ROM missing before → after | Live missing before → after | Final excess ROM / live | Final FP ROM / live |
| --- | ---: | ---: | ---: | ---: |
| `09/23` | 338 → 338 | 338 → 338 | 0 / 0 | 0 / 0 |
| `07/02` | 284 → 251 | 284 → 251 | 0 / 0 | 0 / 0 |
| `07/07` | 239 → 233 | 239 → 233 | 0 / 0 | 0 / 0 |
| `07/0E` | 118 → 101 | 118 → 101 | 0 / 0 | 0 / 0 |
| `06/00` | 89 → 74 | 89 → 74 | 0 / 0 | 0 / 0 |
| `09/0D` | 81 → 81 | 81 → 81 | 0 / 0 | 0 / 0 |
| `09/16` | 71 → 71 | 71 → 71 | 0 / 0 | 0 / 0 |
| `07/15` | 64 → 62 | 64 → 62 | 0 / 0 | 0 / 0 |
| `07/0F` | 56 → 56 | 56 → 56 | 0 / 0 | 0 / 0 |
| `0F/01` | 55 → 55 | 55 → 55 | 0 / 0 | 0 / 0 |
| `07/03` | 53 → 45 | 53 → 45 | 0 / 0 | 0 / 0 |
| `0B/24` | 50 → 50 | 50 → 50 | 0 / 0 | 0 / 0 |
| `01/00` | 48 → 48 | 48 → 48 | 0 / 0 | 0 / 0 |

All **48 crystal/pendant phases** remain within one frame and stable outcomes match. The retained **default 200k ledger matches all records and EOF**; default/ROM-timing accounting is unchanged. Both ordinary builds are warning-free; existing lib-test warnings remain separate. Default library: **2,650 pass / 1 known spotlight failure / 3 ignored**. Reduced: **2,185 pass / 0 fail / 2 ignored**. Both CPU SNES suites: **504 / 0 / 7**. Generator/worklist, routine-coverage and inverse tests, exact regeneration and `git diff --check` pass.

All **1,512** supplied ledger/cache files retain size and mtime. Hashes, commands, disjoint suite manifests and `verification.json` bind this stop. Raw predictors/profiles were deleted after summaries; evidence is **210.0 MB**, below 3 GB. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, cache write, cargo fmt, commit/push or one-off repository script was used. The remaining owner worklist and independent per-routine causal isolation are unfinished.

## Verified batch 40 on `codex/cost-pacing40` (main `3d102ee8`)

**100 predicted / 100 actual ordinary-gameplay flips**, on the **same 100 iterations**: **38 in 07/00**, **62 in 09/00**. Pricing stopped at that target. Three additional 0F/00 flips bring the full ROM improvement to **103**; no new missing iterations appear. Evidence: `target/cost-pacing40/`.

The requested fresh main predictor identifies **2,416 missing iterations**. Full-context before/after profiles reconcile every native total; they replace 53 context-dependent bounded rows. The retained batch-39 census remains the baseline. `owner-predictions.md` gives every raw-owner and admitted-marker prediction; `ranked-owners.json` retains the ranking and per-iteration shortfalls. Already charged drawing, velocity, ground, shadow, inactive and layer aliases receive no second price. Shared module/transition code and divergent ancilla predicates remain outside this batch.

Complete ROM bodies/callers cover Cucco/guard movement, Deadrock, boulder control/drawing/Z motion, the guard's map16 probe, lightning/laser garnish allocation, and Stalfos drawing. The prepared Stalfos path prices its complete executed writer, leaving its skipped coordinate preparation unpriced. Call sites and RTS tables are ROM-checked. **115 blocks / 475 instructions** are added; all **1,230 existing rows are byte-identical**. Twelve new `assert_rom_marker_work` fixtures pass in both complete suites; reused movement bodies retain their existing fixtures. The width-independent `$1D:D018 SEP` join is fixture-covered. All prices are lagless-only markers selected by actual branches/loops, without fitted residuals, scene thresholds/multipliers/tables, or parent/child double charging.

**7,617,748 new marker clocks** reconcile on every cohort iteration, with **zero scalar or attribution delta**. Independent admitted-owner flip predictions overlap: `$05:DF6C` 14; `$06:84E2` 12; `$1E:A488` 11; `$1D:C222` 9; `$06:E92C`, `$06:E93E`, `$1D:E98B`, `$1D:D185` 6 each; `$1D:E475` 5; `$1E:FF62` 2; `$06:E932`, `$06:F15C` 1 each; the other ten admitted owners 0. Their disjoint clocks predict the combined 100 flips, all observed. Source/marker receipts do not establish equivalent call sequences or physical NMI placement.

Both final full census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero, every hold equal to P2. These are CPU logical gates, not continuous-route or exact A/V proof. Counts below are missing / excess / FP.

| Lane / scope | Retained batch 39 | Batch 40 |
| --- | ---: | ---: |
| ROM full | 4352/0/0 | **4249/0/0** |
| ROM 07/00 | 1489/0/0 | **1451/0/0** |
| ROM 09/00 | 927/0/0 | **865/0/0** |
| Live SPC full | 5052/593/132 | **4945/589/131** |
| Live SPC 07/00 | 1489/106/19 | **1451/102/18** |
| Live SPC 09/00 | 927/0/0 | **865/0/0** |

All **48 crystal/pendant phases** remain within one; stable outcomes match batch 39. The **default 200k ledger matches all records and EOF**; default/ROM-timing accounting is unchanged. Zelda library pass/fail/ignored: **default 2666/1/3** (only the known spotlight failure), **reduced 2201/0/2**. Both ordinary builds are warning-free; the existing lib-test-only warning remains separate. Generator/worklist (9), routine coverage (4), shortfall (6), exact regeneration and `git diff --check` pass.

All **1,512 supplied ledger/cache files** retain size and mtime. Complete test manifests, source/binary hashes, per-owner predictions, marker deltas and pruning hashes bind this stop. Large raw artifacts were deleted after summaries; evidence is below **3 GB**. No GPU run, Snes9x execution, oracle-cache write, cargo fmt, commit or push was used.

## Load/transition batch 2

**Verified partial stop on `codex/load-pacing2`, main `dbcb13c0`.** ROM missing/excess/FP: **4,265 / 0 / 0 → 4,130 / 0 / 0**. Measured live after: **4,827 / 590 / 132**; no fresh live main control was run. Supplied main ROM control reused. Evidence: `target/load-pacing2/`.

The extended batch-1 worklist contains **1,066 bounded missing iterations**, all covered by the retained source profile. The `$02:B1FF` alias leads permitted loader candidates at **154 potential closures**; admitted complete bodies predict **135**. Forced-seed windows are diagnostic, distinct from the canonical census; standing polygon/crystal drift signatures match before/after. Category counts are in `cohort-summary.json`.

Completed `$00:FE64..FF2E` and `$00:FF2F..FFB6`, the highest-priority `09/23` mirror-table bodies. **32 blocks / 149 instructions** are ROM-generated; all **1,293** prior rows are unchanged. `$02:B204` dispatch table entries `$B20E/$B211` and `$02:9E0F/$9E15` direct calls bind the bodies. Only their call instructions are newly priced; animation/fixed-color child bodies receive no added charge. The shared RTL is charged once. Markers retire on the executing prefix/body before caller continuation.

The ROM's **27 copies, word indices 216 down to 8**, match the native prefix; its extra trailing groups 224/232 receive no ROM price. Markers follow the ROM's 2/3→8 scratch countdown branches. Two `assert_rom_marker_work` fixtures pass **64 ROM cases**, including parity, arithmetic/completion branches and table equality. No game-state writes, gate rules, fitted prices or protected gameplay files changed.

| Complete body | Matched-cohort new clocks | Predicted closures | Actual gained frames on body iterations |
| --- | ---: | ---: | ---: |
| `$00:FE64` Waving table | 4,704,932 | 135 | 135 |
| `$00:FF2F` Dewaving table | 0 | 0 | 0 |

Matched windows gain **135 frames**, with **0** iterations losing holds. New markers total **4,704,932 clocks**; signed main-counter change is **4,704,932**. The per-body rows are measured batch outcomes on disjoint body executions; they are not isolated causal reruns. Existing scalar clocks (**10,057,048**) move from the main scope into the new table scope without adding work; every matched iteration's main-counter delta equals its new markers. Ownership deltas are retained separately. Whole-route ROM gain is **135 frames**; only `09/23` changes among all ROM modules.

Both final full CPU census gates **PASS**, **1,473,123 iterations** each, unexplained zero, every runtime hold equal to P2. These are periodically seeded logical checks.

| Entry | ROM missing before → after | Live missing after | Excess ROM / live | FP ROM / live |
| --- | ---: | ---: | ---: | ---: |
| `09/23` | 338 → 203 | 203 | 0 / 0 | 0 / 0 |
| `07/02` | 242 → 242 | 242 | 0 / 0 | 0 / 0 |
| `07/07` | 231 → 231 | 231 | 0 / 0 | 0 / 0 |
| `07/0E` | 98 → 98 | 98 | 0 / 0 | 0 / 0 |
| `09/0D` | 81 → 81 | 81 | 0 / 0 | 0 / 0 |
| `06/00` | 74 → 74 | 74 | 0 / 0 | 0 / 0 |
| `09/16` | 71 → 71 | 71 | 0 / 0 | 0 / 0 |
| `07/15` | 62 → 62 | 62 | 0 / 0 | 0 / 0 |
| `07/0F` | 56 → 56 | 56 | 0 / 0 | 0 / 0 |
| `0F/01` | 55 → 55 | 55 | 0 / 0 | 0 / 0 |
| `0B/24` | 50 → 50 | 50 | 0 / 0 | 0 / 0 |
| `01/00` | 48 → 48 | 48 | 0 / 0 | 0 / 0 |
| `07/03` | 45 → 45 | 45 | 0 / 0 | 0 / 0 |
| `0E/02` | 40 → 40 | 361 | 0 / 223 | 0 / 96 |
| `0E/03` | 37 → 37 | 37 | 0 / 0 | 0 / 0 |
| `0E/05` | 33 → 33 | 33 | 0 / 0 | 0 / 0 |
| `0B/17` | 31 → 31 | 31 | 0 / 0 | 0 / 0 |
| `0B/29` | 28 → 28 | 28 | 0 / 0 | 0 / 0 |

All **48 crystal/pendant phases** remain within one frame; stable outcomes match the retained phase control. The retained **default 200k ledger matches every record and EOF**; default/ROM-timing ledgers are unchanged. Ordinary default/reduced builds are warning-free; the existing lib-test dead-code warning remains. Default library: **2,661 pass / 1 known spotlight failure / 3 ignored**. Reduced: **2,196 pass / 0 failures / 2 ignored**. Both CPU SNES suites: **504 / 0 / 7**. Generator/worklist, routine-coverage, inverse, regeneration and `git diff --check` pass.

The alias-normalized `remaining-worklist.json` retains **931 diagnostic missing iterations**, including **19** remaining B1FF potential closures. Mirror initialization/finalization, overlay DMA preparation (`$01:F764`, 37 predicted closures), overlay drawing (`$01:F980`, 25), iris and room/door dispatch remain open. Sprite/cache/disable/proximity, player, ancilla and garnish owners were skipped under the shared-work restriction. All **1,512** supplied ledger/cache file sizes/mtimes are unchanged. Raw profiles/predictors were summarized and deleted; evidence is **214.0 MB**, below 3 GB. No GPU execution, oracle-cache write, cargo fmt, commit/push or scratch repository script was used.


## Marker plumbing cleanup 1

**Verified partial refactor on `codex/marker-cleanup1`, main `68beb4e2`.** No behavior or pricing change was admitted. Evidence: `target/marker-cleanup1/`.

| Family | Entry plumbing before → after |
| --- | --- |
| Moving-background velocity | raw + marked + body → one entry, explicit `MovingBackgroundEntry::Unpriced` / `Rom07B9B3` |
| Ancilla class-2 tile collision | raw + marked + body → one entry, explicit `AncillaClass2Entry::Unpriced` / `Rom088BCF`; distinct inner collision body retained |
| Sword cooldown | marked + body → one complete entry; no alternate/unpriced entry existed |

**Five fewer functions**, **12 source/test files**, **73 added / 54 removed source/test lines (net +19)**; this appendix is the thirteenth file. Marker compounds in the folded bodies use normal multiline style. Caller ROM comments, marker order, routine scopes and unpriced paths are preserved. All three semantic/marker body token sequences are unchanged after removing whitespace. Fixtures change call paths only; no price, generated table, model or scalar changed.

Both fresh main predictors were captured completely before source edits. Both final predictors are **BYTE-IDENTICAL**, including headers and all **1,473,123 rows per lane** (ROM and `--live-apu-reads`); all **four census gates PASS**, unexplained zero. `predictor-comparison.json` retains complete SHA-256 hashes and byte lengths. The four raw CSVs were deleted after `cmp` and hash comparison. These are periodically seeded CPU logical gates.

The retained **default 200k ledger matches every record and EOF**, before and after. Library pass/fail/ignored: **default 2671/1/3** (only the known spotlight caller-return failure), **reduced 2206/0/2**. All routine marker fixtures pass. Both ordinary builds are warning-free; lib-test builds retain exactly their baseline warning groups (**24 default / 1 reduced**), with **no new dead code** and no change to `cfg_attr(not(test))` policy.

**Deferred loader/transition families:** `filter_majorly_whiten_bg` / marked entry; `overworld_check_special_switch_area` / marked entry; `draw_mire_body`; bank-02 decompression wrappers; `dungeon_load_basic_attribute_body`; crystal-peg bank/continuation entries; `tile_map_prep_not_water_on_tag_body`; `room_draw_a_many_32x32_blocks_body`; `room_draw_floor_chunks_body`; pre-dungeon song-bank/reset, module09 long-load and straight-interroom sprite-reset continuations. Broader ending, NMI, game-loop and player-OAM continuation bodies remain separate. None of their files or callers was edited. Remaining sprite/player/ancilla families are listed in `wrapper-inventory.md`, with the complete literal matches and 292 matching definitions beside it; this partial batch does not claim their consolidation or formatting.

All **1,512 supplied ledger/cache files** retain size and mtime. `verification.json` binds source/binary hashes, library manifests, checks and file/line totals. Evidence remains below **3 GB**. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit, push or one-off repository script was used.

## Load/transition batch 3

**Verified partial stop on `codex/load-pacing3`, main `19a803cd`.** ROM missing/excess/FP: **4,025 / 0 / 0 → 3,983 / 0 / 0** (42 frames restored). Measured live after: **4,686 / 596 / 133**. Supplied joint main control reused; no full main controls rerun. Evidence: `target/load-pacing3/`.

Refreshed the stale native worklist against retained, hash-matched source receipts: **1,108 missing iterations / 1,156 frames**, with **0** rows lacking retained source coverage. Ranking uses each missing iteration's P2 inverse and exclusive owner work, with FE64/FF2F tail aliases folded into B1FF and the F762 inline F764 body normalized for ranking. The inherited uppercase-module selector was corrected by replaying only supplemental 07/0E and 09/0D windows. Seeded windows remain diagnostic; canonical census results are separate.

Four complete entries add **13 blocks / 171 ROM instructions**; all **1,440** existing rows remain unchanged. `$01:B88E → F764`, `$01:F396/F650 → F77C`, `$01:F978 → F980`, and `$02:B232 → $00:FDEE` statically bind the callers. F764 falls through its shared body without a child charge; direct watergate calls own F77C. Drawing charges the actual A4/floor branch. Mirror initialization prices the ROM's 224 words in source store order; native off-screen padding remains free. No classifier, scene threshold/multiplier/table or protected gameplay source was changed.

Four `assert_rom_marker_work` fixtures pass **19 unmodified-ROM cases**, covering DMA address/orientation branches, supplied watergate rows/headers, both drawing bodies, and mirror scroll values/table equality. Existing scalars in these four routines total **zero**: no conversion was needed and no scalar was added.

| Complete entry | Old scalar clocks on matched cohort | New marker clocks | Predicted flips | Actual gains on body iterations |
| --- | ---: | ---: | ---: | ---: |
| `$00:fdee` Mirror initialization | 0 | 0 | 0 | 0 |
| `$01:f764` Overlay DMA preparation | 0 | 3,495,580 | 37 | 42 |
| `$01:f77c` Direct watergate DMA | 0 | 0 | 0 | 0 |
| `$01:f980` Overlay drawing | 0 | 1,076,052 | 25 | 42 |

Matched windows gain **42 frames**, with **zero** iterations losing holds. **4,571,632 new clocks** equal the signed main-counter increase on every affected iteration. Per-body actuals count gains on that body's executions under the joint candidate; overlapping executions are not isolated marginal effects and must not be summed. Prefix markers execute before caller continuation/queue reads. The F762 fallthrough contributes its F764 body once; its separate 32-clock STZ prefix remains unpriced. After this tail normalization, new marker totals also equal retained source-body work. Canonical ROM improvement is **42 frames**.

Both final full CPU census gates **PASS**, **1,473,123 iterations** each, unexplained zero; runtime holds equal P2 throughout. These are periodically seeded logical checks.

| Entry | ROM missing after | Live missing after | Excess ROM / live | FP ROM / live |
| --- | ---: | ---: | ---: | ---: |
| `09/23` | 201 | 201 | 0 / 0 | 0 / 0 |
| `07/02` | 242 | 242 | 0 / 0 | 0 / 0 |
| `07/07` | 231 | 231 | 0 / 0 | 0 / 0 |
| `07/0E` | 98 | 98 | 0 / 0 | 0 / 0 |
| `09/0D` | 81 | 81 | 0 / 0 | 0 / 0 |
| `06/00` | 74 | 74 | 0 / 0 | 0 / 0 |
| `09/16` | 71 | 71 | 0 / 0 | 0 / 0 |
| `07/03` | 3 | 3 | 0 / 0 | 0 / 0 |

All **48 crystal/pendant phases** remain within one; stable outcomes match the retained control. The default **200k ledger matches every record and EOF**. Ordinary default/reduced builds are warning-free; existing lib-test warnings remain. Default library: **2,677 pass / 1 known spotlight failure / 3 ignored**. Reduced: **2,212 pass / 0 failures / 2 ignored**. Both CPU SNES libraries: **504 / 0 / 7**. Generator/worklist, routine-coverage, inverse/regeneration and `git diff --check` pass.

The refreshed `remaining-worklist.json` retains **1,066 diagnostic missing iterations**. Mirror finalization/dispatch, iris and room/door ownership remain open. Source tail aliases are candidate groups rather than unpriced whole routines; protected sprite/player/ancilla/garnish owners were skipped. All **1,512** supplied ledger/cache file sizes/mtimes are unchanged. Raw profiles/predictors were summarized and deleted; evidence is **214.3 MB**, below 3 GB. No GPU run, oracle-cache write, cargo fmt, commit/push or one-off repository script was used.

## Marker plumbing cleanup 2

**Verified pure refactor on `codex/marker-cleanup2`, main `6ea303bf`.** Evidence: `target/marker-cleanup2/`.

**17 families folded; 26 fewer functions.** X/Y/XY/XYZ/Z movement (5), damage from/to/both (3), tile collision, general/00 hit boxes (2), single-small/large/shadow drawing (3), inactive/paused predicates (2), and recoil now use explicit ROM-path selectors with `Unpriced`. Existing scalar child costs remain. Bank-06 XY's Y tail, XYZ's XY tail, default/custom shadow scopes, double-return ownership, and distinct predicates/continuations are preserved. Caller ROM comments are unchanged. Twenty body/wrapper token streams are unchanged after selector normalization; marker compounds use multiline style. Remaining families and the complete before/after literal inventory are in `wrapper-inventory.md` and `wrapper-inventory.json`.

**35 source/test files, +2072 / -1712 lines (net +360); 36 files and +376 net lines including this appendix.** No model, generated price, scalar, loader or transition logic changed.

Both fresh main and final full predictors are **BYTE-IDENTICAL**: ROM and `--live-apu-reads`, headers plus **1,473,123 rows per lane**. All four census gates **PASS**, unexplained zero. `predictor-comparison.json` retains byte lengths and SHA-256 hashes; all four CSVs were deleted after byte comparison. These are periodically seeded CPU logical gates.

The retained **default 200k ledger matches every record and EOF**. Library pass/fail/ignored: **default 2673/1/3** (only the known spotlight caller-return failure), **reduced 2208/0/2**. All marker fixtures pass. Both ordinary builds are warning-free; lib-test warning groups match baseline exactly (**24 default / 1 reduced**), with no new dead code or `cfg_attr(not(test))` policy change. All 19 pricing/worklist/coverage/shortfall Python tests, exact table regeneration and `git diff --check` pass.

**Deferred loader families:** `filter_majorly_whiten_bg`; `overworld_check_special_switch_area`, `draw_mire_body`, bank-02 decompression and module09 long-load paths; dungeon basic/crystal-peg attributes; water-tag tile preparation; 32x32/floor-chunk builders; pre-dungeon song/reset and straight-interroom sprite-reset continuations. Their files/callers were not edited. Broader ending, NMI, game-loop and player-OAM continuation bodies remain distinct; ending sprite callers only gained selector arguments.

All **1,512 supplied ledger/cache files** retain size and mtime. Source/binary hashes, test manifests and final counts are in `verification.json`; evidence stays below **3 GB**. No GPU run, oracle-cache write, cargo fmt, commit, push or one-off repository script was used.

## Load/transition batch 4

**Verified partial stop on `codex/load-pacing4`, main `74f04ad6`.** ROM missing/excess/FP: **3,983 / 0 / 0 → 3,923 / 0 / 0** (60 restored frames). Measured live after: **4,623 / 593 / 133**. Supplied joint main control reused; no full main controls rerun. Evidence: `target/load-pacing4/`.

The batch-3 worklist supplies **997 missing iterations** in the seven requested modules, refreshed with matched native windows against hash-verified retained ROM profiles. Rank uses each iteration's P2 shortfall, exclusive unpriced work and tail normalization, rather than clocks alone. Nine admitted entries yield **78 independent predicted flips / 60 joint predicted flips**; matched replay gains **60 frames**, with zero lost holds or new overcounts. The >=80 aim was not reached; remaining tail/continuation ownership needs another source audit. Forced windows remain diagnostic, separate from the canonical census.

**57 generated blocks / 224 instructions** price door decode and door flags, the three shared row-fill entries, star reset/direct restore, mirror animation/filter dispatch and falling-transition dispatch. All **1,453 existing rows** remain unchanged. 32 decoded call sites/table bindings are retained in `source-pricing-audit.json`. Six `assert_rom_marker_work` fixtures pass **314 unmodified-ROM cases** over all nine entries. Shared fallthroughs own one scope; child bodies are excluded. Falling prefixes precede the suspending attribute load; the mirror return marker follows the completed animation child, including its final NMI-request stores.

| Complete entry | Old scalar clocks | Marker clocks | Net clocks closed | Predicted flips | Actual joint gains on positive-net calls |
| --- | ---: | ---: | ---: | ---: | ---: |
| `$00:eee7` Mirror animation/filter dispatch | 0 | 127,734 | 127,734 | 14 | 14 |
| `$00:fda4` Star-tile reset + restore | 0 | 1,261,876 | 1,261,876 | 20 | 36 |
| `$00:fda7` Direct star-tile restore | 0 | 0 | 0 | 0 | 0 |
| `$01:8916` Door decode/dispatch | 0 | 457,930 | 457,930 | 5 | 43 |
| `$01:b0da` Door flags/type | 0 | 933,678 | 933,678 | 13 | 43 |
| `$01:b2ca` Advance/row fill | 1,250,608 | 1,250,608 | 0 | 0 | 0 |
| `$01:b2ce` Width/row fill | 0 | 1,782,784 | 1,782,784 | 17 | 25 |
| `$01:b2d0` A-count/row fill | 0 | 794,400 | 794,400 | 6 | 16 |
| `$02:8e27` Falling-transition dispatch | 0 | 41,748 | 41,748 | 3 | 6 |

Two lagless-only B2CA scalar calls are deleted and replaced: **1,250,608 → 1,250,608 clocks**, equal on every cohort iteration. This conversion receives zero flip credit. Other scalars remain untouched, including default/ROM-timing accounting. Total new markers **6,650,758**, less removed scalars, equal the **5,400,150-clock** signed main-counter increase on every cohort row. Per-body actual gains overlap and must not be summed; they are batch outcomes on executions, not isolated marginal reruns.

Full source-exclusive clocks match the admitted mirror, star reset, door-flag and A-count row bodies on this cohort. Door-dispatch, falling-dispatch and width/advance-fill source scopes also contain tail bodies or unadmitted aliases; those residuals remain open and are not counted as closed work. Iris reset has no calls in the selected missing cohort; mirror finalization remains under the B1FF tail frame. Other room/door targets, mosaic tails and protected sprite/player/ancilla/garnish owners were deferred. No classifier, fitted constant, scene threshold, multiplier or scene table changed.

Both final full CPU census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero; every runtime hold equals P2. These are periodically seeded logical checks.

| Entry | ROM missing before → after | Live missing after | Excess ROM / live | FP ROM / live |
| --- | ---: | ---: | ---: | ---: |
| `07/02` | 242 → 221 | 221 | 0 / 0 | 0 / 0 |
| `07/07` | 231 → 225 | 225 | 0 / 0 | 0 / 0 |
| `09/23` | 201 → 187 | 187 | 0 / 0 | 0 / 0 |
| `07/0E` | 98 → 86 | 86 | 0 / 0 | 0 / 0 |
| `09/0D` | 81 → 81 | 81 | 0 / 0 | 0 / 0 |
| `06/00` | 74 → 67 | 67 | 0 / 0 | 0 / 0 |
| `09/16` | 71 → 71 | 71 | 0 / 0 | 0 / 0 |

All **48 crystal/pendant phases** remain within one; stable outcomes match batch 3. The retained **default 200k ledger matches every record and EOF**. Default and reduced ordinary builds are warning-free; existing lib-test warnings remain. Default library: **2,683 pass / 1 known spotlight failure / 3 ignored**. Reduced: **2,218 pass / 0 failures / 2 ignored**. Both CPU SNES libraries: **504 / 0 / 7**. Block generator/worklist, routine-coverage, inverse, exact regeneration and `git diff --check` pass.

`remaining-worklist.json` retains **937 refreshed missing iterations** in the requested modules; the **69 inherited rows outside those modules** are preserved separately as unprofiled, not asserted unchanged. All **1,512 supplied ledger/cache files** retain size and mtime, with no files added. Raw profiles/predictors were summarized and deleted; retained evidence is **258.5 MB**, below 3 GB. No GPU run, --video, cached-av, Snes9x, GPU test, oracle-cache write, cargo fmt, commit/push or scratch repository script was used.


## Cost-pacing batch 41 — verified partial stop

On `codex/cost-pacing41`, main `e6f9ffcd`: **43 predicted / 43 actual ordinary-gameplay flips**. The **120-frame aim was not reached**; this is a verified partial batch. Evidence: `target/cost-pacing41/`. No main controls were repeated; the permitted fresh ROM predictor reproduces **3,983/0/0**, with **1,451 in 07/00** and **865 in 09/00** across **2,316 missing iterations**. `ranked-owners-summary.json` and `cohort.json` retain the frames-flipped ranking and every inverse shortfall; raw owner gaps include aliases and are not additive prices.

Complete Vitreous-eye entry/state tails, Swamola angle conversion, distress writer, fireball, Zora writer and busy predicate add **87 blocks / 368 instructions**; all **1,453** existing rows are unchanged. ROM callers and the eye's RTS table are checked in `source-pricing-audit.json`. Fireball uses the existing complete small-draw selector. Distress has one entry with explicit `Unpriced` / `Rom06A733` selectors: only the confirmed Cucco fallthrough and falling-sprite calls are priced; pink-ball and archery reuse stay unpriced. Other angle callers stay raw. No loaders, transition logic, classifiers or game-state writes changed.

The eye's old **524-clock lagless prefix scalar is deleted**, with full ROM markers replacing it. The other selected routines had zero scalar charges. Existing default/ROM-timing accounting remains intact. On the exact matched cohort:

| Routine | Old scalar clocks | New marker clocks | Net clocks | Independent predicted flips |
| --- | ---: | ---: | ---: | ---: |
| eye `$1d:e773` | 387,236 | 755,802 | 368,566 | 13 |
| angle `$1d:f614` | 0 | 340,124 | 340,124 | 7 |
| busy `$07:f4d0` | 0 | 324,410 | 324,410 | 5 |
| fireball `$05:9683` | 0 | 1,013,144 | 1,013,144 | 4 |
| distress `$06:a733` | 0 | 140,312 | 140,312 | 4 |
| zora-draw `$05:98f5` | 0 | 364,714 | 364,714 | 3 |
| small-writer `$06:dcef` | 0 | 591,404 | 591,404 | 1 |
| small-wrapper `$06:dbf8` | 0 | 122,740 | 122,740 | 0 |

**3,652,650 new marker clocks − 387,236 retired scalar clocks = 3,265,414 net clocks**. Every cohort main-counter change reconciles to this partition. Independent predictions overlap; the combined prediction uses their disjoint net clocks. `frame-flips.json` records any prediction/actual differences and every module's gains. These are native executed-work receipts, not equivalent-call-sequence or physical-NMI proof.

Six proportionate `assert_rom_marker_work` fixtures pass in both complete library suites, including angle table crossings, clipping, short-circuit gates, AI tails and the actual five-slot busy scan. The old eye setup-only test now measures its complete non-AI return and supplies the caller's current-slot byte. Original full-suite manifests are retained; rebuilt test binaries recheck this corrected case, with explicit replacements in the final manifests. Both full CPU census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero; runtime holds equal P2 throughout. These are periodically seeded logical gates, not continuous-route or A/V acceptance. Counts are missing / excess / FP:

| Lane / scope | Before | After |
| --- | ---: | ---: |
| ROM full | 3,983/0/0 | **3,898/0/0** |
| ROM 07/00 | 1,451/0/0 | **1,433/0/0** |
| ROM 09/00 | 865/0/0 | **840/0/0** |
| Live full | supplied baseline reused | **4,597/592/132** |
| Live 07/00 | supplied baseline reused | **1,433/105/19** |
| Live 09/00 | supplied baseline reused | **840/0/0** |

All **48 crystal/pendant phases** remain within one, with stable outcomes unchanged. The retained schema-3 **default 200k ledger matches every record and EOF**; `default-reference.json` binds the recovered available reference and input hashes. Library pass/fail/ignored: default **2683/1/3** (only the known spotlight failure), reduced **2218/0/2**. Both ordinary builds are warning-free; existing lib-test warnings are separate. Python block/worklist, routine-coverage and shortfall suites, exact regeneration and `git diff --check` pass.

All **1,512 supplied ledger/cache files** retain size and mtime. Frozen source/binary hashes, complete library manifests, commands, prediction/clock summaries and pruning hashes bind the stop. Large raw artifacts were deleted after summaries; evidence is below **3 GB**. No GPU, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit/push or scratch repository script was used. Remaining owner pricing and the 120-frame target are unfinished.

## Continuous lagless 1

**Verified diagnosis stop on `codex/continuous1`, `9bece911` (supplied main plus batch 41). The continuous gate is not achieved.** Evidence is in `target/continuous1/`; `main-zelda3` retains the tested binary, SHA-256 `337ba9dc4f55a762f799272beb5890332c27bb4b4ce301b49c72ab35e95b56cc`. Both lanes boot cold, use recorded inputs and RNG answers, and omit `--seed-every`, including its post-polygon resyncs. The full recorded-answer lane uses `--report 0`; diagnostic cold prefixes add classification, watch and dump outputs.

| Lane / divergence | First iteration and context | Proven cause / classification | Fix or stopping reason |
| --- | --- | --- | --- |
| Recorded APU: map-upload comparison | **471,536**, module `$0E/$07`, room `$0028`; first byte `$0012` native/source `$01/$00`. Also `$0017=$00/$15`, `$001A=$F1/$F2`, `$0200=$02/$03`, `$C00D=$07/$06`, `$C013=$0F/$0E`, and 256 map-buffer bytes. | **Recorder snapshot-boundary defect**, a class-4 observation-path failure. The compared post image belongs to the following iteration's NMI acceptance. Cold replay has 471,536 clean iterations before this boundary and reconverges at 471,537. | No runtime fix or classifier change. Repair requires correct source post-NMI capture; the supplied ledger remains unchanged. Holds agree **5/5**, so a missing price is not established. |
| Live APU: pendant song-end poll | **94,848**, module `$07/$00`, room `$00C8`; native opens `$0E/$02`. First bytes `$0010=$0E/$07`, `$0011=$02/$00`, `$03B5=$01/$02`, `$1CF0..1=$0059/$0186`. | **APU-answer divergence**: native reads `$00`, recorded answer is `$13`. Pendant receipt slot 4 opens dialogue instead of restoring its auxiliary timer to 2. The recorded-answer control stays WRAM-clean at the same boundary. | No fix: underlying SPC/command/clock cause remains open. `audio-handoff.md` retains the diagnosis. No forced live answer or fitted delay was added. |

Both first boundaries have zero input, no RNG draws, and clean RNG consumption. The map boundary has no APU reads. The live boundary consumes one song-end observation on each side, with different answers; local holds agree **0/0**. Native then holds five times at 94,849, while source dialogue setup reaches its five holds at 94,855. Source publishes song `$13` at 94,216; through 94,848 inclusive both native lanes execute **633 callbacks, zero holds and zero masked callbacks**, and source records zero lag NMIs. Earlier missing holds exist, but their responsibility for this song-end difference is unproven; no pricing owner is assigned from this evidence.

The map diagnosis is independently bound to the immutable streams. **Post image 471,537 equals pre image 471,538 byte for byte across all 128 KiB**, SHA-256 `59107f28644d6788c55733d2cdd3d9d993ad7404cd83f85b34e46a868458ac17`. Native's entire `$1000..$10FF` map buffer equals the correct closing pre image 471,537. The recorder's `stale_opening` branch in `crates/snes/examples/logical_frame_ledger.rs` writes its pending post image at the next accepted NMI when the original handler return was not observed. `source-map-boundary-proof.json`, the cold watch/classification log and neighboring immutable input records retain this proof. This is not evidence of a seed-hidden native state carrier.

| Lane | Furthest completed iteration, zero-based | Continuous result |
| --- | ---: | --- |
| Recorded APU | **1,473,122**, all **1,473,123 inputs** | Reaches ending `$19/$1A`, finishes `$1A/$22`; **1,473,119 clean WRAM iterations**, four existing lag-classified map-overrun comparisons; required full summary gate **FAIL, 4**. Ordinary, polygon and song state divergence counts are zero; RNG and replayed APU consumption are clean. |
| Live APU | **436,940**, at least **436,941 completed inputs** retained in predictor | Interrupted after first failure was reproduced. Clean prefix **94,848**; cold boundary probe completes 94,856 iterations with eight song-poll state differences and summary gate **FAIL, 8**. No full-route or ending acceptance. |

The full recorded predictor reports **3,838 missing / 0 excess / 0 false-positive holds** (98,839 native versus 102,677 source lag NMIs; 5,152 masked callbacks). Its source-entry ending inventory contains 20,445 iterations; no ending WRAM differences were reported. Cold recorded prefixes at 10k, 40k, 160k and 400k pass their summary gates. A further cold 471,539-iteration probe isolates the first map mismatch and fails by one. The other three full-run lag-classified comparisons were not individually cold-traced; retained batch-41 seeded classifications identify candidate boundaries **473,561, 885,929 and 1,068,135**. Those candidate locations are not promoted to separately verified cold diagnoses.

The summary's `wram=clean` / `first WRAM divergence None` fields exclude classified lag differences; its final clean-iteration count and required gate expose them. With one continuous segment starting at zero, the gate includes all four. No summary or classifier was widened to pass them.

The fresh **default 200k comparison matches every record and EOF**. No runtime source, pricing marker, default/ROM-timing charge or ledger changed; no fix or new regression test is claimed. Seeded censuses were not rerun because there was no runtime change; their earlier passes are separate from this failing continuous gate. Commands, binary provenance, selected outcomes/predictor rows, pruning hashes and gate logs retain the stopping point. Raw WRAM/VRAM dumps and large raw predictors/outcomes were summarized and deleted. All 1,512 supplied ledger/cache files retain size and mtime; retained evidence is below 3 GB. No GPU run, `--video`, cached-av, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit, push or one-off repository script was used.

## Load/transition batch 5

**Verified partial stop on `codex/load-pacing5`, main `628e4cea`.** ROM missing/excess/FP: **3,923 / 0 / 0 → 3,857 / 0 / 0** (66 restored frames). Measured live after: **4,560 / 596 / 133**. Supplied joint main control reused; no full main controls rerun. Evidence: `target/load-pacing5/`.

The inherited tail/continuation audit excludes already priced map/decompression aliases, masked song polling, raster waits, VWF work and protected gameplay owners. A fresh matched-window cohort covers **1,215 missing iterations** in all fifteen requested modules against the retained, hash-verified ROM profile. Ranking uses each iteration's P2 shortfall after ownership review. The admitted group predicts **66 joint flips** (65 independent, overlapping closures) and gains **66 matched-window frames**, with no lost holds or new overcounts. The >=80 joint-prediction aim remains unfinished. Forced-window results are diagnostic; canonical census gains are separate.

**106 blocks / 429 instructions** price four complete self owners, with existing local-helper blocks reused. All **1,510 existing rows** remain unchanged. ROM call sites/table bindings are retained in `source-pricing-audit.json`; four `assert_rom_marker_work` fixtures pass **278 unmodified-ROM cases**. The test-only stepping bound rises from 100k to 1M instructions so complete graphics children can return. Mirror and mosaic prefixes precede suspending loads; return markers follow the actual completed child. The inline mirror reset is not charged as an F427 child. Shared jump helpers retain only their STY/PLY head scopes after pulling their returns.

| Owner | Old scalar clocks | New marker clocks | Predicted closures | Joint gains on executions |
| --- | ---: | ---: | ---: | ---: |
| `$0a:e5bc` Dungeon-map room row | 0 | 9,681,760 | 23 | 23 |
| `$02:b1ff` Mirror dispatcher + selected self tails | 0 | 290,532 | 18 | 18 |
| `$02:ae5e` Mosaic start + selected self tails | 0 | 96,876 | 17 | 25 |
| `$02:c2e4` Conditional mosaic | 0 | 47,396 | 6 | 25 |
| `$00:879c` Long dispatch child head | 0 | 9,724 | 1 | 18 |
| `$00:8781` Local dispatch child head | 0 | 8,424 | 0 | 25 |

New markers total **10,134,712 clocks**, exactly the signed main-counter increase on every cohort row. The priced self bodies contain no scalar calls: old scalar work is **0**, and no scalar conversion receives credit. Existing reset, palette/filter, graphics and song-child scalars remain disjoint and untouched. Map, mosaic-start and conditional-mosaic marker work equals complete source-exclusive work on every cohort row; the mirror tail-normalized source gap closes exactly. Independent closures overlap, and combined work can cross thresholds no body reaches alone. Per-owner joint gains overlap and must not be summed; they are not isolated marginal reruns. No classifier, fitted price, scene threshold/multiplier/table or protected sprite/player/ancilla/garnish code changed.

Both full CPU census gates **PASS**, **1,473,123 iterations per lane**, unexplained zero; all holds equal P2. These are periodically seeded logical checks.

| Entry | ROM missing before → after | Live missing after | Excess ROM / live | FP ROM / live |
| --- | ---: | ---: | ---: | ---: |
| `07/02` | 221 → 221 | 221 | 0 / 0 | 0 / 0 |
| `07/07` | 225 → 225 | 225 | 0 / 0 | 0 / 0 |
| `09/23` | 187 → 169 | 169 | 0 / 0 | 0 / 0 |
| `07/0E` | 86 → 86 | 86 | 0 / 0 | 0 / 0 |
| `06/00` | 67 → 67 | 67 | 0 / 0 | 0 / 0 |
| `09/0D` | 81 → 67 | 67 | 0 / 0 | 0 / 0 |
| `09/16` | 71 → 71 | 71 | 0 / 0 | 0 / 0 |
| `0F/01` | 55 → 55 | 55 | 0 / 0 | 0 / 0 |
| `0B/24` | 50 → 48 | 48 | 0 / 0 | 0 / 0 |
| `01/00` | 48 → 48 | 48 | 0 / 0 | 0 / 0 |
| `0E/02` | 40 → 40 | 367 | 0 / 223 | 0 / 96 |
| `0E/03` | 37 → 14 | 14 | 0 / 0 | 0 / 0 |
| `0E/05` | 33 → 33 | 33 | 0 / 0 | 0 / 0 |
| `0B/17` | 31 → 22 | 22 | 0 / 0 | 0 / 0 |
| `0B/29` | 28 → 28 | 28 | 0 / 0 | 0 / 0 |

Live drift outside the priced bodies remains open: `07/00` excess **106 → 109**, and `0E/02` missing **364 → 367**. No protected gameplay owner or classifier was changed to suppress it.

All **48 crystal/pendant phases** remain within one; stable outcomes match batch 4. The retained **default 200k ledger matches every record and EOF**. Its pruned old path was replaced with the retained local copy, SHA-256 `18b58a3f887457d770918497218eaf7eb39360e1d04ed789c8b04b0ff4d8ba77`, identical to the second retained block-worklist copy. Default/ROM-timing accounting is unchanged. Both ordinary builds are warning-free; lib-test builds retain the baseline 24/default and 1/reduced warning groups. Default library: **2,687 pass / 1 known spotlight failure / 3 ignored**. Reduced: **2,222 pass / 0 failures / 2 ignored**. Both CPU SNES suites and block/worklist, inverse, routine-coverage, exact regeneration and diff checks pass.

The refreshed targeted worklist retains **1,149 diagnostic missing iterations**; remaining raw deficits still need tail/alias/continuation admission. All **1,512 supplied ledger/cache files** retain size/mtime, with none added. Summarized raw profiles/predictors and duplicate fixture binaries are deleted; evidence is **216.1 MB**, below 3 GB. No GPU execution, --video, cached-av, Snes9x, GPU test, oracle-cache write, cargo fmt, commit/push or scratch repository script was used.

## Continuous lagless 2

**Verified on `codex/continuous2`, main `f4e5641b`: the continuous recorded-answer gate passes.**
The corrected ledger is `/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2`;
`full` remains the immutable backup. Only the recorder and documentation changed.
Native execution, prices, classifiers and summary gates are unchanged. Evidence
is in `target/continuous2/`.

Source receipts contain the completed opcode **and** any interrupt accepted
after it. At all four defects, the opening NMI's `RTI` at `$00:822C` immediately
accepts an IRQ. The old tracker ignored that return, inflated interrupt depth
and left `opening` pending until the next NMI's `stale_opening` fallback.
The recorder now retires RTI before accounting for the new interrupt, captures
the post image there, and still processes that interrupt. Two focused tests
cover the formerly stale opening and an IRQ nested inside NMI. No route
iteration is special-cased.

The normal recorder ran cold from `saves/zelda3.sfc` and the same cache's
`initial.srm` and `input.txt`, for **1,581,079 host calls**, with
`ZELDA3_LEDGER_SEED_EVERY=2000`. RNG remains source CPU beam-derived; ROM,
SRAM, input and cache RNG hashes are bound in `provenance.json`. It wrote
**1,473,123 complete iterations**, with four RTI/IRQ boundary events and
**zero deferred stale captures**. `df -h /` showed **211 GiB available** before
recording. The old ledger was never patched, moved or deleted.

The complete comparison checks every JSON record and all **1,473,124** post
images through EOF, including every scratch and stack byte. Post image j
closes iteration j−1. These are all four differing snapshots and all eight
differing metadata records:

| Compared iteration / finished metadata | Post image / following IRQ metadata | Source host | Raw WRAM bytes changed |
| ---: | ---: | ---: | ---: |
| 471,536 | 471,537 | 510,438 | 1,040 |
| 473,561 | 473,562 | 512,815 | 1,041 |
| 885,929 | 885,930 | 951,566 | 953 |
| 1,068,135 | 1,068,136 | 1,147,222 | 951 |

All four have the same source-boundary cause above. Their finished records
change `nmi_return_line` **225 → 48**, `picture_host` **source host + 1 → source
host**, and `irqs` **7 → 6**. Each following record changes only `irqs`
**0 → 1**: the IRQ accepted after NMI returns belongs to the newly opened
iteration. IRQ totals are conserved. All other post images and JSON fields
match, including joypads, RNG, APU reads/writes, lag counts and opening/closing
hosts. The first two changed images also match an independent 475k-image
comparison. The complete compressed pre-NMI stream is SHA-256 identical,
all **1,499 seed files** match byte for byte, and the **1,502-file set** is
unchanged. There are **no unexplained differences**.

All three native runs use `--report 0` and exit 0. Their unchanged summary
gates use `--expect-iterations 1473123` and exit 0. Counts are missing / excess /
false-positive iterations:

| Lane | Seeding | Gate | Missing / excess / FP |
| --- | --- | --- | ---: |
| Recorded APU, continuous | No `--seed-every`, including no post-polygon resync | PASS | 3,838 / 0 / 0 |
| Recorded APU census | `--seed-every 2000` | PASS | 3,838 / 0 / 0 |
| Live APU census | `--seed-every 2000 --live-apu-reads` | PASS | 4,535 / 590 / 132 |

The continuous run reports **1,473,123 clean WRAM iterations**, no first
WRAM divergence, and clean RNG/APU consumption. It reaches module `$19` at
**1,452,678**, enters `$1A` at **1,456,869**, and finishes at **1,473,122**,
`$1A/$22`, with **20,445 ending iterations**. Recorded holds remain **98,839
versus 102,677 source lag NMIs**, with **5,152 masked callbacks**. ROM census
counts match the supplied main control; live counts match this batch's fresh
`full` control. Holds and masked callbacks match record for record between
the old/new live censuses and between continuous/seeded recorded APU runs.
No timing deficit was fitted away. These are CPU logical-state/count gates;
continuous live-APU acceptance and video/audio parity are not claimed.

The **default 200k comparison matches every record and EOF**, and its baseline
SHA-256 matches the original metadata. Ordinary default, reduced and recorder
builds are warning-free. Asset-backed library results are default **2,689
passed / 1 known spotlight failure / 3 ignored**, reduced **2,224 passed /
0 failed / 2 ignored**; recorder tests are **2/2**. Existing lib-test warnings
are separate from ordinary builds. `git diff --check` passes.

All **1,502 original `full` files retain size, nanosecond mtime and SHA-256**,
with no files added or removed. `full-v2` retains its recorded hashes after
the replays. Both integrity reports, before/after manifests,
`ledger-comparison.json`, `ledger-equivalence-verdict.json`, raw predictors,
gate logs and command/binary provenance retain the proof below **3 GB**.
Documented replay paths switch to `full-v2` only after all checks passed.
No GPU run, `--video`, cached-av, cargo GPU test, oracle-cache write, cargo fmt,
commit, push or one-off repository script was used.
