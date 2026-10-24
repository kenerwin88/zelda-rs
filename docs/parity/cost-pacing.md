# Lagless cost accounting

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
LEDGER=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full
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
