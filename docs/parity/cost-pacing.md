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
LEDGER=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full
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
  --ledger /Users/missingno/Documents/zelda3-rs/target/logical-ledger/full/iterations.jsonl \
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
